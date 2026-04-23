//! Category model for transaction classification.

use super::{Entity, EntityMetadata};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A category for classifying transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// Unique identifier
    pub id: Uuid,

    /// Parent category for hierarchy (optional)
    pub parent_id: Option<Uuid>,

    /// Display name
    pub name: String,

    /// Description
    pub description: Option<String>,

    /// Type of category (income, expense, personal)
    pub category_type: CategoryType,

    /// Default Schedule C line mapping
    pub schedule_c_line: Option<String>,

    /// Whether expenses in this category are tax deductible
    pub is_tax_deductible: bool,

    /// Whether this category is active
    pub is_active: bool,

    /// Display order within parent
    pub sort_order: i32,

    /// Entity metadata
    #[serde(flatten)]
    pub metadata: EntityMetadata,
}

/// Type of category.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CategoryType {
    Income,
    Expense,
    Personal,
}

impl Category {
    /// Create a new category.
    pub fn new(name: impl Into<String>, category_type: CategoryType) -> Self {
        Self {
            id: Uuid::new_v4(),
            parent_id: None,
            name: name.into(),
            description: None,
            category_type,
            schedule_c_line: None,
            is_tax_deductible: false,
            is_active: true,
            sort_order: 100,
            metadata: EntityMetadata::new(),
        }
    }

    /// Create a new expense category.
    pub fn expense(name: impl Into<String>) -> Self {
        Self::new(name, CategoryType::Expense)
    }

    /// Create a new income category.
    pub fn income(name: impl Into<String>) -> Self {
        Self::new(name, CategoryType::Income)
    }

    /// Create a new personal category.
    pub fn personal(name: impl Into<String>) -> Self {
        Self::new(name, CategoryType::Personal)
    }

    /// Set the parent category.
    pub fn with_parent(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Set the Schedule C line mapping.
    pub fn with_schedule_c(mut self, line: impl Into<String>) -> Self {
        self.schedule_c_line = Some(line.into());
        self.is_tax_deductible = true;
        self
    }

    /// Set the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the sort order.
    pub fn with_sort_order(mut self, order: i32) -> Self {
        self.sort_order = order;
        self
    }

    /// Check if this is a top-level category.
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    /// Deactivate the category.
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.metadata.touch();
    }

    /// Reactivate the category.
    pub fn activate(&mut self) {
        self.is_active = true;
        self.metadata.touch();
    }
}

impl Entity for Category {
    fn id(&self) -> Uuid {
        self.id
    }

    fn is_new(&self) -> bool {
        self.metadata.created_at == self.metadata.updated_at
    }
}

/// Default categories for initial setup.
///
/// Returns categories in insertion order: parents always appear before their
/// children so that the FK `parent_id REFERENCES categories(id)` is satisfied
/// when `CategoryRepository::insert_defaults` iterates the vec.
pub fn default_categories() -> Vec<Category> {
    let mut categories: Vec<Category> = vec![
        // Income categories
        Category::income("Business Income")
            .with_description("Income from business activities")
            .with_sort_order(1),
        Category::income("Freelance Income")
            .with_description("Freelance and consulting income")
            .with_sort_order(2),
        Category::income("Other Income")
            .with_description("Other business income")
            .with_sort_order(3),
        // Expense categories with Schedule C mappings
        Category::expense("Advertising")
            .with_schedule_c("L8")
            .with_description("Advertising and marketing expenses")
            .with_sort_order(10),
        Category::expense("Car & Truck")
            .with_schedule_c("L9")
            .with_description("Vehicle expenses for business use")
            .with_sort_order(11),
        Category::expense("Commissions & Fees")
            .with_schedule_c("L10")
            .with_description("Commissions and fees paid")
            .with_sort_order(12),
        Category::expense("Contract Labor")
            .with_schedule_c("L11")
            .with_description("Payments to contractors")
            .with_sort_order(13),
        Category::expense("Insurance")
            .with_schedule_c("L15")
            .with_description("Business insurance premiums")
            .with_sort_order(14),
        Category::expense("Legal & Professional")
            .with_schedule_c("L17")
            .with_description("Legal and professional services")
            .with_sort_order(15),
        Category::expense("Office Expense")
            .with_schedule_c("L18")
            .with_description("Office supplies and expenses")
            .with_sort_order(16),
        Category::expense("Rent or Lease")
            .with_schedule_c("L20b")
            .with_description("Rent for business property")
            .with_sort_order(17),
        Category::expense("Supplies")
            .with_schedule_c("L22")
            .with_description("Supplies used in business")
            .with_sort_order(18),
        Category::expense("Travel")
            .with_schedule_c("L24a")
            .with_description("Business travel expenses")
            .with_sort_order(19),
        Category::expense("Meals")
            .with_schedule_c("L24b")
            .with_description("Business meals (50% deductible)")
            .with_sort_order(20),
        Category::expense("Utilities")
            .with_schedule_c("L25")
            .with_description("Utilities for business property")
            .with_sort_order(21),
        Category::expense("Other Expenses")
            .with_schedule_c("L27a")
            .with_description("Other deductible business expenses")
            .with_sort_order(22),
        // --- Schedule A: Itemized Deductions ---
        // Medical & Dental (Line 1)
        Category::expense("Medical & Dental Expenses")
            .with_schedule_c("A-1")
            .with_description("Medical, dental, vision, prescriptions, health insurance premiums")
            .with_sort_order(30),
        // State & Local Income Taxes (Line 5a)
        Category::expense("State & Local Income Tax")
            .with_schedule_c("A-5a")
            .with_description("State and local income tax payments, estimated tax payments")
            .with_sort_order(31),
        // State & Local Personal Property Taxes (Line 5b)
        Category::expense("Personal Property Tax")
            .with_schedule_c("A-5b")
            .with_description("Vehicle registration fees, personal property taxes")
            .with_sort_order(32),
        // Real Estate Taxes (Line 5c)
        Category::expense("Real Estate Tax")
            .with_schedule_c("A-5c")
            .with_description("Property taxes on primary residence and other personal real estate")
            .with_sort_order(33),
        // Home Mortgage Interest (Line 8a)
        Category::expense("Home Mortgage Interest")
            .with_schedule_c("A-8a")
            .with_description("Mortgage interest paid to financial institutions (Form 1098)")
            .with_sort_order(34),
        // Mortgage Insurance Premiums (Line 10)
        Category::expense("Mortgage Insurance Premiums")
            .with_schedule_c("A-10")
            .with_description("PMI or MIP premiums on qualified residence")
            .with_sort_order(35),
        // Charitable - Cash (Line 12)
        Category::expense("Charitable Donations - Cash")
            .with_schedule_c("A-12")
            .with_description("Cash or check donations to qualified organizations")
            .with_sort_order(36),
        // Charitable - Non-Cash (Line 13)
        Category::expense("Charitable Donations - Non-Cash")
            .with_schedule_c("A-13")
            .with_description("Donated goods, clothing, vehicles to qualified organizations")
            .with_sort_order(37),
        // Casualty & Theft Losses (Line 15)
        Category::expense("Casualty & Theft Losses")
            .with_schedule_c("A-15")
            .with_description("Federally declared disaster area losses only")
            .with_sort_order(38),
        // Other Itemized Deductions (Line 16)
        Category::expense("Other Itemized Deductions")
            .with_schedule_c("A-16")
            .with_description("Gambling losses, impairment-related work expenses, estate tax on IRD")
            .with_sort_order(39),

        // --- Schedule E, Part I: Rental Real Estate Income ---
        Category::income("Rental Income")
            .with_schedule_c("E-3")
            .with_description("Rents received from tenants")
            .with_sort_order(40),
        Category::expense("Rental - Advertising")
            .with_schedule_c("E-5")
            .with_description("Advertising for rental property vacancies")
            .with_sort_order(41),
        Category::expense("Rental - Auto & Travel")
            .with_schedule_c("E-6")
            .with_description("Travel to rental properties for maintenance and management")
            .with_sort_order(42),
        Category::expense("Rental - Cleaning & Maintenance")
            .with_schedule_c("E-7")
            .with_description("Cleaning, landscaping, pest control, general maintenance")
            .with_sort_order(43),
        Category::expense("Rental - Commissions")
            .with_schedule_c("E-8")
            .with_description("Property management fees, leasing commissions")
            .with_sort_order(44),
        Category::expense("Rental - Insurance")
            .with_schedule_c("E-9")
            .with_description("Landlord insurance, liability, flood, umbrella policies")
            .with_sort_order(45),
        Category::expense("Rental - Legal & Professional")
            .with_schedule_c("E-10")
            .with_description("Attorney fees, CPA fees, eviction costs for rental properties")
            .with_sort_order(46),
        Category::expense("Rental - Management Fees")
            .with_schedule_c("E-11")
            .with_description("Property management company fees")
            .with_sort_order(47),
        Category::expense("Rental - Mortgage Interest")
            .with_schedule_c("E-12")
            .with_description("Mortgage interest on rental properties")
            .with_sort_order(48),
        Category::expense("Rental - Other Interest")
            .with_schedule_c("E-13")
            .with_description("Other interest expenses related to rental properties")
            .with_sort_order(49),
        Category::expense("Rental - Repairs")
            .with_schedule_c("E-14")
            .with_description("Repairs to rental properties (not improvements)")
            .with_sort_order(50),
        Category::expense("Rental - Supplies")
            .with_schedule_c("E-15")
            .with_description("Supplies used for rental property maintenance")
            .with_sort_order(51),
        Category::expense("Rental - Taxes")
            .with_schedule_c("E-16")
            .with_description("Property taxes on rental properties")
            .with_sort_order(52),
        Category::expense("Rental - Utilities")
            .with_schedule_c("E-17")
            .with_description("Utilities paid by landlord for rental properties")
            .with_sort_order(53),
        Category::expense("Rental - Depreciation")
            .with_schedule_c("E-18")
            .with_description("Depreciation of rental property and improvements")
            .with_sort_order(54),
        Category::expense("Rental - Other Expenses")
            .with_schedule_c("E-19")
            .with_description("Other rental property expenses not listed above")
            .with_sort_order(55),

        // Personal categories
        Category::personal("Personal")
            .with_description("Personal non-business expenses")
            .with_sort_order(80),
        Category::personal("Transfer")
            .with_description("Transfers between accounts")
            .with_sort_order(81),
        Category::personal("Uncategorized")
            .with_description("Transactions not yet categorized")
            .with_sort_order(99),
    ];

    // --- Investment income (1099-INT / 1099-DIV landing spots) ---
    let investment_income = Category::income("Investment Income")
        .with_description("Interest, dividends, and other investment income (1099-INT, 1099-DIV)")
        .with_sort_order(4);
    let investment_income_id = investment_income.id;
    categories.push(investment_income);
    categories.push(
        Category::income("Interest Income")
            .with_parent(investment_income_id)
            .with_description("Interest from bank accounts, bonds, etc. (1099-INT)")
            .with_sort_order(5),
    );
    categories.push(
        Category::income("Dividend Income")
            .with_parent(investment_income_id)
            .with_description("Dividends from stocks, mutual funds, ETFs (1099-DIV)")
            .with_sort_order(6),
    );

    // --- Bills (recurring obligations) ---
    // "Mortgage Payments" is a holding bucket: the full monthly payment lands
    // here before year-end 1098 reconciliation splits it into A-8a interest,
    // A-5c real estate tax (if escrowed), and non-deductible principal.
    let bills = Category::personal("Bills")
        .with_description("Recurring bill payments (mortgage, loans, subscriptions)")
        .with_sort_order(200);
    let bills_id = bills.id;
    categories.push(bills);
    categories.push(
        Category::personal("Mortgage Payments")
            .with_parent(bills_id)
            .with_description("Full mortgage payment; reconcile at year-end using Form 1098")
            .with_sort_order(201),
    );
    categories.push(
        Category::personal("Credit Card Payments")
            .with_parent(bills_id)
            .with_description("Payments to credit card accounts (often an account transfer)")
            .with_sort_order(202),
    );
    categories.push(
        Category::personal("Loan Payments")
            .with_parent(bills_id)
            .with_description("Personal loan payments; interest portion may need reconciliation")
            .with_sort_order(203),
    );
    categories.push(
        Category::personal("Subscriptions")
            .with_parent(bills_id)
            .with_description("Software, news, memberships (non-streaming)")
            .with_sort_order(204),
    );

    // --- Transportation (personal vehicle and transit) ---
    let transportation = Category::personal("Transportation")
        .with_description("Personal vehicle, transit, and ride share")
        .with_sort_order(210);
    let transportation_id = transportation.id;
    categories.push(transportation);
    categories.push(
        Category::personal("Auto Repairs")
            .with_parent(transportation_id)
            .with_description("Vehicle repairs and maintenance")
            .with_sort_order(211),
    );
    categories.push(
        Category::personal("Auto Insurance")
            .with_parent(transportation_id)
            .with_description("Personal auto insurance premiums")
            .with_sort_order(212),
    );
    categories.push(
        Category::personal("Auto Fuel")
            .with_parent(transportation_id)
            .with_description("Gasoline and charging")
            .with_sort_order(213),
    );
    categories.push(
        Category::personal("License & Registration")
            .with_parent(transportation_id)
            .with_description("Vehicle registration, license renewal")
            .with_sort_order(214),
    );
    categories.push(
        Category::personal("Public Transit")
            .with_parent(transportation_id)
            .with_description("Bus, subway, train passes")
            .with_sort_order(215),
    );
    categories.push(
        Category::personal("Ride Share")
            .with_parent(transportation_id)
            .with_description("Uber, Lyft, and similar services")
            .with_sort_order(216),
    );
    categories.push(
        Category::personal("Parking, Tolls & Tickets")
            .with_parent(transportation_id)
            .with_description("Parking fees, road tolls, traffic tickets")
            .with_sort_order(217),
    );

    // --- Personal Travel (renamed to avoid UNIQUE clash with Schedule C "Travel") ---
    let personal_travel = Category::personal("Personal Travel")
        .with_description("Non-business flights, hotels, and trip expenses")
        .with_sort_order(220);
    let personal_travel_id = personal_travel.id;
    categories.push(personal_travel);
    categories.push(
        Category::personal("Flights")
            .with_parent(personal_travel_id)
            .with_description("Airline tickets for personal travel")
            .with_sort_order(221),
    );
    categories.push(
        Category::personal("Hotel")
            .with_parent(personal_travel_id)
            .with_description("Lodging for personal travel")
            .with_sort_order(222),
    );
    categories.push(
        Category::personal("Travel Ride Share")
            .with_parent(personal_travel_id)
            .with_description("Ride share used while traveling")
            .with_sort_order(223),
    );

    // --- Household Utilities (renamed to avoid UNIQUE clash with Schedule C "Utilities") ---
    let household_utilities = Category::personal("Household Utilities")
        .with_description("Home utility bills (gas, water, power, internet, phone)")
        .with_sort_order(230);
    let household_utilities_id = household_utilities.id;
    categories.push(household_utilities);
    categories.push(
        Category::personal("Natural Gas")
            .with_parent(household_utilities_id)
            .with_description("Home heating gas utility")
            .with_sort_order(231),
    );
    categories.push(
        Category::personal("Water & Sewer")
            .with_parent(household_utilities_id)
            .with_description("Water and sewer service")
            .with_sort_order(232),
    );
    categories.push(
        Category::personal("Electricity")
            .with_parent(household_utilities_id)
            .with_description("Electric utility")
            .with_sort_order(233),
    );
    categories.push(
        Category::personal("Internet Service")
            .with_parent(household_utilities_id)
            .with_description("Home internet service")
            .with_sort_order(234),
    );
    categories.push(
        Category::personal("Streaming Services")
            .with_parent(household_utilities_id)
            .with_description("Streaming subscriptions (Netflix, Hulu, Spotify, etc.)")
            .with_sort_order(235),
    );
    categories.push(
        Category::personal("Mobile Phone")
            .with_parent(household_utilities_id)
            .with_description("Cell phone service")
            .with_sort_order(236),
    );

    // --- Family (household spending) ---
    let family = Category::personal("Family")
        .with_description("Household spending: food, clothing, kids, pets")
        .with_sort_order(240);
    let family_id = family.id;
    categories.push(family);
    categories.push(
        Category::personal("Groceries")
            .with_parent(family_id)
            .with_description("Food and household staples")
            .with_sort_order(241),
    );
    categories.push(
        Category::personal("Dining Out")
            .with_parent(family_id)
            .with_description("Restaurants, takeout, coffee (personal)")
            .with_sort_order(242),
    );
    categories.push(
        Category::personal("Entertainment")
            .with_parent(family_id)
            .with_description("Movies, concerts, events")
            .with_sort_order(243),
    );
    categories.push(
        Category::personal("Clothing & Personal Care")
            .with_parent(family_id)
            .with_description("Clothing, haircuts, toiletries")
            .with_sort_order(244),
    );
    categories.push(
        Category::personal("School Expenses")
            .with_parent(family_id)
            .with_description("Supplies, activities, fees for K-12")
            .with_sort_order(245),
    );
    categories.push(
        Category::personal("Childcare")
            .with_parent(family_id)
            .with_description("Daycare, babysitters (Form 2441 / Dependent Care FSA)")
            .with_sort_order(246),
    );
    categories.push(
        Category::personal("Pets")
            .with_parent(family_id)
            .with_description("Pet food, vet, grooming, supplies")
            .with_sort_order(247),
    );
    categories.push(
        Category::personal("Gifts")
            .with_parent(family_id)
            .with_description("Gifts given (birthdays, holidays)")
            .with_sort_order(248),
    );
    categories.push(
        Category::personal("Education & Tuition")
            .with_parent(family_id)
            .with_description("Higher-education tuition and fees (1098-T)")
            .with_sort_order(249),
    );

    // --- Healthcare - Personal (complements Schedule A "Medical & Dental Expenses") ---
    let healthcare_personal = Category::personal("Healthcare - Personal")
        .with_description("Out-of-pocket health spending not tied to business")
        .with_sort_order(250);
    let healthcare_personal_id = healthcare_personal.id;
    categories.push(healthcare_personal);
    // Co-pays are Schedule A (A-1) deductible.
    categories.push(
        Category::expense("Co-pays")
            .with_parent(healthcare_personal_id)
            .with_schedule_c("A-1")
            .with_description("Medical and dental co-pays (Schedule A deductible)")
            .with_sort_order(251),
    );
    categories.push(
        Category::personal("Vitamins")
            .with_parent(healthcare_personal_id)
            .with_description("Vitamins and supplements (generally not deductible)")
            .with_sort_order(252),
    );
    categories.push(
        Category::personal("Gym Memberships")
            .with_parent(healthcare_personal_id)
            .with_description("Gym and fitness memberships (generally not deductible)")
            .with_sort_order(253),
    );
    categories.push(
        Category::personal("HSA Contributions")
            .with_parent(healthcare_personal_id)
            .with_description("Health Savings Account contributions (1099-SA / Form 8889)")
            .with_sort_order(254),
    );

    // --- Home (non-rental primary residence) ---
    let home = Category::personal("Home")
        .with_description("Primary residence improvements and maintenance")
        .with_sort_order(260);
    let home_id = home.id;
    categories.push(home);
    categories.push(
        Category::personal("Home Improvement")
            .with_parent(home_id)
            .with_description("Capital improvements that adjust cost basis")
            .with_sort_order(261),
    );
    categories.push(
        Category::personal("Home Maintenance")
            .with_parent(home_id)
            .with_description("Routine repairs and upkeep (not capital improvements)")
            .with_sort_order(262),
    );

    categories
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_creation() {
        let cat = Category::expense("Office Supplies")
            .with_schedule_c("L18")
            .with_description("Business office supplies");

        assert_eq!(cat.name, "Office Supplies");
        assert_eq!(cat.category_type, CategoryType::Expense);
        assert_eq!(cat.schedule_c_line, Some("L18".to_string()));
        assert!(cat.is_tax_deductible);
        assert!(cat.is_active);
    }

    #[test]
    fn test_default_categories() {
        let categories = default_categories();
        assert!(!categories.is_empty());

        let expense_count = categories
            .iter()
            .filter(|c| c.category_type == CategoryType::Expense)
            .count();
        assert!(expense_count > 0);
    }

    #[test]
    fn test_default_category_names_are_unique() {
        let categories = default_categories();
        let mut names: Vec<&str> = categories.iter().map(|c| c.name.as_str()).collect();
        names.sort_unstable();
        let len_before = names.len();
        names.dedup();
        assert_eq!(len_before, names.len(), "duplicate category names in defaults");
    }

    #[test]
    fn test_default_category_parents_resolve_and_precede_children() {
        let categories = default_categories();
        let mut seen_ids = std::collections::HashSet::new();
        for cat in &categories {
            if let Some(parent_id) = cat.parent_id {
                assert!(
                    seen_ids.contains(&parent_id),
                    "child '{}' appears before (or without) its parent in default_categories()",
                    cat.name
                );
            }
            seen_ids.insert(cat.id);
        }
    }
}
