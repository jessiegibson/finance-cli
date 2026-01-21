//! Report command handlers.

use crate::config::Config;
use crate::database::Connection;
use crate::error::Result;
use clap::{Args, Subcommand, ValueEnum};

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
    },

    /// Generate Schedule C summary
    ScheduleC {
        /// Tax year
        #[arg(short, long)]
        year: i32,

        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },

    /// Generate summary report
    Summary {
        /// Year for the report
        #[arg(short, long)]
        year: Option<i32>,
    },
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum OutputFormat {
    #[default]
    Table,
    Csv,
    Json,
}

pub fn handle_report(cmd: ReportCommand, config: &Config, conn: &Connection) -> Result<()> {
    use colored::Colorize;

    match cmd.action {
        ReportAction::Pnl { year, format, output } => {
            let year = year.unwrap_or_else(|| chrono::Utc::now().year());
            println!("{}", format!("Profit & Loss Report - {}", year).bold());
            println!();

            // TODO: Implement actual P&L report generation
            println!("{}", "P&L report functionality coming soon!".yellow());
        }

        ReportAction::Cashflow { year, format } => {
            let year = year.unwrap_or_else(|| chrono::Utc::now().year());
            println!("{}", format!("Cash Flow Report - {}", year).bold());
            println!();

            // TODO: Implement actual cash flow report
            println!("{}", "Cash flow report functionality coming soon!".yellow());
        }

        ReportAction::ScheduleC { year, format } => {
            println!("{}", format!("Schedule C Summary - Tax Year {}", year).bold());
            println!();

            // TODO: Implement Schedule C report
            println!("{}", "Schedule C report functionality coming soon!".yellow());
        }

        ReportAction::Summary { year } => {
            let year = year.unwrap_or_else(|| chrono::Utc::now().year());
            println!("{}", format!("Financial Summary - {}", year).bold());
            println!();

            // TODO: Implement summary report
            println!("{}", "Summary report functionality coming soon!".yellow());
        }
    }

    Ok(())
}

use chrono::Datelike;
