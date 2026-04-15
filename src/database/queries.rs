//! Database query repositories.

use super::connection::Connection;
use serde_json;
use super::models::{account_type_to_string, category_type_to_string, row_to_account, row_to_category, row_to_rule, row_to_transaction};
use crate::error::Result;
use crate::models::{Account, Category, DateRange, Rule, Transaction};
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

    /// Get account by name (case-insensitive).
    pub fn find_by_name(&self, name: &str) -> Result<Option<Account>> {
        self.conn.query_row(
            &format!(
                "SELECT id, name, bank, account_type, last_four_digits, is_active \
                 FROM accounts WHERE LOWER(name) = LOWER('{}')",
                name.replace('\'', "''")
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
        let sql = format!(
            "SELECT id, account_id, category_id, import_batch_id, \
             CAST(transaction_date AS VARCHAR), CAST(amount AS VARCHAR), \
             description, raw_category, merchant_name, location, \
             reference_number, transaction_hash, schedule_c_line, \
             is_business_expense, is_tax_deductible, is_recurring, \
             expense_type, categorized_by, confidence_score \
             FROM transactions \
             WHERE transaction_date >= '{}' AND transaction_date <= '{}' \
             ORDER BY transaction_date ASC",
            range.start, range.end
        );
        self.conn.query_map(&sql, row_to_transaction)
    }

    /// Get all transactions (no date filter).
    pub fn find_all(&self) -> Result<Vec<Transaction>> {
        let sql = "SELECT id, account_id, category_id, import_batch_id, \
             CAST(transaction_date AS VARCHAR), CAST(amount AS VARCHAR), \
             description, raw_category, merchant_name, location, \
             reference_number, transaction_hash, schedule_c_line, \
             is_business_expense, is_tax_deductible, is_recurring, \
             expense_type, categorized_by, confidence_score \
             FROM transactions ORDER BY transaction_date ASC";
        self.conn.query_map(sql, row_to_transaction)
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

    /// Insert a new transaction.
    pub fn insert(&self, tx: &Transaction) -> Result<()> {
        let categorized_by_str = tx.categorized_by.as_ref().map(|c| match c {
            crate::models::CategorizedBy::Rule => "rule",
            crate::models::CategorizedBy::Manual => "manual",
            crate::models::CategorizedBy::Default => "default",
            crate::models::CategorizedBy::Ml => "ml",
        });

        let sql = format!(
            "INSERT INTO transactions \
             (id, account_id, category_id, import_batch_id, transaction_date, amount, \
             description, raw_category, merchant_name, location, reference_number, \
             transaction_hash, schedule_c_line, is_business_expense, is_tax_deductible, \
             is_recurring, expense_type, categorized_by, confidence_score) \
             VALUES ('{}', '{}', {}, {}, '{}', {}, '{}', {}, {}, {}, {}, '{}', {}, {}, {}, {}, {}, {}, {})",
            tx.id,
            tx.account_id,
            tx.category_id.map(|id| format!("'{}'", id)).unwrap_or_else(|| "NULL".to_string()),
            tx.import_batch_id.map(|id| format!("'{}'", id)).unwrap_or_else(|| "NULL".to_string()),
            tx.transaction_date,
            tx.amount.0,
            tx.description.replace('\'', "''"),
            tx.raw_category.as_ref().map(|s| format!("'{}'", s.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
            tx.merchant_name.as_ref().map(|s| format!("'{}'", s.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
            tx.location.as_ref().map(|s| format!("'{}'", s.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
            tx.reference_number.as_ref().map(|s| format!("'{}'", s.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
            tx.transaction_hash.replace('\'', "''"),
            tx.schedule_c_line.as_ref().map(|s| format!("'{}'", s.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
            tx.is_business_expense,
            tx.is_tax_deductible,
            tx.is_recurring,
            tx.expense_type.as_ref().map(|s| format!("'{}'", s.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
            categorized_by_str.map(|s| format!("'{}'", s)).unwrap_or_else(|| "NULL".to_string()),
            tx.confidence_score.map(|s| s.to_string()).unwrap_or_else(|| "NULL".to_string()),
        );
        self.conn.execute(&sql)?;
        Ok(())
    }

    /// Get uncategorized transactions (no category assigned), most recent first.
    pub fn find_uncategorized(&self, limit: usize) -> Result<Vec<Transaction>> {
        let sql = format!(
            "SELECT id, account_id, category_id, import_batch_id, \
             CAST(transaction_date AS VARCHAR), CAST(amount AS VARCHAR), \
             description, raw_category, merchant_name, location, \
             reference_number, transaction_hash, schedule_c_line, \
             is_business_expense, is_tax_deductible, is_recurring, \
             expense_type, categorized_by, confidence_score \
             FROM transactions WHERE category_id IS NULL \
             ORDER BY transaction_date DESC LIMIT {}",
            limit
        );
        self.conn.query_map(&sql, row_to_transaction)
    }

    /// Update the category assignment for a transaction.
    pub fn update_category(
        &self,
        transaction_id: Uuid,
        category_id: Uuid,
        confidence: f64,
    ) -> Result<()> {
        let sql = format!(
            "UPDATE transactions SET category_id = '{}', categorized_by = 'manual', \
             confidence_score = {}, updated_at = CURRENT_TIMESTAMP WHERE id = '{}'",
            category_id, confidence, transaction_id
        );
        self.conn.execute(&sql)?;
        Ok(())
    }

    /// Update category with a specific categorized_by method.
    pub fn update_category_with_method(
        &self,
        transaction_id: Uuid,
        category_id: Uuid,
        confidence: f64,
        method: &str,
    ) -> Result<()> {
        let sql = format!(
            "UPDATE transactions SET category_id = '{}', categorized_by = '{}', \
             confidence_score = {}, updated_at = CURRENT_TIMESTAMP WHERE id = '{}'",
            category_id,
            method.replace('\'', "''"),
            confidence,
            transaction_id
        );
        self.conn.execute(&sql)?;
        Ok(())
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
        self.conn.query_map(
            "SELECT id, target_category_id, name, description, priority, \
             CAST(conditions AS VARCHAR), is_active, effectiveness_count \
             FROM rules WHERE is_active = TRUE ORDER BY priority ASC",
            row_to_rule,
        )
    }

    /// Insert a new rule.
    pub fn insert(&self, rule: &Rule) -> Result<()> {
        let conditions_json = serde_json::to_string(&rule.conditions)
            .map_err(|e| crate::error::Error::Internal(e.to_string()))?;
        let sql = format!(
            "INSERT INTO rules (id, target_category_id, name, description, priority, conditions, is_active) \
             VALUES ('{}', '{}', '{}', {}, {}, '{}', {})",
            rule.id,
            rule.target_category_id,
            rule.name.replace('\'', "''"),
            rule.description
                .as_ref()
                .map(|s| format!("'{}'", s.replace('\'', "''")))
                .unwrap_or_else(|| "NULL".to_string()),
            rule.priority,
            conditions_json.replace('\'', "''"),
            rule.is_active,
        );
        self.conn.execute(&sql)?;
        Ok(())
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
    use crate::database::initialize_test;
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
