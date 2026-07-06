//! Phase 156: enforce the generator-owned convention for `frontend/src/types/`.
//!
//! The `ferro generate-types` command writes exactly two files into
//! `frontend/src/types/`: `inertia-props.ts` and `routes.ts`. Any other file
//! in that directory is hand-written and violates the convention — it should
//! live under `frontend/src/lib/types/` instead.
//!
//! Severity is `Warn`, never `Error`: the check is advisory and never blocks
//! the doctor exit code.

use crate::doctor::check::{CheckResult, DoctorCheck};
use std::path::Path;

pub struct FrontendTypesConventionCheck;

const NAME: &str = "frontend_types_convention";

/// Filenames the generator writes (confirmed: generate_types.rs lines 882, 922).
/// If `ferro generate-types` is extended to emit additional files, keep this
/// allowlist in sync — otherwise the new files will be flagged as
/// hand-written and produce false positives.
const GENERATED_ALLOWLIST: &[&str] = &["inertia-props.ts", "routes.ts"];

impl DoctorCheck for FrontendTypesConventionCheck {
    fn name(&self) -> &'static str {
        NAME
    }

    fn run(&self, root: &Path) -> CheckResult {
        check_impl(root)
    }
}

pub(crate) fn check_impl(root: &Path) -> CheckResult {
    let types_dir = root.join("frontend/src/types");
    if !types_dir.is_dir() {
        return CheckResult::ok(NAME, "frontend/src/types absent (clean)");
    }

    let mut hand_written: Vec<String> = match std::fs::read_dir(&types_dir) {
        Ok(rd) => rd
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                if GENERATED_ALLOWLIST.contains(&name.as_str()) {
                    None
                } else {
                    Some(name)
                }
            })
            .collect(),
        Err(_) => return CheckResult::ok(NAME, "frontend/src/types unreadable (skipped)"),
    };
    hand_written.sort();

    if hand_written.is_empty() {
        CheckResult::ok(
            NAME,
            "frontend/src/types contains only generator-owned files",
        )
    } else {
        CheckResult::warn(
            NAME,
            format!(
                "{} hand-written file(s) in frontend/src/types/",
                hand_written.len()
            ),
        )
        .with_details(format!(
            "move to frontend/src/lib/types/: {}",
            hand_written.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::check::CheckStatus;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn name_is_frontend_types_convention() {
        assert_eq!(
            FrontendTypesConventionCheck.name(),
            "frontend_types_convention"
        );
    }

    #[test]
    fn absent_directory_is_ok() {
        let tmp = TempDir::new().unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, CheckStatus::Ok);
        assert!(r.message.contains("absent"));
    }

    #[test]
    fn only_generated_files_is_ok() {
        let tmp = TempDir::new().unwrap();
        let types = tmp.path().join("frontend/src/types");
        fs::create_dir_all(&types).unwrap();
        fs::write(types.join("inertia-props.ts"), "").unwrap();
        fs::write(types.join("routes.ts"), "").unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, CheckStatus::Ok);
        assert!(r.message.contains("only generator-owned"));
    }

    #[test]
    fn only_routes_ts_present_is_ok() {
        let tmp = TempDir::new().unwrap();
        let types = tmp.path().join("frontend/src/types");
        fs::create_dir_all(&types).unwrap();
        fs::write(types.join("routes.ts"), "").unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, CheckStatus::Ok);
    }

    #[test]
    fn hand_written_file_warns() {
        let tmp = TempDir::new().unwrap();
        let types = tmp.path().join("frontend/src/types");
        fs::create_dir_all(&types).unwrap();
        fs::write(types.join("parsed-menu.ts"), "").unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, CheckStatus::Warn);
        let details = r.details.as_ref().expect("details required on warn");
        assert!(details.contains("parsed-menu.ts"));
        assert!(details.contains("frontend/src/lib/types/"));
    }

    #[test]
    fn mixed_generated_and_hand_written_warns_on_hand_written_only() {
        let tmp = TempDir::new().unwrap();
        let types = tmp.path().join("frontend/src/types");
        fs::create_dir_all(&types).unwrap();
        fs::write(types.join("inertia-props.ts"), "").unwrap(); // allowlisted
        fs::write(types.join("theme-config.ts"), "").unwrap(); // hand-written
        let r = check_impl(tmp.path());
        assert_eq!(r.status, CheckStatus::Warn);
        let details = r.details.as_ref().expect("details required on warn");
        assert!(details.contains("theme-config.ts"));
        assert!(!details.contains("inertia-props.ts"));
    }
}
