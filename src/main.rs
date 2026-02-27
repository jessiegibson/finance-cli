//! Finance CLI - Privacy-first personal finance management
//!
//! This is the main entry point for the CLI application.
//!
//! The application provides comprehensive financial management capabilities:
//! - Import transactions from 8 major banks (CSV/QFX formats)
//! - Rule-based automatic categorization
//! - Financial reports (P&L, Cash Flow, Schedule C)
//! - Local encrypted storage with no cloud dependencies
//! - Privacy-first design with AES-256-GCM encryption

use finance_cli::run;
use std::process::ExitCode;

/// Main entry point for the Finance CLI application.
///
/// Sets up logging, handles global error cases, and delegates to the main
/// application logic in the library crate.
///
/// # Exit Codes
/// - 0: Success
/// - 1: General application error
/// - 2: Configuration error
/// - 3: Encryption/security error
/// - 4: I/O error
/// - 5: Database error
fn main() -> ExitCode {
    // Initialize logging early to catch any startup issues
    if let Err(e) = finance_cli::logging::init() {
        eprintln!("Failed to initialize logging: {e}");
        return ExitCode::FAILURE;
    }

    // Run the main application logic
    match run() {
        Ok(()) => {
            tracing::info!("Application completed successfully");
            ExitCode::SUCCESS
        }
        Err(e) => {
            // Log detailed error for debugging (goes to log file/subscriber)
            tracing::error!("Application error: {e:?}");

            // Display user-friendly error to stderr
            eprintln!("Error: {e}");
            if let Some(suggestion) = e.suggestion() {
                eprintln!("Hint: {suggestion}");
            }

            // Provide context-specific exit codes for better error handling
            match &e {
                finance_cli::Error::Config(_) => ExitCode::from(2),
                finance_cli::Error::Encryption(_) => ExitCode::from(3),
                finance_cli::Error::Io { .. } => ExitCode::from(4),
                finance_cli::Error::Database(_) => ExitCode::from(5),
                _ => ExitCode::FAILURE,
            }
        }
    }
}
