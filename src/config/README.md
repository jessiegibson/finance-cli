# src/config/

Application configuration loading and persistence. Stores user preferences in a TOML file under the platform's config directory.

## Files

### mod.rs
Module root. Exposes `load_or_create`, which loads `~/.finance-cli/config.toml` if present or writes a default file otherwise. Re-exports `Config` and `ConfigBuilder`.

### settings.rs
Defines the `Config` struct and its `ConfigBuilder`. Fields include database path, config directory, encryption flag, default category names, and import options (dedupe). Uses the `directories` crate to resolve platform-appropriate paths and provides `ensure_directories` to create missing folders.
