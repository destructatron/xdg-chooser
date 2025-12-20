use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{
    ApplicationWindow, HeaderBar, Label, MenuButton, Orientation, Paned, PopoverMenu, SearchEntry,
    Stack, StackTransitionType,
};

use crate::config::MimeAppsConfig;
use crate::desktop::categories::AppCategory;
use crate::desktop::discovery::AppRegistry;
use crate::ui::category_page::CategoryPage;
use crate::ui::sidebar::CategorySidebar;

/// Main application window
pub struct MainWindow {
    window: ApplicationWindow,
    registry: Rc<AppRegistry>,
    config: Rc<RefCell<MimeAppsConfig>>,
    stack: Stack,
    sidebar: Rc<CategorySidebar>,
}

impl MainWindow {
    pub fn new(app: &gtk::Application) -> Self {
        // Load data
        let registry = Rc::new(AppRegistry::new());
        let config = Rc::new(RefCell::new(
            MimeAppsConfig::load().unwrap_or_else(|e| {
                tracing::warn!("Failed to load config: {}, using defaults", e);
                MimeAppsConfig::default()
            }),
        ));

        // Create window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Default Applications")
            .default_width(900)
            .default_height(650)
            .build();

        // Create header bar
        let header = Self::create_header_bar();
        window.set_titlebar(Some(&header));

        // Main layout
        let paned = Paned::new(Orientation::Horizontal);
        paned.set_position(220);
        paned.set_shrink_start_child(false);
        paned.set_shrink_end_child(false);

        // Sidebar
        let sidebar = Rc::new(CategorySidebar::new());

        // Wrap sidebar in a scrolled window
        let sidebar_scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .child(&sidebar.widget)
            .build();

        paned.set_start_child(Some(&sidebar_scroll));

        // Content stack
        let stack = Stack::new();
        stack.set_transition_type(StackTransitionType::Crossfade);
        stack.set_transition_duration(200);

        // We'll create pages lazily
        paned.set_end_child(Some(&stack));

        window.set_child(Some(&paned));

        let main_window = Self {
            window,
            registry,
            config,
            stack,
            sidebar,
        };

        // Create initial pages
        main_window.create_category_pages();

        // Connect sidebar selection
        main_window.connect_sidebar();

        main_window
    }

    fn create_header_bar() -> HeaderBar {
        let header = HeaderBar::new();

        // Title
        let title = Label::new(Some("Default Applications"));
        title.add_css_class("title");
        header.set_title_widget(Some(&title));

        // Search entry (for future implementation)
        let search = SearchEntry::new();
        search.set_placeholder_text(Some("Search applications..."));
        search.set_width_chars(25);
        search.set_visible(false); // Hide for now, can enable later
        // header.pack_start(&search);

        // Menu button
        let menu_btn = Self::create_menu_button();
        header.pack_end(&menu_btn);

        header
    }

    fn create_menu_button() -> MenuButton {
        let menu_btn = MenuButton::new();
        menu_btn.set_icon_name("open-menu-symbolic");
        menu_btn.set_tooltip_text(Some("Main menu"));

        // Create menu
        let menu = gio::Menu::new();
        menu.append(Some("About"), Some("win.about"));
        menu.append(Some("Quit"), Some("app.quit"));

        let popover = PopoverMenu::from_model(Some(&menu));
        menu_btn.set_popover(Some(&popover));

        menu_btn
    }

    fn create_category_pages(&self) {
        for category in AppCategory::all() {
            self.create_page_for_category(&category);
        }
    }

    fn create_page_for_category(&self, category: &AppCategory) {
        let registry = Rc::clone(&self.registry);
        let config = Rc::clone(&self.config);
        let stack = self.stack.clone();
        let category_clone = category.clone();

        // Create a callback that rebuilds the page when defaults change
        let on_default_changed = move || {
            // Rebuild the page
            if let Some(child) = stack.child_by_name(category_clone.display_name()) {
                stack.remove(&child);
            }

            let page = CategoryPage::new(
                category_clone.clone(),
                Rc::clone(&registry),
                Rc::clone(&config),
                || {}, // No recursive rebuilding
            );
            stack.add_named(&page.widget, Some(category_clone.display_name()));
        };

        let page = CategoryPage::new(
            category.clone(),
            Rc::clone(&self.registry),
            Rc::clone(&self.config),
            on_default_changed,
        );

        self.stack
            .add_named(&page.widget, Some(category.display_name()));
    }

    fn connect_sidebar(&self) {
        let stack = self.stack.clone();
        self.sidebar.connect_category_selected(move |category| {
            stack.set_visible_child_name(category.display_name());
        });
    }

    pub fn present(&self) {
        self.window.present();
    }

    /// Rebuild all category pages (e.g., after settings change)
    pub fn rebuild_pages(&self) {
        // Remove all children
        while let Some(child) = self.stack.first_child() {
            self.stack.remove(&child);
        }

        // Recreate pages
        self.create_category_pages();

        // Re-select current category
        if let Some(row) = self.sidebar.widget.selected_row() {
            let index = row.index() as usize;
            if let Some(category) = self.sidebar.category_at(index) {
                self.stack.set_visible_child_name(category.display_name());
            }
        }
    }
}
