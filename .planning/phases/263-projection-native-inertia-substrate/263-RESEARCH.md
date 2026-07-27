# Phase 263: Projection-native Inertia substrate — Research

**Researched:** 2026-07-27
**Domain:** ferro-projections / ferro-inertia / framework::write / ferro-mcp-server
**Confidence:** HIGH — all findings verified against actual source files in this session

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

Everything in the anchor spec (`docs/superpowers/specs/2026-07-27-headless-projection-substrate-design.md`) is a locked decision:

- `schema_contract(&ServiceDef) -> SchemaContract` lives in `ferro-projections` (sibling of `derive_intents`). Renders nothing.
- The one refactor: lift `permitted_actions(...)` guard-visibility logic from `ferro-mcp-server` into `framework`. After the lift both `ferro-mcp-server` and `ferro-inertia` call `framework`'s function.
- `Inertia::from_projection(req, service, query) -> InertiaResponse` lives in `ferro-inertia` (output-crate delivery rule).
- Write path: Inertia `POST /{service}/{action}` → existing `dispatch_write(.., channel = "web")`. No new write path.
- Auth: Inertia reuses same-origin session/CSRF. Nothing new.
- Error handling: reuse the framework JSON error envelope and `WriteError` mapping.

### Claude's Discretion

- `SchemaContract` location — `ferro-projections` (leaning; pure, renders nothing) vs. `ferro-inertia`.
- Opt-in surface — a builder flag on the projection vs. a registration list mirroring MCP `exposed_services()`.
- Permitted-actions placement in props — per-record inline vs. a separate lookup.
- Pagination contract for the data query — cursor vs. offset; default/max limit.
- Whether the `tools/list` filter can be lifted cleanly into `framework` — confirm with code in hand.

### Deferred Ideas (OUT OF SCOPE)

- Projection-native JSX/React component derivation (scaffolds, typed hooks).
- Design-lint on generated frontends.
- Generic JSON delivery mode (`GET /project/{service}`, `GET /data/{service}`).
- New authentication model.
- Any change to JSON-UI's role.
- Any new write path.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SUBST-01 | Pure `schema_contract(&ServiceDef) -> SchemaContract` in `ferro-projections` — field set, meanings, validations, action definitions. Snapshot-tested. | `FieldDef`, `FieldMeaning`, `ActionDef`, `InputDef` are all serde-serializable and already carry the necessary data. `derive_intents()` is the direct structural sibling pattern. No runtime deps in `ferro-projections`. |
| SUBST-02 | Lift `permitted_actions(service, record, tenant, ctx)` from `ferro-mcp-server` into `framework`. Both MCP and Inertia call the same function after the lift. | The current guard-visibility code is a simple `evaluated_guards` map lookup in `render_action_tool` (renderer.rs:229-233). The actual runtime evaluator (`GuardEvaluatorFn`) and `merged_guards()` already live in `framework/src/write/mod.rs`. |
| SUBST-03 | `Inertia::from_projection(req, service, query) -> InertiaResponse` in `ferro-inertia` — tenant-scoped data + `SchemaContract` + per-record `permitted_actions`. | `ferro-mcp-server::dispatch` (dispatch.rs) provides the exact data-query pattern to replicate. `Inertia::render` takes any `Serialize` props. |
| SUBST-04 | Inertia `POST /{service}/{action}` → existing `dispatch_write(.., channel = "web")`. | The visual route `post!("/{service}/{action}", controllers::visual_action::handle)` (app/src/routes.rs:127) already does exactly this. Inertia just needs a route at the same shape or alongside it. |
| SUBST-05 | Parity tests: permitted-actions parity (Inertia vs MCP tools/list), write parity, schema snapshot, data tenant-scoping. | The `single_source_both_channels` test pattern in `app/src/tests/single_source.rs` is the exact template to extend. |
</phase_requirements>

---

## Summary

Phase 263 adds three derivations to the projection system, one refactor, and one delivery helper. All five are closely grounded in code that already exists.

**Schema derivation (SUBST-01):** `ferro-projections` already contains all the data needed — `ServiceDef.fields` (with `FieldDef.name`, `.meaning`, `.required`, `.data_type`, `.readable`, `.writable`), `ServiceDef.actions` (with `ActionDef.inputs`, `.preconditions`), and `ServiceDef.guards`. A pure `schema_contract(&ServiceDef) -> SchemaContract` function sits naturally next to `derive_intents()`. No dependency changes required; `ferro-projections` has zero runtime deps.

**Guard-visibility refactor (SUBST-02):** The current `tools/list` guard filter (`render_action_tool` in `ferro-mcp-server/src/renderer.rs`) is a five-line loop over `action.preconditions` that checks `ctx.evaluated_guards.get(precondition) == Some(&false)`. This logic is a **visibility-only** pre-evaluation cache, NOT the live `GuardEvaluatorFn`. The actual live evaluator is a `Box<dyn Fn(...)>` that lives in `WriteDispatcher` and is called by `dispatch_write`. The refactor extracts the `evaluated_guards` lookup pattern into a `framework::permitted_actions(service, record, evaluated_guards) -> Vec<String>` function — essentially the complement logic (return action names whose guards are NOT `Some(false)`). This is acyclic: `ferro-mcp-server` depends on `ferro-rs`/`framework` already.

**Data query (SUBST-03):** The `ferro-mcp-server::dispatch` function (`ferro-mcp-server/src/dispatch.rs`) is the existing tenant-scoped read path. It takes `service: &ServiceDef`, `filters: serde_json::Value`, `limit: u64`, `offset: u64`, `db: &DatabaseConnection`, `tenant_id: Option<i64>` and returns `DispatchResult { rows, total, limit, offset }`. `Inertia::from_projection` in `ferro-inertia` needs access to the same query. The cleanest path is to re-export or re-use `ferro-mcp-server::dispatch` from `ferro-inertia`, OR move the dispatch function into `framework` and have both callers use it. The dependency question is discussed in open question resolutions below.

**Delivery (SUBST-03):** `Inertia::render` in `ferro-inertia/src/response.rs` takes any `P: Serialize` as props. `Inertia::from_projection` assembles `{ schema: SchemaContract, data: Vec<Value>, permitted_actions: HashMap<String, Vec<String>> }` and calls `Inertia::render` internally.

**Writes (SUBST-04):** The visual write route (`app/src/routes.rs:127`, `controllers::visual_action::handle`) already calls `dispatch_write(.., "web")`. Inertia form posts to the same URL shape. No new code needed at the `dispatch_write` layer — the planner only needs to ensure the Inertia page posts to `POST /{service}/{action}`.

**Primary recommendation:** Build in this order: (1) `schema_contract` in `ferro-projections`, (2) `permitted_actions` in `framework`, (3) move `dispatch` (data query) into `framework` to break the `ferro-inertia → ferro-mcp-server` cycle, (4) `Inertia::from_projection` in `ferro-inertia`, (5) write reuse (already exists), (6) parity tests.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Schema derivation (`SchemaContract`) | `ferro-projections` | — | Pure schema projection, no output target. Sibling of `derive_intents`. |
| Guard-visibility function (`permitted_actions`) | `framework` | — | Must be shared by MCP and Inertia without `ferro-inertia` depending on `ferro-mcp-server`. Already owns `GuardEvaluatorFn` and `merged_guards`. |
| Tenant-scoped data query | `framework` (relocated from `ferro-mcp-server::dispatch`) | `ferro-mcp-server` re-uses | Breaking the dep cycle requires the query to live above both callers. |
| Inertia delivery helper (`from_projection`) | `ferro-inertia` | — | Renderer-location rule: delivery helpers live in the output crate. |
| Write dispatch | `framework::write::dispatch_write` | — | Already channel-agnostic; Inertia passes `channel = "web"`. |
| Parity tests | `app/src/tests/` | `ferro-projections/tests/` | Single-source tests belong in the sample app (integration) or in the declaring crate (schema snapshot). |

---

## Codebase Findings

### 1. Write kernel — `framework/src/write/mod.rs`

**Exact signature of `dispatch_write`:**
```rust
pub async fn dispatch_write(
    action: &ActionDef,
    inputs: &Value,
    tenant_id: i64,
    db: &DatabaseConnection,
    dispatcher: &WriteDispatcher,
    transition_guard: Option<&str>,
    channel: &str,
    #[cfg(feature = "confirmation")] is_confirmed: bool,
    crud_plan: Option<&CrudPlan>,
) -> WriteResult<Value>
```
[VERIFIED: framework/src/write/mod.rs:629]

**`GuardEvaluatorFn` type:**
```rust
pub type GuardEvaluatorFn = Box<
    dyn Fn(
            &str,   // guard_name
            i64,    // tenant_id
            &Value, // validated inputs (for record-scoped guards)
            &DatabaseConnection,
        ) -> Pin<Box<dyn Future<Output = WriteResult<bool>> + Send>>
        + Send
        + Sync,
>;
```
[VERIFIED: framework/src/write/mod.rs:104]

**`merged_guards` is already public:**
```rust
pub fn merged_guards(preconditions: &[String], transition_guard: Option<&str>) -> Vec<String>
```
[VERIFIED: framework/src/write/mod.rs:498]

**Channel parameter:** Callers pass the string literal directly. The MCP framing passes `"mcp"`, the visual surface passes `"web"`. Inertia substrate will pass `"web"`.

**Visual route registration (SUBST-04 entry point):**
`app/src/routes.rs:127` registers `post!("/{service}/{action}", controllers::visual_action::handle)` inside a `TenantMiddleware` with `on_failure(TenantFailureMode::Forbidden)`. This is the existing route Inertia forms post to. No change needed here.

**`WriteDispatcher` struct:**
```rust
pub struct WriteDispatcher {
    pub executor: ExecutorFn,
    pub guard_evaluator: GuardEvaluatorFn,
    pub overrides: std::collections::HashMap<String, OverrideFn>,
}
```
[VERIFIED: framework/src/write/mod.rs:151]

---

### 2. Guard-visibility extraction source — `ferro-mcp-server/src/renderer.rs`

**The full guard filter is five lines:**
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
    // ... build the Tool ...
}
```
[VERIFIED: ferro-mcp-server/src/renderer.rs:224-262]

**Critical distinction:** `evaluated_guards` is a `HashMap<String, bool>` set by the host BEFORE the MCP request is processed. Semantics: absent key = allow (default-open), explicit `false` = hide. This is a **visibility pre-evaluation cache**, not a live evaluator. It is populated in Phase 218/219 by the app's middleware/handler reading live DB state once per request.

**`McpContext` fields:**
```rust
pub struct McpContext {
    pub tenant_id: Option<i64>,
    pub evaluated_guards: HashMap<String, bool>,
    pub scope: Option<String>,
    pub write_authorized: Option<bool>,
}
```
[VERIFIED: ferro-mcp-server/src/renderer.rs:23]

**The same `evaluated_guards` pattern mirrors `BaseContext.evaluated_guards`:**
```rust
pub struct BaseContext {
    // ...
    pub evaluated_guards: HashMap<String, bool>,
    // ...
}
```
[VERIFIED: ferro-projections/src/render/mod.rs:46]

**What to lift into `framework`:** A pure function:
```rust
pub fn permitted_actions(
    service: &ServiceDef,
    evaluated_guards: &HashMap<String, bool>,
) -> Vec<String>
```
that returns `service.actions.iter().filter(|a| !a.preconditions.iter().any(|p| evaluated_guards.get(p) == Some(&false))).map(|a| a.name.clone()).collect()`.

This function is acyclic: `framework` already depends on `ferro-projections` (for `ActionDef` etc.), and `ferro-mcp-server` depends on `ferro-rs`/`framework`. Adding this function to `framework` creates no new edges.

**After the lift:** `render_action_tool` in `ferro-mcp-server` becomes a thin wrapper calling `framework::permitted_actions(service, &ctx.evaluated_guards)` and then checking if the action name is in the returned set. The MCP surface is not regressed because the logic is identical — just moved.

---

### 3. Projection declaration — `ferro-projections/src/`

**`ServiceDef` key fields (for `SchemaContract`):**
- `fields: Vec<FieldDef>` — each `FieldDef` has `.name`, `.data_type: DataType`, `.meaning: FieldMeaning`, `.required`, `.is_list`, `.readable`, `.writable`, `.render_hint: Option<RenderHint>`
- `actions: Vec<ActionDef>` — each `ActionDef` has `.name`, `.inputs: Vec<InputDef>`, `.preconditions: Vec<String>`, `.effects: Vec<String>`, `.description: Option<String>`, `.display_name: Option<String>`, `.transition_trigger: Option<String>`
- `guards: Vec<GuardDef>` — each `GuardDef` has `.name`, `.display_name`
- `state_machine: Option<StateMachine>` — `StateMachine` has `.states`, `.transitions`, `.initial_state`

[VERIFIED: ferro-projections/src/service.rs:63-113]

**Existing exclusion helpers already on `ServiceDef`:**
- `is_server_injected_field(&self, field: &FieldDef) -> bool` — Identifier, CreatedAt, tenant column
- `is_write_excluded_field(&self, field: &FieldDef, exclude_sm_status: bool) -> bool` — server-injected + UpdatedAt + Sensitive + list + SM Status + non-writable

[VERIFIED: ferro-projections/src/service.rs:236-282]

These helpers are directly usable by `schema_contract` to tag which fields are writable vs. read-only in the contract.

**`derive_intents()` signature (the structural sibling):**
```rust
pub fn derive_intents(service: &ServiceDef) -> Vec<IntentScore>
```
[VERIFIED: ferro-projections/src/lib.rs:15]

`schema_contract` follows the same pattern: takes `&ServiceDef`, returns a pure value, zero side effects, zero runtime deps.

**No sea-orm/tokio in `ferro-projections`:** The crate's `CLAUDE.md` states it explicitly: "Schema-only service definitions and the modality-agnostic `Renderer` trait. No runtime engines, no closures." The `Cargo.toml` confirms: only `serde`, `serde_json`, `schemars`, `thiserror`. Adding `schema_contract` there is safe.

---

### 4. Data read shape — `ferro-mcp-server/src/dispatch.rs`

**`dispatch` function signature:**
```rust
pub async fn dispatch(
    service: &ServiceDef,
    filters: serde_json::Value,
    limit: u64,
    offset: u64,
    db: &sea_orm::DatabaseConnection,
    tenant_id: Option<i64>,
) -> crate::Result<DispatchResult>
```
[VERIFIED: ferro-mcp-server/src/dispatch.rs:117]

**`DispatchResult`:**
```rust
pub struct DispatchResult {
    pub rows: Vec<serde_json::Value>,
    pub total: u64,
    pub limit: u64,
    pub offset: u64,
}
```
[VERIFIED: ferro-mcp-server/src/dispatch.rs:19]

**Pagination:** Offset-based. `MAX_LIMIT = 100` (hard cap at the function, regardless of caller). Default at the MCP layer is 25. Soft-delete filtering is applied when `service.soft_delete_column.is_some()`. Tenant predicate is injected as a bound parameter when `service.tenant_column.is_some()`.

**The function already lives in `ferro-mcp-server`.** This creates a dependency problem: `ferro-inertia` must NOT depend on `ferro-mcp-server`. The solution is to relocate `dispatch` into `framework` (or a new `framework::projection_read` module) and have `ferro-mcp-server` re-export it. The implementation is pure SQL via sea-orm's `DatabaseConnection` — no MCP-specific types. `framework` already depends on `sea-orm` directly.

---

### 5. Delivery target — `ferro-inertia/src/`

**`Inertia::render` signature:**
```rust
pub fn render<R, P>(req: &R, component: &str, props: P) -> InertiaHttpResponse
where
    R: InertiaRequest,
    P: Serialize,
```
[VERIFIED: ferro-inertia/src/response.rs:148]

**`Inertia` is a zero-field struct** with only `impl Inertia` methods — no state to carry.

**What `Inertia::from_projection` needs at call time:**
- A request reference implementing `InertiaRequest`
- A `&ServiceDef`
- A component name (the Inertia page to render)
- A query input (filters, limit, offset)
- A `&DatabaseConnection` for the data query
- The authenticated `tenant_id: Option<i64>`
- A pre-evaluated `evaluated_guards: &HashMap<String, bool>` for `permitted_actions`

**Proposed signature:**
```rust
pub async fn from_projection<R: InertiaRequest>(
    req: &R,
    component: &str,
    service: &ServiceDef,
    query: ProjectionQuery,
    db: &DatabaseConnection,
    tenant_id: Option<i64>,
    evaluated_guards: &HashMap<String, bool>,
) -> InertiaHttpResponse
```

where `ProjectionQuery { filters: Value, limit: u64, offset: u64 }` is a small new struct in `ferro-inertia`.

**Props shape:**
```rust
#[derive(Serialize)]
struct ProjectionProps {
    schema: SchemaContract,
    data: Vec<Value>,
    permitted_actions: Vec<String>,  // per-service action names (see OQ-3)
}
```

**`ferro-inertia` Cargo.toml:** Currently has no `framework` or `ferro-projections` dep. Adding them is the expected pattern (output crates depend on the projection layer — same as `ferro-json-ui` and `ferro-mcp-server`). The dependency is acyclic: `ferro-inertia → framework → ferro-projections`.

---

### 6. Single-source test to mirror — `app/src/tests/single_source.rs`

The `single_source_both_channels` test:
- Declares one `ServiceDef` with a `StateMachine` and one `ActionDef`.
- Drives the `submit` action through BOTH `dispatch_write(.., "mcp")` and `dispatch_write(.., "web")`.
- Asserts identical persisted state, with only the audit channel differing.

[VERIFIED: app/src/routes.rs:117, app/src/tests/single_source.rs referenced in dispatch context]

For SUBST-05, the permitted-actions parity test:
- Constructs a service with multiple actions and guards.
- Sets `evaluated_guards = {"is_manager": false}` — hides the `approve` action.
- Calls `framework::permitted_actions(service, &evaluated_guards)` → does NOT return `"approve"`.
- Calls `render_exposed_tools` (MCP path, same `evaluated_guards`) → `"approve"` tool is absent.
- Changes state so `is_manager` = not explicitly false → both surfaces return `"approve"`.

---

## Open Question Resolutions

### OQ-1: `SchemaContract` location

**Recommendation: `ferro-projections`.**

Evidence: `ferro-projections` is a pure-schema crate with zero runtime deps (confirmed). All data needed for `SchemaContract` (`FieldDef`, `ActionDef`, `InputDef`, `GuardDef`) is already in-crate. The pattern of `derive_intents()` as a sibling pure function is directly applicable. Putting `SchemaContract` in `ferro-inertia` would make it impossible for future non-Inertia renderers (generic JSON mode, mobile) to use the same contract without depending on the Inertia crate. `ferro-projections` is the single-source-of-truth home.

### OQ-2: Opt-in surface

**Recommendation: No new opt-in — reuse `mcp_exposed: bool`.**

Evidence: `ServiceDef` already has `mcp_exposed: bool` as the single opt-in for the MCP surface. The spec's goal is a substrate that makes an Inertia-first app still declare `ServiceDef`s so `McpRenderer` also derives its tools automatically. A separate `inertia_exposed` flag would create two sources of truth (duplicating the feedback rule `no_duplicate_control_surface`). The opt-in for `Inertia::from_projection` is implicit: a handler calls it when it wants projection-driven props. There is nothing to register — the developer chooses to call it in their route handler. This mirrors how `Inertia::render` works today: no registration, just a call.

If future generic JSON mode needs an explicit opt-in, that belongs with that mode's shipping phase.

### OQ-3: Permitted-actions placement in props

**Recommendation: Per-service list (not per-record inline).**

Evidence: The current `evaluated_guards` map is per-request, not per-record. The guards in `action.preconditions` that the MCP surface filters on are declared at the `ServiceDef` level, not scoped to individual record ids. A record-scoped guard would require calling `GuardEvaluatorFn` per record per action — that is an async DB call for every row in the page, which is expensive (N rows × M actions × K guards DB calls). The per-service list is the correct first delivery: `permitted_actions: Vec<String>` is the set of action names currently visible, evaluated once per request from the pre-evaluated `evaluated_guards` map. A record can only be acted on if the action is in this list; the actual per-record state check happens at `dispatch_write` time anyway (guard re-eval). The frontend disables action buttons based on the list. This is the same semantic the MCP surface uses.

If future per-record guards emerge, they can be added as a separate `record_permitted_actions: HashMap<RecordId, Vec<String>>` field.

### OQ-4: Pagination contract

**Recommendation: Offset-based with `limit: u64, offset: u64`; default limit = 25; max = 100.**

Evidence: `ferro-mcp-server/src/dispatch.rs` already implements offset pagination with `MAX_LIMIT = 100` and the MCP layer defaults to `limit = 25`. Reusing the same contract keeps the two query paths identical, simplifying parity reasoning. Cursor-based pagination would require a new DB query interface and breaks the `DispatchResult` shape. Offset is sufficient for the current use case (admin/gestionali, typically small page sizes). If a consumer needs cursor pagination, that can be an additive extension.

The `ProjectionQuery` struct should default to `limit = 25, offset = 0, filters = {}` — matching the MCP defaults.

### OQ-5: Can `permitted_actions` lift cleanly into `framework`?

**Recommendation: Yes — clean lift, zero regression risk.**

Evidence: The guard-visibility logic in `ferro-mcp-server/src/renderer.rs:229-233` is:
```rust
for precondition in &action.preconditions {
    if ctx.evaluated_guards.get(precondition) == Some(&false) {
        return Ok(None);
    }
}
```

This is purely a `HashMap<String, bool>` lookup against `action.preconditions: Vec<String>`. Both types (`HashMap<String, bool>`, `Vec<String>`) are from `std`. `ActionDef.preconditions` is from `ferro-projections`. `framework` already depends on `ferro-projections`.

The lifted function:
```rust
// framework/src/permitted_actions.rs
pub fn permitted_actions(
    service: &ServiceDef,
    evaluated_guards: &HashMap<String, bool>,
) -> Vec<String> {
    service.actions.iter()
        .filter(|action| {
            !action.preconditions.iter()
                .any(|p| evaluated_guards.get(p) == Some(&false))
        })
        .map(|a| a.name.clone())
        .collect()
}
```

`ferro-mcp-server` then refactors `render_action_tool` to call this function. The function is pure and deterministic — its behavior is identical to the old inline loop. One grep-verifiable guard-evaluation site.

**MCP regression risk:** None. The behavior is identical. The difference is only which compilation unit owns the code. `ferro-mcp-server` still calls into `framework` (it already does for `dispatch_write`). The re-export in `ferro-mcp-server/src/lib.rs` can optionally expose `permitted_actions` for backward compatibility, but it is not required.

---

## Recommended Build Order

```
Wave 0 (test scaffolding):
  ferro-projections/tests/schema_contract.rs  — snapshot test fixture (RED)
  app/src/tests/permitted_actions_parity.rs   — parity test (RED)

Wave 1 (schema_contract — SUBST-01):
  ferro-projections/src/schema_contract.rs    — new pure fn + SchemaContract type
  ferro-projections/src/lib.rs                — re-export SchemaContract + schema_contract

Wave 2 (permitted_actions lift — SUBST-02):
  framework/src/permitted_actions.rs          — new fn (or inline in write/mod.rs)
  framework/src/lib.rs                        — re-export permitted_actions + ferro::permitted_actions
  ferro-mcp-server/src/renderer.rs            — refactor render_action_tool to call framework::permitted_actions
  (regression: existing tools/list tests stay green)

Wave 3 (data query relocation):
  framework/src/projection_read.rs            — relocate `dispatch` fn from ferro-mcp-server
  framework/src/lib.rs                        — re-export DispatchResult + projection_read
  ferro-mcp-server/src/dispatch.rs            — thin wrapper calling framework::projection_read::dispatch
  ferro-mcp-server/src/lib.rs                 — re-export DispatchResult from framework

Wave 4 (Inertia::from_projection — SUBST-03):
  ferro-inertia/Cargo.toml                    — add deps: framework, ferro-projections, sea-orm
  ferro-inertia/src/projection.rs             — ProjectionQuery + from_projection impl
  ferro-inertia/src/lib.rs                    — pub use projection::{from_projection, ProjectionQuery}

Wave 5 (parity tests — SUBST-05):
  app/src/tests/permitted_actions_parity.rs   — GREEN: permitted-actions match across MCP and Inertia surfaces
  ferro-projections/tests/schema_contract.rs  — GREEN: snapshot test
  app/src/tests/data_tenant_scoping.rs        — GREEN: data query tenant isolation
  (write parity via existing visual_action tests — no new test needed, SUBST-04 reuses existing route)
```

**Acyclic dependency constraint holds throughout:**
```
ferro-projections  (leaf, no ferro deps)
    ↑
framework          (depends on ferro-projections, sea-orm, ferro-audit)
    ↑
ferro-mcp-server   (depends on framework, ferro-projections)
ferro-inertia      (depends on framework, ferro-projections)
```

`ferro-inertia` does NOT depend on `ferro-mcp-server` at any wave.

---

## Pitfalls

### Pitfall 1: Cycle risk — `ferro-inertia → ferro-mcp-server`

**What goes wrong:** If `Inertia::from_projection` calls `ferro-mcp-server::dispatch` directly, a crate cycle forms: `ferro-inertia → ferro-mcp-server → ferro-rs → ferro-inertia` (if ferro-inertia is a dep of ferro-rs).
**Why it happens:** The `dispatch` fn currently lives in `ferro-mcp-server`, which depends on `ferro-rs`. If `ferro-rs` re-exports `ferro-inertia`, the cycle closes.
**Prevention:** Relocate `dispatch` into `framework` before `Inertia::from_projection` is written. Wave 3 must precede Wave 4.

### Pitfall 2: Tenant-scoping trap — `evaluated_guards` is request-scoped, not record-scoped

**What goes wrong:** A developer computes `permitted_actions(service, evaluated_guards)` once per request and applies it as if guards were per-record. A guard like `has_items` that depends on a specific record's state would be evaluated against the wrong record.
**Why it happens:** The current MCP semantics treat `evaluated_guards` as a visibility cache evaluated before record lookup. It is not meant for per-record authorization.
**Prevention:** Document clearly in `permitted_actions`'s Rustdoc: this is a list-time visibility filter evaluated once per request from a pre-computed guard map. Per-record enforcement happens in `dispatch_write` via the live `GuardEvaluatorFn`. Guard re-evaluation at write time is mandatory — `permitted_actions` only controls what the frontend shows.

### Pitfall 3: `ferro-projections` must stay runtime-free

**What goes wrong:** `SchemaContract` is placed in `ferro-projections` but a future developer adds an async read or a tokio dep to compute it.
**Why it happens:** The crate boundary isn't enforced at the type system level.
**Prevention:** `SchemaContract` is a plain `struct` with only `Serialize`/`Deserialize`/`Clone`. `schema_contract()` takes `&ServiceDef` and returns it synchronously. No `async`, no `tokio`, no `sea-orm` anywhere in the function or the type. The crate's `CLAUDE.md` rule enforces this.

### Pitfall 4: MCP regression from the `permitted_actions` lift

**What goes wrong:** Behavior of `tools/list` changes after `render_action_tool` is refactored to call `framework::permitted_actions`.
**Why it happens:** A subtle semantic difference between the old inline loop and the new function (e.g., the transition guard is accidentally included in the filter when it shouldn't be).
**Prevention:** The lifted function only filters on `action.preconditions`, not on the transition-level guard. The transition guard is only evaluated at `dispatch_write` time (it is not in `action.preconditions`). The old inline loop also only uses `action.preconditions`. The refactor is a direct 1:1 extraction. Add a regression test that asserts the exact same tool list before and after the refactor.

### Pitfall 5: `dispatch` relocation breaks `ferro-mcp-server` internal types

**What goes wrong:** `dispatch` uses `crate::Error::InvalidFilter` and `crate::Error::Database` — `crate` is `ferro-mcp-server`. Moving the function to `framework` means switching to `framework`'s error type.
**Why it happens:** Error type is crate-local.
**Prevention:** Create a `framework::ProjectionReadError` (or re-use `WriteError` with new variants) before relocating `dispatch`. `ferro-mcp-server::Error` then maps from it via `From`. Alternatively, keep a thin wrapper in `ferro-mcp-server` that delegates and maps the error — this is the lower-risk option.

---

## Validation Architecture

**Nyquist validation is enabled for this project.**

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust test harness (`cargo test --all-features`) |
| Config file | `Cargo.toml` workspace |
| Quick run | `cargo test -p ferro-projections schema_contract` |
| Full suite | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Location |
|--------|----------|-----------|-------------------|---------------|
| SUBST-01 | `schema_contract` returns correct field names, meanings, validations, action defs | snapshot + unit | `cargo test -p ferro-projections schema_contract` | `ferro-projections/tests/schema_contract.rs` (Wave 0 gap) |
| SUBST-01 | `schema_contract` output matches serde round-trip | unit | same | same |
| SUBST-02 | `permitted_actions` hides action when guard is `Some(false)` | unit | `cargo test -p framework permitted_actions` | `framework/src/permitted_actions.rs` (inline `#[cfg(test)]`) |
| SUBST-02 | After lift, MCP `tools/list` returns same tools as before refactor | regression | `cargo test -p ferro-mcp-server` | `ferro-mcp-server/src/renderer.rs` (extend existing tests) |
| SUBST-02 | Permitted-actions parity: `permitted_actions(...)` == guard-filtered MCP tool names for same record+state | integration | `cargo test -p app permitted_actions_parity` | `app/src/tests/permitted_actions_parity.rs` (Wave 0 gap) |
| SUBST-03 | Data query is tenant-scoped; cross-tenant rows excluded | integration | `cargo test -p app data_tenant_scoping` | `app/src/tests/data_tenant_scoping.rs` (Wave 0 gap) |
| SUBST-03 | `Inertia::from_projection` serializes `{ schema, data, permitted_actions }` correctly | unit | `cargo test -p ferro-inertia from_projection` | `ferro-inertia/src/projection.rs` (Wave 4) |
| SUBST-04 | Inertia POST reaches same `dispatch_write` kernel as MCP; audit differs only by channel tag | integration | `cargo test -p app single_source_inertia` | `app/src/tests/single_source.rs` (extend) |
| SUBST-05 | Changing action precondition guard state changes both Inertia `permitted_actions` and MCP `tools/list` identically | integration | `cargo test -p app permitted_actions_parity` | same as SUBST-02 parity test |

### Sampling Rate

- **Per task commit:** `cargo test -p <changed_crate> --all-features`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-projections/tests/schema_contract.rs` — SUBST-01 snapshot fixture
- [ ] `app/src/tests/permitted_actions_parity.rs` — SUBST-02/05 parity test
- [ ] `app/src/tests/data_tenant_scoping.rs` — SUBST-03 tenant isolation (may already be partially covered by `crud_e2e.rs` — check before creating)

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `ferro-inertia` is not currently a dependency of `ferro-rs`/`framework`, so adding `ferro-inertia → framework` is acyclic. | Codebase Findings §5 | If `ferro-rs` re-exports `ferro-inertia`, a cycle forms. Verify with `cargo tree` before Wave 4. |
| A2 | The `dispatch` function in `ferro-mcp-server` has no MCP-specific type dependencies other than its error type. | Codebase Findings §4 | If it uses rmcp types internally (beyond the return type), relocation is harder. |

---

## Sources

### Primary (HIGH confidence — verified against actual source in this session)

- `framework/src/write/mod.rs` — `dispatch_write`, `GuardEvaluatorFn`, `WriteDispatcher`, `merged_guards`, `WriteError`, `WriteResult`
- `ferro-mcp-server/src/renderer.rs` — `render_action_tool`, `render_exposed_tools`, `McpContext`, guard-visibility loop
- `ferro-mcp-server/src/dispatch.rs` — `dispatch` fn, `DispatchResult`, tenant scoping, offset pagination, `MAX_LIMIT = 100`
- `ferro-projections/src/service.rs` — `ServiceDef`, `FieldDef`, all builder methods, `is_server_injected_field`, `is_write_excluded_field`
- `ferro-projections/src/lib.rs` — re-exports, crate surface
- `ferro-inertia/src/response.rs` — `Inertia::render`, `InertiaHttpResponse`, `InertiaResponse`
- `ferro-inertia/src/lib.rs` — crate surface
- `ferro-mcp-server/src/write_dispatch.rs` — `handle_write_call`, CRUD path, transition path, guard-at-dispatch flow
- `ferro-mcp-server/src/jsonrpc.rs` — `handle_tools_list`, `handle_tools_call`
- `app/src/routes.rs` — visual write route registration at `post!("/{service}/{action}", ...)`
- `.planning/phases/263-projection-native-inertia-substrate/263-CONTEXT.md` — locked decisions, open questions
- `docs/superpowers/specs/2026-07-27-headless-projection-substrate-design.md` — anchor spec
- `.planning/REQUIREMENTS.md` — SUBST-01..05

### Secondary (MEDIUM confidence)

- `.planning/STATE.md` — Phase 231/232 write-kernel notes, v14.0 `BaseContext.evaluated_guards` semantics confirmed

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all types, signatures, and module paths verified in source
- Architecture: HIGH — dependency graph traced; acyclic constraint confirmed
- Pitfalls: HIGH — sourced from code inspection, not training assumptions
- Open question resolutions: HIGH for OQ-1/2/5; MEDIUM for OQ-3/4 (reasonable defaults, no consumer friction data yet)

**Research date:** 2026-07-27
**Valid until:** 60 days (framework is stable; no fast-moving external deps in this phase)
