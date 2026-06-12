# Menu, License, Icon & English UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an in-app menu (File/Help) with Help/About/License/update-check dialogs, an MIT license visible in app and installers, a real app icon, keyboard shortcuts — and switch the entire UI to English.

**Architecture:** HTML menu bar + modal dialogs inside the existing static frontend (one code path, no native menu). Two new thin Tauri commands (`get_app_info` with compile-time-embedded license, `open_url` with allowlist). Icon as hand-authored SVG → `resvg` → `cargo tauri icon`.

**Tech Stack:** Existing stack (Tauri 2, vanilla JS, no Node). New tooling: `resvg` CLI (one-time `cargo install`).

**Spec:** `docs/superpowers/specs/2026-06-13-menu-license-icon-design.md`

**Baseline:** all 32 existing tests green, incl. the byte-exact reference test — that test must stay green through every task.

---

### Task 1: MIT license file + packaging metadata

**Files:**
- Create: `LICENSE` (repo root)
- Modify: `gui/src-tauri/Cargo.toml`
- Modify: `gui/src-tauri/tauri.conf.json`

- [ ] **Step 1: Create `LICENSE` at the repo root**

```text
MIT License

Copyright (c) 2026 Leon Kasdorf

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

- [ ] **Step 2: Add license metadata to `gui/src-tauri/Cargo.toml`**

In `[package]`, after the `edition` line, add:

```toml
license = "MIT"
```

- [ ] **Step 3: Add `licenseFile` to `gui/src-tauri/tauri.conf.json`**

In the `"bundle"` object, after `"targets": "all",` add:

```json
    "licenseFile": "../../LICENSE",
```

(Path is relative to `tauri.conf.json`; MSI and NSIS installers will show the license during setup.)

- [ ] **Step 4: Verify**

From `gui/src-tauri/`: `cargo check`
Expected: clean. (`tauri.conf.json` is validated at compile time by `tauri_build`.)

- [ ] **Step 5: Commit**

```bash
git add LICENSE gui/src-tauri/Cargo.toml gui/src-tauri/tauri.conf.json
git commit -m "feat: add MIT license, shown by MSI/NSIS installers"
```

---

### Task 2: `get_app_info` command (TDD)

**Files:**
- Modify: `gui/src-tauri/src/commands.rs`
- Modify: `gui/src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing test**

`commands.rs` has no tests module yet. Add at the END of the file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_info_exposes_version_and_license() {
        let info = app_info();
        assert_eq!(info.name, "CRDB CSV Converter");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert!(info.license_text.contains("MIT License"));
        assert!(info.license_text.contains("Leon Kasdorf"));
    }
}
```

Add the stub above the tests module (after `open_folder`):

```rust
#[derive(serde::Serialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub license_text: String,
}

fn app_info() -> AppInfo {
    unimplemented!()
}
```

- [ ] **Step 2: Run test to verify it fails**

From `gui/src-tauri/`: `cargo test app_info`
Expected: panics with `not implemented`.

- [ ] **Step 3: Implement**

```rust
const APP_DISPLAY_NAME: &str = "CRDB CSV Converter";
// Embedded at compile time — works in the portable exe, no runtime file lookup.
const LICENSE_TEXT: &str = include_str!("../../../LICENSE");

fn app_info() -> AppInfo {
    AppInfo {
        name: APP_DISPLAY_NAME.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        license_text: LICENSE_TEXT.to_string(),
    }
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    app_info()
}
```

- [ ] **Step 4: Register the command**

In `gui/src-tauri/src/lib.rs`, extend `generate_handler!`:

```rust
        .invoke_handler(tauri::generate_handler![
            commands::load_config,
            commands::save_config,
            commands::scan_files,
            commands::convert_files,
            commands::open_folder,
            commands::get_app_info
        ])
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test`
Expected: 33 unit + 1 integration green.

- [ ] **Step 6: Commit**

```bash
git add gui/src-tauri/src/commands.rs gui/src-tauri/src/lib.rs
git commit -m "feat(gui): get_app_info command with embedded MIT license text"
```

---

### Task 3: `open_url` command with allowlist (TDD)

**Files:**
- Modify: `gui/src-tauri/src/commands.rs`
- Modify: `gui/src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

Add to the tests module in `commands.rs`:

```rust
    #[test]
    fn allowlist_accepts_repo_urls() {
        assert!(is_allowed_url("https://github.com/lkasdorf/crdb_csv_conv_2026"));
        assert!(is_allowed_url("https://github.com/lkasdorf/crdb_csv_conv_2026/issues"));
        assert!(is_allowed_url(
            "https://github.com/lkasdorf/crdb_csv_conv_2026/releases/tag/v0.1.0-dev"
        ));
    }

    #[test]
    fn allowlist_rejects_foreign_urls() {
        assert!(!is_allowed_url("https://evil.example.com/"));
        assert!(!is_allowed_url("http://github.com/lkasdorf/crdb_csv_conv_2026"));
        assert!(!is_allowed_url("https://github.com/lkasdorf/crdb_csv_conv_2026evil"));
        assert!(!is_allowed_url("file:///C:/Windows"));
    }
```

Add the stub:

```rust
fn is_allowed_url(_url: &str) -> bool {
    unimplemented!()
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test allowlist`
Expected: panics with `not implemented`.

- [ ] **Step 3: Implement**

```rust
const ALLOWED_URL_PREFIX: &str = "https://github.com/lkasdorf/crdb_csv_conv_2026";

fn is_allowed_url(url: &str) -> bool {
    url == ALLOWED_URL_PREFIX
        || url
            .strip_prefix(ALLOWED_URL_PREFIX)
            .is_some_and(|rest| rest.starts_with('/') || rest.starts_with('?') || rest.starts_with('#'))
}

#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    if !is_allowed_url(&url) {
        return Err(format!("URL not allowed: {url}"));
    }
    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("explorer").arg(&url).spawn();
    #[cfg(not(target_os = "windows"))]
    let result = std::process::Command::new("xdg-open").arg(&url).spawn();
    result.map(|_| ()).map_err(|e| e.to_string())
}
```

- [ ] **Step 4: Register the command**

Add `commands::open_url` to `generate_handler!` in `lib.rs` (after `commands::get_app_info`).

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test`
Expected: 35 unit + 1 integration green.

- [ ] **Step 6: Commit**

```bash
git add gui/src-tauri/src/commands.rs gui/src-tauri/src/lib.rs
git commit -m "feat(gui): open_url command restricted to the project repo"
```

---

### Task 4: Backend strings → English

**Files:**
- Modify: `gui/src-tauri/src/batch.rs`
- Modify: `gui/src-tauri/src/commands.rs`
- Modify: `gui/src-tauri/tauri.conf.json`

- [ ] **Step 1: Confirm no test asserts a German string**

Run from `gui/src-tauri/`: `grep -n "bereits\|Zeilen\|unlesbar\|Ordner" src/*.rs`
Expected: hits only in non-test code (batch.rs strings, commands.rs `open_folder`). If a test asserts one of these strings, update that assertion in the same commit.

- [ ] **Step 2: Translate the four user-facing strings**

In `gui/src-tauri/src/batch.rs`:
- `"Log unlesbar, beginne neu: {e}"` → `"Log unreadable, starting fresh: {e}"` (BOTH occurrences in `load_log`)
- `"bereits konvertiert"` → `"already converted"` (in `process_one`)
- `format!("{} Zeilen", conversion.rows)` → `format!("{} rows", conversion.rows)`

In `gui/src-tauri/src/commands.rs`:
- `"Ordner nicht gefunden: {path}"` → `"Folder not found: {path}"`

- [ ] **Step 3: English window title**

In `gui/src-tauri/tauri.conf.json`: `"title": "CRDB → ZOHO CSV Konverter"` → `"title": "CRDB → ZOHO CSV Converter"`.

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: all green (the byte-exact reference test proves the CSV path is untouched).

- [ ] **Step 5: Commit**

```bash
git add gui/src-tauri/src/batch.rs gui/src-tauri/src/commands.rs gui/src-tauri/tauri.conf.json
git commit -m "feat(gui): English backend messages and window title"
```

---

### Task 5: Frontend translation to English (existing UI)

**Files:**
- Modify: `gui/src/index.html`
- Modify: `gui/src/main.js`

- [ ] **Step 1: Translate `index.html`**

- `<html lang="de">` → `<html lang="en">`
- `<title>` → `CRDB → ZOHO CSV Converter`
- `Eingabe:` → `Input:` · `Ausgabe:` → `Output:` · both `– nicht gewählt –` → `– not selected –` · both `Ändern…` → `Change…`
- Table headers `Datei`/`Größe`/`Status` → `File`/`Size`/`Status`
- Empty hint → `No XLS files — choose an input folder or drag files here.`
- `Neu konvertieren erzwingen` → `Force re-conversion`
- `Ausgabeordner öffnen` → `Open output folder` · `Konvertieren` → `Convert`

- [ ] **Step 2: Translate `main.js`**

- `STATUS_LABELS`: `Neu`→`New`, `Bereits konvertiert`→`Already converted`, `Geändert`→`Changed`, `Hinzugefügt`→`Added`
- `RESULT_LABELS`: `✓ konvertiert`→`✓ converted`, `⏭ übersprungen`→`⏭ skipped`, `✗ Fehler`→`✗ error`
- Warning detail: `` `${result.warnings.length} Warnung(en): …` `` → `` `${result.warnings.length} warning(s): …` ``
- `render()` fallback texts `– nicht gewählt –` → `– not selected –` (both)
- Alerts/messages: `Eingabeordner kann nicht gelesen werden:\n` → `Cannot read input folder:\n`; `Konvertierung fehlgeschlagen:\n` → `Conversion failed:\n`; `Initialisierung fehlgeschlagen:\n` → `Initialization failed:\n`; `console.error("Konfiguration konnte nicht gespeichert werden:", e)` → `console.error("Could not save configuration:", e)`
- Dialog titles in `pickFolder`: `` `${which === "input" ? "Eingabe" : "Ausgabe"}ordner wählen` `` → `` `Choose ${which} folder` ``

- [ ] **Step 3: Verify build**

From `gui/src-tauri/`: `cargo build`
Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add gui/src/index.html gui/src/main.js
git commit -m "feat(gui): switch frontend to English"
```

---

### Task 6: Menu bar, dialogs and shortcuts

**Files:**
- Modify: `gui/src/index.html`
- Modify: `gui/src/style.css`
- Modify: `gui/src/main.js`
- Modify: `gui/src-tauri/capabilities/default.json`

- [ ] **Step 1: Add menu + modal markup to `index.html`**

Insert directly after `<body>` (before `<header>`):

```html
  <nav id="menubar">
    <div class="menu">
      <button class="menu-title" data-menu-btn="file">File</button>
      <div class="menu-dropdown" id="menu-file">
        <button class="menu-item" data-action="pick-input">Choose input folder…<span class="shortcut">Ctrl+O</span></button>
        <button class="menu-item" data-action="pick-output">Choose output folder…<span class="shortcut">Ctrl+Shift+O</span></button>
        <button class="menu-item" data-action="rescan">Rescan<span class="shortcut">F5</span></button>
        <button class="menu-item" data-action="convert">Convert<span class="shortcut">Ctrl+Enter</span></button>
        <hr>
        <button class="menu-item" data-action="show-log">Show conversion log</button>
        <hr>
        <button class="menu-item" data-action="exit">Exit</button>
      </div>
    </div>
    <div class="menu">
      <button class="menu-title" data-menu-btn="help">Help</button>
      <div class="menu-dropdown" id="menu-help">
        <button class="menu-item" data-action="help">Help<span class="shortcut">F1</span></button>
        <hr>
        <button class="menu-item" data-action="check-updates">Check for updates…</button>
        <button class="menu-item" data-action="report-issue">Report an issue…</button>
        <hr>
        <button class="menu-item" data-action="license">License</button>
        <button class="menu-item" data-action="about">About CRDB CSV Converter</button>
      </div>
    </div>
  </nav>
```

Insert directly before `<script src="main.js" defer></script>`:

```html
  <div id="modal-overlay" style="display:none">
    <div id="modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
      <div id="modal-header">
        <h2 id="modal-title"></h2>
        <button id="modal-close" aria-label="Close">×</button>
      </div>
      <div id="modal-body"></div>
    </div>
  </div>
```

- [ ] **Step 2: Add menu + modal styles to `style.css`**

Append:

```css
/* --- menu bar --- */
#menubar {
  display: flex;
  background: #fff;
  border-bottom: 1px solid #dde1e7;
  padding: 0 8px;
  user-select: none;
}
.menu { position: relative; }
.menu-title {
  border: none;
  background: transparent;
  border-radius: 0;
  padding: 6px 12px;
  font-size: 13px;
}
.menu-title:hover { background: #eef0f4; }
.menu-dropdown {
  display: none;
  position: absolute;
  top: 100%;
  left: 0;
  z-index: 20;
  min-width: 280px;
  background: #fff;
  border: 1px solid #c5ccd6;
  border-radius: 0 0 6px 6px;
  box-shadow: 0 6px 16px rgba(29, 36, 48, 0.15);
  padding: 4px 0;
}
.menu-dropdown.open { display: block; }
.menu-dropdown hr { border: none; border-top: 1px solid #e7eaef; margin: 4px 0; }
.menu-item {
  display: flex;
  justify-content: space-between;
  gap: 24px;
  width: 100%;
  text-align: left;
  border: none;
  border-radius: 0;
  background: transparent;
  padding: 7px 16px;
  font-size: 13px;
}
.menu-item:hover:not(:disabled) { background: #eef0f4; }
.menu-item .shortcut { color: #8a93a5; }

/* --- modal --- */
#modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 30;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(29, 36, 48, 0.45);
}
#modal {
  background: #fff;
  border-radius: 8px;
  width: min(560px, calc(100vw - 48px));
  max-height: 75vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 12px 32px rgba(29, 36, 48, 0.3);
}
#modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid #e7eaef;
}
#modal-header h2 { font-size: 16px; }
#modal-close { border: none; background: transparent; font-size: 18px; line-height: 1; padding: 4px 8px; }
#modal-body { padding: 16px; overflow-y: auto; }
#modal-body h3 { margin: 12px 0 6px; font-size: 14px; }
#modal-body p, #modal-body li { margin: 6px 0; }
#modal-body ol, #modal-body ul { padding-left: 20px; }
#modal-body a { color: #1a6fb0; }
.license-text {
  font-family: Consolas, monospace;
  font-size: 12px;
  white-space: pre-wrap;
  margin: 0;
}
.shortcut-table td { padding: 3px 12px 3px 0; }
.shortcut-table td:first-child { font-family: Consolas, monospace; color: #44506a; }
.about { text-align: center; }
.about img { margin-bottom: 8px; }
```

- [ ] **Step 3: Add menu/dialog/shortcut logic to `main.js`**

At the top, extend the Tauri imports (after the existing three lines):

```js
const { getCurrentWindow } = window.__TAURI__.window;

const REPO_URL = "https://github.com/lkasdorf/crdb_csv_conv_2026";
```

Add these functions before `init()`:

```js
// --- menu bar ---
function closeMenus() {
  document.querySelectorAll(".menu-dropdown").forEach((d) => d.classList.remove("open"));
}

function toggleMenu(name) {
  const dropdown = el(`menu-${name}`);
  const wasOpen = dropdown.classList.contains("open");
  closeMenus();
  if (!wasOpen) dropdown.classList.add("open");
}

function showLog() {
  if (!state.inputDir) return;
  // mirror the backend log_path rule: the log lives in the parent of the
  // input dir; with no usable parent it falls back into the input dir
  const parent = state.inputDir.replace(/[\\/][^\\/]+[\\/]?$/, "");
  invoke("open_folder", { path: parent || state.inputDir }).catch((e) => alert(e));
}

const MENU_ACTIONS = {
  "pick-input": () => pickFolder("input"),
  "pick-output": () => pickFolder("output"),
  rescan: () => rescan(),
  convert: () => convert(),
  "show-log": () => showLog(),
  exit: () => getCurrentWindow().close(),
  help: () => showHelp(),
  "check-updates": () => showUpdateCheck(),
  "report-issue": () => invoke("open_url", { url: `${REPO_URL}/issues` }).catch((e) => alert(e)),
  license: () => showLicense(),
  about: () => showAbout(),
};

// --- modal ---
function openModal(title, bodyNode) {
  el("modal-title").textContent = title;
  const body = el("modal-body");
  body.innerHTML = "";
  body.appendChild(bodyNode);
  el("modal-overlay").style.display = "flex";
}

function closeModal() {
  el("modal-overlay").style.display = "none";
}

// --- dialogs (innerHTML below contains only static, trusted markup) ---
function showHelp() {
  const div = document.createElement("div");
  div.innerHTML = `
    <p>Convert CRDB Bank XLS statements to ZOHO Books CSV:</p>
    <ol>
      <li>Choose an input folder (e.g. <code>to_convert</code>) and an output folder.</li>
      <li>Optionally drag additional .xls files onto the window.</li>
      <li>Click <strong>Convert</strong>. Files already converted are skipped automatically (SHA256 dedup).</li>
    </ol>
    <h3>Status legend</h3>
    <ul>
      <li><strong>New</strong> — not converted yet</li>
      <li><strong>Already converted</strong> — unchanged since the last conversion, will be skipped</li>
      <li><strong>Changed</strong> — file content changed, will be re-converted</li>
      <li><strong>Added</strong> — added via drag &amp; drop</li>
    </ul>
    <h3>Keyboard shortcuts</h3>
    <table class="shortcut-table">
      <tr><td>Ctrl+O</td><td>Choose input folder</td></tr>
      <tr><td>Ctrl+Shift+O</td><td>Choose output folder</td></tr>
      <tr><td>F5</td><td>Rescan</td></tr>
      <tr><td>Ctrl+Enter</td><td>Convert</td></tr>
      <tr><td>F1</td><td>Help</td></tr>
      <tr><td>Esc</td><td>Close menu or dialog</td></tr>
    </table>`;
  openModal("Help", div);
}

async function showAbout() {
  const info = await invoke("get_app_info");
  const div = document.createElement("div");
  div.className = "about";
  div.innerHTML = `
    <img src="icon.png" alt="" width="64" height="64">
    <h3></h3>
    <p class="version"></p>
    <p>Converts CRDB Bank (Tanzania) XLS statements into ZOHO Books CSV files.</p>
    <p><a href="#" id="about-repo">GitHub repository</a></p>
    <p>MIT licensed — <a href="#" id="about-license">view license</a></p>`;
  div.querySelector("h3").textContent = info.name;
  div.querySelector(".version").textContent = `Version ${info.version}`;
  div.querySelector("#about-repo").addEventListener("click", (e) => {
    e.preventDefault();
    invoke("open_url", { url: REPO_URL }).catch((err) => alert(err));
  });
  div.querySelector("#about-license").addEventListener("click", (e) => {
    e.preventDefault();
    showLicense();
  });
  openModal("About", div);
}

async function showLicense() {
  const info = await invoke("get_app_info");
  const pre = document.createElement("pre");
  pre.className = "license-text";
  pre.textContent = info.license_text;
  openModal("License", pre);
}

async function showUpdateCheck() {
  const div = document.createElement("div");
  div.textContent = "Checking GitHub for releases…";
  openModal("Check for updates", div);
  try {
    const info = await invoke("get_app_info");
    // the releases LIST includes prereleases; /releases/latest would skip them
    const resp = await fetch("https://api.github.com/repos/lkasdorf/crdb_csv_conv_2026/releases");
    if (!resp.ok) throw new Error(`GitHub API: HTTP ${resp.status}`);
    const releases = await resp.json();
    const latest = releases[0];
    div.textContent = "";
    const installed = document.createElement("p");
    installed.textContent = `Installed: ${info.version}`;
    const remote = document.createElement("p");
    remote.textContent = latest
      ? `Latest on GitHub: ${latest.tag_name}`
      : "No releases found on GitHub.";
    div.append(installed, remote);
    if (latest) {
      const btn = document.createElement("button");
      btn.textContent = "Open release page";
      btn.addEventListener("click", () =>
        invoke("open_url", { url: latest.html_url }).catch((e) => alert(e))
      );
      div.appendChild(btn);
    }
  } catch (e) {
    div.textContent = `Could not check for updates: ${e}`;
  }
}
```

In `render()`, after the existing `el("force").disabled = state.converting;` block, add the menu-item guards:

```js
  const setMenuItem = (action, disabled) => {
    const item = document.querySelector(`.menu-item[data-action="${action}"]`);
    if (item) item.disabled = disabled;
  };
  setMenuItem("convert", el("convert").disabled);
  setMenuItem("pick-input", state.converting);
  setMenuItem("pick-output", state.converting);
  setMenuItem("rescan", state.converting || !state.inputDir);
  setMenuItem("show-log", !state.inputDir);
```

In `init()`, inside the `try` block (after the existing button listeners), add:

```js
    // menu bar
    document.querySelectorAll("[data-menu-btn]").forEach((btn) =>
      btn.addEventListener("click", (e) => {
        e.stopPropagation();
        toggleMenu(btn.dataset.menuBtn);
      })
    );
    document.querySelectorAll(".menu-item").forEach((item) =>
      item.addEventListener("click", () => {
        closeMenus();
        MENU_ACTIONS[item.dataset.action]?.();
      })
    );
    document.addEventListener("click", () => closeMenus());

    // modal
    el("modal-close").addEventListener("click", closeModal);
    el("modal-overlay").addEventListener("click", (e) => {
      if (e.target === el("modal-overlay")) closeModal();
    });

    // keyboard shortcuts
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape") { closeModal(); closeMenus(); return; }
      if (e.key === "F1") { e.preventDefault(); showHelp(); return; }
      if (e.key === "F5") { e.preventDefault(); if (!state.converting && state.inputDir) rescan(); return; }
      if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "o") { e.preventDefault(); if (!state.converting) pickFolder("output"); return; }
      if (e.ctrlKey && !e.shiftKey && e.key.toLowerCase() === "o") { e.preventDefault(); if (!state.converting) pickFolder("input"); return; }
      if (e.ctrlKey && e.key === "Enter") { e.preventDefault(); if (!el("convert").disabled) convert(); }
    });
```

- [ ] **Step 4: Allow window close from JS**

In `gui/src-tauri/capabilities/default.json`, extend permissions:

```json
  "permissions": [
    "core:default",
    "core:window:allow-close",
    "dialog:default"
  ]
```

(`core:window:allow-close` may be included in `core:default`; declaring it explicitly is harmless and makes the Exit menu item's requirement visible.)

- [ ] **Step 5: Verify build + tests**

From `gui/src-tauri/`: `cargo build` then `cargo test`
Expected: clean build, all tests green. (The About dialog references `icon.png`, which lands in Task 7 — a 404 image in dev until then is acceptable; the `alt=""` keeps it invisible.)

- [ ] **Step 6: Commit**

```bash
git add gui/src/index.html gui/src/style.css gui/src/main.js gui/src-tauri/capabilities/default.json
git commit -m "feat(gui): menu bar with help/about/license/update dialogs and shortcuts"
```

---

### Task 7: Application icon

**Files:**
- Create: `gui/src-tauri/app-icon.svg`
- Replace: `gui/src-tauri/app-icon.png` (rendered from the SVG)
- Replace: `gui/src-tauri/icons/*` (regenerated)
- Create: `gui/src/icon.png` (128 px, used by the About dialog)

- [ ] **Step 1: Install resvg (one-time)**

Run: `cargo install resvg --locked`
Expected: `Installed package resvg`. (If a transient schannel TLS error occurs, retry with `CARGO_HTTP_MULTIPLEXING=false`.)

- [ ] **Step 2: Write `gui/src-tauri/app-icon.svg`**

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#2585cf"/>
      <stop offset="1" stop-color="#15598d"/>
    </linearGradient>
  </defs>
  <rect width="1024" height="1024" rx="180" fill="url(#bg)"/>
  <!-- document sheet -->
  <rect x="180" y="170" width="470" height="600" rx="40" fill="#ffffff"/>
  <!-- table lines -->
  <rect x="240" y="260" width="350" height="30" rx="15" fill="#c5d5e8"/>
  <rect x="240" y="340" width="350" height="30" rx="15" fill="#c5d5e8"/>
  <rect x="240" y="420" width="230" height="30" rx="15" fill="#c5d5e8"/>
  <!-- semicolon accent -->
  <text x="300" y="700" font-family="Segoe UI, DejaVu Sans, sans-serif"
        font-size="340" font-weight="700" fill="#1a6fb0">;</text>
  <!-- conversion arrow -->
  <path d="M560 600 H740 V500 L920 670 L740 840 V740 H560 Z" fill="#ffffff"/>
</svg>
```

- [ ] **Step 3: Render PNGs**

From `gui/src-tauri/`:

```bash
resvg --width 1024 --height 1024 app-icon.svg app-icon.png
resvg --width 128 --height 128 app-icon.svg ../src/icon.png
```

Expected: both PNGs written; `app-icon.png` replaces the old solid-blue placeholder.

- [ ] **Step 4: Regenerate the icon set**

From `gui/src-tauri/`: `cargo tauri icon app-icon.png -o icons`
Expected: `icons/` regenerated (icon.ico, 32x32.png, 128x128.png, …).

- [ ] **Step 5: Verify build**

Run: `cargo build`
Expected: clean (the new icon.ico is embedded into the exe resource).

- [ ] **Step 6: Commit**

```bash
git add gui/src-tauri/app-icon.svg gui/src-tauri/app-icon.png gui/src-tauri/icons/ gui/src/icon.png
git commit -m "feat(gui): real app icon (SVG source, rendered via resvg)"
```

Note for the executor: the icon's look is verified by the user during the smoke test — flag it explicitly in your report.

---

### Task 8: Version 0.2.0, changelog and docs

**Files:**
- Modify: `gui/src-tauri/Cargo.toml` (version)
- Modify: `gui/src-tauri/tauri.conf.json` (version)
- Modify: `gui/src-tauri/Cargo.lock` (regenerated)
- Modify: `CHANGELOG.md`
- Modify: `README.md`
- Modify: `CLAUDE.md`

- [ ] **Step 1: Bump versions**

- `gui/src-tauri/Cargo.toml`: `version = "0.1.0"` → `version = "0.2.0"`
- `gui/src-tauri/tauri.conf.json`: `"version": "0.1.0"` → `"version": "0.2.0"`

- [ ] **Step 2: Refresh Cargo.lock and verify all tests**

From `gui/src-tauri/`: `cargo test`
Expected: lock file updates the `crdb-csv-gui` version; all tests green (the `app_info` test reads the version via `env!`, so it follows automatically).

- [ ] **Step 3: CHANGELOG entry**

Add at the TOP of the `### Added` list under `## [Unreleased]` in `CHANGELOG.md`:

```markdown
- GUI: in-app menu (File/Help) with Help, About, scrollable MIT license view,
  GitHub update check and report-issue link; keyboard shortcuts; real app icon
  (SVG source); entire UI switched to English. App version 0.2.0.
- MIT license (`LICENSE`), embedded in the app and shown by the installers.
```

- [ ] **Step 4: README + CLAUDE.md**

`README.md`: append a license section at the END of the file:

```markdown
## License

MIT — see [LICENSE](LICENSE).
```

`CLAUDE.md`: in the `**gui/**` architecture paragraph, append:

```markdown
The GUI is English-only and has an HTML menu bar (File/Help) with About/License/
update-check dialogs; `LICENSE` (MIT) is embedded via `include_str!` in
`commands.rs`. The icon source is `gui/src-tauri/app-icon.svg` (rendered with
`resvg`, then `cargo tauri icon`).
```

- [ ] **Step 5: Commit**

```bash
git add gui/src-tauri/Cargo.toml gui/src-tauri/tauri.conf.json gui/src-tauri/Cargo.lock CHANGELOG.md README.md CLAUDE.md
git commit -m "chore(gui): bump to 0.2.0, changelog and docs for menu/license/icon"
```

---

### Task 9: Manual smoke test

**Files:** none

Run from `gui/src-tauri/`: `cargo tauri dev`, then walk this checklist (user does the clicking):

- [ ] **Step 1: English UI** — window title, header, table, footer all English; window/taskbar icon is the new icon (not the blue square).
- [ ] **Step 2: Menus** — File and Help open/close on click, Esc, and click-outside; disabled items are greyed (Convert/Rescan/Show log without folders).
- [ ] **Step 3: Dialogs** — Help (F1) shows guide + shortcut table; About shows icon, name, "Version 0.2.0", working GitHub link; License shows the scrollable MIT text; both About links work.
- [ ] **Step 4: Update check** — shows "Installed: 0.2.0" and "Latest on GitHub: v0.1.0-dev"; "Open release page" opens the browser. 
- [ ] **Step 5: Report an issue** — opens the GitHub issues page.
- [ ] **Step 6: Shortcuts** — Ctrl+O, Ctrl+Shift+O, F5, Ctrl+Enter, F1, Esc all work; during a conversion the shortcuts and menu items are inert.
- [ ] **Step 7: Show conversion log** — with the repo's `to_convert` selected, opens the repo root in Explorer.
- [ ] **Step 8: Exit** — File → Exit closes the app.
- [ ] **Step 9: Regression** — pick folders, convert (skip), force-convert: behavior unchanged from 0.1.0.

If any step fails: fix, re-run `cargo test`, repeat the step.

---

## Notes for the executor

- All `cargo` commands run from `gui/src-tauri/` unless stated otherwise.
- The byte-exact reference test (`cargo test --test reference`) is the regression tripwire for every task — it must never break.
- `include_str!("../../../LICENSE")` resolves relative to `commands.rs` (src → src-tauri → gui → repo root). Task 1 MUST land before Task 2 compiles.
- Known machine quirk: transient schannel TLS errors on crates.io downloads — retry with `CARGO_HTTP_MULTIPLEXING=false`.
- The frontend has no test runner (by design); Tasks 5–6 are verified by `cargo build` plus the Task 9 smoke test.
