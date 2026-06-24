# Phase 243: App Integration, E2E, Envelope Guard & Catalog/Docs — Pattern Map

**Mapped:** 2026-06-24
**Files analyzed:** 6 new/modified files
**Analogs found:** 6 / 6

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `app/src/projections/order.rs` | app projection (MODIFY) | CRUD opt-in | same file — additive builder calls | exact (self-analog) |
| `app/src/tests/crud_e2e.rs` | app e2e test (CREATE) | CRUD + event-driven | `app/src/tests/mcp_write_dispatch.rs` + `single_source.rs` | exact |
| `app/src/tests/mod.rs` | test module registry (MODIFY) | — | `app/src/tests/mod.rs` itself (current state) | exact (self-analog) |
| `ferro-mcp/src/tools/code_templates.rs` | authoring tool (MODIFY) | request-response | same file — add one category function | exact (self-analog) |
| `ferro-mcp/src/tools/generation_context.rs` | authoring tool (MODIFY) | request-response | same file — extend `CommonPatterns` | exact (self-analog) |
| `docs/src/features/projections.md` | docs page (MODIFY) | — | same file — add section at end of `## MCP Tools` | exact (self-analog) |

---

## Pattern Assignments

### `app/src/projections/order.rs` (app projection, CRUD opt-in)

**Analog:** Same file — four additive `.creatable/.updatable/.deletable/.mcp_write_ability` builder calls on the existing `ServiceDef` chain.

**Current state** (lines 11-15 — the top of the builder chain):
```rust
// app/src/projections/order.rs lines 11-15
pub fn service_def() -> ServiceDef {
    ServiceDef::new("order")
        .mcp_exposed(true)
        .tenant_column("tenant_id") // FK column for dispatch predicate injection (D-02)
        .mcp_ability("view-orders") // Gate ability required for tools/call (D-04)
```

**Pattern to add** (insert after `.mcp_ability("view-orders")`, before `.display_name`):
```rust
        .mcp_write_ability("manage-orders") // ADD — gates create_/update_/delete_ tools
        .creatable(true)                    // ADD — derives create_order tool
        .updatable(true)                    // ADD — derives update_order tool
        .deletable(true)                    // ADD — derives delete_order tool (confirmation-gated)
```

D-05 constraint: `status` is excluded from write inputs by `derive_crud_plan` when a `StateMachine` is present; `id`, `created_at`, `tenant_id` are also excluded. The `soft_delete_column` defaults to `deleted_at` (column already in the migration per `m20260623_add_deleted_at_to_orders`; entity field `pub deleted_at: Option<String>` confirmed in `app/src/models/entities/orders.rs` line 20).

---

### `app/src/tests/crud_e2e.rs` (app e2e test, CRUD + confirmation flow + parity)

**Primary analog:** `app/src/tests/mcp_write_dispatch.rs` (setup_db, seed_two_tenants, WriteDispatcher pattern, McpContext with write_authorized, handle_tools_call call shape, confirmation feature gating)

**Secondary analog:** `app/src/tests/single_source.rs` (MCP path + visual path driven from same plan, identical-effect assertion, audit-channel-only divergence, `#[cfg(all(test, not(feature = "confirmation")))]` module gate)

#### Imports pattern (from `mcp_write_dispatch.rs` lines 19-36):
```rust
#[cfg(test)]
mod tests {
    use crate::migrations::Migrator;
    use ferro::serde_json::json;
    #[cfg(not(feature = "confirmation"))]
    use ferro_audit::{history_for_target, AuditTarget};
    #[cfg(feature = "confirmation")]
    use ferro_mcp_server::McpServerConfig;
    use ferro_mcp_server::{handle_tools_call, McpContext, WriteDispatcher};
    use sea_orm::{
        ActiveModelTrait, ActiveValue::Set, ColumnTrait, Database, DatabaseConnection, EntityTrait,
        QueryFilter,
    };
    use sea_orm_migration::prelude::*;
```

Additional imports needed for the CRUD e2e (not in the analog):
```rust
    use ferro::write::{dispatch_write, WriteDispatcher as _, WriteError};
    use ferro_projections::{derive_crud_plan, CrudVerb};
```

#### `setup_db()` pattern — copy verbatim from `mcp_write_dispatch.rs` lines 45-53:
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

#### `seed_two_tenants()` pattern — copy verbatim from `mcp_write_dispatch.rs` lines 59-136:
Seeds tenants 1 (acme) + 2 (globex), users 901 (alice) + 902 (bob), and orders 1-4 with `deleted_at: Set(None)`. Note: orders 1-4 use explicit ids; a `create_order` call must use `ActiveModel { ..Default::default() }` (no explicit id) so SQLite auto-increment assigns id 5+.

```rust
    async fn seed_two_tenants(db: &DatabaseConnection) {
        use crate::models::entities::orders::ActiveModel as OrderActive;
        use crate::models::entities::tenants::ActiveModel as TenantActive;
        use crate::models::entities::users::ActiveModel as UserActive;

        let now = "2026-06-14T00:00:00+00:00";
        // ... (copy full body from mcp_write_dispatch.rs lines 64-135)
    }
```

#### McpContext for CRUD write calls — from `mcp_write_dispatch.rs` lines 270-275:
```rust
    // Authorized CRUD write path — write_authorized: Some(true) is required.
    // Without it, is_crud_write_tool check in write_dispatch.rs lines 157-164
    // returns -32603 before the executor runs.
    let ctx = McpContext {
        tenant_id: Some(tenant_id),
        scope: Some("read_write".to_string()),
        write_authorized: Some(true),
        ..Default::default()
    };
```

For the auth-gate test (CRUD-05), use `write_authorized: None` (or omit) and assert the result contains `error.code == -32603`.

For `list_order` (read path), `McpContext::default()` is sufficient — no `write_authorized` check on reads.

#### `handle_tools_call` invocation pattern — from `mcp_write_dispatch.rs` lines 276-290:
```rust
    let params = json!({ "name": tool_name, "arguments": arguments });
    handle_tools_call(
        params,
        &services,
        db,
        tenant_id,
        &ctx,
        dispatcher,
        #[cfg(feature = "confirmation")]
        &ferro_ai::InMemoryConfirmationStore::new(),
        #[cfg(feature = "confirmation")]
        &test_config(),
    )
    .await
```

The `#[cfg(feature = "confirmation")]` guards on the last two arguments are mandatory — the function signature is conditionally compiled and the argument count changes.

#### `test_config()` helper — from `mcp_write_dispatch.rs` lines 249-257:
```rust
    #[cfg(feature = "confirmation")]
    fn test_config() -> McpServerConfig {
        McpServerConfig {
            app_name: "TestApp".into(),
            app_url: "https://test.example".into(),
            version: "0.0.0".into(),
            confirmation_ttl_seconds: 300,
        }
    }
```

#### `make_crud_dispatcher` — new function (no direct analog; pattern inferred from `mcp_write_dispatch.rs` `make_test_write_dispatcher` lines 161-247):

The CRUD dispatcher differs from the transition dispatcher: it handles `create_order`/`update_order`/`delete_order` with a `match action_name` body instead of calling `derive_transition_plan`. It does NOT call `exposed_services()` or `find_action`.

```rust
    fn make_crud_dispatcher(db: DatabaseConnection) -> WriteDispatcher {
        use crate::models::entities::orders::{ActiveModel as OrderActive, Column, Entity};
        let db_exec = db.clone();
        WriteDispatcher::new(
            Box::new(move |action_name, inputs, tenant_id, _db_arg| {
                let db = db_exec.clone();
                let action_name = action_name.to_string();
                let inputs = inputs.clone();
                Box::pin(async move {
                    match action_name.as_str() {
                        "create_order" => {
                            // status="draft" server-side (D-05); tenant_id injected; id not set
                            let now = chrono::Utc::now().to_rfc3339();
                            let record = OrderActive {
                                customer_name: Set(inputs["customer_name"]
                                    .as_str().unwrap_or("").into()),
                                total: Set(inputs["total"].as_f64().unwrap_or(0.0)),
                                status: Set("draft".into()),
                                tenant_id: Set(tenant_id),
                                created_at: Set(now),
                                deleted_at: Set(None),
                                ..Default::default()
                            }
                            .insert(&db)
                            .await
                            .map_err(|e| ferro::write::WriteError::Database(e.to_string()))?;
                            Ok(json!({ "id": record.id, "status": record.status }))
                        }
                        "update_order" => {
                            let id = inputs["id"].as_i64()
                                .ok_or_else(|| ferro::write::WriteError::Validation("missing id".into()))?;
                            let order = Entity::find_by_id(id as i32)
                                .filter(Column::TenantId.eq(tenant_id))
                                .filter(Column::DeletedAt.is_null())
                                .one(&db).await
                                .map_err(|e| ferro::write::WriteError::Database(e.to_string()))?
                                .ok_or(ferro::write::WriteError::RecordNotFound)?;
                            let mut active: OrderActive = order.into();
                            if let Some(v) = inputs["customer_name"].as_str() {
                                active.customer_name = Set(v.into());
                            }
                            if let Some(v) = inputs["total"].as_f64() {
                                active.total = Set(v);
                            }
                            // status NOT settable here (D-05/CRUD-02)
                            let updated = active.update(&db).await
                                .map_err(|e| ferro::write::WriteError::Database(e.to_string()))?;
                            Ok(json!({ "id": updated.id, "customer_name": updated.customer_name }))
                        }
                        "delete_order" => {
                            let id = inputs["id"].as_i64()
                                .ok_or_else(|| ferro::write::WriteError::Validation("missing id".into()))?;
                            let order = Entity::find_by_id(id as i32)
                                .filter(Column::TenantId.eq(tenant_id))
                                .filter(Column::DeletedAt.is_null())
                                .one(&db).await
                                .map_err(|e| ferro::write::WriteError::Database(e.to_string()))?
                                .ok_or(ferro::write::WriteError::RecordNotFound)?;
                            let mut active: OrderActive = order.into();
                            let now = chrono::Utc::now().to_rfc3339();
                            active.deleted_at = Set(Some(now));
                            active.update(&db).await
                                .map_err(|e| ferro::write::WriteError::Database(e.to_string()))?;
                            Ok(json!({ "id": id, "deleted": true }))
                        }
                        _ => Err(ferro::write::WriteError::ActionNotFound(action_name)),
                    }
                })
            }),
            // No guards on CRUD verbs
            Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
        )
    }
```

#### D-07 envelope assertion pattern — from `mcp_tenant_isolation.rs` lines 279-305:
```rust
    fn assert_write_envelope_ok(result: &ferro::serde_json::Value, tool_name: &str) {
        // Locks the Phase 205 content[] shape for every CRUD verb (D-07).
        let content = result["result"]["content"]
            .as_array()
            .expect("result.content must be an array");
        assert_eq!(
            content[0]["type"].as_str(),
            Some("text"),
            "{tool_name}: content[0] must be a text block (type=text) — Phase 205 envelope"
        );
        assert_eq!(
            result["result"]["structuredContent"]["status"].as_str(),
            Some("ok"),
            "{tool_name}: structuredContent.status must be ok"
        );
        assert_eq!(
            result["result"]["structuredContent"]["action"].as_str(),
            Some(tool_name),
            "{tool_name}: structuredContent.action must equal the tool name"
        );
        assert!(
            result["result"]["structuredContent"]["result"].is_object(),
            "{tool_name}: structuredContent.result must be an object"
        );
        assert_ne!(
            result["result"]["isError"], true,
            "{tool_name}: isError must not be true on success"
        );
    }
```

The `list_order` envelope check uses the rows-variant from `mcp_tenant_isolation.rs` lines 279-295:
```rust
    fn assert_list_envelope(result: &ferro::serde_json::Value) {
        let content = result["result"]["content"]
            .as_array()
            .expect("result.content must be an array");
        assert_eq!(content[0]["type"].as_str(), Some("text"),
            "list envelope: content[0] must be text block");
        let _rows = result["result"]["structuredContent"]["rows"]
            .as_array()
            .expect("structuredContent.rows must be an array");
    }
```

#### Confirmation-required envelope shape — from RESEARCH.md Pattern 5:
```rust
    // Expected shape when delete_order is called without a token (feature = "confirmation")
    assert_eq!(
        result["result"]["structuredContent"]["error_kind"].as_str(),
        Some("confirmation_required")
    );
    assert_eq!(
        result["result"]["structuredContent"]["request_tool"].as_str(),
        Some("request_confirm_delete_order")
    );
    assert_eq!(result["result"]["isError"], true);
```

#### Module gate pattern — from `single_source.rs` line 24:
```rust
// For tests that require feature = "confirmation" off:
#[cfg(all(test, not(feature = "confirmation")))]
mod tests { ... }

// For tests that require feature = "confirmation" on:
// Use separate #[cfg(feature = "confirmation")] #[tokio::test] functions
// inside the main mod tests block (following mcp_write_dispatch.rs precedent)
```

#### Visual parity driver — from `single_source.rs` lines 267-289, adapted for CRUD:
```rust
    // Visual path for CRUD: dispatch_write(.., "web", ..) with a CrudPlan
    // This is the analog of drive_visual() in single_source.rs lines 267-289
    async fn drive_visual_crud(
        verb: CrudVerb,
        tool_name: &str,
        inputs: ferro::serde_json::Value,
        tenant_id: i64,
        db: &DatabaseConnection,
    ) -> ferro::write::WriteResult<ferro::serde_json::Value> {
        let svc = order_service();
        let plan = ferro_projections::derive_crud_plan(&svc, verb, &inputs)
            .expect("derive_crud_plan");
        let crud_action = ferro::ActionDef::new(tool_name);
        let disp = make_crud_dispatcher(db.clone());
        ferro::write::dispatch_write(
            &crud_action,
            &inputs,
            tenant_id,
            db,
            &disp,
            None,       // no transition guard on CRUD verbs
            "web",      // audit channel tag
            #[cfg(feature = "confirmation")]
            false,
            Some(&plan),
        )
        .await
    }
```

Parity assertion (create and update only; delete parity is `#[cfg(not(feature = "confirmation"))]`): both MCP and visual paths must produce a row with the same persisted fields; audit channel (`mcp.action.create_order` vs `web.action.create_order`) is the only divergence — assert this with `history_for_target` following `single_source.rs` lines 337-362.

---

### `app/src/tests/mod.rs` (test module registry, MODIFY)

**Analog:** Same file — current state is 6 entries (lines 1-6):
```rust
// app/src/tests/mod.rs — current state
pub mod magic_link;
pub mod mcp_tenant_isolation;
pub mod mcp_write_dispatch;
pub mod oauth_magic_link_resume_flow;
pub mod single_source;
pub mod visual_action;
```

**Pattern to add** — insert alphabetically or at end:
```rust
pub mod crud_e2e;
```

One line, same `pub mod <name>;` pattern.

---

### `ferro-mcp/src/tools/code_templates.rs` (authoring tool, MODIFY)

**Analog:** Same file — the `build_templates()` function (lines 48-79) extends by adding a new call to `projection_crud_templates()`, and a new private function `projection_crud_templates()` returns `Vec<CodeTemplate>` following the exact same struct-literal pattern as `api_templates()`, `json_view_templates()`, etc.

**Call site to add** in `build_templates()` (lines 48-79), after the existing `templates.extend(api_templates());` line:
```rust
    // Projection CRUD templates
    templates.extend(projection_crud_templates());
```

**New function structure** — mirrors the pattern of any existing category function, e.g. `api_templates()` lines 1400-1630:
```rust
fn projection_crud_templates() -> Vec<CodeTemplate> {
    vec![
        CodeTemplate {
            name: "projection_crud_service_def".to_string(),
            category: "projection_crud".to_string(),
            description: "ServiceDef with CRUD opt-in: derives create_/update_/delete_/list_ MCP tools and a soft-delete gate. Requires a deleted_at column and mcp_write_ability for the write gate.".to_string(),
            code: r#"use ferro::{
    ActionDef, DataType, FieldMeaning, GuardDef, ServiceDef,
    StateDef, StateMachine, Transition,
};

pub fn {{service}}_service_def() -> ServiceDef {
    ServiceDef::new("{{service}}")
        .mcp_exposed(true)
        .tenant_column("tenant_id")
        .mcp_ability("view-{{service}}s")     // read gate
        .mcp_write_ability("manage-{{service}}s") // write gate: create/update/delete tools
        .creatable(true)    // derives create_{{service}} tool
        .updatable(true)    // derives update_{{service}} tool
        .deletable(true)    // derives delete_{{service}} tool (confirmation-gated); requires deleted_at column
        .display_name("{{Service}}")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        // Add your fields here; status is excluded from write inputs when a StateMachine exists
        .field("name", DataType::String, FieldMeaning::EntityName)
}"#.to_string(),
            imports: vec![
                "use ferro::{ActionDef, DataType, FieldMeaning, ServiceDef};".to_string(),
            ],
            placeholders: vec![
                Placeholder {
                    name: "{{service}}".to_string(),
                    description: "Service name in snake_case (used as MCP tool prefix: create_{{service}}, update_{{service}}, delete_{{service}}, list_{{service}})".to_string(),
                    example: "order".to_string(),
                },
                Placeholder {
                    name: "{{Service}}".to_string(),
                    description: "Service display name in PascalCase".to_string(),
                    example: "Order".to_string(),
                },
            ],
        },
    ]
}
```

**Test guard to add** — the existing `test_all_categories_present` test (lines 1636-1676) uses `assert!(categories.contains("X"))`. Adding a new category does NOT break this test (the test only asserts named categories are present; extra categories are ignored). Add a guard assertion for the new category in the same test:
```rust
        assert!(
            categories.contains("projection_crud"),
            "Should have projection_crud templates"
        );
```

---

### `ferro-mcp/src/tools/generation_context.rs` (authoring tool, MODIFY)

**Analog:** Same file — the `CommonPatterns` struct (lines 41-47) has a `crud_handler` field (line 43). The CRUD opt-in pattern is added as a replacement or extension of that field's content.

**Current `crud_handler` field value** (line 80-88 in `execute()`):
```rust
            crud_handler: r#"#[handler]
pub async fn show(req: Request, id: Path<i32>) -> Response {
    let db = req.db();
    ...
}"#.to_string(),
```

**Extension pattern** — replace the `crud_handler` value with content that also covers the projection-CRUD opt-in so an agent reading `generation_context` learns this path:
```rust
            crud_handler: r#"// Option A: Traditional REST handler (web surface)
#[handler]
pub async fn show(req: Request, id: Path<i32>) -> Response {
    let db = req.db();
    let entity = Entity::find_by_id(*id)
        .one(db).await?
        .ok_or_else(|| not_found("Resource not found"))?;
    Ok(json!(entity))
}

// Option B: Projection-derived MCP CRUD tools (agent surface)
// Add to your ServiceDef in src/projections/<service>.rs:
//   .mcp_write_ability("manage-<service>s")  // write gate
//   .creatable(true)   // derives create_<service> MCP tool
//   .updatable(true)   // derives update_<service> MCP tool
//   .deletable(true)   // derives delete_<service> MCP tool (soft-delete via deleted_at)
// Requires: deleted_at column in migration, tenant_column set, mcp_ability for reads.
// Derived tools: create_<svc>, update_<svc>, delete_<svc>, list_<svc> with query polish."#
                .to_string(),
```

The `test_generation_context_has_all_sections` test (lines 168-201) asserts `!context.common_patterns.crud_handler.is_empty()` — it remains satisfied. No struct fields are added; only the string content of `crud_handler` changes.

---

### `docs/src/features/projections.md` (docs page, MODIFY)

**Analog:** Same file — insert a new `## MCP CRUD Opt-In` section. The file currently ends at line 640 with the `### projection_coverage` subsection inside `## MCP Tools`.

**Pattern to follow:** The existing `## MCP Tools` section (lines 612-640) uses `###` subsections with a bullet list of what a tool returns and when to use it. The new section uses `##` level (peer to `## MCP Tools`, not inside it) and follows the code-example format used throughout the file.

**Insertion point:** After the closing content of `## MCP Tools` (after line 640), add:

```markdown
## MCP CRUD Opt-In

A `ServiceDef` can expose CRUD write tools (`create_<svc>`, `update_<svc>`, `delete_<svc>`) to agents on the MCP surface in addition to the read tool (`list_<svc>`). This is an additive opt-in on the `ServiceDef` — it does not affect the visual/form write path, which uses the same `dispatch_write` kernel independently.

### Enabling CRUD tools

Add four builder calls to your service definition:

```rust
ServiceDef::new("order")
    .mcp_exposed(true)
    .tenant_column("tenant_id")
    .mcp_ability("view-orders")          // read gate (existing)
    .mcp_write_ability("manage-orders")  // write gate: scopes create/update/delete
    .creatable(true)                     // derives create_order tool
    .updatable(true)                     // derives update_order tool
    .deletable(true)                     // derives delete_order tool (soft-delete)
    // ... fields, state_machine, actions unchanged ...
```

**Prerequisites:**

| Requirement | Why |
|-------------|-----|
| `tenant_column` set | `tenant_id` is injected into every write; never accepted from the agent's input body |
| `deleted_at` column in the table | `deletable(true)` performs a soft-delete (sets `deleted_at`); hard-delete is not supported |
| `mcp_write_ability` present when any write flag is true | `ServiceDef::validate()` fails at boot otherwise (CRUD-07) |

### Derived tool set

| Tool | Behavior |
|------|----------|
| `create_<svc>` | INSERT a new row; `status` excluded from input when a StateMachine is defined (set server-side to initial state) |
| `update_<svc>` | UPDATE writable fields; `status`, `id`, `created_at`, `tenant_id` excluded from input |
| `delete_<svc>` | Soft-delete (sets `deleted_at`); confirmation-gated when the `confirmation` feature is enabled |
| `list_<svc>` | Read tool (unchanged); excludes soft-deleted rows from results |

### Authorization

A `read_write`-scoped MCP session with `write_authorized: Some(true)` is required for `create_`/`update_`/`delete_` tools. Without it the call returns a `-32603` transport error before reaching the executor. The `list_` tool has no write-authorization requirement.

### Confirmation flow for delete

When the `confirmation` feature is enabled, a bare `delete_<svc>` call returns:

```json
{
  "error_kind": "confirmation_required",
  "request_tool": "request_confirm_delete_<svc>"
}
```

The agent must call `request_confirm_delete_<svc>` to obtain a token, then `confirm_delete_<svc>` with the token to execute the soft-delete. Tokens are single-use.

### Separation from developer-MCP CRUD tools

`ferro-mcp/src/tools/crud_operations.rs` is a separate developer-facing tool (`ferro mcp` CLI) that provides SQL-level model introspection. It is not related to the projection-derived consumer-MCP CRUD tools described here. Do not conflate them.
```

---

## Shared Patterns

### In-process test harness (apply to `crud_e2e.rs`)
**Source:** `app/src/tests/mcp_write_dispatch.rs` lines 45-53 (`setup_db`), lines 59-136 (`seed_two_tenants`), lines 260-290 (`call_write_tool`)
- `setup_db()`: `Database::connect("sqlite::memory:")` + `Migrator::up(&db, None)`
- `seed_two_tenants()`: inserts tenants 1+2, users 901+902, orders 1-4 (with `deleted_at: Set(None)`)
- `call_write_tool()` wrapper: builds `McpContext` + `services` vec + calls `handle_tools_call`

### Feature gating pattern (apply to all CRUD write and confirmation tests)
**Source:** `mcp_write_dispatch.rs` throughout + `single_source.rs` line 24
```rust
// Module gate for confirmation-incompatible tests (the direct delete path):
#[cfg(not(feature = "confirmation"))]
#[tokio::test]
async fn test_name() { ... }

// Confirmation-specific tests:
#[cfg(feature = "confirmation")]
#[tokio::test]
async fn test_name_with_confirmation() { ... }

// In handle_tools_call invocations, always gate the store/config args:
handle_tools_call(
    params, &services, db, tenant_id, &ctx, dispatcher,
    #[cfg(feature = "confirmation")]
    &ferro_ai::InMemoryConfirmationStore::new(),
    #[cfg(feature = "confirmation")]
    &test_config(),
)
.await
```

### Envelope assertion (apply to all CRUD verb result assertions)
**Source:** `mcp_tenant_isolation.rs` lines 279-305 (text-block check + structuredContent.rows), extended in RESEARCH.md Pattern 4 for write-verb structuredContent shape.
- Read path: assert `content[0]["type"] == "text"` + `structuredContent["rows"].as_array()`
- Write path: assert `content[0]["type"] == "text"` + `structuredContent["status"] == "ok"` + `structuredContent["action"] == tool_name` + `structuredContent["result"].is_object()` + `isError != true`

### WriteDispatcher construction (apply to `make_crud_dispatcher`)
**Source:** `mcp_write_dispatch.rs` lines 161-247 (the `make_test_write_dispatcher` body)
- Two-arg `WriteDispatcher::new(executor_box, guard_box)`
- Executor closure: `Box::new(move |action_name, inputs, tenant_id, _db_arg| { ... Box::pin(async move { ... }) })`
- Guard closure: `Box::new(move |guard_name, tenant_id, _inputs, _db_arg| { ... Box::pin(async move { ... }) })`
- No external state captured beyond the `db` clone (Arc-backed, cheap)

---

## No Analog Found

All files have analogs in the codebase. No external patterns needed from RESEARCH.md.

---

## Metadata

**Analog search scope:** `app/src/tests/`, `app/src/projections/`, `ferro-mcp/src/tools/`, `docs/src/features/`
**Files scanned:** 8 source files read in full
**Pattern extraction date:** 2026-06-24
