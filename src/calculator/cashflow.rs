//! Cash flow report calculation.

use crate::models::{DateRange, Money, Transaction};
use chrono::NaiveDate;
use std::collections::BTreeMap;

/// A Cash Flow report.
#[derive(Debug)]
pub struct CashFlowReport {
    /// The date range for this report.
    pub date_range: DateRange,
    /// Starting balance (if known).
    pub starting_balance: Option<Money>,
    /// Ending balance.
    pub ending_balance: Option<Money>,
    /// Total inflows.
    pub total_inflows: Money,
    /// Total outflows.
    pub total_outflows: Money,
    /// Net cash flow.
    pub net_cash_flow: Money,
    /// Daily cash flow breakdown.
    pub daily_flow: BTreeMap<NaiveDate, DailyCashFlow>,
}

/// Cash flow for a single day.
#[derive(Debug, Clone)]
pub struct DailyCashFlow {
    pub date: NaiveDate,
    pub inflows: Money,
    pub outflows: Money,
    pub net: Money,
    pub transaction_count: usize,
}

impl CashFlowReport {
    /// Generate a cash flow report from transactions.
    pub fn generate(transactions: &[Transaction], date_range: DateRange) -> Self {
        let mut daily_flow: BTreeMap<NaiveDate, DailyCashFlow> = BTreeMap::new();
        let mut total_inflows = Money::zero();
        let mut total_outflows = Money::zero();

        for tx in transactions {
            if !date_range.contains(tx.transaction_date) {
                continue;
            }

            let entry = daily_flow
                .entry(tx.transaction_date)
                .or_insert_with(|| DailyCashFlow {
                    date: tx.transaction_date,
                    inflows: Money::zero(),
                    outflows: Money::zero(),
                    net: Money::zero(),
                    transaction_count: 0,
                });

            if tx.amount.is_income() {
                entry.inflows += tx.amount;
                total_inflows += tx.amount;
            } else {
                entry.outflows += tx.amount;
                total_outflows += tx.amount;
            }
            entry.net += tx.amount;
            entry.transaction_count += 1;
        }

        let net_cash_flow = total_inflows + total_outflows;

        Self {
            date_range,
            starting_balance: None,
            ending_balance: None,
            total_inflows,
            total_outflows,
            net_cash_flow,
            daily_flow,
        }
    }

    /// Get monthly aggregation.
    pub fn monthly_summary(&self) -> BTreeMap<(i32, u32), MonthlyCashFlow> {
        let mut monthly: BTreeMap<(i32, u32), MonthlyCashFlow> = BTreeMap::new();

        for (date, daily) in &self.daily_flow {
            let key = (date.year(), date.month());
            let entry = monthly.entry(key).or_insert_with(|| MonthlyCashFlow {
                year: date.year(),
                month: date.month(),
                inflows: Money::zero(),
                outflows: Money::zero(),
                net: Money::zero(),
            });

            entry.inflows += daily.inflows;
            entry.outflows += daily.outflows;
            entry.net += daily.net;
        }

        monthly
    }
}

/// Cash flow for a single month.
#[derive(Debug, Clone)]
pub struct MonthlyCashFlow {
    pub year: i32,
    pub month: u32,
    pub inflows: Money,
    pub outflows: Money,
    pub net: Money,
}

use chrono::Datelike;

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    fn test_tx(date: NaiveDate, amount: f64) -> Transaction {
        Transaction::new(
            Uuid::new_v4(),
            date,
            Money::new(rust_decimal::Decimal::from_f64_retain(amount).unwrap()),
            "Test".to_string(),
        )
    }

    #[test]
    fn test_cashflow_report() {
        let transactions = vec![
            test_tx(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), 1000.0),
            test_tx(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), -200.0),
            test_tx(NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(), -300.0),
            test_tx(NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(), 500.0),
        ];

        let date_range = DateRange::year(2024);
        let report = CashFlowReport::generate(&transactions, date_range);

        assert_eq!(report.total_inflows.0, dec!(1500.0));
        assert_eq!(report.total_outflows.0, dec!(-500.0));
        assert_eq!(report.net_cash_flow.0, dec!(1000.0));
        assert_eq!(report.daily_flow.len(), 3);
    }

    #[test]
    fn test_monthly_summary() {
        let transactions = vec![
            test_tx(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), 1000.0),
            test_tx(NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(), -200.0),
            test_tx(NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(), 500.0),
        ];

        let date_range = DateRange::year(2024);
        let report = CashFlowReport::generate(&transactions, date_range);
        let monthly = report.monthly_summary();

        assert_eq!(monthly.len(), 2);
        assert_eq!(monthly.get(&(2024, 1)).unwrap().net.0, dec!(800.0));
        assert_eq!(monthly.get(&(2024, 2)).unwrap().net.0, dec!(500.0));
    }
}
