# Projection Computed / Derived Fields — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A projection field declared read-only is excluded from `create_`/`update_` MCP input schemas and the write kernel, and a derived field's value (`order.total`) is recomputed server-side on every line-item write — demonstrated end-to-end against the sample app.

**Architecture:** Two parts. **Part A** (framework, `ferro-projections`): add one gate so `is_write_excluded_field` honors the existing `FieldDef.writable` flag. **Part C** (sample `app`, no framework change): redeclare `order.total` as a read-only field, add a real `line_item` CRUD projection, and register a post-persist recompute hook through the *existing* `WriteDispatcher::with_override` API so `order.total = SUM(line_items.amount)` stays correct on every write surface.

**Tech Stack:** Rust, SeaORM, sea-orm-migration, ferro-projections, ferro-mcp-server, ferro `framework::write` kernel, SQLite (sample app DB).

## Global Constraints

- Pre-commit gate (run before every commit): `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`. CI enforces `-D warnings`.
- No co-author / "Generated with" lines in commit messages.
- `ferro-projections` is schema-only: no closures, no runtime logic in `ServiceDef`/`FieldDef` (crate rule).
- The recompute hook is the application's responsibility (the *how*); the framework owns only the *when* (post-persist, via the existing override-hook registry). Do NOT add a new control surface to the kernel.
- WR-01 semantics: the recompute runs post-persist with no surrounding transaction. Treat the base persist as already durable when the hook fires.
- Sample app DB is SQLite; the `:8090` live drive uses `cargo run -p app --features confirmation`.

## Spec Deltas (conscious deviations from the design doc)

- **Per-field AX `description`** (spec Part C, point 1): DEFERRED. `FieldDef` has no `description` field today; adding one ripples into serde, `JsonSchema`, the protocol-schema export test, and every read-schema builder, for marginal value while the formula itself is not yet introspectable. The core AX signal — `total` absent from `create_`/`update_` schemas, present in `list_` — is still delivered. The human-readable "derived from line items" description rides with **Future Direction B** (introspectable formula), where it has real value. Surfaced at handoff.
- **`orders.total DEFAULT 0`**: SQLite cannot alter an existing column's default. The sample app resets via `db:fresh` and has no production data, so this plan edits the original `create_orders` migration to give `total` a default (Task 4) rather than adding a table-rebuild migration. A production app would model a derived column as defaulted/nullable from creation.

## File Structure

- `ferro-projections/src/service.rs` — add Gate F to `is_write_excluded_field` (+ unit test).
- `ferro-mcp-server/src/schema.rs` — add a test proving a read-only field is absent from create/update schemas (no logic change; the builders already delegate to `is_write_excluded_field`).
- `app/src/migrations/m20260624_create_line_items_table.rs` — NEW migration (line_items table).
- `app/src/migrations/m20260611_create_orders_table.rs` — MODIFY (`total` gets `.default(0.0)`).
- `app/src/migrations/mod.rs` — MODIFY (register the new migration).
- `app/src/models/entities/line_items.rs` — NEW SeaORM entity.
- `app/src/models/entities/mod.rs` — MODIFY (register `line_items`).
- `app/src/projections/line_item.rs` — NEW `line_item` ServiceDef.
- `app/src/projections/mod.rs` — MODIFY (register `line_item`).
- `app/src/projections/order.rs` — MODIFY (`total` → `read_only_field`).
- `app/src/controllers/mcp.rs` — MODIFY (`exposed_services()` adds `line_item`; `make_write_dispatcher()` registers the recompute hook).
- `app/src/tests/computed_total_e2e.rs` — NEW in-process e2e (the field test as a CI gate).
- `app/src/tests/mod.rs` — MODIFY (register the test module).

---

### Task 1: Gate F — `is_write_excluded_field` honors `writable`

**Files:**
- Modify: `ferro-projections/src/service.rs` (`is_write_excluded_field`, ~line 254)
- Test: `ferro-projections/src/service.rs` (`#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: `FieldDef { writable: bool, .. }` (existing), `ServiceDef::is_write_excluded_field(&self, field: &FieldDef, exclude_sm_status: bool) -> bool` (existing).
- Produces: `is_write_excluded_field` returns `true` for any field with `writable == false`.

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `ferro-projections/src/service.rs`:

```rust
#[test]
fn is_write_excluded_field_excludes_read_only() {
    let svc = ServiceDef::new("order")
        .field("customer_name", DataType::String, FieldMeaning::EntityName)
        .read_only_field("total", DataType::Float, FieldMeaning::Money);

    let writable = &svc.fields[0];
    let read_only = &svc.fields[1];

    // Writable data field is NOT excluded.
    assert!(!svc.is_write_excluded_field(writable, false));
    // Read-only (writable: false) field IS excluded from write input.
    assert!(svc.is_write_excluded_field(read_only, false));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p ferro-projections is_write_excluded_field_excludes_read_only -- --nocapture`
Expected: FAIL — the read-only assertion fails (`total` is currently not excluded, since no gate checks `writable`).

- [ ] **Step 3: Add Gate F**

In `ferro-projections/src/service.rs`, in `is_write_excluded_field`, add the gate immediately before the final `false`:

```rust
        // Gate F: read-only field — declared non-writable (e.g. read_only_field for a
        // derived/computed value like an order total). Never an agent write input;
        // the `writable` flag is the single source of truth for write eligibility.
        if !field.writable {
            return true;
        }
        false
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p ferro-projections is_write_excluded_field_excludes_read_only -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add ferro-projections/src/service.rs
git commit -m "feat(projections): exclude read-only fields from CRUD write input (Gate F)"
```

---

### Task 2: Schema-builder coverage — read-only field absent from create/update schemas

**Files:**
- Test: `ferro-mcp-server/src/schema.rs` (`#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: `build_create_input_schema(&ServiceDef) -> crate::Result<serde_json::Value>`, `build_update_input_schema(&ServiceDef) -> crate::Result<serde_json::Value>` (existing).
- Produces: regression guard that the `writable` flag governs the derived write schemas.

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `ferro-mcp-server/src/schema.rs` (the module already imports `ServiceDef`, `DataType`, `FieldMeaning` — mirror the existing tests' imports):

```rust
#[test]
fn read_only_field_absent_from_write_schemas() {
    let svc = ServiceDef::new("order")
        .field("customer_name", DataType::String, FieldMeaning::EntityName)
        .read_only_field("total", DataType::Float, FieldMeaning::Money);

    let create = build_create_input_schema(&svc).expect("create schema");
    let create_props = create["properties"].as_object().expect("create properties");
    assert!(create_props.contains_key("customer_name"));
    assert!(
        !create_props.contains_key("total"),
        "read-only `total` must not appear in create_order input"
    );

    let update = build_update_input_schema(&svc).expect("update schema");
    let update_props = update["properties"].as_object().expect("update properties");
    assert!(
        !update_props.contains_key("total"),
        "read-only `total` must not appear in update_order input"
    );
}
```

- [ ] **Step 2: Run test to verify it passes**

Run: `cargo test -p ferro-mcp-server read_only_field_absent_from_write_schemas -- --nocapture`
Expected: PASS (the builders already delegate to `is_write_excluded_field`, fixed in Task 1). If it FAILS, Task 1 is incomplete — fix there, not here.

- [ ] **Step 3: Commit**

```bash
git add ferro-mcp-server/src/schema.rs
git commit -m "test(mcp-server): pin read-only field exclusion from create/update schemas"
```

---

### Task 3: `line_items` table migration

**Files:**
- Create: `app/src/migrations/m20260624_create_line_items_table.rs`
- Modify: `app/src/migrations/mod.rs`

**Interfaces:**
- Produces: a `line_items` table with columns `id` (pk, autoincrement), `order_id` (bigint, not null), `amount` (double, not null), `tenant_id` (bigint, not null), `deleted_at` (timestamp, null).

- [ ] **Step 1: Create the migration file**

Create `app/src/migrations/m20260624_create_line_items_table.rs`:

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(LineItems::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LineItems::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(LineItems::OrderId).big_integer().not_null())
                    .col(ColumnDef::new(LineItems::Amount).double().not_null())
                    .col(ColumnDef::new(LineItems::TenantId).big_integer().not_null())
                    .col(ColumnDef::new(LineItems::DeletedAt).timestamp().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(LineItems::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum LineItems {
    Table,
    Id,
    OrderId,
    Amount,
    TenantId,
    DeletedAt,
}
```

- [ ] **Step 2: Register the migration**

In `app/src/migrations/mod.rs`, add the module declaration after the `m20260623_add_deleted_at_to_orders;` line:

```rust
mod m20260624_create_line_items_table;
```

And add to the `migrations()` vec, after the `m20260623_add_deleted_at_to_orders::Migration` entry:

```rust
            Box::new(m20260624_create_line_items_table::Migration),
```

- [ ] **Step 3: Verify it compiles and migrates**

Run: `cd app && cargo run -p app db:fresh 2>&1 | tail -2 && cd ..`
Expected: "Database refreshed successfully!" with no error.

Run: `sqlite3 app/database.db ".schema line_items"`
Expected: shows the `line_items` table with `order_id`, `amount`, `tenant_id`, `deleted_at`.

- [ ] **Step 4: Commit**

```bash
git add app/src/migrations/m20260624_create_line_items_table.rs app/src/migrations/mod.rs
git commit -m "feat(app): line_items table migration"
```

---

### Task 4: `orders.total` default — let create-without-total insert 0

**Files:**
- Modify: `app/src/migrations/m20260611_create_orders_table.rs`

**Interfaces:**
- Produces: `orders.total` column with a DB default of `0`, so an INSERT that omits `total` (because it is now read-only) succeeds.

- [ ] **Step 1: Add the default to the total column**

In `app/src/migrations/m20260611_create_orders_table.rs`, change the `total` column line:

```rust
                    .col(ColumnDef::new(Orders::Total).double().not_null())
```

to:

```rust
                    // Derived field: excluded from CRUD write input (read-only), so the
                    // INSERT omits it. Default 0 lets an order be created with no line
                    // items; the recompute hook updates it as line items are added.
                    .col(ColumnDef::new(Orders::Total).double().not_null().default(0.0))
```

- [ ] **Step 2: Verify it compiles and applies**

Run: `cd app && cargo run -p app db:fresh 2>&1 | tail -2 && cd ..`
Expected: "Database refreshed successfully!"

Run: `sqlite3 app/database.db ".schema orders" | tr ',' '\n' | grep -i total`
Expected: the `total` column shows `DEFAULT 0` (e.g. `"total" double NOT NULL DEFAULT 0`).

- [ ] **Step 3: Commit**

```bash
git add app/src/migrations/m20260611_create_orders_table.rs
git commit -m "feat(app): default orders.total to 0 (derived field, omitted on create)"
```

---

### Task 5: `line_items` SeaORM entity

**Files:**
- Create: `app/src/models/entities/line_items.rs`
- Modify: `app/src/models/entities/mod.rs`

**Interfaces:**
- Produces: `crate::models::entities::line_items::{Entity, Model, ActiveModel, Column}` with fields `id: i32`, `order_id: i64`, `amount: f64`, `tenant_id: i64`, `deleted_at: Option<String>`.

- [ ] **Step 1: Create the entity file**

Create `app/src/models/entities/line_items.rs` (mirrors `orders.rs` conventions — `String` timestamps, `FerroModel` derive):

```rust
use ferro::FerroModel;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, FerroModel)]
#[sea_orm(table_name = "line_items")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub order_id: i64,
    pub amount: f64,
    pub tenant_id: i64,
    #[sea_orm(column_name = "deleted_at")]
    pub deleted_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

- [ ] **Step 2: Register the entity module**

In `app/src/models/entities/mod.rs`, add (alphabetical order, after `pub mod api_keys;`):

```rust
pub mod line_items;
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p app 2>&1 | tail -3`
Expected: `Finished` with no errors.

- [ ] **Step 4: Commit**

```bash
git add app/src/models/entities/line_items.rs app/src/models/entities/mod.rs
git commit -m "feat(app): line_items SeaORM entity"
```

---

### Task 6: `line_item` projection + `order.total` read-only + expose

**Files:**
- Create: `app/src/projections/line_item.rs`
- Modify: `app/src/projections/mod.rs`
- Modify: `app/src/projections/order.rs` (one line)
- Modify: `app/src/controllers/mcp.rs` (`exposed_services()`)

**Interfaces:**
- Consumes: `ferro::{ServiceDef, DataType, FieldMeaning}`.
- Produces: `crate::projections::line_item::service_def() -> ServiceDef` (name `"line_item"`, creatable/updatable/deletable, `tenant_column("tenant_id")`, `soft_delete_column("deleted_at")`, `mcp_ability("view-orders")`, `mcp_write_ability("manage-orders")`, fields: `id` read-only Identifier, `order_id` ForeignKey, `amount` Money). `exposed_services()` returns `[order, line_item]`.

- [ ] **Step 1: Create the line_item projection**

Create `app/src/projections/line_item.rs`:

```rust
use ferro::{DataType, FieldMeaning, ServiceDef};

/// Build the LineItem service projection.
///
/// A child of `order`. CRUD-enabled so an agent can add/remove line items;
/// `order.total` is recomputed from these rows by the post-persist recompute
/// hook registered in `controllers::mcp::make_write_dispatcher`.
pub fn service_def() -> ServiceDef {
    ServiceDef::new("line_item")
        .mcp_exposed(true)
        .tenant_column("tenant_id") // server-side tenant injection + scoping (Phase 242)
        .mcp_ability("view-orders") // reuse the order read ability
        .mcp_write_ability("manage-orders") // reuse the order write gate (defined in bootstrap)
        .creatable(true)
        .updatable(true)
        .deletable(true)
        .soft_delete_column("deleted_at")
        .display_name("Line Item")
        .read_only_field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("order_id", DataType::Integer, FieldMeaning::ForeignKey)
        .field("amount", DataType::Float, FieldMeaning::Money)
}
```

- [ ] **Step 2: Register the projection module**

In `app/src/projections/mod.rs`, add:

```rust
pub mod line_item;
```

- [ ] **Step 3: Make `order.total` read-only**

In `app/src/projections/order.rs`, change:

```rust
        .field("total", DataType::Float, FieldMeaning::Money)
```

to:

```rust
        // Derived from line_items (SUM of amount); recomputed server-side by the
        // post-persist hook. Read-only → excluded from create_/update_order input.
        .read_only_field("total", DataType::Float, FieldMeaning::Money)
```

- [ ] **Step 4: Expose line_item**

In `app/src/controllers/mcp.rs`, change `exposed_services()`:

```rust
pub(crate) fn exposed_services() -> Vec<ServiceDef> {
    vec![crate::projections::order::service_def()]
}
```

to:

```rust
pub(crate) fn exposed_services() -> Vec<ServiceDef> {
    vec![
        crate::projections::order::service_def(),
        crate::projections::line_item::service_def(),
    ]
}
```

- [ ] **Step 5: Verify it compiles and validates at boot**

Run: `cargo build -p app 2>&1 | tail -3`
Expected: `Finished` with no errors.

Run: `cargo test -p app order_projection_validates_after_crud_flip 2>&1 | tail -3`
Expected: PASS (the existing boot-validate test; `manage-orders` present keeps `validate()` happy).

- [ ] **Step 6: Commit**

```bash
git add app/src/projections/line_item.rs app/src/projections/mod.rs app/src/projections/order.rs app/src/controllers/mcp.rs
git commit -m "feat(app): line_item CRUD projection; order.total read-only; expose line_item"
```

---

### Task 7: Recompute hook — `order.total = SUM(line_items.amount)`

**Files:**
- Modify: `app/src/controllers/mcp.rs` (`make_write_dispatcher()` + new helper `recompute_order_total_hook()`)

**Interfaces:**
- Consumes: `ferro::write::{WriteDispatcher, WriteError, OverrideFn}`, the `WriteDispatcher::with_override(action, hook)` builder.
- Produces: a recompute hook registered for `create_line_item`, `update_line_item`, `delete_line_item`. After each, `orders.total` for the affected `order_id` is set to `COALESCE(SUM(amount), 0)` over non-soft-deleted line items, tenant-scoped.

- [ ] **Step 1: Write the failing test (defer execution to Task 8)**

This task has no standalone unit test; its behavior is proven by the Task 8 e2e (`computed_total_*`). Implement the hook, then Task 8 exercises it. (Rationale: the hook only has observable effect through the full write→recompute→read path.)

- [ ] **Step 2: Add the recompute hook helper**

In `app/src/controllers/mcp.rs`, add this helper above `make_write_dispatcher`:

```rust
/// Post-persist recompute hook for `order.total` (Approach C — derived field).
///
/// Registered for `create_line_item` / `update_line_item` / `delete_line_item`.
/// Runs AFTER the line-item write is durable (WR-01: no surrounding transaction),
/// resolves the affected `order_id`, and sets `orders.total` to the live sum of
/// the order's non-soft-deleted line items. Tenant-scoped on both reads and the
/// update. Returns a freshly boxed closure per call so it can be registered for
/// multiple action names.
fn recompute_order_total_hook() -> ferro::write::OverrideFn {
    Box::new(|_action_name, inputs, tenant_id, db, base_result| {
        // Owned copies so the async block can move them (no 'static borrow trap).
        let inputs = inputs.clone();
        let base_result = base_result.clone();
        let db = db.clone();
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, Statement};

            // Resolve order_id. create/update return the full row (order_id present);
            // delete returns {"id","deleted":true}, so look it up by the line-item id
            // (the row is now soft-deleted but still present).
            let order_id: i64 = match base_result.get("order_id").and_then(|v| v.as_i64()) {
                Some(oid) => oid,
                None => {
                    let li_id = inputs.get("id").and_then(|v| {
                        v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
                    });
                    let li_id = li_id.ok_or_else(|| {
                        ferro::write::WriteError::Database(
                            "recompute: missing line item id".into(),
                        )
                    })?;
                    let row = db
                        .query_one(Statement::from_sql_and_values(
                            db.get_database_backend(),
                            "SELECT order_id FROM line_items WHERE id = ? AND tenant_id = ?",
                            [li_id.into(), tenant_id.into()],
                        ))
                        .await
                        .map_err(|e| ferro::write::WriteError::Database(e.to_string()))?
                        .ok_or_else(|| {
                            ferro::write::WriteError::Database(
                                "recompute: line item not found".into(),
                            )
                        })?;
                    row.try_get_by::<i64, _>("order_id")
                        .map_err(|e| ferro::write::WriteError::Database(e.to_string()))?
                }
            };

            // Recompute the parent total from live, non-deleted line items. Tenant-scoped.
            db.execute(Statement::from_sql_and_values(
                db.get_database_backend(),
                "UPDATE orders SET total = (\
                    SELECT COALESCE(SUM(amount), 0) FROM line_items \
                    WHERE order_id = ? AND deleted_at IS NULL\
                 ) WHERE id = ? AND tenant_id = ?",
                [order_id.into(), order_id.into(), tenant_id.into()],
            ))
            .await
            .map_err(|e| ferro::write::WriteError::Database(e.to_string()))?;

            Ok(())
        })
    })
}
```

- [ ] **Step 3: Register the hook on the three line-item write verbs**

In `app/src/controllers/mcp.rs`, change the end of `make_write_dispatcher` — wrap the returned `WriteDispatcher::new(...)` in the override chain. Replace:

```rust
    WriteDispatcher::new(
        Box::new(|action_name, inputs, tenant_id, db| {
```

with:

```rust
    WriteDispatcher::new(
```

…leaving the executor and guard closures unchanged, and change the final `)` that closes `WriteDispatcher::new(...)` so the dispatcher is built then extended. Concretely, the function currently ends:

```rust
        }),
    )
}
```

Change it to:

```rust
        }),
    )
    .with_override("create_line_item", recompute_order_total_hook())
    .with_override("update_line_item", recompute_order_total_hook())
    .with_override("delete_line_item", recompute_order_total_hook())
}
```

(Only the closing `)` of `WriteDispatcher::new(...)` gains the three chained `.with_override(...)` calls; the executor/guard closures are untouched.)

- [ ] **Step 4: Verify it compiles**

Run: `cargo build -p app --features confirmation 2>&1 | tail -3`
Expected: `Finished` with no errors.

- [ ] **Step 5: Commit**

```bash
git add app/src/controllers/mcp.rs
git commit -m "feat(app): recompute order.total from line_items via post-persist hook"
```

---

### Task 8: In-process e2e — derived total stays correct (field test as CI gate)

**Files:**
- Create: `app/src/tests/computed_total_e2e.rs`
- Modify: `app/src/tests/mod.rs`

**Interfaces:**
- Consumes: the in-process harness pattern from `app/src/tests/crud_e2e.rs`. Its helpers (`setup_db`, `seed_two_tenants`, `test_config`, `call_crud_tool`, `call_list_tool`) are **private to that module** — COPY them into the new module, do not import. Two mandatory adaptations when copying:
  1. The write helper must dispatch with the **real** `crate::controllers::mcp::make_write_dispatcher()` (it carries the recompute overrides) — not a fixture dispatcher.
  2. The `services` list must be the **real** `crate::controllers::mcp::exposed_services()` (order **and** line_item) — `crud_e2e.rs` hardcodes `vec![order_service()]`, which would omit `line_item` and silently bypass the hook.
  Both `make_write_dispatcher` and `exposed_services` are `pub(crate)` in `controllers/mcp.rs`, so they are reachable from `app/src/tests/`. The write `McpContext` keeps `write_authorized: Some(true)`, `scope: Some("read_write".into())`.
- Produces: a test module `computed_total_e2e` asserting: (a) `create_order` input schema has no `total`; (b) creating an order yields `total == 0`; (c) adding two line items makes `total == sum`; (d) deleting one line item updates `total`.

- [ ] **Step 1: Write the test module**

Create `app/src/tests/computed_total_e2e.rs`. Mirror the harness setup of `crud_e2e.rs` (same `setup_db`, migrations, seeding of one tenant + user, and the same `handle_tools_call` wiring with `make_write_dispatcher()` and `exposed_services()`). The assertions specific to this task:

```rust
#[cfg(test)]
mod tests {
    // Copy from crud_e2e.rs: setup_db(), seed_two_tenants(), test_config(). Define TWO
    // local helpers (adapted — see Interfaces):
    //   call_write(db, tool, args)  -> uses exposed_services() + make_write_dispatcher(),
    //                                  McpContext { tenant_id: Some(1), scope: read_write,
    //                                  write_authorized: Some(true), .. }
    //   call_read(db, tool, args)   -> uses exposed_services() + a noop dispatcher,
    //                                  McpContext { tenant_id: Some(1), .. }
    // Both call ferro_mcp_server::handle_tools_call (5/7-arg form per the confirmation feature,
    // exactly as crud_e2e.rs does).
    use ferro::serde_json::json;

    // 1) create_order input schema must not expose `total`.
    #[tokio::test]
    async fn create_order_schema_omits_derived_total() {
        let svc = crate::projections::order::service_def();
        let schema = ferro_mcp_server::schema::build_create_input_schema(&svc)
            .expect("create schema");
        let props = schema["properties"].as_object().expect("properties");
        assert!(props.contains_key("customer_name"));
        assert!(
            !props.contains_key("total"),
            "create_order must not accept a derived `total`"
        );
    }

    // 2)+3)+4) full derived-total lifecycle through the live kernel.
    #[tokio::test]
    async fn order_total_is_derived_from_line_items() {
        let db = setup_db().await; // copied from crud_e2e.rs
        seed_two_tenants(&db).await; // tenant 1 (acme) exists — orders.tenant_id FK

        // Create an order — no `total` supplied; server defaults it to 0.
        let created = call_write(&db, "create_order", json!({ "customer_name": "Mario Rossi" })).await;
        let order_id = created["structuredContent"]["result"]["id"].as_i64().expect("order id");
        assert_eq!(
            created["structuredContent"]["result"]["total"].as_f64(),
            Some(0.0),
            "new order total must be 0"
        );

        // Add two line items.
        call_write(&db, "create_line_item", json!({ "order_id": order_id, "amount": 10.0 })).await;
        call_write(&db, "create_line_item", json!({ "order_id": order_id, "amount": 5.5 })).await;

        // Read the order back — total must equal the sum (read-your-writes).
        let listed = call_read(&db, "list_order", json!({ "id": order_id })).await;
        let rows = listed["structuredContent"]["rows"].as_array().expect("rows");
        let total = rows[0]["total"].as_f64().expect("total");
        assert_eq!(total, 15.5, "order total must equal SUM(line_items.amount)");

        // Delete one line item — total must drop.
        let li_rows = call_read(&db, "list_line_item", json!({ "order_id": order_id })).await;
        let li_id = li_rows["structuredContent"]["rows"][0]["id"].as_i64().expect("line item id");
        call_write(&db, "delete_line_item", json!({ "id": li_id })).await;

        let after = call_read(&db, "list_order", json!({ "id": order_id })).await;
        let total_after = after["structuredContent"]["rows"][0]["total"].as_f64().expect("total");
        assert_eq!(
            total_after, 5.5,
            "deleting a line item must recompute the parent total"
        );
    }
}
```

Note for the implementer: copy `setup_db`, `seed_two_tenants`, and `test_config` from `crud_e2e.rs`. Define `call_write` and `call_read` locally (do NOT copy `call_crud_tool`/`call_list_tool` verbatim — they hardcode `vec![order_service()]`). Both must build `services = crate::controllers::mcp::exposed_services()`; `call_write` must use `crate::controllers::mcp::make_write_dispatcher()` as the dispatcher (carries the recompute hook) and an `McpContext` with `write_authorized: Some(true)`, `scope: Some("read_write".into())`, `tenant_id: Some(1)`. The live Gate path (`manage-orders`) is exercised separately by the `:8090` drive. `delete_line_item` here is a direct soft-delete — gate the lifecycle test `#[cfg(not(feature = "confirmation"))]`, matching `crud_e2e.rs`'s cycle test; the schema-omits-total test (`create_order_schema_omits_derived_total`) needs no feature gate.

- [ ] **Step 2: Register the test module**

In `app/src/tests/mod.rs`, add:

```rust
pub mod computed_total_e2e;
```

- [ ] **Step 3: Run to verify it fails first (if implemented before Tasks 5–7) or passes**

Run: `cargo test -p app computed_total -- --nocapture`
Expected: PASS once Tasks 3–7 are complete. If `order_total_is_derived_from_line_items` fails on the sum, inspect: hook registration names (`create_line_item` etc.), the recompute SQL, and that `line_item` is in `exposed_services()`.

- [ ] **Step 4: Commit**

```bash
git add app/src/tests/computed_total_e2e.rs app/src/tests/mod.rs
git commit -m "test(app): e2e — order.total derived from line_items, recomputed on writes"
```

---

### Task 9: Full gate + live `:8090` confirmation

**Files:** none (verification only)

- [ ] **Step 1: Run the full workspace gate**

Run: `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features 2>&1 | grep -iE "result: FAILED|[1-9][0-9]* failed" || echo "ALL GREEN"`
Expected: `ALL GREEN` (a pre-existing flaky `ferro-cli ... terminate_child_group_reaches_grandchild` may appear under full parallelism; re-run it in isolation with `cargo test -p ferro-cli --all-features --lib terminate_child_group` to confirm it passes alone).

- [ ] **Step 2: Live drive (optional, manual)**

```bash
bash /tmp/ferro-app-start.sh   # rebuilds + restarts :8090 with the new code
```

Then via the registered `mcp__ferro-app__*` tools (or `/tmp/ferro-mcp.sh`): create an order (confirm `total: 0`), `create_line_item` twice, `list_order` (confirm summed total), `delete_line_item`, `list_order` (confirm reduced total). Confirm `create_order`'s input schema no longer lists `total` (`/tmp/ferro-mcp.sh list-full`).

- [ ] **Step 3: No commit** (verification only).

---

## Self-Review

**Spec coverage:**
- Part A (Gate F) → Tasks 1–2. ✓
- Part C declaration (`order.total` read-only) → Task 6. ✓
- Part C recompute hook (existing override registry, synchronous) → Task 7. ✓
- Create-time / NOT NULL (`DEFAULT 0`) → Task 4. ✓
- Field test (`line_item` table + projection + drive) → Tasks 3, 5, 6, 8, 9. ✓
- AX per-field description → consciously DEFERRED (see Spec Deltas). ✓ (documented, not silently dropped)
- WR-01 semantics → encoded in Task 7 hook doc comment. ✓
- Future Direction B → unchanged in the spec; not built here. ✓

**Placeholder scan:** No "TBD"/"handle edge cases"/"similar to". Task 8 instructs copying concrete helpers from `crud_e2e.rs` (a real file) rather than restating ~100 lines of harness; this is a deliberate, located reference, not a placeholder.

**Type consistency:** `recompute_order_total_hook() -> ferro::write::OverrideFn`; `OverrideFn` signature matches `(&str, &Value, i64, &DatabaseConnection, &Value) -> Pin<Box<dyn Future<Output = WriteResult<()>>>>` (verified in `framework/src/write/mod.rs`). `with_override(action, hook)` matches the builder. Entity `Model` field types (`order_id: i64`, `amount: f64`, `tenant_id: i64`, `deleted_at: Option<String>`) match the migration columns. `read_only_field` builder exists with the used signature. `build_create_input_schema`/`build_update_input_schema` are `pub` in `ferro-mcp-server/src/schema.rs`.
