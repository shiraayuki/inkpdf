//! Crash-recovery autosaves: periodic snapshots of unsaved documents in a
//! dedicated directory, plus a small JSON sidecar remembering where the
//! document really lives. The files are ordinary `.inkpdf` documents named by
//! a per-tab UUID; they exist only between an edit and the next manual save
//! (or a clean exit), so anything found at startup means a crash.

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::engine::document::Document;
use crate::engine::storage;

/// Sidecar data for one autosave: where the document should be saved on a
/// plain "save" after recovery, and the tab label to show.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AutosaveMeta {
    pub original_path: Option<PathBuf>,
    pub label: String,
}

/// One recoverable autosave found on disk.
pub struct Recovered {
    pub file: PathBuf,
    pub id: Uuid,
    pub meta: AutosaveMeta,
}

fn document_path(dir: &Path, id: Uuid) -> PathBuf {
    dir.join(format!("{id}.inkpdf"))
}

fn meta_path(dir: &Path, id: Uuid) -> PathBuf {
    dir.join(format!("{id}.meta.json"))
}

/// Writes one autosave snapshot atomically (temp file + rename), so a crash
/// mid-write can never corrupt the previous snapshot.
pub fn write(dir: &Path, id: Uuid, doc: &Document, meta: &AutosaveMeta) -> Result<()> {
    std::fs::create_dir_all(dir)?;

    let tmp = dir.join(format!("{id}.inkpdf.tmp"));
    storage::save(doc, &tmp)?;
    std::fs::rename(&tmp, document_path(dir, id))?;

    let tmp = dir.join(format!("{id}.meta.tmp"));
    std::fs::write(&tmp, serde_json::to_string(meta)?)?;
    std::fs::rename(&tmp, meta_path(dir, id))?;
    Ok(())
}

/// Deletes the autosave (and sidecar) for `id`, if present.
pub fn remove(dir: &Path, id: Uuid) {
    let _ = std::fs::remove_file(document_path(dir, id));
    let _ = std::fs::remove_file(meta_path(dir, id));
}

/// Lists every recoverable autosave in `dir`. A missing/broken sidecar
/// degrades to a default meta (no original path) instead of dropping the
/// snapshot - the document data matters more than its label.
pub fn scan(dir: &Path) -> Vec<Recovered> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<Recovered> = entries
        .flatten()
        .filter_map(|entry| {
            let file = entry.path();
            if !file.extension().is_some_and(|e| e.eq_ignore_ascii_case("inkpdf")) {
                return None;
            }
            let id: Uuid = file.file_stem()?.to_str()?.parse().ok()?;
            let meta = std::fs::read_to_string(meta_path(dir, id))
                .ok()
                .and_then(|data| serde_json::from_str(&data).ok())
                .unwrap_or_default();
            Some(Recovered { file, id, meta })
        })
        .collect();
    found.sort_by_key(|r| r.id);
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::document::{A4, Color, DEFAULT_PATTERN_SPACING, PagePattern};

    fn test_dir() -> PathBuf {
        std::env::temp_dir().join(format!("inkpdf-autosave-test-{}", Uuid::new_v4()))
    }

    #[test]
    fn write_scan_remove_roundtrip() {
        let dir = test_dir();
        let mut doc = Document::new();
        doc.insert_blank_page(0, A4.0, A4.1, Color::WHITE, PagePattern::Grid, DEFAULT_PATTERN_SPACING);
        let id = Uuid::new_v4();
        let meta = AutosaveMeta {
            original_path: Some(PathBuf::from("/tmp/foo.inkpdf")),
            label: "foo.inkpdf".into(),
        };

        write(&dir, id, &doc, &meta).unwrap();
        // Overwriting the same id must not duplicate entries.
        write(&dir, id, &doc, &meta).unwrap();

        let found = scan(&dir);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, id);
        assert_eq!(found[0].meta.original_path.as_deref(), Some(Path::new("/tmp/foo.inkpdf")));
        assert_eq!(storage::load(&found[0].file).unwrap(), doc);

        remove(&dir, id);
        assert!(scan(&dir).is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn scan_survives_missing_meta_and_foreign_files() {
        let dir = test_dir();
        let doc = Document::new();
        let id = Uuid::new_v4();
        write(&dir, id, &doc, &AutosaveMeta::default()).unwrap();
        std::fs::remove_file(dir.join(format!("{id}.meta.json"))).unwrap();
        std::fs::write(dir.join("not-a-uuid.inkpdf"), b"junk").unwrap();
        std::fs::write(dir.join("random.txt"), b"junk").unwrap();

        let found = scan(&dir);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, id);
        assert!(found[0].meta.original_path.is_none());
        std::fs::remove_dir_all(&dir).ok();
    }
}
