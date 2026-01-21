//! Output formatting utilities.

use colored::Colorize;

/// Print a success message.
pub fn success(msg: &str) {
    println!("{} {}", "✓".green(), msg);
}

/// Print an error message.
pub fn error(msg: &str) {
    eprintln!("{} {}", "✗".red(), msg);
}

/// Print a warning message.
pub fn warning(msg: &str) {
    println!("{} {}", "!".yellow(), msg);
}

/// Print an info message.
pub fn info(msg: &str) {
    println!("{} {}", "ℹ".blue(), msg);
}

/// Print a section header.
pub fn header(msg: &str) {
    println!();
    println!("{}", msg.bold());
    println!("{}", "─".repeat(msg.len()));
}

/// Print a key-value pair.
pub fn kv(key: &str, value: &str) {
    println!("  {}: {}", key.dimmed(), value);
}

/// Format money for display.
pub fn format_money(amount: &crate::models::Money) -> String {
    if amount.is_expense() {
        format!("-${:.2}", amount.abs().0).red().to_string()
    } else {
        format!("${:.2}", amount.0).green().to_string()
    }
}
