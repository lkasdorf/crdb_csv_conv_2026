const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;
const { listen } = window.__TAURI__.event;

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
  converted: ["✓ converted", "status-ok"],
  skipped: ["⏭ skipped", "status-skipped"],
  error: ["✗ error", "status-error"],
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

async function init() {
  try {
    el("pick-input").addEventListener("click", () => pickFolder("input"));
    el("pick-output").addEventListener("click", () => pickFolder("output"));
    el("convert").addEventListener("click", convert);
    el("open-output").addEventListener("click", () => {
      invoke("open_folder", { path: state.outputDir }).catch((e) => alert(e));
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
