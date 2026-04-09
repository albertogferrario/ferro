//! `ferro do:init` — generate `.do/app.yaml` starter (Phase 122.2 §5).
//!
//! Writes only `.do/app.yaml`. CI workflow generation lives in
//! `ferro ci:init` (decoupled per SCOPE §7). Hard-errors when
//! `.env.production` is missing.

use console::style;
use std::fs;
use std::path::Path;

use crate::commands::docker_init::{print_dry_run, RenderedFile};
use crate::deploy::app_yaml_existing::parse_existing;
use crate::deploy::bin_detect::detect_web_bin;
use crate::deploy::env_production::parse_env_example_structured;
use crate::project::{find_project_root, package_name, read_bins};
use crate::templates::do_::{
    is_test_like_bin, parse_git_remote, render_app_yaml, sanitize_do_app_name, AppYamlContext,
};

pub fn run(force: bool) {
    run_with(force, false);
}

/// Full entry point supporting `--dry-run`.
pub fn run_with(force: bool, dry_run: bool) {
    if let Err(e) = run_inner(force, dry_run) {
        eprintln!("{} {e}", style("Error:").red().bold());
        std::process::exit(1);
    }
}

/// Library-level entry point used by integration tests. Returns `Result`.
pub fn execute(force: bool, dry_run: bool) -> anyhow::Result<()> {
    run_inner(force, dry_run)
}

fn run_inner(force: bool, dry_run: bool) -> anyhow::Result<()> {
    let root = find_project_root(None)
        .map_err(|_| anyhow::anyhow!("Cargo.toml not found (searched upward from CWD)"))?;
    let pkg = package_name(&root);
    let name = sanitize_do_app_name(&pkg);

    let repo = detect_github_repo(&root).unwrap_or_else(|| "owner/your-repo".to_string());

    let bins: Vec<String> = read_bins(&root).into_iter().map(|b| b.name).collect();
    let web_bin = detect_web_bin(&root)?;
    let workers: Vec<String> = bins
        .iter()
        .filter(|b| **b != web_bin)
        .filter(|b| !is_test_like_bin(b))
        .cloned()
        .collect();

    // Phase 127 D-06: derive envs from `.env.example` (the shape source of
    // truth) rather than `.env.production` (which stays on the dev machine).
    // Missing `.env.example` is a warning, not an error — the rendered envs:
    // block will simply be empty.
    let env_example_path = root.join(".env.example");
    let env_lines = match fs::read_to_string(&env_example_path) {
        Ok(contents) => Some(parse_env_example_structured(&contents)),
        Err(_) => {
            eprintln!(
                "{} .env.example not found; rendering empty envs: block. \
                 Populate envs in .do/app.yaml before `doctl apps create`.",
                style("warning:").yellow().bold()
            );
            None
        }
    };

    // Phase 131 REQ-131-06: preserve identity fields from an existing
    // .do/app.yaml when re-rendering with --force. The parsed fields override
    // the scaffolder defaults (name from package, region fra1, repo from git
    // remote, branch main).
    let existing_app_yaml = root.join(".do/app.yaml");
    let preserved = parse_existing(&existing_app_yaml);

    let (preserved_name, preserved_region, preserved_github_repo, preserved_github_branch) =
        match preserved {
            Some(id) => (id.name, id.region, id.repo, id.branch),
            None => (None, None, None, None),
        };

    let ctx = AppYamlContext {
        name,
        repo,
        web_bin,
        workers,
        env_lines,
        preserved_name,
        preserved_region,
        preserved_github_repo,
        preserved_github_branch,
    };
    let yaml = render_app_yaml(&ctx);

    if dry_run {
        // D-17: render every output to memory and print with headers.
        // Render errors remain hard errors (D-19).
        let files = [RenderedFile {
            relative_path: ".do/app.yaml".into(),
            contents: yaml,
        }];
        print_dry_run(&files);
        return Ok(());
    }

    let target = root.join(".do/app.yaml");
    write_with_force(&target, &yaml, force)?;
    println!("{} Generated {}", style("✓").green(), target.display());

    // D-13, D-15: cargo-style "Next steps" footer.
    print!("{}", do_init_footer());
    Ok(())
}

/// Build the cargo-style "Next steps" footer for `do:init`. Pure string —
/// the `print!` call site exists so tests can assert on the content directly.
fn do_init_footer() -> String {
    "\nNext steps:\n  Review .do/app.yaml and populate envs.\n  doctl apps create --spec .do/app.yaml\n"
        .to_string()
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
    fn do_init_footer_contents() {
        let s = do_init_footer();
        assert!(s.contains("doctl apps create --spec"));
        assert!(s.contains(".do/app.yaml"));
    }

    #[test]
    fn do_init_footer_line_count() {
        let s = do_init_footer();
        let n = s.lines().filter(|l| !l.trim().is_empty()).count();
        assert!((3..=5).contains(&n), "footer has {n} non-empty lines: {s}");
        assert!(s.is_ascii(), "footer must be ASCII-only");
    }

    #[test]
    fn dry_run_propagates_render_error() {
        let _guard = crate::commands::CWD_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        // D-19: --dry-run must not demote render errors to soft warnings.
        // Running run_inner in an empty tempdir (no Cargo.toml) hits the
        // hard error at find_project_root.
        let td = TempDir::new().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(td.path()).unwrap();
        let result = run_inner(true, true);
        std::env::set_current_dir(prev).unwrap();
        assert!(
            result.is_err(),
            "dry-run must propagate render errors as Err"
        );
    }

    /// REQ-131-06: `do:init --force` preserves identity fields from an existing
    /// `.do/app.yaml` with non-default name/region/repo/branch.
    #[test]
    fn do_init_preserves_identity() {
        let _guard = crate::commands::CWD_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let td = TempDir::new().unwrap();
        let root = td.path();

        // Seed a minimal project.
        write(
            root,
            "Cargo.toml",
            "[package]\nname = \"myapp\"\nversion = \"0.1.0\"\n\n[[bin]]\nname = \"myapp\"\npath = \"src/main.rs\"\n",
        );
        write(root, "src/main.rs", "fn main() {}\n");

        // Seed an existing .do/app.yaml with non-default identity fields.
        let existing_yaml = concat!(
            "# Generated by ferro do:init — edit to your needs\n",
            "name: custom-app-name\n",
            "region: nyc3\n",
            "\n",
            "services:\n",
            "  - name: web\n",
            "    dockerfile_path: Dockerfile\n",
            "    source_dir: /\n",
            "    github:\n",
            "      repo: myorg/my-repo\n",
            "      branch: production\n",
            "      deploy_on_push: true\n",
            "    http_port: 8080\n",
            "    instance_size_slug: apps-s-1vcpu-0.5gb\n",
            "    instance_count: 1\n",
            "\n",
            "# workers:\n",
            "envs:\n",
        );
        write(root, ".do/app.yaml", existing_yaml);

        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(root).unwrap();
        let result = run_inner(true, false);
        std::env::set_current_dir(prev).unwrap();

        assert!(result.is_ok(), "do:init --force should succeed: {result:?}");

        let written = fs::read_to_string(root.join(".do/app.yaml")).expect("app.yaml should exist");

        assert!(
            written.contains("name: custom-app-name"),
            "preserved name must survive --force\ngot:\n{written}"
        );
        assert!(
            written.contains("region: nyc3"),
            "preserved region must survive --force\ngot:\n{written}"
        );
        assert!(
            written.contains("repo: myorg/my-repo"),
            "preserved repo must survive --force\ngot:\n{written}"
        );
        assert!(
            written.contains("branch: production"),
            "preserved branch must survive --force\ngot:\n{written}"
        );
    }

    #[test]
    fn run_inner_succeeds_with_missing_env_example() {
        let _guard = crate::commands::CWD_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        // Phase 127 D-06: missing .env.example is a warning, not an error.
        let td = TempDir::new().unwrap();
        write(
            td.path(),
            "Cargo.toml",
            "[package]\nname = \"sample\"\nversion = \"0.1.0\"\n\n[[bin]]\nname = \"sample\"\npath = \"src/main.rs\"\n",
        );
        write(td.path(), "src/main.rs", "fn main() {}\n");
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(td.path()).unwrap();
        let result = run_inner(true, false);
        std::env::set_current_dir(prev).unwrap();
        assert!(
            result.is_ok(),
            "run_inner must succeed without .env.example: {result:?}"
        );
        let yaml = fs::read_to_string(td.path().join(".do/app.yaml")).expect("app.yaml written");
        assert!(!yaml.contains("- key: "), "envs block must be empty");
        assert!(yaml.contains("envs:"));
    }
}
