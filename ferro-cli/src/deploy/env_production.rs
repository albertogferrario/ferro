//! Phase 122.2 §6: key-only `.env.production` parser.
//!
//! No value parsing, no comment stripping, no classification. The deploy
//! scaffolder uses these keys to emit a commented `envs:` block in
//! `.do/app.yaml`. Values stay on the developer machine.

use std::fs;
use std::path::Path;

/// Read `.env.production` and return the list of declared keys, in order.
/// Hard-errors when the file is missing — `do:init` requires it.
pub fn read_env_production_keys(path: &Path) -> anyhow::Result<Vec<String>> {
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()))?;
    let mut keys = Vec::new();
    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, _)) = line.split_once('=') {
            let key = k.trim();
            if !key.is_empty() {
                keys.push(key.to_string());
            }
        }
    }
    Ok(keys)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_env(content: &str) -> (TempDir, std::path::PathBuf) {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".env.production");
        fs::write(&path, content).unwrap();
        (tmp, path)
    }

    #[test]
    fn extracts_keys_in_order() {
        let (_tmp, path) = write_env("KEY=value\nOTHER=x\n");
        assert_eq!(
            read_env_production_keys(&path).unwrap(),
            vec!["KEY", "OTHER"]
        );
    }

    #[test]
    fn skips_blank_and_comment_lines() {
        let (_tmp, path) = write_env("\n# a comment\n  # indented comment\nA=1\n\nB=2\n");
        assert_eq!(read_env_production_keys(&path).unwrap(), vec!["A", "B"]);
    }

    #[test]
    fn trims_whitespace_around_keys() {
        let (_tmp, path) = write_env("  KEY = value  \n");
        assert_eq!(read_env_production_keys(&path).unwrap(), vec!["KEY"]);
    }

    #[test]
    fn skips_lines_without_equals() {
        let (_tmp, path) = write_env("not-a-kv-line\nA=1\n");
        assert_eq!(read_env_production_keys(&path).unwrap(), vec!["A"]);
    }

    #[test]
    fn missing_file_errors() {
        let tmp = TempDir::new().unwrap();
        let missing = tmp.path().join(".env.production");
        assert!(read_env_production_keys(&missing).is_err());
    }
}
