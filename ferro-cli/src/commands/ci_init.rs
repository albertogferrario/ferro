//! ferro ci:init — drop `.github/workflows/ci.yml` (D-13, D-17, D-21).
//!
//! Standalone CI scaffold for projects not on DigitalOcean. Idempotent:
//! refuses to overwrite an existing workflow unless `--force` is passed.

use console::style;
use std::fs;
use std::path::Path;

use crate::project::{find_project_root, package_name};
use crate::templates::ci_workflow::{render_ci_workflow, CiWorkflowContext};

pub fn run(force: bool) {
    let root = match find_project_root(None) {
        Ok(r) => r,
        Err(_) => {
            eprintln!(
                "{} Cargo.toml not found (searched upward from CWD)",
                style("Error:").red().bold()
            );
            std::process::exit(1);
        }
    };

    match generate_in(&root, force) {
        Ok(path) => {
            println!("{} Generated {}", style("✓").green(), path.display());
        }
        Err(GenerateError::Exists(path)) => {
            eprintln!(
                "{} {} already exists (use --force)",
                style("Error:").red().bold(),
                path.display()
            );
            std::process::exit(1);
        }
        Err(GenerateError::Io(e)) => {
            eprintln!("{} {}", style("Error:").red().bold(), e);
            std::process::exit(1);
        }
    }
}

#[derive(Debug)]
enum GenerateError {
    Exists(std::path::PathBuf),
    Io(std::io::Error),
}

impl From<std::io::Error> for GenerateError {
    fn from(e: std::io::Error) -> Self {
        GenerateError::Io(e)
    }
}

fn generate_in(root: &Path, force: bool) -> Result<std::path::PathBuf, GenerateError> {
    let workflows_dir = root.join(".github").join("workflows");
    let ci_yml = workflows_dir.join("ci.yml");

    if ci_yml.exists() && !force {
        return Err(GenerateError::Exists(ci_yml));
    }

    fs::create_dir_all(&workflows_dir)?;

    let pkg = package_name(root);
    let content = render_ci_workflow(&CiWorkflowContext { package_name: &pkg });
    fs::write(&ci_yml, content)?;
    Ok(ci_yml)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_min_project(td: &TempDir) {
        fs::write(
            td.path().join("Cargo.toml"),
            "[package]\nname = \"sample\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
    }

    #[test]
    fn writes_ci_yml_under_github_workflows() {
        let td = TempDir::new().unwrap();
        write_min_project(&td);
        let path = generate_in(td.path(), false).unwrap();
        assert!(path.ends_with(".github/workflows/ci.yml"));
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("Swatinem/rust-cache@v2"));
        assert!(body.contains("cargo fmt --all -- --check"));
    }

    #[test]
    fn refuses_to_overwrite_without_force() {
        let td = TempDir::new().unwrap();
        write_min_project(&td);
        generate_in(td.path(), false).unwrap();
        match generate_in(td.path(), false) {
            Err(GenerateError::Exists(_)) => {}
            other => panic!("expected Exists error, got {other:?}"),
        }
    }

    #[test]
    fn force_overwrites_existing_with_identical_content() {
        let td = TempDir::new().unwrap();
        write_min_project(&td);
        let path = generate_in(td.path(), false).unwrap();
        let first = fs::read_to_string(&path).unwrap();
        // Mutate file to confirm overwrite happens.
        fs::write(&path, "stale\n").unwrap();
        let path2 = generate_in(td.path(), true).unwrap();
        let second = fs::read_to_string(&path2).unwrap();
        assert_eq!(first, second);
    }
}
