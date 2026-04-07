//! Path deps check (D-06): warn (never error) on any `ferro*` path dep.

use crate::doctor::check::{CheckResult, DoctorCheck};
use std::path::Path;

pub struct PathDepsCheck;

const NAME: &str = "path_deps";

impl DoctorCheck for PathDepsCheck {
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
    fn name_is_path_deps() {
        assert_eq!(PathDepsCheck.name(), "path_deps");
    }
}
