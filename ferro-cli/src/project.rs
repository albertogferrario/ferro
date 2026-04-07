//! Shared project introspection helpers used by deploy-scaffold commands
//! (`docker:init`, `do:init`) and other ferro-cli commands that need to read
//! `Cargo.toml`, detect optional project directories, or resolve the Rust
//! toolchain version.
//!
//! All helpers are deliberately tolerant of malformed or missing input: they
//! return safe defaults rather than propagating errors, mirroring the legacy
//! `get_package_name()` behavior they replace.

#![allow(dead_code)] // Consumed by plans 122-02..07; tests cover the full surface.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use toml::Value;

const DEFAULT_RUST_IMAGE: &str = "rust:1.88-slim-bookworm";
const DEFAULT_PACKAGE_NAME: &str = "app";

/// Phase 122.2 §1: provider-neutral Dockerfile inputs declared by the project
/// in `[package.metadata.ferro.deploy]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FerroDeployMetadata {
    pub runtime_apt: Vec<String>,
    pub copy_dirs: Vec<String>,
    pub ferro_version: Option<String>,
}

impl Default for FerroDeployMetadata {
    fn default() -> Self {
        Self {
            runtime_apt: vec![],
            copy_dirs: vec![
                "themes".into(),
                "lang".into(),
                "public".into(),
                "migrations".into(),
            ],
            ferro_version: None,
        }
    }
}

/// Read `<root>/Cargo.toml` and return `[package.metadata.ferro.deploy]`.
/// Missing table → defaults. Invalid field types → Err.
pub fn read_deploy_metadata(project_root: &Path) -> anyhow::Result<FerroDeployMetadata> {
    let path = project_root.join("Cargo.toml");
    let content = fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()))?;
    let parsed: Value = content
        .parse()
        .map_err(|e| anyhow::anyhow!("failed to parse {}: {e}", path.display()))?;

    let Some(table) = parsed
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("ferro"))
        .and_then(|f| f.get("deploy"))
    else {
        return Ok(FerroDeployMetadata::default());
    };

    let mut meta = FerroDeployMetadata::default();

    if let Some(v) = table.get("runtime_apt") {
        let arr = v.as_array().ok_or_else(|| {
            anyhow::anyhow!("[package.metadata.ferro.deploy].runtime_apt must be an array")
        })?;
        meta.runtime_apt = arr
            .iter()
            .map(|item| {
                item.as_str().map(String::from).ok_or_else(|| {
                    anyhow::anyhow!(
                        "[package.metadata.ferro.deploy].runtime_apt entries must be strings"
                    )
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
    }

    if let Some(v) = table.get("copy_dirs") {
        let arr = v.as_array().ok_or_else(|| {
            anyhow::anyhow!("[package.metadata.ferro.deploy].copy_dirs must be an array")
        })?;
        meta.copy_dirs = arr
            .iter()
            .map(|item| {
                item.as_str().map(String::from).ok_or_else(|| {
                    anyhow::anyhow!(
                        "[package.metadata.ferro.deploy].copy_dirs entries must be strings"
                    )
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
    }

    if let Some(v) = table.get("ferro_version") {
        let s = v.as_str().ok_or_else(|| {
            anyhow::anyhow!("[package.metadata.ferro.deploy].ferro_version must be a string")
        })?;
        meta.ferro_version = Some(s.to_string());
    }

    Ok(meta)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinEntry {
    pub name: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectDirs {
    pub has_frontend: bool,
    pub has_themes: bool,
    pub has_lang: bool,
    pub has_public: bool,
    pub has_migrations: bool,
}

/// Walk up from `start` (or the current working directory if `None`) until a
/// directory containing `Cargo.toml` is found. Returns `NotFound` otherwise.
pub fn find_project_root(start: Option<&Path>) -> io::Result<PathBuf> {
    let mut dir = match start {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir()?,
    };

    loop {
        if dir.join("Cargo.toml").is_file() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Cargo.toml not found (searched upward from start)",
            ));
        }
    }
}

/// Parse `<root>/Cargo.toml` and return the `[package] name`. Returns
/// `"app"` on any failure (missing file, parse error, missing key).
pub fn package_name(root: &Path) -> String {
    let parsed = match read_cargo_toml(root) {
        Some(v) => v,
        None => return DEFAULT_PACKAGE_NAME.to_string(),
    };

    parsed
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or(DEFAULT_PACKAGE_NAME)
        .to_string()
}

/// Parse `<root>/Cargo.toml` and return every `[[bin]]` entry. If no
/// `[[bin]]` table exists, synthesize one entry from `[package] name`.
/// Returns an empty Vec only if `Cargo.toml` is unreadable.
pub fn read_bins(root: &Path) -> Vec<BinEntry> {
    let parsed = match read_cargo_toml(root) {
        Some(v) => v,
        None => return Vec::new(),
    };

    let bins = parsed
        .get("bin")
        .and_then(|b| b.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|entry| {
                    let name = entry.get("name").and_then(|n| n.as_str())?.to_string();
                    let path = entry.get("path").and_then(|p| p.as_str()).map(String::from);
                    Some(BinEntry { name, path })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if bins.is_empty() {
        return vec![BinEntry {
            name: parsed
                .get("package")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or(DEFAULT_PACKAGE_NAME)
                .to_string(),
            path: None,
        }];
    }

    bins
}

/// Parse `<root>/Cargo.toml` and return `[workspace] members` verbatim.
/// Returns an empty Vec when no `[workspace]` table exists or parsing fails.
pub fn read_workspace_members(root: &Path) -> Vec<String> {
    let parsed = match read_cargo_toml(root) {
        Some(v) => v,
        None => return Vec::new(),
    };

    parsed
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// If `<root>/rust-toolchain.toml` exists and contains
/// `[toolchain] channel = "X.Y.Z"`, return `rust:X.Y.Z-slim-bookworm`.
/// Else return the hardcoded default `rust:1.88-slim-bookworm`.
pub fn resolve_rust_base_image(root: &Path) -> String {
    let path = root.join("rust-toolchain.toml");
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return DEFAULT_RUST_IMAGE.to_string(),
    };

    let parsed: Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return DEFAULT_RUST_IMAGE.to_string(),
    };

    parsed
        .get("toolchain")
        .and_then(|t| t.get("channel"))
        .and_then(|c| c.as_str())
        .map(|channel| format!("rust:{channel}-slim-bookworm"))
        .unwrap_or_else(|| DEFAULT_RUST_IMAGE.to_string())
}

/// Probe the project root for the optional directories used by the deploy
/// scaffold templates.
pub fn detect_dirs(root: &Path) -> ProjectDirs {
    ProjectDirs {
        has_frontend: root.join("frontend/package.json").is_file(),
        has_themes: root.join("themes").is_dir(),
        has_lang: root.join("lang").is_dir(),
        has_public: root.join("public").is_dir(),
        has_migrations: root.join("migrations").is_dir(),
    }
}

fn read_cargo_toml(root: &Path) -> Option<Value> {
    let content = fs::read_to_string(root.join("Cargo.toml")).ok()?;
    content.parse::<Value>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write(root: &Path, rel: &str, content: &str) {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn find_project_root_walks_up_to_cargo_toml() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        write(root, "Cargo.toml", "[package]\nname = \"x\"\n");
        let nested = root.join("a/b/c");
        fs::create_dir_all(&nested).unwrap();

        let found = find_project_root(Some(&nested)).unwrap();
        assert_eq!(found, root);
    }

    #[test]
    fn find_project_root_returns_not_found_when_absent() {
        let tmp = TempDir::new().unwrap();
        let nested = tmp.path().join("a/b");
        fs::create_dir_all(&nested).unwrap();

        let err = find_project_root(Some(&nested)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn package_name_parses_valid_cargo_toml() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "Cargo.toml", "[package]\nname = \"foo\"\n");
        assert_eq!(package_name(tmp.path()), "foo");
    }

    #[test]
    fn package_name_falls_back_to_app() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(package_name(tmp.path()), "app");
    }

    #[test]
    fn read_bins_returns_explicit_entries() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "Cargo.toml",
            r#"
[package]
name = "multi"

[[bin]]
name = "alpha"

[[bin]]
name = "beta"
path = "src/bin/beta.rs"
"#,
        );

        let bins = read_bins(tmp.path());
        assert_eq!(bins.len(), 2);
        assert_eq!(
            bins[0],
            BinEntry {
                name: "alpha".into(),
                path: None
            }
        );
        assert_eq!(
            bins[1],
            BinEntry {
                name: "beta".into(),
                path: Some("src/bin/beta.rs".into()),
            }
        );
    }

    #[test]
    fn read_bins_synthesizes_from_package_when_no_bin_table() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "Cargo.toml", "[package]\nname = \"solo\"\n");
        let bins = read_bins(tmp.path());
        assert_eq!(
            bins,
            vec![BinEntry {
                name: "solo".into(),
                path: None
            }]
        );
    }

    #[test]
    fn read_bins_returns_empty_when_cargo_toml_missing() {
        let tmp = TempDir::new().unwrap();
        assert!(read_bins(tmp.path()).is_empty());
    }

    #[test]
    fn read_workspace_members_returns_declared_members() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "Cargo.toml",
            "[workspace]\nmembers = [\"a\", \"b\"]\n",
        );
        assert_eq!(
            read_workspace_members(tmp.path()),
            vec!["a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn read_workspace_members_empty_without_workspace_table() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "Cargo.toml", "[package]\nname = \"x\"\n");
        assert!(read_workspace_members(tmp.path()).is_empty());
    }

    #[test]
    fn resolve_rust_base_image_uses_toolchain_channel() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "rust-toolchain.toml",
            "[toolchain]\nchannel = \"1.90.0\"\n",
        );
        assert_eq!(
            resolve_rust_base_image(tmp.path()),
            "rust:1.90.0-slim-bookworm"
        );
    }

    #[test]
    fn resolve_rust_base_image_falls_back_when_missing() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(resolve_rust_base_image(tmp.path()), DEFAULT_RUST_IMAGE);
    }

    #[test]
    fn detect_dirs_reports_present_directories_only() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("themes")).unwrap();
        fs::create_dir_all(tmp.path().join("public")).unwrap();

        let dirs = detect_dirs(tmp.path());
        assert_eq!(
            dirs,
            ProjectDirs {
                has_themes: true,
                has_public: true,
                ..Default::default()
            }
        );
    }

    #[test]
    fn read_deploy_metadata_full_table() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "Cargo.toml",
            r#"
[package]
name = "x"

[package.metadata.ferro.deploy]
runtime_apt = ["chromium", "fonts-liberation"]
copy_dirs = ["themes", "public"]
ferro_version = "0.1.87"
"#,
        );
        let m = read_deploy_metadata(tmp.path()).unwrap();
        assert_eq!(m.runtime_apt, vec!["chromium", "fonts-liberation"]);
        assert_eq!(m.copy_dirs, vec!["themes", "public"]);
        assert_eq!(m.ferro_version.as_deref(), Some("0.1.87"));
    }

    #[test]
    fn read_deploy_metadata_partial_uses_defaults() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "Cargo.toml",
            r#"
[package]
name = "x"

[package.metadata.ferro.deploy]
runtime_apt = ["chromium"]
"#,
        );
        let m = read_deploy_metadata(tmp.path()).unwrap();
        assert_eq!(m.runtime_apt, vec!["chromium"]);
        assert_eq!(m.copy_dirs, vec!["themes", "lang", "public", "migrations"]);
        assert_eq!(m.ferro_version, None);
    }

    #[test]
    fn read_deploy_metadata_missing_table_returns_default() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "Cargo.toml", "[package]\nname = \"x\"\n");
        let m = read_deploy_metadata(tmp.path()).unwrap();
        assert_eq!(m, FerroDeployMetadata::default());
    }

    #[test]
    fn read_deploy_metadata_invalid_type_errors() {
        let tmp = TempDir::new().unwrap();
        write(
            tmp.path(),
            "Cargo.toml",
            r#"
[package]
name = "x"

[package.metadata.ferro.deploy]
runtime_apt = "not-an-array"
"#,
        );
        assert!(read_deploy_metadata(tmp.path()).is_err());
    }

    #[test]
    fn detect_dirs_has_frontend_requires_package_json_file() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("frontend")).unwrap();
        assert!(!detect_dirs(tmp.path()).has_frontend);

        write(tmp.path(), "frontend/package.json", "{}");
        assert!(detect_dirs(tmp.path()).has_frontend);
    }
}
