//! Core logic, kept free of GTK dependencies so it stays testable in isolation.

pub mod document;
pub mod pdf;
pub mod storage;

use std::path::Path;

use anyhow::Result;

use document::{Document, Page, PageKind, PdfSource};
use pdf::PdfDocument;

/// A loaded document plus the runtime poppler handle needed to render its PDF pages.
/// The `model` is what gets serialized; `pdf` is rebuilt from the embedded bytes.
pub struct OpenDocument {
    pub model: Document,
    pub pdf: Option<PdfDocument>,
}

impl OpenDocument {
    pub fn from_pdf_path(path: &Path) -> Result<Self> {
        let data = std::fs::read(path)
            .map_err(|e| anyhow::anyhow!("could not read PDF: {e}"))?;
        let name = file_name(path);
        Self::from_pdf_bytes(name, data)
    }

    pub fn from_pdf_bytes(name: String, data: Vec<u8>) -> Result<Self> {
        let pdf = PdfDocument::from_bytes(data.clone())?;
        let pages = (0..pdf.n_pages())
            .map(|i| {
                let (w, h) = pdf.page_size(i);
                Page {
                    kind: PageKind::Pdf { page_index: i },
                    width: w,
                    height: h,
                    annotations: Vec::new(),
                }
            })
            .collect();

        let model = Document { source: Some(PdfSource { name, bytes: data }), pages };
        Ok(Self { model, pdf: Some(pdf) })
    }

    pub fn from_inkpdf_path(path: &Path) -> Result<Self> {
        let model = storage::load(path)?;
        let pdf = match &model.source {
            Some(src) => Some(PdfDocument::from_bytes(src.bytes.clone())?),
            None => None,
        };
        Ok(Self { model, pdf })
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdf_to_inkpdf_roundtrip_rebuilds_renderer() {
        let Ok(path) = std::env::var("INKPDF_TEST_PDF") else {
            eprintln!("INKPDF_TEST_PDF not set - skipping");
            return;
        };

        let opened = OpenDocument::from_pdf_path(Path::new(&path)).unwrap();
        assert!(!opened.model.pages.is_empty());
        assert!(opened.model.source.is_some());

        let out = std::env::temp_dir().join(format!("inkpdf-it-{}.inkpdf", uuid::Uuid::new_v4()));
        storage::save(&opened.model, &out).unwrap();

        let reopened = OpenDocument::from_inkpdf_path(&out).unwrap();
        std::fs::remove_file(&out).ok();

        assert_eq!(opened.model, reopened.model);
        // The poppler handle is rebuilt from the embedded bytes, not the original file.
        let pdf = reopened.pdf.expect("renderer rebuilt from embedded bytes");
        assert_eq!(pdf.n_pages(), opened.model.pages.len());
    }
}
