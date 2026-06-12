//! Batch conversion with SHA256 dedup, log-compatible with batch_convert.py.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogEntry {
    pub hash: String,
    pub converted_at: String,
    pub output_file: String,
}

pub type ConversionLog = BTreeMap<String, LogEntry>;

#[derive(Serialize, PartialEq, Eq, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum FileStatus {
    New,
    Converted,
    Changed,
}

pub fn load_log(path: &Path) -> (ConversionLog, Option<String>) {
    match std::fs::read_to_string(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (ConversionLog::new(), None),
        Err(e) => (
            ConversionLog::new(),
            Some(format!("Log unreadable, starting fresh: {e}")),
        ),
        Ok(text) => match serde_json::from_str(&text) {
            Ok(log) => (log, None),
            Err(e) => (
                ConversionLog::new(),
                Some(format!("Log unreadable, starting fresh: {e}")),
            ),
        },
    }
}

/// Plain (non-atomic) write — acceptable for this single-user desktop tool:
/// the log is tiny, written sequentially, and a torn write is recovered by
/// the corrupt-log path in `load_log`.
pub fn save_log(path: &Path, log: &ConversionLog) -> Result<(), String> {
    let json = serde_json::to_string_pretty(log).map_err(|e| e.to_string())?;
    std::fs::write(path, json)
        .map_err(|e| format!("cannot write {}: {e}", path.display()))
}

pub fn classify(log: &ConversionLog, name: &str, hash: &str) -> FileStatus {
    match log.get(name) {
        None => FileStatus::New,
        Some(e) if e.hash == hash => FileStatus::Converted,
        Some(_) => FileStatus::Changed,
    }
}

pub fn hash_file(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let bytes =
        std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn log_path(input_dir: &Path) -> PathBuf {
    match input_dir.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.join(".conversion_log.json"),
        _ => input_dir.join(".conversion_log.json"),
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct ConvertOutcome {
    pub name: String,
    pub status: String, // "converted" | "skipped" | "error"
    pub message: String,
    pub warnings: Vec<String>,
}

pub fn convert_files(
    input_dir: &Path,
    files: &[PathBuf],
    output_dir: &Path,
    force: bool,
    mut on_progress: impl FnMut(&ConvertOutcome),
) -> Result<Vec<ConvertOutcome>, String> {
    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("cannot create {}: {e}", output_dir.display()))?;

    let log_file = log_path(input_dir);
    let (mut log, log_warning) = load_log(&log_file);
    let mut results = Vec::new();

    for file in files {
        let name = file
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();

        let outcome = match process_one(file, &name, output_dir, force, &mut log, &log_file) {
            Ok(o) => o,
            Err(message) => ConvertOutcome {
                name: name.clone(),
                status: "error".to_string(),
                message,
                warnings: Vec::new(),
            },
        };
        on_progress(&outcome);
        results.push(outcome);
    }

    if let Some(w) = log_warning {
        eprintln!("{w}");
    }
    Ok(results)
}

fn process_one(
    file: &Path,
    name: &str,
    output_dir: &Path,
    force: bool,
    log: &mut ConversionLog,
    log_file: &Path,
) -> Result<ConvertOutcome, String> {
    let hash = hash_file(file)?;

    if !force && classify(log, name, &hash) == FileStatus::Converted {
        return Ok(ConvertOutcome {
            name: name.to_string(),
            status: "skipped".to_string(),
            message: "already converted".to_string(),
            warnings: Vec::new(),
        });
    }

    let stem = Path::new(name)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| name.to_string());
    let csv_path = output_dir.join(format!("{stem}.csv"));

    let conversion = crate::converter::convert_xls_to_csv(file, &csv_path)?;

    let output_abs = std::path::absolute(&csv_path)
        .unwrap_or(csv_path.clone())
        .to_string_lossy()
        .into_owned();
    log.insert(
        name.to_string(),
        LogEntry {
            hash,
            // Format-compatible with Python's datetime.isoformat(); informational
            // only — dedup compares only `hash`. (Python omits the fraction at
            // exactly zero microseconds; we always write six digits.)
            converted_at: chrono::Local::now()
                .format("%Y-%m-%dT%H:%M:%S%.6f")
                .to_string(),
            output_file: output_abs,
        },
    );
    save_log(log_file, log)?;

    Ok(ConvertOutcome {
        name: name.to_string(),
        status: "converted".to_string(),
        message: format!("{} rows", conversion.rows),
        warnings: conversion.warnings,
    })
}

#[derive(Serialize, Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub status: FileStatus,
}

#[derive(Serialize, Clone, Debug)]
pub struct ScanResult {
    pub files: Vec<FileEntry>,
    pub log_warning: Option<String>,
}

pub fn scan_input_dir(input_dir: &Path) -> Result<ScanResult, String> {
    let (log, log_warning) = load_log(&log_path(input_dir));
    let rd = std::fs::read_dir(input_dir)
        .map_err(|e| format!("cannot read {}: {e}", input_dir.display()))?;

    let mut entries = Vec::new();
    for entry in rd.flatten() {
        let path = entry.path();
        let is_xls = path
            .extension()
            .map(|e| e.eq_ignore_ascii_case("xls"))
            .unwrap_or(false);
        if !is_xls || !path.is_file() {
            continue;
        }
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        let hash = hash_file(&path)?;
        entries.push(FileEntry {
            status: classify(&log, &name, &hash),
            name,
            path: path.to_string_lossy().into_owned(),
            size,
        });
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(ScanResult { files: entries, log_warning })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    fn example_xls() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../example/202601_Statement_TZS.xls")
    }

    #[test]
    fn convert_files_converts_then_skips_then_forces() {
        let root = tempfile::tempdir().unwrap();
        let input = root.path().join("to_convert");
        let output = root.path().join("converted");
        std::fs::create_dir(&input).unwrap();
        std::fs::copy(example_xls(), input.join("stmt.xls")).unwrap();
        let files = vec![input.join("stmt.xls")];

        // 1st run: converts
        let r1 = convert_files(&input, &files, &output, false, |_| {}).unwrap();
        assert_eq!(r1[0].status, "converted");
        assert!(output.join("stmt.csv").is_file());
        assert!(log_path(&input).is_file());

        // 2nd run: skips (hash unchanged)
        let r2 = convert_files(&input, &files, &output, false, |_| {}).unwrap();
        assert_eq!(r2[0].status, "skipped");

        // 3rd run with force: converts again
        let r3 = convert_files(&input, &files, &output, true, |_| {}).unwrap();
        assert_eq!(r3[0].status, "converted");

        // 4th run: file content changed -> hash differs -> reconverts without force
        let mut bytes = std::fs::read(input.join("stmt.xls")).unwrap();
        bytes.push(0u8);
        std::fs::write(input.join("stmt.xls"), &bytes).unwrap();
        let r4 = convert_files(&input, &files, &output, false, |_| {}).unwrap();
        // note: the appended byte makes the XLS invalid for calamine on some
        // versions; accept either reconversion or a clean error - the key
        // assertion is that it is NOT skipped
        assert_ne!(r4[0].status, "skipped");
    }

    #[test]
    fn convert_files_error_does_not_abort_run() {
        let root = tempfile::tempdir().unwrap();
        let input = root.path().join("to_convert");
        let output = root.path().join("converted");
        std::fs::create_dir(&input).unwrap();
        std::fs::write(input.join("broken.xls"), b"this is not an xls file").unwrap();
        std::fs::copy(example_xls(), input.join("good.xls")).unwrap();

        let files = vec![input.join("broken.xls"), input.join("good.xls")];
        let results = convert_files(&input, &files, &output, false, |_| {}).unwrap();

        assert_eq!(results[0].status, "error");
        assert!(!results[0].message.is_empty());
        assert_eq!(results[1].status, "converted");
    }

    #[test]
    fn convert_files_reports_progress() {
        let root = tempfile::tempdir().unwrap();
        let input = root.path().join("to_convert");
        let output = root.path().join("converted");
        std::fs::create_dir(&input).unwrap();
        std::fs::copy(example_xls(), input.join("stmt.xls")).unwrap();

        let mut seen = Vec::new();
        convert_files(&input, &[input.join("stmt.xls")], &output, false, |o| {
            seen.push(o.status.clone())
        })
        .unwrap();
        assert_eq!(seen, vec!["converted".to_string()]);
    }

    #[test]
    fn convert_files_surfaces_row_warnings() {
        // a file whose XLS structure is valid but contains one unparseable row
        // cannot be fabricated easily; instead exercise the warning plumbing
        // directly at the converter level and the outcome mapping here.
        let root = tempfile::tempdir().unwrap();
        let input = root.path().join("to_convert");
        let output = root.path().join("converted");
        std::fs::create_dir(&input).unwrap();
        std::fs::copy(example_xls(), input.join("stmt.xls")).unwrap();

        let results =
            convert_files(&input, &[input.join("stmt.xls")], &output, false, |_| {}).unwrap();
        // the reference statement parses cleanly: warnings empty but PRESENT in the outcome
        assert_eq!(results[0].status, "converted");
        assert!(results[0].warnings.is_empty());
    }

    #[test]
    fn hash_file_known_sha256() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(b"hello").unwrap();
        f.flush().unwrap();
        // echo -n hello | sha256sum
        assert_eq!(
            hash_file(f.path()).unwrap(),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn log_path_is_parent_of_input_dir() {
        let p = log_path(Path::new("C:/repo/to_convert"));
        assert_eq!(p, PathBuf::from("C:/repo/.conversion_log.json"));
    }

    #[test]
    fn log_path_falls_back_into_input_dir_without_parent() {
        // bare relative dir: parent() is Some("") — must not use it
        let p = log_path(Path::new("to_convert"));
        assert_eq!(p, PathBuf::from("to_convert/.conversion_log.json"));
    }

    #[test]
    fn log_path_falls_back_for_drive_root() {
        // Path::new("C:/").parent() is None on Windows — distinct from the Some("") case
        let p = log_path(Path::new("C:/"));
        assert_eq!(p, PathBuf::from("C:/.conversion_log.json"));
    }

    #[test]
    fn log_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".conversion_log.json");
        let mut log = ConversionLog::new();
        log.insert(
            "a.xls".to_string(),
            LogEntry {
                hash: "abc".to_string(),
                converted_at: "2026-06-12T10:00:00.000000".to_string(),
                output_file: "C:/out/a.csv".to_string(),
            },
        );
        save_log(&path, &log).unwrap();
        let (loaded, warning) = load_log(&path);
        assert!(warning.is_none());
        assert_eq!(loaded.get("a.xls").unwrap().hash, "abc");
    }

    #[test]
    fn missing_log_is_empty_without_warning() {
        let (log, warning) = load_log(Path::new("does/not/exist.json"));
        assert!(log.is_empty());
        assert!(warning.is_none());
    }

    #[test]
    fn corrupt_log_starts_fresh_with_warning() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".conversion_log.json");
        std::fs::write(&path, "{ not json").unwrap();
        let (log, warning) = load_log(&path);
        assert!(log.is_empty());
        assert!(warning.is_some());
    }

    #[test]
    fn classify_statuses() {
        let mut log = ConversionLog::new();
        log.insert(
            "a.xls".to_string(),
            LogEntry {
                hash: "h1".to_string(),
                converted_at: String::new(),
                output_file: String::new(),
            },
        );
        assert_eq!(classify(&log, "b.xls", "h9"), FileStatus::New);
        assert_eq!(classify(&log, "a.xls", "h1"), FileStatus::Converted);
        assert_eq!(classify(&log, "a.xls", "h2"), FileStatus::Changed);
    }

    #[test]
    fn log_reads_python_written_format() {
        // exactly what batch_convert.py writes with json.dump(indent=2)
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".conversion_log.json");
        std::fs::write(
            &path,
            r#"{
  "202601_Statement_TZS.xls": {
    "hash": "deadbeef",
    "converted_at": "2026-06-01T10:24:00.123456",
    "output_file": "/repo/converted/202601_Statement_TZS.csv"
  }
}"#,
        )
        .unwrap();
        let (log, warning) = load_log(&path);
        assert!(warning.is_none());
        assert_eq!(log.get("202601_Statement_TZS.xls").unwrap().hash, "deadbeef");
    }

    #[test]
    fn scan_finds_xls_with_status() {
        let root = tempfile::tempdir().unwrap();
        let input = root.path().join("to_convert");
        std::fs::create_dir(&input).unwrap();
        std::fs::write(input.join("b.xls"), b"bbb").unwrap();
        std::fs::write(input.join("a.xls"), b"aaa").unwrap();
        std::fs::write(input.join("ignore.txt"), b"x").unwrap();

        // pre-seed the log so a.xls counts as already converted
        let mut log = ConversionLog::new();
        log.insert(
            "a.xls".to_string(),
            LogEntry {
                hash: hash_file(&input.join("a.xls")).unwrap(),
                converted_at: String::new(),
                output_file: String::new(),
            },
        );
        save_log(&log_path(&input), &log).unwrap();

        let result = scan_input_dir(&input).unwrap();
        assert_eq!(result.files.len(), 2);
        assert_eq!(result.files[0].name, "a.xls");
        assert_eq!(result.files[0].status, FileStatus::Converted);
        assert_eq!(result.files[1].name, "b.xls");
        assert_eq!(result.files[1].status, FileStatus::New);
        assert_eq!(result.files[1].size, 3);
        assert!(result.log_warning.is_none());
    }

    #[test]
    fn scan_surfaces_corrupt_log_warning() {
        let root = tempfile::tempdir().unwrap();
        let input = root.path().join("to_convert");
        std::fs::create_dir(&input).unwrap();
        std::fs::write(input.join("a.xls"), b"aaa").unwrap();
        std::fs::write(log_path(&input), "{ not json").unwrap();

        let result = scan_input_dir(&input).unwrap();
        assert_eq!(result.files.len(), 1);
        assert_eq!(result.files[0].status, FileStatus::New);
        assert!(result.log_warning.is_some());
    }
}
