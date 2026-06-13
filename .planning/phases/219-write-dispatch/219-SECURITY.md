---
phase: 219-write-dispatch
status: secured
audited_at: 2026-06-14T00:00:00Z
asvs_level: 1
threats_total: 4
threats_closed: 4
threats_open: 0
residual_risks:
  - id: RR-01
    category: info-disclosure (audit log PII)
    disposition: accepted
    detail: >
      WR-02 (accepted): audit_log `after` field stores the raw executor result verbatim.
      Current executor returns only `{"id", "status"}` which is safe. The ExecutorFn
      type alias now carries a doc-comment audit contract forbidding PII in returned
      Values. No runtime scrub at dispatch_write call site — executor is the enforcement
      point. Risk accepted; executor conformance is a code-review obligation for future
      action registrations.
  - id: RR-02
    category: info-disclosure (guard message in audit log)
    disposition: accepted
    detail: >
      IN-01 (informational): denial audit entry stores the GuardFailed message in the
      `after.guard` field, which includes the guard name. Not client-facing. Acceptable
      for forensic purposes; guard names in this codebase carry no sensitive data.
  - id: RR-03
    category: architectural (TenantScoped global pool)
    disposition: accepted
    detail: >
      IN-02 (informational): TenantScoped::find_for_tenant on Order uses the global
      connection pool (ferro::DB::connection()), not the injected `db` arg. The MCP
      write executor inlines find_by_id + TenantId filter directly using the injected
      `db`, so TenantScoped is not called on the write path. A comment was added to the
      impl warning against wiring it into the MCP executor. No security gap in current
      code.
---

# Phase 219: Security Audit Report

**Phase:** 219 — write-dispatch
**Audited:** 2026-06-14T00:00:00Z
**ASVS Level:** 1
**Block on:** high

## Summary

All four declared threats are CLOSED. Both code-review criticals (CR-01 error-leak
redaction, CR-02 fail-open unknown guard) are independently verified present in source.
No `ctx.evaluated_guards` reference appears in executable code paths. No `ferro-ai`
dependency was added. The idempotency composite UNIQUE index and length cap are both
present. Three residual risks are accepted per the WR/IN findings in 219-REVIEW.md.

## Threat Verification

| Threat ID | Category | Disposition | Status | Evidence |
|-----------|----------|-------------|--------|----------|
| T-219-02 | Elevation (guard bypass) | mitigate | CLOSED | See below |
| T-219-01 | Elevation (cross-tenant write / BOLA) | mitigate | CLOSED | See below |
| T-219-03 | Tampering (retry double-write) | mitigate | CLOSED | See below |
| T-219-PI | Info Disclosure (prompt injection / error leak) | mitigate | CLOSED | See below |

---

### T-219-02 — Guard Bypass (Elevation of Privilege)

**Declared mitigation:** Every `action.precondition` re-evaluated via the live
`GuardEvaluator` at call time; fail-closed (false OR error → deny); `ctx.evaluated_guards`
NEVER consulted. Unknown guard names fail-closed (CR-02 fix).

**Verified:**

1. `ferro-mcp-server/src/write_dispatch.rs:246-255` — `dispatch_write` iterates
   `action.preconditions` and calls `(dispatcher.guard_evaluator)(guard_name, ...)` for
   each. Any `Err` or `Ok(false)` returns `Err(crate::Error::GuardFailed(...))` before
   the executor is called.

2. `ferro-mcp-server/src/write_dispatch.rs:52` (doc comment) and lines 222, 242, 497 —
   ALL four occurrences of `evaluated_guards` in the file are in doc-comments or `//`
   comments. Zero references in executable code. The guard_evaluator callback is the
   sole authorization path at call time.

3. `ferro-mcp-server/src/write_dispatch.rs:499-516` — test `guard_denied_at_call_time`
   supplies a `guard_evaluator` returning `Ok(false)` and an `executor` that panics.
   Asserts `Err(GuardFailed(_))` — proves guard fires before executor.

4. `app/src/controllers/mcp.rs:104-114` — production `guard_evaluator` default arm:
   ```rust
   _ => Err(ferro_mcp_server::Error::GuardFailed(
       format!("unknown guard '{guard_name}': no evaluator registered")
   ))
   ```
   Fail-closed on unknown guard names (CR-02, commit 127c3ada).

5. `app/src/tests/mcp_write_dispatch.rs:229-232` — test `make_test_write_dispatcher`
   default arm identical: `Err(GuardFailed("unknown guard ..."))`.

**Status: CLOSED**

---

### T-219-01 — Cross-Tenant Write / BOLA (Elevation of Privilege)

**Declared mitigation:** Executor loads target via `find_for_tenant(id, tenant_id)` /
filtered query — `None` → deny BEFORE mutation. `tenant_id` is the authenticated
principal, NEVER from the payload. Idempotency lookup scoped by `(tenant_id,
idempotency_key)` composite.

**Verified:**

1. `ferro-mcp-server/src/write_dispatch.rs:42` — `ExecutorFn` type comment: `i64, //
   tenant_id (from auth, never from payload)`.

2. `ferro-mcp-server/src/write_dispatch.rs:336-341` — `handle_write_call` unwraps
   `tenant_id: Option<i64>` from the function parameter (the authenticated principal
   threaded from `handle_tools_call`). `None` → returns `-32603 auth: tenant required`
   before any dispatch.

3. `app/src/controllers/mcp.rs:290` — `let tenant_id = ferro::current_tenant().map(|t|
   t.id);` — sourced from the framework's authenticated current_tenant, not from the
   MCP payload.

4. `app/src/controllers/mcp.rs:68-80` — executor inlines `Entity::find_by_id(id as
   i32).filter(Column::TenantId.eq(tenant_id)).one(&db)` — returns `None` for
   cross-tenant record → `Err(Validation("not found or cross-tenant access denied"))`.
   Mutation (`active.update`) never reached on `None`.

5. `ferro-mcp-server/src/write_dispatch.rs:139,147` — idempotency lookup SQL:
   `WHERE tenant_id = ? AND idempotency_key = ?` — composite scope.

6. `app/src/tests/mcp_write_dispatch.rs:268-289` — test `cross_tenant_write_denied`
   calls `submit` for order id=3 (tenant 2) as tenant 1. Asserts `isError == true` AND
   verifies order 3 `status` is unchanged in the DB.

7. `app/src/models/orders.rs:19-33` — `TenantScoped` impl filters by both `id` AND
   `Column::TenantId.eq(tenant_id)`. (Not called on the MCP write path — executor
   inlines the filter directly using the injected `db` arg — but the impl is correct
   and consistent with D-03.)

**Status: CLOSED**

---

### T-219-03 — Retry Double-Write (Tampering)

**Declared mitigation:** Composite UNIQUE `(tenant_id, idempotency_key)`;
lookup-before-execute; `INSERT OR IGNORE` / `ON CONFLICT DO NOTHING`; replay without
re-execute; exactly one DB write after two identical calls.

**Verified:**

1. `ferro-mcp-oauth/src/migration.rs:235-242` — `MigrationMcpIdempotencyKeys` creates
   index `idx_mcp_idempotency_keys_tenant_key` with both `McpIdempotencyKeys::TenantId`
   and `McpIdempotencyKeys::IdempotencyKey` and `.unique()`.

2. `ferro-mcp-oauth/src/migration.rs:461-469` — migration test
   `mcp_idempotency_keys_migration_creates_table_and_indexes` asserts the composite
   unique index name is present in sqlite_master.

3. `ferro-mcp-server/src/write_dispatch.rs:265-279` — length check (`key.len() > 128`
   → `Err(Validation(...))`) then lookup-before-execute: if stored result found,
   `return Ok(stored_result)` (replay) before executor fires.

4. `ferro-mcp-server/src/write_dispatch.rs:189-200` — `store_idempotency` uses:
   - Postgres: `ON CONFLICT (tenant_id, idempotency_key) DO NOTHING`
   - SQLite: `INSERT OR IGNORE INTO mcp_idempotency_keys`

5. `ferro-mcp-server/src/write_dispatch.rs:524-559` — test
   `idempotent_replay_does_not_re_execute` with `AtomicUsize` exec_count asserts both
   results equal and `exec_count == 1`.

6. `app/src/tests/mcp_write_dispatch.rs:352-456` — e2e test `idempotent_write_e2e`
   drives two identical `handle_tools_call` invocations with `idempotency_key:
   "e2e-idem-key-001"`, asserts `exec_count == 1` and equal structured content.

7. WR-01 (idempotency_key length cap) verified: `ferro-mcp-server/src/write_dispatch.rs:266`
   — `if key.len() > 128 { return Err(Validation(...)) }` — present before
   lookup/store, closing the unbounded-TEXT DoS surface.

**Status: CLOSED**

---

### T-219-PI — Info Disclosure (Prompt Injection / Error Leak)

**Declared mitigation:** Results use `CallToolResult::structured` / `write_tool_error_result`
typed envelopes. Internal DB error strings NOT forwarded verbatim (CR-01 fix, commit
0daa9b1a): only `Validation` / `ActionNotFound` / `GuardFailed` messages pass through;
all other variants return generic `"write operation failed"`.

**Verified:**

1. `ferro-mcp-server/src/write_dispatch.rs:114-125` — `write_tool_error_result` is the
   sole error-result constructor. Extracts `"message"` from the payload and wraps in
   `content: [{type: "text", text: ...}], isError: true`.

2. `ferro-mcp-server/src/write_dispatch.rs:392-407` — `handle_write_call` match arms:
   ```rust
   Err(ref e @ crate::Error::Validation(_))
   | Err(ref e @ crate::Error::ActionNotFound(_)) => {
       // passes e.to_string() — safe: caller-supplied strings only
   }
   Err(_) => {
       // ALL other variants (Database, Serialization, Auth, etc.) → generic string
       json!({ "result": write_tool_error_result(json!({
           "error_kind": "execution_error",
           "message": "write operation failed"
       })) })
   }
   ```
   `GuardFailed` is handled in a separate arm (line 375) before these, passing only the
   guard-denied message (no internal DB state).

3. `ferro-mcp-server/src/write_dispatch.rs:571-631` — test
   `write_tool_result_parses_as_valid_mcp_content` asserts success result parses as
   `CallToolResult` with `is_error == Some(false)` and guard-denied result with
   `is_error == Some(true)`.

4. No `ferro-ai` dependency in `ferro-mcp-server/Cargo.toml` (verified via grep).
   The D-08 seam comment at line 281-284 is a comment-only reference; no import or
   call to `ferro_ai` exists.

**Status: CLOSED**

---

## Scope Gate (WR-03) — Authorization Boundary

WR-03 (commit b3f4ff02) restructured `app/src/controllers/mcp.rs` so the Gate check
runs only for `list_` tools (inside the `if let Some(service_name) =
tool_name.strip_prefix("list_")` block at line 235). Write tools fall through to
`handle_tools_call` which enforces:
- Scope gate (Phase 217, `jsonrpc.rs:71`) — read-scoped API key rejected before dispatch
- `dispatch_write` guard re-evaluation (D-02) — live DB guards at call time

The 14-line authorization-boundary comment at `app/src/controllers/mcp.rs:220-234`
documents this split explicitly. No residual authorization gap for write tools: the
scope gate at `jsonrpc.rs:71` fires before `is_write_tool` routing at line 84, covering
all write-tool calls including those from unauthenticated contexts.

---

## Unregistered Threat Flags

None. No `## Threat Flags` section was present in the SUMMARY files for Phase 219.

---

## Accepted Risks Log

| Risk ID | Finding | Accepted Because |
|---------|---------|-----------------|
| RR-01 | WR-02: audit log `after` field may carry PII from future executors | Current executor returns only `{"id","status"}`; ExecutorFn type alias has an explicit doc-comment audit contract. Scrub at dispatch_write would require field-level knowledge of each executor's schema — delegated to executor authors. |
| RR-02 | IN-01: denial audit `after.guard` field stores guard name | Audit-internal only, not returned to agent. Guard names in current codebase carry no sensitive data. |
| RR-03 | IN-02: TenantScoped::find_for_tenant uses global pool | Not called on MCP write path; executor inlines filter with injected `db`. Comment added to impl. |
