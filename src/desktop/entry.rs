use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Wrapper around a parsed .desktop file with convenient accessors
#[derive(Debug, Clone)]
pub struct AppEntry {
    /// Desktop file ID (e.g., "firefox.desktop")
    pub id: String,
    /// Localized application name
    pub name: String,
    /// Generic name (e.g., "Web Browser")
    pub generic_name: Option<String>,
    /// Application description
    pub comment: Option<String>,
    /// Icon name or path
    pub icon: Option<String>,
    /// Exec command line
    pub exec: Option<String>,
    /// Whether the app needs a terminal
    pub terminal: bool,
    /// NoDisplay flag (hidden from menus)
    pub no_display: bool,
    /// Hidden flag
    pub hidden: bool,
    /// Supported MIME types
    pub mime_types: Vec<String>,
    /// Application categories
    pub categories: Vec<String>,
    /// Path to the .desktop file
    pub path: PathBuf,
}

impl AppEntry {
    /// Parse an AppEntry from a .desktop file path
    pub fn from_path(path: &Path, locales: &[String]) -> Option<Self> {
        let content = fs::read_to_string(path).ok()?;
        Self::parse(&content, path, locales)
    }

    /// Parse an AppEntry from .desktop file content
    pub fn parse(content: &str, path: &Path, locales: &[String]) -> Option<Self> {
        let mut in_desktop_entry = false;
        let mut values: HashMap<String, String> = HashMap::new();

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') {
                in_desktop_entry = line == "[Desktop Entry]";
                continue;
            }

            if !in_desktop_entry {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                values.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        // Skip if not an Application type
        let entry_type = values.get("Type")?;
        if entry_type != "Application" {
            return None;
        }

        // Get the desktop file name as ID
        let id = path.file_name()?.to_str()?.to_string();

        // Get localized name with fallback
        let name = Self::get_localized(&values, "Name", locales)?;

        // Check NoDisplay and Hidden flags
        let no_display = values.get("NoDisplay").map(|v| v == "true").unwrap_or(false);
        let hidden = values.get("Hidden").map(|v| v == "true").unwrap_or(false);

        // Skip hidden apps for the main list
        if no_display || hidden {
            return None;
        }

        // Parse optional fields
        let generic_name = Self::get_localized(&values, "GenericName", locales);
        let comment = Self::get_localized(&values, "Comment", locales);
        let icon = values.get("Icon").cloned();
        let exec = values.get("Exec").cloned();
        let terminal = values.get("Terminal").map(|v| v == "true").unwrap_or(false);

        // Parse MIME types
        let mime_types = values
            .get("MimeType")
            .map(|s| {
                s.split(';')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.trim().to_string())
                    .collect()
            })
            .unwrap_or_default();

        // Parse categories
        let categories = values
            .get("Categories")
            .map(|s| {
                s.split(';')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.trim().to_string())
                    .collect()
            })
            .unwrap_or_default();

        Some(Self {
            id,
            name,
            generic_name,
            comment,
            icon,
            exec,
            terminal,
            no_display,
            hidden,
            mime_types,
            categories,
            path: path.to_path_buf(),
        })
    }

    /// Get a localized value with fallback to non-localized
    /// Handles locale formats: lang, lang_COUNTRY, lang@MODIFIER, lang_COUNTRY@MODIFIER
    fn get_localized(
        values: &HashMap<String, String>,
        key: &str,
        locales: &[String],
    ) -> Option<String> {
        // Try localized keys first
        for locale in locales {
            // Try full locale with modifier (e.g., "Name[sr@latin]" or "Name[sr_RS@latin]")
            let localized_key = format!("{}[{}]", key, locale);
            if let Some(value) = values.get(&localized_key) {
                return Some(value.clone());
            }

            // If locale has @modifier, try without it
            if let Some((base_locale, _modifier)) = locale.split_once('@') {
                let base_key = format!("{}[{}]", key, base_locale);
                if let Some(value) = values.get(&base_key) {
                    return Some(value.clone());
                }

                // Try language only from base (e.g., "sr" from "sr_RS")
                if let Some((lang, _)) = base_locale.split_once('_') {
                    let lang_key = format!("{}[{}]", key, lang);
                    if let Some(value) = values.get(&lang_key) {
                        return Some(value.clone());
                    }
                }
            } else if let Some((lang, _)) = locale.split_once('_') {
                // Try language only (e.g., "Name[en]" from "en_US")
                let lang_key = format!("{}[{}]", key, lang);
                if let Some(value) = values.get(&lang_key) {
                    return Some(value.clone());
                }
            }
        }

        // Fall back to non-localized key
        values.get(key).cloned()
    }

    /// Check if this app supports a given MIME type
    pub fn supports_mime_type(&self, mime: &str) -> bool {
        self.mime_types
            .iter()
            .any(|m| m == mime || Self::mime_matches_pattern(m, mime))
    }

    /// Check if this app has a given desktop category
    pub fn has_category(&self, category: &str) -> bool {
        self.categories
            .iter()
            .any(|c| c.eq_ignore_ascii_case(category))
    }

    /// Check if a MIME pattern matches a type (e.g., "audio/*" matches "audio/mpeg")
    fn mime_matches_pattern(pattern: &str, mime: &str) -> bool {
        if let Some(prefix) = pattern.strip_suffix("/*") {
            mime.starts_with(prefix) && mime.len() > prefix.len() + 1
        } else {
            false
        }
    }
}
