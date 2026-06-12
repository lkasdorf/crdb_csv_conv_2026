# Menu, License, Icon & English UI — Design

**Date:** 2026-06-13
**Status:** Approved (design discussion 2026-06-13)
**Builds on:** `2026-06-12-tauri-gui-design.md` (the GUI this extends)

## Goal

Round out the Tauri GUI for its first proper release: an in-app menu (Help, About/Version, License), an MIT license that is visible inside the app and in the installers, a real application icon, a handful of small conveniences (update check, shortcuts, report-issue link, log access) — and a fully **English** UI throughout.

## Decisions

| Topic | Decision |
|---|---|
| License | MIT, copyright holder "Leon Kasdorf", year 2026 |
| Menu approach | HTML menu bar inside the window (no native/Rust menu) — one code path, identical on Windows + Linux |
| Update check | Read-only: query GitHub releases API from the frontend, show versions, link to the release page. No auto-updater, no semver comparison. |
| Icon | Hand-authored SVG in the repo, rendered via `resvg`, processed by `cargo tauri icon` |
| UI language | English everywhere (was German). Includes existing strings, window title, and all new surfaces. |
| Version | Bump to 0.2.0 as the final step of this feature |

## License

- `LICENSE` file at the repo root: standard MIT text, `Copyright (c) 2026 Leon Kasdorf`.
- `gui/src-tauri/Cargo.toml`: add `license = "MIT"` to `[package]`.
- `gui/src-tauri/tauri.conf.json`: add `bundle.licenseFile: "../../LICENSE"` — the MSI and NSIS installers then show the license during setup.
- **In-app:** the license text is embedded at compile time (`include_str!` of the repo-root LICENSE) and served by the `get_app_info` command — works in the portable exe with no runtime file lookup.

## English UI

The entire app switches to English:

- `index.html`: `lang="en"`, all labels (`Input`/`Output`, `Change…`, `File`/`Size`/`Status` headers, `Force re-conversion`, `Open output folder`, `Convert`, empty-state hint).
- `main.js`: status labels (`New`, `Already converted`, `Changed`, `Added`, `✓ converted`, `⏭ skipped`, `✗ error`), alerts and detail strings (`%d warning(s)`, error prefixes).
- Backend user-facing strings in `batch.rs`/`commands.rs`: `"bereits konvertiert"` → `"already converted"`, `"{} Zeilen"` → `"{} rows"`, `"Log unlesbar, beginne neu"` → `"Log unreadable, starting fresh"`, `"Ordner nicht gefunden"` → `"Folder not found"`.
- Window title in `tauri.conf.json`: `CRDB → ZOHO CSV Converter`.
- The CSV output contains no language-dependent text; the byte-exact reference test is unaffected. No test currently asserts a German string (verify during implementation; adjust any that do).

## Menu bar (HTML)

A slim menu bar above the existing header with two menus. Items respect the same guards as the buttons (disabled while a conversion runs; log access requires a chosen input dir). Esc or clicking elsewhere closes an open menu.

**File**
| Item | Shortcut | Action |
|---|---|---|
| Choose input folder… | Ctrl+O | same as header button |
| Choose output folder… | Ctrl+Shift+O | same as header button |
| Rescan | F5 | re-run `scan_files` |
| Convert | Ctrl+Enter | same as Convert button |
| — | | |
| Show conversion log | | `open_folder` on the log file's directory (parent of input dir) |
| — | | |
| Exit | | close the window |

**Help**
| Item | Shortcut | Action |
|---|---|---|
| Help | F1 | help dialog |
| — | | |
| Check for updates… | | update dialog (see below) |
| Report an issue… | | `open_url` → GitHub issues page |
| — | | |
| License | | license dialog |
| About CRDB CSV Converter | | about dialog |

Keyboard shortcuts are global `keydown` handlers in `main.js`; actions that mutate state share the existing `state.converting` guard. Esc closes an open modal or menu.

## Dialogs

One generic modal component (overlay + card, close button, Esc):

- **Help:** short usage guide (pick folders, drag & drop, force checkbox), status legend, shortcut table.
- **About:** app icon, name, version (from `get_app_info`), one-line description, GitHub repo link (via `open_url`), "MIT licensed" note with a button that opens the License dialog.
- **License:** scrollable monospace MIT text from `get_app_info`.
- **Update check:** shows `Installed: <version>` and `Latest on GitHub: <tag>` (first entry of `https://api.github.com/repos/lkasdorf/crdb_csv_conv_2026/releases` — the releases list, NOT `/latest`, which skips prereleases), plus an "Open release page" button (`open_url`). Deliberately no version comparison — just information and a link. Offline/API failure → friendly error inside the dialog.

## Backend additions (commands.rs)

- `get_app_info() -> AppInfo { name: String, version: String, license_text: String }` — `name` is the const display name `"CRDB CSV Converter"`, `version` comes from `env!("CARGO_PKG_VERSION")` (Cargo.toml is the source of truth; `tauri.conf.json` is kept in sync), license via `include_str!` of the repo-root LICENSE. No `AppHandle` dependency — the data assembly is a pure function with unit tests.
- `open_url(url: String) -> Result<(), String>` — opens the default browser (`explorer` on Windows / `xdg-open` on Linux, same pattern as `open_folder`). **Allowlist:** only URLs starting with `https://github.com/lkasdorf/crdb_csv_conv_2026` are accepted; anything else returns an error. The allowlist check is a pure function with unit tests.
- Both registered in `lib.rs`'s `generate_handler!`.

GitHub API access happens in the frontend (`fetch`); the API sends permissive CORS headers, and the webview has no CSP restriction. No backend involvement in the update check.

## Icon

- New `gui/src-tauri/app-icon.svg` checked into the repo: rounded square in app blue `#1a6fb0`, white document sheet with table lines, a bold conversion arrow, a semicolon accent. Simple enough to read at 16 px.
- Pipeline: `resvg` (install once via `cargo install resvg`) renders the SVG to a 1024-px `app-icon.png`, then `cargo tauri icon app-icon.png -o icons` regenerates the full icon set (window, taskbar, installer icons). The old solid-blue placeholder PNG is replaced; the SVG is the canonical source.

## Error handling

- Update check: network/API errors render an error message inside the update dialog; no alert storms.
- `open_url`: allowlist rejection or spawn failure → alert with the error.
- `get_app_info` cannot fail (all data embedded at compile time).
- Menu actions inherit existing guards; no new failure modes in the conversion path.

## Testing

- Rust unit tests: `open_url` allowlist (accepts the repo URL and subpaths; rejects other hosts/schemes), `get_app_info` content (version equals `CARGO_PKG_VERSION`, license text contains "MIT License" and "Leon Kasdorf").
- Existing suite (32 tests incl. byte-exact reference) must stay green — the English translation must not touch the CSV output path.
- Frontend (menu, dialogs, shortcuts, update dialog): covered by the manual smoke test, including one deliberate offline check for the update dialog.

## Wrap-up

- Version bump to **0.2.0** in `gui/src-tauri/Cargo.toml` and `tauri.conf.json` (shown in the About dialog).
- CHANGELOG entry under `[Unreleased]`.
- Docs: README/CLAUDE.md mention the menu and the MIT license.

## Out of scope

- Auto-updater (tauri-updater + signing infrastructure).
- Code signing.
- macOS support.
- Multi-language UI / i18n framework (English only).
