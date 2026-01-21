//! Database connection management.

use crate::config::Config;
use crate::error::{DatabaseError, Error, Result};
use duckdb::{params, Connection as DuckDbConnection};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Database configuration.
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Path to the database file
    pub path: PathBuf,
    /// Whether to create the database if it doesn't exist
    pub create_if_missing: bool,
}

impl DatabaseConfig {
    /// Create a new database configuration.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            create_if_missing: true,
        }
    }

    /// Create configuration from application config.
    pub fn from_config(config: &Config) -> Self {
        Self::new(&config.database_path)
    }
}

/// A thread-safe database connection wrapper.
pub struct Connection {
    inner: Arc<Mutex<DuckDbConnection>>,
    config: DatabaseConfig,
}

impl Connection {
    /// Open a database connection.
    pub fn open(config: &DatabaseConfig) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = config.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        let conn = DuckDbConnection::open(&config.path).map_err(|e| {
            Error::Database(DatabaseError::ConnectionFailed(format!(
                "Failed to open database at {}: {}",
                config.path.display(),
                e
            )))
        })?;

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
            config: config.clone(),
        })
    }

    /// Open an in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self> {
        let conn = DuckDbConnection::open_in_memory().map_err(|e| {
            Error::Database(DatabaseError::ConnectionFailed(format!(
                "Failed to open in-memory database: {}",
                e
            )))
        })?;

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
            config: DatabaseConfig {
                path: PathBuf::from(":memory:"),
                create_if_missing: true,
            },
        })
    }

    /// Execute a SQL statement.
    pub fn execute(&self, sql: &str) -> Result<usize> {
        let conn = self.inner.lock().map_err(|_| {
            Error::Database(DatabaseError::ConnectionFailed(
                "Failed to acquire database lock".into(),
            ))
        })?;

        conn.execute(sql, []).map_err(|e| {
            Error::Database(DatabaseError::QueryFailed(format!(
                "Failed to execute SQL: {}",
                e
            )))
        })
    }

    /// Execute a SQL statement with parameters.
    pub fn execute_with_params<P: duckdb::Params>(&self, sql: &str, params: P) -> Result<usize> {
        let conn = self.inner.lock().map_err(|_| {
            Error::Database(DatabaseError::ConnectionFailed(
                "Failed to acquire database lock".into(),
            ))
        })?;

        conn.execute(sql, params).map_err(|e| {
            Error::Database(DatabaseError::QueryFailed(format!(
                "Failed to execute SQL: {}",
                e
            )))
        })
    }

    /// Execute a batch of SQL statements.
    pub fn execute_batch(&self, sql: &str) -> Result<()> {
        let conn = self.inner.lock().map_err(|_| {
            Error::Database(DatabaseError::ConnectionFailed(
                "Failed to acquire database lock".into(),
            ))
        })?;

        conn.execute_batch(sql).map_err(|e| {
            Error::Database(DatabaseError::QueryFailed(format!(
                "Failed to execute batch SQL: {}",
                e
            )))
        })
    }

    /// Query and map results.
    pub fn query_map<T, F>(&self, sql: &str, f: F) -> Result<Vec<T>>
    where
        F: FnMut(&duckdb::Row<'_>) -> std::result::Result<T, duckdb::Error>,
    {
        let conn = self.inner.lock().map_err(|_| {
            Error::Database(DatabaseError::ConnectionFailed(
                "Failed to acquire database lock".into(),
            ))
        })?;

        let mut stmt = conn.prepare(sql).map_err(|e| {
            Error::Database(DatabaseError::QueryFailed(format!(
                "Failed to prepare SQL: {}",
                e
            )))
        })?;

        let rows = stmt.query_map([], f).map_err(|e| {
            Error::Database(DatabaseError::QueryFailed(format!("Query failed: {}", e)))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| {
                Error::Database(DatabaseError::QueryFailed(format!(
                    "Failed to read row: {}",
                    e
                )))
            })?);
        }

        Ok(results)
    }

    /// Query a single row.
    pub fn query_row<T, F>(&self, sql: &str, f: F) -> Result<Option<T>>
    where
        F: FnOnce(&duckdb::Row<'_>) -> std::result::Result<T, duckdb::Error>,
    {
        let conn = self.inner.lock().map_err(|_| {
            Error::Database(DatabaseError::ConnectionFailed(
                "Failed to acquire database lock".into(),
            ))
        })?;

        let mut stmt = conn.prepare(sql).map_err(|e| {
            Error::Database(DatabaseError::QueryFailed(format!(
                "Failed to prepare SQL: {}",
                e
            )))
        })?;

        match stmt.query_row([], f) {
            Ok(row) => Ok(Some(row)),
            Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Error::Database(DatabaseError::QueryFailed(format!(
                "Query failed: {}",
                e
            )))),
        }
    }

    /// Get the database file path.
    pub fn path(&self) -> &Path {
        &self.config.path
    }

    /// Check if the database is in-memory.
    pub fn is_in_memory(&self) -> bool {
        self.config.path.to_string_lossy() == ":memory:"
    }
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(conn.is_in_memory());
    }

    #[test]
    fn test_execute_basic_sql() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("CREATE TABLE test (id INTEGER, name VARCHAR)")
            .unwrap();
        conn.execute("INSERT INTO test VALUES (1, 'hello')")
            .unwrap();

        let results: Vec<(i32, String)> = conn
            .query_map("SELECT id, name FROM test", |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], (1, "hello".to_string()));
    }
}
