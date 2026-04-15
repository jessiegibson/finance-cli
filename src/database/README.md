# src/database/

DuckDB-backed persistence layer. Provides a thread-safe connection wrapper, schema migrations, row-to-model conversions, and repository types per domain entity.

## Files

### mod.rs
Module root. Re-exports `Connection`, `DatabaseConfig`, and the four repositories. Exposes `initialize()`, which opens the DuckDB file and runs pending migrations.

### connection.rs
Thread-safe DuckDB wrapper. `DatabaseConfig` holds the database path and creation flag. `Connection` wraps `duckdb::Connection` in `Arc<Mutex<...>>` and exposes `execute`, `query`, and transaction helpers.

### migrations.rs
Schema management. Tracks applied migrations in a `schema_migrations` table and runs all pending migrations up to `SCHEMA_VERSION`. Creates tables for accounts, categories, transactions, and rules.

### models.rs
Row-to-domain conversions. Functions like `row_to_account`, `row_to_category`, `row_to_transaction`, and `row_to_rule` translate `duckdb::Row` values into strongly typed domain models, plus inverse helpers such as `account_type_to_string`.

### queries.rs
Repository implementations: `AccountRepository`, `CategoryRepository`, `TransactionRepository`, and `RuleRepository`. Each exposes `find_all`, `find_by_id`, `insert`, `update`, `delete`, and domain-specific queries (for example, `find_uncategorized`, `insert_defaults`, `count`).
