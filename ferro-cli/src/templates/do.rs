//! Phase 122.2 §4: `.do/app.yaml` starter renderer.
//!
//! Pure string substitution over `files/do/app.yaml.tpl`. Caller is
//! responsible for I/O (reading Cargo.toml, .env.production, git remote).

// Renderer is pure and I/O-free. Web-bin resolution uses
// `crate::deploy::bin_detect::detect_web_bin` at the caller
// (`commands/do_init.rs`) so that the Dockerfile ENTRYPOINT and the DO web
// service stay in sync by construction (ferro 127, D-02).
#[cfg(test)]
use crate::deploy::env_production::parse_env_example_structured;
use crate::deploy::env_production::EnvLine;
use crate::deploy::secret_keys::is_secret_key;

const TEMPLATE: &str = include_str!("files/do/app.yaml.tpl");

/// Inputs for [`render_app_yaml`]. All fields are pre-resolved by the caller.
pub struct AppYamlContext {
    /// Sanitized DO app name (output of [`sanitize_do_app_name`]).
    pub name: String,
    /// `owner/repo` for `github.repo`, or fallback placeholder.
    pub repo: String,
    /// The bin matching the package name — kept for symmetry, not currently
    /// emitted (the `web` service is hardcoded in the template).
    pub web_bin: String,
    /// Non-web, non-test-like bins to emit as workers.
    pub workers: Vec<String>,
    /// Structured `.env.example` lines (keys + blank separators). `None` when
    /// `.env.example` is missing — the renderer emits an empty envs block and
    /// the caller logs a warning (D-06 graceful missing-file path).
    pub env_lines: Option<Vec<EnvLine>>,
}

/// Render `.do/app.yaml` from a fully-resolved context.
pub fn render_app_yaml(ctx: &AppYamlContext) -> String {
    let workers_block = render_workers_block(&ctx.workers);
    let envs_block = match &ctx.env_lines {
        Some(lines) => render_envs_block_from_lines(lines),
        None => String::new(),
    };
    let rendered = TEMPLATE
        .replace("{{NAME}}", &ctx.name)
        .replace("{{REPO}}", &ctx.repo)
        .replace("{{WORKERS_BLOCK}}", &workers_block)
        .replace("{{ENVS_BLOCK}}", &envs_block);
    debug_assert!(
        !rendered.contains("{{"),
        "unresolved template token in rendered .do/app.yaml"
    );
    rendered
}

/// Test helper: render an envs block from raw `.env.example` contents.
#[cfg(test)]
fn render_envs_block(env_example_contents: &str) -> String {
    let lines = parse_env_example_structured(env_example_contents);
    render_envs_block_from_lines(&lines)
}

fn render_envs_block_from_lines(lines: &[EnvLine]) -> String {
    let mut out = String::new();
    let indent = "  "; // child of `envs:` — matches template indent level
    for line in lines {
        match line {
            EnvLine::Key(key) => {
                out.push_str(indent);
                out.push_str("- key: ");
                out.push_str(key);
                out.push('\n');
                out.push_str(indent);
                out.push_str("  value: \"\"\n");
                if is_secret_key(key) {
                    out.push_str(indent);
                    out.push_str("  type: SECRET\n");
                    out.push_str(indent);
                    out.push_str("  scope: RUN_AND_BUILD_TIME\n");
                } else {
                    out.push_str(indent);
                    out.push_str("  scope: RUN_TIME\n");
                }
            }
            EnvLine::Blank => {
                out.push('\n');
            }
            EnvLine::Comment => {
                // Comments in .env.example are dropped; blank-line separators
                // carry the grouping into the generated YAML.
            }
        }
    }
    while out.ends_with('\n') {
        out.pop();
    }
    out
}

fn render_workers_block(workers: &[String]) -> String {
    if workers.is_empty() {
        // SCOPE §4: emit a commented example so users see the shape.
        return "\
# workers: (one entry per non-test/dev/debug [[bin]] other than the service)
# workers:
#   - name: example-worker
#     dockerfile_path: Dockerfile
#     source_dir: /
#     run_command: /usr/local/bin/example-worker
#     instance_size_slug: apps-s-1vcpu-0.5gb
#     instance_count: 1
"
        .to_string();
    }

    let mut out = String::from(
        "# workers: (one entry per non-test/dev/debug [[bin]] other than the service)\nworkers:\n",
    );
    for name in workers {
        out.push_str(&format!(
            "  - name: {name}\n    dockerfile_path: Dockerfile\n    source_dir: /\n    run_command: /usr/local/bin/{name}\n    instance_size_slug: apps-s-1vcpu-0.5gb\n    instance_count: 1\n"
        ));
    }
    out
}

/// Sanitize a package name to a DigitalOcean-compatible app name.
///
/// Kept because DO rejects otherwise-valid names with a cryptic remote
/// error; local sanitization gives a fast-fail with a clear reason. This is
/// the only sanitizer surviving Phase 122.2.
pub fn sanitize_do_app_name(package_name: &str) -> String {
    let lowered = package_name.to_lowercase();
    let mut out = String::with_capacity(lowered.len());
    for c in lowered.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            out.push(c);
        } else if c == '-' || c == '_' || c == ' ' {
            out.push('-');
        }
        // Other chars stripped.
    }
    // Collapse runs of dashes.
    let mut collapsed = String::with_capacity(out.len());
    let mut prev_dash = false;
    for c in out.chars() {
        if c == '-' {
            if !prev_dash {
                collapsed.push(c);
            }
            prev_dash = true;
        } else {
            collapsed.push(c);
            prev_dash = false;
        }
    }
    collapsed.trim_matches('-').to_string()
}

/// Parse a git remote URL (HTTPS or SSH form) to `"owner/repo"`. GitHub only.
pub fn parse_git_remote(remote_url: &str) -> Option<String> {
    let url = remote_url.trim();
    let tail = if let Some(rest) = url.strip_prefix("https://github.com/") {
        rest
    } else if let Some(rest) = url.strip_prefix("git@github.com:") {
        rest
    } else {
        return None;
    };
    let tail = tail.strip_suffix(".git").unwrap_or(tail);
    let mut parts = tail.splitn(3, '/');
    let owner = parts.next()?;
    let repo = parts.next()?;
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some(format!("{owner}/{repo}"))
}

/// Heuristic: filter test/dev/debug bins from the workers block.
///
/// This is the ONLY surviving heuristic after the Phase 122.2 simplification.
/// Kept because the alternative (explicit workers list in
/// `[package.metadata.ferro.deploy]`) would duplicate the `[[bin]]` entries
/// the user already wrote in Cargo.toml. See Phase 122.2 SCOPE §4.
pub fn is_test_like_bin(name: &str) -> bool {
    const PREFIXES: &[&str] = &["test_", "test-", "dev_", "dev-", "debug_", "debug-"];
    PREFIXES.iter().any(|p| name.starts_with(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(name: &str, repo: &str, workers: Vec<&str>, envs: Vec<&str>) -> AppYamlContext {
        AppYamlContext {
            name: name.to_string(),
            repo: repo.to_string(),
            web_bin: name.to_string(),
            workers: workers.into_iter().map(String::from).collect(),
            env_lines: Some(
                envs.into_iter()
                    .map(|k| EnvLine::Key(k.to_string()))
                    .collect(),
            ),
        }
    }

    pub(super) fn ctx_without_env(name: &str, repo: &str) -> AppYamlContext {
        AppYamlContext {
            name: name.to_string(),
            repo: repo.to_string(),
            web_bin: name.to_string(),
            workers: Vec::new(),
            env_lines: None,
        }
    }

    #[test]
    fn sanitize_simple_passthrough() {
        assert_eq!(sanitize_do_app_name("gestiscilo"), "gestiscilo");
    }

    #[test]
    fn sanitize_lowercases_and_replaces_underscores_and_spaces() {
        assert_eq!(sanitize_do_app_name("My_Cool App"), "my-cool-app");
    }

    #[test]
    fn sanitize_collapses_dashes() {
        assert_eq!(sanitize_do_app_name("foo__bar"), "foo-bar");
        assert_eq!(sanitize_do_app_name("foo---bar"), "foo-bar");
    }

    #[test]
    fn sanitize_strips_non_alphanum() {
        assert_eq!(sanitize_do_app_name("X!@#"), "x");
    }

    #[test]
    fn parse_git_remote_https_with_dot_git() {
        assert_eq!(
            parse_git_remote("https://github.com/owner/repo.git"),
            Some("owner/repo".to_string())
        );
    }

    #[test]
    fn parse_git_remote_https_no_dot_git() {
        assert_eq!(
            parse_git_remote("https://github.com/owner/repo"),
            Some("owner/repo".to_string())
        );
    }

    #[test]
    fn parse_git_remote_ssh_with_dot_git() {
        assert_eq!(
            parse_git_remote("git@github.com:owner/repo.git"),
            Some("owner/repo".to_string())
        );
    }

    #[test]
    fn parse_git_remote_ssh_no_dot_git() {
        assert_eq!(
            parse_git_remote("git@github.com:owner/repo"),
            Some("owner/repo".to_string())
        );
    }

    #[test]
    fn parse_git_remote_rejects_non_github() {
        assert_eq!(parse_git_remote("https://gitlab.com/x/y"), None);
    }

    #[test]
    fn is_test_like_bin_matches_prefixes() {
        for n in [
            "test_foo",
            "test-foo",
            "dev_foo",
            "dev-foo",
            "debug_foo",
            "debug-foo",
        ] {
            assert!(is_test_like_bin(n), "expected {n} to be test-like");
        }
    }

    #[test]
    fn is_test_like_bin_rejects_normal_names() {
        for n in ["web", "worker", "screenshot-worker", "api"] {
            assert!(!is_test_like_bin(n));
        }
    }

    #[test]
    fn render_app_yaml_contains_static_fields() {
        let c = ctx("myapp", "owner/repo", vec![], vec![]);
        let out = render_app_yaml(&c);
        assert!(out.starts_with("# Generated by ferro do:init — edit to your needs"));
        assert!(out.contains("name: myapp"));
        assert!(out.contains("region: fra1"));
        assert!(out.contains("repo: owner/repo"));
        assert!(out.contains("branch: main"));
        assert!(out.contains("services:"));
        assert!(out.contains("name: web"));
        assert!(out.contains("envs:"));
        assert!(!out.contains("databases:"));
    }

    #[test]
    fn render_app_yaml_with_empty_workers_emits_commented_example() {
        let c = ctx("a", "o/r", vec![], vec![]);
        let out = render_app_yaml(&c);
        assert!(out.contains("# workers:"));
        assert!(out.contains("# workers: (one entry per"));
    }

    #[test]
    fn render_app_yaml_emits_each_worker() {
        let c = ctx(
            "a",
            "o/r",
            vec!["screenshot-worker", "queue-worker"],
            vec![],
        );
        let out = render_app_yaml(&c);
        assert!(out.contains("workers:\n"));
        assert!(out.contains("- name: screenshot-worker"));
        assert!(out.contains("run_command: /usr/local/bin/screenshot-worker"));
        assert!(out.contains("- name: queue-worker"));
        assert!(out.contains("run_command: /usr/local/bin/queue-worker"));
    }

    #[test]
    fn render_app_yaml_emits_real_envs_entries() {
        let c = ctx(
            "a",
            "o/r",
            vec![],
            vec!["APP_ENV", "APP_URL", "DATABASE_URL"],
        );
        let out = render_app_yaml(&c);
        assert!(out.contains("- key: APP_ENV"));
        assert!(out.contains("- key: APP_URL"));
        assert!(out.contains("- key: DATABASE_URL"));
        assert!(!out.contains("# - APP_ENV"));
    }
}

#[cfg(test)]
mod envs_block_tests {
    use super::*;

    #[test]
    fn envs_block_from_env_example() {
        let src = "DATABASE_URL=\nSTRIPE_SECRET_KEY=\nAPP_NAME=\n";
        let out = render_envs_block(src);
        assert!(out.contains("- key: DATABASE_URL"));
        assert!(out.contains("- key: STRIPE_SECRET_KEY"));
        assert!(out.contains("- key: APP_NAME"));
    }

    #[test]
    fn secret_scope_and_type() {
        let src = "DATABASE_URL=\nSTRIPE_SECRET_KEY=\nAPP_NAME=\n";
        let out = render_envs_block(src);

        let stripe_idx = out.find("- key: STRIPE_SECRET_KEY").unwrap();
        let stripe_rest = &out[stripe_idx..];
        // Slice up to the next `- key: ` (or end of string).
        let stripe_end = stripe_rest[1..]
            .find("- key: ")
            .map(|i| i + 1)
            .unwrap_or(stripe_rest.len());
        let stripe_slice = &stripe_rest[..stripe_end];
        assert!(
            stripe_slice.contains("type: SECRET"),
            "STRIPE_SECRET_KEY must have type: SECRET, got: {stripe_slice}"
        );
        assert!(
            stripe_slice.contains("scope: RUN_AND_BUILD_TIME"),
            "STRIPE_SECRET_KEY must have scope: RUN_AND_BUILD_TIME"
        );

        let db_idx = out.find("- key: DATABASE_URL").unwrap();
        let db_rest = &out[db_idx..];
        let db_end = db_rest[1..]
            .find("- key: ")
            .map(|i| i + 1)
            .unwrap_or(db_rest.len());
        let db_slice = &db_rest[..db_end];
        assert!(
            !db_slice.contains("type: SECRET"),
            "DATABASE_URL must NOT have type: SECRET"
        );
        assert!(
            db_slice.contains("scope: RUN_TIME"),
            "DATABASE_URL must have scope: RUN_TIME"
        );
    }

    #[test]
    fn envs_preserve_source_order() {
        let src = "Z_NAME=\nA_NAME=\nM_NAME=\n";
        let out = render_envs_block(src);
        let z = out.find("Z_NAME").unwrap();
        let a = out.find("A_NAME").unwrap();
        let m = out.find("M_NAME").unwrap();
        assert!(z < a && a < m);
    }

    #[test]
    fn envs_preserve_blank_separators() {
        let src = "A_NAME=\n\nB_NAME=\n";
        let out = render_envs_block(src);
        let a = out.find("- key: A_NAME").unwrap();
        let b = out.find("- key: B_NAME").unwrap();
        assert!(
            out[a..b].contains("\n\n"),
            "expected blank line separator between A_NAME and B_NAME"
        );
    }
}

#[cfg(test)]
mod app_yaml_structure_tests {
    use super::tests::ctx_without_env;
    use super::*;

    #[test]
    fn web_service_has_no_run_command() {
        let c = AppYamlContext {
            name: "x".into(),
            repo: "o/r".into(),
            web_bin: "x".into(),
            workers: Vec::new(),
            env_lines: Some(vec![EnvLine::Key("APP_NAME".into())]),
        };
        let out = render_app_yaml(&c);
        // Locate the web service block (between `services:` and the workers block).
        let services_idx = out.find("services:").expect("services: block");
        let workers_idx = out[services_idx..]
            .find("# workers:")
            .map(|i| services_idx + i)
            .unwrap_or(out.len());
        let web_block = &out[services_idx..workers_idx];
        assert!(
            !web_block.contains("run_command:"),
            "web service must not set run_command (D-05), got: {web_block}"
        );
    }

    #[test]
    fn web_service_has_entrypoint_comment() {
        let c = AppYamlContext {
            name: "x".into(),
            repo: "o/r".into(),
            web_bin: "x".into(),
            workers: Vec::new(),
            env_lines: Some(vec![EnvLine::Key("APP_NAME".into())]),
        };
        let out = render_app_yaml(&c);
        assert!(
            out.contains("Dockerfile ENTRYPOINT"),
            "expected inline comment pointing at Dockerfile ENTRYPOINT"
        );
    }

    #[test]
    fn envs_missing_env_example_emits_empty_block() {
        let c = ctx_without_env("x", "o/r");
        let out = render_app_yaml(&c);
        assert!(
            !out.contains("- key: "),
            "expected empty envs block when .env.example missing"
        );
        // The `envs:` header itself still renders.
        assert!(out.contains("envs:"));
    }
}
