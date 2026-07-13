pub mod engine;
pub mod ui;

use adw::prelude::*;
use gtk::gio;

const APP_ID: &str = "de.nikolas.inkpdf";

pub fn run() -> gtk::glib::ExitCode {
    // Sandboxed subprocess entry points (see engine::pdf_worker) - re-
    // invocations of this very binary. Dispatched before any GTK/libadwaita
    // init: workers never open a display connection at all.
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some(engine::pdf_worker::RENDER_WORKER_ARG) {
        engine::pdf_worker::run_render_worker();
    }
    if args.get(1).map(String::as_str) == Some(engine::pdf_worker::EXPORT_WORKER_ARG) {
        let dest = args.get(2).expect("export worker needs a destination path");
        engine::pdf_worker::run_export_worker(dest.into());
    }
    if args.get(1).map(String::as_str) == Some(engine::pdf_worker::SANDBOX_SELFTEST_ARG) {
        engine::sandbox::run_selftest();
    }

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
