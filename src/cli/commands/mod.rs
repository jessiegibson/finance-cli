//! CLI command implementations.

pub mod category;
pub mod config;
pub mod report;
pub mod transaction;

pub use category::{handle_category, CategoryCommand};
pub use config::{handle_config, ConfigCommand};
pub use report::{handle_report, ReportCommand};
pub use transaction::{handle_transaction, TransactionCommand};

use crate::config::Config;
use crate::database::{CategoryRepository, Connection, TransactionRepository};
use crate::error::Result;

/// Handle the init command.
pub fn handle_init(config: &Config, conn: &Connection) -> Result<()> {
    use colored::Colorize;

    println!("{}", "Initializing Finance CLI...".green());

    // Ensure directories exist
    config.ensure_directories()?;
    println!("  {} Created data directories", "✓".green());

    // Insert default categories
    let category_repo = CategoryRepository::new(conn);
    category_repo.insert_defaults()?;
    let count = category_repo.count()?;
    println!("  {} Created {} default categories", "✓".green(), count);

    println!();
    println!("{}", "Initialization complete!".green().bold());
    println!();
    println!("Next steps:");
    println!("  1. Import transactions: finance transaction import <file>");
    println!("  2. Categorize transactions: finance transaction categorize");
    println!("  3. Generate reports: finance report pnl --year 2024");

    Ok(())
}

/// Handle the status command.
pub fn handle_status(config: &Config, conn: &Connection) -> Result<()> {
    use colored::Colorize;

    println!("{}", "Finance CLI Status".bold());
    println!();

    // Database info
    println!("{}", "Database:".bold());
    println!("  Path: {}", config.database_path.display());

    // Transaction counts
    let tx_repo = TransactionRepository::new(conn);
    let tx_count = tx_repo.count()?;
    let uncategorized = tx_repo.count_uncategorized()?;

    println!();
    println!("{}", "Transactions:".bold());
    println!("  Total: {}", tx_count);
    println!("  Uncategorized: {}", uncategorized);
    if tx_count > 0 {
        let categorized_pct = ((tx_count - uncategorized) as f64 / tx_count as f64) * 100.0;
        println!("  Categorized: {:.1}%", categorized_pct);
    }

    // Category counts
    let cat_repo = CategoryRepository::new(conn);
    let cat_count = cat_repo.count()?;

    println!();
    println!("{}", "Categories:".bold());
    println!("  Total: {}", cat_count);

    Ok(())
}
