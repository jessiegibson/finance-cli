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
            "wealthfront_cash" | "wealthfrontcash" => Institution::WealthfrontCash,
            "ally" => Institution::Ally,
            "american_express" | "amex" => Institution::AmericanExpress,
            "discover" => Institution::Discover,
            "citi" | "citibank" => Institution::Citi,
            "capital_one" => Institution::CapitalOne,
            "sofi" => Institution::SoFi,
            _ => detect_institution(content),
        })
        .unwrap_or_else(|| detect_institution(content));

    result.institution = Some(inst.display_name().to_string());
    let mut mapping = inst.csv_mapping();

    // Parse CSV
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(mapping.has_header)
        .flexible(true)
        .from_reader(content.as_bytes());

    // For AMEX, try to detect columns by header names for better robustness
    if matches!(inst, Institution::AmericanExpress) && mapping.has_header {
        if let Ok(headers) = reader.headers() {
            if let Some(updated_mapping) = detect_amex_columns(headers, &mapping) {
                mapping = updated_mapping;
            }
        }
        // Reset reader since we read headers
        reader = csv::ReaderBuilder::new()
            .has_headers(mapping.has_header)
            .flexible(true)
            .from_reader(content.as_bytes());
        let _ = reader.headers();
    }

    for (line_num, record) in reader.records().enumerate() {
        match record {
            Ok(row) => {
                // Skip rows that look like section headers or account-info separators
                // (e.g. "Chase Bank (Account ****2790),,,"). A real date field always
                // contains at least one digit and a '/' or '-' separator.
                if let Some(date_field) = row.get(mapping.date_column) {
                    if !looks_like_date(date_field.trim()) {
                        continue;
                    }
                }
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

/// For AMEX CSVs, detect column positions by header names for flexibility
fn detect_amex_columns(headers: &csv::StringRecord, base_mapping: &super::detect::CsvMapping) -> Option<super::detect::CsvMapping> {
    let headers_lower: Vec<String> = headers
        .iter()
        .map(|h| h.trim().to_lowercase())
        .collect();

    let mut new_mapping = base_mapping.clone();

    // Find Date column (could be "Date" or "Transaction Date")
    for (idx, header) in headers_lower.iter().enumerate() {
        if header.contains("date") && !header.contains("posting") {
            new_mapping.date_column = idx;
            break;
        }
    }

    // Find Amount column
    for (idx, header) in headers_lower.iter().enumerate() {
        if header == "amount" {
            new_mapping.amount_column = idx;
            break;
        }
    }

    // Find Description column
    for (idx, header) in headers_lower.iter().enumerate() {
        if header == "description" {
            new_mapping.description_column = idx;
            break;
        }
    }

    // Find Category column if it exists
    for (idx, header) in headers_lower.iter().enumerate() {
        if header == "category" {
            new_mapping.category_column = Some(idx);
            break;
        }
    }

    Some(new_mapping)
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

/// Return true if `s` looks like a date value.
///
/// A date must contain at least one ASCII digit and at least one date
/// separator (`/` or `-`).  This quick check lets the row-iteration loop
/// silently skip section-header or account-info rows that banks sometimes
/// insert mid-file (e.g. "Chase Bank (Account ****2790),,,").
fn looks_like_date(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_digit()) && (s.contains('/') || s.contains('-'))
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
        .replace(['$', ','], "")
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

    #[test]
    fn test_looks_like_date() {
        assert!(looks_like_date("3/7/2026"));
        assert!(looks_like_date("01/15/2024"));
        assert!(looks_like_date("2024-01-15"));
        assert!(!looks_like_date("Chase Bank (Account ****2790)"));
        assert!(!looks_like_date("Individual Cash Account"));
        assert!(!looks_like_date(""));
        assert!(!looks_like_date("Transaction date"));
    }

    #[test]
    fn test_skip_section_header_rows() {
        // Simulate a SoFi-format CSV that contains a mid-file account-info separator
        // row ("Chase Bank (Account ****2790),,,"). The parser should skip it silently
        // and still import the surrounding real transactions.
        let csv = "Transaction date,Description,Type,Amount\n\
                   3/12/2026,Citi Autopay,Withdrawal,-25.00\n\
                   Chase Bank (Account ****2790),,,\n\
                   3/11/2026,Paycheck,Deposit,869.00";
        let account = test_account();
        let result = parse_csv_content(csv, &account, Some("sofi")).unwrap();

        assert_eq!(result.transactions.len(), 2, "separator row must be skipped");
        assert!(result.errors.is_empty(), "no errors expected: {:?}", result.errors);
    }
}
