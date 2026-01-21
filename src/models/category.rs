//! Category model for transaction classification.

use super::{Entity, EntityMetadata};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A category for classifying transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// Unique identifier
    pub id: Uuid,

    /// Parent category for hierarchy (optional)
    pub parent_id: Option<Uuid>,

    /// Display name
    pub name: String,

    /// Description
    pub description: Option<String>,

    /// Type of category (income, expense, personal)
    pub category_type: CategoryType,

    /// Default Schedule C line mapping
    pub schedule_c_line: Option<String>,

    /// Whether expenses in this category are tax deductible
    pub is_tax_deductible: bool,

    /// Whether this category is active
    pub is_active: bool,

    /// Display order within parent
    pub sort_order: i32,

    /// Entity metadata
    #[serde(flatten)]
    pub metadata: EntityMetadata,
}

/// Type of category.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CategoryType {
    Income,
    Expense,
    Personal,
}

impl Category {
    /// Create a new category.
    pub fn new(name: impl Into<String>, category_type: CategoryType) -> Self {
        Self {
            id: Uuid::new_v4(),
            parent_id: None,
            name: name.into(),
            description: None,
            category_type,
            schedule_c_line: None,
            is_tax_deductible: false,
            is_active: true,
            sort_order: 100,
            metadata: EntityMetadata::new(),
        }
    }

    /// Create a new expense category.
    pub fn expense(name: impl Into<String>) -> Self {
        Self::new(name, CategoryType::Expense)
    }

    /// Create a new income category.
    pub fn income(name: impl Into<String>) -> Self {
        Self::new(name, CategoryType::Income)
    }

    /// Create a new personal category.
    pub fn personal(name: impl Into<String>) -> Self {
        Self::new(name, CategoryType::Personal)
    }

    /// Set the parent category.
    pub fn with_parent(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Set the Schedule C line mapping.
    pub fn with_schedule_c(mut self, line: impl Into<String>) -> Self {
        self.schedule_c_line = Some(line.into());
        self.is_tax_deductible = true;
        self
    }

    /// Set the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the sort order.
    pub fn with_sort_order(mut self, order: i32) -> Self {
        self.sort_order = order;
        self
    }

    /// Check if this is a top-level category.
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    /// Deactivate the category.
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.metadata.touch();
    }

    /// Reactivate the category.
    pub fn activate(&mut self) {
        self.is_active = true;
        self.metadata.touch();
    }
}

impl Entity for Category {
    fn id(&self) -> Uuid {
        self.id
    }

    fn is_new(&self) -> bool {
        self.metadata.created_at == self.metadata.updated_at
    }
}

/// Default categories for initial setup.
pub fn default_categories() -> Vec<Category> {
    vec![
        // Income categories
        Category::income("Business Income")
            .with_description("Income from business activities")
            .with_sort_order(1),
        Category::income("Freelance Income")
            .with_description("Freelance and consulting income")
            .with_sort_order(2),
        Category::income("Other Income")
            .with_description("Other business income")
            .with_sort_order(3),
        // Expense categories with Schedule C mappings
        Category::expense("Advertising")
            .with_schedule_c("L8")
            .with_description("Advertising and marketing expenses")
            .with_sort_order(10),
        Category::expense("Car & Truck")
            .with_schedule_c("L9")
            .with_description("Vehicle expenses for business use")
            .with_sort_order(11),
        Category::expense("Commissions & Fees")
            .with_schedule_c("L10")
            .with_description("Commissions and fees paid")
            .with_sort_order(12),
        Category::expense("Contract Labor")
            .with_schedule_c("L11")
            .with_description("Payments to contractors")
            .with_sort_order(13),
        Category::expense("Insurance")
            .with_schedule_c("L15")
            .with_description("Business insurance premiums")
            .with_sort_order(14),
        Category::expense("Legal & Professional")
            .with_schedule_c("L17")
            .with_description("Legal and professional services")
            .with_sort_order(15),
        Category::expense("Office Expense")
            .with_schedule_c("L18")
            .with_description("Office supplies and expenses")
            .with_sort_order(16),
        Category::expense("Rent or Lease")
            .with_schedule_c("L20b")
            .with_description("Rent for business property")
            .with_sort_order(17),
        Category::expense("Supplies")
            .with_schedule_c("L22")
            .with_description("Supplies used in business")
            .with_sort_order(18),
        Category::expense("Travel")
            .with_schedule_c("L24a")
            .with_description("Business travel expenses")
            .with_sort_order(19),
        Category::expense("Meals")
            .with_schedule_c("L24b")
            .with_description("Business meals (50% deductible)")
            .with_sort_order(20),
        Category::expense("Utilities")
            .with_schedule_c("L25")
            .with_description("Utilities for business property")
            .with_sort_order(21),
        Category::expense("Other Expenses")
            .with_schedule_c("L27a")
            .with_description("Other deductible business expenses")
            .with_sort_order(22),
        // Personal categories
        Category::personal("Personal")
            .with_description("Personal non-business expenses")
            .with_sort_order(50),
        Category::personal("Transfer")
            .with_description("Transfers between accounts")
            .with_sort_order(51),
        Category::personal("Uncategorized")
            .with_description("Transactions not yet categorized")
            .with_sort_order(99),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_creation() {
        let cat = Category::expense("Office Supplies")
            .with_schedule_c("L18")
            .with_description("Business office supplies");

        assert_eq!(cat.name, "Office Supplies");
        assert_eq!(cat.category_type, CategoryType::Expense);
        assert_eq!(cat.schedule_c_line, Some("L18".to_string()));
        assert!(cat.is_tax_deductible);
        assert!(cat.is_active);
    }

    #[test]
    fn test_default_categories() {
        let categories = default_categories();
        assert!(!categories.is_empty());

        let expense_count = categories
            .iter()
            .filter(|c| c.category_type == CategoryType::Expense)
            .count();
        assert!(expense_count > 0);
    }
}
