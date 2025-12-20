use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, ListBox, ListBoxRow, Orientation};

use crate::desktop::categories::AppCategory;
use crate::utils::icons::category_icon;

/// Navigation sidebar for selecting application categories
pub struct CategorySidebar {
    pub widget: ListBox,
    categories: Vec<AppCategory>,
}

impl CategorySidebar {
    pub fn new() -> Self {
        let widget = ListBox::builder()
            .selection_mode(gtk::SelectionMode::Single)
            .css_classes(["navigation-sidebar"])
            .vexpand(true)
            .build();

        let categories = AppCategory::all();

        for category in &categories {
            let row = Self::create_row(category);
            widget.append(&row);
        }

        // Select first row by default
        if let Some(first) = widget.row_at_index(0) {
            widget.select_row(Some(&first));
        }

        Self { widget, categories }
    }

    fn create_row(category: &AppCategory) -> ListBoxRow {
        let hbox = GtkBox::new(Orientation::Horizontal, 12);
        hbox.set_margin_start(12);
        hbox.set_margin_end(12);
        hbox.set_margin_top(8);
        hbox.set_margin_bottom(8);

        let icon = category_icon(category.icon_name(), 24);
        hbox.append(&icon);

        let label = Label::new(Some(category.display_name()));
        label.set_halign(gtk::Align::Start);
        label.set_hexpand(true);
        hbox.append(&label);

        let row = ListBoxRow::new();
        row.set_child(Some(&hbox));
        row
    }

    /// Connect a callback for when a category is selected
    pub fn connect_category_selected<F>(&self, callback: F)
    where
        F: Fn(AppCategory) + 'static,
    {
        let categories = self.categories.clone();
        self.widget.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                let index = row.index() as usize;
                if let Some(category) = categories.get(index) {
                    callback(category.clone());
                }
            }
        });
    }

    /// Get the category at the given index
    pub fn category_at(&self, index: usize) -> Option<&AppCategory> {
        self.categories.get(index)
    }

    /// Get the number of categories
    pub fn len(&self) -> usize {
        self.categories.len()
    }

    /// Check if sidebar is empty
    pub fn is_empty(&self) -> bool {
        self.categories.is_empty()
    }
}

impl Default for CategorySidebar {
    fn default() -> Self {
        Self::new()
    }
}
