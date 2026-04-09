//! Deploy scaffold primitives. Phase 122.2 reduced this surface dramatically;
//! after plan 122.2-04 only the helpers consumed by surviving doctor checks
//! remain. Plans 06..12 will replace these with the new design in SCOPE.

#![allow(dead_code)]

pub mod bin_detect;
pub mod env_production;
pub mod rewrite_ferro_version;
pub mod secret_keys;

use std::path::Path;
use toml::Value;

/// Read `package.version` from a path dep's `Cargo.toml`. Shared by
/// `cargo_docker_toml_staleness` and `ferro_version_skew` checks and by
/// the `Cargo.docker.toml` rewriter. Returns `None` if the file is
/// missing, unparseable, or lacks `package.version`.
pub(crate) fn read_path_dep_version(project_root: &Path, rel_path: &str) -> Option<String> {
    let dep_cargo = project_root.join(rel_path).join("Cargo.toml");
    let content = std::fs::read_to_string(&dep_cargo).ok()?;
    let parsed: Value = content.parse().ok()?;
    parsed
        .get("package")?
        .get("version")?
        .as_str()
        .map(String::from)
}

// ---------------------------------------------------------------------------
// Env parsing (.env / .env.example / .env.production)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvEntry {
    pub key: String,
    pub value: String,
}

/// Parse a `.env`-style file body into ordered (key, value) entries.
/// Skips blank and comment lines. Splits on the first `=`. Strips a trailing
/// ` # comment` from unquoted values.
pub fn parse_env_entries(content: &str) -> Vec<EnvEntry> {
    let mut out = Vec::new();
    for raw in content.lines() {
        let trimmed = raw.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = raw.split_once('=') else {
            continue;
        };
        let key = key.trim().to_string();
        if key.is_empty() {
            continue;
        }
        let stripped = strip_inline_comment(value.trim());
        out.push(EnvEntry {
            key,
            value: stripped.to_string(),
        });
    }
    out
}

fn strip_inline_comment(value: &str) -> &str {
    let bytes = value.as_bytes();
    if bytes.first() == Some(&b'"') || bytes.first() == Some(&b'\'') {
        return value;
    }
    let mut prev_ws = false;
    for (i, b) in bytes.iter().enumerate() {
        if *b == b'#' && prev_ws {
            return value[..i].trim_end();
        }
        prev_ws = *b == b' ' || *b == b'\t';
    }
    value
}

// ---------------------------------------------------------------------------
// ferro path dep discovery (used by doctor path check)
// ---------------------------------------------------------------------------

const DEP_TABLES: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

/// Walk dependency tables of a parsed Cargo.toml string and collect every key
/// starting with `ferro` whose value is a table containing a `path` field.
pub fn find_ferro_path_deps(content: &str) -> Vec<String> {
    let parsed: Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let mut out: Vec<String> = Vec::new();
    for table_name in DEP_TABLES {
        let Some(table) = parsed.get(*table_name).and_then(|v| v.as_table()) else {
            continue;
        };
        for (key, value) in table {
            if !key.starts_with("ferro") {
                continue;
            }
            let is_path_dep = value
                .as_table()
                .map(|t| t.contains_key("path"))
                .unwrap_or(false);
            if is_path_dep && !out.iter().any(|k| k == key) {
                out.push(key.clone());
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_env_entries_basic() {
        let got = parse_env_entries("# c\nFOO=bar\n\nBAZ=qux # tail");
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].key, "FOO");
        assert_eq!(got[0].value, "bar");
        assert_eq!(got[1].value, "qux");
    }

    #[test]
    fn finds_ferro_path_deps() {
        let deps = find_ferro_path_deps("[dependencies]\nferro = { path = \"..\" }\n");
        assert_eq!(deps, vec!["ferro".to_string()]);
    }
}
