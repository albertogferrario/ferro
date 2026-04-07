//! Migrations check (D-04): pending vs applied count via `cargo run -- db:status`.

use crate::doctor::check::{CheckResult, DoctorCheck};
use std::path::Path;

pub struct MigrationsCheck;

const NAME: &str = "migrations";

impl DoctorCheck for MigrationsCheck {
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
    fn name_is_migrations() {
        assert_eq!(MigrationsCheck.name(), "migrations");
    }
}
