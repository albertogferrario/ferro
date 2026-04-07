//! Generated artifacts check (D-08): warn (never error) when Dockerfile,
//! .dockerignore, or .do/app.yaml are missing.

use crate::doctor::check::{CheckResult, DoctorCheck};
use std::path::Path;

pub struct ArtifactsCheck;

const NAME: &str = "artifacts";

impl DoctorCheck for ArtifactsCheck {
    fn name(&self) -> &'static str {
        NAME
    }
    fn run(&self, root: &Path) -> CheckResult {
        check_impl(root)
    }
}

pub(crate) fn check_impl(_root: &Path) -> CheckResult {
    CheckResult::ok(NAME, "stub")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_artifacts() {
        assert_eq!(ArtifactsCheck.name(), "artifacts");
    }
}
