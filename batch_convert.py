#!/usr/bin/env python3
"""
CRDB Bank Batch Converter

Automatically converts all XLS files in the 'to_convert' directory
to ZOHO Books CSV format and saves them in the 'converted' directory.
Maintains a log to avoid re-converting files.
"""

import os
import json
import hashlib
import glob
from datetime import datetime
from crdb_to_zoho import convert_xls_to_csv


SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
LOG_FILE = os.path.join(SCRIPT_DIR, '.conversion_log.json')
TO_CONVERT_DIR = os.path.join(SCRIPT_DIR, 'to_convert')
CONVERTED_DIR = os.path.join(SCRIPT_DIR, 'converted')


def calculate_file_hash(file_path):
    """Calculate SHA256 hash of a file"""
    sha256_hash = hashlib.sha256()
    with open(file_path, "rb") as f:
        # Read file in chunks to handle large files
        for byte_block in iter(lambda: f.read(4096), b""):
            sha256_hash.update(byte_block)
    return sha256_hash.hexdigest()


def load_conversion_log():
    """Load conversion log from JSON file"""
    if os.path.exists(LOG_FILE):
        try:
            with open(LOG_FILE, 'r') as f:
                return json.load(f)
        except json.JSONDecodeError:
            print(f"Warning: Could not read log file {LOG_FILE}, creating new log")
            return {}
    return {}


def save_conversion_log(log_data):
    """Save conversion log to JSON file"""
    with open(LOG_FILE, 'w') as f:
        json.dump(log_data, indent=2, fp=f)


def get_xls_files():
    """Get all XLS files in the to_convert directory"""
    pattern = os.path.join(TO_CONVERT_DIR, '*.xls')
    return glob.glob(pattern)


def process_file(xls_file, log_data):
    """
    Process a single XLS file if it hasn't been converted yet
    Returns True if conversion was performed, False if skipped
    """
    # Get file info
    file_name = os.path.basename(xls_file)
    file_hash = calculate_file_hash(xls_file)

    # Check if file was already converted
    if file_name in log_data:
        if log_data[file_name]['hash'] == file_hash:
            print(f"⏭️  Skipped: {file_name} (already converted)")
            return False
        else:
            print(f"🔄 File changed: {file_name} (re-converting)")

    # Generate output filename
    base_name = os.path.splitext(file_name)[0]
    csv_file = os.path.join(CONVERTED_DIR, f"{base_name}.csv")

    # Convert file
    try:
        print(f"🔄 Converting: {file_name}")
        convert_xls_to_csv(xls_file, csv_file)

        # Update log
        log_data[file_name] = {
            'hash': file_hash,
            'converted_at': datetime.now().isoformat(),
            'output_file': csv_file
        }

        print(f"✅ Completed: {file_name} → {csv_file}")
        return True

    except Exception as e:
        print(f"❌ Error converting {file_name}: {e}")
        return False


def main():
    """Main batch conversion function"""
    print("=" * 60)
    print("CRDB Bank Batch Converter")
    print("=" * 60)

    # Ensure directories exist
    os.makedirs(TO_CONVERT_DIR, exist_ok=True)
    os.makedirs(CONVERTED_DIR, exist_ok=True)

    # Load conversion log
    log_data = load_conversion_log()

    # Get all XLS files
    xls_files = get_xls_files()

    if not xls_files:
        print(f"\n⚠️  No XLS files found in '{TO_CONVERT_DIR}/' directory")
        print(f"   Please place your CRDB Bank XLS files there and run again.")
        return

    print(f"\nFound {len(xls_files)} XLS file(s) in '{TO_CONVERT_DIR}/'")
    print()

    # Process each file
    converted_count = 0
    skipped_count = 0
    error_count = 0

    for xls_file in xls_files:
        try:
            if process_file(xls_file, log_data):
                converted_count += 1
            else:
                skipped_count += 1
        except Exception as e:
            print(f"❌ Fatal error processing {xls_file}: {e}")
            error_count += 1

    # Save updated log
    save_conversion_log(log_data)

    # Print summary
    print()
    print("=" * 60)
    print("Conversion Summary")
    print("=" * 60)
    print(f"✅ Converted: {converted_count}")
    print(f"⏭️  Skipped:   {skipped_count} (already converted)")
    if error_count > 0:
        print(f"❌ Errors:    {error_count}")
    print(f"\nOutput directory: {CONVERTED_DIR}/")
    print(f"Log file: {LOG_FILE}")
    print()


if __name__ == '__main__':
    main()
