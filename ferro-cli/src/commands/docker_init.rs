//! docker:init command — generate production Dockerfile, .dockerignore, and
//! the path->git ferro dep rewrite script.

use console::style;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::deploy::ferro_deps::render_rewrite_script;
use crate::project::{
    detect_dirs, find_project_root, package_name, read_bins, read_workspace_members,
    resolve_rust_base_image,
};
use crate::templates::{dockerignore_template, render_dockerfile, DockerfileContext};

pub fn run(force: bool, ferro_ref: &str, runtime_deps: &[String]) {
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

    match generate_in(&root, force, ferro_ref, runtime_deps) {
        Ok(true) => print_next_steps(&root),
        Ok(false) => {
            eprintln!(
                "{} Dockerfile already exists (use --force to overwrite)",
                style("Info:").yellow().bold()
            );
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("{} {}", style("Error:").red().bold(), e);
            std::process::exit(1);
        }
    }
}

pub fn generate(force: bool, ferro_ref: &str, runtime_deps: &[String]) -> bool {
    let Ok(root) = find_project_root(None) else {
        return false;
    };
    generate_in(&root, force, ferro_ref, runtime_deps).unwrap_or(false)
}

fn generate_in(
    root: &Path,
    force: bool,
    ferro_ref: &str,
    runtime_deps: &[String],
) -> std::io::Result<bool> {
    let dockerfile = root.join("Dockerfile");
    let dockerignore = root.join(".dockerignore");
    let scripts_dir = root.join("scripts");
    let rewrite_sh = scripts_dir.join("rewrite-ferro-deps.sh");

    if dockerfile.exists() && !force {
        return Ok(false);
    }

    let pkg = package_name(root);
    let bins = read_bins(root);
    let workspace = read_workspace_members(root);
    let base_image = resolve_rust_base_image(root);
    let dirs = detect_dirs(root);

    let ctx = DockerfileContext {
        package_name: &pkg,
        bins: &bins,
        dirs,
        runtime_deps,
        rust_base_image: &base_image,
        workspace_members: &workspace,
        ferro_ref,
    };
    let dockerfile_content = render_dockerfile(&ctx);
    fs::write(&dockerfile, dockerfile_content)?;
    println!("{} Wrote {}", style("✓").green(), dockerfile.display());

    if !dockerignore.exists() || force {
        fs::write(&dockerignore, dockerignore_template())?;
        println!("{} Wrote {}", style("✓").green(), dockerignore.display());
    }

    fs::create_dir_all(&scripts_dir)?;
    let script = render_rewrite_script(&root.join("Cargo.toml"), ferro_ref)?;
    fs::write(&rewrite_sh, script)?;
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&rewrite_sh)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&rewrite_sh, perms)?;
    }
    println!("{} Wrote {}", style("✓").green(), rewrite_sh.display());

    Ok(true)
}

fn print_next_steps(root: &Path) {
    let pkg = package_name(root);
    println!();
    println!("{}", style("Docker scaffolding complete.").cyan().bold());
    println!();
    println!("Build:");
    println!(
        "  {}",
        style(format!(
            "docker build --build-arg GITHUB_TOKEN=$GITHUB_TOKEN -t {pkg} ."
        ))
        .cyan()
    );
    println!();
    println!("Run:");
    println!(
        "  {}",
        style(format!(
            "docker run -p 8080:8080 --env-file .env.production {pkg}"
        ))
        .cyan()
    );
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_cargo(root: &Path, name: &str) {
        fs::write(
            root.join("Cargo.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        )
        .unwrap();
    }

    #[test]
    fn generates_full_set_on_empty_project() {
        let td = TempDir::new().unwrap();
        write_cargo(td.path(), "sample");
        let r = generate_in(td.path(), false, "main", &[]).unwrap();
        assert!(r);
        assert!(td.path().join("Dockerfile").is_file());
        assert!(td.path().join(".dockerignore").is_file());
        assert!(td.path().join("scripts/rewrite-ferro-deps.sh").is_file());
    }

    #[test]
    fn refuses_to_overwrite_without_force() {
        let td = TempDir::new().unwrap();
        write_cargo(td.path(), "sample");
        fs::write(td.path().join("Dockerfile"), "EXISTING").unwrap();
        let r = generate_in(td.path(), false, "main", &[]).unwrap();
        assert!(!r);
        let content = fs::read_to_string(td.path().join("Dockerfile")).unwrap();
        assert_eq!(content, "EXISTING");
    }

    #[test]
    fn overwrites_with_force() {
        let td = TempDir::new().unwrap();
        write_cargo(td.path(), "sample");
        fs::write(td.path().join("Dockerfile"), "OLD").unwrap();
        let r = generate_in(td.path(), true, "main", &[]).unwrap();
        assert!(r);
        let content = fs::read_to_string(td.path().join("Dockerfile")).unwrap();
        assert!(content.contains("cargo chef"));
    }

    #[test]
    fn writes_ferro_ref_into_script_header() {
        let td = TempDir::new().unwrap();
        write_cargo(td.path(), "sample");
        generate_in(td.path(), true, "v0.1.87", &[]).unwrap();
        let script = fs::read_to_string(td.path().join("scripts/rewrite-ferro-deps.sh")).unwrap();
        assert!(script.contains("FERRO_REF=\"v0.1.87\""));
    }

    #[test]
    fn runtime_deps_appear_in_dockerfile() {
        let td = TempDir::new().unwrap();
        write_cargo(td.path(), "sample");
        let deps = vec!["chromium".into(), "fonts-liberation".into()];
        generate_in(td.path(), true, "main", &deps).unwrap();
        let df = fs::read_to_string(td.path().join("Dockerfile")).unwrap();
        assert!(df.contains("chromium"));
        assert!(df.contains("ferro:runtime-deps"));
    }
}
