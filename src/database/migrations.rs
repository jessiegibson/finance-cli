//! Database migrations for schema management.

use super::connection::Connection;
use crate::error::{DatabaseError, Error, Result};

/// Current schema version.
pub const SCHEMA_VERSION: i32 = 1;

/// Run all pending migrations.
pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Create migrations tracking table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )?;

    // Get current version
    let current_version = get_current_version(conn)?;

    // Run migrations
    if current_version < 1 {
        migrate_v1(conn)?;
    }

    Ok(())
}

/// Get the current schema version.
fn get_current_version(conn: &Connection) -> Result<i32> {
    let result: Option<i32> = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        |row| row.get(0),
    )?;

    Ok(result.unwrap_or(0))
}

/// Migration v1: Initial schema.
fn migrate_v1(conn: &Connection) -> Result<()> {
    tracing::info!("Running migration v1: Initial schema");

    conn.execute_batch(
        r#"
        -- Accounts table
        CREATE TABLE IF NOT EXISTS accounts (
            id VARCHAR PRIMARY KEY,
            name VARCHAR NOT NULL,
            bank VARCHAR NOT NULL,
            account_type VARCHAR NOT NULL,
            last_four_digits VARCHAR,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        -- Categories table
        CREATE TABLE IF NOT EXISTS categories (
            id VARCHAR PRIMARY KEY,
            parent_id VARCHAR REFERENCES categories(id),
            name VARCHAR NOT NULL UNIQUE,
            description VARCHAR,
            category_type VARCHAR NOT NULL,
            schedule_c_line VARCHAR,
            is_tax_deductible BOOLEAN NOT NULL DEFAULT FALSE,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            sort_order INTEGER NOT NULL DEFAULT 100,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        -- Transactions table
        CREATE TABLE IF NOT EXISTS transactions (
            id VARCHAR PRIMARY KEY,
            account_id VARCHAR NOT NULL REFERENCES accounts(id),
            category_id VARCHAR REFERENCES categories(id),
            import_batch_id VARCHAR,
            transaction_date DATE NOT NULL,
            amount DECIMAL(12,2) NOT NULL,
            description VARCHAR NOT NULL,
            raw_category VARCHAR,
            merchant_name VARCHAR,
            location VARCHAR,
            reference_number VARCHAR,
            transaction_hash VARCHAR NOT NULL UNIQUE,
            schedule_c_line VARCHAR,
            is_business_expense BOOLEAN NOT NULL DEFAULT FALSE,
            is_tax_deductible BOOLEAN NOT NULL DEFAULT FALSE,
            is_recurring BOOLEAN NOT NULL DEFAULT FALSE,
            expense_type VARCHAR,
            categorized_by VARCHAR,
            confidence_score DECIMAL(3,2),
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        -- Rules table
        CREATE TABLE IF NOT EXISTS rules (
            id VARCHAR PRIMARY KEY,
            target_category_id VARCHAR NOT NULL REFERENCES categories(id),
            name VARCHAR NOT NULL,
            description VARCHAR,
            priority INTEGER NOT NULL DEFAULT 100,
            conditions JSON NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            effectiveness_count INTEGER NOT NULL DEFAULT 0,
            last_applied_at TIMESTAMP,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        -- Import batches table
        CREATE TABLE IF NOT EXISTS import_batches (
            id VARCHAR PRIMARY KEY,
            filename VARCHAR NOT NULL,
            file_type VARCHAR NOT NULL,
            institution VARCHAR NOT NULL,
            transaction_count INTEGER NOT NULL DEFAULT 0,
            duplicate_count INTEGER NOT NULL DEFAULT 0,
            error_count INTEGER NOT NULL DEFAULT 0,
            status VARCHAR NOT NULL DEFAULT 'started',
            summary JSON,
            imported_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        -- User preferences table
        CREATE TABLE IF NOT EXISTS user_preferences (
            id VARCHAR PRIMARY KEY,
            preference_type VARCHAR NOT NULL,
            settings JSON NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        -- Indexes for common queries
        CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(transaction_date);
        CREATE INDEX IF NOT EXISTS idx_transactions_account ON transactions(account_id);
        CREATE INDEX IF NOT EXISTS idx_transactions_category ON transactions(category_id);
        CREATE INDEX IF NOT EXISTS idx_transactions_hash ON transactions(transaction_hash);
        CREATE INDEX IF NOT EXISTS idx_transactions_business ON transactions(is_business_expense);
        CREATE INDEX IF NOT EXISTS idx_categories_type ON categories(category_type);
        CREATE INDEX IF NOT EXISTS idx_rules_category ON rules(target_category_id);
        CREATE INDEX IF NOT EXISTS idx_rules_priority ON rules(priority);

        -- Record migration
        INSERT INTO schema_migrations (version) VALUES (1);
        "#,
    )?;

    tracing::info!("Migration v1 complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_migrations() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Verify tables exist
        let tables: Vec<String> = conn
            .query_map(
                "SELECT table_name FROM information_schema.tables WHERE table_schema = 'main'",
                |row| row.get(0),
            )
            .unwrap();

        assert!(tables.contains(&"accounts".to_string()));
        assert!(tables.contains(&"categories".to_string()));
        assert!(tables.contains(&"transactions".to_string()));
        assert!(tables.contains(&"rules".to_string()));
    }

    #[test]
    fn test_migrations_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap(); // Should not fail
    }
}
