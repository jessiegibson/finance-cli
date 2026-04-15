# Finance CLI Roadmap

Last updated: 2026-04-14 17:24 EDT
Last updated by: Jessie Gibson

Use this file to track the overall development of the Finance CLI application. Move items between sections as work progresses. Keep entries short, link to issues or PRs where possible, and record completion dates.

## Legend

- [x] Shipped
- [~] In progress
- [ ] Planned
- [?] Under evaluation

## Vision

A privacy-first personal finance CLI for freelancers and small business owners. Local-only, encrypted, tax-ready. No cloud, no telemetry, no network calls.

## Shipped

### Core infrastructure
- [x] Layered architecture (CLI, business logic, data, infrastructure)
- [x] `clap`-based CLI with global `--verbose`, `--quiet`, `--config` flags
- [x] Thread-safe DuckDB connection wrapper with `Arc<Mutex<...>>`
- [x] Schema migrations with version tracking table
- [x] TOML configuration loading under `~/.finance-cli/config.toml`
- [x] `thiserror`-based error taxonomy with exit codes and user suggestions
- [x] `tracing`-based structured logging with `EnvFilter`

### Security
- [x] AES-256-GCM authenticated encryption with per-call random nonce
- [x] Argon2id password hashing with OWASP parameters (64 MB, 3 iter, 4 parallel)
- [x] HKDF-SHA256 domain separation for database, config, and backup keys
- [x] `SecureString` and `SecureBytes` with automatic zeroization
- [x] `unsafe_code = "forbid"` at crate level

### Data import
- [x] CSV parser with institution detection
- [x] QFX/OFX parser using `quick-xml`
- [x] IIF (QuickBooks) parser with split-transaction support; compilation errors fixed (2026-04-14)
- [x] Support for 8 banks: Chase, Bank of America, Wealthfront, Ally, American Express, Discover, Citi, Capital One
- [x] SHA-256 dedupe hash on transactions
- [x] Dry-run and no-dedupe import flags
- [x] `finance tx import <file> --account <name>` CLI command end-to-end: parse → dedup → persist (2026-04-14)
- [x] `finance tx list` CLI command with year/month/uncategorized filters (2026-04-14)

### Transactions and categorization
- [x] Transaction domain model with Schedule C line mapping and tax-deductible flag
- [x] Hierarchical category model with income/expense/personal types
- [x] Rule model with AND/OR/NOT conditions over description, amount, and merchant
- [x] `CategorizationEngine` orchestrator with confidence scoring
- [x] Default category seeding on `finance init` (44 categories: Schedule C, E, A + Personal)
- [x] `TransactionRepository::insert()`, `update_category()`, `find_uncategorized()` (2026-04-14)
- [x] `RuleRepository::find_active()` and `insert()` wired to database (2026-04-14)
- [x] `row_to_rule()` DB converter with JSON conditions deserialization (2026-04-14)
- [x] Interactive one-by-one `finance tx categorize` with `dialoguer` Select prompt (2026-04-14)
- [x] Inline rule creation during `tx categorize` — saves auto-categorization rules on the spot (2026-04-14)
- [x] `finance category rules` command: list active rules per category with conditions and effectiveness count (2026-04-14)

### Reporting
- [x] Profit and Loss report with per-category breakdown
- [x] Cash flow report grouped by period
- [x] Schedule C tax report command surface
- [x] Table, CSV, and JSON output formats for reports

### Testing and quality
- [x] Embedded unit tests across 23+ modules
- [x] In-memory DuckDB fixtures for isolated tests
- [x] `rstest`, `proptest`, `serial_test` wired into `Cargo.toml`
- [x] Criterion benchmark scaffolding for parsers and categorization
- [x] Realistic per-bank parser benchmarks with 100-row fixtures for all 8 institutions and a QFX fixture (2026-04-08)
- [x] Clippy lints for `unwrap_used`, `expect_used`, `panic`, `todo`, `dbg_macro`

## In progress

- [~] README.md files per source directory (documentation pass)
- [~] Roadmap tracking doc (this file)

## Near term (next 1 to 2 releases)

### Parsing and import
- [ ] Robust error reporting for malformed CSV rows with line numbers
- [ ] Incremental import that skips rows already hashed in the database
- [ ] Support for Plaid-exported CSVs as a ninth bank format

### Categorization
- [ ] Real ML categorizer to replace the `MlCategorizer` placeholder (for example, a TF-IDF plus logistic regression baseline)
- [ ] Rule priority resolution when multiple rules match a single transaction
- [ ] Bulk re-categorization command to apply new rules to historical data
- [ ] Confidence threshold configuration in `config.toml`

### Reporting
- [ ] Year-over-year comparison for P&L
- [ ] Quarterly estimated tax summary (Schedule SE inputs)
- [ ] Export Schedule C report to PDF
- [ ] Expense category trend chart (ASCII sparkline in the terminal)

### Security and reliability
- [ ] Encrypted database file format on disk (AES-256-GCM wrapper around DuckDB)
- [ ] Backup and restore commands with encryption
- [ ] Recovery phrase flow for password reset
- [ ] Property-based tests for cipher round-trip and key derivation determinism

### Developer experience
- [ ] Shell completion scripts for bash, zsh, and fish
- [ ] `finance doctor` diagnostic command for common setup issues
- [ ] GitHub Actions CI with build, test, clippy, and audit steps
- [ ] `cargo-deny` configuration for license and advisory checks

## Mid term

- [ ] Multi-account reconciliation reports
- [ ] Budget definition and variance tracking
- [ ] Recurring transaction detection and forecasting
- [ ] Multi-currency support with exchange rate snapshots
- [ ] Receipt attachment storage (encrypted blob in the database)
- [ ] TUI mode with `ratatui` for interactive browsing

## Long term / under evaluation

- [?] Optional sync layer over user-supplied storage (for example, encrypted blobs on a self-hosted S3-compatible endpoint)
- [?] Plugin system for community-contributed bank parsers
- [?] Mobile companion read-only viewer
- [?] Double-entry accounting mode for businesses filing Schedule C as an LLC

## Known issues and tech debt

- [ ] `src/logging/formatters.rs` is a stub file with only a doc comment
- [ ] `MlCategorizer` is a placeholder returning no suggestions
- [ ] Categorization benchmarks still run placeholder `1 + 1` loops (parser benchmarks resolved 2026-04-08)
- [ ] Schema version is hardcoded at 1 with no downgrade path
- [ ] No integration test suite covering the full import-to-report flow

## Release history

| Version | Date | Highlights |
| --- | --- | --- |
| 0.1.0 | TBD | Initial private build: import, categorize, P&L, cash flow, Schedule C |

## Notes

When closing a roadmap item, move the entry to the Shipped section and add a line to the release history table if the change belongs to a tagged release. Link PRs in brackets next to the checkbox when available.
