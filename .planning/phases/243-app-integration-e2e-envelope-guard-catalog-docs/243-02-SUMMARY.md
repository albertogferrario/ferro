---
phase: 243-app-integration-e2e-envelope-guard-catalog-docs
plan: "02"
subsystem: app-tests
tags: [crud, e2e, mcp, write-dispatch, envelope-guard, confirmation, visual-parity, tenant-isolation]
dependency_graph:
  requires: [Phase 239 deleted_at migration, Phase 241 derive_crud_plan, Phase 242 write-authz gate, Plan 243-01 order CRUD flip]
  provides: [CRUD-01..07 e2e coverage, D-07 per-verb envelope regression guard, MCP↔visual parity proof, SC#3 confirmation flow]
  affects: [app/src/tests/crud_e2e.rs, app/src/tests/mod.rs, app/src/projections/order.rs, app/Cargo.toml]
tech_stack:
  added: [ferro-projections as dev-dep (derive_crud_plan + CrudVerb for visual parity)]
  patterns: [in-process MCP e2e harness, Phase 205 structured-envelope regression guard, dispatch_write(.."web") parity, InMemoryConfirmationStore two-step flow]
key_files:
  created:
    - app/src/tests/crud_e2e.rs
  modified:
    - app/src/tests/mod.rs
    - app/src/projections/order.rs
    - app/Cargo.toml
decisions:
  - "Reused make_test_write_dispatcher (executor closure bypassed for CRUD when crud_plan=Some); no per-name CRUD SQL dispatcher written"
  - "Rule 2 fix: added .soft_delete_column(deleted_at) to order projection — required for dispatch to filter soft-deleted rows from list_order (CRUD-04)"
  - "assert_list_envelope gated #[cfg(not(feature = confirmation))] to suppress dead_code warning when cycle test is excluded"
  - "Audit target for CRUD creates uses record_id='' (inputs.get(id) is None at create time — sourced from write/mod.rs line 795)"
  - "ferro-projections added as dev-dep for derive_crud_plan + CrudVerb; not re-exported by ferro (framework)"
metrics:
  duration_seconds: 383
  completed_date: "2026-06-24"
  tasks_completed: 2
  files_modified: 4
---

# Phase 243 Plan 02: CRUD E2E Harness + Envelope Guard + Parity + Confirmation Summary

**One-liner:** In-process MCP e2e drives create→list→update→delete through the shipped `execute_crud_plan` kernel with per-verb Phase 205 envelope guards, MCP↔visual parity proof, write-authorization gate, cross-tenant non-disclosure, and a feature-gated delete confirmation flow.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | CRUD e2e harness + create→list→update→delete cycle + per-verb envelope guard + auth gate | a18266f5 | app/src/tests/crud_e2e.rs, app/src/tests/mod.rs, app/src/projections/order.rs, app/Cargo.toml |
| 2 | MCP↔visual single-source parity + delete confirmation flow (feature-gated) | a18266f5 | app/src/tests/crud_e2e.rs |

Both tasks delivered in a single commit since they share the same file.

## What Was Built

### Test module: `app/src/tests/crud_e2e.rs`

Five test functions covering all Plan 02 acceptance criteria:

**`crud_cycle_create_list_update_delete`** (`#[cfg(not(feature = "confirmation"))]`)
- Creates an order via `handle_tools_call("create_order", ...)` — status is set to `"draft"` server-side; tenant_id is injected from `McpContext`, not from agent input.
- Asserts Phase 205 envelope (D-07) for create: `content[0].type==text`, `structuredContent.status==ok`, `action=="create_order"`, `result` is object.
- Lists via `list_order` — confirms the new record appears.
- Updates via `update_order` (customer_name only; no status — CRUD-02).
- Deletes via `delete_order` (direct soft-delete; feature off).
- Confirms list_order NOW excludes the soft-deleted record (CRUD-03/04).

**`crud_write_requires_write_authorization`** (both feature states)
- Calls `create_order` with `write_authorized: None`.
- Asserts response has `error.code == -32603` and message contains `"write ability denied"` (CRUD-05).

**`crud_cross_tenant_non_disclosure`** (both feature states)
- Tenant 1 calls `update_order` targeting order id=3 (owned by tenant 2).
- Asserts `isError==true` and `structuredContent.result` is not an object (non-disclosing).
- Asserts the foreign order's `customer_name` is unchanged (T-243-01).

**`crud_mcp_visual_single_source_parity`** (`#[cfg(not(feature = "confirmation"))]`)
- MCP path: `handle_tools_call("create_order", ...)` → row with `status="draft"`.
- Visual path: `drive_visual_crud(CrudVerb::Create, "create_order", ...)` → same `dispatch_write(.., "web", Some(&plan))` → row with `status="draft"`.
- Both rows have identical `status` and `tenant_id` — single-source proof (CRUD-06).
- Audit action prefix is the ONLY divergence: `mcp.crud.create_order` vs `web.crud.create_order` (the `.crud.` prefix, not `.action.`, confirmed from `write/mod.rs` line 798).

**`delete_order_confirmation_flow`** (`#[cfg(feature = "confirmation")]`)
- Creates a target row.
- Bare `delete_order` → `confirmation_required` with `request_tool=="request_confirm_delete_order"` and `isError==true`.
- `request_confirm_delete_order` → token with `cfm_` prefix (CSPRNG, 256-bit entropy).
- `confirm_delete_order` with token → ok envelope (`assert_write_envelope_ok(&confirm, "delete_order")`).
- SAME `InMemoryConfirmationStore` instance threaded through both request and confirm calls.
- After confirm: `list_order` excludes the soft-deleted row (SC#3/CRUD-03).

### Architecture

The key invariant is maintained throughout: `crud_plan=Some(...)` causes `dispatch_write` to call `execute_crud_plan` directly, bypassing the `dispatcher.executor` closure. The test reuses `make_test_write_dispatcher` unchanged — its executor handles transition actions; for CRUD verbs it is never invoked.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical Functionality] Added `.soft_delete_column("deleted_at")` to order projection**
- **Found during:** Task 1 — `crud_cycle_create_list_update_delete` failed with "soft-deleted record must be filtered out of list_order"
- **Issue:** `ferro-mcp-server/src/dispatch.rs` line 285 gates the `deleted_at IS NULL` filter on `service.soft_delete_column.is_some()`. The Plan 01 projection flip added `.deletable(true)` but omitted `.soft_delete_column("deleted_at")`, so `list_order` did not filter soft-deleted rows.
- **Fix:** Added `.soft_delete_column("deleted_at")` to `app/src/projections/order.rs` alongside the existing CRUD flags.
- **Files modified:** `app/src/projections/order.rs`
- **Commit:** a18266f5

**2. [Rule 3 - Blocking Issue] Added `ferro-projections` as dev-dependency**
- **Found during:** Task 2 — `ferro_projections::CrudVerb` and `ferro_projections::derive_crud_plan` are not re-exported by `ferro` (framework `lib.rs`). The visual parity driver requires them directly.
- **Fix:** Added `ferro-projections = { path = "../ferro-projections" }` to `[dev-dependencies]` in `app/Cargo.toml`.
- **Files modified:** `app/Cargo.toml`
- **Commit:** a18266f5

**3. [Rule 1 - Bug] Fixed assert logic for cross-tenant non-disclosure**
- **Found during:** Task 1 — `is_null()` on a `serde_json::Value` conflicted with SeaORM's `is_null()` method (type mismatch).
- **Fix:** Rewrote assertion as `sc["result"].as_object().is_none()`.
- **Files modified:** `app/src/tests/crud_e2e.rs`
- **Commit:** a18266f5

## Known Stubs

None. All test assertions exercise real shipped behavior through the full kernel path.

## Threat Flags

No new network endpoints, auth paths, or schema changes introduced. All test code is `#[cfg(test)]`-gated. The auth gate test (`write_authorized: None` → -32603) and cross-tenant test prove the production gates are real and not masked by test defaults.

## Self-Check: PASSED

- `app/src/tests/crud_e2e.rs` — FOUND (created, 869 lines)
- `app/src/tests/mod.rs` — FOUND (modified, `pub mod crud_e2e;` present)
- `app/src/projections/order.rs` — FOUND (modified, `.soft_delete_column("deleted_at")` present)
- `app/Cargo.toml` — FOUND (modified, `ferro-projections` dev-dep present)
- Commit `a18266f5` — FOUND (`feat(243-02): CRUD e2e harness...`)
- `cargo test -p app crud_e2e` — 4 passed (default features)
- `cargo test -p app --features confirmation crud_e2e` — 3 passed
- `grep -c 'match action_name' app/src/tests/crud_e2e.rs` — 0 (no per-name CRUD SQL dispatcher)
