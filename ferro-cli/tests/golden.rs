//! Golden-file integration tests for Phase 122 deploy scaffolding.
//!
//! Fixture layout:
//!   tests/fixtures/<name>/
//!     Cargo.toml
//!     .env.example
//!     [frontend/package.json]
//!     expected/
//!       Dockerfile
//!       app.yaml
//!
//! Regenerate golden files with:
//!   UPDATE_GOLDEN=1 cargo test -p ferro-cli --test golden

use ferro_cli::deploy::env_example::parse_env_example;
use ferro_cli::project::{
    detect_dirs, package_name, read_bins, read_workspace_members, resolve_rust_base_image,
};
use ferro_cli::templates::do_spec::{render_app_yaml, AppYamlContext};
use ferro_cli::templates::{render_dockerfile, DockerfileContext};
use std::fs;
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn update_mode() -> bool {
    std::env::var("UPDATE_GOLDEN").ok().as_deref() == Some("1")
}

struct FixtureCase {
    name: &'static str,
    repo: &'static str,
    region: &'static str,
    ferro_ref: &'static str,
    runtime_deps: &'static [&'static str],
}

const CASES: &[FixtureCase] = &[
    FixtureCase {
        name: "gestiscilo",
        repo: "gestiscilo-it/app",
        region: "fra1",
        ferro_ref: "main",
        runtime_deps: &["chromium", "fonts-liberation"],
    },
    FixtureCase {
        name: "mkmenu",
        repo: "gestiscilo-it/mkmenu",
        region: "fra1",
        ferro_ref: "main",
        runtime_deps: &[],
    },
];

fn render_for(case: &FixtureCase) -> (String, String) {
    let root = fixtures_dir().join(case.name);
    let pkg = package_name(&root);
    let bins = read_bins(&root);
    let workspace = read_workspace_members(&root);
    let base_image = resolve_rust_base_image(&root);
    let dirs = detect_dirs(&root);
    let runtime_deps: Vec<String> = case.runtime_deps.iter().map(|s| s.to_string()).collect();

    let dctx = DockerfileContext {
        package_name: &pkg,
        bins: &bins,
        dirs,
        runtime_deps: &runtime_deps,
        rust_base_image: &base_image,
        workspace_members: &workspace,
        ferro_ref: case.ferro_ref,
    };
    let dockerfile = render_dockerfile(&dctx);

    let env_content = fs::read_to_string(root.join(".env.example")).unwrap_or_default();
    let env_entries = parse_env_example(&env_content);
    let actx = AppYamlContext {
        package_name: &pkg,
        github_repo: case.repo,
        region: case.region,
        bins: &bins,
        env_entries: &env_entries,
    };
    let app_yaml = render_app_yaml(&actx);
    (dockerfile, app_yaml)
}

fn check_or_update(path: &Path, actual: &str, case_name: &str, kind: &str) {
    if update_mode() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, actual).unwrap();
        return;
    }
    let expected = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => panic!(
            "Missing golden file for {case_name} {kind} at {}. \
             Run `UPDATE_GOLDEN=1 cargo test -p ferro-cli --test golden` to create it.",
            path.display()
        ),
    };
    if expected != actual {
        panic!(
            "Golden mismatch for {case_name} {kind}.\n\
             Run `UPDATE_GOLDEN=1 cargo test -p ferro-cli --test golden` if the change is intentional.\n\
             --- expected ---\n{expected}\n--- actual ---\n{actual}"
        );
    }
}

#[test]
fn golden_gestiscilo_and_mkmenu() {
    for case in CASES {
        let (dockerfile, app_yaml) = render_for(case);
        let expected_dir = fixtures_dir().join(case.name).join("expected");
        check_or_update(
            &expected_dir.join("Dockerfile"),
            &dockerfile,
            case.name,
            "Dockerfile",
        );
        check_or_update(
            &expected_dir.join("app.yaml"),
            &app_yaml,
            case.name,
            "app.yaml",
        );
    }

    // Per-case content invariants — must hold regardless of golden regeneration.
    let (ges_df, ges_yaml) = render_for(&CASES[0]);
    assert!(ges_df.contains("--bin gestiscilo"));
    assert!(ges_df.contains("--bin screenshot-worker"));
    assert!(ges_df.contains("chromium"));
    assert!(ges_df.contains("ferro:runtime-deps"));
    assert!(ges_df.contains("COPY crates/core/"));
    assert!(ges_df.contains("COPY migration/"));
    assert!(ges_yaml.contains("region: fra1"));
    assert!(ges_yaml.contains("databases:"));
    assert!(ges_yaml.contains("workers:"));
    assert!(ges_yaml.contains("screenshot-worker"));
    assert!(ges_yaml.contains("STRIPE_SECRET_KEY"));
    assert!(ges_yaml.contains("type: SECRET"));

    let (mk_df, mk_yaml) = render_for(&CASES[1]);
    assert!(mk_df.contains("FROM node:22-slim AS frontend-builder"));
    assert!(mk_df.contains("--bin mkmenu"));
    assert!(!mk_df.contains("--bin screenshot-worker"));
    assert!(mk_yaml.contains("region: fra1"));
    assert!(mk_yaml.contains("databases:"));
    assert!(!mk_yaml.contains("workers:"));
}
