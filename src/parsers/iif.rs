//! IIF (Interchange File Format) parsing for QuickBooks exports.
//!
//! IIF is a text-based format used by QuickBooks and other accounting software
//! to export transactions. This parser extracts transaction records and converts
//! them to the standard Transaction format.
//!
//! Format overview:
//! - Lines starting with ! are keywords
//! - Transactions delimited by !TRNS and !ENDTRNS
//! - Splits delimited by !SPLIT and !ENDSPLIT
//! - Key-value pairs: !KEYWORD	VALUE

use crate::error::{ParseError, Result};
use crate::models::{Account, Money, Transaction, TransactionBuilder};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;
use std::str::FromStr;
use uuid::Uuid;

/// Parse an IIF file.
pub fn parse_iif_file(path: &Path, account: &Account) -> Result<super::ParseResult> {
    let content = std::fs::read_to_string(path).map_err(|e| crate::error::Error::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    parse_iif_content(&content, account)
}

/// Parse IIF content string.
pub fn parse_iif_content(content: &str, account: &Account) -> Result<super::ParseResult> {
    let mut result = super::ParseResult::new(super::FileFormat::Iif);
    result.institution = Some("QuickBooks IIF".to_string());

    let transactions = extract_transaction_blocks(content);

    for block in transactions {
        match parse_transaction_block(&block, account) {
            Ok(tx) => result.transactions.push(tx),
            Err(e) => result.errors.push(format!("Error parsing transaction: {}", e)),
        }
    }

    Ok(result)
}

/// Extract individual transaction blocks from IIF content.
///
/// Returns a Vec of strings, each representing one transaction from !TRNS to !ENDTRNS.
fn extract_transaction_blocks(content: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current_block = String::new();
    let mut in_transaction = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("!TRNS") {
            in_transaction = true;
            current_block.clear();
        }

        if in_transaction {
            current_block.push_str(trimmed);
            current_block.push('\n');
        }

        if trimmed.starts_with("!ENDTRNS") && in_transaction {
            blocks.push(current_block.clone());
            current_block.clear();
            in_transaction = false;
        }
    }

    blocks
}

/// Parse a single transaction block (from !TRNS to !ENDTRNS).
fn parse_transaction_block(content: &str, account: &Account) -> Result<Transaction> {
    // Extract required fields
    let date_str = extract_field(content, "DATE")
        .ok_or_else(|| ParseError::MissingField("IIF transaction missing !DATE".to_string()))?;

    let amount_str = extract_field(content, "AMOUNT")
        .ok_or_else(|| ParseError::MissingField("IIF transaction missing !AMOUNT".to_string()))?;

    // Parse date and amount
    let transaction_date = parse_iif_date(&date_str)?;
    let amount = parse_iif_amount(&amount_str)?;

    // Extract optional fields
    let payee = extract_field(content, "NAME");
    let memo = extract_field(content, "MEMO");
    let docnum = extract_field(content, "DOCNUM");
    let trnsid = extract_field(content, "TRNSID");

    // Build description from payee and memo
    let description = match (&payee, &memo) {
        (Some(p), Some(m)) => format!("{} - {}", p, m),
        (Some(p), None) => p.clone(),
        (None, Some(m)) => m.clone(),
        (None, None) => "IIF Transaction".to_string(),
    };

    // Extract splits if present
    let splits = extract_splits(content);
    let raw_category = if !splits.is_empty() {
        Some(splits_to_category_hint(&splits))
    } else {
        extract_field(content, "SPLITACCNT") // If no splits, check for account field
    };

    // Reference number: prefer DOCNUM (check number) over TRNSID
    let reference_number = docnum.or(trnsid);

    // Build transaction using builder
    let mut tx = TransactionBuilder::new()
        .account_id(account.id)
        .date(transaction_date)
        .amount(amount)
        .description(description)
        .build()
        .map_err(|e| crate::error::Error::InvalidInput(format!("Failed to build transaction: {}", e)))?;

    if let Some(cat) = raw_category {
        tx.raw_category = Some(cat);
    }

    if let Some(ref_num) = reference_number {
        tx.reference_number = Some(ref_num);
    }

    Ok(tx)
}

/// Extract a field value from an IIF block.
///
/// IIF format: !KEYWORD	VALUE (tab-separated)
fn extract_field(content: &str, keyword: &str) -> Option<String> {
    let search_key = format!("!{}", keyword);

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&search_key) {
            // Split on tab or whitespace following the keyword
            if let Some(value) = trimmed.strip_prefix(&search_key) {
                let value = value.trim();
                if value.is_empty() {
                    return None;
                }
                return Some(value.to_string());
            }
        }
    }

    None
}

/// Parse an IIF date string (MM/DD/YYYY format).
fn parse_iif_date(s: &str) -> Result<NaiveDate> {
    Ok(NaiveDate::parse_from_str(s.trim(), "%m/%d/%Y").map_err(|_| {
        ParseError::InvalidDate(format!("Invalid IIF date format: {} (expected MM/DD/YYYY)", s))
    })?)
}

/// Parse an IIF amount string (decimal, can be positive or negative).
fn parse_iif_amount(s: &str) -> Result<Money> {
    let trimmed = s.trim();

    // Remove any currency symbols or commas
    let cleaned = trimmed
        .replace('$', "")
        .replace(',', "")
        .trim()
        .to_string();

    Ok(Decimal::from_str(&cleaned)
        .map(Money::new)
        .map_err(|_| ParseError::InvalidAmount(format!("Invalid IIF amount: {}", s)))?)
}

/// Represents a split transaction line.
#[derive(Debug, Clone)]
struct IifSplit {
    account: String,
    amount: String,
    name: Option<String>,
}

/// Extract split transaction records from a transaction block.
fn extract_splits(content: &str) -> Vec<IifSplit> {
    let mut splits = Vec::new();
    let mut current_split = String::new();
    let mut in_split = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("!SPLIT") {
            in_split = true;
            current_split.clear();
        }

        if in_split {
            current_split.push_str(trimmed);
            current_split.push('\n');
        }

        if trimmed.starts_with("!ENDSPLIT") && in_split {
            if let Some(split) = parse_split_block(&current_split) {
                splits.push(split);
            }
            current_split.clear();
            in_split = false;
        }
    }

    splits
}

/// Parse a single split block.
fn parse_split_block(content: &str) -> Option<IifSplit> {
    let account = extract_field(content, "SPLITACCNT")?;
    let amount = extract_field(content, "SPLITAMT")?;
    let name = extract_field(content, "SPLITNAME");

    Some(IifSplit { account, amount, name })
}

/// Convert split information to a category hint string.
///
/// Format: "split: Account1, Account2, ..." to help with categorization.
fn splits_to_category_hint(splits: &[IifSplit]) -> String {
    let accounts: Vec<String> = splits
        .iter()
        .map(|s| {
            if let Some(name) = &s.name {
                format!("{} ({})", s.account, name)
            } else {
                s.account.clone()
            }
        })
        .collect();

    format!("split: {}", accounts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_account() -> Account {
        Account::new("Test Account", "QuickBooks", crate::models::AccountType::Checking)
    }

    #[test]
    fn test_extract_field() {
        let content = "!TRNS\n!DATE\t03/15/2024\n!AMOUNT\t-125.50\n!ENDTRNS";
        assert_eq!(extract_field(content, "DATE"), Some("03/15/2024".to_string()));
        assert_eq!(extract_field(content, "AMOUNT"), Some("-125.50".to_string()));
        assert_eq!(extract_field(content, "MISSING"), None);
    }

    #[test]
    fn test_parse_iif_date() {
        let date = parse_iif_date("03/15/2024").unwrap();
        assert_eq!(date.to_string(), "2024-03-15");

        let date = parse_iif_date("12/31/2025").unwrap();
        assert_eq!(date.to_string(), "2025-12-31");

        assert!(parse_iif_date("15-03-2024").is_err());
        assert!(parse_iif_date("invalid").is_err());
    }

    #[test]
    fn test_parse_iif_amount() {
        let amount = parse_iif_amount("-125.50").unwrap();
        assert_eq!(amount.0.to_string(), "-125.50");

        let amount = parse_iif_amount("1000.00").unwrap();
        assert_eq!(amount.0.to_string(), "1000");

        let amount = parse_iif_amount("$1,250.75").unwrap();
        assert_eq!(amount.0.to_string(), "1250.75");

        assert!(parse_iif_amount("invalid").is_err());
    }

    #[test]
    fn test_extract_transaction_blocks() {
        let content = r#"!TRNS	CHECK
!DATE	03/15/2024
!AMOUNT	-125.50
!ENDTRNS
!TRNS	DEBIT
!DATE	03/20/2024
!AMOUNT	-50.00
!ENDTRNS"#;

        let blocks = extract_transaction_blocks(content);
        assert_eq!(blocks.len(), 2);
        assert!(blocks[0].contains("!DATE\t03/15/2024"));
        assert!(blocks[1].contains("!DATE\t03/20/2024"));
    }

    #[test]
    fn test_parse_simple_transaction() {
        let content = r#"!TRNS	CHECK
!TRNSID	CHK001
!DATE	03/15/2024
!ACCNT	Checking Account
!NAME	Office Max
!AMOUNT	-125.50
!MEMO	Supplies
!DOCNUM	CHK-5001
!ENDTRNS"#;

        let account = test_account();
        let tx = parse_transaction_block(content, &account).unwrap();

        assert_eq!(tx.transaction_date.to_string(), "2024-03-15");
        assert_eq!(tx.amount.0.to_string(), "-125.50");
        assert!(tx.description.contains("Office Max"));
        assert!(tx.description.contains("Supplies"));
        assert_eq!(tx.reference_number, Some("CHK-5001".to_string()));
    }

    #[test]
    fn test_parse_transaction_missing_date() {
        let content = r#"!TRNS	CHECK
!AMOUNT	-125.50
!NAME	Office Max
!ENDTRNS"#;

        let account = test_account();
        assert!(parse_transaction_block(content, &account).is_err());
    }

    #[test]
    fn test_parse_transaction_missing_amount() {
        let content = r#"!TRNS	CHECK
!DATE	03/15/2024
!NAME	Office Max
!ENDTRNS"#;

        let account = test_account();
        assert!(parse_transaction_block(content, &account).is_err());
    }

    #[test]
    fn test_extract_splits() {
        let content = r#"!TRNS	DEBIT
!DATE	03/20/2024
!AMOUNT	-1000.00
!SPLIT
!SPLITACCNT	Expenses:Office Supplies
!SPLITAMT	-600.00
!SPLITNAME	Supplies
!ENDSPLIT
!SPLIT
!SPLITACCNT	Expenses:Travel
!SPLITAMT	-400.00
!SPLITNAME	Travel
!ENDSPLIT
!ENDTRNS"#;

        let splits = extract_splits(content);
        assert_eq!(splits.len(), 2);
        assert_eq!(splits[0].account, "Expenses:Office Supplies");
        assert_eq!(splits[0].amount, "-600.00");
        assert_eq!(splits[0].name, Some("Supplies".to_string()));
        assert_eq!(splits[1].account, "Expenses:Travel");
    }

    #[test]
    fn test_parse_iif_content_simple() {
        let content = r#"!TRNS	CHECK
!TRNSID	CHK001
!DATE	03/15/2024
!ACCNT	Checking Account
!NAME	Office Max
!AMOUNT	-125.50
!MEMO	Supplies
!DOCNUM	CHK-5001
!ENDTRNS
!TRNS	CHECK
!TRNSID	CHK002
!DATE	03/16/2024
!ACCNT	Checking Account
!NAME	Gas Station
!AMOUNT	-45.00
!MEMO	Fuel
!ENDTRNS"#;

        let account = test_account();
        let result = parse_iif_content(content, &account).unwrap();

        assert_eq!(result.transactions.len(), 2);
        assert!(result.errors.is_empty());
        assert_eq!(result.transactions[0].description, "Office Max - Supplies");
        assert_eq!(result.transactions[1].description, "Gas Station - Fuel");
    }
}
