---
phase: 242-write-authorization-tenant-injection-non-disclosure
verified: 2026-06-24T00:00:00Z
status: passed
score: 4/4 must-haves verified
overrides_applied: 0
re_verification: null
gaps: []
deferred: []
human_verification: []
---

# Phase 242: Write Authorization, Tenant Injection & Non-Disclosure — Verification Report

**Phase Goal:** Make every CRUD write require `read_write` key scope and pass the `.mcp_write_ability` policy Gate, inject `tenant_id` from context (never an agent input), and ensure cross-tenant or soft-deleted targets are indistinguishable from "not found". Verify the shipped CRUD-07 validate() write-ability fail-fast rule at this boundary.
**Verified:** 2026-06-24T00:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A `read`-scope key calling any `create_`/`update_`/`delete_` tool is rejected (scope-denied) before dispatch; a `read_write` key that fails the `.mcp_write_ability` Gate is denied | VERIFIED | Scope gate at `ferro-mcp-server/src/jsonrpc.rs:77` fires before dispatch. `write_authorized: Option<bool>` field in `McpContext` (renderer.rs:32); `is_crud_write_tool && ctx.write_authorized != Some(true)` gate at write_dispatch.rs:157. Tests `write_authorized_none_denies` and `write_authorized_false_denies` pass (3 of 3 framing tests green). |
| 2 | `tenant_id` is injected from context on create and predicated (`AND tenant_id = ctx`) on update/delete; the tenant column is absent from every write input schema | VERIFIED | `execute_crud_plan` Create arm appends tenant column to col_names + ph_parts with `placeholder(backend, columns.len() + 1)` and binds `sea_orm::Value::BigInt(Some(tenant_id))` (framework/src/write/mod.rs:301-323). Update/Delete arms append `AND {tc_col} = {tenant_ph}` to WHERE clause (mod.rs:424-443, 503-520). `is_server_injected_field` in service.rs:236-244 marks tenant column as server-injected; `is_write_excluded_field` gates it out of write schemas. |
| 3 | An update/delete targeting another tenant's row, or a soft-deleted row, returns the same non-disclosing "not found / denied" envelope — no row/column/filter leakage | VERIFIED | Tenant predicate makes cross-tenant row produce 0 affected rows, falling through to existing `WriteError::RecordNotFound` path (write/mod.rs:62). No new error variant. Tests `crud_cross_tenant_update_not_found` and `crud_cross_tenant_delete_not_found` assert `Err(RecordNotFound)` AND assert the seeded row is left physically unchanged (status/deleted_at unmodified) — confirmed at mod.rs:2068-2133. |
| 4 | A boot-time test confirms `ServiceDef::validate()` rejects a projection that enables any CRUD verb without `mcp_write_ability` | VERIFIED | Rule at service.rs:504-510 (shipped in `5cb17d60`, D-10 = test-only change). Test `validate_rejects_crud_verb_without_write_ability` at service.rs:2314 covers create/update/delete all three verbs + the passing case with `mcp_write_ability`. Test passes green (1/1). |

**Score:** 4/4 truths verified

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-projections/src/executor.rs` | `derive_crud_plan` fills `tenant_column` from `svc.tenant_column` at all three CRUD variant construction sites | VERIFIED | `grep -c "svc.tenant_column.as_ref().map(|col| TenantColumn"` = 3. Lines 247-249, 274-276, 293-295 all carry identical expression. No `tenant_column: None` remains in the derive_crud_plan body. |
| `framework/src/write/mod.rs` | `execute_crud_plan` binds runtime `tenant_id` when `tenant_column` is `Some` for Create/Update/Delete + post-update SELECT | VERIFIED | `grep -c "sea_orm::Value::BigInt(Some(tenant_id))"` = 10 (well above the required >=4). `grep -c "_tenant_id"` = 0. `grep -c "tenant_column: _"` = 0. Placeholder index = `columns.len() + 1` for Create (not +2, confirmed at mod.rs:313). |
| `ferro-mcp-server/src/renderer.rs` | `McpContext` carries dedicated `write_authorized: Option<bool>` field, NOT in `evaluated_guards` | VERIFIED | `pub write_authorized: Option<bool>` at renderer.rs:32. Doc comment explicitly states separation from `evaluated_guards`. `grep "evaluated_guards" write_dispatch.rs` returns no authz usage — only a comment reference at line 122. |
| `ferro-mcp-server/src/write_dispatch.rs` | Fail-closed write-ability gate applies ONLY to CRUD verb tools (CR-01 fix); transition-action tools pass through | VERIFIED | `is_crud_write_tool` detector at write_dispatch.rs:131-150 strips optional confirm prefixes, then checks exactly one of `create_`/`update_`/`delete_` prefixes against `s.creatable`/`s.updatable`/`s.deletable` flags. Gate fires only when `is_crud_write_tool && ctx.write_authorized != Some(true)` (line 157). `transition_action_not_denied_by_write_ability_gate` test passes. |
| `app/src/controllers/mcp.rs` | Host computes `write_authorized` via `Gate::authorize_for` for CRUD verb tools; `None` for read/transition-action tools | VERIFIED | `find_map` over `["create_", "update_", "delete_"]` with single `strip_prefix` per verb at mcp.rs:338-351. Returns `Some(Gate::authorize_for(...).is_ok())` for CRUD tools, `None` for read/transition-action tools (line 374). `McpContext { write_authorized, .. }` at line 382-387. |
| `ferro-projections/src/service.rs` | `validate_rejects_crud_verb_without_write_ability` test covers create/update/delete + passing case | VERIFIED | Test at service.rs:2314-2345 asserts all four cases. Three older per-verb tests (lines 2021-2049) also gain message-content assertions (WR-02 fix, commit `4a57149a`). |
| Post-INSERT SELECT tenant predicate (CR-02 fix) | Post-INSERT SELECT in SQLite arm of `execute_crud_plan` carries `AND {tc_col} = ?` when `tenant_column` is `Some` | VERIFIED | `(select_sql, select_values)` conditional at write/mod.rs:370-387. When `tenant_column` is `Some`: `SELECT * FROM {table} WHERE id = ? AND {tc_col} = {t_ph}` with `placeholder(backend, 2)` and two bound values. Matches post-UPDATE SELECT pattern exactly. |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| `execute_crud_plan` Create arm | INSERT column list + bound values | tenant column appended after `created_at`; placeholder index = `columns.len() + 1` | WIRED | Confirmed at mod.rs:301-323. `created_at` is pushed as SQL literal (`now_expr.to_string()`) and does NOT consume a placeholder slot — verified by `placeholder(backend, columns.len() + 1)` not +2. |
| `execute_crud_plan` Update/Delete arms | WHERE predicate | `AND {tc_col} = {tenant_ph}` bound to `tenant_id`; 0 rows → `RecordNotFound` | WIRED | Update: `AND {soft_delete_column} IS NULL AND {tc_col} = {tenant_ph}` at mod.rs:424-430. Delete: same pattern at mod.rs:503-510. Existing `rows_affected() == 0 → Err(WriteError::RecordNotFound)` paths at mod.rs reused unchanged. |
| `handle_write_call` | CRUD authorization | `is_crud_write_tool && ctx.write_authorized != Some(true)` before any service lookup | WIRED | Gate at write_dispatch.rs:157 is line 157; CRUD prefix loop starts at line ~201. Ordering confirmed: Gate precedes every service lookup and CRUD dispatch. |
| `app/src/controllers/mcp.rs` | `McpContext.write_authorized` | `Gate::authorize_for(&user, ability, None)` via `find_map` over CRUD verb prefixes | WIRED | Two `Gate::authorize_for` call sites: one for read tools (`mcp_ability`), one for write tools (`mcp_write_ability`). `write_authorized` field populated before `McpContext` construction at mcp.rs:385. |
| `derive_crud_plan` | `svc.tenant_column` | `as_ref().map(|col| TenantColumn { column: col.clone() })` | WIRED | Three sites in executor.rs, verified by grep count = 3. `CrudPlan::Create/Update/Delete.tenant_column` field carries the derived value. |

---

## Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|-------------------|--------|
| `execute_crud_plan` Create arm | `tenant_id: i64` parameter | Auth context (`ferro::current_tenant().map(|t| t.id)` in mcp.rs:381) | Yes — bound from JWT sub claim, not agent input | FLOWING |
| `execute_crud_plan` Update/Delete arms | `tenant_column: Option<TenantColumn>` | Derived from `svc.tenant_column` in `derive_crud_plan` (static projection declaration) | Yes — comes from service schema, never agent payload | FLOWING |
| `McpContext.write_authorized` | `write_authorized: Option<bool>` | `Gate::authorize_for` live call against the concrete `User` (DB-loaded in mcp.rs:358-366) | Yes — real Gate evaluation against RBAC policy | FLOWING |

---

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `write_authorized=None` denies CRUD write tool | `cargo test -p ferro-mcp-server --lib --features confirmation write_authorized_none_denies` | ok | PASS |
| `write_authorized=Some(false)` denies CRUD write tool | `cargo test -p ferro-mcp-server --lib --features confirmation write_authorized_false_denies` | ok | PASS |
| `write_authorized=Some(true)` allows dispatch | `cargo test -p ferro-mcp-server --lib --features confirmation write_authorized_true_proceeds` | ok | PASS |
| Transition-action tool with `write_authorized=None` is NOT denied by write-ability gate | `cargo test -p ferro-mcp-server --lib --features confirmation transition_action_not_denied_by_write_ability_gate` | ok | PASS |
| CRUD-07 boot-time validate() rejects CRUD verb without `mcp_write_ability` | `cargo test -p ferro-projections --lib validate_rejects_crud_verb_without_write_ability` | ok (1 passed) | PASS |

---

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| CRUD-05 | Plans 01, 02, 03 | `create`/`update`/`delete` require `read_write` scope + `.mcp_write_ability` Gate; `tenant_id` server-injected and excluded from write schema; cross-tenant/soft-deleted non-disclosing | SATISFIED | Scope gate (jsonrpc.rs:77, shipped). Write-ability gate (write_dispatch.rs:131-164, new). Tenant injection (write/mod.rs, verified). Tenant column excluded via `is_server_injected_field` + `is_write_excluded_field` (service.rs:236-265). RecordNotFound non-disclosure (no new variant). All 5 dispatch tests pass. |
| CRUD-07 | Plan 04 | `ServiceDef::validate()` fails fast at registration when any CRUD verb is enabled without `mcp_write_ability` | SATISFIED | Rule at service.rs:504-510 (shipped in `5cb17d60`, confirmed unchanged). Test `validate_rejects_crud_verb_without_write_ability` passes (1/1). Three per-verb sibling tests also pass with message-content assertions (WR-02 fix). |

---

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | — | — | — | No TODO/FIXME, no placeholder returns, no hardcoded empty data, no stub implementations found in any of the 6 phase files. |

---

## CR-01 / CR-02 Fix Verification (Additional Focus)

Both critical fixes from the code review are confirmed present in the actual code:

**CR-01 (transition-action gate scope):** `write_dispatch.rs:131-150` contains the `is_crud_write_tool` detector — it strips `request_confirm_`/`confirm_` prefixes, then checks for exactly one CRUD verb prefix (`create_`/`update_`/`delete_`) combined with the matching service CRUD flag (`s.creatable`/`s.updatable`/`s.deletable`). The gate at line 157 is `if is_crud_write_tool && ctx.write_authorized != Some(true)`. Transition-action tools (no CRUD prefix) have `is_crud_write_tool = false` and bypass the gate. `app/src/controllers/mcp.rs:338-374` uses `find_map` over CRUD verb prefixes with single `strip_prefix` per verb (no `trim_start_matches` chain), returning `None` for transition-action tools. The `transition_action_not_denied_by_write_ability_gate` regression test passes.

**CR-02 (post-INSERT SELECT tenant predicate):** `framework/src/write/mod.rs:370-387` shows the `(select_sql, select_values)` conditional — when `tenant_column` is `Some`, the SELECT is `SELECT * FROM {table} WHERE id = ? AND {tc_col} = {t_ph}` with `placeholder(backend, 2)` and two bound values (`inserted_id`, `tenant_id`). This mirrors the post-UPDATE SELECT pattern at mod.rs:462-471.

**`write_authorized` is a DEDICATED `McpContext` field:** `renderer.rs:32` — `pub write_authorized: Option<bool>`. The doc comment at line 21 and the comment in `write_dispatch.rs:122` both explicitly state it is NOT stored in `evaluated_guards`. `grep "evaluated_guards" write_dispatch.rs` returns only one line (the clarifying comment), not a live read.

**Non-disclosure uses existing `WriteError::RecordNotFound`:** `enum WriteError` at `framework/src/write/mod.rs:31-63` has no new cross-tenant variant. The `RecordNotFound` variant at line 59-62 covers all non-disclosure cases (cross-tenant, soft-deleted, genuinely missing).

**Create tenant placeholder index is `columns.len() + 1`:** `write/mod.rs:313` confirms `placeholder(backend, columns.len() + 1)`. Not +2. The comment at lines 309-311 explicitly documents why: `created_at` is a SQL literal (`now_expr`) that does not consume a placeholder slot.

**CRUD-07 `validate()` function body unchanged:** `service.rs:499` — `pub fn validate(&self)` — the rule at lines 504-510 is identical to what shipped in `5cb17d60`. Plan 04 added only a test (`ed12c64a`), no modification to `validate()` itself.

---

## Human Verification Required

None. All success criteria are fully verifiable via static analysis and the targeted test runs performed above.

---

## Gaps Summary

No gaps. All four roadmap success criteria are verified against the actual codebase:

1. **SC#1 (scope + write-ability gate):** Scope gate at jsonrpc.rs:77 (shipped, tested by existing scope-deny test). Write-ability gate at write_dispatch.rs:157 scoped to CRUD verb tools via `is_crud_write_tool` (CR-01 fix confirmed). Three framing tests pass.

2. **SC#2 (tenant injection + schema exclusion):** Tenant column derived from `svc.tenant_column` at all 3 CrudPlan sites. Runtime `tenant_id` bound at 10 sites in `execute_crud_plan`. Tenant column excluded from write schemas via `is_server_injected_field`. All 5 dispatch tests pass.

3. **SC#3 (non-disclosure):** Cross-tenant rows produce 0 affected rows, falling through to existing `WriteError::RecordNotFound`. No new error variant. Post-call row-integrity assertions in `crud_cross_tenant_update_not_found` and `crud_cross_tenant_delete_not_found` verify no partial write. CR-02 fix adds tenant predicate to post-INSERT SELECT (race window closed).

4. **SC#4 (CRUD-07 boot-time test):** `validate_rejects_crud_verb_without_write_ability` test covers all three verbs and the passing case. `validate()` function body unchanged.

---

_Verified: 2026-06-24T00:00:00Z_
_Verifier: Claude (gsd-verifier)_
