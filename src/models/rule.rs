//! Rule model for automatic transaction categorization.

use super::{Entity, EntityMetadata, Money};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A categorization rule for automatic transaction classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Unique identifier
    pub id: Uuid,

    /// Target category to assign
    pub target_category_id: Uuid,

    /// Display name
    pub name: String,

    /// Description
    pub description: Option<String>,

    /// Priority (lower = higher priority)
    pub priority: i32,

    /// Rule conditions
    pub conditions: RuleConditions,

    /// Whether this rule is active
    pub is_active: bool,

    /// Number of transactions matched by this rule
    pub effectiveness_count: i32,

    /// Last time this rule was applied
    pub last_applied_at: Option<DateTime<Utc>>,

    /// Entity metadata
    #[serde(flatten)]
    pub metadata: EntityMetadata,
}

/// Container for rule conditions with AND/OR logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConditions {
    /// Logical operator for combining conditions
    pub operator: LogicalOperator,

    /// List of conditions
    pub conditions: Vec<RuleCondition>,
}

/// Logical operator for combining conditions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogicalOperator {
    And,
    Or,
}

/// A single condition within a rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    /// Field to match against
    pub field: ConditionField,

    /// Comparison operator
    pub operator: RuleOperator,

    /// Value to compare against
    pub value: String,

    /// Whether comparison is case-sensitive
    #[serde(default)]
    pub case_sensitive: bool,
}

/// Fields that can be matched in conditions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConditionField {
    Description,
    MerchantName,
    Amount,
    AccountId,
    RawCategory,
}

/// Comparison operators for conditions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuleOperator {
    Contains,
    Equals,
    StartsWith,
    EndsWith,
    Regex,
    GreaterThan,
    LessThan,
    Between,
}

impl Rule {
    /// Create a new rule.
    pub fn new(name: impl Into<String>, target_category_id: Uuid, conditions: RuleConditions) -> Self {
        Self {
            id: Uuid::new_v4(),
            target_category_id,
            name: name.into(),
            description: None,
            priority: 100,
            conditions,
            is_active: true,
            effectiveness_count: 0,
            last_applied_at: None,
            metadata: EntityMetadata::new(),
        }
    }

    /// Set the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the priority.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Record that this rule was applied.
    pub fn record_application(&mut self) {
        self.effectiveness_count += 1;
        self.last_applied_at = Some(Utc::now());
        self.metadata.touch();
    }

    /// Deactivate the rule.
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.metadata.touch();
    }

    /// Reactivate the rule.
    pub fn activate(&mut self) {
        self.is_active = true;
        self.metadata.touch();
    }
}

impl Entity for Rule {
    fn id(&self) -> Uuid {
        self.id
    }

    fn is_new(&self) -> bool {
        self.metadata.created_at == self.metadata.updated_at
    }
}

impl RuleConditions {
    /// Create conditions with AND logic.
    pub fn all(conditions: Vec<RuleCondition>) -> Self {
        Self {
            operator: LogicalOperator::And,
            conditions,
        }
    }

    /// Create conditions with OR logic.
    pub fn any(conditions: Vec<RuleCondition>) -> Self {
        Self {
            operator: LogicalOperator::Or,
            conditions,
        }
    }

    /// Create a single condition.
    pub fn single(condition: RuleCondition) -> Self {
        Self {
            operator: LogicalOperator::And,
            conditions: vec![condition],
        }
    }
}

impl RuleCondition {
    /// Create a "contains" condition.
    pub fn contains(field: ConditionField, value: impl Into<String>) -> Self {
        Self {
            field,
            operator: RuleOperator::Contains,
            value: value.into(),
            case_sensitive: false,
        }
    }

    /// Create an "equals" condition.
    pub fn equals(field: ConditionField, value: impl Into<String>) -> Self {
        Self {
            field,
            operator: RuleOperator::Equals,
            value: value.into(),
            case_sensitive: false,
        }
    }

    /// Create a "starts with" condition.
    pub fn starts_with(field: ConditionField, value: impl Into<String>) -> Self {
        Self {
            field,
            operator: RuleOperator::StartsWith,
            value: value.into(),
            case_sensitive: false,
        }
    }

    /// Create a "regex" condition.
    pub fn regex(field: ConditionField, pattern: impl Into<String>) -> Self {
        Self {
            field,
            operator: RuleOperator::Regex,
            value: pattern.into(),
            case_sensitive: true,
        }
    }

    /// Create an amount comparison condition.
    pub fn amount_greater_than(amount: Money) -> Self {
        Self {
            field: ConditionField::Amount,
            operator: RuleOperator::GreaterThan,
            value: amount.0.to_string(),
            case_sensitive: false,
        }
    }

    /// Create an amount comparison condition.
    pub fn amount_less_than(amount: Money) -> Self {
        Self {
            field: ConditionField::Amount,
            operator: RuleOperator::LessThan,
            value: amount.0.to_string(),
            case_sensitive: false,
        }
    }

    /// Set case sensitivity.
    pub fn case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_sensitive = sensitive;
        self
    }

    /// Test if a string value matches this condition.
    pub fn matches_string(&self, value: &str) -> bool {
        let (test_value, match_value) = if self.case_sensitive {
            (value.to_string(), self.value.clone())
        } else {
            (value.to_lowercase(), self.value.to_lowercase())
        };

        match self.operator {
            RuleOperator::Contains => test_value.contains(&match_value),
            RuleOperator::Equals => test_value == match_value,
            RuleOperator::StartsWith => test_value.starts_with(&match_value),
            RuleOperator::EndsWith => test_value.ends_with(&match_value),
            RuleOperator::Regex => {
                if let Ok(re) = Regex::new(&self.value) {
                    re.is_match(value)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Test if an amount matches this condition.
    pub fn matches_amount(&self, amount: &Money) -> bool {
        let threshold: rust_decimal::Decimal = match self.value.parse() {
            Ok(v) => v,
            Err(_) => return false,
        };

        match self.operator {
            RuleOperator::GreaterThan => amount.0 > threshold,
            RuleOperator::LessThan => amount.0 < threshold,
            RuleOperator::Equals => amount.0 == threshold,
            RuleOperator::Between => {
                // Value format: "min,max"
                let parts: Vec<&str> = self.value.split(',').collect();
                if parts.len() != 2 {
                    return false;
                }
                let min: rust_decimal::Decimal = match parts[0].trim().parse() {
                    Ok(v) => v,
                    Err(_) => return false,
                };
                let max: rust_decimal::Decimal = match parts[1].trim().parse() {
                    Ok(v) => v,
                    Err(_) => return false,
                };
                amount.0 >= min && amount.0 <= max
            }
            _ => false,
        }
    }
}

/// Builder for creating rules.
pub struct RuleBuilder {
    name: String,
    target_category_id: Uuid,
    conditions: Vec<RuleCondition>,
    operator: LogicalOperator,
    priority: i32,
    description: Option<String>,
}

impl RuleBuilder {
    pub fn new(name: impl Into<String>, target_category_id: Uuid) -> Self {
        Self {
            name: name.into(),
            target_category_id,
            conditions: Vec::new(),
            operator: LogicalOperator::And,
            priority: 100,
            description: None,
        }
    }

    pub fn add_condition(mut self, condition: RuleCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    pub fn description_contains(self, value: impl Into<String>) -> Self {
        self.add_condition(RuleCondition::contains(ConditionField::Description, value))
    }

    pub fn merchant_contains(self, value: impl Into<String>) -> Self {
        self.add_condition(RuleCondition::contains(ConditionField::MerchantName, value))
    }

    pub fn or_logic(mut self) -> Self {
        self.operator = LogicalOperator::Or;
        self
    }

    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn build(self) -> Rule {
        let conditions = RuleConditions {
            operator: self.operator,
            conditions: self.conditions,
        };

        let mut rule = Rule::new(self.name, self.target_category_id, conditions)
            .with_priority(self.priority);

        if let Some(desc) = self.description {
            rule = rule.with_description(desc);
        }

        rule
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_rule_condition_contains() {
        let condition = RuleCondition::contains(ConditionField::Description, "AMAZON");
        assert!(condition.matches_string("AMAZON.COM PURCHASE"));
        assert!(condition.matches_string("amazon marketplace"));
        assert!(!condition.matches_string("WALMART"));
    }

    #[test]
    fn test_rule_condition_regex() {
        let condition = RuleCondition::regex(ConditionField::Description, r"^AMZN.*MKTP");
        assert!(condition.matches_string("AMZN MKTP US"));
        assert!(!condition.matches_string("AMAZON MKTP"));
    }

    #[test]
    fn test_rule_condition_amount() {
        let condition = RuleCondition::amount_greater_than(Money::new(dec!(100.0)));
        assert!(condition.matches_amount(&Money::new(dec!(150.0))));
        assert!(!condition.matches_amount(&Money::new(dec!(50.0))));
    }

    #[test]
    fn test_rule_builder() {
        let category_id = Uuid::new_v4();
        let rule = RuleBuilder::new("Amazon Purchases", category_id)
            .description_contains("AMAZON")
            .description_contains("AMZN")
            .or_logic()
            .priority(50)
            .description("Matches Amazon purchases")
            .build();

        assert_eq!(rule.name, "Amazon Purchases");
        assert_eq!(rule.priority, 50);
        assert_eq!(rule.conditions.operator, LogicalOperator::Or);
        assert_eq!(rule.conditions.conditions.len(), 2);
    }
}
