#!/bin/bash
# =============================================================================
# Finance CLI - Import All Transaction Files
# =============================================================================
# This script imports all QFX and CSV transaction files into the finance-cli
# database. Run from your home directory or wherever finance-cli is accessible.
#
# Before running:
#   1. cargo build --release   (rebuild with parser fixes)
#   2. finance-cli init        (initialize DB + default categories if needed)
#
# Usage: bash import_all.sh /path/to/your/financial/files
# =============================================================================

set -e

DATA_DIR="${1:-$HOME/Personal}"

echo "============================================="
echo "Finance CLI - Bulk Transaction Import"
echo "============================================="
echo "Data directory: $DATA_DIR"
echo ""

# Check that finance-cli is available
if ! command -v finance-cli &> /dev/null; then
    echo "Error: finance-cli not found in PATH."
    echo "Make sure you've run: cargo install --path ."
    exit 1
fi

# Initialize database if needed
echo "--- Initializing database (if needed) ---"
finance-cli init 2>/dev/null || true
echo ""

import_file() {
    local file="$1"
    local label="$2"
    echo "--- Importing: $label ---"
    echo "    File: $(basename "$file")"
    finance-cli transaction import "$file" || echo "    WARNING: Import had errors"
    echo ""
}

# =============================================
# QFX Files - Bank Accounts
# =============================================
echo "============================================="
echo "BANK ACCOUNTS (QFX)"
echo "============================================="
echo ""

# Chase Checking (...2790)
import_file "$DATA_DIR/Chase2790_Activity_20260315.QFX" "Chase Checking (...2790)"

# Citibank Credit Line (...8152) - 2025
import_file "$DATA_DIR/Last year (2025).QFX" "Citibank Credit Line (...8152) - 2025"

# Citibank Credit Line (...8152) - 2026 YTD
import_file "$DATA_DIR/Year to date.QFX" "Citibank Credit Line (...8152) - 2026 YTD"

# =============================================
# QFX Files - AMEX Cards
# =============================================
echo "============================================="
echo "AMEX CARDS (QFX)"
echo "============================================="
echo ""

# AMEX Card ...71001 (5 QFX files)
import_file "$DATA_DIR/activity.qfx" "AMEX Card (...71001) - Period 1"
import_file "$DATA_DIR/activity (1).qfx" "AMEX Card (...71001) - Period 2"
import_file "$DATA_DIR/activity (2).qfx" "AMEX Card (...71001) - Period 3"
import_file "$DATA_DIR/activity (3).qfx" "AMEX Card (...71001) - Period 4"
import_file "$DATA_DIR/activity (4).qfx" "AMEX Card (...71001) - Period 5"

# AMEX Card ...91000 (3 QFX files)
import_file "$DATA_DIR/activity (5).qfx" "AMEX Card (...91000) - Period 1"
import_file "$DATA_DIR/activity (6).qfx" "AMEX Card (...91000) - Period 2"
import_file "$DATA_DIR/activity (7).qfx" "AMEX Card (...91000) - Period 3"

# AMEX Card ...33003 (2 QFX files)
import_file "$DATA_DIR/activity (8).qfx" "AMEX Card (...33003) - Period 1"
import_file "$DATA_DIR/activity (9).qfx" "AMEX Card (...33003) - Period 2"

# AMEX Card ...41004 (8 QFX files)
import_file "$DATA_DIR/activity (10).qfx" "AMEX Card (...41004) - Period 1"
import_file "$DATA_DIR/activity (11).qfx" "AMEX Card (...41004) - Period 2"
import_file "$DATA_DIR/activity (12).qfx" "AMEX Card (...41004) - Period 3"
import_file "$DATA_DIR/activity (13).qfx" "AMEX Card (...41004) - Period 4"
import_file "$DATA_DIR/activity (14).qfx" "AMEX Card (...41004) - Period 5"
import_file "$DATA_DIR/activity (15).qfx" "AMEX Card (...41004) - Period 6"
import_file "$DATA_DIR/activity (16).qfx" "AMEX Card (...41004) - Period 7"
import_file "$DATA_DIR/activity (17).qfx" "AMEX Card (...41004) - Period 8"

# =============================================
# CSV Files
# =============================================
echo "============================================="
echo "CSV FILES"
echo "============================================="
echo ""

# Discover Card
import_file "$DATA_DIR/Discover-AllAvailable-20260315.csv" "Discover Card (CSV)"

# SoFi Individual Cash Account
import_file "$DATA_DIR/Individual Cash Account - All-time.csv" "SoFi Individual Cash Account (CSV)"

# AMEX CSV exports (these overlap with QFX data - dedup will handle it)
import_file "$DATA_DIR/activity.csv" "AMEX CSV Export 1"
import_file "$DATA_DIR/activity (1).csv" "AMEX CSV Export 2"
import_file "$DATA_DIR/activity (2).csv" "AMEX CSV Export 3"
import_file "$DATA_DIR/activity (3).csv" "AMEX CSV Export 4"
import_file "$DATA_DIR/activity (4).csv" "AMEX CSV Export 5"
import_file "$DATA_DIR/activity (5).csv" "AMEX CSV Export 6"
import_file "$DATA_DIR/activity (6).csv" "AMEX CSV Export 7"

# =============================================
# Summary
# =============================================
echo "============================================="
echo "IMPORT COMPLETE"
echo "============================================="
echo ""
finance-cli status
echo ""
echo "Next steps:"
echo "  1. Review: finance-cli transaction list --limit 50"
echo "  2. Categorize: finance-cli transaction categorize --limit 50"
echo "  3. Reports:"
echo "     finance-cli report summary --year 2025"
echo "     finance-cli report pnl --year 2025"
echo "     finance-cli report cashflow --year 2025"
echo "     finance-cli report summary --year 2026"
