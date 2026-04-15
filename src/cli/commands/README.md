# src/cli/commands/

Subcommand handler implementations. Each file defines a clap `Args`/`Subcommand` pair and a `handle_*` function invoked by the top-level dispatcher.

## Files

### mod.rs
Re-exports every command handler and defines `handle_init` (creates data directories and seeds default categories) and `handle_status` (prints database statistics).

### transaction.rs
Handles `finance transaction` (alias `tx`) with four actions:
- `import` — Parses a CSV/QFX file with `--account`, `--dry-run`, and `--no-dedupe` flags, then persists rows via `TransactionRepository`.
- `list` — Lists transactions filtered by `--limit`, `--year`, `--month`, or `--uncategorized`.
- `categorize` — Interactive categorization loop for uncategorized rows.
- `search` — Full-text search across transaction descriptions.

### report.rs
Handles `finance report` with three report types:
- `pnl` — Profit and Loss statement for a given year or date range.
- `cashflow` — Cash flow statement grouped by period.
- `schedule-c` — IRS Schedule C tax summary.
Each supports `--format` (table/csv/json) and `--output` to write to a file.

### category.rs
Handles `finance category` actions: `list`, `create`, `update`, `delete`, and `rules` (for managing categorization rules). Uses `CategoryRepository` for persistence.

### config.rs
Handles `finance config` actions: `show` (print current TOML config), `set` (update a key/value pair), and `path` (display the config file location).
