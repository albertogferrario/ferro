//! Doctor framework: trait, status enum, result/report types, exit code logic.
//!
//! All concrete checks live in `super::checks`. The registry composes them
//! in declared order (D-01).

use serde::Serialize;
use std::path::Path;

/// Filter bucket for `ferro doctor --deploy` and MCP `deploy_check` (Phase 128 D-02).
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CheckCategory {
    General,
    Deploy,
}

/// Status of a single doctor check (D-09).
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Ok,
    Warn,
    Error,
}

/// Result of running a single doctor check.
#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub name: &'static str,
    pub status: CheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl CheckResult {
    pub fn ok(name: &'static str, message: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Ok,
            message: message.into(),
            details: None,
        }
    }

    pub fn warn(name: &'static str, message: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Warn,
            message: message.into(),
            details: None,
        }
    }

    pub fn error(name: &'static str, message: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Error,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

/// Trait every concrete check implements.
pub trait DoctorCheck {
    fn name(&self) -> &'static str;
    fn run(&self, root: &Path) -> CheckResult;
    /// Filter bucket for `ferro doctor --deploy` and MCP `deploy_check`
    /// (Phase 128 D-02). Defaults to `General`.
    fn category(&self) -> CheckCategory {
        CheckCategory::General
    }
}

/// Aggregate report — emitted as human or JSON output.
#[derive(Debug, Serialize)]
pub struct Report {
    pub summary: ReportSummary,
    pub checks: Vec<CheckResult>,
}

#[derive(Debug, Serialize)]
pub struct ReportSummary {
    pub overall: CheckStatus,
    pub ok: usize,
    pub warn: usize,
    pub error: usize,
}

impl Report {
    pub fn build(checks: Vec<CheckResult>) -> Self {
        let mut ok = 0usize;
        let mut warn = 0usize;
        let mut error = 0usize;
        for c in &checks {
            match c.status {
                CheckStatus::Ok => ok += 1,
                CheckStatus::Warn => warn += 1,
                CheckStatus::Error => error += 1,
            }
        }
        let overall = if error > 0 {
            CheckStatus::Error
        } else if warn > 0 {
            CheckStatus::Warn
        } else {
            CheckStatus::Ok
        };
        Self {
            summary: ReportSummary {
                overall,
                ok,
                warn,
                error,
            },
            checks,
        }
    }

    /// D-09: non-zero iff any check returned `error`. Warnings do not block.
    pub fn exit_code(&self) -> i32 {
        match self.summary.overall {
            CheckStatus::Error => 1,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_status_serializes_as_lowercase() {
        assert_eq!(serde_json::to_string(&CheckStatus::Ok).unwrap(), "\"ok\"");
        assert_eq!(
            serde_json::to_string(&CheckStatus::Warn).unwrap(),
            "\"warn\""
        );
        assert_eq!(
            serde_json::to_string(&CheckStatus::Error).unwrap(),
            "\"error\""
        );
    }

    #[test]
    fn report_summary_overall_is_error_when_any_error() {
        let report = Report::build(vec![
            CheckResult::ok("a", "fine"),
            CheckResult::warn("b", "meh"),
            CheckResult::error("c", "boom"),
        ]);
        assert_eq!(report.summary.overall, CheckStatus::Error);
        assert_eq!(report.summary.ok, 1);
        assert_eq!(report.summary.warn, 1);
        assert_eq!(report.summary.error, 1);
    }

    #[test]
    fn report_summary_overall_is_warn_when_only_warns() {
        let report = Report::build(vec![
            CheckResult::ok("a", "fine"),
            CheckResult::warn("b", "meh"),
        ]);
        assert_eq!(report.summary.overall, CheckStatus::Warn);
    }

    #[test]
    fn report_summary_overall_is_ok_when_all_ok() {
        let report = Report::build(vec![CheckResult::ok("a", "fine")]);
        assert_eq!(report.summary.overall, CheckStatus::Ok);
    }

    #[test]
    fn exit_code_is_zero_for_ok_and_warn() {
        let ok_report = Report::build(vec![CheckResult::ok("a", "fine")]);
        assert_eq!(ok_report.exit_code(), 0);

        let warn_report = Report::build(vec![CheckResult::warn("a", "meh")]);
        assert_eq!(warn_report.exit_code(), 0);
    }

    #[test]
    fn exit_code_is_one_for_any_error() {
        let report = Report::build(vec![
            CheckResult::ok("a", "fine"),
            CheckResult::error("b", "boom"),
        ]);
        assert_eq!(report.exit_code(), 1);
    }

    #[test]
    fn report_serializes_with_stable_shape() {
        let report = Report::build(vec![
            CheckResult::ok("toolchain", "rustc 1.88"),
            CheckResult::warn("artifacts", "missing files").with_details("Dockerfile"),
        ]);
        let json = serde_json::to_value(&report).unwrap();
        assert!(json.get("summary").is_some());
        assert!(json.get("checks").is_some());
        let checks = json.get("checks").unwrap().as_array().unwrap();
        assert_eq!(checks.len(), 2);
        assert_eq!(checks[0].get("name").unwrap(), "toolchain");
        assert_eq!(checks[0].get("status").unwrap(), "ok");
        assert!(checks[0].get("details").is_none());
        assert_eq!(checks[1].get("details").unwrap(), "Dockerfile");
    }

    #[test]
    fn details_omitted_when_none() {
        let result = CheckResult::ok("x", "fine");
        let s = serde_json::to_string(&result).unwrap();
        assert!(!s.contains("details"));
    }

    #[test]
    fn non_deploy_checks_return_general_category() {
        use crate::doctor::registry::default_checks;
        let general_names = &[
            "toolchain_match",
            "db_connection",
            "migrations_pending",
            "local_env_parity",
            "deploy_env_parity",
            "generated_artifacts",
            "database_url_sqlite_in_prod",
            "git_clean_and_pushed",
            "frontend_types_convention",
        ];
        let deploy_names = &["copy_dirs_dockerignore_collision", "docker_template_drift"];
        for check in default_checks() {
            if general_names.contains(&check.name()) {
                assert_eq!(
                    check.category(),
                    CheckCategory::General,
                    "{} should be General",
                    check.name()
                );
            }
            if deploy_names.contains(&check.name()) {
                assert_eq!(
                    check.category(),
                    CheckCategory::Deploy,
                    "{} should be Deploy",
                    check.name()
                );
            }
        }
    }
}
