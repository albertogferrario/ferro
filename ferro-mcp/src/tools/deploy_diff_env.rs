//! deploy_diff_env — compare local env source against .do/app.yaml envs block.
//!
//! Read-only (D-11). 3-column table output per D-06 (key, local, remote),
//! plus classification flagging scope mismatches (is_secret keys marked PLAIN).

use crate::error::{McpError, Result};
use crate::tools::deploy_common::{is_secret, parse_env_example, EnvEntry};
use regex::Regex;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    Aligned,
    MissingLocal,
    MissingRemote,
    ScopeMismatch,
}

#[derive(Debug, Serialize)]
pub struct DiffRow {
    pub key: String,
    pub local: Option<String>,
    pub remote: Option<String>,
    pub classification: Classification,
}

#[derive(Debug, Serialize)]
pub struct DiffEnvReport {
    /// "env" when `.env` was used, "env_example" when the `.env.example` fallback was used.
    pub source: String,
    pub rows: Vec<DiffRow>,
    pub drift_count: usize,
    pub secrets_marked_plain: Vec<String>,
}

/// (source_label, parsed entries) or None if neither `.env` nor `.env.example` exists.
fn load_local_env(root: &Path) -> Option<(String, Vec<EnvEntry>)> {
    let env = root.join(".env");
    if env.exists() {
        if let Ok(content) = fs::read_to_string(&env) {
            return Some(("env".to_string(), parse_env_example(&content)));
        }
    }
    let example = root.join(".env.example");
    if example.exists() {
        if let Ok(content) = fs::read_to_string(&example) {
            return Some(("env_example".to_string(), parse_env_example(&content)));
        }
    }
    None
}

/// Parsed app.yaml env entry: (key, value, explicit type/scope label).
/// `scope` is `Some("SECRET")` when `type: SECRET` is present; otherwise None.
fn parse_app_yaml_envs(content: &str) -> Vec<(String, String, Option<String>)> {
    // Split on `- key:` boundaries, then walk line-by-line inside each block.
    let line_re = Regex::new(r"^\s*(key|value|type):\s*(.+)\s*$").unwrap();
    let mut out: Vec<(String, String, Option<String>)> = Vec::new();

    let mut current: Option<(Option<String>, Option<String>, Option<String>)> = None;
    let flush = |cur: &mut Option<(Option<String>, Option<String>, Option<String>)>,
                 out: &mut Vec<(String, String, Option<String>)>| {
        if let Some((Some(k), v, t)) = cur.take() {
            out.push((k, v.unwrap_or_default(), t));
        } else {
            *cur = None;
        }
    };

    for raw in content.lines() {
        let trimmed = raw.trim_start();
        if let Some(rest) = trimmed.strip_prefix("- key:") {
            // New block — flush previous.
            flush(&mut current, &mut out);
            let key = rest.trim().trim_matches('"').trim_matches('\'').to_string();
            current = Some((Some(key), None, None));
            continue;
        }

        if let Some(ref mut block) = current {
            if let Some(caps) = line_re.captures(raw) {
                let field = caps.get(1).unwrap().as_str();
                let value = caps
                    .get(2)
                    .unwrap()
                    .as_str()
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                match field {
                    "key" => block.0 = Some(value),
                    "value" => block.1 = Some(value),
                    "type" => block.2 = Some(value),
                    _ => {}
                }
            }
        }
    }
    flush(&mut current, &mut out);
    out
}

pub fn execute(project_root: &Path) -> Result<DiffEnvReport> {
    let (source, local_entries) = load_local_env(project_root).ok_or_else(|| {
        McpError::ToolError(
            "deploy_diff_env: no local env source (.env or .env.example)".to_string(),
        )
    })?;

    let app_yaml_path = project_root.join(".do/app.yaml");
    if !app_yaml_path.exists() {
        return Err(McpError::ToolError(
            "deploy_diff_env: .do/app.yaml not found".to_string(),
        ));
    }
    let yaml = fs::read_to_string(&app_yaml_path)
        .map_err(|e| McpError::ToolError(format!("deploy_diff_env: read .do/app.yaml: {e}")))?;
    let remote_entries = parse_app_yaml_envs(&yaml);

    let mut map: BTreeMap<String, DiffRow> = BTreeMap::new();

    for EnvEntry { key, value } in &local_entries {
        map.insert(
            key.clone(),
            DiffRow {
                key: key.clone(),
                local: Some(value.clone()),
                remote: None,
                classification: Classification::MissingRemote,
            },
        );
    }

    let mut secrets_marked_plain: Vec<String> = Vec::new();

    for (key, value, scope) in &remote_entries {
        let entry = map.entry(key.clone()).or_insert_with(|| DiffRow {
            key: key.clone(),
            local: None,
            remote: None,
            classification: Classification::MissingLocal,
        });
        entry.remote = Some(value.clone());

        // Re-classify given now we have both (or still only remote) sides.
        let is_secret_key = is_secret(key);
        let marked_secret = scope.as_deref() == Some("SECRET");

        if is_secret_key && !marked_secret {
            secrets_marked_plain.push(key.clone());
        }

        entry.classification = match (&entry.local, &entry.remote) {
            (Some(_), Some(_)) => {
                if is_secret_key && !marked_secret {
                    Classification::ScopeMismatch
                } else {
                    Classification::Aligned
                }
            }
            (None, Some(_)) => Classification::MissingLocal,
            (Some(_), None) => Classification::MissingRemote,
            (None, None) => Classification::MissingLocal,
        };
    }

    let rows: Vec<DiffRow> = map.into_values().collect();
    let drift_count = rows
        .iter()
        .filter(|r| r.classification != Classification::Aligned)
        .count();

    secrets_marked_plain.sort();
    secrets_marked_plain.dedup();

    Ok(DiffEnvReport {
        source,
        rows,
        drift_count,
        secrets_marked_plain,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_app_yaml(dir: &Path, body: &str) {
        fs::create_dir_all(dir.join(".do")).unwrap();
        fs::write(dir.join(".do/app.yaml"), body).unwrap();
    }

    #[test]
    fn test_drift_both_sides() {
        let td = TempDir::new().unwrap();
        fs::write(
            td.path().join(".env"),
            "APP_URL=http://local\nDATABASE_URL=postgres://x\n",
        )
        .unwrap();
        write_app_yaml(
            td.path(),
            "services:\n  - name: web\n    envs:\n      - key: APP_URL\n        value: \"http://prod\"\n      - key: FOO\n        value: bar\n",
        );

        let report = execute(td.path()).unwrap();
        assert_eq!(report.source, "env");
        let keys: Vec<&str> = report.rows.iter().map(|r| r.key.as_str()).collect();
        assert_eq!(keys, vec!["APP_URL", "DATABASE_URL", "FOO"]);

        let app_url = report.rows.iter().find(|r| r.key == "APP_URL").unwrap();
        assert_eq!(app_url.local.as_deref(), Some("http://local"));
        assert_eq!(app_url.remote.as_deref(), Some("http://prod"));
        // APP_URL is not secret -> Aligned.
        assert_eq!(app_url.classification, Classification::Aligned);

        let db = report
            .rows
            .iter()
            .find(|r| r.key == "DATABASE_URL")
            .unwrap();
        assert_eq!(db.classification, Classification::MissingRemote);

        let foo = report.rows.iter().find(|r| r.key == "FOO").unwrap();
        assert_eq!(foo.classification, Classification::MissingLocal);

        assert_eq!(report.drift_count, 2);
    }

    #[test]
    fn test_secret_marked_plain_flagged() {
        let td = TempDir::new().unwrap();
        fs::write(td.path().join(".env"), "DATABASE_URL=postgres://x\n").unwrap();
        write_app_yaml(
            td.path(),
            "services:\n  - name: web\n    envs:\n      - key: DATABASE_URL\n        value: postgres://prod\n        scope: RUN_AND_BUILD_TIME\n",
        );

        let report = execute(td.path()).unwrap();
        assert_eq!(
            report.secrets_marked_plain,
            vec!["DATABASE_URL".to_string()]
        );
        let db = report
            .rows
            .iter()
            .find(|r| r.key == "DATABASE_URL")
            .unwrap();
        assert_eq!(db.classification, Classification::ScopeMismatch);
        assert_eq!(report.drift_count, 1);
    }

    #[test]
    fn test_aligned_secret_with_type_secret() {
        let td = TempDir::new().unwrap();
        fs::write(td.path().join(".env"), "DATABASE_URL=postgres://x\n").unwrap();
        write_app_yaml(
            td.path(),
            "services:\n  - name: web\n    envs:\n      - key: DATABASE_URL\n        value: postgres://prod\n        scope: RUN_AND_BUILD_TIME\n        type: SECRET\n",
        );

        let report = execute(td.path()).unwrap();
        assert!(report.secrets_marked_plain.is_empty());
        let db = report
            .rows
            .iter()
            .find(|r| r.key == "DATABASE_URL")
            .unwrap();
        assert_eq!(db.classification, Classification::Aligned);
        assert_eq!(report.drift_count, 0);
    }

    #[test]
    fn test_env_example_fallback() {
        let td = TempDir::new().unwrap();
        fs::write(td.path().join(".env.example"), "FOO=bar\n").unwrap();
        write_app_yaml(
            td.path(),
            "services:\n  - name: web\n    envs:\n      - key: FOO\n        value: bar\n",
        );
        let report = execute(td.path()).unwrap();
        assert_eq!(report.source, "env_example");
        assert_eq!(report.drift_count, 0);
    }

    #[test]
    fn test_no_local_env_source_errors() {
        let td = TempDir::new().unwrap();
        write_app_yaml(td.path(), "services: []\n");
        let err = execute(td.path()).unwrap_err();
        assert!(err.to_string().contains("no local env source"));
    }

    #[test]
    fn test_missing_app_yaml_errors() {
        let td = TempDir::new().unwrap();
        fs::write(td.path().join(".env"), "FOO=bar\n").unwrap();
        let err = execute(td.path()).unwrap_err();
        assert!(err.to_string().contains("app.yaml"));
    }

    #[test]
    fn test_rows_sorted_alphabetically() {
        let td = TempDir::new().unwrap();
        fs::write(td.path().join(".env"), "ZETA=1\nALPHA=2\nMIKE=3\n").unwrap();
        write_app_yaml(td.path(), "services:\n  - name: web\n    envs: []\n");
        let report = execute(td.path()).unwrap();
        let keys: Vec<&str> = report.rows.iter().map(|r| r.key.as_str()).collect();
        assert_eq!(keys, vec!["ALPHA", "MIKE", "ZETA"]);
    }
}
