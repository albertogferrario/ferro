---
phase: 117-catalog-and-json-schema
plan: "01"
subsystem: ferro-json-ui
tags: [catalog, scaffold, jsonschema, drift-guard]
dependency_graph:
  requires: []
  provides: [ferro-json-ui/catalog.rs skeleton, jsonschema 0.46 dep]
  affects: [ferro-json-ui/src/lib.rs, ferro-json-ui/Cargo.toml]
tech_stack:
  added: [jsonschema = "0.46"]
  patterns: [OnceLock singleton, thiserror error enum, pub(crate) scaffold fields]
key_files:
  created:
    - ferro-json-ui/src/catalog.rs
  modified:
    - ferro-json-ui/Cargo.toml
    - ferro-json-ui/src/lib.rs
    - Cargo.lock
decisions:
  - "jsonschema 0.46 (NOT 0.28 per CONTEXT D-09) — RESEARCH H-1 confirmed 0.46 is current stable with validator_for() API"
  - "Added #[allow(dead_code)] to Catalog struct — fields are pub(crate) scaffold stubs; Plan 02 populates them"
  - "pub use catalog::... inserted before pub use plugin::... in lib.rs to satisfy rustfmt ordering"
metrics:
  duration: "~8 minutes"
  completed: "2026-04-18T13:11:24Z"
  tasks_completed: 1
  files_changed: 4
---

# Phase 117 Plan 01: Catalog Scaffold Summary

Scaffolded the `ferro-json-ui` catalog module: added `jsonschema = "0.46"` dependency, created `catalog.rs` with all five public type skeletons (`Catalog`, `ComponentSpec`, `CatalogError`, `global_catalog()`), wired the module into `lib.rs`, and added a drift-guard unit test that pins `BUILTIN_TYPES.len() == 39`.

## What Was Built

- **`ferro-json-ui/Cargo.toml`** — `jsonschema = { version = "0.46", default-features = false }` added. `default-features = false` excludes the `reqwest`-based remote `$ref` resolver per RESEARCH §4 / threat model T-117-01-01.
- **`ferro-json-ui/src/catalog.rs`** (151 lines) — Public type skeletons:
  - `pub struct Catalog` with five `pub(crate)` fields: `components`, `plugin_components`, `full_schema`, `per_component_schemas`, `validator: jsonschema::Validator`
  - `pub struct ComponentSpec` with five public fields
  - `pub enum CatalogError` with five `thiserror`-derived variants (`UnknownType`, `PropsInvalid`, `SpecInvalid`, `BuildFailed`, `SchemaSerialization`)
  - `pub fn global_catalog() -> &'static Catalog` via `OnceLock<Catalog>` mirroring the `global_plugin_registry` pattern
  - `Catalog::build()` stubbed as `unimplemented!("Plan 02 populates BUILTIN_SPECS and implements build")`
  - Drift-guard test: `builtin_types_count_is_39` asserts `crate::render::BUILTIN_TYPES.len() == 39`
- **`ferro-json-ui/src/lib.rs`** — `pub mod catalog;` added (alphabetically after `pub mod action;`); `pub use catalog::{global_catalog, Catalog, CatalogError, ComponentSpec};` re-export added. `pub const COMPONENT_CATALOG` retained (deleted in Plan 06).

## Verification Results

```
jsonschema = { version = "0.46", default-features = false }   ✓ in Cargo.toml
cargo tree -p ferro-json-ui | grep jsonschema                  → jsonschema v0.46.0
wc -l ferro-json-ui/src/catalog.rs                             → 151 lines
cargo test -p ferro-json-ui --lib catalog::tests::builtin_types_count_is_39  → ok
cargo clippy -p ferro-json-ui --all-targets --all-features -- -D warnings    → clean
cargo fmt --all -- --check                                      → clean
cargo test --all-features (full workspace)                      → all test result: ok
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Suppressed dead_code warning on Catalog struct fields**
- **Found during:** Part D sanity build
- **Issue:** `pub(crate)` fields on `Catalog` generated `dead_code` warnings since `build()` is `unimplemented!`. Clippy `-D warnings` would have failed.
- **Fix:** Added `#[allow(dead_code)]` to the `Catalog` struct. This is intentional scaffold suppression — Plan 02 removes it when fields are populated.
- **Files modified:** `ferro-json-ui/src/catalog.rs`

**2. [Rule 1 - Bug] Fixed rustfmt ordering of `pub use catalog::...` in lib.rs**
- **Found during:** `cargo fmt --all -- --check`
- **Issue:** Initial placement after `pub use visibility::...` violated rustfmt's use-group ordering. `catalog` sorts before `plugin` alphabetically.
- **Fix:** Moved `pub use catalog::...` to immediately before `pub use plugin::...`.
- **Files modified:** `ferro-json-ui/src/lib.rs`

## Dependency Details

```
jsonschema v0.46.0 (installed via cargo tree)
  ├── fancy-regex v0.17.0
  ├── fluent-uri v0.4.1
  ├── fraction v0.15.3
  ├── referencing v0.46.0
  └── ... (17 transitive deps added to Cargo.lock)
```

`default-features = false` excludes the HTTP-based `$ref` resolver — no `reqwest` pulled in.

## Known Stubs

| Stub | File | Reason |
|------|------|--------|
| `Catalog::build()` is `unimplemented!` | `catalog.rs:108` | Plan 02 populates `BUILTIN_SPECS` and implements build; no caller invokes it in Plan 01 |
| `global_catalog()` would panic on first call | `catalog.rs:122` | Same — `build()` is unimplemented; no test or caller in this plan invokes `global_catalog()` |

## Threat Flags

None — new surface is a Rust type scaffold with no network endpoints, auth paths, or file access. The `jsonschema` dep is pinned at `"0.46"` with `default-features = false`; Cargo.lock verifies registry checksums.

## Next Step

Plan 02: Populate `BUILTIN_SPECS` static table with all 39 entries and implement `Catalog::build()` fully. The `unimplemented!` stub and `#[allow(dead_code)]` are removed in that plan.

## Self-Check: PASSED

- `ferro-json-ui/src/catalog.rs` — FOUND
- `ferro-json-ui/Cargo.toml` — FOUND
- Commit `90d05957` — FOUND
