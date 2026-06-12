//! Batch conversion with SHA256 dedup, log-compatible with batch_convert.py.

use std::path::{Path, PathBuf};

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
}
