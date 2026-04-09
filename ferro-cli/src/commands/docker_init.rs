//! `ferro docker:init` — generate a production-ready Dockerfile and static
//! `.dockerignore` from project metadata. Phase 122.2 §3.
//!
//! Phase 127 Plan 04: `--dry-run` renders every output file to memory and
//! prints it to stdout without touching the filesystem (D-17, D-18). The
//! "Next steps" footer (D-13, D-14) is printed after a successful
//! non-dry-run invocation and is suppressed in dry-run (D-16). Render
//! errors remain hard errors in both modes (D-19).
//!
//! Phase 130: the dual-manifest pattern is retired. Docker builds read the
//! project `Cargo.toml` directly; ferro developers who need to point at an
//! unpublished local checkout maintain an uncommitted `[patch.crates-io]`
//! block by hand.

use std::fs;
use std::path::{Path, PathBuf};

use crate::deploy::bin_detect::detect_web_bin;
use crate::project::{find_project_root, package_name, read_bins, read_deploy_metadata};
use crate::templates::docker::{
    dockerignore_template, read_rust_channel, render_dockerfile, DockerContext,
};

/// One rendered output file carried in memory between the render and persist
/// phases. Enables `--dry-run` to print everything without writing anything.
pub(crate) struct RenderedFile {
    pub relative_path: PathBuf,
    pub contents: String,
}

pub(crate) fn print_dry_run(files: &[RenderedFile]) {
    for f in files {
        println!("--- {} ---", f.relative_path.display());
        println!("{}", f.contents);
    }
}

/// Entry point used by `main.rs`. Returns a process-style exit: prints errors
/// to stderr and returns without panicking so clap stays happy.
pub fn run(force: bool) {
    run_with(force, None, false);
}

/// Full entry point supporting the `--ferro-version` override and `--dry-run`.
pub fn run_with(force: bool, ferro_version: Option<String>, dry_run: bool) {
    if let Err(e) = execute(force, ferro_version.as_deref(), dry_run) {
        eprintln!("docker:init failed: {e:#}");
    }
}

/// Library-level entry point used by integration tests. Returns `Result`
/// instead of printing to stderr, so tests can assert on failures.
pub fn execute(
    force: bool,
    _ferro_version_flag: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<()> {
    let root = find_project_root(None)
        .map_err(|e| anyhow::anyhow!("could not locate project Cargo.toml: {e}"))?;

    let metadata = read_deploy_metadata(&root)?;
    let rust_channel = read_rust_channel(&root);
    let bins: Vec<String> = read_bins(&root).into_iter().map(|b| b.name).collect();
    let has_frontend = root.join("frontend/package.json").is_file();
    let copy_dirs_present: Vec<String> = metadata
        .copy_dirs
        .iter()
        .filter(|d| root.join(d).exists())
        .cloned()
        .collect();

    let web_bin = detect_web_bin(&root)?;
    let ctx = DockerContext {
        rust_channel,
        has_frontend,
        bins,
        web_bin,
        copy_dirs_present,
        runtime_apt: metadata.runtime_apt.clone(),
    };

    // Render everything to memory first. Any render error is a hard error in
    // every mode, including --dry-run (D-19).
    let dockerfile = render_dockerfile(&ctx);

    let files: Vec<RenderedFile> = vec![
        RenderedFile {
            relative_path: "Dockerfile".into(),
            contents: dockerfile,
        },
        RenderedFile {
            relative_path: ".dockerignore".into(),
            contents: dockerignore_template().to_string(),
        },
    ];

    if dry_run {
        // D-16, D-17, D-18: print every rendered file, write nothing, suppress
        // the "Next steps" footer.
        print_dry_run(&files);
        return Ok(());
    }

    // Persist. Template outputs honor --force.
    for f in &files {
        let target = root.join(&f.relative_path);
        write_if_absent_or_force(&target, &f.contents, force)?;
    }

    println!(
        "docker:init wrote Dockerfile and .dockerignore in {}",
        root.display()
    );

    // D-13, D-14: cargo-style "Next steps" footer.
    let pkg = package_name(&root);
    print!("{}", docker_init_footer(&pkg));
    Ok(())
}

/// Build the cargo-style "Next steps" footer for `docker:init`. Pure string —
/// the `print_*` wrapper exists so tests can assert on the content directly.
fn docker_init_footer(pkg: &str) -> String {
    format!(
        "\nNext steps:\n  docker build -t {pkg}:test .\n  docker run --rm -p 8080:8080 --env-file .env.production {pkg}:test\n"
    )
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

#[cfg(test)]
mod footer_tests {
    use super::*;

    #[test]
    fn docker_init_footer_contents() {
        let s = docker_init_footer("myapp");
        assert!(s.contains("docker build"));
        assert!(s.contains("docker run"));
        assert!(s.contains("--env-file .env.production"));
        assert!(s.contains("myapp:test"));
    }

    #[test]
    fn docker_init_footer_line_count() {
        let s = docker_init_footer("app");
        let n = s.lines().filter(|l| !l.trim().is_empty()).count();
        assert!((3..=5).contains(&n), "footer has {n} non-empty lines: {s}");
        assert!(s.is_ascii(), "footer must be ASCII-only");
    }
}
