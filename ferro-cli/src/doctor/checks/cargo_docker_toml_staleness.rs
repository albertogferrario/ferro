//! Cargo.docker.toml staleness (SCOPE §12.6): when present, every `ferro*`
//! version dep should still match the path-dep workspace version declared in
//! the live `Cargo.toml`.

use crate::doctor::check::{CheckResult, DoctorCheck};
use std::fs;
use std::path::Path;
use toml::Value;

pub struct CargoDockerTomlStalenessCheck;

const NAME: &str = "cargo_docker_toml_staleness";
const DEP_TABLES: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

impl DoctorCheck for CargoDockerTomlStalenessCheck {
    fn name(&self) -> &'static str {
        NAME
    }
    fn run(&self, root: &Path) -> CheckResult {
        check_impl(root)
    }
}

pub(crate) fn check_impl(root: &Path) -> CheckResult {
    let docker_toml = root.join("Cargo.docker.toml");
    if !docker_toml.is_file() {
        return CheckResult::ok(NAME, "skipped (Cargo.docker.toml absent)");
    }
    let cargo_toml = root.join("Cargo.toml");
    let docker_content = match fs::read_to_string(&docker_toml) {
        Ok(s) => s,
        Err(e) => {
            return CheckResult::error(NAME, format!("failed to read Cargo.docker.toml: {e}"))
        }
    };
    let cargo_content = match fs::read_to_string(&cargo_toml) {
        Ok(s) => s,
        Err(e) => return CheckResult::error(NAME, format!("failed to read Cargo.toml: {e}")),
    };

    let docker_parsed: Value = match docker_content.parse() {
        Ok(v) => v,
        Err(e) => {
            return CheckResult::error(NAME, format!("failed to parse Cargo.docker.toml: {e}"))
        }
    };
    let cargo_parsed: Value = match cargo_content.parse() {
        Ok(v) => v,
        Err(e) => return CheckResult::error(NAME, format!("failed to parse Cargo.toml: {e}")),
    };

    let mut drift: Vec<String> = Vec::new();
    for table_name in DEP_TABLES {
        let docker_table = docker_parsed.get(*table_name).and_then(|v| v.as_table());
        let cargo_table = cargo_parsed.get(*table_name).and_then(|v| v.as_table());
        let (Some(dt), Some(ct)) = (docker_table, cargo_table) else {
            continue;
        };
        for (key, value) in dt {
            if !key.starts_with("ferro") {
                continue;
            }
            let docker_version = value
                .as_table()
                .and_then(|t| t.get("version"))
                .and_then(|v| v.as_str())
                .or_else(|| value.as_str());
            let Some(docker_version) = docker_version else {
                continue;
            };
            // Skip the wildcard fallback — staleness is undefined.
            if docker_version == "*" {
                continue;
            }
            let cargo_path = ct
                .get(key)
                .and_then(|v| v.as_table())
                .and_then(|t| t.get("path"))
                .and_then(|p| p.as_str());
            let Some(rel_path) = cargo_path else {
                continue;
            };
            if let Some(workspace_version) = crate::deploy::read_path_dep_version(root, rel_path) {
                if workspace_version != docker_version {
                    drift.push(format!(
                        "{key}: Cargo.docker.toml={docker_version}, workspace={workspace_version}"
                    ));
                }
            }
        }
    }

    if drift.is_empty() {
        CheckResult::ok(NAME, "Cargo.docker.toml in sync with workspace versions")
    } else {
        CheckResult::warn(
            NAME,
            format!("{} ferro dep version drift detected", drift.len()),
        )
        .with_details(drift.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn name_is_cargo_docker_toml_staleness() {
        assert_eq!(
            CargoDockerTomlStalenessCheck.name(),
            "cargo_docker_toml_staleness"
        );
    }

    #[test]
    fn skipped_when_docker_toml_absent() {
        let tmp = TempDir::new().unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Ok);
        assert!(r.message.contains("skipped"));
    }

    #[test]
    fn ok_when_versions_match() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("project");
        write(
            &project.join("Cargo.toml"),
            "[package]\nname=\"x\"\nversion=\"0.1.0\"\n[dependencies]\nferro={path=\"../framework\"}\n",
        );
        write(
            &project.join("Cargo.docker.toml"),
            "[package]\nname=\"x\"\nversion=\"0.1.0\"\n[dependencies]\nferro={version=\"0.1.87\"}\n",
        );
        write(
            &tmp.path().join("framework/Cargo.toml"),
            "[package]\nname=\"ferro\"\nversion=\"0.1.87\"\n",
        );
        let r = check_impl(&project);
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Ok);
    }

    #[test]
    fn warn_on_drift() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("project");
        write(
            &project.join("Cargo.toml"),
            "[package]\nname=\"x\"\nversion=\"0.1.0\"\n[dependencies]\nferro={path=\"../framework\"}\n",
        );
        write(
            &project.join("Cargo.docker.toml"),
            "[package]\nname=\"x\"\nversion=\"0.1.0\"\n[dependencies]\nferro={version=\"0.1.50\"}\n",
        );
        write(
            &tmp.path().join("framework/Cargo.toml"),
            "[package]\nname=\"ferro\"\nversion=\"0.1.87\"\n",
        );
        let r = check_impl(&project);
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Warn);
        assert!(r.details.unwrap().contains("ferro"));
    }
}
