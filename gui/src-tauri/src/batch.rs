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
            Some(format!("Log unlesbar, beginne neu: {e}")),
        ),
        Ok(text) => match serde_json::from_str(&text) {
            Ok(log) => (log, None),
            Err(e) => (
                ConversionLog::new(),
                Some(format!("Log unlesbar, beginne neu: {e}")),
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
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub status: FileStatus,
}

pub fn scan_input_dir(input_dir: &Path) -> Result<Vec<FileEntry>, String> {
    let (log, _) = load_log(&log_path(input_dir));
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
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

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

        let entries = scan_input_dir(&input).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "a.xls");
        assert_eq!(entries[0].status, FileStatus::Converted);
        assert_eq!(entries[1].name, "b.xls");
        assert_eq!(entries[1].status, FileStatus::New);
        assert_eq!(entries[1].size, 3);
    }
}
