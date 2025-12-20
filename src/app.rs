use gtk::prelude::*;
use gtk::{gio, glib, Application};

use crate::window::MainWindow;

const APP_ID: &str = "org.gtkchooser.GtkChooser";

/// Main application struct
pub struct GtkChooserApp {
    app: Application,
}

impl GtkChooserApp {
    pub fn new() -> Self {
        let app = Application::builder()
            .application_id(APP_ID)
            .flags(gio::ApplicationFlags::default())
            .build();

        Self { app }
    }

    pub fn run(&self) -> glib::ExitCode {
        // Connect activate signal
        self.app.connect_activate(|app| {
            // Check if window already exists
            if let Some(window) = app.active_window() {
                window.present();
                return;
            }

            // Create new window
            let window = MainWindow::new(app);
            window.present();
        });

        // Set up application actions
        self.setup_actions();

        self.app.run()
    }

    fn setup_actions(&self) {
        // Quit action
        let quit_action = gio::SimpleAction::new("quit", None);
        let app = self.app.clone();
        quit_action.connect_activate(move |_, _| {
            app.quit();
        });
        self.app.add_action(&quit_action);

        // Set keyboard shortcuts
        self.app.set_accels_for_action("app.quit", &["<Ctrl>q"]);
    }
}

impl Default for GtkChooserApp {
    fn default() -> Self {
        Self::new()
    }
}
