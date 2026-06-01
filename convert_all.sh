#!/bin/bash
# CRDB Batch Converter Wrapper Script

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check if virtual environment exists
if [ ! -d "$SCRIPT_DIR/venv" ]; then
    echo "Error: Virtual environment not found!"
    echo "Please run: python3 -m venv venv && ./venv/bin/pip install -r requirements.txt"
    exit 1
fi

# Run the batch converter script with the virtual environment's Python
"$SCRIPT_DIR/venv/bin/python3" "$SCRIPT_DIR/batch_convert.py" "$@"
