# Phase 217: Tenant Context + Per-Tenant API-Key Auth - Pattern Map

**Mapped:** 2026-06-13
**Files analyzed:** 9 new/modified files
**Analogs found:** 9 / 9

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-mcp-oauth/src/validate.rs` | validator (MODIFY) | request-response | Same file — `validate_bearer` function (lines 53-98) | exact — new function parallels existing |
| `ferro-mcp-oauth/src/migration.rs` | migration (MODIFY) | CRUD | Same file — `CreateOauthClientsTable` (lines 19-77) | exact — new migration alongside existing |
| `ferro-mcp-server/src/auth.rs` | auth unifier (REPLACE) | request-response | `ferro-mcp-oauth/src/validate.rs` `validate_bearer` delegator pattern | role-match |
| `ferro-mcp-server/src/renderer.rs` | context struct (MODIFY) | — | `framework/src/api/api_key.rs` `ApiKeyInfo` struct + `ferro_projections::BaseContext` shape | role-match |
| `ferro-mcp-server/src/error.rs` | error enum (MODIFY) | — | Same file — existing `Error` enum with `thiserror` (lines 1-18) | exact — new variant added |
| `ferro-mcp-server/src/jsonrpc.rs` | RPC handler (MODIFY) | request-response | Same file — `handle_tools_call` error mapping pattern (lines 100-106) | exact — same file |
| `ferro-mcp-server/Cargo.toml` | config (MODIFY) | — | Same file line 14: `ferro-projections = { path = "../ferro-projections", version = "0.2" }` | exact — same pattern |
| `ferro-mcp-server/tests/mcp_tenant_isolation.rs` | integration test (CREATE) | request-response | `ferro-mcp-server/tests/dispatch_integration.rs` + `ferro-mcp-server/tests/common/mod.rs` | exact — in-process SQLite fixture |
| `ferro-mcp-server/tests/jsonrpc_integration.rs` | integration test (MODIFY) | — | Same file — `handle_tools_list` call sites (lines 37, current signature) | exact — same file |

---

## Pattern Assignments

### `ferro-mcp-oauth/src/validate.rs` — add `validate_api_key` + `generate_mcp_api_key` (MODIFY)

**Analog:** Same file, `validate_bearer` function (lines 53-98) and `framework/src/api/api_key.rs` hashing primitives (lines 114-150).

**Imports pattern to add** (mirror existing imports at lines 25-26):
```rust
use sha2::{Digest, Sha256};
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement, DatabaseBackend};
```

**Core validator pattern** — mirror `validate_bearer` (lines 53-98) exactly; only the body differs:
```rust
// EXISTING (lines 53-57) — signature to parallel:
pub fn validate_bearer(
    authorization_header: Option<&str>,
    config: &OAuthConfig,
    expected_tenant: Option<i64>,
) -> BearerCheck

// NEW signature (async because DB lookup):
pub async fn validate_api_key(
    authorization_header: Option<&str>,
    db: &DatabaseConnection,
    expected_tenant: Option<i64>,
) -> BearerCheck
```

**Bearer-prefix strip pattern** (lines 59-66) — copy verbatim, then add `ferro_` prefix guard:
```rust
// Step 1: header presence + Bearer prefix (identical to validate_bearer lines 59-66)
let header = match authorization_header {
    None => return BearerCheck::Unauthenticated,
    Some(h) => h,
};
let token = match header.strip_prefix("Bearer ") {
    None | Some("") => return BearerCheck::Unauthenticated,
    Some(t) => t,
};
// Step 2: ferro_ prefix guard (defensive — caller should have routed here already)
if !token.starts_with("ferro_") {
    return BearerCheck::Unauthenticated;
}
```

**Authenticated principal shape** (lines 94-97) — must match exactly, add `scope`:
```rust
// EXISTING (lines 94-97):
BearerCheck::Authenticated(serde_json::json!({
    "sub": claims.sub,
    "tenant_id": claims.tenant_id,
}))

// NEW (include scope from DB row):
BearerCheck::Authenticated(serde_json::json!({
    "sub": row_id.to_string(),     // key row id as sub; no OAuth user sub
    "tenant_id": row_tenant_id,    // i64
    "scope": row_scope,            // "read" | "read_write"
}))
```

**Key generator** — mirror `framework/src/api/api_key.rs` lines 114-139 exactly (same BASE62, same `rand::thread_rng()`, same SHA-256 hex pattern):
```rust
// FROM framework/src/api/api_key.rs lines 42, 114-139 (exact pattern to copy):
const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub fn generate_mcp_api_key() -> (String, String) {
    let mut rng = rand::thread_rng();
    let random: String = (0..43)
        .map(|_| {
            let idx = rand::Rng::gen_range(&mut rng, 0..62);
            BASE62[idx] as char
        })
        .collect();
    let raw_key = format!("ferro_{random}");
    let key_hash = {
        let mut h = Sha256::new();
        h.update(raw_key.as_bytes());
        format!("{:x}", h.finalize())
    };
    (raw_key, key_hash)
}
```

**SHA-256 hash helper** — mirror `framework/src/api/api_key.rs` lines 136-140:
```rust
// Copy pattern from framework/src/api/api_key.rs:136-140
pub fn hash_mcp_api_key(raw_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_key.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

**DB lookup pattern** (SeaORM raw SQL — consistent with how `dispatch.rs` and `migration.rs` tests use raw SQL):
```rust
// Use Statement::from_sql_and_values (same pattern as dispatch.rs lines 177, 212)
let key_hash = hash_mcp_api_key(token);
let stmt = Statement::from_sql_and_values(
    db.get_database_backend(),
    "SELECT id, tenant_id, scope, revoked_at FROM mcp_api_keys WHERE key_hash = ?",
    [sea_orm::Value::String(Some(Box::new(key_hash)))],
);
let row = db.query_one(stmt).await ...;
```

**Test pattern** (lines 100-219) — follow exact same structure: `fn config()` helper, `fn bearer()` helper, one `#[test]` per behavior:
```rust
// mirror the test module structure from validate.rs lines 100-219:
#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_api_keys_db() -> sea_orm::DatabaseConnection { ... }

    #[tokio::test]
    async fn valid_api_key_returns_authenticated() { ... }

    #[tokio::test]
    async fn invalid_api_key_returns_invalid() { ... }

    #[tokio::test]
    async fn revoked_api_key_returns_invalid() { ... }

    #[tokio::test]
    async fn wrong_tenant_returns_forbidden() { ... }
}
```

---

### `ferro-mcp-oauth/src/migration.rs` — add `CreateMcpApiKeysTable` migration (MODIFY)

**Analog:** Same file, `Migration` struct for `CreateOauthClientsTable` (lines 19-77).

**Migration struct pattern** (lines 18-77) — copy the entire shape; only the `up`/`down` body and the `DeriveIden` enum differ:
```rust
// FROM ferro-mcp-oauth/src/migration.rs lines 18-77 — exact structural pattern:
#[derive(DeriveMigrationName)]
pub struct MigrationMcpApiKeys;  // new struct name

#[async_trait::async_trait]
impl MigrationTrait for MigrationMcpApiKeys {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table( ... ).await?;
        manager.create_index( ... ).await  // idx on key_hash (unique) + idx on tenant_id
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(McpApiKeys::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum McpApiKeys {
    Table,
    Id,
    TenantId,
    KeyHash,
    Scope,
    RevokedAt,
    CreatedAt,
    UpdatedAt,
}
```

**Column types** — follow the `OauthClients` column pattern (lines 31-46): `big_integer().auto_increment().primary_key()` for id; `string().not_null()` for text columns; `timestamp_with_time_zone()` for timestamps where Postgres compat is needed (or `string()` for SQLite-first apps):
```rust
// FROM migration.rs lines 31-46 — column definition pattern to follow:
.col(
    ColumnDef::new(McpApiKeys::Id)
        .big_integer()
        .not_null()
        .auto_increment()
        .primary_key(),
)
.col(ColumnDef::new(McpApiKeys::TenantId).big_integer().not_null())
.col(ColumnDef::new(McpApiKeys::KeyHash).string().not_null())
.col(ColumnDef::new(McpApiKeys::Scope).string().not_null().default("read"))
.col(ColumnDef::new(McpApiKeys::RevokedAt).timestamp_with_time_zone().null())
.col(
    ColumnDef::new(McpApiKeys::CreatedAt)
        .timestamp_with_time_zone()
        .not_null()
        .default(Expr::current_timestamp()),
)
```

**Unique index pattern** (lines 51-60):
```rust
// FROM migration.rs lines 51-60 — exact index pattern:
manager.create_index(
    Index::create()
        .name("idx_mcp_api_keys_key_hash")
        .table(McpApiKeys::Table)
        .col(McpApiKeys::KeyHash)
        .unique()   // key_hash must be unique
        .to_owned(),
).await?;
// Second index on tenant_id (not unique):
manager.create_index(
    Index::create()
        .name("idx_mcp_api_keys_tenant_id")
        .table(McpApiKeys::Table)
        .col(McpApiKeys::TenantId)
        .to_owned(),
).await
```

**Migration test pattern** (lines 79-149) — copy `TestMigrator` struct + `migration_creates_table_and_index` test verbatim, changing table name and index name strings. The `sqlite_master` query pattern at lines 104-130 is the verification idiom to follow exactly.

**lib.rs export** — mirror line 26: `pub use migration::Migration as CreateOauthClientsTable;` → add:
```rust
pub use migration::MigrationMcpApiKeys as CreateMcpApiKeysTable;
```

---

### `ferro-mcp-server/src/auth.rs` — replace `BearerOutcome` stub with `resolve_tenant` (REPLACE)

**Analog:** `ferro-mcp-oauth/src/validate.rs` line 53 (`validate_bearer` signature); the existing stub file (lines 1-10) is entirely replaced.

**Module doc pattern** — mirror `validate.rs` line 1-24 style (validation steps numbered, HTTP-status table):
```rust
//! Auth unifier for the MCP endpoint (Phase 217).
//!
//! `resolve_tenant` branches on token shape and delegates to:
//! - `ferro_mcp_oauth::validate_api_key` — `ferro_`-prefixed tokens (DB lookup)
//! - `ferro_mcp_oauth::validate_bearer` — JWT tokens (sync decode)
//!
//! Both paths return `BearerCheck` from `ferro-mcp-oauth`.
```

**Import pattern** (requires the new Cargo.toml dep):
```rust
use ferro_mcp_oauth::{validate_bearer, validate::validate_api_key, BearerCheck, OAuthConfig};
use sea_orm::DatabaseConnection;
```

**Unifier function** — the replacement body for the entire file:
```rust
/// Resolve the calling tenant from the Authorization header.
///
/// Branches on token shape: `ferro_`-prefix → `validate_api_key` (async DB lookup),
/// otherwise → `validate_bearer` (sync JWT decode, wrapped in async fn for uniform call site).
pub async fn resolve_tenant(
    authorization_header: Option<&str>,
    db: &DatabaseConnection,
    oauth_config: &OAuthConfig,
) -> BearerCheck {
    let token = match authorization_header.and_then(|h| h.strip_prefix("Bearer ")) {
        None | Some("") => return BearerCheck::Unauthenticated,
        Some(t) => t,
    };
    if token.starts_with("ferro_") {
        validate_api_key(authorization_header, db, None).await
    } else {
        validate_bearer(authorization_header, oauth_config, None)
    }
}
```

**lib.rs re-export** — replace line 14 (`pub use auth::BearerOutcome;`):
```rust
// REMOVE:  pub use auth::BearerOutcome;
// ADD:
pub use auth::resolve_tenant;
pub use ferro_mcp_oauth::BearerCheck;   // re-export so consumer app imports from one crate
```

---

### `ferro-mcp-server/src/renderer.rs` — extend `McpContext` struct (MODIFY)

**Analog:** Same file, lines 7-10 (current `McpContext` definition). Also `framework/src/api/api_key.rs` lines 58-67 (`ApiKeyInfo` struct pattern — simple named fields, `#[derive(Debug, Clone)]`).

**Current state** (lines 7-10):
```rust
#[derive(Debug, Clone, Default)]
pub struct McpContext;
```

**Replacement** (D-07 — add fields, keep derives):
```rust
use std::collections::HashMap;

/// Per-request MCP context — tenant identity and evaluated permission guards.
///
/// `tenant_id`: resolved from the auth credential (JWT or API key); `None` = unauthenticated
/// or single-tenant (dispatch will fail-closed if the projection requires a tenant).
///
/// `evaluated_guards`: populated in Phase 218/219 for write-tool gate checks.
/// Absent key = allow; explicit `false` = deny (same semantics as `BaseContext`).
///
/// `scope`: key credential scope, if present. `None` = OAuth JWT path (full access).
/// `"read"` = read-only key; `"read_write"` = full key.
#[derive(Debug, Clone, Default)]
pub struct McpContext {
    pub tenant_id: Option<i64>,
    pub evaluated_guards: HashMap<String, bool>,
    pub scope: Option<String>,
}
```

**Call-site impact** — existing callers using `&McpContext` (unit struct) will break. The zero-value construction `McpContext` (no braces) must become `McpContext::default()` or `McpContext { ..Default::default() }`. Occurrences to update:
- `renderer.rs` line 87: `renderer.render(service, &intents, &McpContext)` → `&McpContext::default()`
- `renderer.rs` line 137: `render_exposed_tools(&services, &McpContext)` → `&McpContext::default()`
- `jsonrpc.rs` line 34: `render_exposed_tools(services, &McpContext)` → must receive real `ctx` parameter
- `jsonrpc_integration.rs` line 37: `handle_tools_list(&services, &config)` → must pass `&McpContext::default()`

---

### `ferro-mcp-server/src/error.rs` — add `Auth(String)` variant (MODIFY)

**Analog:** Same file, lines 1-18 (existing `Error` enum). Also `ferro-mcp-oauth/src/error.rs` for `thiserror` variant style.

**Current state** (lines 1-18) — full file content:
```rust
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("render error: {0}")]
    Render(String),
    #[error("invalid filter: {0}")]
    InvalidFilter(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

**Addition** — insert `Auth` variant before `Serialization` (alphabetical is fine, just be consistent):
```rust
    /// Caller is not authenticated or their credential scope is insufficient.
    /// Maps to JSON-RPC `-32603` at the jsonrpc layer (same envelope as OAuth invalid-token).
    #[error("auth error: {0}")]
    Auth(String),
```

**jsonrpc.rs error-mapping pattern** (lines 100-106) — the new `Auth` branch maps to `-32603` (same as other non-`InvalidFilter` errors):
```rust
// FROM jsonrpc.rs lines 100-106 — existing error-mapping pattern to extend:
Err(crate::Error::InvalidFilter(msg)) => {
    json!({ "error": { "code": -32602, "message": msg } })
}
Err(e) => json!({ "error": { "code": -32603, "message": e.to_string() } }),

// Auth errors fall through to the catch-all (-32603) — the `Display` impl from
// #[error("auth error: {0}")] produces the message. No separate branch needed
// unless the planner wants a distinct code (not required by D-08).
```

---

### `ferro-mcp-server/src/jsonrpc.rs` — extend `handle_tools_list` + scope gate (MODIFY)

**Analog:** Same file — existing `handle_tools_list` (lines 33-38) and `handle_tools_call` (lines 49-107).

**`handle_tools_list` signature change** — add `ctx: &McpContext` parameter (breaks existing call sites):
```rust
// CURRENT (line 33):
pub async fn handle_tools_list(services: &[ServiceDef], _config: &McpServerConfig) -> Value

// REPLACEMENT:
pub async fn handle_tools_list(
    services: &[ServiceDef],
    ctx: &McpContext,
    _config: &McpServerConfig,
) -> Value {
    match render_exposed_tools(services, ctx) {
        Ok(tools) => json!({ "result": { "tools": tools } }),
        Err(e) => json!({ "error": { "code": -32603, "message": e.to_string() } }),
    }
}
```

**Scope gate in `handle_tools_call`** — insert after service resolution (after line 66), before `dispatch` call (before line 84):
```rust
// Scope enforcement (D-06 / SC#3): re-check at call time, independent of listing filter.
// Absent scope (OAuth JWT path) = full access. "read" key cannot call non-read tools.
// In Phase 217 all tools are list_* (read-only); this gate fires once Phase 218 adds write tools.
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

**Note:** `handle_tools_call` signature does NOT change — it already takes `tenant_id: Option<i64>` (line 53). The `ctx` is not threaded into `handle_tools_call` in Phase 217; instead, the caller extracts `ctx.tenant_id` and `ctx.scope` before calling. The planner may choose to add a `ctx: &McpContext` parameter to `handle_tools_call` too, which would simplify the scope-gate access pattern.

---

### `ferro-mcp-server/Cargo.toml` — add `ferro-mcp-oauth` dependency (MODIFY)

**Analog:** Same file, line 14 (`ferro-projections` path dependency).

**Addition** (one line, after line 14):
```toml
# EXISTING pattern (line 14):
ferro-projections = { path = "../ferro-projections", version = "0.2" }

# ADD:
ferro-mcp-oauth = { path = "../ferro-mcp-oauth", version = "0.2" }
```

No other Cargo.toml changes required — all needed deps (`sha2`, `subtle`, `rand`, `sea-orm`, `thiserror`) are already present in `ferro-mcp-oauth/Cargo.toml` and `ferro-mcp-server/Cargo.toml`.

---

### `ferro-mcp-server/tests/mcp_tenant_isolation.rs` — create new integration test (CREATE)

**Analog:** `ferro-mcp-server/tests/dispatch_integration.rs` (fixture pattern) + `ferro-mcp-server/tests/common/mod.rs` (setup_db helper) + `ferro-mcp-server/tests/jsonrpc_integration.rs` (jsonrpc call pattern).

**File header and imports** — mirror `dispatch_integration.rs` lines 1-13:
```rust
//! Cross-tenant isolation, scope enforcement, and auth-parity tests for Phase 217.
//!
//! Uses an in-memory SQLite fixture — no consumer app models or Migrator.
//! Tables are created via raw SQL CREATE TABLE + INSERT (same pattern as
//! dispatch_integration.rs and dispatch.rs unit tests).

mod common;

use ferro_mcp_oauth::{validate_bearer, BearerCheck};
use ferro_mcp_oauth::validate::validate_api_key;
use ferro_mcp_server::dispatch;
use ferro_projections::{DataType, FieldMeaning, ServiceDef};
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
use serde_json::json;
```

**SQLite fixture setup** — mirror `dispatch.rs` lines 234-269 AND add `mcp_api_keys` table:
```rust
// Primary analog: dispatch.rs lines 234-269 + jsonrpc.rs lines 117-147 (both identical).
// Pattern: Database::connect("sqlite::memory:"), then Statement::from_string per table,
// then INSERT rows.
async fn setup_isolation_db() -> sea_orm::DatabaseConnection {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("sqlite connect");

    // mcp_api_keys table (canonical Phase 217 schema)
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "CREATE TABLE mcp_api_keys (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            tenant_id  INTEGER NOT NULL,
            key_hash   TEXT NOT NULL UNIQUE,
            scope      TEXT NOT NULL DEFAULT 'read',
            revoked_at TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )".to_string(),
    ))
    .await
    .expect("create mcp_api_keys");

    // orders table (same schema as dispatch.rs fixture, lines 242-250)
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "CREATE TABLE orders (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            customer_name TEXT NOT NULL,
            total         REAL NOT NULL,
            status        TEXT NOT NULL,
            tenant_id     INTEGER NOT NULL
        )".to_string(),
    ))
    .await
    .expect("create orders");

    // Seed two tenants' worth of orders (mirror dispatch.rs lines 256-264)
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "INSERT INTO orders (customer_name, total, status, tenant_id) VALUES
            ('Alice', 100.0, 'pending', 1),
            ('Bob',   200.0, 'shipped', 1),
            ('Carol', 150.0, 'pending', 2),
            ('Dave',  250.0, 'shipped', 2)".to_string(),
    ))
    .await
    .expect("seed orders");

    db
}
```

**ServiceDef helper** — mirror `dispatch.rs` lines 271-282 (same `order_service_with_tenant` pattern):
```rust
// FROM dispatch.rs lines 271-282 — copy verbatim:
fn order_service() -> ServiceDef {
    ServiceDef::new("order")
        .mcp_exposed(true)
        .tenant_column("tenant_id")
        .mcp_ability("view-orders")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("customer_name", DataType::String, FieldMeaning::EntityName)
        .field("total", DataType::Float, FieldMeaning::Money)
        .field("status", DataType::String, FieldMeaning::Status)
        .field("tenant_id", DataType::Integer, FieldMeaning::ForeignKey)
}
```

**API key seeding helper** — use `generate_mcp_api_key()` from `ferro-mcp-oauth` and raw SQL INSERT:
```rust
async fn seed_api_key(
    db: &sea_orm::DatabaseConnection,
    tenant_id: i64,
    scope: &str,
    revoked: bool,
) -> String {
    let (raw_key, key_hash) = ferro_mcp_oauth::validate::generate_mcp_api_key();
    let revoked_at = if revoked { "'2020-01-01T00:00:00Z'" } else { "NULL" };
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        format!(
            "INSERT INTO mcp_api_keys (tenant_id, key_hash, scope, revoked_at) \
             VALUES ({tenant_id}, '{key_hash}', '{scope}', {revoked_at})"
        ),
    ))
    .await
    .expect("seed api key");
    raw_key
}
```

**Test method pattern** — mirror `dispatch.rs` test block style (lines 295-363): `#[tokio::test]`, setup once, assert specific properties, named after the SC they verify:

SC#2 — auth parity:
```rust
#[tokio::test]
async fn api_key_and_jwt_produce_same_tenant_id() {
    // Use validate_api_key + validate_bearer (from ferro-mcp-oauth); assert both
    // return BearerCheck::Authenticated with principal["tenant_id"] == 1.
    // Mirror validate.rs lines 125-136 for the JWT half.
}
```

SC#3 — scope enforcement (no real write tool needed; test the gate logic directly):
```rust
#[tokio::test]
async fn read_scope_key_rejected_on_write_tool_name() {
    // Build McpContext { scope: Some("read".to_string()), ..Default::default() }
    // Call scope-check logic inline (or via handle_tools_call with a synthetic write tool name).
    // Assert json error with code -32603.
}
```

SC#4 — invalid/revoked key:
```rust
#[tokio::test]
async fn invalid_api_key_returns_bearer_invalid() { ... }

#[tokio::test]
async fn revoked_api_key_returns_bearer_invalid() { ... }
```

SC#5 — cross-tenant isolation:
```rust
// Mirror dispatch.rs lines 295-325 (tenant_scoping + tenant_isolation tests)
#[tokio::test]
async fn api_key_cross_tenant_isolation() {
    let db = setup_isolation_db().await;
    let raw_key = seed_api_key(&db, 1, "read", false).await;
    // validate_api_key → extract tenant_id=1
    // dispatch with tenant_id=1
    // assert all rows have tenant_id==1, count==2
}
```

---

### `ferro-mcp-server/tests/jsonrpc_integration.rs` — update `handle_tools_list` call sites (MODIFY)

**Analog:** Same file, existing test bodies (lines 17-93).

**Breaking call-site update** — line 37 must change from:
```rust
// CURRENT (line 37):
let resp = handle_tools_list(&services, &config).await;

// AFTER signature change (add ctx parameter as 2nd arg):
let resp = handle_tools_list(&services, &McpContext::default(), &config).await;
```

**Import addition** needed at top of file:
```rust
use ferro_mcp_server::renderer::McpContext;
// or via the re-export:
use ferro_mcp_server::McpContext;
```

No other test bodies change — `handle_tools_call` signature is unchanged.

---

## Shared Patterns

### SHA-256 hashing (applies to `validate.rs` and `migration.rs` tests)

**Source:** `framework/src/api/api_key.rs` lines 136-150
**Apply to:** `generate_mcp_api_key()`, `hash_mcp_api_key()` in `ferro-mcp-oauth/src/validate.rs`
```rust
// Exact copy pattern — sha2 already in ferro-mcp-oauth/Cargo.toml:
use sha2::{Digest, Sha256};

fn hash_mcp_api_key(raw_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_key.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

### `BearerCheck` outcome enum (applies to all auth paths)

**Source:** `ferro-mcp-oauth/src/validate.rs` lines 35-44
**Apply to:** `validate_api_key` return type, `resolve_tenant` return type, isolation test assertions
```rust
// Enum to produce/match on:
pub enum BearerCheck {
    Unauthenticated,  // 401 — no header or not Bearer
    Invalid,          // 401 — bad hash, not found, revoked
    Forbidden,        // 403 — tenant mismatch
    Authenticated(serde_json::Value),  // principal = json!({"sub":..., "tenant_id":..., "scope":...})
}
```

### SeaORM raw-SQL fixture pattern (applies to all tests)

**Source:** `ferro-mcp-server/tests/common/mod.rs` lines 4-28 and `ferro-mcp-server/src/dispatch.rs` lines 234-269
**Apply to:** `setup_isolation_db()` in the new `mcp_tenant_isolation.rs`
```rust
// Pattern: Database::connect("sqlite::memory:"), then Statement::from_string per DDL,
// then Statement::from_string for DML inserts. Do NOT use Statement::from_sql_and_values
// for test fixtures with hardcoded literals (comment in common/mod.rs lines 13-16 explains why).
let db = Database::connect("sqlite::memory:").await.expect("connect");
db.execute(Statement::from_string(DatabaseBackend::Sqlite, "CREATE TABLE ...".to_string()))
    .await.expect("create");
```

### thiserror enum variant pattern (applies to `error.rs`)

**Source:** `ferro-mcp-server/src/error.rs` lines 6-18
**Apply to:** New `Auth(String)` variant
```rust
// Exact style: #[error("lowercase noun: {0}")] Variant(String)
#[error("auth error: {0}")]
Auth(String),
```

### JSON-RPC error envelope (applies to `jsonrpc.rs`)

**Source:** `ferro-mcp-server/src/jsonrpc.rs` lines 64 and 100-106
**Apply to:** Scope-rejection response in `handle_tools_call`
```rust
// Error envelope pattern used throughout jsonrpc.rs:
json!({ "error": { "code": -32603, "message": "<message string>" } })
// -32602 = Invalid params (client error — bad filter key)
// -32603 = Internal error (used for auth rejection too, per D-08)
// -32601 = Method not found (unknown tool)
```

### SeaORM migration DeriveIden pattern (applies to `migration.rs`)

**Source:** `ferro-mcp-oauth/src/migration.rs` lines 69-77
**Apply to:** `McpApiKeys` enum in new migration
```rust
// Exact copy pattern:
#[derive(DeriveIden)]
enum McpApiKeys {
    Table,       // maps to the table name
    Id,
    TenantId,
    KeyHash,
    Scope,
    RevokedAt,
    CreatedAt,
    UpdatedAt,   // include if used in migration
}
```

---

## No Analog Found

All files have close analogs in the codebase. No file requires falling back to RESEARCH.md-only patterns.

---

## Critical Implementation Notes for Planner

1. **`ferro-mcp-oauth` dep must be Wave 0.** `ferro-mcp-server/Cargo.toml` currently has no `ferro-mcp-oauth` dep (confirmed: no grep match). The `auth.rs` replacement cannot compile until this line is added.

2. **`handle_tools_list` signature change is a Wave 0 breaking change.** Three existing test assertions in `renderer.rs` unit tests (lines 87, 137) and one in `jsonrpc_integration.rs` (line 37) pass `&McpContext` (unit struct, no braces). After the struct gains fields, these become `&McpContext::default()`. All must be updated in the same commit as the struct change.

3. **`mcp_tenant_isolation.rs` must use in-process SQLite.** The existing isolation tests at `app/src/tests/mcp_tenant_isolation.rs` import consumer-app `Migrator` and SeaORM entity models — those are not importable from `ferro-mcp-server`. The new file must use raw SQL DDL + DML (common/mod.rs pattern), not the app's migration stack.

4. **`validate_bearer` is sync; `validate_api_key` is async.** The unifier `resolve_tenant` in `auth.rs` must be `async fn`. The consumer app call site must `.await` it.

5. **`scope` absent on JWT principal = full access.** The OAuth `validate_bearer` returns `json!({"sub":..., "tenant_id":...})` with NO `scope` key (line 94-97). The scope-check gate in `handle_tools_call` must treat absent `scope` as `"read_write"` to preserve backward compatibility with the OAuth path.

6. **Table name is `mcp_api_keys`, not `api_keys`.** The existing general REST API key table is `api_keys` (schema: `name, prefix, hashed_key, created_at` — no tenant_id, no scope). The MCP keys table is new and distinct. Do not extend the existing table.

---

## Metadata

**Analog search scope:** `ferro-mcp-oauth/src/`, `ferro-mcp-server/src/`, `ferro-mcp-server/tests/`, `framework/src/api/`
**Files read:** 12 source files (validate.rs, jwt.rs, migration.rs, lib.rs, auth.rs, renderer.rs, error.rs, jsonrpc.rs, dispatch.rs, api_key.rs, common/mod.rs, dispatch_integration.rs, jsonrpc_integration.rs, both Cargo.toml files)
**Pattern extraction date:** 2026-06-13
