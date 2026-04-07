//! `ferro docker:init` — generate a production-ready Dockerfile, static
//! .dockerignore, and Cargo.docker.toml from project metadata. Phase 122.2 §3.

use std::fs;
use std::path::Path;

use crate::deploy::rewrite_ferro_version::rewrite_cargo_docker_toml;
use crate::project::{find_project_root, read_deploy_metadata};
use crate::templates::docker::{
    dockerignore_template, read_bins, read_rust_channel, render_dockerfile, DockerContext,
};

/// Entry point used by `main.rs`. Returns a process-style exit: prints errors
/// to stderr and returns without panicking so clap stays happy.
pub fn run(force: bool) {
    run_with(force, None);
}

/// Full entry point supporting the `--ferro-version` override.
pub fn run_with(force: bool, ferro_version: Option<String>) {
    if let Err(e) = execute(force, ferro_version.as_deref()) {
        eprintln!("docker:init failed: {e:#}");
    }
}

fn execute(force: bool, ferro_version_flag: Option<&str>) -> anyhow::Result<()> {
    let root = find_project_root(None)
        .map_err(|e| anyhow::anyhow!("could not locate project Cargo.toml: {e}"))?;

    let metadata = read_deploy_metadata(&root)?;
    let rust_channel = read_rust_channel(&root);
    let bins = read_bins(&root)?;
    let has_frontend = root.join("frontend/package.json").is_file();
    let copy_dirs_present: Vec<String> = metadata
        .copy_dirs
        .iter()
        .filter(|d| root.join(d).exists())
        .cloned()
        .collect();

    let ctx = DockerContext {
        rust_channel,
        has_frontend,
        bins,
        copy_dirs_present,
        runtime_apt: metadata.runtime_apt.clone(),
    };
    let dockerfile = render_dockerfile(&ctx);

    write_if_absent_or_force(&root.join("Dockerfile"), &dockerfile, force)?;
    write_if_absent_or_force(&root.join(".dockerignore"), dockerignore_template(), force)?;

    // Version override precedence: --ferro-version flag → metadata.ferro_version
    // → path-dep workspace version → "*".
    let override_str = ferro_version_flag.or(metadata.ferro_version.as_deref());
    rewrite_cargo_docker_toml(&root, override_str)?;

    println!(
        "docker:init wrote Dockerfile, .dockerignore, Cargo.docker.toml in {}",
        root.display()
    );
    Ok(())
}

fn write_if_absent_or_force(path: &Path, content: &str, force: bool) -> anyhow::Result<()> {
    if path.exists() && !force {
        println!(
            "skip {}: already exists (use --force to overwrite)",
            path.display()
        );
        return Ok(());
    }
    fs::write(path, content)
        .map_err(|e| anyhow::anyhow!("failed to write {}: {e}", path.display()))?;
    Ok(())
}
