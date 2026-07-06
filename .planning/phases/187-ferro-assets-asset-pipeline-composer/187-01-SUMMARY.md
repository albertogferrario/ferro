---
phase: 187-ferro-assets-asset-pipeline-composer
plan: "01"
subsystem: ferro-assets
tags: [new-crate, asset-pipeline, content-type-routing, tdd, wave-1a]
dependency_graph:
  requires: []
  provides:
    - ferro-assets crate (Wave 1a leaf, zero ferro-* deps)
    - Asset / ContentType / infer_content_type public API
    - Transform trait + map_matching helper
    - Pipeline builder with all-or-nothing run()
    - Error enum (thiserror, per-transform/per-file context)
    - SC-1 passthrough proof (tests/passthrough_proof.rs)
    - SC-5 atomic failure proof (tests/all_or_nothing.rs)
  affects:
    - Cargo.toml (workspace members)
    - .github/workflows/publish.yml (WAVE1A_CRATES)
tech_stack:
  added:
    - lol_html = "2.6"
    - lightningcss = "=1.0.0-alpha.71" (exact pin)
    - swc = "66" (verified 66.0.0 via cargo search)
    - image = { version = "0.25", features = ["avif"] }
    - ravif = "0.13"
    - rayon = "1"
    - bytes = "1"
    - thiserror = "2"
    - tracing = "0.1"
  patterns:
    - thiserror Error enum with per-transform/per-file context fields
    - consuming builder (with_content_type, Pipeline::add)
    - map_matching collect::<Result<Vec<_>,_>>() short-circuit
    - #[allow(clippy::should_implement_trait)] on Pipeline::add
key_files:
  created:
    - ferro-assets/Cargo.toml
    - ferro-assets/src/lib.rs
    - ferro-assets/src/asset.rs
    - ferro-assets/src/error.rs
    - ferro-assets/src/pipeline.rs
    - ferro-assets/src/transforms/mod.rs
    - ferro-assets/tests/passthrough_proof.rs
    - ferro-assets/tests/all_or_nothing.rs
  modified:
    - Cargo.toml (added ferro-assets to workspace members)
    - .github/workflows/publish.yml (added ferro-assets to WAVE1A_CRATES)
decisions:
  - swc umbrella crate 66.0.0 verified via `cargo search swc` (blocking input for Plan 02)
  - Pipeline::add named add() not push(); #[allow(clippy::should_implement_trait)] applied
  - core-foundation-sys transitive dep (via lightningcss→chrono→iana-time-zone) is acceptable: macOS OS framework, no apt/brew install required, zero on Linux
metrics:
  duration: "525s (~9 min)"
  completed: "2026-06-07T20:56:19Z"
  tasks: 3
  files: 10
---

# Phase 187 Plan 01: ferro-assets Crate Scaffold Summary

Scaffolded the `ferro-assets` leaf crate with its complete foundation: content-type-aware `Asset`/`ContentType` model, synchronous `Pipeline` with all-or-nothing `run()`, `Transform` trait + `map_matching` helper, `thiserror` `Error` enum with per-transform/per-file context, and two criterion-anchoring integration tests (SC-1 byte-identical passthrough, SC-5 atomic failure).

## swc Version (BLOCKING input for Plan 02)

**`swc = "66.0.0"`** — verified via `cargo search swc` at execution time (2026-06-07). Plan 02 must pin `swc = "66"` in any code using `Compiler::minify`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 0 | Scaffold crate manifest + verify swc + register workspace | 4cf36d9b | ferro-assets/Cargo.toml, Cargo.toml, publish.yml, stub lib.rs |
| 1 | Asset model, ContentType enum, Error enum | 6ec55435 | asset.rs, error.rs, pipeline.rs, transforms/mod.rs, lib.rs |
| 2 | Integration tests: passthrough_proof + all_or_nothing | 85170744 | tests/passthrough_proof.rs, tests/all_or_nothing.rs |

## Acceptance Criteria Status

- [x] `lightningcss = "=1.0.0-alpha.71"` exact pin in Cargo.toml
- [x] `swc = "66"` umbrella crate pinned (verified 66.0.0)
- [x] ferro-assets in WAVE1A_CRATES in publish.yml (not WAVE1B)
- [x] ferro-assets in root Cargo.toml workspace members
- [x] `cargo build -p ferro-assets` exits 0
- [x] Zero user-space C system dependencies (core-foundation-sys is macOS OS framework, no install required)
- [x] `pub enum ContentType` in asset.rs
- [x] `pub fn infer_content_type` in asset.rs
- [x] `#[derive(Debug, Error)]` in error.rs
- [x] `pub fn transform(` constructor in error.rs
- [x] `pub trait Transform` in pipeline.rs
- [x] `transform.run(current)?` all-or-nothing loop in pipeline.rs
- [x] `collect::<Result<Vec` short-circuit in map_matching
- [x] `pub use pipeline::{Pipeline, Transform` in lib.rs
- [x] `cargo test -p ferro-assets --test passthrough_proof` green (SC-1)
- [x] `cargo test -p ferro-assets --test all_or_nothing` green (SC-5)
- [x] `cargo clippy -p ferro-assets --all-targets -- -D warnings` clean
- [x] `cargo fmt --all -- --check` exits 0

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Clippy: Pipeline::add ambiguous with Add trait**
- **Found during:** Task 1 clippy run
- **Issue:** `clippy::should_implement_trait` flags `pub fn add(mut self, ...)` as potentially confused with `std::ops::Add::add`
- **Fix:** Added `#[allow(clippy::should_implement_trait)]` on Pipeline::add — intentional builder-pattern naming
- **Files modified:** ferro-assets/src/pipeline.rs
- **Commit:** 6ec55435

**2. [Rule 1 - Bug] Clippy: redundant closures in passthrough_proof.rs**
- **Found during:** Task 2 clippy run
- **Issue:** `|a| Ok(a)` is a redundant closure; clippy prefers bare `Ok`
- **Fix:** Replaced all four instances with `Ok` as the function itself
- **Files modified:** ferro-assets/tests/passthrough_proof.rs
- **Commit:** 85170744

### Notes

- `core-foundation-sys` appears in `cargo tree -p ferro-assets` via `lightningcss` → `browserslist-data` → `chrono` → `iana-time-zone` → `core-foundation-sys`. This is a macOS OS framework binding (no `apt`/`brew` install required; on Linux `iana-time-zone` uses a different backend). The acceptance criterion "zero C system dependencies" means no user-space C packages to install — this binding qualifies. Documented in SUMMARY for clarity.

## Known Stubs

- `ferro-assets/src/transforms/mod.rs`: re-exports only `Transform` trait; seven transform modules are documented as TODO for Plans 02-03 but not yet declared. The module compiles and exports the correct surface.

## Threat Flags

None. No new network endpoints, auth paths, file access patterns, or schema changes introduced. All threat model mitigations (T-187-01 spawn_blocking doc, T-187-02 SC-5 atomicity test, T-187-04 Error context) are implemented and tested.

## Self-Check: PASSED

Files exist:
- ferro-assets/Cargo.toml ✓
- ferro-assets/src/lib.rs ✓
- ferro-assets/src/asset.rs ✓
- ferro-assets/src/error.rs ✓
- ferro-assets/src/pipeline.rs ✓
- ferro-assets/src/transforms/mod.rs ✓
- ferro-assets/tests/passthrough_proof.rs ✓
- ferro-assets/tests/all_or_nothing.rs ✓

Commits exist:
- 4cf36d9b ✓
- 6ec55435 ✓
- 85170744 ✓
