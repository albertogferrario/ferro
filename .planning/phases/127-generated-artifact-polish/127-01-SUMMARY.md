---
phase: 127-generated-artifact-polish
plan: 01
subsystem: ferro-cli/deploy
tags: [deploy, docker, digitalocean, helpers, toml_edit]
requires:
  - ferro-cli 0.2.0 existing deploy scaffold surface
provides:
  - "crate::deploy::bin_detect::detect_web_bin"
  - "crate::deploy::secret_keys::is_secret_key"
  - "crate::deploy::env_production::{EnvLine, parse_env_example_structured}"
  - "toml_edit-based rewrite_ferro_version preserving dep-table order"
affects:
  - ferro-cli/src/commands/do_init.rs (delegates to detect_web_bin)
  - ferro-cli/src/project.rs (adds FerroDeployMetadata.web_bin)
tech-stack:
  added:
    - toml_edit = "0.22"
  patterns:
    - structural Cargo.toml edits via DocumentMut (no value round-trip)
key-files:
  created:
    - ferro-cli/src/deploy/bin_detect.rs
    - ferro-cli/src/deploy/secret_keys.rs
  modified:
    - ferro-cli/Cargo.toml
    - ferro-cli/src/deploy/mod.rs
    - ferro-cli/src/deploy/env_production.rs
    - ferro-cli/src/deploy/rewrite_ferro_version.rs
    - ferro-cli/src/project.rs
    - ferro-cli/src/commands/do_init.rs
decisions:
  - "FerroDeployMetadata gained web_bin: Option<String> (D-02 explicit override slot)"
  - "Secret heuristic lives in deploy/secret_keys.rs, reusable by Phase 128 preflight"
  - "rewrite_ferro_version exposes pure rewrite_contents(&str,...) for easier testing"
metrics:
  duration: ~25min
  completed: 2026-04-08
---

# Phase 127 Plan 01: Deploy Scaffold Helpers Summary

Foundation helpers for the Phase 127 deploy artifact polish: shared web-bin
detection, a secret-shaped env key classifier, a structured `.env.example`
parser, and a `toml_edit`-based Cargo.toml rewriter that preserves dep-table
key order across the path→version transform.

## New helper signatures

```rust
// ferro-cli/src/deploy/bin_detect.rs
pub fn detect_web_bin(project_root: &Path) -> anyhow::Result<String>;
```
Resolves the web bin per D-02's 4-step order: explicit
`[package.metadata.ferro.deploy].web_bin` → bin matching `package.name` →
first declared `[[bin]]` → `package.name` fallback.

```rust
// ferro-cli/src/deploy/secret_keys.rs
pub fn is_secret_key(key: &str) -> bool;
```
Case-insensitive substring match against
`{secret, password, passwd, token, key, api_key, dsn, private, credential}`.
Keys ending in `_URL` are non-secret unless a substring also matches the
portion before the trailing `_url`.

```rust
// ferro-cli/src/deploy/env_production.rs
pub enum EnvLine { Key(String), Blank, Comment }
pub fn parse_env_example_structured(contents: &str) -> Vec<EnvLine>;
```
Preserves source order and blank-line separators so the generated
`.do/app.yaml` envs block keeps human grouping (D-09).

## toml_edit migration

`rewrite_ferro_version.rs` now parses into `DocumentMut`, walks the three
dep tables (`dependencies`, `dev-dependencies`, `build-dependencies`), finds
every ferro* path dep, removes its `path` field, and inserts/overwrites its
`version` field. Sibling fields — `package`, `features`,
`default-features`, `optional`, `registry`, `rename` — are left in place by
construction: `toml_edit` edits the document tree in place rather than
round-tripping through a value model, so key order and whitespace survive
byte-for-byte. The `preserves_package_rename_and_features` regression
(gestiscilo's `package = "ferro-rs"` case) continues to pass unchanged.

A new `preserves_dep_table_order` test asserts that a six-dep
`[dependencies]` block with mixed ordering
(`zed, alpha, ferro, middle, beta, gamma`) survives the rewrite with keys
in the original order.

The rewriter is now split into `rewrite_cargo_docker_toml` (filesystem
entry point) and `rewrite_contents(&str, &Path, Option<&str>) -> Result<String>`
(pure string-in/string-out core), making future tests that need inline
fixtures straightforward.

## do_init delegation

`ferro-cli/src/commands/do_init.rs` no longer inlines bin detection. It now
calls `crate::deploy::bin_detect::detect_web_bin(&root)?` and uses the
result to filter out the web bin from the worker list. The explicit
override path — setting
`[package.metadata.ferro.deploy] web_bin = "custom"` in `Cargo.toml` —
now works automatically via the shared helper and will behave identically
for `docker:init` once Plan 127-02 wires the same helper into the
Dockerfile template renderer.

## DeployMetadata.web_bin

`FerroDeployMetadata` gained a `web_bin: Option<String>` field, parsed
from `[package.metadata.ferro.deploy]` alongside the existing
`runtime_apt`, `copy_dirs`, and `ferro_version` fields. The field is the
D-02 step-1 override slot.

## Known stubs / intentional carve-outs

**`SLACK_WEBHOOK_URL` classifies as non-secret.** The `_URL` carve-out
trumps the lack of other substring hits, so plain webhook URL envs fall
through as non-secret. This is shipped intentionally per Plan 127-01's
`<behavior>` note — tightening the heuristic (e.g. by adding `webhook`
to the vocabulary, or by requiring an explicit allow-list of non-secret
`_URL` keys) is out of scope for Phase 127 and deferred to Phase 128
preflight where the classifier will be reused.

## Deviations from Plan

**1. [Rule 3 — Blocker] Project API shape**
- **Found during:** Task 1
- **Issue:** Plan referenced a `Project` struct with `package_name()` /
  `bin_names()` / `deploy_metadata()` methods; actual ferro-cli exposes
  free functions `package_name(root)`, `read_bins(root)`, and
  `read_deploy_metadata(root)` with no `Project` struct.
- **Fix:** Implemented `detect_web_bin(project_root: &Path)` against the
  real free-function surface using `crate::templates::docker::read_bins`
  (the one already consumed by `do_init.rs`).
- **Files modified:** `ferro-cli/src/deploy/bin_detect.rs`
- **Commit:** `08090ab1`

**2. [Rule 2 — Missing field] `FerroDeployMetadata.web_bin` did not exist**
- **Found during:** Task 1
- **Issue:** Plan assumed `DeployMetadata.web_bin: Option<String>` was
  either present or trivially addable; the actual struct is
  `FerroDeployMetadata` and lacked the field. Without adding it, D-02
  step 1 (explicit override) is unreachable.
- **Fix:** Added `web_bin: Option<String>` to `FerroDeployMetadata` with
  string-type validation in `read_deploy_metadata`.
- **Files modified:** `ferro-cli/src/project.rs`
- **Commit:** `08090ab1`

## Deferred Issues

**Full `cargo test --all-features` could not run in this session**
because `/` has only ~1 GB free and the workspace's transitive
`async-stripe → aws-lc-sys` build spawns C compilations that exhaust
the disk mid-link. This is a pre-existing environmental blocker
unrelated to Plan 127-01 changes (no ferro-stripe code touched).
Scoped verification that DID pass:

- `cargo test -p ferro-cli --all-features` — **441 passed, 0 failed**
  (includes the 4 `bin_detect_*`, 12 `is_secret_key_*`, 4
  `env_example_parser_*`, and 7 `rewrite_ferro_version` tests)
- `cargo clippy --all --all-targets -- -D warnings` — **clean**
- `cargo fmt --all -- --check` — **clean**

Recommend freeing disk on the host before running the next Phase 127
plan so full-workspace `cargo test --all-features` can be invoked.
Tracked here rather than in the rewriter since the blocker is disk,
not code.

## Verification trace

| Decision | Test | Result |
|----------|------|--------|
| D-02 bin detection 4-step order | `bin_detect_{explicit_override,package_name_match,first_bin_fallback,no_bins_uses_package_name}` | ✅ 4/4 |
| D-08 secret heuristic + _URL carve-out | `is_secret_key_*` (12 cases) | ✅ 12/12 |
| D-09 structured env parser | `env_example_parser_{preserves_order,preserves_blank_separators,skips_comments,trims_keys}` | ✅ 4/4 |
| D-11 dep-table key order preserved | `preserves_dep_table_order` (6 deps) | ✅ |
| D-12 existing rewriter regression | `preserves_package_rename_and_features` + 4 others | ✅ 5/5 |

## Self-Check: PASSED

- `ferro-cli/src/deploy/bin_detect.rs` — FOUND
- `ferro-cli/src/deploy/secret_keys.rs` — FOUND
- `grep -q 'toml_edit' ferro-cli/Cargo.toml` — FOUND
- `grep -q 'pub mod bin_detect;' ferro-cli/src/deploy/mod.rs` — FOUND
- `grep -q 'pub mod secret_keys;' ferro-cli/src/deploy/mod.rs` — FOUND
- `grep -q 'pub enum EnvLine' ferro-cli/src/deploy/env_production.rs` — FOUND
- commit `08090ab1` (Task 1) — FOUND
- commit `605c951b` (Task 2) — FOUND
- commit `2f7a3acd` (Task 3) — FOUND
