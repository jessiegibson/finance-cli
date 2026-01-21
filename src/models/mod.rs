//! Data models for the Finance CLI application.
//!
//! This module contains the core data structures used throughout the application,
//! including transactions, categories, accounts, and rules.

pub mod account;
pub mod category;
pub mod rule;
pub mod transaction;

pub use account::{Account, AccountType, Institution};
pub use category::{Category, CategoryType};
pub use rule::{ConditionField, LogicalOperator, Rule, RuleBuilder, RuleCondition, RuleConditions, RuleOperator};
pub use transaction::{CategorizedBy, Transaction, TransactionBuilder, TransactionStatus};

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Common trait for all entities with identifiers.
pub trait Entity {
    /// Get the entity's unique identifier.
    fn id(&self) -> Uuid;

    /// Check if this is a new (unsaved) entity.
    fn is_new(&self) -> bool;
}

/// Metadata for tracking entity lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMetadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for EntityMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
        }
    }
}

impl EntityMetadata {
    /// Create new metadata with current timestamp.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark the entity as updated.
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// Date range for filtering and reporting.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl DateRange {
    /// Create a new date range.
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        Self { start, end }
    }

    /// Create a date range for a specific year.
    pub fn year(year: i32) -> Self {
        Self {
            start: NaiveDate::from_ymd_opt(year, 1, 1).expect("valid date"),
            end: NaiveDate::from_ymd_opt(year, 12, 31).expect("valid date"),
        }
    }

    /// Create a date range for a specific month.
    pub fn month(year: i32, month: u32) -> Self {
        let start = NaiveDate::from_ymd_opt(year, month, 1).expect("valid date");
        let end = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("valid date")
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).expect("valid date")
        }
        .pred_opt()
        .expect("valid date");
        Self { start, end }
    }

    /// Check if a date is within this range.
    pub fn contains(&self, date: NaiveDate) -> bool {
        date >= self.start && date <= self.end
    }
}

/// Money amount with currency (USD assumed for now).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Money(pub Decimal);

impl Money {
    /// Create a new Money value from a decimal.
    pub fn new(amount: Decimal) -> Self {
        Self(amount)
    }

    /// Create from cents (integer).
    pub fn from_cents(cents: i64) -> Self {
        Self(Decimal::new(cents, 2))
    }

    /// Get the value as cents.
    pub fn cents(&self) -> i64 {
        (self.0 * Decimal::new(100, 0))
            .to_string()
            .parse()
            .unwrap_or(0)
    }

    /// Check if this is an expense (negative amount).
    pub fn is_expense(&self) -> bool {
        self.0.is_sign_negative()
    }

    /// Check if this is income (positive amount).
    pub fn is_income(&self) -> bool {
        self.0.is_sign_positive()
    }

    /// Get the absolute value.
    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }

    /// Zero value.
    pub fn zero() -> Self {
        Self(Decimal::ZERO)
    }
}

impl std::ops::Add for Money {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl std::ops::Sub for Money {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl std::ops::AddAssign for Money {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_sign_negative() {
            write!(f, "-${:.2}", self.0.abs())
        } else {
            write!(f, "${:.2}", self.0)
        }
    }
}

/// Import batch metadata for tracking file imports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportBatch {
    pub id: Uuid,
    pub filename: String,
    pub file_type: String,
    pub institution: String,
    pub transaction_count: i32,
    pub duplicate_count: i32,
    pub error_count: i32,
    pub status: ImportStatus,
    pub imported_at: DateTime<Utc>,
}

/// Status of an import operation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImportStatus {
    Started,
    Processing,
    Completed,
    Failed,
    Partial,
}

impl ImportBatch {
    /// Create a new import batch.
    pub fn new(filename: String, file_type: String, institution: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            filename,
            file_type,
            institution,
            transaction_count: 0,
            duplicate_count: 0,
            error_count: 0,
            status: ImportStatus::Started,
            imported_at: Utc::now(),
        }
    }
}
