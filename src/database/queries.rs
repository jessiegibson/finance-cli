//! Database query repositories.

use super::connection::Connection;
use super::models::{account_type_to_string, category_type_to_string, row_to_account, row_to_category};
use crate::error::{DatabaseError, Error, Result};
use crate::models::{Account, Category, DateRange, Money, Rule, Transaction};
use chrono::NaiveDate;
use uuid::Uuid;

/// Repository for Account operations.
pub struct AccountRepository<'a> {
    conn: &'a Connection,
}

impl<'a> AccountRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Get all accounts.
    pub fn find_all(&self) -> Result<Vec<Account>> {
        self.conn.query_map(
            "SELECT id, name, bank, account_type, last_four_digits, is_active FROM accounts ORDER BY name",
            row_to_account,
        )
    }

    /// Get active accounts.
    pub fn find_active(&self) -> Result<Vec<Account>> {
        self.conn.query_map(
            "SELECT id, name, bank, account_type, last_four_digits, is_active FROM accounts WHERE is_active = TRUE ORDER BY name",
            row_to_account,
        )
    }

    /// Get account by ID.
    pub fn find_by_id(&self, id: Uuid) -> Result<Option<Account>> {
        self.conn.query_row(
            &format!(
                "SELECT id, name, bank, account_type, last_four_digits, is_active FROM accounts WHERE id = '{}'",
                id
            ),
            row_to_account,
        )
    }

    /// Insert a new account.
    pub fn insert(&self, account: &Account) -> Result<()> {
        let sql = format!(
            "INSERT INTO accounts (id, name, bank, account_type, last_four_digits, is_active) VALUES ('{}', '{}', '{}', '{}', {}, {})",
            account.id,
            account.name.replace('\'', "''"),
            account.bank.replace('\'', "''"),
            account_type_to_string(&account.account_type),
            account.last_four_digits.as_ref().map(|s| format!("'{}'", s.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
            account.is_active
        );
        self.conn.execute(&sql)?;
        Ok(())
    }

    /// Update an existing account.
    pub fn update(&self, account: &Account) -> Result<()> {
        let sql = format!(
            "UPDATE accounts SET name = '{}', bank = '{}', account_type = '{}', last_four_digits = {}, is_active = {}, updated_at = CURRENT_TIMESTAMP WHERE id = '{}'",
            account.name.replace('\'', "''"),
            account.bank.replace('\'', "''"),
            account_type_to_string(&account.account_type),
            account.last_four_digits.as_ref().map(|s| format!("'{}'", s.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
            account.is_active,
            account.id
        );
        self.conn.execute(&sql)?;
        Ok(())
    }
}

/// Repository for Category operations.
pub struct CategoryRepository<'a> {
    conn: &'a Connection,
}

impl<'a> CategoryRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Get all categories.
    pub fn find_all(&self) -> Result<Vec<Category>> {
        self.conn.query_map(
            "SELECT id, parent_id, name, description, category_type, schedule_c_line, is_tax_deductible, is_active, sort_order FROM categories ORDER BY sort_order, name",
            row_to_category,
        )
    }

    /// Get active categories.
    pub fn find_active(&self) -> Result<Vec<Category>> {
        self.conn.query_map(
            "SELECT id, parent_id, name, description, category_type, schedule_c_line, is_tax_deductible, is_active, sort_order FROM categories WHERE is_active = TRUE ORDER BY sort_order, name",
            row_to_category,
        )
    }

    /// Get category by ID.
    pub fn find_by_id(&self, id: Uuid) -> Result<Option<Category>> {
        self.conn.query_row(
            &format!(
                "SELECT id, parent_id, name, description, category_type, schedule_c_line, is_tax_deductible, is_active, sort_order FROM categories WHERE id = '{}'",
                id
            ),
            row_to_category,
        )
    }

    /// Get category by name.
    pub fn find_by_name(&self, name: &str) -> Result<Option<Category>> {
        self.conn.query_row(
            &format!(
                "SELECT id, parent_id, name, description, category_type, schedule_c_line, is_tax_deductible, is_active, sort_order FROM categories WHERE name = '{}'",
                name.replace('\'', "''")
            ),
            row_to_category,
        )
    }

    /// Insert a new category.
    pub fn insert(&self, category: &Category) -> Result<()> {
        let sql = format!(
            "INSERT INTO categories (id, parent_id, name, description, category_type, schedule_c_line, is_tax_deductible, is_active, sort_order) VALUES ('{}', {}, '{}', {}, '{}', {}, {}, {}, {})",
            category.id,
            category.parent_id.map(|id| format!("'{}'", id)).unwrap_or_else(|| "NULL".to_string()),
            category.name.replace('\'', "''"),
            category.description.as_ref().map(|s| format!("'{}'", s.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
            category_type_to_string(&category.category_type),
            category.schedule_c_line.as_ref().map(|s| format!("'{}'", s)).unwrap_or_else(|| "NULL".to_string()),
            category.is_tax_deductible,
            category.is_active,
            category.sort_order
        );
        self.conn.execute(&sql)?;
        Ok(())
    }

    /// Insert default categories.
    pub fn insert_defaults(&self) -> Result<()> {
        let defaults = crate::models::category::default_categories();
        for category in defaults {
            // Skip if already exists
            if self.find_by_name(&category.name)?.is_none() {
                self.insert(&category)?;
            }
        }
        Ok(())
    }

    /// Count categories.
    pub fn count(&self) -> Result<i64> {
        let result: Option<i64> = self
            .conn
            .query_row("SELECT COUNT(*) FROM categories", |row| row.get(0))?;
        Ok(result.unwrap_or(0))
    }
}

/// Repository for Transaction operations.
pub struct TransactionRepository<'a> {
    conn: &'a Connection,
}

impl<'a> TransactionRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Get transactions by date range.
    pub fn find_by_date_range(&self, range: &DateRange) -> Result<Vec<Transaction>> {
        // Simplified - would need full row mapping
        let sql = format!(
            "SELECT id, account_id, transaction_date, amount, description FROM transactions WHERE transaction_date BETWEEN '{}' AND '{}' ORDER BY transaction_date DESC",
            range.start, range.end
        );

        // For now, return empty - full implementation would map rows
        Ok(Vec::new())
    }

    /// Check if a transaction hash already exists.
    pub fn hash_exists(&self, hash: &str) -> Result<bool> {
        let result: Option<i64> = self.conn.query_row(
            &format!(
                "SELECT 1 FROM transactions WHERE transaction_hash = '{}'",
                hash.replace('\'', "''")
            ),
            |row| row.get(0),
        )?;
        Ok(result.is_some())
    }

    /// Count transactions.
    pub fn count(&self) -> Result<i64> {
        let result: Option<i64> = self
            .conn
            .query_row("SELECT COUNT(*) FROM transactions", |row| row.get(0))?;
        Ok(result.unwrap_or(0))
    }

    /// Count uncategorized transactions.
    pub fn count_uncategorized(&self) -> Result<i64> {
        let result: Option<i64> = self.conn.query_row(
            "SELECT COUNT(*) FROM transactions WHERE category_id IS NULL",
            |row| row.get(0),
        )?;
        Ok(result.unwrap_or(0))
    }
}

/// Repository for Rule operations.
pub struct RuleRepository<'a> {
    conn: &'a Connection,
}

impl<'a> RuleRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Get all active rules ordered by priority.
    pub fn find_active(&self) -> Result<Vec<Rule>> {
        // Would need full row mapping
        Ok(Vec::new())
    }

    /// Count rules.
    pub fn count(&self) -> Result<i64> {
        let result: Option<i64> = self
            .conn
            .query_row("SELECT COUNT(*) FROM rules", |row| row.get(0))?;
        Ok(result.unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{initialize_test, migrations};
    use crate::models::AccountType;

    #[test]
    fn test_account_crud() {
        let conn = initialize_test().unwrap();
        let repo = AccountRepository::new(&conn);

        let account =
            Account::new("Test Account", "Test Bank", AccountType::Checking).with_last_four("1234");

        repo.insert(&account).unwrap();

        let found = repo.find_by_id(account.id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Account");
    }

    #[test]
    fn test_category_defaults() {
        let conn = initialize_test().unwrap();
        let repo = CategoryRepository::new(&conn);

        repo.insert_defaults().unwrap();

        let count = repo.count().unwrap();
        assert!(count > 0);

        let office = repo.find_by_name("Office Expense").unwrap();
        assert!(office.is_some());
        assert_eq!(office.unwrap().schedule_c_line, Some("L18".to_string()));
    }
}
