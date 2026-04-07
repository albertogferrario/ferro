//! Env completeness check (D-05): every key in `.env.example` is set in `.env`.
//!
//! Reuses `crate::deploy::parse_env_entries` for both files — no duplicate parser.

use crate::deploy::parse_env_entries;
use crate::doctor::check::{CheckResult, DoctorCheck};
use std::collections::HashSet;
use std::fs;
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

pub(crate) fn check_impl(root: &Path) -> CheckResult {
    let example_path = root.join(".env.example");
    let env_path = root.join(".env");
    let example_exists = example_path.is_file();
    let env_exists = env_path.is_file();

    if !example_exists && !env_exists {
        return CheckResult::warn(NAME, ".env and .env.example both missing");
    }
    if !example_exists {
        return CheckResult::warn(NAME, ".env.example missing");
    }
    if !env_exists {
        return CheckResult::error(NAME, ".env missing — required by .env.example")
            .with_details("Copy .env.example to .env and fill in real values");
    }

    let example = fs::read_to_string(&example_path).unwrap_or_default();
    let env = fs::read_to_string(&env_path).unwrap_or_default();

    let example_keys: Vec<String> = parse_env_entries(&example)
        .into_iter()
        .map(|e| e.key)
        .collect();
    let env_keys: HashSet<String> = parse_env_entries(&env).into_iter().map(|e| e.key).collect();

    let missing: Vec<&String> = example_keys
        .iter()
        .filter(|k| !env_keys.contains(*k))
        .collect();

    if missing.is_empty() {
        CheckResult::ok(NAME, format!("{} keys present", example_keys.len()))
    } else {
        let list = missing
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        CheckResult::error(NAME, format!("{} key(s) missing from .env", missing.len()))
            .with_details(format!("missing: {list}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write(root: &Path, name: &str, content: &str) {
        fs::write(root.join(name), content).unwrap();
    }

    #[test]
    fn name_is_env_completeness() {
        assert_eq!(EnvCompletenessCheck.name(), "env_completeness");
    }

    #[test]
    fn matching_keys_returns_ok() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), ".env.example", "KEY1=\nKEY2=\n");
        write(tmp.path(), ".env", "KEY1=a\nKEY2=b\n");
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Ok);
    }

    #[test]
    fn missing_key_returns_error_with_details() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), ".env.example", "KEY1=\nKEY2=\n");
        write(tmp.path(), ".env", "KEY1=a\n");
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Error);
        assert!(r.details.unwrap().contains("KEY2"));
    }

    #[test]
    fn both_files_missing_warns() {
        let tmp = TempDir::new().unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Warn);
    }
}
