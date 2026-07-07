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
    ///
    /// Callers obtain bin entries via `crate::project::read_bins` (which
    /// returns `Vec<BinEntry>`) and convert to names at the call site:
    /// `bins: read_bins(root).into_iter().map(|b| b.name).collect()`.
    /// The renderer only needs names; keeping this field as `Vec<String>`
    /// preserves the "pure render" boundary (no project-module types leak into
    /// the template module).
    pub bins: Vec<String>,
    /// Resolved web bin name (D-02). Used for the runtime ENTRYPOINT.
    pub web_bin: String,
    /// `metadata.copy_dirs` filtered down to dirs that actually exist.
    pub copy_dirs_present: Vec<String>,
    /// Verbatim `metadata.runtime_apt`.
    pub runtime_apt: Vec<String>,
    /// Resolved ferro-cli version used by the `types-gen` Docker stage's
    /// `cargo install ferro-cli --version ...` pin. Caller-resolved via
    /// `resolve_ferro_version(root)`, which parses the project's Cargo.lock
    /// for the `ferro-rs` package and falls back to `env!("CARGO_PKG_VERSION")`
    /// when absent. Never empty.
    ///
    /// Phase 156 (D-16/D-21): closes the convention contradiction by ensuring
    /// `frontend/src/types/` is regenerated inside the Docker build context.
    pub ferro_version: String,
    /// Opt-in "fast build" mode. When `true`, the renderer emits a
    /// BuildKit-cache-mount + mold-linker optimized Dockerfile: cache mounts on
    /// `/usr/local/cargo/{registry,git}` and `/app/target`, `mold` + `clang` in
    /// the chef stage, cargo linker env pointing at mold, and a per-bin copy of
    /// release artifacts into `/out`. When `false` (default), output is
    /// byte-identical to the plain scaffolder baseline. Sourced from
    /// `[package.metadata.ferro.deploy].fast_build`.
    pub fast_build: bool,
}

/// Render a Dockerfile from the supplied context. Pure string substitution.
pub fn render_dockerfile(ctx: &DockerContext) -> String {
    let frontend_stage = if ctx.has_frontend {
        format!("{TYPES_GEN_STAGE_BODY}{FRONTEND_STAGE_WITH_TYPES_COPY_BODY}")
    } else {
        String::new()
    };

    let bin_copies = ctx
        .bins
        .iter()
        .map(|b| {
            if ctx.fast_build {
                format!("COPY --from=backend-builder /out/{b} /usr/local/bin/{b}")
            } else {
                format!("COPY --from=backend-builder /app/target/release/{b} /usr/local/bin/{b}")
            }
        })
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

    // Fast-build fragments. Each is the empty string / plain baseline when
    // `fast_build == false`, keeping the rendered output byte-identical to the
    // pre-fast-build template.
    let fast_header = if ctx.fast_build { FAST_HEADER } else { "" };
    let chef_apt_install = if ctx.fast_build {
        FAST_CHEF_APT_INSTALL
    } else {
        PLAIN_CHEF_APT_INSTALL
    };
    let chef_env = if ctx.fast_build { FAST_CHEF_ENV } else { "" };

    let cook_step = if ctx.fast_build {
        format!(
            "{FAST_COOK_COMMENT}{FAST_CACHE_MOUNT_RUN}cargo chef cook --release --recipe-path recipe.json"
        )
    } else {
        "RUN cargo chef cook --release --recipe-path recipe.json".to_string()
    };

    let build_step = if ctx.fast_build {
        let bin_out_copies = ctx
            .bins
            .iter()
            .map(|b| format!("    && cp target/release/{b} /out/{b}"))
            .collect::<Vec<_>>()
            .join(" \\\n");
        format!(
            "{FAST_CACHE_MOUNT_RUN}cargo build --release \\\n    && mkdir -p /out \\\n{bin_out_copies}"
        )
    } else {
        "RUN cargo build --release".to_string()
    };

    let rendered = DOCKERFILE_TPL
        .replace("{{FRONTEND_STAGE}}", &frontend_stage)
        .replace("{{FAST_HEADER}}", fast_header)
        .replace("{{RUST_IMAGE_TAG}}", &rust_image_tag)
        .replace("{{CHEF_APT_INSTALL}}", chef_apt_install)
        .replace("{{CHEF_ENV}}", chef_env)
        .replace("{{COOK_STEP}}", &cook_step)
        .replace("{{BUILD_STEP}}", &build_step)
        .replace("{{FERRO_VERSION}}", &ctx.ferro_version)
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

// ============================================================================
// Fast-build fragments (opt-in via `[package.metadata.ferro.deploy].fast_build`)
// ============================================================================
//
// When `DockerContext.fast_build == true` the renderer substitutes these
// fragments in place of the plain baseline. They add: a header rationale block,
// `mold` + `clang` in the chef stage, cargo linker env pointing at mold,
// BuildKit cache mounts on the cargo registry/git indexes and the target dir,
// and a per-bin copy of release artifacts into `/out`.

/// Header rationale comment lines, inserted after the `--force.` line. No
/// trailing newline — the template supplies the newline before the blank line
/// that precedes `FROM ... AS chef`.
const FAST_HEADER: &str = "\n#\n# Build speed: BuildKit cache mounts on /usr/local/cargo/{registry,git} and\n# /app/target persist between deploys. cargo-chef + cache mounts means a build\n# with an unchanged Cargo.lock recompiles only the workspace crates, not the\n# 600+ third-party deps. mold linker cuts final link time ~30% over ld.";

/// Plain chef apt-install line: a single continuation line.
const PLAIN_CHEF_APT_INSTALL: &str =
    "    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \\";

/// Fast chef apt-install: splits to two lines and adds `mold clang`.
const FAST_CHEF_APT_INSTALL: &str = "    && apt-get install -y --no-install-recommends \\\n       pkg-config libssl-dev ca-certificates mold clang \\";

/// Fast chef ENV block: tell cargo to drive the linker through mold. Leading
/// newline splices it directly after `cargo install cargo-chef --locked`.
const FAST_CHEF_ENV: &str = "\n# Tell cargo to invoke clang as the linker driver and pass it through to mold.\nENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang\nENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS=\"-C link-arg=-fuse-ld=mold\"";

/// Fast cook-step comment preceding the cache-mounted `cargo chef cook`. Ends
/// with a newline so the RUN starts on its own line.
const FAST_COOK_COMMENT: &str = "# Cook dependencies. Cache mounts persist between deploys so the registry,\n# git index, and pre-built target/ artifacts don't have to be re-downloaded\n# or re-compiled when Cargo.lock hasn't changed.\n";

/// The three BuildKit cache-mount lines, reused by the cook and build steps.
/// Renders as `RUN --mount=... \` followed by two indented `--mount=... \`
/// continuation lines, then a `\n    ` so the command appends on the next
/// indented line.
const FAST_CACHE_MOUNT_RUN: &str = "RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry,sharing=locked \\\n    --mount=type=cache,target=/usr/local/cargo/git,id=cargo-git,sharing=locked \\\n    --mount=type=cache,target=/app/target,id=app-target,sharing=locked \\\n    ";

/// Phase 156 §6 (D-15): the new `types-gen` Rust stage. Emitted unconditionally
/// when `has_frontend == true`. Uses the same `rust:{{RUST_IMAGE_TAG}}` base as
/// the backend builder chain; the slim variant has the full Rust toolchain
/// (verified — see RESEARCH Pitfall 5).
///
/// Pins `ferro-cli` to `{{FERRO_VERSION}}` (resolved from the project's
/// Cargo.lock). The `--locked` flag uses ferro-cli's published Cargo.lock for
/// reproducibility.
const TYPES_GEN_STAGE_BODY: &str = r#"
FROM rust:{{RUST_IMAGE_TAG}} AS types-gen
WORKDIR /app
RUN cargo install ferro-cli --version {{FERRO_VERSION}} --locked
COPY . .
RUN ferro generate-types
"#;

/// Phase 156 §6 (D-15): the frontend stage gains `COPY --from=types-gen` so
/// the gitignored `frontend/src/types/` is materialized inside the build
/// context before `npm run build` (which invokes `tsc`). Replaces
/// `FRONTEND_STAGE_BODY` when `has_frontend == true`. The COPY line is
/// positioned immediately BEFORE `RUN npm run build` — order matters: tsc
/// resolves `./types/*` imports during `npm run build`.
const FRONTEND_STAGE_WITH_TYPES_COPY_BODY: &str = r#"
FROM node:20-bookworm-slim AS frontend-builder
WORKDIR /frontend
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci || npm install
COPY frontend/ ./
COPY --from=types-gen /app/frontend/src/types ./src/types
RUN npm run build
"#;

/// Read `[toolchain] channel` from `<root>/rust-toolchain.toml`. Defaults to
/// `"stable"` when the file is missing or unparseable.
///
/// Note: `[[bin]]` enumeration is handled by `crate::project::read_bins`.
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

/// Phase 156 §6 (D-16/D-21): resolve the ferro-cli version to pin in the
/// `types-gen` Docker stage. Parses the project's `Cargo.lock` for the
/// `ferro-rs` package; falls back to the current binary's own version
/// (`env!("CARGO_PKG_VERSION")`) when the lockfile is absent or has no
/// `ferro-rs` entry. Never returns an empty string.
///
/// Used by `crate::commands::docker_init`, `crate::doctor::checks::docker_template_drift`,
/// and the `gestiscilo_fixture` integration test to construct `DockerContext` at
/// the I/O boundary before passing the resolved version into the pure renderer.
pub fn resolve_ferro_version(project_root: &Path) -> String {
    if let Ok(lock) = fs::read_to_string(project_root.join("Cargo.lock")) {
        if let Ok(parsed) = lock.parse::<Value>() {
            if let Some(pkgs) = parsed.get("package").and_then(|v| v.as_array()) {
                for pkg in pkgs {
                    let name = pkg.get("name").and_then(|n| n.as_str());
                    let ver = pkg.get("version").and_then(|v| v.as_str());
                    if name == Some("ferro-rs") {
                        if let Some(v) = ver {
                            return v.to_string();
                        }
                    }
                }
            }
        }
    }
    env!("CARGO_PKG_VERSION").to_string()
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
            ferro_version: "0.0.0-test".to_string(),
            fast_build: false,
        }
    }

    /// Byte-for-byte golden test for the opt-in fast_build Dockerfile, using
    /// the real gestiscilo deploy metadata as the parameterization. Any drift in
    /// the fast-build fragments (whitespace, line continuations, ordering) fails
    /// here.
    #[test]
    fn fast_build_renders_expected_dockerfile_verbatim() {
        let ctx = DockerContext {
            rust_channel: "stable".to_string(),
            has_frontend: false,
            bins: vec![
                "gestiscilo".to_string(),
                "screenshot-worker".to_string(),
                "pdf-worker".to_string(),
            ],
            web_bin: "gestiscilo".to_string(),
            copy_dirs_present: vec!["themes".to_string(), "src/views".to_string()],
            runtime_apt: vec![
                "chromium".to_string(),
                "fonts-liberation".to_string(),
                "ca-certificates".to_string(),
            ],
            ferro_version: "0.0.0-test".to_string(),
            fast_build: true,
        };

        let expected = r#"# syntax=docker/dockerfile:1.6
# Generated by `ferro docker:init` (Phase 122.2 §2). Edit freely; ferro will
# only overwrite this file when invoked with --force.
#
# Build speed: BuildKit cache mounts on /usr/local/cargo/{registry,git} and
# /app/target persist between deploys. cargo-chef + cache mounts means a build
# with an unchanged Cargo.lock recompiles only the workspace crates, not the
# 600+ third-party deps. mold linker cuts final link time ~30% over ld.

FROM rust:slim-bookworm AS chef
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       pkg-config libssl-dev ca-certificates mold clang \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-chef --locked
# Tell cargo to invoke clang as the linker driver and pass it through to mold.
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS backend-builder
COPY --from=planner /app/recipe.json recipe.json
# Cook dependencies. Cache mounts persist between deploys so the registry,
# git index, and pre-built target/ artifacts don't have to be re-downloaded
# or re-compiled when Cargo.lock hasn't changed.
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,id=cargo-git,sharing=locked \
    --mount=type=cache,target=/app/target,id=app-target,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,id=cargo-git,sharing=locked \
    --mount=type=cache,target=/app/target,id=app-target,sharing=locked \
    cargo build --release \
    && mkdir -p /out \
    && cp target/release/gestiscilo /out/gestiscilo \
    && cp target/release/screenshot-worker /out/screenshot-worker \
    && cp target/release/pdf-worker /out/pdf-worker

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*
# ferro:runtime-apt
RUN apt-get update \
    && apt-get install -y --no-install-recommends chromium fonts-liberation ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=backend-builder /out/gestiscilo /usr/local/bin/gestiscilo
COPY --from=backend-builder /out/screenshot-worker /usr/local/bin/screenshot-worker
COPY --from=backend-builder /out/pdf-worker /usr/local/bin/pdf-worker
COPY themes themes
COPY src/views src/views
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/gestiscilo"]
CMD ["serve"]
"#;

        let rendered = render_dockerfile(&ctx);
        assert_eq!(
            rendered, expected,
            "fast_build Dockerfile drifted from golden output"
        );
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
    fn types_gen_stage_present_when_has_frontend() {
        let mut c = ctx();
        c.has_frontend = true;
        let out = render_dockerfile(&c);
        assert!(
            out.contains("AS types-gen"),
            "expected types-gen stage; got:\n{out}"
        );
    }

    #[test]
    fn types_gen_stage_absent_when_no_frontend() {
        let mut c = ctx();
        c.has_frontend = false;
        let out = render_dockerfile(&c);
        assert!(!out.contains("AS types-gen"));
        assert!(!out.contains("frontend-builder"));
    }

    #[test]
    fn copy_from_types_gen_before_npm_build() {
        let mut c = ctx();
        c.has_frontend = true;
        let out = render_dockerfile(&c);
        let copy_pos = out
            .find("COPY --from=types-gen /app/frontend/src/types ./src/types")
            .expect("missing COPY --from=types-gen line");
        let build_pos = out
            .find("RUN npm run build")
            .expect("missing npm run build");
        assert!(
            copy_pos < build_pos,
            "COPY --from=types-gen must appear before RUN npm run build (copy={copy_pos}, build={build_pos})"
        );
    }

    #[test]
    fn ferro_version_token_resolved() {
        let mut c = ctx();
        c.has_frontend = true;
        c.ferro_version = "9.9.9".to_string();
        let out = render_dockerfile(&c);
        assert!(
            out.contains("--version 9.9.9"),
            "expected --version 9.9.9; got:\n{out}"
        );
        assert!(!out.contains("{{FERRO_VERSION}}"));
    }

    #[test]
    fn types_gen_stage_uses_same_rust_image_tag() {
        let mut c = ctx();
        c.has_frontend = true;
        c.rust_channel = "stable".into();
        let out = render_dockerfile(&c);
        assert!(out.contains("FROM rust:slim-bookworm AS types-gen"));
        c.rust_channel = "1.90.0".into();
        let out = render_dockerfile(&c);
        assert!(out.contains("FROM rust:1.90.0-slim-bookworm AS types-gen"));
    }

    #[test]
    fn no_unresolved_tokens_with_frontend_stage() {
        let mut c = ctx();
        c.has_frontend = true;
        let out = render_dockerfile(&c);
        assert!(
            !out.contains("{{"),
            "unresolved template token in rendered Dockerfile:\n{out}"
        );
    }

    #[test]
    fn resolve_ferro_version_reads_cargo_lock() {
        use std::fs;
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("Cargo.lock"),
            r#"
[[package]]
name = "other-crate"
version = "0.5.0"

[[package]]
name = "ferro-rs"
version = "1.2.3"
            "#,
        )
        .unwrap();
        assert_eq!(resolve_ferro_version(tmp.path()), "1.2.3");
    }

    #[test]
    fn resolve_ferro_version_falls_back_when_lockfile_absent() {
        let tmp = TempDir::new().unwrap();
        let v = resolve_ferro_version(tmp.path());
        assert_eq!(v, env!("CARGO_PKG_VERSION"));
        assert!(!v.is_empty());
    }

    #[test]
    fn resolve_ferro_version_falls_back_when_ferro_rs_absent() {
        use std::fs;
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("Cargo.lock"),
            r#"
[[package]]
name = "other-crate"
version = "0.5.0"
            "#,
        )
        .unwrap();
        assert_eq!(resolve_ferro_version(tmp.path()), env!("CARGO_PKG_VERSION"));
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
            ferro_version: "0.0.0-test".to_string(),
            fast_build: false,
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
            ferro_version: "0.0.0-test".to_string(),
            fast_build: false,
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
