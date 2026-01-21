//! Transaction model representing financial transactions.

use super::{Entity, EntityMetadata, Money};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// A financial transaction imported from a bank or manually entered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique identifier
    pub id: Uuid,

    /// Account this transaction belongs to
    pub account_id: Uuid,

    /// Category assigned to this transaction (optional)
    pub category_id: Option<Uuid>,

    /// Import batch this transaction came from (optional)
    pub import_batch_id: Option<Uuid>,

    /// Date the transaction occurred
    pub transaction_date: NaiveDate,

    /// Amount (negative for expenses, positive for income)
    pub amount: Money,

    /// Original description from bank
    pub description: String,

    /// Original category from bank export
    pub raw_category: Option<String>,

    /// Extracted/cleaned merchant name
    pub merchant_name: Option<String>,

    /// Transaction location if available
    pub location: Option<String>,

    /// Bank reference number
    pub reference_number: Option<String>,

    /// SHA-256 hash for duplicate detection
    pub transaction_hash: String,

    /// IRS Schedule C line item mapping
    pub schedule_c_line: Option<String>,

    /// Whether this is a business expense
    pub is_business_expense: bool,

    /// Whether this is tax deductible
    pub is_tax_deductible: bool,

    /// Whether this is a recurring transaction
    pub is_recurring: bool,

    /// Business expense type classification
    pub expense_type: Option<String>,

    /// How the category was assigned
    pub categorized_by: Option<CategorizedBy>,

    /// Confidence score for categorization (0.0 - 1.0)
    pub confidence_score: Option<f64>,

    /// Entity metadata (created_at, updated_at)
    #[serde(flatten)]
    pub metadata: EntityMetadata,
}

/// How a transaction was categorized.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CategorizedBy {
    Rule,
    Manual,
    Default,
    Ml,
}

/// Status for transaction processing.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Pending,
    Categorized,
    Reviewed,
    Excluded,
}

impl Transaction {
    /// Create a new transaction.
    pub fn new(
        account_id: Uuid,
        transaction_date: NaiveDate,
        amount: Money,
        description: String,
    ) -> Self {
        let hash = Self::compute_hash(&transaction_date, &amount, &description);
        Self {
            id: Uuid::new_v4(),
            account_id,
            category_id: None,
            import_batch_id: None,
            transaction_date,
            amount,
            description,
            raw_category: None,
            merchant_name: None,
            location: None,
            reference_number: None,
            transaction_hash: hash,
            schedule_c_line: None,
            is_business_expense: false,
            is_tax_deductible: false,
            is_recurring: false,
            expense_type: None,
            categorized_by: None,
            confidence_score: None,
            metadata: EntityMetadata::new(),
        }
    }

    /// Compute a hash for duplicate detection.
    pub fn compute_hash(date: &NaiveDate, amount: &Money, description: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(date.to_string().as_bytes());
        hasher.update(amount.0.to_string().as_bytes());
        hasher.update(description.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Check if this transaction is categorized.
    pub fn is_categorized(&self) -> bool {
        self.category_id.is_some()
    }

    /// Assign a category to this transaction.
    pub fn categorize(&mut self, category_id: Uuid, by: CategorizedBy, confidence: Option<f64>) {
        self.category_id = Some(category_id);
        self.categorized_by = Some(by);
        self.confidence_score = confidence;
        self.metadata.touch();
    }

    /// Clear the category assignment.
    pub fn uncategorize(&mut self) {
        self.category_id = None;
        self.categorized_by = None;
        self.confidence_score = None;
        self.metadata.touch();
    }

    /// Mark as business expense.
    pub fn mark_business(&mut self, schedule_c_line: Option<String>, expense_type: Option<String>) {
        self.is_business_expense = true;
        self.is_tax_deductible = true;
        self.schedule_c_line = schedule_c_line;
        self.expense_type = expense_type;
        self.metadata.touch();
    }

    /// Mark as personal (non-business).
    pub fn mark_personal(&mut self) {
        self.is_business_expense = false;
        self.is_tax_deductible = false;
        self.schedule_c_line = None;
        self.expense_type = None;
        self.metadata.touch();
    }
}

impl Entity for Transaction {
    fn id(&self) -> Uuid {
        self.id
    }

    fn is_new(&self) -> bool {
        self.metadata.created_at == self.metadata.updated_at
    }
}

/// Builder for creating transactions with optional fields.
#[derive(Debug, Default)]
pub struct TransactionBuilder {
    account_id: Option<Uuid>,
    transaction_date: Option<NaiveDate>,
    amount: Option<Money>,
    description: Option<String>,
    raw_category: Option<String>,
    merchant_name: Option<String>,
    location: Option<String>,
    reference_number: Option<String>,
    import_batch_id: Option<Uuid>,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn account_id(mut self, id: Uuid) -> Self {
        self.account_id = Some(id);
        self
    }

    pub fn date(mut self, date: NaiveDate) -> Self {
        self.transaction_date = Some(date);
        self
    }

    pub fn amount(mut self, amount: Money) -> Self {
        self.amount = Some(amount);
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn raw_category(mut self, cat: impl Into<String>) -> Self {
        self.raw_category = Some(cat.into());
        self
    }

    pub fn merchant_name(mut self, name: impl Into<String>) -> Self {
        self.merchant_name = Some(name.into());
        self
    }

    pub fn location(mut self, loc: impl Into<String>) -> Self {
        self.location = Some(loc.into());
        self
    }

    pub fn reference_number(mut self, ref_num: impl Into<String>) -> Self {
        self.reference_number = Some(ref_num.into());
        self
    }

    pub fn import_batch_id(mut self, id: Uuid) -> Self {
        self.import_batch_id = Some(id);
        self
    }

    pub fn build(self) -> Result<Transaction, &'static str> {
        let account_id = self.account_id.ok_or("account_id is required")?;
        let transaction_date = self.transaction_date.ok_or("transaction_date is required")?;
        let amount = self.amount.ok_or("amount is required")?;
        let description = self.description.ok_or("description is required")?;

        let mut tx = Transaction::new(account_id, transaction_date, amount, description);
        tx.raw_category = self.raw_category;
        tx.merchant_name = self.merchant_name;
        tx.location = self.location;
        tx.reference_number = self.reference_number;
        tx.import_batch_id = self.import_batch_id;

        Ok(tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_transaction_hash_uniqueness() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let amount = Money::new(dec!(-49.99));
        let desc = "AMAZON.COM";

        let hash1 = Transaction::compute_hash(&date, &amount, desc);
        let hash2 = Transaction::compute_hash(&date, &amount, desc);
        assert_eq!(hash1, hash2);

        let different_amount = Money::new(dec!(-50.00));
        let hash3 = Transaction::compute_hash(&date, &different_amount, desc);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_transaction_categorization() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let mut tx = Transaction::new(
            Uuid::new_v4(),
            date,
            Money::new(dec!(-49.99)),
            "Test".to_string(),
        );

        assert!(!tx.is_categorized());

        let cat_id = Uuid::new_v4();
        tx.categorize(cat_id, CategorizedBy::Manual, Some(1.0));

        assert!(tx.is_categorized());
        assert_eq!(tx.category_id, Some(cat_id));
        assert_eq!(tx.categorized_by, Some(CategorizedBy::Manual));
    }
}
