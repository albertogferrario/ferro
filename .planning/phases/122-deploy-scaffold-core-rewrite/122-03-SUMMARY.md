---
phase: 122-deploy-scaffold-core-rewrite
plan: 03
subsystem: ferro-cli
tags: [ferro-cli, deploy-scaffold, dockerfile-renderer, templates]
requires:
  - ferro-cli/src/project.rs (BinEntry, ProjectDirs from 122-01)
provides:
  - ferro-cli/src/templates/docker.rs (DockerfileContext, render_dockerfile)
  - ferro-cli/src/templates/files/docker/Dockerfile.tpl (parameterized skeleton)
affects:
  - ferro-cli/src/templates/mod.rs (no change required; re-export already wildcard)
tech-stack:
  added: []
  patterns:
    - pure renderer over include_str! template
    - brace-placeholder substitution via chained .replace()
    - private helper-per-section composition
    - dockerfile_template shim for legacy callers (removed by 122-04)
key-files:
  created:
    - .planning/phases/122-deploy-scaffold-core-rewrite/122-03-SUMMARY.md
  modified:
    - ferro-cli/src/templates/files/docker/Dockerfile.tpl
    - ferro-cli/src/templates/docker.rs
decisions:
  - "Brace placeholder names are disjoint, so chained .replace() is sufficient — no templating engine added."
  - "FERRO_REWRITE_LINE is unconditional in both planner and builder stages (D-09); always invoking the script is harmless when there are no path deps because the generated sed script is a no-op."
  - "Frontend stage is a single const FRONTEND_STAGE block injected verbatim — kept inline rather than a separate .tpl to keep the renderer self-contained."
  - "render_dockerignore is a placeholder shim with #[allow(dead_code)] — plan 122-06 owns the real impl."
  - "dockerfile_template legacy shim retained to keep do_init/docker_init compiling — plans 122-04/05 will replace callers and delete it."
requirements: [D-01, D-02, D-03, D-04, D-05, D-06, D-07, D-09]
metrics:
  duration: ~5min
  completed: 2026-04-07
---

# Phase 122 Plan 03: Dockerfile Renderer Rewrite Summary

Replaces the static one-bin Dockerfile template with a parameterized skeleton plus a pure Rust renderer (`DockerfileContext` + `render_dockerfile`) that produces zero-hand-edit Dockerfiles for both frontend single-bin apps and multi-bin server apps with runtime apt deps and workspace crates.

## What Was Built

**`ferro-cli/src/templates/files/docker/Dockerfile.tpl`** — Rewritten as a brace-placeholder skeleton with 11 substitution slots: `{frontend_stage}`, `{rust_base_image}`, `{workspace_copy_planner/builder}`, `{ferro_rewrite_planner/builder}`, `{cargo_build_bins}`, `{runtime_apt_block}`, `{runtime_bin_copies}`, `{runtime_optional_copies}`, `{entrypoint_bin}`, `{ferro_ref}`. Bakes `ARG GITHUB_TOKEN=""` and the `git config insteadOf` workaround for private ferro git deps. Adds `git` to the chef apt install for git-deps support.

**`ferro-cli/src/templates/docker.rs`** — Rewritten around `DockerfileContext<'a>` (package_name, bins, dirs, runtime_deps, rust_base_image, workspace_members, ferro_ref) and a single pure `render_dockerfile(&ctx) -> String`. Private helpers compose each conditional section: `workspace_copy_block`, `runtime_apt_block` (with `# >>> ferro:runtime-deps` markers), `runtime_bin_copies`, `runtime_optional_copies`, `cargo_build_bins`, `entrypoint_bin`. Empty `bins` falls back to a synthesized single-bin from `package_name`. The legacy `dockerfile_template(name)` is retained as a `#[doc(hidden)]` shim until plan 122-04 rewrites the callers.

## Tasks Completed

| Task | Name                                                        | Commit   | Files                                                       |
| ---- | ----------------------------------------------------------- | -------- | ----------------------------------------------------------- |
| 1    | Rewrite Dockerfile.tpl as parameterized skeleton            | 8fabe0cb | ferro-cli/src/templates/files/docker/Dockerfile.tpl         |
| 2    | DockerfileContext + render_dockerfile renderer with tests   | a7bb808e | ferro-cli/src/templates/docker.rs                           |

## Verification

- `cargo fmt -p ferro-cli` — clean
- `cargo clippy -p ferro-cli --all-targets -- -D warnings` — clean
- `cargo test -p ferro-cli templates::docker` — 4 scenarios passing (A: single-bin frontend, B: multi-bin chromium workspace, C: custom rust toolchain, D: empty bins fallback)
- `cargo test -p ferro-cli` — 330 passing (includes preexisting `test_dockerfile_template_substitution` via shim)
- All 13 placeholders grep-verified in `Dockerfile.tpl`
- All 7 acceptance criteria from Task 2 pass (`pub struct DockerfileContext`, `pub fn render_dockerfile`, `ferro:runtime-deps`, `include_str!`, 4 test scenarios, clippy clean, build clean)

## Decisions Made

See frontmatter `decisions:`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocker] `render_dockerignore` shim trips dead-code lint**
- **Found during:** Task 2 verification (`cargo clippy -- -D warnings`)
- **Issue:** The plan instructs to keep `render_dockerignore` signature unchanged for plan 122-06; with no caller it fails the dead_code lint.
- **Fix:** Added `#[doc(hidden)] #[allow(dead_code)]` to the placeholder shim. Same precedent as 122-01/02.
- **Files modified:** `ferro-cli/src/templates/docker.rs`
- **Commit:** a7bb808e

### Deferred Issues

Pre-existing fmt drift in `ferro-json-ui` (same files flagged in 122-01/02). Out of scope per scope-boundary rule. Per-crate `cargo fmt -p ferro-cli` is clean; plan-level verification used the per-crate form.

## Auth Gates

None.

## Self-Check: PASSED

- FOUND: ferro-cli/src/templates/files/docker/Dockerfile.tpl
- FOUND: ferro-cli/src/templates/docker.rs (DockerfileContext + render_dockerfile)
- FOUND: commit 8fabe0cb
- FOUND: commit a7bb808e
- FOUND: 4 passing templates::docker scenario tests
- FOUND: 330 passing ferro-cli tests overall
