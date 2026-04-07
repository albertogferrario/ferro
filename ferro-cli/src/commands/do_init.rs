//! `ferro do:init` — generate `.do/app.yaml` starter (Phase 122.2 §5).
//!
//! Writes only `.do/app.yaml`. CI workflow generation lives in
//! `ferro ci:init` (decoupled per SCOPE §7). Hard-errors when
//! `.env.production` is missing.

use console::style;
use std::fs;
use std::path::Path;

use crate::deploy::env_production::read_env_production_keys;
use crate::project::{find_project_root, package_name};
use crate::templates::do_::{
    is_test_like_bin, parse_git_remote, render_app_yaml, sanitize_do_app_name, AppYamlContext,
};
use crate::templates::docker::read_bins;

pub fn run(force: bool) {
    if let Err(e) = run_inner(force) {
        eprintln!("{} {e}", style("Error:").red().bold());
        std::process::exit(1);
    }
}

fn run_inner(force: bool) -> anyhow::Result<()> {
    let root = find_project_root(None)
        .map_err(|_| anyhow::anyhow!("Cargo.toml not found (searched upward from CWD)"))?;
    let pkg = package_name(&root);
    let name = sanitize_do_app_name(&pkg);

    let repo = detect_github_repo(&root).unwrap_or_else(|| "owner/your-repo".to_string());

    let bins = read_bins(&root)?;
    let web_bin = bins
        .iter()
        .find(|b| **b == name || **b == pkg)
        .cloned()
        .unwrap_or_else(|| name.clone());
    let workers: Vec<String> = bins
        .iter()
        .filter(|b| **b != web_bin)
        .filter(|b| !is_test_like_bin(b))
        .cloned()
        .collect();

    let env_path = root.join(".env.production");
    if !env_path.exists() {
        anyhow::bail!(
            ".env.production not found.\nCreate it from .env.example and fill in production values:\n  cp .env.example .env.production\n  $EDITOR .env.production"
        );
    }
    let env_keys = read_env_production_keys(&env_path)?;

    let ctx = AppYamlContext {
        name,
        repo,
        web_bin,
        workers,
        env_keys,
    };
    let yaml = render_app_yaml(&ctx);

    let target = root.join(".do/app.yaml");
    write_with_force(&target, &yaml, force)?;
    println!("{} Generated {}", style("✓").green(), target.display());
    Ok(())
}

fn detect_github_repo(root: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?;
    parse_git_remote(s.trim())
}

fn write_with_force(path: &Path, content: &str, force: bool) -> anyhow::Result<()> {
    if path.exists() && !force {
        anyhow::bail!("{} already exists (use --force)", path.display());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(root: &Path, rel: &str, body: &str) {
        let p = root.join(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, body).unwrap();
    }

    #[test]
    fn write_with_force_refuses_existing() {
        let td = TempDir::new().unwrap();
        let p = td.path().join(".do/app.yaml");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, "old").unwrap();
        assert!(write_with_force(&p, "new", false).is_err());
        assert_eq!(fs::read_to_string(&p).unwrap(), "old");
    }

    #[test]
    fn write_with_force_overwrites_with_force() {
        let td = TempDir::new().unwrap();
        let p = td.path().join(".do/app.yaml");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, "old").unwrap();
        write_with_force(&p, "new", true).unwrap();
        assert_eq!(fs::read_to_string(&p).unwrap(), "new");
    }

    #[test]
    fn run_inner_errors_on_missing_env_production() {
        let td = TempDir::new().unwrap();
        write(
            td.path(),
            "Cargo.toml",
            "[package]\nname = \"sample\"\nversion = \"0.1.0\"\n",
        );
        // Run from inside the project root via CWD swap.
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(td.path()).unwrap();
        let result = run_inner(false);
        std::env::set_current_dir(prev).unwrap();
        let err = result.unwrap_err().to_string();
        assert!(err.contains(".env.production not found"));
    }
}
