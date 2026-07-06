---
phase: 219-write-dispatch
plan: "01"
subsystem: ferro-mcp-server, ferro-mcp-oauth
tags: [write-dispatch, mcp, guard-reevaluation, idempotency, audit, security, tdd-green]
dependency_graph:
  requires: [219-00]
  provides: [dispatch_write pipeline, handle_write_call router, write_tool_error_result helper, SC#1/SC#3/SC#5 GREEN]
  affects: [ferro-mcp-server, ferro-mcp-oauth]
tech_stack:
  added: []
  patterns: [live guard re-evaluation before executor, INSERT OR IGNORE idempotency, ferro-audit builder chain, write_tool_error_result sole error constructor]
key_files:
  created: []
  modified:
    - ferro-mcp-server/src/write_dispatch.rs
    - ferro-mcp-server/src/jsonrpc.rs
    - ferro-mcp-server/tests/jsonrpc_integration.rs
    - ferro-mcp-server/tests/mcp_tenant_isolation.rs
decisions:
  - dispatch_write reads live GuardEvaluatorFn callback for every precondition — ctx.evaluated_guards (218 visibility cache) never consulted at call time (D-02)
  - idempotency lookup scoped by (tenant_id, idempotency_key) composite — prevents cross-tenant replay (T-219-01)
  - write_tool_error_result is the sole error-result constructor — no bare content[] arrays elsewhere (D-06)
  - audit_log table created via raw SQL in test setup_db() alongside mcp_idempotency_keys — avoids async-trait/sea-orm-migration in ferro-mcp-server dev-deps (carried forward from Plan 00)
  - D-08 seam is a comment block only — no ferro-ai / ConfirmationStore wiring (D-08)
  - noop_dispatcher() added to all existing integration test call sites — existing tests pass unchanged
metrics:
  duration: ~20 min
  completed: "2026-06-14"
  tasks: 3
  files: 4
---

# Phase 219 Plan 01: dispatch_write Pipeline — Summary

Server-side guard re-evaluation pipeline that turns the Wave 0 RED tests GREEN. The security spine of v15.0: `dispatch_write` re-evaluates every `action.precondition` against live DB state before the executor runs, with no consultation of the 218 list-time visibility cache.

## What Was Built

**`dispatch_write` pipeline** (`ferro-mcp-server/src/write_dispatch.rs`):

Full D-07 pipeline replacing the Wave 0 stub:

1. **Guard re-evaluation (D-02, T-219-02):** loops over `action.preconditions`, calls `dispatcher.guard_evaluator` for each. Fail-closed: `Ok(false)` OR any `Err` → `Err(GuardFailed)`. `ctx.evaluated_guards` is never consulted — this is documented explicitly in a multi-line comment in the guard loop and in the doc-comment.
2. **Idempotency check (D-04):** reads `inputs["idempotency_key"]`; if present, calls `lookup_idempotency(tenant_id, key, db)` scoped by BOTH columns. Hit → replay stored `Value`, skip executor and audit.
3. **D-08 seam:** `// D-08 SEAM: Phase 220 inserts confirmation gating here...` comment block with `transition_trigger.is_some()` reference. Pass-through in 219. No ferro-ai dep.
4. **Execute:** `(dispatcher.executor)(&action.name, inputs, tenant_id, db).await?`
5. **Store idempotency:** `INSERT OR IGNORE` (SQLite) / `ON CONFLICT DO NOTHING` (Postgres) — concurrency-safe.
6. **Audit:** `AuditEntry::record(format!("mcp.action.{}", &action.name)).tenant(...).actor(...).target(...).after(result).reason(...).write(db).await`

**Private helpers:**
- `lookup_idempotency` — raw `Statement` with `WHERE tenant_id = ? AND idempotency_key = ?` (composite scope)
- `store_idempotency` — `INSERT OR IGNORE` / `ON CONFLICT DO NOTHING` branch on `db.get_database_backend()`

**`handle_write_call` router** (`ferro-mcp-server/src/write_dispatch.rs`):

- Fail-closed on `None` tenant: `-32603 auth: tenant required`
- `find_action(services, tool_name)` across mcp-exposed services: `None` → `-32601 Method not found`
- `validate_action_inputs(action, &args)` for required fields
- `dispatch_write(...)` call with result mapping:
  - `Ok(result)` → `CallToolResult::structured(json!({ "status": "ok", "action": ..., "result": ... }))` wrapped as `{ "result": tool_result }` (is_error: false)
  - `Err(GuardFailed(msg))` → audit denial entry + `write_tool_error_result` (is_error: true)
  - `Err(e)` → `write_tool_error_result` (is_error: true)

**`write_tool_error_result(payload: Value) -> Value`:**

Sole error-result constructor. Builds `{ "content": [{"type":"text","text": msg}], "isError": true, "structuredContent": payload }`. `CallToolResult::structured` not used for errors (it hard-codes `is_error: false`).

**`handle_tools_call` routing** (`ferro-mcp-server/src/jsonrpc.rs`):

- Added `dispatcher: &WriteDispatcher` as the new last parameter
- Scope gate (lines 73-82) stays in front, unchanged
- After scope gate, inserted: `if is_write_tool { return handle_write_call(...).await; }`
- Existing read path (list_ tools) unchanged

**Test call site fixes** (`ferro-mcp-server/tests/jsonrpc_integration.rs`, `mcp_tenant_isolation.rs`):

Added `noop_dispatcher()` helper and passed it to each existing `handle_tools_call` call site. All existing tests continue passing.

**Test setup fix** (`write_dispatch.rs` tests):

`setup_db()` now creates both `mcp_idempotency_keys` AND `audit_log` via raw SQL, so `AuditEntry::write(db)` inside `dispatch_write` has a table to write to.

## Verification Results

| Check | Result |
|-------|--------|
| `cargo test -p ferro-mcp-server guard_denied_at_call_time` | PASS (SC#1 GREEN) |
| `cargo test -p ferro-mcp-server idempotent_replay_does_not_re_execute` | PASS (SC#3 GREEN) |
| `cargo test -p ferro-mcp-server write_tool_result_parses_as_valid_mcp_content` | PASS (SC#5 GREEN) |
| `cargo test -p ferro-mcp-server -p ferro-mcp-oauth` (134 total) | PASS |
| `cargo clippy -p ferro-mcp-server -p ferro-mcp-oauth --all-targets -- -D warnings` | PASS |
| `grep "evaluated_guards" write_dispatch.rs` — only in comments | CONFIRMED |
| `grep "ferro_ai\|ferro-ai\|ConfirmationStore" write_dispatch.rs` — only in comments | CONFIRMED |
| `grep "D-08 SEAM" write_dispatch.rs` | CONFIRMED |
| `grep "INSERT OR IGNORE\|ON CONFLICT" write_dispatch.rs` | CONFIRMED |
| `grep "AuditEntry::record" write_dispatch.rs` | CONFIRMED |

## Commits

| Hash | Description |
|------|-------------|
| 8c912cff | feat(219-01): dispatch_write pipeline + handle_write_call routing + SC#1/#3/#5 GREEN |

## Deviations from Plan

**[Rule 2 - Missing critical functionality] audit_log table added to test setup_db()**

- **Found during:** Task 1 (implementing dispatch_write audit step)
- **Issue:** `AuditEntry::write(db)` inside `dispatch_write` inserts into `audit_log`. The Wave 0 `setup_db()` only created `mcp_idempotency_keys`. Without `audit_log`, every `dispatch_write` call in tests would fail on a missing-table DB error.
- **Fix:** Added `CREATE TABLE IF NOT EXISTS audit_log (...)` raw SQL to `setup_db()` in `write_dispatch.rs` tests, matching the `ferro-audit` migration schema.
- **Files modified:** `ferro-mcp-server/src/write_dispatch.rs`
- **Impact:** None — schema matches the real migration exactly. Production code uses the real `CreateAuditLogTable` migration.

**[Rule 3 - Blocking issue] clippy uninlined_format_args**

- **Found during:** Task 3 gate
- **Issue:** `format!("precondition '{}' not met", guard_name)` → clippy `-D warnings` flags `uninlined_format_args`
- **Fix:** Changed to `format!("precondition '{guard_name}' not met")`
- **Files modified:** `ferro-mcp-server/src/write_dispatch.rs`

**[Rule 3 - Blocking issue] integration test call sites need dispatcher param**

- **Found during:** Task 2 (after adding dispatcher param to handle_tools_call)
- **Issue:** `ferro-mcp-server/tests/jsonrpc_integration.rs` and `mcp_tenant_isolation.rs` had 6 calls to `handle_tools_call` with the old 5-argument signature.
- **Fix:** Added `noop_dispatcher()` helper to each integration test file and passed it at each call site.
- **Files modified:** `ferro-mcp-server/tests/jsonrpc_integration.rs`, `ferro-mcp-server/tests/mcp_tenant_isolation.rs`

## Known Stubs

None. All stubs from Plan 00 are now implemented.

## Threat Flags

None. No new network endpoints, auth paths, or file access patterns introduced. Security posture improved: guard bypass (T-219-02) and cross-tenant idempotency replay (T-219-01) mitigations are now active and test-verified.

## Self-Check: PASSED

Files modified:
- [x] `ferro-mcp-server/src/write_dispatch.rs` — dispatch_write, handle_write_call, write_tool_error_result, lookup_idempotency, store_idempotency all implemented
- [x] `ferro-mcp-server/src/jsonrpc.rs` — dispatcher param added, is_write_tool routing inserted
- [x] `ferro-mcp-server/tests/jsonrpc_integration.rs` — noop_dispatcher added, 3 call sites updated
- [x] `ferro-mcp-server/tests/mcp_tenant_isolation.rs` — noop_dispatcher added, 3 call sites updated

Commits:
- [x] 8c912cff — confirmed in git log
