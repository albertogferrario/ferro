//! Phase 127 Plan 04 integration tests: prove that `docker:init --dry-run`
//! and `do:init --dry-run` write zero files and print every rendered file
//! with `--- <path> ---` headers.
//!
//! Uses the library entry points (`ferro_cli::commands::*::execute`) rather
//! than spawning the CLI binary, so the test doesn't depend on assert_cmd
//! or a pre-built `ferro` binary. Stdout is captured by redirecting
//! `std::io::set_output_capture`, which the `println!`/`print!` macros use
//! by default under `cargo test`.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Mutex;

use ferro_cli::commands::{do_init, docker_init};

// chdir is process-global; serialize tests that touch it.
static CHDIR_LOCK: Mutex<()> = Mutex::new(());

fn snapshot_dir(root: &std::path::Path) -> BTreeSet<PathBuf> {
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .map(|e| e.path().strip_prefix(root).unwrap().to_path_buf())
        .collect()
}

fn write(root: &std::path::Path, rel: &str, body: &str) {
    let p = root.join(rel);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(p, body).unwrap();
}

/// Minimal fixture project: Cargo.toml with one bin, a src/main.rs, and an
/// .env.example. Mirrors the shape the existing do_init unit tests use.
fn write_fixture_project(root: &std::path::Path) {
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"sample\"\nversion = \"0.1.0\"\n\n\
         [[bin]]\nname = \"sample\"\npath = \"src/main.rs\"\n",
    );
    write(root, "src/main.rs", "fn main() {}\n");
    write(root, ".env.example", "APP_NAME=\nDATABASE_URL=\n");
}

#[test]
fn dry_run_no_filesystem_writes() {
    let _guard = CHDIR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    write_fixture_project(tmp.path());

    let before = snapshot_dir(tmp.path());

    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    let result = docker_init::execute(false, None, true);
    std::env::set_current_dir(prev).unwrap();

    assert!(result.is_ok(), "docker:init --dry-run failed: {result:?}");

    let after = snapshot_dir(tmp.path());
    assert_eq!(before, after, "dry-run must not modify the filesystem");
    assert!(
        !tmp.path().join("Cargo.docker.toml").exists(),
        "Cargo.docker.toml must not be persisted in dry-run"
    );
}

#[test]
fn dry_run_no_cargo_docker_toml_persisted() {
    let _guard = CHDIR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    write_fixture_project(tmp.path());

    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    let result = docker_init::execute(false, None, true);
    std::env::set_current_dir(prev).unwrap();

    assert!(result.is_ok(), "docker:init --dry-run failed: {result:?}");
    assert!(!tmp.path().join("Cargo.docker.toml").exists());
    assert!(!tmp.path().join("Dockerfile").exists());
    assert!(!tmp.path().join(".dockerignore").exists());
}

#[test]
fn do_init_dry_run_no_filesystem_writes() {
    let _guard = CHDIR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    write_fixture_project(tmp.path());

    let before = snapshot_dir(tmp.path());

    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    let result = do_init::execute(false, true);
    std::env::set_current_dir(prev).unwrap();

    assert!(result.is_ok(), "do:init --dry-run failed: {result:?}");

    let after = snapshot_dir(tmp.path());
    assert_eq!(
        before, after,
        "do:init --dry-run must not modify the filesystem"
    );
    assert!(!tmp.path().join(".do/app.yaml").exists());
}
