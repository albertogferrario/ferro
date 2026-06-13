---
phase: 219-write-dispatch
reviewed: 2026-06-14T00:00:00Z
depth: standard
files_reviewed: 9
files_reviewed_list:
  - ferro-mcp-server/src/write_dispatch.rs
  - ferro-mcp-server/src/jsonrpc.rs
  - ferro-mcp-server/src/error.rs
  - ferro-mcp-oauth/src/migration.rs
  - app/src/controllers/mcp.rs
  - app/src/models/orders.rs
  - app/src/migrations/m20260614_create_mcp_idempotency_keys_table.rs
  - app/src/migrations/m20260614_create_audit_log_table.rs
  - app/src/tests/mcp_write_dispatch.rs
findings:
  critical: 2
  warning: 3
  info: 2
  total: 7
status: issues_found
---

# Phase 219: Code Review Report

**Reviewed:** 2026-06-14T00:00:00Z
**Depth:** standard
**Files Reviewed:** 9
**Status:** issues_found

## Summary

Phase 219 delivers the write-dispatch security envelope for the MCP endpoint. The core pipeline — guard re-evaluation, idempotency, seam, execute, audit — is structurally correct and the most critical invariants are in place: `dispatch_write` never consults `ctx.evaluated_guards`, guard failure is tested before executor invocation, and all SQL in `write_dispatch.rs` is fully parameterized. The migration strategy (local wrappers with file-stem names) correctly solves the version-collision problem.

Two critical issues require fixes before this lands:

1. **Internal DB/execution error messages are forwarded verbatim to the agent** in the `execution_error` branch. A SeaORM error string can include table names, column values, constraint names, and SQL fragments — this is an information disclosure on every executor failure.
2. **Unknown guard names are silently allowed** in both the production `guard_evaluator` and the test dispatcher. If a future `ActionDef` declares a precondition whose name is not recognized by the evaluator, the guard passes. This inverts the fail-closed invariant for unknown guard names and creates a silent privilege escalation if the action name drifts.

Three warnings cover: (a) the double service-lookup in the controller for write tools carrying gate overhead twice, (b) the audit `after` field on write-path success stores the raw executor result, which may contain PII that should be scrubbed at the projection layer, and (c) `idempotency_key` is consumed directly from `inputs` but is not declared in `ActionDef.inputs`, so `validate_action_inputs` never validates or strips it before forwarding to the executor.

---

## Critical Issues

### CR-01: Execution errors leak internal detail to agent response

**File:** `ferro-mcp-server/src/write_dispatch.rs:369-374`

**Issue:** The catch-all `Err(e)` arm in `handle_write_call` serializes `e.to_string()` directly into the `"message"` field of the MCP tool-error response returned to the agent:

```rust
Err(e) => {
    json!({ "result": write_tool_error_result(json!({
        "error_kind": "execution_error",
        "message": e.to_string()   // ← raw internal error forwarded verbatim
    })) })
}
```

`e` at this point is `crate::Error`, which wraps `sea_orm::DbErr::Query(...)` strings verbatim via `crate::Error::Database(e.to_string())` in both the executor (`app/src/controllers/mcp.rs:73-74`, `93-94`) and in `store_idempotency`/`lookup_idempotency`. SeaORM query error strings include the SQL fragment, table name, column names, and constraint names. The `Validation("not found or cross-tenant access denied")` variant is safe, but any DB-level failure (connection drop, constraint violation, lock timeout) exposes internals.

**Fix:** Classify before forwarding. Only pass through `Validation` and `ActionNotFound` messages; redact everything else to a generic string.

```rust
Err(ref e @ crate::Error::Validation(_))
| Err(ref e @ crate::Error::ActionNotFound(_)) => {
    json!({ "result": write_tool_error_result(json!({
        "error_kind": "execution_error",
        "message": e.to_string()
    })) })
}
Err(_) => {
    json!({ "result": write_tool_error_result(json!({
        "error_kind": "execution_error",
        "message": "write operation failed"
    })) })
}
```

---

### CR-02: Unknown guard names are silently allowed (fail-open for future guards)

**File:** `app/src/controllers/mcp.rs:107-110` and `app/src/tests/mcp_write_dispatch.rs:224`

**Issue:** The production `guard_evaluator` in `make_write_dispatcher` and the test dispatcher in `make_test_write_dispatcher` both default to `Ok(true)` for unrecognized guard names:

```rust
// app/src/controllers/mcp.rs:104-110
match guard_name.as_str() {
    "is_manager" => Ok(check_is_manager(tenant_id, &db).await),
    _ => Ok(true),   // ← unknown guard = pass
}
```

`dispatch_write` calls `guard_evaluator` for every name in `action.preconditions`. If an `ActionDef` is later annotated with a guard whose name the evaluator doesn't recognize (e.g., `"is_admin"`, `"order_in_draft_state"`), the guard silently passes and the action executes. This is the opposite of fail-closed. The issue is latent now (only `"is_manager"` is declared), but the pattern established here will propagate to every new action.

**Fix:** Default to `Err(...)` for unknown guards so `dispatch_write`'s `map_err(|e| crate::Error::GuardFailed(...))` fires:

```rust
_ => Err(ferro_mcp_server::Error::GuardFailed(
    format!("unknown guard '{guard_name}': no evaluator registered")
)),
```

Apply the same change in the test dispatcher (`app/src/tests/mcp_write_dispatch.rs:224`). If there are guards that genuinely should be passthrough (e.g., guards evaluated only at list-time), model them explicitly rather than via a wildcard `Ok(true)`.

---

## Warnings

### WR-01: `idempotency_key` extracted from inputs but not declared in ActionDef, bypasses validation

**File:** `ferro-mcp-server/src/write_dispatch.rs:250`

**Issue:** `dispatch_write` reads `idempotency_key` directly from `inputs`:

```rust
let idempotency_key = inputs.get("idempotency_key").and_then(|v| v.as_str());
```

`validate_action_inputs` (line 88) checks only fields declared in `action.inputs`. Since no `ActionDef` in the codebase declares `idempotency_key` as an input, the key is never validated — its length, character set, and format are unconstrained. An agent can supply an arbitrarily long or specially-crafted idempotency key. The key is bound via parameterized SQL so SQL injection is not the concern, but an unbounded string stored in the DB without any length constraint is a denial-of-service surface against storage. The `mcp_idempotency_keys` table defines `idempotency_key` as `TEXT NOT NULL` with no length limit in either the migration or the runtime check.

**Fix:** Add a length cap before lookup/store:

```rust
let idempotency_key = inputs.get("idempotency_key")
    .and_then(|v| v.as_str())
    .filter(|k| k.len() <= 128);   // reject oversized keys
```

Alternatively, declare `idempotency_key` as a standard `InputDef` field (optional, string, max length) and let `validate_action_inputs` handle it uniformly.

---

### WR-02: Audit `after` field stores raw executor result — may contain PII

**File:** `ferro-mcp-server/src/write_dispatch.rs:283`

**Issue:** The audit entry stores the complete executor result in the `after` field:

```rust
.after(result.clone())
```

In the current executor the result is `json!({ "id": updated.id, "status": updated.status })`, which is safe. However the `ExecutorFn` type contract makes no restriction on what a future executor may return. If an executor returns a result containing customer names, email addresses, amounts, or other fields, those values enter the append-only audit log. The audit log is an unencrypted append-only table; writes from any tenant are co-mingled, and if audit-log readers do not filter by `tenant_id`, PII from one tenant can be visible in another's audit read.

**Fix:** Document the contract on `ExecutorFn`: the returned `Value` is stored verbatim in the audit log and must contain only identifiers and status values, never PII fields. Alternatively, introduce an audit-scrub projection at the `dispatch_write` call site that allows through only known-safe keys (`id`, `status`, `action`).

---

### WR-03: Double service-lookup in controller adds Gate overhead for write-tool calls

**File:** `app/src/controllers/mcp.rs:213-227`

**Issue:** For write-tool calls, the controller resolves the service by name and runs the Gate check (lines 213-264), then calls `handle_tools_call` which calls `handle_write_call` which calls `find_action` — a second lookup over the same services slice. The Gate check and the `find_action` lookup have different semantics: the controller uses `strip_prefix("list_")` for write-tool names (line 214), which means for a write tool named `"submit"`, `service_name` becomes `"submit"` and the `find(|s| s.name == service_name)` at line 218 will fail to find the `"order"` service. The `None` arm returns a 32601 Method-not-found error before `handle_tools_call` is ever reached — so the Gate check at line 255 is never exercised for write tools, and the `None` arm short-circuits correctly but for the wrong service-name.

This is currently a logic dead-letter: write tools reach the `None => return Method not found` branch because `"submit".strip_prefix("list_")` is still `"submit"`, not `"order"`. The Gate check for write-tool calls is therefore never reached. The security consequence depends on whether Gate rejection is relied upon for write-tool calls — if it is, the gap exists now.

**Fix:** For write-tool calls the controller does not need to look up the service for the Gate check (the Gate check is on the action, not the service; the service lookup for Gate purposes should use the service that owns the action, not `service_name = strip_prefix("list_", tool_name)`). The simplest fix: apply the Gate check only for `list_` tools in the controller, and have `dispatch_write` handle authorization for write tools. If Gate coverage for write tools is required, pass the correct service (looked up from `find_action`) to `Gate::authorize_for`.

---

## Info

### IN-01: Guard-denied audit entry includes the raw guard message in `"guard"` field

**File:** `ferro-mcp-server/src/write_dispatch.rs:360`

**Issue:** The denial audit entry stores the guard error message in the `after` payload:

```rust
.after(json!({ "denied": true, "reason": "guard_failed", "guard": msg }))
```

`msg` is the string from `crate::Error::GuardFailed(format!("{guard_name}: {e}"))`, which includes the guard name. This is useful forensically and is not a client-facing disclosure (the `write_tool_error_result` message is separate and passes `msg` there too). Worth noting that if guard names encode business-sensitive state (e.g. `"order_value_exceeds_50k"`), the audit log now contains that signal per tenant. Acceptable as-is; document the invariant.

---

### IN-02: `TenantScoped::find_for_tenant` in `orders.rs` uses global connection pool, not the injected `db` arg

**File:** `app/src/models/orders.rs:26-33`

**Issue:** `TenantScoped::find_for_tenant` acquires the connection from `ferro::DB::connection()` (the global pool), while the write executor in `app/src/controllers/mcp.rs:59-96` correctly uses the `db` arg injected from `dispatch_write`. The `TenantScoped` impl is not called from the write executor (the executor inlines the `find_by_id + filter` pattern directly). But the `TenantScoped` impl is a public API and a reader of this code may expect it to be used; if it ever is wired into the executor, tests using in-memory DBs will silently fail because `DB::connection()` returns the global pool (which has no test data).

This is documentation-level only — the executor correctly avoids this trap. Add a comment on `TenantScoped::find_for_tenant` noting it uses the global pool and is unsuitable for the MCP write path, where the `db` arg must be threaded through.

---

_Reviewed: 2026-06-14T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
