//! Generated artifacts (SCOPE §12.7): warn-level presence check for
//! Dockerfile, .dockerignore, .do/app.yaml.

use crate::doctor::check::{CheckResult, DoctorCheck};
use std::path::Path;

pub struct GeneratedArtifactsCheck;

const NAME: &str = "generated_artifacts";
const ARTIFACTS: &[&str] = &["Dockerfile", ".dockerignore", ".do/app.yaml"];

impl DoctorCheck for GeneratedArtifactsCheck {
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
    fn name_is_generated_artifacts() {
        assert_eq!(GeneratedArtifactsCheck.name(), "generated_artifacts");
    }

    #[test]
    fn all_present_returns_ok() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("Dockerfile"), "").unwrap();
        fs::write(tmp.path().join(".dockerignore"), "").unwrap();
        fs::create_dir(tmp.path().join(".do")).unwrap();
        fs::write(tmp.path().join(".do/app.yaml"), "").unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Ok);
    }

    #[test]
    fn missing_warns_never_errors() {
        let tmp = TempDir::new().unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Warn);
    }
}
