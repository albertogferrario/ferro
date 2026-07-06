---
phase: 122-deploy-scaffold-core-rewrite
plan: "08"
subsystem: ferro-cli/deploy
tags: [tests, golden-files, deploy-scaffold]
requires: [122-01, 122-02, 122-03, 122-04, 122-05, 122-06, 122-07]
provides:
  - golden-file integration tests for Dockerfile + app.yaml
  - ferro_cli library crate (lib + bin split)
affects:
  - ferro-cli/Cargo.toml
  - ferro-cli/src/main.rs
key-files:
  created:
    - ferro-cli/src/lib.rs
    - ferro-cli/tests/golden.rs
    - ferro-cli/tests/fixtures/gestiscilo/{Cargo.toml,.env.example,expected/Dockerfile,expected/app.yaml}
    - ferro-cli/tests/fixtures/mkmenu/{Cargo.toml,.env.example,frontend/package.json,expected/Dockerfile,expected/app.yaml}
  modified:
    - ferro-cli/Cargo.toml
    - ferro-cli/src/main.rs
    - ferro-cli/src/commands/make_api.rs
    - Cargo.toml (workspace exclude for fixtures)
decisions:
  - Lib-first split: main.rs now `use ferro_cli::commands;`, lib re-exports ai/analyzer/commands/deploy/project/templates
  - Workspace `exclude = ["ferro-cli/tests/fixtures"]` so nested fixture Cargo.toml files don't trip cargo
  - Hand-authored fixtures since real gestiscilo/mkmenu repos are not on this machine
metrics:
  duration: ~15min
  completed: 2026-04-07
---

# Phase 122 Plan 08: Golden File Integration Tests Summary

Snapshot-style golden tests covering the full Phase 122 contract for two scenarios (gestiscilo: multi-bin/postgres/chromium/workspace; mkmenu: single-bin/frontend) executed against the renderers from plans 03 and 05. Any future drift in `docker.rs`, `do.rs`, `project.rs`, or `deploy/*` now fails `cargo test -p ferro-cli`.

## What Changed

1. **ferro-cli lib/bin split.** Added `src/lib.rs` re-exporting `ai`, `analyzer`, `commands`, `deploy`, `project`, `templates`. `main.rs` now imports `commands` from the library. Required so `tests/golden.rs` can call `render_dockerfile` / `render_app_yaml` directly without going through clap.
2. **Hand-authored fixtures** for gestiscilo and mkmenu under `ferro-cli/tests/fixtures/`, derived from SCOPE.md. Workspace root `Cargo.toml` now excludes that subtree.
3. **Golden runner** `ferro-cli/tests/golden.rs` writes expected files when `UPDATE_GOLDEN=1` is set, otherwise asserts byte equality. Both pass for both fixtures. Includes per-case content invariants beyond byte equality (chromium, frontend-builder stage, workers block, SECRET classification).

## Verification

- `cargo fmt --all -- --check` — clean
- `cargo clippy --all --all-targets -- -D warnings` — clean
- `cargo test --all-features` — clean (whole workspace)
- `UPDATE_GOLDEN=1 cargo test -p ferro-cli --test golden` — passes, regenerates files
- `cargo test -p ferro-cli --test golden` — passes against committed expectations

## Deviations from Plan

- **[Rule 3 - blocking] Pre-existing fmt drift in ferro-json-ui.** `cargo fmt --all -- --check` failed on `ferro-json-ui/src/component.rs` and `render.rs` (committed before this plan). Ran `cargo fmt --all` to satisfy the workspace check; the diff is line-wrap only.
- **[Rule 3 - blocking] `make_api::filter_resource_fields` visibility.** Was `pub` but referenced a `pub(crate)` `FieldInfo`. Triggers `private_interfaces` warning, blocking `-D warnings` once `commands` is reachable through the library. Tightened to `pub(crate)`.
- **[Rule 2 - critical] Workspace exclude for fixtures.** Plan didn't mention that nested fixture `Cargo.toml` files would be picked up by cargo's workspace walker. Added `exclude = ["ferro-cli/tests/fixtures"]` to root `Cargo.toml`.

## Known Gap

The committed golden files were captured by running the renderers themselves against hand-authored fixtures, **not** against the real `../../gestiscilo-it/app` and `../../gestiscilo-it/mkmenu` repos (not on this machine). The tests guarantee the renderers stay self-consistent; they do **not** guarantee parity with the live deployed Dockerfiles.

## Manual Verification Checklist (deferred)

When the real reference apps are available locally, perform the SCOPE.md Verification pass:

- [ ] Delete `Dockerfile`, `.dockerignore`, `.do/app.yaml` from `gestiscilo-it/app` and regenerate via `ferro docker:init --runtime-deps chromium,fonts-liberation` + `ferro do:init --repo gestiscilo-it/app`
- [ ] Diff against the previous hand-patched files; confirm zero hand edits required for build
- [ ] Repeat for `gestiscilo-it/mkmenu`
- [ ] If drift is found, update fixtures here, run `UPDATE_GOLDEN=1 cargo test -p ferro-cli --test golden`, and commit the new expected files

## Commits

- `c73771d0` test(122-08): add gestiscilo and mkmenu deploy fixtures
- `fb5b423a` test(122-08): add golden file integration tests for deploy scaffold

## Self-Check: PASSED
- ferro-cli/src/lib.rs — FOUND
- ferro-cli/tests/golden.rs — FOUND
- ferro-cli/tests/fixtures/gestiscilo/expected/Dockerfile — FOUND
- ferro-cli/tests/fixtures/gestiscilo/expected/app.yaml — FOUND
- ferro-cli/tests/fixtures/mkmenu/expected/Dockerfile — FOUND
- ferro-cli/tests/fixtures/mkmenu/expected/app.yaml — FOUND
- commit c73771d0 — FOUND
- commit fb5b423a — FOUND
