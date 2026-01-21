//! Config command handlers.

use crate::config::Config;
use crate::error::Result;
use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Show current configuration
    Show,

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,

        /// Configuration value
        value: String,
    },

    /// Show configuration file path
    Path,
}

pub fn handle_config(cmd: ConfigCommand, config: &Config) -> Result<()> {
    use colored::Colorize;

    match cmd.action {
        ConfigAction::Show => {
            println!("{}", "Configuration".bold());
            println!();
            println!("Database path: {}", config.database_path.display());
            println!("Config directory: {}", config.config_dir.display());
            println!("Log directory: {}", config.log_dir.display());
            println!("Backup directory: {}", config.backup_dir.display());
            println!();
            println!("Date format: {}", config.date_format);
            println!("Currency: {}", config.currency_symbol);
            println!("Color output: {}", config.color_output);
            println!("Log level: {}", config.log_level);
        }

        ConfigAction::Set { key, value } => {
            println!("{}", "Set Configuration".bold());
            println!();
            println!("Key: {}", key);
            println!("Value: {}", value);
            println!();
            println!("{}", "Configuration update coming soon!".yellow());
        }

        ConfigAction::Path => {
            if let Ok(path) = Config::default_config_path() {
                println!("{}", path.display());
            }
        }
    }

    Ok(())
}
