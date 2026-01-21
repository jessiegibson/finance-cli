//! Transaction file parsing module.
//!
//! This module handles parsing transaction files from various banks and formats.
//! Supported formats: CSV, QFX/OFX.
//!
//! Supported institutions:
//! - Chase
//! - Bank of America
//! - Wealthfront
//! - Ally
//! - American Express
//! - Discover
//! - Citi
//! - Capital One

pub mod csv;
pub mod detect;
pub mod qfx;

pub use detect::{detect_format, detect_institution, FileFormat};

use crate::error::{ParseError, Result};
use crate::models::{Account, Money, Transaction, TransactionBuilder};
use std::path::Path;

/// Result of parsing a transaction file.
#[derive(Debug)]
pub struct ParseResult {
    /// Successfully parsed transactions.
    pub transactions: Vec<Transaction>,
    /// Potential duplicate transactions (based on hash).
    pub duplicates: Vec<Transaction>,
    /// Parsing errors that occurred.
    pub errors: Vec<String>,
    /// Detected file format.
    pub format: FileFormat,
    /// Detected institution.
    pub institution: Option<String>,
}

impl ParseResult {
    pub fn new(format: FileFormat) -> Self {
        Self {
            transactions: Vec::new(),
            duplicates: Vec::new(),
            errors: Vec::new(),
            format,
            institution: None,
        }
    }

    /// Get the total number of transactions (including duplicates).
    pub fn total_count(&self) -> usize {
        self.transactions.len() + self.duplicates.len()
    }

    /// Check if parsing had any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// Parse a transaction file.
pub fn parse_file(path: &Path, account: &Account) -> Result<ParseResult> {
    // Detect format
    let format = detect_format(path)?;

    match format {
        FileFormat::Csv => csv::parse_csv_file(path, account),
        FileFormat::Qfx | FileFormat::Ofx => qfx::parse_qfx_file(path, account),
        FileFormat::Unknown => Err(crate::error::Error::Parse(ParseError::UnknownFormat)),
    }
}

/// Parse raw CSV content.
pub fn parse_csv_content(content: &str, account: &Account, institution: Option<&str>) -> Result<ParseResult> {
    csv::parse_csv_content(content, account, institution)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AccountType;

    fn test_account() -> Account {
        Account::new("Test Account", "Test Bank", AccountType::Checking)
    }

    #[test]
    fn test_parse_result_new() {
        let result = ParseResult::new(FileFormat::Csv);
        assert!(result.transactions.is_empty());
        assert!(!result.has_errors());
    }
}
