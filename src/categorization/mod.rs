//! Categorization engine for automatic transaction classification.
//!
//! This module provides rule-based and ML-assisted categorization
//! of financial transactions.

pub mod engine;
pub mod ml;
pub mod rules;

pub use engine::CategorizationEngine;
pub use ml::MlCategorizer;
pub use rules::RuleMatcher;

use crate::models::{Category, Rule, Transaction};

/// Result of categorizing a single transaction.
#[derive(Debug)]
pub struct CategorizationResult {
    /// The transaction that was categorized.
    pub transaction_id: uuid::Uuid,
    /// The assigned category (if any).
    pub category: Option<Category>,
    /// The rule that matched (if any).
    pub matched_rule: Option<Rule>,
    /// Confidence score (0.0 - 1.0).
    pub confidence: f64,
    /// How the categorization was determined.
    pub method: CategorizationMethod,
}

/// Method used for categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategorizationMethod {
    /// Matched a user-defined rule.
    Rule,
    /// Used default category based on transaction type.
    Default,
    /// ML model prediction.
    MachineLearning,
    /// Manual user assignment.
    Manual,
    /// No categorization applied.
    None,
}
