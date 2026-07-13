//! A small embedded file browser that slides in from the left: pick a
//! `.pdf`/`.inkpdf` file and it opens straight into a new tab, no dialog.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;

use crate::engine::document::FILE_EXTENSION;
use crate::ui::window::WindowUi;

const PANEL_WIDTH: i32 = 240;

pub struct FileBrowser {
    pub revealer: gtk::Revealer,
}

impl FileBrowser {
    pub fn new(ui: &WindowUi) -> Self {
        let dir = Rc::new(RefCell::new(initial_dir()));
        let entries: Rc<RefCell<Vec<(PathBuf, bool)>>> = Rc::new(RefCell::new(Vec::new()));

        let list = gtk::ListBox::new();
        list.add_css_class("navigation-sidebar");

        let scroller = gtk::ScrolledWindow::builder().child(&list).vexpand(true).build();

        let path_label = gtk::Label::builder().xalign(0.0).hexpand(true).build();
        path_label.add_css_class("caption");
        path_label.add_css_class("dim-label");
        path_label.set_ellipsize(gtk::pango::EllipsizeMode::Start);

        let up_button = icon_button("go-up-symbolic", "Ordner nach oben");
        let home_button = icon_button("go-home-symbolic", "Home-Verzeichnis");

        let header_row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        header_row.append(&up_button);
        header_row.append(&home_button);
        header_row.append(&path_label);

        let column = gtk::Box::new(gtk::Orientation::Vertical, 6);
        column.set_width_request(PANEL_WIDTH);
        column.set_margin_top(8);
        column.set_margin_bottom(8);
        column.set_margin_start(8);
        column.set_margin_end(8);
        column.append(&header_row);
        column.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        column.append(&scroller);

        let revealer = gtk::Revealer::builder()
            .transition_type(gtk::RevealerTransitionType::SlideRight)
            .reveal_child(false)
            .child(&column)
            .build();

        refresh(&list, &path_label, &entries, &dir);

        {
            let dir = dir.clone();
            let entries = entries.clone();
            let list = list.clone();
            let path_label = path_label.clone();
            up_button.connect_clicked(move |_| {
                let parent = dir.borrow().parent().map(|p| p.to_path_buf());
                if let Some(parent) = parent {
                    *dir.borrow_mut() = parent;
                    refresh(&list, &path_label, &entries, &dir);
                }
            });
        }
        {
            let dir = dir.clone();
            let entries = entries.clone();
            let list = list.clone();
            let path_label = path_label.clone();
            home_button.connect_clicked(move |_| {
                *dir.borrow_mut() = initial_dir();
                refresh(&list, &path_label, &entries, &dir);
            });
        }
        {
            let dir = dir.clone();
            let entries = entries.clone();
            let list = list.clone();
            let path_label = path_label.clone();
            let ui = ui.clone();
            list.connect_row_activated(move |list, row| {
                let Some((path, is_dir)) = entries.borrow().get(row.index() as usize).cloned() else {
                    return;
                };
                if is_dir {
                    *dir.borrow_mut() = path;
                    refresh(list, &path_label, &entries, &dir);
                } else {
                    ui.open_path_in_new_tab(&path);
                }
            });
        }

        Self { revealer }
    }
}

fn initial_dir() -> PathBuf {
    glib::home_dir()
}

fn icon_button(icon: &str, tip: &str) -> gtk::Button {
    gtk::Button::builder().icon_name(icon).tooltip_text(tip).css_classes(["flat"]).build()
}

/// Rebuilds the row list for the current directory, filtering files down to
/// `.pdf`/`.inkpdf` (folders are always shown, for navigation). Keeps
/// `entries` in sync (same order as the rows) so `row-activated` can look up
/// what was clicked by index.
fn refresh(
    list: &gtk::ListBox,
    path_label: &gtk::Label,
    entries: &Rc<RefCell<Vec<(PathBuf, bool)>>>,
    dir: &Rc<RefCell<PathBuf>>,
) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    let current = dir.borrow().clone();
    path_label.set_text(&current.display().to_string());

    let found = list_dir_entries(&current);

    for (path, is_dir) in &found {
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        row.set_margin_top(4);
        row.set_margin_bottom(4);
        row.set_margin_start(8);
        row.set_margin_end(8);
        row.append(&gtk::Image::from_icon_name(if *is_dir { "folder-symbolic" } else { "x-office-document-symbolic" }));
        let label = gtk::Label::new(Some(&name));
        label.set_xalign(0.0);
        label.set_hexpand(true);
        label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        row.append(&label);
        list.append(&row);
    }

    *entries.borrow_mut() = found;
}

/// Lists `dir`'s entries for the browser: directories first, then
/// `.pdf`/`.inkpdf` files (other files are hidden), both alphabetical.
fn list_dir_entries(dir: &std::path::Path) -> Vec<(PathBuf, bool)> {
    let mut found: Vec<(PathBuf, bool)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                return Some((path, true));
            }
            let is_doc = path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf") || ext.eq_ignore_ascii_case(FILE_EXTENSION));
            is_doc.then_some((path, false))
        })
        .collect();
    found.sort_by(|(a, a_dir), (b, b_dir)| b_dir.cmp(a_dir).then_with(|| a.file_name().cmp(&b.file_name())));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_dir_entries_shows_dirs_and_pdf_inkpdf_only() {
        let root = std::env::temp_dir().join(format!("inkpdf-browser-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("zsubdir")).unwrap();
        std::fs::create_dir_all(root.join("asubdir")).unwrap();
        std::fs::write(root.join("notes.pdf"), b"").unwrap();
        std::fs::write(root.join("doc.inkpdf"), b"").unwrap();
        std::fs::write(root.join("ignore.txt"), b"").unwrap();

        let found = list_dir_entries(&root);
        let names: Vec<String> =
            found.iter().map(|(p, is_dir)| format!("{}{}", p.file_name().unwrap().to_string_lossy(), if *is_dir { "/" } else { "" })).collect();

        std::fs::remove_dir_all(&root).ok();

        // Dirs first (alphabetical), then only the pdf/inkpdf files (alphabetical).
        assert_eq!(names, vec!["asubdir/", "zsubdir/", "doc.inkpdf", "notes.pdf"]);
    }
}
