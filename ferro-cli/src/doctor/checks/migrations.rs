//! Migrations check (D-04): pending vs applied count via `cargo run -- db:status`.
//!
//! Mirrors the subprocess pattern from `commands::db_status` to avoid pulling
//! SeaORM into doctor's compile graph.

use crate::doctor::check::{CheckResult, DoctorCheck};
use crate::doctor::checks::run_cargo_subcommand;
use std::path::Path;

pub struct MigrationsCheck;

const NAME: &str = "migrations_pending";

impl DoctorCheck for MigrationsCheck {
    fn name(&self) -> &'static str {
        NAME
    }
    fn run(&self, root: &Path) -> CheckResult {
        check_impl(root)
    }
}

pub(crate) fn check_impl(root: &Path) -> CheckResult {
    if !root.join("src/migrations").exists() {
        return CheckResult::warn(NAME, "no migrations directory");
    }

    let output = match run_cargo_subcommand(root, &["db:status"]) {
        Ok(o) => o,
        Err(e) => return CheckResult::error(NAME, format!("could not invoke cargo: {e}")),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return CheckResult::error(NAME, "db:status failed").with_details(stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let pending = count_lines_containing(&stdout, "pending");
    let applied = count_lines_containing(&stdout, "applied");

    if pending == 0 {
        CheckResult::ok(NAME, format!("{applied} applied, 0 pending"))
    } else {
        CheckResult::warn(NAME, format!("{applied} applied, {pending} pending"))
    }
}

fn count_lines_containing(text: &str, needle: &str) -> usize {
    text.lines()
        .filter(|l| l.to_ascii_lowercase().contains(needle))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_migrations() {
        assert_eq!(MigrationsCheck.name(), "migrations_pending");
    }

    #[test]
    fn count_lines_containing_is_case_insensitive() {
        let text = "Pending: m1\nApplied: m0\nPending: m2";
        assert_eq!(count_lines_containing(text, "pending"), 2);
        assert_eq!(count_lines_containing(text, "applied"), 1);
    }
}
