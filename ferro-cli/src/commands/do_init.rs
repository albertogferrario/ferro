//! do:init command — generate DigitalOcean App Platform spec.

use console::style;
use std::fs;

use crate::deploy::env_example::{parse_env_example, EnvEntry};
use crate::project::{find_project_root, package_name, read_bins};
use crate::templates::do_spec::{render_app_yaml, AppYamlContext};

pub fn run(repo: &str, region: &str, force: bool, ferro_ref: &str) {
    if !is_valid_repo(repo) {
        eprintln!(
            "{} --repo must be in the form owner/repo (got: {})",
            style("Error:").red().bold(),
            repo
        );
        std::process::exit(1);
    }

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

    let do_dir = root.join(".do");
    let app_yaml = do_dir.join("app.yaml");
    if app_yaml.exists() && !force {
        eprintln!(
            "{} {} already exists (use --force)",
            style("Error:").red().bold(),
            app_yaml.display()
        );
        std::process::exit(1);
    }

    if let Err(e) = fs::create_dir_all(&do_dir) {
        eprintln!(
            "{} Failed to create .do: {}",
            style("Error:").red().bold(),
            e
        );
        std::process::exit(1);
    }

    let pkg = package_name(&root);
    let bins = read_bins(&root);
    let env_entries: Vec<EnvEntry> = fs::read_to_string(root.join(".env.example"))
        .map(|s| parse_env_example(&s))
        .unwrap_or_default();

    let ctx = AppYamlContext {
        package_name: &pkg,
        github_repo: repo,
        region,
        bins: &bins,
        env_entries: &env_entries,
    };
    let yaml = render_app_yaml(&ctx);

    if let Err(e) = fs::write(&app_yaml, yaml) {
        eprintln!(
            "{} Failed to write app.yaml: {}",
            style("Error:").red().bold(),
            e
        );
        std::process::exit(1);
    }
    println!("{} Wrote {}", style("✓").green(), app_yaml.display());

    // Ensure Dockerfile + rewrite script exist.
    super::docker_init::generate(force, ferro_ref, &[]);

    println!();
    println!("{}", style("DO App Platform spec generated.").cyan().bold());
    println!("  Region: {region}");
    println!("  Repo:   {repo}");
    println!();
    println!("Next: push to GitHub and create the app on DO App Platform.");
}

pub(crate) fn is_valid_repo(s: &str) -> bool {
    let mut parts = s.split('/');
    let (Some(owner), Some(name), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    if owner.is_empty() || name.is_empty() {
        return false;
    }
    let valid = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.';
    owner.chars().all(valid) && name.chars().all(valid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn repo_validation_matrix() {
        assert!(is_valid_repo("owner/repo"));
        assert!(is_valid_repo("foo-bar/baz_qux"));
        assert!(is_valid_repo("a.b/c.d"));
        assert!(!is_valid_repo("nope"));
        assert!(!is_valid_repo("/foo"));
        assert!(!is_valid_repo("foo/"));
        assert!(!is_valid_repo("a/b/c"));
        assert!(!is_valid_repo("foo bar/baz"));
    }

    fn write_min_project(td: &TempDir, envs: &str) {
        fs::write(
            td.path().join("Cargo.toml"),
            "[package]\nname = \"sample\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        if !envs.is_empty() {
            fs::write(td.path().join(".env.example"), envs).unwrap();
        }
    }

    // Test helper that mirrors run() but takes an explicit root and skips
    // process::exit + the docker_init chain.
    fn run_for_test(root: &Path, repo: &str, region: &str, force: bool) {
        assert!(is_valid_repo(repo));
        let do_dir = root.join(".do");
        let app_yaml = do_dir.join("app.yaml");
        if app_yaml.exists() && !force {
            return;
        }
        fs::create_dir_all(&do_dir).unwrap();
        let pkg = package_name(root);
        let bins = read_bins(root);
        let env_entries: Vec<EnvEntry> = fs::read_to_string(root.join(".env.example"))
            .map(|s| parse_env_example(&s))
            .unwrap_or_default();
        let ctx = AppYamlContext {
            package_name: &pkg,
            github_repo: repo,
            region,
            bins: &bins,
            env_entries: &env_entries,
        };
        fs::write(&app_yaml, render_app_yaml(&ctx)).unwrap();
    }

    #[test]
    fn writes_app_yaml_with_region_and_envs() {
        let td = TempDir::new().unwrap();
        write_min_project(
            &td,
            "APP_URL=https://sample.io\nDATABASE_URL=postgres://x\n",
        );
        run_for_test(td.path(), "owner/repo", "nyc", true);
        let yaml = fs::read_to_string(td.path().join(".do/app.yaml")).unwrap();
        assert!(yaml.contains("region: nyc"));
        assert!(yaml.contains("APP_URL"));
        assert!(yaml.contains("${db.DATABASE_URL}"));
        assert!(yaml.contains("databases:"));
    }
}
