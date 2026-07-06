# Phase 219: Write Dispatch - Pattern Map

**Mapped:** 2026-06-13
**Files analyzed:** 9 new/modified files
**Analogs found:** 9 / 9

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-mcp-server/src/write_dispatch.rs` | service | request-response | `ferro-mcp-server/src/dispatch.rs` | exact (same crate, same dispatch role, tenant discipline) |
| `ferro-mcp-server/src/jsonrpc.rs` | middleware/router | request-response | `ferro-mcp-server/src/jsonrpc.rs` (existing, modify) | self (MODIFY) |
| `ferro-mcp-server/src/error.rs` | utility | N/A | `ferro-mcp-server/src/error.rs` (existing, modify) | self (MODIFY) |
| `ferro-mcp-server/src/config.rs` | config | N/A | `ferro-mcp-server/src/config.rs` (READ ONLY — stays identity-only) | self (READ) |
| `ferro-mcp-oauth/src/migration.rs` | migration | batch | `ferro-mcp-oauth/src/migration.rs` `MigrationMcpApiKeys` (existing, add struct) | self (MODIFY, same file) |
| `ferro-mcp-server/Cargo.toml` | config | N/A | `ferro-mcp-server/Cargo.toml` (existing, modify) | self (MODIFY) |
| `app/src/controllers/mcp.rs` | controller | request-response | `app/src/controllers/mcp.rs` (existing, modify) | self (MODIFY) |
| `app/src/models/orders.rs` | model | CRUD | `framework/src/tenant/scoped.rs` trait + `app/src/tests/mcp_tenant_isolation.rs` usage | role-match |
| `app/src/tests/mcp_write_dispatch.rs` | test | event-driven | `app/src/tests/mcp_tenant_isolation.rs` | exact (same test harness) |

---

## Pattern Assignments

### `ferro-mcp-server/src/write_dispatch.rs` (service, request-response) — CREATE

**Analog:** `ferro-mcp-server/src/dispatch.rs`

**Imports pattern** (`dispatch.rs` lines 1-3):
```rust
use crate::schema::is_filter_field;
use ferro_projections::{FieldMeaning, ServiceDef};
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
```
For `write_dispatch.rs`, replace with:
```rust
use ferro_audit::{AuditActor, AuditEntry, AuditTarget};
use ferro_projections::{ActionDef, ServiceDef};
use sea_orm::DatabaseConnection;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
```

**Tenant fail-closed pattern** (`dispatch.rs` lines 153-167 — the load-bearing security pattern to mirror):
```rust
// In dispatch():
if let Some(ref col) = service.tenant_column {
    match tenant_id {
        Some(tid) => {
            where_clauses.push(format!("\"{}\" = {}", col, placeholder(backend, idx)));
            values.push(sea_orm::Value::BigInt(Some(tid)));
            idx += 1;
        }
        None => {
            // Fail-closed (D-06): tenant-scoped projection + no tenant context → deny.
            return Err(crate::Error::InvalidFilter(
                "tenant context required but not present".to_string(),
            ));
        }
    }
}
```
For `dispatch_write`: `tenant_id` parameter is `i64` (never `Option<i64>`) — the `handle_write_call` function unwraps it before calling `dispatch_write`, consistent with the invariant that writes always require an authenticated tenant. The fail-closed check happens in `handle_write_call`, not inside `dispatch_write` itself.

**Boxed-future registration API** (RESEARCH.md §Registration API — no async-trait dep):
```rust
// ferro-mcp-server/src/write_dispatch.rs
pub type ExecutorFn = Box<
    dyn Fn(
            &str,                       // action_name
            &Value,                     // validated inputs
            i64,                        // tenant_id (from auth, never from payload)
            &DatabaseConnection,
        ) -> Pin<Box<dyn Future<Output = crate::Result<Value>> + Send>>
        + Send
        + Sync,
>;

pub type GuardEvaluatorFn = Box<
    dyn Fn(
            &str,                       // guard_name
            i64,                        // tenant_id
            &Value,                     // validated inputs (for record-scoped guards)
            &DatabaseConnection,
        ) -> Pin<Box<dyn Future<Output = crate::Result<bool>> + Send>>
        + Send
        + Sync,
>;

pub struct WriteDispatcher {
    pub executor: ExecutorFn,
    pub guard_evaluator: GuardEvaluatorFn,
}
```
Note: `ferro-mcp-server/Cargo.toml` has no `async-trait` dep (verified). This boxed-future pattern avoids adding that dep.

**Guard re-evaluation core pattern** (D-02 — from RESEARCH.md §Guard Re-evaluation, `ferro-projections/src/action.rs:35` confirms `preconditions: Vec<String>`):
```rust
// dispatch_write — guard loop (live state, NEVER ctx.evaluated_guards)
for guard_name in &action.preconditions {
    let passes = (dispatcher.guard_evaluator)(guard_name, tenant_id, inputs, db).await
        .map_err(|e| crate::Error::GuardFailed(format!("{guard_name}: {e}")))?;
    if !passes {
        return Err(crate::Error::GuardFailed(
            format!("precondition '{}' not met", guard_name),
        ));
    }
}
```

**Idempotency check pattern** (D-04 — from RESEARCH.md §Idempotency):
```rust
let idempotency_key = inputs.get("idempotency_key").and_then(|v| v.as_str());
if let Some(key) = idempotency_key {
    if let Some(stored_result) = lookup_idempotency(tenant_id, key, db).await? {
        return Ok(stored_result);   // replay — skip execution and audit
    }
}
// ... execute ...
if let Some(key) = idempotency_key {
    // INSERT OR IGNORE / ON CONFLICT DO NOTHING — race-condition safe
    store_idempotency(tenant_id, key, &result, db).await?;
}
```

**Audit pattern** (`ferro-audit/src/entry.rs` lines 45-56 builder, `ferro-audit/src/lib.rs` re-exports):
```rust
// After successful execution (also fire on guard-denied with denied=true payload)
AuditEntry::record(format!("mcp.action.{}", &action.name))
    .tenant(tenant_id.to_string())
    .actor(AuditActor::User(tenant_id.to_string()))
    .target(AuditTarget::new(&action.name, record_id_string))
    .after(result.clone())
    .reason(action_name)
    .write(db)
    .await
    .map_err(|e| crate::Error::Database(e.to_string()))?;
```
`before()` is optional (confirmed: `entry.rs` lines 91-99, field is `Option<JsonValue>`). Omit it for MCP write events — this is a call-level audit, not a before/after delta.

**D-08 confirmation seam** (pass-through comment only, no code):
```rust
// D-08 SEAM: Phase 220 inserts confirmation gating here for destructive actions.
// In 219: pass through directly.
// if action.transition_trigger.is_some() { /* Phase 220 intercepts */ }
```
`transition_trigger: Option<String>` confirmed on `ActionDef` (RESEARCH.md §Confirmation Seam).

---

### `ferro-mcp-server/src/jsonrpc.rs` (router, request-response) — MODIFY

**Analog:** Same file; read path in `handle_tools_call` (lines 54-133).

**Scope gate pattern to keep in front** (`jsonrpc.rs` lines 68-82 — MUST remain before write routing):
```rust
let is_write_tool = !tool_name.starts_with("list_");
let key_scope = ctx.scope.as_deref().unwrap_or("read_write");
if is_write_tool && key_scope == "read" {
    return json!({
        "error": {
            "code": -32603,
            "message": crate::Error::Auth(
                "scope insufficient: read key cannot call write tools".to_string()
            ).to_string()
        }
    });
}
```

**-32601 placeholder to replace** (`jsonrpc.rs` lines 84-92 — this block currently serves write tools with -32601):
```rust
let service = match services
    .iter()
    .find(|s| s.name == service_name && s.mcp_exposed)
{
    Some(s) => s,
    None => {
        return json!({ "error": { "code": -32601, "message": "Method not found" } });
    }
};
```
The `service_name = tool_name.strip_prefix("list_").unwrap_or(tool_name)` at line 66 makes a write tool name like `"submit_order"` fail the `find()` → `-32601`. Phase 219 intercepts `is_write_tool` before that lookup.

**Routing insertion point** (after scope gate, before service lookup — lines 83-84):
```rust
// Phase 219: route write-tool calls to the write dispatch path
if is_write_tool {
    return handle_write_call(call_params, services, db, tenant_id, ctx, dispatcher).await;
}
```
This requires adding `dispatcher: &WriteDispatcher` to `handle_tools_call`'s signature and to the call site in `app/src/controllers/mcp.rs` line 177.

**`CallToolResult::structured` pattern for success** (`jsonrpc.rs` lines 115-123):
```rust
Ok(result) => {
    let payload = serde_json::json!({
        "rows": result.rows,
        "total": result.total,
        "limit": result.limit,
        "offset": result.offset
    });
    let tool_result = CallToolResult::structured(payload);
    json!({ "result": tool_result })
}
```
`CallToolResult::structured()` always sets `is_error: Some(false)`. For write success, wrap the executor result analogously:
```rust
let payload = serde_json::json!({ "status": "ok", "action": action_name, "result": execution_result });
let tool_result = CallToolResult::structured(payload);
json!({ "result": tool_result })
```

**isError error result pattern** (`app/src/controllers/mcp.rs` lines 35-47 — `make_tool_deny_response`):
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
For write-path errors (GuardFailed, validation, executor error), the same `result`-keyed envelope with `isError: true` applies. A `write_tool_error_result(payload: Value) -> Value` helper in `write_dispatch.rs` that builds `{ "result": { "content": [{"type":"text","text": msg}], "isError": true, "structuredContent": payload } }` avoids hand-built arrays and satisfies D-06's "no bare content[] construction" constraint. `CallToolResult::structured()` cannot be used for errors because it hard-codes `is_error: Some(false)`.

**Existing regression test to extend** (`jsonrpc.rs` lines 193-235 — `tools_call_result_parses_as_valid_mcp_content`):
```rust
let parsed: CallToolResult = serde_json::from_value(response["result"].clone())
    .expect("result must parse as CallToolResult (D-04 interop)");
assert_eq!(parsed.is_error, Some(false));
assert_eq!(parsed.content.len(), 1, "structured() produces exactly one content block");
let content_json = serde_json::to_value(&parsed.content).unwrap();
assert_eq!(content_json[0]["type"].as_str(), Some("text"));
```
SC#5 write-path tests mirror this pattern for both `is_error: Some(false)` (success) and `is_error: Some(true)` (guard denied).

---

### `ferro-mcp-server/src/error.rs` (utility) — MODIFY

**Analog:** Same file (lines 1-22 — existing thiserror variants).

**Existing variants pattern** (`error.rs` lines 1-22 verbatim):
```rust
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("render error: {0}")]
    Render(String),
    #[error("invalid filter: {0}")]
    InvalidFilter(String),
    /// Caller is not authenticated or their credential scope is insufficient.
    /// Maps to JSON-RPC -32603 at the jsonrpc layer.
    #[error("auth error: {0}")]
    Auth(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

**Variants to add** (follow exactly the same doc-comment + error-string convention):
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

**JSON-RPC envelope mapping rule** (RESEARCH.md §Error Variants):

| Variant | Envelope key | code / isError |
|---------|-------------|----------------|
| `ActionNotFound` | `"error"` | `-32601` |
| `Auth` (scope) | `"error"` | `-32603` |
| `GuardFailed` | `"result"` | `isError: true` |
| `Validation` | `"result"` | `isError: true` |
| Executor `Err` | `"result"` | `isError: true` |

---

### `ferro-mcp-server/src/config.rs` (config) — READ ONLY

**Stays identity-only.** `McpServerConfig` holds `app_name`, `app_url`, `version` (lines 9-16). Do NOT add `WriteDispatcher` here. The research recommendation (RESEARCH.md §Registration API) is to thread `dispatcher: &WriteDispatcher` as a new parameter at call sites, not to hold it in config. This keeps the library surface clean and mirrors the pattern of threading `db` and `tenant_id` as plain args (confirmed: `app/src/controllers/mcp.rs` line 177 already threads all args explicitly).

---

### `ferro-mcp-oauth/src/migration.rs` (migration) — MODIFY

**Analog:** `MigrationMcpApiKeys` struct in same file (lines 82-179). Copy the exact pattern.

**Verbatim template** (`migration.rs` lines 89-179 — `MigrationMcpApiKeys`):
```rust
#[derive(DeriveMigrationName)]
pub struct MigrationMcpApiKeys;

#[async_trait::async_trait]
impl MigrationTrait for MigrationMcpApiKeys {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(McpApiKeys::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(McpApiKeys::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(McpApiKeys::TenantId).big_integer().not_null())
                    .col(ColumnDef::new(McpApiKeys::KeyHash).string().not_null())
                    // ... more cols ...
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_mcp_api_keys_key_hash")
                    .table(McpApiKeys::Table)
                    .col(McpApiKeys::KeyHash)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_mcp_api_keys_tenant_id")
                    .table(McpApiKeys::Table)
                    .col(McpApiKeys::TenantId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(McpApiKeys::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum McpApiKeys {
    Table, Id, TenantId, KeyHash, Scope, RevokedAt, CreatedAt, UpdatedAt,
}
```

**New `MigrationMcpIdempotencyKeys` schema** (copy structure, swap columns):
- `id` big_integer PK auto_increment
- `tenant_id` big_integer NOT NULL
- `idempotency_key` string NOT NULL
- `result` text NOT NULL (JSON-serialized `Value`)
- `created_at` timestamp_with_time_zone NOT NULL DEFAULT CURRENT_TIMESTAMP
- UNIQUE index on `(tenant_id, idempotency_key)` — the race-condition-safe enforcement primitive (composite, unlike `key_hash` which is a single-column unique)
- Non-unique index on `tenant_id` alone (lookup performance)

**UNIQUE composite index pattern** (differs from `MigrationMcpApiKeys` which uses single-column unique):
```rust
manager
    .create_index(
        Index::create()
            .name("idx_mcp_idempotency_keys_tenant_key")
            .table(McpIdempotencyKeys::Table)
            .col(McpIdempotencyKeys::TenantId)
            .col(McpIdempotencyKeys::IdempotencyKey)
            .unique()
            .to_owned(),
    )
    .await?;
```

**Test pattern** (`migration.rs` lines 252-331 — `mcp_api_keys_migration_creates_table_and_indexes`):
```rust
struct TestMigratorMcpIdempotencyKeys;

#[async_trait::async_trait]
impl MigratorTrait for TestMigratorMcpIdempotencyKeys {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(super::MigrationMcpIdempotencyKeys)]
    }
}

#[tokio::test]
async fn mcp_idempotency_keys_migration_creates_table_and_indexes() {
    let conn = Database::connect("sqlite::memory:").await.expect("connect");
    TestMigratorMcpIdempotencyKeys::up(&conn, None).await.expect("up");
    // sqlite_master checks for table + composite unique index
    let table_row = conn.query_one(Statement::from_string(
        DatabaseBackend::Sqlite,
        "SELECT name FROM sqlite_master WHERE type='table' AND name='mcp_idempotency_keys'"
            .to_string(),
    )).await.expect("query").expect("table exists");
    // ... index checks ...
    TestMigratorMcpIdempotencyKeys::down(&conn, None).await.expect("down");
}
```

---

### `ferro-mcp-server/Cargo.toml` (config) — MODIFY

**Add `ferro-audit` dep** (no circular dep — `ferro-audit` is in Wave 1A of `publish.yml` line 211; `ferro-mcp-server` is in Wave 2 line 275):
```toml
[dependencies]
# ... existing deps unchanged ...
ferro-audit = { path = "../ferro-audit", version = "0.2" }
```
`ferro-audit` deps: `sea-orm`, `uuid`, `serde_json`, `thiserror`, `tracing`, `chrono` — no `ferro-mcp-server` or `ferro-projections` (no cycle confirmed).

---

### `app/src/controllers/mcp.rs` (controller) — MODIFY

**Analog:** Same file (existing `handle_tools_call` call at line 177; `make_tool_deny_response` at lines 35-47).

**Current call site** (`mcp.rs` line 177 — the line to extend):
```rust
handle_tools_call(params, &services, db.inner(), tenant_id, &ctx).await
```
After modification:
```rust
let dispatcher = make_write_dispatcher();
handle_tools_call(params, &services, db.inner(), tenant_id, &ctx, &dispatcher).await
```

**`make_write_dispatcher` registration pattern** (RESEARCH.md §Registration API — mirrors the boxed-closure pattern in `mcp_tenant_isolation.rs`'s `build_test_lookup`):
```rust
fn make_write_dispatcher() -> WriteDispatcher {
    WriteDispatcher {
        executor: Box::new(|action_name, inputs, tenant_id, db| {
            Box::pin(async move {
                match action_name {
                    "submit" | "approve" | "ship" => {
                        let id: i64 = inputs["id"].as_i64()
                            .ok_or_else(|| crate::Error::Validation("missing id".into()))?;
                        // TenantScoped::find_for_tenant — None → cross-tenant denial (D-03)
                        use ferro::tenant::TenantScoped;
                        let order = crate::models::orders::Order::find_for_tenant(id as i32, tenant_id)
                            .await
                            .map_err(|e| /* map FrameworkError */)?
                            .ok_or_else(|| /* Error::not found or cross-tenant */)?;
                        // apply state transition via SeaORM ActiveModel
                        Ok(serde_json::json!({ "id": order.id, "status": order.status }))
                    }
                    _ => Err(/* ActionNotFound */),
                }
            })
        }),
        guard_evaluator: Box::new(|guard_name, tenant_id, inputs, db| {
            Box::pin(async move {
                match guard_name {
                    "is_manager" => {
                        // live DB query — NEVER ctx.evaluated_guards
                        Ok(check_is_manager(tenant_id, db).await?)
                    }
                    _ => Ok(true),
                }
            })
        }),
    }
}
```

**`make_tool_deny_response` isError shape** (`mcp.rs` lines 35-47 — use as model for `write_tool_error_result`):
```rust
fn make_tool_deny_response(message: &str, id: &Value) -> Value {
    let mut payload = json!({
        "result": {
            "content": [{ "type": "text", "text": message }],
            "isError": true
        }
    });
    if let Some(obj) = payload.as_object_mut() {
        obj.insert("jsonrpc".into(), json!("2.0"));
        obj.insert("id".into(), id.clone());
    }
    payload
}
```
The write path helper omits the `jsonrpc`/`id` splice (that is done at the outer envelope level in the handler) but keeps the `result.content[].type=text` + `isError:true` shape.

---

### `app/src/models/orders.rs` (model) — MODIFY

**Analog:** `framework/src/tenant/scoped.rs` (trait definition, lines 27-41).

**`TenantScoped` trait signature** (`framework/src/tenant/scoped.rs` lines 27-41 verbatim):
```rust
#[async_trait]
pub trait TenantScoped: Sized + Send + Sync {
    type Id: std::str::FromStr + Send;

    async fn find_for_tenant(id: Self::Id, tenant_id: i64) -> Result<Option<Self>, FrameworkError>;
}
```

**Order entity facts** (`app/src/models/entities/orders.rs` lines 9-19):
- Auto-generated, do NOT edit
- `id: i32` (primary key type)
- `tenant_id: i64`
- Deriving `DeriveEntityModel`, `Serialize`, `Deserialize`, `FerroModel`

**`TenantScoped` impl to add** in `app/src/models/orders.rs` (the safe extension file, currently only `pub type Order = Model`):
```rust
use ferro::async_trait;
use ferro::FrameworkError;
use ferro::tenant::TenantScoped;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

#[async_trait]
impl TenantScoped for Model {
    type Id = i32;

    async fn find_for_tenant(id: i32, tenant_id: i64) -> Result<Option<Self>, FrameworkError> {
        use super::entities::orders::{Column, Entity};
        Entity::find_by_id(id)
            .filter(Column::TenantId.eq(tenant_id))
            .one(ferro::DB::connection()
                .map_err(|e| FrameworkError::Database(e.to_string()))?)
            .await
            .map_err(|e| FrameworkError::Database(e.to_string()))
    }
}
```
Note: `ferro::async_trait` is the re-export of `async_trait` from the framework crate (pattern from `TenantScoped` docstring in `scoped.rs` line 17-21 example).

---

### `app/src/tests/mcp_write_dispatch.rs` (test) — CREATE

**Analog:** `app/src/tests/mcp_tenant_isolation.rs` (full file, 442 lines).

**DB setup pattern** (`mcp_tenant_isolation.rs` lines 25-33):
```rust
async fn setup_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("in-memory SQLite connect failed");
    Migrator::up(&db, None)
        .await
        .expect("migrations failed on test DB");
    db
}
```
The same `Migrator::up` call applies — the idempotency table migration and ferro-audit migration must be registered in `app/src/migrations/` before the tests run.

**Seeding pattern** (`mcp_tenant_isolation.rs` lines 36-116):
```rust
async fn seed_two_tenants(db: &DatabaseConnection) {
    use crate::models::entities::orders::ActiveModel as OrderActive;
    // ... insert tenants, users, orders with explicit IDs ...
    for (id, tid, customer) in [
        (1i32, 1i64, "Alice Acme"),
        (2i32, 1i64, "Alice Acme"),
        (3i32, 2i64, "Bob Globex"),
        (4i32, 2i64, "Bob Globex"),
    ] {
        OrderActive {
            id: Set(id),
            customer_name: Set(customer.into()),
            total: Set(10.0 * id as f64),
            status: Set("submitted".into()),
            created_at: Set(now.into()),
            tenant_id: Set(tid),
        }
        .insert(db).await.unwrap_or_else(|e| panic!("seed: {e}"));
    }
}
```

**SC#2 cross-tenant test pattern** (RESEARCH.md §TenantScoped, modeled on `tenant_a_isolation`):
```rust
#[tokio::test]
async fn cross_tenant_write_denied() {
    let db = setup_db().await;
    seed_two_tenants(&db).await;
    let dispatcher = make_test_dispatcher_for_submit();

    // Order id=3 belongs to tenant 2; calling as tenant 1 → find_for_tenant → None → Err
    let result = call_write_tool("submit", json!({"id": 3}), Some(1), &db, &dispatcher).await;

    let parsed: CallToolResult = serde_json::from_value(result["result"].clone())
        .expect("must parse as CallToolResult");
    assert_eq!(parsed.is_error, Some(true), "cross-tenant write must return isError:true");

    // Assert tenant 2's order status unchanged in DB
    let order = load_order(3, &db).await;
    assert_eq!(order.status, "submitted", "Tenant B order must not be mutated");
}
```

**SC#3 idempotency test pattern** (RESEARCH.md §Idempotency — exec_count via shared counter):
```rust
#[tokio::test]
async fn idempotent_replay_does_not_re_execute() {
    let db = setup_db().await;
    seed_two_tenants(&db).await;
    let exec_count = Arc::new(AtomicUsize::new(0));

    let dispatcher = WriteDispatcher {
        executor: Box::new({
            let count = exec_count.clone();
            move |_, _, _, _| {
                count.fetch_add(1, Ordering::SeqCst);
                Box::pin(async { Ok(serde_json::json!({ "status": "submitted" })) })
            }
        }),
        guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
    };

    let args = json!({ "id": 1, "idempotency_key": "test-key-abc" });
    let result1 = call_write_tool_with_args("submit", args.clone(), Some(1), &db, &dispatcher).await;
    let result2 = call_write_tool_with_args("submit", args.clone(), Some(1), &db, &dispatcher).await;

    assert_eq!(result1["result"]["structuredContent"], result2["result"]["structuredContent"],
        "idempotent replay must return same result");
    assert_eq!(exec_count.load(Ordering::SeqCst), 1, "executor must fire exactly once");
}
```

**SC#4 audit test pattern** (`ferro-audit/src/entry.rs` lines 204-240, query pattern):
```rust
#[tokio::test]
async fn write_call_produces_audit_entry() {
    let db = setup_db().await;
    seed_two_tenants(&db).await;
    let dispatcher = make_test_dispatcher_for_submit();

    call_write_tool("submit", json!({"id": 1}), Some(1), &db, &dispatcher).await;

    use ferro_audit::{history_for_target, AuditTarget};
    let entries = history_for_target(&AuditTarget::new("submit", "1"), &db)
        .await
        .expect("history_for_target");
    assert!(!entries.is_empty(), "audit entry must be written after write call");
    let entry = &entries[0];
    assert_eq!(entry.action, "mcp.action.submit");
    assert_eq!(entry.tenant_id, Some("1".to_string()));
    assert!(entry.after.is_some(), "after must contain execution result");
}
```

**SC#5 write-result parse test** (mirrored from `jsonrpc.rs` lines 193-235):
```rust
#[tokio::test]
async fn write_tool_result_parses_as_valid_mcp_content() {
    // success case
    let parsed: CallToolResult = serde_json::from_value(result["result"].clone())
        .expect("must parse as CallToolResult");
    assert_eq!(parsed.is_error, Some(false));
    assert_eq!(parsed.content.len(), 1);
    assert_eq!(serde_json::to_value(&parsed.content).unwrap()[0]["type"].as_str(), Some("text"));

    // guard-denied case
    let denied_parsed: CallToolResult = serde_json::from_value(denied_result["result"].clone())
        .expect("guard-denied must parse as CallToolResult");
    assert_eq!(denied_parsed.is_error, Some(true));
}
```

---

## Shared Patterns

### Tenant Discipline
**Source:** `ferro-mcp-server/src/dispatch.rs` lines 99-107 (doc comment) + lines 153-167 (fail-closed impl)
**Apply to:** `write_dispatch.rs`, `handle_write_call` in `jsonrpc.rs`
```rust
// Security: tenant value is NEVER sourced from the call payload.
// Always from the `tenant_id` parameter passed by the caller
// (the app handler reads `current_tenant().map(|t| t.id)`).
// Fail-closed: None tenant + tenant-requiring context → Err, never executes action.
```

### Structured Result Construction
**Source:** `ferro-mcp-server/src/jsonrpc.rs` lines 115-123 (`CallToolResult::structured`) + `app/src/controllers/mcp.rs` lines 35-47 (`make_tool_deny_response`)
**Apply to:** `write_dispatch.rs` / `handle_write_call` for all 5 outcome types
- Success: `CallToolResult::structured(payload)` → `is_error: Some(false)`
- All error outcomes (guard denied, validation, executor error): `{ "result": { "content": [{"type":"text","text": msg}], "isError": true, "structuredContent": payload } }` — do NOT use `CallToolResult::structured()` for errors

### Error Variant Convention
**Source:** `ferro-mcp-server/src/error.rs` (all variants use `#[error("lowercase noun: {0}")]` with thiserror derive)
**Apply to:** new `GuardFailed`, `ActionNotFound`, `Validation` variants

### Migration Structure
**Source:** `ferro-mcp-oauth/src/migration.rs` lines 82-331
**Apply to:** `MigrationMcpIdempotencyKeys` in same file
- `#[derive(DeriveMigrationName)]` on struct
- `#[async_trait::async_trait]` on `impl MigrationTrait`
- `Table::create().table(...).if_not_exists()` with `.col()` chain
- `Index::create().name(...).table(...).col(...).unique()` for constraint enforcement
- Companion test struct using `sqlite_master WHERE type='table'` and `type='index'` pattern

### In-Memory SQLite Test Harness
**Source:** `app/src/tests/mcp_tenant_isolation.rs` lines 25-116
**Apply to:** `app/src/tests/mcp_write_dispatch.rs`
- `Database::connect("sqlite::memory:")` + `Migrator::up(&db, None)`
- `ActiveModel { field: Set(value), ... }.insert(db).await`
- Arc-backed boxed closures for test-local dispatchers

### ferro-audit Builder Chain
**Source:** `ferro-audit/src/entry.rs` lines 25-57, `ferro-audit/src/lib.rs` re-exports
**Apply to:** `write_dispatch.rs` audit call after execution
```
AuditEntry::record(action_str)
    .tenant(tenant_id.to_string())
    .actor(AuditActor::User(tenant_id.to_string()))
    .target(AuditTarget::new(kind_str, id_str))
    .after(result_json)
    .reason(reason_str)
    .write(db).await?
```
`AuditActor`, `AuditEntry`, `AuditTarget` are all pub re-exports from `ferro_audit::` root.

---

## No Analog Found

All files have analogs. No new-from-scratch patterns required.

---

## Key Notes for Planner

1. **publish.yml wave order is correct:** `ferro-audit` is in Wave 1A (line 211 of publish.yml); `ferro-mcp-server` is in Wave 2 (line 275). Adding `ferro-audit` as a dep of `ferro-mcp-server` requires no wave reordering.

2. **`async-trait` dep not needed:** The boxed-future `Pin<Box<dyn Future<...>>>` pattern for `ExecutorFn`/`GuardEvaluatorFn` avoids the dep entirely. Do not add it to `ferro-mcp-server/Cargo.toml`.

3. **Wave B test imports:** `mcp_write_dispatch.rs` needs `ferro_audit::history_for_target` which requires `ferro-audit` to be a dep of the `app` crate or available via `ferro`. Confirm `app/Cargo.toml` has `ferro-audit` before writing Wave B tests.

4. **`TenantScoped` impl note:** `app/src/models/entities/orders.rs` is AUTO-GENERATED (do not edit). The impl goes in `app/src/models/orders.rs` (the extension file, currently only `pub type Order = Model`). The `find_for_tenant` needs a DB connection — check whether to use `ferro::DB::connection()` (global) or thread `db: &DatabaseConnection` as a param. In tests it must accept an explicit `&DatabaseConnection`; in the executor closure `db` is already passed as an arg.

5. **Idempotency INSERT concurrency:** Use `INSERT OR IGNORE` (SQLite) / `INSERT ... ON CONFLICT DO NOTHING` (Postgres) for the idempotency store to handle concurrent identical requests without constraint error propagation.

---

## Metadata

**Analog search scope:** `ferro-mcp-server/src/`, `ferro-mcp-oauth/src/`, `app/src/controllers/`, `app/src/models/`, `app/src/tests/`, `ferro-audit/src/`, `framework/src/tenant/`
**Files scanned:** 11 source files read directly
**Pattern extraction date:** 2026-06-13
