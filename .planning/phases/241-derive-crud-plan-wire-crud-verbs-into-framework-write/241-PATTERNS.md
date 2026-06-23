# Phase 241: `derive_crud_plan` + wire CRUD verbs into `framework::write` — Pattern Map

**Mapped:** 2026-06-23
**Files analyzed:** 5 files to modify (no new files)
**Analogs found:** 5 / 5

---

## File Classification

| Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---------------|------|-----------|----------------|---------------|
| `ferro-projections/src/executor.rs` | model / derivation | transform (pure, no I/O) | Same file — `TransitionPlan` + `derive_transition_plan` (lines 22–115) | **exact** |
| `ferro-projections/src/lib.rs` | config / re-export | — | Same file — line 17 re-export of `derive_transition_plan` / `TransitionPlan` | **exact** |
| `framework/src/write/mod.rs` | service / kernel | CRUD + request-response | Same file — `dispatch_write` pipeline (lines 313–436) + `setup_db` test helper (lines 451–490) | **exact** |
| `ferro-mcp-server/src/write_dispatch.rs` | controller / framing | request-response | Same file — NTI CRUD block (lines 155–180) + `handle_request_confirm` / `handle_confirm` (lines 300–566) + confirmation tests (lines 827–1319) | **exact** |
| `ferro-mcp-server/src/renderer.rs` | controller / rendering | request-response | Same file — transition confirm-tool synthesis block (lines 115–155) + `render_request_confirm_tool` / `render_confirm_tool` (lines 325–427) | **exact** |

---

## Pattern Assignments

### `ferro-projections/src/executor.rs` — add `CrudVerb`, `TenantColumn`, `CrudPlan`, `derive_crud_plan`

**Analog:** Same file — `TransitionPlan` struct + `derive_transition_plan` function (lines 22–115)

**Derive macro pattern** (lines 22–23 — copy verbatim for `CrudPlan` / `CrudVerb` / `TenantColumn`):
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TransitionPlan {
```

**Serde skip pattern for optional fields** (lines 33–37):
```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub guard: Option<String>,
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub effects: Vec<String>,
```

**Pure-derivation function signature** (lines 56–59):
```rust
pub fn derive_transition_plan(
    svc: &crate::ServiceDef,
    action_name: &str,
) -> Result<TransitionPlan, crate::Error> {
```

**Error creation pattern** (lines 61–64, 78–82):
```rust
.ok_or_else(|| crate::Error::Validation(format!("no action '{action_name}'")))?;
// ...
return Err(crate::Error::UndeclaredTransition {
    action: action_name.to_string(),
    event: event.to_string(),
});
```

**Test fixture pattern** (lines 127–152 — copy `order_service()` helper and extend for CRUD):
```rust
fn order_service() -> ServiceDef {
    let machine = StateMachine::new("order_lifecycle")
        .initial("draft")
        .transition(Transition::new("draft", "submit", "submitted").actions(vec!["log_submit"]))
        // ...
    ServiceDef::new("order")
        .state_machine(machine)
        .action(ActionDef::new("submit").transition_trigger("submit").effect("notify"))
        // ...
}
```

**Table test pattern** (lines 154–168 — copy exact assertion style):
```rust
#[test]
fn derive_transition_plan() {
    let svc = order_service();
    let plan = super::derive_transition_plan(&svc, "submit").unwrap();
    assert_eq!(plan.action, "submit");
    assert_eq!(plan.event, "submit");
    assert_eq!(plan.to_state, "submitted");
    assert_eq!(plan.from_states, vec!["draft".to_string()]);
    assert_eq!(plan.guard, None);
    assert_eq!(plan.effects, vec!["log_submit".to_string(), "notify".to_string()]);
}
```

**Serde round-trip test pattern** (lines 245–251 — copy for `crud_plan_serde_round_trip`):
```rust
#[test]
fn transition_plan_serde_round_trip() {
    let svc = order_service();
    let plan = super::derive_transition_plan(&svc, "submit").unwrap();
    let json = serde_json::to_string(&plan).unwrap();
    let back: TransitionPlan = serde_json::from_str(&json).unwrap();
    assert_eq!(plan, back);
}
```

**New error variants** — add to `ferro-projections/src/error.rs` following existing style (lines 1–33):
- `VerbNotEnabled(String)` — CRUD verb requested but `.creatable`/`.updatable`/`.deletable` is false
- No new variant needed for row-not-found in the projections crate (that is a `WriteError` in framework)

**Key constraint:** `CrudPlan` must store `serde_json::Value` for column values, NOT `sea_orm::Value`. `ferro-projections` has no sea-orm dependency (`ferro-projections/Cargo.toml` verified). The executor in `framework` coerces `serde_json::Value` → `sea_orm::Value`.

---

### `ferro-projections/src/lib.rs` — add re-exports

**Analog:** Line 17 — current transition plan re-export:
```rust
pub use executor::{derive_transition_plan, TransitionPlan};
```

**Pattern to extend** (line 17):
```rust
// Before:
pub use executor::{derive_transition_plan, TransitionPlan};
// After:
pub use executor::{derive_crud_plan, derive_transition_plan, CrudPlan, CrudVerb, TransitionPlan};
// TenantColumn is also exported if it is made pub in executor.rs
```

---

### `framework/src/write/mod.rs` — add `execute_crud_plan`, extend `dispatch_write`

**Analog:** Same file — the full `dispatch_write` pipeline (lines 313–436) and idempotency helpers (lines 195–279)

**Imports block** (lines 15–21 — add `CrudPlan` from `ferro_projections`):
```rust
use ferro_audit::{AuditActor, AuditEntry, AuditTarget};
use ferro_projections::ActionDef;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;
```

**`WriteError` enum extension** (lines 30–54 — add new variants at end):
```rust
#[derive(Error, Debug)]
pub enum WriteError {
    #[error("database error: {0}")]
    Database(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("guard failed: {0}")]
    GuardFailed(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("action not found: {0}")]
    ActionNotFound(String),
    #[cfg(feature = "confirmation")]
    #[error("confirmation required for action: {0}")]
    ConfirmationRequired(String),
    // New in Phase 241:
    // #[error("crud verb not enabled for service: {0}")]
    // CrudVerbNotEnabled(String),
    // #[error("record not found or already deleted")]
    // RecordNotFound,
}
```

**`dispatch_write` extension** — new `crud_plan: Option<&CrudPlan>` parameter (last positional, after `is_confirmed`). Current signature (lines 313–322):
```rust
#[allow(clippy::too_many_arguments)]
pub async fn dispatch_write(
    action: &ActionDef,
    inputs: &Value,
    tenant_id: i64,
    db: &DatabaseConnection,
    dispatcher: &WriteDispatcher,
    transition_guard: Option<&str>,
    channel: &str,
    #[cfg(feature = "confirmation")] is_confirmed: bool,
) -> WriteResult<Value> {
```

**Confirmation seam to extend** (lines 378–383):
```rust
#[cfg(feature = "confirmation")]
if action.transition_trigger.is_some() && !is_confirmed {
    return Err(WriteError::ConfirmationRequired(action.name.clone()));
}
#[cfg(not(feature = "confirmation"))]
let _ = &action.transition_trigger;
```
Extend to:
```rust
#[cfg(feature = "confirmation")]
{
    let is_destructive = action.transition_trigger.is_some()
        || matches!(crud_plan, Some(CrudPlan::Delete { .. }));
    if is_destructive && !is_confirmed {
        return Err(WriteError::ConfirmationRequired(action.name.clone()));
    }
}
```

**Executor call to branch** (line 388):
```rust
// Current:
let result = (dispatcher.executor)(&action.name, inputs, tenant_id, db).await?;
// Extended:
let result = if let Some(plan) = crud_plan {
    execute_crud_plan(plan, tenant_id, db).await?
} else {
    (dispatcher.executor)(&action.name, inputs, tenant_id, db).await?
};
```

**Audit label to extend** (line 414):
```rust
// Current:
AuditEntry::record(format!("{channel}.action.{}", &action.name))
// For CRUD verbs use distinct prefix (D-08 / Claude's discretion):
// format!("{channel}.crud.{}", &action.name)
// Implement by checking crud_plan.is_some() in the audit step.
```

**Override hook lookup** (lines 431–433) — reused unchanged:
```rust
if let Some(hook) = dispatcher.overrides.get(&action.name) {
    (hook)(&action.name, inputs, tenant_id, db, &result).await?;
}
```

**Backend-parameterized SQL helper** — copy from `dispatch.rs:28-33` (same file replicates it; put it in `write/mod.rs` as a private function):
```rust
fn placeholder(backend: DatabaseBackend, index: usize) -> String {
    match backend {
        DatabaseBackend::Postgres => format!("${index}"),
        _ => "?".to_string(),
    }
}
```

**`json_to_sea_value` helper** — copy from `dispatch.rs:36-50`:
```rust
fn json_to_sea_value(val: &serde_json::Value) -> sea_orm::Value {
    match val {
        serde_json::Value::Null => sea_orm::Value::String(None),
        serde_json::Value::Bool(b) => sea_orm::Value::Bool(Some(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                sea_orm::Value::BigInt(Some(i))
            } else {
                sea_orm::Value::Double(n.as_f64())
            }
        }
        serde_json::Value::String(s) => sea_orm::Value::String(Some(Box::new(s.clone()))),
        other => sea_orm::Value::String(Some(Box::new(other.to_string()))),
    }
}
```

**`setup_db` test helper** (lines 451–490 — copy verbatim, extend with the orders table):
```rust
async fn setup_db() -> sea_orm::DatabaseConnection {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("in-memory SQLite connect failed");
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "CREATE TABLE IF NOT EXISTS mcp_idempotency_keys (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tenant_id INTEGER NOT NULL,
            idempotency_key TEXT NOT NULL,
            result TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE (tenant_id, idempotency_key)
        )".to_string(),
    )).await.expect("create mcp_idempotency_keys table");
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "CREATE TABLE IF NOT EXISTS audit_log (
            id TEXT PRIMARY KEY NOT NULL,
            tenant_id TEXT,
            actor_kind TEXT NOT NULL,
            actor_id TEXT,
            action TEXT NOT NULL,
            target_kind TEXT,
            target_id TEXT,
            before TEXT,
            after TEXT,
            reason TEXT,
            correlation_id TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )".to_string(),
    )).await.expect("create audit_log table");
    // Phase 241 addition — target service table for CRUD executor tests:
    // db.execute(Statement::from_string(DatabaseBackend::Sqlite,
    //     "CREATE TABLE IF NOT EXISTS orders (
    //         id INTEGER PRIMARY KEY AUTOINCREMENT,
    //         status TEXT NOT NULL,
    //         amount TEXT,
    //         created_at TEXT DEFAULT (datetime('now')),
    //         deleted_at TEXT
    //     )".to_string(),
    // )).await.expect("create orders table");
    db
}
```

**Existing dispatch call sites that must add `, None`** (all callers of `dispatch_write`):
- `write_dispatch.rs:220-230` (bare transition call, `crud_plan: None`)
- `write_dispatch.rs:532-541` (confirm handler call, `crud_plan: None`)
- All tests in `write/mod.rs` (lines 521–532, 555–566, and throughout the test module) must append `, None`

---

### `ferro-mcp-server/src/write_dispatch.rs` — replace NTI block, extend confirm handlers

**Analog:** Same file — NTI block (lines 155–180), `handle_request_confirm` (lines 300–406), `handle_confirm` (lines 415–566), confirmation tests (lines 827–1319)

**NTI block to replace** (lines 155–180 — the entire `for prefix in [...]` block):
```rust
// Phase 240: CRUD verb tools are listed but not yet executable (Phase 241 wires execution).
let crud_verb_opted_in = |s: &ServiceDef, prefix: &str| match prefix {
    "create_" => s.creatable,
    "update_" => s.updatable,
    "delete_" => s.deletable,
    _ => false,
};
for prefix in ["create_", "update_", "delete_"] {
    if let Some(svc_name) = tool_name.strip_prefix(prefix) {
        if services
            .iter()
            .any(|s| s.mcp_exposed && s.name == svc_name && crud_verb_opted_in(s, prefix))
        {
            let tool_result = CallToolResult::structured(serde_json::json!({
                "error_kind": "not_yet_implemented",
                "message": format!("{} execution is not yet wired (Phase 241)", tool_name)
            }));
            return json!({ "result": tool_result });
        }
    }
}
```
Replace with: service lookup + `derive_crud_plan` call + `dispatch_write(..., Some(&plan))` + route result through `CallToolResult::structured(payload)` (line 239 pattern).

**Success-path result envelope** (lines 233–240 — copy pattern for CRUD success):
```rust
Ok(result) => {
    let payload = json!({
        "status": "ok",
        "action": action.name,
        "result": result
    });
    let tool_result = CallToolResult::structured(payload);
    json!({ "result": tool_result })
}
```

**Error-path mapping** (lines 242–284 — copy the match arms; add `RecordNotFound` variant mapping alongside `ActionNotFound`):
```rust
Err(WriteError::GuardFailed(ref msg)) => { /* audit denial + error envelope */ }
#[cfg(feature = "confirmation")]
Err(WriteError::ConfirmationRequired(ref action_name)) => {
    json!({ "result": write_tool_error_result(json!({
        "error_kind": "confirmation_required",
        "message": format!("use request_confirm_{action_name} first"),
        "request_tool": format!("request_confirm_{action_name}")
    })) })
}
Err(ref e @ WriteError::Validation(_)) | Err(ref e @ WriteError::ActionNotFound(_)) => { /* pass-through */ }
Err(_) => { /* redacted */ }
```

**`handle_request_confirm` signature** (lines 300–310 — copy for CRUD delete confirm):
```rust
#[cfg(feature = "confirmation")]
#[allow(clippy::too_many_arguments)]
pub async fn handle_request_confirm(
    call_params: Value,
    services: &[ServiceDef],
    db: &DatabaseConnection,
    tenant_id: Option<i64>,
    _ctx: &crate::McpContext,
    dispatcher: &WriteDispatcher,
    store: &dyn ferro_ai::ConfirmationStore,
    action_name: &str,
    ttl_secs: u64,
) -> Value {
```
For CRUD delete: the `find_action` call returns `None` (CRUD verbs are not `ActionDef`s). Branch before `find_action` using `action_name.starts_with("delete_")` to locate `svc` by name-stripping instead. The `action_name` is `"delete_<svc>"` — strip `"delete_"` prefix to find the `ServiceDef`.

**Token binding payload** (lines 368–376 — reuse unchanged for delete):
```rust
let binding_payload = json!({
    "_binding": {
        "tenant_id": tid,
        "action_name": action_name,  // = "delete_order"
        "record_id": record_id
    },
    "inputs": args
});
```

**`handle_confirm` binding verification** (lines 468–491 — reused unchanged):
```rust
if binding["tenant_id"].as_i64() != Some(tid) { /* mismatch */ }
if binding["action_name"].as_str() != Some(action_name) { /* mismatch */ }
let call_record_id = args.get("id");
let stored_record_id = binding.get("record_id");
if call_record_id != stored_record_id { /* mismatch */ }
```

**Confirmation test `setup_db`** (lines 837–876 — copy verbatim, extend with orders table):
```rust
async fn setup_db() -> sea_orm::DatabaseConnection { /* identical to write_dispatch tests setup_db */ }
```

**`allow_dispatcher` test helper** (lines 906–915):
```rust
fn allow_dispatcher(exec_count: Arc<AtomicUsize>) -> WriteDispatcher {
    WriteDispatcher {
        guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
        executor: Box::new(move |_, _, _, _| {
            exec_count.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Ok(json!({ "status": "approved" })) })
        }),
        overrides: std::collections::HashMap::new(),
    }
}
```

**Two-step flow test pattern** (lines 971–1000 — copy structure for `delete_two_step_flow_soft_deletes`):
```rust
#[tokio::test]
async fn sc2_two_step_flow_executes_once() {
    let db = setup_db().await;
    let exec_count = Arc::new(AtomicUsize::new(0));
    let dispatcher = allow_dispatcher(exec_count.clone());
    let store = InMemoryConfirmationStore::new();
    let services = vec![order_service()];
    let ctx = crate::McpContext::default();

    // Step 1: request_confirm
    let req_response = handle_request_confirm(
        json!({ "name": "request_confirm_submit", "arguments": { "id": 1 } }),
        &services, &db, Some(1), &ctx, &dispatcher, &store, "submit", 300,
    ).await;
    let token = req_response["result"]["structuredContent"]["confirmation_token"]
        .as_str().expect("token must be present");

    // Step 2: first confirm — must execute
    let confirm_response = handle_confirm(
        json!({ "name": "confirm_submit", "arguments": { "confirmation_token": token, "id": 1 } }),
        // ...
    ).await;
}
```

---

### `ferro-mcp-server/src/renderer.rs` — synthesize `request_confirm_delete_<svc>` / `confirm_delete_<svc>`

**Analog:** Same file — transition confirm-tool synthesis block (lines 115–155) and `render_request_confirm_tool` / `render_confirm_tool` (lines 325–427)

**Confirm synthesis loop structure** (lines 118–155 — the `#[cfg(feature = "confirmation")]` block):
```rust
#[cfg(feature = "confirmation")]
{
    let destructive: Vec<(String, String, ferro_projections::ActionDef)> = tagged
        .iter()
        .filter_map(|(svc_name, tool)| {
            services.iter()
                .filter(|s| s.mcp_exposed && s.name == *svc_name)
                .flat_map(|s| s.actions.iter())
                .find(|a| {
                    let disambiguated = format!("{}_on_{}", a.name, svc_name);
                    tool.name.as_ref() == a.name || tool.name.as_ref() == disambiguated
                })
                .filter(|a| a.transition_trigger.is_some())
                .map(|a| (svc_name.clone(), tool.name.to_string(), a.clone()))
        })
        .collect();

    for (svc_name, base_name, action) in destructive {
        if let Some(req_tool) =
            render_request_confirm_tool(&base_name, &action, services, &svc_name, ctx)?
        {
            tagged.push((svc_name.clone(), req_tool));
        }
        if let Some(cfm_tool) =
            render_confirm_tool(&base_name, &action, services, &svc_name, ctx)?
        {
            tagged.push((svc_name.clone(), cfm_tool));
        }
    }
}
```
Extend this block: after the transition `destructive` loop, add a second loop over services with `.deletable == true`, synthesizing `request_confirm_delete_<svc>` (using `render_delete_tool`'s schema, `destructiveHint=false`) and `confirm_delete_<svc>` (token + id schema, `destructiveHint=true`).

**`render_request_confirm_tool` pattern** (lines 325–372 — copy for delete; uses `build_action_input_schema`; for CRUD use `build_delete_input_schema` instead):
```rust
#[cfg(feature = "confirmation")]
fn render_request_confirm_tool(
    base_name: &str,
    action: &ferro_projections::ActionDef,
    services: &[ferro_projections::ServiceDef],
    service_name: &str,
    ctx: &McpContext,
) -> std::result::Result<Option<Tool>, ProjError> {
    // guard-visibility filter
    // find owning service
    let name = format!("request_confirm_{base_name}");
    let description = format!("Request confirmation to: {}", action.description...);
    let schema_value = crate::schema::build_action_input_schema(action, service)...;
    let annotations = ToolAnnotations::new().read_only(false).destructive(false);
    Ok(Some(Tool::new(name, description, Arc::new(schema_map)).annotate(annotations)))
}
```

**`render_confirm_tool` minimal schema** (lines 374–427 — copy verbatim for delete confirm tool):
```rust
let mut schema = serde_json::Map::new();
schema.insert("type".to_string(), serde_json::json!("object"));
let mut props = serde_json::Map::new();
props.insert("confirmation_token".to_string(),
    serde_json::json!({ "type": "string", "description": "Token returned by request_confirm" }));
props.insert("id".to_string(),
    serde_json::json!({ "type": "integer", "description": "Record id (must match the one used in request_confirm)" }));
schema.insert("properties".to_string(), serde_json::Value::Object(props));
schema.insert("required".to_string(), serde_json::json!(["confirmation_token", "id"]));
let annotations = ToolAnnotations::new().read_only(false).destructive(true);
```

**Tool name pattern** — CRUD delete confirm tools use `"delete_<svc>"` as `base_name`, producing `"request_confirm_delete_order"` and `"confirm_delete_order"`. The `strip_prefix("request_confirm_")` routing in `handle_write_call` strips to `"delete_order"` → passed as `action_name` to `handle_request_confirm`.

---

## Shared Patterns

### sea-orm raw SQL (read + write)
**Source:** `framework/src/write/mod.rs:200-220` (read) and `framework/src/write/mod.rs:248-277` (write)
**Apply to:** `execute_crud_plan` function in `framework/src/write/mod.rs`
```rust
let backend = db.get_database_backend();
let (sql, values) = match backend {
    DatabaseBackend::Postgres => ("... $1 $2 ...", vec![
        sea_orm::Value::BigInt(Some(tenant_id)),
        sea_orm::Value::String(Some(Box::new(key.to_string()))),
    ]),
    _ => ("... ? ? ...", vec![
        sea_orm::Value::BigInt(Some(tenant_id)),
        sea_orm::Value::String(Some(Box::new(key.to_string()))),
    ]),
};
let stmt = Statement::from_sql_and_values(backend, &sql, values);
db.execute(stmt).await.map_err(|e| WriteError::Database(e.to_string()))?;
```

### Backend-parameterized placeholder
**Source:** `ferro-mcp-server/src/dispatch.rs:28-33`
**Apply to:** `execute_crud_plan` in `framework/src/write/mod.rs`
```rust
fn placeholder(backend: DatabaseBackend, index: usize) -> String {
    match backend {
        DatabaseBackend::Postgres => format!("${index}"),
        _ => "?".to_string(),
    }
}
```

### `json_to_sea_value` coercion
**Source:** `ferro-mcp-server/src/dispatch.rs:36-50`
**Apply to:** `execute_crud_plan` in `framework/src/write/mod.rs` (copy as private function)
```rust
fn json_to_sea_value(val: &serde_json::Value) -> sea_orm::Value { ... }
```

### `CallToolResult::structured` envelope
**Source:** `ferro-mcp-server/src/write_dispatch.rs:233-240` (success) and `ferro-mcp-server/src/jsonrpc.rs:144,215`
**Apply to:** CRUD result path in `write_dispatch.rs` NTI replacement
```rust
let payload = json!({ "status": "ok", "action": tool_name, "result": result });
let tool_result = CallToolResult::structured(payload);
json!({ "result": tool_result })
```

### CSPRNG confirmation token
**Source:** `ferro-mcp-server/src/write_dispatch.rs:81-92`
**Apply to:** `handle_request_confirm` for CRUD delete — reused unchanged, no copy needed
```rust
#[cfg(feature = "confirmation")]
fn generate_confirmation_token() -> String {
    use rand::Rng;
    const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::thread_rng();
    let random: String = (0..43).map(|_| { let idx = rng.gen_range(0..62usize); BASE62[idx] as char }).collect();
    format!("cfm_{random}")
}
```

### Service resolver accessors
**Source:** `ferro-projections/src/service.rs:215-276`
**Apply to:** `derive_crud_plan` function in `ferro-projections/src/executor.rs`
- `svc.resolved_table()` — backing table name
- `svc.resolved_soft_delete_column()` — soft-delete column name
- `svc.is_server_injected_field(field)` — exclude Identifier, CreatedAt, tenant column
- `svc.is_write_excluded_field(field, exclude_sm_status)` — full write-exclusion gate

---

## No Analog Found

None. All five modified files have direct analogs within themselves (same-file patterns).

---

## Key Anti-Patterns (from RESEARCH.md — do not repeat)

| Anti-Pattern | Why Wrong |
|---|---|
| Store `sea_orm::Value` in `CrudPlan` | `sea_orm::Value` lacks `JsonSchema` + `Serialize` — breaks `ferro-projections` boundary |
| Fabricate `ActionDef` with `transition_trigger` for CRUD | Confirmation seam fires on wrong condition; SC#4 violated |
| Add a second CRUD dispatcher | Duplicate write-control surface; `feedback_no_duplicate_control_surface` violated |
| Omit `deleted_at IS NULL` from `CrudPlan::Update` | Soft-deleted rows become patchable — security gap |
| Call `find_action` for `"delete_order"` in confirm handlers | `find_action` searches `svc.actions`; CRUD verbs are not there → `-32601` |
| Inject `tenant_id` in Phase 241 | D-09 explicitly defers; 241 leaves `tenant_column: None` in all plans |

---

## Metadata

**Analog search scope:** `ferro-projections/src/`, `framework/src/write/`, `ferro-mcp-server/src/`
**Files read:** 9 source files (`executor.rs`, `error.rs`, `lib.rs`, `service.rs`, `write/mod.rs`, `write_dispatch.rs`, `renderer.rs`, `dispatch.rs`, `jsonrpc.rs` line ranges)
**Pattern extraction date:** 2026-06-23
