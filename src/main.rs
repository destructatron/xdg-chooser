mod app;
mod config;
mod desktop;
mod ui;
mod utils;
mod window;

use gtk::glib;

fn main() -> glib::ExitCode {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create and run the application
    let app = app::GtkChooserApp::new();
    app.run()
}
