---
phase: 190-async-rule-infrastructure-unique-rule
plan: "02"
subsystem: validation
tags: [async-validation, unique-rule, sea-orm, sql-injection-guard, sqlite, postgres]
dependency_graph:
  requires:
    - AsyncRule trait (framework/src/validation/async_rule.rs) — Plan 01
    - Wave 0 SQLite test fixture (framework/tests/async_rule_fixture.rs) — Plan 01
  provides:
    - Unique rule struct + unique() constructor (framework/src/validation/rules_async.rs)
    - json_value_to_sea_value helper (pub(crate), same file)
    - mod rules_async; declaration in validation/mod.rs
  affects:
    - framework/src/validation/mod.rs
tech_stack:
  added: []
  patterns:
    - "Per-backend SQL branching: DatabaseBackend::Postgres → $1/$2, _ → ?"
    - "__infra_error__: sentinel for DB/infra failures (D-12 contract)"
    - "Identifier guard [A-Za-z0-9_] before any DB access (T-190-01)"
    - "DB::connection() singleton access via Deref to DatabaseConnection"
    - "translate_validation + English fallback (mirrors all existing rules)"
    - "Inline fixture helpers in #[cfg(test)] using crate:: paths"
key_files:
  created:
    - framework/src/validation/rules_async.rs
  modified:
    - framework/src/validation/mod.rs
decisions:
  - "Inlined fixture helpers (init_test_db/seed_widget) using crate:: paths instead of #[path] include — #[path] in a nested mod tests block resolves from a virtual subdirectory, making the relative path three levels wrong"
  - "Added #![allow(dead_code)] to rules_async.rs — same pattern as async_rule.rs in Plan 01; Plan 04 adds the pub use re-exports that make the symbols reachable"
  - "build_sql extracted as a private associated fn — enables unit-testable Postgres placeholder path without a live Postgres instance"
metrics:
  duration: "327s"
  completed: "2026-06-09"
  tasks_completed: 2
  files_changed: 2
requirements: [VALID-01, VALID-02]
---

# Phase 190 Plan 02: Unique Async Rule Summary

`Unique` async rule: parameterized `SELECT COUNT(*)` uniqueness check with per-backend placeholder handling, `.ignore()`/`.ignore_on()` exclude-self, identifier injection guard, and `validation.unique` localized message with English fallback.

## What Was Built

**Task 1:** `framework/src/validation/rules_async.rs` — `Unique` struct, `unique(table, col)` constructor, `.ignore(id)` (default PK `"id"`) and `.ignore_on(pk_col, id)` consuming builders, `validate_identifier()` guard (`[A-Za-z0-9_]+`), `quote_ident()` double-quoting helper, and `pub(crate) json_value_to_sea_value()` conversion helper. All struct-level and helper unit tests pass (8 pure-unit tests: identifier accept/reject, quote_ident, builder state, value conversion).

Rustdoc on `Unique` documents both the T-190-01 trust boundary (developer-controlled identifiers, defense-in-depth guard) and the system-wide-only scope limitation (no per-tenant/`.where_eq` scoping — documented as a follow-up).

`framework/src/validation/mod.rs` receives one additive `mod rules_async;` line. No `pub use` — Plan 04 scope.

**Task 2:** `AsyncRule` impl for `Unique` in the same file. `build_sql(backend, table, col)` private fn produces per-backend SQL — `$1`/`$2` for Postgres, `?` for SQLite/MySQL — verified by two pure-unit string-assertion tests without a live Postgres. `validate()` runs identifier guards (T-190-01) before any `DB::connection()` call, maps `DB::connection()` and `query_one` errors to the `__infra_error__:` sentinel (D-12), binds value and pk_val as `sea_orm::Value` parameters (T-190-03), and returns `translate_validation("validation.unique", &[("attribute", field)])` with English fallback on duplicate.

Six additional tests: 2 guard tests (no DB — prove short-circuit before any DB access), 4 `#[serial]` DB-backed tests (detect duplicate, pass on free value, ignore own row, ignore via custom PK column).

## Verification

- `cargo test -p ferro-rs --lib validation::` — 62 passed, 0 failed (includes all 14 new rules_async tests)
- `cargo clippy -p ferro-rs --all-targets -- -D warnings` — clean
- `cargo fmt --all -- --check` — clean
- `mod.rs` has single additive `mod rules_async;`, no `pub use`
- `rules_async.rs` contains `impl AsyncRule for Unique`, `__infra_error__:`, `DatabaseBackend::Postgres`, `fn build_sql`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed #[path] fixture include — wrong virtual path in nested mod**
- **Found during:** Task 2 test compilation
- **Issue:** `#[path = "../../../tests/async_rule_fixture.rs"] mod fixture;` inside `#[cfg(test)] mod tests` resolves from a virtual directory `rules_async/tests/`, not from the file's actual location. The compiler error was: `couldn't read framework/src/validation/rules_async/tests/../../../tests/async_rule_fixture.rs`.
- **Fix:** Inlined `init_test_db()` and `seed_widget()` directly in the test module using `crate::` paths (`crate::database::{DatabaseConfig, DB}`). Functionally identical to the Plan 01 fixture; the inline approach is the correct pattern for unit tests in lib source files.
- **Files modified:** `framework/src/validation/rules_async.rs`
- **Commit:** fdd37228

**2. [Rule 1 - Bug] Suppress dead_code lint (same pattern as Plan 01)**
- **Found during:** Task 1/2 clippy gate
- **Issue:** `pub fn unique`, `pub fn ignore`, `pub fn ignore_on` are declared in a private module with no `pub use` re-export yet. Clippy `-D warnings` promotes `dead_code` to error.
- **Fix:** Added `#![allow(dead_code)]` at top of `rules_async.rs`. Intentionally temporary — removed when Plan 04 adds `pub use rules_async::unique;`.
- **Files modified:** `framework/src/validation/rules_async.rs`
- **Commit:** fdd37228

**3. [Rule 3 - Formatting] Apply cargo fmt**
- **Found during:** Pre-commit fmt check
- **Issue:** Line length violations in `build_sql` match arms and test assertions.
- **Fix:** Applied `cargo fmt --all`.
- **Files modified:** `framework/src/validation/rules_async.rs`
- **Commit:** fdd37228

## Known Stubs

None. `Unique` performs a real DB query against the live `DB::connection()` singleton. No hardcoded counts or placeholder return values.

## Threat Surface Scan

No new network endpoints or auth paths. `Unique::validate` introduces a new SQL execution path:

- **T-190-01 (mitigated):** `table`/`col`/`pk_col` identifiers are validated against `[A-Za-z0-9_]+` and rejected before any DB access. Double-quoted via `quote_ident`. Guard test proves short-circuit: `unique("bad;name", "slug")` never reaches `DB::connection()`.
- **T-190-02 (mitigated):** `DB::connection()` and `query_one` errors surface as `__infra_error__:` prefix strings, never swallowed as passes or emitted as field messages.
- **T-190-03 (mitigated):** The checked value and pk exclusion value are bound as `sea_orm::Value` parameters via `Statement::from_sql_and_values`, never interpolated.

## Self-Check: PASSED

| Item | Status |
|------|--------|
| framework/src/validation/rules_async.rs | FOUND |
| framework/src/validation/mod.rs | FOUND (mod rules_async; present, no pub use) |
| commit fdd37228 | FOUND |
| 14 rules_async tests pass | VERIFIED (62/62 total) |
| clippy clean | VERIFIED |
| fmt clean | VERIFIED |
