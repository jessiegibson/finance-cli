//! Report command handlers.

use crate::calculator::pnl::PnLReport;
use crate::calculator::CashFlowReport;
use crate::calculator::metrics;
use crate::calculator;
use crate::config::Config;
use crate::database::{CategoryRepository, Connection, TransactionRepository};
use crate::error::Result;
use crate::models::{DateRange, Money};
use clap::{Args, Subcommand, ValueEnum};
use std::collections::HashMap;
use serde_json::json;

#[derive(Args, Debug)]
pub struct ReportCommand {
    #[command(subcommand)]
    pub action: ReportAction,
}

#[derive(Subcommand, Debug)]
pub enum ReportAction {
    /// Generate Profit & Loss report
    Pnl {
        /// Year for the report
        #[arg(short, long)]
        year: Option<i32>,

        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,

        /// Output file (if not specified, prints to stdout)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },

    /// Generate Cash Flow report
    Cashflow {
        /// Year for the report
        #[arg(short, long)]
        year: Option<i32>,

        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,

        /// Output file (if not specified, prints to stdout)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },

    /// Generate Schedule C summary (self-employment)
    ScheduleC {
        /// Tax year
        #[arg(short, long)]
        year: i32,

        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,

        /// Output file (if not specified, prints to stdout)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },

    /// Generate Schedule A summary (itemized deductions)
    ScheduleA {
        /// Tax year
        #[arg(short, long)]
        year: i32,

        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,

        /// Output file (if not specified, prints to stdout)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },

    /// Generate Schedule E summary (rental real estate income)
    ScheduleE {
        /// Tax year
        #[arg(short, long)]
        year: i32,

        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,

        /// Output file (if not specified, prints to stdout)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },

    /// Compare tax data across years (cross-check returns)
    TaxCompare {
        /// Years to compare (comma-separated, e.g. "2023,2024,2025")
        #[arg(short = 'Y', long)]
        years: String,

        /// Which schedule to compare (a, c, e, or all)
        #[arg(short, long, default_value = "all")]
        schedule: String,
    },

    /// Generate summary report
    Summary {
        /// Year for the report
        #[arg(short, long)]
        year: Option<i32>,

        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,

        /// Output file (if not specified, prints to stdout)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum OutputFormat {
    #[default]
    Table,
    Csv,
    Json,
}

pub fn handle_report(cmd: ReportCommand, _config: &Config, conn: &Connection) -> Result<()> {
    match cmd.action {
        ReportAction::Pnl { year, format, output } => {
            let year = year.unwrap_or_else(|| chrono::Utc::now().year());
            handle_pnl(conn, year, format, output)
        }

        ReportAction::Cashflow { year, format, output } => {
            let year = year.unwrap_or_else(|| chrono::Utc::now().year());
            handle_cashflow(conn, year, format, output)
        }

        ReportAction::ScheduleC { year, format, output } => {
            handle_schedule_report(conn, year, "C", "Schedule C - Self-Employment", format, output)
        }

        ReportAction::ScheduleA { year, format, output } => {
            handle_schedule_report(conn, year, "A", "Schedule A - Itemized Deductions", format, output)
        }

        ReportAction::ScheduleE { year, format, output } => {
            handle_schedule_report(conn, year, "E", "Schedule E - Rental Real Estate", format, output)
        }

        ReportAction::TaxCompare { years, schedule } => {
            handle_tax_compare(conn, &years, &schedule)
        }

        ReportAction::Summary { year, format, output } => {
            let year = year.unwrap_or_else(|| chrono::Utc::now().year());
            handle_summary(conn, year, format, output)
        }
    }
}

use chrono::Datelike;
use std::path::PathBuf;
use crate::cli::output;

// ─── CSV/JSON Export Helpers ──────────────────────────────────

/// Write rows to CSV — file if output path given, stdout otherwise.
fn write_csv(headers: &[&str], rows: Vec<Vec<String>>, output: Option<&PathBuf>) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(headers)
        .map_err(|e| crate::error::Error::Report(e.to_string()))?;
    for row in rows {
        wtr.write_record(&row)
            .map_err(|e| crate::error::Error::Report(e.to_string()))?;
    }
    let data = wtr.into_inner()
        .map_err(|e| crate::error::Error::Report(e.to_string()))?;

    match output {
        Some(path) => {
            std::fs::write(path, &data)
                .map_err(|e| crate::error::Error::Io { path: path.clone(), source: e })?;
            output::success(&format!("Saved to {}", path.display()));
        }
        None => print!("{}", String::from_utf8_lossy(&data)),
    }
    Ok(())
}

/// Write JSON — file if output path given, stdout otherwise.
fn write_json(value: serde_json::Value, output: Option<&PathBuf>) -> Result<()> {
    let data = serde_json::to_string_pretty(&value)
        .map_err(|e| crate::error::Error::Report(e.to_string()))?;
    match output {
        Some(path) => {
            std::fs::write(path, &data)
                .map_err(|e| crate::error::Error::Io { path: path.clone(), source: e })?;
            output::success(&format!("Saved to {}", path.display()));
        }
        None => println!("{}", data),
    }
    Ok(())
}

// ─── P&L Report ───────────────────────────────────────────────

fn handle_pnl(conn: &Connection, year: i32, format: OutputFormat, output: Option<PathBuf>) -> Result<()> {
    use colored::Colorize;

    let date_range = DateRange::year(year)
        .ok_or_else(|| crate::error::Error::InvalidInput(format!("Invalid year: {}", year)))?;

    let tx_repo = TransactionRepository::new(conn);
    let cat_repo = CategoryRepository::new(conn);
    let transactions = tx_repo.find_by_date_range(&date_range)?;
    let categories = cat_repo.find_all()?;

    if transactions.is_empty() {
        println!("{}", format!("No transactions found for {}", year).yellow());
        return Ok(());
    }

    let report = PnLReport::generate(&transactions, &categories, date_range);

    match format {
        OutputFormat::Table => {
            println!("{}", format!("═══ Profit & Loss Report - {} ═══", year).bold());
            println!();

    // Income section
    println!("{}", "INCOME".green().bold());
    println!("{}", "─".repeat(50));
    for item in report.income_sorted() {
        println!(
            "  {:<35} {:>12}  ({})",
            item.category_name,
            format!("{}", item.total).green(),
            item.transaction_count
        );
    }
    println!("{}", "─".repeat(50));
    println!(
        "  {:<35} {:>12}",
        "Total Income".bold(),
        format!("{}", report.total_income).green().bold()
    );
    println!();

    // Expenses section
    println!("{}", "EXPENSES".red().bold());
    println!("{}", "─".repeat(50));
    for item in report.expenses_sorted() {
        println!(
            "  {:<35} {:>12}  ({})",
            item.category_name,
            format!("{}", item.total).red(),
            item.transaction_count
        );
    }
    println!("{}", "─".repeat(50));
    println!(
        "  {:<35} {:>12}",
        "Total Expenses".bold(),
        format!("{}", report.total_expenses).red().bold()
    );
    println!();

    // Net
    println!("{}", "═".repeat(50));
    if report.is_profitable() {
        println!(
            "  {:<35} {:>12}",
            "NET PROFIT".bold(),
            format!("{}", report.net_profit).green().bold()
        );
    } else {
        println!(
            "  {:<35} {:>12}",
            "NET LOSS".bold(),
            format!("{}", report.net_profit).red().bold()
        );
    }

            // Warn about uncategorized
            let uncategorized_count = transactions.iter().filter(|t| t.category_id.is_none()).count();
            if uncategorized_count > 0 {
                println!();
                println!(
                    "{}",
                    format!(
                        "Warning: {} transactions are uncategorized and excluded from this report.",
                        uncategorized_count
                    )
                    .yellow()
                );
                println!(
                    "{}",
                    "Run 'finance transaction categorize' to assign categories.".yellow()
                );
            }
        }
        OutputFormat::Csv => {
            let headers = vec!["type", "category", "schedule_line", "amount", "transaction_count"];
            let mut rows = Vec::new();

            for item in report.income_sorted() {
                rows.push(vec![
                    "income".to_string(),
                    item.category_name.clone(),
                    "".to_string(),
                    format!("{}", item.total),
                    item.transaction_count.to_string(),
                ]);
            }

            for item in report.expenses_sorted() {
                rows.push(vec![
                    "expense".to_string(),
                    item.category_name.clone(),
                    "".to_string(),
                    format!("{}", item.total),
                    item.transaction_count.to_string(),
                ]);
            }

            rows.push(vec![
                "net".to_string(),
                "".to_string(),
                "".to_string(),
                format!("{}", report.net_profit),
                "".to_string(),
            ]);

            write_csv(&headers, rows, output.as_ref())?;
        }
        OutputFormat::Json => {
            let income_items: Vec<_> = report.income_sorted()
                .iter()
                .map(|item| json!({
                    "category": &item.category_name,
                    "amount": item.total.to_string(),
                    "transactions": item.transaction_count
                }))
                .collect();

            let expense_items: Vec<_> = report.expenses_sorted()
                .iter()
                .map(|item| json!({
                    "category": &item.category_name,
                    "amount": item.total.to_string(),
                    "transactions": item.transaction_count
                }))
                .collect();

            let json_output = json!({
                "period": year.to_string(),
                "total_income": report.total_income.to_string(),
                "total_expenses": report.total_expenses.to_string(),
                "net_profit": report.net_profit.to_string(),
                "income": income_items,
                "expenses": expense_items
            });

            write_json(json_output, output.as_ref())?;
        }
    }

    Ok(())
}

// ─── Cash Flow Report ─────────────────────────────────────────

fn handle_cashflow(conn: &Connection, year: i32, format: OutputFormat, output: Option<PathBuf>) -> Result<()> {
    use colored::Colorize;

    let date_range = DateRange::year(year)
        .ok_or_else(|| crate::error::Error::InvalidInput(format!("Invalid year: {}", year)))?;

    let tx_repo = TransactionRepository::new(conn);
    let transactions = tx_repo.find_by_date_range(&date_range)?;

    if transactions.is_empty() {
        println!("{}", format!("No transactions found for {}", year).yellow());
        return Ok(());
    }

    let report = CashFlowReport::generate(&transactions, date_range);
    let monthly = report.monthly_summary();

    match format {
        OutputFormat::Table => {
            println!("{}", format!("═══ Cash Flow Report - {} ═══", year).bold());
            println!();

    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun",
        "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    println!(
        "  {:<8} {:>14} {:>14} {:>14}",
        "Month".bold(),
        "Inflows".green().bold(),
        "Outflows".red().bold(),
        "Net".bold()
    );
    println!("  {}", "─".repeat(52));

    for month_num in 1..=12u32 {
        if let Some(m) = monthly.get(&(year, month_num)) {
            let net_str = format!("{}", m.net);
            let net_colored = if m.net.is_income() {
                net_str.green()
            } else {
                net_str.red()
            };
            println!(
                "  {:<8} {:>14} {:>14} {:>14}",
                months[(month_num - 1) as usize],
                format!("{}", m.inflows).green(),
                format!("{}", m.outflows).red(),
                net_colored
            );
        }
    }

            println!("  {}", "─".repeat(52));
            let net_str = format!("{}", report.net_cash_flow);
            let net_colored = if report.net_cash_flow.is_income() {
                net_str.green()
            } else {
                net_str.red()
            };
            println!(
                "  {:<8} {:>14} {:>14} {:>14}",
                "Total".bold(),
                format!("{}", report.total_inflows).green().bold(),
                format!("{}", report.total_outflows).red().bold(),
                net_colored.bold()
            );
        }
        OutputFormat::Csv => {
            let headers = vec!["year", "month", "month_name", "inflows", "outflows", "net"];
            let mut rows = Vec::new();
            let months = [
                "January", "February", "March", "April", "May", "June",
                "July", "August", "September", "October", "November", "December",
            ];

            for month_num in 1..=12u32 {
                if let Some(m) = monthly.get(&(year, month_num)) {
                    rows.push(vec![
                        year.to_string(),
                        month_num.to_string(),
                        months[(month_num - 1) as usize].to_string(),
                        format!("{}", m.inflows),
                        format!("{}", m.outflows),
                        format!("{}", m.net),
                    ]);
                }
            }

            write_csv(&headers, rows, output.as_ref())?;
        }
        OutputFormat::Json => {
            let months = [
                "January", "February", "March", "April", "May", "June",
                "July", "August", "September", "October", "November", "December",
            ];
            let mut monthly_items = Vec::new();

            for month_num in 1..=12u32 {
                if let Some(m) = monthly.get(&(year, month_num)) {
                    monthly_items.push(json!({
                        "year": year,
                        "month": month_num,
                        "month_name": months[(month_num - 1) as usize],
                        "inflows": m.inflows.to_string(),
                        "outflows": m.outflows.to_string(),
                        "net": m.net.to_string()
                    }));
                }
            }

            let json_output = json!({
                "period": year.to_string(),
                "total_inflows": report.total_inflows.to_string(),
                "total_outflows": report.total_outflows.to_string(),
                "net_cash_flow": report.net_cash_flow.to_string(),
                "monthly": monthly_items
            });

            write_json(json_output, output.as_ref())?;
        }
    }

    Ok(())
}

// ─── Schedule Reports (A, C, E) ──────────────────────────────

/// IRS line item labels for each schedule.
fn schedule_line_labels(schedule: &str) -> Vec<(&'static str, &'static str)> {
    match schedule {
        "A" => vec![
            ("A-1",  "Medical and dental expenses"),
            ("A-5a", "State/local income taxes"),
            ("A-5b", "Personal property taxes"),
            ("A-5c", "Real estate taxes"),
            ("A-8a", "Home mortgage interest"),
            ("A-10", "Mortgage insurance premiums"),
            ("A-12", "Charitable contributions - cash"),
            ("A-13", "Charitable contributions - non-cash"),
            ("A-15", "Casualty and theft losses"),
            ("A-16", "Other itemized deductions"),
        ],
        "C" => vec![
            ("L8",   "Advertising"),
            ("L9",   "Car and truck expenses"),
            ("L10",  "Commissions and fees"),
            ("L11",  "Contract labor"),
            ("L15",  "Insurance"),
            ("L17",  "Legal and professional services"),
            ("L18",  "Office expense"),
            ("L20b", "Rent or lease - other business property"),
            ("L22",  "Supplies"),
            ("L24a", "Travel"),
            ("L24b", "Meals (subject to 50% limitation)"),
            ("L25",  "Utilities"),
            ("L27a", "Other expenses"),
        ],
        "E" => vec![
            ("E-3",  "Rents received"),
            ("E-5",  "Advertising"),
            ("E-6",  "Auto and travel"),
            ("E-7",  "Cleaning and maintenance"),
            ("E-8",  "Commissions"),
            ("E-9",  "Insurance"),
            ("E-10", "Legal and professional fees"),
            ("E-11", "Management fees"),
            ("E-12", "Mortgage interest paid"),
            ("E-13", "Other interest"),
            ("E-14", "Repairs"),
            ("E-15", "Supplies"),
            ("E-16", "Taxes"),
            ("E-17", "Utilities"),
            ("E-18", "Depreciation expense"),
            ("E-19", "Other"),
        ],
        _ => vec![],
    }
}

fn handle_schedule_report(conn: &Connection, year: i32, schedule: &str, title: &str, format: OutputFormat, output: Option<PathBuf>) -> Result<()> {
    use colored::Colorize;

    let date_range = DateRange::year(year)
        .ok_or_else(|| crate::error::Error::InvalidInput(format!("Invalid year: {}", year)))?;

    let tx_repo = TransactionRepository::new(conn);
    let cat_repo = CategoryRepository::new(conn);
    let transactions = tx_repo.find_by_date_range(&date_range)?;
    let categories = cat_repo.find_all()?;

    if transactions.is_empty() {
        println!("{}", format!("No transactions found for {}", year).yellow());
        return Ok(());
    }

    // Build a map: schedule line code -> (category_name, category_id)
    let mut line_to_category: HashMap<String, (String, uuid::Uuid)> = HashMap::new();
    for cat in &categories {
        if let Some(ref line) = cat.schedule_c_line {
            let prefix = if schedule == "C" {
                // Schedule C lines are stored as "L8", "L9", etc. (no prefix)
                line.starts_with('L')
            } else {
                line.starts_with(&format!("{}-", schedule))
            };
            if prefix {
                line_to_category.insert(line.clone(), (cat.name.clone(), cat.id));
            }
        }
    }

    // Aggregate amounts by schedule line
    let cat_totals = calculator::aggregate_by_category(&transactions);

    // Map category totals to schedule lines
    let mut line_totals: HashMap<String, (Money, usize)> = HashMap::new();
    for (cat_id, total) in &cat_totals {
        // Find the schedule line for this category
        for cat in &categories {
            if cat.id == *cat_id {
                if let Some(ref line) = cat.schedule_c_line {
                    let is_match = if schedule == "C" {
                        line.starts_with('L')
                    } else {
                        line.starts_with(&format!("{}-", schedule))
                    };
                    if is_match {
                        let entry = line_totals.entry(line.clone()).or_insert((Money::zero(), 0));
                        entry.0 += *total;
                        // Count transactions for this category
                        let count = transactions
                            .iter()
                            .filter(|t| t.category_id == Some(cat.id))
                            .count();
                        entry.1 += count;
                    }
                }
                break;
            }
        }
    }

    // Also count transactions tagged at the transaction level (schedule_c_line field)
    for tx in &transactions {
        if let Some(ref line) = tx.schedule_c_line {
            let is_match = if schedule == "C" {
                line.starts_with('L')
            } else {
                line.starts_with(&format!("{}-", schedule))
            };
            if is_match && tx.category_id.is_none() {
                let entry = line_totals.entry(line.clone()).or_insert((Money::zero(), 0));
                entry.0 += tx.amount;
                entry.1 += 1;
            }
        }
    }

    let labels = schedule_line_labels(schedule);

    match format {
        OutputFormat::Table => {
            println!(
                "{}",
                format!("═══ {} - Tax Year {} ═══", title, year).bold()
            );
            println!();
            let mut grand_total = Money::zero();
            let mut income_total = Money::zero();
            let mut expense_total = Money::zero();

            println!(
                "  {:<8} {:<40} {:>12} {:>6}",
                "Line".bold(),
                "Description".bold(),
                "Amount".bold(),
                "Txns".bold()
            );
            println!("  {}", "─".repeat(68));

            for (line_code, description) in &labels {
        let (amount, count) = line_totals
            .get(*line_code)
            .copied()
            .unwrap_or((Money::zero(), 0));

        if amount.is_income() {
            income_total += amount;
        } else {
            expense_total += amount;
        }
        grand_total += amount;

                let amount_str = if count > 0 {
                    let s = format!("{}", amount);
                    if amount.is_income() {
                        s.green().to_string()
                    } else {
                        s.red().to_string()
                    }
                } else {
                    "$0.00".dimmed().to_string()
                };

                // Extract the line number for display
                let display_line = if schedule == "C" {
                    line_code.to_string()
                } else {
                    line_code.replace(&format!("{}-", schedule), "L")
                };

                println!(
                    "  {:<8} {:<40} {:>12} {:>6}",
                    display_line,
                    description,
                    amount_str,
                    if count > 0 {
                        count.to_string()
                    } else {
                        "-".dimmed().to_string()
                    }
                );
            }

            println!("  {}", "─".repeat(68));

            // Print totals based on schedule type
            match schedule {
                "E" => {
                    println!(
                        "  {:<49} {:>12}",
                        "Total Rental Income".bold(),
                        format!("{}", income_total).green().bold()
                    );
                    println!(
                        "  {:<49} {:>12}",
                        "Total Rental Expenses".bold(),
                        format!("{}", expense_total).red().bold()
                    );
                    let net = income_total + expense_total;
                    let label = if net.is_income() {
                        "Net Rental Income"
                    } else {
                        "Net Rental Loss"
                    };
                    let net_str = format!("{}", net);
                    let colored = if net.is_income() {
                        net_str.green().bold()
                    } else {
                        net_str.red().bold()
                    };
                    println!("  {:<49} {:>12}", label.bold(), colored);
                }
                "A" => {
                    println!(
                        "  {:<49} {:>12}",
                        "Total Itemized Deductions".bold(),
                        format!("{}", expense_total.abs()).bold()
                    );
                    println!();
                    println!(
                        "{}",
                        "Note: Medical expenses are subject to 7.5% AGI floor.".dimmed()
                    );
                    println!(
                        "{}",
                        "State/local taxes are capped at $10,000 (SALT cap).".dimmed()
                    );
                }
                _ => {
                    println!(
                        "  {:<49} {:>12}",
                        "Total".bold(),
                        format!("{}", grand_total).bold()
                    );
                }
            }

            // Show uncategorized warning
            let uncategorized = transactions
                .iter()
                .filter(|t| t.category_id.is_none() && t.schedule_c_line.is_none())
                .count();
            if uncategorized > 0 {
                println!();
                println!(
                    "{}",
                    format!(
                        "Warning: {} transactions are uncategorized. Some deductions are likely missing.",
                        uncategorized
                    )
                    .yellow()
                );
                println!(
                    "{}",
                    "Run 'finance transaction categorize' to assign categories.".yellow()
                );
            }
        }
        OutputFormat::Csv => {
            let headers = vec!["line", "description", "amount", "transaction_count"];
            let mut rows = Vec::new();

            for (line_code, description) in &labels {
                let (amount, count) = line_totals
                    .get(*line_code)
                    .copied()
                    .unwrap_or((Money::zero(), 0));

                if count > 0 {
                    rows.push(vec![
                        line_code.to_string(),
                        description.to_string(),
                        format!("{}", amount),
                        count.to_string(),
                    ]);
                }
            }

            write_csv(&headers, rows, output.as_ref())?;
        }
        OutputFormat::Json => {
            let mut line_items = Vec::new();

            for (line_code, description) in &labels {
                let (amount, count) = line_totals
                    .get(*line_code)
                    .copied()
                    .unwrap_or((Money::zero(), 0));

                if count > 0 {
                    line_items.push(json!({
                        "line": line_code.to_string(),
                        "description": description.to_string(),
                        "amount": amount.to_string(),
                        "transactions": count
                    }));
                }
            }

            let json_output = json!({
                "schedule": schedule,
                "year": year,
                "lines": line_items
            });

            write_json(json_output, output.as_ref())?;
        }
    }

    Ok(())
}

// ─── Tax Year Comparison ──────────────────────────────────────

fn handle_tax_compare(conn: &Connection, years_str: &str, schedule: &str) -> Result<()> {
    use colored::Colorize;

    let years: Vec<i32> = years_str
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    if years.is_empty() {
        println!(
            "{}",
            "Please provide valid years, e.g. --years 2023,2024,2025".red()
        );
        return Ok(());
    }

    let schedules: Vec<&str> = if schedule == "all" {
        vec!["A", "C", "E"]
    } else {
        vec![schedule]
    };

    let cat_repo = CategoryRepository::new(conn);
    let tx_repo = TransactionRepository::new(conn);
    let categories = cat_repo.find_all()?;

    for sched in &schedules {
        let labels = schedule_line_labels(sched);
        if labels.is_empty() {
            continue;
        }

        let title = match *sched {
            "A" => "Schedule A - Itemized Deductions",
            "C" => "Schedule C - Self-Employment",
            "E" => "Schedule E - Rental Real Estate",
            _ => "Unknown Schedule",
        };

        println!();
        println!(
            "{}",
            format!("═══ {} - Year Comparison ═══", title).bold()
        );
        println!();

        // Header row
        let year_width = 14;
        print!("  {:<8} {:<32}", "Line".bold(), "Description".bold());
        for y in &years {
            print!(" {:>width$}", format!("{}", y).bold(), width = year_width);
        }
        if years.len() >= 2 {
            print!(" {:>width$}", "Change".bold(), width = year_width);
        }
        println!();
        print!("  {}", "─".repeat(42));
        for _ in &years {
            print!("{}", "─".repeat(year_width + 1));
        }
        if years.len() >= 2 {
            print!("{}", "─".repeat(year_width + 1));
        }
        println!();

        // Gather data per year
        let mut year_data: HashMap<i32, HashMap<String, Money>> = HashMap::new();
        for y in &years {
            let range = DateRange::year(*y);
            if let Some(range) = range {
                let txs = tx_repo.find_by_date_range(&range)?;
                let cat_totals = calculator::aggregate_by_category(&txs);

                let mut line_map: HashMap<String, Money> = HashMap::new();
                for (cat_id, total) in &cat_totals {
                    for cat in &categories {
                        if cat.id == *cat_id {
                            if let Some(ref line) = cat.schedule_c_line {
                                let is_match = if *sched == "C" {
                                    line.starts_with('L')
                                } else {
                                    line.starts_with(&format!("{}-", sched))
                                };
                                if is_match {
                                    let entry =
                                        line_map.entry(line.clone()).or_insert(Money::zero());
                                    *entry += *total;
                                }
                            }
                            break;
                        }
                    }
                }

                // Also gather from transaction-level schedule_c_line
                for tx in &txs {
                    if let Some(ref line) = tx.schedule_c_line {
                        let is_match = if *sched == "C" {
                            line.starts_with('L')
                        } else {
                            line.starts_with(&format!("{}-", sched))
                        };
                        if is_match && tx.category_id.is_none() {
                            let entry = line_map.entry(line.clone()).or_insert(Money::zero());
                            *entry += tx.amount;
                        }
                    }
                }

                year_data.insert(*y, line_map);
            }
        }

        // Print each line
        let mut year_totals: HashMap<i32, Money> = HashMap::new();
        for y in &years {
            year_totals.insert(*y, Money::zero());
        }

        for (line_code, description) in &labels {
            let display_line = if *sched == "C" {
                line_code.to_string()
            } else {
                line_code.replace(&format!("{}-", sched), "L")
            };

            print!("  {:<8} {:<32}", display_line, description);

            let mut amounts: Vec<Money> = Vec::new();
            for y in &years {
                let amount = year_data
                    .get(y)
                    .and_then(|m| m.get(*line_code))
                    .copied()
                    .unwrap_or(Money::zero());

                *year_totals.entry(*y).or_insert(Money::zero()) += amount;
                amounts.push(amount);

                let s = format!("{}", amount);
                let colored = if amount.is_expense() {
                    s.red()
                } else if amount.is_income() {
                    s.green()
                } else {
                    s.dimmed()
                };
                print!(" {:>width$}", colored, width = year_width);
            }

            // Change column (last year vs first year)
            if years.len() >= 2 {
                let first = amounts[0];
                let last = amounts[amounts.len() - 1];
                let diff = last - first;
                let diff_str = format!("{}", diff);
                let colored = if diff.is_income() {
                    diff_str.green()
                } else if diff.is_expense() {
                    diff_str.red()
                } else {
                    diff_str.dimmed()
                };
                print!(" {:>width$}", colored, width = year_width);
            }
            println!();
        }

        // Totals row
        print!("  {}", "─".repeat(42));
        for _ in &years {
            print!("{}", "─".repeat(year_width + 1));
        }
        if years.len() >= 2 {
            print!("{}", "─".repeat(year_width + 1));
        }
        println!();

        print!("  {:<8} {:<32}", "", "TOTAL".bold());
        let mut total_amounts: Vec<Money> = Vec::new();
        for y in &years {
            let total = year_totals.get(y).copied().unwrap_or(Money::zero());
            total_amounts.push(total);
            print!(
                " {:>width$}",
                format!("{}", total).bold(),
                width = year_width
            );
        }
        if years.len() >= 2 {
            let diff = total_amounts[total_amounts.len() - 1] - total_amounts[0];
            let diff_str = format!("{}", diff);
            let colored = if diff.is_income() {
                diff_str.green().bold()
            } else {
                diff_str.red().bold()
            };
            print!(" {:>width$}", colored, width = year_width);
        }
        println!();
    }

    println!();
    println!(
        "{}",
        "Tip: Compare these totals against your filed returns to find discrepancies.".dimmed()
    );

    Ok(())
}

// ─── Summary Report ───────────────────────────────────────────

fn handle_summary(conn: &Connection, year: i32, format: OutputFormat, output: Option<PathBuf>) -> Result<()> {
    use colored::Colorize;

    let date_range = DateRange::year(year)
        .ok_or_else(|| crate::error::Error::InvalidInput(format!("Invalid year: {}", year)))?;

    let tx_repo = TransactionRepository::new(conn);
    let cat_repo = CategoryRepository::new(conn);
    let transactions = tx_repo.find_by_date_range(&date_range)?;
    let categories = cat_repo.find_all()?;

    if transactions.is_empty() {
        println!("{}", format!("No transactions found for {}", year).yellow());
        return Ok(());
    }

    // Calculate metrics before matching on format
    let counts = metrics::transaction_counts(&transactions);
    let total_in = calculator::total_income(&transactions);
    let total_out = calculator::total_expenses(&transactions);
    let net = calculator::net_total(&transactions);
    let avg_monthly = metrics::average_monthly_expenses(&transactions);
    let report = PnLReport::generate(&transactions, &categories, date_range);
    let top_expenses = report.expenses_sorted();

    match format {
        OutputFormat::Table => {
            println!("{}", format!("═══ Financial Summary - {} ═══", year).bold());
            println!();

            // Transaction counts
            println!("{}", "Transaction Overview".bold());
            println!("  Total transactions:    {}", counts.total);
            println!("  Income transactions:   {}", counts.income);
            println!("  Expense transactions:  {}", counts.expense);
            println!(
                "  Categorized:           {} ({:.1}%)",
                counts.categorized,
                if counts.total > 0 {
                    counts.categorized as f64 / counts.total as f64 * 100.0
                } else {
                    0.0
                }
            );
            println!("  Uncategorized:         {}", counts.uncategorized);
            println!();

            // Income / Expenses / Net
            println!("{}", "Financial Totals".bold());
            println!("  Total Income:          {}", format!("{}", total_in).green());
            println!(
                "  Total Expenses:        {}",
                format!("{}", total_out).red()
            );
            let net_str = format!("{}", net);
            if net.is_income() {
                println!("  Net:                   {}", net_str.green().bold());
            } else {
                println!("  Net:                   {}", net_str.red().bold());
            }
            println!();

            // Average monthly
            println!(
                "  Avg Monthly Spending:  {}",
                format!("{}", avg_monthly).red()
            );
            println!();

            // Top 10 expense categories
            if !top_expenses.is_empty() {
                println!("{}", "Top 10 Expense Categories".bold());
                for (i, item) in top_expenses.iter().take(10).enumerate() {
                    println!(
                        "  {}. {:<30} {:>12}  ({} txns)",
                        i + 1,
                        item.category_name,
                        format!("{}", item.total).red(),
                        item.transaction_count
                    );
                }
                println!();
            }

            // Largest transactions
            if let Some(biggest_expense) = metrics::largest_expense(&transactions) {
                println!("{}", "Notable Transactions".bold());
                println!(
                    "  Largest expense: {} on {} - {}",
                    format!("{}", biggest_expense.amount).red(),
                    biggest_expense.transaction_date,
                    biggest_expense.description
                );
            }
            if let Some(biggest_income) = metrics::largest_income(&transactions) {
                println!(
                    "  Largest income:  {} on {} - {}",
                    format!("{}", biggest_income.amount).green(),
                    biggest_income.transaction_date,
                    biggest_income.description
                );
            }

            // Tax schedule summary
            println!();
            println!("{}", "Tax Schedule Coverage".bold());

            let schedule_a_count: usize = categories
                .iter()
                .filter(|c| {
                    c.schedule_c_line
                        .as_ref()
                        .map(|l| l.starts_with("A-"))
                        .unwrap_or(false)
                })
                .map(|c| {
                    transactions
                        .iter()
                        .filter(|t| t.category_id == Some(c.id))
                        .count()
                })
                .sum();

            let schedule_c_count: usize = categories
                .iter()
                .filter(|c| {
                    c.schedule_c_line
                        .as_ref()
                        .map(|l| l.starts_with('L'))
                        .unwrap_or(false)
                })
                .map(|c| {
                    transactions
                        .iter()
                        .filter(|t| t.category_id == Some(c.id))
                        .count()
                })
                .sum();

            let schedule_e_count: usize = categories
                .iter()
                .filter(|c| {
                    c.schedule_c_line
                        .as_ref()
                        .map(|l| l.starts_with("E-"))
                        .unwrap_or(false)
                })
                .map(|c| {
                    transactions
                        .iter()
                        .filter(|t| t.category_id == Some(c.id))
                        .count()
                })
                .sum();

            println!(
                "  Schedule A (Itemized Deductions):  {} categorized transactions",
                schedule_a_count
            );
            println!(
                "  Schedule C (Self-Employment):      {} categorized transactions",
                schedule_c_count
            );
            println!(
                "  Schedule E (Rental Real Estate):   {} categorized transactions",
                schedule_e_count
            );

            if schedule_a_count == 0 && schedule_c_count == 0 && schedule_e_count == 0 {
                println!();
                println!(
                    "{}",
                    "No transactions are mapped to tax schedules yet.".yellow()
                );
                println!(
                    "{}",
                    "Categorize transactions to populate Schedule A, C, and E reports.".yellow()
                );
            }
        }
        OutputFormat::Csv => {
            let headers = vec!["metric", "value"];
            let mut rows = Vec::new();

            rows.push(vec!["total_transactions".to_string(), counts.total.to_string()]);
            rows.push(vec!["income_transactions".to_string(), counts.income.to_string()]);
            rows.push(vec!["expense_transactions".to_string(), counts.expense.to_string()]);
            rows.push(vec!["categorized".to_string(), counts.categorized.to_string()]);
            rows.push(vec!["uncategorized".to_string(), counts.uncategorized.to_string()]);
            rows.push(vec!["total_income".to_string(), format!("{}", total_in)]);
            rows.push(vec!["total_expenses".to_string(), format!("{}", total_out)]);
            rows.push(vec!["net".to_string(), format!("{}", net)]);
            rows.push(vec!["avg_monthly_expenses".to_string(), format!("{}", avg_monthly)]);
            if !top_expenses.is_empty() {
                rows.push(vec!["top_expense_category".to_string(), top_expenses[0].category_name.clone()]);
            }

            write_csv(&headers, rows, output.as_ref())?;
        }
        OutputFormat::Json => {
            let json_output = json!({
                "year": year,
                "transactions": {
                    "total": counts.total,
                    "income": counts.income,
                    "expense": counts.expense,
                    "categorized": counts.categorized,
                    "uncategorized": counts.uncategorized
                },
                "financial": {
                    "total_income": total_in.to_string(),
                    "total_expenses": total_out.to_string(),
                    "net": net.to_string(),
                    "avg_monthly_expenses": avg_monthly.to_string()
                },
                "top_expenses": top_expenses.iter().take(10).map(|item| json!({
                    "category": item.category_name,
                    "amount": item.total.to_string(),
                    "transactions": item.transaction_count
                })).collect::<Vec<_>>()
            });

            write_json(json_output, output.as_ref())?;
        }
    }

    Ok(())
}
