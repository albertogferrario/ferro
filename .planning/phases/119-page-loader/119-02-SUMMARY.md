---
phase: 119
plan: 02
subsystem: ferro-json-ui
tags: [loader, cache, spec, json-ui, page-loader]
requirements: [LOAD-01, LOAD-03]

dependency_graph:
  requires:
    - ferro-json-ui/src/spec.rs (Spec::from_json, SpecError)
    - ferro-json-ui/src/catalog.rs (global_catalog, CatalogError, Catalog::validate)
  provides:
    - ferro-json-ui/src/loader.rs (LoadError, load_cached, global spec cache)
    - ferro-json-ui::load_cached (re-exported from lib.rs)
    - ferro-json-ui::LoadError (re-exported from lib.rs)
  affects:
    - framework/src/json_ui/mod.rs (Plan 03 will consume load_cached and LoadError)

tech_stack:
  added: []
  patterns:
    - OnceLock<RwLock<HashMap<PathBuf, (Arc<Spec>, SystemTime)>>> global cache singleton
    - thiserror derive on LoadError with three variants (Io, Parse, Catalog)
    - Per-request fs::metadata().modified() for dev-mode mtime invalidation
    - Parse + validate outside write lock (no fallible code inside the lock guard)

key_files:
  created:
    - ferro-json-ui/src/loader.rs (271 lines including 6 unit tests)
  modified:
    - ferro-json-ui/src/lib.rs (pub mod loader; + pub use loader::{load_cached, LoadError};)

decisions:
  - Catalog variant on LoadError uses map_err(LoadError::Catalog) not #[from] because Vec<CatalogError> does not implement std::error::Error
  - Tests use a static AtomicU64 counter for unique tempfile names instead of the tempfile crate (no new dependencies permitted)
  - Sleep of 1100ms in dev_mode_invalidation test to reliably advance mtime past 1-second filesystem resolution on macOS/Linux

metrics:
  duration: ~25min
  completed: "2026-04-21"
  tasks_completed: 2
  files_created: 1
  files_modified: 1
---

# Phase 119 Plan 02: Loader Module Summary

One-liner: `load_cached(&Path, bool) -> Result<Arc<Spec>, LoadError>` with OnceLock<RwLock<HashMap>> spec cache and mtime-based dev-mode invalidation.

## What Was Built

### Task 1: ferro-json-ui/src/loader.rs (271 lines)

New file providing the file-loading pipeline for Phase 119:

- `LoadError` — `thiserror::Error` enum with three variants:
  - `Io(#[from] std::io::Error)` — file missing, unreadable, or canonicalize failure
  - `Parse(#[from] SpecError)` — invalid JSON or structural spec validation failure
  - `Catalog(Vec<CatalogError>)` — catalog validation failure (no `#[from]` — `Vec<CatalogError>` does not implement `std::error::Error`)

- `SPEC_CACHE: OnceLock<RwLock<SpecCache>>` — process-level singleton following the exact pattern of `GLOBAL_REGISTRY` in `layout.rs` and `GLOBAL_CATALOG` in `catalog.rs`

- `load_cached(path: &Path, reload_if_changed: bool) -> Result<Arc<Spec>, LoadError>`:
  1. `fs::canonicalize(path)` — normalize path for cache key, fail with `Io` if missing
  2. Read lock fast path — returns `Arc::clone` on hit
  3. Dev-mode: checks `fs::metadata().modified()` against cached mtime; falls through if mtime advanced
  4. `fs::read_to_string` + `Spec::from_json` + `global_catalog().validate()` — all outside any lock
  5. Write lock insert of `(Arc<Spec>, mtime)` — only the bare insert runs under the lock

### Task 2: ferro-json-ui/src/lib.rs (2 lines added)

- `pub mod loader;` inserted between `layout` and `plugin` (alphabetical order)
- `pub use loader::{load_cached, LoadError};` added to the public re-export block

## Test Outcomes (6/6 pass)

All 6 loader tests pass when run in isolation (`cargo test -p ferro-json-ui --lib loader::tests`):

| Test | Outcome | What it verifies |
|------|---------|-----------------|
| `load_spec_valid` | PASS | Valid v2 spec loads and returns Arc<Spec> with correct root |
| `load_spec_invalid_json` | PASS | Invalid JSON returns LoadError::Parse |
| `load_spec_catalog_error` | PASS | Unknown component type returns LoadError::Catalog with non-empty errors |
| `load_spec_missing_file` | PASS | Non-existent path returns LoadError::Io |
| `cache_hit` | PASS | Second call with reload_if_changed=false returns Arc::ptr_eq to first |
| `dev_mode_invalidation` | PASS | After 1100ms sleep + file rewrite, second call with reload_if_changed=true returns new Arc with updated content |

### Mtime Resolution Note

The `dev_mode_invalidation` test sleeps 1100ms between writes. This is intentional: macOS APFS and Linux ext4 both record mtime to 1-second resolution by default. A shorter sleep produces flaky results. The 1100ms budget was sufficient in all observed runs.

## Pre-existing Test Suite Issue (Not Caused by This Plan)

When the full ferro-json-ui test suite is run in parallel, the loader tests that call `global_catalog()` may fail if `catalog::tests::build_discovers_plugins_and_rejects_invalid_schema` has already registered `BadPlugin_117` into the global plugin registry before `GLOBAL_CATALOG` is first initialized. This is a pre-existing design issue with the catalog test suite (5 catalog tests also fail for the same reason when run single-threaded). It is not caused by changes in this plan and is out of scope per the scope boundary rule.

The plan's acceptance criteria (`cargo test -p ferro-json-ui --lib loader::tests`) are satisfied.

## Dependency Confirmation

No new dependencies were added to `ferro-json-ui/Cargo.toml`. All implementation uses:
- `std::fs`, `std::path`, `std::sync::{Arc, OnceLock, RwLock}`, `std::time::SystemTime` — stdlib
- `thiserror` — already present in Cargo.toml
- `crate::catalog::{global_catalog, CatalogError}` — intra-crate
- `crate::spec::{Spec, SpecError}` — intra-crate

## Deviations from Plan

None — plan executed exactly as written. Implementation matches the exact code provided in the task action block. Cargo fmt reformatted two style differences (pub use ordering in lib.rs and a long match arm in the catalog error test).

## Self-Check: PASSED

- ferro-json-ui/src/loader.rs: FOUND
- ferro-json-ui/src/lib.rs: FOUND (modified)
- .planning/phases/119-page-loader/119-02-SUMMARY.md: FOUND
- Commit 4db62220 (Task 1 - loader.rs): FOUND
- Commit 987c5482 (Task 2 - lib.rs): FOUND
