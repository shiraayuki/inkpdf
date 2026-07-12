use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use anyhow::{Context, Result};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

use crate::engine::document::Document;

/// Writes a document as gzipped JSON (`.inkpdf`).
pub fn save(doc: &Document, path: &Path) -> Result<()> {
    let json = serde_json::to_vec(doc).context("serializing document")?;
    let file = File::create(path).with_context(|| format!("creating {}", path.display()))?;
    let mut encoder = GzEncoder::new(BufWriter::new(file), Compression::default());
    encoder.write_all(&json).context("writing document")?;
    encoder.finish().context("finalizing document")?;
    Ok(())
}

pub fn load(path: &Path) -> Result<Document> {
    let file = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut decoder = GzDecoder::new(BufReader::new(file));
    let mut json = Vec::new();
    decoder.read_to_end(&mut json).context("decompressing document")?;
    serde_json::from_slice(&json).context("parsing document")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::document::*;
    use uuid::Uuid;

    fn sample() -> Document {
        Document {
            source: Some(PdfSource {
                name: "sample.pdf".into(),
                bytes: vec![0x25, 0x50, 0x44, 0x46, 0x2d, 1, 2, 3, 4, 5],
            }),
            pages: vec![
                Page {
                    kind: PageKind::Pdf { page_index: 0 },
                    width: 595.0,
                    height: 842.0,
                    annotations: vec![Annotation {
                        id: Uuid::new_v4(),
                        kind: AnnotationKind::Text(TextAnnotation {
                            x: 12.5,
                            y: 33.0,
                            size: 14.0,
                            runs: vec![TextRun {
                                text: "Hällo Ümlaut".into(),
                                color: Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
                            }],
                        }),
                    }],
                },
                Page {
                    kind: PageKind::Blank { color: Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 } },
                    width: 595.0,
                    height: 842.0,
                    annotations: vec![],
                },
            ],
        }
    }

    fn temp_path() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("inkpdf-test-{}.inkpdf", Uuid::new_v4()))
    }

    #[test]
    fn roundtrip_preserves_document() {
        let doc = sample();
        let path = temp_path();

        save(&doc, &path).unwrap();
        let loaded = load(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(doc, loaded);
    }

    #[test]
    fn load_rejects_garbage() {
        let path = temp_path();
        std::fs::write(&path, b"not gzip").unwrap();
        let result = load(&path);
        std::fs::remove_file(&path).ok();
        assert!(result.is_err());
    }
}
