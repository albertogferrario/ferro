//! Dirty git tree (SCOPE §12.9): warn when `git status --porcelain` has output
//! or when there are unpushed commits relative to upstream.

use crate::doctor::check::{CheckResult, DoctorCheck};
use std::path::Path;
use std::process::Command;

pub struct DirtyGitTreeCheck;

const NAME: &str = "git_clean_and_pushed";

impl DoctorCheck for DirtyGitTreeCheck {
    fn name(&self) -> &'static str {
        NAME
    }
    fn run(&self, root: &Path) -> CheckResult {
        check_impl(root)
    }
}

pub(crate) fn check_impl(root: &Path) -> CheckResult {
    if !root.join(".git").exists() {
        return CheckResult::ok(NAME, "skipped (not a git repo)");
    }

    let porcelain = match run_git(root, &["status", "--porcelain"]) {
        Ok(s) => s,
        Err(e) => return CheckResult::warn(NAME, format!("git status failed: {e}")),
    };
    let dirty = !porcelain.trim().is_empty();

    let unpushed = match run_git(root, &["log", "@{upstream}..HEAD", "--oneline"]) {
        Ok(s) => !s.trim().is_empty(),
        Err(_) => false, // No upstream configured — treat as clean for this signal.
    };

    match (dirty, unpushed) {
        (false, false) => CheckResult::ok(NAME, "working tree clean and pushed"),
        (true, false) => CheckResult::warn(NAME, "working tree has uncommitted changes"),
        (false, true) => CheckResult::warn(NAME, "local commits not pushed to upstream"),
        (true, true) => CheckResult::warn(NAME, "uncommitted changes and unpushed commits present"),
    }
}

fn run_git(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into_owned());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn name_is_git_clean_and_pushed() {
        assert_eq!(DirtyGitTreeCheck.name(), "git_clean_and_pushed");
    }

    #[test]
    fn skipped_when_not_a_git_repo() {
        let tmp = TempDir::new().unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Ok);
        assert!(r.message.contains("skipped"));
    }
}
