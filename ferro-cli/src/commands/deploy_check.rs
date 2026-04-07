//! deploy:check — pre-flight pushed-ness check for the chosen ferro git ref.
//!
//! Verifies that the chosen ferro git ref is reachable on the canonical remote
//! before a Docker build attempts to fetch it. Enforces D-11 from
//! `.planning/phases/122-deploy-scaffold-core-rewrite/SCOPE.md`.

use console::style;
use std::process::Command;

const FERRO_REPO: &str = "https://github.com/albertogferrario/ferro";

pub fn run(ferro_ref: &str) {
    match check_ref(FERRO_REPO, ferro_ref) {
        Ok(true) => {
            println!(
                "{} ferro ref '{}' is reachable on {}",
                style("✓").green(),
                ferro_ref,
                FERRO_REPO
            );
        }
        Ok(false) => {
            eprintln!(
                "{} ferro ref '{}' is NOT reachable on {}",
                style("Error:").red().bold(),
                ferro_ref,
                FERRO_REPO
            );
            eprintln!(
                "{}",
                style("Push your ferro commits to the remote, then re-run `ferro deploy:check`.")
                    .dim()
            );
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!(
                "{} git ls-remote failed: {}",
                style("Error:").red().bold(),
                e
            );
            std::process::exit(2);
        }
    }
}

/// Check whether `ref_name` is reachable on `repo_url` via `git ls-remote --exit-code`.
///
/// Returns:
/// - `Ok(true)` when git exits 0 (ref found).
/// - `Ok(false)` when git exits 2 (ref not found, per `--exit-code` semantics).
/// - `Err(msg)` for any other failure (network, auth, missing binary, signal).
pub fn check_ref(repo_url: &str, ref_name: &str) -> Result<bool, String> {
    let output = Command::new("git")
        .args(["ls-remote", "--exit-code", repo_url, ref_name])
        .output()
        .map_err(|e| format!("failed to invoke git: {e}"))?;

    match output.status.code() {
        Some(0) => Ok(true),
        Some(2) => Ok(false),
        Some(code) => Err(format!(
            "git ls-remote exited with code {code}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
        None => Err("git ls-remote terminated by signal".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn git_available() -> bool {
        Command::new("git").arg("--version").output().is_ok()
    }

    fn run_git(cwd: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?} failed");
    }

    #[test]
    fn reachable_ref_returns_true() {
        if !git_available() {
            return;
        }
        let td = TempDir::new().unwrap();
        let bare = td.path().join("repo.git");
        fs::create_dir(&bare).unwrap();
        run_git(&bare, &["init", "--bare", "--initial-branch=main"]);

        let work = td.path().join("work");
        fs::create_dir(&work).unwrap();
        run_git(&work, &["init", "--initial-branch=main"]);
        run_git(&work, &["config", "user.email", "t@t.test"]);
        run_git(&work, &["config", "user.name", "t"]);
        fs::write(work.join("a.txt"), "x").unwrap();
        run_git(&work, &["add", "a.txt"]);
        run_git(&work, &["commit", "-m", "init"]);
        let bare_url = bare.to_string_lossy().to_string();
        run_git(&work, &["remote", "add", "origin", &bare_url]);
        run_git(&work, &["push", "origin", "main"]);

        assert!(check_ref(&bare_url, "main").unwrap());
    }

    #[test]
    fn unreachable_ref_returns_false() {
        if !git_available() {
            return;
        }
        let td = TempDir::new().unwrap();
        let bare = td.path().join("repo.git");
        fs::create_dir(&bare).unwrap();
        run_git(&bare, &["init", "--bare", "--initial-branch=main"]);
        let bare_url = bare.to_string_lossy().to_string();
        assert!(!check_ref(&bare_url, "nonexistent-branch").unwrap());
    }

    #[test]
    fn invalid_repo_returns_err() {
        if !git_available() {
            return;
        }
        let r = check_ref("/this/does/not/exist/at/all.git", "main");
        // git may report 128 (fatal) or 2 (no match); both are acceptable —
        // the contract is "not Ok(true)".
        assert!(r.is_err() || matches!(r, Ok(false)));
    }
}
