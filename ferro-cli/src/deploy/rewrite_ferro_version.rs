//! Phase 122.2 §2 path→version rewriter.
//!
//! At `docker:init` time we read the project `Cargo.toml`, find every `ferro*`
//! dependency declared as a path dep, and rewrite it as a version dep into
//! `Cargo.docker.toml`. The Dockerfile then `COPY Cargo.docker.toml Cargo.toml`
//! before any cargo work, so the build pulls ferro from crates.io rather than
//! requiring the workspace checkout to be present in the build context.

use std::fs;
use std::path::{Path, PathBuf};
use toml::map::Map;
use toml::Value;

const DEP_TABLES: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

/// Rewrite the project Cargo.toml into `<project_root>/Cargo.docker.toml`,
/// replacing every `ferro*` path dep with a version dep. Returns the path of
/// the file written.
pub fn rewrite_cargo_docker_toml(
    project_root: &Path,
    ferro_version_override: Option<&str>,
) -> anyhow::Result<PathBuf> {
    let cargo_path = project_root.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_path)
        .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", cargo_path.display()))?;
    let mut parsed: Value = content
        .parse()
        .map_err(|e| anyhow::anyhow!("failed to parse {}: {e}", cargo_path.display()))?;

    for table_name in DEP_TABLES {
        let Some(table) = parsed.get_mut(*table_name).and_then(|v| v.as_table_mut()) else {
            continue;
        };
        let keys: Vec<String> = table
            .iter()
            .filter(|(k, v)| {
                k.starts_with("ferro")
                    && v.as_table()
                        .map(|t| t.contains_key("path"))
                        .unwrap_or(false)
            })
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys {
            let path_str = table
                .get(&key)
                .and_then(|v| v.as_table())
                .and_then(|t| t.get("path"))
                .and_then(|p| p.as_str())
                .map(String::from);

            let version = match ferro_version_override {
                Some(v) => v.to_string(),
                None => path_str
                    .as_deref()
                    .and_then(|p| read_path_dep_version(project_root, p))
                    .unwrap_or_else(|| "*".to_string()),
            };

            let mut replacement = Map::new();
            replacement.insert("version".to_string(), Value::String(version));
            table.insert(key, Value::Table(replacement));
        }
    }

    let serialized = toml::to_string_pretty(&parsed)
        .map_err(|e| anyhow::anyhow!("failed to serialize Cargo.docker.toml: {e}"))?;
    let out_path = project_root.join("Cargo.docker.toml");
    fs::write(&out_path, serialized)
        .map_err(|e| anyhow::anyhow!("failed to write {}: {e}", out_path.display()))?;
    Ok(out_path)
}

fn read_path_dep_version(project_root: &Path, rel_path: &str) -> Option<String> {
    let dep_cargo = project_root.join(rel_path).join("Cargo.toml");
    let content = fs::read_to_string(&dep_cargo).ok()?;
    let parsed: Value = content.parse().ok()?;
    parsed
        .get("package")?
        .get("version")?
        .as_str()
        .map(String::from)
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
        assert!(body.contains("[dependencies.ferro]") || body.contains("ferro = "));
        assert!(body.contains("0.1.87"));
        assert!(!body.contains("path"));
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
        assert!(!body.contains("path"));
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
        // Three ferro* deps rewritten, each carrying the version.
        assert_eq!(body.matches("0.1.87").count(), 3);
        assert!(!body.contains("path"));
        // serde untouched.
        assert!(body.contains("serde"));
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
}
