//! Phase 122.2 §2 path→version rewriter (Phase 127 D-11, D-12 toml_edit migration).
//!
//! At `docker:init` time we read the project `Cargo.toml`, find every `ferro*`
//! dependency declared as a path dep, and rewrite it as a version dep into
//! `Cargo.docker.toml`. The Dockerfile then `COPY Cargo.docker.toml Cargo.toml`
//! before any cargo work, so the build pulls ferro from crates.io rather than
//! requiring the workspace checkout to be present in the build context.
//!
//! Uses `toml_edit` rather than the value-level `toml` crate so dependency
//! table key order, sibling tables, and whitespace survive the rewrite
//! byte-for-byte — the only mutations are `path` removal and `version`
//! insertion inside the touched ferro* deps.

use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, Item, Value};

const DEP_TABLES: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

/// Rewrite the project Cargo.toml into `<project_root>/Cargo.docker.toml`,
/// replacing every `ferro*` path dep with a version dep. Returns the path of
/// the file written.
///
/// Thin wrapper around [`compute_cargo_docker_toml`] + [`persist_cargo_docker_toml`]
/// — callers that need a `--dry-run` path should use `compute_*` directly and
/// skip the persist step (Phase 127 D-18).
pub fn rewrite_cargo_docker_toml(
    project_root: &Path,
    ferro_version_override: Option<&str>,
) -> anyhow::Result<PathBuf> {
    let rewritten = compute_cargo_docker_toml(project_root, ferro_version_override)?;
    let out_path = project_root.join("Cargo.docker.toml");
    persist_cargo_docker_toml(&out_path, &rewritten)?;
    Ok(out_path)
}

/// Pure compute — reads source `Cargo.toml` from disk and returns the
/// rewritten `Cargo.docker.toml` contents as a `String`. Does NOT write.
///
/// The only filesystem reads are the project `Cargo.toml` itself and any
/// `ferro*` path-dep `Cargo.toml` files consulted to resolve workspace
/// versions — no target files are created or modified. Used by `--dry-run`
/// (Phase 127 D-18) and as the first half of [`rewrite_cargo_docker_toml`].
pub fn compute_cargo_docker_toml(
    project_root: &Path,
    ferro_version_override: Option<&str>,
) -> anyhow::Result<String> {
    let cargo_path = project_root.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_path)
        .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", cargo_path.display()))?;
    rewrite_contents(&content, project_root, ferro_version_override)
}

/// Persist previously-computed `Cargo.docker.toml` contents at `path`.
///
/// Split from [`compute_cargo_docker_toml`] so `--dry-run` can skip this
/// step entirely (Phase 127 D-18).
pub fn persist_cargo_docker_toml(path: &Path, contents: &str) -> anyhow::Result<()> {
    fs::write(path, contents)
        .map_err(|e| anyhow::anyhow!("failed to write {}: {e}", path.display()))?;
    Ok(())
}

/// Pure string-in/string-out rewriter used by `rewrite_cargo_docker_toml`
/// and the regression tests. Preserves dependency-table key order.
pub fn rewrite_contents(
    contents: &str,
    project_root: &Path,
    ferro_version_override: Option<&str>,
) -> anyhow::Result<String> {
    let mut doc: DocumentMut = contents
        .parse()
        .map_err(|e| anyhow::anyhow!("failed to parse Cargo.toml: {e}"))?;

    for table_name in DEP_TABLES {
        let Some(table_item) = doc.get_mut(table_name) else {
            continue;
        };
        let Some(table) = table_item.as_table_like_mut() else {
            continue;
        };

        // Collect ferro* keys that are path deps. Iterate by key name to avoid
        // aliasing the table while mutating.
        let ferro_keys: Vec<String> = table
            .iter()
            .filter_map(|(k, v)| {
                if !k.starts_with("ferro") {
                    return None;
                }
                let is_path_dep = match v {
                    Item::Value(Value::InlineTable(t)) => t.contains_key("path"),
                    Item::Table(t) => t.contains_key("path"),
                    _ => false,
                };
                if is_path_dep {
                    Some(k.to_string())
                } else {
                    None
                }
            })
            .collect();

        for key in ferro_keys {
            let path_str: Option<String> = match table.get(&key) {
                Some(Item::Value(Value::InlineTable(t))) => {
                    t.get("path").and_then(|v| v.as_str()).map(String::from)
                }
                Some(Item::Table(t)) => t
                    .get("path")
                    .and_then(|i| i.as_value())
                    .and_then(|v| v.as_str())
                    .map(String::from),
                _ => None,
            };

            let version = match ferro_version_override {
                Some(v) => v.to_string(),
                None => path_str
                    .as_deref()
                    .and_then(|p| super::read_path_dep_version(project_root, p))
                    .unwrap_or_else(|| "*".to_string()),
            };

            match table.get_mut(&key) {
                Some(Item::Value(Value::InlineTable(t))) => {
                    t.remove("path");
                    // Overwrite-or-insert `version` while preserving sibling
                    // keys (package, features, default-features, optional,
                    // registry, rename, etc.) in their original order.
                    t.insert("version", Value::from(version));
                }
                Some(Item::Table(t)) => {
                    t.remove("path");
                    t.insert("version", toml_edit::value(version));
                }
                _ => {}
            }
        }
    }

    Ok(doc.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn override_wins_over_workspace_version() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("project");
        write(
            &project.join("Cargo.toml"),
            r#"
[package]
name = "demo"
version = "0.1.0"

[dependencies]
ferro = { path = "../framework" }
"#,
        );
        write(
            &tmp.path().join("framework/Cargo.toml"),
            "[package]\nname = \"ferro\"\nversion = \"9.9.9\"\n",
        );

        let out = rewrite_cargo_docker_toml(&project, Some("0.1.87")).unwrap();
        let body = fs::read_to_string(&out).unwrap();
        assert!(body.contains("0.1.87"));
        assert!(!body.contains("path ="));
    }

    #[test]
    fn reads_workspace_version_when_no_override() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("project");
        write(
            &project.join("Cargo.toml"),
            r#"
[package]
name = "demo"
version = "0.1.0"

[dependencies]
ferro = { path = "../framework" }
"#,
        );
        write(
            &tmp.path().join("framework/Cargo.toml"),
            "[package]\nname = \"ferro\"\nversion = \"0.1.87\"\n",
        );

        let out = rewrite_cargo_docker_toml(&project, None).unwrap();
        let body = fs::read_to_string(&out).unwrap();
        assert!(body.contains("0.1.87"));
        assert!(!body.contains("path ="));
    }

    #[test]
    fn falls_back_to_star_when_path_dep_unreadable() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("project");
        write(
            &project.join("Cargo.toml"),
            r#"
[package]
name = "demo"
version = "0.1.0"

[dependencies]
ferro = { path = "../missing" }
"#,
        );

        let out = rewrite_cargo_docker_toml(&project, None).unwrap();
        let body = fs::read_to_string(&out).unwrap();
        assert!(body.contains("\"*\""));
    }

    #[test]
    fn rewrites_multiple_ferro_deps() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("project");
        write(
            &project.join("Cargo.toml"),
            r#"
[package]
name = "demo"
version = "0.1.0"

[dependencies]
ferro = { path = "../framework" }
ferro-macros = { path = "../ferro-macros" }
ferro-events = { path = "../ferro-events" }
serde = "1"
"#,
        );
        for (dir, ver) in [
            ("framework", "0.1.87"),
            ("ferro-macros", "0.1.87"),
            ("ferro-events", "0.1.87"),
        ] {
            write(
                &tmp.path().join(format!("{dir}/Cargo.toml")),
                &format!("[package]\nname = \"x\"\nversion = \"{ver}\"\n"),
            );
        }

        let out = rewrite_cargo_docker_toml(&project, None).unwrap();
        let body = fs::read_to_string(&out).unwrap();
        assert_eq!(body.matches("0.1.87").count(), 3);
        assert!(!body.contains("path ="));
        assert!(body.contains("serde"));
    }

    /// Regression: gestiscilo declares `ferro = { path = "...", package = "ferro-rs",
    /// features = ["json-ui", "theme"] }`. The rewriter must preserve `package` and
    /// `features` so the resulting dep resolves to crate `ferro-rs` on crates.io
    /// with the requested features, not a phantom crate literally named `ferro`.
    #[test]
    fn preserves_package_rename_and_features() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("project");
        write(
            &project.join("Cargo.toml"),
            r#"
[package]
name = "demo"
version = "0.1.0"

[dependencies]
ferro = { path = "../framework", package = "ferro-rs", features = ["json-ui", "theme"] }
ferro-json-ui = { path = "../ferro-json-ui", default-features = false, optional = true }
"#,
        );
        write(
            &tmp.path().join("framework/Cargo.toml"),
            "[package]\nname = \"ferro-rs\"\nversion = \"0.2.0\"\n",
        );
        write(
            &tmp.path().join("ferro-json-ui/Cargo.toml"),
            "[package]\nname = \"ferro-json-ui\"\nversion = \"0.2.0\"\n",
        );

        let out = rewrite_cargo_docker_toml(&project, Some("0.2.0")).unwrap();
        let body = fs::read_to_string(&out).unwrap();
        // Parse with toml (value-level) to sanity-check semantic content.
        let parsed: toml::Value = body.parse().unwrap();
        let deps = parsed
            .get("dependencies")
            .and_then(|v| v.as_table())
            .unwrap();

        let ferro = deps.get("ferro").and_then(|v| v.as_table()).unwrap();
        assert_eq!(ferro.get("version").and_then(|v| v.as_str()), Some("0.2.0"));
        assert_eq!(
            ferro.get("package").and_then(|v| v.as_str()),
            Some("ferro-rs"),
            "package rename must survive rewrite"
        );
        let features: Vec<&str> = ferro
            .get("features")
            .and_then(|v| v.as_array())
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        assert_eq!(features, vec!["json-ui", "theme"]);
        assert!(ferro.get("path").is_none(), "path must be stripped");

        let json_ui = deps
            .get("ferro-json-ui")
            .and_then(|v| v.as_table())
            .unwrap();
        assert_eq!(
            json_ui.get("default-features").and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            json_ui.get("optional").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert!(json_ui.get("path").is_none());
    }

    #[test]
    fn leaves_non_ferro_deps_untouched() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("project");
        write(
            &project.join("Cargo.toml"),
            r#"
[package]
name = "demo"
version = "0.1.0"

[dependencies]
mything = { path = "../mything" }
serde = "1"
"#,
        );
        let out = rewrite_cargo_docker_toml(&project, Some("0.1.87")).unwrap();
        let body = fs::read_to_string(&out).unwrap();
        assert!(body.contains("path"));
        assert!(body.contains("mything"));
        assert!(body.contains("serde"));
    }

    /// D-18: `compute_cargo_docker_toml` returns rewritten contents without
    /// creating or modifying `Cargo.docker.toml` on disk.
    #[test]
    fn compute_returns_string_without_writing() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("project");
        write(
            &project.join("Cargo.toml"),
            r#"
[package]
name = "demo"
version = "0.1.0"

[dependencies]
ferro = { path = "../framework" }
"#,
        );
        write(
            &tmp.path().join("framework/Cargo.toml"),
            "[package]\nname = \"ferro\"\nversion = \"0.2.0\"\n",
        );

        let out = compute_cargo_docker_toml(&project, Some("0.2.0")).expect("compute");
        assert!(out.contains("ferro = { version = \"0.2.0\""));
        assert!(
            !project.join("Cargo.docker.toml").exists(),
            "compute must not write Cargo.docker.toml"
        );
    }

    /// Phase 129 / REPORT §14: `ferro_versions` override table is a schema
    /// reservation parsed in `project.rs` but not yet consumed by the rewriter.
    /// This test locks in the current toml_edit behavior — the
    /// `[package.metadata.ferro.deploy.ferro_versions]` table must survive
    /// `rewrite_cargo_docker_toml` byte-identically because the rewriter only
    /// mutates `[dependencies]*` tables.
    #[test]
    fn preserves_ferro_versions_override_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("project");
        write(
            &project.join("Cargo.toml"),
            r#"
[package]
name = "demo"
version = "0.1.0"

[dependencies]
ferro = { path = "../framework" }

[package.metadata.ferro.deploy]
ferro_version = "0.2.0"

[package.metadata.ferro.deploy.ferro_versions]
ferro-json-ui = "0.2.1"
ferro-whatsapp = "0.2.0"
"#,
        );
        write(
            &tmp.path().join("framework/Cargo.toml"),
            "[package]\nname = \"ferro\"\nversion = \"0.2.0\"\n",
        );

        let out = rewrite_cargo_docker_toml(&project, Some("0.2.0")).unwrap();
        let body = fs::read_to_string(&out).unwrap();

        // Dep table rewritten as expected.
        assert!(
            body.contains("ferro = { version = \"0.2.0\""),
            "ferro dep should be rewritten to a version dep: {body}"
        );
        assert!(
            !body.contains("../framework"),
            "path dep should be stripped: {body}"
        );

        // ferro_versions override survives byte-wise.
        assert!(
            body.contains("[package.metadata.ferro.deploy.ferro_versions]"),
            "override table header missing: {body}"
        );
        assert!(
            body.contains("ferro-json-ui = \"0.2.1\""),
            "override entry missing: {body}"
        );
        assert!(
            body.contains("ferro-whatsapp = \"0.2.0\""),
            "override entry missing: {body}"
        );

        // And semantically.
        let parsed: toml::Value = body.parse().unwrap();
        let overrides = parsed
            .get("package")
            .unwrap()
            .get("metadata")
            .unwrap()
            .get("ferro")
            .unwrap()
            .get("deploy")
            .unwrap()
            .get("ferro_versions")
            .unwrap()
            .as_table()
            .unwrap();
        assert_eq!(
            overrides.get("ferro-json-ui").and_then(|v| v.as_str()),
            Some("0.2.1")
        );
        assert_eq!(
            overrides.get("ferro-whatsapp").and_then(|v| v.as_str()),
            Some("0.2.0")
        );
    }

    /// D-18: `persist_cargo_docker_toml` writes exactly the supplied bytes.
    #[test]
    fn persist_writes_computed_contents() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("Cargo.docker.toml");
        persist_cargo_docker_toml(&target, "foo = \"bar\"\n").expect("persist");
        assert_eq!(fs::read_to_string(&target).unwrap(), "foo = \"bar\"\n");
    }

    /// D-11: dependency table key order must survive the rewrite. `toml_edit`
    /// keeps keys in source order, unlike the value-level `toml` crate which
    /// re-serialized alphabetically.
    #[test]
    fn preserves_dep_table_order() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("project");
        write(
            &project.join("Cargo.toml"),
            r#"
[package]
name = "app"
version = "0.1.0"

[dependencies]
zed = "1.0"
alpha = "2.0"
ferro = { path = "../framework" }
middle = "3.0"
beta = "4.0"
gamma = "5.0"
"#,
        );
        write(
            &tmp.path().join("framework/Cargo.toml"),
            "[package]\nname = \"ferro\"\nversion = \"0.2.0\"\n",
        );

        let out = rewrite_cargo_docker_toml(&project, Some("0.2.0")).unwrap();
        let body = fs::read_to_string(&out).unwrap();

        // Extract key names from the [dependencies] block preserving order.
        let deps_block = body
            .split("[dependencies]")
            .nth(1)
            .expect("deps block present");
        let keys: Vec<&str> = deps_block
            .lines()
            .filter_map(|l| {
                let t = l.trim();
                if t.is_empty() || t.starts_with('[') || t.starts_with('#') {
                    return None;
                }
                t.split('=').next().map(str::trim)
            })
            .take(6)
            .collect();

        assert_eq!(
            keys,
            vec!["zed", "alpha", "ferro", "middle", "beta", "gamma"]
        );
        assert!(!body.contains("../framework"));
        assert!(body.contains("ferro = { version = \"0.2.0\""));
    }
}
