//! QFX/OFX file parsing.
//!
//! QFX (Quicken Financial Exchange) and OFX (Open Financial Exchange)
//! are standard formats for financial data interchange.

use super::{FileFormat, ParseResult};
use crate::error::{ParseError, Result};
use crate::models::{Account, Money, Transaction, TransactionBuilder};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;
use std::str::FromStr;

/// Parse a QFX/OFX file.
pub fn parse_qfx_file(path: &Path, account: &Account) -> Result<ParseResult> {
    let content = std::fs::read_to_string(path).map_err(|e| crate::error::Error::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    parse_qfx_content(&content, account)
}

/// Parse QFX/OFX content.
pub fn parse_qfx_content(content: &str, account: &Account) -> Result<ParseResult> {
    let mut result = ParseResult::new(FileFormat::Qfx);

    // Extract institution name if available
    if let Some(org) = extract_tag(content, "ORG") {
        result.institution = Some(org);
    }

    // Find all transaction blocks
    let transactions = extract_transactions(content);

    for (idx, tx_content) in transactions.iter().enumerate() {
        match parse_transaction_block(tx_content, account) {
            Ok(tx) => result.transactions.push(tx),
            Err(e) => result.errors.push(format!("Transaction {}: {}", idx + 1, e)),
        }
    }

    Ok(result)
}

/// Extract all STMTTRN blocks from OFX content.
fn extract_transactions(content: &str) -> Vec<String> {
    let mut transactions = Vec::new();
    let content_upper = content.to_uppercase();

    let mut start = 0;
    while let Some(tx_start) = content_upper[start..].find("<STMTTRN>") {
        let abs_start = start + tx_start;
        if let Some(tx_end) = content_upper[abs_start..].find("</STMTTRN>") {
            let abs_end = abs_start + tx_end + "</STMTTRN>".len();
            transactions.push(content[abs_start..abs_end].to_string());
            start = abs_end;
        } else {
            // No closing tag, try to find next transaction
            break;
        }
    }

    transactions
}

/// Parse a single STMTTRN block.
fn parse_transaction_block(content: &str, account: &Account) -> Result<Transaction> {
    // Extract required fields
    let date_str = extract_tag(content, "DTPOSTED")
        .ok_or_else(|| crate::error::Error::Parse(ParseError::MissingField("DTPOSTED".into())))?;

    let amount_str = extract_tag(content, "TRNAMT")
        .ok_or_else(|| crate::error::Error::Parse(ParseError::MissingField("TRNAMT".into())))?;

    // NAME or MEMO for description
    let description = extract_tag(content, "NAME")
        .or_else(|| extract_tag(content, "MEMO"))
        .ok_or_else(|| crate::error::Error::Parse(ParseError::MissingField("NAME or MEMO".into())))?;

    // Parse date (format: YYYYMMDD or YYYYMMDDHHMMSS)
    let date = parse_ofx_date(&date_str)?;

    // Parse amount
    let amount = parse_ofx_amount(&amount_str)?;

    // Optional fields
    let reference = extract_tag(content, "FITID");

    // Build transaction
    let mut builder = TransactionBuilder::new()
        .account_id(account.id)
        .date(date)
        .amount(amount)
        .description(description);

    if let Some(ref_num) = reference {
        builder = builder.reference_number(ref_num);
    }

    builder
        .build()
        .map_err(|e| crate::error::Error::Parse(ParseError::MissingField(e.into())))
}

/// Extract a tag value from OFX content.
fn extract_tag(content: &str, tag: &str) -> Option<String> {
    let open_tag = format!("<{}>", tag.to_uppercase());
    let close_tag = format!("</{}>", tag.to_uppercase());

    let content_upper = content.to_uppercase();

    if let Some(start) = content_upper.find(&open_tag) {
        let value_start = start + open_tag.len();

        // Check for closing tag
        if let Some(end) = content_upper[value_start..].find(&close_tag) {
            return Some(content[value_start..value_start + end].trim().to_string());
        }

        // No closing tag - value ends at next tag or newline
        let remaining = &content[value_start..];
        if let Some(end) = remaining.find('<') {
            return Some(remaining[..end].trim().to_string());
        }
        if let Some(end) = remaining.find('\n') {
            return Some(remaining[..end].trim().to_string());
        }
    }

    None
}

/// Parse OFX date format (YYYYMMDD or YYYYMMDDHHMMSS).
fn parse_ofx_date(s: &str) -> Result<NaiveDate> {
    let date_part = if s.len() >= 8 { &s[..8] } else { s };

    NaiveDate::parse_from_str(date_part, "%Y%m%d").map_err(|_| {
        crate::error::Error::Parse(ParseError::InvalidDate(format!("OFX date: '{}'", s)))
    })
}

/// Parse OFX amount.
fn parse_ofx_amount(s: &str) -> Result<Money> {
    let decimal = Decimal::from_str(s.trim()).map_err(|_| {
        crate::error::Error::Parse(ParseError::InvalidAmount(format!("OFX amount: '{}'", s)))
    })?;

    Ok(Money::new(decimal))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AccountType;

    fn test_account() -> Account {
        Account::new("Test", "Test Bank", AccountType::Checking)
    }

    #[test]
    fn test_extract_tag() {
        let content = "<DTPOSTED>20240115<NAME>Test Purchase";
        assert_eq!(extract_tag(content, "DTPOSTED"), Some("20240115".to_string()));
        assert_eq!(extract_tag(content, "NAME"), Some("Test Purchase".to_string()));
    }

    #[test]
    fn test_parse_ofx_date() {
        let date = parse_ofx_date("20240115").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());

        let date = parse_ofx_date("20240115120000").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
    }

    #[test]
    fn test_parse_ofx_amount() {
        assert_eq!(parse_ofx_amount("-50.00").unwrap().0, Decimal::from_str("-50.00").unwrap());
        assert_eq!(parse_ofx_amount("100.50").unwrap().0, Decimal::from_str("100.50").unwrap());
    }

    #[test]
    fn test_extract_transactions() {
        let content = r#"
            <STMTTRN>
            <TRNTYPE>DEBIT
            <DTPOSTED>20240115
            <TRNAMT>-50.00
            <NAME>Test 1
            </STMTTRN>
            <STMTTRN>
            <TRNTYPE>CREDIT
            <DTPOSTED>20240116
            <TRNAMT>100.00
            <NAME>Test 2
            </STMTTRN>
        "#;

        let transactions = extract_transactions(content);
        assert_eq!(transactions.len(), 2);
    }
}
