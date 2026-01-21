//! Rule matching logic for categorization.

use crate::models::{ConditionField, LogicalOperator, Rule, RuleCondition, RuleOperator, Transaction};

/// Matcher for applying rules to transactions.
pub struct RuleMatcher;

impl RuleMatcher {
    /// Check if a rule matches a transaction.
    pub fn matches(rule: &Rule, transaction: &Transaction) -> bool {
        if !rule.is_active {
            return false;
        }

        let conditions = &rule.conditions;

        match conditions.operator {
            LogicalOperator::And => {
                conditions.conditions.iter().all(|c| Self::matches_condition(c, transaction))
            }
            LogicalOperator::Or => {
                conditions.conditions.iter().any(|c| Self::matches_condition(c, transaction))
            }
        }
    }

    /// Check if a single condition matches a transaction.
    fn matches_condition(condition: &RuleCondition, transaction: &Transaction) -> bool {
        match condition.field {
            ConditionField::Description => {
                condition.matches_string(&transaction.description)
            }
            ConditionField::MerchantName => {
                transaction
                    .merchant_name
                    .as_ref()
                    .map(|m| condition.matches_string(m))
                    .unwrap_or(false)
            }
            ConditionField::Amount => {
                condition.matches_amount(&transaction.amount)
            }
            ConditionField::AccountId => {
                condition.value == transaction.account_id.to_string()
            }
            ConditionField::RawCategory => {
                transaction
                    .raw_category
                    .as_ref()
                    .map(|c| condition.matches_string(c))
                    .unwrap_or(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Money, RuleBuilder, RuleConditions};
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    fn test_transaction(desc: &str) -> Transaction {
        Transaction::new(
            Uuid::new_v4(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            Money::new(dec!(-50.00)),
            desc.to_string(),
        )
    }

    #[test]
    fn test_contains_match() {
        let rule = RuleBuilder::new("Test", Uuid::new_v4())
            .description_contains("AMAZON")
            .build();

        assert!(RuleMatcher::matches(&rule, &test_transaction("AMAZON.COM")));
        assert!(RuleMatcher::matches(&rule, &test_transaction("amazon marketplace")));
        assert!(!RuleMatcher::matches(&rule, &test_transaction("WALMART")));
    }

    #[test]
    fn test_and_conditions() {
        let rule = RuleBuilder::new("Test", Uuid::new_v4())
            .description_contains("AMAZON")
            .description_contains("PRIME")
            .build();

        assert!(RuleMatcher::matches(&rule, &test_transaction("AMAZON PRIME")));
        assert!(!RuleMatcher::matches(&rule, &test_transaction("AMAZON.COM")));
    }

    #[test]
    fn test_or_conditions() {
        let rule = RuleBuilder::new("Test", Uuid::new_v4())
            .description_contains("AMAZON")
            .description_contains("AMZN")
            .or_logic()
            .build();

        assert!(RuleMatcher::matches(&rule, &test_transaction("AMAZON.COM")));
        assert!(RuleMatcher::matches(&rule, &test_transaction("AMZN MKTP")));
        assert!(!RuleMatcher::matches(&rule, &test_transaction("WALMART")));
    }

    #[test]
    fn test_inactive_rule() {
        let mut rule = RuleBuilder::new("Test", Uuid::new_v4())
            .description_contains("AMAZON")
            .build();
        rule.deactivate();

        assert!(!RuleMatcher::matches(&rule, &test_transaction("AMAZON.COM")));
    }
}
