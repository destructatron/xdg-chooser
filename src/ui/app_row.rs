use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, Image, Label, ListBoxRow, Orientation};

use crate::desktop::entry::AppEntry;
use crate::utils::icons::app_icon;

/// Helper to set all margins at once
fn set_margins(widget: &impl WidgetExt, margin: i32) {
    widget.set_margin_start(margin);
    widget.set_margin_end(margin);
    widget.set_margin_top(margin);
    widget.set_margin_bottom(margin);
}

/// A row displaying an application with Set as Default button
pub struct AppRow {
    pub widget: ListBoxRow,
    pub app_id: String,
    set_default_btn: Option<Button>,
    test_btn: Button,
}

impl AppRow {
    pub fn new(app: &AppEntry, is_current_default: bool) -> Self {
        let hbox = GtkBox::new(Orientation::Horizontal, 12);
        set_margins(&hbox, 12);

        // App icon
        let icon = app_icon(app.icon.as_deref(), 32);
        hbox.append(&icon);

        // Name and comment
        let text_box = GtkBox::new(Orientation::Vertical, 4);
        text_box.set_hexpand(true);

        let name_label = Label::new(Some(&app.name));
        name_label.set_halign(gtk::Align::Start);
        name_label.add_css_class("heading");
        text_box.append(&name_label);

        if let Some(comment) = &app.comment {
            let comment_label = Label::new(Some(comment));
            comment_label.set_halign(gtk::Align::Start);
            comment_label.add_css_class("dim-label");
            comment_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            comment_label.set_max_width_chars(50);
            text_box.append(&comment_label);
        } else if let Some(generic) = &app.generic_name {
            let generic_label = Label::new(Some(generic));
            generic_label.set_halign(gtk::Align::Start);
            generic_label.add_css_class("dim-label");
            text_box.append(&generic_label);
        }

        hbox.append(&text_box);

        // Current default indicator or Set Default button
        let set_default_btn = if is_current_default {
            let check = Image::from_icon_name("emblem-ok-symbolic");
            check.add_css_class("success");
            check.set_tooltip_text(Some("Current default"));
            hbox.append(&check);
            None
        } else {
            let btn = Button::with_label("Set as Default");
            btn.add_css_class("suggested-action");
            hbox.append(&btn);
            Some(btn)
        };

        // Test button
        let test_btn = Button::from_icon_name("media-playback-start-symbolic");
        test_btn.set_tooltip_text(Some("Launch this application"));
        test_btn.add_css_class("flat");
        hbox.append(&test_btn);

        let row = ListBoxRow::new();
        row.set_child(Some(&hbox));

        Self {
            widget: row,
            app_id: app.id.clone(),
            set_default_btn,
            test_btn,
        }
    }

    /// Connect a callback for the Set as Default button
    pub fn connect_set_default<F>(&self, callback: F)
    where
        F: Fn(String) + Clone + 'static,
    {
        if let Some(btn) = &self.set_default_btn {
            let app_id = self.app_id.clone();
            let callback = callback.clone();
            btn.connect_clicked(move |_| {
                callback(app_id.clone());
            });
        }
    }

    /// Connect a callback for the Test button
    pub fn connect_test<F>(&self, callback: F)
    where
        F: Fn(String) + Clone + 'static,
    {
        let app_id = self.app_id.clone();
        self.test_btn.connect_clicked(move |_| {
            callback(app_id.clone());
        });
    }
}

/// A compact row for the current default display
pub struct CurrentDefaultRow {
    pub widget: GtkBox,
}

impl CurrentDefaultRow {
    pub fn new(app: Option<&AppEntry>) -> Self {
        let hbox = GtkBox::new(Orientation::Horizontal, 12);
        set_margins(&hbox, 12);
        hbox.add_css_class("card");

        if let Some(app) = app {
            let icon = app_icon(app.icon.as_deref(), 48);
            hbox.append(&icon);

            let text_box = GtkBox::new(Orientation::Vertical, 4);
            text_box.set_hexpand(true);

            let name_label = Label::new(Some(&app.name));
            name_label.set_halign(gtk::Align::Start);
            name_label.add_css_class("title-3");
            text_box.append(&name_label);

            if let Some(comment) = &app.comment {
                let comment_label = Label::new(Some(comment));
                comment_label.set_halign(gtk::Align::Start);
                comment_label.add_css_class("dim-label");
                comment_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
                text_box.append(&comment_label);
            }

            hbox.append(&text_box);

            let check = Image::from_icon_name("emblem-ok-symbolic");
            check.add_css_class("success");
            check.set_pixel_size(24);
            hbox.append(&check);
        } else {
            let label = Label::new(Some("No default application set"));
            label.add_css_class("dim-label");
            label.set_hexpand(true);
            hbox.append(&label);
        }

        Self { widget: hbox }
    }
}
