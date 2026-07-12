mod engine;
mod ui;

use adw::prelude::*;
use gtk::gio;

const APP_ID: &str = "de.nikolas.inkpdf";

fn main() -> gtk::glib::ExitCode {
    adw::init().expect("failed to initialize libadwaita");

    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_activate(|app| {
        ui::window::build(app);
    });

    app.connect_open(|app, files, _hint| {
        let ui = ui::window::build(app);
        if let Some(file) = files.first()
            && let Some(path) = file.path()
        {
            ui.load_path(&path);
        }
    });

    app.run()
}
