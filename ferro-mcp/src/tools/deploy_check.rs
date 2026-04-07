//! deploy_check MCP tool — pre-flight deploy validation.
//!
//! Read-only (D-11). Returns a structured severity-tagged report per D-01/D-02/D-03.
//! Detects missing Dockerfile/.do/app.yaml, ferro path deps in Cargo.toml,
//! sqlite DATABASE_URL, env drift between `.env.example` and `.do/app.yaml`,
//! dirty git tree, and unpushed commits on the current branch.

use crate::error::Result;
use crate::tools::deploy_common::{find_ferro_path_deps, parse_env_example};
use serde::Serialize;
use serde_json::json;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Blocker,
    Warning,
    Info,
}

#[derive(Debug, Serialize)]
pub struct Finding {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub detail: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct DeployCheckReport {
    pub status: String,
    pub findings: Vec<Finding>,
    pub checked: CheckedFiles,
}

#[derive(Debug, Serialize, Default)]
pub struct CheckedFiles {
    pub dockerfile: bool,
    pub app_yaml: bool,
    pub env_example: bool,
    pub cargo_toml: bool,
}

pub fn execute(project_root: &Path) -> Result<DeployCheckReport> {
    let mut findings: Vec<Finding> = Vec::new();
    let mut checked = CheckedFiles::default();

    // check_dockerfile
    let dockerfile_path = project_root.join("Dockerfile");
    if dockerfile_path.exists() {
        checked.dockerfile = true;
    } else {
        findings.push(Finding {
            code: "missing_dockerfile".into(),
            severity: Severity::Blocker,
            message: "Dockerfile not found at project root".into(),
            detail: None,
        });
    }

    // check_app_yaml
    let app_yaml_path = project_root.join(".do/app.yaml");
    let app_yaml_content: Option<String> = if app_yaml_path.exists() {
        checked.app_yaml = true;
        fs::read_to_string(&app_yaml_path).ok()
    } else {
        findings.push(Finding {
            code: "missing_app_yaml".into(),
            severity: Severity::Blocker,
            message: ".do/app.yaml not found".into(),
            detail: None,
        });
        None
    };

    // check_ferro_path_deps
    let cargo_toml_path = project_root.join("Cargo.toml");
    if cargo_toml_path.exists() {
        checked.cargo_toml = true;
        if let Ok(content) = fs::read_to_string(&cargo_toml_path) {
            let path_deps = find_ferro_path_deps(&content);
            if !path_deps.is_empty() {
                findings.push(Finding {
                    code: "ferro_path_deps".into(),
                    severity: Severity::Blocker,
                    message:
                        "Cargo.toml has ferro path dependencies; remote deploy cannot fetch them"
                            .into(),
                    detail: Some(json!({ "crates": path_deps })),
                });
            }
        }
    }

    // check_sqlite_database_url + gather env_example keys
    let env_example_path = project_root.join(".env.example");
    let mut env_example_keys: BTreeSet<String> = BTreeSet::new();
    if env_example_path.exists() {
        checked.env_example = true;
        if let Ok(content) = fs::read_to_string(&env_example_path) {
            let entries = parse_env_example(&content);
            for entry in &entries {
                env_example_keys.insert(entry.key.clone());
                if entry.key == "DATABASE_URL" {
                    let v = entry.value.trim().trim_matches('"').trim_matches('\'');
                    if v.starts_with("sqlite:") {
                        findings.push(Finding {
                            code: "sqlite_database_url".into(),
                            severity: Severity::Blocker,
                            message: "DATABASE_URL points to sqlite; not supported in production"
                                .into(),
                            detail: Some(json!({ "value": v })),
                        });
                    }
                }
            }
        }
    }

    // check_env_drift
    if let Some(ref yaml) = app_yaml_content {
        if !env_example_keys.is_empty() {
            let app_yaml_keys = parse_app_yaml_env_keys(yaml);
            for key in env_example_keys.difference(&app_yaml_keys) {
                findings.push(Finding {
                    code: "missing_env_var".into(),
                    severity: Severity::Warning,
                    message: format!(
                        "Key `{key}` present in .env.example but missing from .do/app.yaml"
                    ),
                    detail: Some(json!({ "key": key })),
                });
            }
            for key in app_yaml_keys.difference(&env_example_keys) {
                findings.push(Finding {
                    code: "extra_env_var".into(),
                    severity: Severity::Info,
                    message: format!(
                        "Key `{key}` present in .do/app.yaml but missing from .env.example"
                    ),
                    detail: Some(json!({ "key": key })),
                });
            }
        }
    }

    // check_git_state (best-effort)
    check_git_state(project_root, &mut findings);

    let has_blocker = findings.iter().any(|f| f.severity == Severity::Blocker);
    let has_warning = findings.iter().any(|f| f.severity == Severity::Warning);
    let status = if has_blocker {
        "blocked"
    } else if has_warning {
        "warnings"
    } else {
        "ok"
    }
    .to_string();

    Ok(DeployCheckReport {
        status,
        findings,
        checked,
    })
}

fn parse_app_yaml_env_keys(yaml: &str) -> BTreeSet<String> {
    // Minimal regex-free parser: match lines like `- key: FOO` (any indentation).
    let mut out = BTreeSet::new();
    for line in yaml.lines() {
        let t = line.trim_start();
        let Some(rest) = t.strip_prefix("- key:") else {
            continue;
        };
        let key = rest.trim().trim_matches('"').trim_matches('\'').to_string();
        if !key.is_empty() {
            out.insert(key);
        }
    }
    out
}

fn check_git_state(project_root: &Path, findings: &mut Vec<Finding>) {
    // `git status --porcelain`
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(project_root)
        .output();
    let Ok(status) = status else {
        return;
    };
    if !status.status.success() {
        // Not a git repo or git missing — silently skip.
        return;
    }
    if !status.stdout.is_empty() {
        findings.push(Finding {
            code: "dirty_git_tree".into(),
            severity: Severity::Warning,
            message: "Working tree has uncommitted changes".into(),
            detail: None,
        });
    }

    // Upstream + unpushed commits
    let unpushed = Command::new("git")
        .args(["rev-list", "--count", "@{u}..HEAD"])
        .current_dir(project_root)
        .output();
    match unpushed {
        Ok(out) if out.status.success() => {
            let count = String::from_utf8_lossy(&out.stdout)
                .trim()
                .parse::<u64>()
                .unwrap_or(0);
            if count > 0 {
                findings.push(Finding {
                    code: "unpushed_commits".into(),
                    severity: Severity::Warning,
                    message: format!("{count} unpushed commit(s) on current branch"),
                    detail: Some(json!({ "count": count })),
                });
            }
        }
        _ => {
            findings.push(Finding {
                code: "no_upstream".into(),
                severity: Severity::Info,
                message: "Current branch has no upstream configured".into(),
                detail: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn codes(report: &DeployCheckReport) -> Vec<&str> {
        report.findings.iter().map(|f| f.code.as_str()).collect()
    }

    #[test]
    fn test_missing_files_flagged() {
        let td = TempDir::new().unwrap();
        let report = execute(td.path()).unwrap();
        let c = codes(&report);
        assert!(c.contains(&"missing_dockerfile"));
        assert!(c.contains(&"missing_app_yaml"));
        assert_eq!(report.status, "blocked");
        assert!(!report.checked.dockerfile);
        assert!(!report.checked.app_yaml);
    }

    #[test]
    fn test_ferro_path_dep_blocks() {
        let td = TempDir::new().unwrap();
        fs::write(
            td.path().join("Cargo.toml"),
            r#"
[package]
name = "x"
version = "0.1.0"

[dependencies]
ferro = { path = "../ferro" }
"#,
        )
        .unwrap();
        let report = execute(td.path()).unwrap();
        assert!(codes(&report).contains(&"ferro_path_deps"));
        assert_eq!(report.status, "blocked");
    }

    #[test]
    fn test_sqlite_database_url_blocks() {
        let td = TempDir::new().unwrap();
        fs::write(
            td.path().join(".env.example"),
            "DATABASE_URL=sqlite://db.sqlite\n",
        )
        .unwrap();
        let report = execute(td.path()).unwrap();
        assert!(codes(&report).contains(&"sqlite_database_url"));
        assert_eq!(report.status, "blocked");
    }

    #[test]
    fn test_env_drift_warning() {
        let td = TempDir::new().unwrap();
        fs::write(td.path().join("Dockerfile"), "FROM scratch\n").unwrap();
        fs::create_dir_all(td.path().join(".do")).unwrap();
        fs::write(
            td.path().join(".do/app.yaml"),
            "services:\n  - name: web\n    envs:\n      - key: BAR\n        value: x\n",
        )
        .unwrap();
        fs::write(td.path().join(".env.example"), "FOO=1\n").unwrap();
        let report = execute(td.path()).unwrap();
        let c = codes(&report);
        assert!(c.contains(&"missing_env_var"), "expected FOO missing");
        assert!(c.contains(&"extra_env_var"), "expected BAR extra");
        // Find detail for FOO
        let foo = report
            .findings
            .iter()
            .find(|f| f.code == "missing_env_var")
            .unwrap();
        assert_eq!(foo.detail.as_ref().unwrap()["key"], "FOO");
    }

    #[test]
    fn test_clean_project_ok() {
        let td = TempDir::new().unwrap();
        fs::write(td.path().join("Dockerfile"), "FROM scratch\n").unwrap();
        fs::create_dir_all(td.path().join(".do")).unwrap();
        fs::write(
            td.path().join(".do/app.yaml"),
            "services:\n  - name: web\n    envs:\n      - key: DATABASE_URL\n        value: x\n      - key: APP_ENV\n        value: production\n",
        )
        .unwrap();
        fs::write(
            td.path().join("Cargo.toml"),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n\n[dependencies]\nserde = \"1\"\n",
        )
        .unwrap();
        fs::write(
            td.path().join(".env.example"),
            "DATABASE_URL=postgres://user:pw@host/db\nAPP_ENV=production\n",
        )
        .unwrap();
        let report = execute(td.path()).unwrap();
        // git checks may add warnings/info depending on test env; filter to blockers/warnings
        // caused by our target detections.
        let target_codes = [
            "missing_dockerfile",
            "missing_app_yaml",
            "ferro_path_deps",
            "sqlite_database_url",
            "missing_env_var",
            "extra_env_var",
        ];
        for f in &report.findings {
            assert!(
                !target_codes.contains(&f.code.as_str()),
                "unexpected finding in clean project: {}",
                f.code
            );
        }
        assert!(report.checked.dockerfile);
        assert!(report.checked.app_yaml);
        assert!(report.checked.env_example);
        assert!(report.checked.cargo_toml);
    }
}
