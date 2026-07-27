# Phase 263: Projection-native Inertia substrate — Pattern Map

**Mapped:** 2026-07-27
**Files analyzed:** 9 (new or modified)
**Analogs found:** 9 / 9

> **⚠ CORRECTION (2026-07-27, operator-approved): `Inertia::from_projection` placement.**
> This map was authored under RESEARCH assumption A1 (`ferro-inertia` could depend on
> `framework`). A1 is **false** — `framework` already depends on `ferro-inertia` (optional
> `inertia` feature), so `ferro-inertia → framework` is a hard Cargo cycle. **`from_projection`
> therefore lives on the framework-side Inertia facade: `framework/src/inertia/projection.rs`**
> (the `Request`-aware `Inertia` delivery module that already wraps `ferro_inertia::Inertia::render`),
> NOT the `ferro-inertia` crate. There is **no** `ferro-inertia/Cargo.toml` change (ferro-inertia
> gains no new deps). Wherever the tables/excerpts below say `ferro-inertia/src/projection.rs` or add
> deps to `ferro-inertia/Cargo.toml`, read `framework/src/inertia/projection.rs` and "no ferro-inertia
> Cargo change". The analog code excerpts (mirroring `Inertia::render`) still apply — only the crate
> home moves. See 263-04-PLAN.md Task 0. Cycle class matches Phase 261's `ferro-bundle`.

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-projections/src/schema_contract.rs` | pure-projection fn + types | transform | `ferro-projections/src/derive.rs` — `derive_intents` | exact |
| `ferro-projections/src/lib.rs` | re-export | — | self (extend) | exact |
| `framework/src/permitted_actions.rs` | utility fn | transform | `ferro-mcp-server/src/renderer.rs:229-233` guard loop | role-match |
| `framework/src/lib.rs` | re-export | — | self (extend) | exact |
| `framework/src/projection_read.rs` | query service | CRUD / request-response | `ferro-mcp-server/src/dispatch.rs` | exact (relocation) |
| `ferro-mcp-server/src/dispatch.rs` | thin wrapper | CRUD | `framework/src/projection_read.rs` (after move) | delegation |
| `ferro-mcp-server/src/renderer.rs` | modified | request-response | self (refactor render_action_tool) | exact |
| `ferro-inertia/src/projection.rs` | delivery helper | request-response | `ferro-inertia/src/response.rs` `Inertia::render` | role-match |
| `ferro-inertia/src/lib.rs` | re-export | — | self (extend) | exact |
| `ferro-inertia/Cargo.toml` | config | — | `ferro-mcp-server/Cargo.toml` dep pattern | role-match |
| `ferro-projections/tests/schema_contract.rs` | test | transform | `ferro-projections/tests/catalog.rs` | role-match |
| `app/src/tests/permitted_actions_parity.rs` | integration test | request-response | `app/src/tests/single_source.rs` | exact |
| `app/src/tests/data_tenant_scoping.rs` | integration test | CRUD | `ferro-mcp-server/src/dispatch.rs` inline tests (tenant_scoping) | exact |

---

## Pattern Assignments

### `ferro-projections/src/schema_contract.rs` (pure projection fn + types, transform)

**Analog:** `ferro-projections/src/derive.rs`

**Why this analog:** `derive_intents(&ServiceDef) -> Vec<IntentScore>` is the direct structural sibling: pure function, takes `&ServiceDef`, returns an owned value, zero side effects, zero async, no non-`std`/non-`serde` deps. `schema_contract(&ServiceDef) -> SchemaContract` follows the identical pattern.

**Imports pattern** (`derive.rs` lines 1-8):
```rust
use std::collections::HashMap;

use crate::field::FieldMeaning;
use crate::intent::{Intent, IntentHint, IntentScore};
use crate::relationship::{Cardinality, NavigationHint};
use crate::render::is_system_field;
use crate::service::ServiceDef;
```

For `schema_contract.rs`, use only what is needed — no `HashMap`, no intent/relationship imports:
```rust
use serde::{Deserialize, Serialize};

use crate::action::{ActionDef, InputDef};
use crate::field::{DataType, FieldDef, FieldMeaning};
use crate::service::ServiceDef;
```

**Core function pattern** (`derive.rs` lines 75-113):
```rust
/// Derives ranked intents from a ServiceDef's structural signals.
///
/// Always returns at least one IntentScore. Default: Focus with 0.5 confidence.
pub fn derive_intents(service: &ServiceDef) -> Vec<IntentScore> {
    // 1. Collect signals from all 5 analyzers.
    // ...
    scores
}
```

Mirror exactly for `schema_contract`:
```rust
/// Derives the schema contract from a service definition.
///
/// Pure derivation — no runtime deps, no async, no side effects.
/// Returns a `SchemaContract` describing fields, their access modes,
/// action definitions, and declared guards.
pub fn schema_contract(service: &ServiceDef) -> SchemaContract {
    SchemaContract {
        name: service.name.clone(),
        display_name: service.display_name.clone(),
        fields: service.fields.iter().map(FieldContract::from).collect(),
        actions: service.actions.iter().map(ActionContract::from).collect(),
        guards: service.guards.iter().map(|g| g.name.clone()).collect(),
        has_state_machine: service.state_machine.is_some(),
    }
}
```

**Output type pattern** — all types must derive `Serialize, Deserialize, Debug, Clone` (crate convention from `CLAUDE.md`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaContract {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub fields: Vec<FieldContract>,
    pub actions: Vec<ActionContract>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guards: Vec<String>,
    pub has_state_machine: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldContract {
    pub name: String,
    pub data_type: DataType,        // already Serialize/Deserialize
    pub meaning: FieldMeaning,      // already Serialize/Deserialize
    pub required: bool,
    pub readable: bool,
    pub writable: bool,
    pub is_list: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionContract {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub preconditions: Vec<String>,
    pub inputs: Vec<InputContract>,
    pub is_transition: bool,        // transition_trigger.is_some()
}
```

**`FieldContract::from` helper** — use `ServiceDef::is_write_excluded_field` to tag fields correctly:
```rust
// Reference: ferro-projections/src/service.rs:254-282
impl From<&FieldDef> for FieldContract {
    fn from(f: &FieldDef) -> Self {
        FieldContract {
            name: f.name.clone(),
            data_type: f.data_type.clone(),
            meaning: f.meaning.clone(),
            required: f.required,
            readable: f.readable,
            writable: f.writable,
            is_list: f.is_list,
        }
    }
}
```

**`serde` enum convention** (`derive.rs` / `action.rs` patterns):
```rust
// Enums: #[serde(rename_all = "snake_case")] — see ActionDef, FieldMeaning
```

**Test pattern** (mirror `catalog.rs` serde round-trip style):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ActionDef, DataType, FieldMeaning, GuardDef, ServiceDef};

    #[test]
    fn schema_contract_field_set() {
        let service = ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("total", DataType::Float, FieldMeaning::Money);
        let contract = schema_contract(&service);
        assert_eq!(contract.name, "order");
        assert_eq!(contract.fields.len(), 2);
    }

    #[test]
    fn schema_contract_serde_round_trip() {
        let service = ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier);
        let contract = schema_contract(&service);
        let json = serde_json::to_string(&contract).unwrap();
        let parsed: SchemaContract = serde_json::from_str(&json).unwrap();
        assert_eq!(contract.name, parsed.name);
    }
}
```

---

### `ferro-projections/src/lib.rs` (re-export, extend)

**Analog:** Current `ferro-projections/src/lib.rs` (lines 1-28).

**Existing pattern** (lines 14-27):
```rust
pub use action::{ActionDef, GuardDef, InputDef};
pub use derive::derive_intents;
// ...
pub use service::{FieldMetadata, ModelMetadata, ServiceDef};
```

**Extension** — add below `pub use derive::derive_intents;`:
```rust
mod schema_contract;
pub use schema_contract::{schema_contract, ActionContract, FieldContract, SchemaContract};
```

---

### `framework/src/permitted_actions.rs` (utility fn, transform)

**Analog:** `ferro-mcp-server/src/renderer.rs` lines 229-233 (guard-visibility loop).

**Extraction source** (renderer.rs:224-233):
```rust
fn render_action_tool(
    service: &ServiceDef,
    action: &ActionDef,
    ctx: &McpContext,
) -> std::result::Result<Option<Tool>, ProjError> {
    for precondition in &action.preconditions {
        if ctx.evaluated_guards.get(precondition) == Some(&false) {
            return Ok(None);  // hide this action tool
        }
    }
    // ...
}
```

**Lifted function:**
```rust
use std::collections::HashMap;

use ferro_projections::ServiceDef;

/// Returns the names of actions in `service` whose preconditions are not
/// explicitly denied by `evaluated_guards`.
///
/// Semantics: absent key = allow (default-open); `Some(false)` = deny.
/// This is a list-time VISIBILITY filter evaluated once per request from
/// a pre-computed guard map. Per-record enforcement happens at
/// `dispatch_write` time via the live `GuardEvaluatorFn`.
///
/// Both the MCP `tools/list` surface and the Inertia `from_projection`
/// delivery call this function — it is the single guard-evaluation site.
pub fn permitted_actions(
    service: &ServiceDef,
    evaluated_guards: &HashMap<String, bool>,
) -> Vec<String> {
    service
        .actions
        .iter()
        .filter(|action| {
            !action
                .preconditions
                .iter()
                .any(|p| evaluated_guards.get(p) == Some(&false))
        })
        .map(|a| a.name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_projections::{ActionDef, GuardDef, ServiceDef, DataType, FieldMeaning};

    #[test]
    fn hides_action_when_guard_is_false() {
        let service = ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .guard(GuardDef::new("is_manager"))
            .action(ActionDef::new("approve").precondition("is_manager"))
            .action(ActionDef::new("submit"));
        let guards = [("is_manager".to_string(), false)].into_iter().collect();
        let allowed = permitted_actions(&service, &guards);
        assert!(!allowed.contains(&"approve".to_string()));
        assert!(allowed.contains(&"submit".to_string()));
    }

    #[test]
    fn absent_guard_key_allows_action() {
        let service = ServiceDef::new("order")
            .guard(GuardDef::new("is_manager"))
            .action(ActionDef::new("approve").precondition("is_manager"));
        let guards = HashMap::new(); // absent = allow
        let allowed = permitted_actions(&service, &guards);
        assert!(allowed.contains(&"approve".to_string()));
    }

    #[test]
    fn explicit_true_allows_action() {
        let service = ServiceDef::new("order")
            .guard(GuardDef::new("is_manager"))
            .action(ActionDef::new("approve").precondition("is_manager"));
        let guards = [("is_manager".to_string(), true)].into_iter().collect();
        let allowed = permitted_actions(&service, &guards);
        assert!(allowed.contains(&"approve".to_string()));
    }
}
```

**Feature gate** — this function depends on `ferro-projections` which is already gated on the `projections` feature in `framework/Cargo.toml`. Place the module inside `#[cfg(feature = "projections")]` consistent with how `pub mod write` is gated (framework/src/lib.rs line 58-59):
```rust
// framework/src/lib.rs — add inside the projections feature block
#[cfg(feature = "projections")]
pub mod permitted_actions;
#[cfg(feature = "projections")]
pub use permitted_actions::permitted_actions;
```

---

### `framework/src/projection_read.rs` (query service, CRUD, relocated)

**Analog:** `ferro-mcp-server/src/dispatch.rs` — exact relocation.

**Key signature to preserve** (dispatch.rs lines 117-124):
```rust
pub async fn dispatch(
    service: &ServiceDef,
    mut filters: serde_json::Value,
    limit: u64,
    offset: u64,
    db: &sea_orm::DatabaseConnection,
    tenant_id: Option<i64>,
) -> crate::Result<DispatchResult>
```

**`DispatchResult` struct** (dispatch.rs lines 19-25):
```rust
#[derive(Debug, Serialize)]
pub struct DispatchResult {
    pub rows: Vec<serde_json::Value>,
    pub total: u64,
    pub limit: u64,
    pub offset: u64,
}
```

**Error type adaptation** — `dispatch` currently returns `crate::Result<DispatchResult>` where `crate` is `ferro-mcp-server`. After relocation, introduce a `ProjectionReadError` in `framework` or re-use a new variant. The lowest-risk path matching the research recommendation: define `ProjectionReadError` in `framework/src/projection_read.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProjectionReadError {
    #[error("invalid filter: {0}")]
    InvalidFilter(String),
    #[error("database error: {0}")]
    Database(String),
}

pub type ProjectionReadResult<T> = Result<T, ProjectionReadError>;
```

Then `ferro-mcp-server/src/dispatch.rs` becomes a thin delegation:
```rust
// ferro-mcp-server/src/dispatch.rs (after relocation)
pub use framework::projection_read::{DispatchResult, ProjectionReadError};

pub async fn dispatch(
    service: &ServiceDef,
    filters: serde_json::Value,
    limit: u64,
    offset: u64,
    db: &sea_orm::DatabaseConnection,
    tenant_id: Option<i64>,
) -> crate::Result<DispatchResult> {
    framework::projection_read::dispatch(service, filters, limit, offset, db, tenant_id)
        .await
        .map_err(|e| match e {
            ProjectionReadError::InvalidFilter(m) => crate::Error::InvalidFilter(m),
            ProjectionReadError::Database(m) => crate::Error::Database(m),
        })
}
```

**Internal helpers to move verbatim** (dispatch.rs lines 1-115): `MAX_LIMIT`, `MAX_OFFSET`, `placeholder()`, `json_to_sea_value()`, `split_op_key()`, `rows_to_json()` — all pure functions, no `crate::` type references other than the error type. Move them unchanged.

**Feature gate:** `projection_read.rs` uses `sea-orm` which framework already has. Gate it under `projections` feature, same as `permitted_actions` and `write`:
```rust
// framework/src/lib.rs
#[cfg(feature = "projections")]
pub mod projection_read;
#[cfg(feature = "projections")]
pub use projection_read::{DispatchResult, ProjectionReadError, ProjectionReadResult};
```

---

### `ferro-mcp-server/src/renderer.rs` (modified, refactor render_action_tool)

**Analog:** Current `ferro-mcp-server/src/renderer.rs` — self-refactor.

**Before** (lines 229-233):
```rust
fn render_action_tool(
    service: &ServiceDef,
    action: &ActionDef,
    ctx: &McpContext,
) -> std::result::Result<Option<Tool>, ProjError> {
    for precondition in &action.preconditions {
        if ctx.evaluated_guards.get(precondition) == Some(&false) {
            return Ok(None);
        }
    }
    // ... build the Tool ...
}
```

**After** (call framework::permitted_actions, check membership):
```rust
fn render_action_tool(
    service: &ServiceDef,
    action: &ActionDef,
    ctx: &McpContext,
) -> std::result::Result<Option<Tool>, ProjError> {
    let allowed = ferro_rs::permitted_actions(service, &ctx.evaluated_guards);
    if !allowed.contains(&action.name) {
        return Ok(None);
    }
    // ... build the Tool (unchanged) ...
}
```

The behavior is identical: `permitted_actions` returns action names not denied, `!allowed.contains(&action.name)` is the complement of "any precondition is `Some(false)`".

**Import addition** at top of renderer.rs:
```rust
// already: use ferro_projections::{ActionDef, Error as ProjError, ...};
// add: ferro_rs is already a dep; permitted_actions is re-exported from it
use ferro_rs::permitted_actions;
```

---

### `ferro-inertia/src/projection.rs` (delivery helper, request-response)

**Analog:** `ferro-inertia/src/response.rs` — `Inertia::render` as the call target.

**`Inertia::render` signature** (response.rs line 148-154):
```rust
pub fn render<R, P>(req: &R, component: &str, props: P) -> InertiaHttpResponse
where
    R: InertiaRequest,
    P: Serialize,
```

**New `from_projection` — mirrors the render call with assembled props:**
```rust
use std::collections::HashMap;

use sea_orm::DatabaseConnection;
use serde::Serialize;
use serde_json::Value;

use ferro_projections::{ServiceDef, schema_contract, SchemaContract};
use framework::projection_read::{dispatch, DispatchResult};
use framework::permitted_actions;

use crate::request::InertiaRequest;
use crate::response::{Inertia, InertiaHttpResponse};

/// Query parameters for a projection-driven Inertia page.
///
/// Default: `limit = 25`, `offset = 0`, `filters = {}`.
#[derive(Debug, Clone)]
pub struct ProjectionQuery {
    pub filters: Value,
    pub limit: u64,
    pub offset: u64,
}

impl Default for ProjectionQuery {
    fn default() -> Self {
        Self {
            filters: Value::Object(Default::default()),
            limit: 25,
            offset: 0,
        }
    }
}

/// Props shape delivered to the Inertia component.
#[derive(Debug, Serialize)]
struct ProjectionProps {
    schema: SchemaContract,
    data: Vec<Value>,
    permitted_actions: Vec<String>,
    total: u64,
    limit: u64,
    offset: u64,
}

impl Inertia {
    /// Render an Inertia response from a projection declaration.
    ///
    /// Assembles `{ schema, data, permitted_actions, total, limit, offset }` props
    /// from `service` and `evaluated_guards`, then delegates to `Inertia::render`.
    ///
    /// `permitted_actions` is a per-request list (not per-record). The actual
    /// per-record guard enforcement happens at `dispatch_write` time.
    pub async fn from_projection<R: InertiaRequest>(
        req: &R,
        component: &str,
        service: &ServiceDef,
        query: ProjectionQuery,
        db: &DatabaseConnection,
        tenant_id: Option<i64>,
        evaluated_guards: &HashMap<String, bool>,
    ) -> InertiaHttpResponse {
        let schema = schema_contract(service);
        let actions = permitted_actions(service, evaluated_guards);

        let result = match dispatch(
            service,
            query.filters,
            query.limit,
            query.offset,
            db,
            tenant_id,
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                // Return a 500 Inertia error page on data-query failure.
                return Inertia::render(
                    req,
                    component,
                    serde_json::json!({ "error": e.to_string() }),
                );
            }
        };

        let props = ProjectionProps {
            schema,
            data: result.rows,
            permitted_actions: actions,
            total: result.total,
            limit: result.limit,
            offset: result.offset,
        };

        Inertia::render(req, component, props)
    }
}
```

**`ferro-inertia/Cargo.toml` additions** (follow `ferro-mcp-server/Cargo.toml` dep pattern):
```toml
[dependencies]
# existing
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
# new — mirror ferro-mcp-server's pattern for projection deps
ferro-rs = { path = "../framework", version = "0.2", default-features = false, features = ["projections"] }
ferro-projections = { path = "../ferro-projections", version = "0.2" }
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-rustls"] }
```

**`ferro-inertia/src/lib.rs` extension** (mirror current pattern at line 57-63):
```rust
// existing
mod config;
mod manifest;
mod request;
mod response;
mod shared;
// add
mod projection;

pub use config::InertiaConfig;
pub use request::InertiaRequest;
pub use response::{Inertia, InertiaHttpResponse, InertiaResponse};
pub use shared::InertiaShared;
pub use projection::ProjectionQuery;  // from_projection is on Inertia struct directly
```

---

## Test Patterns

### `ferro-projections/tests/schema_contract.rs` (snapshot test, transform)

**Analog:** `ferro-projections/tests/catalog.rs`

**Catalog test structure** (catalog.rs lines 21-25 imports, then fixture pattern):
```rust
use ferro_projections::derive_intents;
use ferro_projections::{
    ActionDef, Cardinality, DataType, FieldMeaning, GuardDef, Intent, ...
    ServiceDef, StateDef, StateMachine, Transition,
};
```

**Schema contract test follows the same structure:**
```rust
//! Snapshot test for `schema_contract` (SUBST-01).

use ferro_projections::{
    schema_contract, ActionDef, DataType, FieldMeaning, GuardDef, InputDef, ServiceDef,
    StateDef, StateMachine, Transition,
};

#[test]
fn schema_contract_field_names_and_access() {
    let service = ServiceDef::new("order")
        .read_only_field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("customer_name", DataType::String, FieldMeaning::EntityName)
        .optional_field("notes", DataType::String, FieldMeaning::FreeText);

    let contract = schema_contract(&service);
    assert_eq!(contract.fields.len(), 3);
    let id = &contract.fields[0];
    assert!(!id.writable);
    assert!(id.readable);
    let name = &contract.fields[1];
    assert!(name.writable);
}

#[test]
fn schema_contract_actions_and_preconditions() {
    let service = ServiceDef::new("order")
        .guard(GuardDef::new("is_manager"))
        .action(ActionDef::new("approve").precondition("is_manager"))
        .action(ActionDef::new("submit"));

    let contract = schema_contract(&service);
    assert_eq!(contract.actions.len(), 2);
    assert_eq!(contract.guards, vec!["is_manager"]);
    let approve = &contract.actions[0];
    assert_eq!(approve.preconditions, vec!["is_manager"]);
}

#[test]
fn schema_contract_serde_round_trip() {
    let service = ServiceDef::new("order")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .guard(GuardDef::new("g"))
        .action(ActionDef::new("act").precondition("g"));
    let c = schema_contract(&service);
    let json = serde_json::to_string(&c).unwrap();
    let parsed: ferro_projections::SchemaContract = serde_json::from_str(&json).unwrap();
    assert_eq!(c.name, parsed.name);
    assert_eq!(c.fields.len(), parsed.fields.len());
}
```

---

### `app/src/tests/permitted_actions_parity.rs` (integration test, parity)

**Analog:** `app/src/tests/single_source.rs`

**Single-source test structure** (single_source.rs lines 24-40):
```rust
#[cfg(all(test, not(feature = "confirmation")))]
mod tests {
    use crate::migrations::Migrator;
    use ferro::serde_json::json;
    use ferro::write::{dispatch_write, WriteDispatcher, WriteError};
    use ferro_mcp_server::{handle_tools_call, McpContext};
    use sea_orm::{...};

    async fn setup_db() -> DatabaseConnection { ... }
    async fn seed(db: &DatabaseConnection) { ... }
    fn order_service() -> ferro::ServiceDef { ... }
    // ...
    #[tokio::test]
    async fn single_source_both_channels() { ... }
}
```

**Parity test mirrors this structure:**
```rust
//! Permitted-actions parity test (SUBST-02 / SUBST-05).
//!
//! Asserts that `framework::permitted_actions(service, &guards)` returns the
//! same set as the guard-filtered MCP tool list from `render_exposed_tools`.

#[cfg(all(test, not(feature = "confirmation")))]
mod tests {
    use std::collections::HashMap;

    use ferro::permitted_actions;
    use ferro_mcp_server::{render_exposed_tools, McpContext};
    use ferro_projections::{ActionDef, GuardDef, ServiceDef, DataType, FieldMeaning};

    fn order_service_with_guards() -> ServiceDef {
        // Mirror the ServiceDef builder pattern from single_source.rs
        ServiceDef::new("order")
            .mcp_exposed(true)
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .guard(GuardDef::new("is_manager"))
            .action(ActionDef::new("approve").precondition("is_manager"))
            .action(ActionDef::new("submit"))
    }

    #[test]
    fn permitted_actions_matches_mcp_tools_list() {
        let service = order_service_with_guards();
        let guards: HashMap<String, bool> = [("is_manager".to_string(), false)].into();

        let ctx = McpContext {
            evaluated_guards: guards.clone(),
            ..Default::default()
        };

        // Inertia path: framework::permitted_actions
        let inertia_allowed = permitted_actions(&service, &guards);
        assert!(!inertia_allowed.contains(&"approve".to_string()));
        assert!(inertia_allowed.contains(&"submit".to_string()));

        // MCP path: render_exposed_tools → tool names (excluding list_ read tools)
        let tools = render_exposed_tools(&[service], &ctx).expect("render ok");
        let tool_names: Vec<&str> = tools
            .iter()
            .filter(|t| !t.name.starts_with("list_"))
            .map(|t| t.name.as_ref())
            .collect();
        assert!(!tool_names.contains(&"approve"), "approve must be hidden from MCP too");
        assert!(tool_names.contains(&"submit"), "submit must be visible in MCP too");
    }

    #[test]
    fn state_change_updates_both_surfaces_identically() {
        let service = order_service_with_guards();

        // Guard false: approve hidden everywhere.
        let guards_deny: HashMap<String, bool> = [("is_manager".to_string(), false)].into();
        let inertia_deny = permitted_actions(&service, &guards_deny);
        let ctx_deny = McpContext { evaluated_guards: guards_deny, ..Default::default() };
        let tools_deny = render_exposed_tools(&[service.clone()], &ctx_deny).unwrap();

        // Guard absent (allow): approve visible everywhere.
        let guards_allow: HashMap<String, bool> = HashMap::new();
        let inertia_allow = permitted_actions(&service, &guards_allow);
        let ctx_allow = McpContext { evaluated_guards: guards_allow, ..Default::default() };
        let tools_allow = render_exposed_tools(&[service], &ctx_allow).unwrap();

        assert!(!inertia_deny.contains(&"approve".to_string()));
        assert!(inertia_allow.contains(&"approve".to_string()));

        let mcp_deny_names: Vec<_> = tools_deny.iter().filter(|t| !t.name.starts_with("list_")).map(|t| t.name.as_ref()).collect();
        let mcp_allow_names: Vec<_> = tools_allow.iter().filter(|t| !t.name.starts_with("list_")).map(|t| t.name.as_ref()).collect();
        assert!(!mcp_deny_names.contains(&"approve"));
        assert!(mcp_allow_names.contains(&"approve"));
    }
}
```

---

### `app/src/tests/data_tenant_scoping.rs` (integration test, CRUD)

**Analog:** `ferro-mcp-server/src/dispatch.rs` tests (lines 354-490 — `setup_orders_db`, `tenant_scoping`, `tenant_isolation`, `tenant_fail_closed`).

Check `app/src/tests/crud_e2e.rs` first; if `seed_two_tenants` + a `dispatch` call already covers the scenario, extend that file rather than creating a new one.

**Pattern to replicate** (dispatch.rs lines 360-475):
```rust
async fn setup_orders_db() -> sea_orm::DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.expect("sqlite connect");
    // CREATE TABLE + INSERT rows for two tenants
    db
}

fn order_service_with_tenant() -> ServiceDef {
    ServiceDef::new("order")
        .mcp_exposed(true)
        .tenant_column("tenant_id")
        // ...fields...
}

#[tokio::test]
async fn tenant_scoping() {
    let db = setup_orders_db().await;
    let service = order_service_with_tenant();
    let result = dispatch(&service, serde_json::json!({}), 10, 0, &db, Some(1))
        .await
        .expect("dispatch ok");
    assert_eq!(result.rows.len(), 2);
    for row in &result.rows {
        assert_eq!(row["tenant_id"].as_i64().unwrap(), 1);
    }
}
```

For the app-level test, import from `framework::projection_read` (after relocation) rather than `ferro_mcp_server::dispatch`, and use the app's Migrator for DB setup (matching `single_source.rs` / `crud_e2e.rs` pattern):
```rust
use framework::projection_read::dispatch;
use crate::migrations::Migrator;

async fn setup_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");
    db
}
```

---

## Shared Patterns

### Builder + consuming method chaining
**Source:** `ferro-projections/src/service.rs` (any `with_*` / builder method)
**Apply to:** Any new config structs (`ProjectionQuery`)
```rust
// All builder methods: `mut self -> Self` (consuming)
pub fn limit(mut self, n: u64) -> Self {
    self.limit = n;
    self
}
```

### `serde` skip directives
**Source:** `ferro-projections/src/service.rs` lines 65-113 (ServiceDef fields)
**Apply to:** `SchemaContract`, `FieldContract`, `ActionContract`
```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub display_name: Option<String>,
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub guards: Vec<String>,
```

### Feature-gating new framework modules
**Source:** `framework/src/lib.rs` lines 58-59
**Apply to:** `permitted_actions` and `projection_read` modules
```rust
#[cfg(feature = "projections")]
pub mod permitted_actions;
#[cfg(feature = "projections")]
pub use permitted_actions::permitted_actions;
```

### Error handling in delivery helpers
**Source:** `ferro-inertia/src/response.rs` (pattern: return a rendered error page, not panic)
**Apply to:** `Inertia::from_projection` data-query failure path
```rust
Err(e) => {
    return Inertia::render(req, component, serde_json::json!({ "error": e.to_string() }));
}
```

### Acyclic dependency constraint (enforced throughout)
```
ferro-projections  (leaf — serde, schemars, thiserror only)
    ↑
framework          (projections feature: ferro-projections, sea-orm, ferro-audit)
    ↑
ferro-mcp-server   (depends on framework, ferro-projections, rmcp, sea-orm)
ferro-inertia      (depends on framework, ferro-projections, sea-orm)
```

`ferro-inertia` must NOT depend on `ferro-mcp-server`. The `dispatch` relocation to `framework::projection_read` is what makes the Inertia → framework dependency acyclic.

---

## No Analog Found

No files are entirely without analog. All patterns have direct precedents in the codebase.

---

## Dependency Changes Required

| Crate | Change | Reason |
|---|---|---|
| `ferro-inertia/Cargo.toml` | Add `ferro-rs` (projections feature), `ferro-projections`, `sea-orm` | `from_projection` needs `schema_contract`, `permitted_actions`, `dispatch` |
| `framework/Cargo.toml` | No new dep — `ferro-projections` and `sea-orm` already present under `projections` feature | `projection_read` and `permitted_actions` use existing deps |
| `ferro-mcp-server/Cargo.toml` | No change — already depends on `ferro-rs` | Calls `ferro_rs::permitted_actions` via existing dep |

---

## Pitfall Reminders for Executor

1. **Wave ordering is mandatory:** Wave 3 (relocate `dispatch`) must complete before Wave 4 (`ferro-inertia` writes `from_projection`). `ferro-inertia` must never import from `ferro-mcp-server`.

2. **`dispatch.rs` error type mapping:** When relocating, switch the error type to `ProjectionReadError` in `framework`. The `ferro-mcp-server` thin wrapper maps `ProjectionReadError` → `crate::Error` via `From` or `match`.

3. **`permitted_actions` is NOT the live guard evaluator:** Document this clearly in the Rustdoc. It reads a pre-computed `HashMap<String, bool>`, not live DB state. Per-record guard enforcement is `GuardEvaluatorFn` in `dispatch_write`.

4. **`ferro-projections` stays runtime-free:** `SchemaContract` and `schema_contract` must be synchronous, `Serialize`/`Deserialize` only. No `tokio`, no `sea-orm`, no `async`.

5. **MCP regression test before merging Wave 2:** Run `cargo test -p ferro-mcp-server` after refactoring `render_action_tool`. The behavior must be identical — the only change is which compilation unit owns the guard-visibility logic.

---

## Metadata

**Analog search scope:** `ferro-projections/src/`, `framework/src/write/`, `ferro-mcp-server/src/`, `ferro-inertia/src/`, `app/src/tests/`
**Files scanned:** 14 source files
**Pattern extraction date:** 2026-07-27
