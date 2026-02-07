# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Converts CRDB Bank (Tanzania) XLS account statements into semicolon-delimited CSV files for ZOHO Books import. Two modes: single-file (`convert.sh`) and batch with deduplication log (`convert_all.sh`).

## Commands

```bash
# Setup
python3 -m venv venv && ./venv/bin/pip install -r requirements.txt

# Batch conversion (recommended) - all XLS in to_convert/ → converted/
./convert_all.sh

# Single file
./convert.sh <xls_file> [output_csv]

# Verify against reference output
diff <(./venv/bin/python3 crdb_to_zoho.py example/202601_Statement_TZS.xls /dev/stdout 2>/dev/null) example/202601_Statement_TZS_ref99_trim.csv

# Reset deduplication log
rm .conversion_log.json
```

Only dependency: `xlrd` (reads legacy Excel 97-2003 CDFV2 `.xls` format).

## Architecture

**`crdb_to_zoho.py`** — Core conversion logic, also used as library by batch_convert.py via `from crdb_to_zoho import convert_xls_to_csv`.

- CRDB XLS structure is fixed: rows 0-13 are header/metadata, row 14 is column headers, rows 15+ are transaction data
- XLS columns: `[0] Posting Date`, `[1] Details`, `[2] Value Date`, `[3] Debit`, `[4] Credit`, `[5] Book Balance`
- Output CSV columns: `Date;Withdrawals;Deposits;Payee;Description;Reference Number`
  - Payee is always empty, Description is always "Transfer"
  - Reference Number = cleaned transaction details (multi-space collapsed, max 99 chars)
- Date transform: `DD.MM.YYYY HH:MM:SS` → `YYYY-MM-DD`
- Amounts: comma-thousands stripped, converted to float
- CSV must use `lineterminator='\n'` (Unix LF) — ZOHO Books requires this

**`batch_convert.py`** — Batch wrapper with SHA256-based deduplication.

- All paths resolve relative to `__file__` (via `SCRIPT_DIR`), so `convert_all.sh` works from any CWD
- Directories `to_convert/` and `converted/` are auto-created
- `.conversion_log.json` maps filename → `{hash, converted_at, output_file}`
- A file is re-converted only when its SHA256 hash differs from the log entry

**Shell wrappers** (`convert.sh`, `convert_all.sh`) — Resolve venv path relative to script location, verify venv exists, delegate to Python.

## Modifying the Code

- **CRDB format changes**: Update row index offset (currently 15) and column indices in `convert_xls_to_csv()`
- **Output format changes**: Modify `convert_xls_to_csv()` in `crdb_to_zoho.py` — batch converter picks it up automatically
- **Batch behavior**: `process_file()` controls per-file logic, `get_xls_files()` controls file discovery
