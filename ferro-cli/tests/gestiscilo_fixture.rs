//! Phase 131 Plan 01 — Wave 0 gap report.
//! Phase 131 Plan 02 — Identity preservation + docker_template_drift check.
//!
//! Byte-identical regeneration tests + regression tests for the gestiscilo
//! reference fixture (commit 6f6d397).
//!
//! Run all tests:   `cargo test -p ferro-cli --test gestiscilo_fixture`
//! Run with output: `cargo test -p ferro-cli --test gestiscilo_fixture -- --nocapture`

use std::fs;
use std::path::PathBuf;

use ferro_cli::deploy::app_yaml_existing::parse_existing;
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

    // Use the known gestiscilo identity — in a real run these come from
    // `git remote get-url origin` and the existing .do/app.yaml.
    let repo = "gestiscilo-it/app".to_string();

    let _ = metadata; // metadata used above for web_bin via deploy_metadata
    AppYamlContext {
        name: app_name,
        repo,
        web_bin,
        workers,
        env_lines,
        preserved_name: None,
        preserved_region: None,
        preserved_github_repo: None,
        preserved_github_branch: None,
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
// Byte-identical regeneration tests (131-02: fixtures updated to scaffolder header)
// ============================================================================

#[test]
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

// ============================================================================
// Phase 131 Plan 02 — Identity preservation + drift check integration tests
// ============================================================================

/// REQ-131-06: `parse_existing` reads the gestiscilo fixture app.yaml and
/// returns the correct identity fields.
#[test]
fn parse_existing_reads_gestiscilo_fixture() {
    let path = fixture_dir().join("app.yaml");
    let identity = parse_existing(&path).expect("should return Some for present file");
    // The fixture is the scaffolder-output form (131-02 updated header); identity
    // fields come from the YAML content lines, not the header comment.
    assert_eq!(identity.name.as_deref(), Some("gestiscilo"));
    assert_eq!(identity.region.as_deref(), Some("fra1"));
    assert_eq!(identity.repo.as_deref(), Some("gestiscilo-it/app"));
    assert_eq!(identity.branch.as_deref(), Some("main"));
}

/// `parse_existing` returns None when the file does not exist.
#[test]
fn parse_existing_returns_none_for_missing_file() {
    let result = parse_existing(std::path::Path::new("/tmp/nonexistent-ferro-test/app.yaml"));
    assert!(result.is_none());
}

/// REQ-131-06: `render_app_yaml` uses preserved identity fields over defaults.
///
/// This tests the render path directly; the `do_init_preserves_identity`
/// unit test in `commands/do_init.rs` covers the full round-trip.
#[test]
fn render_app_yaml_uses_preserved_identity_over_defaults() {
    use ferro_cli::deploy::env_production::EnvLine;

    let ctx = AppYamlContext {
        name: "derived-name".to_string(),
        repo: "derived/repo".to_string(),
        web_bin: "derived-name".to_string(),
        workers: Vec::new(),
        env_lines: Some(vec![EnvLine::Key("APP_NAME".to_string())]),
        preserved_name: Some("custom-app-name".to_string()),
        preserved_region: Some("nyc3".to_string()),
        preserved_github_repo: Some("myorg/my-repo".to_string()),
        preserved_github_branch: Some("production".to_string()),
    };

    let out = render_app_yaml(&ctx);

    assert!(
        out.contains("name: custom-app-name"),
        "preserved name must be used\ngot:\n{out}"
    );
    assert!(
        out.contains("region: nyc3"),
        "preserved region must be used\ngot:\n{out}"
    );
    assert!(
        out.contains("repo: myorg/my-repo"),
        "preserved repo must be used\ngot:\n{out}"
    );
    assert!(
        out.contains("branch: production"),
        "preserved branch must be used\ngot:\n{out}"
    );
    // Derived values must NOT appear.
    assert!(
        !out.contains("derived-name"),
        "derived name must be suppressed\ngot:\n{out}"
    );
    assert!(
        !out.contains("fra1"),
        "default region must be suppressed\ngot:\n{out}"
    );
}
