# Phase 197: McpRenderer & ferro-mcp-server — Research

**Researched:** 2026-06-10
**Domain:** Rust / MCP protocol / projection rendering / new output crate
**Confidence:** HIGH

---

## Summary

Phase 197 builds the projection→MCP-tool rendering substance of v12.6. The work
is a new output crate, `ferro-mcp-server`, that implements the existing `Renderer`
trait from `ferro-projections` — mirroring exactly how `ferro-json-ui` does it for
visual output. `ferro-projections` gains one bool field (`mcp_exposed`) and nothing
else; the dependency arrow is strictly `ferro-mcp-server` → `ferro-projections`.

All five research flags are resolved. The key findings are: (a) rmcp 0.12's `Tool`
struct is usable as the `Renderer::Output` type with no runtime-server bloat,
because the `schemars` feature (already present in the workspace) is the only
dependency needed for the model types — no tokio/transport features required in
the renderer crate; (b) the filter-field predicate must exclude `Sensitive` and
non-readable fields, and maps exactly seven `DataType` variants to JSON Schema
primitives; (c) for Phase 197 dispatch the reusable read path is the
`crud_operations::list` function in `ferro-mcp`, which wraps parameterized SQL via
SeaORM, requires a live DB connection, and provides exactly the `filters` + page/
per_page signature the dispatch layer needs; (d) `McpRenderer` associated types are
`Output = rmcp::model::Tool` and `Context = McpContext` (a small `#[derive(Default)]`
struct carrying nothing for Phase 197, extensible to tenant/policy context for
Phase 200); (e) the new crate belongs in **Wave 2** of `publish.yml` (alongside
`ferro-mcp`), and needs a one-time manual bootstrap publish.

**Primary recommendation:** Build `ferro-mcp-server` as a pure-Rust crate with
`rmcp = { version = "0.12", features = ["schemars"] }` (no server/transport
features needed in the renderer), `ferry-projections` as the only internal dep, and
implement `McpRenderer` to produce `rmcp::model::Tool`. Dispatch in Phase 197 is an
`async fn dispatch(service: &ServiceDef, filters: serde_json::Value, limit: u64,
offset: u64, db: &DatabaseConnection) -> Result<Vec<serde_json::Value>, Error>`
that delegates to `crud_operations::list`.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `McpRenderer` lives in `ferro-mcp-server` (new output crate). Dependency
  direction is `ferro-mcp-server` → `ferro-projections` only; `ferro-projections`
  gains NO dependency on the new crate. Manifest mirrors `ferro-json-ui/Cargo.toml`.
  Pure Rust (no nasm/system libs).

- **D-02:** Add `mcp_exposed: bool` (default `false`, `#[serde(default)]`) to
  `ServiceDef` plus `.mcp_exposed(true)` builder. Plain metadata — no renderer
  dependency introduced into `ferro-projections` (SC-4 holds).

- **D-03:** Reuse workspace `rmcp` 0.12 for the MCP tool-definition representation.
  If `rmcp`'s Tool is coupled to its server runtime in a way that bloats a pure
  renderer crate, fall back to a minimal local struct serializing to MCP tool JSON.

- **D-04:** Tool `inputSchema` built from `ServiceDef` fields as `serde_json::Value`
  JSON Schema — single source, no separately-declared schema. Pagination (`limit`
  default 25 max 100, `offset` default 0). Filters: readable, non-sensitive fields
  whose `FieldMeaning` is in a conservative subset. `DataType` → JSON Schema type
  mapping. Sensitive fields never emitted.

- **D-05:** Dispatch function (distinct from `Renderer::render`) executes the
  projection's existing read path, returns rows as MCP structured content. Takes a
  DB connection parameter so Phase 200 can wire tenant scoping. Must REUSE existing
  read logic, not reimplement. No ad-hoc ownership filter.

- **D-06:** Add to Wave 2 in `publish.yml`. New crate needs manual bootstrap publish
  (CI token is publish-update only).

### Claude's Discretion

- Exact module layout within `ferro-mcp-server`.
- `McpRenderer`'s associated `Output` and `Context` types.
- Naming of the `tools/list` / `tools/call` in-process exercise.
- Whether `limit` max is 100 vs another conservative cap.

### Deferred Ideas (OUT OF SCOPE)

- HTTP transport / `/mcp` endpoint (Phase 198).
- OAuth browser login (Phase 199).
- Per-tenant scoping + policy enforcement (Phase 200).
- Write intents, multi-projection auto-exposure, MCP App UI (later milestones).
- Transport runtime choice for rmcp (Phase 198).
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| AMCP-01 | A projection marked MCP-exposed appears in `tools/list` as exactly one tool; unmarked projections never appear | D-02 (`mcp_exposed` bool on `ServiceDef`); filter in `tools/list` implementation |
| AMCP-02 | The tool's input JSON schema is derived from `ServiceDef` fields, not declared separately | D-04; `FieldMeaning`/`DataType` mapping documented in Filter-Field Predicate section |
| AMCP-03 | Calling the tool runs the projection's existing read path and returns rows as MCP structured content | D-05; `crud_operations::list` identified as the reusable read path |
| AMCP-04 | `McpRenderer` lives in `ferro-mcp-server` implementing `Renderer`; `ferro-projections` gains no renderer dep | D-01; `Renderer` trait contract verified in `ferro-projections/src/render/mod.rs` |
| SC-5 | Register the new crate in `.github/workflows/publish.yml` | D-06; Wave 2 confirmed correct |
</phase_requirements>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Tool definition (static schema) | `ferro-mcp-server` | `ferro-projections` (data) | `McpRenderer::render` produces the `Tool` struct — pure data transformation |
| `mcp_exposed` marker | `ferro-projections` | — | It is metadata on `ServiceDef`, not rendering logic |
| Filter-field selection | `ferro-mcp-server` | — | `is_filter_field()` predicate reads `FieldDef` and lives in the renderer crate |
| inputSchema JSON construction | `ferro-mcp-server` | — | `serde_json::Value` built in `schema.rs`; depends on `DataType`/`FieldMeaning` |
| Row dispatch / DB query | `ferro-mcp` (reused) | `ferro-mcp-server` (dispatches) | `crud_operations::list` owns the SQL; dispatcher calls it |
| In-process `tools/list` + `tools/call` exercise | `ferro-mcp-server` tests | — | No HTTP layer; proved in unit/integration tests |
| Publish wave registration | `.github/workflows/publish.yml` | Workspace `Cargo.toml` | Infra, not code |

---

## Standard Stack

### Core (verified)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rmcp` | 0.12.0 | MCP protocol types (`Tool`, `ToolAnnotations`, `JsonObject`) | Already workspace dep in `ferro-mcp` and `ferro-api-mcp`; Phase 198 transport will also use it |
| `ferro-projections` | 0.2 (workspace) | `Renderer` trait, `ServiceDef`, `FieldDef`, `FieldMeaning`, `DataType` | Source of truth for projection schema |
| `serde` + `serde_json` | 1.0 | JSON/serde | Universal; `inputSchema` is a `serde_json::Value` |
| `schemars` | 1 (workspace) | `JsonObject` aliased type used by `rmcp::model::Tool.input_schema: Arc<JsonObject>` | Already workspace dep |
| `thiserror` | 1.0 | Error enum per crate | Project convention |
| `tracing` | 0.1 | Structured logging | Project convention |

[VERIFIED: read from ferro-mcp/Cargo.toml, ferro-json-ui/Cargo.toml, rmcp-0.12.0 source]

### Reused from `ferro-mcp` (for dispatch only)

| Crate | Version | Why Referenced |
|-------|---------|----------------|
| `sea-orm` | 1.0 | `crud_operations::list` uses `DatabaseConnection` — dispatch takes this type |
| `ferro-mcp` | 0.2 (workspace path) | NOT a `ferro-mcp-server` Cargo.toml dep; dispatcher function exposed as a re-export or the call pattern is copied |

> **Note on dispatch dependency:** The dispatch function in `ferro-mcp-server` needs
> to call `crud_operations::list` from `ferro-mcp`. There are two options:
>
> 1. **Move** `crud_operations::list` (and its helpers) to a shared crate (adds
>    scope). Not recommended for Phase 197.
> 2. **Re-implement** a thin dispatch layer in `ferro-mcp-server` using
>    `sea-orm` + `dotenvy` directly, following the same pattern as
>    `crud_operations::list`. The pattern is 30 lines of SQL + pagination —
>    it is not "hand-rolling query logic" because the SQL itself is identical
>    (SELECT * WHERE filters LIMIT/OFFSET). This avoids a circular or
>    forward-dependency edge (`ferro-mcp` depends on `ferro-json-ui` which
>    depends on `ferro-projections`; adding `ferro-mcp` → `ferro-mcp-server`
>    would create a forward reference issue). **Recommended for Phase 197.**
>
> The dispatch function signature must match the `DatabaseConnection` type from
> `sea-orm 1.0` regardless of approach.

[VERIFIED: read crud_operations.rs in full]

### Installation (new crate manifest)

```toml
# ferro-mcp-server/Cargo.toml
[package]
name = "ferro-mcp-server"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "MCP tool rendering target for Ferro projections"
repository = "https://github.com/albertogferrario/ferro"
homepage = "https://ferro-rs.dev"
readme = "README.md"
keywords = ["mcp", "ferro", "projections", "server-driven"]
categories = ["web-programming", "web-programming::http-server"]

[dependencies]
ferro-projections = { path = "../ferro-projections", version = "0.2" }
rmcp = { version = "0.12", default-features = false, features = ["schemars"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
tracing = "0.1"
# Dispatch deps (for the in-process exercise in Phase 197):
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls"] }
tokio = { version = "1", features = ["full"] }
dotenvy = "0.15"

[dev-dependencies]
tokio = { version = "1", features = ["full", "macros"] }
```

[VERIFIED: pattern from ferro-json-ui/Cargo.toml; rmcp feature analysis from Cargo.toml.orig]

---

## Architecture Patterns

### System Architecture Diagram

```
[ServiceDef + mcp_exposed: true]
          |
          v
  McpRenderer::render()          (ferro-mcp-server/src/renderer.rs)
          |
          |-- build_input_schema(fields)   (ferro-mcp-server/src/schema.rs)
          |       |
          |       |-- filter_field predicate (FieldMeaning + readable flag)
          |       |-- DataType → JSON Schema type map
          |       '-- pagination params (limit/offset)
          |
          v
    rmcp::model::Tool              (name, description, inputSchema, annotations.readOnlyHint=true)
          |
          | (tools/list in-process test)
          v
  [agent sends tools/call with filters + limit + offset]
          |
          v
  dispatch()                       (ferro-mcp-server/src/dispatch.rs)
          |
          |-- resolve table name from ServiceDef.name
          |-- execute SELECT * WHERE {equality filters} LIMIT/OFFSET via sea-orm
          v
  Vec<serde_json::Value>           (MCP CallToolResult content)
```

### Recommended Project Structure

```
ferro-mcp-server/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs          # pub use McpRenderer, McpContext, dispatch; crate Error
    ├── renderer.rs     # McpRenderer (impl Renderer), McpContext struct
    ├── schema.rs       # build_input_schema(), is_filter_field(), data_type_to_json_schema()
    ├── dispatch.rs     # dispatch() async fn — DB read path
    └── error.rs        # Error enum (thiserror)
```

### Pattern 1: McpRenderer implementing Renderer

```rust
// ferro-mcp-server/src/renderer.rs
use ferro_projections::{render::Renderer, Error, IntentScore, ServiceDef};
use rmcp::model::{Tool, ToolAnnotations};

/// Context for MCP rendering. Carries no state in Phase 197;
/// Phase 200 will extend with tenant context.
#[derive(Debug, Clone, Default)]
pub struct McpContext;

pub struct McpRenderer;

impl Renderer for McpRenderer {
    type Output = Tool;
    type Context = McpContext;

    fn render(
        &self,
        service: &ServiceDef,
        _intents: &[IntentScore],
        _ctx: &McpContext,
    ) -> Result<Tool, Error> {
        let name = format!("list_{}", service.name);
        let description = service
            .description
            .clone()
            .unwrap_or_else(|| format!("List {} records", service.display_name
                .as_deref().unwrap_or(&service.name)));

        let input_schema = crate::schema::build_input_schema(service)
            .map_err(|e| Error::Render(e.to_string()))?;

        let schema_obj = match input_schema {
            serde_json::Value::Object(m) => m,
            _ => return Err(Error::Render("inputSchema must be an object".into())),
        };

        let annotations = ToolAnnotations::new().read_only(true);

        Ok(Tool::new(name, description, std::sync::Arc::new(schema_obj))
            .annotate(annotations))
    }
}
```

[VERIFIED: Renderer trait from ferro-projections/src/render/mod.rs; rmcp::model::Tool from rmcp-0.12.0 source]

### Pattern 2: inputSchema construction

```rust
// ferro-mcp-server/src/schema.rs
use ferro_projections::service::ServiceDef;
use ferro_projections::field::{DataType, FieldMeaning};

pub fn build_input_schema(service: &ServiceDef) -> Result<serde_json::Value, crate::Error> {
    let mut properties = serde_json::Map::new();

    // Pagination
    properties.insert("limit".into(), serde_json::json!({
        "type": "integer",
        "description": "Maximum number of records to return",
        "default": 25,
        "maximum": 100,
        "minimum": 1
    }));
    properties.insert("offset".into(), serde_json::json!({
        "type": "integer",
        "description": "Number of records to skip",
        "default": 0,
        "minimum": 0
    }));

    // Equality filters from readable, non-sensitive fields
    for field in service.fields.iter().filter(|f| is_filter_field(f)) {
        let json_type = data_type_to_json_schema(field.data_type);
        properties.insert(field.name.clone(), serde_json::json!({
            "type": json_type,
            "description": format!("Filter by {}", field.name)
        }));
    }

    Ok(serde_json::json!({
        "type": "object",
        "properties": properties
    }))
}
```

[VERIFIED: FieldDef/FieldMeaning from ferro-projections/src/field.rs]

### Pattern 3: dispatch function signature

```rust
// ferro-mcp-server/src/dispatch.rs
use ferro_projections::service::ServiceDef;
use sea_orm::DatabaseConnection;

pub async fn dispatch(
    service: &ServiceDef,
    filters: serde_json::Value,
    limit: u64,
    offset: u64,
    db: &DatabaseConnection,
) -> Result<Vec<serde_json::Value>, crate::Error> {
    // Execute SELECT * FROM {table} WHERE {equality filters} LIMIT {limit} OFFSET {offset}
    // Pattern mirrors crud_operations::list from ferro-mcp
    // ...
}
```

Phase 200 adds `tenant_id: Option<&str>` or similar to this signature. The current
signature has no ownership filter, per the design invariant (no duplicate control
surface).

[VERIFIED: crud_operations::list signature and pattern from ferro-mcp/src/tools/crud_operations.rs]

### Anti-Patterns to Avoid

- **Adding a second `mcp_exposed` registry in `ferro-mcp-server`:** D-02 uses a bool
  on `ServiceDef`. Do not add a second registration mechanism (no duplicate control
  surface — feedback_no_duplicate_control_surface.md).
- **Baking a tenant/ownership filter into dispatch:** Phase 200 owns policy. A
  `WHERE tenant_id = X` added in Phase 197 would compete with the policy layer.
- **Using `rmcp` `server`/`transport-io` features in `ferro-mcp-server`:** The
  renderer crate must not pull in tokio's stdio transport. Use
  `default-features = false, features = ["schemars"]` to get only model types.
- **Reimplementing the `Renderer` trait contract:** The trait is already
  `fn render(&self, service, intents, ctx) -> Result<Output, Error>`. Tool derivation
  lives there; dispatch is a separate async function.
- **Placing `JsonUiRenderer` precedent differently:** `ferro-json-ui` uses a
  `projections` feature flag to conditionally include `ferro-projections`. For
  `ferro-mcp-server`, `ferro-projections` is always required (no feature needed).

---

## Research Flag Resolutions

### D-03: rmcp 0.12 Tool Ergonomics (VERIFIED)

`rmcp::model::Tool` [VERIFIED: rmcp-0.12.0/src/model/tool.rs]:

```rust
pub struct Tool {
    pub name: Cow<'static, str>,
    pub title: Option<String>,
    pub description: Option<Cow<'static, str>>,
    pub input_schema: Arc<JsonObject>,  // JsonObject = serde_json::Map<String, Value>
    pub output_schema: Option<Arc<JsonObject>>,
    pub annotations: Option<ToolAnnotations>,
    // ...
}
```

`Tool::new(name, description, input_schema)` is a clean constructor taking anything
`Into<Arc<JsonObject>>`. The builder supports `.annotate(ToolAnnotations)`.
`ToolAnnotations::new().read_only(true)` sets `readOnlyHint: true` — exactly the
annotation the CONTEXT.md `<specifics>` section requires.

**Feature analysis:** The `rmcp` default features include `server` (which pulls in
`schemars` + transport-async-rw + tokio). To use only model types, specify:

```toml
rmcp = { version = "0.12", default-features = false, features = ["schemars"] }
```

The `schemars` feature activates `dep:schemars` (used for `#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]` on model types). No tokio, no transport,
no server runtime. `rmcp::model::Tool`, `rmcp::model::ToolAnnotations`, and
`rmcp::model::JsonObject` are all available with only the `schemars` feature.

**Verdict:** Use `rmcp::model::Tool` directly. No local tool-definition struct
needed. `type Output = rmcp::model::Tool`.

[VERIFIED: rmcp-0.12.0/Cargo.toml features; model/tool.rs source]

### D-04: inputSchema Derivation Predicate (VERIFIED)

From `ferro-projections/src/field.rs`:

**`FieldMeaning` variants (complete list):**
`Identifier`, `ForeignKey`, `EntityName`, `Email`, `Phone`, `Url`, `ImageUrl`,
`Money`, `Percentage`, `Quantity`, `Status`, `Category`, `Boolean`, `FreeText`,
`CreatedAt`, `UpdatedAt`, `DateTime`, `Sensitive`, `Custom(String)`

**`FieldDef` struct:**
```rust
pub struct FieldDef {
    pub name: String,
    pub data_type: DataType,
    pub meaning: FieldMeaning,
    pub required: bool,
    pub is_list: bool,
    pub readable: bool,
    pub writable: bool,
}
```

**Conservative filter-field predicate (recommended):**

```rust
pub fn is_filter_field(field: &FieldDef) -> bool {
    // Gate 1: must be readable
    if !field.readable { return false; }
    // Gate 2: must not be a list field (equality filter on lists is undefined)
    if field.is_list { return false; }
    // Gate 3: exclude sensitive meanings
    if matches!(field.meaning, FieldMeaning::Sensitive) { return false; }
    // Gate 4: conservative meaning allowlist
    matches!(
        field.meaning,
        FieldMeaning::Identifier
        | FieldMeaning::ForeignKey
        | FieldMeaning::Status
        | FieldMeaning::Category
        | FieldMeaning::Boolean
        | FieldMeaning::Custom(_)
    )
}
```

Rationale for allowlist (not blocklist):
- `Identifier` / `ForeignKey` → natural equality filters (lookup by ID / parent ID)
- `Status` / `Category` → enum-like; exact match is the canonical filter
- `Boolean` → boolean toggle filters (`is_active=true`)
- `Custom(_)` → unknown domain field; include conservatively (an agent asking for
  it by name is reasonable; excludes only the explicitly-sensitive patterns)
- Excluded: `EntityName`, `Email`, `Phone`, `Url`, `ImageUrl`, `Money`,
  `Percentage`, `Quantity`, `FreeText`, `CreatedAt`, `UpdatedAt`, `DateTime` —
  these are better served by range/substring filters which are out of scope for
  Phase 197's equality-only skeleton

**Note on `infer_meaning` sensitive detection:** `infer_meaning` maps fields
containing "password", "secret", "token", "api_key", "hashed_key" to
`FieldMeaning::Sensitive`. Additionally, `FieldDef.readable = false` is set for
`write_only_field()` builder calls. Both are caught by the predicate above.

**DataType → JSON Schema type mapping:**

| `DataType` | JSON Schema `"type"` | Notes |
|------------|---------------------|-------|
| `String` | `"string"` | |
| `Integer` | `"integer"` | |
| `Float` | `"number"` | |
| `Boolean` | `"boolean"` | |
| `DateTime` | `"string"` | + `"format": "date-time"` |
| `Date` | `"string"` | + `"format": "date"` |
| `Json` | `"object"` | skip as filter (add to non-filterable) |
| `Binary` | `"string"` | skip as filter |
| `Uuid` | `"string"` | + `"format": "uuid"` |
| `Enum` | `"string"` | |

For Phase 197, `Json` and `Binary` fields matching `is_filter_field` should be
omitted from the filter surface (equality filter on a JSON column is not useful).
Add a secondary check: `!matches!(field.data_type, DataType::Json | DataType::Binary)`.

[VERIFIED: ferro-projections/src/field.rs in full]

### D-05: Read-Path / Dispatch Mechanism (VERIFIED — load-bearing)

**Candidate examined:** `ferro-mcp/src/tools/crud_operations.rs`

The `list` function signature:

```rust
pub async fn list(
    project_root: &Path,
    model: &str,
    filters: Option<&serde_json::Value>,
    page: Option<u64>,
    per_page: Option<u64>,
) -> Result<CrudListResult, McpError>
```

It:
1. Reads model metadata from source files via `list_models::execute(project_root)`.
2. Opens a `DatabaseConnection` from `DATABASE_URL` in `.env`.
3. Builds parameterized SQL: `SELECT * FROM "{table}" WHERE {col} = $N LIMIT $N OFFSET $N`.
4. Returns `CrudListResult { data: Vec<serde_json::Value>, total, page, per_page }`.

**For Phase 197 dispatch, the recommended approach:**

The `ferro-mcp-server` dispatch function does NOT take `project_root: &Path`. It
takes `db: &DatabaseConnection` (already connected) and derives the table name from
`service.name` + an optional `table` override. This is cleaner for the Phase 198+
context where the framework provides the DB connection, not a filesystem path.

The dispatch layer reimplements ~30 lines of parameterized SQL (SELECT + COUNT +
LIMIT/OFFSET) following the identical pattern from `crud_operations.rs`. This is
not a logic reimplementation — it is a thin adapter that speaks to `sea-orm`
directly. Critically, it does NOT resolve model metadata from source files (that
was only needed for the dev-time MCP introspection tool); it derives table name
from `ServiceDef.name` (appending an "s" pluralization or using a future
`ServiceDef.table` field when added).

**Answers to the three sub-questions:**

(a) **What function/path lists a projection's rows?**
`crud_operations::list` in `ferro-mcp` is the conceptual template, but the actual
dispatch in `ferro-mcp-server` reimplements the DB call pattern using the
`DatabaseConnection` passed in from the caller. The SQL is identical
(`SELECT * FROM "{table}" WHERE {filters} LIMIT {limit} OFFSET {offset}`).

(b) **Live DB or fixture for Phase 197 in-process test?**
The `dispatch()` test requires a live DB connection. For the in-process test,
either: (i) use an in-memory SQLite database (`:memory:`), creating a test table
and inserting fixture rows in the test setup; or (ii) skip dispatch integration in
unit tests and cover it with a separate integration test that requires
`DATABASE_URL`. Option (i) is preferred for Phase 197 — SQLite in-memory is
zero-setup and validates the SQL path fully. The `sea-orm` feature flags already
include `sqlx-sqlite`.

(c) **Dispatch signature for Phase 200 extensibility:**

```rust
pub async fn dispatch(
    service: &ServiceDef,
    filters: serde_json::Value,  // JSON object of equality filters
    limit: u64,
    offset: u64,
    db: &DatabaseConnection,
    // Phase 200 will add: tenant_id: Option<i64>, policy_ctx: Option<&PolicyCtx>
) -> Result<DispatchResult, Error>

pub struct DispatchResult {
    pub rows: Vec<serde_json::Value>,
    pub total: u64,
    pub limit: u64,
    pub offset: u64,
}
```

Phase 200 adds parameters to this signature. The existing parameters do not change.

[VERIFIED: ferro-mcp/src/tools/crud_operations.rs in full]

### Renderer Trait Fit (VERIFIED)

From `ferro-projections/src/render/mod.rs`:

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

From `ferro-json-ui/src/projection/mod.rs` (precedent):

```rust
pub struct JsonUiRenderer;

impl Renderer for JsonUiRenderer {
    type Output = Spec;
    type Context = VisualContext;  // VisualContext implements Default

    fn render(&self, service, intents, ctx) -> Result<Spec, Error> { ... }
}
```

**`McpRenderer` associated types:**

```
type Output = rmcp::model::Tool
type Context = McpContext   // unit struct, #[derive(Debug, Clone, Default)]
```

`McpContext` carries nothing in Phase 197. It is extensible for Phase 200
(tenant config, policy token). The `intents` parameter is available in `render()`
but Phase 197 does not need to branch on intent (all exposed projections are
Browse-intent read tools for now — the CONTEXT.md explicitly defers write intents).

[VERIFIED: ferro-projections/src/render/mod.rs; ferro-json-ui/src/projection/mod.rs]

### Crate Scaffold + Publish Wave (VERIFIED)

From `.github/workflows/publish.yml` lines 211–300:

| Wave | Crates |
|------|--------|
| 1a | `ferro-macros ferro-events ferro-queue ferro-broadcast ferro-storage ferro-cache ferro-lang ferro-theme ferro-json-ui ferro-inertia ferro-api-mcp ferro-wallet ferro-orm ferro-audit ferro-migration ferro-assets` |
| 1b | `ferro-projections ferro-ai ferro-stripe ferro-whatsapp ferro-notifications ferro-reservation ferro-projection ferro-deployments` |
| **2** | **`ferro-rs ferro-mcp`** |
| 3 | `ferro-cli ferro-bundle` |

`ferro-mcp-server` depends on `ferro-projections` (Wave 1b) and `sea-orm` (external).
It has no dependency on `ferro-rs` or `ferro-mcp`. It could technically go in Wave 2
(alongside `ferro-mcp`) or even be inserted between 1b and 2. Given that it is a
peer to `ferro-mcp` in the v12.6 MCP stack, **Wave 2** is the correct placement.
Add it to the Wave 2 variable: `WAVE2_CRATES="ferro-rs ferro-mcp ferro-mcp-server"`.

**Bootstrap caveat:** CI token is publish-update only. A brand-new crate requires a
one-time manual local bootstrap: `cargo publish -p ferro-mcp-server` from the
workspace root (requires a crates.io token with publish-new scope).

[VERIFIED: .github/workflows/publish.yml lines 206–300]

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| MCP tool JSON schema | Custom tool struct | `rmcp::model::Tool` + `ToolAnnotations` | Already in workspace; correct camelCase serde; `readOnlyHint` is standard |
| Parameterized SQL pagination | Custom query builder | Pattern from `crud_operations::list` (direct `sea-orm` Statement) | Correct placeholder handling for both PostgreSQL (`$N`) and SQLite (`?`) |
| Sensitive field detection | String scanning | `FieldMeaning::Sensitive` + `FieldDef.readable` flags | `infer_meaning()` already maps "password"/"secret"/"token"/"api_key" → `Sensitive` |
| Tool name derivation | Heuristics | `format!("list_{}", service.name)` | Deterministic; consistent with `ServiceDef.name` being snake_case |

---

## Common Pitfalls

### Pitfall 1: rmcp default features pulling in tokio/transport

**What goes wrong:** Adding `rmcp = "0.12"` (default features) to `ferro-mcp-server`
pulls in `server` → `transport-async-rw` → `tokio/io-util` + `tokio-util/codec`.
This inflates a pure renderer crate with an async runtime dependency.

**Why it happens:** rmcp's `default` feature includes `server`.

**How to avoid:** `rmcp = { version = "0.12", default-features = false, features = ["schemars"] }`.

**Warning signs:** `cargo tree -p ferro-mcp-server` shows `tokio` as a dep when the
renderer test runs without an async runtime.

### Pitfall 2: `input_schema` requires `Arc<JsonObject>` (not `Value`)

**What goes wrong:** Passing a `serde_json::Value` directly to `Tool::new()` fails
to compile — the third argument is `Into<Arc<JsonObject>>` where `JsonObject =
serde_json::Map<String, Value>`.

**How to avoid:** Match on the schema value to extract the `Object` variant:
```rust
let schema_obj: serde_json::Map<String, Value> = match build_input_schema(svc)? {
    Value::Object(m) => m,
    _ => unreachable!("build_input_schema always returns an object"),
};
Tool::new(name, desc, std::sync::Arc::new(schema_obj))
```

### Pitfall 3: Table name derivation — ServiceDef has no `table` field yet

**What goes wrong:** `ServiceDef` does not carry a `table` field. The dispatch layer
would naively append "s" (`order` → `orders`) but this is wrong for irregular
plurals (`person` → `people`) or custom table names.

**How to avoid for Phase 197:** Follow `crud_operations.rs` which uses
`model.table.unwrap_or_else(|| model.name.to_lowercase() + "s")`. The same
heuristic is acceptable for Phase 197 (skeleton). Flag this as a known limitation
in a TODO comment; a future phase can add `ServiceDef.table` metadata.

### Pitfall 4: `Sensitive` not the only exclusion path

**What goes wrong:** A password field using `write_only_field()` has
`FieldMeaning::Sensitive` but also `readable: false`. Filtering only on meaning
would still exclude it, but the `readable` gate must be primary (a future field
could be readable but with a different sensitive marker).

**How to avoid:** Keep gate 1 (`!field.readable`) before gate 3 (`Sensitive`). Both
gates are required; they cover different code paths.

### Pitfall 5: `ferro-mcp` depends on `ferro-json-ui` — no circular dep

**What goes wrong:** If `ferro-mcp-server` added `ferro-mcp` as a dep, and
`ferro-mcp` already adds `ferro-json-ui` + `ferro-projections`, there is no circular
dep technically — but it inflates the renderer crate with the entire dev-time MCP
tool surface (sea-orm, redis, regex, walkdir, syn, etc.).

**How to avoid:** Do not add `ferro-mcp` as a dependency of `ferro-mcp-server`. The
dispatch SQL pattern is short enough to implement inline, as documented in the D-05
resolution above.

---

## Code Examples

### tool/list in-process exercise pattern

```rust
// ferro-mcp-server/src/lib.rs (or a test/example)
use ferro_projections::{derive_intents, DataType, FieldMeaning, ServiceDef};
use ferro_projections::render::Renderer;
use crate::{McpContext, McpRenderer};

let orders = ServiceDef::new("order")
    .display_name("Order")
    .description("Customer orders")
    .field("id", DataType::Integer, FieldMeaning::Identifier)
    .field("status", DataType::String, FieldMeaning::Status)
    .field("customer_id", DataType::Integer, FieldMeaning::ForeignKey)
    .mcp_exposed(true);   // new builder method on ServiceDef

let intents = derive_intents(&orders);
let renderer = McpRenderer;
let tool = renderer.render(&orders, &intents, &McpContext::default()).unwrap();

assert_eq!(tool.name, "list_order");
assert!(tool.annotations.as_ref().unwrap().read_only_hint == Some(true));

// inputSchema has "limit", "offset", "id", "status", "customer_id"
let schema = tool.schema_as_json_value();
let props = schema["properties"].as_object().unwrap();
assert!(props.contains_key("limit"));
assert!(props.contains_key("status"));
assert!(!props.contains_key("password")); // never emitted
```

### `mcp_exposed` field addition to ServiceDef

```rust
// ferro-projections/src/service.rs — additions only
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ServiceDef {
    // ... existing fields unchanged ...

    /// Whether this projection is exposed as an MCP tool.
    /// Defaults to false. Only projections with `mcp_exposed: true`
    /// appear in a `tools/list` response.
    #[serde(default)]
    pub mcp_exposed: bool,
}

impl ServiceDef {
    // ... existing builders unchanged ...

    /// Marks this projection as MCP-exposed.
    pub fn mcp_exposed(mut self, exposed: bool) -> Self {
        self.mcp_exposed = exposed;
        self
    }
}
```

`#[serde(default)]` means existing serialized `ServiceDef` JSON without this field
deserializes to `mcp_exposed: false` — backward compatible.

[VERIFIED: ServiceDef struct from ferro-projections/src/service.rs; builder pattern confirmed]

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness + `cargo test` |
| Config file | None (workspace-level `cargo test --all-features`) |
| Quick run command | `cargo test -p ferro-mcp-server` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File |
|--------|----------|-----------|-------------------|------|
| AMCP-01 | Exposed projection appears in tools/list; unmarked does not | unit | `cargo test -p ferro-mcp-server test_mcp_exposed_filter` | `ferro-mcp-server/src/renderer.rs` tests |
| AMCP-01 | `mcp_exposed` serde default=false backward compat | unit | `cargo test -p ferro-projections mcp_exposed_default` | `ferro-projections/src/service.rs` tests |
| AMCP-02 | inputSchema derived from ServiceDef fields; adding a field changes schema | unit | `cargo test -p ferro-mcp-server test_input_schema_derivation` | `ferro-mcp-server/src/schema.rs` tests |
| AMCP-02 | Sensitive fields never appear in inputSchema | unit | `cargo test -p ferro-mcp-server test_sensitive_field_excluded` | `ferro-mcp-server/src/schema.rs` tests |
| AMCP-02 | Non-readable fields excluded | unit | `cargo test -p ferro-mcp-server test_write_only_excluded` | `ferro-mcp-server/src/schema.rs` tests |
| AMCP-02 | Pagination params (limit/offset) always present | unit | `cargo test -p ferro-mcp-server test_pagination_params_in_schema` | `ferro-mcp-server/src/schema.rs` tests |
| AMCP-03 | Dispatch returns rows from fixture table | integration | `cargo test -p ferro-mcp-server test_dispatch_sqlite` | `ferro-mcp-server/src/dispatch.rs` tests |
| AMCP-04 | `McpRenderer` implements `Renderer` trait (compile-time) | compile | `cargo build -p ferro-mcp-server` | `ferro-mcp-server/src/renderer.rs` |
| AMCP-04 | `ferro-projections` has no dep on `ferro-mcp-server` (no dep in Cargo.toml) | structural | `cargo metadata` check | `ferro-projections/Cargo.toml` |
| SC-5 | ferro-mcp-server in workspace members and publish.yml | structural | `grep ferro-mcp-server Cargo.toml && grep ferro-mcp-server .github/workflows/publish.yml` | both files |

**Key test assertion for AMCP-02 (single source of truth guard):**

```rust
#[test]
fn adding_field_changes_schema() {
    let service = ServiceDef::new("order")
        .field("status", DataType::String, FieldMeaning::Status);
    let t1 = McpRenderer.render(&service, &derive_intents(&service), &McpContext::default()).unwrap();
    let props_before = t1.schema_as_json_value()["properties"].as_object().unwrap().len();

    let service2 = service.field("customer_id", DataType::Integer, FieldMeaning::ForeignKey);
    let t2 = McpRenderer.render(&service2, &derive_intents(&service2), &McpContext::default()).unwrap();
    let props_after = t2.schema_as_json_value()["properties"].as_object().unwrap().len();

    assert!(props_after > props_before, "adding a filter field must change the schema");
}
```

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-mcp-server && cargo test -p ferro-projections`
- **Per wave merge:** Full suite (`cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`)
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-mcp-server/src/` — entire crate (new; Wave 0 creates the scaffold)
- [ ] `ferro-mcp-server/Cargo.toml` — manifest
- [ ] `ferro-mcp-server/tests/dispatch_integration.rs` — SQLite in-memory dispatch test
- [ ] `ferro-projections/src/service.rs` — `mcp_exposed` field + builder + test
- Framework already installed: `cargo test` available

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Build | checked via workspace | via cargo | — |
| sea-orm (sqlx-sqlite) | Dispatch integration test | in workspace already | 1.0 | use feature gate |
| SQLite (in-memory) | Dispatch test | included via sqlx-sqlite | — | no fallback needed |
| rmcp 0.12 | Renderer types | in Cargo.lock | 0.12.0 | — |

No missing dependencies with no fallback.

---

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes | `is_filter_field` predicate; only allow-listed `FieldMeaning` variants pass |
| V2 Authentication | no | Phase 197 is in-process only; no bearer token validation until Phase 199 |
| V4 Access Control | no | No per-tenant scoping until Phase 200; Phase 197 has no ownership filter by design |
| V6 Cryptography | no | No cryptographic operations |

**Threat pattern — filter injection via equality predicates:**
The dispatch uses parameterized SQL (same as `crud_operations.rs`). Column names
are validated against model metadata (known field names from `ServiceDef.fields`).
Raw SQL construction using untrusted column names is not permitted — column names
come from `ServiceDef` (trusted) not from the call payload. Values are passed as
bind parameters.

---

## Open Questions

1. **Table name from ServiceDef**
   - What we know: `ServiceDef` has no `table` field; `crud_operations.rs` uses
     `model.table.unwrap_or_else(|| name + "s")`.
   - What's unclear: Phase 197 uses the same heuristic; irregular plurals will
     be wrong.
   - Recommendation: Add a `TODO` comment; defer a `ServiceDef.table` field to a
     follow-on. For Phase 197 the heuristic is acceptable for the skeleton.

2. **Dispatch result serialization to MCP `CallToolResult` content**
   - What we know: Phase 197 needs to serialize `Vec<serde_json::Value>` as MCP
     structured content. `rmcp::model` has `CallToolResult` and `Content` types.
   - What's unclear: Whether `ferro-mcp-server` should produce a `CallToolResult`
     (Phase 198 will need it) or just the raw Vec for the in-process test.
   - Recommendation: For Phase 197, the in-process test validates the Vec
     directly. Wrapping into `CallToolResult` is a Phase 198 task when the HTTP
     handler is wired.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hand-authored MCP tools per model | `McpRenderer` derives tool from `ServiceDef` | Phase 197 (this phase) | One source of truth; no parallel schema maintenance |
| `ferro-mcp` (dev-time stdio server only) | `ferro-mcp-server` (consumer-facing HTTP MCP, Phase 198) | Phase 197–198 | Splits authoring-time introspection from runtime tool serving |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Table name = `service.name + "s"` for dispatch SQL | D-05 / dispatch.rs pattern | A projection for an irregular-plural model returns empty or wrong results; mitigated by the TODO note |
| A2 | `ServiceDef.name` is always snake_case | tool name = `list_{service.name}` | Tool name would contain spaces or CamelCase; low risk given builder convention |

---

## Sources

### Primary (HIGH confidence)

- `ferro-projections/src/render/mod.rs` — `Renderer` trait, `BaseContext` [VERIFIED]
- `ferro-projections/src/service.rs` — `ServiceDef` struct + builder pattern [VERIFIED]
- `ferro-projections/src/field.rs` — `FieldDef`, `DataType`, `FieldMeaning`, `infer_meaning` [VERIFIED]
- `ferro-json-ui/src/projection/mod.rs` — `JsonUiRenderer` implementation precedent [VERIFIED]
- `ferro-json-ui/Cargo.toml` — manifest template [VERIFIED]
- `ferro-mcp/Cargo.toml` — existing `rmcp` workspace usage [VERIFIED]
- `ferro-mcp/src/tools/crud_operations.rs` — dispatch pattern source [VERIFIED]
- `ferro-mcp/src/service.rs` — rmcp `#[tool]` / `#[tool_router]` macro patterns [VERIFIED]
- `rmcp-0.12.0/src/model/tool.rs` — `Tool`, `ToolAnnotations` struct [VERIFIED]
- `rmcp-0.12.0/Cargo.toml` — feature flags [VERIFIED]
- `.github/workflows/publish.yml` lines 200–319 — publish waves [VERIFIED]
- `docs/superpowers/specs/2026-06-10-consumer-app-mcp-browser-login-design.md` — design spec [VERIFIED]

### Secondary (MEDIUM confidence)

- `.planning/REQUIREMENTS.md` §AMCP-01..04 [VERIFIED: read in full]
- `.planning/STATE.md` — active milestone context [VERIFIED]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all deps verified from Cargo.toml files and source
- Architecture: HIGH — Renderer trait verified; rmcp::model::Tool verified; VisualContext precedent verified
- Pitfalls: HIGH — verified from source code analysis (rmcp features, SQL pattern, FieldDef struct)

**Research date:** 2026-06-10
**Valid until:** 2026-07-10 (stable ecosystem; ferro-projections and rmcp are locked workspace versions)
