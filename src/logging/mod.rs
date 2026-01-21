//! Logging module for the Finance CLI application.
//!
//! Provides structured logging using the tracing ecosystem.

use crate::error::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize the logging subsystem.
pub fn init() -> Result<()> {
    init_with_level("info")
}

/// Initialize logging with a specific level.
pub fn init_with_level(level: &str) -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false).with_thread_ids(false))
        .try_init()
        .ok(); // Ignore if already initialized

    Ok(())
}

/// Initialize logging for tests (quieter output).
#[cfg(test)]
pub fn init_for_test() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("error")
        .with_test_writer()
        .try_init();
}
