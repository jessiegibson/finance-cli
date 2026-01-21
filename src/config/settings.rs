//! Application settings and configuration.

use crate::error::{Error, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to the database file.
    pub database_path: PathBuf,

    /// Path to configuration directory.
    pub config_dir: PathBuf,

    /// Path to log files.
    pub log_dir: PathBuf,

    /// Path to backup files.
    pub backup_dir: PathBuf,

    /// Default date format for display.
    #[serde(default = "default_date_format")]
    pub date_format: String,

    /// Default currency symbol.
    #[serde(default = "default_currency")]
    pub currency_symbol: String,

    /// Whether to show colored output.
    #[serde(default = "default_true")]
    pub color_output: bool,

    /// Log level.
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Maximum number of recent imports to keep.
    #[serde(default = "default_max_recent")]
    pub max_recent_imports: usize,
}

fn default_date_format() -> String {
    "%Y-%m-%d".to_string()
}

fn default_currency() -> String {
    "$".to_string()
}

fn default_true() -> bool {
    true
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_recent() -> usize {
    10
}

impl Default for Config {
    fn default() -> Self {
        let base_dir = Self::default_base_dir().unwrap_or_else(|_| PathBuf::from(".finance-cli"));

        Self {
            database_path: base_dir.join("finance.db"),
            config_dir: base_dir.clone(),
            log_dir: base_dir.join("logs"),
            backup_dir: base_dir.join("backups"),
            date_format: default_date_format(),
            currency_symbol: default_currency(),
            color_output: default_true(),
            log_level: default_log_level(),
            max_recent_imports: default_max_recent(),
        }
    }
}

impl Config {
    /// Get the default base directory for the application.
    pub fn default_base_dir() -> Result<PathBuf> {
        ProjectDirs::from("com", "financecli", "finance-cli")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .ok_or_else(|| Error::Config("Could not determine home directory".to_string()))
    }

    /// Get the default config file path.
    pub fn default_config_path() -> Result<PathBuf> {
        let base = Self::default_base_dir()?;
        Ok(base.join("config.toml"))
    }

    /// Load configuration from a file.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| Error::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        toml::from_str(&content).map_err(|e| Error::Config(format!("Invalid config file: {}", e)))
    }

    /// Save configuration to a file.
    pub fn save(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        let content =
            toml::to_string_pretty(self).map_err(|e| Error::Config(format!("Serialize error: {}", e)))?;

        std::fs::write(path, content).map_err(|e| Error::Io {
            path: path.to_path_buf(),
            source: e,
        })
    }

    /// Create a configuration for testing.
    #[cfg(test)]
    pub fn for_testing(base_path: &Path) -> Result<Self> {
        Ok(Self {
            database_path: base_path.join("test.db"),
            config_dir: base_path.to_path_buf(),
            log_dir: base_path.join("logs"),
            backup_dir: base_path.join("backups"),
            ..Default::default()
        })
    }

    /// Ensure all required directories exist.
    pub fn ensure_directories(&self) -> Result<()> {
        for dir in [&self.config_dir, &self.log_dir, &self.backup_dir] {
            std::fs::create_dir_all(dir).map_err(|e| Error::Io {
                path: dir.clone(),
                source: e,
            })?;
        }
        Ok(())
    }
}

/// Builder for creating Config instances.
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    pub fn database_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.database_path = path.into();
        self
    }

    pub fn config_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.config_dir = path.into();
        self
    }

    pub fn log_level(mut self, level: impl Into<String>) -> Self {
        self.config.log_level = level.into();
        self
    }

    pub fn color_output(mut self, enabled: bool) -> Self {
        self.config.color_output = enabled;
        self
    }

    pub fn build(self) -> Config {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.color_output);
        assert_eq!(config.currency_symbol, "$");
    }

    #[test]
    fn test_config_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = Config::default();
        config.save(&config_path).unwrap();

        let loaded = Config::load(&config_path).unwrap();
        assert_eq!(loaded.date_format, config.date_format);
    }

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .log_level("debug")
            .color_output(false)
            .build();

        assert_eq!(config.log_level, "debug");
        assert!(!config.color_output);
    }
}
