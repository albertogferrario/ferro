# Phase 239: Soft-delete data model + `deleted_at` migration - Pattern Map

**Mapped:** 2026-06-23
**Files analyzed:** 5 new/modified files
**Analogs found:** 5 / 5

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `app/src/migrations/m20260623_add_deleted_at_to_orders.rs` | migration | batch | `app/src/migrations/m20260611_add_tenant_id_to_users.rs` | exact |
| `app/src/migrations/mod.rs` | config | batch | existing `mod.rs` registration entries (same file) | exact |
| `ferro-projections/src/service.rs` | model/utility | request-response | existing `ServiceDef` accessor methods + `#[cfg(test)]` block (same file) | exact |
| `ferro-mcp-server/src/dispatch.rs` | service | request-response | tenant-predicate block at lines 151–167 + `setup_orders_db` test at lines 234–268 (same file) | exact |
| `app/src/models/entities/orders.rs` | model | CRUD | `app/src/models/entities/orders.rs` current state (same file, additive edit) | exact |

---

## Pattern Assignments

### `app/src/migrations/m20260623_add_deleted_at_to_orders.rs` (migration, batch)

**Analog:** `app/src/migrations/m20260611_add_tenant_id_to_users.rs`

**Full file pattern** (lines 1–35, the complete analog):
```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::TenantId).big_integer().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::TenantId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    TenantId,
}
```

**Adaptation for Phase 239** — substitute `Orders` for `Users`, `DeletedAt` for `TenantId`, and `.timestamp().null()` for `.big_integer().null()`:
```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .add_column(ColumnDef::new(Orders::DeletedAt).timestamp().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .drop_column(Orders::DeletedAt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    DeletedAt,
}
```

**Key notes:**
- `timestamp().null()` = backend-portable nullable timestamp on both SQLite and Postgres under sea-orm 1.0. Do NOT use `not_null()` or any default — SQLite only allows `ADD COLUMN` without `NOT NULL` constraint unless a constant default is provided. Nullable with no default is correct.
- `DeriveMigrationName` derives the version name from the file stem `m20260623_add_deleted_at_to_orders`. The date prefix `20260623` does not collide with any existing entry in `mod.rs`.
- The `DeriveIden` enum must only name the columns referenced in this migration (just `Table` and `DeletedAt`). Do not copy the full column list from the create migration.

---

### `app/src/migrations/mod.rs` (config, batch — registration only)

**Analog:** existing entries in the same file

**Current state of `mod.rs`** (lines 1–34, verbatim):
```rust
pub use sea_orm_migration::prelude::*;

mod m20251208_160100_create_users_table;
mod m20251208_200000_create_todos_table;
mod m20260228_create_api_keys_table;
mod m20260611_add_tenant_id_to_users;
mod m20260611_create_oauth_clients_table;
mod m20260611_create_orders_table;
mod m20260611_create_sessions_table;
mod m20260611_create_tenants_table;
// MCP write-dispatch tables: local wrappers give unique version names derived
// from the file stem, avoiding collisions with external crate "migration" stems.
mod m20260614_create_audit_log_table;
mod m20260614_create_mcp_idempotency_keys_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251208_160100_create_users_table::Migration),
            Box::new(m20251208_200000_create_todos_table::Migration),
            Box::new(m20260228_create_api_keys_table::Migration),
            Box::new(m20260611_create_oauth_clients_table::Migration),
            Box::new(m20260611_create_tenants_table::Migration),
            Box::new(m20260611_add_tenant_id_to_users::Migration),
            Box::new(m20260611_create_orders_table::Migration),
            Box::new(m20260611_create_sessions_table::Migration),
            Box::new(m20260614_create_mcp_idempotency_keys_table::Migration),
            Box::new(m20260614_create_audit_log_table::Migration),
        ]
    }
}
```

**Required edits** (two additions — append after the last `m20260614_*` entry):
1. Add `mod m20260623_add_deleted_at_to_orders;` at line 12 (after the last `mod` declaration).
2. Add `Box::new(m20260623_add_deleted_at_to_orders::Migration),` at the end of the `vec![]`, after `Box::new(m20260614_create_audit_log_table::Migration)`.

Chronological order is enforced by sea-orm's migration runner from the version name derived from the file stem — the `20260623` prefix sorts after all existing `20260614` entries.

---

### `ferro-projections/src/service.rs` (model/utility, request-response — add methods + tests)

**Analog for method structure:** existing consuming builder methods and `&self` accessor pattern in the same `impl ServiceDef` block (lines 116–421)

**Imports already present** (lines 1–10):
```rust
use std::collections::HashSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::action::{ActionDef, GuardDef};
use crate::field::{infer_meaning, DataType, FieldDef, FieldMeaning, RenderHint};
use crate::intent::IntentHint;
use crate::relationship::{Cardinality, RelationshipDef};
use crate::state::{StateMachine, Warning};
```

`FieldDef` and `FieldMeaning` are already imported — `is_server_injected_field` can use them directly.

**Existing builder shape to mirror** (lines 198–208, the `table` and `soft_delete_column` builders):
```rust
/// Declares the backing table for derived CRUD dispatch (field→column binding).
pub fn table(mut self, table: impl Into<String>) -> Self {
    self.table = Some(table.into());
    self
}

/// Declares the soft-delete column (defaults to `deleted_at` when unset).
pub fn soft_delete_column(mut self, col: impl Into<String>) -> Self {
    self.soft_delete_column = Some(col.into());
    self
}
```

**New methods to add** (insert after the `soft_delete_column` builder at line 208, still in `impl ServiceDef`):
```rust
/// Returns the backing table name: explicit `.table()` value or the
/// default `format!("{}s", name.to_lowercase())`.
///
/// Matches the inline derivation previously at dispatch.rs:123.
pub fn resolved_table(&self) -> String {
    self.table
        .clone()
        .unwrap_or_else(|| format!("{}s", self.name.to_lowercase()))
}

/// Returns the soft-delete column name: explicit `.soft_delete_column()` value
/// or the default `"deleted_at"`.
pub fn resolved_soft_delete_column(&self) -> &str {
    self.soft_delete_column.as_deref().unwrap_or("deleted_at")
}

/// Returns true if the field must be server-injected and never an agent input.
///
/// Covers:
/// - Identifier fields (primary key — set by DB auto-increment)
/// - CreatedAt fields (set by DB DEFAULT current_timestamp)
/// - The tenant column (injected from McpContext, never from agent payload)
///
/// Used by Phase 240 to derive write input schemas.
pub fn is_server_injected_field(&self, field: &FieldDef) -> bool {
    matches!(field.meaning, FieldMeaning::Identifier | FieldMeaning::CreatedAt)
        || self
            .tenant_column
            .as_deref()
            .map(|tc| tc == field.name)
            .unwrap_or(false)
}
```

**Existing test block shape** (lines 1917–2002, the Track A CRUD tests the new tests must follow):
```rust
// ── Track A: CRUD data-surface declaration ──────────────────────────────

#[test]
fn crud_flags_default_false() {
    let def = ServiceDef::new("order");
    assert!(!def.creatable);
    assert!(def.table.is_none());
    assert!(def.soft_delete_column.is_none());
}

#[test]
fn write_surface_builders_set_fields() {
    let def = ServiceDef::new("order")
        .mcp_write_ability("manage-orders")
        .table("orders")
        .soft_delete_column("deleted_at");
    assert_eq!(def.table.as_deref(), Some("orders"));
    assert_eq!(def.soft_delete_column.as_deref(), Some("deleted_at"));
}
```

**New tests to add** (append inside the existing `#[cfg(test)] mod tests` block, after line 2002):
```rust
// ── Phase 239: resolver accessors ───────────────────────────────────────

#[test]
fn resolved_table_default() {
    assert_eq!(ServiceDef::new("order").resolved_table(), "orders");
}

#[test]
fn resolved_table_default_lowercases() {
    assert_eq!(ServiceDef::new("Order").resolved_table(), "orders");
}

#[test]
fn resolved_table_explicit_override() {
    let def = ServiceDef::new("order").table("purchase_orders");
    assert_eq!(def.resolved_table(), "purchase_orders");
}

#[test]
fn resolved_soft_delete_column_default() {
    assert_eq!(
        ServiceDef::new("order").resolved_soft_delete_column(),
        "deleted_at"
    );
}

#[test]
fn resolved_soft_delete_column_explicit_override() {
    let def = ServiceDef::new("order").soft_delete_column("removed_at");
    assert_eq!(def.resolved_soft_delete_column(), "removed_at");
}

// ── Phase 239: is_server_injected_field ─────────────────────────────────

#[test]
fn server_injected_identifier() {
    let svc = ServiceDef::new("order");
    let f = FieldDef {
        name: "id".to_string(),
        data_type: DataType::Integer,
        meaning: FieldMeaning::Identifier,
        required: true,
        is_list: false,
        readable: true,
        writable: false,
        render_hint: None,
    };
    assert!(svc.is_server_injected_field(&f));
}

#[test]
fn server_injected_created_at() {
    let svc = ServiceDef::new("order");
    let f = FieldDef {
        name: "created_at".to_string(),
        data_type: DataType::String,
        meaning: FieldMeaning::CreatedAt,
        required: true,
        is_list: false,
        readable: true,
        writable: false,
        render_hint: None,
    };
    assert!(svc.is_server_injected_field(&f));
}

#[test]
fn server_injected_tenant_column() {
    let svc = ServiceDef::new("order").tenant_column("tenant_id");
    let f = FieldDef {
        name: "tenant_id".to_string(),
        data_type: DataType::Integer,
        meaning: FieldMeaning::ForeignKey,
        required: true,
        is_list: false,
        readable: true,
        writable: true,
        render_hint: None,
    };
    assert!(svc.is_server_injected_field(&f));
}

#[test]
fn server_injected_false_for_regular_field() {
    let svc = ServiceDef::new("order").tenant_column("tenant_id");
    let f = FieldDef {
        name: "customer_name".to_string(),
        data_type: DataType::String,
        meaning: FieldMeaning::EntityName,
        required: true,
        is_list: false,
        readable: true,
        writable: true,
        render_hint: None,
    };
    assert!(!svc.is_server_injected_field(&f));
}
```

---

### `ferro-mcp-server/src/dispatch.rs` (service, request-response — two changes + one test extension)

**Analog for predicate injection:** the tenant predicate block at lines 151–167 (verbatim):
```rust
// Tenant predicate — injected AFTER user filters, BEFORE count/data queries.
// Never sourced from the call payload; always from current_tenant() passed by caller.
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

**Change 1: Replace the inline table derivation** at line 122–123.

Current code (lines 122–123):
```rust
// TODO: ServiceDef.table field for irregular plurals / custom table names
let table = format!("{}s", service.name.to_lowercase());
```

Replace with:
```rust
let table = service.resolved_table();
```

No other change to surrounding code. `resolved_table()` returns `String` matching the type of the old expression.

**Change 2: Add the `deleted_at IS NULL` predicate block** immediately after the closing `}` of the tenant predicate block (after line 167), before line 169 (`let where_str = ...`):

```rust
// Soft-delete predicate — injected AFTER tenant predicate, BEFORE WHERE assembly.
// IS NULL has no bound value; idx is NOT incremented.
if service.soft_delete_column.is_some() {
    let col = service.resolved_soft_delete_column();
    where_clauses.push(format!("\"{}\" IS NULL", col));
    // No values.push() — IS NULL takes no bound parameter.
    // idx NOT incremented: LIMIT/OFFSET placeholders at lines 203–208 keep correct indices.
}
```

**Critical constraint:** Do NOT call `values.push()` and do NOT increment `idx` after this block. The LIMIT/OFFSET placeholders at lines 203–208 (`placeholder(backend, idx)` and `placeholder(backend, idx + 1)`) depend on `idx` counting only bound values. `IS NULL` has no bound value. Incrementing `idx` here would shift LIMIT/OFFSET to `$3/$4` on Postgres when they should be `$2/$3` (in a tenant-scoped + soft-delete query).

**Change 3: Extend the `setup_orders_db` test helper** — add `deleted_at TEXT NULL` column to the `CREATE TABLE` SQL and a soft-deleted seed row.

Current `setup_orders_db` (lines 234–269):
```rust
async fn setup_orders_db() -> sea_orm::DatabaseConnection {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("sqlite connect");

    // Create table
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "CREATE TABLE IF NOT EXISTS orders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            customer_name TEXT NOT NULL,
            total REAL NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            tenant_id INTEGER NOT NULL
        )"
        .to_string(),
    ))
    .await
    .expect("create table");

    // Seed rows: 2 rows for tenant 1, 2 rows for tenant 2
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "INSERT INTO orders (customer_name, total, status, tenant_id) VALUES
            ('Alice', 100.0, 'pending', 1),
            ('Bob',   200.0, 'shipped', 1),
            ('Carol', 150.0, 'pending', 2),
            ('Dave',  250.0, 'shipped', 2)"
            .to_string(),
    ))
    .await
    .expect("seed rows");

    db
}
```

**New test** to add inside `#[cfg(test)] mod tests` after the existing `non_tenant_unscoped` test (after line 363):
```rust
/// SC#3: soft-deleted row is excluded from dispatch results.
#[tokio::test]
async fn soft_delete_excluded() {
    // Use a separate in-memory DB to keep this test independent of setup_orders_db.
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("sqlite connect");

    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "CREATE TABLE IF NOT EXISTS orders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            customer_name TEXT NOT NULL,
            total REAL NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            tenant_id INTEGER NOT NULL,
            deleted_at TEXT NULL
        )"
        .to_string(),
    ))
    .await
    .expect("create table");

    // Seed: 1 active row, 1 soft-deleted row (same tenant).
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "INSERT INTO orders (customer_name, total, status, tenant_id, deleted_at) VALUES
            ('Alice', 100.0, 'pending', 1, NULL),
            ('Bob',   200.0, 'shipped', 1, '2026-06-23 12:00:00')"
            .to_string(),
    ))
    .await
    .expect("seed rows");

    let service = ServiceDef::new("order")
        .mcp_exposed(true)
        .soft_delete_column("deleted_at")
        .tenant_column("tenant_id")
        .mcp_ability("view-orders")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("customer_name", DataType::String, FieldMeaning::EntityName)
        .field("total", DataType::Float, FieldMeaning::Money)
        .field("status", DataType::String, FieldMeaning::Status)
        .field("created_at", DataType::String, FieldMeaning::CreatedAt)
        .field("tenant_id", DataType::Integer, FieldMeaning::ForeignKey);

    let result = dispatch(&service, serde_json::json!({}), 10, 0, &db, Some(1))
        .await
        .expect("dispatch ok");

    assert_eq!(
        result.rows.len(),
        1,
        "soft-deleted row must be excluded; only 1 active row"
    );
    assert_eq!(
        result.rows[0]["customer_name"],
        serde_json::Value::String("Alice".to_string())
    );
    assert_eq!(result.total, 1, "total count must also exclude soft-deleted row");
}
```

**Note:** The `setup_orders_db` helper itself does NOT need to be modified. It lacks `deleted_at` and is correct for the existing tenant tests which don't use soft-delete. The new `soft_delete_excluded` test uses its own local setup to avoid coupling.

---

### `app/src/models/entities/orders.rs` (model, CRUD — additive field addition)

**Analog:** current file state (lines 1–23):
```rust
// AUTO-GENERATED FILE - DO NOT EDIT
// Generated by `ferro db:sync` - Changes will be overwritten
// Add custom code to src/models/orders.rs instead

use ferro::FerroModel;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, FerroModel)]
#[sea_orm(table_name = "orders")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub customer_name: String,
    pub total: f64,
    pub status: String,
    pub created_at: String,
    pub tenant_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
```

**Required addition:** After running the migration, the `Model` struct must include `deleted_at`. Match the type of `created_at: String` but nullable — `Option<String>`:

```rust
pub deleted_at: Option<String>,
```

Insert as the last field in `Model` (after `tenant_id: i64`). Sea-orm entity field order matches column order in the table, and the new column is added after existing columns by `ALTER TABLE ADD COLUMN`.

**Preferred approach:** Run `cargo run --bin app -- db:sync` after migrating to regenerate the file, which will add the field automatically. If running `db:sync` is not available in the execution environment, add the field manually with the `#[sea_orm(column_name = "deleted_at")]` attribute for clarity:
```rust
#[sea_orm(column_name = "deleted_at")]
pub deleted_at: Option<String>,
```

Note the file header warns "DO NOT EDIT / will be overwritten" — this is safe to edit now since `db:sync` would produce the same result, and the migration gate (SC#1) needs this to compile before any ORM code references `deleted_at`.

---

## Shared Patterns

### Sea-orm Migration Boilerplate
**Source:** `app/src/migrations/m20260611_add_tenant_id_to_users.rs` (complete file)
**Apply to:** `m20260623_add_deleted_at_to_orders.rs`

Three required pieces every additive migration needs:
1. `use sea_orm_migration::prelude::*;` — the only import needed
2. `#[derive(DeriveMigrationName)] pub struct Migration;` — derives version name from file stem
3. `#[derive(DeriveIden)] enum <Table> { Table, <Column> }` — local IdenStatic for type-safe column references

### Parameterized SQL Assembly
**Source:** `ferro-mcp-server/src/dispatch.rs` lines 125–148 (filter loop), 151–167 (tenant block), 169–173 (where_str), 176–177 (count), 203–212 (data)
**Apply to:** The new soft-delete predicate block at lines ~168–174 (inserted)

The invariant for the `idx` counter: `idx` is incremented exactly once per `values.push()` call. The `IS NULL` predicate has zero bound values → zero increments. Any deviation shifts LIMIT/OFFSET indices off by one on Postgres.

### `impl ServiceDef` Resolver Method Shape
**Source:** `ferro-projections/src/service.rs` lines 198–208 (`table` and `soft_delete_column` builders)
**Apply to:** `resolved_table()`, `resolved_soft_delete_column()`, `is_server_injected_field()` (all in the same `impl ServiceDef` block)

The pattern: `&self` methods that dereference `Option<String>` fields with `.as_deref().unwrap_or(...)` or `.clone().unwrap_or_else(|| ...)`. No allocation when returning a `'static` literal; clone when constructing a default `String`.

### Test Infrastructure (dispatch.rs)
**Source:** `ferro-mcp-server/src/dispatch.rs` lines 226–364
**Apply to:** The new `soft_delete_excluded` test

The test pattern: `use super::*; use ferro_projections::{DataType, FieldMeaning, ServiceDef}; use sea_orm::{ConnectionTrait, Database, Statement};` already imported at line 228–230. New tests reuse those imports; no new `use` statements needed inside `mod tests`.

The `ServiceDef` builder for tests uses `.mcp_exposed(true)`, then field declarations using the `DataType`/`FieldMeaning` pair pattern seen at lines 271–282.

---

## No Analog Found

All 5 files have close analogs in the codebase. No entries in this section.

---

## Metadata

**Analog search scope:** `app/src/migrations/`, `ferro-projections/src/`, `ferro-mcp-server/src/`, `app/src/models/entities/`
**Files read:** 8
**Pattern extraction date:** 2026-06-23
