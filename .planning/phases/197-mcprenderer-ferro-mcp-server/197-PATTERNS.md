# Phase 197: McpRenderer & ferro-mcp-server — Pattern Map

**Mapped:** 2026-06-10
**Files analyzed:** 8 (7 new + 1 field edit)
**Analogs found:** 8 / 8

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-mcp-server/Cargo.toml` | config | — | `ferro-json-ui/Cargo.toml` | exact |
| `ferro-mcp-server/src/lib.rs` | config | — | `ferro-json-ui/src/lib.rs` (re-exports) | role-match |
| `ferro-mcp-server/src/renderer.rs` | service | request-response | `ferro-json-ui/src/projection/mod.rs` | exact |
| `ferro-mcp-server/src/error.rs` | utility | — | `ferro-projections/src/error.rs` | exact |
| `ferro-mcp-server/src/schema.rs` | utility | transform | `ferro-projections/src/field.rs` (`DataType`, `FieldMeaning`) | role-match |
| `ferro-mcp-server/src/dispatch.rs` | service | CRUD | `ferro-mcp/src/tools/crud_operations.rs` (list path) | exact |
| `ferro-mcp-server/tests/dispatch_integration.rs` | test | CRUD | `ferro-reservation/tests/concurrent_hold.rs` (`fresh_db()`) | role-match |
| `ferro-projections/src/service.rs` (field edit) | model | — | same file, existing builder methods (`display_name`, `description`) | exact |
| `Cargo.toml` (workspace root) `members` | config | — | existing `members` list | exact |
| `.github/workflows/publish.yml` Wave 2 | config | — | lines 269–274 `WAVE2_CRATES=` | exact |

---

## Pattern Assignments

### `ferro-mcp-server/Cargo.toml` (config)

**Analog:** `ferro-json-ui/Cargo.toml`

**Full manifest pattern** (lines 1–28):
```toml
[package]
name = "ferro-json-ui"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "JSON-based server-driven UI schema types for Ferro"
repository = "https://github.com/albertogferrario/ferro"
homepage = "https://ferro-rs.dev"
readme = "README.md"
keywords = ["json-ui", "sdui", "server-driven-ui", "ferro"]
categories = ["web-programming", "web-programming::http-server"]

[features]
projections = ["dep:ferro-projections", "dep:ferro-theme"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
schemars = { version = "1", features = ["derive"] }
strum = { version = "0.26", features = ["derive"] }
thiserror = "1.0"
tracing = "0.1"
ferro-projections = { path = "../ferro-projections", version = "0.2", optional = true }
ferro-theme = { path = "../ferro-theme", version = "0.2", optional = true }

[dev-dependencies]
serde_json = "1.0"
```

**Key differences for `ferro-mcp-server`:**
- `ferro-projections` is NOT optional — always required (no feature gate)
- Add `rmcp = { version = "0.12", default-features = false, features = ["schemars"] }` — critical: `default-features = false` prevents tokio/transport bloat
- Add `sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls"] }` and `tokio = { version = "1", features = ["full"] }` for dispatch
- Add `dotenvy = "0.15"` if loading DB URL from env in tests
- `ferro-mcp` uses `thiserror = "2"` (note version difference vs `ferro-projections` using `"1.0"`) — use `"1.0"` matching `ferro-json-ui`

---

### `ferro-mcp-server/src/renderer.rs` (service, request-response)

**Analog:** `ferro-json-ui/src/projection/mod.rs`

**Imports pattern** (lines 20–28):
```rust
use ferro_projections::render::Renderer;
use ferro_projections::Error;
use ferro_projections::IntentScore;
use ferro_projections::ServiceDef;
```

**Context struct pattern** (lines 45–68) — copy `VisualContext` shape, simplify to unit struct:
```rust
/// Visual rendering context for `JsonUiRenderer`.
#[derive(Debug, Clone)]
pub struct VisualContext {
    pub intent_index: usize,
    pub current_state: Option<String>,
    pub mode: RenderMode,
    pub templates: Option<ThemeTemplates>,
}

impl Default for VisualContext {
    fn default() -> Self {
        Self { intent_index: 0, current_state: None, mode: RenderMode::Display, templates: None }
    }
}
```
For `McpContext`, derive `Default` instead of implementing it manually (no fields in Phase 197):
```rust
#[derive(Debug, Clone, Default)]
pub struct McpContext;
```

**`impl Renderer` core pattern** (lines 98–112):
```rust
pub struct JsonUiRenderer;

impl Renderer for JsonUiRenderer {
    type Output = Spec;
    type Context = VisualContext;

    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &VisualContext,
    ) -> Result<Spec, Error> {
        Spec::from_service_def(service, intents, ctx).map_err(|e| Error::Render(e.to_string()))
    }
}
```
Mirror exactly: `type Output = rmcp::model::Tool`, `type Context = McpContext`. The `_intents` param is available but Phase 197 does not branch on intent.

**`Renderer` trait contract** (`ferro-projections/src/render/mod.rs` lines 33–53):
```rust
pub trait Renderer: Send + Sync {
    type Output;
    type Context: Default;

    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &Self::Context,
    ) -> Result<Self::Output, Error>;
}
```

**Test pattern** (lines 114–191) — copy the `sample_service()` helper + error-path assertions:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ferro_projections::{derive_intents, DataType, FieldMeaning, ServiceDef};

    fn sample_service() -> ServiceDef {
        ServiceDef::new("product")
            .display_name("Product")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("price", DataType::Float, FieldMeaning::Money)
    }
```

---

### `ferro-mcp-server/src/error.rs` (utility)

**Analog:** `ferro-projections/src/error.rs` (lines 1–13)

**Exact pattern to copy:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("service definition error: {0}")]
    Definition(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("render error: {0}")]
    Render(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

**For `ferro-mcp-server` Error enum**, adapt variants to the crate's domain:
- Keep `Render(String)` for `McpRenderer::render` failure path
- Add `Database(String)` for dispatch SQL errors (pattern from `ferro-mcp/src/error.rs` line 25)
- Add `#[from] serde_json::Error` for JSON construction failures

Also add a `Result<T>` type alias (pattern from `ferro-mcp/src/error.rs` line 5):
```rust
pub type Result<T> = std::result::Result<T, Error>;
```

---

### `ferro-mcp-server/src/schema.rs` (utility, transform)

**Analog:** `ferro-projections/src/field.rs`

**`DataType` enum** (lines 8–21) — import and match on these exact variants:
```rust
#[serde(rename_all = "snake_case")]
pub enum DataType {
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    Date,
    Json,
    Binary,
    Uuid,
    Enum,
}
```

**`FieldMeaning` enum** (lines 35–56) — import and match on these exact variants for the filter predicate:
```rust
pub enum FieldMeaning {
    Identifier,
    ForeignKey,
    EntityName,
    Email,
    Phone,
    Url,
    ImageUrl,
    Money,
    Percentage,
    Quantity,
    Status,
    Category,
    Boolean,
    FreeText,
    CreatedAt,
    UpdatedAt,
    DateTime,
    Sensitive,
    #[serde(untagged)]
    Custom(String),
}
```

**`FieldDef` struct** (lines 59–72) — the filter predicate reads these fields:
```rust
pub struct FieldDef {
    pub name: String,
    pub data_type: DataType,
    pub meaning: FieldMeaning,
    pub required: bool,
    pub is_list: bool,
    pub readable: bool,   // gate 1: must be true
    pub writable: bool,
}
```

**`infer_meaning` sensitive-detection pattern** (lines 139–143) — these field name patterns map to `FieldMeaning::Sensitive`:
```rust
const SENSITIVE: &[&str] = &["password", "secret", "token", "api_key", "hashed_key"];
if SENSITIVE.iter().any(|s| field_name.contains(s)) {
    return FieldMeaning::Sensitive;
}
```

The `is_filter_field` predicate in `schema.rs` must gate on `readable` first (gate 1), then `is_list` (gate 2), then `Sensitive` meaning (gate 3), then the conservative `FieldMeaning` allowlist (gate 4). Order matters: gate 1 catches `write_only_field()` entries that may or may not have `Sensitive` meaning.

---

### `ferro-mcp-server/src/dispatch.rs` (service, CRUD)

**Analog:** `ferro-mcp/src/tools/crud_operations.rs` — the `list` function (lines 338–419)

**Imports pattern** (lines 1–7):
```rust
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement};
```

**WHERE clause construction pattern** (lines 353–376):
```rust
let mut where_clauses = Vec::new();
let mut values: Vec<sea_orm::Value> = Vec::new();
let mut idx = 1usize;

if let Some(obj) = filters.as_object() {
    for (key, val) in obj {
        let col = validate_column(&meta, key)?;
        let field_meta = find_field(&meta, key);
        let field_type = field_meta.map(|f| f.field_type.as_str()).unwrap_or("");

        where_clauses.push(format!("\"{}\" = {}", col, placeholder(backend, idx)));
        values.push(json_to_sea_value(val, field_type));
        idx += 1;
    }
}

let where_str = if where_clauses.is_empty() {
    String::new()
} else {
    format!(" WHERE {}", where_clauses.join(" AND "))
};
```

**Count query pattern** (lines 378–391):
```rust
let count_sql = format!(
    "SELECT COUNT(*) as cnt FROM \"{}\"{}",
    meta.table_name, where_str
);
let count_stmt = Statement::from_sql_and_values(backend, &count_sql, values.clone());
let count_row = db.query_one(count_stmt).await
    .map_err(|e| McpError::DatabaseError(format!("Count query failed: {e}")))?;
let total: u64 = count_row
    .and_then(|r| r.try_get_by::<i64, _>("cnt").ok())
    .unwrap_or(0) as u64;
```

**Data query + pagination pattern** (lines 393–412):
```rust
let limit_str = format!(
    " LIMIT {} OFFSET {}",
    placeholder(backend, idx),
    placeholder(backend, idx + 1)
);
values.push(sea_orm::Value::BigInt(Some(per_page as i64)));
values.push(sea_orm::Value::BigInt(Some(offset as i64)));

let data_sql = format!(
    "SELECT * FROM \"{}\"{}{}",
    meta.table_name, where_str, limit_str
);
let data_stmt = Statement::from_sql_and_values(backend, &data_sql, values);
let rows = db.query_all(data_stmt).await
    .map_err(|e| McpError::DatabaseError(format!("List query failed: {e}")))?;
```

**`placeholder` helper** (lines 140–145) — copy verbatim:
```rust
pub(crate) fn placeholder(backend: DatabaseBackend, index: usize) -> String {
    match backend {
        DatabaseBackend::Postgres => format!("${index}"),
        _ => "?".to_string(),
    }
}
```

**`rows_to_json` helper** (lines 148–186) — copy verbatim (extracts columns from `QueryResult`, tries String → i64 → f64 → bool → Null).

**`json_to_sea_value` helper** (lines 114–137) — copy verbatim.

**Table name derivation** (line 53): `model.table.unwrap_or_else(|| model.name.to_lowercase() + "s")`. In `ferro-mcp-server`, `ServiceDef` has no `table` field yet; use `service.name.to_lowercase() + "s"` with a `// TODO: ServiceDef.table field` comment.

**`DispatchResult` struct** mirrors `CrudListResult` (lines 21–26):
```rust
#[derive(Debug, Serialize)]
pub struct CrudListResult {
    pub data: Vec<serde_json::Value>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}
```
Rename to `DispatchResult` with `rows` instead of `data`, and use `limit`/`offset` instead of `page`/`per_page` (the dispatch API uses offset-based pagination, not page-based).

**Column validation** — in `dispatch.rs`, column names come from `ServiceDef.fields[].name` (trusted), not from the call payload. Validate filter keys against `service.fields` before building the WHERE clause (same security principle as `validate_column` in `crud_operations.rs` line 190).

---

### `ferro-mcp-server/tests/dispatch_integration.rs` (test, CRUD)

**Analog:** `ferro-reservation/tests/concurrent_hold.rs` — `fresh_db()` helper (lines 38–42)

**In-memory SQLite setup pattern:**
```rust
async fn fresh_db() -> DatabaseConnection {
    let conn = Database::connect("sqlite::memory:").await.expect("connect");
    TestMigrator::up(&conn, None).await.expect("migrate");
    conn
}
```

For `dispatch_integration.rs`, Phase 197 does not have a `MigratorTrait` implementation. Use raw DDL instead:
```rust
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement};

async fn setup_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.expect("connect");
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "CREATE TABLE items (id INTEGER PRIMARY KEY, status TEXT NOT NULL, customer_id INTEGER)".to_string(),
    ))
    .await
    .expect("create table");
    db
}
```

**Test skeleton pattern** (from `ferro-queue/tests/race_claim_sqlite.rs` lines 33–43):
```rust
#[tokio::test]
async fn dispatch_returns_rows_from_fixture_table() {
    let db = setup_db().await;
    // insert fixture rows, call dispatch(), assert rows
}
```

**Import pattern** for sea-orm integration tests (`ferro-reservation/tests/concurrent_hold.rs` lines 17–22):
```rust
use sea_orm::{
    ConnectionTrait, Database, DatabaseConnection, ...
};
```

---

### `ferro-projections/src/service.rs` — `mcp_exposed` field + builder (model edit)

**Analog:** existing builder methods in same file (lines 98–108):
```rust
/// Sets the human-readable display name.
pub fn display_name(mut self, name: impl Into<String>) -> Self {
    self.display_name = Some(name.into());
    self
}

/// Sets the service description.
pub fn description(mut self, desc: impl Into<String>) -> Self {
    self.description = Some(desc.into());
    self
}
```

**Struct field pattern** (lines 62–80) — `#[serde(default, skip_serializing_if = ...)]` idiom:
```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub actions: Vec<ActionDef>,
```
For `mcp_exposed`, use `#[serde(default)]` without `skip_serializing_if` (a bool with value `false` is still useful to serialize explicitly when true):
```rust
/// Whether this projection is exposed as an MCP tool.
/// Defaults to `false`. Set to `true` to include in `tools/list` responses.
#[serde(default)]
pub mcp_exposed: bool,
```

**Builder method to add** — consuming `mut self -> Self` pattern:
```rust
/// Marks this projection as MCP-exposed.
pub fn mcp_exposed(mut self, exposed: bool) -> Self {
    self.mcp_exposed = exposed;
    self
}
```

**Test pattern** (`service.rs` lines 504–508):
```rust
#[test]
fn service_def_json_omits_none_fields() {
    let service = ServiceDef::new("order");
    let json = serde_json::to_string(&service).unwrap();
    assert!(!json.contains("display_name"));
}
```
Add analogous serde backward-compat test:
```rust
#[test]
fn mcp_exposed_defaults_false_when_absent() {
    let json = r#"{"name":"order","fields":[]}"#;
    let parsed: ServiceDef = serde_json::from_str(json).unwrap();
    assert!(!parsed.mcp_exposed);
}
```

---

### `.github/workflows/publish.yml` — Wave 2 edit

**Analog:** lines 269–290 of `.github/workflows/publish.yml`

**Current Wave 2 declaration** (line 274):
```yaml
WAVE2_CRATES="ferro-rs ferro-mcp"
```

**Target** — add `ferro-mcp-server` to Wave 2:
```yaml
WAVE2_CRATES="ferro-rs ferro-mcp ferro-mcp-server"
```

The surrounding loop (lines 276–289) picks this up without any other change.

---

### `Cargo.toml` workspace `members` edit

**Analog:** existing member list (lines 2–33 of root `Cargo.toml`)

**Current list ends with:** `"ferro-deployments"`, `"ferro-assets"`. Add `"ferro-mcp-server"` to the members array. Position is after `"ferro-mcp"` by convention (Wave 2 peer):
```toml
members = [
    ...
    "ferro-mcp",
    "ferro-mcp-server",     # <-- add here
    ...
]
```

---

## Shared Patterns

### Error enum with `type Result<T>` alias
**Source:** `ferro-mcp/src/error.rs` lines 5–6 + `ferro-projections/src/error.rs`
**Apply to:** `ferro-mcp-server/src/error.rs`
```rust
pub type Result<T> = std::result::Result<T, Error>;
```
All public functions in `dispatch.rs` return `Result<T>`, not `std::result::Result<T, Error>`.

### `#[serde(rename_all = "snake_case")]` on enums
**Source:** `ferro-projections/src/field.rs` lines 9, 32
**Apply to:** any new `pub enum` in `ferro-mcp-server` (e.g., future status/category enums)

### `mut self -> Self` consuming builder
**Source:** `ferro-projections/src/service.rs` lines 98–108
**Apply to:** `ServiceDef::mcp_exposed()` builder added in `ferro-projections/src/service.rs`

### `Statement::from_sql_and_values` parameterized SQL
**Source:** `ferro-mcp/src/tools/crud_operations.rs` line 301, 383, 405
**Apply to:** `ferro-mcp-server/src/dispatch.rs` — all SQL execution uses this pattern, never string interpolation of user values

### `rows_to_json` column extraction
**Source:** `ferro-mcp/src/tools/crud_operations.rs` lines 148–186 — string-first, then i64, then f64, then bool, then Null
**Apply to:** `ferro-mcp-server/src/dispatch.rs` — copy this helper verbatim; do not reimplement

---

## No Analog Found

All files have close analogs. No entries in this section.

---

## Metadata

**Analog search scope:** `ferro-json-ui/`, `ferro-projections/`, `ferro-mcp/`, `ferro-reservation/`, `ferro-orm/`, `ferro-queue/`, workspace root `Cargo.toml`, `.github/workflows/publish.yml`
**Files scanned:** 14
**Pattern extraction date:** 2026-06-10
