use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{Box as GtkBox, Expander, Label, ListBox, Orientation, ScrolledWindow};

use crate::config::MimeAppsConfig;
use crate::desktop::categories::AppCategory;
use crate::desktop::discovery::AppRegistry;
use crate::desktop::entry::AppEntry;
use crate::ui::app_row::{AppRow, CurrentDefaultRow};
use crate::utils::icons::category_icon;

/// Helper to set all margins at once
fn set_margins(widget: &impl WidgetExt, margin: i32) {
    widget.set_margin_start(margin);
    widget.set_margin_end(margin);
    widget.set_margin_top(margin);
    widget.set_margin_bottom(margin);
}

/// Page displaying a category with its default and available applications
pub struct CategoryPage {
    pub widget: ScrolledWindow,
    category: AppCategory,
}

impl CategoryPage {
    pub fn new<F>(
        category: AppCategory,
        registry: Rc<AppRegistry>,
        config: Rc<RefCell<MimeAppsConfig>>,
        on_default_changed: F,
    ) -> Self
    where
        F: Fn() + 'static,
    {
        let on_default_changed: Rc<dyn Fn()> = Rc::new(on_default_changed);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .build();

        let content = GtkBox::new(Orientation::Vertical, 24);
        set_margins(&content, 24);

        // Header
        let header = Self::create_header(&category);
        content.append(&header);

        // Current default section
        let current_app = Self::get_current_default(&category, &registry, &config.borrow());
        let current_section = Self::create_current_default_section(current_app);
        content.append(&current_section);

        // Available applications
        let available_section = Self::create_available_apps_section(
            &category,
            &registry,
            &config.borrow(),
            Rc::clone(&registry),
            Rc::clone(&config),
            Rc::clone(&on_default_changed),
        );
        content.append(&available_section);

        // MIME type expander (for categories with multiple MIME types)
        let primary_mimes = category.primary_mime_types();
        if primary_mimes.len() > 1 {
            let expander = Self::create_mime_expander(&category, &registry, &config.borrow());
            content.append(&expander);
        }

        scrolled.set_child(Some(&content));

        Self {
            widget: scrolled,
            category,
        }
    }

    fn create_header(category: &AppCategory) -> GtkBox {
        let header = GtkBox::new(Orientation::Horizontal, 16);
        header.set_margin_bottom(8);

        let icon = category_icon(category.icon_name(), 48);
        header.append(&icon);

        let title = Label::new(Some(category.display_name()));
        title.add_css_class("title-1");
        title.set_halign(gtk::Align::Start);
        header.append(&title);

        header
    }

    fn get_current_default<'a>(
        category: &AppCategory,
        registry: &'a AppRegistry,
        config: &MimeAppsConfig,
    ) -> Option<&'a AppEntry> {
        // Try primary MIME types first
        for mime in category.primary_mime_types() {
            if let Some(app_id) = config.get_default(mime) {
                if let Some(app) = registry.get_app(app_id) {
                    return Some(app);
                }
            }
        }

        // For categories without MIME types, just return the first available app
        None
    }

    fn create_current_default_section(current_app: Option<&AppEntry>) -> GtkBox {
        let section = GtkBox::new(Orientation::Vertical, 8);

        let label = Label::new(Some("Current Default"));
        label.add_css_class("heading");
        label.set_halign(gtk::Align::Start);
        section.append(&label);

        let row = CurrentDefaultRow::new(current_app);
        section.append(&row.widget);

        section
    }

    fn create_available_apps_section(
        category: &AppCategory,
        registry: &AppRegistry,
        config: &MimeAppsConfig,
        registry_rc: Rc<AppRegistry>,
        config_rc: Rc<RefCell<MimeAppsConfig>>,
        on_default_changed: Rc<dyn Fn()>,
    ) -> GtkBox {
        let section = GtkBox::new(Orientation::Vertical, 8);

        let label = Label::new(Some("Available Applications"));
        label.add_css_class("heading");
        label.set_halign(gtk::Align::Start);
        section.append(&label);

        let list = ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list"])
            .build();

        // Get apps for this category
        let apps = registry.apps_for_app_category(category);

        // Determine current default
        let current_default = category
            .primary_mime_types()
            .first()
            .and_then(|m| config.get_default(m));

        if apps.is_empty() {
            let empty_label = Label::new(Some("No applications found for this category"));
            empty_label.add_css_class("dim-label");
            set_margins(&empty_label, 24);
            section.append(&empty_label);
        } else {
            for app in apps {
                let is_current = current_default == Some(&app.id);
                let row = AppRow::new(app, is_current);

                // Connect set default handler
                let category_clone = category.clone();
                let config_clone = Rc::clone(&config_rc);
                let on_changed = Rc::clone(&on_default_changed);
                row.connect_set_default(move |app_id| {
                    let mut config = config_clone.borrow_mut();
                    for mime in category_clone.primary_mime_types() {
                        if let Err(e) = config.set_default(mime, &app_id) {
                            tracing::error!("Failed to set default for {}: {}", mime, e);
                            return;
                        }
                    }
                    if let Err(e) = config.save() {
                        tracing::error!("Failed to save config: {}", e);
                    }
                    drop(config);
                    on_changed();
                });

                // Connect test handler
                let registry_clone = Rc::clone(&registry_rc);
                row.connect_test(move |app_id| {
                    if let Some(app) = registry_clone.get_app(&app_id) {
                        if let Err(e) = crate::utils::exec::launch_app(app) {
                            tracing::error!("Failed to launch app: {}", e);
                        }
                    }
                });

                list.append(&row.widget);
            }
            section.append(&list);
        }

        section
    }

    fn create_mime_expander(
        category: &AppCategory,
        registry: &AppRegistry,
        config: &MimeAppsConfig,
    ) -> Expander {
        let expander = Expander::new(Some("Individual MIME Type Settings"));
        expander.set_margin_top(16);

        let content = GtkBox::new(Orientation::Vertical, 8);
        set_margins(&content, 8);

        // List all associated MIME types with their current defaults
        for mime in category.primary_mime_types() {
            let mime_row = Self::create_mime_row(mime, registry, config);
            content.append(&mime_row);
        }

        // Extended MIME types
        for mime in category.extended_mime_types() {
            let mime_row = Self::create_mime_row(mime, registry, config);
            content.append(&mime_row);
        }

        expander.set_child(Some(&content));
        expander
    }

    fn create_mime_row(mime: &str, registry: &AppRegistry, config: &MimeAppsConfig) -> GtkBox {
        let row = GtkBox::new(Orientation::Horizontal, 12);
        row.set_margin_top(4);
        row.set_margin_bottom(4);

        let mime_label = Label::new(Some(mime));
        mime_label.set_halign(gtk::Align::Start);
        mime_label.set_hexpand(true);
        mime_label.add_css_class("monospace");
        row.append(&mime_label);

        let current = config
            .get_default(mime)
            .and_then(|id| registry.get_app(id))
            .map(|app| app.name.as_str())
            .unwrap_or("(none)");

        let current_label = Label::new(Some(current));
        current_label.add_css_class("dim-label");
        row.append(&current_label);

        row
    }

    /// Get the category this page represents
    pub fn category(&self) -> &AppCategory {
        &self.category
    }
}
