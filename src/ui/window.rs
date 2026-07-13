use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gdk, gio};

use crate::engine::OpenDocument;
use crate::engine::document::{A4, Color, Document, FILE_EXTENSION, PagePattern, ShapeKind};
use crate::engine::pdf::PdfDocument;
use crate::engine::storage;
use crate::ui::canvas::{Canvas, Relative, Tool};

const DEFAULT_WIDTH: i32 = 900;
const DEFAULT_HEIGHT: i32 = 700;
/// Side of the square color swatch in the details panel.
const SWATCH: i32 = 30;
/// Points added/removed per click on the page width/height steppers.
const PAGE_SIZE_STEP: f64 = 10.0;

/// One open document. Only the active tab's document/pdf actually live in the
/// `Canvas` at any time; `switch_to_tab` moves them in and out of here.
struct Tab {
    model: Document,
    pdf: Option<PdfDocument>,
    /// The `.inkpdf` file this tab saves to on plain "Save" (None until it has
    /// one: a fresh doc, or a PDF that has not yet been saved as `.inkpdf`).
    save_path: Option<PathBuf>,
    /// Snapshot of the document as it was last loaded/saved, to detect unsaved
    /// changes before an action that would discard them.
    saved_snapshot: Option<Document>,
    zoom: f64,
    label: String,
}

impl Tab {
    fn blank() -> Self {
        let mut model = Document::new();
        model.insert_blank_page(0, A4.0, A4.1, Color::WHITE, PagePattern::Plain);
        Tab {
            saved_snapshot: Some(model.clone()),
            model,
            pdf: None,
            save_path: None,
            zoom: 1.0,
            label: "Unbenannt".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct WindowUi {
    window: adw::ApplicationWindow,
    canvas: Canvas,
    title: adw::WindowTitle,
    tab_bar: gtk::Box,
    tabs: Rc<RefCell<Vec<Tab>>>,
    active_tab: Rc<Cell<usize>>,
}

impl WindowUi {
    /// Loads a `.pdf` or `.inkpdf` file into the active tab, dispatched by extension.
    pub fn load_path(&self, path: &Path) {
        let is_inkpdf = path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case(FILE_EXTENSION));

        let opened = if is_inkpdf {
            OpenDocument::from_inkpdf_path(path)
        } else {
            OpenDocument::from_pdf_path(path)
        };

        match opened {
            Ok(open) => {
                let save_path = is_inkpdf.then(|| path.to_path_buf());
                self.with_active_tab(|t| {
                    t.saved_snapshot = Some(open.model.clone());
                    t.save_path = save_path;
                });
                self.canvas.set_open_document(open);
                self.show_title(Some(path));
            }
            Err(err) => show_error(&self.window, &format!("{err:#}")),
        }
    }

    fn with_active_tab<R>(&self, f: impl FnOnce(&mut Tab) -> R) -> R {
        let mut tabs = self.tabs.borrow_mut();
        f(&mut tabs[self.active_tab.get()])
    }

    fn save_path(&self) -> Option<PathBuf> {
        self.tabs.borrow()[self.active_tab.get()].save_path.clone()
    }

    /// Whether the active tab's document differs from its last loaded/saved snapshot.
    fn is_dirty(&self) -> bool {
        let saved = self.tabs.borrow()[self.active_tab.get()].saved_snapshot.clone();
        match (self.canvas.document(), saved) {
            (Some(current), Some(saved)) => current != saved,
            (Some(_), None) => true,
            (None, _) => false,
        }
    }

    /// Sets the header title to the file name and the subtitle to its full path
    /// (or "Unbenannt" for a document with no file yet); keeps the active tab's
    /// label and the tab bar in sync.
    fn show_title(&self, path: Option<&Path>) {
        let label = match path {
            Some(p) => file_label(p),
            None => "Unbenannt".to_string(),
        };
        self.title.set_title(&label);
        self.title.set_subtitle(path.map(|p| p.display().to_string()).unwrap_or_default().as_str());
        self.with_active_tab(|t| t.label = label);
        self.rebuild_tab_bar();
    }

    /// Plain save: write to the known file if there is one, else fall back to "save as".
    fn save(&self) {
        self.save_then(|_| {});
    }

    /// Like `save`, but calls `and_then` once the save has actually completed
    /// (immediately for a known path, or after the async "save as" dialog).
    fn save_then(&self, and_then: impl Fn(&WindowUi) + 'static) {
        let path = self.save_path();
        match path {
            Some(path) => {
                let Some(model) = self.canvas.document() else {
                    return;
                };
                match storage::save(&model, &path) {
                    Ok(()) => {
                        self.with_active_tab(|t| t.saved_snapshot = Some(model));
                        and_then(self);
                    }
                    Err(err) => show_error(&self.window, &format!("{err:#}")),
                }
            }
            None => save_dialog_then(self, and_then),
        }
    }

    fn insert_page(&self, rel: Relative) {
        self.canvas.insert_blank_page(rel);
    }

    /// Creates a fresh blank tab, stashes the previously active tab's live canvas
    /// state, and makes the new tab active.
    fn new_tab(&self) {
        let idx = {
            let mut tabs = self.tabs.borrow_mut();
            tabs.push(Tab::blank());
            tabs.len() - 1
        };
        self.switch_to_tab(idx);
    }

    /// Switches the canvas to a different tab, stashing the current tab's live
    /// state (document, pdf handle, zoom) back into its slot first.
    fn switch_to_tab(&self, new_idx: usize) {
        let old_idx = self.active_tab.get();
        if new_idx == old_idx {
            return;
        }

        if let Some(open) = self.canvas.take_open_document() {
            let zoom = self.canvas.zoom();
            let mut tabs = self.tabs.borrow_mut();
            tabs[old_idx].model = open.model;
            tabs[old_idx].pdf = open.pdf;
            tabs[old_idx].zoom = zoom;
        }

        let (model, pdf, zoom, path) = {
            let mut tabs = self.tabs.borrow_mut();
            let tab = &mut tabs[new_idx];
            (tab.model.clone(), tab.pdf.take(), tab.zoom, tab.save_path.clone())
        };
        self.active_tab.set(new_idx);
        self.canvas.set_open_document_with_zoom(OpenDocument { model, pdf }, zoom);
        self.show_title(path.as_deref());
    }

    /// Closes a tab, asking for confirmation first if it has unsaved changes.
    /// No-op if it's the only remaining tab.
    fn close_tab(&self, idx: usize) {
        if self.tabs.borrow().len() <= 1 {
            return;
        }

        let dirty = if idx == self.active_tab.get() {
            self.is_dirty()
        } else {
            let tabs = self.tabs.borrow();
            match &tabs[idx].saved_snapshot {
                Some(saved) => tabs[idx].model != *saved,
                None => true,
            }
        };

        if !dirty {
            self.close_tab_now(idx);
            return;
        }

        let dialog = gtk::AlertDialog::builder()
            .message("Ungespeicherte Änderungen")
            .detail("Dieser Tab hat ungespeicherte Änderungen. Trotzdem schließen?")
            .buttons(["Abbrechen", "Schließen"])
            .cancel_button(0)
            .default_button(0)
            .modal(true)
            .build();
        let ui = self.clone();
        dialog.choose(Some(&self.window), gio::Cancellable::NONE, move |response| {
            if let Ok(1) = response {
                ui.close_tab_now(idx);
            }
        });
    }

    fn close_tab_now(&self, idx: usize) {
        let active = self.active_tab.get();
        self.tabs.borrow_mut().remove(idx);
        let count = self.tabs.borrow().len();

        if active == idx {
            let new_active = idx.min(count - 1);
            self.active_tab.set(new_active);
            let (model, pdf, zoom, path) = {
                let mut tabs = self.tabs.borrow_mut();
                let tab = &mut tabs[new_active];
                (tab.model.clone(), tab.pdf.take(), tab.zoom, tab.save_path.clone())
            };
            self.canvas.set_open_document_with_zoom(OpenDocument { model, pdf }, zoom);
            self.show_title(path.as_deref());
        } else {
            if active > idx {
                self.active_tab.set(active - 1);
            }
            self.rebuild_tab_bar();
        }
    }

    /// Rebuilds the tab strip under the header from the current tab list.
    fn rebuild_tab_bar(&self) {
        while let Some(child) = self.tab_bar.first_child() {
            self.tab_bar.remove(&child);
        }
        let tabs = self.tabs.borrow();
        // A single tab needs no bar at all.
        self.tab_bar.set_visible(tabs.len() > 1);
        if tabs.len() <= 1 {
            return;
        }
        let active = self.active_tab.get();

        for (i, tab) in tabs.iter().enumerate() {
            let chip = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            chip.add_css_class("inkpdf-tab");
            chip.set_hexpand(true);

            let label = gtk::Button::builder().label(tab.label.as_str()).css_classes(["flat"]).build();
            label.add_css_class("inkpdf-tab-label");
            label.set_hexpand(true);
            if i == active {
                label.add_css_class("active");
            }
            {
                let ui = self.clone();
                label.connect_clicked(move |_| ui.switch_to_tab(i));
            }
            chip.append(&label);

            let close = flat_icon_button("window-close-symbolic", "Tab schließen");
            {
                let ui = self.clone();
                close.connect_clicked(move |_| ui.close_tab(i));
            }
            chip.append(&close);

            self.tab_bar.append(&chip);
        }
    }
}

pub fn build(app: &adw::Application) -> WindowUi {
    let title = adw::WindowTitle::new("inkpdf", "");

    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&title));

    let open_button = gtk::Button::builder()
        .icon_name("document-open-symbolic")
        .tooltip_text("Open")
        .css_classes(["flat"])
        .build();
    let save_button = gtk::Button::builder()
        .icon_name("document-save-symbolic")
        .tooltip_text("Save as inkpdf")
        .css_classes(["flat"])
        .build();
    let new_tab_button = gtk::Button::builder()
        .icon_name("tab-new-symbolic")
        .tooltip_text("Neuer Tab")
        .css_classes(["flat"])
        .build();
    header.pack_start(&open_button);
    header.pack_start(&save_button);
    header.pack_start(&new_tab_button);

    // Dark/light toggle (default dark = not active).
    let theme_button = gtk::ToggleButton::builder()
        .icon_name("weather-clear-night-symbolic")
        .tooltip_text("Hell/Dunkel umschalten")
        .css_classes(["flat"])
        .build();
    theme_button.connect_toggled(|btn| {
        let manager = adw::StyleManager::default();
        if btn.is_active() {
            manager.set_color_scheme(adw::ColorScheme::ForceLight);
            btn.set_icon_name("weather-clear-symbolic");
        } else {
            manager.set_color_scheme(adw::ColorScheme::ForceDark);
            btn.set_icon_name("weather-clear-night-symbolic");
        }
    });

    let canvas = Canvas::new();

    // Settings menu (behind the gear): zoom, page size, and the theme toggle.
    let (zoom_row, zoom_label, zoom_minus, zoom_plus) = value_stepper();
    let (width_row, width_label, width_minus, width_plus) = value_stepper();
    let (height_row, height_label, height_minus, height_plus) = value_stepper();
    let reset_button = gtk::Button::builder()
        .label("Auf PDF-Größe zurücksetzen")
        .css_classes(["flat"])
        .build();

    // Page-size controls act on blank pages only; grouped so they can be disabled
    // together on a rendered PDF page.
    let size_section = gtk::Box::new(gtk::Orientation::Vertical, 6);
    size_section.append(&caption("Breite"));
    size_section.append(&width_row);
    size_section.append(&caption("Höhe"));
    size_section.append(&height_row);
    size_section.append(&reset_button);

    let settings_menu = gtk::Box::new(gtk::Orientation::Vertical, 8);
    settings_menu.set_margin_top(10);
    settings_menu.set_margin_bottom(10);
    settings_menu.set_margin_start(10);
    settings_menu.set_margin_end(10);
    settings_menu.append(&caption("Zoom"));
    settings_menu.append(&zoom_row);
    settings_menu.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    settings_menu.append(&size_section);
    settings_menu.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    settings_menu.append(&theme_button);
    let settings_popover = gtk::Popover::builder().child(&settings_menu).build();
    let settings_button = gtk::MenuButton::builder()
        .icon_name("inkpdf-settings-symbolic")
        .tooltip_text("Settings")
        .css_classes(["flat"])
        .popover(&settings_popover)
        .build();
    header.pack_end(&settings_button);

    // Keeps the labels in sync with the current zoom/page and disables page-size
    // controls on non-blank pages. Called when the popover opens and after each step.
    let refresh_settings: Rc<dyn Fn()> = {
        let canvas = canvas.clone();
        let zoom_label = zoom_label.clone();
        let width_label = width_label.clone();
        let height_label = height_label.clone();
        let size_section = size_section.clone();
        Rc::new(move || {
            zoom_label.set_text(&format!("{} %", (canvas.zoom() * 100.0).round() as i32));
            if let Some((w, h)) = canvas.current_page_size() {
                width_label.set_text(&format!("{} pt", w.round() as i32));
                height_label.set_text(&format!("{} pt", h.round() as i32));
            }
            size_section.set_sensitive(canvas.current_page_is_blank());
        })
    };

    let step_zoom = |zoom_in: bool| {
        let canvas = canvas.clone();
        let refresh = refresh_settings.clone();
        move |_: &gtk::Button| {
            if zoom_in {
                canvas.zoom_in();
            } else {
                canvas.zoom_out();
            }
            refresh();
        }
    };
    zoom_minus.connect_clicked(step_zoom(false));
    zoom_plus.connect_clicked(step_zoom(true));

    let step_size = |dw: f64, dh: f64| {
        let canvas = canvas.clone();
        let refresh = refresh_settings.clone();
        move |_: &gtk::Button| {
            if let Some((w, h)) = canvas.current_page_size() {
                canvas.resize_current_page(w + dw, h + dh);
                refresh();
            }
        }
    };
    width_minus.connect_clicked(step_size(-PAGE_SIZE_STEP, 0.0));
    width_plus.connect_clicked(step_size(PAGE_SIZE_STEP, 0.0));
    height_minus.connect_clicked(step_size(0.0, -PAGE_SIZE_STEP));
    height_plus.connect_clicked(step_size(0.0, PAGE_SIZE_STEP));
    {
        let canvas = canvas.clone();
        let refresh = refresh_settings.clone();
        reset_button.connect_clicked(move |_| {
            canvas.reset_current_page_size();
            refresh();
        });
    }
    {
        let refresh = refresh_settings.clone();
        settings_popover.connect_show(move |_| refresh());
    }

    load_css();

    // Floating Rnote-style panels overlaid on the canvas: tools on the right,
    // tool details on the left.
    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(&canvas.root));

    let (details, add_page_button, remove_page_button) = build_details_panel(&canvas);
    details.set_halign(gtk::Align::Start);
    details.set_valign(gtk::Align::Center);
    details.set_margin_start(16);
    details.set_visible(false); // shown only while a tool is active
    overlay.add_overlay(&details);

    let tool_strip = build_tool_strip(&canvas, &details);
    tool_strip.set_halign(gtk::Align::End);
    tool_strip.set_valign(gtk::Align::Center);
    tool_strip.set_margin_end(16);
    overlay.add_overlay(&tool_strip);

    // Give both side panels the same width (the wider one, i.e. the details
    // panel whose color swatch sets its minimum width).
    let panel_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
    panel_group.add_widget(&details);
    panel_group.add_widget(&tool_strip);
    // Keep the group alive for the app's lifetime (widgets don't own it).
    std::mem::forget(panel_group);

    let tab_bar = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    tab_bar.add_css_class("inkpdf-tab-bar");
    // Tabs share the full width equally (1 tab = 100%, 2 = 50/50, ...).
    tab_bar.set_homogeneous(true);
    tab_bar.set_hexpand(true);

    let content = adw::ToolbarView::new();
    content.add_top_bar(&header);
    content.add_top_bar(&tab_bar);
    content.set_content(Some(&overlay));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(DEFAULT_WIDTH)
        .default_height(DEFAULT_HEIGHT)
        .content(&content)
        .build();

    let ui = WindowUi {
        window: window.clone(),
        canvas: canvas.clone(),
        title,
        tab_bar,
        tabs: Rc::new(RefCell::new(vec![Tab::blank()])),
        active_tab: Rc::new(Cell::new(0)),
    };

    // Start on the first (blank) tab.
    let (model, pdf, zoom) = {
        let mut tabs = ui.tabs.borrow_mut();
        let tab = &mut tabs[0];
        (tab.model.clone(), tab.pdf.take(), tab.zoom)
    };
    canvas.set_open_document_with_zoom(OpenDocument { model, pdf }, zoom);
    ui.show_title(None);

    register_shortcuts(app, &window, &ui);

    {
        let ui = ui.clone();
        open_button.connect_clicked(move |_| open_entry(&ui));
    }
    {
        let ui = ui.clone();
        save_button.connect_clicked(move |_| ui.save());
    }
    {
        let ui = ui.clone();
        new_tab_button.connect_clicked(move |_| ui.new_tab());
    }
    {
        let ui = ui.clone();
        add_page_button.connect_clicked(move |_| ui.insert_page(Relative::After));
    }
    {
        let ui = ui.clone();
        remove_page_button.connect_clicked(move |_| ui.canvas.delete_current_page());
    }

    // Right-click menus for choosing before/after the current page.
    {
        let ui = ui.clone();
        let anchor = add_page_button.clone();
        add_secondary_click(&add_page_button, move || {
            let before = ui.clone();
            let after = ui.clone();
            show_menu(
                &anchor,
                vec![
                    ("Insert before current page", true, Box::new(move || before.insert_page(Relative::Before))),
                    ("Insert after current page", true, Box::new(move || after.insert_page(Relative::After))),
                ],
            );
        });
    }
    {
        let ui = ui.clone();
        let anchor = remove_page_button.clone();
        add_secondary_click(&remove_page_button, move || {
            let count = ui.canvas.page_count();
            let current = ui.canvas.current_index();
            let before_ok = count > 0 && current > 0;
            let after_ok = count > 0 && current + 1 < count;
            let before = ui.clone();
            let after = ui.clone();
            show_menu(
                &anchor,
                vec![
                    ("Delete page before current", before_ok, Box::new(move || before.canvas.delete_page(Relative::Before))),
                    ("Delete page after current", after_ok, Box::new(move || after.canvas.delete_page(Relative::After))),
                ],
            );
        });
    }

    window.present();
    ui
}

/// Wires Ctrl+O / Ctrl+S / Ctrl+Shift+S to open, save, and save-as.
fn register_shortcuts(app: &adw::Application, window: &adw::ApplicationWindow, ui: &WindowUi) {
    let actions: [(&str, &str, Box<dyn Fn(&WindowUi)>); 3] = [
        ("open", "<Control>o", Box::new(open_entry)),
        ("save", "<Control>s", Box::new(|ui: &WindowUi| ui.save())),
        ("save-as", "<Control><Shift>s", Box::new(save_dialog)),
    ];
    for (name, accel, handler) in actions {
        let action = gio::SimpleAction::new(name, None);
        let ui = ui.clone();
        action.connect_activate(move |_, _| handler(&ui));
        window.add_action(&action);
        app.set_accels_for_action(&format!("win.{name}"), &[accel]);
    }
}

/// Entry point for "Open": asks whether to replace the active tab's document or
/// open the new file in a fresh tab, then proceeds accordingly.
fn open_entry(ui: &WindowUi) {
    let dialog = gtk::AlertDialog::builder()
        .message("Neues Dokument öffnen")
        .detail("Soll das aktuelle Dokument ersetzt oder in einem neuen Tab geöffnet werden?")
        .buttons(["Abbrechen", "Neuer Tab", "Ersetzen"])
        .cancel_button(0)
        .default_button(2)
        .modal(true)
        .build();

    let ui = ui.clone();
    let parent = ui.window.clone();
    dialog.choose(Some(&parent), gio::Cancellable::NONE, move |response| match response {
        Ok(1) => {
            ui.new_tab();
            open_dialog(&ui);
        }
        Ok(2) => confirm_unsaved_then(&ui, open_dialog),
        _ => {}
    });
}

fn open_dialog(ui: &WindowUi) {
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("PDF or inkpdf"));
    filter.add_mime_type("application/pdf");
    filter.add_suffix("pdf");
    filter.add_suffix(FILE_EXTENSION);

    let filters = gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);

    let dialog = gtk::FileDialog::builder()
        .title("Open")
        .filters(&filters)
        .modal(true)
        .build();

    let ui = ui.clone();
    let parent = ui.window.clone();
    dialog.open(Some(&parent), gio::Cancellable::NONE, move |result| {
        let file = match result {
            Ok(file) => file,
            Err(_) => return,
        };
        match file.path() {
            Some(path) => ui.load_path(&path),
            None => show_error(&ui.window, "The file has no local path."),
        }
    });
}

fn save_dialog(ui: &WindowUi) {
    save_dialog_then(ui, |_| {});
}

/// Like `save_dialog`, but calls `and_then` once the file has actually been written.
fn save_dialog_then(ui: &WindowUi, and_then: impl Fn(&WindowUi) + 'static) {
    let Some(model) = ui.canvas.document() else {
        return;
    };

    // Default the file name to the opened document's name (with the .inkpdf ext).
    let title = ui.title.title();
    let stem = Path::new(title.as_str()).file_stem().map(|s| s.to_string_lossy().into_owned());
    let initial = match stem {
        Some(s) if !s.is_empty() && title != "Unbenannt" => format!("{s}.{FILE_EXTENSION}"),
        _ => format!("untitled.{FILE_EXTENSION}"),
    };

    let dialog = gtk::FileDialog::builder()
        .title("Save as inkpdf")
        .initial_name(initial)
        .modal(true)
        .build();

    let ui = ui.clone();
    let parent = ui.window.clone();
    dialog.save(Some(&parent), gio::Cancellable::NONE, move |result| {
        let file = match result {
            Ok(file) => file,
            Err(_) => return,
        };
        let Some(path) = file.path() else {
            show_error(&ui.window, "The file has no local path.");
            return;
        };
        let path = with_extension(path);

        match storage::save(&model, &path) {
            Ok(()) => {
                ui.with_active_tab(|t| {
                    t.save_path = Some(path.clone());
                    t.saved_snapshot = Some(model.clone());
                });
                ui.show_title(Some(&path));
                and_then(&ui);
            }
            Err(err) => show_error(&ui.window, &format!("{err:#}")),
        }
    });
}

/// If the current document has unsaved changes, asks the user whether to save,
/// discard, or cancel before proceeding; otherwise proceeds straight away.
fn confirm_unsaved_then(ui: &WindowUi, and_then: impl Fn(&WindowUi) + 'static) {
    if !ui.is_dirty() {
        and_then(ui);
        return;
    }

    let dialog = gtk::AlertDialog::builder()
        .message("Ungespeicherte Änderungen")
        .detail("Das aktuelle Dokument hat ungespeicherte Änderungen. Möchtest du sie speichern, bevor du fortfährst?")
        .buttons(["Abbrechen", "Verwerfen", "Speichern"])
        .cancel_button(0)
        .default_button(2)
        .modal(true)
        .build();

    let ui = ui.clone();
    let parent = ui.window.clone();
    dialog.choose(Some(&parent), gio::Cancellable::NONE, move |response| match response {
        Ok(1) => and_then(&ui),
        Ok(2) => ui.save_then(and_then),
        _ => {}
    });
}

/// Ensures the path ends in `.inkpdf`.
fn with_extension(mut path: PathBuf) -> PathBuf {
    if path
        .extension()
        .is_none_or(|e| !e.eq_ignore_ascii_case(FILE_EXTENSION))
    {
        path.set_extension(FILE_EXTENSION);
    }
    path
}

fn show_error(window: &adw::ApplicationWindow, message: &str) {
    let dialog = gtk::AlertDialog::builder()
        .message("Something went wrong")
        .detail(message)
        .modal(true)
        .build();
    dialog.show(Some(window));
}

fn file_label(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

/// Right-hand tool strip: exclusive tool toggles (all off = move/select mode),
/// then undo/redo. Selecting a tool switches the details panel to its page.
fn build_tool_strip(canvas: &Canvas, details: &gtk::Stack) -> gtk::Box {
    let strip = gtk::Box::new(gtk::Orientation::Vertical, 6);
    strip.add_css_class("inkpdf-panel");

    let tools: [(&str, &str, Tool, &str); 6] = [
        ("inkpdf-pen-symbolic", "Pen", Tool::Pen, "pen"),
        ("inkpdf-shapes-symbolic", "Shapes", Tool::Shape, "shapes"),
        ("inkpdf-text-symbolic", "Text", Tool::Text, "text"),
        ("inkpdf-eraser-symbolic", "Eraser", Tool::Eraser, "eraser"),
        ("inkpdf-markdown-symbolic", "Markdown text", Tool::Markdown, "markdown"),
        ("inkpdf-pages-symbolic", "Pages", Tool::Pages, "pages"),
    ];

    let buttons: Rc<Vec<gtk::ToggleButton>> = Rc::new(
        tools
            .iter()
            .map(|(icon, tip, _, _)| {
                let button = gtk::ToggleButton::builder().icon_name(*icon).tooltip_text(*tip).build();
                button.add_css_class("flat");
                button.add_css_class("circular");
                strip.append(&button);
                button
            })
            .collect(),
    );

    for (i, button) in buttons.iter().enumerate() {
        let canvas = canvas.clone();
        let all = buttons.clone();
        let details = details.clone();
        let tool = tools[i].2;
        let page = tools[i].3.to_string();
        button.connect_toggled(move |btn| {
            if btn.is_active() {
                for other in all.iter() {
                    if other != btn {
                        other.set_active(false);
                    }
                }
                canvas.set_tool(tool);
                details.set_visible_child_name(&page);
                details.set_visible(true);
            } else if all.iter().all(|b| !b.is_active()) {
                canvas.set_tool(Tool::Select);
                details.set_visible(false);
            }
        });
    }

    strip.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    let undo = flat_icon_button("inkpdf-undo-symbolic", "Rückgängig (Strg+Z)");
    {
        let canvas = canvas.clone();
        undo.connect_clicked(move |_| canvas.undo());
    }
    strip.append(&undo);
    let redo = flat_icon_button("inkpdf-redo-symbolic", "Wiederholen (Strg+Y)");
    {
        let canvas = canvas.clone();
        redo.connect_clicked(move |_| canvas.redo());
    }
    strip.append(&redo);

    strip
}

/// Left-hand details panel: a compact Rnote-style column of options per tool.
/// The stack itself is the styled card; it is hidden when no tool is active.
/// Returns the add/remove-page buttons too, since their click handlers can only
/// be wired once `WindowUi` exists (see `build()`).
fn build_details_panel(canvas: &Canvas) -> (gtk::Stack, gtk::Button, gtk::Button) {
    let stack = gtk::Stack::new();
    stack.add_css_class("inkpdf-panel");
    // Same width for every page (consistent panel), but height follows each page's elements.
    stack.set_hhomogeneous(true);
    stack.set_vhomogeneous(false);
    let (pages_page, add_page_button, remove_page_button) = page_pages(canvas);
    stack.add_named(&pages_page, Some("pages"));
    stack.add_named(&page_pen(canvas), Some("pen"));
    stack.add_named(&page_shapes(canvas), Some("shapes"));
    stack.add_named(&page_text(canvas), Some("text"));
    stack.add_named(&page_eraser(canvas), Some("eraser"));
    stack.add_named(&page_markdown(), Some("markdown"));
    stack.set_visible_child_name("pen");
    (stack, add_page_button, remove_page_button)
}

fn detail_column() -> gtk::Box {
    gtk::Box::new(gtk::Orientation::Vertical, 6)
}

fn flat_icon_button(icon: &str, tip: &str) -> gtk::Button {
    let button = gtk::Button::builder().icon_name(icon).tooltip_text(tip).build();
    button.add_css_class("flat");
    button.add_css_class("circular");
    button
}

/// A dim caption label for a settings section.
fn caption(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0);
    label.add_css_class("caption");
    label.add_css_class("dim-label");
    label
}

/// Horizontal stepper for the settings popover: `[-]  value  [+]`. Returns the row,
/// its centered value label, and the two buttons (wired by the caller).
fn value_stepper() -> (gtk::Box, gtk::Label, gtk::Button, gtk::Button) {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let minus = flat_icon_button("list-remove-symbolic", "Kleiner");
    let plus = flat_icon_button("list-add-symbolic", "Größer");
    let label = gtk::Label::new(None);
    label.set_hexpand(true);
    label.set_width_chars(7);
    row.append(&minus);
    row.append(&label);
    row.append(&plus);
    (row, label, minus, plus)
}

fn flat_toggle(icon: &str, tip: &str) -> gtk::ToggleButton {
    let button = gtk::ToggleButton::builder().icon_name(icon).tooltip_text(tip).build();
    button.add_css_class("flat");
    button.add_css_class("circular");
    button
}

fn color_button() -> gtk::ColorDialogButton {
    swatch_button(gdk::RGBA::new(0.0, 0.0, 0.0, 1.0)) // default black everywhere
}

/// A square color swatch button with the given initial color.
fn swatch_button(initial: gdk::RGBA) -> gtk::ColorDialogButton {
    let button = gtk::ColorDialogButton::new(Some(gtk::ColorDialog::new()));
    button.set_rgba(&initial);
    button.set_size_request(SWATCH, SWATCH);
    button.set_halign(gtk::Align::Center);
    button.set_valign(gtk::Align::Center);
    button
}

fn color_from_rgba(rgba: &gdk::RGBA) -> Color {
    Color {
        r: rgba.red() as f64,
        g: rgba.green() as f64,
        b: rgba.blue() as f64,
        a: rgba.alpha() as f64,
    }
}

fn hsep() -> gtk::Separator {
    gtk::Separator::new(gtk::Orientation::Horizontal)
}

fn fmt_size(value: f64, decimals: usize) -> String {
    if decimals == 0 {
        format!("{}", value.round() as i64)
    } else {
        format!("{value:.decimals$}")
    }
}

/// Uniform vertical size control: +, an editable field, − (all stacked).
/// Supports manual entry (float when `decimals > 0`). `on_change` is called with
/// every new value.
fn size_stepper(
    default: f64,
    min: f64,
    max: f64,
    step: f64,
    decimals: usize,
    on_change: impl Fn(f64) + 'static,
) -> gtk::Box {
    let column = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let value = Rc::new(Cell::new(default));
    let on_change = Rc::new(on_change);

    let plus = flat_icon_button("list-add-symbolic", "Größer");
    let minus = flat_icon_button("list-remove-symbolic", "Kleiner");
    let entry = gtk::Entry::builder()
        .width_chars(3)
        .max_width_chars(4)
        .xalign(0.5)
        .text(fmt_size(default, decimals))
        .build();

    // Parses/clamps the typed text, rewrites it, and reports the new value.
    let commit = {
        let value = value.clone();
        let on_change = on_change.clone();
        move |entry: &gtk::Entry| {
            let parsed = entry.text().trim().replace(',', ".").parse::<f64>().unwrap_or(value.get());
            let v = parsed.clamp(min, max);
            value.set(v);
            entry.set_text(&fmt_size(v, decimals));
            on_change(v);
        }
    };
    {
        let commit = commit.clone();
        entry.connect_activate(move |entry| commit(entry));
    }
    {
        let commit = commit.clone();
        let target = entry.clone();
        let focus = gtk::EventControllerFocus::new();
        focus.connect_leave(move |_| commit(&target));
        entry.add_controller(focus);
    }
    let step_by = {
        let value = value.clone();
        let entry = entry.clone();
        let on_change = on_change.clone();
        move |delta: f64| {
            let v = (value.get() + delta).clamp(min, max);
            value.set(v);
            entry.set_text(&fmt_size(v, decimals));
            on_change(v);
        }
    };
    {
        let step_by = step_by.clone();
        plus.connect_clicked(move |_| step_by(step));
    }
    {
        let step_by = step_by.clone();
        minus.connect_clicked(move |_| step_by(-step));
    }

    column.append(&plus);
    column.append(&entry);
    column.append(&minus);
    column
}

/// "Pages" tool page: insert/delete the current page. Left click acts on the
/// current page directly; right click opens a before/after choice (wired in
/// `build()`, once `WindowUi` exists).
/// Page patterns offered by the dropdown, in display order.
const PAGE_PATTERNS: [(&str, PagePattern); 4] = [
    ("Leer", PagePattern::Plain),
    ("Kariert", PagePattern::Grid),
    ("Gepunktet", PagePattern::Dotted),
    ("Liniert", PagePattern::Lined),
];

fn page_pages(canvas: &Canvas) -> (gtk::Box, gtk::Button, gtk::Button) {
    let page = detail_column();
    let add = flat_icon_button("inkpdf-page-add", "Insert page after current");
    let remove = flat_icon_button("inkpdf-page-remove", "Delete current page");
    page.append(&add);
    page.append(&remove);

    page.append(&hsep());
    let labels: [&str; 4] = std::array::from_fn(|i| PAGE_PATTERNS[i].0);
    let pattern_dropdown = gtk::DropDown::from_strings(&labels);
    let current = PAGE_PATTERNS.iter().position(|(_, p)| *p == canvas.blank_pattern()).unwrap_or(0);
    pattern_dropdown.set_selected(current as u32);
    {
        let canvas = canvas.clone();
        pattern_dropdown.connect_selected_notify(move |dd| {
            if let Some((_, pattern)) = PAGE_PATTERNS.get(dd.selected() as usize) {
                canvas.set_blank_pattern(*pattern);
            }
        });
    }
    page.append(&pattern_dropdown);

    (page, add, remove)
}

fn page_pen(canvas: &Canvas) -> gtk::Box {
    let page = detail_column();

    let color = color_button();
    {
        let canvas = canvas.clone();
        color.connect_rgba_notify(move |btn| canvas.set_pen_color(color_from_rgba(&btn.rgba())));
    }
    page.append(&color);

    {
        let canvas = canvas.clone();
        page.append(&size_stepper(3.0, 0.5, 20.0, 0.5, 1, move |v| canvas.set_pen_width(v)));
    }
    page
}

fn page_shapes(canvas: &Canvas) -> gtk::Box {
    let page = detail_column();

    let shapes: [(&str, &str, ShapeKind); 3] = [
        ("inkpdf-rect-symbolic", "Rechteck", ShapeKind::Rectangle),
        ("inkpdf-ellipse-symbolic", "Ellipse", ShapeKind::Ellipse),
        ("inkpdf-line-symbolic", "Linie", ShapeKind::Line),
    ];
    let mut group: Option<gtk::ToggleButton> = None;
    for (icon, tip, kind) in shapes {
        let toggle = flat_toggle(icon, tip);
        if let Some(first) = &group {
            toggle.set_group(Some(first));
        } else {
            toggle.set_active(true);
            group = Some(toggle.clone());
        }
        let canvas = canvas.clone();
        toggle.connect_toggled(move |btn| {
            if btn.is_active() {
                canvas.set_shape_kind(kind);
            }
        });
        page.append(&toggle);
    }

    page.append(&hsep());
    let color = color_button();
    {
        let canvas = canvas.clone();
        color.connect_rgba_notify(move |btn| canvas.set_shape_color(color_from_rgba(&btn.rgba())));
    }
    page.append(&color);

    {
        let canvas = canvas.clone();
        page.append(&size_stepper(3.0, 1.0, 20.0, 1.0, 0, move |v| canvas.set_shape_width(v)));
    }
    page
}

fn page_text(canvas: &Canvas) -> gtk::Box {
    let page = detail_column();

    // Font family: a compact icon button opening the system family picker. Only the
    // family is applied (size/style come from our own controls). A FontDialogButton
    // would show the full font name and blow up the panel width.
    let font = flat_icon_button("font-x-generic-symbolic", "Schriftart");
    {
        let canvas = canvas.clone();
        font.connect_clicked(move |btn| {
            let dialog = gtk::FontDialog::new();
            let parent = btn.root().and_downcast::<gtk::Window>();
            let canvas = canvas.clone();
            let initial: Option<&gtk::pango::FontFamily> = None;
            dialog.choose_family(parent.as_ref(), initial, gio::Cancellable::NONE, move |res| {
                if let Ok(family) = res {
                    canvas.set_text_font(family.name().to_string());
                }
            });
        });
    }
    page.append(&font);

    {
        let canvas = canvas.clone();
        page.append(&size_stepper(16.0, 8.0, 72.0, 1.0, 0, move |v| canvas.set_text_size(v)));
    }

    let color = color_button();
    {
        let canvas = canvas.clone();
        color.connect_rgba_notify(move |btn| canvas.set_text_color(color_from_rgba(&btn.rgba())));
    }
    page.append(&color);

    // Plain (momentary) buttons, not toggles: they act on the selection and must not
    // stick in a blue :checked state.
    page.append(&hsep());
    let styles: [(&str, &str, fn(&Canvas)); 4] = [
        ("format-text-bold-symbolic", "Fett", Canvas::toggle_bold),
        ("format-text-italic-symbolic", "Kursiv", Canvas::toggle_italic),
        ("format-text-underline-symbolic", "Unterstrichen", Canvas::toggle_underline),
        ("format-text-strikethrough-symbolic", "Durchgestrichen", Canvas::toggle_strikethrough),
    ];
    for (icon, tip, action) in styles {
        let button = flat_icon_button(icon, tip);
        let canvas = canvas.clone();
        button.connect_clicked(move |_| action(&canvas));
        page.append(&button);
    }

    // Marker (highlighter): the swatch picks the color, the apply button paints it
    // onto the selection, the clear button removes it.
    page.append(&hsep());
    let marker = swatch_button(gdk::RGBA::new(1.0, 0.9, 0.2, 0.4));
    page.append(&marker);
    let apply = flat_icon_button("object-select-symbolic", "Markieren");
    {
        let canvas = canvas.clone();
        let marker = marker.clone();
        apply.connect_clicked(move |_| canvas.set_highlight(color_from_rgba(&marker.rgba())));
    }
    page.append(&apply);
    let clear = flat_icon_button("edit-clear-symbolic", "Marker entfernen");
    {
        let canvas = canvas.clone();
        clear.connect_clicked(move |_| canvas.clear_highlight());
    }
    page.append(&clear);
    page
}

fn page_eraser(canvas: &Canvas) -> gtk::Box {
    let page = detail_column();
    let canvas = canvas.clone();
    page.append(&size_stepper(10.0, 1.0, 40.0, 0.5, 1, move |v| canvas.set_eraser_width(v)));
    page
}

fn page_markdown() -> gtk::Box {
    let page = detail_column();
    page.append(&size_stepper(16.0, 8.0, 72.0, 1.0, 0, |_| {}));
    page
}

// Theme-aware floating panels: the background and text follow the light/dark
// palette (via libadwaita named colors), so the symbolic icons — which paint in
// the inherited foreground color — recolor automatically with the theme.
const PANEL_CSS: &str = "\
.inkpdf-panel { \
  background-color: alpha(@window_bg_color, 0.92); \
  color: @window_fg_color; \
  border: 1px solid alpha(@window_fg_color, 0.12); \
  border-radius: 20px; \
  padding: 10px 6px; \
  box-shadow: 0 1px 5px rgba(0, 0, 0, 0.35); \
}\
.inkpdf-panel separator { \
  background-color: alpha(@window_fg_color, 0.15); \
  margin: 4px 10px; \
}\
.inkpdf-panel button:checked { \
  background-color: @accent_bg_color; \
  color: @accent_fg_color; \
}\
.inkpdf-tab-bar { \
  padding: 0 8px 6px 8px; \
}\
.inkpdf-tab-label { \
  border-radius: 8px; \
}\
.inkpdf-tab-label.active { \
  background-color: alpha(@accent_bg_color, 0.18); \
  color: @window_fg_color; \
}";

/// Installs the panel styling and bundled tool icons once for the default display.
fn load_css() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        // Default to dark mode; the header toggle can switch to light.
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);

        let Some(display) = gdk::Display::default() else {
            return;
        };

        let provider = gtk::CssProvider::new();
        provider.load_from_string(PANEL_CSS);
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // Bundled tool icons (under a hicolor tree, so every theme finds them).
        let icons = concat!(env!("CARGO_MANIFEST_DIR"), "/data/icons");
        gtk::IconTheme::for_display(&display).add_search_path(icons);
    });
}

/// Runs `on_press` when the widget receives a right-click.
fn add_secondary_click(widget: &impl IsA<gtk::Widget>, on_press: impl Fn() + 'static) {
    let gesture = gtk::GestureClick::builder().button(gdk::BUTTON_SECONDARY).build();
    gesture.connect_pressed(move |_, _, _, _| on_press());
    widget.add_controller(gesture);
}

type MenuItem = (&'static str, bool, Box<dyn Fn()>);

/// Pops up a small menu of labelled actions anchored to `anchor`.
fn show_menu(anchor: &impl IsA<gtk::Widget>, items: Vec<MenuItem>) {
    let list = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let popover = gtk::Popover::builder().autohide(true).build();

    for (label, enabled, callback) in items {
        let item = gtk::Button::builder().label(label).sensitive(enabled).build();
        item.add_css_class("flat");
        let popover = popover.clone();
        item.connect_clicked(move |_| {
            callback();
            popover.popdown();
        });
        list.append(&item);
    }

    popover.set_child(Some(&list));
    popover.set_parent(anchor);
    popover.connect_closed(|p| p.unparent());
    popover.popup();
}
