use gtk::gdk;

/// Look up an icon from the current theme with fallback
pub fn lookup_icon(icon_name: &str, size: i32) -> Option<gtk::Image> {
    let display = gdk::Display::default()?;
    let theme = gtk::IconTheme::for_display(&display);

    // Check if the icon exists in the theme
    if theme.has_icon(icon_name) {
        let image = gtk::Image::from_icon_name(icon_name);
        image.set_pixel_size(size);
        return Some(image);
    }

    // Try freedesktop-icons as fallback
    if let Some(path) = freedesktop_icons::lookup(icon_name)
        .with_size(size as u16)
        .find()
    {
        let image = gtk::Image::from_file(&path);
        image.set_pixel_size(size);
        return Some(image);
    }

    None
}

/// Create an image widget for an application, with fallback
pub fn app_icon(icon_name: Option<&str>, size: i32) -> gtk::Image {
    if let Some(name) = icon_name {
        // Try the specified icon
        if let Some(image) = lookup_icon(name, size) {
            return image;
        }

        // If it's a path, try loading directly
        if name.starts_with('/') {
            let image = gtk::Image::from_file(name);
            image.set_pixel_size(size);
            return image;
        }

        tracing::debug!("Icon '{}' not found, using fallback", name);
    }

    // Fallback to generic application icon
    let image = gtk::Image::from_icon_name("application-x-executable");
    image.set_pixel_size(size);
    image
}

/// Create an image widget for a category
pub fn category_icon(icon_name: &str, size: i32) -> gtk::Image {
    lookup_icon(icon_name, size).unwrap_or_else(|| {
        let image = gtk::Image::from_icon_name("application-x-executable");
        image.set_pixel_size(size);
        image
    })
}
