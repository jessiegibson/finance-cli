//! Database model conversions.
//!
//! This module provides conversions between database rows and domain models.

use crate::models::{
    Account, AccountType, Category, CategoryType, Money, Rule, RuleConditions, Transaction,
    CategorizedBy,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;
use serde_json;

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

/// Convert a database row to a Transaction.
pub fn row_to_transaction(row: &duckdb::Row<'_>) -> Result<Transaction, duckdb::Error> {
    let id: String = row.get(0)?;
    let account_id: String = row.get(1)?;
    let category_id: Option<String> = row.get(2)?;
    let import_batch_id: Option<String> = row.get(3)?;
    let date_str: String = row.get(4)?;
    let amount_str: String = row.get(5)?;
    let description: String = row.get(6)?;
    let raw_category: Option<String> = row.get(7)?;
    let merchant_name: Option<String> = row.get(8)?;
    let location: Option<String> = row.get(9)?;
    let reference_number: Option<String> = row.get(10)?;
    let transaction_hash: String = row.get(11)?;
    let schedule_c_line: Option<String> = row.get(12)?;
    let is_business_expense: bool = row.get(13)?;
    let is_tax_deductible: bool = row.get(14)?;
    let is_recurring: bool = row.get(15)?;
    let expense_type: Option<String> = row.get(16)?;
    let categorized_by_str: Option<String> = row.get(17)?;
    let confidence_score: Option<f64> = row.get(18)?;

    let transaction_date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());

    let amount = Money::new(
        Decimal::from_str(&amount_str).unwrap_or_else(|_| Decimal::ZERO),
    );

    let categorized_by = categorized_by_str.map(|s| match s.as_str() {
        "rule" => CategorizedBy::Rule,
        "manual" => CategorizedBy::Manual,
        "ml" => CategorizedBy::Ml,
        _ => CategorizedBy::Default,
    });

    Ok(Transaction {
        id: Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::new_v4()),
        account_id: Uuid::parse_str(&account_id).unwrap_or_else(|_| Uuid::new_v4()),
        category_id: category_id.and_then(|s| Uuid::parse_str(&s).ok()),
        import_batch_id: import_batch_id.and_then(|s| Uuid::parse_str(&s).ok()),
        transaction_date,
        amount,
        description,
        raw_category,
        merchant_name,
        location,
        reference_number,
        transaction_hash,
        schedule_c_line,
        is_business_expense,
        is_tax_deductible,
        is_recurring,
        expense_type,
        categorized_by,
        confidence_score,
        metadata: Default::default(),
    })
}

/// Convert a database row to a Rule.
pub fn row_to_rule(row: &duckdb::Row<'_>) -> Result<Rule, duckdb::Error> {
    let id_str: String = row.get(0)?;
    let target_category_id_str: String = row.get(1)?;
    let name: String = row.get(2)?;
    let description: Option<String> = row.get(3)?;
    let priority: i32 = row.get(4)?;
    let conditions_json: String = row.get(5)?;
    let is_active: bool = row.get(6)?;
    let effectiveness_count: i32 = row.get(7)?;

    let id = Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4());
    let target_category_id =
        Uuid::parse_str(&target_category_id_str).unwrap_or_else(|_| Uuid::new_v4());
    let conditions: RuleConditions = serde_json::from_str(&conditions_json)
        .unwrap_or_else(|_| RuleConditions::all(vec![]));

    Ok(Rule {
        id,
        target_category_id,
        name,
        description,
        priority,
        conditions,
        is_active,
        effectiveness_count,
        last_applied_at: None,
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
