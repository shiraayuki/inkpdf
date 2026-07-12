use std::path::Path;

use anyhow::{Context, Result};
use gtk::cairo;
use gtk::glib;
use poppler::Document;

/// An opened PDF with its page sizes (in PDF points) cached up front.
pub struct PdfDocument {
    doc: Document,
    page_sizes: Vec<(f64, f64)>,
}

impl PdfDocument {
    pub fn open(path: &Path) -> Result<Self> {
        let data = std::fs::read(path)
            .with_context(|| format!("could not read PDF: {}", path.display()))?;

        let bytes = glib::Bytes::from_owned(data);
        let doc = Document::from_bytes(&bytes, None)
            .map_err(|e| anyhow::anyhow!("could not open PDF: {e}"))?;

        let n = doc.n_pages().max(0) as usize;
        let mut page_sizes = Vec::with_capacity(n);
        for i in 0..n {
            let size = doc
                .page(i as i32)
                .map(|p| p.size())
                .unwrap_or((595.0, 842.0));
            page_sizes.push(size);
        }

        Ok(Self { doc, page_sizes })
    }

    pub fn n_pages(&self) -> usize {
        self.page_sizes.len()
    }

    pub fn page_size(&self, index: usize) -> (f64, f64) {
        self.page_sizes[index]
    }

    pub fn page_sizes(&self) -> &[(f64, f64)] {
        &self.page_sizes
    }

    /// Renders a page onto `ctx`; the caller sets the zoom via the context's transform.
    pub fn render_page(&self, index: usize, ctx: &cairo::Context) {
        if let Some(page) = self.doc.page(index as i32) {
            page.render(ctx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_page_with_content() {
        let Ok(path) = std::env::var("INKPDF_TEST_PDF") else {
            eprintln!("INKPDF_TEST_PDF not set - skipping");
            return;
        };

        let pdf = PdfDocument::open(Path::new(&path)).expect("PDF should load");
        assert!(pdf.n_pages() > 0);

        let (w, h) = pdf.page_size(0);
        let (pw, ph) = (w.ceil() as i32, h.ceil() as i32);
        let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, pw, ph).unwrap();
        {
            let c = cairo::Context::new(&surface).unwrap();
            c.set_source_rgb(1.0, 1.0, 1.0);
            c.paint().unwrap();
            pdf.render_page(0, &c);
        }
        surface.flush();

        let data = surface.data().expect("surface data readable");
        let non_white = data.iter().filter(|&&b| b != 0xFF).count();
        assert!(non_white > 0, "rendered page is blank");
    }
}
