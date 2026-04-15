# src/models/

Core domain models shared across the application. Each struct uses `serde` for serialization and `uuid::Uuid` for identifiers.

## Files

### mod.rs
Module root. Re-exports every public type and declares common primitives (`Entity`, `EntityMetadata`, `Money`, `DateRange`) used by the individual model files.

### transaction.rs
Defines `Transaction`, `TransactionBuilder`, `TransactionStatus`, and `CategorizedBy`. A transaction carries a UUID, account reference, optional category, date, amount (as `Money`/`Decimal`), description, merchant, Schedule C line mapping, tax deductibility flag, SHA-256 dedupe hash, and confidence score.

### category.rs
Defines `Category` and `CategoryType` (Income, Expense, Personal). Supports hierarchy via `parent_id`, plus Schedule C line mapping and tax deductibility metadata.

### account.rs
Defines `Account`, `AccountType` (Checking, Savings, CreditCard, Business variants), and `Institution`. Stores bank name, last-four digits, and active status.

### rule.rs
Defines `Rule`, `RuleBuilder`, `RuleConditions`, `RuleCondition`, `ConditionField` (description, amount, merchant), `LogicalOperator` (AND, OR, NOT), and `RuleOperator`. Describes how a rule matches transactions and which category to assign.
