---
phase: 130-invert-dep-convention-simple
plan: 01
subsystem: ferro-cli / deploy-scaffold
tags: [deploy, doctor, templates, docs]
requires: []
provides:
  - single-manifest docker build convention
  - scaffold Cargo.toml patch.crates-io hint
affects:
  - ferro-cli/src/doctor
  - ferro-cli/src/deploy
  - ferro-cli/src/commands/{docker_init,do_init}
  - ferro-cli/src/templates/{files/docker,files/backend,docker.rs}
  - docs/src/cli/doctor.md
  - docs/src/reference/cli.md
  - ferro-mcp/src/service.rs
  - PUBLISHING.md
tech-stack:
  added: []
  patterns:
    - "[patch.crates-io] for local ferro dev, uncommitted by consumer project"
key-files:
  deleted:
    - ferro-cli/src/doctor/checks/cargo_docker_toml_staleness.rs
    - ferro-cli/src/doctor/checks/ferro_version_skew.rs
    - ferro-cli/src/deploy/rewrite_ferro_version.rs
  modified:
    - ferro-cli/src/doctor/checks/mod.rs
    - ferro-cli/src/doctor/registry.rs
    - ferro-cli/src/deploy/mod.rs
    - ferro-cli/src/commands/docker_init.rs
    - ferro-cli/src/commands/do_init.rs
    - ferro-cli/src/main.rs
    - ferro-cli/src/project.rs
    - ferro-cli/src/templates/files/docker/Dockerfile.tpl
    - ferro-cli/src/templates/docker.rs
    - ferro-cli/src/templates/files/backend/Cargo.toml.tpl
    - ferro-cli/tests/docker_init_dry_run.rs
    - docs/src/cli/doctor.md
    - docs/src/reference/cli.md
    - ferro-mcp/src/service.rs
    - PUBLISHING.md
decisions:
  - Doctor registry shrinks from 11 to 9 checks; deploy category contains only copy_dirs_dockerignore_collision.
  - docker:init now writes exactly Dockerfile + .dockerignore. --ferro-version flag kept in CLI surface but ignored (prefixed _ in execute signature) to preserve wire compatibility.
  - ferro_versions metadata field kept as a parsed-but-ignored reservation; comment updated.
metrics:
  duration: ~15min
  completed: 2026-04-09
---

# Phase 130 Plan 01: Invert dep convention (simple) Summary

Retire the `Cargo.docker.toml` dual-manifest and the two doctor checks
(`cargo_docker_toml_staleness`, `ferro_version_skew`) that existed only to
reconcile it. Docker builds now consume the project `Cargo.toml` directly.
Ferro developers pointing at an unpublished local checkout maintain an
uncommitted `[patch.crates-io]` block by hand; the `ferro new` scaffold
carries a one-line hint comment to that effect.

## What changed

**Deleted (3 files, net -1038 lines across both commits):**

- `ferro-cli/src/doctor/checks/cargo_docker_toml_staleness.rs`
- `ferro-cli/src/doctor/checks/ferro_version_skew.rs`
- `ferro-cli/src/deploy/rewrite_ferro_version.rs` (including its 10+ unit tests)

**Modified:** see frontmatter `key-files.modified` list.

**Commit log:**

- `4b6529a9` refactor(130-01): delete Cargo.docker.toml dual-manifest apparatus
- `ea53f98f` refactor(130-01): update templates, tests and docs for single-manifest convention

## Doctor registry

Check count: 11 → 9. Deploy category shrinks from 3 checks to 1
(`copy_dirs_dockerignore_collision`). Registry unit tests updated
accordingly (`default_checks_returns_nine_in_declared_order`,
`deploy_category_filter_returns_one`).

## CLI surface

`ferro docker:init` signature unchanged — `--force` and `--ferro-version`
flags still accepted by clap for wire compatibility, but `--ferro-version`
is now a no-op (prefixed with `_` in the `execute` signature to silence
unused-argument warnings). Future work can drop the flag entirely once
downstream tooling has migrated.

Completion log line is now:

    docker:init wrote Dockerfile and .dockerignore in <root>

`ferro do:init --dry-run` no longer includes a `Cargo.docker.toml` preview.

## Templates

`Dockerfile.tpl`: two `COPY Cargo.docker.toml Cargo.toml` lines removed
(one per stage). The pre-existing `COPY . .` lines in both stages still
provide the full workspace; the renderer test
`dockerfile_copies_workspace_in_both_stages` locks this in.

`Cargo.toml.tpl` (scaffold): adds a single comment line directly above
`[dependencies]`:

    # Local ferro dev: append an uncommitted [patch.crates-io] block at the bottom of this file.

## Docs

- `docs/src/cli/doctor.md`: 11-row check table → 9-row table, numbering
  re-contiguous; preflight section shrunk to the one surviving Deploy
  check; `--deploy` filter description updated.
- `docs/src/reference/cli.md`: `docker:init` section rewritten to describe
  Dockerfile + `.dockerignore` output only; summary table row updated.
- `PUBLISHING.md` §Version Model: replaces the `ferro_version` →
  `Cargo.docker.toml` rewrite narrative with the `[patch.crates-io]`
  convention.
- `ferro-mcp/src/service.rs` `deploy_check` tool description: references
  to "Cargo.docker.toml" staleness and ferro version skew removed; the
  tool now only mentions copy_dirs/.dockerignore collisions.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing cross-reference cleanup] `PUBLISHING.md` and `ferro-mcp/src/service.rs` still mentioned Cargo.docker.toml**

- **Found during:** final workspace grep after Task 2.
- **Issue:** the plan's must_have "No file in the repository references `Cargo.docker.toml`" covers the whole tree, but the interfaces block only named `ferro-cli/`, `docs/`, and `tests/`. `PUBLISHING.md` §Version Model and the `deploy_check` MCP tool description in `ferro-mcp/src/service.rs` both described the retired rewrite.
- **Fix:** rewrote the `PUBLISHING.md` paragraph to describe the `[patch.crates-io]` convention; trimmed the `deploy_check` tool description to cover only `copy_dirs`/`.dockerignore` collisions.
- **Files modified:** `PUBLISHING.md`, `ferro-mcp/src/service.rs`.
- **Commit:** `ea53f98f`.

**2. [Rule 3 - Stale TODO pointing at deleted module] `ferro-cli/src/project.rs` ferro_versions doc comment**

- **Found during:** Task 1 compile-check.
- **Issue:** the `ferro_versions` field carried a TODO naming `deploy::rewrite_ferro_version::rewrite_cargo_docker_toml` as the future consumer. With that module deleted, the comment pointed at a nonexistent path.
- **Fix:** reworded the comment to describe the field as a parsed-but-ignored schema reservation that Phase 130 left dormant.
- **Files modified:** `ferro-cli/src/project.rs`.
- **Commit:** `4b6529a9`.

### Scope boundary notes

- `--ferro-version` CLI flag was not removed from `DockerInit` clap variant; only made a no-op. This preserves wire compatibility and stays inside the plan's "no new CLI verbs, no CLI removals" boundary. If the next phase wants to drop the flag, it's a trivial follow-up.
- `find_ferro_path_deps` in `deploy/mod.rs` is now unused in live code but is explicitly called out as KEEP in the plan interfaces block. Left in place under the existing `#![allow(dead_code)]` guard.

## Verification

Full pre-commit gate ran cleanly at the end of Task 2:

```
cargo fmt --all -- --check          # clean
cargo clippy --all --all-targets -- -D warnings   # no warnings
cargo test --all-features           # all green
```

Workspace-wide grep for `Cargo\.docker\.toml`, `cargo_docker_toml_staleness`,
`ferro_version_skew`, `compute_cargo_docker_toml`, and `rewrite_ferro_version`
outside `.planning/` returns zero hits.

## Self-Check: PASSED

- Deleted files verified absent: `cargo_docker_toml_staleness.rs`,
  `ferro_version_skew.rs`, `rewrite_ferro_version.rs`.
- Commits present in `git log`: `4b6529a9`, `ea53f98f`.
- Grep for retired symbols in `ferro-cli/` and `docs/src/`: zero hits.
- Pre-commit gate: passed.
