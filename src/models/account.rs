//! Account model representing bank and credit card accounts.

use super::{Entity, EntityMetadata};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A bank or credit card account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Unique identifier
    pub id: Uuid,

    /// Display name
    pub name: String,

    /// Bank/institution name
    pub bank: String,

    /// Type of account
    pub account_type: AccountType,

    /// Last 4 digits for identification
    pub last_four_digits: Option<String>,

    /// Whether this account is active
    pub is_active: bool,

    /// Entity metadata
    #[serde(flatten)]
    pub metadata: EntityMetadata,
}

/// Type of account.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Checking,
    Savings,
    CreditCard,
    BusinessChecking,
    BusinessSavings,
    BusinessCredit,
}

impl AccountType {
    /// Get a human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            AccountType::Checking => "Checking",
            AccountType::Savings => "Savings",
            AccountType::CreditCard => "Credit Card",
            AccountType::BusinessChecking => "Business Checking",
            AccountType::BusinessSavings => "Business Savings",
            AccountType::BusinessCredit => "Business Credit",
        }
    }

    /// Check if this is a business account.
    pub fn is_business(&self) -> bool {
        matches!(
            self,
            AccountType::BusinessChecking
                | AccountType::BusinessSavings
                | AccountType::BusinessCredit
        )
    }

    /// Check if this is a credit account.
    pub fn is_credit(&self) -> bool {
        matches!(self, AccountType::CreditCard | AccountType::BusinessCredit)
    }
}

impl Account {
    /// Create a new account.
    pub fn new(name: impl Into<String>, bank: impl Into<String>, account_type: AccountType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            bank: bank.into(),
            account_type,
            last_four_digits: None,
            is_active: true,
            metadata: EntityMetadata::new(),
        }
    }

    /// Set the last four digits.
    pub fn with_last_four(mut self, digits: impl Into<String>) -> Self {
        self.last_four_digits = Some(digits.into());
        self
    }

    /// Get a display string for the account.
    pub fn display_name(&self) -> String {
        if let Some(ref digits) = self.last_four_digits {
            format!("{} {} (****{})", self.bank, self.name, digits)
        } else {
            format!("{} {}", self.bank, self.name)
        }
    }

    /// Deactivate the account.
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.metadata.touch();
    }

    /// Reactivate the account.
    pub fn activate(&mut self) {
        self.is_active = true;
        self.metadata.touch();
    }
}

impl Entity for Account {
    fn id(&self) -> Uuid {
        self.id
    }

    fn is_new(&self) -> bool {
        self.metadata.created_at == self.metadata.updated_at
    }
}

/// Supported bank institutions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Institution {
    Chase,
    BankOfAmerica,
    Wealthfront,
    Ally,
    AmericanExpress,
    Discover,
    Citi,
    CapitalOne,
    Generic,
}

impl Institution {
    /// Get the institution name.
    pub fn name(&self) -> &'static str {
        match self {
            Institution::Chase => "Chase",
            Institution::BankOfAmerica => "Bank of America",
            Institution::Wealthfront => "Wealthfront",
            Institution::Ally => "Ally",
            Institution::AmericanExpress => "American Express",
            Institution::Discover => "Discover",
            Institution::Citi => "Citi",
            Institution::CapitalOne => "Capital One",
            Institution::Generic => "Generic",
        }
    }

    /// Get all supported institutions.
    pub fn all() -> Vec<Institution> {
        vec![
            Institution::Chase,
            Institution::BankOfAmerica,
            Institution::Wealthfront,
            Institution::Ally,
            Institution::AmericanExpress,
            Institution::Discover,
            Institution::Citi,
            Institution::CapitalOne,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let account = Account::new("Sapphire Reserve", "Chase", AccountType::CreditCard)
            .with_last_four("1234");

        assert_eq!(account.name, "Sapphire Reserve");
        assert_eq!(account.bank, "Chase");
        assert_eq!(account.account_type, AccountType::CreditCard);
        assert_eq!(account.last_four_digits, Some("1234".to_string()));
    }

    #[test]
    fn test_display_name() {
        let account = Account::new("Checking", "Chase", AccountType::Checking)
            .with_last_four("5678");

        assert_eq!(account.display_name(), "Chase Checking (****5678)");
    }

    #[test]
    fn test_account_type_properties() {
        assert!(AccountType::BusinessChecking.is_business());
        assert!(!AccountType::Checking.is_business());
        assert!(AccountType::CreditCard.is_credit());
        assert!(!AccountType::Savings.is_credit());
    }
}
