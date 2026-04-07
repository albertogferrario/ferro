//! Path deps check (D-06): warn (never error) on any `ferro*` path dep.
//!
//! Reuses `crate::deploy::ferro_deps::find_ferro_path_deps` (Phase 123).
//! Production-context detection (warn vs ok) is future work — Phase 124
//! always emits Warn when path deps exist.

use crate::deploy::ferro_deps::find_ferro_path_deps;
use crate::doctor::check::{CheckResult, CheckStatus, DoctorCheck};
use std::fs;
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

pub(crate) fn check_impl(root: &Path) -> CheckResult {
    let cargo_toml = match fs::read_to_string(root.join("Cargo.toml")) {
        Ok(s) => s,
        Err(_) => return CheckResult::warn(NAME, "Cargo.toml unreadable"),
    };

    let deps = find_ferro_path_deps(&cargo_toml);
    if deps.is_empty() {
        CheckResult::ok(NAME, "no ferro* path deps")
    } else {
        // Always Warn — never Error (D-06).
        let _force_warn = CheckStatus::Warn;
        CheckResult::warn(NAME, format!("{} ferro* path dep(s) detected", deps.len())).with_details(
            format!(
                "{} — ok for dev, warn for prod (production-context detection deferred)",
                deps.join(", ")
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn name_is_path_deps() {
        assert_eq!(PathDepsCheck.name(), "path_deps");
    }

    #[test]
    fn no_path_deps_is_ok() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname=\"x\"\n[dependencies]\nserde=\"1\"\n",
        )
        .unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, CheckStatus::Ok);
    }

    #[test]
    fn path_deps_warn_never_error() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname=\"x\"\n[dependencies]\nferro={path=\"..\"}\n",
        )
        .unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, CheckStatus::Warn);
        assert_ne!(r.status, CheckStatus::Error);
    }
}
