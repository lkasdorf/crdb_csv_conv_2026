# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `.gitattributes` enforcing LF line endings repo-wide (`* text=auto eol=lf`), with `*.xls`/`*.xlsx` marked binary. Stops spurious CRLF-only diffs on Windows checkouts while keeping the shell wrappers bash-runnable and the ZOHO Books CSV output as required Unix LF.

### Changed

- `batch_convert.py` now reconfigures `stdout`/`stderr` to UTF-8 at startup, so the emoji progress output runs cleanly on Windows consoles (cp1252) without needing `PYTHONUTF8=1`.
- `.gitignore` now also excludes colon-named Windows `Zone.Identifier` artifacts (`*Zone.Identifier`) and `*.bak` backups.
- `CLAUDE.md` documents the GitHub repository and the `git push` workflow.

### Removed

- Deleted `example/*:Zone.Identifier` files that were committed by mistake in the initial commit.

### Fixed

- `UnicodeEncodeError` crash in the batch converter on non-UTF-8 Windows consoles.
- Restored the executable bit (`755`) on `convert.sh`, `convert_all.sh`, `batch_convert.py`, and `crdb_to_zoho.py` after a Windows checkout dropped it.

### Notes

- May 2026 statements (`202605_Statement_TZS` and `202605_Statement_USD`) were converted to `converted/`. Output files are gitignored and not tracked.
