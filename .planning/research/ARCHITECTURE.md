# Architecture Research — v15.0 Agent-Operable App (Consumer MCP)

**Domain:** Rust web framework — projection-derived consumer MCP endpoint
**Researched:** 2026-06-13
**Confidence:** HIGH (grounded in actual source files; all integration points verified)

---

## Existing Architecture Baseline

All claims below are drawn from source files read during this session. Confidence is HIGH throughout.

### Renderer trait (ferro-projections/src/render/mod.rs)

```rust
pub trait Renderer: Send + Sync {
    type Output;
    type Context: Default;
    fn render(&self, service: &ServiceDef, intents: &[IntentScore], ctx: &Self::Context)
        -> Result<Self::Output, Error>;
}
```

`BaseContext` (same file) carries `evaluated_guards: HashMap<String, bool>`, `verbosity: Verbosity`, `intent_index`, `current_state`. Absent guard key = render; explicit `false` = filter. Shipped in Phase 215 (v14.0).

The boundary rule from `ferro-projections/CLAUDE.md` is explicit: ferro-projections owns only the `Renderer` trait, `derive_intents()`, and `ServiceDef`. Concrete renderers live in output crates.

### ServiceDef fields relevant to v15.0 (ferro-projections/src/service.rs)

```
mcp_exposed: bool          -- opt-in filter (default false)
tenant_column: Option<String>  -- FK column name for tenant scoping
mcp_ability: Option<String>    -- Gate ability for per-projection authz
actions: Vec<ActionDef>    -- write/act operations
guards: Vec<GuardDef>      -- named boolean conditions
```

`ActionDef` (ferro-projections/src/action.rs) carries `inputs: Vec<InputDef>`, `preconditions: Vec<String>` (guard names), `effects: Vec<String>`, `transition_trigger: Option<String>`. `InputDef` carries `data_type`, `meaning`, `required`.

### ferro-mcp-server (already exists, partially implements v15.0 foundation)

Files: `src/renderer.rs`, `src/dispatch.rs`, `src/auth.rs`, `src/schema.rs`, `src/jsonrpc.rs`, `src/config.rs`, `src/error.rs`.

`McpRenderer` in `ferro-mcp-server/src/renderer.rs` already implements `Renderer<Output = Tool, Context = McpContext>`. It is a real output crate following the v11.5 boundary rule — it imports `ferro_projections::render::Renderer` and produces `rmcp::model::Tool`. This is the correct home.

`dispatch()` in `ferro-mcp-server/src/dispatch.rs` handles read-only SQL queries with filter-key allowlisting, tenant predicate injection (fail-closed on missing tenant), and LIMIT/OFFSET clamp. `tenant_id: Option<i64>` is a function parameter — never sourced from the call payload (security invariant already implemented).

`handle_initialize` / `handle_tools_list` / `handle_tools_call` in `src/jsonrpc.rs` are pure JSON-RPC dispatch functions already wired to `McpRenderer`.

`McpContext` in `src/renderer.rs` is currently empty (`struct McpContext;`). The comment says "Phase 200 will extend with tenant/policy context." This is the v15.0 extension point.

`BearerOutcome` in `src/auth.rs` is a stub enum. The real validation lives in `ferro-mcp-oauth/src/validate.rs` (`validate_bearer` function).

### ferro-mcp-oauth (already exists)

OAuth 2.1 full browser-login flow: discovery, DCR, PKCE, consent, JWT minting. `validate_bearer(header, config, expected_tenant) -> BearerCheck` in `src/validate.rs` validates JWT signature + expiry + audience + tenant match. `McpTokenClaims` carries `sub` and `tenant_id: Option<i64>`.

The crate is designed to be mounted by the app-level route layer. `ferro-mcp-server` gains no new dependency from it (stated in `ferro-mcp-oauth/src/lib.rs` module docs).

### ferro-ai (ferro-ai/src/)

`Classifier<T>` (src/classifier/mod.rs): provider-agnostic LLM structured-output wrapper. Accepts `system_prompt`, `user_prompt`, `schema: &serde_json::Value`. Returns `ClassificationResult<T>` with `value: T`, `confidence: Option<f64>`, `raw_json`.

`ToolRegistry` (src/tools/mod.rs): registers `ToolDef` (name + description + parameters_schema + async handler). `dispatch(messages, client)` loops until LLM returns Text or hits `max_iterations`. Hard cap, no override. Tool errors surface to LLM as model-legible strings, not aborts.

`ConfirmationStore` / `InMemoryConfirmationStore` (src/confirmation/): TTL-gated payload store for destructive action gating. `request_confirmation(key, payload, ttl)` → `confirm(key)` → `Some(payload)`.

---

## Decision (a): Where Does projection→MCP-tools Live?

**Decision: Extend `ferro-mcp-server`, not a new crate. The `McpRenderer` is already there and is already the correct output crate.**

Justification against the v11.5 boundary rule:

- The rule is: concrete renderers live in their output crate, not in ferro-projections.
- `ferro-mcp-server/src/renderer.rs` already IS the output crate for MCP projection rendering. `McpRenderer` already implements `Renderer<Output = rmcp::model::Tool>`.
- Adding write-tool rendering (one extra `Tool` per `ActionDef`, annotated `destructive_hint: true`) extends `McpRenderer` or adds `render_action_tool()` to the same file. No new crate is justified.
- A new crate (`ferro-mcp-projections` or similar) would split what is conceptually one output renderer across two crates, violating the conceptual coherence principle.

For v15.0 the `McpContext` struct is extended to carry the tenant ID and evaluated guards so the write-tool filter can work:

```rust
// ferro-mcp-server/src/renderer.rs (modified)
#[derive(Debug, Clone, Default)]
pub struct McpContext {
    pub tenant_id: Option<i64>,
    pub evaluated_guards: HashMap<String, bool>,
}
```

Read tools stay as `list_<name>` (already works). Write tools emit `<action_name>_<service_name>` (e.g. `submit_order`) with `readOnlyHint: false` and `destructiveHint: true/false` depending on whether the action has a `transition_trigger`. Guard-filtered: if `evaluated_guards.get(precondition) == Some(&false)`, the action tool is omitted from `tools/list`.

---

## Decision (b): ServiceDef → Tool Mapping

### Read tools (queries)

Already implemented in `ferro-mcp-server/src/renderer.rs`:
- One tool per `mcp_exposed` `ServiceDef`, named `list_<service.name>`
- `inputSchema` derived from filterable fields via `schema::build_input_schema()` in `src/schema.rs`
- `readOnlyHint: true`

### Write tools (actions)

New, to be added in `ferro-mcp-server/src/renderer.rs` and `src/schema.rs`:

Each `ActionDef` in a `mcp_exposed` `ServiceDef` becomes one MCP tool.

**Tool name:** `<action.name>` (already unique within a service; if collision across services then `<action.name>_on_<service.name>`).

**Guard filtering:** For each `action.precondition` name, check `ctx.evaluated_guards.get(precondition)`. If any precondition evaluates to explicit `false`, omit the tool from `tools/list`. Absent key = offer the tool (same semantics as `BaseContext.evaluated_guards` in v14.0 visual path).

**inputSchema derivation:** From `action.inputs: Vec<InputDef>`. Each `InputDef` maps to a JSON Schema property using the same `data_type_to_json_schema()` function already in `src/schema.rs`. Required fields land in `required: [...]`. The identifier field of the parent `ServiceDef` (first `FieldMeaning::Identifier` field) is always injected as a required parameter — this is the record to act on.

**Annotation:** `ToolAnnotations::new().read_only(false).destructive(action.transition_trigger.is_some())`.

**Route resolution:** `ActionDef` already carries `name` as a stable verb. The app handler layer maps `action.name` to the actual HTTP route at registration time — the `McpRenderer` emits the tool name, the app-layer `handle_tools_call` routes the call to the right handler. No route URL is baked into the tool definition (that would couple the MCP layer to the HTTP layer).

### Action route/precondition → tool input schema mapping

```
ActionDef.inputs[i]        → inputSchema.properties[inputs[i].name]
  .data_type               →   type/format via data_type_to_json_schema()
  .required                →   appears in inputSchema.required[] if true
  .description             →   description field
ActionDef.preconditions[j] → guard-filter at tools/list render time (not in schema)
ServiceDef.Identifier field → always injected as required integer param (the record ID)
```

---

## Decision (c): Inbound Intent Loop

### Classification strategy

The inbound loop classifies a natural-language message into an **action** directly (not through an intermediate Intent). Reason: v15.0 tools are already organized by `ServiceDef` + `ActionDef` — an intent classification step would add a level of indirection without narrowing the space, since each tool already encodes its purpose in its name and description.

**Classification path:**

```
NL message
    ↓
ferro_ai::Classifier<ToolSelection>
    (system prompt = tool list + guard-filtered available tools)
    (user prompt   = the NL message)
    ↓
ToolSelection { tool_name: String, confidence: f64, arguments: Map<String, Value> }
    ↓ (confidence < threshold → ask for clarification)
dispatch_write(tool_name, arguments, tenant_id, db)
    ↓ (precondition guards fail → surface to agent)
Result → MCP tool_result content block
```

`ferro_ai::Classifier<ToolSelection>` calls `classify(system, user, schema)` where `schema` is the JSON Schema for `ToolSelection`. The output type `ToolSelection` is defined in `ferro-mcp-server` (not in ferro-ai — it is projection-specific).

### Parameter elicitation

When `ToolSelection.confidence < threshold` or required arguments are missing: the handler returns a `tool_result` with `isError: false` and a structured payload asking for clarification (e.g. `{ "status": "needs_clarification", "missing_params": [...], "question": "..." }`). The agent surfaces this to the user. No separate elicitation state machine is needed — the MCP protocol's request/response loop handles multi-turn naturally.

### Confirmation gating

`ferro_ai::ConfirmationStore` gates destructive actions (those where `ActionDef.transition_trigger.is_some()` or an explicit `requires_confirmation` flag is added to `ActionDef`). Flow:

```
agent calls write tool
    → handler checks ConfirmationStore
    → if destructive: request_confirmation(key, payload, ttl=60s)
    → return { "status": "pending_confirmation", "confirmation_key": key }
agent calls confirm_<action_name> tool with key
    → ConfirmationStore.confirm(key) → payload
    → execute action
```

`confirm_<action_name>` is a synthesized tool emitted by `render_exposed_tools` for each destructive action alongside the action tool itself.

### Where the loop lives

The intent loop wires inside the app-layer MCP handler (in the consumer application's `src/` tree), not inside ferro-mcp-server. ferro-mcp-server provides `dispatch_write()` as a new function alongside the existing `dispatch()`. The consumer app registers a `/mcp/chat` endpoint that hosts the classify → dispatch → confirm loop. This keeps ferro-mcp-server projection-agnostic.

---

## Decision (d): Per-Tenant API-Key Auth

### Slot into ferro-mcp-server

The existing `BearerOutcome` in `ferro-mcp-server/src/auth.rs` is a stub. v15.0 adds API-key auth as a second auth path alongside OAuth JWT:

```
HTTP Authorization header
    ↓
ferro-mcp-server/src/auth.rs (modified)
    Case 1: "Bearer eyJ..." → ferro_mcp_oauth::validate_bearer(header, oauth_config, None) → McpTokenClaims.tenant_id
    Case 2: "Bearer ferro_..." (API key prefix) → look up key in api_keys table → tenant_id
    Case 3: absent → 401
```

The API key table is consumer-app DB (same SeaORM connection). ferro-mcp-server adds `resolve_tenant_from_api_key(key, db) -> Result<i64, Error>` function in `src/auth.rs`. The function does a single parameterized `SELECT tenant_id FROM api_keys WHERE key_hash = SHA256(key) AND revoked_at IS NULL`. The key is never stored plaintext — the consumer app populates `key_hash` on key creation.

### Tenant threading into render context

After auth resolves `tenant_id`:

1. `McpContext { tenant_id: Some(id), evaluated_guards: ... }` is constructed at the top of the request handler.
2. `render_exposed_tools(services, &ctx)` reads `ctx.tenant_id` — currently unused for filtering tools/list (the guard filter already handles per-action filtering), but available for ability checks: `if let Some(ability) = service.mcp_ability { check_gate(ability, tenant_id) }`.
3. `handle_tools_call` passes `tenant_id` to `dispatch()` (already implemented — `dispatch` takes `Option<i64>`).
4. `dispatch()` injects the tenant predicate if `service.tenant_column.is_some()` (already implemented, fail-closed).

### TenantScoped integration

`TenantScoped` (v13.1, `ferro-macros`) operates at the handler level in the HTTP layer. The MCP server is not an HTTP handler — it is a JSON-RPC endpoint. The security guarantee that matters is: `dispatch()` already enforces fail-closed tenant scoping at the SQL level. `TenantScoped` applies when the consumer app's own handlers execute the actions (not through ferro-mcp-server's generic dispatch). For write actions that delegate to the app's own HTTP handlers (Option B in build order below), `TenantScoped` operates naturally.

---

## System Architecture Diagram

```
Consumer App Request
        |
        v
[app/src/routes.rs]   -- mounts /mcp and /chat endpoints
        |
        | Authorization header
        v
[ferro-mcp-server/src/auth.rs]   -- resolve_tenant_from_bearer() or resolve_tenant_from_api_key()
        |                             returns tenant_id: i64 (or 401/403)
        v
 ┌──────────────────────────────────────────────────┐
 │             MCP Request Router                   │
 │  ferro-mcp-server/src/jsonrpc.rs                 │
 │                                                  │
 │  initialize   → handle_initialize()              │
 │  tools/list   → handle_tools_list()              │
 │  tools/call   → handle_tools_call()              │
 │  tools/call   → handle_write_call() [NEW]        │
 └───────────┬──────────────────────────────────────┘
             │
     ┌───────┴────────────────────────────┐
     │                                    │
     v (read)                             v (write)
[ferro-mcp-server/src/renderer.rs]   [ferro-mcp-server/src/renderer.rs]
 McpRenderer: ServiceDef → list_Tool   render_action_tool(): ActionDef → write_Tool
 McpContext { tenant_id, guards }      guard-filter via ctx.evaluated_guards
     │                                    │
     v                                    v
[ferro-mcp-server/src/dispatch.rs]   [ferro-mcp-server/src/write_dispatch.rs NEW]
 dispatch(service, filters,            dispatch_write(action, inputs, tenant_id, db)
   limit, offset, db, tenant_id)        → validates inputs, runs app callback
     │                                    │
     v                                    v (optional confirmation gate)
 SQL SELECT (tenant-scoped)          [ferro-ai/src/confirmation/]
 via SeaORM DatabaseConnection        InMemoryConfirmationStore
     │                                    │
     v                                    v
 DispatchResult { rows, total, ... }  ActionResult / confirmation pending

              ↑ inbound intent loop (optional path)
[/mcp/chat endpoint — consumer app]
 ferro_ai::Classifier<ToolSelection>
    classifies NL message → tool_name + arguments
    → delegates to handle_write_call()
```

---

## Component Boundaries: New vs Modified

### Modified crates

**ferro-mcp-server** (primary v15.0 site)
- `src/renderer.rs`: extend `McpContext` with `tenant_id` + `evaluated_guards`; add `render_action_tool()` for write tools; extend `render_exposed_tools()` to emit both read and write tools
- `src/schema.rs`: add `build_action_input_schema(action, service)` — derives inputSchema from `ActionDef.inputs` + identifier field injection
- `src/dispatch.rs`: add `dispatch_write(action_name, inputs, tenant_id, db, callback)` — validates inputs against `ActionDef.inputs`, checks guard conditions, invokes callback
- `src/auth.rs`: replace stub `BearerOutcome` with real `resolve_tenant_from_bearer(header, oauth_config)` + new `resolve_tenant_from_api_key(raw_key, db)`; unify into `resolve_tenant(header, db, oauth_config) -> Result<i64, AuthError>`
- `src/error.rs`: add `Auth(String)`, `GuardFailed(String)`, `ActionNotFound(String)` variants
- `src/jsonrpc.rs`: add `handle_write_call(params, services, db, tenant_id)` route; hook guard evaluation before tool dispatch

**ferro-projections** (minimal, additive only)
- `src/action.rs`: consider adding `requires_confirmation: bool` to `ActionDef` (optional — can be inferred from `transition_trigger.is_some()` if that heuristic is sufficient). Decision: defer to phase; the field is non-breaking additive.
- No renderer changes — boundary rule is maintained.

**ferro-ai** (no changes to the crate itself)
- `ConfirmationStore` / `InMemoryConfirmationStore` are already correct. The MCP server consumes them.
- `Classifier<T>` is already correct. The consumer app's chat endpoint uses it.

### New crates

None required for the core four capabilities. The motivation to create a new crate would be if the write-dispatch machinery grew large enough to warrant separation (e.g. `ferro-mcp-write`), but at v15.0 scope that is premature. All new code belongs in ferro-mcp-server.

---

## Data Flow: Message → Tool → Guard → Execute → Result

### Read path (already works, v12.6)

```
tools/call { name: "list_order", arguments: { status: "pending", limit: 10 } }
    ↓
handle_tools_call() strips pagination → filters object
    ↓
dispatch(service, filters, limit, offset, db, tenant_id=Some(7))
    ↓ (service.tenant_column="tenant_id" → inject AND tenant_id=7)
SELECT * FROM "orders" WHERE status=? AND tenant_id=? LIMIT ? OFFSET ?
    ↓
DispatchResult { rows: [...], total: 12, limit: 10, offset: 0 }
    ↓
CallToolResult::structured(payload)
```

### Write path (new in v15.0)

```
tools/call { name: "submit_order", arguments: { id: 42, notes: "urgent" } }
    ↓
handle_write_call()
    ↓
find ActionDef where action.name = "submit_order" in mcp_exposed service
    ↓
evaluate guards: evaluated_guards.get("has_items") = Some(true)? → proceed
    ↓ (any precondition = Some(false) → return tool error "action not available")
validate inputs against ActionDef.inputs schema
    ↓ (missing required? → return error with missing param list)
dispatch_write(action, validated_inputs, tenant_id=7, db)
    ↓ (if action.transition_trigger.is_some() → gate via ConfirmationStore)
    ↓ (confirmation pending? → return { status: "pending_confirmation", key: "..." })
    ↓ (confirmed or no confirmation required)
invoke app callback (HTTP POST to app's own route, or direct DB call)
    ↓
ActionResult { success: true, ... }
```

### Inbound intent loop (new in v15.0, consumer app layer)

```
POST /mcp/chat { message: "approve the order from Alice" }
    ↓ auth → tenant_id
    ↓
Classifier<ToolSelection>.classify(
    system = render_exposed_tools(services, ctx) as tool descriptions,
    user   = message,
    schema = ToolSelection JSON Schema
)
    ↓ confidence < 0.7 → return { needs_clarification: true, question: "..." }
    ↓ confidence >= 0.7
ToolSelection { tool_name: "submit_order", arguments: { id: 42 } }
    ↓
handle_write_call(params, services, db, tenant_id)
    (same path as direct MCP write above)
```

---

## Build Order (dependency-ordered phases)

Dependencies run strictly: each phase can only build on what the previous phase completed.

### Phase 1 — Auth foundation + McpContext extension

**What:** Replace stub `BearerOutcome` with real tenant resolution; extend `McpContext`.

**Files:**
- `ferro-mcp-server/src/auth.rs`: `resolve_tenant_from_bearer()` (delegates to `ferro_mcp_oauth::validate_bearer`) + `resolve_tenant_from_api_key(raw_key_prefix, db)` (SHA-256 lookup in `api_keys` table)
- `ferro-mcp-server/src/renderer.rs`: extend `McpContext { tenant_id: Option<i64>, evaluated_guards: HashMap<String, bool> }`
- `ferro-mcp-server/src/error.rs`: add `Auth(String)` variant

**Why first:** Every subsequent phase depends on a real `tenant_id` threaded through the context. The guard filtering and write dispatch both need it. Without this, every other phase is stubs-on-stubs.

**Note:** API-key table migration (CREATE TABLE api_keys) belongs in the consumer app, not in ferro-mcp-server. ferro-mcp-server exposes `resolve_tenant_from_api_key(key, db)` as a library function; the consumer defines the table.

### Phase 2 — Write-tool rendering (actions → MCP tools)

**What:** Project `ActionDef` lists into MCP `Tool` definitions, guard-filtered.

**Files:**
- `ferro-mcp-server/src/schema.rs`: `build_action_input_schema(action: &ActionDef, service: &ServiceDef) -> Result<Value>`
- `ferro-mcp-server/src/renderer.rs`: `render_action_tool(service, action, ctx) -> Tool`; extend `render_exposed_tools()` to call it per `ActionDef`
- Tests: guard-filter with `evaluated_guards = {has_items: false}` omits the tool; schema has identifier field + action inputs; `destructiveHint` true for actions with `transition_trigger`

**Why second:** Depends on `McpContext` extension from Phase 1. The guard-filter logic reads `ctx.evaluated_guards`.

### Phase 3 — Write dispatch (action execution)

**What:** Execute an action tool call — validate inputs, check guards, invoke the app callback.

**Files:**
- `ferro-mcp-server/src/write_dispatch.rs` (new file): `dispatch_write(action, inputs, tenant_id, callback, confirmation_store)`
- `ferro-mcp-server/src/jsonrpc.rs`: `handle_write_call()` — routes `tools/call` for non-list_ tool names to `dispatch_write`
- `ferro-mcp-server/src/error.rs`: `GuardFailed(String)`, `ActionNotFound(String)`

**Why third:** Depends on Phase 2 (write tools must be registered before they can be called). The callback signature is `async fn(action_name: &str, inputs: Value, tenant_id: i64, db: &DatabaseConnection) -> Result<Value, Error>` — app registers it at server setup.

### Phase 4 — Confirmation gating

**What:** Gate destructive actions via `ferro_ai::ConfirmationStore`.

**Files:**
- `ferro-mcp-server/src/write_dispatch.rs`: check `action.transition_trigger.is_some()` (or future `requires_confirmation` flag); call `store.request_confirmation(key, payload, ttl)`; synthesize `confirm_<action>` tool in `render_exposed_tools()`
- `ferro-mcp-server/Cargo.toml`: add `ferro-ai` dependency (for `ConfirmationStore` trait + `InMemoryConfirmationStore`)

**Why fourth:** Depends on Phase 3 (dispatch_write must exist before confirmation can gate it). Confirmation is a wrapper around dispatch.

**Dependency note:** Adding `ferro-ai` to `ferro-mcp-server`'s Cargo.toml introduces an LLM-client dependency into the server crate. If this is undesirable (e.g. it pulls in reqwest/async-http-client), narrow the dependency to only the `confirmation` module via a feature flag (`ferro-ai/confirmation-only`), or extract `ConfirmationStore` into a small standalone crate (e.g. `ferro-confirmation`). At current v15.0 scope, a feature flag in ferro-ai is the cleanest option.

### Phase 5 — Inbound intent loop

**What:** NL message → classify → tool selection → dispatch.

**Files:**
- Consumer app `src/handlers/mcp_chat.rs` (new handler in app, not in ferro-mcp-server)
- `ferro-mcp-server/src/lib.rs`: re-export a `render_tool_descriptions(services, ctx) -> Vec<ToolDescription>` helper that formats the available tools as a concise text block for use as classifier system prompt

**Why fifth (last):** Depends on Phase 3 and 4 (tools and dispatch and confirmation must exist). The intent loop is a consumer of the other three capabilities; it adds the NL entry point without changing the underlying machinery.

---

## Integration Points: Real File References

| Integration | File (from) | File (to) | What crosses the boundary |
|-------------|-------------|-----------|---------------------------|
| Renderer trait | ferro-projections/src/render/mod.rs | ferro-mcp-server/src/renderer.rs | `Renderer` trait impl: `Output = Tool` |
| ServiceDef schema | ferro-projections/src/service.rs | ferro-mcp-server/src/schema.rs | `ServiceDef.fields`, `.actions`, `.guards`, `.tenant_column`, `.mcp_exposed` |
| ActionDef inputs | ferro-projections/src/action.rs | ferro-mcp-server/src/schema.rs | `ActionDef.inputs` → inputSchema properties |
| Guard evaluation | ferro-projections/src/action.rs | ferro-mcp-server/src/renderer.rs | `ActionDef.preconditions` vs `ctx.evaluated_guards` |
| Tenant scoping | ferro-mcp-server/src/dispatch.rs | app handler (consumer) | `tenant_id: Option<i64>` parameter |
| OAuth validation | ferro-mcp-oauth/src/validate.rs | ferro-mcp-server/src/auth.rs (modified) | `validate_bearer() -> BearerCheck` |
| Confirmation store | ferro-ai/src/confirmation/mod.rs | ferro-mcp-server/src/write_dispatch.rs (new) | `ConfirmationStore` trait + `request_confirmation()` |
| Intent classification | ferro-ai/src/classifier/mod.rs | consumer app mcp_chat handler | `Classifier<ToolSelection>.classify()` |
| Dispatch (read) | ferro-mcp-server/src/dispatch.rs | ferro-mcp-server/src/jsonrpc.rs | `dispatch() -> DispatchResult` |
| Dispatch (write) | ferro-mcp-server/src/write_dispatch.rs (new) | ferro-mcp-server/src/jsonrpc.rs | `dispatch_write() -> ActionResult` |

---

## Anti-Patterns

### Anti-Pattern 1: Adding a McpRenderer to ferro-projections

**What:** Put `McpRenderer` inside ferro-projections instead of ferro-mcp-server.
**Why wrong:** Violates the v11.5 boundary rule. ferro-projections must stay renderer-free. Adding `rmcp` as a dependency to ferro-projections would pull a network/async crate into the schema-only crate, pollute the dependency tree of every downstream consumer, and break the crate's stated contract.
**Do this instead:** Keep `McpRenderer` in `ferro-mcp-server/src/renderer.rs` where it already lives.

### Anti-Pattern 2: Sourcing tenant_id from the tool call arguments

**What:** Let the agent pass `tenant_id` as a tool argument rather than deriving it from the auth token.
**Why wrong:** This is the root IDOR vulnerability. `dispatch()` already explicitly rejects this pattern (see `dispatch.rs` comment: "The tenant value is NEVER sourced from the call payload — it is always the `tenant_id` function parameter"). An agent-supplied tenant_id would allow horizontal privilege escalation.
**Do this instead:** tenant_id always comes from `resolve_tenant_from_bearer()` or `resolve_tenant_from_api_key()` at auth time and is threaded as a typed parameter, never extracted from JSON.

### Anti-Pattern 3: Classifying to Intent before tool selection

**What:** Run `ferro_ai::Classifier<Intent>` first, then pick a tool from the intent.
**Why wrong:** v15.0 tools are already intent-specific (each action has a name and description). Adding an Intent classification step requires maintaining a mapping from `Intent → [candidate tools]` that will drift from the actual ServiceDef. Direct tool-name classification with the tool list as the schema anchor is more robust and simpler.
**Do this instead:** Classify directly to `ToolSelection { tool_name, arguments, confidence }` using the projected tool descriptions as the classifier's system prompt.

### Anti-Pattern 4: Confirmation gating via a new MCP tool per confirmation request

**What:** Synthesize a unique `confirm_abc123` tool per pending action.
**Why wrong:** Tool lists in MCP are static within a session. Dynamic per-request tool names would require clients to re-fetch `tools/list` after every write, which breaks caching and most MCP client implementations.
**Do this instead:** Synthesize a stable `confirm_<action_name>` tool (one per destructive action, not per invocation). The tool accepts `confirmation_key: string`. The key is returned in the action's response payload, not as a tool name.

---

## Sources

- `ferro-mcp-server/src/renderer.rs` — existing `McpRenderer` implementation and `McpContext` stub
- `ferro-mcp-server/src/dispatch.rs` — tenant scoping implementation and fail-closed guarantee
- `ferro-mcp-server/src/jsonrpc.rs` — existing JSON-RPC method handlers
- `ferro-mcp-server/src/schema.rs` — `build_input_schema`, `is_filter_field`, `data_type_to_json_schema`
- `ferro-mcp-oauth/src/validate.rs` — `validate_bearer`, `BearerCheck`, `McpTokenClaims`
- `ferro-projections/src/render/mod.rs` — `Renderer` trait, `BaseContext`, `Verbosity`
- `ferro-projections/src/service.rs` — `ServiceDef` with `mcp_exposed`, `tenant_column`, `mcp_ability`
- `ferro-projections/src/action.rs` — `ActionDef`, `InputDef`, `GuardDef`
- `ferro-projections/CLAUDE.md` — boundary rule: renderers live in output crates
- `ferro-ai/src/classifier/mod.rs` — `Classifier<T>`, `ClassifierConfig`, `ClassificationResult`
- `ferro-ai/src/tools/mod.rs` — `ToolRegistry`, `ToolDef`, `make_handler`, dispatch loop
- `ferro-ai/src/confirmation/mod.rs` — `ConfirmationStore`, `InMemoryConfirmationStore`
- `.planning/PROJECT.md` — v15.0 milestone scope, v12.6 consumer MCP OAuth baseline

---
*Architecture research for: Ferro v15.0 Agent-Operable App (Consumer MCP)*
*Researched: 2026-06-13*
