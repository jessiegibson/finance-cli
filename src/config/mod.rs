//! Configuration management for the Finance CLI application.
//!
//! This module handles loading, saving, and managing application configuration.

pub mod settings;

pub use settings::{Config, ConfigBuilder};

use crate::error::{Error, Result};
use std::path::Path;

/// Load configuration from file or create default.
pub fn load_or_create() -> Result<Config> {
    let config_path = Config::default_config_path()?;

    if config_path.exists() {
        Config::load(&config_path)
    } else {
        let config = Config::default();
        config.save(&config_path)?;
        Ok(config)
    }
}

/// Load configuration from a specific path.
pub fn load_from(path: &Path) -> Result<Config> {
    Config::load(path)
}
