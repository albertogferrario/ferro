# Phase 243: App Integration, E2E, Envelope Guard & Catalog/Docs — Research

**Researched:** 2026-06-24
**Domain:** Rust in-process MCP e2e, ServiceDef CRUD flip, structured-envelope regression guard, confirmation flow, ferro-mcp authoring surface
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** CI gate is the in-process `handle_tools_call` harness — in-memory SQLite + full `Migrator::up` + `read_write`-scoped `McpContext` (`write_authorized: Some(true)`) — mirroring `mcp_write_dispatch.rs` + `single_source.rs`.
- **D-02:** Live `:8090/mcp` + seeded `read_write` bearer drive is a documented **manual UAT smoke** only — NOT a CI gate.
- **D-03:** MCP↔visual parity uses the `single_source.rs` (Phase 232) approach: same `CrudPlan` through MCP framing and visual handler, identical persisted effects; audit channel is the only divergence.
- **D-04:** Add `.creatable(true).updatable(true).deletable(true).mcp_write_ability("manage-orders")` to `app/src/projections/order.rs`. Keep existing `.mcp_ability("view-orders")` and `.tenant_column("tenant_id")`.
- **D-05:** `create_order` sets `status` server-side to `draft`; `status` is never an `update_order` input; `id`, `created_at`, `tenant_id` excluded from write inputs. `soft_delete_column` defaults to `deleted_at`.
- **D-06:** `validate()` must pass at boot for the flipped projection (CRUD-07). The host `mcp.rs` write-ability path resolves `order` → `manage-orders` → `Gate::authorize_for`.
- **D-07:** Extend the envelope-assertion pattern (in `mcp_tenant_isolation.rs`) to assert well-formed `content[]` for each of `create_order`, `update_order`, `delete_order`.
- **D-08:** Update authoring-facing surface: `ferro-mcp` `code_templates` + `generation_context` + `docs/src/` projection-CRUD section.
- **D-09:** Do NOT conflate projection-derived consumer-MCP CRUD tools with `ferro-mcp/src/tools/crud_operations.rs`. They are different surfaces.
- **D-10:** Verify the json-ui builtin-component drift-guard count is NOT falsely tripped; CRUD tools are not json-ui components.

### Claude's Discretion

- Exact test-module layout and fixture naming.
- Whether confirmation-flow assertions are gated `#[cfg(feature = "confirmation")]` (follow `single_source.rs` precedent for destructive paths).
- Precise wording of docs/code_templates additions.

### Deferred Ideas (OUT OF SCOPE)

None — this is the Track A closeout phase. Tracks B–D are future milestones.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CRUD-01 | `create_<svc>` tool derived from projection opt-in | D-04/D-05 confirm what is needed; harness pattern in `mcp_write_dispatch.rs` shows how to drive it |
| CRUD-02 | `update_<svc>` tool derived; Status excluded when SM exists | Same harness; write_dispatch.rs CrudVerb::Update path |
| CRUD-03 | `delete_<svc>` soft-deletes, confirmation-gated, filtered from list | confirmed by `deleted_at` column presence; confirmation path documented below |
| CRUD-04 | `list_<svc>` range/comparison filters + pagination | Exercised through existing test pattern; validated when list returns post-create record |
| CRUD-05 | `read_write` scope + `.mcp_write_ability` gate | `McpContext { write_authorized: Some(true), scope: Some("read_write") }` wiring documented below |
| CRUD-06 | CRUD verbs dispatch through `framework::write` kernel; parity with visual surface | `dispatch_write` call chain + `single_source.rs` parity pattern documented below |
| CRUD-07 | `ServiceDef::validate()` fails fast when CRUD verb enabled without `mcp_write_ability` | The flip adds `.mcp_write_ability("manage-orders")` — validate() passes; missing ability → compile-time boot failure |
</phase_requirements>

---

## Summary

Phase 243 is a pure exercise-and-document phase: no new framework capability is needed. Everything required shipped in Phases 239–242. The planner must schedule four deliverables: (1) flip the `order` projection to CRUD, (2) write an in-process e2e test driving `create_order` → `list_order` → `update_order` → `delete_order` with the established harness, (3) extend the structured-envelope regression guard to each new verb, and (4) update `ferro-mcp` code_templates/generation_context and `docs/src/` with the CRUD opt-in pattern.

The key structural insight is that the existing `mcp_write_dispatch.rs` harness already shows the exact `McpContext` shape required (`write_authorized: Some(true)`, `scope: "read_write"`). The `single_source.rs` test shows how the parity assertion works: the SAME service drives both `handle_tools_call` (MCP) and `dispatch_write(.., "web")` (visual), asserting identical persisted state. The CRUD e2e test extends that pattern to the four CRUD verbs.

The confirmation flow for `delete_order` follows the already-tested `request_confirm_` / `confirm_` prefix-routing in `write_dispatch.rs`. The envelope shape for CRUD success is `{ status, action, result }` wrapped in `CallToolResult::structured()`, and for confirmation-required it is `write_tool_error_result({ error_kind: "confirmation_required", request_tool: "request_confirm_delete_order" })`.

**Primary recommendation:** Write one new test file `app/src/tests/crud_e2e.rs`, registered in `app/src/tests/mod.rs`. Add the projection flip as additive builder calls. Extend envelope assertions in the same new test file. Update three files in `ferro-mcp/src/tools/` and one `docs/src/` page.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Projection CRUD opt-in (`.creatable` etc.) | App `projections/order.rs` | — | Additive builder calls on the existing ServiceDef |
| E2E harness (in-process) | `app/src/tests/crud_e2e.rs` | `app/src/tests/mod.rs` | New test module; reuses setup_db/seed patterns |
| Envelope regression guard | Same test file | — | D-07: extends existing pattern into new module |
| CRUD write dispatch (already shipped) | `ferro-mcp-server/src/write_dispatch.rs` | `framework/src/write/mod.rs` | NOT modified in this phase |
| Visual parity (CRUD variant) | `app/src/tests/crud_e2e.rs` | `app/src/controllers/visual_action.rs` | Same `dispatch_write(.., "web")` call for create/update |
| Confirmation flow (delete) | `ferro-mcp-server/src/write_dispatch.rs` (shipped) | `ferro-ai::InMemoryConfirmationStore` | Exercised by the new test, not modified |
| Authoring surface updates | `ferro-mcp/src/tools/code_templates.rs` + `generation_context.rs` | `docs/src/features/projections.md` | SC#4 deliverable |

---

## Standard Stack

All libraries already present in the workspace. No new dependencies.

| Library | Version | Purpose |
|---------|---------|---------|
| `ferro-mcp-server` | workspace | `handle_tools_call`, `McpContext`, `WriteDispatcher` |
| `ferro_projections` | workspace | `derive_crud_plan`, `CrudVerb`, `ServiceDef` |
| `ferro::write` | workspace | `dispatch_write`, `WriteDispatcher`, `WriteError` |
| `ferro_audit` | workspace | `history_for_target`, `AuditTarget` — for audit assertions |
| `ferro_ai::InMemoryConfirmationStore` | workspace | Confirmation token store for delete flow |
| `sea_orm` | workspace | `Database::connect`, `Migrator::up`, entity ops |

[VERIFIED: reading imports in `mcp_write_dispatch.rs`, `single_source.rs`, `mcp_tenant_isolation.rs`]

---

## Architecture Patterns

### System Architecture Diagram (e2e data flow)

```
Test harness (crud_e2e.rs)
       │
       ├─ setup_db()
       │       └─ Database::connect("sqlite::memory:") + Migrator::up (incl. m20260623_add_deleted_at_to_orders)
       │
       ├─ seed_two_tenants() — tenants + users (for is_manager guard on existing actions)
       │
       ├─ McpContext { tenant_id: Some(1), scope: "read_write", write_authorized: Some(true) }
       │       (write_authorized=None → -32603 auth denied — tested in write_auth_gate)
       │
       ├─ create_order { customer_name, total }
       │       → handle_tools_call → handle_write_call
       │       → is_crud_write_tool check (write_authorized gate)
       │       → derive_crud_plan(svc, CrudVerb::Create, &args)
       │       → dispatch_write(&ActionDef::new("create_order"), .., "mcp", false, Some(&plan))
       │       → executor: INSERT orders (status="draft" server-side, tenant_id injected)
       │       → CallToolResult::structured({ status:"ok", action:"create_order", result:{id, status} })
       │
       ├─ list_order { limit: 10 }
       │       → handle_tools_call → read path (no write_authorized check)
       │       → content[0].type=="text" + structuredContent.rows contains created record
       │       → assert rows exclude soft-deleted records
       │
       ├─ update_order { id, customer_name: "new" }
       │       → handle_tools_call → handle_write_call
       │       → derive_crud_plan(svc, CrudVerb::Update, &args) — status excluded
       │       → dispatch_write → executor: UPDATE orders SET customer_name=...
       │       → CallToolResult::structured({ status:"ok", action:"update_order", result:{...} })
       │
       ├─ delete_order { id } [feature = "confirmation"]
       │       → Err(WriteError::ConfirmationRequired("delete_order"))
       │       → write_tool_error_result({ error_kind:"confirmation_required",
       │                                   request_tool:"request_confirm_delete_order" })
       │
       ├─ request_confirm_delete_order { id } → cfm_<token>
       ├─ confirm_delete_order { confirmation_token, id }
       │       → dispatch_write(is_confirmed=true, Some(&crud_plan))
       │       → executor: UPDATE orders SET deleted_at=... WHERE id=? AND tenant_id=? AND deleted_at IS NULL
       │       → CallToolResult::structured({ status:"ok", action:"delete_order", result:{id, deleted:true} })
       │
       └─ Parity assertion (feature off, create/update only):
              dispatch_write(&ActionDef::new("create_order"), .., "web", false, Some(&plan))
              → identical persisted row as MCP path
              → audit "web.action.create_order" vs "mcp.action.create_order" only divergence
```

### Recommended Project Structure (changes only)

```
app/src/tests/
├── mod.rs              # EDIT: add `pub mod crud_e2e;`
└── crud_e2e.rs         # NEW: CRUD e2e + envelope guard + parity assertions

app/src/projections/
└── order.rs            # EDIT: add 4 builder calls (creatable/updatable/deletable/mcp_write_ability)

ferro-mcp/src/tools/
├── code_templates.rs   # EDIT: add projection_crud template category
└── generation_context.rs  # EDIT: extend CommonPatterns with CRUD capability note

docs/src/features/
└── projections.md      # EDIT: add "MCP CRUD Opt-In" section
```

### Pattern 1: McpContext for CRUD e2e

The `write_authorized: Some(true)` field is the Phase 242 authorization signal. Without it, all CRUD write calls return `-32603` before reaching the executor.

```rust
// Source: verified from app/src/tests/mcp_write_dispatch.rs lines 270-275
let ctx = McpContext {
    tenant_id: Some(tenant_id),
    scope: Some("read_write".to_string()),
    write_authorized: Some(true),   // required for create_/update_/delete_ tools
    ..Default::default()
};
// For list_order (read), McpContext::default() is sufficient (no write_authorized check)
```

[VERIFIED: `write_dispatch.rs` lines 157-164 — `is_crud_write_tool && ctx.write_authorized != Some(true)` → deny]

### Pattern 2: handle_tools_call signature (with/without confirmation feature)

```rust
// Source: verified from app/src/tests/mcp_write_dispatch.rs lines 277-290
handle_tools_call(
    json!({ "name": tool_name, "arguments": arguments }),
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

### Pattern 3: WriteDispatcher for CRUD verbs

The existing `make_test_write_dispatcher` in `mcp_write_dispatch.rs` handles transition actions (uses `derive_transition_plan` to get `to_state`). CRUD verbs need a separate executor that handles `create_order`, `update_order`, `delete_order` by name and performs the corresponding SeaORM operation. The `CrudPlan` is passed via `dispatch_write`'s `Some(&plan)` arg — it is NOT threaded through the executor signature.

```rust
// Source: inferred from write_dispatch.rs lines 253-303 + mcp_write_dispatch.rs pattern
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
                        // status = "draft" server-side (D-05); tenant_id injected
                        let record = OrderActive {
                            customer_name: Set(inputs["customer_name"].as_str()
                                .unwrap_or("").into()),
                            total: Set(inputs["total"].as_f64().unwrap_or(0.0)),
                            status: Set("draft".into()),
                            tenant_id: Set(tenant_id),
                            created_at: Set(/* now */),
                            deleted_at: Set(None),
                            ..Default::default()
                        }.insert(&db).await
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
                        active.deleted_at = Set(Some("2026-06-24T00:00:00+00:00".into()));
                        active.update(&db).await
                            .map_err(|e| ferro::write::WriteError::Database(e.to_string()))?;
                        Ok(json!({ "id": id, "deleted": true }))
                    }
                    _ => Err(ferro::write::WriteError::ActionNotFound(action_name)),
                }
            })
        }),
        Box::new(|_, _, _, _| Box::pin(async { Ok(true) })), // no guards on CRUD
    )
}
```

### Pattern 4: CRUD success envelope shape (D-07)

From `write_dispatch.rs` lines 267-274, a successful CRUD call wraps the executor result in:
```json
{
  "result": {
    "content": [{ "type": "text", "text": "..." }],
    "isError": false,
    "structuredContent": {
      "status": "ok",
      "action": "create_order",
      "result": { "id": 5, "status": "draft" }
    }
  }
}
```

The D-07 regression guard assertion:
```rust
// Source: verified from mcp_tenant_isolation.rs lines 279-288 + write_dispatch.rs lines 267-274
let content = result["result"]["content"].as_array().expect("content is array");
assert_eq!(content[0]["type"].as_str(), Some("text"),
    "content[0] must be a text block (Phase 205 envelope shape)");
assert_eq!(result["result"]["structuredContent"]["status"].as_str(), Some("ok"),
    "{tool_name}: structuredContent.status must be ok");
assert_eq!(result["result"]["structuredContent"]["action"].as_str(), Some(tool_name));
assert!(result["result"]["structuredContent"]["result"].is_object());
assert_ne!(result["result"]["isError"], true);
```

### Pattern 5: Confirmation-required envelope shape for delete_order

From `write_dispatch.rs` lines 277-283:
```json
{
  "result": {
    "content": [{ "type": "text", "text": "use request_confirm_delete_order first" }],
    "isError": true,
    "structuredContent": {
      "error_kind": "confirmation_required",
      "message": "use request_confirm_delete_order first",
      "request_tool": "request_confirm_delete_order"
    }
  }
}
```

Tool names for the delete confirmation flow:
- Bare delete (triggers gate): `delete_order`
- Request token: `request_confirm_delete_order`
- Confirm and execute: `confirm_delete_order`

[VERIFIED: `write_dispatch.rs` lines 170-199 (prefix routing), lines 455-517 (`handle_request_confirm` delete path), lines 690-753 (`handle_confirm` delete path)]

### Pattern 6: MCP↔visual parity for CRUD

The visual parity call for CRUD mirrors `single_source.rs` `drive_visual` but uses `CrudPlan` instead of `TransitionPlan`:

```rust
// Source: inferred from single_source.rs lines 264-289 + write_dispatch.rs lines 253-268
use ferro::write::{dispatch_write, WriteResult};
use ferro_projections::{derive_crud_plan, ActionDef, CrudVerb};

async fn drive_visual_crud(
    verb: CrudVerb,
    tool_name: &str,
    inputs: Value,
    tenant_id: i64,
    db: &DatabaseConnection,
) -> WriteResult<Value> {
    let svc = order_service();
    let plan = derive_crud_plan(&svc, verb, &inputs).expect("derive_crud_plan");
    let crud_action = ActionDef::new(tool_name);
    let disp = make_crud_dispatcher(db.clone());
    dispatch_write(
        &crud_action,
        &inputs,
        tenant_id,
        db,
        &disp,
        None,   // transition_guard: CRUD has none
        "web",
        #[cfg(feature = "confirmation")]
        false,
        Some(&plan),
    )
    .await
}
```

Parity assertion: both MCP and visual paths produce the same persisted row, audit channel (`mcp.action.create_order` vs `web.action.create_order`) is the only divergence.

Delete parity is gated `#[cfg(not(feature = "confirmation"))]` — the feature-on path requires the two-step flow (following `single_source.rs` precedent for destructive actions).

### Pattern 7: Order projection flip (exact additions)

`app/src/projections/order.rs` currently:
```rust
ServiceDef::new("order")
    .mcp_exposed(true)
    .tenant_column("tenant_id")
    .mcp_ability("view-orders")
    .display_name("Order")
    // ... fields, state_machine, guards, actions ...
```

After flip (four additive lines, D-04):
```rust
ServiceDef::new("order")
    .mcp_exposed(true)
    .tenant_column("tenant_id")
    .mcp_ability("view-orders")
    .mcp_write_ability("manage-orders")  // ADD — gates create/update/delete tools
    .creatable(true)                     // ADD — derives create_order tool
    .updatable(true)                     // ADD — derives update_order tool
    .deletable(true)                     // ADD — derives delete_order tool (confirmation-gated)
    .display_name("Order")
    // ... rest unchanged ...
```

`deleted_at` is already present:
- Migration: `m20260623_add_deleted_at_to_orders` is in `Migrator::migrations()`. [VERIFIED: `app/src/migrations/mod.rs` line 16]
- Entity: `pub deleted_at: Option<String>` in `app/src/models/entities/orders.rs` line 20. [VERIFIED]

### Pattern 8: code_templates.rs additions

`code_templates.rs` has categories: handler, model, migration, middleware, validation, json_view, rate_limiting, broadcasting, api. No projection category exists. A new `projection_crud` category adds the CRUD opt-in template without breaking any existing category assertion tests (the `test_all_categories_present` test checks only the listed categories with `assert!(categories.contains("X"))` — it will not fail when an extra category is added).

The template content must show:
1. `.creatable(true).updatable(true).deletable(true).mcp_write_ability("...")` builder additions
2. The derived tool set: `create_<svc>`, `update_<svc>`, `delete_<svc>` (confirmation-gated), `list_<svc>`
3. The `deleted_at` requirement for soft-delete
4. The distinction between `mcp_ability` (read gate) and `mcp_write_ability` (write gate)

[VERIFIED: `code_templates.rs` lines 1637-1676 — test asserts specific categories; extra categories do not break it]

### Pattern 9: json-ui drift guard — confirmed safe

The builtin-component count is asserted at **47** in two locations:
- `ferro-json-ui/src/catalog.rs` line 1101: `assert_eq!(crate::render::BUILTIN_TYPES.len(), 47)`
- `ferro-mcp/src/tools/json_ui_catalog.rs` line 293: `assert_eq!(catalog.components.len(), 47, ...)`

CRUD MCP tools (`create_order`, `update_order`, `delete_order`) are MCP tools, not json-ui builtin components. Adding CRUD flags to a projection does NOT add any entry to `BUILTIN_TYPES` or `BUILTIN_SPECS`. Both drift guards stay at 47. [VERIFIED: D-10 confirmed by code reading]

### Anti-Patterns to Avoid

- **`write_authorized: None` in CRUD write tests:** Produces `-32603` before the executor runs. Always set `Some(true)` for the authorized case.
- **Passing `status` to `update_order`:** Excluded by `derive_crud_plan` when a StateMachine exists (CRUD-02). The test should never include `status` in update inputs.
- **Adding CRUD verb names to `svc.actions`:** `find_action` searches `svc.actions` — CRUD verbs bypass `find_action` entirely. Do not add `"create_order"` etc. to actions.
- **Touching `crud_operations.rs`:** Developer-MCP introspection tool (D-09 boundary). Not in scope.
- **Changing json-ui component counts:** CRUD tools are not components; counts stay at 47.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Confirmation token | Custom random | `ferro-ai::InMemoryConfirmationStore` + existing `generate_confirmation_token()` in `write_dispatch.rs` | Already defined with correct format (`cfm_` prefix + 43 BASE62 chars) |
| Envelope shape | New helper | Copy `content[0]["type"] == "text"` assertion from `mcp_tenant_isolation.rs` lines 279-305 | Regression-pinned — must stay identical |
| DB setup | New migration helper | `setup_db()` from `mcp_write_dispatch.rs` (copy verbatim) | Exact same `Migrator::up` needed |
| CRUD executor | Generic dispatcher | Concrete per-verb `match action_name` in `make_crud_dispatcher` | Explicit and readable; no abstraction saves meaningful work |

---

## Common Pitfalls

### Pitfall 1: write_authorized not set for CRUD tools
**What goes wrong:** `create_order`, `update_order`, `delete_order` return `-32603 authorization: write ability denied`.
**Why it happens:** `handle_write_call` lines 157-164 in `write_dispatch.rs`: `is_crud_write_tool && ctx.write_authorized != Some(true)` → transport-level error before the executor.
**How to avoid:** Set `write_authorized: Some(true)` in `McpContext` for all CRUD write calls.
**Warning signs:** All CRUD tool calls return `error.code == -32603` regardless of tenant or tool content.

### Pitfall 2: Confirmation feature gating compile errors
**What goes wrong:** `handle_tools_call` fails to compile — "expected N arguments, found M".
**Why it happens:** `handle_tools_call` takes `#[cfg(feature = "confirmation")]` conditional args (`store`, `config`). The test must mirror this exactly.
**How to avoid:** Follow `mcp_write_dispatch.rs` lines 277-290 precisely — use `#[cfg(feature = "confirmation")]` guards on all confirmation-related imports and call-site args.

### Pitfall 3: CRUD executor calls derive_transition_plan
**What goes wrong:** `ActionNotFound` errors when calling `create_order`, because the executor body tries to find the action in `exposed_services()` via `derive_transition_plan`.
**Why it happens:** The existing `make_test_write_dispatcher` in `mcp_write_dispatch.rs` derives `to_state` from `derive_transition_plan` — correct for transition actions, wrong for CRUD verbs.
**How to avoid:** Write a separate `make_crud_dispatcher` with a `match action_name` body that handles `"create_order"` / `"update_order"` / `"delete_order"` directly.

### Pitfall 4: Projection flip breaks a tool-count test
**What goes wrong:** Some test asserting `services[0].tools.len()` fails after adding three new CRUD tools.
**Why it happens:** The flip causes the `McpRenderer` to emit three new tools.
**How to avoid:** Grep for any test asserting a specific tool count for the order service before implementing the flip. [VERIFIED: No such test found in `mcp_tenant_isolation.rs`, `mcp_write_dispatch.rs`, `single_source.rs`, or `visual_action.rs`.]

### Pitfall 5: Seeding conflict — auto-increment IDs
**What goes wrong:** `create_order` in the e2e tries to INSERT, but the explicit-id seeding (orders 1–4) may conflict with auto-increment on some SQLite configurations.
**Why it happens:** SQLite auto-increment behavior after explicit inserts depends on MAX(id) + 1. With orders 1–4 seeded, the next auto-increment ID is 5.
**How to avoid:** The `create_order` executor should use `ActiveModel { ..Default::default() }` (id not set). After the INSERT, read back the new record's `id` from the executor result and use it for subsequent `update_order` / `delete_order` calls in the e2e cycle.

### Pitfall 6: update_order includes status
**What goes wrong:** `derive_crud_plan(svc, CrudVerb::Update, &args)` returns a validation error when `status` is in `args`.
**Why it happens:** When a StateMachine exists, `Status` is excluded from update inputs (CRUD-02). The plan derivation rejects it.
**How to avoid:** Never pass `status` to `update_order` in tests. Pass only `id`, `customer_name`, `total`.

---

## Code Examples

### Test file registration

```rust
// Source: verified from app/src/tests/mod.rs (current state: 6 entries)
// Add one line:
pub mod crud_e2e;
```

### Minimal e2e cycle

```rust
// Source: inferred from mcp_write_dispatch.rs + write_dispatch.rs patterns
#[tokio::test]
async fn crud_cycle_create_list_update_delete() {
    let db = setup_db().await;
    seed_two_tenants(&db).await; // tenants + users for is_manager guard

    let dispatcher = make_crud_dispatcher(db.clone());

    // Step 1: create_order
    let create_result = call_crud_tool(
        "create_order",
        json!({ "customer_name": "Test Customer", "total": 42.0 }),
        1, // tenant 1
        &db, &dispatcher, true,
    ).await;
    assert_write_envelope_ok(&create_result, "create_order");
    let new_id = create_result["result"]["structuredContent"]["result"]["id"]
        .as_i64().expect("created record must have id");

    // Step 2: list_order — must include the new record
    let list_result = call_list_tool("list_order", json!({ "limit": 100 }), 1, &db).await;
    let rows = list_result["result"]["structuredContent"]["rows"].as_array().expect("rows");
    assert!(rows.iter().any(|r| r["id"].as_i64() == Some(new_id)));

    // Step 3: update_order
    let update_result = call_crud_tool(
        "update_order",
        json!({ "id": new_id, "customer_name": "Updated Customer" }),
        1, &db, &dispatcher, true,
    ).await;
    assert_write_envelope_ok(&update_result, "update_order");

    // Step 4: delete_order (feature off: direct; feature on: via confirm flow)
    #[cfg(not(feature = "confirmation"))]
    {
        let delete_result = call_crud_tool(
            "delete_order", json!({ "id": new_id }), 1, &db, &dispatcher, true,
        ).await;
        assert_write_envelope_ok(&delete_result, "delete_order");

        // list_order must now exclude the soft-deleted record
        let list_after = call_list_tool("list_order", json!({ "limit": 100 }), 1, &db).await;
        let rows_after = list_after["result"]["structuredContent"]["rows"]
            .as_array().expect("rows after delete");
        assert!(!rows_after.iter().any(|r| r["id"].as_i64() == Some(new_id)));
    }
}
```

### Confirmation flow for delete_order

```rust
// Source: verified from write_dispatch.rs lines 277-283 (ConfirmationRequired) +
//         lines 489-516 (handle_request_confirm delete path) +
//         lines 690-753 (handle_confirm delete path)
#[cfg(feature = "confirmation")]
#[tokio::test]
async fn delete_order_confirmation_flow() {
    let db = setup_db().await;
    seed_two_tenants(&db).await;
    let dispatcher = make_crud_dispatcher(db.clone());

    // Create a record to delete
    let create_result = call_crud_tool("create_order",
        json!({ "customer_name": "To Delete", "total": 1.0 }),
        1, &db, &dispatcher, true).await;
    let target_id = create_result["result"]["structuredContent"]["result"]["id"].as_i64().unwrap();

    // Bare delete → confirmation_required
    let bare_delete = call_crud_tool("delete_order",
        json!({ "id": target_id }), 1, &db, &dispatcher, true).await;
    assert_eq!(bare_delete["result"]["structuredContent"]["error_kind"].as_str(),
        Some("confirmation_required"));
    assert_eq!(bare_delete["result"]["structuredContent"]["request_tool"].as_str(),
        Some("request_confirm_delete_order"));
    assert_eq!(bare_delete["result"]["isError"], true);

    // request_confirm_delete_order → token
    let store = ferro_ai::InMemoryConfirmationStore::new();
    let req_result = call_request_confirm("delete_order",
        json!({ "id": target_id }), 1, &db, &dispatcher, &store).await;
    let token = req_result["result"]["structuredContent"]["confirmation_token"]
        .as_str().expect("token").to_string();

    // confirm_delete_order → soft-delete
    let confirm_result = call_confirm("delete_order",
        json!({ "confirmation_token": token, "id": target_id }),
        1, &db, &dispatcher, &store).await;
    assert_write_envelope_ok(&confirm_result, "delete_order");

    // Record is soft-deleted; list_order no longer returns it
    let list_after = call_list_tool("list_order", json!({}), 1, &db).await;
    let rows = list_after["result"]["structuredContent"]["rows"].as_array().unwrap();
    assert!(!rows.iter().any(|r| r["id"].as_i64() == Some(target_id)));
}
```

---

## Runtime State Inventory

SKIPPED — this is an integration/docs phase, not a rename/refactor. No stored data strings are changed.

---

## Environment Availability

SKIPPED — all dependencies are in-workspace. No external tools required.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `#[tokio::test]` + `#[test]` (no separate config) |
| Config file | Workspace `Cargo.toml` |
| Quick run command | `cargo test -p app crud_e2e` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CRUD-01 | `create_order` inserts row, status="draft", returns ok envelope | in-process e2e | `cargo test -p app crud_e2e` | No — Wave 0 |
| CRUD-02 | `update_order` patches data fields, excludes status | in-process e2e | `cargo test -p app crud_e2e` | No — Wave 0 |
| CRUD-03 | `delete_order` soft-deletes (sets `deleted_at`), filtered from `list_order` after | in-process e2e | `cargo test -p app crud_e2e` | No — Wave 0 |
| CRUD-04 | `list_order` returns created record; excludes soft-deleted | in-process e2e | `cargo test -p app crud_e2e` | No — Wave 0 |
| CRUD-05 | `write_authorized: None` → -32603; `write_authorized: Some(true)` → success | in-process e2e | `cargo test -p app crud_e2e` | No — Wave 0 |
| CRUD-06 | MCP path + visual path (`dispatch_write(.., "web")`) → identical persisted state | parity e2e | `cargo test -p app crud_e2e` | No — Wave 0 |
| CRUD-07 | `order_service().validate()` passes after flip | unit | `cargo test -p app crud_e2e` | No — Wave 0 |
| D-07 | `content[0]["type"]=="text"` + `structuredContent.status=="ok"` for create/update/delete | regression | `cargo test -p app crud_e2e` | No — Wave 0 |
| SC#3 | bare `delete_order` → `confirmation_required`; confirm flow → soft-delete | feature-gated e2e | `cargo test -p app --features confirmation crud_e2e` | No — Wave 0 |
| SC#4 | `code_templates` projection_crud category present; `generation_context` updated | unit | `cargo test -p ferro-mcp` | Partial (framework exists; new content needs guard) |

### Sampling Rate
- **Per task commit:** `cargo test -p app crud_e2e` (after Wave 1: projection flip + test file)
- **Per wave merge:** Full suite gate (fmt + clippy `--all --all-targets -D warnings` + test `--all-features`)
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `app/src/tests/crud_e2e.rs` — new file covering CRUD-01..07, D-07, SC#3 (confirmation-gated)
- [ ] `app/src/tests/mod.rs` — add `pub mod crud_e2e;`
- [ ] `app/src/projections/order.rs` — projection flip (needed before tests compile against new tool set)
- [ ] `ferro-mcp/src/tools/code_templates.rs` — `projection_crud` category + a guard test
- [ ] `docs/src/features/projections.md` — "MCP CRUD Opt-In" section

No test framework installation needed — existing `tokio::test` infrastructure used throughout.

---

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | Yes | `derive_crud_plan` excludes `Identifier`/`CreatedAt`/`tenant_id`/`Status` from write inputs |
| V4 Access Control | Yes | `write_authorized: Some(true)` gate + `tenant_id` injected from auth context, never body |
| V6 Cryptography | Yes (delete confirmation) | `generate_confirmation_token()` uses `rand::thread_rng` + BASE62 (43 chars ~256-bit entropy) |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Cross-tenant write via `update_order`/`delete_order` | Spoofing | Executor filters `Column::TenantId.eq(tenant_id)` + `Column::DeletedAt.is_null()` — test with tenant 1 targeting tenant 2's order |
| Agent supplying `tenant_id` in `create_order` inputs | Spoofing | `tenant_id` excluded from write schema by `derive_crud_plan` (CRUD-05) |
| Agent supplying `status` in `create_order`/`update_order` | Tampering | `Status` excluded when StateMachine present (CRUD-01/02) |
| Replay of `confirm_delete_order` token | Replay | `InMemoryConfirmationStore::confirm` is single-use (tested in `write_dispatch.rs` SC#2) |
| Soft-deleted record appearing in list/update/delete after deletion | Elevation | Executor and list path filter `Column::DeletedAt.is_null()` |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `deleted_at` column type in entity is `Option<String>` (not `Option<DateTime>`) — executor uses string timestamps | Pattern 3 / CRUD executor | Test compile error if type differs; LOW risk |
| A2 | No existing test asserts a fixed tool count for the order service | Pitfall 4 | Three new tools (create/update/delete) would break such a test; LOW risk |

**A1** [VERIFIED: `app/src/models/entities/orders.rs` line 20 — `pub deleted_at: Option<String>`]
**A2** [VERIFIED: read all six test files in `app/src/tests/` — none assert a tool count for the order service]

The assumptions log is provided for completeness; both items are verified.

---

## Open Questions

1. **Should the CRUD parity assertion use a real DB write or just assert `WriteResult::Ok`?**
   - What we know: `single_source.rs` asserts the persisted state (reads back the DB row). The CRUD parity can do the same.
   - What's unclear: whether `create_order` on the visual channel needs a different seed to avoid id collision with the MCP channel's create.
   - Recommendation: Use separate orders for each channel (seed a new one for visual) and assert both have the same `status = "draft"` and the same `customer_name`.

2. **Does `list_order` already filter soft-deleted records?**
   - What we know: CRUD-03 states "filtered out of `list_<svc>`". This was shipped in Phase 239/240.
   - Recommendation: The e2e must include a post-delete list assertion to regression-pin this behavior. If the filter is absent, the test will catch it.

---

## Sources

### Primary (HIGH confidence)

- `ferro-mcp-server/src/write_dispatch.rs` — CRUD routing (lines 201-306), confirmation handlers (lines 427-827), `is_crud_write_tool` gate (lines 131-164), envelope shape (lines 267-274), error shapes (lines 56-72)
- `app/src/tests/mcp_write_dispatch.rs` — harness structure, `McpContext` shape (`write_authorized: Some(true)`), `WriteDispatcher` pattern, `#[cfg(feature = "confirmation")]` gating
- `app/src/tests/single_source.rs` — parity pattern: `drive_visual` calls `dispatch_write(.., "web")`, identical-state assertion, audit-channel-only divergence
- `app/src/tests/mcp_tenant_isolation.rs` — envelope assertion pattern (lines 278-305, 351-363): `content[0]["type"] == "text"`, `structuredContent.rows`
- `app/src/projections/order.rs` — current state: no CRUD flags; existing `.mcp_ability("view-orders")` + `.tenant_column("tenant_id")`
- `app/src/migrations/mod.rs` + `m20260623_add_deleted_at_to_orders.rs` — `deleted_at` migration present and in Migrator
- `app/src/models/entities/orders.rs` — `pub deleted_at: Option<String>` confirmed
- `ferro-mcp/src/tools/code_templates.rs` — existing categories (no `projection_crud`); test structure safe for new category addition
- `ferro-mcp/src/tools/generation_context.rs` — `CommonPatterns` struct; `crud_handler` field is the extension point
- `ferro-json-ui/src/catalog.rs` lines 1093-1101 — `builtin_types_count_drift_guard` asserts 47 components
- `ferro-mcp/src/tools/json_ui_catalog.rs` lines 285-297 — cross-crate mirror count 47
- `app/src/tests/mod.rs` — current module registrations; `crud_e2e` not yet present
- `app/src/controllers/visual_action.rs` lines 71-83 — `dispatch_write(.., "web")` call shape

### Secondary (MEDIUM confidence)

- `.planning/REQUIREMENTS.md` — CRUD-01..07 descriptions and traceability table
- `243-CONTEXT.md` — D-01..D-10 locked decisions (primary constraint source)
- `docs/src/features/projections.md` — current content (no CRUD section; confirmed by grep)

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries in workspace, verified by imports
- Architecture: HIGH — all patterns verified directly from source files
- Pitfalls: HIGH — derived from reading actual dispatch code paths
- Catalog/docs scope: HIGH — verified by reading code_templates.rs tests and docs content

**Research date:** 2026-06-24
**Valid until:** 2026-07-24 (workspace-internal, stable)
