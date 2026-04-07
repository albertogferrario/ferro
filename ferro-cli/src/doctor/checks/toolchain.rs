//! Toolchain check (D-02): rustc/cargo version vs `rust-toolchain.toml`.

use crate::doctor::check::{CheckResult, DoctorCheck};
use std::path::Path;

pub struct ToolchainCheck;

const NAME: &str = "toolchain";

impl DoctorCheck for ToolchainCheck {
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
    fn name_is_toolchain() {
        assert_eq!(ToolchainCheck.name(), "toolchain");
    }
}
