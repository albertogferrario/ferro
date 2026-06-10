# Phase 200: Per-Tenant Scoping, Policy Authorization & Dogfood Acceptance — Pattern Map

**Mapped:** 2026-06-10
**Files analyzed:** 14 new/modified files
**Analogs found:** 12 / 14

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-projections/src/service.rs` | model/schema | transform | self (extend `mcp_exposed` pattern) | exact |
| `ferro-mcp-server/src/dispatch.rs` | service | request-response / CRUD | self (extend where-clause loop) | exact |
| `ferro-mcp-server/src/jsonrpc.rs` | service | request-response | self (extend `handle_tools_call` call) | exact |
| `app/src/middleware/bearer_auth.rs` | middleware | request-response | `framework/src/api/api_key.rs` `ApiKeyMiddleware` | role-match |
| `app/src/tenant_resolver.rs` | resolver | request-response | `framework/src/tenant/resolver.rs` | role-match |
| `app/src/controllers/mcp.rs` | controller | request-response | self (extend Phase 199 seam) | exact |
| `app/src/migrations/m20260611_create_tenants_table.rs` | migration | CRUD | `app/src/migrations/m20260611_create_oauth_clients_table.rs` | exact |
| `app/src/migrations/m20260611_create_orders_table.rs` | migration | CRUD | `app/src/migrations/m20251208_160100_create_users_table.rs` | exact |
| `app/src/migrations/m20260611_add_tenant_id_to_users.rs` | migration | CRUD | `app/src/migrations/m20260611_create_oauth_clients_table.rs` | exact |
| `app/src/migrations/mod.rs` | config | — | self (extend Migrator list) | exact |
| `app/src/models/entities/tenants.rs` | model | CRUD | `app/src/models/entities/todos.rs` | exact |
| `app/src/models/entities/orders.rs` | model | CRUD | `app/src/models/entities/todos.rs` | exact |
| `app/src/models/tenants.rs` | model | CRUD | `app/src/models/todos.rs` | exact |
| `app/src/models/orders.rs` | model | CRUD | `app/src/models/users.rs` | exact |
| `app/src/models/mod.rs` | config | — | self (extend re-export list) | exact |
| `app/src/projections/order.rs` | service | request-response | self (extend builder chain) | exact |
| `app/src/routes.rs` | route | request-response | self (extend with middleware groups) | exact |
| `app/src/bootstrap.rs` | config | — | self (extend `register()`) | exact |
| `dogfood/run_dogfood.ts` | utility | request-response | none (no scripted MCP client exists) | no analog |
| `.planning/phases/200-.../200-ACCEPTANCE.md` | doc | — | none | no analog |

---

## Pattern Assignments

### `ferro-projections/src/service.rs` (schema, transform)

**Analog:** self — `mcp_exposed` field added in Phase 197 (lines 81–84, 99–100, 116–120)

**Existing `mcp_exposed` field pattern** (lines 81–84):
```rust
/// Whether this projection is exposed as an MCP tool.
/// Defaults to `false`. Only projections with `mcp_exposed: true`
/// appear in a `tools/list` response.
#[serde(default)]
pub mcp_exposed: bool,
```

**Existing struct init pattern** (lines 99–100):
```rust
mcp_exposed: false,
```

**Existing builder method pattern** (lines 116–120):
```rust
/// Marks this projection as MCP-exposed.
pub fn mcp_exposed(mut self, exposed: bool) -> Self {
    self.mcp_exposed = exposed;
    self
}
```

**Copy for `tenant_column` and `mcp_ability`:** add two `Option<String>` fields following the exact same `#[serde(skip_serializing_if = "Option::is_none")]` pattern already used by `display_name` and `description` (lines 65–68). Add builder methods following the `description()` method shape (lines 112–115).

```rust
// Field declarations — add after mcp_exposed (line 84)
#[serde(skip_serializing_if = "Option::is_none")]
pub tenant_column: Option<String>,
#[serde(skip_serializing_if = "Option::is_none")]
pub mcp_ability: Option<String>,

// Struct init — add after mcp_exposed: false (line 100)
tenant_column: None,
mcp_ability: None,

// Builder methods — add after mcp_exposed() (line 120)
/// Declares the FK column name used to scope reads to a tenant.
pub fn tenant_column(mut self, col: impl Into<String>) -> Self {
    self.tenant_column = Some(col.into());
    self
}

/// Declares the Gate ability required to call this projection via MCP.
pub fn mcp_ability(mut self, ability: impl Into<String>) -> Self {
    self.mcp_ability = Some(ability.into());
    self
}
```

**Serde test pattern** — copy from `mcp_exposed_defaults_false_when_absent` (lines 1272–1282):
```rust
#[test]
fn tenant_column_defaults_none_when_absent() {
    let json = r#"{"name":"order","fields":[]}"#;
    let parsed: ServiceDef = serde_json::from_str(json).unwrap();
    assert!(parsed.tenant_column.is_none());
    assert!(parsed.mcp_ability.is_none());
}
```

---

### `ferro-mcp-server/src/dispatch.rs` (service, request-response / CRUD)

**Analog:** self — extend the WHERE-clause build loop (lines 118–148)

**Existing signature** (lines 102–108):
```rust
pub async fn dispatch(
    service: &ServiceDef,
    filters: serde_json::Value,
    limit: u64,
    offset: u64,
    db: &sea_orm::DatabaseConnection,
) -> crate::Result<DispatchResult>
```

**New signature** — add `tenant_id: Option<i64>` as the last parameter. `ferro-mcp-server` has NO `framework` dependency (confirmed from Cargo.toml — deps are `ferro-projections`, `rmcp`, `serde`, `schemars`, `thiserror`, `tracing`, `sea-orm` only). The value is passed in by the app handler, not read from a task-local here.

```rust
pub async fn dispatch(
    service: &ServiceDef,
    filters: serde_json::Value,
    limit: u64,
    offset: u64,
    db: &sea_orm::DatabaseConnection,
    tenant_id: Option<i64>,   // NEW: passed from app handler via current_tenant()
) -> crate::Result<DispatchResult>
```

**Tenant predicate injection** — insert immediately after the user-filter loop (after line 141, before the `where_str` build at line 144). Copy the `where_clauses.push` / `values.push` / `idx += 1` pattern from the filter loop (lines 138–140):

```rust
// Tenant predicate — injected AFTER user filters, BEFORE count/data queries.
// Never sourced from call payload; always from current_tenant() passed by caller.
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

**Call sites to update simultaneously:**
- `ferro-mcp-server/src/jsonrpc.rs` line 82: `dispatch(service, filters, limit, offset, db).await` → add `tenant_id` argument
- All `dispatch` unit tests in `dispatch.rs`: pass `None` for non-tenant scenarios

---

### `ferro-mcp-server/src/jsonrpc.rs` (service, request-response)

**Analog:** self — extend `handle_tools_call` (lines 48–99)

**Current `handle_tools_call` signature** (lines 48–52):
```rust
pub async fn handle_tools_call(
    call_params: Value,
    services: &[ServiceDef],
    db: &sea_orm::DatabaseConnection,
) -> Value
```

**New signature** — add `tenant_id: Option<i64>`:
```rust
pub async fn handle_tools_call(
    call_params: Value,
    services: &[ServiceDef],
    db: &sea_orm::DatabaseConnection,
    tenant_id: Option<i64>,   // NEW: forwarded from app handler
) -> Value
```

**Dispatch call update** (line 82): `dispatch(service, filters, limit, offset, db, tenant_id).await`

**The `tools/call` arm in `app/src/controllers/mcp.rs`** must pass `tenant_id` from `ferro::current_tenant().map(|t| t.id)` at dispatch time (after `TenantMiddleware` has run).

---

### `app/src/middleware/bearer_auth.rs` (middleware, request-response) — NEW FILE

**Analog:** `framework/src/api/api_key.rs` `ApiKeyMiddleware` (lines 172–260) — same Middleware trait impl shape. Also see `app/src/middleware/auth.rs` for the minimal app middleware shape.

**Middleware trait impl shape** (from `app/src/middleware/auth.rs` lines 1–17):
```rust
use ferro::{async_trait, HttpResponse, Middleware, Next, Request, Response};

pub struct BearerAuthMiddleware {
    mcp_config: McpServerConfig,
}

#[async_trait]
impl Middleware for BearerAuthMiddleware {
    async fn handle(&self, mut request: Request, next: Next) -> Response {
        // ...
    }
}
```

**Key points from `ApiKeyMiddleware` pattern** (lines 198–260): `Request` is taken by value (not reference); middleware calls `next(request).await` to continue the chain.

**Full pattern for `BearerAuthMiddleware`** (from RESEARCH.md §Pattern 1):
```rust
use ferro::{async_trait, HttpResponse, Middleware, Next, Request, Response};
use ferro_mcp_oauth::{validate_bearer, BearerCheck, OAuthConfig};
use ferro_mcp_server::McpServerConfig;

pub struct BearerAuthMiddleware {
    pub mcp_config: McpServerConfig,
}

#[async_trait]
impl Middleware for BearerAuthMiddleware {
    async fn handle(&self, mut request: Request, next: Next) -> Response {
        let auth_header = request.header("Authorization").map(|s| s.to_owned());
        let oauth_config = OAuthConfig::from_env()
            .map_err(|_| challenge_response(&self.mcp_config))?;

        // expected_tenant: None — tenant validation is TenantMiddleware's job.
        // BearerAuthMiddleware runs BEFORE TenantMiddleware; current_tenant() is None here.
        match validate_bearer(auth_header.as_deref(), &oauth_config, None) {
            BearerCheck::Unauthenticated => Err(challenge_response(&self.mcp_config)),
            BearerCheck::Invalid => Err(HttpResponse::new()
                .status(401)
                .header("WWW-Authenticate", "Bearer error=\"invalid_token\"")),
            BearerCheck::Forbidden => Err(HttpResponse::new().status(403)),
            BearerCheck::Authenticated(principal) => {
                // Insert claims so JwtClaimResolver and the handler can read them.
                // Must use serde_json::Value — TypeId must match resolver.rs line 210.
                request.insert::<serde_json::Value>(principal);
                next(request).await
            }
        }
    }
}

fn challenge_response(config: &McpServerConfig) -> HttpResponse {
    let challenge = format!(
        "Bearer resource_metadata=\"{}/.well-known/oauth-protected-resource\"",
        config.app_url
    );
    HttpResponse::new()
        .status(401)
        .header("WWW-Authenticate", challenge)
}
```

**Critical:** `request.insert::<serde_json::Value>(principal)` — use exactly `serde_json::Value` as the type parameter. `JwtClaimResolver` reads `req.get::<serde_json::Value>()` (resolver.rs line 210). Any other type (`McpTokenClaims`, `Map<...>`) will silently return `None` because the extensions HashMap keys on `TypeId`.

---

### `app/src/controllers/mcp.rs` (controller, request-response)

**Analog:** self (Phase 199) — fill the seams left in Phase 199 (lines 52, 63–65, 82–90)

**Seam 1 — principal → extensions** (current line 63–65):
```rust
BearerCheck::Authenticated(_principal) => {
    // Phase 200 inserts principal into request extensions for JwtClaimResolver.
}
```
After Phase 200: this block is replaced. Bearer insertion now happens in `BearerAuthMiddleware`; the handler only needs to read `req.get::<serde_json::Value>()`.

**Seam 2 — Gate check before `handle_tools_call`** (current line 89):
```rust
handle_tools_call(params, &exposed_services(), db.inner()).await
```
Phase 200 replaces this with (RESEARCH.md §Pattern 4):
```rust
// Retrieve principal inserted by BearerAuthMiddleware upstream.
let principal = req.get::<serde_json::Value>()
    .ok_or_else(|| HttpResponse::new().status(401))?;
let user_id: i64 = principal["sub"].as_str()
    .and_then(|s| s.parse().ok())
    .ok_or_else(|| HttpResponse::new().status(400))?;

// On tools/call: load concrete User for Gate check (D-04 boundary).
let user = crate::models::User::find_by_id(user_id)
    .await
    .map_err(|e| HttpResponse::json(json!({ "jsonrpc":"2.0","id":id.clone(),"error":{"code":-32603,"message":e.to_string()} })))?
    .ok_or_else(|| HttpResponse::new().status(401))?;

// Fail-closed: mcp_ability = None → deny (D-04, D-06).
let ability = match service.mcp_ability.as_deref() {
    Some(a) => a,
    None => {
        let mut payload = json!({ "result": { "content": [{"type":"text","text":"Access denied. This resource requires an explicit ability declaration."}], "isError": true } });
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("jsonrpc".into(), json!("2.0"));
            obj.insert("id".into(), id.clone());
        }
        return Ok(HttpResponse::json(payload));
    }
};

match ferro::authorization::Gate::authorize_for(&user, ability, None) {
    Ok(()) => { /* proceed to dispatch */ }
    Err(_err) => {
        let mut payload = json!({ "result": { "content": [{"type":"text","text":"Access denied. You do not have permission to view this resource."}], "isError": true } });
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("jsonrpc".into(), json!("2.0"));
            obj.insert("id".into(), id.clone());
        }
        return Ok(HttpResponse::json(payload));
    }
}
```

**Seam 3 — tenant_id forwarded to dispatch** (RESEARCH.md §Pattern 3):
```rust
let tenant_id = ferro::current_tenant().map(|t| t.id);
// Pass to handle_tools_call which forwards to dispatch
handle_tools_call(params, &exposed_services(), db.inner(), tenant_id).await
```

**Policy-deny tool-error shape** (D-09): JSON-RPC success envelope with `isError: true`. Copy the envelope-splice pattern from lines 94–98:
```rust
// Envelope splice pattern (existing, lines 94-98):
if let Some(obj) = payload.as_object_mut() {
    obj.insert("jsonrpc".into(), json!("2.0"));
    obj.insert("id".into(), id);
}
Ok(HttpResponse::json(payload))
```

---

### `app/src/migrations/m20260611_create_tenants_table.rs` (migration, CRUD) — NEW FILE

**Analog:** `app/src/migrations/m20260611_create_oauth_clients_table.rs` (exact same date-format naming, `big_integer()` PK style, `timestamp_with_time_zone` + `Expr::current_timestamp()`)

**Full shape to copy** (`m20260611_create_oauth_clients_table.rs` lines 1–62):
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
                    .table(Tenants::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tenants::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Tenants::Slug).string().not_null())
                    .col(ColumnDef::new(Tenants::Name).string().not_null())
                    .col(
                        ColumnDef::new(Tenants::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tenants_slug_unique")
                    .table(Tenants::Table)
                    .col(Tenants::Slug)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tenants::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Tenants {
    Table,
    Id,
    Slug,
    Name,
    CreatedAt,
}
```

---

### `app/src/migrations/m20260611_create_orders_table.rs` (migration, CRUD) — NEW FILE

**Analog:** `app/src/migrations/m20251208_160100_create_users_table.rs` (FK column pattern via `ForeignKey::create()`)

**Column names are AUTHORITATIVE from the projection** (`app/src/projections/order.rs` lines 13–18):
- `id`, `customer_name`, `total`, `status`, `created_at`, `tenant_id`

**Table name:** `orders` — matches `format!("{}s", "order")` in dispatch (line 116)

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
                    .table(Orders::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Orders::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Orders::CustomerName).string().not_null())
                    .col(ColumnDef::new(Orders::Total).double().not_null())
                    .col(ColumnDef::new(Orders::Status).string().not_null())
                    .col(
                        ColumnDef::new(Orders::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Orders::TenantId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Orders::Table, Orders::TenantId)
                            .to(Tenants::Table, Tenants::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Orders::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    Id,
    CustomerName,
    Total,
    Status,
    CreatedAt,
    TenantId,
}

// Minimal IdenStatic for the FK target reference
#[derive(DeriveIden)]
enum Tenants {
    Table,
    Id,
}
```

---

### `app/src/migrations/m20260611_add_tenant_id_to_users.rs` (migration, CRUD) — NEW FILE

**Analog:** `app/src/migrations/m20260611_create_oauth_clients_table.rs` (same date prefix, `AlterTable` shape)

Adds `tenant_id INTEGER NULLABLE REFERENCES tenants(id)` to the `users` table to wire User → tenant association.

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

---

### `app/src/migrations/mod.rs` (config)

**Analog:** self — add three new migrations in chronological order after the existing four.

**Registration list shape** (current lines 1–20):
```rust
pub use sea_orm_migration::prelude::*;

mod m20251208_160100_create_users_table;
mod m20251208_200000_create_todos_table;
mod m20260228_create_api_keys_table;
mod m20260611_create_oauth_clients_table;
// ADD:
mod m20260611_create_tenants_table;
mod m20260611_add_tenant_id_to_users;
mod m20260611_create_orders_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251208_160100_create_users_table::Migration),
            Box::new(m20251208_200000_create_todos_table::Migration),
            Box::new(m20260228_create_api_keys_table::Migration),
            Box::new(m20260611_create_oauth_clients_table::Migration),
            // ADD (order matters — tenants first, then users alter, then orders):
            Box::new(m20260611_create_tenants_table::Migration),
            Box::new(m20260611_add_tenant_id_to_users::Migration),
            Box::new(m20260611_create_orders_table::Migration),
        ]
    }
}
```

---

### `app/src/models/entities/tenants.rs` (model, CRUD) — NEW FILE

**Analog:** `app/src/models/entities/todos.rs` (exact shape — auto-generated style, derive list, `#[sea_orm(table_name = ...)]`)

```rust
// AUTO-GENERATED FILE - DO NOT EDIT
// Generated by `ferro db:sync` - Changes will be overwritten
// Add custom code to src/models/tenants.rs instead

use ferro::FerroModel;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, FerroModel)]
#[sea_orm(table_name = "tenants")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub slug: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
```

---

### `app/src/models/entities/orders.rs` (model, CRUD) — NEW FILE

**Analog:** `app/src/models/entities/todos.rs`

**Column names must match projection field names** (`app/src/projections/order.rs` lines 13–18): `id`, `customer_name`, `total`, `status`, `created_at`. Add `tenant_id` as FK.

```rust
// AUTO-GENERATED FILE - DO NOT EDIT

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

---

### `app/src/models/tenants.rs` (model, CRUD) — NEW FILE

**Analog:** `app/src/models/todos.rs` (lines 1–47) — custom wrapper over entities, re-export pattern, type alias

```rust
//! Tenant model

pub use super::entities::tenants::*;
use sea_orm::ColumnTrait;

#[allow(dead_code)]
pub type Tenant = Model;

impl Model {
    pub async fn find_by_slug(slug: &str) -> Result<Option<Self>, ferro::FrameworkError> {
        Self::query().filter(Column::Slug.eq(slug)).first().await
    }

    pub async fn find_by_id(id: i64) -> Result<Option<Self>, ferro::FrameworkError> {
        Self::query().filter(Column::Id.eq(id)).first().await
    }
}
```

---

### `app/src/models/orders.rs` (model, CRUD) — NEW FILE

**Analog:** `app/src/models/todos.rs` (minimal wrapper) — no `Authenticatable` needed

```rust
//! Order model

pub use super::entities::orders::*;

#[allow(dead_code)]
pub type Order = Model;
```

---

### `app/src/models/mod.rs` (config)

**Analog:** self — add two new re-export lines after existing four (current lines 1–4):
```rust
pub mod api_key;
pub mod entities;
pub mod orders;    // ADD
pub mod tenants;   // ADD
pub mod todos;
pub mod users;
```

---

### `app/src/projections/order.rs` (service, request-response)

**Analog:** self — extend the builder chain (lines 10–45)

**Current `service_def()` builder chain** (line 11): starts with `ServiceDef::new("order").mcp_exposed(true)...`

**Add `tenant_column` and `mcp_ability`** immediately after `.mcp_exposed(true)` (consistent with Phase 197 pattern of adding metadata before field declarations):

```rust
pub fn service_def() -> ServiceDef {
    ServiceDef::new("order")
        .mcp_exposed(true)
        .tenant_column("tenant_id")   // ADD: FK column for dispatch predicate (D-02)
        .mcp_ability("view-orders")   // ADD: Gate ability name (D-04)
        .display_name("Order")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        // ... rest unchanged
```

---

### `app/src/routes.rs` (route, request-response)

**Analog:** self — extend with group middleware wrapping for `/authorize` and `/mcp`

**Existing group middleware pattern** (lines 23–25 and 43–44):
```rust
group!("/protected", {
    get!("/", controllers::home::index).name("protected.home"),
}).middleware(AuthMiddleware),
```

**Pattern for wrapping `/mcp` with ordered middleware stack:**

The middleware ordering must be `[BearerAuthMiddleware → TenantMiddleware]`. The framework's `group!` with `.middleware()` applies one middleware per call. Check whether the framework supports chaining `.middleware().middleware()` — follow the pattern used for the existing protected groups.

```rust
use ferro::{TenantFailureMode, TenantMiddleware};
use ferro_mcp_oauth::JwtClaimResolver;
use crate::middleware::bearer_auth::BearerAuthMiddleware;

// MCP endpoint — bearer auth → tenant scoping → handler
group!("/mcp-auth", {
    post!("/mcp", controllers::mcp::handle).name("mcp.endpoint"),
    get!("/mcp", controllers::mcp::method_not_allowed).name("mcp.endpoint.get"),
}).middleware(BearerAuthMiddleware { mcp_config: McpServerConfig::from_env() })
  .middleware(TenantMiddleware::new()
      .resolver(JwtClaimResolver::new("tenant_id", tenant_lookup.clone()))
      .on_failure(TenantFailureMode::Forbidden)),

// Authorization endpoint — tenant scoping for claim binding at authorize time
group!("/auth-tenant", {
    get!("/authorize", authorize_get),
    post!("/authorize", authorize_post),
}).middleware(TenantMiddleware::new()
      .resolver(/* SubdomainResolver or HeaderResolver — see D-07 open question */
          JwtClaimResolver::new("tenant_id", tenant_lookup.clone()))
      .on_failure(TenantFailureMode::Allow)),
```

**Note on `tenant_lookup`:** The `DbTenantLookup` instance needs to be constructed once and shared as `Arc<dyn TenantLookup>`. This is bootstrap-level state — construct it in `bootstrap::register()` and either pass it through or store as a singleton. See `DbTenantLookup::new` pattern in Shared Patterns below.

**Note on `/authorize` resolver:** The open question (RESEARCH.md §Open Question 1) is which resolver to use on `/authorize`. The cleanest for the dogfood is a `HeaderResolver("X-Tenant-Slug")` or a custom resolver that reads `tenant_id` from the authenticated user record. The route wiring pattern is identical regardless of resolver type — swap the resolver implementation.

---

### `app/src/bootstrap.rs` (config)

**Analog:** self — extend `register()` (lines 43–81)

**Gate::define pattern** (CLAUDE.md §Key Patterns):
```rust
use ferro::authorization::Gate;

// In register() after existing bindings:
Gate::define("view-orders", |user, _resource| {
    // All authenticated users may view orders for their tenant.
    // The tenant scoping is enforced by dispatch (D-02), not this callback.
    user.as_any()
        .downcast_ref::<crate::models::User>()
        .map(|_u| ferro::authorization::AuthResponse::allow())
        .unwrap_or_else(ferro::authorization::AuthResponse::deny_silent)
});
```

**Seed data insertion:** The app has no dedicated seeder mechanism — seed data is inserted via a migration or via application startup code. The cleanest pattern given the existing structure is to add a seed migration that inserts rows using raw SQL via `sea_orm::Statement`. Copy the `Statement::from_sql_and_values` pattern from dispatch. Alternatively, add an `if cfg!(debug_assertions)` block in bootstrap that inserts seed rows if the tenants table is empty.

**DbTenantLookup construction** (from `framework/src/tenant/lookup.rs` lines 68–95):
```rust
use ferro::{DbTenantLookup, TenantContext};
use std::sync::Arc;

let tenant_lookup: Arc<dyn ferro::TenantLookup> = Arc::new(DbTenantLookup::new(
    |slug| Box::pin(async move {
        crate::models::Tenant::find_by_slug(&slug)
            .await
            .ok()
            .flatten()
            .map(|t| TenantContext {
                id: t.id,
                slug: t.slug,
                name: t.name,
                plan: None,
            })
    }),
    |id| Box::pin(async move {
        crate::models::Tenant::find_by_id(id)
            .await
            .ok()
            .flatten()
            .map(|t| TenantContext {
                id: t.id,
                slug: t.slug,
                name: t.name,
                plan: None,
            })
    }),
));
```

This `tenant_lookup` Arc needs to be accessible to the route builder for `JwtClaimResolver::new("tenant_id", tenant_lookup.clone())`. Options: (a) store via `singleton!()` macro, (b) return from `bootstrap::register()`, (c) build at route-definition time. Prefer (a) or (b) — follow existing `bind!` / `singleton!` patterns in `bootstrap.rs`.

---

### `dogfood/run_dogfood.ts` (utility, request-response) — NEW FILE, no analog

No scripted MCP client exists in the codebase. RESEARCH.md confirms MCP SDK for Node/Python is unverified. The acceptance procedure document (`200-ACCEPTANCE.md`) must record the actual runner used.

**Minimum viable acceptance script structure** (no existing analog — reference MCP spec):
1. Discovery: `GET /.well-known/oauth-authorization-server`
2. Dynamic client registration: `POST /register`
3. Authorization flow: browser login (human step) → redirect with `code`
4. Token exchange: `POST /token`
5. `tools/list`: `POST /mcp` with Bearer token
6. `tools/call`: `POST /mcp` with `{"method":"tools/call","params":{"name":"list_order","arguments":{"limit":5,"offset":0}}}`
7. Verify: response contains `result.content` rows, all with `tenant_id` matching the authenticated tenant

---

## Shared Patterns

### Middleware Trait Implementation

**Source:** `framework/src/middleware/mod.rs` (trait definition) + `app/src/middleware/auth.rs` (minimal app middleware)

**Apply to:** `app/src/middleware/bearer_auth.rs`

```rust
// The Middleware trait — Request is passed by VALUE, not reference.
// Mutations to request (insert, header reads) happen on the owned Request.
#[async_trait]
impl Middleware for MyMiddleware {
    async fn handle(&self, mut request: Request, next: Next) -> Response {
        // Read headers BEFORE consuming body (body is single-read).
        // Insert into extensions: request.insert::<T>(value)
        // Continue chain: next(request).await
        // Short-circuit: return Err(HttpResponse::new().status(N))
    }
}
```

### Request Extensions Insert/Get

**Source:** `framework/src/http/request.rs` lines 87–93

**Apply to:** `app/src/middleware/bearer_auth.rs` (insert), `app/src/controllers/mcp.rs` (get)

```rust
// Insert: type parameter determines the TypeId key
request.insert::<serde_json::Value>(principal);

// Retrieve: must use EXACT same type as inserted
let principal = req.get::<serde_json::Value>();  // None if BearerAuthMiddleware didn't run
```

### TenantMiddleware Wiring

**Source:** `framework/src/tenant/middleware.rs` lines 37–58 + `framework/src/tenant/resolver.rs` lines 200–215

**Apply to:** `app/src/routes.rs`, `app/src/bootstrap.rs`

```rust
// Full wiring chain (verified pattern from framework/src/tenant/resolver.rs lines 209-211):
TenantMiddleware::new()
    .resolver(JwtClaimResolver::new("tenant_id", tenant_lookup.clone()))
    .on_failure(TenantFailureMode::Forbidden)

// JwtClaimResolver reads: req.get::<serde_json::Value>()["tenant_id"].as_i64()
// This ONLY works if BearerAuthMiddleware ran first and inserted the serde_json::Value claims.
```

### Gate::authorize_for (explicit user — NOT session-based)

**Source:** `framework/src/authorization/gate.rs` lines 172–189

**Apply to:** `app/src/controllers/mcp.rs` `tools/call` arm

```rust
// Use authorize_for (explicit user), NOT authorize (session-based).
// Gate::authorize checks Auth::id() from session context — not set in MCP path.
// Gate::authorize_for takes an explicit &dyn Authenticatable — correct for MCP.
match Gate::authorize_for(&user, "view-orders", None) {
    Ok(()) => { /* dispatch */ }
    Err(err) => {
        // err.message() carries the denial message (may be None for deny_silent).
        // Never log err fields that might contain data about the resource.
        /* return policy-deny tool error (D-09) */
    }
}
```

### SeaORM Migration Shape

**Source:** `app/src/migrations/m20251208_160100_create_users_table.rs` (integer PK) + `app/src/migrations/m20260611_create_oauth_clients_table.rs` (big_integer PK, timestamp_with_time_zone)

**Apply to:** all three new migrations

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Table::create()...).await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(T::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum T { Table, Id, /* columns */ }
```

### Policy-Deny Tool Error Shape (D-09)

**Source:** `app/src/controllers/mcp.rs` lines 94–98 (envelope splice pattern)

**Apply to:** `app/src/controllers/mcp.rs` `tools/call` deny path

```rust
// MCP tool error = JSON-RPC SUCCESS envelope with isError: true in result.
// NOT a transport-level 401/403 (those are for request-level auth failures).
// No rows, no column data, no filter values, no table name disclosed.
let mut payload = json!({
    "result": {
        "content": [{"type": "text", "text": "Access denied. You do not have permission to view this resource."}],
        "isError": true
    }
});
// Splice jsonrpc envelope (existing pattern, lines 94-98):
if let Some(obj) = payload.as_object_mut() {
    obj.insert("jsonrpc".into(), json!("2.0"));
    obj.insert("id".into(), id.clone());
}
return Ok(HttpResponse::json(payload));
```

### SeaORM Entity Shape

**Source:** `app/src/models/entities/todos.rs` (minimal entity) + `app/src/models/entities/users.rs` (with unique index)

**Apply to:** `app/src/models/entities/tenants.rs`, `app/src/models/entities/orders.rs`

```rust
use ferro::FerroModel;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, FerroModel)]
#[sea_orm(table_name = "table_name")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,  // or i64 for big_integer PK
    // ... columns matching migration exactly
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
```

---

## No Analog Found

| File | Role | Data Flow | Reason |
|---|---|---|---|
| `dogfood/run_dogfood.ts` (or `.py`) | utility | request-response | No scripted MCP client exists in the codebase; planner should use MCP SDK docs |
| `200-ACCEPTANCE.md` | doc | — | Phase acceptance artifact; no template exists |

---

## Implementation Ordering Notes (for Planner)

The dependency chain within this phase is strict:

1. `ferro-projections/src/service.rs` — add fields (`tenant_column`, `mcp_ability`) first; downstream crates read them
2. `ferro-mcp-server/src/dispatch.rs` + `ferro-mcp-server/src/jsonrpc.rs` — extend signature; all call sites must update simultaneously
3. App migrations (`tenants` → `add_tenant_id_to_users` → `orders`) — in this order; FK constraints require tenants to exist first
4. App models (`entities/` then custom wrappers) — after migrations
5. `app/src/middleware/bearer_auth.rs` — new file; no dependencies within this phase
6. `app/src/bootstrap.rs` — `Gate::define` + `DbTenantLookup` construction (needs Tenant model)
7. `app/src/routes.rs` — middleware wiring (needs `BearerAuthMiddleware` + `TenantMiddleware` + `tenant_lookup`)
8. `app/src/controllers/mcp.rs` — fill seams (needs `User` model, `Gate::authorize_for`, updated `handle_tools_call` signature)
9. `app/src/projections/order.rs` — add builder calls (needs new `ServiceDef` methods from step 1)
10. Dogfood script + `200-ACCEPTANCE.md` — after everything compiles and the app runs

---

## Metadata

**Analog search scope:** `ferro-projections/src/`, `ferro-mcp-server/src/`, `app/src/`, `framework/src/tenant/`, `framework/src/authorization/`, `framework/src/api/`, `ferro-mcp-oauth/src/`
**Files scanned:** 24
**Pattern extraction date:** 2026-06-10
