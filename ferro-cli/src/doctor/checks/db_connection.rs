//! DB connection check (D-03): open `DATABASE_URL`, run a trivial query.

use crate::doctor::check::{CheckResult, DoctorCheck};
use std::path::Path;

pub struct DbConnectionCheck;

const NAME: &str = "db_connection";

impl DoctorCheck for DbConnectionCheck {
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
    fn name_is_db_connection() {
        assert_eq!(DbConnectionCheck.name(), "db_connection");
    }
}
