//! Database module for the Finance CLI application.
//!
//! This module provides database operations using DuckDB for local SQL storage.
//! All data is stored locally with no cloud dependencies.

pub mod connection;
pub mod migrations;
pub mod models;
pub mod queries;

pub use connection::{Connection, DatabaseConfig};
pub use queries::{AccountRepository, CategoryRepository, RuleRepository, TransactionRepository};

use crate::config::Config;
use crate::error::Result;

/// Initialize the database with the given configuration.
pub fn initialize(config: &Config) -> Result<Connection> {
    let db_config = DatabaseConfig::from_config(config);
    let conn = Connection::open(&db_config)?;

    // Run migrations
    migrations::run_migrations(&conn)?;

    Ok(conn)
}

/// Initialize an in-memory database for testing.
#[cfg(test)]
pub fn initialize_test() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    migrations::run_migrations(&conn)?;
    Ok(conn)
}
