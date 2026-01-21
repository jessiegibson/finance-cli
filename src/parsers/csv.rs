//! CSV file parsing for bank transaction exports.

use super::detect::{detect_institution, Institution};
use super::{FileFormat, ParseResult};
use crate::error::{ParseError, Result};
use crate::models::{Account, Money, Transaction, TransactionBuilder};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;
use std::str::FromStr;

/// Parse a CSV file.
pub fn parse_csv_file(path: &Path, account: &Account) -> Result<ParseResult> {
    let content = std::fs::read_to_string(path).map_err(|e| crate::error::Error::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    let institution = detect_institution(&content);
    parse_csv_content(&content, account, Some(institution.as_str()))
}

/// Parse CSV content with optional institution hint.
pub fn parse_csv_content(
    content: &str,
    account: &Account,
    institution: Option<&str>,
) -> Result<ParseResult> {
    let mut result = ParseResult::new(FileFormat::Csv);

    // Detect institution from content or use provided hint
    let inst = institution
        .map(|s| match s.to_lowercase().as_str() {
            "chase" => Institution::Chase,
            "bank_of_america" | "bofa" => Institution::BankOfAmerica,
            "wealthfront" => Institution::Wealthfront,
            "ally" => Institution::Ally,
            "american_express" | "amex" => Institution::AmericanExpress,
            "discover" => Institution::Discover,
            "citi" | "citibank" => Institution::Citi,
            "capital_one" => Institution::CapitalOne,
            _ => detect_institution(content),
        })
        .unwrap_or_else(|| detect_institution(content));

    result.institution = Some(inst.display_name().to_string());
    let mapping = inst.csv_mapping();

    // Parse CSV
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(mapping.has_header)
        .flexible(true)
        .from_reader(content.as_bytes());

    // Skip header if present
    if mapping.has_header {
        let _ = reader.headers();
    }

    for (line_num, record) in reader.records().enumerate() {
        match record {
            Ok(row) => {
                match parse_csv_row(&row, account, &mapping, line_num + 1) {
                    Ok(tx) => result.transactions.push(tx),
                    Err(e) => result.errors.push(format!("Line {}: {}", line_num + 2, e)),
                }
            }
            Err(e) => {
                result.errors.push(format!("Line {}: CSV parse error: {}", line_num + 2, e));
            }
        }
    }

    Ok(result)
}

/// Parse a single CSV row into a Transaction.
fn parse_csv_row(
    row: &csv::StringRecord,
    account: &Account,
    mapping: &super::detect::CsvMapping,
    _line_num: usize,
) -> Result<Transaction> {
    // Extract date
    let date_str = row
        .get(mapping.date_column)
        .ok_or_else(|| crate::error::Error::Parse(ParseError::MissingField("date".into())))?
        .trim();

    let date = parse_date(date_str, mapping.date_format)?;

    // Extract amount
    let amount_str = row
        .get(mapping.amount_column)
        .ok_or_else(|| crate::error::Error::Parse(ParseError::MissingField("amount".into())))?
        .trim();

    let mut amount = parse_amount(amount_str)?;
    if mapping.negate_amounts {
        amount = Money::new(-amount.0);
    }

    // Extract description
    let description = row
        .get(mapping.description_column)
        .ok_or_else(|| crate::error::Error::Parse(ParseError::MissingField("description".into())))?
        .trim()
        .to_string();

    // Extract category if available
    let raw_category = mapping
        .category_column
        .and_then(|col| row.get(col))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    // Build transaction
    let mut builder = TransactionBuilder::new()
        .account_id(account.id)
        .date(date)
        .amount(amount)
        .description(description);

    if let Some(cat) = raw_category {
        builder = builder.raw_category(cat);
    }

    builder
        .build()
        .map_err(|e| crate::error::Error::Parse(ParseError::MissingField(e.into())))
}

/// Parse a date string with the given format.
fn parse_date(s: &str, format: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s, format).map_err(|_| {
        crate::error::Error::Parse(ParseError::InvalidDate(format!("'{}' (expected {})", s, format)))
    })
}

/// Parse an amount string, handling currency symbols and parentheses.
fn parse_amount(s: &str) -> Result<Money> {
    let cleaned = s
        .trim()
        .replace('$', "")
        .replace(',', "")
        .replace(['(', ')'], "");

    let is_negative = s.contains('(') || s.starts_with('-');
    let cleaned = cleaned.trim_start_matches('-');

    let decimal = Decimal::from_str(cleaned).map_err(|_| {
        crate::error::Error::Parse(ParseError::InvalidAmount(format!("'{}'", s)))
    })?;

    let amount = if is_negative && decimal.is_sign_positive() {
        -decimal
    } else {
        decimal
    };

    Ok(Money::new(amount))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AccountType;

    fn test_account() -> Account {
        Account::new("Test", "Test Bank", AccountType::Checking)
    }

    #[test]
    fn test_parse_amount() {
        assert_eq!(parse_amount("100.00").unwrap().0, Decimal::from_str("100.00").unwrap());
        assert_eq!(parse_amount("-50.00").unwrap().0, Decimal::from_str("-50.00").unwrap());
        assert_eq!(parse_amount("$1,234.56").unwrap().0, Decimal::from_str("1234.56").unwrap());
        assert_eq!(parse_amount("(100.00)").unwrap().0, Decimal::from_str("-100.00").unwrap());
    }

    #[test]
    fn test_parse_date() {
        let date = parse_date("01/15/2024", "%m/%d/%Y").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());

        let date = parse_date("2024-01-15", "%Y-%m-%d").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
    }

    #[test]
    fn test_parse_csv_content() {
        let csv = "Date,Amount,Description\n2024-01-15,-50.00,Test Purchase";
        let account = test_account();
        let result = parse_csv_content(csv, &account, None).unwrap();

        assert_eq!(result.transactions.len(), 1);
        assert_eq!(result.transactions[0].description, "Test Purchase");
    }
}
