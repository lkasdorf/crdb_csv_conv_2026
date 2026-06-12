# Tauri GUI for CRDB → ZOHO CSV Converter — Design

**Date:** 2026-06-12
**Status:** Approved (design discussion 2026-06-12)

## Goal

A lean, fast desktop GUI for converting CRDB Bank XLS statements to ZOHO Books CSV, running on Windows and Linux. The GUI mirrors the existing batch workflow (`convert_all.sh`) including SHA256 deduplication, and stays interoperable with the existing Python CLI.

## Decisions

| Topic | Decision |
|---|---|
| GUI framework | Tauri v2 (Rust + system WebView) |
| Conversion logic | Full Rust port using the `calamine` crate (reads legacy Excel 97-2003 `.xls`) |
| Feature scope | Batch workflow: input/output folder, dedup log, per-file status, drag & drop of additional files |
| Python code | Stays in the repo as a working CLI / reference implementation; not modified |
| Frontend | Static HTML/CSS/JS, no framework, no Node toolchain — build requires only Rust/Cargo |

## Architecture

New subdirectory `gui/` in this repo, standard Tauri v2 layout:

```
gui/
├── src/            # static frontend (index.html, style.css, main.js)
└── src-tauri/      # Rust backend (Cargo project)
    └── src/
        ├── converter.rs   # XLS → CSV core (port of convert_xls_to_csv)
        ├── batch.rs       # folder scan, SHA256, dedup log
        └── lib.rs/main.rs # Tauri commands + app setup
```

`tauri.conf.json` points `frontendDist` at the static `gui/src/` folder; there is no `npm`/bundler step.

### `converter` module — port of `crdb_to_zoho.py`

Must reproduce the Python output **byte-exactly**. Port rules:

- **XLS structure:** sheet 0; rows 0–13 metadata, row 14 column headers, data from row 15 (0-indexed). Columns: `[0] Posting Date`, `[1] Details`, `[2] Value Date`, `[3] Debit`, `[4] Credit`, `[5] Book Balance`.
- **Row skip:** skip rows where column 0 is empty after trimming (cells read as strings; non-string cells are treated via their string representation).
- **Date:** trim, take the first whitespace-separated token, split on `.` as `DD.MM.YYYY`, emit `YYYY-MM-DD` (month/day zero-padded to 2).
- **Amounts:** trim, strip `,` thousands separators, parse as f64.
- **Reference:** trim details, collapse all whitespace runs to a single space, truncate to 99 characters.
- **Row errors:** a row that fails to parse is skipped; a warning (row index + message) is collected and surfaced per file (Python prints these to stderr).
- **CSV output:** header `Date;Withdrawals;Deposits;Payee;Description;Reference Number`; delimiter `;`; line terminator `\n` (Unix LF — ZOHO Books requirement); UTF-8. Per row: date, withdrawals, deposits, empty Payee, literal `Transfer`, reference.
- **Float formatting:** must match Python `str(float)` — shortest round-trip representation with at least one decimal digit (`0.0`, `977000.0`, `304.92`). Rust's `{:?}` (Debug) formatting of `f64` produces this; the byte-exact integration test is the arbiter.

### `batch` module — port of `batch_convert.py` semantics

- Scan the input folder for `*.xls` (non-recursive).
- SHA256-hash each file; compare against the dedup log to classify: **New** (not in log), **Already converted** (hash matches), **Changed** (hash differs).
- Convert New/Changed files (and everything when *force* is set) to `<output_dir>/<basename>.csv`; update the log after each successful conversion.
- **Log format — compatible with Python:** JSON object mapping `filename` → `{hash, converted_at, output_file}` with `converted_at` as local-time ISO 8601 and `output_file` as absolute path. Log entries are keyed by basename only (same limitation as Python: two different files with the same basename collide). `converted_at` is informational only (dedup compares only `hash`); the GUI always writes six fractional digits while Python's `isoformat()` omits the fraction in the rare zero-microsecond case.
- **Log location:** `<parent of input dir>/.conversion_log.json`. With the default workflow (input `<repo>/to_convert`) this is exactly where the Python CLI reads/writes it, so GUI and CLI share dedup state. Edge case: if the input dir has no parent (drive root), the log lives inside the input dir itself.
- **Corrupt log:** start over with an empty log and show a warning (same as Python).
- Dropped files (drag & drop) from arbitrary locations join the same list and the same log, keyed by their basename; their CSV also goes to the chosen output dir.

### Tauri commands

| Command | Purpose |
|---|---|
| `scan_files(input_dir)` | List `*.xls` in the folder with name, size, dedup status |
| `convert_files(input_dir, files, output_dir, force)` | Convert the given files (scanned and dropped alike); the dedup log location is derived from `input_dir`; emits per-file progress events; returns per-file results (converted / skipped / error + warnings) |
| `open_folder(path)` | Open a folder in the system file manager (Explorer / xdg-open) |

## UI

Single window, three areas:

1. **Header:** input and output folder (path display + change button). Last-used folders are persisted in a small app-config JSON (Tauri app config dir); on first launch the user picks both.
2. **File list:** per XLS file — name, size, status. Before conversion: New / Already converted / Changed. After: ✓ converted / ⏭ skipped / ✗ error (with message; per-row warnings viewable). Additional `.xls` files can be dragged onto the window from anywhere.
3. **Footer:** "Convert" button, "Force re-conversion" checkbox (ignores the log for this run, still updates it afterward), "Open output folder" button.

### Data flow

Launch → load last-used folders from config → `scan_files` → render list → user clicks Convert → per file: hash check → convert or skip → update log → live status updates in the list (progress events).

## Error handling

- A failing file never aborts the run; it is marked ✗ with its error message and the remaining files continue (matches `batch_convert.py`).
- Unparseable rows inside a file are skipped and collected as per-file warnings shown in the UI.
- Corrupt dedup log → recreate with warning.
- Missing/unwritable output dir → surfaced as a clear error before conversion starts.

## Testing

- **Rust unit tests** for the ports of `parse_date`, `parse_amount`, `truncate_description` (including whitespace collapsing and the 99-char cut).
- **Integration test (hard acceptance criterion):** convert `example/202601_Statement_TZS.xls` and compare the result **byte-exactly** against `example/202601_Statement_TZS_ref99_trim.csv`. The port is correct only when this passes.
- Batch/dedup tests: log round-trip, hash-unchanged → skip, hash-changed → reconvert, force flag.

## Prerequisites

- Dev machine: Rust toolchain (rustup), Tauri v2 CLI.
- Runtime Windows: WebView2 (present on Windows 11).
- Runtime Linux: WebKitGTK (one-time install via package manager).

## Out of scope

- Installers / CI release pipeline (local `cargo tauri build`; the Tauri bundler produces MSI/AppImage as a side effect anyway).
- Transaction preview before conversion.
- macOS support.
- Changes to the Python CLI.
