//! Deploy preflight (Phase 128 D-05): detect version drift between
//! local `ferro*` path deps and what `Cargo.docker.toml` rewrites to.
//! Covers REPORT items 4 and 13.

use crate::deploy::read_path_dep_version;
use crate::doctor::check::{CheckCategory, CheckResult, DoctorCheck};
use std::fs;
use std::path::Path;
use toml::Value;

pub struct FerroVersionSkewCheck;

const NAME: &str = "ferro_version_skew";
const DEP_TABLES: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

impl DoctorCheck for FerroVersionSkewCheck {
    fn name(&self) -> &'static str {
        NAME
    }
    fn run(&self, root: &Path) -> CheckResult {
        check_impl(root)
    }
    fn category(&self) -> CheckCategory {
        CheckCategory::Deploy
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DriftKind {
    None,
    Patch,
    MajorMinor,
}

fn classify(local: &str, docker: &str) -> Option<DriftKind> {
    if local == docker {
        return Some(DriftKind::None);
    }
    let lp: Vec<&str> = local.split('.').collect();
    let dp: Vec<&str> = docker.split('.').collect();
    if lp.len() < 2 || dp.len() < 2 {
        return None;
    }
    let lmaj: u32 = lp[0].parse().ok()?;
    let lmin: u32 = lp[1].parse().ok()?;
    let dmaj: u32 = dp[0].parse().ok()?;
    let dmin: u32 = dp[1].parse().ok()?;
    if lmaj != dmaj || lmin != dmin {
        Some(DriftKind::MajorMinor)
    } else {
        Some(DriftKind::Patch)
    }
}

pub(crate) fn check_impl(root: &Path) -> CheckResult {
    let docker_toml = root.join("Cargo.docker.toml");
    if !docker_toml.is_file() {
        return CheckResult::ok(NAME, "skipped (Cargo.docker.toml absent)");
    }
    let cargo_path = root.join("Cargo.toml");
    let cargo_src = match fs::read_to_string(&cargo_path) {
        Ok(s) => s,
        Err(e) => return CheckResult::error(NAME, format!("failed to read Cargo.toml: {e}")),
    };
    let docker_src = match fs::read_to_string(&docker_toml) {
        Ok(s) => s,
        Err(e) => {
            return CheckResult::error(NAME, format!("failed to read Cargo.docker.toml: {e}"))
        }
    };
    let cargo: Value = match cargo_src.parse() {
        Ok(v) => v,
        Err(e) => return CheckResult::error(NAME, format!("failed to parse Cargo.toml: {e}")),
    };
    let docker: Value = match docker_src.parse() {
        Ok(v) => v,
        Err(e) => {
            return CheckResult::error(NAME, format!("failed to parse Cargo.docker.toml: {e}"))
        }
    };

    let mut errors: Vec<String> = Vec::new();
    let mut warns: Vec<String> = Vec::new();
    let mut checked_any = false;

    for table_name in DEP_TABLES {
        let Some(ct) = cargo.get(*table_name).and_then(|v| v.as_table()) else {
            continue;
        };
        let Some(dt) = docker.get(*table_name).and_then(|v| v.as_table()) else {
            continue;
        };
        for (key, ct_value) in ct {
            if !key.starts_with("ferro") {
                continue;
            }
            let Some(rel_path) = ct_value
                .as_table()
                .and_then(|t| t.get("path"))
                .and_then(|p| p.as_str())
            else {
                continue;
            };
            let Some(local_version) = read_path_dep_version(root, rel_path) else {
                continue;
            };
            let docker_version = dt
                .get(key)
                .and_then(|v| {
                    v.as_table()
                        .and_then(|t| t.get("version"))
                        .and_then(|v| v.as_str())
                        .or_else(|| v.as_str())
                });
            let Some(docker_version) = docker_version else {
                continue;
            };
            if docker_version == "*" {
                continue;
            }
            checked_any = true;
            match classify(&local_version, docker_version) {
                Some(DriftKind::None) => {}
                Some(DriftKind::Patch) => warns.push(format!(
                    "{key}: local={local_version}, Cargo.docker.toml={docker_version} (patch drift)"
                )),
                Some(DriftKind::MajorMinor) => errors.push(format!(
                    "{key}: local={local_version}, Cargo.docker.toml={docker_version} (major/minor drift)"
                )),
                None => errors.push(format!(
                    "{key}: unparseable version (local={local_version}, docker={docker_version})"
                )),
            }
        }
    }

    if !checked_any {
        return CheckResult::ok(NAME, "no ferro path deps");
    }
    if !errors.is_empty() {
        return CheckResult::error(
            NAME,
            format!("{} ferro crate(s) with major/minor drift", errors.len()),
        )
        .with_details([errors, warns].concat().join("; "));
    }
    if !warns.is_empty() {
        return CheckResult::warn(
            NAME,
            format!("{} ferro crate(s) with patch drift", warns.len()),
        )
        .with_details(warns.join("; "));
    }
    CheckResult::ok(NAME, "ferro versions aligned")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::check::CheckStatus;
    use tempfile::TempDir;

    fn write(p: &Path, body: &str) {
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, body).unwrap();
    }

    fn scaffold(tmp: &Path, local: &str, docker: &str) {
        let project = tmp.join("project");
        write(
            &project.join("Cargo.toml"),
            "[package]\nname=\"x\"\nversion=\"0.1.0\"\n[dependencies]\nferro={path=\"../framework\"}\n",
        );
        write(
            &project.join("Cargo.docker.toml"),
            &format!(
                "[package]\nname=\"x\"\nversion=\"0.1.0\"\n[dependencies]\nferro={{version=\"{docker}\"}}\n"
            ),
        );
        write(
            &tmp.join("framework/Cargo.toml"),
            &format!("[package]\nname=\"ferro\"\nversion=\"{local}\"\n"),
        );
    }

    #[test]
    fn name_and_category() {
        assert_eq!(FerroVersionSkewCheck.name(), NAME);
        assert_eq!(FerroVersionSkewCheck.category(), CheckCategory::Deploy);
    }

    #[test]
    fn skipped_when_docker_toml_absent() {
        let td = TempDir::new().unwrap();
        write(
            &td.path().join("Cargo.toml"),
            "[package]\nname=\"x\"\nversion=\"0.1.0\"\n",
        );
        let r = check_impl(td.path());
        assert_eq!(r.status, CheckStatus::Ok);
        assert!(r.message.contains("skipped"));
    }

    #[test]
    fn error_on_major_minor_drift() {
        let td = TempDir::new().unwrap();
        scaffold(td.path(), "0.2.0", "0.1.0");
        let r = check_impl(&td.path().join("project"));
        assert_eq!(r.status, CheckStatus::Error);
        assert!(r.details.unwrap().contains("major/minor drift"));
    }

    #[test]
    fn warn_on_patch_drift() {
        let td = TempDir::new().unwrap();
        scaffold(td.path(), "0.2.0", "0.2.5");
        let r = check_impl(&td.path().join("project"));
        assert_eq!(r.status, CheckStatus::Warn);
        assert!(r.details.unwrap().contains("patch drift"));
    }

    #[test]
    fn ok_when_aligned() {
        let td = TempDir::new().unwrap();
        scaffold(td.path(), "0.2.0", "0.2.0");
        let r = check_impl(&td.path().join("project"));
        assert_eq!(r.status, CheckStatus::Ok);
        assert_eq!(r.message, "ferro versions aligned");
    }
}
