//! Workspace check (D-07): cargo-chef recipe target dirs exist.

use crate::doctor::check::{CheckResult, DoctorCheck};
use std::path::Path;

pub struct WorkspaceCheck;

const NAME: &str = "workspace";

impl DoctorCheck for WorkspaceCheck {
    fn name(&self) -> &'static str {
        NAME
    }
    fn run(&self, root: &Path) -> CheckResult {
        check_impl(root)
    }
}

pub(crate) fn check_impl(root: &Path) -> CheckResult {
    let target = root.join("target").is_dir();
    let recipe = root.join("recipe.json").is_file();

    match (target, recipe) {
        (true, true) => CheckResult::ok(NAME, "target/ and recipe.json present"),
        (true, false) => CheckResult::warn(NAME, "recipe.json missing")
            .with_details("Run `cargo chef prepare` to enable cached Docker builds"),
        (false, true) => CheckResult::warn(NAME, "target/ missing"),
        (false, false) => CheckResult::warn(NAME, "target/ and recipe.json missing")
            .with_details("Run `cargo build` and `cargo chef prepare`"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn name_is_workspace() {
        assert_eq!(WorkspaceCheck.name(), "workspace");
    }

    #[test]
    fn missing_artifacts_warns() {
        let tmp = TempDir::new().unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Warn);
    }

    #[test]
    fn both_present_is_ok() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join("target")).unwrap();
        fs::write(tmp.path().join("recipe.json"), "{}").unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Ok);
    }
}
