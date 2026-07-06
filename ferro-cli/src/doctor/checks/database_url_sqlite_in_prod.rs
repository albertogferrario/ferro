//! Database URL not sqlite in prod (SCOPE §12.8): if `.env.production` exists
//! and its `DATABASE_URL` starts with `sqlite:`, hard-error.

use crate::doctor::check::{CheckResult, DoctorCheck};
use std::fs;
use std::path::Path;

pub struct DatabaseUrlSqliteInProdCheck;

const NAME: &str = "database_url_sqlite_in_prod";

impl DoctorCheck for DatabaseUrlSqliteInProdCheck {
    fn name(&self) -> &'static str {
        NAME
    }
    fn run(&self, root: &Path) -> CheckResult {
        check_impl(root)
    }
}

pub(crate) fn check_impl(root: &Path) -> CheckResult {
    let path = root.join(".env.production");
    if !path.is_file() {
        return CheckResult::ok(NAME, "skipped (.env.production absent)");
    }
    let content = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return CheckResult::error(NAME, format!("failed to read .env.production: {e}")),
    };
    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        if k.trim() == "DATABASE_URL" {
            let value = v.trim().trim_matches('"').trim_matches('\'');
            if value.starts_with("sqlite:") {
                return CheckResult::error(NAME, "DATABASE_URL in .env.production uses sqlite:")
                    .with_details(
                        "Production must use a network-accessible database (postgres, mysql)",
                    );
            }
            return CheckResult::ok(NAME, "DATABASE_URL is non-sqlite");
        }
    }
    CheckResult::warn(NAME, "DATABASE_URL not declared in .env.production")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn name_is_database_url_sqlite_in_prod() {
        assert_eq!(
            DatabaseUrlSqliteInProdCheck.name(),
            "database_url_sqlite_in_prod"
        );
    }

    #[test]
    fn skipped_when_env_production_absent() {
        let tmp = TempDir::new().unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Ok);
    }

    #[test]
    fn errors_on_sqlite_url() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".env.production"),
            "DATABASE_URL=sqlite:./db.sqlite\n",
        )
        .unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Error);
    }

    #[test]
    fn ok_on_postgres_url() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".env.production"),
            "DATABASE_URL=postgres://u:p@h/db\n",
        )
        .unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Ok);
    }
}
