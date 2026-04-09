//! Phase 131 Plan 01 — Wave 0 gap report.
//!
//! Byte-identical regeneration tests + regression tests for the gestiscilo
//! reference fixture (commit 6f6d397). The two byte-identical tests are
//! `#[ignore]`'d — they surface the delta between the scaffolder and the
//! hand-maintained files. All other tests must pass today.
//!
//! Run non-ignored tests:  `cargo test -p ferro-cli --test gestiscilo_fixture`
//! Run gap report:         `cargo test -p ferro-cli --test gestiscilo_fixture -- --ignored --nocapture`

use std::fs;
use std::path::PathBuf;

use ferro_cli::deploy::bin_detect::detect_web_bin;
use ferro_cli::deploy::env_production::parse_env_example_structured;
use ferro_cli::project::read_deploy_metadata;
use ferro_cli::templates::do_::{
    is_test_like_bin, render_app_yaml, sanitize_do_app_name, AppYamlContext,
};
use ferro_cli::templates::docker::{
    read_bins, read_rust_channel, render_dockerfile, DockerContext,
};

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gestiscilo")
}

/// Reconstruct the `DockerContext` from the fixture files the same way
/// `commands::docker_init` does — no hardcoding.
fn build_docker_context() -> DockerContext {
    let root = fixture_dir();
    let metadata = read_deploy_metadata(&root).expect("read_deploy_metadata");
    let bins = read_bins(&root).expect("read_bins");
    let web_bin = detect_web_bin(&root).expect("detect_web_bin");
    let rust_channel = read_rust_channel(&root);
    let copy_dirs_present: Vec<String> = metadata
        .copy_dirs
        .iter()
        .filter(|d| root.join(d).exists())
        .cloned()
        .collect();
    DockerContext {
        rust_channel,
        has_frontend: root.join("frontend/package.json").is_file(),
        bins,
        web_bin,
        copy_dirs_present,
        runtime_apt: metadata.runtime_apt,
    }
}

/// Reconstruct the `AppYamlContext` from the fixture files the same way
/// `commands::do_init` does — no hardcoding.
fn build_app_yaml_context() -> AppYamlContext {
    let root = fixture_dir();
    let metadata = read_deploy_metadata(&root).expect("read_deploy_metadata");
    let bins = read_bins(&root).expect("read_bins");
    let web_bin = detect_web_bin(&root).expect("detect_web_bin");
    let pkg_name = ferro_cli::project::package_name(&root);
    let app_name = sanitize_do_app_name(&pkg_name);

    let workers: Vec<String> = bins
        .iter()
        .filter(|b| *b != &web_bin && !is_test_like_bin(b))
        .cloned()
        .collect();

    // Resolve env_lines from the fixture .env.example (same as do_init D-06).
    let env_example_path = root.join(".env.example");
    let env_lines = if env_example_path.is_file() {
        let content = fs::read_to_string(&env_example_path).expect("read .env.example");
        Some(parse_env_example_structured(&content))
    } else {
        None
    };

    // Use the known gestiscilo repo binding — in a real run this comes from
    // `git remote get-url origin`.
    let repo = "gestiscilo-it/app".to_string();

    let _ = metadata; // metadata used above for web_bin via deploy_metadata
    AppYamlContext {
        name: app_name,
        repo,
        web_bin,
        workers,
        env_lines,
    }
}

/// Small helper: produce a line-by-line diff report (no external crate needed).
fn line_diff(label: &str, expected: &str, got: &str) -> String {
    let exp_lines: Vec<&str> = expected.lines().collect();
    let got_lines: Vec<&str> = got.lines().collect();
    let max = exp_lines.len().max(got_lines.len());
    let mut report = format!("\n=== {label} diff (expected vs got) ===\n");
    let mut diffs = 0usize;
    for i in 0..max {
        let e = exp_lines.get(i).copied().unwrap_or("<missing>");
        let g = got_lines.get(i).copied().unwrap_or("<missing>");
        if e != g {
            report.push_str(&format!("  line {:>3}: - {e}\n", i + 1));
            report.push_str(&format!("             + {g}\n"));
            diffs += 1;
        }
    }
    if diffs == 0 {
        report.push_str("  (trailing-newline-normalized: no diffs)\n");
    } else {
        report.push_str(&format!("\n  total differing lines: {diffs}\n"));
        report.push_str(&format!(
            "  expected {} lines, got {} lines\n",
            exp_lines.len(),
            got_lines.len()
        ));
    }
    report
}

// ============================================================================
// Byte-identical regeneration tests (EXPECTED TO FAIL pending 131-02)
// ============================================================================

#[test]
#[ignore = "Wave 0 diff — unignored once 131-02 lands identity preservation and any Dockerfile deltas are resolved"]
fn dockerfile_matches_gestiscilo_6f6d397() {
    let fixture = fs::read_to_string(fixture_dir().join("Dockerfile")).expect("fixture Dockerfile");
    let ctx = build_docker_context();
    let rendered = render_dockerfile(&ctx);

    let fixture_norm = fixture.trim_end_matches('\n');
    let rendered_norm = rendered.trim_end_matches('\n');

    if fixture_norm != rendered_norm {
        let report = line_diff("Dockerfile", fixture_norm, rendered_norm);
        println!("{report}");
        panic!("Dockerfile does not match gestiscilo 6f6d397{report}");
    }
}

#[test]
#[ignore = "Wave 0 diff — unignored once 131-02 lands identity preservation and any Dockerfile deltas are resolved"]
fn app_yaml_matches_gestiscilo_6f6d397() {
    let fixture = fs::read_to_string(fixture_dir().join("app.yaml")).expect("fixture app.yaml");
    let ctx = build_app_yaml_context();
    let rendered = render_app_yaml(&ctx);

    let fixture_norm = fixture.trim_end_matches('\n');
    let rendered_norm = rendered.trim_end_matches('\n');

    if fixture_norm != rendered_norm {
        let report = line_diff("app.yaml", fixture_norm, rendered_norm);
        println!("{report}");
        panic!("app.yaml does not match gestiscilo 6f6d397{report}");
    }
}

// ============================================================================
// Regression tests (MUST PASS today — validate "already implemented" claims)
// ============================================================================

/// REQ-131-08: The rendered app.yaml must never emit `health_check:`.
/// Research confirmed the current template does not emit it; this test locks
/// that property in.
#[test]
fn app_yaml_never_emits_health_check() {
    let ctx = build_app_yaml_context();
    let rendered = render_app_yaml(&ctx);
    assert!(
        !rendered.contains("health_check:"),
        "app.yaml must not emit health_check:, got:\n{rendered}"
    );
}

/// REQ-131-09: No Node.js frontend build stage when fixture has no
/// `frontend/package.json` (gestiscilo is server-rendered).
#[test]
fn dockerfile_has_no_frontend_builder_when_frontend_absent() {
    assert!(
        !fixture_dir().join("frontend/package.json").exists(),
        "fixture contract: no frontend/package.json"
    );
    let ctx = build_docker_context();
    assert!(
        !ctx.has_frontend,
        "context must have has_frontend=false for this fixture"
    );
    let rendered = render_dockerfile(&ctx);
    assert!(
        !rendered.contains("node:"),
        "Dockerfile must not contain node: stage when no frontend, got:\n{rendered}"
    );
    assert!(
        !rendered.contains("FROM node"),
        "Dockerfile must not contain FROM node stage, got:\n{rendered}"
    );
    assert!(
        !rendered.contains("frontend-builder"),
        "Dockerfile must not contain frontend-builder stage, got:\n{rendered}"
    );
}

/// REQ-131-01: Every bin in the fixture Cargo.toml appears in the rendered
/// Dockerfile as `COPY --from=backend-builder /app/target/release/<bin>`.
#[test]
fn dockerfile_covers_every_bin() {
    let root = fixture_dir();
    let bins = read_bins(&root).expect("read_bins");
    assert!(
        !bins.is_empty(),
        "fixture must declare at least one [[bin]]"
    );

    let ctx = build_docker_context();
    let rendered = render_dockerfile(&ctx);

    for bin in &bins {
        let expected =
            format!("COPY --from=backend-builder /app/target/release/{bin} /usr/local/bin/{bin}");
        assert!(
            rendered.contains(&expected),
            "Dockerfile missing COPY line for bin '{bin}'\nexpected to find: {expected}\nDockerfile:\n{rendered}"
        );
    }
}

/// REQ-131-02: Each non-web bin appears under `workers:` in the rendered
/// app.yaml.
#[test]
fn app_yaml_workers_from_non_web_bins() {
    let root = fixture_dir();
    let bins = read_bins(&root).expect("read_bins");
    let web_bin = detect_web_bin(&root).expect("detect_web_bin");
    let worker_bins: Vec<&String> = bins
        .iter()
        .filter(|b| *b != &web_bin && !is_test_like_bin(b))
        .collect();

    assert!(
        !worker_bins.is_empty(),
        "fixture must have at least one worker bin (screenshot-worker)"
    );

    let ctx = build_app_yaml_context();
    let rendered = render_app_yaml(&ctx);

    assert!(
        rendered.contains("workers:\n"),
        "app.yaml must contain workers: block\ngot:\n{rendered}"
    );
    for worker in &worker_bins {
        assert!(
            rendered.contains(&format!("- name: {worker}")),
            "workers block missing entry for '{worker}'\napp.yaml:\n{rendered}"
        );
        assert!(
            rendered.contains(&format!("run_command: /usr/local/bin/{worker}")),
            "workers block missing run_command for '{worker}'\napp.yaml:\n{rendered}"
        );
    }
}

/// REQ-131-03: Each `copy_dirs` entry that exists on disk appears as
/// `COPY {dir} {dir}` in the rendered Dockerfile.
#[test]
fn dockerfile_copy_dirs_emitted() {
    let root = fixture_dir();
    let metadata = read_deploy_metadata(&root).expect("read_deploy_metadata");

    let present_dirs: Vec<&String> = metadata
        .copy_dirs
        .iter()
        .filter(|d| root.join(d).exists())
        .collect();

    assert!(
        !present_dirs.is_empty(),
        "fixture must have at least one present copy_dir (themes/)"
    );

    let ctx = build_docker_context();
    let rendered = render_dockerfile(&ctx);

    for dir in &present_dirs {
        assert!(
            rendered.contains(&format!("COPY {dir} {dir}")),
            "Dockerfile missing 'COPY {dir} {dir}'\nDockerfile:\n{rendered}"
        );
    }
}

/// REQ-131-05: When `runtime_apt` is non-empty, the rendered Dockerfile must
/// contain the `# ferro:runtime-apt` marker and each package name.
#[test]
fn dockerfile_runtime_apt_layer() {
    let root = fixture_dir();
    let metadata = read_deploy_metadata(&root).expect("read_deploy_metadata");

    assert!(
        !metadata.runtime_apt.is_empty(),
        "fixture Cargo.toml must declare runtime_apt packages"
    );

    let ctx = build_docker_context();
    let rendered = render_dockerfile(&ctx);

    assert!(
        rendered.contains("# ferro:runtime-apt"),
        "Dockerfile missing # ferro:runtime-apt marker\nDockerfile:\n{rendered}"
    );

    for pkg in &metadata.runtime_apt {
        assert!(
            rendered.contains(pkg.as_str()),
            "Dockerfile missing runtime_apt package '{pkg}'\nDockerfile:\n{rendered}"
        );
    }
}
