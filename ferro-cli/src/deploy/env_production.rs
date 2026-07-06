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

/// Structured line from `.env.example` used by `do:init` envs-block rendering
/// (D-09). Preserves key order and blank-line separators from the source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvLine {
    Key(String),
    Blank,
    Comment,
}

/// Parse `.env.example` preserving key order and blank-line separators.
///
/// Blank lines in the source become a `Blank` variant so the resulting
/// `envs:` block in `.do/app.yaml` keeps the human grouping. Comment lines
/// become `Comment` (consumers may drop them). Keys are trimmed.
pub fn parse_env_example_structured(contents: &str) -> Vec<EnvLine> {
    let mut out = Vec::new();
    for raw in contents.lines() {
        let line = raw.trim_end();
        if line.trim().is_empty() {
            out.push(EnvLine::Blank);
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            out.push(EnvLine::Comment);
            continue;
        }
        if let Some(eq) = trimmed.find('=') {
            let key = trimmed[..eq].trim().to_string();
            if !key.is_empty() {
                out.push(EnvLine::Key(key));
            }
        }
    }
    out
}

#[cfg(test)]
mod structured_tests {
    use super::*;

    #[test]
    fn env_example_parser_preserves_order() {
        let input = "Z=1\nA=2\nM=3\n";
        let out = parse_env_example_structured(input);
        let keys: Vec<_> = out
            .iter()
            .filter_map(|l| match l {
                EnvLine::Key(k) => Some(k.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(keys, vec!["Z", "A", "M"]);
    }

    #[test]
    fn env_example_parser_preserves_blank_separators() {
        let input = "A=1\n\nB=2\n";
        let out = parse_env_example_structured(input);
        assert_eq!(
            out,
            vec![
                EnvLine::Key("A".into()),
                EnvLine::Blank,
                EnvLine::Key("B".into())
            ]
        );
    }

    #[test]
    fn env_example_parser_skips_comments() {
        let input = "# header\nA=1\n";
        let out = parse_env_example_structured(input);
        assert!(out.iter().any(|l| matches!(l, EnvLine::Key(k) if k == "A")));
    }

    #[test]
    fn env_example_parser_trims_keys() {
        let input = "  KEY  =val\n";
        let out = parse_env_example_structured(input);
        assert!(out
            .iter()
            .any(|l| matches!(l, EnvLine::Key(k) if k == "KEY")));
    }
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
