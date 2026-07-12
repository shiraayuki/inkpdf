use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const FILE_EXTENSION: &str = "inkpdf";

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
}

/// Default blank-page size in PDF points (A4), used when no page exists to match.
pub const A4: (f64, f64) = (595.0, 842.0);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum PageKind {
    Pdf { page_index: usize },
    Blank { color: Color },
}

/// A run of text sharing one color.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TextRun {
    pub text: String,
    pub color: Color,
}

/// A text box: `runs` hold colored spans (so different passages can be colored).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TextAnnotation {
    pub x: f64,
    pub y: f64,
    pub size: f64,
    pub runs: Vec<TextRun>,
}

impl TextAnnotation {
    /// The characters with their colors, flattened across runs.
    pub fn glyphs(&self) -> Vec<(char, Color)> {
        self.runs
            .iter()
            .flat_map(|run| run.text.chars().map(|ch| (ch, run.color)))
            .collect()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum AnnotationKind {
    Text(TextAnnotation),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Annotation {
    pub id: Uuid,
    pub kind: AnnotationKind,
}

/// A page in the document. Coordinates/sizes are in PDF points.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Page {
    pub kind: PageKind,
    pub width: f64,
    pub height: f64,
    pub annotations: Vec<Annotation>,
}

/// The imported PDF, embedded so a saved document stays self-contained.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PdfSource {
    pub name: String,
    pub bytes: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Document {
    pub source: Option<PdfSource>,
    pub pages: Vec<Page>,
}

impl Document {
    pub fn new() -> Self {
        Self { source: None, pages: Vec::new() }
    }

    pub fn insert_blank_page(&mut self, at: usize, width: f64, height: f64, color: Color) {
        let page = Page {
            kind: PageKind::Blank { color },
            width,
            height,
            annotations: Vec::new(),
        };
        self.pages.insert(at.min(self.pages.len()), page);
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_blank_page_adds_page_at_index() {
        let mut doc = Document::new();
        doc.insert_blank_page(0, 595.0, 842.0, Color::WHITE);
        doc.insert_blank_page(1, 200.0, 300.0, Color::WHITE);

        assert_eq!(doc.pages.len(), 2);
        assert_eq!(doc.pages[1].width, 200.0);
        assert!(matches!(doc.pages[1].kind, PageKind::Blank { .. }));
        // Out-of-range index is clamped to the end.
        doc.insert_blank_page(999, 100.0, 100.0, Color::WHITE);
        assert_eq!(doc.pages.len(), 3);
        assert_eq!(doc.pages[2].width, 100.0);
    }
}

