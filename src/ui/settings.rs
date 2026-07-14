//! Persists tool defaults (pen/shape/eraser/text, blank-page pattern, theme)
//! across restarts as a small JSON file under the user's config dir.

use std::path::PathBuf;

use anyhow::Result;
use gtk::glib;
use serde::{Deserialize, Serialize};

use crate::engine::document::{Color, DEFAULT_PATTERN_SPACING, PagePattern, ShapeKind, TextStyle};

const FILE_NAME: &str = "settings.json";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct AppSettings {
    pub dark_mode: bool,
    pub pen_color: Color,
    pub pen_width: f64,
    pub marker_color: Color,
    pub marker_width: f64,
    pub shape_kind: ShapeKind,
    pub shape_color: Color,
    pub shape_width: f64,
    pub eraser_width: f64,
    pub text_size: f64,
    pub text_color: Color,
    pub text_font: String,
    pub blank_pattern: PagePattern,
    pub blank_pattern_spacing: f64,
    /// Minutes between crash-recovery autosaves; 0 disables them.
    pub autosave_minutes: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        let text_style = TextStyle::default();
        Self {
            dark_mode: true,
            pen_color: Color::BLACK,
            pen_width: 3.0,
            marker_color: Color { r: 1.0, g: 0.85, b: 0.2, a: 0.35 },
            marker_width: 12.0,
            shape_kind: ShapeKind::Rectangle,
            shape_color: Color::BLACK,
            shape_width: 3.0,
            eraser_width: 10.0,
            text_size: 16.0,
            text_color: text_style.color,
            text_font: text_style.font,
            blank_pattern: PagePattern::Plain,
            blank_pattern_spacing: DEFAULT_PATTERN_SPACING,
            autosave_minutes: 2,
        }
    }
}

fn path() -> PathBuf {
    glib::user_config_dir().join("inkpdf").join(FILE_NAME)
}

/// Loads settings from disk, falling back to defaults if the file is missing
/// or unreadable (never surfaces an error to the user for this).
pub fn load() -> AppSettings {
    load_from(&path())
}

pub fn save(settings: &AppSettings) -> Result<()> {
    save_to(&path(), settings)
}

fn load_from(path: &std::path::Path) -> AppSettings {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default()
}

fn save_to(path: &std::path::Path, settings: &AppSettings) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(settings)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_preserves_settings() {
        let path = std::env::temp_dir().join(format!("inkpdf-settings-test-{}.json", uuid::Uuid::new_v4()));

        let mut settings = AppSettings::default();
        settings.dark_mode = false;
        settings.pen_width = 7.5;
        settings.blank_pattern = PagePattern::Grid;

        save_to(&path, &settings).unwrap();
        let loaded = load_from(&path);
        std::fs::remove_file(&path).ok();

        assert_eq!(settings, loaded);
    }

    #[test]
    fn load_from_missing_file_returns_defaults() {
        let path = std::env::temp_dir().join(format!("inkpdf-settings-missing-{}.json", uuid::Uuid::new_v4()));
        assert_eq!(load_from(&path), AppSettings::default());
    }
}
