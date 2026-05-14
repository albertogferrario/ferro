//! Doctor check: ensures projects with migrations have a PRE_DEPLOY migrate
//! job configured in `.do/app.yaml`. Prevents the failure mode where a deploy
//! starts with a stale schema because the runtime migration runner swallowed
//! the error.

use std::fs;
use std::path::Path;

use crate::doctor::check::{CheckCategory, CheckResult, DoctorCheck};

const NAME: &str = "migrate_gate";

/// Migrate-gate check: errors if migrations exist but no PRE_DEPLOY migrate
/// job is configured in `.do/app.yaml`.
pub struct MigrateGateCheck;

impl DoctorCheck for MigrateGateCheck {
    fn name(&self) -> &'static str {
        NAME
    }

    fn category(&self) -> CheckCategory {
        CheckCategory::Deploy
    }

    fn run(&self, root: &Path) -> CheckResult {
        check_impl(root)
    }
}

pub(crate) fn check_impl(root: &Path) -> CheckResult {
    let has_migrations = root.join("migrations").is_dir() || root.join("src/migrations").is_dir();
    if !has_migrations {
        return CheckResult::ok(NAME, "no migrations directory — skipped");
    }

    let app_yaml = root.join(".do/app.yaml");
    if !app_yaml.exists() {
        return CheckResult::ok(NAME, "skipped — not a DO deploy project");
    }

    let yaml = match fs::read_to_string(&app_yaml) {
        Ok(s) => s,
        Err(e) => {
            return CheckResult::error(NAME, format!("failed to read .do/app.yaml: {e}"));
        }
    };

    if has_predeploy_migrate_job(&yaml) {
        CheckResult::ok(NAME, "PRE_DEPLOY migrate job present")
    } else {
        CheckResult::error(NAME, "no PRE_DEPLOY migrate job in .do/app.yaml")
            .with_details("Run `ferro do:init --force` to scaffold a migrate job, then commit.")
    }
}

/// Line-scan for a `jobs:` section containing an entry with both
/// `kind: PRE_DEPLOY` and a `run_command` referencing `db:migrate`.
/// Implemented as a line scan (no `serde_yaml` dep) — only presence is
/// detected, no schema validation.
fn has_predeploy_migrate_job(yaml: &str) -> bool {
    let mut in_jobs = false;
    let mut current_job_predeploy = false;
    let mut current_job_migrate = false;
    for raw in yaml.lines() {
        let line = raw.trim_end();
        let trimmed = line.trim_start();

        // jobs: at top level
        if trimmed == "jobs:" {
            in_jobs = true;
            continue;
        }
        if !in_jobs {
            continue;
        }
        // Exit jobs: section when we hit another top-level key
        // (a non-indented line that is not blank and not the jobs: header).
        if !line.is_empty() && !line.starts_with(' ') && !line.starts_with('-') {
            // top-level key encountered — flush current job
            if current_job_predeploy && current_job_migrate {
                return true;
            }
            in_jobs = false;
            continue;
        }
        // New job entry starts with "- " (possibly indented)
        if trimmed.starts_with("- ") || trimmed == "-" {
            if current_job_predeploy && current_job_migrate {
                return true;
            }
            current_job_predeploy = false;
            current_job_migrate = false;
        }
        if trimmed.contains("kind: PRE_DEPLOY") {
            current_job_predeploy = true;
        }
        if trimmed.contains("db:migrate") {
            current_job_migrate = true;
        }
    }
    current_job_predeploy && current_job_migrate
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::check::CheckStatus;
    use std::fs;
    use tempfile::TempDir;

    fn write(root: &Path, rel: &str, content: &str) {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    }

    #[test]
    fn skips_when_no_migrations_directory() {
        let tmp = TempDir::new().unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, CheckStatus::Ok);
        assert!(r.message.contains("skipped"));
    }

    #[test]
    fn skips_when_no_app_yaml() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("migrations")).unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, CheckStatus::Ok);
        assert!(r.message.contains("skipped"));
    }

    #[test]
    fn errors_when_app_yaml_has_no_jobs() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("migrations")).unwrap();
        write(
            tmp.path(),
            ".do/app.yaml",
            "name: foo\nservices:\n  - name: web\n",
        );
        let r = check_impl(tmp.path());
        assert_eq!(r.status, CheckStatus::Error);
        assert_eq!(r.name, "migrate_gate");
        assert!(r.message.contains("no PRE_DEPLOY migrate job"));
    }

    #[test]
    fn errors_when_jobs_block_has_no_predeploy() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("migrations")).unwrap();
        write(
            tmp.path(),
            ".do/app.yaml",
            "jobs:\n  - name: foo\n    kind: POST_DEPLOY\n    run_command: /usr/local/bin/app db:migrate\n",
        );
        let r = check_impl(tmp.path());
        assert_eq!(r.status, CheckStatus::Error);
    }

    #[test]
    fn errors_when_predeploy_present_but_no_migrate_command() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("migrations")).unwrap();
        write(
            tmp.path(),
            ".do/app.yaml",
            "jobs:\n  - name: foo\n    kind: PRE_DEPLOY\n    run_command: /usr/local/bin/app seed\n",
        );
        let r = check_impl(tmp.path());
        assert_eq!(r.status, CheckStatus::Error);
    }

    #[test]
    fn ok_when_predeploy_migrate_job_present() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("migrations")).unwrap();
        write(
            tmp.path(),
            ".do/app.yaml",
            "jobs:\n  - name: migrate\n    kind: PRE_DEPLOY\n    run_command: /usr/local/bin/app db:migrate\n",
        );
        let r = check_impl(tmp.path());
        assert_eq!(r.status, CheckStatus::Ok);
        assert!(r.message.contains("PRE_DEPLOY migrate job present"));
    }

    #[test]
    fn ok_when_migrations_under_src_migrations() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("src/migrations")).unwrap();
        // No app.yaml — still skip
        let r = check_impl(tmp.path());
        assert_eq!(r.status, CheckStatus::Ok);
        assert!(r.message.contains("skipped"));
    }

    #[test]
    fn scanner_detects_pre_deploy_migrate_across_lines() {
        let yaml = r#"
jobs:
  - name: migrate
    kind: PRE_DEPLOY
    run_command: /usr/local/bin/app db:migrate
"#;
        assert!(has_predeploy_migrate_job(yaml));
    }

    #[test]
    fn scanner_rejects_when_kind_and_command_in_different_jobs() {
        let yaml = r#"
jobs:
  - name: first
    kind: PRE_DEPLOY
    run_command: /usr/local/bin/app seed
  - name: second
    kind: POST_DEPLOY
    run_command: /usr/local/bin/app db:migrate
"#;
        assert!(!has_predeploy_migrate_job(yaml));
    }
}
