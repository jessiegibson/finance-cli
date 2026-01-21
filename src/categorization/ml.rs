//! Machine learning integration placeholder for future ML-based categorization.
//!
//! This module will provide ML-based transaction categorization in the future.

use crate::models::Transaction;

/// ML-based categorization (placeholder for future implementation).
pub struct MlCategorizer;

impl MlCategorizer {
    /// Create a new ML categorizer.
    pub fn new() -> Self {
        Self
    }

    /// Predict category for a transaction (placeholder).
    pub fn predict(&self, _transaction: &Transaction) -> Option<uuid::Uuid> {
        // ML prediction will be implemented in the future
        None
    }
}

impl Default for MlCategorizer {
    fn default() -> Self {
        Self::new()
    }
}
