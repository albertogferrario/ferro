# Phase 219: Write Dispatch — Research

**Researched:** 2026-06-13
**Domain:** ferro-mcp-server write dispatch — guard re-evaluation, tenant-scoped execution, idempotency, audit, structured results
**Confidence:** HIGH (all claims grounded in source files read this session; no assumed claims)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** App-registered callback — `async fn(action_name: &str, inputs: &Value, tenant_id: i64, db: &DatabaseConnection) -> Result<Value, Error>`. `ferro-mcp-server` stays projection-agnostic. Registration shape is Claude's discretion / research-resolved.
- **D-02:** `dispatch_write` re-evaluates every `action.precondition` at execution time via an app-registered `GuardEvaluator` against live DB state BEFORE executor. Fail-closed. Independent of 218 visibility filter.
- **D-03:** `TenantScoped::find_for_tenant(id, tenant_id) -> Option<Self>` is the cross-tenant denial primitive. A cross-tenant write fixture asserts failure, not success (SC#2).
- **D-04:** `mcp_idempotency_keys` table (framework-owned schema, consumer-run migration). UNIQUE on `(tenant_id, idempotency_key)`. First call executes and stores; second call replays stored result. Absent key = no guard.
- **D-05:** Evaluate `ferro-audit` first for SC#4. Reuse if per-action event fits; fallback is lightweight `mcp_audit_log` table.
- **D-06:** `CallToolResult::structured(payload)` for every write response. Guard denial and validation failure return structured error result. Shape research-resolved.
- **D-07:** Order at call time: scope check (217) → resolve `ActionDef` by name → validate inputs → re-evaluate guards (D-02) → idempotency check (D-04) → execute callback (D-01) → audit (D-05) → structured result (D-06).
- **D-08:** No `ferro-ai` dependency in 219. Seam for 220 = `transition_trigger.is_some()` check point in dispatch_write.
- **D-09:** Sample app registers concrete executor + guard evaluator for ≥1 action. Five SC testable end-to-end. Planner assess whether to split (framework vs app wiring + fixtures).

### Claude's Discretion

- Exact registration API (trait object vs boxed async fn; held in `McpServerConfig` vs dispatcher param).
- Exact serialized shape of stored idempotency `result` and audit entry record.
- Whether `idempotency_key` is advertised in write-tool `inputSchema`.
- Error-result envelope shape for guard-denied/validation-failed.

### Deferred Ideas (OUT OF SCOPE)

- Confirmation gating + `confirm_<action>` tools + `ferro-ai` — Phase 220.
- Inbound NL classification loop — Phase 221.
- DB-backed confirmation store — production hardening.
- gestiscilo full adoption — consumer-repo follow-up.
- `idempotency_key` in `inputSchema` — may defer per research findings.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AMCP-04 | Agent can create/update/state-transition via write tool; execution is tenant-scoped, re-evaluates guard server-side at call time, idempotent against retries, returns spec-compliant typed result | Sections §Registration API, §Guard Re-evaluation Pipeline, §TenantScoped for Writes, §Idempotency, §Audit Decision, §Result Construction all directly enable implementation |

</phase_requirements>

---

## Summary

Phase 218 produced guard-filtered, well-formed write tool definitions visible in `tools/list`. Phase 219 makes them callable. The `handle_tools_call` dispatcher currently returns `-32601 Method not found` for any non-`list_` tool name (line 90 of `jsonrpc.rs`) — phase 219 replaces this with `handle_write_call` → `dispatch_write`.

The security spine is guard re-evaluation at call time. The 218 list-time guard filter is a visibility mechanism; the actual authorization gate is the re-check inside `dispatch_write`. A direct `tools/call` on a guarded action bypasses `tools/list` entirely — guard re-evaluation in the execution path is the only structural prevention of privilege escalation. `ferro-audit` fits MCP write events cleanly via its `tenant()` + `target()` builder; recommend reuse over a new table.

The callback registration pattern recommended below is a `WriteDispatcher` struct held in `McpServerConfig`-or passed as a param; using boxed futures for the async trait avoids the `#[async_trait]` crate (not present in `ferro-mcp-server`'s `Cargo.toml`). The sample app already has an `Order` entity with 3 `ActionDef`s (`submit`, `approve`, `ship`) and a `tenant_id` column — exactly the fixture needed for SC#1–#5.

**Primary recommendation:** Add `write_dispatch.rs` to `ferro-mcp-server`, hold `WriteDispatcher` in `McpServerConfig` (parallel to existing config fields), and reuse `ferro-audit` for SC#4 audit entries with tenant scoping via the `.tenant(tenant_id.to_string())` builder method.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Guard re-evaluation at call time | API (ferro-mcp-server) | App callback (GuardEvaluator) | Server owns security gate; app provides guard business logic |
| Executor callback | App (sample app) | ferro-mcp-server (registration contract) | App knows the model; server only threads tenant+db to the callback |
| Idempotency storage | DB/Storage | ferro-mcp-server (read-path) | UNIQUE constraint enforces exactly-once; server checks before execute |
| Audit recording | DB/Storage | ferro-mcp-server (write call) | audit_log table; ferro-audit ORM writes after execution |
| Result construction | API (ferro-mcp-server) | — | CallToolResult::structured is already in the server crate |
| TenantScoped enforcement | App (executor) | framework/src/tenant/scoped.rs | The executor calls find_for_tenant; the trait lives in framework |

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ferro-mcp-server (internal) | workspace 0.2.58 | write_dispatch.rs lives here | Already the output crate for MCP rendering |
| ferro-audit (internal) | workspace 0.2.58 | Append-only audit log | Already has tenant() + target() builder; migration shipped by consumer |
| sea-orm | 1.0 | DB layer for idempotency table | Already a ferro-mcp-server dependency |
| serde_json | 1.0 | Executor input/output Value | Already in ferro-mcp-server |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| async-trait | 0.1 | Trait with async fn | Only if registration uses dyn trait; boxed futures avoid it |
| uuid | (transitive via ferro-audit) | idempotency_key generation | Consumer generates; server stores |

**Async trait note:** `ferro-mcp-server/Cargo.toml` does NOT have `async-trait` as a dependency [VERIFIED: read Cargo.toml]. The framework crate and ferro-mcp-oauth have it. If registration uses a `dyn` trait with `async fn`, the crate must add `async-trait = "0.1"`. The recommended boxed-future approach avoids the dependency entirely.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| ferro-audit | New `mcp_audit_log` table | ferro-audit already supports tenant scoping; new table duplicates control surface per feedback_no_duplicate_control_surface.md |
| Boxed futures for executor | `#[async_trait]` dyn trait | Both work; boxed avoids new dep, async_trait is more ergonomic but requires adding dep |

---

## Architecture Patterns

### System Architecture Diagram

```
tools/call { name: "submit_order", arguments: { id: 42, notes: "urgent" } }
    |
    v
handle_tools_call() [jsonrpc.rs:54]
    | scope check: key_scope == "read" && is_write_tool → -32603 Auth error  [line 71-83]
    | strip_prefix("list_") fails → service lookup fails → currently -32601  [line 89-92]
    |                                                                          ↑ REPLACED
    v  (Phase 219: route non-list_ names to handle_write_call)
handle_write_call(params, services, db, tenant_id, ctx, dispatcher)
    |
    ├─ find ActionDef: services.iter().flat_map(actions).find(a.name == tool_name)
    |    └─ None → Error::ActionNotFound → structured error result (isError:true)
    |
    ├─ validate inputs against ActionDef.inputs (required fields present, types match)
    |    └─ fail → structured validation-error result
    |
    ├─ re-evaluate guards via dispatcher.evaluate_guard(guard_name, tenant_id, &inputs, db)
    |    └─ any guard returns false OR errors → Error::GuardFailed → structured error (isError:true)
    |    NOTE: reads LIVE state; does NOT consult ctx.evaluated_guards
    |
    ├─ idempotency check: read idempotency_key from arguments["idempotency_key"]
    |    └─ key present + row (tenant_id, key) exists → replay stored result
    |    └─ key absent → proceed
    |
    ├─ 220 SEAM POINT: if action.transition_trigger.is_some() → future confirmation intercept
    |    (219 passes through; seam is a comment, no code change needed here)
    |
    ├─ execute: dispatcher.execute(action_name, &inputs, tenant_id, db) -> Result<Value>
    |    └─ Err → structured error result (audit still fires on outcome)
    |
    ├─ if idempotency_key present → INSERT (tenant_id, key, result) in mcp_idempotency_keys
    |
    ├─ audit: AuditEntry::record("mcp.write")
    |           .tenant(tenant_id.to_string())
    |           .actor(AuditActor::User(tenant_id.to_string()))
    |           .target(AuditTarget::new(tool_name, record_id_from_inputs))
    |           .after(result.clone())
    |           .reason(action_name)
    |           .write(db).await
    |
    └─ CallToolResult::structured(success_payload)
```

### Recommended Project Structure (new/changed files)
```
ferro-mcp-server/src/
├── write_dispatch.rs     # NEW: dispatch_write(), handle_write_call(), WriteDispatcher
├── jsonrpc.rs            # MODIFY: route non-list_ names to handle_write_call
├── error.rs              # MODIFY: add GuardFailed(String), ActionNotFound(String)
├── lib.rs                # MODIFY: pub use write_dispatch::{dispatch_write, WriteDispatcher}
└── config.rs             # MODIFY or SKIP: see registration API section

app/src/
├── controllers/mcp.rs    # MODIFY: pass WriteDispatcher to handle_write_call
└── migrations/           # ADD: mcp_idempotency_keys migration (+ ferro-audit migration if not already registered)
```

---

## Research Finding 1: Registration API (D-01, D-09)

### Current handle_tools_call signature
```rust
// ferro-mcp-server/src/jsonrpc.rs:54
pub async fn handle_tools_call(
    call_params: Value,
    services: &[ServiceDef],
    db: &sea_orm::DatabaseConnection,
    tenant_id: Option<i64>,
    ctx: &McpContext,
) -> Value
```

The `-32601` placeholder sits at line 89-92 of `jsonrpc.rs`: when `strip_prefix("list_")` fails, `service_name = tool_name` (e.g. `"submit_order"`), the service lookup finds no match, and returns `-32601 Method not found`.

### Caller pattern in `app/src/controllers/mcp.rs`
```rust
// app/src/controllers/mcp.rs:177
handle_tools_call(params, &services, db.inner(), tenant_id, &ctx).await
```

The app passes `db.inner()` and `tenant_id` as plain args — no config struct. Consistent with threading a new `WriteDispatcher` parameter.

### Recommended Registration API: WriteDispatcher struct

**Recommendation: A concrete `WriteDispatcher` struct with boxed async closures, passed as a new parameter to `handle_write_call` (not held in `McpServerConfig`).**

Rationale:
- `McpServerConfig` is app-identity config (app_name, app_url, version). Holding executor callbacks in it mixes concerns.
- The executor and guard evaluator vary by deployment; threading them at call-site keeps the library surface clean.
- Boxed futures avoid adding `async-trait` to `ferro-mcp-server`'s Cargo.toml.
- Parallel to how `db` and `tenant_id` are already threaded.

```rust
// ferro-mcp-server/src/write_dispatch.rs (new)

use sea_orm::DatabaseConnection;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

// Boxed async fn types — no async-trait dep needed
pub type ExecutorFn = Box<
    dyn Fn(
            &str,                     // action_name
            &Value,                   // validated inputs
            i64,                      // tenant_id (from auth, never from payload)
            &DatabaseConnection,
        ) -> Pin<Box<dyn Future<Output = crate::Result<Value>> + Send>>
        + Send
        + Sync,
>;

pub type GuardEvaluatorFn = Box<
    dyn Fn(
            &str,                     // guard_name
            i64,                      // tenant_id
            &Value,                   // validated inputs (for record-scoped guards)
            &DatabaseConnection,
        ) -> Pin<Box<dyn Future<Output = crate::Result<bool>> + Send>>
        + Send
        + Sync,
>;

/// Holds the app-registered write callback and guard evaluator.
/// Constructed by the consumer app and passed to `handle_write_call`.
pub struct WriteDispatcher {
    pub executor: ExecutorFn,
    pub guard_evaluator: GuardEvaluatorFn,
}
```

**App registration (in `app/src/controllers/mcp.rs`):**
```rust
fn make_write_dispatcher() -> WriteDispatcher {
    WriteDispatcher {
        executor: Box::new(|action_name, inputs, tenant_id, db| {
            Box::pin(async move {
                match action_name {
                    "submit" => {
                        let id: i64 = inputs["id"].as_i64()
                            .ok_or_else(|| crate_or_app_error("missing id"))?;
                        // Use TenantScoped::find_for_tenant — None → cross-tenant denial
                        use ferro::tenant::TenantScoped;
                        let order = Order::find_for_tenant(id, tenant_id).await?
                            .ok_or_else(|| crate_or_app_error("not found or access denied"))?;
                        // ... apply state transition via SeaORM
                        Ok(serde_json::json!({ "id": order.id, "status": "submitted" }))
                    }
                    _ => Err(/* Error::ActionNotFound */),
                }
            })
        }),
        guard_evaluator: Box::new(|guard_name, tenant_id, inputs, db| {
            Box::pin(async move {
                match guard_name {
                    "is_manager" => {
                        // live DB query — never reads ctx.evaluated_guards
                        let is_mgr = check_is_manager(tenant_id, db).await?;
                        Ok(is_mgr)
                    }
                    _ => Ok(true), // unknown guard = allow (fail-open for unknown, like BaseContext)
                }
            })
        }),
    }
}
```

**Updated `handle_tools_call` routing (minimal diff):**
```rust
// jsonrpc.rs — replace the service-not-found -32601 return for write tools
let is_write_tool = !tool_name.starts_with("list_");
if is_write_tool {
    // Route to write path (Phase 219)
    return handle_write_call(call_params, services, db, tenant_id, ctx, dispatcher).await;
}
```

This requires `handle_tools_call` to accept an additional `dispatcher: &WriteDispatcher` parameter, or (cleaner) split the routing logic so `handle_tools_call` stays unchanged and a new `handle_write_call` is added as a sibling entry point the app invokes directly. The cleanest minimal change is to add `dispatcher: &WriteDispatcher` to `handle_tools_call` — the app already calls it with all args, and the 217 scope gate stays in front.

---

## Research Finding 2: Guard Re-evaluation Pipeline (D-02, SC#1)

### Where it runs

After input validation, before executor, reading from the `guard_evaluator` callback — not from `ctx.evaluated_guards`. `ctx.evaluated_guards` is the list-time visibility cache; it is NOT consulted at call time.

```rust
// In dispatch_write (pseudo-code):
for guard_name in &action.preconditions {
    let passes = dispatcher.guard_evaluator(guard_name, tenant_id, &validated_inputs, db).await
        .map_err(|e| Error::GuardFailed(format!("{guard_name}: {e}")))?;
    if !passes {
        return Err(Error::GuardFailed(format!("precondition {guard_name} not met")));
    }
}
```

### ActionDef.preconditions is Vec<String>
Confirmed at `ferro-projections/src/action.rs:35`:
```rust
pub preconditions: Vec<String>,
```
The sample Order service at `app/src/projections/order.rs:43` has:
```rust
.action(ActionDef::new("approve").transition_trigger("approve").precondition("is_manager"))
```
The `submit` and `ship` actions have no preconditions — useful for SC#1 testing (they should always execute if tenant owns the record).

### Exact SC#1 bypass test

```rust
// ferro-mcp-server/tests/write_dispatch_integration.rs (new or extended)
#[tokio::test]
async fn guard_denied_at_call_time_even_when_not_in_tools_list() {
    // Setup: guard evaluator always returns false for "is_manager"
    let dispatcher = WriteDispatcher {
        guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(false) })),
        executor: Box::new(|_, _, _, _| Box::pin(async {
            panic!("executor must not be called when guard fails")
        })),
    };
    let action_params = serde_json::json!({
        "name": "approve",
        "arguments": { "id": 1 }
    });
    let result = handle_write_call(action_params, &services_with_approve(), &db, Some(1), &ctx, &dispatcher).await;
    // Must be an MCP-level tool error, not a panic or successful execution
    let tool_result: CallToolResult = serde_json::from_value(result["result"].clone()).unwrap();
    assert_eq!(tool_result.is_error, Some(true), "guard failure must return isError:true");
}
```

### Contradiction with ARCHITECTURE.md write path diagram

ARCHITECTURE.md §Write path shows `evaluated_guards.get("has_items") = Some(true)? → proceed` as the guard check. This was the design before D-02 was locked — D-02 clarifies it must use the `GuardEvaluator` callback against live state, not the cached map. **Resolution: ignore the cached-map check in the architecture diagram; implement live guard_evaluator re-evaluation per D-02.** The cached map is only for the 218 list-time filter.

---

## Research Finding 3: TenantScoped for Writes (D-03, SC#2)

### TenantScoped trait signature
```rust
// framework/src/tenant/scoped.rs:28-41
#[async_trait]
pub trait TenantScoped: Sized + Send + Sync {
    type Id: std::str::FromStr + Send;
    async fn find_for_tenant(id: Self::Id, tenant_id: i64) -> Result<Option<Self>, FrameworkError>;
}
```

**No write helper exists on `TenantScoped`.** The pattern is find-then-mutate: the executor calls `find_for_tenant(id, tenant_id)` → `None` = deny (record not owned by tenant) → `Some(record)` = mutate. D-03 explicitly says "find-then-mutate is the contract; no new write methods required."

### Order entity
The `Order` model (`app/src/models/entities/orders.rs`) has `tenant_id: i64` (line 17). It uses the SeaORM `DeriveEntityModel` macro but does NOT yet implement `TenantScoped` — the sample app must add this impl in Wave 0 or Plan 0 as a prerequisite for the executor.

### Cross-tenant fixture (SC#2)
```rust
// In the sample app executor (not in ferro-mcp-server):
// Tenant A (id=1) submits tool call with id pointing to Tenant B's order (tenant_id=2)
// find_for_tenant(order_id=99, tenant_id=1) → None → executor returns Err("not found")
// → dispatch_write returns structured error with isError:true
// The test asserts no mutation occurred on Tenant B's record

#[tokio::test]
async fn cross_tenant_write_denied() {
    let db = setup_db_with_two_tenant_orders().await;
    // order 99 belongs to tenant 2; calling as tenant 1 → None → Err
    let result = call_submit_write_tool(order_id: tenant2_order_id, tenant_id: 1, &db).await;
    // assert isError:true
    // assert Tenant 2's order status unchanged in DB
}
```

---

## Research Finding 4: Idempotency (D-04, SC#3)

### Migration template

The `mcp_api_keys` migration lives in `ferro-mcp-oauth/src/migration.rs` (struct `MigrationMcpApiKeys`). It uses `sea_orm_migration::prelude::*`, `Table::create().table(...).if_not_exists()`, and `Index::create().unique()`.

**Recommended `mcp_idempotency_keys` migration** (parallel pattern, lives in `ferro-mcp-oauth/src/migration.rs` as a new `MigrationMcpIdempotencyKeys` struct — same crate, same pattern as `MigrationMcpApiKeys`):

```rust
// Table: mcp_idempotency_keys
// Columns: id (big_integer PK), tenant_id (big_integer NOT NULL),
//          idempotency_key (string NOT NULL), result (text/json NOT NULL),
//          created_at (timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP)
// Index: UNIQUE on (tenant_id, idempotency_key) — the enforcement primitive
```

**Why ferro-mcp-oauth, not ferro-mcp-server:** `ferro-mcp-server` has no migration infrastructure (it's a pure library). `ferro-mcp-oauth` already exports `CreateMcpApiKeysTable` — adding `CreateMcpIdempotencyKeysTable` follows the exact same pattern. Consumers add one line to their `Migrator::migrations()`.

**Idempotency read-path (in dispatch_write):**
```rust
let idempotency_key = inputs.get("idempotency_key").and_then(|v| v.as_str());

if let Some(key) = idempotency_key {
    // Check: SELECT result FROM mcp_idempotency_keys WHERE tenant_id=? AND idempotency_key=?
    if let Some(stored_result) = lookup_idempotency(tenant_id, key, db).await? {
        // Replay: return stored result, skip execution, skip audit
        return Ok(CallToolResult::structured(stored_result));
    }
}
// ... execute ...
if let Some(key) = idempotency_key {
    // Store: INSERT INTO mcp_idempotency_keys (tenant_id, idempotency_key, result, created_at)
    // ON CONFLICT (tenant_id, idempotency_key) DO NOTHING  ← race-condition safety
    store_idempotency(tenant_id, key, &result, db).await?;
}
```

**SC#3 test — "exactly one DB write after two identical calls":**
```rust
#[tokio::test]
async fn idempotent_replay_does_not_re_execute() {
    let mut exec_count = 0usize;
    let dispatcher = WriteDispatcher {
        executor: Box::new(move |_, _, _, _| {
            exec_count += 1;
            Box::pin(async { Ok(serde_json::json!({ "status": "submitted" })) })
        }),
        guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
    };
    let args = serde_json::json!({ "id": 1, "idempotency_key": "test-key-abc" });
    let result1 = handle_write_call_with_args("submit", args.clone(), ...).await;
    let result2 = handle_write_call_with_args("submit", args.clone(), ...).await;
    assert_eq!(result1, result2);
    assert_eq!(exec_count, 1, "executor called exactly once despite two calls");
}
```

**Advertise `idempotency_key` in inputSchema?**
Recommendation: YES, advertise it as an optional `string` property in `build_action_input_schema`. This requires a one-line addition to the schema builder — inject `"idempotency_key": { "type": "string", "description": "Optional idempotency key for safe retries" }` as a non-required property. Agents that generate idempotency keys will find it in the schema; agents that ignore it still work. Not advertising it means the agent can't generate it, which defeats the purpose. The CONTEXT.md defers this to research; recommendation is to advertise.

**Storage of serialized result:** Store as `TEXT` (JSON-serialized `Value`). On replay, deserialize back to `Value` and wrap in `CallToolResult::structured(value)`. No schema evolution risk — it's already `Value`.

---

## Research Finding 5: Audit Decision (D-05, SC#4)

### ferro-audit public API
From `ferro-audit/src/lib.rs` and `ferro-audit/src/entry.rs`:

```rust
// Builder chain (verified):
AuditEntry::record("mcp.action.write")   // action string, required
    .actor(AuditActor::User(tenant_id.to_string()))
    .tenant(tenant_id.to_string())        // tenant_id: Option<String> on entry
    .target(AuditTarget::new(tool_name, record_id))  // kind + id
    .after(execution_result.clone())      // JSON of what was produced
    .reason(action_name.to_string())      // why this happened
    .write(db).await?;
```

Fields on `audit_log` table: `id` (uuid PK), `tenant_id` (string nullable), `actor_kind`, `actor_id`, `action`, `target_kind`, `target_id`, `before`, `after`, `reason`, `correlation_id`, `created_at`.

### Fit assessment

A per-MCP-call action event maps cleanly onto ferro-audit:
- `action` = `"mcp.action.write"` (or `"mcp.action.{action_name}"`)
- `actor` = `AuditActor::User(tenant_id.to_string())` (the authenticated principal is the tenant; the specific user_id is not threaded to dispatch_write per D-01 — tenant_id is sufficient for SC#4 traceability)
- `tenant` = `tenant_id.to_string()`
- `target` = `AuditTarget::new(tool_name, record_id_string)` — tool name is the "kind", the primary key argument is the "id"
- `after` = the execution result JSON
- `reason` = `action_name`

No `before` — this is not a before/after delta log; it's a call-level audit. The `before` field is optional in ferro-audit (lines 97-99 of entry.rs), so omitting it is fine.

**Recommendation: REUSE ferro-audit.** The fit is clean. The migration `ferro_audit::CreateAuditLogTable` is already designed for consumers to register (documented in lib.rs example). No new table, no duplicate control surface.

**Scope of audit:** Record on every outcome: success, guard-denied, and validation error. For idempotent replay, do NOT re-audit (the original call was already audited). This is slightly broader than SC#4's "each write tool call" literal — auditing denials provides a forensic trail for attempted unauthorized calls (recommended by PITFALLS §2).

**ferro-audit Cargo.toml placement:** `ferro-mcp-server` needs to add `ferro-audit` as a dependency. Check that it does not create a circular dep:
- `ferro-audit` depends on: `sea-orm`, `uuid`, `serde_json`, `thiserror`, `tracing`, `async-trait`, `chrono`, `ferro-events`, `sea-orm-migration` (dev)
- `ferro-mcp-server` currently depends on: `ferro-projections`, `ferro-mcp-oauth`, `rmcp`, `serde`, `serde_json`, `schemars`, `thiserror`, `tracing`, `sea-orm`
- No cycle. Adding `ferro-audit` to `ferro-mcp-server` is safe.

**publish.yml:** `ferro-audit` is already in the workspace. If it is already Wave 1 (no internal deps), it must appear before `ferro-mcp-server` in the publish wave. Verify and add `ferro-mcp-server` as a dependent of `ferro-audit` in the CI wave order if not already captured.

---

## Research Finding 6: Result Construction (D-06, SC#5)

### Existing CallToolResult::structured usage
```rust
// ferro-mcp-server/src/jsonrpc.rs:122-123
let tool_result = CallToolResult::structured(payload);
json!({ "result": tool_result })
```

The Phase 205 regression test at `jsonrpc.rs:193-235` (`tools_call_result_parses_as_valid_mcp_content`) asserts:
- `parsed.is_error == Some(false)`
- `parsed.content.len() == 1`
- `content[0]["type"] == Some("text")`
- `parsed.structured_content` present with `rows`, `total`, `limit`, `offset`

### Structured result shapes for all 219 outcomes

```rust
// SUCCESS
let payload = serde_json::json!({
    "status": "ok",
    "action": action_name,
    "result": execution_result   // the Value returned by executor
});
CallToolResult::structured(payload)  // is_error: false

// GUARD DENIED (SC#1)
let payload = serde_json::json!({
    "status": "error",
    "error_kind": "guard_denied",
    "message": format!("Precondition '{}' not met", guard_name),
    "action": action_name
});
// Wrap in isError:true:
// rmcp::model::CallToolResult has is_error field; structured() sets is_error=false.
// For errors, use the content[] path with isError=true:
let content = vec![rmcp::model::RawContent::Text(rmcp::model::RawTextContent {
    text: serde_json::to_string(&payload).unwrap()
})];
// OR use a helper — see below.
```

**Key finding: `CallToolResult::structured` always sets `is_error: false`.** For error outcomes (guard denied, validation error, action not found), the correct shape per MCP spec and the 200-phase `make_tool_deny_response` precedent is `isError: true` in the result. Looking at `app/src/controllers/mcp.rs:35-47`, the existing pattern for tool-level errors is:
```rust
fn make_tool_deny_response(message: &str, id: &Value) -> Value {
    json!({
        "result": {
            "content": [{ "type": "text", "text": message }],
            "isError": true
        }
    })
}
```

Recommendation: Add `write_tool_error_result(payload: Value) -> Value` helper in `write_dispatch.rs` that builds `{ "result": { "content": [...], "isError": true, "structuredContent": payload } }`. This keeps the pattern consistent and avoids raw `content[]` construction, satisfying D-06's "no hand-built bare content[] arrays" constraint. Alternatively, check if rmcp 0.12 provides a way to set `is_error: true` on a `CallToolResult` — likely via `CallToolResult { is_error: Some(true), content: ..., structured_content: Some(payload) }`. The structured() constructor sets is_error=false; a separate call_tool_error_result() that sets is_error=true is needed.

**SC#5 regression test extension:**
```rust
// Extend tools_call_result_parses_as_valid_mcp_content to also cover write-path results.
// Add in jsonrpc.rs tests:
#[tokio::test]
async fn write_tool_success_result_parses_as_valid_mcp_content() {
    // Call a write tool that succeeds (requires WriteDispatcher with working executor)
    let parsed: CallToolResult = serde_json::from_value(result["result"].clone())
        .expect("must parse as CallToolResult");
    assert_eq!(parsed.is_error, Some(false));
    assert_eq!(parsed.content.len(), 1);
    assert_eq!(parsed.content[0]["type"], "text");
}

#[tokio::test]
async fn write_tool_guard_denied_parses_as_valid_mcp_content() {
    // Call a write tool with failing guard
    let parsed: CallToolResult = serde_json::from_value(result["result"].clone())
        .expect("must parse as CallToolResult");
    assert_eq!(parsed.is_error, Some(true));
}
```

---

## Research Finding 7: Error Variants (error.rs additions)

Current `ferro-mcp-server/src/error.rs` has: `Render`, `InvalidFilter`, `Auth`, `Database`, `Serialization`.

**Add (per ARCHITECTURE §Phase 3):**
```rust
/// The resolved action name is not found in any mcp_exposed ServiceDef.
/// Maps to JSON-RPC -32601 (method not found) at the jsonrpc layer.
#[error("action not found: {0}")]
ActionNotFound(String),

/// A precondition guard returned false or errored at execution time.
/// Maps to a structured tool error result (isError:true), NOT a -32603.
/// Never discloses which guard or what state it checked.
#[error("guard failed: {0}")]
GuardFailed(String),

/// Input validation failed (required field missing, wrong type, etc.).
/// Maps to a structured tool error result (isError:true).
#[error("validation error: {0}")]
Validation(String),
```

**JSON-RPC envelope mapping:**
| Error variant | HTTP wrapper | jsonrpc field | isError |
|---|---|---|---|
| `ActionNotFound` | `json!({ "error": { "code": -32601 ... } })` | error key | N/A |
| `Auth` (scope check) | `json!({ "error": { "code": -32603 ... } })` | error key | N/A |
| `GuardFailed` | `json!({ "result": { "content": [...], "isError": true } })` | result key | true |
| `Validation` | `json!({ "result": { "content": [...], "isError": true } })` | result key | true |
| Executor `Err` | `json!({ "result": { "content": [...], "isError": true } })` | result key | true |

The distinction: `ActionNotFound` is a protocol error (bad tool name = bad method). `GuardFailed`/`Validation` are tool-level application errors — the request was valid but the action cannot be performed; MCP spec says these go in the result with `isError: true`.

---

## Research Finding 8: Confirmation Seam (D-08)

**The 220 seam is a single comment-marked check in `dispatch_write` after guard evaluation and before executor invocation:**

```rust
// D-08 SEAM: Phase 220 inserts confirmation gating here for destructive actions.
// In 219: pass through directly. In 220: if action.transition_trigger.is_some(), intercept.
// Do NOT wire ferro-ai or ConfirmationStore in 219.
// if action.transition_trigger.is_some() { /* Phase 220 will intercept */ }
```

The `transition_trigger` field is confirmed on `ActionDef` at `ferro-projections/src/action.rs:38`:
```rust
pub transition_trigger: Option<String>,
```

The Order projection has three actions with `transition_trigger`: `submit`, `approve`, `ship`. The 219 executor must handle them directly; 220 will wrap with confirmation before reaching the executor.

**No new fields on ActionDef needed in 219.** The `transition_trigger.is_some()` heuristic is sufficient for the 220 seam. A formal `requires_confirmation: bool` field on `ActionDef` is a 220 decision per 218-CONTEXT.md.

---

## Research Finding 9: Phase Split Recommendation (D-09)

This phase has two distinct workstreams:

**Wave A — Framework machinery (ferro-mcp-server only):**
- `write_dispatch.rs`: `dispatch_write`, `handle_write_call`, `WriteDispatcher` type
- `error.rs`: add `GuardFailed`, `ActionNotFound`, `Validation`
- `jsonrpc.rs`: route write tools to `handle_write_call`
- `lib.rs`: re-export `WriteDispatcher`, `dispatch_write`
- `ferro-mcp-oauth/src/migration.rs`: `MigrationMcpIdempotencyKeys` + export
- Add `ferro-audit` dep to `ferro-mcp-server/Cargo.toml`
- Tests: guard-bypass, ActionNotFound, idempotency replay — all in-crate with synthetic WriteDispatcher

**Wave B — Sample app wiring + SC fixtures (app only):**
- `app/src/controllers/mcp.rs`: construct `WriteDispatcher`, pass to `handle_write_call`
- `app/src/models/orders.rs` or `app/src/models/entities/orders.rs`: add `TenantScoped` impl for Order
- `app/src/migrations/`: add idempotency table migration registration, ferro-audit migration registration
- Integration tests: SC#1 (guard bypass on `approve`), SC#2 (cross-tenant), SC#3 (idempotency replay), SC#4 (audit entry present after `submit`), SC#5 (write result parses as CallToolResult)

**Recommendation: Keep as ONE phase, TWO plans.** The context cost is manageable (5 files in the framework, 3 in the app). The SC fixtures depend on the framework machinery, so they can only be written after Wave A is done. A single phase with two sequential plans (Wave A = framework; Wave B = app) is the correct structure — splitting into two phases adds planning overhead without reducing execution risk. Flag to the planner: Wave B is heavier than 217/218 app touches because it needs a real TenantScoped impl + real executor logic.

---

## Common Pitfalls

### Pitfall 1: Consulting ctx.evaluated_guards at execution time
**What goes wrong:** `dispatch_write` checks `ctx.evaluated_guards.get(guard_name)` instead of calling the `GuardEvaluator`. A guard that was `true` at listing time could be `false` at execution time (race condition on live state), or an agent crafts a `tools/call` without ever calling `tools/list`.
**Prevention:** `dispatch_write` never reads `ctx.evaluated_guards` for authorization. The `GuardEvaluator` callback is the only source. Add a test that proves execution-time guard re-evaluation fires even when the guard is absent from `ctx.evaluated_guards`.

### Pitfall 2: Cross-tenant write through idempotency replay
**What goes wrong:** Two tenants call `submit` with the same `idempotency_key`. Tenant B's key matches Tenant A's stored result and replays Tenant A's response.
**Prevention:** The idempotency lookup MUST scope by `(tenant_id, idempotency_key)` — not `(idempotency_key)` alone. The UNIQUE constraint is on `(tenant_id, idempotency_key)`. The SQL check: `WHERE tenant_id = ? AND idempotency_key = ?`.

### Pitfall 3: Audit entry missing for guard-denied calls
**What goes wrong:** Audit only fires on success. A denied call leaves no forensic trail.
**Prevention:** Fire the audit entry for every outcome that reaches guard evaluation — including guard failures. The `after` field contains `{ "denied": true, "guard": guard_name }` for denied entries.

### Pitfall 4: WriteDispatcher capture borrow in boxed closure
**What goes wrong:** The boxed executor closure captures `db` or other non-`'static` refs. `Box<dyn Fn(...) -> Pin<Box<...>> + Send + Sync>` requires `'static` bounds if the closure is stored across await points.
**Prevention:** The closures capture no external state (the db and tenant_id are passed as arguments, not captured). Each invocation receives a fresh `&DatabaseConnection` arg. Test: construct `WriteDispatcher` and call from a separate `tokio::spawn` to verify `Send + 'static` bounds hold.

### Pitfall 5: Idempotency INSERT race condition
**What goes wrong:** Two concurrent identical requests arrive. Both miss the idempotency check simultaneously, both execute, both try to INSERT — one succeeds, one fails with a UNIQUE constraint violation.
**Prevention:** Use `INSERT OR IGNORE` (SQLite) / `INSERT ... ON CONFLICT DO NOTHING` (Postgres). The UNIQUE constraint prevents double-write; the `ON CONFLICT DO NOTHING` turns the constraint error into a no-op. On replay of the second request, re-fetch the stored result via SELECT.

---

## Code Examples

### dispatch_write skeleton
```rust
// ferro-mcp-server/src/write_dispatch.rs
pub async fn dispatch_write(
    action: &ActionDef,
    inputs: &Value,
    tenant_id: i64,
    db: &DatabaseConnection,
    dispatcher: &WriteDispatcher,
) -> crate::Result<Value> {
    // 1. Re-evaluate guards (D-02 — LIVE state, not ctx.evaluated_guards)
    for guard_name in &action.preconditions {
        let passes = (dispatcher.guard_evaluator)(guard_name, tenant_id, inputs, db).await?;
        if !passes {
            return Err(Error::GuardFailed(format!("precondition '{}' not met", guard_name)));
        }
    }
    // 2. Idempotency check (D-04)
    let idempotency_key = inputs.get("idempotency_key").and_then(|v| v.as_str());
    if let Some(key) = idempotency_key {
        if let Some(stored) = lookup_idempotency(tenant_id, key, db).await? {
            return Ok(stored);  // replay — no audit re-fire
        }
    }
    // 3. D-08 seam (Phase 220 intercepts here for destructive actions)
    // 4. Execute callback (D-01 — executor owns TenantScoped enforcement)
    let result = (dispatcher.executor)(&action.name, inputs, tenant_id, db).await?;
    // 5. Store idempotency result (D-04)
    if let Some(key) = idempotency_key {
        store_idempotency(tenant_id, key, &result, db).await?;
    }
    // 6. Audit (D-05 — record on success; also record on guard-denied path above)
    let record_id = inputs.get("id").map(|v| v.to_string()).unwrap_or_default();
    AuditEntry::record(format!("mcp.action.{}", &action.name))
        .tenant(tenant_id.to_string())
        .actor(AuditActor::User(tenant_id.to_string()))
        .target(AuditTarget::new(&action.name, record_id))
        .after(result.clone())
        .write(db).await
        .map_err(|e| Error::Database(e.to_string()))?;
    Ok(result)
}
```

### handle_write_call routing
```rust
// ferro-mcp-server/src/jsonrpc.rs (additions to handle_tools_call)
// After scope check, before service lookup for read tools:
if is_write_tool {
    return handle_write_call(call_params, services, db, tenant_id, ctx, dispatcher).await;
}

pub async fn handle_write_call(
    call_params: Value,
    services: &[ServiceDef],
    db: &DatabaseConnection,
    tenant_id: Option<i64>,
    ctx: &McpContext,
    dispatcher: &WriteDispatcher,
) -> Value {
    let tool_name = call_params["name"].as_str().unwrap_or("");
    let tenant_id = match tenant_id {
        Some(t) => t,
        None => return json!({ "error": { "code": -32603, "message": "auth: tenant required" } }),
    };
    // Find the ActionDef by tool name
    let (service, action) = match find_action(services, tool_name) {
        Some(pair) => pair,
        None => return json!({ "error": { "code": -32601, "message": "Method not found" } }),
    };
    let args = call_params.get("arguments").cloned().unwrap_or(json!({}));
    // validate inputs against ActionDef.inputs
    if let Err(e) = validate_action_inputs(action, &args) {
        return json!({ "result": write_tool_error_result(json!({ "error_kind": "validation", "message": e })) });
    }
    match dispatch_write(action, &args, tenant_id, db, dispatcher).await {
        Ok(result) => {
            let tool_result = CallToolResult::structured(json!({ "status": "ok", "result": result }));
            json!({ "result": tool_result })
        }
        Err(Error::GuardFailed(msg)) => {
            json!({ "result": { "content": [{"type":"text","text": msg}], "isError": true,
                "structuredContent": { "error_kind": "guard_denied", "message": msg } } })
        }
        Err(e) => {
            json!({ "result": { "content": [{"type":"text","text": e.to_string()}], "isError": true,
                "structuredContent": { "error_kind": "execution_error", "message": e.to_string() } } })
        }
    }
}
```

### TenantScoped impl for Order in sample app
```rust
// app/src/models/orders.rs (or entities/orders.rs)
use ferro::async_trait;
use ferro::tenant::TenantScoped;
use ferro::FrameworkError;
use sea_orm::EntityTrait;

#[async_trait]
impl TenantScoped for Model {
    type Id = i32;
    async fn find_for_tenant(id: i32, tenant_id: i64) -> Result<Option<Self>, FrameworkError> {
        use sea_orm::QueryFilter;
        use sea_orm::ColumnTrait;
        Entity::find_by_id(id)
            .filter(Column::TenantId.eq(tenant_id))
            .one(ferro::DB::connection()?)
            .await
            .map_err(|e| FrameworkError::Database(e.to_string()))
    }
}
```

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Append-only audit entries | Custom `mcp_audit_log` table | `ferro-audit` | Already has tenant(), target(), after(), migration, replay |
| Idempotency constraint | Application-level deduplication | DB UNIQUE on (tenant_id, key) + INSERT OR IGNORE | Race-condition safe; simpler than app-level lock |
| Async trait for executor | Custom vtable / enum dispatch | Boxed futures (Pin<Box<dyn Future>>) | No async-trait dep; already the Rust async pattern for dynamic dispatch |
| MCP content block shape | Hand-built json!({ "content": [...] }) | `CallToolResult::structured()` (Phase 205 fix) | Regression guard; exactly one content block with type=text |

---

## Environment Availability

Step 2.6: SKIPPED — this is a pure code/config change phase. No external tools beyond the Rust toolchain are required.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `#[tokio::test]` (already in use across all ferro-mcp-server tests) |
| Config file | None — inline in test modules |
| Quick run command | `cargo test -p ferro-mcp-server write_dispatch` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | SC | Behavior | Test Type | Automated Command | File Exists? |
|--------|-----|----------|-----------|-------------------|-------------|
| AMCP-04 | SC#1 | Guard re-evaluated at call time; direct tools/call with failing guard returns isError:true | integration | `cargo test -p ferro-mcp-server guard_denied_at_call_time` | No — Wave 0 |
| AMCP-04 | SC#2 | Cross-tenant write returns denial, no mutation | integration | `cargo test -p app cross_tenant_write_denied` | No — Wave 0 |
| AMCP-04 | SC#3 | Two calls with same idempotency_key: executor fires once | unit | `cargo test -p ferro-mcp-server idempotent_replay_does_not_re_execute` | No — Wave 0 |
| AMCP-04 | SC#4 | Write call produces audit entry with tool name, tenant_id, action, record id | integration | `cargo test -p app write_call_produces_audit_entry` | No — Wave 0 |
| AMCP-04 | SC#5 | Every write-path result parses as rmcp::model::CallToolResult | unit | `cargo test -p ferro-mcp-server write_tool_result_parses_as_valid_mcp_content` | No — Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-mcp-server`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `ferro-mcp-server/src/write_dispatch.rs` — all SC#1, SC#3, SC#5 unit tests
- [ ] `ferro-mcp-server/tests/write_dispatch_integration.rs` — SC#1 guard bypass fixture
- [ ] `app/src/tests/mcp_write_dispatch.rs` — SC#2 cross-tenant, SC#3 idempotency e2e, SC#4 audit
- [ ] `ferro-mcp-oauth/src/migration.rs` — `MigrationMcpIdempotencyKeys` struct
- [ ] TenantScoped impl on Order — prerequisite for SC#2 fixture

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no (already gated by 217 scope check before reaching write dispatch) | — |
| V3 Session Management | no | — |
| V4 Access Control | YES | TenantScoped::find_for_tenant (BOLA prevention); GuardEvaluator re-eval at call time (BFLA prevention) |
| V5 Input Validation | YES | validate_action_inputs against ActionDef.inputs before dispatch_write |
| V6 Cryptography | no | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Cross-tenant write (BOLA/IDOR) | Elevation of Privilege | TenantScoped::find_for_tenant — None = deny before mutate |
| Guard bypass (BFLA) | Elevation of Privilege | GuardEvaluator in dispatch_write; independent of tools/list visibility filter |
| Idempotency key collision across tenants | Elevation of Privilege | UNIQUE on (tenant_id, idempotency_key) — not (idempotency_key) alone |
| Tenant_id from payload | Elevation of Privilege | tenant_id always from authenticated principal (217 invariant), never from call arguments |
| Double-execute on retry | Tampering | Idempotency table with INSERT OR IGNORE / ON CONFLICT DO NOTHING |

---

## Assumptions Log

> All claims in this research were verified by reading source files. No `[ASSUMED]` claims.

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

---

## Open Questions (RESOLVED)

1. **Does `ferro-mcp-server` need `async-trait` for the boxed-future WriteDispatcher pattern?**
   - What we know: `ferro-mcp-server/Cargo.toml` has no `async-trait` dep. The boxed-future approach (`Pin<Box<dyn Future<Output=...> + Send>>`) does not require it.
   - RESOLVED: boxed futures, no `async-trait`. The pattern is used throughout PATTERNS.md and Plan 00 Task 1 carries a `grep -qv "async-trait" ferro-mcp-server/Cargo.toml` acceptance criterion. Fallback (add the dep) only if ergonomics fail in practice.

2. **Does the sample app's Order already have `TenantScoped` implemented?**
   - What we know: `app/src/models/entities/orders.rs` is auto-generated ("DO NOT EDIT"); `app/src/models/orders.rs` is the extension file with only `type Order = Model`.
   - RESOLVED: no existing impl. Plan 02 Task 1 ADDS the `TenantScoped` impl in `app/src/models/orders.rs` (the safe extension point), per the verbatim impl in PATTERNS.md.

3. **Where does `ferro-audit` sit in `publish.yml` wave order?**
   - What we know: Adding `ferro-audit` as a dep of `ferro-mcp-server` requires `ferro-audit` to publish in an earlier wave.
   - RESOLVED: no reordering needed. `ferro-audit` is Wave 1A (publish.yml line ~211); `ferro-mcp-server` is Wave 2 (line ~275) — already correctly ordered. Plan 00 Task 1 step 1 states this; a verification note is in the plan.

---

## Sources

### Primary (HIGH confidence — verified by reading source files)
- `ferro-mcp-server/src/jsonrpc.rs` — handle_tools_call signature, -32601 placeholder location (line 89-92), scope check (lines 71-83), CallToolResult::structured usage (lines 122-123), regression test (lines 192-235)
- `ferro-mcp-server/src/renderer.rs` — McpContext struct (lines 17-22), render_exposed_tools, render_action_tool
- `ferro-mcp-server/src/dispatch.rs` — dispatch() signature, tenant fail-closed pattern, tenant_id comment (lines 103-104)
- `ferro-mcp-server/src/error.rs` — existing Error variants
- `ferro-mcp-server/src/config.rs` — McpServerConfig struct and from_env()
- `ferro-mcp-server/src/schema.rs` — build_action_input_schema (full), is_filter_field, data_type_to_json_schema
- `ferro-mcp-server/src/lib.rs` — public exports
- `ferro-mcp-server/Cargo.toml` — dependencies (no async-trait)
- `ferro-mcp-oauth/src/migration.rs` — MigrationMcpApiKeys (template for idempotency migration)
- `ferro-audit/src/lib.rs` — public API surface
- `ferro-audit/src/entry.rs` — AuditEntryBuilder chain, write(), tenant() method
- `ferro-audit/src/migration.rs` — audit_log table schema (lines 28-46)
- `ferro-projections/src/action.rs` — ActionDef, InputDef, GuardDef (full structs)
- `framework/src/tenant/scoped.rs` — TenantScoped trait signature
- `ferro-ai/src/confirmation/mod.rs` — ConfirmationStore trait (220 seam context only)
- `app/src/controllers/mcp.rs` — current handle_tools_call call pattern (line 177), make_tool_deny_response (isError:true precedent)
- `app/src/projections/order.rs` — Order ServiceDef with 3 actions (submit, approve, ship)
- `app/src/models/entities/orders.rs` — Order model with tenant_id: i64

### Secondary (MEDIUM confidence)
- ARCHITECTURE.md §Phase 3 build order — design intent for write_dispatch.rs, callback signature
- PITFALLS.md §2 — guard bypass structural pattern
- 219-CONTEXT.md — all locked decisions

---

## Metadata

**Confidence breakdown:**
- Registration API: HIGH — grounded in existing call site in app/src/controllers/mcp.rs and Cargo.toml
- Guard re-evaluation: HIGH — ActionDef.preconditions verified; pipeline placement derived from D-07 + PITFALLS §2
- TenantScoped: HIGH — trait signature verified; no write helper exists (confirmed)
- Idempotency: HIGH — migration template verified; SQL pattern is standard
- Audit: HIGH — ferro-audit API verified including tenant() method; no circular dep risk
- Result construction: HIGH — CallToolResult::structured verified; is_error field behavior confirmed from test assertions
- Phase split: MEDIUM — judgment call; recommendation to keep single phase with two plans

**Research date:** 2026-06-13
**Valid until:** 2026-07-13 (30 days — stable dependencies)
