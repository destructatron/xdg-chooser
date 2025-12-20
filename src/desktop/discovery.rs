use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use super::categories::AppCategory;
use super::entry::AppEntry;

/// Registry of all discovered applications
pub struct AppRegistry {
    /// All discovered applications, keyed by desktop file ID
    apps: HashMap<String, AppEntry>,
    /// Applications indexed by MIME type
    by_mime: HashMap<String, Vec<String>>,
    /// Applications indexed by desktop category
    by_category: HashMap<String, Vec<String>>,
}

impl AppRegistry {
    /// Scan the system for all available applications
    pub fn new() -> Self {
        let locales = get_locales();
        let mut registry = Self {
            apps: HashMap::new(),
            by_mime: HashMap::new(),
            by_category: HashMap::new(),
        };

        // Get application directories in order (user dirs first)
        let app_dirs = get_application_dirs();

        for dir in app_dirs {
            registry.scan_directory(&dir, &locales);
        }

        registry
    }

    fn scan_directory(&mut self, dir: &PathBuf, locales: &[String]) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => {
                tracing::debug!("Could not read directory {}: {}", dir.display(), e);
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();

            // Only process .desktop files
            if path.extension().is_none_or(|ext| ext != "desktop") {
                continue;
            }

            if let Some(app) = AppEntry::from_path(&path, locales) {
                self.index_app(app);
            }
        }
    }

    fn index_app(&mut self, app: AppEntry) {
        let id = app.id.clone();

        // Don't override user apps with system apps (first one wins)
        if self.apps.contains_key(&id) {
            return;
        }

        // Index by MIME types
        for mime in &app.mime_types {
            self.by_mime
                .entry(mime.clone())
                .or_default()
                .push(id.clone());
        }

        // Index by categories
        for cat in &app.categories {
            self.by_category
                .entry(cat.to_lowercase())
                .or_default()
                .push(id.clone());
        }

        self.apps.insert(id, app);
    }

    /// Get an application by its desktop file ID
    pub fn get_app(&self, id: &str) -> Option<&AppEntry> {
        self.apps.get(id)
    }

    /// Get all applications that support a MIME type
    pub fn apps_for_mime(&self, mime: &str) -> Vec<&AppEntry> {
        use std::collections::HashSet;

        let mut seen: HashSet<&String> = HashSet::new();
        let mut app_ids: Vec<&String> = Vec::new();

        // First check direct matches
        if let Some(direct_matches) = self.by_mime.get(mime) {
            for id in direct_matches {
                if seen.insert(id) {
                    app_ids.push(id);
                }
            }
        }

        // Also check pattern matches (e.g., apps that declare "audio/*")
        if let Some((main_type, _subtype)) = mime.split_once('/') {
            let pattern = format!("{}/*", main_type);
            if let Some(pattern_apps) = self.by_mime.get(&pattern) {
                for id in pattern_apps {
                    if seen.insert(id) {
                        app_ids.push(id);
                    }
                }
            }
        }

        app_ids
            .into_iter()
            .filter_map(|id| self.apps.get(id))
            .collect()
    }

    /// Get all applications with a given desktop category
    pub fn apps_for_category(&self, category: &str) -> Vec<&AppEntry> {
        self.by_category
            .get(&category.to_lowercase())
            .map(|ids| ids.iter().filter_map(|id| self.apps.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all applications for an AppCategory (combines MIME and desktop category search)
    pub fn apps_for_app_category(&self, category: &AppCategory) -> Vec<&AppEntry> {
        let mut seen = std::collections::HashSet::new();
        let mut apps = Vec::new();

        // Search by MIME types
        for mime in category.primary_mime_types() {
            for app in self.apps_for_mime(mime) {
                if seen.insert(&app.id) {
                    apps.push(app);
                }
            }
        }

        // Search by desktop categories
        for cat in category.desktop_categories() {
            for app in self.apps_for_category(cat) {
                if seen.insert(&app.id) {
                    apps.push(app);
                }
            }
        }

        // Sort by name
        apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        apps
    }

    /// Search applications by name
    pub fn search(&self, query: &str) -> Vec<&AppEntry> {
        let query = query.to_lowercase();
        let mut results: Vec<&AppEntry> = self
            .apps
            .values()
            .filter(|app| {
                app.name.to_lowercase().contains(&query)
                    || app
                        .generic_name
                        .as_ref()
                        .is_some_and(|g| g.to_lowercase().contains(&query))
                    || app
                        .comment
                        .as_ref()
                        .is_some_and(|c| c.to_lowercase().contains(&query))
            })
            .collect();

        results.sort_by(|a, b| a.name.cmp(&b.name));
        results
    }

    /// Get all applications
    pub fn all_apps(&self) -> Vec<&AppEntry> {
        let mut apps: Vec<&AppEntry> = self.apps.values().collect();
        apps.sort_by(|a, b| a.name.cmp(&b.name));
        apps
    }
}

impl Default for AppRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the list of locales to try for localized values
fn get_locales() -> Vec<String> {
    let mut locales = Vec::new();

    // Try LANGUAGE first (colon-separated list)
    if let Ok(langs) = env::var("LANGUAGE") {
        for lang in langs.split(':') {
            if !lang.is_empty() {
                locales.push(lang.to_string());
            }
        }
    }

    // Try LC_ALL, LC_MESSAGES, LANG
    for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(locale) = env::var(var) {
            // Strip encoding suffix (e.g., ".UTF-8") but preserve @modifier
            // Format can be: lang_COUNTRY.ENCODING@MODIFIER or lang_COUNTRY@MODIFIER
            let locale = if let Some((base, modifier)) = locale.split_once('@') {
                // Has modifier - strip encoding from base but keep modifier
                let base = base.split('.').next().unwrap_or(base);
                format!("{}@{}", base, modifier)
            } else {
                // No modifier - just strip encoding
                locale.split('.').next().unwrap_or(&locale).to_string()
            };

            if !locale.is_empty() && locale != "C" && locale != "POSIX" {
                if !locales.contains(&locale) {
                    locales.push(locale);
                }
            }
        }
    }

    locales
}

/// Get the list of directories to scan for .desktop files
fn get_application_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // User directory first (higher priority)
    if let Ok(data_home) = env::var("XDG_DATA_HOME") {
        dirs.push(PathBuf::from(data_home).join("applications"));
    } else if let Ok(home) = env::var("HOME") {
        dirs.push(PathBuf::from(home).join(".local/share/applications"));
    }

    // System directories
    if let Ok(data_dirs) = env::var("XDG_DATA_DIRS") {
        for dir in data_dirs.split(':') {
            if !dir.is_empty() {
                dirs.push(PathBuf::from(dir).join("applications"));
            }
        }
    } else {
        // Default system directories
        dirs.push(PathBuf::from("/usr/local/share/applications"));
        dirs.push(PathBuf::from("/usr/share/applications"));
    }

    dirs
}
