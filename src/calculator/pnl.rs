//! Profit and Loss report calculation.

use crate::models::{Category, DateRange, Money, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

/// A Profit and Loss report.
#[derive(Debug)]
pub struct PnLReport {
    /// The date range for this report.
    pub date_range: DateRange,
    /// Total income.
    pub total_income: Money,
    /// Total expenses.
    pub total_expenses: Money,
    /// Net profit (or loss if negative).
    pub net_profit: Money,
    /// Income breakdown by category.
    pub income_by_category: HashMap<Uuid, CategoryTotal>,
    /// Expense breakdown by category.
    pub expenses_by_category: HashMap<Uuid, CategoryTotal>,
}

/// Total for a single category.
#[derive(Debug, Clone)]
pub struct CategoryTotal {
    pub category_id: Uuid,
    pub category_name: String,
    pub total: Money,
    pub transaction_count: usize,
}

impl PnLReport {
    /// Generate a P&L report from transactions.
    pub fn generate(
        transactions: &[Transaction],
        categories: &[Category],
        date_range: DateRange,
    ) -> Self {
        let category_map: HashMap<Uuid, &Category> =
            categories.iter().map(|c| (c.id, c)).collect();

        let mut income_by_category: HashMap<Uuid, CategoryTotal> = HashMap::new();
        let mut expenses_by_category: HashMap<Uuid, CategoryTotal> = HashMap::new();
        let mut total_income = Money::zero();
        let mut total_expenses = Money::zero();

        for tx in transactions {
            if !date_range.contains(tx.transaction_date) {
                continue;
            }

            if let Some(cat_id) = tx.category_id {
                let category_name = category_map
                    .get(&cat_id)
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());

                if tx.amount.is_income() {
                    total_income += tx.amount;
                    let entry = income_by_category.entry(cat_id).or_insert(CategoryTotal {
                        category_id: cat_id,
                        category_name: category_name.clone(),
                        total: Money::zero(),
                        transaction_count: 0,
                    });
                    entry.total += tx.amount;
                    entry.transaction_count += 1;
                } else {
                    total_expenses += tx.amount;
                    let entry = expenses_by_category.entry(cat_id).or_insert(CategoryTotal {
                        category_id: cat_id,
                        category_name,
                        total: Money::zero(),
                        transaction_count: 0,
                    });
                    entry.total += tx.amount;
                    entry.transaction_count += 1;
                }
            }
        }

        let net_profit = total_income + total_expenses; // expenses are negative

        Self {
            date_range,
            total_income,
            total_expenses,
            net_profit,
            income_by_category,
            expenses_by_category,
        }
    }

    /// Check if this is a profitable period.
    pub fn is_profitable(&self) -> bool {
        self.net_profit.is_income()
    }

    /// Get income categories sorted by amount (descending).
    pub fn income_sorted(&self) -> Vec<&CategoryTotal> {
        let mut items: Vec<_> = self.income_by_category.values().collect();
        items.sort_by(|a, b| b.total.0.cmp(&a.total.0));
        items
    }

    /// Get expense categories sorted by amount (ascending - most negative first).
    pub fn expenses_sorted(&self) -> Vec<&CategoryTotal> {
        let mut items: Vec<_> = self.expenses_by_category.values().collect();
        items.sort_by(|a, b| a.total.0.cmp(&b.total.0));
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CategoryType;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    fn test_tx(amount: f64, category_id: Uuid) -> Transaction {
        let mut tx = Transaction::new(
            Uuid::new_v4(),
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            Money::new(rust_decimal::Decimal::from_f64_retain(amount).unwrap()),
            "Test".to_string(),
        );
        tx.category_id = Some(category_id);
        tx
    }

    #[test]
    fn test_pnl_report() {
        let income_cat = Category::income("Business Income");
        let expense_cat = Category::expense("Office Supplies");

        let transactions = vec![
            test_tx(1000.0, income_cat.id),
            test_tx(500.0, income_cat.id),
            test_tx(-200.0, expense_cat.id),
            test_tx(-100.0, expense_cat.id),
        ];

        let categories = vec![income_cat, expense_cat];
        let date_range = DateRange::year(2024);

        let report = PnLReport::generate(&transactions, &categories, date_range);

        assert_eq!(report.total_income.0, dec!(1500.0));
        assert_eq!(report.total_expenses.0, dec!(-300.0));
        assert_eq!(report.net_profit.0, dec!(1200.0));
        assert!(report.is_profitable());
    }
}
