# src/cli/

Command-line interface layer. Parses arguments with `clap`, dispatches to subcommand handlers, and formats terminal output.

## Files

### mod.rs
Defines the top-level `Cli` struct and `Commands` enum using `clap` derive macros. Declares global flags (`--verbose`, `--quiet`, `--config`) and wires each subcommand variant to its handler in `commands/`.

### output.rs
Terminal output helpers built on the `colored` crate. Provides `success`, `error`, `warning`, `info`, and `header` functions for consistent, color-coded user messages across all subcommands.

## Subdirectories

- `commands/` — Individual subcommand handler modules.
