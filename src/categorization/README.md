# src/categorization/

Automatic transaction classification. Applies user-defined rules first and falls back to ML-assisted categorization when no rule matches.

## Files

### mod.rs
Module root. Declares `CategorizationMethod` and `CategorizationResult` and re-exports `CategorizationEngine`, `RuleMatcher`, and `MlCategorizer`.

### engine.rs
Main orchestrator. `CategorizationEngine` holds loaded rules and categories, then iterates over transactions and assigns a category plus a confidence score. Returns a `CategorizationResult` describing which method (rule or ML) produced the assignment.

### rules.rs
Rule evaluation. `RuleMatcher::matches` checks whether a `Rule` applies to a `Transaction` by evaluating each `RuleCondition` against the transaction's description, amount, or merchant field, combined with the rule's logical operator (AND, OR, NOT).

### ml.rs
Placeholder for ML-based categorization. `MlCategorizer` currently returns `None` for all transactions. Reserved for a future model that learns from previously categorized rows.
