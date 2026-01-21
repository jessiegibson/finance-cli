//! Finance CLI Library
//!
//! This crate provides the core functionality for the Privacy-First Personal Finance CLI application.
//!
//! # Architecture
//!
//! The application follows a layered architecture:
//!
//! - **Interface Layer**: [`cli`] - Command-line interface and user interaction
//! - **Business Logic Layer**: [`categorization`], [`calculator`] - Core business logic
//! - **Data Layer**: [`parsers`], [`database`], [`config`] - Data access and management
//! - **Infrastructure Layer**: [`encryption`], [`logging`], [`error`] - Cross-cutting concerns
//!
//! # Security
//!
//! All sensitive data is encrypted at rest using AES-256-GCM with keys derived from user passwords
//! via Argon2id. The application operates completely offline with no external network connections.
//!
//! # Privacy
//!
//! - No telemetry or analytics
//! - No cloud storage or external APIs
//! - All data remains on local filesystem
//! - Encrypted storage prevents unauthorized access

// Interface Layer
pub mod cli;

// Business Logic Layer
pub mod calculator;
pub mod categorization;

// Data Layer
pub mod config;
pub mod database;
pub mod parsers;

// Infrastructure Layer
pub mod encryption;
pub mod error;
pub mod logging;

// Shared Models
pub mod models;

// Re-export commonly used types
pub use error::{Error, Result};

/// Run the Finance CLI application.
///
/// This is the main entry point called from `main.rs`. It handles:
/// 1. Command-line argument parsing
/// 2. Configuration loading
/// 3. Database initialization
/// 4. Command execution
/// 5. Graceful error handling
pub fn run() -> Result<()> {
    tracing::info!("Starting Finance CLI application");

    // Parse command line arguments
    let cli_args = cli::parse_args()?;
    tracing::debug!("Parsed CLI arguments");

    // Initialize logging level based on CLI flags
    if cli_args.verbose {
        logging::init_with_level("debug")?;
    }

    // Load or initialize configuration
    let config = config::load_or_create()?;
    tracing::debug!("Loaded configuration");

    // Ensure required directories exist
    config.ensure_directories()?;

    // Initialize database connection
    let db = database::initialize(&config)?;
    tracing::debug!("Database initialized");

    // Execute the requested command
    cli::execute_command(cli_args, config, db)?;

    tracing::info!("Application completed successfully");
    Ok(())
}
