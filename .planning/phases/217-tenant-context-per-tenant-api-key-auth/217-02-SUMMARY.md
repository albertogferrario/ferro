---
phase: 217-tenant-context-per-tenant-api-key-auth
plan: "02"
subsystem: ferro-mcp-server
tags: [auth, mcp, api-key, tenant-context, tdd-green, scope-gate, cross-tenant-isolation]
dependency_graph:
  requires:
    - ferro-mcp-server::McpContext (extended — from 217-00)
    - ferro-mcp-server::handle_tools_list ctx param (from 217-00)
    - ferro-mcp-server::scope gate in handle_tools_call (from 217-00)
    - ferro-mcp-oauth::validate_api_key (real — from 217-01)
    - ferro-mcp-oauth::generate_mcp_api_key (real — from 217-01)
  provides:
    - ferro-mcp-server::mcp_tenant_isolation tests GREEN (SC#3 + SC#5)
    - strict SC#5 cross-tenant assertion (both == 1 and != 2 per-row)
  affects:
    - ferro-mcp-server/tests/mcp_tenant_isolation.rs (strengthened assertion)
tech_stack:
  added: []
  patterns:
    - assert_eq + assert_ne dual assertion for strict cross-tenant isolation proof (SC#5)
    - scope gate (is_write_tool && key_scope == "read") wired before service lookup in handle_tools_call
    - McpContext.tenant_id as single source of truth for dispatch — never from call_params
key_files:
  created: []
  modified:
    - ferro-mcp-server/tests/mcp_tenant_isolation.rs
decisions:
  - Added explicit assert_ne!(row_tid, Some(2)) alongside assert_eq!(row_tid, Some(1)) in api_key_cross_tenant_isolation to satisfy the dual-assertion acceptance criterion; the equality check alone is logically sufficient but the explicit != 2 form makes the security property visible in the test
  - Task 1 (McpContext threading) was fully complete from Plans 00/01 — jsonrpc.rs had ctx param in both handle_tools_list and handle_tools_call with scope gate wired; verified by greps and build
  - dispatch.rs left unmodified — fail-closed tenant predicate injection already in place
metrics:
  duration_minutes: ~5
  completed_date: "2026-06-13"
  tasks_completed: 2
  tasks_total: 2
  files_created: 0
  files_modified: 1
---

# Phase 217 Plan 02: McpContext Threading + GREEN Integration Tests Summary

SC#3 (scope gate) and SC#5 (cross-tenant isolation) integration tests GREEN against real `validate_api_key`; strict per-row dual assertion (== 1 and != 2) added to `api_key_cross_tenant_isolation`.

## What Was Built

**Task 1 — McpContext threading through tools/list and tools/call (verification):**

The threading was already complete from Plans 00 and 01. Verified:

- `handle_tools_list(services, ctx, _config)` passes `ctx` into `render_exposed_tools(services, ctx)` — grep confirmed, line 39 of `jsonrpc.rs`
- `handle_tools_call` receives `ctx: &McpContext` as 5th parameter; scope gate reads `ctx.scope.as_deref().unwrap_or("read_write")` — never touches `call_params` for tenant or scope
- `tenant_id` passed to `dispatch` is the standalone `tenant_id: Option<i64>` parameter (caller-resolved auth principal), not sourced from `call_params` — grep for `tenant` in `call_params` context returned no match
- `dispatch.rs` unmodified — `git diff --stat ferro-mcp-server/src/dispatch.rs` returned empty
- Scope gate fires BEFORE service lookup: write tools rejected for read-scoped keys even when the tool name is not in the service list (synthetic `create_order` test)
- Comment in `handle_tools_list` body marks the scope-filtering-on-list as no-op-now / active-218 boundary (all tools are `list_*` read tools in Phase 217)

**Task 2 — SC#3 and SC#5 tests GREEN + strict assertion:**

All 4 `mcp_tenant_isolation.rs` tests were already GREEN with Plan 01's real `validate_api_key`. The only code change was strengthening `api_key_cross_tenant_isolation`:

Before:
```rust
for row in rows {
    assert_eq!(row["tenant_id"].as_i64(), Some(1), "...");
}
```

After:
```rust
for row in rows {
    let row_tid = row["tenant_id"].as_i64();
    assert_eq!(row_tid, Some(1), "all rows must belong to tenant 1, got: {row}");
    assert_ne!(row_tid, Some(2), "tenant 2 rows must never surface under tenant 1 key, got: {row}");
}
```

**Test results (final run):**

```
running 4 tests (mcp_tenant_isolation)
test read_scope_key_rejected_on_write_tool_name ... ok   (SC#3: write-tool rejection)
test read_scope_key_allowed_on_read_tool ... ok          (SC#3: read-tool pass-through)
test api_key_cross_tenant_isolation ... ok               (SC#5: strict per-row isolation)
test api_key_and_jwt_produce_same_tenant_id ... ok       (SC#2: auth parity)
test result: ok. 4 passed; 0 failed

running 5 tests (jsonrpc_integration)
test initialize_returns_correct_protocol_version ... ok
test tools_list_returns_only_exposed ... ok
test tools_call_unknown_tool_is_method_not_found ... ok
test tools_call_unknown_filter_is_invalid_params ... ok
test tools_call_returns_rows ... ok
test result: ok. 5 passed; 0 failed
```

Total: 9/9 ferro-mcp-server tests pass.

## Security Properties Proven

| Threat | Test | Assertion |
|--------|------|-----------|
| T-217-01 (cross-tenant) | `api_key_cross_tenant_isolation` | rows.len()==2 AND per-row tenant_id==Some(1) AND tenant_id!=Some(2) |
| T-217-04 (scope creep) | `read_scope_key_rejected_on_write_tool_name` | error.code==-32603, message contains "scope insufficient" |
| T-217-03 (tenant from payload) | grep acceptance criterion | `call_params` has no `tenant` references |

## Deviations from Plan

None — plan executed exactly as written. Task 1 was already complete from prior plans; Task 2 required only the strictness enhancement to the cross-tenant assertion. The "note" in Task 2's action about "fix TEST WIRING only if still-RED" was not needed — all tests were GREEN from Plan 01's real `validate_api_key`.

## Known Stubs

None — all stubs from Plans 00 and 01 are replaced with real implementations.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes. All changes are in `ferro-mcp-server/tests/` (test-only). No new threat surface beyond the plan's threat register.

## Self-Check

- `ferro-mcp-server/tests/mcp_tenant_isolation.rs` — contains `api_key_cross_tenant_isolation`, `Some(1)` assertion, `Some(2)` assertion (assert_ne): FOUND
- `ferro-mcp-server/src/jsonrpc.rs` — contains `scope insufficient`, `ctx: &McpContext`, `render_exposed_tools(services, ctx)`: FOUND
- `cargo test -p ferro-mcp-server --test mcp_tenant_isolation` — 4/4 ok: VERIFIED
- `cargo test -p ferro-mcp-server` — 9/9 ok (incl. jsonrpc_integration): VERIFIED
- No tenant extracted from call_params: VERIFIED (grep returned no match)
- `dispatch.rs` unmodified: VERIFIED (git diff empty)
- Commit `c7a8a349` exists: FOUND

## Self-Check: PASSED
