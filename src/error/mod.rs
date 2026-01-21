//! Error handling module for the Finance CLI application.
//!
//! Provides centralized error types, user-friendly error messages,
//! and recovery guidance.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias using our custom Error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the Finance CLI application.
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Database errors
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    /// Encryption/decryption errors
    #[error("Encryption error: {0}")]
    Encryption(#[from] EncryptionError),

    /// File parsing errors
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    /// I/O errors with path context
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Categorization errors
    #[error("Categorization error: {0}")]
    Categorization(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// User input errors
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Transaction-related errors
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Report generation errors
    #[error("Report error: {0}")]
    Report(String),

    /// Generic internal errors
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Database-specific errors.
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Transaction not found: {0}")]
    NotFound(String),

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    #[error("Integrity error: {0}")]
    IntegrityError(String),

    #[error("DuckDB error: {0}")]
    DuckDb(String),
}

/// Encryption-specific errors.
#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Invalid password")]
    InvalidPassword,

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid recovery code")]
    InvalidRecoveryCode,

    #[error("Corrupted data")]
    CorruptedData,

    #[error("Missing encryption key")]
    MissingKey,
}

/// Parse-specific errors.
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unknown file format")]
    UnknownFormat,

    #[error("Unknown institution")]
    UnknownInstitution,

    #[error("Invalid CSV: {0}")]
    InvalidCsv(String),

    #[error("Invalid QFX/OFX: {0}")]
    InvalidQfx(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid date format: {0}")]
    InvalidDate(String),

    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Encoding error: {0}")]
    EncodingError(String),
}

impl Error {
    /// Returns a user-friendly suggestion for how to fix the error.
    pub fn suggestion(&self) -> Option<&'static str> {
        match self {
            Error::Config(_) => Some("Check your configuration file at ~/.finance-cli/config.toml"),
            Error::Encryption(EncryptionError::InvalidPassword) => {
                Some("Make sure you're using the correct password. If forgotten, use recovery code.")
            }
            Error::Parse(ParseError::UnknownFormat) => {
                Some("Supported formats are CSV and QFX/OFX files from supported banks.")
            }
            Error::Parse(ParseError::UnknownInstitution) => {
                Some("Try specifying the bank with --bank flag, or use generic CSV import.")
            }
            Error::Database(DatabaseError::ConnectionFailed(_)) => {
                Some("Ensure the database file exists and you have write permissions.")
            }
            _ => None,
        }
    }

    /// Returns true if this error is recoverable.
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Error::InvalidInput(_) | Error::Validation(_) | Error::Parse(_)
        )
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io {
            path: PathBuf::new(),
            source: err,
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Config(err.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::Config(err.to_string())
    }
}
