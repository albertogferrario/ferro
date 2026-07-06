---
phase: 122-deploy-scaffold-core-rewrite
plan: 04
subsystem: ferro-cli
tags: [ferro-cli, deploy-scaffold, docker-init, cli]
requires:
  - ferro-cli/src/project.rs (find_project_root, package_name, read_bins, read_workspace_members, resolve_rust_base_image, detect_dirs)
  - ferro-cli/src/deploy/ferro_deps.rs (render_rewrite_script)
  - ferro-cli/src/templates/docker.rs (DockerfileContext, render_dockerfile)
provides:
  - "ferro docker:init --force --ferro-ref <ref> --runtime-deps <csv> command"
  - ferro-cli/src/commands/docker_init.rs::run / generate / generate_in
affects:
  - ferro-cli/src/main.rs (DockerInit clap variant + dispatch)
  - ferro-cli/src/commands/do_init.rs (bridged to new generate signature)
  - ferro-cli/src/templates/docker.rs (legacy dockerfile_template shim removed)
  - ferro-cli/src/templates/mod.rs (test for removed shim deleted)
tech-stack:
  added: []
  patterns:
    - command orchestrator over pure helpers (project + deploy + templates)
    - tempfile-based unit tests covering all five behavior bullets
    - 0755 unix permissions on generated shell script
key-files:
  created:
    - .planning/phases/122-deploy-scaffold-core-rewrite/122-04-SUMMARY.md
  modified:
    - ferro-cli/src/main.rs
    - ferro-cli/src/commands/docker_init.rs
    - ferro-cli/src/commands/do_init.rs
    - ferro-cli/src/templates/docker.rs
    - ferro-cli/src/templates/mod.rs
decisions:
  - "Imports go through templates wildcard re-export (use crate::templates::{render_dockerfile, DockerfileContext, dockerignore_template}) — the templates::docker submodule is private."
  - "do_init.rs bridge uses (false, \"main\", &[]) — plan 122-05 owns the real wiring of --force / --ferro-ref / --runtime-deps into do_init."
  - "generate() retains a thin wrapper around generate_in(root, ...) so do_init can still call into docker_init without re-implementing project root discovery."
requirements: [D-03, D-08, D-09, D-10, D-16, D-17, D-19]
metrics:
  duration: ~5min
  completed: 2026-04-07
---

# Phase 122 Plan 04: docker:init Command Rewrite Summary

`ferro docker:init` rewritten as a thin orchestrator over plans 01–03 helpers, with `--force`, `--ferro-ref`, and `--runtime-deps` flags wired through clap. Generates Dockerfile, .dockerignore, and `scripts/rewrite-ferro-deps.sh` (chmod 0755) at the discovered project root with zero hand-edit hooks.

## What Was Built

**`ferro-cli/src/main.rs`** — `Commands::DockerInit` is now a struct variant with `force: bool`, `ferro_ref: String` (default `"main"`), and `runtime_deps: Vec<String>` (comma-separated). Dispatch arm forwards to `commands::docker_init::run(force, &ferro_ref, &runtime_deps)`.

**`ferro-cli/src/commands/docker_init.rs`** — Completely rewritten. Public surface: `run(force, ferro_ref, runtime_deps)` (CLI entry) and `generate(force, ferro_ref, runtime_deps) -> bool` (do_init re-use). Both delegate to private `generate_in(root, ...)` which is fully unit-testable against a `tempfile::TempDir`. The function composes a `DockerfileContext` from the six `project::*` introspection helpers and writes the three artifacts. The legacy `get_package_name()` local copy is gone.

**`ferro-cli/src/templates/docker.rs`** — `dockerfile_template` shim removed; only `render_dockerfile` + `DockerfileContext` + `dockerignore_template` remain as the docker-template public surface.

**`ferro-cli/src/templates/mod.rs`** — `test_dockerfile_template_substitution` deleted (the shim it tested no longer exists).

**`ferro-cli/src/commands/do_init.rs`** — Single bridge edit: `super::docker_init::generate()` → `super::docker_init::generate(false, "main", &[])`. Plan 122-05 owns the real wiring.

## Tasks Completed

| Task | Name                                                          | Commit   | Files                                                                                                                       |
| ---- | ------------------------------------------------------------- | -------- | --------------------------------------------------------------------------------------------------------------------------- |
| 1    | Extend clap DockerInit variant with new flags                 | 02d22723 | ferro-cli/src/main.rs                                                                                                       |
| 2    | Rewrite docker_init.rs to orchestrate project + deploy + tpl  | c987d8a3 | ferro-cli/src/commands/docker_init.rs, ferro-cli/src/commands/do_init.rs, ferro-cli/src/templates/docker.rs, templates/mod.rs |

## Verification

- `cargo fmt -p ferro-cli` — clean
- `cargo clippy -p ferro-cli --all-targets -- -D warnings` — clean
- `cargo test -p ferro-cli commands::docker_init` — 5/5 passing
  - generates_full_set_on_empty_project
  - refuses_to_overwrite_without_force
  - overwrites_with_force
  - writes_ferro_ref_into_script_header
  - runtime_deps_appear_in_dockerfile
- `cargo test -p ferro-cli` — 334 passing, 0 failing
- All grep-based acceptance criteria satisfied:
  - `pub fn run(force: bool, ferro_ref: &str, runtime_deps: &[String])` ✓
  - `find_project_root` referenced ✓
  - `render_dockerfile` referenced ✓
  - `render_rewrite_script` referenced ✓
  - `fn get_package_name` absent ✓
  - `dockerfile_template` absent from templates/docker.rs ✓

## Decisions Made

See frontmatter `decisions:`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocker] `templates::docker` submodule is private**
- **Found during:** Task 2 first clippy run
- **Issue:** Plan instructed `use crate::templates::docker::{render_dockerfile, DockerfileContext}` but `mod docker;` in `templates/mod.rs` is private — only the wildcard re-export `pub use docker::*;` is reachable.
- **Fix:** Switched the import to `use crate::templates::{dockerignore_template, render_dockerfile, DockerfileContext};` which goes through the existing wildcard re-export. No public-API changes; just routes through the established path.
- **Files modified:** `ferro-cli/src/commands/docker_init.rs`
- **Commit:** c987d8a3

### Deferred Issues

Pre-existing fmt drift in `ferro-json-ui` (same files flagged in 122-01/02/03). Out of scope per scope-boundary rule. Per-crate `cargo fmt -p ferro-cli` is clean.

## Auth Gates

None.

## Self-Check: PASSED

- FOUND: ferro-cli/src/main.rs (DockerInit struct variant with runtime_deps + ferro_ref + force)
- FOUND: ferro-cli/src/commands/docker_init.rs (run/generate/generate_in)
- FOUND: ferro-cli/src/commands/do_init.rs (bridged generate call)
- FOUND: ferro-cli/src/templates/docker.rs (no dockerfile_template)
- FOUND: ferro-cli/src/templates/mod.rs (no test_dockerfile_template_substitution)
- FOUND: commit 02d22723
- FOUND: commit c987d8a3
- FOUND: 5 passing commands::docker_init tests
- FOUND: 334 passing ferro-cli tests overall
