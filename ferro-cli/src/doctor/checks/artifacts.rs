//! Generated artifacts check (D-08): warn (NEVER error) when Dockerfile,
//! .dockerignore, or .do/app.yaml are missing.

use crate::doctor::check::{CheckResult, CheckStatus, DoctorCheck};
use std::path::Path;

pub struct ArtifactsCheck;

const NAME: &str = "artifacts";

const ARTIFACTS: &[&str] = &["Dockerfile", ".dockerignore", ".do/app.yaml"];

impl DoctorCheck for ArtifactsCheck {
    fn name(&self) -> &'static str {
        NAME
    }
    fn run(&self, root: &Path) -> CheckResult {
        check_impl(root)
    }
}

pub(crate) fn check_impl(root: &Path) -> CheckResult {
    let missing: Vec<&str> = ARTIFACTS
        .iter()
        .filter(|f| !root.join(f).exists())
        .copied()
        .collect();

    if missing.is_empty() {
        CheckResult::ok(NAME, "Dockerfile, .dockerignore, .do/app.yaml present")
    } else {
        // D-08: warn, never error.
        let _force_warn = CheckStatus::Warn;
        CheckResult::warn(NAME, format!("{} artifact(s) missing", missing.len()))
            .with_details(format!("missing: {}", missing.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn name_is_artifacts() {
        assert_eq!(ArtifactsCheck.name(), "artifacts");
    }

    #[test]
    fn all_present_returns_ok() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("Dockerfile"), "").unwrap();
        fs::write(tmp.path().join(".dockerignore"), "").unwrap();
        fs::create_dir(tmp.path().join(".do")).unwrap();
        fs::write(tmp.path().join(".do/app.yaml"), "").unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, CheckStatus::Ok);
    }

    #[test]
    fn one_missing_warns_never_errors() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("Dockerfile"), "").unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, CheckStatus::Warn);
        assert_ne!(r.status, CheckStatus::Error);
        let details = r.details.unwrap();
        assert!(details.contains(".dockerignore"));
        assert!(details.contains(".do/app.yaml"));
    }

    #[test]
    fn all_missing_warns_never_errors() {
        let tmp = TempDir::new().unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, CheckStatus::Warn);
    }
}
