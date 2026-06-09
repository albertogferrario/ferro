---
phase: 190-async-rule-infrastructure-unique-rule
plan: "01"
subsystem: validation
tags: [async-validation, trait-definition, test-infrastructure, sqlite]
dependency_graph:
  requires: []
  provides:
    - AsyncRule trait (framework/src/validation/async_rule.rs)
    - Wave 0 SQLite test fixture (framework/tests/async_rule_fixture.rs)
  affects:
    - framework/src/validation/mod.rs
tech_stack:
  added: []
  patterns:
    - "#[async_trait] dyn-compatible trait (mirrors TenantLookup pattern)"
    - "DB::init_with + sqlite::memory: for isolated test fixture"
key_files:
  created:
    - framework/src/validation/async_rule.rs
    - framework/tests/async_rule_fixture.rs
  modified:
    - framework/src/validation/mod.rs
decisions:
  - "Additive mod async_rule declaration only — pub use deferred to Plan 04 to avoid mod.rs conflicts across plans 01-03"
  - "Added #![allow(dead_code)] to async_rule.rs — trait has no implementors until Plans 02/03; removed when Plan 04 adds pub use re-export"
  - "Test fixture uses DatabaseConfig::builder().url(sqlite::memory:).build() — exact real DB init path verified from database/config.rs"
metrics:
  duration: "214s"
  completed: "2026-06-09"
  tasks_completed: 2
  files_changed: 3
requirements: [VALID-03]
---

# Phase 190 Plan 01: AsyncRule Trait + Wave 0 Test Fixture Summary

AsyncRule trait definition and in-memory SQLite test fixture for DB-backed async validation infrastructure.

## What Was Built

**Task 1:** `framework/src/validation/async_rule.rs` — the `AsyncRule` trait using `#[async_trait]` for dyn-compatibility. Mirrors the sync `Rule` trait exactly: `async fn validate(&self, field, value, data) -> Result<(), String>` plus `fn name() -> &'static str`. Documents the `__infra_error__:` sentinel contract (D-12) that Plan 02 (`Unique`) emits and Plan 03 (`AsyncValidator`) consumes. No `impl` blocks — trait only.

`framework/src/validation/mod.rs` receives one additive `mod async_rule;` line. No `pub use` — that is Plan 04's scope, keeping `mod.rs` edits non-conflicting across the four plans.

**Task 2:** `framework/tests/async_rule_fixture.rs` — a shared test helper exposing `init_test_db()` and `seed_widget()`. `init_test_db` calls `DB::init_with(DatabaseConfig::builder().url("sqlite::memory:").build())`, then creates a `widgets(id, slug)` scratch table via `CREATE TABLE IF NOT EXISTS`. `seed_widget` inserts a row. Downstream plans (02 `Unique`, 03 `AsyncValidator`) include this file and annotate tests `#[serial]` because the `DB` singleton is process-global.

## Verification

- `cargo check -p ferro-rs --lib` exits 0
- `cargo check -p ferro-rs --tests` exits 0
- `cargo fmt --all -- --check` exits 0
- `cargo clippy --all --all-targets -- -D warnings` exits 0
- `async_rule.rs` contains `#[async_trait]`, `pub trait AsyncRule: Send + Sync`, `__infra_error__`, zero `impl` blocks
- `mod.rs` contains `mod async_rule;`, no `pub use async_rule`
- `async_rule_fixture.rs` contains `init_test_db`, `DB::init_with`, `widgets` table

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Suppress dead_code lint on AsyncRule trait**
- **Found during:** Task 1 clippy gate
- **Issue:** `pub trait AsyncRule` is declared in a private module (`mod async_rule`) with no `pub use` re-export yet (Plan 04 adds those). Clippy `-D warnings` promoted the `dead_code` warning to an error.
- **Fix:** Added `#![allow(dead_code)]` at the module level of `async_rule.rs`. This suppression is intentionally temporary — Plan 04's `pub use async_rule::AsyncRule;` in `mod.rs` will make the trait reachable from the crate root, at which point the `#![allow]` can be removed.
- **Files modified:** `framework/src/validation/async_rule.rs`
- **Commit:** d8c91d73

**2. [Rule 3 - Formatting] Apply cargo fmt to new files**
- **Found during:** Task 1/2 pre-commit gate
- **Issue:** `async_rule.rs` and `async_rule_fixture.rs` had multi-line signatures that rustfmt collapsed to single lines.
- **Fix:** Applied `cargo fmt --all`.
- **Files modified:** `framework/src/validation/async_rule.rs`, `framework/tests/async_rule_fixture.rs`
- **Commit:** d8c91d73

## Known Stubs

None. This plan ships trait infrastructure only — no data flow stubs.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes. The `AsyncRule` trait introduces no SQL and no end-user-reachable input path. T-190-01 (SQL injection via identifiers) is mitigated at its source in Plan 02's `Unique` implementation.

## Self-Check: PASSED

| Item | Status |
|------|--------|
| framework/src/validation/async_rule.rs | FOUND |
| framework/tests/async_rule_fixture.rs | FOUND |
| framework/src/validation/mod.rs | FOUND |
| commit a838c07d (AsyncRule trait) | FOUND |
| commit 30b7fa49 (test fixture) | FOUND |
| commit d8c91d73 (fmt + clippy fix) | FOUND |
