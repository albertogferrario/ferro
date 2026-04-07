//! DB connection check (D-03): inspect `DATABASE_URL` and probe via the
//! project's existing `db:status` subprocess (mirrors `commands::db_status`).

use crate::doctor::check::{CheckResult, DoctorCheck};
use crate::doctor::checks::run_cargo_subcommand;
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

pub(crate) fn check_impl(root: &Path) -> CheckResult {
    // Load .env if present so DATABASE_URL becomes visible.
    let _ = dotenvy::from_path(root.join(".env"));

    let url = match std::env::var("DATABASE_URL") {
        Ok(v) if !v.is_empty() => v,
        _ => {
            return CheckResult::warn(NAME, "DATABASE_URL not set")
                .with_details("Set DATABASE_URL in .env to enable connectivity checks");
        }
    };

    // If the project has no migrations dir, we can't shell out to db:status.
    if !root.join("src/migrations").exists() {
        return CheckResult::warn(NAME, "no migrations crate to probe connectivity");
    }

    match run_cargo_subcommand(root, &["db:status"]) {
        Ok(out) if out.status.success() => {
            CheckResult::ok(NAME, format!("connected ({})", redact_url(&url)))
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            CheckResult::error(NAME, "db:status failed").with_details(stderr)
        }
        Err(e) => CheckResult::error(NAME, format!("could not invoke cargo: {e}")),
    }
}

fn redact_url(url: &str) -> String {
    // Strip credentials between `://` and `@`.
    if let Some(scheme_end) = url.find("://") {
        let rest = &url[scheme_end + 3..];
        if let Some(at) = rest.find('@') {
            return format!("{}://***@{}", &url[..scheme_end], &rest[at + 1..]);
        }
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_db_connection() {
        assert_eq!(DbConnectionCheck.name(), "db_connection");
    }

    #[test]
    fn redact_url_strips_credentials() {
        assert_eq!(
            redact_url("postgres://user:pass@localhost/db"),
            "postgres://***@localhost/db"
        );
        assert_eq!(redact_url("sqlite::memory:"), "sqlite::memory:");
    }
}
