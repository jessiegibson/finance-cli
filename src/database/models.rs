//! Database model conversions.
//!
//! This module provides conversions between database rows and domain models.

use crate::models::{
    Account, AccountType, Category, CategoryType, Money, Rule, RuleConditions, Transaction,
};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

/// Convert a database row to an Account.
pub fn row_to_account(row: &duckdb::Row<'_>) -> Result<Account, duckdb::Error> {
    let id: String = row.get(0)?;
    let name: String = row.get(1)?;
    let bank: String = row.get(2)?;
    let account_type_str: String = row.get(3)?;
    let last_four_digits: Option<String> = row.get(4)?;
    let is_active: bool = row.get(5)?;

    let account_type = match account_type_str.as_str() {
        "checking" => AccountType::Checking,
        "savings" => AccountType::Savings,
        "credit_card" => AccountType::CreditCard,
        "business_checking" => AccountType::BusinessChecking,
        "business_savings" => AccountType::BusinessSavings,
        "business_credit" => AccountType::BusinessCredit,
        _ => AccountType::Checking,
    };

    Ok(Account {
        id: Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::new_v4()),
        name,
        bank,
        account_type,
        last_four_digits,
        is_active,
        metadata: Default::default(),
    })
}

/// Convert a database row to a Category.
pub fn row_to_category(row: &duckdb::Row<'_>) -> Result<Category, duckdb::Error> {
    let id: String = row.get(0)?;
    let parent_id: Option<String> = row.get(1)?;
    let name: String = row.get(2)?;
    let description: Option<String> = row.get(3)?;
    let category_type_str: String = row.get(4)?;
    let schedule_c_line: Option<String> = row.get(5)?;
    let is_tax_deductible: bool = row.get(6)?;
    let is_active: bool = row.get(7)?;
    let sort_order: i32 = row.get(8)?;

    let category_type = match category_type_str.as_str() {
        "income" => CategoryType::Income,
        "expense" => CategoryType::Expense,
        "personal" => CategoryType::Personal,
        _ => CategoryType::Expense,
    };

    Ok(Category {
        id: Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::new_v4()),
        parent_id: parent_id.and_then(|s| Uuid::parse_str(&s).ok()),
        name,
        description,
        category_type,
        schedule_c_line,
        is_tax_deductible,
        is_active,
        sort_order,
        metadata: Default::default(),
    })
}

/// Convert AccountType to string for database storage.
pub fn account_type_to_string(account_type: &AccountType) -> &'static str {
    match account_type {
        AccountType::Checking => "checking",
        AccountType::Savings => "savings",
        AccountType::CreditCard => "credit_card",
        AccountType::BusinessChecking => "business_checking",
        AccountType::BusinessSavings => "business_savings",
        AccountType::BusinessCredit => "business_credit",
    }
}

/// Convert CategoryType to string for database storage.
pub fn category_type_to_string(category_type: &CategoryType) -> &'static str {
    match category_type {
        CategoryType::Income => "income",
        CategoryType::Expense => "expense",
        CategoryType::Personal => "personal",
    }
}
