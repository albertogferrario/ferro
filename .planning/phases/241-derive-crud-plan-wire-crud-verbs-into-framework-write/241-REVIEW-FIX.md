---
phase: 241-derive-crud-plan-wire-crud-verbs-into-framework-write
fixed_at: 2026-06-24T00:00:00Z
review_path: .planning/phases/241-derive-crud-plan-wire-crud-verbs-into-framework-write/241-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 3
skipped: 1
status: partial
---

# Phase 241: Code Review Fix Report

**Fixed at:** 2026-06-24
**Source review:** `.planning/phases/241-derive-crud-plan-wire-crud-verbs-into-framework-write/241-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope (WR-*): 4
- Fixed: 3 (WR-01, WR-02, WR-04)
- Skipped: 1 (WR-03 — deliberate deferral, see below)

Note: WR-03 is an intentional skip, not a failure. The three actionable warnings (WR-01, WR-02, WR-04) all landed cleanly.

## Fixed Issues

### WR-01: Post-update SELECT missing soft-delete predicate

**Files modified:** `framework/src/write/mod.rs`
**Commit:** `af1b85df`
**Applied fix:** Added `AND {soft_delete_column} IS NULL` to the post-update `SELECT` in `execute_crud_plan`'s `CrudPlan::Update` arm. The format string now reads `SELECT * FROM {table} WHERE {id_column} = {id_ph2} AND {soft_delete_column} IS NULL`, preventing a concurrent soft-delete between UPDATE and SELECT from returning a deleted record to the caller or audit log. Added a comment citing the guard invariant.

---

### WR-02: Missing/null `id` silently deferred to SQL semantics on Update and Delete

**Files modified:** `ferro-projections/src/executor.rs`
**Commit:** `e2d6ade7` (fix), `fea573f8` (fmt)
**Applied fix:** In `derive_crud_plan`, both the `CrudVerb::Update` and `CrudVerb::Delete` arms now use `.get("id").filter(|v| !v.is_null()).cloned().ok_or_else(|| crate::Error::Validation(...))` instead of `unwrap_or(Value::Null)`. A missing or explicitly null `id` returns `crate::Error::Validation` immediately at derivation time. The `Error::Validation(String)` variant was confirmed present in `ferro-projections/src/error.rs`. Two new unit tests added:
- `executor::tests::derive_crud_plan_update_missing_id_is_validation_error` — asserts both absent and null id on Update
- `executor::tests::derive_crud_plan_delete_missing_id_is_validation_error` — asserts both absent and null id on Delete

---

### WR-04: Guard pre-check loop absent on CRUD delete arm of `handle_request_confirm`

**Files modified:** `ferro-mcp-server/src/write_dispatch.rs`
**Commit:** `1b9efae3`
**Applied fix:** Inserted an explicit guard pre-check loop immediately after the `svc` lookup in the CRUD delete arm of `handle_request_confirm`, mirroring the structure of the transition-action path (lines 474-491). The loop iterates over `crud_guards: Vec<String>` (currently empty — Phase 241 synthesized CRUD verbs carry no preconditions), so it is a correct no-op. Uses the same `dispatcher.guard_evaluator` call pattern and `guard_denied` error envelope as the transition path. Phase 242 populates `crud_guards` from the service's preconditions when `mcp_write_ability`/per-record guards are wired. The `let _ = svc` suppressor is preserved since `svc` is used for the `.deletable + .mcp_exposed` lookup above the loop.

---

## Skipped Issues

### WR-03: Unbounded string length on `CrudPlan::Create` columns

**File:** `ferro-projections/src/executor.rs:224-229`
**Reason:** Deliberate deferral — not a failure to fix. The correct fix belongs at the schema layer (`FieldDef.max_length`) as a deliberate design decision, or as a `ServiceDef`-level constraint; bolting a magic-number byte cap into the executor would introduce an unjustified new control-surface knob with no defined requirement. Values are parameter-bound (no SQL injection risk). The `idempotency_key` 128-char cap precedent cited in the review applies to a protocol field, not to application data columns. No Phase-241/242 requirement defines a length cap. Deferred to the phase or design session that introduces `FieldDef.max_length`.

---

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| fmt | `cargo fmt --all` | Clean (minor line-wrap reformat in executor.rs test, committed as `fea573f8`) |
| clippy | `cargo clippy --all --all-targets -- -D warnings` | Clean — 0 warnings, 0 errors. `CrudPlan`'s `PartialEq`-without-`Eq` did not trigger `clippy::derive_partial_eq_without_eq` (no annotation needed; IN-01 remains an informational note only) |
| tests | `cargo test --all-features` | All suites pass — 0 failures across full workspace, including both new WR-02 tests |

Post-test tree: `docs/protocol/schemas/protocol.json`, `docs/protocol/schemas/service-def.json`, `Cargo.lock`, `.planning/config.json` are dirty (Phase-94 schema regeneration artifacts + lock churn) — NOT committed.

---

_Fixed: 2026-06-24_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
