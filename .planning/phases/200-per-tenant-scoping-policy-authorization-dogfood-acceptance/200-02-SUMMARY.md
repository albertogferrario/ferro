---
phase: 200-per-tenant-scoping-policy-authorization-dogfood-acceptance
plan: "02"
subsystem: ferro-mcp-server
tags: [tenant-scoping, security, dispatch, mcp, sql-injection-prevention]
dependency_graph:
  requires:
    - 200-01 (ServiceDef.tenant_column + mcp_ability fields in ferro-projections)
  provides:
    - dispatch tenant predicate injection (bound parameter, never from payload)
    - handle_tools_call tenant_id forwarding
    - fail-closed enforcement when tenant_column=Some + tenant_id=None
  affects:
    - app/src/controllers/mcp.rs (call site now broken — fixed in Plan 200-05)
tech_stack:
  added: []
  patterns:
    - bound-parameter tenant predicate injection via sea_orm::Value::BigInt
    - fail-closed: tenant_column=Some + tenant_id=None -> Err(InvalidFilter)
    - Option<i64> parameter threading from handle_tools_call -> dispatch
key_files:
  created: []
  modified:
    - ferro-mcp-server/src/dispatch.rs
    - ferro-mcp-server/src/jsonrpc.rs
    - ferro-mcp-server/tests/dispatch_integration.rs
    - ferro-mcp-server/tests/jsonrpc_integration.rs
decisions:
  - Tenant predicate injected after user-filter loop, before where_str/count/data SQL build — single injection site covers both COUNT and SELECT queries
  - sea_orm::Value::BigInt(Some(tid)) used for the bound value, matching the tenant_id column type
  - Fail-closed enforced via InvalidFilter variant (client parameter problem, maps to -32602)
  - Existing dispatch and jsonrpc integration tests updated to pass None (non-tenant scenarios remain valid)
metrics:
  duration: 268s
  completed: "2026-06-10"
  tasks: 2
  files_modified: 4
---

# Phase 200 Plan 02: Tenant Predicate Injection in dispatch Summary

Tenant predicate injection into `ferro-mcp-server` dispatch read path as a bound SQL parameter, with `tenant_id: Option<i64>` threaded from `handle_tools_call` through to `dispatch`. Fail-closed enforcement on tenant-scoped projections with missing tenant context (SC-1, T-200-FAILOPEN).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add tenant_id parameter and tenant predicate injection to dispatch | 7438d6bf | dispatch.rs, dispatch_integration.rs |
| 2 | Forward tenant_id through handle_tools_call | 16abbdfb | jsonrpc.rs, jsonrpc_integration.rs |

## Decisions Made

- **Single injection site for both queries:** The tenant predicate is pushed onto `where_clauses` after the user-filter loop and before `where_str` is built. Since `where_str` is then used identically by both the COUNT and SELECT statements, one injection covers both — no risk of predicate divergence between count and data.
- **`sea_orm::Value::BigInt(Some(tid))`:** Matches the `i64` type of `tenant_id` on the orders table. Never string-interpolated.
- **Fail-closed via `InvalidFilter`:** Reuses the existing error variant (maps to JSON-RPC -32602) so the caller can distinguish tenant-context failures from internal DB errors (-32603).
- **No `framework` dependency introduced:** `tenant_id` is passed as a plain function parameter. `ferro-mcp-server` remains framework-free (`cargo tree -p ferro-mcp-server | grep framework` returns nothing relevant).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated existing integration test call sites for new dispatch signature**
- **Found during:** Task 1 GREEN phase
- **Issue:** `tests/dispatch_integration.rs` and `tests/jsonrpc_integration.rs` had 5+3 existing call sites using the old 5-argument signature; they failed to compile after the signature change.
- **Fix:** Added `None` as the trailing argument at each call site (non-tenant scenarios remain correct with `None`).
- **Files modified:** `ferro-mcp-server/tests/dispatch_integration.rs`, `ferro-mcp-server/tests/jsonrpc_integration.rs`
- **Commit:** 7438d6bf (dispatch tests), 16abbdfb (jsonrpc tests)

**2. [Rule 3 - Blocking] Wrong FieldMeaning variants in test helpers**
- **Found during:** Task 1 RED phase compilation
- **Issue:** Test helper used `FieldMeaning::DisplayName`, `FieldMeaning::Amount`, `FieldMeaning::Timestamp` which do not exist in the enum. Actual variants are `EntityName`, `Money`, `CreatedAt`.
- **Fix:** Corrected all three variant names in the test helper.
- **Files modified:** `ferro-mcp-server/src/dispatch.rs`
- **Commit:** 7438d6bf

## Known Stubs

None — no stub data paths or placeholder values in the implementation.

## Threat Surface Scan

No new unplanned threat surface. The changes implement mitigations that are explicitly in the plan's threat model:

| Flag | File | Description |
|------|------|-------------|
| mitigated: T-200-01 | ferro-mcp-server/src/dispatch.rs | Tenant predicate is bound parameter; value comes from function parameter only; two-tenant isolation proven by tenant_scoping + tenant_isolation tests |
| mitigated: T-200-02 | ferro-mcp-server/src/dispatch.rs | Tenant scope appended outside filter loop from separate parameter; filter-key allowlist already excludes tenant_column injection via payload |
| mitigated: T-200-FAILOPEN | ferro-mcp-server/src/dispatch.rs | tenant_column=Some + tenant_id=None -> Err; covered by tenant_fail_closed test |

## Self-Check: PASSED
