//! Financial calculation module.
//!
//! This module provides calculations for financial reports including
//! Profit & Loss, Cash Flow, and Schedule C tax summaries.

pub mod cashflow;
pub mod metrics;
pub mod pnl;

pub use cashflow::CashFlowReport;
pub use pnl::PnLReport;

use crate::models::{DateRange, Money, Transaction};

/// Aggregate transactions by category.
pub fn aggregate_by_category(
    transactions: &[Transaction],
) -> std::collections::HashMap<uuid::Uuid, Money> {
    use std::collections::HashMap;

    let mut totals: HashMap<uuid::Uuid, Money> = HashMap::new();

    for tx in transactions {
        if let Some(cat_id) = tx.category_id {
            let entry = totals.entry(cat_id).or_insert_with(Money::zero);
            *entry = *entry + tx.amount;
        }
    }

    totals
}

/// Calculate total income from transactions.
pub fn total_income(transactions: &[Transaction]) -> Money {
    transactions
        .iter()
        .filter(|tx| tx.amount.is_income())
        .fold(Money::zero(), |acc, tx| acc + tx.amount)
}

/// Calculate total expenses from transactions.
pub fn total_expenses(transactions: &[Transaction]) -> Money {
    transactions
        .iter()
        .filter(|tx| tx.amount.is_expense())
        .fold(Money::zero(), |acc, tx| acc + tx.amount)
}

/// Calculate net (income - expenses).
pub fn net_total(transactions: &[Transaction]) -> Money {
    transactions
        .iter()
        .fold(Money::zero(), |acc, tx| acc + tx.amount)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    fn test_tx(amount: f64) -> Transaction {
        Transaction::new(
            Uuid::new_v4(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            Money::new(rust_decimal::Decimal::from_f64_retain(amount).unwrap()),
            "Test".to_string(),
        )
    }

    #[test]
    fn test_total_income() {
        let txs = vec![test_tx(100.0), test_tx(-50.0), test_tx(200.0)];
        let income = total_income(&txs);
        assert_eq!(income.0, dec!(300.0));
    }

    #[test]
    fn test_total_expenses() {
        let txs = vec![test_tx(100.0), test_tx(-50.0), test_tx(-25.0)];
        let expenses = total_expenses(&txs);
        assert_eq!(expenses.0, dec!(-75.0));
    }

    #[test]
    fn test_net_total() {
        let txs = vec![test_tx(100.0), test_tx(-50.0), test_tx(-25.0)];
        let net = net_total(&txs);
        assert_eq!(net.0, dec!(25.0));
    }
}
