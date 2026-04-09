// ============================================================================
// Docker Templates
// ============================================================================
//
// Phase 122.2 §2: new Dockerfile renderer driven by metadata + rust-toolchain
// + [[bin]] enumeration. No GITHUB_TOKEN, no shell scripts, no workspace
// member walking. The static .dockerignore template is reused for the
// `docker:init` and `new` scaffolds.
//
// Phase 127 Plan 02: the ENTRYPOINT block is composed from a caller-resolved
// `web_bin` (see `crate::deploy::bin_detect::detect_web_bin`) — the renderer
// stays pure and does no I/O. `detect_web_bin` is called at the boundary in
// `commands::docker_init` so tests can exercise the renderer with any
// arbitrary web_bin without touching the filesystem.

use std::fs;
use std::path::Path;

use toml::Value;

const DOCKERIGNORE_TPL: &str = include_str!("files/docker/dockerignore.tpl");
const DOCKERFILE_TPL: &str = include_str!("files/docker/Dockerfile.tpl");

/// Static `.dockerignore` body. Phase 122.2 Plan 06 owns the canonical content.
pub fn dockerignore_template() -> &'static str {
    DOCKERIGNORE_TPL
}

/// Inputs the Dockerfile renderer needs. All fields come from the project
/// (rust-toolchain.toml, Cargo.toml, on-disk dirs, deploy metadata) and are
/// fully resolved by the caller — `render_dockerfile` itself does no I/O.
#[derive(Debug, Clone)]
pub struct DockerContext {
    /// Rust release channel — e.g. "stable", "1.90.0".
    pub rust_channel: String,
    /// Whether `frontend/package.json` exists in the project root.
    pub has_frontend: bool,
    /// All `[[bin]]` names declared in the project Cargo.toml.
    pub bins: Vec<String>,
    /// Resolved web bin name (D-02). Used for the runtime ENTRYPOINT.
    pub web_bin: String,
    /// `metadata.copy_dirs` filtered down to dirs that actually exist.
    pub copy_dirs_present: Vec<String>,
    /// Verbatim `metadata.runtime_apt`.
    pub runtime_apt: Vec<String>,
}

/// Render a Dockerfile from the supplied context. Pure string substitution.
pub fn render_dockerfile(ctx: &DockerContext) -> String {
    let frontend_stage = if ctx.has_frontend {
        FRONTEND_STAGE_BODY.to_string()
    } else {
        String::new()
    };

    let bin_copies = ctx
        .bins
        .iter()
        .map(|b| format!("COPY --from=backend-builder /app/target/release/{b} /usr/local/bin/{b}"))
        .collect::<Vec<_>>()
        .join("\n");

    let copy_dirs = ctx
        .copy_dirs_present
        .iter()
        .map(|d| format!("COPY {d} {d}"))
        .collect::<Vec<_>>()
        .join("\n");

    let runtime_apt = if ctx.runtime_apt.is_empty() {
        String::new()
    } else {
        let pkgs = ctx.runtime_apt.join(" ");
        format!(
            "# ferro:runtime-apt\nRUN apt-get update \\\n    && apt-get install -y --no-install-recommends {pkgs} \\\n    && rm -rf /var/lib/apt/lists/*"
        )
    };

    // Docker Hub publishes `rust:slim-bookworm` (unversioned, tracks stable)
    // and `rust:<version>-slim-bookworm` (e.g. `rust:1.90.0-slim-bookworm`).
    // There is no `rust:stable-slim-bookworm` tag, so when the channel is the
    // generic "stable" we drop the prefix instead of constructing a phantom
    // tag that fails to pull.
    let rust_image_tag = if ctx.rust_channel == "stable" {
        "slim-bookworm".to_string()
    } else {
        format!("{}-slim-bookworm", ctx.rust_channel)
    };

    let entrypoint_block = format!(
        "ENTRYPOINT [\"/usr/local/bin/{}\"]\nCMD [\"serve\"]",
        ctx.web_bin
    );

    let rendered = DOCKERFILE_TPL
        .replace("{{FRONTEND_STAGE}}", &frontend_stage)
        .replace("{{RUST_IMAGE_TAG}}", &rust_image_tag)
        .replace("{{ENTRYPOINT}}", &entrypoint_block)
        .replace("{{BIN_COPIES}}", &bin_copies)
        .replace("{{COPY_DIRS}}", &copy_dirs)
        .replace("{{RUNTIME_APT}}", &runtime_apt);

    debug_assert!(
        !rendered.contains("{{"),
        "unresolved template token in rendered Dockerfile:\n{rendered}"
    );
    rendered
}

const FRONTEND_STAGE_BODY: &str = r#"
FROM node:20-bookworm-slim AS frontend-builder
WORKDIR /frontend
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci || npm install
COPY frontend/ ./
RUN npm run build
"#;

/// Read `[toolchain] channel` from `<root>/rust-toolchain.toml`. Defaults to
/// `"stable"` when the file is missing or unparseable.
pub fn read_rust_channel(project_root: &Path) -> String {
    let path = project_root.join("rust-toolchain.toml");
    let Ok(content) = fs::read_to_string(&path) else {
        return "stable".to_string();
    };
    let Ok(parsed) = content.parse::<Value>() else {
        return "stable".to_string();
    };
    parsed
        .get("toolchain")
        .and_then(|t| t.get("channel"))
        .and_then(|c| c.as_str())
        .map(String::from)
        .unwrap_or_else(|| "stable".to_string())
}

/// Parse `<root>/Cargo.toml` and return every `[[bin]] name`. Falls back to
/// `[package] name` if no `[[bin]]` table is declared. Errors when Cargo.toml
/// is unreadable or unparseable.
pub fn read_bins(project_root: &Path) -> anyhow::Result<Vec<String>> {
    let path = project_root.join("Cargo.toml");
    let content = fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()))?;
    let parsed: Value = content
        .parse()
        .map_err(|e| anyhow::anyhow!("failed to parse {}: {e}", path.display()))?;

    let from_bins: Vec<String> = parsed
        .get("bin")
        .and_then(|b| b.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|entry| entry.get("name").and_then(|n| n.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    if !from_bins.is_empty() {
        return Ok(from_bins);
    }

    let pkg_name = parsed
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(String::from);

    Ok(pkg_name.into_iter().collect())
}

// ============================================================================
// docker-compose.yml renderer (unchanged from Phase 122)
// ============================================================================

pub fn docker_compose_template(
    project_name: &str,
    include_mailpit: bool,
    include_minio: bool,
) -> String {
    let mailpit_service = if include_mailpit {
        include_str!("files/docker/mailpit.service.tpl").replace("{project_name}", project_name)
    } else {
        String::new()
    };

    let minio_service = if include_minio {
        include_str!("files/docker/minio.service.tpl").replace("{project_name}", project_name)
    } else {
        String::new()
    };

    let additional_volumes = if include_minio {
        "\n  minio_data:".to_string()
    } else {
        String::new()
    };

    include_str!("files/docker/docker-compose.yml.tpl")
        .replace("{project_name}", project_name)
        .replace("{mailpit_service}", &mailpit_service)
        .replace("{minio_service}", &minio_service)
        .replace("{additional_volumes}", &additional_volumes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn ctx() -> DockerContext {
        DockerContext {
            rust_channel: "stable".to_string(),
            has_frontend: false,
            bins: vec!["app".to_string()],
            web_bin: "app".to_string(),
            copy_dirs_present: vec![],
            runtime_apt: vec![],
        }
    }

    #[test]
    fn frontend_stage_present_only_when_has_frontend() {
        let mut c = ctx();
        c.has_frontend = false;
        assert!(!render_dockerfile(&c).contains("frontend-builder"));
        c.has_frontend = true;
        assert!(render_dockerfile(&c).contains("frontend-builder"));
    }

    #[test]
    fn base_image_uses_rust_channel() {
        let mut c = ctx();
        c.rust_channel = "1.90.0".into();
        let out = render_dockerfile(&c);
        assert!(out.contains("rust:1.90.0-slim-bookworm"));
    }

    /// Regression: `rust:stable-slim-bookworm` does not exist on Docker Hub.
    /// When the channel is the generic "stable", we must emit the unversioned
    /// `rust:slim-bookworm` tag (which tracks stable) instead.
    #[test]
    fn stable_channel_emits_unversioned_slim_bookworm() {
        let mut c = ctx();
        c.rust_channel = "stable".into();
        let out = render_dockerfile(&c);
        assert!(out.contains("FROM rust:slim-bookworm AS chef"));
        assert!(!out.contains("rust:stable-slim-bookworm"));
    }

    #[test]
    fn multi_bin_emits_per_bin_copy_without_per_bin_build() {
        let mut c = ctx();
        c.bins = vec!["web".into(), "worker".into()];
        c.web_bin = "web".into();
        let out = render_dockerfile(&c);
        // D-10: no per-bin build invocations; the plain `cargo build --release`
        // already builds every declared [[bin]].
        assert_eq!(out.matches("cargo build --release --bin").count(), 0);
        assert!(
            out.contains("COPY --from=backend-builder /app/target/release/web /usr/local/bin/web")
        );
        assert!(out.contains(
            "COPY --from=backend-builder /app/target/release/worker /usr/local/bin/worker"
        ));
    }

    #[test]
    fn copy_dirs_emits_only_present_entries() {
        let mut c = ctx();
        c.copy_dirs_present = vec!["themes".into(), "migrations".into()];
        let out = render_dockerfile(&c);
        assert!(out.contains("COPY themes themes"));
        assert!(out.contains("COPY migrations migrations"));
        assert!(!out.contains("COPY public public"));
    }

    #[test]
    fn runtime_apt_empty_emits_no_block() {
        let c = ctx();
        let out = render_dockerfile(&c);
        assert!(!out.contains("ferro:runtime-apt"));
    }

    #[test]
    fn runtime_apt_nonempty_emits_marker_and_packages() {
        let mut c = ctx();
        c.runtime_apt = vec!["chromium".into(), "fonts-liberation".into()];
        let out = render_dockerfile(&c);
        assert!(out.contains("# ferro:runtime-apt"));
        assert!(out.contains("chromium fonts-liberation"));
    }

    #[test]
    fn dockerfile_copies_workspace_in_both_stages() {
        let out = render_dockerfile(&ctx());
        // `COPY . .` provides the workspace for planner + builder stages;
        // no dual-manifest overlay is copied on top.
        assert_eq!(out.matches("COPY . .").count(), 2);
        assert!(!out.contains("Cargo.docker"));
    }

    #[test]
    fn no_obsolete_phase_122_features() {
        let out = render_dockerfile(&ctx());
        for forbidden in ["GITHUB_TOKEN", "insteadOf", "rewrite-ferro-deps"] {
            assert!(
                !out.contains(forbidden),
                "found forbidden token: {forbidden}"
            );
        }
    }

    #[test]
    fn read_rust_channel_returns_default_when_missing() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(read_rust_channel(tmp.path()), "stable");
    }

    #[test]
    fn read_rust_channel_returns_channel_from_toolchain() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("rust-toolchain.toml"),
            "[toolchain]\nchannel = \"1.90.0\"\n",
        )
        .unwrap();
        assert_eq!(read_rust_channel(tmp.path()), "1.90.0");
    }

    #[test]
    fn read_bins_returns_bin_names() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            r#"
[package]
name = "demo"

[[bin]]
name = "web"

[[bin]]
name = "worker"
"#,
        )
        .unwrap();
        assert_eq!(read_bins(tmp.path()).unwrap(), vec!["web", "worker"]);
    }

    #[test]
    fn read_bins_falls_back_to_package_name() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"solo\"\n",
        )
        .unwrap();
        assert_eq!(read_bins(tmp.path()).unwrap(), vec!["solo"]);
    }

    /// Phase 122.2 §2: the renderer must mention the runtime-apt marker so the
    /// pattern is greppable for downstream tooling and tests.
    #[test]
    fn renderer_module_mentions_runtime_apt_marker() {
        // ferro:runtime-apt
        let mut c = ctx();
        c.runtime_apt = vec!["foo".into()];
        assert!(render_dockerfile(&c).contains("# ferro:runtime-apt"));
    }

    /// Generate docker-compose.yml for local development
    #[test]
    fn dockerignore_is_static_passthrough() {
        assert!(dockerignore_template().contains("database.db"));
    }
}

#[cfg(test)]
mod entrypoint_tests {
    use super::*;

    fn test_ctx_single(bin: &str) -> DockerContext {
        DockerContext {
            rust_channel: "stable".to_string(),
            has_frontend: false,
            bins: vec![bin.to_string()],
            web_bin: bin.to_string(),
            copy_dirs_present: vec![],
            runtime_apt: vec![],
        }
    }

    fn test_ctx_multi(web: &str, bins: &[&str]) -> DockerContext {
        DockerContext {
            rust_channel: "stable".to_string(),
            has_frontend: false,
            bins: bins.iter().map(|s| s.to_string()).collect(),
            web_bin: web.to_string(),
            copy_dirs_present: vec![],
            runtime_apt: vec![],
        }
    }

    #[test]
    fn entrypoint_emitted_for_single_bin() {
        let out = render_dockerfile(&test_ctx_single("myapp"));
        assert!(out.contains(r#"ENTRYPOINT ["/usr/local/bin/myapp"]"#));
        assert!(out.contains(r#"CMD ["serve"]"#));
    }

    #[test]
    fn entrypoint_emitted_for_multi_bin() {
        let out = render_dockerfile(&test_ctx_multi("api", &["api", "worker"]));
        assert!(out.contains(r#"ENTRYPOINT ["/usr/local/bin/api"]"#));
    }

    #[test]
    fn cmd_is_serve() {
        let out = render_dockerfile(&test_ctx_single("x"));
        assert!(out.contains(r#"CMD ["serve"]"#));
    }

    #[test]
    fn dockerfile_single_build_invocation() {
        let out = render_dockerfile(&test_ctx_single("x"));
        let build_releases = out.matches("cargo build --release").count();
        assert_eq!(
            build_releases, 1,
            "expected exactly one `cargo build --release`"
        );
        assert_eq!(out.matches("cargo build --release --bin").count(), 0);
    }

    #[test]
    fn no_unresolved_tokens_in_dockerfile() {
        let out = render_dockerfile(&test_ctx_single("x"));
        assert!(!out.contains("{{"), "unresolved token(s) in output:\n{out}");
    }

    #[test]
    fn dockerignore_whitelists_readme() {
        let out = dockerignore_template();
        assert!(out.contains("*.md"));
        assert!(out.contains("!README.md"));
        let idx_whitelist = out.find("!README.md").unwrap();
        let before = &out[..idx_whitelist];
        let last_line = before
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("");
        assert!(
            last_line.trim_start().starts_with('#'),
            "expected comment above !README.md, got: {last_line}"
        );
    }
}
