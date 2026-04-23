//! Category command handlers.

use crate::config::Config;
use crate::database::{CategoryRepository, Connection, RuleRepository};
use crate::error::Result;
use crate::models::{Category, CategoryType, ConditionField, LogicalOperator, RuleOperator};
use clap::{Args, Subcommand};
use colored::Colorize;

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

        /// Parent category name (to nest this category underneath)
        #[arg(short, long)]
        parent: Option<String>,
    },

    /// Show category rules
    Rules {
        /// Category name or ID
        category: Option<String>,
    },
}

pub fn handle_category(cmd: CategoryCommand, _config: &Config, conn: &Connection) -> Result<()> {

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
            let income: Vec<_> = categories
                .iter()
                .filter(|c| matches!(c.category_type, crate::models::CategoryType::Income))
                .collect();
            let expense: Vec<_> = categories
                .iter()
                .filter(|c| matches!(c.category_type, crate::models::CategoryType::Expense))
                .collect();
            let personal: Vec<_> = categories
                .iter()
                .filter(|c| matches!(c.category_type, crate::models::CategoryType::Personal))
                .collect();

            // Build parent_id -> children map across the full category set so
            // that a child of a different type (e.g. Co-pays is Expense but
            // nests under the Personal "Healthcare - Personal" parent) still
            // renders under its parent.
            let by_parent: std::collections::HashMap<uuid::Uuid, Vec<&Category>> = {
                let mut map: std::collections::HashMap<uuid::Uuid, Vec<&Category>> =
                    std::collections::HashMap::new();
                for cat in &categories {
                    if let Some(pid) = cat.parent_id {
                        map.entry(pid).or_default().push(cat);
                    }
                }
                for v in map.values_mut() {
                    v.sort_by_key(|c| (c.sort_order, c.name.clone()));
                }
                map
            };

            let render_category = |cat: &Category, bullet: colored::ColoredString| {
                let schedule_c = cat
                    .schedule_c_line
                    .as_ref()
                    .map(|s| format!(" [{}]", s))
                    .unwrap_or_default();
                println!("  {} {}{}", bullet, cat.name, schedule_c.dimmed());
                if let Some(children) = by_parent.get(&cat.id) {
                    for child in children {
                        let child_sc = child
                            .schedule_c_line
                            .as_ref()
                            .map(|s| format!(" [{}]", s))
                            .unwrap_or_default();
                        println!("      {} {}{}", "↳".dimmed(), child.name, child_sc.dimmed());
                    }
                }
            };

            if !income.is_empty() {
                println!("{}", "Income:".green().bold());
                for cat in &income {
                    if cat.parent_id.is_none() {
                        render_category(cat, "•".green());
                    }
                }
                println!();
            }

            if !expense.is_empty() {
                println!("{}", "Expense:".red().bold());
                for cat in &expense {
                    if cat.parent_id.is_none() {
                        render_category(cat, "•".red());
                    }
                }
                println!();
            }

            if !personal.is_empty() {
                println!("{}", "Personal:".blue().bold());
                for cat in &personal {
                    if cat.parent_id.is_none() {
                        render_category(cat, "•".blue());
                    }
                }
            }
        }

        CategoryAction::Create {
            name,
            category_type,
            schedule_c,
            parent,
        } => {
            let repo = CategoryRepository::new(conn);

            // Check for duplicate name
            if let Some(existing) = repo.find_by_name(&name)? {
                println!(
                    "{} Category '{}' already exists (type: {:?}).",
                    "Error:".red().bold(),
                    existing.name,
                    existing.category_type
                );
                return Ok(());
            }

            // Parse category type
            let cat_type = match category_type.to_lowercase().as_str() {
                "income" => CategoryType::Income,
                "expense" => CategoryType::Expense,
                "personal" => CategoryType::Personal,
                other => {
                    println!(
                        "{} Unknown category type '{}'. Use: income, expense, personal.",
                        "Error:".red().bold(),
                        other
                    );
                    return Ok(());
                }
            };

            // Resolve parent if provided
            let parent_cat = match parent.as_deref() {
                Some(parent_name) => match repo.find_by_name(parent_name)? {
                    Some(p) => Some(p),
                    None => {
                        println!(
                            "{} Parent category '{}' not found.",
                            "Error:".red().bold(),
                            parent_name
                        );
                        return Ok(());
                    }
                },
                None => None,
            };

            // Build category
            let mut category = Category::new(&name, cat_type);
            if let Some(ref sc) = schedule_c {
                category = category.with_schedule_c(sc);
            }
            if let Some(ref p) = parent_cat {
                category = category.with_parent(p.id);
            }

            repo.insert(&category)?;

            println!("{}", "Category created".green().bold());
            println!("  Name: {}", name);
            println!("  Type: {}", category_type);
            if let Some(ref sc) = schedule_c {
                println!("  Schedule C: {}", sc);
            }
            if let Some(ref p) = parent_cat {
                println!("  Parent: {}", p.name);
            }
            if category.is_tax_deductible {
                println!("  Tax deductible: {}", "yes".green());
            }
        }

        CategoryAction::Rules { category } => {
            handle_rules(category, conn)?;
        }
    }

    Ok(())
}

fn handle_rules(category_filter: Option<String>, conn: &Connection) -> Result<()> {
    let rule_repo = RuleRepository::new(conn);
    let cat_repo = CategoryRepository::new(conn);

    let all_rules = rule_repo.find_active()?;

    // Filter by category if provided
    let rules: Vec<_> = if let Some(ref filter) = category_filter {
        match cat_repo.find_by_name(filter)? {
            Some(cat) => all_rules
                .into_iter()
                .filter(|r| r.target_category_id == cat.id)
                .collect(),
            None => {
                println!("{}", format!("Category '{}' not found.", filter).yellow());
                return Ok(());
            }
        }
    } else {
        all_rules
    };

    if rules.is_empty() {
        if category_filter.is_some() {
            println!("{}", "No rules for that category.".dimmed());
        } else {
            println!("{}", "No rules yet.".dimmed());
            println!(
                "{}",
                "Create rules during `finance tx categorize`.".dimmed()
            );
        }
        return Ok(());
    }

    // Load categories for name lookup
    let categories = cat_repo.find_all()?;

    println!("{}", "Rules".bold());
    if let Some(ref f) = category_filter {
        println!("{}", format!("Filtered to: {}", f).dimmed());
    }
    println!();

    // Table header
    println!(
        "  {:<30}  {:<8}  {:<40}  {}",
        "Name".bold(),
        "Priority".bold(),
        "Conditions".bold(),
        "Applied".bold(),
    );
    println!("  {}", "─".repeat(90));

    for rule in &rules {
        let cat_name = categories
            .iter()
            .find(|c| c.id == rule.target_category_id)
            .map(|c| c.name.as_str())
            .unwrap_or("?");

        let conditions_str = conditions_display(&rule.conditions);

        let name_display = if rule.conditions.conditions.len() > 0 {
            rule.name.clone()
        } else {
            rule.name.clone()
        };

        println!(
            "  {:<30}  {:<8}  {:<40}  {} → {}",
            truncate(&name_display, 30),
            rule.priority.to_string().dimmed(),
            truncate(&conditions_str, 40),
            rule.effectiveness_count.to_string().cyan(),
            cat_name.green(),
        );
    }

    println!();
    println!("{}", format!("{} rule{}", rules.len(), if rules.len() == 1 { "" } else { "s" }).dimmed());

    Ok(())
}

/// Render RuleConditions as a human-readable string.
fn conditions_display(conditions: &crate::models::RuleConditions) -> String {
    let op_str = match conditions.operator {
        LogicalOperator::And => " AND ",
        LogicalOperator::Or => " OR ",
    };

    let parts: Vec<String> = conditions
        .conditions
        .iter()
        .map(|c| {
            let field = match c.field {
                ConditionField::Description => "description",
                ConditionField::MerchantName => "merchant",
                ConditionField::Amount => "amount",
                ConditionField::AccountId => "account",
                ConditionField::RawCategory => "raw_category",
            };
            let op = match c.operator {
                RuleOperator::Contains => "contains",
                RuleOperator::Equals => "=",
                RuleOperator::StartsWith => "starts with",
                RuleOperator::EndsWith => "ends with",
                RuleOperator::Regex => "matches",
                RuleOperator::GreaterThan => ">",
                RuleOperator::LessThan => "<",
                RuleOperator::Between => "between",
            };
            format!("{} {} '{}'", field, op, c.value)
        })
        .collect();

    parts.join(op_str)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        format!("{}…", &s[..s.char_indices().nth(max - 1).map(|(i, _)| i).unwrap_or(max - 1)])
    } else {
        s.to_string()
    }
}
