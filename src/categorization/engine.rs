//! Categorization engine implementation.

use super::{CategorizationMethod, CategorizationResult, RuleMatcher};
use crate::database::Connection;
use crate::error::Result;
use crate::models::{Category, CategorizedBy, Rule, Transaction};
use uuid::Uuid;

/// The categorization engine applies rules to transactions.
pub struct CategorizationEngine {
    rules: Vec<Rule>,
    categories: Vec<Category>,
}

impl CategorizationEngine {
    /// Create a new categorization engine.
    pub fn new(rules: Vec<Rule>, categories: Vec<Category>) -> Self {
        // Sort rules by priority (lower = higher priority)
        let mut rules = rules;
        rules.sort_by_key(|r| r.priority);

        Self { rules, categories }
    }

    /// Load engine from database.
    pub fn from_database(conn: &Connection) -> Result<Self> {
        use crate::database::{CategoryRepository, RuleRepository};

        let categories = CategoryRepository::new(conn).find_active()?;
        let rules = RuleRepository::new(conn).find_active()?;

        Ok(Self::new(rules, categories))
    }

    /// Categorize a single transaction.
    pub fn categorize(&self, transaction: &Transaction) -> CategorizationResult {
        // Try to match rules in priority order
        for rule in &self.rules {
            if !rule.is_active {
                continue;
            }

            if RuleMatcher::matches(rule, transaction) {
                let category = self
                    .categories
                    .iter()
                    .find(|c| c.id == rule.target_category_id)
                    .cloned();

                return CategorizationResult {
                    transaction_id: transaction.id,
                    category,
                    matched_rule: Some(rule.clone()),
                    confidence: 1.0,
                    method: CategorizationMethod::Rule,
                };
            }
        }

        // No rule matched
        CategorizationResult {
            transaction_id: transaction.id,
            category: None,
            matched_rule: None,
            confidence: 0.0,
            method: CategorizationMethod::None,
        }
    }

    /// Categorize multiple transactions.
    pub fn categorize_batch(&self, transactions: &[Transaction]) -> Vec<CategorizationResult> {
        transactions.iter().map(|tx| self.categorize(tx)).collect()
    }

    /// Get a category by ID.
    pub fn get_category(&self, id: Uuid) -> Option<&Category> {
        self.categories.iter().find(|c| c.id == id)
    }

    /// Get a category by name.
    pub fn get_category_by_name(&self, name: &str) -> Option<&Category> {
        self.categories.iter().find(|c| c.name == name)
    }

    /// Get the "Uncategorized" category.
    pub fn uncategorized_category(&self) -> Option<&Category> {
        self.get_category_by_name("Uncategorized")
    }

    /// Get all active categories.
    pub fn categories(&self) -> &[Category] {
        &self.categories
    }

    /// Get all active rules.
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// Test a rule against transactions without applying.
    pub fn test_rule<'a>(&self, rule: &Rule, transactions: &'a [Transaction]) -> Vec<&'a Transaction> {
        transactions
            .iter()
            .filter(|tx| RuleMatcher::matches(rule, tx))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        Category, CategoryType, ConditionField, Money, Rule, RuleBuilder, RuleCondition,
        RuleConditions, Transaction,
    };
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    fn test_transaction(description: &str, amount: f64) -> Transaction {
        Transaction::new(
            Uuid::new_v4(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            Money::new(rust_decimal::Decimal::from_f64_retain(amount).unwrap()),
            description.to_string(),
        )
    }

    fn test_category() -> Category {
        Category::expense("Office Supplies").with_schedule_c("L18")
    }

    #[test]
    fn test_categorize_with_rule() {
        let category = test_category();
        let rule = RuleBuilder::new("Amazon Rule", category.id)
            .description_contains("AMAZON")
            .build();

        let engine = CategorizationEngine::new(vec![rule], vec![category.clone()]);

        let tx = test_transaction("AMAZON.COM PURCHASE", -50.0);
        let result = engine.categorize(&tx);

        assert_eq!(result.method, CategorizationMethod::Rule);
        assert!(result.category.is_some());
        assert_eq!(result.category.unwrap().name, "Office Supplies");
    }

    #[test]
    fn test_no_matching_rule() {
        let category = test_category();
        let rule = RuleBuilder::new("Amazon Rule", category.id)
            .description_contains("AMAZON")
            .build();

        let engine = CategorizationEngine::new(vec![rule], vec![category]);

        let tx = test_transaction("WALMART PURCHASE", -50.0);
        let result = engine.categorize(&tx);

        assert_eq!(result.method, CategorizationMethod::None);
        assert!(result.category.is_none());
    }

    #[test]
    fn test_rule_priority() {
        let cat1 = Category::expense("Category 1");
        let cat2 = Category::expense("Category 2");

        let rule1 = RuleBuilder::new("Rule 1", cat1.id)
            .description_contains("TEST")
            .priority(100)
            .build();

        let rule2 = RuleBuilder::new("Rule 2", cat2.id)
            .description_contains("TEST")
            .priority(50) // Higher priority
            .build();

        let engine = CategorizationEngine::new(vec![rule1, rule2], vec![cat1, cat2.clone()]);

        let tx = test_transaction("TEST TRANSACTION", -50.0);
        let result = engine.categorize(&tx);

        // Should match rule2 (priority 50) over rule1 (priority 100)
        assert_eq!(result.category.unwrap().name, "Category 2");
    }
}
