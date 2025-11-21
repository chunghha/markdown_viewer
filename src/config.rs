//! Configuration management for the Markdown Viewer
//!
//! This module handles loading and managing application configuration from RON files.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info, warn};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AppConfig {
    /// Window configuration
    pub window: WindowConfig,

    /// File handling configuration
    pub files: FileConfig,

    /// File watcher configuration
    pub file_watcher: FileWatcherConfig,

    /// Scroll behavior configuration
    pub scroll: ScrollConfig,

    /// Theme and styling configuration
    pub theme: ThemeConfig,

    /// Logging configuration
    pub logging: LoggingConfig,
}

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowConfig {
    /// Default window width in pixels
    pub width: f32,

    /// Default window height in pixels
    pub height: f32,

    /// Window title
    pub title: String,
}

/// File handling configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileConfig {
    /// Default files to try loading (in order)
    pub default_files: Vec<String>,

    /// Supported file extensions
    pub supported_extensions: Vec<String>,
}

/// File watcher configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileWatcherConfig {
    /// Enable automatic file watching and reloading
    pub enabled: bool,

    /// Debounce timeout in milliseconds
    pub debounce_ms: u64,
}

/// Scroll behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScrollConfig {
    /// Page scroll percentage (0.0 to 1.0)
    pub page_scroll_percentage: f32,

    /// Arrow key scroll increment in pixels
    pub arrow_key_increment: f32,

    /// Space key scroll percentage (0.0 to 1.0)
    pub space_scroll_percentage: f32,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeConfig {
    /// Primary font family
    pub primary_font: String,

    /// Code font family
    pub code_font: String,

    /// Base text size in pixels
    pub base_text_size: f32,

    /// Line height multiplier
    pub line_height_multiplier: f32,

    /// Content height buffer in pixels
    pub content_height_buffer: f32,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoggingConfig {
    /// Default log level (trace, debug, info, warn, error)
    pub default_level: String,

    /// Enable file logging
    pub enable_file_logging: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1024.0,
            height: 768.0,
            title: "Markdown Viewer".to_string(),
        }
    }
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            default_files: vec!["README.md".to_string(), "TODO.md".to_string()],
            supported_extensions: vec!["md".to_string(), "markdown".to_string(), "txt".to_string()],
        }
    }
}

impl Default for FileWatcherConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debounce_ms: 100,
        }
    }
}

impl Default for ScrollConfig {
    fn default() -> Self {
        Self {
            page_scroll_percentage: 0.8,
            arrow_key_increment: 20.0,
            space_scroll_percentage: 0.2,
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            primary_font: "Google Sans Code".to_string(),
            code_font: "monospace".to_string(),
            base_text_size: 19.2,
            line_height_multiplier: 1.5,
            content_height_buffer: 200.0,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            default_level: "info".to_string(),
            enable_file_logging: false,
        }
    }
}

impl AppConfig {
    /// Load configuration from a file, falling back to defaults if file doesn't exist
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            info!("Configuration file not found at {:?}, using defaults", path);
            return Ok(Self::default());
        }

        debug!("Loading configuration from {:?}", path);
        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read configuration file: {:?}", path))?;

        let config: AppConfig =
            ron::from_str(&content).context("Failed to parse configuration file")?;

        config.validate()?;

        info!("Configuration loaded successfully from {:?}", path);
        Ok(config)
    }

    /// Load configuration from default location (config.ron in current directory)
    pub fn load() -> Result<Self> {
        Self::load_from_file("config.ron")
    }

    /// Save configuration to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();

        debug!("Saving configuration to {:?}", path);
        let content = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .context("Failed to serialize configuration")?;

        std::fs::write(path, content)
            .context(format!("Failed to write configuration file: {:?}", path))?;

        info!("Configuration saved successfully to {:?}", path);
        Ok(())
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        // Validate window dimensions
        if self.window.width <= 0.0 || self.window.height <= 0.0 {
            anyhow::bail!("Window dimensions must be positive");
        }

        // Validate scroll percentages
        if !(0.0..=1.0).contains(&self.scroll.page_scroll_percentage) {
            anyhow::bail!("Page scroll percentage must be between 0.0 and 1.0");
        }

        if !(0.0..=1.0).contains(&self.scroll.space_scroll_percentage) {
            anyhow::bail!("Space scroll percentage must be between 0.0 and 1.0");
        }

        // Validate scroll increment
        if self.scroll.arrow_key_increment <= 0.0 {
            anyhow::bail!("Arrow key increment must be positive");
        }

        // Validate theme values
        if self.theme.base_text_size <= 0.0 {
            anyhow::bail!("Base text size must be positive");
        }

        if self.theme.line_height_multiplier <= 0.0 {
            anyhow::bail!("Line height multiplier must be positive");
        }

        // Validate logging level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.default_level.as_str()) {
            warn!(
                "Invalid log level '{}', using 'info'. Valid levels: {:?}",
                self.logging.default_level, valid_levels
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn default_config_is_valid() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn default_window_config() {
        let config = WindowConfig::default();
        assert_eq!(config.width, 1024.0);
        assert_eq!(config.height, 768.0);
        assert_eq!(config.title, "Markdown Viewer");
    }

    #[test]
    fn default_file_config() {
        let config = FileConfig::default();
        assert_eq!(config.default_files, vec!["README.md", "TODO.md"]);
        assert_eq!(config.supported_extensions, vec!["md", "markdown", "txt"]);
    }

    #[test]
    fn default_scroll_config() {
        let config = ScrollConfig::default();
        assert_eq!(config.page_scroll_percentage, 0.8);
        assert_eq!(config.arrow_key_increment, 20.0);
        assert_eq!(config.space_scroll_percentage, 0.2);
    }

    #[test]
    fn default_theme_config() {
        let config = ThemeConfig::default();
        assert_eq!(config.primary_font, "Google Sans Code");
        assert_eq!(config.code_font, "monospace");
        assert_eq!(config.base_text_size, 19.2);
        assert_eq!(config.line_height_multiplier, 1.5);
        assert_eq!(config.content_height_buffer, 200.0);
    }

    #[test]
    fn default_logging_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.default_level, "info");
        assert!(!config.enable_file_logging);
    }

    #[test]
    fn load_nonexistent_file_returns_default() {
        let result = AppConfig::load_from_file("nonexistent_config.ron");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), AppConfig::default());
    }

    #[test]
    fn save_and_load_config() {
        let config = AppConfig::default();
        let path = "test_config_save_load.ron";

        // Save
        config.save_to_file(path).expect("Failed to save config");

        // Load
        let loaded = AppConfig::load_from_file(path).expect("Failed to load config");

        assert_eq!(config, loaded);

        // Cleanup
        fs::remove_file(path).ok();
    }

    #[test]
    fn validate_rejects_invalid_window_dimensions() {
        let mut config = AppConfig::default();
        config.window.width = -100.0;
        assert!(config.validate().is_err());

        config.window.width = 1024.0;
        config.window.height = 0.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_rejects_invalid_scroll_percentage() {
        let mut config = AppConfig::default();
        config.scroll.page_scroll_percentage = 1.5;
        assert!(config.validate().is_err());

        config.scroll.page_scroll_percentage = -0.1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_rejects_invalid_scroll_increment() {
        let mut config = AppConfig::default();
        config.scroll.arrow_key_increment = -10.0;
        assert!(config.validate().is_err());

        config.scroll.arrow_key_increment = 0.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_rejects_invalid_text_size() {
        let mut config = AppConfig::default();
        config.theme.base_text_size = 0.0;
        assert!(config.validate().is_err());

        config.theme.base_text_size = -5.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_rejects_invalid_line_height() {
        let mut config = AppConfig::default();
        config.theme.line_height_multiplier = 0.0;
        assert!(config.validate().is_err());

        config.theme.line_height_multiplier = -1.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_accepts_invalid_log_level_with_warning() {
        let mut config = AppConfig::default();
        config.logging.default_level = "invalid".to_string();
        // Should not error, just warn
        assert!(config.validate().is_ok());
    }

    #[test]
    fn parse_valid_ron_config() {
        let ron_content = r#"
(
    window: (
        width: 1280.0,
        height: 720.0,
        title: "Custom Viewer",
    ),
    files: (
        default_files: ["README.md"],
        supported_extensions: ["md"],
    ),
    scroll: (
        page_scroll_percentage: 0.9,
        arrow_key_increment: 30.0,
        space_scroll_percentage: 0.3,
    ),
    theme: (
        primary_font: "Arial",
        code_font: "Courier",
        base_text_size: 16.0,
        line_height_multiplier: 1.6,
        content_height_buffer: 500.0,
    ),
    logging: (
        default_level: "debug",
        enable_file_logging: true,
    ),
    file_watcher: (
        enabled: true,
        debounce_ms: 100,
    ),
)
"#;

        let config: AppConfig = ron::from_str(ron_content).expect("Failed to parse RON");
        assert_eq!(config.window.width, 1280.0);
        assert_eq!(config.window.title, "Custom Viewer");
        assert_eq!(config.scroll.page_scroll_percentage, 0.9);
        assert_eq!(config.theme.primary_font, "Arial");
        assert_eq!(config.logging.default_level, "debug");
        assert!(config.logging.enable_file_logging);
    }
}
