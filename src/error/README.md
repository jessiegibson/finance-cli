# src/error/

Centralized error handling. Defines a single `Error` enum covering every failure mode in the application, plus helpers for adding context and producing user-friendly suggestions.

## Files

### mod.rs
Declares `Error` (a `thiserror`-derived enum with variants for Config, Database, Encryption, Parse, Validation, and I/O errors) and `Result<T>`, a type alias using the custom error type. Also maps each variant to an exit code and a human-readable suggestion string.

### types.rs
Extension traits for enriching errors. `ErrorContext` adds eager (`context`) and lazy (`with_context`) helpers to any `Result<T, E: Into<Error>>`, letting callers attach descriptive messages without losing the original error chain.
