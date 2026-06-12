const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;
const { listen } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;

const REPO_URL = "https://github.com/lkasdorf/crdb_csv_conv_2026";

const state = {
  inputDir: null,
  outputDir: null,
  scanned: [],   // FileEntry from scan_files
  dropped: [],   // { name, path } added via drag & drop
  results: {},   // name -> ConvertOutcome
  converting: false,
};

const el = (id) => document.getElementById(id);

const STATUS_LABELS = {
  new: ["New", "status-new"],
  converted: ["Already converted", "status-converted"],
  changed: ["Changed", "status-changed"],
  dropped: ["Added", "status-dropped"],
};

const RESULT_LABELS = {
  converted: ["✓ Converted", "status-ok"],
  skipped: ["⏭ Skipped", "status-skipped"],
  error: ["✗ Error", "status-error"],
};

function fmtSize(bytes) {
  if (bytes == null) return "";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function allFiles() {
  const scannedPaths = new Set(state.scanned.map((f) => f.path));
  const extra = state.dropped.filter((f) => !scannedPaths.has(f.path));
  return [...state.scanned, ...extra];
}

function render() {
  const rows = el("file-rows");
  rows.innerHTML = "";
  const files = allFiles();
  el("empty-hint").style.display = files.length ? "none" : "block";

  for (const f of files) {
    const tr = document.createElement("tr");
    // keyed by basename — two files with the same name share one outcome,
    // mirroring the documented log limitation
    const result = state.results[f.name];
    let label, cls, detail = "";
    if (result) {
      [label, cls] = RESULT_LABELS[result.status] ?? [result.status, ""];
      if (result.status === "error") detail = result.message;
      if (result.warnings?.length) {
        detail = `${result.warnings.length} warning(s): ${result.warnings.join(" | ")}`;
      }
    } else {
      [label, cls] = STATUS_LABELS[f.status ?? "dropped"] ?? ["?", ""];
    }

    const tdName = document.createElement("td");
    tdName.textContent = f.name;
    const tdSize = document.createElement("td");
    tdSize.className = "size";
    tdSize.textContent = fmtSize(f.size);
    const tdStatus = document.createElement("td");
    const span = document.createElement("span");
    span.className = cls;
    span.textContent = label;
    tdStatus.appendChild(span);
    if (detail) {
      const div = document.createElement("div");
      div.className = "warnings";
      div.textContent = detail;
      tdStatus.appendChild(div);
    }
    tr.append(tdName, tdSize, tdStatus);
    rows.appendChild(tr);
  }

  el("input-dir").textContent = state.inputDir ?? "– not selected –";
  el("output-dir").textContent = state.outputDir ?? "– not selected –";
  el("convert").disabled =
    state.converting || !(state.inputDir && state.outputDir && files.length);
  el("open-output").disabled = !state.outputDir;
  el("pick-input").disabled = state.converting;
  el("pick-output").disabled = state.converting;
  el("force").disabled = state.converting;

  const setMenuItem = (action, disabled) => {
    const item = document.querySelector(`.menu-item[data-action="${action}"]`);
    if (item) item.disabled = disabled;
  };
  setMenuItem("convert", el("convert").disabled);
  setMenuItem("pick-input", state.converting);
  setMenuItem("pick-output", state.converting);
  setMenuItem("rescan", state.converting || !state.inputDir);
  setMenuItem("show-log", !state.inputDir);
}

async function rescan() {
  if (!state.inputDir) return;
  try {
    const res = await invoke("scan_files", { inputDir: state.inputDir });   // ★ scan_files returns {files, log_warning}
    state.scanned = res.files;                                              // ★
    if (res.log_warning) alert(res.log_warning);                            // ★ surface corrupt-log warning
  } catch (e) {
    alert(`Cannot read input folder:\n${e}`);
    state.scanned = [];
  }
  render();
}

async function pickFolder(which) {
  const dir = await open({ directory: true, title: `Choose ${which} folder` });
  if (!dir) return;
  if (which === "input") {
    state.inputDir = dir;
    state.results = {};
    await rescan();
  } else {
    state.outputDir = dir;
  }
  try {                                                                     // ★ save_config failures must be surfaced, not unhandled
    await invoke("save_config", {
      config: { input_dir: state.inputDir, output_dir: state.outputDir },
    });
  } catch (e) {
    console.error("Could not save configuration:", e);                       // ★
  }
  render();
}

async function convert() {
  if (state.converting) return;
  state.converting = true;
  const files = allFiles().map((f) => f.path);
  state.results = {};
  render();
  try {
    await invoke("convert_files", {
      inputDir: state.inputDir,
      files,
      outputDir: state.outputDir,
      force: el("force").checked,
    });
  } catch (e) {
    alert(`Conversion failed:\n${e}`);
  } finally {
    state.converting = false;
    await rescan(); // refresh pre-conversion statuses from the updated log
  }
}

// --- menu bar ---
function closeMenus() {
  document.querySelectorAll(".menu-dropdown").forEach((d) => d.classList.remove("open"));
  document.querySelectorAll("[data-menu-btn]").forEach((b) => b.setAttribute("aria-expanded", "false"));
}

function toggleMenu(name) {
  const dropdown = el(`menu-${name}`);
  const wasOpen = dropdown.classList.contains("open");
  closeMenus();
  if (!wasOpen) {
    dropdown.classList.add("open");
    document.querySelector(`[data-menu-btn="${name}"]`)?.setAttribute("aria-expanded", "true");
  }
}

function showLog() {
  if (!state.inputDir) return;
  // mirror the backend log_path rule: the log lives in the parent of the
  // input dir; with no usable parent it falls back into the input dir
  let parent = state.inputDir.replace(/[\\/][^\\/]+[\\/]?$/, "");
  if (/^[A-Za-z]:$/.test(parent)) parent += "\\"; // "C:" is drive-relative — anchor it
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
let modalInvoker = null;

function openModal(title, bodyNode) {
  modalInvoker = document.activeElement;
  el("modal-title").textContent = title;
  const body = el("modal-body");
  body.innerHTML = "";
  body.appendChild(bodyNode);
  el("modal-overlay").style.display = "flex";
  el("modal-close").focus();
}

function closeModal() {
  el("modal-overlay").style.display = "none";
  if (modalInvoker?.focus) modalInvoker.focus();
  modalInvoker = null;
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

async function init() {
  try {
    el("pick-input").addEventListener("click", () => pickFolder("input"));
    el("pick-output").addEventListener("click", () => pickFolder("output"));
    el("convert").addEventListener("click", convert);
    el("open-output").addEventListener("click", () => {
      invoke("open_folder", { path: state.outputDir }).catch((e) => alert(e));
    });

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
      if (el("modal-overlay").style.display !== "none") return; // dialog open — only Esc acts
      if (e.key === "F1") { e.preventDefault(); showHelp(); return; }
      if (e.key === "F5") { e.preventDefault(); if (!state.converting && state.inputDir) rescan(); return; }
      if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "o") { e.preventDefault(); if (!state.converting) pickFolder("output"); return; }
      if (e.ctrlKey && !e.shiftKey && e.key.toLowerCase() === "o") { e.preventDefault(); if (!state.converting) pickFolder("input"); return; }
      if (e.ctrlKey && e.key === "Enter") { e.preventDefault(); if (!el("convert").disabled) convert(); }
    });

    // live per-file status during conversion
    await listen("file-status", (event) => {
      state.results[event.payload.name] = event.payload;
      render();
    });

    // drag & drop of additional .xls files from anywhere
    await listen("tauri://drag-drop", (event) => {
      for (const p of event.payload.paths ?? []) {
        if (!p.toLowerCase().endsWith(".xls")) continue;
        if (state.dropped.some((f) => f.path === p)) continue;
        const name = p.split(/[\\/]/).pop();
        state.dropped.push({ name, path: p });
      }
      render();
    });

    const config = await invoke("load_config");
    state.inputDir = config.input_dir ?? null;
    state.outputDir = config.output_dir ?? null;
    await rescan();
    render();
  } catch (e) {
    alert(`Initialization failed:\n${e}`);
  }
}

init();
