//! Deploy preflight (Phase 128 D-04): flag `copy_dirs` entries that
//! `.dockerignore` would silently exclude from the Docker build context.

use crate::doctor::check::{CheckCategory, CheckResult, DoctorCheck};
use crate::project::read_deploy_metadata;
use std::fs;
use std::path::Path;

pub struct CopyDirsDockerignoreCollisionCheck;

const NAME: &str = "copy_dirs_dockerignore_collision";

impl DoctorCheck for CopyDirsDockerignoreCollisionCheck {
    fn name(&self) -> &'static str {
        NAME
    }
    fn run(&self, root: &Path) -> CheckResult {
        check_impl(root)
    }
    fn category(&self) -> CheckCategory {
        CheckCategory::Deploy
    }
}

pub(crate) fn check_impl(root: &Path) -> CheckResult {
    let dockerignore = root.join(".dockerignore");
    if !dockerignore.is_file() {
        return CheckResult::ok(NAME, "skipped (.dockerignore absent)");
    }
    let metadata = match read_deploy_metadata(root) {
        Ok(m) => m,
        Err(_) => return CheckResult::ok(NAME, "skipped (deploy metadata absent)"),
    };
    if metadata.copy_dirs.is_empty() {
        return CheckResult::ok(NAME, "no copy_dirs declared");
    }
    let ignore_content = match fs::read_to_string(&dockerignore) {
        Ok(s) => s,
        Err(e) => {
            return CheckResult::error(NAME, format!("failed to read .dockerignore: {e}"))
        }
    };
    let ignore_lines: Vec<&str> = ignore_content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with('!'))
        .collect();

    let mut collisions: Vec<String> = Vec::new();
    for entry in &metadata.copy_dirs {
        let entry_head = entry.split('/').next().unwrap_or(entry);
        for line in &ignore_lines {
            let stripped = line.trim_end_matches('/');
            if stripped == entry_head || stripped == entry.as_str() {
                collisions.push(format!(
                    "'{entry}' excluded by .dockerignore rule '{line}'"
                ));
                break;
            }
        }
    }

    if collisions.is_empty() {
        CheckResult::ok(NAME, "copy_dirs entries not excluded by .dockerignore")
    } else {
        CheckResult::error(
            NAME,
            format!("{} copy_dirs entries collide with .dockerignore", collisions.len()),
        )
        .with_details(collisions.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::check::CheckStatus;
    use tempfile::TempDir;

    fn write(p: &Path, body: &str) {
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, body).unwrap();
    }

    #[test]
    fn name_and_category() {
        assert_eq!(CopyDirsDockerignoreCollisionCheck.name(), NAME);
        assert_eq!(
            CopyDirsDockerignoreCollisionCheck.category(),
            CheckCategory::Deploy
        );
    }

    #[test]
    fn skipped_when_dockerignore_absent() {
        let td = TempDir::new().unwrap();
        write(
            &td.path().join("Cargo.toml"),
            "[package]\nname=\"x\"\nversion=\"0.1.0\"\n[package.metadata.ferro.deploy]\ncopy_dirs=[\"data\"]\n",
        );
        let r = check_impl(td.path());
        assert_eq!(r.status, CheckStatus::Ok);
        assert!(r.message.contains("skipped"));
    }

    #[test]
    fn errors_when_copy_dir_collides() {
        let td = TempDir::new().unwrap();
        write(
            &td.path().join("Cargo.toml"),
            "[package]\nname=\"x\"\nversion=\"0.1.0\"\n[package.metadata.ferro.deploy]\ncopy_dirs=[\"data\"]\n",
        );
        write(
            &td.path().join(".dockerignore"),
            "# comment\ntarget/\ndata/\n*.log\n",
        );
        let r = check_impl(td.path());
        assert_eq!(r.status, CheckStatus::Error);
        assert!(r.details.as_ref().unwrap().contains("data"));
        assert!(r.details.as_ref().unwrap().contains("data/"));
    }

    #[test]
    fn ok_when_no_collision() {
        let td = TempDir::new().unwrap();
        write(
            &td.path().join("Cargo.toml"),
            "[package]\nname=\"x\"\nversion=\"0.1.0\"\n[package.metadata.ferro.deploy]\ncopy_dirs=[\"migrations\"]\n",
        );
        write(&td.path().join(".dockerignore"), "target/\ndata/\n");
        let r = check_impl(td.path());
        assert_eq!(r.status, CheckStatus::Ok);
    }

    #[test]
    fn negation_lines_ignored() {
        let td = TempDir::new().unwrap();
        write(
            &td.path().join("Cargo.toml"),
            "[package]\nname=\"x\"\nversion=\"0.1.0\"\n[package.metadata.ferro.deploy]\ncopy_dirs=[\"data\"]\n",
        );
        write(&td.path().join(".dockerignore"), "!data/\n");
        let r = check_impl(td.path());
        assert_eq!(r.status, CheckStatus::Ok);
    }
}
