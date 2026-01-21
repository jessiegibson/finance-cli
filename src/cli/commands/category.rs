//! Category command handlers.

use crate::config::Config;
use crate::database::{CategoryRepository, Connection};
use crate::error::Result;
use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct CategoryCommand {
    #[command(subcommand)]
    pub action: CategoryAction,
}

#[derive(Subcommand, Debug)]
pub enum CategoryAction {
    /// List all categories
    List {
        /// Show inactive categories too
        #[arg(long)]
        all: bool,
    },

    /// Create a new category
    Create {
        /// Category name
        name: String,

        /// Category type (income, expense, personal)
        #[arg(short, long, default_value = "expense")]
        category_type: String,

        /// Schedule C line mapping
        #[arg(short, long)]
        schedule_c: Option<String>,
    },

    /// Show category rules
    Rules {
        /// Category name or ID
        category: Option<String>,
    },
}

pub fn handle_category(cmd: CategoryCommand, config: &Config, conn: &Connection) -> Result<()> {
    use colored::Colorize;

    match cmd.action {
        CategoryAction::List { all } => {
            println!("{}", "Categories".bold());
            println!();

            let repo = CategoryRepository::new(conn);
            let categories = if all {
                repo.find_all()?
            } else {
                repo.find_active()?
            };

            if categories.is_empty() {
                println!("No categories found. Run 'finance init' to create defaults.");
                return Ok(());
            }

            // Group by type
            let mut income: Vec<_> = categories
                .iter()
                .filter(|c| matches!(c.category_type, crate::models::CategoryType::Income))
                .collect();
            let mut expense: Vec<_> = categories
                .iter()
                .filter(|c| matches!(c.category_type, crate::models::CategoryType::Expense))
                .collect();
            let mut personal: Vec<_> = categories
                .iter()
                .filter(|c| matches!(c.category_type, crate::models::CategoryType::Personal))
                .collect();

            if !income.is_empty() {
                println!("{}", "Income:".green().bold());
                for cat in &income {
                    println!("  {} {}", "•".green(), cat.name);
                }
                println!();
            }

            if !expense.is_empty() {
                println!("{}", "Expense:".red().bold());
                for cat in &expense {
                    let schedule_c = cat
                        .schedule_c_line
                        .as_ref()
                        .map(|s| format!(" [{}]", s))
                        .unwrap_or_default();
                    println!("  {} {}{}", "•".red(), cat.name, schedule_c.dimmed());
                }
                println!();
            }

            if !personal.is_empty() {
                println!("{}", "Personal:".blue().bold());
                for cat in &personal {
                    println!("  {} {}", "•".blue(), cat.name);
                }
            }
        }

        CategoryAction::Create {
            name,
            category_type,
            schedule_c,
        } => {
            println!("{}", "Create Category".bold());
            println!();
            println!("Name: {}", name);
            println!("Type: {}", category_type);
            if let Some(ref sc) = schedule_c {
                println!("Schedule C: {}", sc);
            }

            // TODO: Implement actual category creation
            println!();
            println!("{}", "Category creation coming soon!".yellow());
        }

        CategoryAction::Rules { category } => {
            println!("{}", "Category Rules".bold());
            println!();

            if let Some(cat) = category {
                println!("Category: {}", cat);
            }

            // TODO: Implement rule listing
            println!();
            println!("{}", "Rule listing coming soon!".yellow());
        }
    }

    Ok(())
}
