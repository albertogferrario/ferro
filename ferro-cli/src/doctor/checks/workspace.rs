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

pub(crate) fn check_impl(_root: &Path) -> CheckResult {
    CheckResult::ok(NAME, "stub")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_workspace() {
        assert_eq!(WorkspaceCheck.name(), "workspace");
    }
}
