//! Transaction command handlers.

use crate::categorization::CategorizationEngine;
use crate::cli::output;
use crate::config::Config;
use crate::database::{AccountRepository, CategoryRepository, Connection, RuleRepository, TransactionRepository};
use crate::error::{Error, Result};
use crate::models::{Account, AccountType, DateRange, RuleBuilder};
use crate::parsers;
use clap::{Args, Subcommand};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct TransactionCommand {
    #[command(subcommand)]
    pub action: TransactionAction,
}

#[derive(Subcommand, Debug)]
pub enum TransactionAction {
    /// Import transactions from a file
    Import {
        /// Path to the file to import
        file: PathBuf,

        /// Account name to import into (required)
        #[arg(short, long)]
        account: Option<String>,

        /// Skip duplicate detection
        #[arg(long)]
        no_dedupe: bool,

        /// Dry run - show what would be imported without saving
        #[arg(long)]
        dry_run: bool,
    },

    /// List transactions
    List {
        /// Number of transactions to show
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Show only uncategorized transactions
        #[arg(long)]
        uncategorized: bool,

        /// Filter by year
        #[arg(short, long)]
        year: Option<i32>,

        /// Filter by month (1-12)
        #[arg(short, long)]
        month: Option<u32>,
    },

    /// Interactively categorize transactions
    Categorize {
        /// Number of transactions to categorize
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Apply rules to all uncategorized transactions in bulk
    BulkCategorize {
        /// Only show what would change, don't apply
        #[arg(long)]
        dry_run: bool,

        /// Also re-categorize transactions that already have a category
        #[arg(long)]
        recategorize: bool,

        /// Filter by year
        #[arg(short, long)]
        year: Option<i32>,

        /// Filter by month (1-12)
        #[arg(short, long)]
        month: Option<u32>,
    },

    /// Search transactions
    Search {
        /// Search query
        query: String,
    },

    /// Delete transactions by ID or account
    Delete {
        /// Transaction ID to delete (mutually exclusive with --account)
        #[arg(short, long)]
        id: Option<String>,

        /// Account name to delete all transactions from (mutually exclusive with --id)
        #[arg(short, long)]
        account: Option<String>,

        /// Preview changes without deleting
        #[arg(long)]
        dry_run: bool,
    },
}

pub fn handle_transaction(cmd: TransactionCommand, _config: &Config, conn: &Connection) -> Result<()> {
    match cmd.action {
        TransactionAction::Import {
            file,
            account,
            no_dedupe,
            dry_run,
        } => handle_import(file, account, no_dedupe, dry_run, conn),

        TransactionAction::List {
            limit,
            uncategorized,
            year,
            month,
        } => handle_list(limit, uncategorized, year, month, conn),

        TransactionAction::Categorize { limit } => handle_categorize(limit, conn),

        TransactionAction::BulkCategorize {
            dry_run,
            recategorize,
            year,
            month,
        } => handle_bulk_categorize(dry_run, recategorize, year, month, conn),

        TransactionAction::Search { query } => {
            println!("{}", "Search Transactions".bold());
            println!();
            println!("Query: {}", query);
            println!();
            println!("{}", "Search functionality coming soon!".yellow());
            Ok(())
        }

        TransactionAction::Delete { id, account, dry_run } => {
            handle_delete(id, account, dry_run, conn)
        }
    }
}

fn handle_import(
    file: PathBuf,
    account_name: Option<String>,
    no_dedupe: bool,
    dry_run: bool,
    conn: &Connection,
) -> Result<()> {
    // Require --account flag
    let account_name = account_name.ok_or_else(|| {
        Error::InvalidInput(
            "--account is required. Specify an account name, e.g. --account \"Chase Checking\". \
             Run `finance category list` to see existing categories, or provide any name to auto-create."
                .to_string(),
        )
    })?;

    output::header(&format!("Import Transactions"));
    output::kv("File", &file.display().to_string());
    output::kv("Account", &account_name);
    if dry_run {
        output::warning("Dry run — no changes will be saved");
    }
    println!();

    // Resolve or create account
    let account_repo = AccountRepository::new(conn);
    let account = match account_repo.find_by_name(&account_name)? {
        Some(existing) => existing,
        None => {
            let new_account = Account::new(&account_name, "Unknown", AccountType::Checking);
            if !dry_run {
                account_repo.insert(&new_account)?;
                output::info(&format!("Created new account: {}", account_name));
            } else {
                output::info(&format!("Would create new account: {}", account_name));
            }
            new_account
        }
    };

    // Parse the file
    if !file.exists() {
        return Err(Error::InvalidInput(format!(
            "File not found: {}",
            file.display()
        )));
    }

    let parse_result = parsers::parse_file(&file, &account)?;

    // Display detected format / institution
    if let Some(ref institution) = parse_result.institution {
        output::kv("Institution", institution);
    }
    output::kv("Format", &format!("{:?}", parse_result.format));
    output::kv("Parsed", &format!("{} transactions", parse_result.transactions.len()));
    if !parse_result.errors.is_empty() {
        output::kv("Parse errors", &format!("{}", parse_result.errors.len()));
    }
    println!();

    if dry_run {
        // Just report what would happen
        output::info(&format!(
            "Would import up to {} transactions",
            parse_result.transactions.len()
        ));
        if !parse_result.errors.is_empty() {
            output::warning(&format!(
                "{} rows could not be parsed:",
                parse_result.errors.len()
            ));
            for err in &parse_result.errors {
                println!("  {}", err.dimmed());
            }
        }
        return Ok(());
    }

    // Import transactions
    let tx_repo = TransactionRepository::new(conn);
    let mut imported = 0usize;
    let mut skipped_dupes = 0usize;

    for tx in &parse_result.transactions {
        if !no_dedupe && tx_repo.hash_exists(&tx.transaction_hash)? {
            skipped_dupes += 1;
            continue;
        }
        tx_repo.insert(tx)?;
        imported += 1;
    }

    // Summary
    output::success(&format!("Imported {} transactions", imported));
    if skipped_dupes > 0 {
        output::info(&format!("Skipped {} duplicates", skipped_dupes));
    }
    if !parse_result.errors.is_empty() {
        output::warning(&format!(
            "{} rows could not be parsed:",
            parse_result.errors.len()
        ));
        for err in &parse_result.errors {
            println!("  {}", err.dimmed());
        }
    }

    Ok(())
}

fn handle_list(
    limit: usize,
    uncategorized: bool,
    year: Option<i32>,
    month: Option<u32>,
    conn: &Connection,
) -> Result<()> {
    let tx_repo = TransactionRepository::new(conn);

    // Fetch transactions
    let all = match (year, month) {
        (Some(y), Some(m)) => {
            let range = DateRange::month(y, m).ok_or_else(|| {
                Error::InvalidInput(format!("Invalid year/month: {}/{}", y, m))
            })?;
            tx_repo.find_by_date_range(&range)?
        }
        (Some(y), None) => {
            let range = DateRange::year(y).ok_or_else(|| {
                Error::InvalidInput(format!("Invalid year: {}", y))
            })?;
            tx_repo.find_by_date_range(&range)?
        }
        _ => tx_repo.find_all()?,
    };

    // Apply filters
    let filtered: Vec<_> = all
        .iter()
        .filter(|tx| !uncategorized || !tx.is_categorized())
        .collect();

    let total = filtered.len();
    let shown: Vec<_> = filtered.into_iter().take(limit).collect();

    // Header
    println!("{}", "Transactions".bold());
    if uncategorized {
        println!("{}", "Showing uncategorized only".dimmed());
    }
    match (year, month) {
        (Some(y), Some(m)) => println!("{}", format!("Period: {}-{:02}", y, m).dimmed()),
        (Some(y), None) => println!("{}", format!("Year: {}", y).dimmed()),
        _ => {}
    }
    println!();

    if shown.is_empty() {
        output::info("No transactions found. Run `finance tx import <file> --account <name>` to add some.");
        return Ok(());
    }

    // Column widths
    let date_w = 10;
    let amount_w = 12;
    let desc_w = 40;
    let cat_w = 20;

    // Header row
    println!(
        "  {:<date_w$}  {:>amount_w$}  {:<desc_w$}  {:<cat_w$}",
        "Date".bold(),
        "Amount".bold(),
        "Description".bold(),
        "Category".bold(),
        date_w = date_w,
        amount_w = amount_w,
        desc_w = desc_w,
        cat_w = cat_w,
    );
    println!(
        "  {}",
        "─".repeat(date_w + 2 + amount_w + 2 + desc_w + 2 + cat_w)
    );

    for tx in &shown {
        let date = tx.transaction_date.to_string();
        let amount_str = output::format_money(&tx.amount);

        let raw_desc = &tx.description;
        let desc = if raw_desc.chars().count() > desc_w {
            format!("{}…", &raw_desc[..raw_desc.char_indices().nth(desc_w - 1).map(|(i, _)| i).unwrap_or(desc_w - 1)])
        } else {
            raw_desc.clone()
        };

        let category = if tx.is_categorized() {
            "Categorized".to_string()
        } else {
            "Uncategorized".dimmed().to_string()
        };

        println!(
            "  {:<date_w$}  {:>amount_w$}  {:<desc_w$}  {:<cat_w$}",
            date,
            amount_str,
            desc,
            category,
            date_w = date_w,
            amount_w = amount_w + 10, // colored strings have extra escape chars
            desc_w = desc_w,
            cat_w = cat_w,
        );
    }

    println!();
    if total > limit {
        println!(
            "{}",
            format!("Showing {} of {} transactions (use --limit to see more)", shown.len(), total).dimmed()
        );
    } else {
        println!("{}", format!("Total: {} transactions", total).dimmed());
    }

    Ok(())
}

fn handle_categorize(limit: usize, conn: &Connection) -> Result<()> {
    let theme = ColorfulTheme::default();
    let tx_repo = TransactionRepository::new(conn);
    let cat_repo = CategoryRepository::new(conn);
    let rule_repo = RuleRepository::new(conn);

    // Load uncategorized transactions
    let transactions = tx_repo.find_uncategorized(limit)?;
    if transactions.is_empty() {
        output::info("No uncategorized transactions. Run `finance tx import` to add more.");
        return Ok(());
    }

    // Load categorization engine (rules + categories)
    let engine = CategorizationEngine::from_database(conn)?;

    // Load categories for the selection menu
    let categories = cat_repo.find_active()?;

    // Build display labels grouped: suggested first, then Income, Expense, Personal
    // We'll rebuild per-transaction, but prepare the full list ordering once
    let income_cats: Vec<_> = categories.iter()
        .filter(|c| matches!(c.category_type, crate::models::CategoryType::Income))
        .collect();
    let expense_cats: Vec<_> = categories.iter()
        .filter(|c| matches!(c.category_type, crate::models::CategoryType::Expense))
        .collect();
    let personal_cats: Vec<_> = categories.iter()
        .filter(|c| matches!(c.category_type, crate::models::CategoryType::Personal))
        .collect();

    // Flat ordered list (without suggestion — we'll prepend per-tx)
    let mut ordered_cats: Vec<&crate::models::Category> = Vec::new();
    ordered_cats.extend(income_cats);
    ordered_cats.extend(expense_cats);
    ordered_cats.extend(personal_cats);

    let total = transactions.len();
    println!();
    println!(
        "{}",
        format!("Categorizing {} transaction{} — Ctrl+C to stop early", total,
            if total == 1 { "" } else { "s" }).bold()
    );
    println!("{}", "Type to fuzzy-search categories, arrows to navigate, Enter to select.".dimmed());
    println!();

    let mut categorized = 0usize;
    let mut skipped = 0usize;

    for (idx, tx) in transactions.iter().enumerate() {
        // Transaction header
        println!(
            "{} {}",
            format!("[{}/{}]", idx + 1, total).dimmed(),
            "─".repeat(50).dimmed()
        );
        println!(
            "  {}  {}  {}",
            tx.transaction_date.to_string().cyan(),
            output::format_money(&tx.amount),
            tx.description.bold(),
        );
        if let Some(ref merchant) = tx.merchant_name {
            println!("  {}", format!("Merchant: {}", merchant).dimmed());
        }
        if let Some(ref raw) = tx.raw_category {
            println!("  {}", format!("Bank category: {}", raw).dimmed());
        }
        println!();

        // Run engine to get suggestion
        let suggestion = engine.categorize(tx);
        let suggested_cat = suggestion.category.as_ref();

        // Build options list: suggested first (if any), then ordered, then Skip
        let mut options: Vec<String> = Vec::new();
        let mut option_indices: Vec<Option<usize>> = Vec::new(); // index into ordered_cats, None = skip

        if let Some(cat) = suggested_cat {
            options.push(format!("[Suggested] {} (via rule)", cat.name).green().to_string());
            option_indices.push(ordered_cats.iter().position(|c| c.id == cat.id));
        }

        for (i, cat) in ordered_cats.iter().enumerate() {
            let label = if let Some(ref line) = cat.schedule_c_line {
                format!("{} ({})", cat.name, line)
            } else {
                cat.name.clone()
            };
            options.push(label);
            option_indices.push(Some(i));
        }

        options.push("Skip".to_string());
        option_indices.push(None);

        let selection = Select::with_theme(&theme)
            .with_prompt("Category")
            .items(&options)
            .default(0)
            .interact_opt()
            .map_err(|e: dialoguer::Error| Error::InvalidInput(e.to_string()))?;

        let selection = match selection {
            Some(i) => i,
            None => {
                // User pressed Escape
                skipped += 1;
                println!();
                continue;
            }
        };

        // "Skip" is last option
        if selection == options.len() - 1 {
            skipped += 1;
            println!();
            continue;
        }

        // Resolve the chosen category
        let cat_index = match option_indices[selection] {
            Some(i) => i,
            None => {
                skipped += 1;
                println!();
                continue;
            }
        };
        let chosen = ordered_cats[cat_index];
        let confidence = if suggested_cat.map(|s| s.id) == Some(chosen.id) {
            1.0
        } else {
            0.9
        };

        tx_repo.update_category(tx.id, chosen.id, confidence)?;
        categorized += 1;
        output::success(&format!("Assigned: {}", chosen.name));

        // Offer rule creation
        println!();
        let create_rule = Confirm::with_theme(&theme)
            .with_prompt(format!(
                "Save a rule to auto-categorize future \"{}\" transactions?",
                tx.merchant_name.as_deref().unwrap_or(&tx.description[..tx.description.len().min(30)])
            ))
            .default(false)
            .interact()
            .map_err(|e| Error::InvalidInput(e.to_string()))?;

        if create_rule {
            let default_pattern = tx.merchant_name.clone()
                .unwrap_or_else(|| tx.description.clone());
            let default_name = format!("{} → {}", default_pattern, chosen.name);

            let rule_name: String = Input::with_theme(&theme)
                .with_prompt("Rule name")
                .with_initial_text(&default_name)
                .interact_text()
                .map_err(|e| Error::InvalidInput(e.to_string()))?;

            let pattern: String = Input::with_theme(&theme)
                .with_prompt("Match description containing")
                .with_initial_text(&default_pattern)
                .interact_text()
                .map_err(|e| Error::InvalidInput(e.to_string()))?;

            let rule = RuleBuilder::new(rule_name, chosen.id)
                .description_contains(pattern)
                .build();
            rule_repo.insert(&rule)?;
            output::success("Rule saved.");
        }

        println!();
    }

    // Summary
    println!("{}", "─".repeat(55).dimmed());
    println!(
        "  {}  {}",
        format!("Categorized: {}", categorized).green().bold(),
        format!("Skipped: {}", skipped).dimmed(),
    );
    if categorized > 0 {
        println!();
        output::info("Run `finance report pnl` to see your financial summary.");
    }

    Ok(())
}

fn handle_bulk_categorize(
    dry_run: bool,
    recategorize: bool,
    year: Option<i32>,
    month: Option<u32>,
    conn: &Connection,
) -> Result<()> {
    output::header("Bulk Categorize");

    let tx_repo = TransactionRepository::new(conn);
    let engine = CategorizationEngine::from_database(conn)?;

    let rules = engine.rules();
    if rules.is_empty() {
        output::warning("No categorization rules found.");
        output::info("Create rules with `finance tx categorize` first, then re-run bulk.");
        return Ok(());
    }
    output::kv("Active rules", &rules.len().to_string());

    // Fetch target transactions
    let transactions = if recategorize {
        match (year, month) {
            (Some(y), Some(m)) => {
                let range = DateRange::month(y, m).ok_or_else(|| {
                    Error::InvalidInput(format!("Invalid year/month: {}/{}", y, m))
                })?;
                tx_repo.find_by_date_range(&range)?
            }
            (Some(y), None) => {
                let range = DateRange::year(y).ok_or_else(|| {
                    Error::InvalidInput(format!("Invalid year: {}", y))
                })?;
                tx_repo.find_by_date_range(&range)?
            }
            _ => tx_repo.find_all()?,
        }
    } else {
        // Uncategorized only — fetch all then filter by date if needed
        match (year, month) {
            (Some(y), Some(m)) => {
                let range = DateRange::month(y, m).ok_or_else(|| {
                    Error::InvalidInput(format!("Invalid year/month: {}/{}", y, m))
                })?;
                let all = tx_repo.find_by_date_range(&range)?;
                all.into_iter().filter(|tx| !tx.is_categorized()).collect()
            }
            (Some(y), None) => {
                let range = DateRange::year(y).ok_or_else(|| {
                    Error::InvalidInput(format!("Invalid year: {}", y))
                })?;
                let all = tx_repo.find_by_date_range(&range)?;
                all.into_iter().filter(|tx| !tx.is_categorized()).collect()
            }
            _ => {
                // No date filter — use the dedicated uncategorized query (no limit)
                tx_repo.find_uncategorized(usize::MAX)?
            }
        }
    };

    if transactions.is_empty() {
        if recategorize {
            output::info("No transactions found for the specified period.");
        } else {
            output::info("No uncategorized transactions found.");
        }
        return Ok(());
    }

    output::kv("Transactions to process", &transactions.len().to_string());
    if recategorize {
        output::warning("--recategorize: will overwrite existing categories where rules match");
    }
    if dry_run {
        output::warning("Dry run — no changes will be saved");
    }
    println!();

    // Run categorization engine on all transactions
    let results = engine.categorize_batch(&transactions);

    let mut matched = 0usize;
    let mut no_match = 0usize;
    let mut changed = 0usize;

    for result in &results {
        let category = match result.category {
            Some(ref cat) => cat,
            None => {
                no_match += 1;
                continue;
            }
        };

        matched += 1;

        // Find the original transaction to check current state
        let tx = match transactions.iter().find(|t| t.id == result.transaction_id) {
            Some(t) => t,
            None => continue,
        };
        let already_same = tx.category_id == Some(category.id);

        if already_same {
            continue;
        }

        changed += 1;

        if dry_run {
            let rule_name = result
                .matched_rule
                .as_ref()
                .map(|r| r.name.as_str())
                .unwrap_or("?");
            println!(
                "  {} {} → {} {}",
                tx.transaction_date.to_string().dimmed(),
                truncate_str(&tx.description, 35),
                category.name.green(),
                format!("(rule: {})", rule_name).dimmed(),
            );
        } else {
            tx_repo.update_category_with_method(
                result.transaction_id,
                category.id,
                result.confidence,
                "rule",
            )?;
        }
    }

    // Summary
    println!();
    println!("{}", "─".repeat(55).dimmed());
    if dry_run {
        println!(
            "  {} matched rules, {} would be {}categorized, {} no match",
            matched.to_string().cyan(),
            changed.to_string().green().bold(),
            if recategorize { "re-" } else { "" },
            no_match.to_string().dimmed(),
        );
        println!();
        output::info("Run without --dry-run to apply changes.");
    } else {
        println!(
            "  {} matched rules, {} {}categorized, {} no match",
            matched.to_string().cyan(),
            changed.to_string().green().bold(),
            if recategorize { "re-" } else { "" },
            no_match.to_string().dimmed(),
        );
        if no_match > 0 {
            println!();
            output::info(&format!(
                "{} transactions still uncategorized. Use `finance tx categorize` for manual review.",
                no_match
            ));
        }
    }

    Ok(())
}

fn handle_delete(id: Option<String>, account: Option<String>, dry_run: bool, conn: &Connection) -> Result<()> {
    use colored::Colorize;

    // Validate that exactly one of --id or --account is provided
    match (&id, &account) {
        (None, None) => return Err(crate::error::Error::InvalidInput(
            "Must provide either --id <ID> or --account <NAME>".to_string()
        )),
        (Some(_), Some(_)) => return Err(crate::error::Error::InvalidInput(
            "Cannot use both --id and --account".to_string()
        )),
        _ => {}
    }

    let tx_repo = TransactionRepository::new(conn);

    if let Some(transaction_id) = id {
        // Delete by transaction ID
        let tx_uuid = uuid::Uuid::parse_str(&transaction_id)
            .map_err(|_| crate::error::Error::InvalidInput(format!("Invalid UUID: {}", transaction_id)))?;

        // Find the transaction to display it
        if let Ok(Some(tx)) = tx_repo.find_by_id(tx_uuid) {
            println!("  {}", format!("Transaction to delete:").bold());
            println!("    Date: {}", tx.transaction_date);
            println!("    Amount: {}", tx.amount);
            println!("    Description: {}", truncate_str(&tx.description, 50));

            if dry_run {
                println!();
                println!("{}", "  [DRY RUN] Would delete this transaction".yellow());
                return Ok(());
            }

            tx_repo.delete_by_id(tx_uuid)?;
            println!();
            println!("{}", "✓ Transaction deleted".green());
        } else {
            return Err(crate::error::Error::InvalidInput(format!("Transaction not found: {}", transaction_id)));
        }
    } else if let Some(account_name) = account {
        // Delete all transactions for account
        let acc_repo = crate::database::AccountRepository::new(conn);
        let found_account = acc_repo.find_by_name(&account_name)?
            .ok_or_else(|| crate::error::Error::InvalidInput(format!("Account not found: {}", account_name)))?;

        println!("{}", format!("Account: {}", found_account.name).bold());

        if dry_run {
            println!("  All transactions for this account would be deleted");
            println!();
            println!("{}", "  [DRY RUN] Would delete all transactions for this account".yellow());
            return Ok(());
        }

        tx_repo.delete_by_account(found_account.id)?;
        println!();
        println!("{}", format!("✓ All transactions deleted for account: {}", found_account.name).green());
    }

    Ok(())
}

/// Truncate a string for display.
fn truncate_str(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        let end = s.char_indices().nth(max - 1).map(|(i, _)| i).unwrap_or(max - 1);
        format!("{}…", &s[..end])
    } else {
        s.to_string()
    }
}
