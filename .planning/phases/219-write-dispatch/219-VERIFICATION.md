---
phase: 219-write-dispatch
verified: 2026-06-14T00:00:00Z
status: passed
score: 5/5
overrides_applied: 0
re_verification: false
---

# Phase 219: Write Dispatch — Verification Report

**Phase Goal:** An agent can invoke a write tool and the server executes the action tenant-scoped with guards re-evaluated at execution time, idempotency enforced, and an audit trail recorded.
**Verified:** 2026-06-14T00:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `dispatch_write()` re-evaluates the action's guard at execution using LIVE DB state; a direct tools/call on a guarded action with a tenant whose guard is false returns an MCP error — not a successful execution. Does NOT consult `ctx.evaluated_guards`. Fail-closed on unknown guard names. | VERIFIED | `dispatch_write` loops over `action.preconditions` calling `dispatcher.guard_evaluator` (live DB) before the executor. Comment explicitly states `ctx.evaluated_guards` is never consulted. CR-02 fix (commit `127c3ada`) changed the unknown-guard default from `Ok(true)` to `Err(GuardFailed)` in both production (`app/src/controllers/mcp.rs:111-113`) and test (`mcp_write_dispatch.rs:229-231`) dispatchers. SC#1 test `guard_denied_at_call_time` GREEN (3/3 write_dispatch tests pass). |
| 2 | All write operations route through the `TenantScoped` contract; a cross-tenant write fixture (tenant A targeting tenant B's resource) asserts failure, not silent success. | VERIFIED | Executor inlines `Entity::find_by_id(id).filter(Column::TenantId.eq(tenant_id))` — `None` maps to `Err(Validation("not found or cross-tenant access denied"))`. `TenantScoped` impl on `Order` also uses this filter. `cross_tenant_write_denied` test (SC#2): order id=3 (tenant 2), called as tenant 1 → `isError:true`, order status unchanged ("submitted"). Test GREEN. |
| 3 | Two calls with the same idempotency_key return the same result; the second does not re-execute — exactly one DB write after two identical calls (scoped by tenant_id AND idempotency_key). | VERIFIED | `dispatch_write` checks idempotency before executing: `lookup_idempotency(tenant_id, key, db)` with SQL `WHERE tenant_id=? AND idempotency_key=?` (composite scope). On hit → returns stored result, skips execute + audit. `store_idempotency` uses `INSERT OR IGNORE` (SQLite) / `ON CONFLICT DO NOTHING` (Postgres). Unit test `idempotent_replay_does_not_re_execute` uses `AtomicUsize` exec_count; asserts count==1 after two calls. E2E test `idempotent_write_e2e` does the same through `handle_tools_call`. Both GREEN. |
| 4 | Each write tool call produces an audit log entry (ferro-audit) with tool name, tenant ID, action name, relevant parameter IDs — recoverable after the fact. | VERIFIED | `dispatch_write` calls `AuditEntry::record(format!("mcp.action.{}", action.name)).tenant(...).actor(...).target(AuditTarget::new(&action.name, record_id)).after(result.clone()).write(db)` after successful execution. Guard-denied path also records an audit entry in `handle_write_call`. Test `write_call_produces_audit_entry`: calls `history_for_target(&AuditTarget::new("submit", "1"), &db)`, asserts entry non-empty, `action == "mcp.action.submit"`, `tenant_id == Some("1")`, `after.is_some()`. GREEN. |
| 5 | `CallToolResult::structured` is the result constructor for every write tool response; no hand-built bare content[] arrays (success structured; errors via write_tool_error_result isError:true). | VERIFIED | Success path (`jsonrpc.rs:372`): `CallToolResult::structured(payload)`. All error paths go through `write_tool_error_result` (the sole error-result constructor, `write_dispatch.rs:114-125`). Only one `"content"` literal exists in `write_dispatch.rs` (line 121, inside `write_tool_error_result`). CR-01 fix separates `Validation`/`ActionNotFound` (message passed through) from all other errors (redacted to "write operation failed"). SC#5 test `write_tool_result_parses_as_valid_mcp_content` asserts success parses as `CallToolResult` with `is_error==Some(false)` and guard-denied with `is_error==Some(true)`. GREEN. |

**Score:** 5/5 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-mcp-server/src/write_dispatch.rs` | WriteDispatcher, dispatch_write pipeline, handle_write_call, write_tool_error_result | VERIFIED | All types present. dispatch_write implements 6-step pipeline (guard re-eval, idempotency, D-08 seam, execute, store, audit). handle_write_call routes with tenant fail-closed, ActionDef resolution, input validation, structured envelopes. |
| `ferro-mcp-server/src/error.rs` | GuardFailed, ActionNotFound, Validation variants | VERIFIED | Confirmed via earlier read of jsonrpc.rs and write_dispatch.rs error matches. All three variants in use. |
| `ferro-mcp-oauth/src/migration.rs` | MigrationMcpIdempotencyKeys with composite UNIQUE (tenant_id, idempotency_key) | VERIFIED | Lines 191-271: struct present, `Index::create().name("idx_mcp_idempotency_keys_tenant_key").col(TenantId).col(IdempotencyKey).unique()`. Migration test `mcp_idempotency_keys_migration_creates_table_and_indexes` confirms. |
| `app/src/models/orders.rs` | TenantScoped impl for Order filtering by tenant_id | VERIFIED | Lines 19-34: `impl TenantScoped for Model`, `filter(Column::TenantId.eq(tenant_id))` present. |
| `app/src/controllers/mcp.rs` | make_write_dispatcher() + threaded WriteDispatcher to handle_tools_call | VERIFIED | `make_write_dispatcher()` at line 52; executor uses `Column::TenantId.eq(tenant_id)` (line 71); guard_evaluator has fail-closed default (line 111-113); `handle_tools_call(..., &dispatcher)` at line 297. |
| `app/src/tests/mcp_write_dispatch.rs` | SC#2 cross-tenant, SC#4 audit, SC#3 e2e idempotency fixtures | VERIFIED | All three test functions present: `cross_tenant_write_denied`, `write_call_produces_audit_entry`, `idempotent_write_e2e`. All GREEN. |
| `app/src/migrations/mod.rs` | Both idempotency and audit migrations registered | VERIFIED | Lines 30-31: `m20260614_create_mcp_idempotency_keys_table::Migration` and `m20260614_create_audit_log_table::Migration`. Local wrapper pattern resolves `DeriveMigrationName` collision. |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ferro-mcp-server/src/lib.rs` | `write_dispatch.rs` | `pub mod write_dispatch + pub use` | VERIFIED | `use crate::write_dispatch::{handle_write_call, WriteDispatcher}` in jsonrpc.rs confirms export. |
| `ferro-mcp-server/Cargo.toml` | ferro-audit | path dependency | VERIFIED | `ferro-audit = { path = "../ferro-audit", version = "0.2" }` at line 16. No async-trait. |
| `ferro-mcp-server/src/jsonrpc.rs` | `write_dispatch.rs` | `handle_tools_call` routes `is_write_tool` to `handle_write_call` | VERIFIED | Lines 69-86: scope gate runs, then `if is_write_tool { return handle_write_call(...) }`. Scope gate is BEFORE the routing. |
| `write_dispatch.rs` dispatch_write | ferro-audit | `AuditEntry::record` builder chain | VERIFIED | Lines 301-309 (success path) and 378-385 (guard-denied path). |
| `app/src/controllers/mcp.rs` | `ferro-mcp-server handle_tools_call` | passes `make_write_dispatcher()` as dispatcher arg | VERIFIED | Line 296-297: `let dispatcher = make_write_dispatcher(); handle_tools_call(params, &services, db.inner(), tenant_id, &ctx, &dispatcher)`. |
| `app/src/controllers/mcp.rs` executor | `Order::find_for_tenant` (inline) | `Entity::find_by_id(id).filter(Column::TenantId.eq(tenant_id))` | VERIFIED | Lines 70-79: cross-tenant denial via find-then-mutate, `None -> Err(Validation)`. |
| `app/src/tests/mcp_write_dispatch.rs` | `ferro_audit::history_for_target` | audit recovery assertion | VERIFIED | Line 316: `history_for_target(&AuditTarget::new("submit", "1"), &db)`. |

---

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `write_dispatch.rs:dispatch_write` | `result` (executor output) | `(dispatcher.executor)(action.name, inputs, tenant_id, db)` — real SeaORM update | Yes — order status transitioned in DB, returned as `{"id": ..., "status": ...}` | FLOWING |
| `write_dispatch.rs:lookup_idempotency` | stored result | `SELECT result FROM mcp_idempotency_keys WHERE tenant_id=? AND idempotency_key=?` | Yes — parameterized SQL against real table | FLOWING |
| `mcp_write_dispatch.rs` test fixtures | idempotency/audit tables | `Migrator::up(&db, None)` creates real tables in in-memory SQLite | Yes — full migration run, not mocked | FLOWING |

---

### Behavioral Spot-Checks

| Behavior | Result | Status |
|----------|--------|--------|
| SC#1: guard_denied_at_call_time — executor panics if called after guard returns Ok(false) | Err(GuardFailed(_)) returned, panic never fires | PASS |
| SC#3: idempotent_replay_does_not_re_execute — AtomicUsize counter == 1 after two calls | exec_count == 1, both results equal | PASS |
| SC#5: write_tool_result_parses_as_valid_mcp_content — CallToolResult deserialization | success is_error=Some(false), guard-denied is_error=Some(true) | PASS |
| SC#2: cross_tenant_write_denied — order id=3 (tenant 2) called as tenant 1 | isError:true, order status unchanged | PASS |
| SC#4: write_call_produces_audit_entry — history_for_target returns entry | action="mcp.action.submit", tenant_id=Some("1"), after.is_some() | PASS |
| SC#3 e2e: idempotent_write_e2e — AtomicUsize counter == 1 through full handle_tools_call | counter == 1, structuredContent identical | PASS |

All 6 spot-checks from direct test execution (cargo test -p ferro-mcp-server write_dispatch: 3/3; cargo test -p app mcp_write_dispatch: 3/3).

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| AMCP-04 | 219-00/01/02-PLAN.md | Agent can create/update/state-transition a record via write tool; execution tenant-scoped with server-side guard re-evaluation at call time (agent never trusted), idempotent, spec-compliant typed result. | SATISFIED | All 5 success criteria verified above. Guard re-eval live at call time (SC#1), tenant-scoped via find-then-mutate (SC#2), idempotency enforced (SC#3), audit trail recoverable (SC#4), CallToolResult::structured used exclusively (SC#5). |

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `app/src/models/orders.rs` | 27 | `TenantScoped::find_for_tenant` uses `ferro::DB::connection()` (global pool) not injected `db` arg | INFO | Code review finding IN-02 — documented with comment. Executor in `mcp.rs` correctly inlines the filter with the injected `db` arg; the TenantScoped impl is not called from the write path. No impact on tests or production write path. |

No blockers or warnings found. One informational pattern (IN-02) was noted in the code review and is documented via inline comment in `orders.rs`.

---

### Human Verification Required

None. All 5 success criteria have programmatic test coverage with GREEN results. Security properties (fail-closed guards, cross-tenant denial, audit trail) are proven via unit and integration tests against in-memory SQLite fixtures.

---

## Gaps Summary

No gaps. All 5 roadmap success criteria are met with direct code evidence and passing tests.

**Security-critical findings from code review were all fixed before verification:**
- CR-01 (`0daa9b1a`): internal DB error strings no longer forwarded to agent
- CR-02 (`127c3ada`): fail-closed for unknown guard names in production AND test dispatchers — the `_ => Ok(true)` fallback is gone; unregistered guard names now return `Err(GuardFailed(...))`
- WR-01 (`2f3784f1`): idempotency_key length capped at 128 characters
- WR-02 (`2f3784f1`): ExecutorFn audit PII contract documented
- WR-03 (`b3f4ff02`): authorization boundary made explicit — Gate check only for read tools, scope gate + dispatch_write guards cover write tools

The 219-02-SUMMARY contains a stale statement (`default → Ok(true)`) in its description of `make_write_dispatcher` — this was the pre-CR-02 state. The actual code at `app/src/controllers/mcp.rs:111-113` is fail-closed. The SUMMARY was written before the review fix commit; the code is the ground truth.

---

_Verified: 2026-06-14T00:00:00Z_
_Verifier: Claude (gsd-verifier)_
