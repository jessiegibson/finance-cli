# src/calculator/

Financial report calculations. Aggregates transactions into reports consumed by the CLI report commands.

## Files

### mod.rs
Module root. Re-exports `PnLReport` and `CashFlowReport` and defines shared helpers such as `aggregate_by_category` for summing transactions grouped by category.

### pnl.rs
Profit and Loss calculation. `PnLReport` holds a `DateRange`, total income, total expenses, net profit, and per-category breakdowns. Separates income categories from expense categories and computes margin.

### cashflow.rs
Cash flow calculation. `CashFlowReport` groups transactions into period buckets (using a `BTreeMap<NaiveDate, Money>`), tracks starting and ending balances, and derives period-over-period deltas.

### metrics.rs
Stateless financial metric helpers. Functions like `average_transaction`, `median_transaction`, and category-share calculations operate on transaction slices without needing a full report struct.
