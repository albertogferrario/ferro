//! `ferro ignore:sync` — reconcile `.gitignore` and `.dockerignore` in an
//! existing project against the canonical `ignore_patterns.toml` (D-18..D-20).

use crate::project::find_project_root;
use crate::templates::ignore_patterns::{load_default, render_dockerignore, render_gitignore};
use console::style;
use std::fs;
use std::path::Path;
use std::process;

pub fn run(dry_run: bool, force: bool) {
    let root = match find_project_root(None) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} {e}", style("error:").red().bold());
            process::exit(1);
        }
    };

    let patterns = load_default();
    let target_docker = render_dockerignore(&patterns);
    let target_git = render_gitignore(&patterns);

    let docker_path = root.join(".dockerignore");
    let git_path = root.join(".gitignore");

    let docker_drift = current_differs(&docker_path, &target_docker);
    let git_drift = current_differs(&git_path, &target_git);

    if !docker_drift && !git_drift {
        println!(
            "{} .gitignore and .dockerignore already in sync",
            style("✓").green()
        );
        return;
    }

    if dry_run {
        if docker_drift {
            println!("{} .dockerignore would change", style("DRIFT").yellow());
            print_diff(&docker_path, &target_docker);
        }
        if git_drift {
            println!("{} .gitignore would change", style("DRIFT").yellow());
            print_diff(&git_path, &target_git);
        }
        // non-zero so CI can detect drift without erroring as a hard failure
        process::exit(2);
    }

    if docker_drift {
        write_file(&docker_path, &target_docker, force, "dockerignore");
    }
    if git_drift {
        write_file(&git_path, &target_git, force, "gitignore");
    }
}

/// True when `path` is missing or its on-disk bytes differ from `target`
/// after normalizing trailing whitespace at EOF.
fn current_differs(path: &Path, target: &str) -> bool {
    match fs::read_to_string(path) {
        Ok(actual) => normalize(&actual) != normalize(target),
        Err(_) => true,
    }
}

fn normalize(s: &str) -> String {
    let mut t = s.replace("\r\n", "\n");
    while t.ends_with('\n') {
        t.pop();
    }
    t
}

fn print_diff(path: &Path, target: &str) {
    let actual_lines = fs::read_to_string(path)
        .map(|s| s.lines().count())
        .unwrap_or(0);
    let target_lines = target.lines().count();
    println!(
        "  {}: {} lines on disk → {} lines from canonical source",
        path.display(),
        actual_lines,
        target_lines
    );
}

fn write_file(path: &Path, target: &str, force: bool, label: &str) {
    let exists = path.exists();
    if exists && !force {
        // Allow overwrite if only trailing-whitespace differs (auto-fixable).
        let actual = fs::read_to_string(path).unwrap_or_default();
        if normalize(&actual) != normalize(target) {
            // Real content drift: refuse without --force.
            let body_changed =
                strip_blank_and_comments(&actual) != strip_blank_and_comments(target);
            if body_changed {
                eprintln!(
                    "{} {} exists and contains custom patterns. Re-run with --force to overwrite.",
                    style("error:").red().bold(),
                    path.display()
                );
                process::exit(1);
            }
        }
    }

    if let Err(e) = fs::write(path, target) {
        eprintln!(
            "{} failed to write {}: {e}",
            style("error:").red().bold(),
            path.display()
        );
        process::exit(1);
    }
    println!(
        "{} synced .{label} ({} bytes)",
        style("✓").green(),
        target.len()
    );
}

fn strip_blank_and_comments(s: &str) -> Vec<&str> {
    s.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn current_differs_when_missing() {
        let tmp = TempDir::new().unwrap();
        assert!(current_differs(&tmp.path().join("nope"), "anything"));
    }

    #[test]
    fn current_differs_ignores_trailing_newlines() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("f");
        fs::write(&p, "a\nb\n\n\n").unwrap();
        assert!(!current_differs(&p, "a\nb"));
    }

    #[test]
    fn current_differs_detects_real_drift() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("f");
        fs::write(&p, "a\nb\n").unwrap();
        assert!(current_differs(&p, "a\nc\n"));
    }
}
