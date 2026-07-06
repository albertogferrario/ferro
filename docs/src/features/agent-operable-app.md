# Agent-Operable App (Consumer MCP)

A Ferro application can expose its own data and guarded actions to an AI agent through a
per-tenant MCP endpoint. The tools an agent sees are derived from the same `ServiceDef`
projections that drive the visual and text renderers — reads, guarded writes, and a
natural-language turn — so an agent operates the business through the same authorization,
tenant-scoping, and guard rules as the dashboard.

This page is an end-to-end worked example. It builds on the two authentication pages —
[MCP OAuth Authorization Server](mcp-oauth.md) and
[MCP Per-Tenant API-Key Auth](mcp-api-key-auth.md) — which secure the `/mcp` endpoint; here we
add the projection-derived tools, write dispatch, confirmation gating, and the inbound
intent loop on top of them.

## The example domain

An order-fulfillment workflow: an order moves through a guarded state machine
(`draft → submitted → approved → shipped → delivered/cancelled`), where approval requires a
manager. The agent should be able to list a tenant's orders and advance their state — but a
destructive transition must be confirmed, and a non-manager must not be able to approve.

## Step 1 — Define the projection with guarded actions

The tool surface is derived entirely from a `ServiceDef`. Opting a projection into MCP
(`mcp_exposed`) and declaring its actions is all that is required — no per-tool code.

```rust
use ferro::{
    ActionDef, DataType, FieldMeaning, GuardDef, ServiceDef, StateDef, StateMachine, Transition,
};

pub fn service_def() -> ServiceDef {
    ServiceDef::new("order")
        .mcp_exposed(true)            // expose this projection over /mcp
        .tenant_column("tenant_id")   // every query/write is scoped to this column
        .mcp_ability("view-orders")   // Gate ability required to call the read tool
        .display_name("Order")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("customer_name", DataType::String, FieldMeaning::EntityName)
        .field("total", DataType::Float, FieldMeaning::Money)
        .field("status", DataType::String, FieldMeaning::Status)
        .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
        .state_machine(
            StateMachine::new("order_lifecycle")
                .initial("draft")
                .state(StateDef::new("draft"))
                .state(StateDef::new("submitted"))
                .state(StateDef::new("approved"))
                .state(StateDef::new("shipped"))
                .state(StateDef::new("delivered").final_state())
                .state(StateDef::new("cancelled").final_state())
                .transition(Transition::new("draft", "submit", "submitted"))
                .transition(Transition::new("submitted", "approve", "approved").guard("is_manager"))
                .transition(Transition::new("submitted", "reject", "cancelled"))
                .transition(Transition::new("approved", "ship", "shipped"))
                .transition(Transition::new("shipped", "deliver", "delivered"))
                .transition(Transition::new("draft", "cancel", "cancelled")),
        )
        .guard(GuardDef::new("is_manager").display_name("Manager Approval Required"))
        .action(ActionDef::new("submit").transition_trigger("submit"))
        .action(
            ActionDef::new("approve")
                .transition_trigger("approve")
                .precondition("is_manager"),
        )
        .action(ActionDef::new("ship").transition_trigger("ship"))
        .belongs_to("customer", "user")
        .has_many("line_items", "line_item")
}
```

This single definition derives the entire tool surface:

| Tool | Derived from | Kind |
|------|--------------|------|
| `list_order` | the projection's fields | read |
| `submit`, `approve`, `ship` | each `ActionDef` | write |
| `request_confirm_<action>` / `confirm_<action>` | each `transition_trigger` action (destructive) | confirmation |

An action's `transition_trigger` marks it destructive, which is what synthesizes the two-step
confirmation tools. A `precondition` (here `is_manager`) is the guard re-evaluated server-side
at call time.

## Step 2 — Mount the endpoint

The MCP endpoint is a single route group behind the bearer + tenant middleware. The optional
`/mcp/chat` route adds the natural-language turn (Step 5).

```rust
group!("/", {
    post!("/mcp", controllers::mcp::handle).name("mcp.endpoint"),
    post!("/mcp/chat", controllers::mcp_chat::handle_chat).name("mcp.chat"),
})
.middleware(BearerAuthMiddleware { mcp_config: McpServerConfig::from_env() })
.middleware(
    TenantMiddleware::new()
        .resolver(JwtClaimResolver::new("tenant_id", crate::tenant_lookup::get()))
        .on_failure(TenantFailureMode::Forbidden),
);
```

`BearerAuthMiddleware` accepts either an OAuth JWT or a per-tenant `ferro_` API key (see the
two auth pages); both resolve to the same tenant context, which the dispatch paths read.

## Step 3 — Wire the write dispatcher

Reads need no wiring. Writes need a `WriteDispatcher`: an **executor** that performs the
mutation tenant-scoped, and a **guard evaluator** that re-checks preconditions against the
live database. The framework never trusts the agent's view of guards — the evaluator runs at
call time and is **fail-closed** (an unknown guard name is denied, not allowed).

`WriteDispatcher`, the callback types, and the `dispatch_write` pipeline live on the
`ferro::write` facade. See the [Write Kernel](write-kernel.md) for the full pipeline and the
[Transition Planning](transition-planning.md) page for how the target state is derived from the
state machine — the same single kernel backs both this MCP path and the visual write path.

```rust
use ferro::write::{WriteDispatcher, WriteError};
use ferro::derive_transition_plan;

pub(crate) fn make_write_dispatcher() -> WriteDispatcher {
    WriteDispatcher::new(
        // executor: perform the tenant-scoped mutation
        Box::new(|action_name, inputs, tenant_id, db| {
            let action_name = action_name.to_string();
            let id_val = inputs["id"].as_i64();
            let db = db.clone();
            Box::pin(async move {
                let id = id_val
                    .ok_or_else(|| WriteError::Validation("missing id".into()))?;

                // find_for_tenant: filter by id AND tenant_id — None => cross-tenant denial.
                let order = Entity::find_by_id(id as i32)
                    .filter(Column::TenantId.eq(tenant_id))
                    .one(&db).await
                    .map_err(|e| WriteError::Database(e.to_string()))?
                    .ok_or_else(|| WriteError::Validation(
                        "not found or cross-tenant access denied".into()))?;

                // The target state is DERIVED from the StateMachine, not a hand-written
                // `match action_name => status`. derive_transition_plan reads Transition.to —
                // the single source of truth — so the executor and the state machine cannot drift.
                let svc = crate::projections::order::service_def();
                let plan = derive_transition_plan(&svc, &action_name)
                    .map_err(|e| WriteError::Validation(e.to_string()))?;

                let mut active: OrderActive = order.into();
                active.status = Set(plan.to_state.clone());
                let updated = active.update(&db).await
                    .map_err(|e| WriteError::Database(e.to_string()))?;
                Ok(json!({ "id": updated.id, "status": updated.status }))
            })
        }),
        // guard_evaluator: re-check preconditions against LIVE DB state, fail-closed
        Box::new(|guard_name, tenant_id, _inputs, db| {
            let guard_name = guard_name.to_string();
            let db = db.clone();
            Box::pin(async move {
                match guard_name.as_str() {
                    "is_manager" => Ok(check_is_manager(tenant_id, &db).await), // live DB check
                    // Fail-closed: an unrecognized guard is denied, never silently allowed.
                    _ => Err(WriteError::GuardFailed(format!(
                        "unknown guard '{guard_name}': no evaluator registered"))),
                }
            })
        }),
    )
}

pub(crate) fn exposed_services() -> Vec<ServiceDef> {
    vec![crate::projections::order::service_def()]
}
```

The executor's `find_by_id(id).filter(tenant_id)` pattern is what makes cross-tenant writes
structurally impossible: a row owned by another tenant resolves to `None` and the call fails
before any mutation. The target state comes from `derive_transition_plan` rather than a
hand-written status map, so the executor cannot drift from the declared state machine — the
same dispatcher is reused unchanged by the visual write path.

## Step 4 — Confirmation gating for destructive actions

Enable the `confirmation` feature and provide a store. Every action with a `transition_trigger`
is then gated by a two-step `request_confirm_<action>` → `confirm_<action>` flow; calling the
destructive tool directly returns a `confirmation_required` error instead of executing.

```toml
# Cargo.toml
[features]
confirmation = ["ferro-mcp-server/confirmation", "dep:ferro-ai"]
```

```rust
static CONFIRMATION_STORE: OnceLock<ferro_ai::InMemoryConfirmationStore> = OnceLock::new();

pub(crate) fn confirmation_store() -> &'static ferro_ai::InMemoryConfirmationStore {
    CONFIRMATION_STORE.get_or_init(ferro_ai::InMemoryConfirmationStore::new)
}
```

`request_confirm_<action>` validates inputs, re-evaluates guards, and mints a single-use,
`cfm_`-prefixed token bound to `(tenant, action, record)` with a TTL (default 300s);
`confirm_<action>` consumes the token exactly once, re-evaluates guards again, and executes.
The TTL is configured through `McpServerConfig`.

## Step 5 — The inbound natural-language turn (`/mcp/chat`)

`/mcp/chat` accepts `{ "message": "..." }`, classifies it to a tool + arguments with
`ferro-ai::Classifier`, and routes the result through the **same** read/write/confirm machinery
— it adds no parallel dispatch logic. The classifier output is treated as untrusted: a
classified tool name and arguments pass the identical validation, guard re-evaluation, and
tenant scoping as any direct tool call.

Enable the `ai-live` feature (it implies `confirmation` and pulls the live provider) and the
endpoint instantiates `AnthropicProvider` from the environment:

```toml
[features]
ai-live = ["ferro-mcp-server/ai-live", "ferro-mcp-server/confirmation", "dep:ferro-ai", "ferro-ai/llm", "confirmation"]
```

The loop is CI-testable without live-LLM spend: with `FERRO_AI_LIVE_EVAL` unset it runs from
recorded transcript fixtures through a reqwest-free replay provider, exercising every branch
with no network. Set `FERRO_AI_LIVE_EVAL=1` (with `ANTHROPIC_API_KEY`) to make a real call;
the live path announces an estimated cost before the first request. A low-confidence
classification returns a `needs_clarification` response rather than dispatching to the wrong
tool.

## What the agent sees — a real session

Authenticated as a tenant (here tenant 1, "Acme"), an agent lists the tools and operates on
real data. The tool surface and responses below are the actual endpoint output.

`tools/list` returns the derived surface:

```json
["list_order", "submit", "approve", "ship",
 "request_confirm_submit", "confirm_submit",
 "request_confirm_approve", "confirm_approve",
 "request_confirm_ship", "confirm_ship"]
```

`tools/call list_order` returns only the caller's rows, with a structured result:

```json
{ "result": {
    "content": [{ "type": "text", "text": "{\"rows\":[ ... ],\"total\":2}" }],
    "structuredContent": { "rows": [
        { "id": 1, "customer_name": "Alice Acme", "total": 120.0, "status": "submitted", "tenant_id": 1 },
        { "id": 2, "customer_name": "Alice Acme", "total": 85.5,  "status": "delivered", "tenant_id": 1 }
    ], "total": 2 },
    "isError": false } }
```

Calling a destructive tool directly is blocked:

```json
// tools/call submit { "id": 1 }
{ "result": {
    "structuredContent": { "error_kind": "confirmation_required",
        "message": "use request_confirm_submit first",
        "request_tool": "request_confirm_submit" },
    "isError": true } }
```

The two-step flow mints a token, then executes once:

```json
// tools/call request_confirm_submit { "id": 1 }
{ "result": { "structuredContent": {
    "confirmation_token": "cfm_…", "expires_in_seconds": 300 }, "isError": false } }

// tools/call confirm_submit { "confirmation_token": "cfm_…", "id": 1 }
{ "result": { "structuredContent": {
    "status": "ok", "action": "submit", "result": { "id": 1, "status": "submitted" } },
    "isError": false } }

// replay the same token => rejected (single-use)
{ "result": { "structuredContent": { "error_kind": "confirmation_expired" }, "isError": true } }
```

## Security properties

Every property is enforced server-side, regardless of what the agent (or a prompt-injected
message) claims:

- **Tenant isolation** — reads and writes are scoped to the authenticated tenant's
  `tenant_column`; a cross-tenant id resolves to `None` and the call fails. `tenant_id` comes
  from the authenticated principal, never from tool arguments.
- **Guard re-evaluation** — preconditions (`is_manager`) are checked against the live database
  at call time, fail-closed; the classifier's or agent's view of guards is never trusted.
- **Confirmation** — destructive actions require a server-minted, single-use, TTL-bound token;
  the same token cannot be replayed.
- **Authorization** — read tools require the projection's `mcp_ability` via the app `Gate`;
  the natural-language read path enforces the same ability as the direct path.
- **Untrusted classification** — `/mcp/chat` arguments enter the identical validation and
  dispatch pipeline as any direct call; classification is an entry point, not a trust shortcut.

## See also

- [MCP OAuth Authorization Server](mcp-oauth.md) — the browser authorization-code and device flows.
- [MCP Per-Tenant API-Key Auth](mcp-api-key-auth.md) — the `ferro_` key path for headless agents.
- [Write Kernel](write-kernel.md) — the channel-agnostic `dispatch_write` pipeline this page's writes run through.
- [Transition Planning](transition-planning.md) — deriving the write target from the state machine.
- [Multi-Tenancy](multi-tenancy.md) — the tenant context the dispatch paths read.
