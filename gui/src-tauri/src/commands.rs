use std::path::{Path, PathBuf};
use tauri::{Emitter, Manager};

use crate::batch;

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct AppConfig {
    pub input_dir: Option<String>,
    pub output_dir: Option<String>,
}

fn config_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("config.json"))
}

#[tauri::command]
pub fn load_config(app: tauri::AppHandle) -> AppConfig {
    config_file(&app)
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

#[tauri::command]
pub fn save_config(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    let path = config_file(&app)?;
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn scan_files(input_dir: String) -> Result<batch::ScanResult, String> {
    batch::scan_input_dir(Path::new(&input_dir))
}

/// Intentionally synchronous: Tauri v2 dispatches sync commands on their own
/// thread, so the batch runs off the event loop and the per-file "file-status"
/// events are delivered live. Do not move this onto the main thread.
#[tauri::command]
pub fn convert_files(
    app: tauri::AppHandle,
    input_dir: String,
    files: Vec<String>,
    output_dir: String,
    force: bool,
) -> Result<Vec<batch::ConvertOutcome>, String> {
    let paths: Vec<PathBuf> = files.into_iter().map(PathBuf::from).collect();
    batch::convert_files(
        Path::new(&input_dir),
        &paths,
        Path::new(&output_dir),
        force,
        |outcome| {
            let _ = app.emit("file-status", outcome);
        },
    )
}

#[tauri::command]
pub fn open_folder(path: String) -> Result<(), String> {
    // explorer/xdg-open silently no-op on bad paths; validate for a truthful result
    if !Path::new(&path).is_dir() {
        return Err(format!("Ordner nicht gefunden: {path}"));
    }
    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("explorer").arg(&path).spawn();
    #[cfg(not(target_os = "windows"))]
    let result = std::process::Command::new("xdg-open").arg(&path).spawn();
    result.map(|_| ()).map_err(|e| e.to_string())
}

const APP_DISPLAY_NAME: &str = "CRDB CSV Converter";
// Embedded at compile time — works in the portable exe, no runtime file lookup.
// Path is relative to this source file: src/ → src-tauri/ → gui/ → repo root.
const LICENSE_TEXT: &str = include_str!("../../../LICENSE");

#[derive(serde::Serialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub license_text: String,
}

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
}
