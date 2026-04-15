# src/

Root source directory for the Finance CLI library and binary.

## Files

### main.rs
Binary entry point. Initializes logging, invokes `finance_cli::run()`, and maps application errors to process exit codes (0 success, 1 general, 2 config, 3 encryption, 4 I/O, 5 database).

### lib.rs
Library crate root. Declares the top-level modules (`cli`, `calculator`, `categorization`, `config`, `database`, `encryption`, `error`, `logging`, `models`, `parsers`) and exposes the public `run()` function used by `main.rs`.

## Subdirectories

- `calculator/` — Financial report calculations (P&L, cash flow, metrics).
- `categorization/` — Rule-based and ML-assisted transaction classification.
- `cli/` — Clap-based command-line interface and subcommand handlers.
- `config/` — TOML configuration loading and settings management.
- `database/` — DuckDB connection, migrations, queries, and repositories.
- `encryption/` — AES-256-GCM cipher, Argon2id key derivation, secure memory.
- `error/` — Centralized error types and user-facing suggestions.
- `logging/` — Tracing subscriber setup and log formatters.
- `models/` — Domain models (Transaction, Category, Account, Rule).
- `parsers/` — CSV and QFX/OFX file parsers with bank detection.
