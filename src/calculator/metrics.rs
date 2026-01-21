//! Financial metrics calculations.

use crate::models::{Money, Transaction};

/// Calculate average transaction amount.
pub fn average_transaction(transactions: &[Transaction]) -> Money {
    if transactions.is_empty() {
        return Money::zero();
    }

    let total = transactions
        .iter()
        .fold(Money::zero(), |acc, tx| acc + tx.amount);

    Money::new(total.0 / rust_decimal::Decimal::from(transactions.len()))
}

/// Calculate average monthly spending.
pub fn average_monthly_expenses(transactions: &[Transaction]) -> Money {
    let expenses: Vec<_> = transactions
        .iter()
        .filter(|tx| tx.amount.is_expense())
        .collect();

    if expenses.is_empty() {
        return Money::zero();
    }

    // Find date range
    let min_date = expenses.iter().map(|tx| tx.transaction_date).min();
    let max_date = expenses.iter().map(|tx| tx.transaction_date).max();

    if let (Some(min), Some(max)) = (min_date, max_date) {
        let months = (max.year() - min.year()) * 12 + (max.month() as i32 - min.month() as i32) + 1;
        let months = std::cmp::max(1, months);

        let total = expenses.iter().fold(Money::zero(), |acc, tx| acc + tx.amount);

        Money::new(total.0 / rust_decimal::Decimal::from(months))
    } else {
        Money::zero()
    }
}

/// Find the largest expense.
pub fn largest_expense(transactions: &[Transaction]) -> Option<&Transaction> {
    transactions
        .iter()
        .filter(|tx| tx.amount.is_expense())
        .min_by(|a, b| a.amount.0.cmp(&b.amount.0)) // Most negative = largest expense
}

/// Find the largest income.
pub fn largest_income(transactions: &[Transaction]) -> Option<&Transaction> {
    transactions
        .iter()
        .filter(|tx| tx.amount.is_income())
        .max_by(|a, b| a.amount.0.cmp(&b.amount.0))
}

/// Count transactions by type.
pub fn transaction_counts(transactions: &[Transaction]) -> TransactionCounts {
    let income_count = transactions.iter().filter(|tx| tx.amount.is_income()).count();
    let expense_count = transactions.iter().filter(|tx| tx.amount.is_expense()).count();
    let categorized_count = transactions.iter().filter(|tx| tx.category_id.is_some()).count();

    TransactionCounts {
        total: transactions.len(),
        income: income_count,
        expense: expense_count,
        categorized: categorized_count,
        uncategorized: transactions.len() - categorized_count,
    }
}

/// Transaction count breakdown.
#[derive(Debug, Clone)]
pub struct TransactionCounts {
    pub total: usize,
    pub income: usize,
    pub expense: usize,
    pub categorized: usize,
    pub uncategorized: usize,
}

use chrono::Datelike;

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
    fn test_average_transaction() {
        let txs = vec![test_tx(100.0), test_tx(200.0), test_tx(-50.0)];
        let avg = average_transaction(&txs);
        // (100 + 200 - 50) / 3 = 83.33...
        assert!(avg.0 > dec!(83.0) && avg.0 < dec!(84.0));
    }

    #[test]
    fn test_transaction_counts() {
        let mut txs = vec![test_tx(100.0), test_tx(-50.0), test_tx(-25.0)];
        txs[0].category_id = Some(Uuid::new_v4());

        let counts = transaction_counts(&txs);
        assert_eq!(counts.total, 3);
        assert_eq!(counts.income, 1);
        assert_eq!(counts.expense, 2);
        assert_eq!(counts.categorized, 1);
        assert_eq!(counts.uncategorized, 2);
    }
}
