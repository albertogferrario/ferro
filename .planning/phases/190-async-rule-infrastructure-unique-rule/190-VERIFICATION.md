---
phase: 190-async-rule-infrastructure-unique-rule
verified: 2026-06-09T08:00:00Z
status: passed
score: 5/5
overrides_applied: 0
---

# Phase 190: Async Rule Infrastructure + `unique` Rule — Verification Report

**Phase Goal:** Developers can validate field uniqueness against the DB before insert/update, with exclude-self for edit forms, through a new async validation path that leaves the existing sync API untouched.
**Verified:** 2026-06-09T08:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A handler can call `AsyncValidator::new().async_rule("slug", unique(...)).validate_async()` and get a field-level error on duplicate — not a raw SQL error or 500 | VERIFIED | `duplicate_value_is_validation_error` integration test asserts `AsyncValidationError::Validation(e)` with `e.has("slug")`; rules_async.rs line 160 returns translated field message; WR-02 fix ensures COUNT read failure maps to `__infra_error__` sentinel, not swallowed to 0 |
| 2 | `.ignore(record_id)` exclude-self does not reject an unchanged slug on edit | VERIFIED | `exclude_self_passes_on_edit` integration test + `unique_ignore_excludes_self` unit test in rules_async.rs; `ignore()` sets `("id", pk_val)` in the ignore tuple; SQL appends `AND "id" <> ?` |
| 3 | `validate_async()` runs sync rules first and skips async rules on fields with sync errors | VERIFIED | `async_validator.rs` lines 240-244: Phase 2 loop checks `errors.has(field)` and `continue`s; `sync_failure_skips_async` integration test + `async_validator_skips_async_on_sync_error` unit test with CountingRule asserts counter remains 0 |
| 4 | Existing `Validator` / `validate()` sync API unchanged and compiles with no modifications | VERIFIED | `git log` shows `validator.rs` last touched at commit `a80b3b33` (pre-v12.4 era, not in phase 190 commits `a838c07d`–`eb67dea7`); no phase 190 commit touches `validator.rs`; it is not listed in any plan's `files_modified` |
| 5 | `DB::connection()` is the access pattern inside `Unique` — no DB connection threaded through rule signature or `validate_async()` | VERIFIED | `rules_async.rs` line 133: `let db = DB::connection().map_err(...)?;` inside `Unique::validate`; `validate_async()` signature is `pub async fn validate_async(self) -> Result<(), AsyncValidationError>` — no connection parameter anywhere in the call chain |

**Score:** 5/5 truths verified

---

### Code Review Fix Verification (commit `eb67dea7`)

Three warnings from the code review were addressed. Confirming the fixes are present in the codebase:

| Fix | Item | Status | Evidence |
|-----|------|--------|----------|
| WR-01 | `Value::Null` binds as SQL NULL (`String(None)`), not string `"null"` | VERIFIED | `rules_async.rs` line 192: `serde_json::Value::Null => sea_orm::Value::String(None)`; test `json_value_to_sea_value_null_binds_sql_null` asserts `String(None)` |
| WR-02 | COUNT column read failure maps to `__infra_error__` sentinel, not swallowed to 0 | VERIFIED | `rules_async.rs` lines 152-156: `match row { Some(r) => r.try_get(...).map_err(|e| format!("__infra_error__: {e}"))?, None => return Err("__infra_error__: uniqueness COUNT returned no row"...`) }` — no `unwrap_or(0)` remains |
| WR-03 | `From<AsyncValidationError> for ActionError` lossy impl removed | VERIFIED | `async_validator.rs` lines 52-58: only a comment block explaining WHY the impl is absent; grep confirms no `impl From<AsyncValidationError>` block exists |

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `framework/src/validation/async_rule.rs` | AsyncRule trait (`#[async_trait]`, `Send + Sync`, `__infra_error__` sentinel documented) | VERIFIED | 35 lines; trait only, no impl blocks; `#[async_trait]` present; sentinel documented in rustdoc (line 18); `pub trait AsyncRule: Send + Sync` |
| `framework/src/validation/rules_async.rs` | `Unique` struct + `unique()` constructor + `.ignore()` / `.ignore_on()` + `AsyncRule` impl | VERIFIED | `Unique` struct with `ignore: Option<(String, sea_orm::Value)>`; per-backend SQL builder; identifier guard (`validate_identifier`); bound parameters via `Statement::from_sql_and_values` |
| `framework/src/validation/async_validator.rs` | `AsyncValidator` builder + `AsyncValidationError` enum + `validate_async()` run loop | VERIFIED | `AsyncValidationError` with `Validation(ValidationError)` and `Infra(FrameworkError)` variants; sync-first Phase 1 / async Phase 2 loop; D-12 `__infra_error__:` sentinel handling at lines 269-273 |
| `framework/src/validation/mod.rs` | `mod` declarations + `pub use` re-exports for all three new modules | VERIFIED | Lines 53-59: all four mod declarations present; lines 63-70: `pub use async_rule::AsyncRule`, `pub use async_validator::{AsyncValidationError, AsyncValidator}`, `pub use rules_async::unique` all present |
| `framework/src/lib.rs` | Crate-root `pub use validation::{AsyncRule, AsyncValidationError, AsyncValidator, unique, ...}` | VERIFIED | Lines 313-318 confirm `unique`, `AsyncRule`, `AsyncValidationError`, `AsyncValidator` in the `pub use validation::{...}` block |
| `framework/tests/async_rule_fixture.rs` | `init_test_db()` + `seed_widget()` helpers for in-memory SQLite | VERIFIED | 55 lines; `DatabaseConfig::builder().url("sqlite::memory:")` + `DB::init_with`; `CREATE TABLE IF NOT EXISTS widgets`; `seed_widget` function present |
| `framework/tests/async_validation_integration.rs` | 5 end-to-end tests using public API only | VERIFIED | 152 lines; imports from `ferro_rs::{...}` only; 5 test functions: `duplicate_value_is_validation_error`, `free_value_passes`, `exclude_self_passes_on_edit`, `sync_failure_skips_async`, `redirect_back_shape`; all `#[tokio::test] #[serial]` |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `framework/src/validation/mod.rs` | `async_rule::AsyncRule` | `pub use` re-export | WIRED | Line 63 confirmed |
| `framework/src/validation/mod.rs` | `async_validator::{AsyncValidationError, AsyncValidator}` | `pub use` re-export | WIRED | Line 64 confirmed |
| `framework/src/validation/mod.rs` | `rules_async::unique` | `pub use` re-export | WIRED | Line 70 confirmed |
| `framework/src/lib.rs` | `validation::{AsyncRule, AsyncValidationError, AsyncValidator, unique}` | `pub use validation::{...}` block | WIRED | Lines 313-318 confirmed |
| `Unique::validate` | `DB::connection()` | Direct call (singleton) | WIRED | Line 133 of `rules_async.rs`; not threaded through trait or validator |
| `async_validator.rs` Phase 2 | `errors.has(field)` skip | Fail-fast before async rule call | WIRED | Lines 241-244: `if errors.has(field) { continue; }` |
| `async_validator.rs` | `__infra_error__:` sentinel → `AsyncValidationError::Infra` | `msg.strip_prefix(...)` | WIRED | Lines 269-273 |
| `framework/tests/async_validation_integration.rs` | `async_rule_fixture.rs` | `#[path = "async_rule_fixture.rs"] mod fixture;` | WIRED | Line 9-10 of integration test |

---

### Data-Flow Trace (Level 4)

Not applicable — this phase produces a validation library, not a component that renders dynamic data. The integration tests provide the behavioral data-flow proof.

---

### Behavioral Spot-Checks

| Behavior | Evidence Source | Status |
|----------|-----------------|--------|
| Duplicate slug → `Validation(e)` with field error | `duplicate_value_is_validation_error` test; WR-02 COUNT error → infra (not swallowed) | VERIFIED via test |
| Free slug → `Ok(())` | `free_value_passes` test | VERIFIED via test |
| `.ignore(1_i64)` allows own slug through | `exclude_self_passes_on_edit` + `unique_ignore_excludes_self` unit test | VERIFIED via test |
| Sync failure (`required()`) skips async rule (counter = 0) | `sync_failure_skips_async` + `async_validator_skips_async_on_sync_error` with CountingRule | VERIFIED via test |
| `with_old_input().into_action_error()` produces non-Internal ActionError | `redirect_back_shape` test; Debug string does not contain "Internal" | VERIFIED via test |
| Bad identifier rejected before DB access | `unique_rejects_bad_identifier_before_db` unit test; no `init_test_db()` called | VERIFIED via test |

Full quality gate (from 190-04-SUMMARY.md and confirmed by commit `9c311935` following `eb67dea7`):
- `cargo fmt --all -- --check` — clean
- `cargo clippy --all --all-targets -- -D warnings` — clean (0 warnings)
- `cargo test --all-features` — all test suites pass

---

### Requirements Coverage

| Requirement | Plans | Description | Status | Evidence |
|-------------|-------|-------------|--------|----------|
| VALID-01 | 190-02, 190-03, 190-04 | Async `unique(table, column)` rule produces field-level error before insert | SATISFIED | `rules_async.rs` Unique impl; integration test `duplicate_value_is_validation_error`; public at `ferro_rs::unique` |
| VALID-02 | 190-02, 190-04 | `.ignore(id)` exclude-self on edit forms | SATISFIED | `Unique::ignore()` and `ignore_on()`; `exclude_self_passes_on_edit`; `unique_ignore_excludes_self` unit test including "excluding different row still fails" assertion |
| VALID-03 | 190-01, 190-03, 190-04 | `AsyncValidator` / `validate_async` path; sync API untouched; `DB::connection()` singleton; `ValidationError` → redirect-back flow | SATISFIED | Parallel async path in new files only; `validator.rs` unmodified; `DB::connection()` in `Unique::validate`; `redirect_back_shape` test proves the redirect-back chain |

VALID-04, VALID-05, VALID-06 are out of scope for Phase 190 — assigned to Phases 191 and 192 in REQUIREMENTS.md traceability table.

No orphaned requirements: Phase 190 is mapped to exactly VALID-01, VALID-02, VALID-03 in REQUIREMENTS.md.

---

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| `framework/tests/async_rule_fixture.rs` line 50 | String interpolation in SQL (`format!("INSERT INTO widgets (id, slug) VALUES ({id}, '{slug}')")`) | Info (IN-02 from review) | Test-only fixture; `slug` is developer-controlled; string interpolation is a bad habit but does not affect production code or test correctness for the fixture values used |
| `framework/src/validation/async_validator.rs` lines 131-132 | `rules()` uses `insert()` (overwrites) while `rule()` appends — inconsistency with sync Validator | Info (IN-01 from review) | Does not affect any phase success criterion; integration tests use `.rules()` only once per field |

Both items are Info-severity, carry no blocking impact on phase goal achievement, and are present in the code review report (IN-01, IN-02) for tracking. The critical WR-01/02/03 warnings from the code review were all addressed in commit `eb67dea7`.

No stub patterns, TODOs, placeholder returns, or hardcoded empty data found in production code. No `return null`, `return []`, or `=> {}` empty handlers in any new file.

---

### Human Verification Required

None. All five success criteria are verifiable programmatically:

- SC1/SC2/SC3: integration tests exercise the runtime behavior against a real in-memory SQLite DB.
- SC4: git log confirms `validator.rs` was not modified in any phase 190 commit.
- SC5: source inspection confirms `DB::connection()` call inside `Unique::validate`; no connection parameter in trait or validator signatures.

---

### Gaps Summary

No gaps. All five ROADMAP success criteria are satisfied by the actual codebase. The code review warnings WR-01, WR-02, WR-03 were all resolved in commit `eb67dea7` before this verification. The two remaining Info items (IN-01 `rules()` overwrite inconsistency, IN-02 test fixture SQL interpolation) are non-blocking and pre-existing patterns.

---

_Verified: 2026-06-09T08:00:00Z_
_Verifier: Claude (gsd-verifier)_
