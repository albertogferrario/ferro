---
phase: 241-derive-crud-plan-wire-crud-verbs-into-framework-write
verified: 2026-06-24T08:00:00Z
status: passed
score: 4/4
overrides_applied: 0
re_verification: null
gaps: []
deferred:
  - truth: "tenant_column: Option<TenantColumn> slot is present but None on all variants (D-09 extension point)"
    addressed_in: "Phase 242"
    evidence: "Phase 242 success criteria 2: 'tenant_id injected from context on create and predicated on update/delete'"
human_verification: []
---

# Phase 241: `derive_crud_plan` + wire CRUD verbs into `framework::write` — Verification Report

**Phase Goal:** Add the CRUD analog of `derive_transition_plan` — `derive_crud_plan(svc, verb, inputs)` in `ferro-projections` producing a pure, serializable INSERT/UPDATE/soft-delete plan — and teach the EXISTING `framework::write` kernel a CRUD verb alongside the transition path, so create/update/soft-delete execute through the SAME dispatcher, override registry, idempotency, audit, and confirmation that transitions already use. The kernel is extended, never forked.
**Verified:** 2026-06-24
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `create_<svc>` inserts a row with creatable columns plus server-set `created_at` and (under SM) initial `Status`, returning the created record | VERIFIED | `execute_crud_plan` emits `INSERT INTO {table} ({cols}, created_at) VALUES (…, datetime('now'))` then fetches the row; `derive_crud_plan` pushes `status=initial_state` when SM exists; `crud_create_inserts_row` test proves returned record has `id` and DB count==1. `derive_crud_plan_create` test proves initial Status in columns and `created_at` absent from plan (executor-injected). |
| 2 | `update_<svc>` patches via `UPDATE … WHERE id=? AND deleted_at IS NULL`; `delete_<svc>` sets `deleted_at`; soft-deleted row absent from `list_<svc>` filter | VERIFIED | `execute_crud_plan` Update emits `AND {soft_delete_column} IS NULL`; Delete emits `UPDATE … SET {soft_delete_col}=now WHERE id=? AND {soft_delete_col} IS NULL`. `grep -c "IS NULL" framework/src/write/mod.rs` == 11 (Update + Delete predicates). `crud_update_soft_deleted_not_found`, `crud_delete_sets_deleted_at`, `crud_deleted_row_hidden_from_list` tests prove all three behaviours. No `DELETE FROM` in the kernel (`grep -c "DELETE FROM" framework/src/write/mod.rs` == 0). |
| 3 | `with_override("create_order", …)` fires the hook on the CRUD path with no new mechanism; generic plan is the default when no override is registered | VERIFIED | Override hook at `framework/src/write/mod.rs:735` (`dispatcher.overrides.get(&action.name)`) runs on both CRUD and transition paths — it is architecturally after the `if let Some(plan) = crud_plan` executor branch. `crud_override_replaces_generic` test proves: (a) hook fires with the created record, (b) row was inserted (generic path ran), (c) executor function is bypassed (panics if called). No new mechanism — same `WriteDispatcher::with_override()` method and `HashMap` lookup. |
| 4 | Exactly ONE `dispatch_write` definition; no second CRUD dispatcher; no transition `match` re-encoded on the CRUD path; same derived plan drives MCP and visual surfaces | VERIFIED | `grep -rn "pub async fn dispatch_write" framework/src/ ferro-mcp-server/src/` returns exactly 1 line (`framework/src/write/mod.rs:596`). `grep -c "not_yet_implemented" ferro-mcp-server/src/write_dispatch.rs` == 0 (NTI stub replaced). `grep -c "DELETE FROM" framework/src/write/mod.rs` == 0. CRUD path adds one `Option<&CrudPlan>` parameter and one `if let Some(plan)` branch at step 4 — no transition match re-encoded. Both MCP (`ferro-mcp-server`) and visual (`app/src/controllers/visual_action.rs`) call sites patch to `, None` on the transition path or pass `Some(&plan)` on the CRUD path — single kernel, channel as the only divergence. |

**Score:** 4/4 truths verified

### Deferred Items

Items not yet met but explicitly addressed in later milestone phases.

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | `tenant_column: Option<TenantColumn>` slot is `None` on all CrudPlan variants (D-09 extension point — intentional) | Phase 242 | Phase 242 SC#2: "tenant_id injected from context on create and predicated (AND tenant_id = ctx) on update/delete; tenant column absent from every write input schema" |

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-projections/src/executor.rs` | `CrudVerb`, `TenantColumn`, `CrudPlan` enums + `derive_crud_plan` + 6 unit tests | VERIFIED | All four types confirmed present (`grep -c "pub enum CrudPlan"` == 1, `pub enum CrudVerb` == 1, `pub struct TenantColumn` == 1, `pub fn derive_crud_plan` == 1). 6 test functions at lines 461, 514, 534, 586, 610, 639. `grep -c "sea_orm" executor.rs` == 0 (schema-only boundary preserved). |
| `ferro-projections/src/error.rs` | `VerbNotEnabled` error variant | VERIFIED | `grep -c "VerbNotEnabled" ferro-projections/src/error.rs` == 1. |
| `ferro-projections/src/lib.rs` | re-exports: `derive_crud_plan`, `CrudPlan`, `CrudVerb`, `TenantColumn` | VERIFIED | `pub use executor::{derive_crud_plan, derive_transition_plan, CrudPlan, CrudVerb, TenantColumn, TransitionPlan};` confirmed. |
| `framework/src/write/mod.rs` | `execute_crud_plan` + `dispatch_write crud_plan` param + 8 sqlite-in-memory tests + orders table fixture | VERIFIED | `async fn execute_crud_plan` at line 272. `crud_plan: Option<&CrudPlan>` at line 605. Orders table fixture at line 796. All 8 test functions at lines 1326, 1364, 1417, 1468, 1546, 1601, 1642, 1714. |
| `ferro-mcp-server/src/write_dispatch.rs` | NTI block replaced; CRUD dispatch path; 4 framing tests | VERIFIED | `grep -c "not_yet_implemented" write_dispatch.rs` == 0. `derive_crud_plan` called 3 times. `CallToolResult::structured` used 7 times. 4 framing tests at lines 1549, 1587, 1626, 1699. |
| `ferro-mcp-server/src/renderer.rs` | `request_confirm_delete_<svc>` / `confirm_delete_<svc>` synthesis for `.deletable` services | VERIFIED | `grep -c "request_confirm_delete_\|confirm_delete_" renderer.rs` == 4 (function references + synthesis). `grep -c "build_delete_input_schema" renderer.rs` == 4. |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ferro-projections/src/executor.rs::derive_crud_plan` | `service.rs` resolver accessors | `is_write_excluded_field`, `resolved_soft_delete_column`, `resolved_table` | VERIFIED | `grep -c "is_write_excluded_field\|resolved_soft_delete_column" executor.rs` >= 3 (Create/Update/Delete paths). |
| `ferro-projections/src/lib.rs` | executor module | `pub use executor::{derive_crud_plan, CrudPlan, CrudVerb, TenantColumn, …}` | VERIFIED | Re-export confirmed present, all 4 new symbols listed. |
| `framework/src/write/mod.rs::dispatch_write` step 4 | `execute_crud_plan` | `if let Some(plan) = crud_plan { execute_crud_plan(plan, tenant_id, db).await? }` | VERIFIED | Line 681-682 confirmed. |
| `dispatch_write` step 3 (confirmation seam) | `CrudPlan::Delete` gate | `matches!(crud_plan, Some(CrudPlan::Delete { .. }))` | VERIFIED | `grep -c "matches!(crud_plan, Some(CrudPlan::Delete"` == 1. |
| `ferro-mcp-server/src/write_dispatch.rs` CRUD branch | `derive_crud_plan` + `dispatch_write` | `derive_crud_plan(svc, verb, &args)` then `dispatch_write(…, Some(&plan))` | VERIFIED | Lines 209-219 confirmed (CRUD dispatch call). `derive_crud_plan` used 3 times in file. |
| `ferro-mcp-server/src/write_dispatch.rs` confirm handlers | `ServiceDef` by prefix strip | `strip_prefix("delete_")` in both `handle_request_confirm` and `handle_confirm` | VERIFIED | `grep -E 'strip_prefix\("delete_"\)'` returns 2 matches. Confirmed-delete dispatch at line 645+654 passes `Some(&crud_plan)`. |
| `ferro-mcp-server/src/renderer.rs` | delete confirm tools | synthesis loop over `.deletable` services | VERIFIED | 4 occurrences of `request_confirm_delete_` / `confirm_delete_` patterns. `build_delete_input_schema` used 4 times. |

---

## Data-Flow Trace (Level 4)

Not applicable — this phase delivers a kernel and schema layer (no frontend rendering). All data flows are exercised through sqlite-in-memory dispatch tests rather than rendering pipelines.

---

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Exactly one `dispatch_write` definition workspace-wide (SC#4) | `grep -rn "pub async fn dispatch_write" framework/src/ ferro-mcp-server/src/` | 1 match: `framework/src/write/mod.rs:596` | PASS |
| NTI stub gone from MCP write dispatch | `grep -c "not_yet_implemented" ferro-mcp-server/src/write_dispatch.rs` | 0 | PASS |
| No physical DELETE in write kernel | `grep -c "DELETE FROM" framework/src/write/mod.rs` | 0 | PASS |
| Soft-delete predicate present | `grep -c "IS NULL" framework/src/write/mod.rs` | 11 | PASS |
| schema-only boundary: no sea-orm in ferro-projections executor | `grep -c "sea_orm" ferro-projections/src/executor.rs` | 0 | PASS |
| CRUD audit prefix distinct from transition prefix | `grep -E 'format!.*channel.*crud' framework/src/write/mod.rs` | matched `"{channel}.crud.{}"` | PASS |
| tenant_column slot on all 3 CrudPlan variants | `grep -c "tenant_column: Option<TenantColumn>" ferro-projections/src/executor.rs` | 4 (3 variant definitions + function body) | PASS |

---

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| CRUD-03 | 241-02-PLAN.md, 241-03-PLAN.md | `delete_<svc>` soft-deletes (`deleted_at`), is confirmation-gated, filtered from `list_<svc>` | SATISFIED | `crud_delete_sets_deleted_at` (deleted_at set, row physically present), `crud_deleted_row_hidden_from_list` (hidden by IS NULL), `crud_delete_requires_confirmation` (ConfirmationRequired on bare delete), `delete_two_step_flow` (confirmed delete soft-deletes), `delete_bare_call_returns_confirmation_required` (framing layer returns confirmation_required). |
| CRUD-06 | 241-01-PLAN.md, 241-02-PLAN.md, 241-03-PLAN.md | CRUD verbs dispatch via `derive_crud_plan` through `framework::write` kernel, reusing override-hook, idempotency, channel-parameterized audit, confirmation; single-source across MCP and visual surfaces; does NOT rebuild the dispatcher | SATISFIED | One `dispatch_write` definition (framework/src/write/mod.rs:596). `derive_crud_plan` in ferro-projections mirrors `derive_transition_plan`. All 7 kernel steps (guards, idempotency, confirmation, execute, idempotency-store, audit, override hook) fire on the CRUD path unchanged. `crud_create_idempotent` proves idempotency reuse. `crud_override_replaces_generic` proves override-hook reuse. Audit prefix `{channel}.crud.{name}` distinct but same infrastructure. MCP and `app/` visual call sites all use the same `dispatch_write` signature. |

---

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `framework/src/write/mod.rs` | ~406-420 | Post-update SELECT lacks `AND {soft_delete_col} IS NULL` | Warning (WR-01 from REVIEW.md) | Minor robustness: a concurrent soft-delete between UPDATE and SELECT could return a row with `deleted_at` non-null. Not a correctness defect in normal usage. |
| `ferro-projections/src/executor.rs` | ~254-255 | `unwrap_or(serde_json::Value::Null)` for missing `id` in Update/Delete | Warning (WR-02 from REVIEW.md) | NULL id deferred to SQL semantics rather than explicit validation error. Returns `RecordNotFound` in practice but the guard is implicit. |
| `ferro-projections/src/executor.rs` | ~224-229 | No string length cap on Create column values | Warning (WR-03 from REVIEW.md) | DoS surface (unbounded string storage). Low urgency; fix belongs at FieldDef level. |
| `ferro-mcp-server/src/write_dispatch.rs` | ~411-449 | CRUD delete `handle_request_confirm` branch skips guard re-evaluation | Warning (WR-04 from REVIEW.md) | Asymmetry with transition path; harmless now (no guards on CRUD delete in Phase 241), but is a phase-242 extension-point correctness concern. |

All four are ADVISORY (from 241-REVIEW.md: 0 critical, 4 warnings). None block the phase goal. The REVIEW.md documents these explicitly; they are tracked for Phase 242.

---

## Human Verification Required

None. All Phase 241 behaviors have automated verification (VALIDATION.md confirms: "All Phase 241 behaviors have automated verification; e2e over `:8090/mcp` is Phase 243").

---

## Gaps Summary

No gaps. All four roadmap success criteria are verified by code evidence and passing tests. The four review warnings are advisory and do not block goal achievement. The `tenant_column: None` slot is correctly deferred to Phase 242 (D-09 extension point by design, documented in the scope note and Phase 242 roadmap).

The full workspace gate (`cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`) passed at phase close per 241-03-SUMMARY.md (per-wave gate: exit 0, 0 failed across all crates).

---

_Verified: 2026-06-24T08:00:00Z_
_Verifier: Claude (gsd-verifier)_
