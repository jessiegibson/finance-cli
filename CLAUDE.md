# CLAUDE.md - Finance CLI Application

## Roadmap Maintenance (REQUIRED)

Whenever a new feature is built, shipped, or moved between status sections in `roadmap.md`, you MUST update the two header lines at the top of `roadmap.md`:

1. `Last updated: YYYY-MM-DD HH:MM TZ` — set to the current date and time in the local timezone. Get the value by running `date "+%Y-%m-%d %H:%M %Z"` in the shell. Do not guess the timestamp.
2. `Last updated by: <Name>` — set to the person (or agent) who made the change. Default to `Jessie Gibson` unless another contributor is explicitly identified.

Scope of "new feature" for this rule:
- Any item moved into the Shipped section
- Any item added to In progress, Near term, Mid term, or Long term
- Any Known issue added or resolved
- Any release added to the release history table

Do not skip this step. A stale `Last updated` field makes the roadmap unreliable, which is worse than having no roadmap at all. If you edit `roadmap.md` for any reason other than a typo fix, refresh both header lines in the same commit.

## Project Overview

**Finance CLI** is a privacy-first personal finance command-line application built in Rust. It provides local-only transaction management, categorization, and tax-ready financial reporting with strong encryption—no cloud, no internet required.

## Quick Start

```bash
# Build the project
cargo build --release

# Run tests
cargo test

# Run with verbose logging
cargo run -- --verbose <command>

# Check code quality
cargo clippy
```

## Architecture

The application follows a **layered architecture**:

```
┌─────────────────────────────────────────────────┐
│           Interface Layer (CLI)                  │
│  clap-based commands: transaction, report,      │
│  category, config, init, status                 │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────┴────────────────────────────┐
│          Business Logic Layer                    │
│  - Categorization (rules + ML engine)           │
│  - Calculator (P&L, CashFlow, Metrics)          │
└──────────┬─────────────────────────┬────────────┘
           │                         │
┌──────────┴──────────┐   ┌──────────┴──────────┐
│    Data Layer       │   │ Infrastructure Layer │
│  - DuckDB database  │   │  - AES-256-GCM       │
│  - CSV/QFX parsers  │   │  - Argon2id KDF      │
│  - Models           │   │  - Tracing logging   │
│  - TOML config      │   │  - Error handling    │
└─────────────────────┘   └─────────────────────┘
```

## Directory Structure

```
finance-cli/
├── src/
│   ├── main.rs              # Entry point with exit codes
│   ├── lib.rs               # Library exports, run() function
│   ├── cli/
│   │   ├── mod.rs           # CLI structure (clap)
│   │   ├── commands/        # Subcommand implementations
│   │   │   ├── transaction.rs
│   │   │   ├── report.rs
│   │   │   ├── category.rs
│   │   │   └── config.rs
│   │   └── output.rs        # Terminal output formatting
│   ├── models/
│   │   ├── transaction.rs   # Transaction with Schedule C support
│   │   ├── category.rs      # Hierarchical categories
│   │   ├── account.rs       # Bank/credit accounts
│   │   ├── rule.rs          # Categorization rules
│   │   └── mod.rs
│   ├── database/
│   │   ├── connection.rs    # Thread-safe DuckDB wrapper
│   │   ├── migrations.rs    # Schema setup
│   │   ├── models.rs        # DB models
│   │   ├── queries.rs       # Repository implementations
│   │   └── mod.rs
│   ├── encryption/
│   │   ├── cipher.rs        # AES-256-GCM operations
│   │   ├── key.rs           # Argon2id + HKDF key derivation
│   │   ├── secure_memory.rs # Zeroizable memory types
│   │   └── mod.rs
│   ├── parsers/
│   │   ├── csv.rs           # CSV parsing (8 banks)
│   │   ├── qfx.rs           # QFX/OFX format
│   │   ├── detect.rs        # Format/institution detection
│   │   └── mod.rs
│   ├── categorization/
│   │   ├── engine.rs        # Main orchestrator
│   │   ├── rules.rs         # Rule matching
│   │   ├── ml.rs            # ML-based categorization
│   │   └── mod.rs
│   ├── calculator/
│   │   ├── pnl.rs           # Profit & Loss
│   │   ├── cashflow.rs      # Cash flow analysis
│   │   ├── metrics.rs       # Financial metrics
│   │   └── mod.rs
│   ├── config/
│   │   ├── settings.rs      # TOML configuration
│   │   └── mod.rs
│   ├── error/
│   │   ├── types.rs         # Error definitions
│   │   └── mod.rs           # User-friendly suggestions
│   └── logging/
│       ├── formatters.rs    # Custom log formatters
│       └── mod.rs
├── benches/
│   ├── parser_bench.rs
│   └── categorization_bench.rs
├── Cargo.toml
└── Cargo.lock
```

## CLI Commands

```
finance [OPTIONS] <COMMAND>

Options:
  --verbose, -v    Enable verbose output
  --quiet, -q      Suppress non-essential output
  --config <PATH>  Custom config file path

Commands:
  transaction (tx)   Manage transactions
    import           Import from CSV/QFX (--account, --dry-run, --no-dedupe)
    list             List transactions (--limit, --year, --month, --uncategorized)
    categorize       Interactive categorization (--limit)
    search           Search transactions

  report             Generate financial reports
    pnl              Profit & Loss statement
    cashflow         Cash flow report
    schedule-c       IRS Schedule C report

  category           Manage categories
    list             List all categories
    add              Add new category
    rules            Manage categorization rules

  config             Configuration management
    show             Display current config
    set              Modify settings

  init               Initialize new database
  status             Show application statistics
```

## Key Dependencies

| Category | Crate | Purpose |
|----------|-------|---------|
| CLI | `clap` 4.4 | Command parsing with derive |
| Database | `duckdb` 1.0 | Embedded analytics database |
| Encryption | `aes-gcm` 0.10 | AES-256-GCM encryption |
| Key Derivation | `argon2` 0.5 | Argon2id password hashing |
| Secure Memory | `zeroize` 1.7, `secrecy` 0.8 | Memory protection |
| Serialization | `serde` 1.0, `toml` 0.8 | Data serialization |
| Parsing | `csv` 1.3, `quick-xml` 0.31 | File format parsing |
| Money | `rust_decimal` 1.33 | Precise decimal arithmetic |
| Dates | `chrono` 0.4 | Date/time handling |
| Errors | `thiserror` 1.0, `anyhow` 1.0 | Error handling |
| Logging | `tracing` 0.1 | Structured logging |
| Terminal | `dialoguer` 0.11, `indicatif` 0.17 | Interactive UI |

## Security Architecture

### Encryption Flow

```
User Password
      ↓
Argon2id (OWASP settings)
  - Memory: 64 MB
  - Iterations: 3
  - Parallelism: 4
      ↓
Master Key (256-bit, never stored)
      ↓
HKDF-SHA256 derives purpose-specific keys:
  ├── "database" → Database encryption key
  ├── "config"   → Config encryption key
  └── "backup"   → Backup encryption key
      ↓
AES-256-GCM (authenticated encryption)
```

### Security Guarantees

- **Local-only**: No network calls, no cloud storage
- **Zero-knowledge**: Master key never persisted
- **Memory protection**: `SecureBytes`/`SecureString` auto-zeroize
- **Integrity**: GCM mode detects tampering
- **Forward secrecy**: Keys zeroized after use

## Supported Banks

CSV and QFX import support for:
- Chase
- Bank of America
- Wealthfront
- Ally
- American Express
- Discover
- Citi
- Capital One

## Data Models

### Transaction
- UUID identifier
- Amount (Decimal), date, description
- Merchant name extraction
- SHA-256 hash for duplicate detection
- IRS Tax Prep Schedule A,C,E line mapping
- Tax deductibility flag
- Categorization confidence score

### Category
- Hierarchical (parent_id support)
- Types: Income, Expense, Personal
- Schedule A, C, & E line mapping
- Tax deductibility flag

### Account
- Types: Checking, Savings, CreditCard, Business variants
- Institution name, last 4 digits

### Rule
- Field matching (description, amount, merchant)
- Logical operators (AND, OR, NOT)
- Target category with confidence/priority

## Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run benchmarks
cargo bench
```

**Test frameworks used:**
- `rstest` - Parametrized tests
- `proptest` - Property-based testing
- `serial_test` - Sequential test execution
- `criterion` - Benchmarking

**Testing patterns:**
- In-memory DuckDB for isolation: `Connection::open_in_memory()`
- 23+ embedded tests throughout codebase

## Code Quality Standards

### Enforced via Cargo.toml

```toml
[lints.rust]
unsafe_code = "forbid"        # No unsafe blocks
unused_imports = "warn"
dead_code = "warn"

[lints.clippy]
unwrap_used = "warn"          # Prefer proper error handling
expect_used = "warn"
panic = "warn"
todo = "warn"
dbg_macro = "warn"
```

### Best Practices

1. **No panics in production code** - Use `Result<T, Error>` everywhere
2. **No `.unwrap()` or `.expect()`** - Handle errors properly
3. **No `unsafe` blocks** - Forbidden at compiler level
4. **Decimal for money** - Never use floats for financial calculations
5. **Zeroize sensitive data** - Use `SecureBytes`/`SecureString`

## Error Handling

Errors are categorized with user-friendly suggestions:

| Category | Example | Suggestion |
|----------|---------|------------|
| Config | Missing config file | "Run `finance init` to create default config" |
| Database | Connection failed | "Check file permissions and path" |
| Encryption | Wrong password | "Verify your password and try again" |
| Parse | Unknown bank format | "Supported banks: Chase, BofA, ..." |
| Validation | Invalid amount | "Amount must be a valid decimal number" |

Exit codes:
- `0` - Success
- `1` - General error
- `2` - Configuration error
- `3` - Database error
- `4` - Encryption error
- `5` - Parse error

## Configuration

Default location: `~/.finance-cli/config.toml`

```toml
[database]
path = "~/.finance-cli/finance.db"

[encryption]
enabled = true

[categories]
default_expense = "Uncategorized"
default_income = "Other Income"

[import]
dedupe = true
```

## Development Workflow

### Adding a New Command

1. Add subcommand to `src/cli/mod.rs`
2. Implement in `src/cli/commands/<name>.rs`
3. Wire up in command dispatch
4. Add tests

### Adding a New Bank Parser

1. Add institution variant to `src/parsers/detect.rs`
2. Implement CSV column mapping in `src/parsers/csv.rs`
3. Add detection heuristics
4. Add test fixtures

### Adding a New Report

1. Create module in `src/calculator/`
2. Implement calculation logic
3. Add CLI subcommand in `src/cli/commands/report.rs`
4. Add output formatting

## Git Workflow

```bash
# Feature branch
git checkout -b feature/your-feature
git push -u origin feature/your-feature

# Commit message format (imperative mood)
git commit -m "Add transaction search by date range

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

## Performance

Release build optimizations (Cargo.toml):
- LTO enabled
- Single codegen unit
- Panic = abort
- Binary stripping

Run benchmarks:
```bash
cargo bench
# HTML report at target/criterion/report/index.html
```

## Common Tasks

### Import transactions
```bash
finance tx import ~/Downloads/chase-2024.csv --account checking
```

### Generate P&L report
```bash
finance report pnl --year 2024
```

### Categorize uncategorized transactions
```bash
finance tx categorize --limit 50
```

### Search transactions
```bash
finance tx search "coffee" --year 2024
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "Database locked" | Close other finance-cli instances |
| "Decryption failed" | Verify password, check file corruption |
| "Unknown bank format" | Check CSV headers match supported banks |
| Build fails on macOS | Install `pkg-config`: `brew install pkg-config` |

## Related Projects

- **Orchestrator repo**: `jessiegibson/agents` - Multi-agent system that coordinates development
- See `/Users/jag/workspace/github.com/jessiegibson/agents/CLAUDE.md` for orchestration context
