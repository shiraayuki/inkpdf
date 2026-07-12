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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum PageKind {
    Pdf { page_index: usize },
    Blank { color: Color },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TextAnnotation {
    pub x: f64,
    pub y: f64,
    pub content: String,
    pub size: f64,
    pub color: Color,
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

