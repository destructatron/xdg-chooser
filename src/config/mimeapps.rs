use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

/// Manages MIME type application associations via mimeapps.list
#[derive(Debug, Default)]
pub struct MimeAppsConfig {
    /// Default applications for each MIME type
    pub default_apps: HashMap<String, Vec<String>>,
    /// Additional associations (shown in "Open With" menus)
    pub added_associations: HashMap<String, Vec<String>>,
    /// Explicitly removed associations
    pub removed_associations: HashMap<String, Vec<String>>,
    /// Path to the user's config file (where we write changes)
    path: PathBuf,
}

/// Parsed content from a single mimeapps.list file
#[derive(Debug, Default)]
struct ParsedMimeApps {
    default_apps: HashMap<String, Vec<String>>,
    added_associations: HashMap<String, Vec<String>>,
    removed_associations: HashMap<String, Vec<String>>,
}

impl MimeAppsConfig {
    /// Load the user's mimeapps.list configuration, merging from all XDG locations
    /// per the MIME Applications Associations specification.
    ///
    /// Priority order (highest to lowest):
    /// 1. ~/.config/<desktop>-mimeapps.list (desktop-specific user config)
    /// 2. ~/.config/mimeapps.list (user config)
    /// 3. /etc/xdg/<desktop>-mimeapps.list (desktop-specific system config)
    /// 4. /etc/xdg/mimeapps.list (system config)
    /// 5. ~/.local/share/applications/mimeapps.list (user data)
    /// 6. /usr/share/applications/mimeapps.list (system data)
    pub fn load() -> Result<Self> {
        let xdg_dirs = xdg::BaseDirectories::new()
            .context("Failed to determine XDG directories")?;

        let desktop = env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
        let desktop_lower = desktop.to_lowercase();

        // Collect all config file paths in priority order (highest first)
        let mut config_paths: Vec<PathBuf> = Vec::new();

        // User config directory (~/.config)
        let config_home = xdg_dirs.get_config_home();

        // Desktop-specific user config (highest priority)
        if !desktop_lower.is_empty() {
            let desktop_file = config_home.join(format!("{}-mimeapps.list", desktop_lower));
            if desktop_file.exists() {
                config_paths.push(desktop_file);
            }
        }

        // User config
        let user_config = config_home.join("mimeapps.list");
        if user_config.exists() {
            config_paths.push(user_config.clone());
        }

        // System config directories (/etc/xdg)
        let config_dirs = env::var("XDG_CONFIG_DIRS")
            .unwrap_or_else(|_| "/etc/xdg".to_string());

        for dir in config_dirs.split(':') {
            if dir.is_empty() {
                continue;
            }
            let dir_path = PathBuf::from(dir);

            // Desktop-specific system config
            if !desktop_lower.is_empty() {
                let desktop_file = dir_path.join(format!("{}-mimeapps.list", desktop_lower));
                if desktop_file.exists() {
                    config_paths.push(desktop_file);
                }
            }

            // System config
            let system_config = dir_path.join("mimeapps.list");
            if system_config.exists() {
                config_paths.push(system_config);
            }
        }

        // Data directories for associations
        // User data directory (~/.local/share/applications)
        let data_home = xdg_dirs.get_data_home();
        let user_data = data_home.join("applications/mimeapps.list");
        if user_data.exists() {
            config_paths.push(user_data);
        }

        // System data directories (/usr/share/applications, etc.)
        let data_dirs = env::var("XDG_DATA_DIRS")
            .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());

        for dir in data_dirs.split(':') {
            if dir.is_empty() {
                continue;
            }
            let data_file = PathBuf::from(dir).join("applications/mimeapps.list");
            if data_file.exists() {
                config_paths.push(data_file);
            }
        }

        // Merge all configs (lower priority files are loaded first, higher priority overwrites)
        let mut merged = MimeAppsConfig {
            path: user_config, // We always write to ~/.config/mimeapps.list
            ..Default::default()
        };

        // Load in reverse order so higher priority files overwrite lower priority
        for path in config_paths.into_iter().rev() {
            if let Ok(parsed) = Self::parse_file(&path) {
                merged.merge_from(parsed);
            }
        }

        Ok(merged)
    }

    /// Merge another parsed config into this one (other takes priority)
    fn merge_from(&mut self, other: ParsedMimeApps) {
        // For default apps, later entries override earlier ones
        for (mime, apps) in other.default_apps {
            self.default_apps.insert(mime, apps);
        }

        // For added associations, merge the lists
        for (mime, apps) in other.added_associations {
            let entry = self.added_associations.entry(mime).or_default();
            for app in apps {
                if !entry.contains(&app) {
                    entry.push(app);
                }
            }
        }

        // For removed associations, merge the lists
        for (mime, apps) in other.removed_associations {
            let entry = self.removed_associations.entry(mime).or_default();
            for app in apps {
                if !entry.contains(&app) {
                    entry.push(app);
                }
            }
        }
    }

    fn parse_file(path: &PathBuf) -> Result<ParsedMimeApps> {
        let mut parsed = ParsedMimeApps::default();

        if !path.exists() {
            return Ok(parsed);
        }

        let file = fs::File::open(path)
            .with_context(|| format!("Failed to open {}", path.display()))?;
        let reader = BufReader::new(file);

        let mut current_section = String::new();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Section header
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed[1..trimmed.len() - 1].to_string();
                continue;
            }

            // Key=value pair
            if let Some((key, value)) = trimmed.split_once('=') {
                let mime = key.trim().to_string();
                let apps: Vec<String> = value
                    .split(';')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.trim().to_string())
                    .collect();

                if apps.is_empty() {
                    continue;
                }

                match current_section.as_str() {
                    "Default Applications" => {
                        parsed.default_apps.insert(mime, apps);
                    }
                    "Added Associations" => {
                        parsed.added_associations.insert(mime, apps);
                    }
                    "Removed Associations" => {
                        parsed.removed_associations.insert(mime, apps);
                    }
                    _ => {}
                }
            }
        }

        Ok(parsed)
    }

    /// Validate a MIME type format (must be type/subtype)
    fn validate_mime_type(mime: &str) -> Result<()> {
        if !mime.contains('/') {
            bail!("Invalid MIME type format '{}': must be type/subtype", mime);
        }

        let parts: Vec<&str> = mime.split('/').collect();
        if parts.len() != 2 {
            bail!(
                "Invalid MIME type format '{}': must have exactly one '/'",
                mime
            );
        }

        let (type_part, subtype) = (parts[0], parts[1]);

        if type_part.is_empty() {
            bail!("Invalid MIME type '{}': type cannot be empty", mime);
        }

        if subtype.is_empty() {
            bail!("Invalid MIME type '{}': subtype cannot be empty", mime);
        }

        // Check for valid characters (alphanumeric, dash, dot, plus, x- prefix for extensions)
        for part in [type_part, subtype] {
            if !part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '+')
            {
                bail!(
                    "Invalid MIME type '{}': contains invalid characters",
                    mime
                );
            }
        }

        Ok(())
    }

    /// Validate a desktop file ID format
    fn validate_app_id(app_id: &str) -> Result<()> {
        if !app_id.ends_with(".desktop") {
            bail!(
                "Invalid application ID '{}': must end with .desktop",
                app_id
            );
        }

        if app_id.len() <= 8 {
            // Just ".desktop"
            bail!("Invalid application ID '{}': name cannot be empty", app_id);
        }

        Ok(())
    }

    /// Get the default application for a MIME type
    pub fn get_default(&self, mime: &str) -> Option<&str> {
        self.default_apps
            .get(mime)
            .and_then(|apps| apps.first())
            .map(|s| s.as_str())
    }

    /// Set the default application for a MIME type
    pub fn set_default(&mut self, mime: &str, app_id: &str) -> Result<()> {
        // Validate inputs
        Self::validate_mime_type(mime)?;
        Self::validate_app_id(app_id)?;

        // Set as the default
        self.default_apps
            .insert(mime.to_string(), vec![app_id.to_string()]);

        // Also add to associations if not present
        let associations = self
            .added_associations
            .entry(mime.to_string())
            .or_default();

        // Remove if already present (to avoid duplicates)
        associations.retain(|a| a != app_id);
        // Insert at the beginning
        associations.insert(0, app_id.to_string());

        Ok(())
    }

    /// Set the default application for multiple MIME types
    pub fn set_default_for_mimes(&mut self, mimes: &[&str], app_id: &str) -> Result<()> {
        for mime in mimes {
            self.set_default(mime, app_id)?;
        }
        Ok(())
    }

    /// Remove the default application for a MIME type
    pub fn remove_default(&mut self, mime: &str) {
        self.default_apps.remove(mime);
    }

    /// Get all applications associated with a MIME type
    pub fn get_associations(&self, mime: &str) -> Vec<&str> {
        let mut apps = Vec::new();

        // Add default first
        if let Some(default_apps) = self.default_apps.get(mime) {
            for app in default_apps {
                if !apps.contains(&app.as_str()) {
                    apps.push(app.as_str());
                }
            }
        }

        // Add other associations
        if let Some(added) = self.added_associations.get(mime) {
            for app in added {
                if !apps.contains(&app.as_str()) {
                    apps.push(app.as_str());
                }
            }
        }

        // Filter out removed associations
        if let Some(removed) = self.removed_associations.get(mime) {
            apps.retain(|app| !removed.iter().any(|r| r == *app));
        }

        apps
    }

    /// Save the configuration to disk
    pub fn save(&self) -> Result<()> {
        let mut content = String::new();

        // Write Default Applications
        if !self.default_apps.is_empty() {
            content.push_str("[Default Applications]\n");
            let mut sorted: Vec<_> = self.default_apps.iter().collect();
            sorted.sort_by_key(|(k, _)| k.as_str());
            for (mime, apps) in sorted {
                content.push_str(&format!("{}={}\n", mime, apps.join(";")));
            }
            content.push('\n');
        }

        // Write Added Associations
        if !self.added_associations.is_empty() {
            content.push_str("[Added Associations]\n");
            let mut sorted: Vec<_> = self.added_associations.iter().collect();
            sorted.sort_by_key(|(k, _)| k.as_str());
            for (mime, apps) in sorted {
                content.push_str(&format!("{}={};\n", mime, apps.join(";")));
            }
            content.push('\n');
        }

        // Write Removed Associations
        if !self.removed_associations.is_empty() {
            content.push_str("[Removed Associations]\n");
            let mut sorted: Vec<_> = self.removed_associations.iter().collect();
            sorted.sort_by_key(|(k, _)| k.as_str());
            for (mime, apps) in sorted {
                content.push_str(&format!("{}={};\n", mime, apps.join(";")));
            }
        }

        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        // Write atomically: write to temp file, then rename
        let temp_path = self.path.with_extension("tmp");
        let mut file = fs::File::create(&temp_path)
            .with_context(|| format!("Failed to create {}", temp_path.display()))?;

        file.write_all(content.as_bytes())
            .with_context(|| format!("Failed to write to {}", temp_path.display()))?;

        file.sync_all()?;

        fs::rename(&temp_path, &self.path).with_context(|| {
            format!(
                "Failed to rename {} to {}",
                temp_path.display(),
                self.path.display()
            )
        })?;

        Ok(())
    }

    /// Get the path to the config file
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_mime_type() {
        assert!(MimeAppsConfig::validate_mime_type("text/plain").is_ok());
        assert!(MimeAppsConfig::validate_mime_type("application/x-desktop").is_ok());
        assert!(MimeAppsConfig::validate_mime_type("x-scheme-handler/http").is_ok());
        assert!(MimeAppsConfig::validate_mime_type("image/svg+xml").is_ok());

        assert!(MimeAppsConfig::validate_mime_type("invalid").is_err());
        assert!(MimeAppsConfig::validate_mime_type("/subtype").is_err());
        assert!(MimeAppsConfig::validate_mime_type("type/").is_err());
        assert!(MimeAppsConfig::validate_mime_type("").is_err());
    }

    #[test]
    fn test_validate_app_id() {
        assert!(MimeAppsConfig::validate_app_id("firefox.desktop").is_ok());
        assert!(MimeAppsConfig::validate_app_id("org.mozilla.Firefox.desktop").is_ok());

        assert!(MimeAppsConfig::validate_app_id("firefox").is_err());
        assert!(MimeAppsConfig::validate_app_id(".desktop").is_err());
        assert!(MimeAppsConfig::validate_app_id("").is_err());
    }
}
