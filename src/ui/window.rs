use std::path::{Path, PathBuf};

use adw::prelude::*;
use gtk::{gdk, gio};

use crate::engine::OpenDocument;
use crate::engine::document::FILE_EXTENSION;
use crate::engine::storage;
use crate::ui::canvas::{Canvas, Relative};

const DEFAULT_WIDTH: i32 = 900;
const DEFAULT_HEIGHT: i32 = 700;

#[derive(Clone)]
pub struct WindowUi {
    window: adw::ApplicationWindow,
    canvas: Canvas,
    stack: gtk::Stack,
    title: adw::WindowTitle,
}

impl WindowUi {
    /// Loads a `.pdf` or `.inkpdf` file, dispatched by extension.
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
                self.canvas.set_open_document(open);
                self.stack.set_visible_child_name("canvas");
                self.title.set_subtitle(&file_label(path));
            }
            Err(err) => show_error(&self.window, &format!("{err:#}")),
        }
    }

    fn insert_page(&self, rel: Relative) {
        self.canvas.insert_blank_page(rel);
        self.stack.set_visible_child_name("canvas");
        if self.title.subtitle().is_empty() {
            self.title.set_subtitle("untitled");
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
        .build();
    let save_button = gtk::Button::builder()
        .icon_name("document-save-symbolic")
        .tooltip_text("Save as inkpdf")
        .build();
    let add_page_button = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Insert page after current")
        .build();
    let remove_page_button = gtk::Button::builder()
        .icon_name("list-remove-symbolic")
        .tooltip_text("Delete current page")
        .build();
    let page_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    page_box.add_css_class("linked");
    page_box.append(&add_page_button);
    page_box.append(&remove_page_button);

    let text_button = gtk::ToggleButton::builder()
        .icon_name("document-edit-symbolic")
        .tooltip_text("Text mode: click a page to add text")
        .build();

    header.pack_start(&open_button);
    header.pack_start(&save_button);
    header.pack_start(&page_box);
    header.pack_start(&text_button);

    let zoom_out_button = gtk::Button::builder()
        .icon_name("zoom-out-symbolic")
        .tooltip_text("Zoom out")
        .build();
    let zoom_in_button = gtk::Button::builder()
        .icon_name("zoom-in-symbolic")
        .tooltip_text("Zoom in")
        .build();
    let zoom_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    zoom_box.add_css_class("linked");
    zoom_box.append(&zoom_out_button);
    zoom_box.append(&zoom_in_button);
    header.pack_end(&zoom_box);

    let canvas = Canvas::new();

    let placeholder = adw::StatusPage::builder()
        .icon_name("document-open-symbolic")
        .title("No PDF open")
        .description("Click Open to load a PDF or inkpdf file.")
        .build();

    let stack = gtk::Stack::new();
    stack.add_named(&placeholder, Some("placeholder"));
    stack.add_named(&canvas.root, Some("canvas"));
    stack.set_visible_child_name("placeholder");

    let content = adw::ToolbarView::new();
    content.add_top_bar(&header);
    content.set_content(Some(&stack));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(DEFAULT_WIDTH)
        .default_height(DEFAULT_HEIGHT)
        .content(&content)
        .build();

    let ui = WindowUi {
        window: window.clone(),
        canvas: canvas.clone(),
        stack,
        title,
    };

    {
        let canvas = canvas.clone();
        zoom_in_button.connect_clicked(move |_| canvas.zoom_in());
    }
    {
        let canvas = canvas.clone();
        zoom_out_button.connect_clicked(move |_| canvas.zoom_out());
    }
    {
        let ui = ui.clone();
        open_button.connect_clicked(move |_| open_dialog(&ui));
    }
    {
        let ui = ui.clone();
        save_button.connect_clicked(move |_| save_dialog(&ui));
    }
    {
        let ui = ui.clone();
        add_page_button.connect_clicked(move |_| ui.insert_page(Relative::After));
    }
    {
        let ui = ui.clone();
        remove_page_button.connect_clicked(move |_| ui.canvas.delete_current_page());
    }
    {
        let ui = ui.clone();
        text_button.connect_toggled(move |btn| ui.canvas.set_text_mode(btn.is_active()));
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
    let Some(model) = ui.canvas.document() else {
        return;
    };

    let dialog = gtk::FileDialog::builder()
        .title("Save as inkpdf")
        .initial_name(format!("untitled.{FILE_EXTENSION}"))
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
            Ok(()) => ui.title.set_subtitle(&file_label(&path)),
            Err(err) => show_error(&ui.window, &format!("{err:#}")),
        }
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
