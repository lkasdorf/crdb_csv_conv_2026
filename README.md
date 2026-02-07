# CRDB Bank to ZOHO Books CSV Converter

This script converts CRDB Bank (Tanzania) account statements in XLS format into CSV files that can be imported into ZOHO Books.

## Features

- Converts CRDB Bank XLS files to ZOHO Books CSV format
- Cleans up whitespace in descriptions
- Truncates descriptions to a maximum of 99 characters
- Converts date format to YYYY-MM-DD
- Removes thousands separators from amounts

## Installation

1. Install Python 3 (if not already available)
2. Create a virtual environment and install dependencies:

```bash
python3 -m venv venv
./venv/bin/pip install -r requirements.txt
```

## Usage

### Method 1: Batch Conversion (Recommended)

The easiest method for multiple files:

```bash
# 1. Place your XLS files in the 'to_convert/' folder
# 2. Run the batch conversion
./convert_all.sh
```

**Features:**
- Automatically converts all XLS files in `to_convert/`
- Saves results to `converted/`
- Log system prevents duplicate conversions
- Automatically detects modified files (via SHA256 hash)

The script creates a `.conversion_log.json` file that tracks which files have already been converted. When you run the script again, only new or modified files will be converted.

### Method 2: Single File Conversion

For individual files you can use the single conversion script:

```bash
./convert.sh <xls_file> [output_csv]
```

**Examples:**

```bash
# Conversion with automatic output name
./convert.sh statement.xls
# Creates: statement_zoho.csv

# Conversion with custom output name
./convert.sh statement.xls my_output.csv
```

### Method 3: Direct Python Invocation

```bash
./venv/bin/python3 crdb_to_zoho.py <xls_file> [output_csv]
```

## CSV Format

The generated CSV file has the following columns:

- **Date**: Date in YYYY-MM-DD format
- **Withdrawals**: Debit amounts
- **Deposits**: Credit amounts
- **Payee**: Recipient (empty)
- **Description**: Description (always "Transfer")
- **Reference Number**: Transaction details (max. 99 characters)

The CSV file uses semicolon (;) as delimiter.

## Input File Format

The script expects CRDB Bank account statement files in legacy Excel 97-2003 (`.xls`, CDFV2) format. These can be downloaded from the CRDB Bank online banking portal.

**File layout:**

| Rows   | Content                  |
|--------|--------------------------|
| 0-13   | Header and bank metadata |
| 14     | Column headers           |
| 15+    | Transaction data         |

**Transaction columns (row 15+):**

| Column | Content        | Example                          |
|--------|----------------|----------------------------------|
| 0      | Posting Date   | ` 15.01.2026 12:30:00`           |
| 1      | Details        | `POS Purchase at SHOP XYZ`       |
| 2      | Value Date     | ` 15.01.2026 12:30:00`           |
| 3      | Debit          | ` 977,000.00`                    |
| 4      | Credit         | ` 0.00`                          |
| 5      | Book Balance   | ` 1,234,567.89`                  |

**Format details:**
- Dates use the format `DD.MM.YYYY HH:MM:SS` (with leading space)
- Amounts use comma as thousands separator and dot as decimal separator (e.g. `977,000.00`)
- Only the first sheet of the workbook is processed

## Directory Structure

```
crdb_csv_conv/
├── to_convert/          # Place your XLS files here
├── converted/           # Converted CSV files are saved here
├── example/             # Example files
├── venv/                # Python virtual environment
├── .conversion_log.json # Log of converted files (created automatically)
├── crdb_to_zoho.py      # Main conversion script
├── batch_convert.py     # Batch conversion script
├── convert.sh           # Wrapper for single file conversion
└── convert_all.sh       # Wrapper for batch conversion
```

## Log System

The batch conversion system uses a `.conversion_log.json` file to track:
- Which files have already been converted
- When the conversion took place
- Hash of the original file (to detect changes)

If a file in the `to_convert/` folder is modified, the system automatically detects this (via SHA256 hash) and re-converts the file.

### Reset Log

If you want to re-convert all files:

```bash
rm .conversion_log.json
./convert_all.sh
```

## Examples

### Batch Conversion

```bash
# Copy XLS files to the folder
cp /path/to/statements/*.xls to_convert/

# Run batch conversion
./convert_all.sh

# View results
ls -l converted/
```

### Single File Conversion

```bash
./convert.sh example/202601_Statement_TZS.xls example/output.csv
```

This converts the example XLS file into a ZOHO Books compatible CSV file.
