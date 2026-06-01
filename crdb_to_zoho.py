#!/usr/bin/env python3
"""
CRDB Bank XLS to ZOHO Books CSV Converter

Converts CRDB Bank statement XLS files to CSV format compatible with ZOHO Books.
"""

import xlrd
import csv
import sys
import os
import re


def parse_date(date_str):
    """
    Parse date string from XLS (format: ' DD.MM.YYYY HH:MM:SS')
    Returns date in YYYY-MM-DD format
    """
    date_str = date_str.strip()
    # Extract date part (before space)
    date_part = date_str.split()[0]
    # Parse DD.MM.YYYY
    day, month, year = date_part.split('.')
    return f"{year}-{month.zfill(2)}-{day.zfill(2)}"


def parse_amount(amount_str):
    """
    Parse amount string (format: ' 977,000.00' or ' 0.00')
    Returns float value
    """
    amount_str = amount_str.strip()
    # Remove thousands separator (comma)
    amount_str = amount_str.replace(',', '')
    return float(amount_str)


def truncate_description(text, max_length=99):
    """
    Truncate text to maximum length and clean up whitespace
    """
    text = text.strip()
    # Replace multiple spaces with single space
    text = re.sub(r'\s+', ' ', text)
    if len(text) > max_length:
        return text[:max_length]
    return text


def convert_xls_to_csv(xls_file, csv_file):
    """
    Convert CRDB Bank XLS statement to ZOHO Books CSV format
    """
    # Open XLS file
    workbook = xlrd.open_workbook(xls_file)
    sheet = workbook.sheet_by_index(0)

    # Open CSV file for writing with Unix line endings
    with open(csv_file, 'w', newline='\n', encoding='utf-8') as csvfile:
        writer = csv.writer(csvfile, delimiter=';', lineterminator='\n')

        # Write header
        writer.writerow(['Date', 'Withdrawals', 'Deposits', 'Payee', 'Description', 'Reference Number'])

        # Process data rows (starting from row 15)
        # Row 14 contains column headers, data starts at row 15
        for row_idx in range(15, sheet.nrows):
            row = sheet.row_values(row_idx)

            # Skip empty rows
            if not row or not row[0].strip():
                continue

            # Extract data
            posting_date = row[0]  # Posting Date
            details = row[1]       # Details
            debit = row[3]         # Debit (Withdrawals)
            credit = row[4]        # Credit (Deposits)

            # Convert data
            try:
                date = parse_date(posting_date)
                withdrawals = parse_amount(debit)
                deposits = parse_amount(credit)
                reference = truncate_description(details, 99)

                # Write row
                writer.writerow([
                    date,
                    withdrawals,
                    deposits,
                    '',          # Payee (empty)
                    'Transfer',  # Description
                    reference    # Reference Number
                ])
            except Exception as e:
                print(f"Warning: Error processing row {row_idx}: {e}", file=sys.stderr)
                continue

    print(f"Conversion complete: {csv_file}")


def main():
    if len(sys.argv) < 2:
        print("Usage: python3 crdb_to_zoho.py <xls_file> [output_csv]")
        print("\nExample:")
        print("  python3 crdb_to_zoho.py statement.xls")
        print("  python3 crdb_to_zoho.py statement.xls output.csv")
        sys.exit(1)

    xls_file = sys.argv[1]

    # Check if file exists
    if not os.path.exists(xls_file):
        print(f"Error: File not found: {xls_file}", file=sys.stderr)
        sys.exit(1)

    # Generate output filename if not provided
    if len(sys.argv) >= 3:
        csv_file = sys.argv[2]
    else:
        # Replace .xls with .csv
        base_name = os.path.splitext(xls_file)[0]
        csv_file = f"{base_name}_zoho.csv"

    # Convert
    try:
        convert_xls_to_csv(xls_file, csv_file)
        print(f"\nInput:  {xls_file}")
        print(f"Output: {csv_file}")
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
