---
phase: 122-deploy-scaffold-core-rewrite
plan: 05
subsystem: ferro-cli
tags: [ferro-cli, deploy-scaffold, do-init, app-yaml, cli]
requires:
  - ferro-cli/src/project.rs (find_project_root, package_name, read_bins)
  - ferro-cli/src/deploy/env_example.rs (parse_env_example, EnvEntry)
  - ferro-cli/src/deploy/classify.rs (is_secret)
  - ferro-cli/src/templates/docker.rs (legacy do_app_yaml_template removed)
provides:
  - "ferro do:init --repo owner/repo --region <slug> --force --ferro-ref <ref>"
  - ferro-cli/src/templates/do.rs::AppYamlContext + render_app_yaml
affects:
  - ferro-cli/src/main.rs (DoInit clap variant + dispatch)
  - ferro-cli/src/commands/do_init.rs (full rewrite)
  - ferro-cli/src/templates/docker.rs (legacy do_app_yaml_template shim removed)
  - ferro-cli/src/templates/mod.rs (do_spec module registration via #[path])
  - ferro-cli/src/templates/files/do/app.yaml.tpl (full rewrite with placeholders)
tech-stack:
  added: []
  patterns:
    - context-struct + render_* mirror of templates/docker.rs
    - tolerant template-block composition (envs/databases/workers each render to "" when absent)
    - hand-rolled repo validator (no regex dep)
key-files:
  created:
    - ferro-cli/src/templates/do.rs
    - .planning/phases/122-deploy-scaffold-core-rewrite/122-05-SUMMARY.md
  modified:
    - ferro-cli/src/templates/files/do/app.yaml.tpl
    - ferro-cli/src/templates/mod.rs
    - ferro-cli/src/templates/docker.rs
    - ferro-cli/src/main.rs
    - ferro-cli/src/commands/do_init.rs
decisions:
  - "templates/do.rs registered via `#[path = \"do.rs\"] pub mod do_spec;` to dodge the `do` keyword while keeping the file name aligned with templates/docker.rs."
  - "envs block uses 4-space prefix to nest under the existing `services:` `- name: web` entry; databases/workers blocks emit a leading newline so they slot in at column 0 after the services block."
  - "DATABASE_URL detection is case-insensitive (eq_ignore_ascii_case) — matches the classify::is_secret tolerance."
  - "is_valid_repo accepts owner/repo with [A-Za-z0-9_.-] only; rejects empty parts, leading/trailing slash, three-segment paths, and whitespace. Hand-rolled to avoid regex churn."
  - "do_init test uses a private run_for_test helper to bypass std::process::exit and the docker_init chain — keeps the unit pure."
requirements: [D-12, D-13, D-14, D-15, D-16, D-17, D-18, D-19]
metrics:
  duration: ~6min
  completed: 2026-04-07
---

# Phase 122 Plan 05: do:init Rewrite Summary

`ferro do:init` rewritten end-to-end to emit a deployable `.do/app.yaml` with `--region`, `--force`, `--ferro-ref`, and validated `--repo` flags. The new `templates::do_spec::render_app_yaml` mirrors `templates::docker::render_dockerfile`: typed context in, complete YAML out, no IO. Envs are derived from `.env.example` with SECRET classification, `databases:` is emitted on `DATABASE_URL` presence, and one `workers:` entry is emitted per non-server `[[bin]]`.

## What Was Built

**`ferro-cli/src/templates/files/do/app.yaml.tpl`** — Full rewrite. Placeholders: `{app_name}`, `{region}`, `{github_repo}`, `{envs_block}`, `{databases_block}`, `{workers_block}`. The three block placeholders sit at column 0 so the renderer can substitute pre-indented content (or "") without breaking YAML.

**`ferro-cli/src/templates/do.rs`** — New module exposing `AppYamlContext<'a>` and `render_app_yaml(ctx: &AppYamlContext) -> String`. Builds the envs block (with `value: ${db.DATABASE_URL}` substitution + `type: SECRET` tagging via `deploy::classify::is_secret`), the databases block (single PG cluster named `db`, only when DATABASE_URL is present), and the workers block (one component per bin whose name ≠ package_name). YAML escaping handles quotes and backslashes.

**`ferro-cli/src/templates/mod.rs`** — Registers `do.rs` as `pub mod do_spec` via `#[path = "do.rs"]` to dodge the `do` keyword.

**`ferro-cli/src/templates/docker.rs`** — Legacy `do_app_yaml_template(package_name, github_repo)` deleted; sole DO renderer is now `do_spec::render_app_yaml`.

**`ferro-cli/src/main.rs`** — `Commands::DoInit` rewritten as a struct variant with `repo: String` (required), `region: String` (default `"fra1"`), `force: bool`, `ferro_ref: String` (default `"main"`). Dispatch arm forwards to `commands::do_init::run(&repo, &region, force, &ferro_ref)`.

**`ferro-cli/src/commands/do_init.rs`** — Full rewrite. `run(repo, region, force, ferro_ref)` validates the repo first (no disk writes on failure), walks up for the project root, creates `.do/`, parses `.env.example` if present, composes the context, writes `app.yaml`, then chains `docker_init::generate(force, ferro_ref, &[])` to ensure a Dockerfile exists. `is_valid_repo` is a `pub(crate)` hand-rolled validator. Tests cover the 8-case validation matrix and an end-to-end yaml generation against a tempdir project with DATABASE_URL.

## Tasks Completed

| Task | Name                                                              | Commit   | Files                                                                                                                              |
| ---- | ----------------------------------------------------------------- | -------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| 1    | app.yaml.tpl skeleton + do.rs renderer                            | 80a6efc7 | ferro-cli/src/templates/{files/do/app.yaml.tpl, do.rs, mod.rs, docker.rs}, ferro-cli/src/commands/do_init.rs (interim bridge stub) |
| 2    | Rewrite do_init.rs + clap DoInit variant                          | 148634c4 | ferro-cli/src/main.rs, ferro-cli/src/commands/do_init.rs, ferro-cli/src/templates/do.rs (drop allow(dead_code))                    |

## Verification

- `cargo fmt -p ferro-cli` — clean
- `cargo clippy -p ferro-cli --all-targets -- -D warnings` — clean
- `cargo test -p ferro-cli templates::do_spec` — 4/4 passing (scenario A, B, D + yaml_escape)
- `cargo test -p ferro-cli commands::do_init` — 2/2 passing (repo_validation_matrix 8 cases + writes_app_yaml_with_region_and_envs)
- `cargo test -p ferro-cli` — **340 passing**, 0 failing
- `cargo build -p ferro-cli` — clean

All grep-based acceptance criteria satisfied:
- `pub struct AppYamlContext` ✓
- `pub fn render_app_yaml` ✓
- `is_secret` referenced from do.rs ✓
- `{envs_block}`, `{databases_block}`, `{workers_block}`, `{region}` in app.yaml.tpl ✓
- `pub fn run(repo: &str, region: &str, force: bool, ferro_ref: &str)` in do_init.rs ✓
- `is_valid_repo`, `AppYamlContext`, `parse_env_example` referenced from do_init.rs ✓
- `default_value = "fra1"` in main.rs ✓

## Decisions Made

See frontmatter `decisions:`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocker] `do_app_yaml_template` removal breaks legacy do_init.rs**
- **Found during:** Task 1 build
- **Issue:** Plan 122-04 left `do_init.rs` calling `templates::do_app_yaml_template`. Removing the shim during Task 1 (per plan) breaks compilation, but Task 2 owns the full rewrite. Task 1 must still commit cleanly on its own.
- **Fix:** In Task 1's commit, replaced the legacy call with an inline format-string stub and dropped the `use crate::templates;` import. Task 2 then immediately replaced the entire file with the proper context-driven implementation.
- **Files modified:** `ferro-cli/src/commands/do_init.rs`
- **Commit:** 80a6efc7 (stub) → 148634c4 (real impl)

**2. [Rule 3 - Blocker] `templates/do.rs` private helpers tripped clippy in Task 1**
- **Found during:** Task 1 clippy
- **Issue:** With the new `do.rs` module compiled but no consumer in Task 1, clippy `-D warnings` flagged `render_app_yaml` and the four private builders as `dead_code`.
- **Fix:** Added file-level `#![allow(dead_code)]` to `do.rs` for the duration of Task 1, then removed it in Task 2 once `commands::do_init` started consuming the module.
- **Files modified:** `ferro-cli/src/templates/do.rs`
- **Commit:** 80a6efc7 (allow added) → 148634c4 (allow removed)

### Deferred Issues

Pre-existing fmt drift in `ferro-json-ui` — same files flagged across plans 01–04. Out of scope per scope-boundary rule. Per-crate `cargo fmt -p ferro-cli` passes.

## Auth Gates

None.

## Self-Check: PASSED

- FOUND: ferro-cli/src/templates/files/do/app.yaml.tpl
- FOUND: ferro-cli/src/templates/do.rs
- FOUND: ferro-cli/src/templates/mod.rs (do_spec registered)
- FOUND: ferro-cli/src/templates/docker.rs (no do_app_yaml_template)
- FOUND: ferro-cli/src/main.rs (DoInit struct variant with repo/region/force/ferro_ref)
- FOUND: ferro-cli/src/commands/do_init.rs (run + is_valid_repo + AppYamlContext)
- FOUND: commit 80a6efc7
- FOUND: commit 148634c4
- FOUND: 4 passing templates::do_spec tests
- FOUND: 2 passing commands::do_init tests
- FOUND: 340 passing ferro-cli tests overall
