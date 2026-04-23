//! File format and institution detection.

use crate::error::Result;
use std::path::Path;

/// Supported file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    Csv,
    Qfx,
    Ofx,
    Iif,
    Unknown,
}

impl FileFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileFormat::Csv => "csv",
            FileFormat::Qfx => "qfx",
            FileFormat::Ofx => "ofx",
            FileFormat::Iif => "iif",
            FileFormat::Unknown => "unknown",
        }
    }
}

/// Detect the file format based on extension and content.
pub fn detect_format(path: &Path) -> Result<FileFormat> {
    // First check extension
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext.to_lowercase().as_str() {
            "csv" => return Ok(FileFormat::Csv),
            "qfx" => return Ok(FileFormat::Qfx),
            "ofx" => return Ok(FileFormat::Ofx),
            "iif" => return Ok(FileFormat::Iif),
            _ => {}
        }
    }

    // Try to detect from content
    let content = std::fs::read_to_string(path).map_err(|e| crate::error::Error::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    detect_format_from_content(&content)
}

/// Detect format from file content.
pub fn detect_format_from_content(content: &str) -> Result<FileFormat> {
    let trimmed = content.trim();

    // IIF files start with !TRNS or other ! keywords on first line
    if trimmed.starts_with("!TRNS") || trimmed.starts_with("!ACCNT") || trimmed.starts_with("!SPLIT") {
        return Ok(FileFormat::Iif);
    }

    // QFX/OFX files typically start with OFXHEADER or <?xml
    if trimmed.starts_with("OFXHEADER") || trimmed.contains("<OFX>") {
        return Ok(FileFormat::Qfx);
    }

    // Check for CSV-like content (comma-separated values with headers)
    if let Some(first_line) = trimmed.lines().next() {
        if first_line.contains(',') && !first_line.contains('<') && !first_line.starts_with('!') {
            return Ok(FileFormat::Csv);
        }
    }

    Ok(FileFormat::Unknown)
}

/// Known institution identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Institution {
    Chase,
    BankOfAmerica,
    /// Wealthfront investing / brokerage account ("Date,Amount,Description", ISO dates)
    Wealthfront,
    /// Wealthfront Individual Cash Account ("Transaction date,Description,Type,Amount", M/D/YYYY)
    WealthfrontCash,
    Ally,
    AmericanExpress,
    Discover,
    Citi,
    CapitalOne,
    SoFi,
    Unknown,
}

impl Institution {
    pub fn as_str(&self) -> &'static str {
        match self {
            Institution::Chase => "chase",
            Institution::BankOfAmerica => "bank_of_america",
            Institution::Wealthfront => "wealthfront",
            Institution::WealthfrontCash => "wealthfront_cash",
            Institution::Ally => "ally",
            Institution::AmericanExpress => "american_express",
            Institution::Discover => "discover",
            Institution::Citi => "citi",
            Institution::CapitalOne => "capital_one",
            Institution::SoFi => "sofi",
            Institution::Unknown => "unknown",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Institution::Chase => "Chase",
            Institution::BankOfAmerica => "Bank of America",
            Institution::Wealthfront => "Wealthfront",
            Institution::WealthfrontCash => "Wealthfront Cash",
            Institution::Ally => "Ally",
            Institution::AmericanExpress => "American Express",
            Institution::Discover => "Discover",
            Institution::Citi => "Citi",
            Institution::CapitalOne => "Capital One",
            Institution::SoFi => "SoFi",
            Institution::Unknown => "Unknown",
        }
    }
}

/// Check if `text` contains `word` as a whole word (not as a substring of another word).
fn contains_word(text: &str, word: &str) -> bool {
    for (i, _) in text.match_indices(word) {
        let before_ok = i == 0 || !text.as_bytes()[i - 1].is_ascii_alphanumeric();
        let after_idx = i + word.len();
        let after_ok = after_idx >= text.len() || !text.as_bytes()[after_idx].is_ascii_alphanumeric();
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

/// Detect institution from CSV headers or content.
///
/// Uses header-pattern matching first (most reliable), then falls back
/// to whole-word keyword matching to avoid false positives.
pub fn detect_institution(content: &str) -> Institution {
    let lower = content.to_lowercase();

    // Header-pattern matching: check the first few lines for known column layouts.
    // This is more reliable than keyword searches since AMEX CSVs don't contain "amex".
    let first_lines: String = lower.lines().take(3).collect::<Vec<_>>().join("\n");

    // Discover: headers contain "trans. date" and "post date"
    if first_lines.contains("trans. date") && first_lines.contains("post date") {
        return Institution::Discover;
    }

    // AMEX: headers contain "card member" and "account #"
    if first_lines.contains("card member") && first_lines.contains("account #") {
        return Institution::AmericanExpress;
    }

    // Wealthfront Individual Cash Account: "Transaction date,Description,Type,Amount"
    // (distinct from the investing account which uses "Date,Amount,Description")
    if first_lines.contains("transaction date") && first_lines.contains(",type,") {
        return Institution::WealthfrontCash;
    }

    // Chase: "Details,Posting Date,Description,Amount"
    if first_lines.contains("details,posting date,description,amount") {
        return Institution::Chase;
    }

    // Fall back to whole-word keyword matching — restrict to the header area (first 10
    // lines) so that bank names appearing in transaction *descriptions* (e.g. a transfer
    // to "Chase Bank") don't trigger the wrong format.
    let header_area: String = lower.lines().take(10).collect::<Vec<_>>().join("\n");

    if contains_word(&header_area, "chase") {
        Institution::Chase
    } else if header_area.contains("bank of america") || contains_word(&header_area, "bofa") {
        Institution::BankOfAmerica
    } else if contains_word(&header_area, "wealthfront") {
        Institution::Wealthfront
    } else if contains_word(&header_area, "ally") {
        Institution::Ally
    } else if header_area.contains("american express") || contains_word(&header_area, "amex") {
        Institution::AmericanExpress
    } else if contains_word(&header_area, "discover") {
        Institution::Discover
    } else if contains_word(&header_area, "citibank") || contains_word(&header_area, "citi") {
        Institution::Citi
    } else if header_area.contains("capital one") {
        Institution::CapitalOne
    } else if contains_word(&header_area, "sofi") {
        Institution::SoFi
    } else {
        Institution::Unknown
    }
}

/// Institution-specific CSV column mappings.
#[derive(Clone)]
pub struct CsvMapping {
    pub date_column: usize,
    pub amount_column: usize,
    pub description_column: usize,
    pub category_column: Option<usize>,
    pub date_format: &'static str,
    pub has_header: bool,
    pub negate_amounts: bool,
}

impl Institution {
    /// Get the CSV column mapping for this institution.
    pub fn csv_mapping(&self) -> CsvMapping {
        match self {
            Institution::Chase => CsvMapping {
                date_column: 1,        // Posting Date
                amount_column: 3,      // Amount
                description_column: 2, // Description
                category_column: Some(4),
                date_format: "%m/%d/%Y",
                has_header: true,
                negate_amounts: false,
            },
            Institution::BankOfAmerica => CsvMapping {
                date_column: 0,
                amount_column: 2,
                description_column: 1,
                category_column: None,
                date_format: "%m/%d/%Y",
                has_header: true,
                negate_amounts: false,
            },
            Institution::Wealthfront => CsvMapping {
                // Investing / brokerage account export: Date,Amount,Description
                date_column: 0,
                amount_column: 1,
                description_column: 2,
                category_column: None,
                date_format: "%Y-%m-%d",
                has_header: true,
                negate_amounts: false,
            },
            Institution::WealthfrontCash => CsvMapping {
                // Individual Cash Account export: Transaction date,Description,Type,Amount
                date_column: 0,
                description_column: 1,
                category_column: Some(2), // "Type" column (Withdrawal/Deposit/Transfer/etc.)
                amount_column: 3,
                date_format: "%m/%d/%Y",
                has_header: true,
                negate_amounts: false,
            },
            Institution::AmericanExpress => CsvMapping {
                date_column: 0,
                amount_column: 4,      // Date,Description,Card Member,Account #,Amount
                description_column: 1,
                category_column: None,
                date_format: "%m/%d/%Y",
                has_header: true,
                negate_amounts: true, // AMEX shows expenses as positive
            },
            Institution::Discover => CsvMapping {
                date_column: 0,        // Trans. Date
                amount_column: 3,      // Amount
                description_column: 2, // Description
                category_column: Some(4), // Category
                date_format: "%m/%d/%Y",
                has_header: true,
                negate_amounts: true, // Discover shows expenses as positive
            },
            Institution::SoFi => CsvMapping {
                date_column: 0,        // Transaction Date
                amount_column: 3,      // Amount
                description_column: 1, // Description
                category_column: Some(2), // Type
                date_format: "%m/%d/%Y",
                has_header: true,
                negate_amounts: false, // SoFi already uses correct sign convention
            },
            _ => CsvMapping {
                // Generic fallback
                date_column: 0,
                amount_column: 1,
                description_column: 2,
                category_column: None,
                date_format: "%Y-%m-%d",
                has_header: true,
                negate_amounts: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_csv() {
        let content = "Date,Amount,Description\n2024-01-15,-50.00,Test";
        assert_eq!(detect_format_from_content(content).unwrap(), FileFormat::Csv);
    }

    #[test]
    fn test_detect_format_qfx() {
        let content = "OFXHEADER:100\nDATA:OFXSGML";
        assert_eq!(detect_format_from_content(content).unwrap(), FileFormat::Qfx);
    }

    #[test]
    fn test_detect_institution_chase() {
        let content = "Details,Posting Date,Description,Amount,Type,Balance\nDEBIT,01/15/2024,AMAZON,-50.00,ACH,1000.00";
        assert_eq!(detect_institution(content), Institution::Chase);
    }

    #[test]
    fn test_detect_wealthfront_cash() {
        // The Individual Cash Account header should be detected as WealthfrontCash.
        let content = "Transaction date,Description,Type,Amount\n3/12/2026,Paycheck,Deposit,869.00";
        assert_eq!(detect_institution(content), Institution::WealthfrontCash);
    }

    #[test]
    fn test_detect_no_false_positive_from_description() {
        // "Chase Bank" appearing only in a transaction description must not trigger
        // Chase format detection — that would assign the wrong column mapping.
        let content = "Transaction date,Description,Type,Amount\n\
                       3/7/2026,Chase Bank (Account ****2790),Withdrawal,-3000.00\n\
                       3/6/2026,Nys Dol Ui Dd-Ui Dd,Deposit,869.00";
        assert_eq!(detect_institution(content), Institution::WealthfrontCash);
    }
}
