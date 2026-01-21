//! Command-line interface module.
//!
//! This module handles all user interaction through the command line.

pub mod commands;
pub mod output;

use crate::config::Config;
use crate::database::Connection;
use crate::error::Result;
use clap::{Parser, Subcommand};

/// Main CLI application structure.
#[derive(Parser, Debug)]
#[command(
    name = "finance",
    about = "Privacy-first personal finance management CLI",
    long_about = "A privacy-first personal finance management tool for freelancers and small business owners.
Import transactions from bank exports, categorize them automatically, and generate tax-ready financial reports.
All data is encrypted and stored locally with no cloud dependencies.",
    version
)]
pub struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    pub config: Option<std::path::PathBuf>,

    /// The command to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Transaction management (import, list, categorize)
    #[command(alias = "tx")]
    Transaction(commands::TransactionCommand),

    /// Generate financial reports
    Report(commands::ReportCommand),

    /// Manage categories and categorization rules
    Category(commands::CategoryCommand),

    /// Application configuration
    Config(commands::ConfigCommand),

    /// Initialize a new database
    Init,

    /// Show application status and statistics
    Status,
}

/// Parse command line arguments.
pub fn parse_args() -> Result<Cli> {
    Ok(Cli::parse())
}

/// Execute the parsed command.
pub fn execute_command(cli: Cli, config: Config, conn: Connection) -> Result<()> {
    match cli.command {
        Commands::Init => commands::handle_init(&config, &conn),
        Commands::Status => commands::handle_status(&config, &conn),
        Commands::Transaction(cmd) => commands::handle_transaction(cmd, &config, &conn),
        Commands::Report(cmd) => commands::handle_report(cmd, &config, &conn),
        Commands::Category(cmd) => commands::handle_category(cmd, &config, &conn),
        Commands::Config(cmd) => commands::handle_config(cmd, &config),
    }
}
