// ============================================================================
// Docker Templates
// ============================================================================

use crate::project::{BinEntry, ProjectDirs};

const DOCKERFILE_TPL: &str = include_str!("files/docker/Dockerfile.tpl");
const DOCKERIGNORE_TPL: &str = include_str!("files/docker/dockerignore.tpl");

const FRONTEND_STAGE: &str = "\n# ============ Stage: frontend-builder ============
FROM node:22-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci
COPY frontend/ ./
RUN npm run build
";

const FERRO_REWRITE_LINE: &str = "RUN bash scripts/rewrite-ferro-deps.sh";

/// Context driving Dockerfile rendering. See plan 122-03.
pub struct DockerfileContext<'a> {
    pub package_name: &'a str,
    pub bins: &'a [BinEntry],
    pub dirs: ProjectDirs,
    pub runtime_deps: &'a [String],
    pub rust_base_image: &'a str,
    pub workspace_members: &'a [String],
    pub ferro_ref: &'a str,
}

/// Render a Dockerfile from a context. Pure function — no IO.
pub fn render_dockerfile(ctx: &DockerfileContext) -> String {
    // Synthesize a single bin from package_name when bins is empty.
    let synthesized: Vec<BinEntry>;
    let bins: &[BinEntry] = if ctx.bins.is_empty() {
        synthesized = vec![BinEntry {
            name: ctx.package_name.to_string(),
            path: None,
        }];
        &synthesized
    } else {
        ctx.bins
    };

    let frontend_stage = if ctx.dirs.has_frontend {
        FRONTEND_STAGE.to_string()
    } else {
        String::new()
    };

    let workspace_copy = workspace_copy_block(ctx.workspace_members);
    let runtime_apt = runtime_apt_block(ctx.runtime_deps);
    let runtime_bins = runtime_bin_copies(bins);
    let runtime_optional = runtime_optional_copies(&ctx.dirs);
    let cargo_bins = cargo_build_bins(bins);
    let entrypoint = entrypoint_bin(ctx.package_name, bins);

    let ferro_rewrite = FERRO_REWRITE_LINE;

    DOCKERFILE_TPL
        .replace("{ferro_ref}", ctx.ferro_ref)
        .replace("{frontend_stage}", &frontend_stage)
        .replace("{rust_base_image}", ctx.rust_base_image)
        .replace("{workspace_copy_planner}", &workspace_copy)
        .replace("{workspace_copy_builder}", &workspace_copy)
        .replace("{ferro_rewrite_planner}", ferro_rewrite)
        .replace("{ferro_rewrite_builder}", ferro_rewrite)
        .replace("{cargo_build_bins}", &cargo_bins)
        .replace("{runtime_apt_block}", &runtime_apt)
        .replace("{runtime_bin_copies}", &runtime_bins)
        .replace("{runtime_optional_copies}", &runtime_optional)
        .replace("{entrypoint_bin}", entrypoint)
}

/// Generate .dockerignore file. Plan 122-06 will replace this.
pub fn dockerignore_template() -> &'static str {
    DOCKERIGNORE_TPL
}

/// Render a .dockerignore with extra entries appended (plan 122-06 owns the impl).
#[doc(hidden)]
#[allow(dead_code)]
pub fn render_dockerignore(_extra_entries: &[&str]) -> String {
    DOCKERIGNORE_TPL.to_string()
}

/// Generate docker-compose.yml for local development
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

// ============================================================================
// DigitalOcean App Platform Templates
// ============================================================================

/// Generate app.yaml for DigitalOcean App Platform deployment
pub fn do_app_yaml_template(package_name: &str, github_repo: &str) -> String {
    include_str!("files/do/app.yaml.tpl")
        .replace("{package_name}", package_name)
        .replace("{github_repo}", github_repo)
}

// ============================================================================
// Helpers
// ============================================================================

fn workspace_copy_block(members: &[String]) -> String {
    members
        .iter()
        .map(|m| format!("COPY {m}/ ./{m}/"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn runtime_apt_block(deps: &[String]) -> String {
    if deps.is_empty() {
        return String::new();
    }
    let joined = deps.join(" ");
    format!(
        "# >>> ferro:runtime-deps (regenerated by ferro docker:init --runtime-deps=...)
RUN apt-get update && apt-get install -y --no-install-recommends \\
    {joined} \\
    && rm -rf /var/lib/apt/lists/*
# <<< ferro:runtime-deps"
    )
}

fn runtime_bin_copies(bins: &[BinEntry]) -> String {
    bins.iter()
        .map(|b| {
            format!(
                "COPY --from=backend-builder --chown=appuser:appuser /app/target/release/{name} /usr/local/bin/{name}",
                name = b.name
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn runtime_optional_copies(dirs: &ProjectDirs) -> String {
    let mut lines = Vec::new();
    if dirs.has_themes {
        lines.push(
            "COPY --from=backend-builder --chown=appuser:appuser /app/themes ./themes".to_string(),
        );
    }
    if dirs.has_lang {
        lines.push(
            "COPY --from=backend-builder --chown=appuser:appuser /app/lang ./lang".to_string(),
        );
    }
    if dirs.has_public {
        lines.push(
            "COPY --from=backend-builder --chown=appuser:appuser /app/public ./public".to_string(),
        );
    }
    if dirs.has_migrations {
        lines.push(
            "COPY --from=backend-builder --chown=appuser:appuser /app/migrations ./migrations"
                .to_string(),
        );
    }
    lines.join("\n")
}

fn cargo_build_bins(bins: &[BinEntry]) -> String {
    bins.iter()
        .map(|b| format!("--bin {}", b.name))
        .collect::<Vec<_>>()
        .join(" ")
}

fn entrypoint_bin<'a>(package_name: &'a str, bins: &'a [BinEntry]) -> &'a str {
    if let Some(b) = bins.iter().find(|b| b.name == package_name) {
        return &b.name;
    }
    if let Some(b) = bins.first() {
        return &b.name;
    }
    package_name
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn bin(name: &str) -> BinEntry {
        BinEntry {
            name: name.to_string(),
            path: None,
        }
    }

    #[test]
    fn scenario_a_single_bin_frontend() {
        let bins = vec![bin("mkmenu")];
        let ctx = DockerfileContext {
            package_name: "mkmenu",
            bins: &bins,
            dirs: ProjectDirs {
                has_frontend: true,
                has_themes: false,
                has_lang: true,
                has_public: true,
                has_migrations: true,
            },
            runtime_deps: &[],
            rust_base_image: "rust:1.88-slim-bookworm",
            workspace_members: &[],
            ferro_ref: "main",
        };
        let out = render_dockerfile(&ctx);
        assert!(out.contains("FROM node:22-slim AS frontend-builder"));
        assert!(out.contains("cargo build --release --bin mkmenu"));
        assert!(out.contains(
            "COPY --from=backend-builder --chown=appuser:appuser /app/target/release/mkmenu /usr/local/bin/mkmenu"
        ));
        assert!(
            out.contains("COPY --from=backend-builder --chown=appuser:appuser /app/lang ./lang")
        );
        assert!(out
            .contains("COPY --from=backend-builder --chown=appuser:appuser /app/public ./public"));
        assert!(out.contains(
            "COPY --from=backend-builder --chown=appuser:appuser /app/migrations ./migrations"
        ));
        assert!(!out.contains("/app/themes"));
        assert!(!out.contains("chromium"));
        assert!(out.contains("ENTRYPOINT [\"/usr/local/bin/mkmenu\"]"));
    }

    #[test]
    fn scenario_b_multibin_no_frontend_chromium_workspace() {
        let bins = vec![bin("gestiscilo"), bin("screenshot-worker")];
        let deps = vec!["chromium".to_string(), "fonts-liberation".to_string()];
        let members = vec!["crates/core".to_string(), "migration".to_string()];
        let ctx = DockerfileContext {
            package_name: "gestiscilo",
            bins: &bins,
            dirs: ProjectDirs {
                has_frontend: false,
                has_themes: true,
                has_lang: false,
                has_public: true,
                has_migrations: true,
            },
            runtime_deps: &deps,
            rust_base_image: "rust:1.88-slim-bookworm",
            workspace_members: &members,
            ferro_ref: "v0.1.87",
        };
        let out = render_dockerfile(&ctx);
        assert!(!out.contains("FROM node"));
        assert!(out.contains("cargo build --release --bin gestiscilo --bin screenshot-worker"));
        assert!(out.contains("/app/target/release/gestiscilo /usr/local/bin/gestiscilo"));
        assert!(
            out.contains("/app/target/release/screenshot-worker /usr/local/bin/screenshot-worker")
        );
        assert_eq!(out.matches("COPY crates/core/ ./crates/core/").count(), 2);
        assert_eq!(out.matches("COPY migration/ ./migration/").count(), 2);
        assert!(out.contains("# >>> ferro:runtime-deps"));
        assert!(out.contains("# <<< ferro:runtime-deps"));
        assert!(out.contains("chromium fonts-liberation"));
        assert!(out.contains("ENTRYPOINT [\"/usr/local/bin/gestiscilo\"]"));
        assert!(out.contains("ferro ref: v0.1.87"));
        assert_eq!(
            out.matches("RUN bash scripts/rewrite-ferro-deps.sh")
                .count(),
            2
        );
    }

    #[test]
    fn scenario_c_custom_rust_toolchain() {
        let bins = vec![bin("app")];
        let ctx = DockerfileContext {
            package_name: "app",
            bins: &bins,
            dirs: ProjectDirs::default(),
            runtime_deps: &[],
            rust_base_image: "rust:1.90.0-slim-bookworm",
            workspace_members: &[],
            ferro_ref: "main",
        };
        let out = render_dockerfile(&ctx);
        assert!(out.contains("FROM rust:1.90.0-slim-bookworm AS chef"));
    }

    #[test]
    fn scenario_d_empty_bins_synthesizes_from_package_name() {
        let ctx = DockerfileContext {
            package_name: "solo",
            bins: &[],
            dirs: ProjectDirs::default(),
            runtime_deps: &[],
            rust_base_image: "rust:1.88-slim-bookworm",
            workspace_members: &[],
            ferro_ref: "main",
        };
        let out = render_dockerfile(&ctx);
        assert!(out.contains("cargo build --release --bin solo"));
        assert!(out.contains("ENTRYPOINT [\"/usr/local/bin/solo\"]"));
    }
}
