//! File format and institution detection.

use crate::error::{ParseError, Result};
use std::path::Path;

/// Supported file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    Csv,
    Qfx,
    Ofx,
    Unknown,
}

impl FileFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileFormat::Csv => "csv",
            FileFormat::Qfx => "qfx",
            FileFormat::Ofx => "ofx",
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

    // QFX/OFX files typically start with OFXHEADER or <?xml
    if trimmed.starts_with("OFXHEADER") || trimmed.contains("<OFX>") {
        return Ok(FileFormat::Qfx);
    }

    // Check for CSV-like content (comma-separated values with headers)
    if let Some(first_line) = trimmed.lines().next() {
        if first_line.contains(',') && !first_line.contains('<') {
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
    Wealthfront,
    Ally,
    AmericanExpress,
    Discover,
    Citi,
    CapitalOne,
    Unknown,
}

impl Institution {
    pub fn as_str(&self) -> &'static str {
        match self {
            Institution::Chase => "chase",
            Institution::BankOfAmerica => "bank_of_america",
            Institution::Wealthfront => "wealthfront",
            Institution::Ally => "ally",
            Institution::AmericanExpress => "american_express",
            Institution::Discover => "discover",
            Institution::Citi => "citi",
            Institution::CapitalOne => "capital_one",
            Institution::Unknown => "unknown",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Institution::Chase => "Chase",
            Institution::BankOfAmerica => "Bank of America",
            Institution::Wealthfront => "Wealthfront",
            Institution::Ally => "Ally",
            Institution::AmericanExpress => "American Express",
            Institution::Discover => "Discover",
            Institution::Citi => "Citi",
            Institution::CapitalOne => "Capital One",
            Institution::Unknown => "Unknown",
        }
    }
}

/// Detect institution from CSV headers or content.
pub fn detect_institution(content: &str) -> Institution {
    let lower = content.to_lowercase();

    // Check headers and content patterns
    if lower.contains("chase") || lower.contains("details,posting date,description,amount") {
        Institution::Chase
    } else if lower.contains("bank of america") || lower.contains("bofa") {
        Institution::BankOfAmerica
    } else if lower.contains("wealthfront") {
        Institution::Wealthfront
    } else if lower.contains("ally") {
        Institution::Ally
    } else if lower.contains("american express") || lower.contains("amex") {
        Institution::AmericanExpress
    } else if lower.contains("discover") {
        Institution::Discover
    } else if lower.contains("citi") || lower.contains("citibank") {
        Institution::Citi
    } else if lower.contains("capital one") {
        Institution::CapitalOne
    } else {
        Institution::Unknown
    }
}

/// Institution-specific CSV column mappings.
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
                date_column: 0,
                amount_column: 1,
                description_column: 2,
                category_column: None,
                date_format: "%Y-%m-%d",
                has_header: true,
                negate_amounts: false,
            },
            Institution::AmericanExpress => CsvMapping {
                date_column: 0,
                amount_column: 2,
                description_column: 1,
                category_column: None,
                date_format: "%m/%d/%Y",
                has_header: true,
                negate_amounts: true, // AMEX shows expenses as positive
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
}
