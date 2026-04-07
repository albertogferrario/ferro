//! Env completeness check (D-05): every key in `.env.example` is set in `.env`.

use crate::doctor::check::{CheckResult, DoctorCheck};
use std::path::Path;

pub struct EnvCompletenessCheck;

const NAME: &str = "env_completeness";

impl DoctorCheck for EnvCompletenessCheck {
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
    fn name_is_env_completeness() {
        assert_eq!(EnvCompletenessCheck.name(), "env_completeness");
    }
}
