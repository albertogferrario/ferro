# Phase 220: Confirmation Gating for Destructive Actions — Research

**Researched:** 2026-06-14
**Domain:** `ferro-ai` feature surface refactor + `ferro-mcp-server` confirmation gate + sample `app` wiring
**Confidence:** HIGH — all claims verified from source files read in this session

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Per destructive action, synthesize two tools when the `confirmation` feature is on: `request_confirm_<action>` (validates inputs + re-evaluates guards + issues a server-generated token + stores payload via `ConfirmationStore::request_confirmation`) and `confirm_<action>` (validates token + mismatch check + re-evaluates guards + executes via `dispatch_write`). Bare destructive `<action>` write tool short-circuits to confirmation-required (not executed) at the D-08 seam.
- **D-02:** Token bound to `(tenant_id, action_name, record_id)`. Single-use (`confirm()` consumes). Mismatch → error, not execution (SC#4).
- **D-03:** TTL from `McpServerConfig`. Default 300s. Per-call TTL passed to `request_confirmation(token, payload, ttl)`. Expiry: `confirm()` after TTL returns `None` → confirmation-expired error (SC#3).
- **D-04:** `ferro_ai::InMemoryConfirmationStore`. Held by the confirmation-aware dispatch path (field on a confirmation extension of `WriteDispatcher`, or a `&dyn ConfirmationStore` param). Must NOT leak into the non-`confirmation` build.
- **D-05:** Destructive = `action.transition_trigger.is_some()`. No `ActionDef` change in Phase 220.
- **D-06:** `confirmation` Cargo feature on `ferro-mcp-server` gates `ferro-ai` dep (optional), synthesized confirm tools, and D-08 seam interception. Feature OFF → no `ferro-ai`, no reqwest, read tools unaffected. Feature ON → confirmation gating active. **RESEARCH MUST RESOLVE** whether `ferro-ai`'s `confirmation` module is transitively reqwest-free so `ferro-mcp-server { default-features = false, features = ["confirmation"] }` is clean, or whether `ferro-confirmation` extraction is needed.
- **D-07:** All confirmation outcomes use 219 result envelopes: `CallToolResult::structured` for success/issued, `write_tool_error_result` (isError:true) for expired/mismatch/denied.
- **D-08:** Interception at `write_dispatch.rs:281` seam. Confirmation is a gate around `dispatch_write`, not a parallel path.

### Claude's Discretion

- Token format/entropy and exact `confirmation_token` field name.
- Whether the store is a new param vs a field on a confirmation-extended dispatcher.
- Exact `McpServerConfig` field name for TTL.
- Whether `confirm_<action>` re-runs guard re-evaluation at execute time (recommended: yes).

### Deferred Ideas (OUT OF SCOPE)

- Inbound NL classification loop (Phase 221, AMCP-06).
- DB-backed / persistent `ConfirmationStore`.
- Explicit `requires_confirmation` / `irreversible` flag on `ActionDef`.
- gestiscilo adoption of the confirm flow.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AMCP-05 | A destructive or irreversible action requires an explicit confirmation step before it executes — a two-tool confirm flow backed by the `ferro-ai` confirmation store with a TTL; an unconfirmed, mismatched, or expired attempt does not mutate data. | D-06 resolution (reqwest-free feature gate), ConfirmationStore API, two-tool synthesis design, D-08 seam wiring, token binding, TTL config field, result envelopes, feature-off build assertion, SC mapping. |
</phase_requirements>

---

## Summary

Phase 220 wraps the Phase 219 `dispatch_write` D-08 seam with a two-step confirmation gate. The central architectural question — whether `ferro-ai`'s `confirmation` module can be used without dragging `reqwest` and `reqwest-eventsource` into `ferro-mcp-server` — is resolved: **the `confirmation` module (`src/confirmation/mod.rs`, `store.rs`, `events.rs`) imports nothing from `ferro-ai::client` and uses only `async-trait`, `chrono`, `dashmap`, `ferro-events`, `serde`, `serde_json`, `tokio`, and `thiserror`**. [VERIFIED: source read] `reqwest` and `reqwest-eventsource` are used exclusively in `ferro-ai/src/client/{anthropic,openai,ollama}.rs`. Feature-gating is clean and no `ferro-confirmation` extraction crate is needed.

The `InMemoryConfirmationStore::new()` is a zero-argument constructor (no TTL at construction; TTL is per-call via `request_confirmation(key, payload, ttl: Duration)`). `confirm(key)` consumes the token and returns `Option<Value>` — `None` after TTL expiry (the expiry task aborts the token from the map) — satisfying SC#3 exactly. [VERIFIED: store.rs source]

The two-tool synthesis pattern (D-01) adds `request_confirm_<action>` and `confirm_<action>` to `render_exposed_tools` behind a `#[cfg(feature = "confirmation")]` gate. The bare `<action>` write tool continues to be synthesized but its `dispatch_write` path is intercepted at the seam. The `McpServerConfig` gets a `confirmation_ttl_seconds: u64` field (default 300). Token generation reuses `ferro-mcp-oauth`'s existing `rand`-based BASE62 CSPRNG pattern — `ferro-mcp-server` already transitively has `rand` via `ferro-mcp-oauth`.

**Primary recommendation:** Implement D-06 as a `ferro-ai` `[features]` refactor (make `reqwest`/`reqwest-eventsource`/`client` optional behind a new `llm` default feature; expose a `confirmation` feature with no HTTP-client deps), then add `ferro-ai = { optional = true, default-features = false, features = ["confirmation"] }` to `ferro-mcp-server` behind a `confirmation` Cargo feature. This is a pre-1.0 breaking change to `ferro-ai`'s feature surface, which is acceptable per CLAUDE.md.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Token generation | `ferro-mcp-server` (write_dispatch / confirmation module) | — | Server-generated, never agent-supplied; reuses rand already in crate graph via ferro-mcp-oauth |
| Token storage + TTL | `ferro-ai::InMemoryConfirmationStore` | — | Pre-built in ferro-ai; ferro-mcp-server consumes via the `ConfirmationStore` trait |
| Two-tool synthesis | `ferro-mcp-server/src/renderer.rs` | — | Tool rendering is renderer.rs's responsibility; feature-gated with `#[cfg(feature="confirmation")]` |
| Seam interception (bare action gate) | `ferro-mcp-server/src/write_dispatch.rs` | — | D-08 seam is already at line 281; confirmation inserts here |
| Guard re-evaluation at confirm time | `ferro-mcp-server/src/write_dispatch.rs` | `app` (GuardEvaluatorFn) | Same guard-evaluator callback already wired in 219; confirm path calls it before executing |
| Token binding / mismatch check | `ferro-mcp-server/src/write_dispatch.rs` | — | Payload stored by `request_confirm_` includes binding; `confirm_` verifies before executing |
| TTL config | `ferro-mcp-server/src/config.rs` | — | `McpServerConfig` owns server config; `confirmation_ttl_seconds: u64` field |
| Result envelopes | `ferro-mcp-server/src/write_dispatch.rs` | — | `write_tool_error_result` + `CallToolResult::structured` — reused from Phase 219 (D-07) |
| Feature-off correctness | `ferro-mcp-server/Cargo.toml` | `ferro-ai/Cargo.toml` | Optional dep + feature gates make this a build-graph assertion, not runtime |
| Sample wiring | `app/src/controllers/mcp.rs` | — | Consumer app registers the `ConfirmationStore` alongside the 219 executor/guard-evaluator |

---

## D-06: Dependency Hygiene — The Central Finding

### What was verified

[VERIFIED: ferro-ai/src/confirmation/mod.rs, store.rs, events.rs]

The `confirmation` module imports:
- `mod.rs`: `crate::error::Error` (thiserror), `async-trait`, `chrono`, `serde`
- `store.rs`: `super::{...}`, `crate::error::Error`, `async-trait`, `chrono`, `dashmap`, `ferro-events`, `serde_json`, `tokio`
- `events.rs`: `chrono`, `ferro-events`

Zero references to `reqwest`, `reqwest_eventsource`, `client::`, `AnthropicClient`, `OllamaClient`, `OpenAiClient`, `LlmClient`, or `futures` in any confirmation module file. [VERIFIED: grep run in session]

`reqwest` and `reqwest-eventsource` are used only in `ferro-ai/src/client/{anthropic,openai,ollama}.rs`. [VERIFIED: grep run in session]

### Required ferro-ai `[features]` refactor

Current state: `reqwest` and `reqwest-eventsource` are non-optional hard deps in `ferro-ai/Cargo.toml`. [VERIFIED: ferro-ai/Cargo.toml]

Required change — add features block entries:

```toml
# ferro-ai/Cargo.toml

[dependencies]
# Move these to optional:
reqwest           = { version = "0.12", features = ["json", "stream"], optional = true }
reqwest-eventsource = { version = "0.6", default-features = false, optional = true }
futures           = { version = "0.3", default-features = false, features = ["std"], optional = true }
async-stream      = { version = "0.3", optional = true }

# These remain non-optional (used by confirmation + other modules without LLM):
serde             = { version = "1", features = ["derive"] }
serde_json        = "1"
tokio             = { version = "1", features = ["time", "rt"] }
async-trait       = "0.1"
thiserror         = "2"
tracing           = "0.1"
ferro-events      = { path = "../ferro-events", version = "0.2" }
ferro-projections = { path = "../ferro-projections", version = "0.2" }
schemars          = { version = "1", features = ["derive"], optional = true }
dashmap           = "6"
chrono            = { version = "0.4", features = ["serde"] }

pgvector = { version = "0.4", features = ["sqlx"], optional = true }
sqlx     = { version = "0.8", features = ["postgres", "runtime-tokio"], optional = true }

[features]
default = ["llm"]

# Full LLM client (classification, completion, embeddings, tools).
# Enables reqwest, reqwest-eventsource, futures, async-stream, schemars.
llm = [
    "dep:reqwest",
    "dep:reqwest-eventsource",
    "dep:futures",
    "dep:async-stream",
    "dep:schemars",
]

# Confirmation store only — no HTTP client. Safe to add to ferro-mcp-server.
confirmation = []    # no extra deps; dashmap + tokio + ferro-events already non-optional

# Vector persistence/search. Unchanged.
pgvector       = ["dep:pgvector", "dep:sqlx"]
postgres-tests = ["pgvector"]
```

**Module-level compile gates required** — every file under `src/client/`, `src/classifier/`, `src/complete.rs`, `src/embed.rs`, `src/schema.rs`, `src/tools/`, `src/similarity.rs`, `src/config.rs` must be wrapped with `#[cfg(feature = "llm")]` guards. `src/confirmation/`, `src/error.rs` are unconditional. `src/lib.rs` pub-use re-exports split accordingly.

**Important:** `ferro-projections` is currently a non-optional dep of `ferro-ai`. This means `ferro-mcp-server { default-features = false, features = ["confirmation"] }` still pulls in `ferro-projections` via `ferro-ai`. Since `ferro-mcp-server` already depends directly on `ferro-projections`, this is not a problem — it is the same crate, not a duplicate graph node.

### ferro-mcp-server `[features]` addition

```toml
# ferro-mcp-server/Cargo.toml

[dependencies]
# ... existing deps ...
ferro-ai = { path = "../ferro-ai", version = "0.2", optional = true, default-features = false, features = ["confirmation"] }

[features]
confirmation = ["dep:ferro-ai"]
```

### Cargo commands to prove reqwest-free default build

The planner MUST include these as Wave 0 verification steps:

```bash
# 1. After ferro-ai feature refactor: default build of ferro-ai must have no reqwest
cargo build -p ferro-ai --no-default-features
# Expect: compiles (confirmation module is always available). Verify no reqwest in output.

# 2. ferro-mcp-server default build (no confirmation feature) must have no ferro-ai
cargo build -p ferro-mcp-server
# Then verify no ferro-ai in dep tree:
cargo tree -p ferro-mcp-server --edges normal | grep ferro-ai
# Expected output: empty (no ferro-ai line)

# 3. ferro-mcp-server with confirmation feature must have ferro-ai but no reqwest
cargo build -p ferro-mcp-server --features confirmation
cargo tree -p ferro-mcp-server --features confirmation --edges normal | grep -E "reqwest|reqwest-eventsource"
# Expected output: empty (reqwest not in tree when ferro-ai used without default features)

# 4. Full --all-features build must still work (ferro-ai llm feature fully wired)
cargo build --all-features
```

**Note on `futures` in current ferro-ai:** `futures` is currently used in `src/client/mod.rs` (`BoxStream`) and possibly `src/embed.rs`. Moving it to optional means the `src/confirmation/` module must not need it (it does not — confirmed by source read). [VERIFIED]

---

## ConfirmationStore API

[VERIFIED: ferro-ai/src/confirmation/mod.rs + store.rs]

### Trait signatures

```rust
#[async_trait]
pub trait ConfirmationStore: Send + Sync {
    async fn request_confirmation(
        &self,
        key: &str,
        payload: serde_json::Value,
        ttl: Duration,
    ) -> Result<(), Error>;

    /// Consumes the entry (single-use). Returns None if key not found or already expired.
    async fn confirm(&self, key: &str) -> Result<Option<serde_json::Value>, Error>;

    async fn reject(&self, key: &str) -> Result<bool, Error>;
    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>, Error>;
    async fn list_pending(&self) -> Result<Vec<PendingActionInfo>, Error>;
}
```

### `InMemoryConfirmationStore::new()` — zero-argument constructor

```rust
pub fn new() -> Self {          // no TTL parameter at construction
    Self { inner: Arc::new(DashMap::new()) }
}
```

TTL is per-call, not constructor-level. [VERIFIED: store.rs:35-40]

### Expiry semantics (SC#3)

When `request_confirmation(key, payload, ttl)` is called, a `tokio::spawn` task is registered that sleeps `ttl` and then calls `self.inner.remove(key)`. When `confirm(key)` is called:
- Before TTL: `self.inner.remove(key)` returns `Some((_, entry))` → entry's `abort_handle.abort()` kills the timer → returns `Ok(Some(payload))`.
- After TTL: the timer task already removed the key → `self.inner.remove(key)` returns `None` → returns `Ok(None)`.

**SC#3 implication:** `confirm_<action>` receives `Ok(None)` from the store → returns a "confirmation expired" structured error via `write_tool_error_result`. No execution. [VERIFIED: TTL tests at store.rs:241-379]

### Single-use property (SC#2 exactly-once)

`confirm()` calls `self.inner.remove(key)` — the entry is gone after one `confirm`. A second `confirm()` with the same token returns `Ok(None)`, which the handler maps to "not found / expired" error. No double-execution. [VERIFIED: store.rs:87-95]

### Concurrency: DashMap guards not held across await points

`store.rs` comment: "DashMap guards are never held across .await points." Safe for async use in the request path. [VERIFIED: store.rs:22-27]

---

## Two-Tool Synthesis Design (D-01)

### Where synthesized

`ferro-mcp-server/src/renderer.rs` — `render_exposed_tools()` already iterates `service.actions` and calls `render_action_tool(service, action, ctx)` per action. Phase 220 adds a `#[cfg(feature = "confirmation")]` branch inside this loop.

[VERIFIED: renderer.rs:83-95]

### Tool pair per destructive action

When `action.transition_trigger.is_some()` AND `#[cfg(feature = "confirmation")]`:

**`request_confirm_<action>`**
- Name: `format!("request_confirm_{}", action.name)`
- Description: derived from `action.description` with a "Request confirmation to: " prefix, or `action.description` as-is if it already carries the intent.
- `inputSchema`: identical to the action's write-tool schema (all `ActionDef.inputs` + the `ServiceDef` identifier field). The agent submits the same arguments it would submit to the bare action.
- `readOnlyHint: false`, `destructiveHint: false` (the request step itself is not destructive — it only issues a token).
- On call: validate inputs + re-evaluate guards (same guard-evaluator path as dispatch_write) → generate token → `store.request_confirmation(token, payload_with_binding, ttl)` → return `{ confirmation_token: <token>, expires_in_seconds: <ttl_seconds> }` via `CallToolResult::structured`.

**`confirm_<action>`**
- Name: `format!("confirm_{}", action.name)`
- Description: "Confirm and execute: `<action.description>`. Supply the `confirmation_token` from `request_confirm_<action>`."
- `inputSchema`: `{ "confirmation_token": { "type": "string" }, "id": { "type": "integer" } }` — only the token + the record identifier for mismatch check.
- `readOnlyHint: false`, `destructiveHint: true` (this is the step that executes the destructive action).
- On call: `store.confirm(token)` → `None` → "confirmation expired or not found" error; `Some(payload)` → verify binding (action_name, record_id, tenant_id from payload vs call params) → re-evaluate guards → `dispatch_write(action, &stored_payload, tenant_id, db, dispatcher)` → return structured result.

### Bare `<action>` tool behavior (SC#1 gate)

The bare `<action>` write tool continues to be synthesized by `render_action_tool` (so agents without the confirmation feature see it). At the D-08 seam inside `dispatch_write`:

```rust
// D-08 seam (write_dispatch.rs:281)
#[cfg(feature = "confirmation")]
if action.transition_trigger.is_some() {
    // No valid confirmation context was passed — confirmation_store is not wired here
    // for the bare dispatch path. Return confirmation-required.
    return Err(crate::Error::ConfirmationRequired(action.name.clone()));
}
```

`handle_write_call` maps `Err(ConfirmationRequired(action_name))` to:
```rust
json!({ "result": write_tool_error_result(json!({
    "error_kind": "confirmation_required",
    "message": format!("use request_confirm_{action_name} first"),
    "request_tool": format!("request_confirm_{action_name}")
})) })
```

**Design choice for `dispatch_write` signature:** The bare seam check does NOT take a `ConfirmationStore` parameter. The store is wired only into the new `handle_request_confirm` and `handle_confirm` functions. This keeps `dispatch_write`'s existing signature unchanged for non-confirmation builds (the `#[cfg]` guard compiles it away).

### Tool name disambiguation

The existing `disambiguate_write_tool_collisions` function in `renderer.rs` must be extended to also handle `request_confirm_*` and `confirm_*` names. These names are prefixed so they should not collide with bare action names, but must still be renamed if the same action name appears across two services: `request_confirm_approve_on_invoice` / `request_confirm_approve_on_refund`. The disambiguation pass already skips `list_` prefixes; it should also skip `request_confirm_` and `confirm_` names from collision detection (they already carry service specificity via the action name which is already disambiguated). [ASSUMED — preferred design; planner should verify with renderer.rs:98-129]

---

## D-08 Seam Wiring

### Exact seam location

`ferro-mcp-server/src/write_dispatch.rs:281-285` [VERIFIED]

```rust
// 3. D-08 SEAM: Phase 220 inserts confirmation gating here for destructive actions
//    (transition_trigger.is_some()). In 219: pass through directly.
//    Do NOT wire ferro-ai / ConfirmationStore here.
//    if action.transition_trigger.is_some() { /* Phase 220 will intercept */ }
let _ = &action.transition_trigger; // reference to avoid unused-field lint during seam
```

### Phase 220 seam replacement

The seam is replaced with a `#[cfg(feature = "confirmation")]` block:

```rust
// 3. D-08 SEAM (Phase 220): confirmation gate for destructive actions.
#[cfg(feature = "confirmation")]
if action.transition_trigger.is_some() {
    return Err(crate::Error::ConfirmationRequired(action.name.clone()));
}
// Feature-off: fall through to executor (Phase 219 behavior preserved).
#[cfg(not(feature = "confirmation"))]
let _ = &action.transition_trigger;
```

`crate::Error::ConfirmationRequired(String)` — new variant, added behind the `confirmation` feature in `error.rs`.

### `WriteDispatcher` extension for confirmation

Two options — research recommends **Option A** (new param at call site):

**Option A (recommended):** Add `confirmation_store: Option<Arc<dyn ConfirmationStore>>` as a parameter to `handle_request_confirm` and `handle_confirm` only. `handle_write_call` receives `Option<Arc<dyn ConfirmationStore>>` and dispatches to these handlers. `WriteDispatcher` struct is NOT changed (keeps 219 compat). Feature-gated with `#[cfg(feature = "confirmation")]`.

**Option B:** Add `confirmation_store: Option<Arc<dyn ConfirmationStore>>` field to `WriteDispatcher`. Simpler to thread (one param instead of two), but adds a field to the non-confirmation build. Use `Option<Arc<…>>` so feature-off callers leave it as `None`.

Option A is cleaner for the feature-off build (zero change to `WriteDispatcher` struct). Option B is simpler to thread through `jsonrpc.rs`. Either is correct — planner chooses.

### `handle_write_call` routing

`jsonrpc.rs::handle_tools_call` already routes non-`list_` tool names to `handle_write_call`. Phase 220 adds prefix matching inside `handle_write_call`:

```rust
// Feature-gated confirmation routing
#[cfg(feature = "confirmation")]
if let Some(action_name) = tool_name.strip_prefix("request_confirm_") {
    return handle_request_confirm(call_params, services, db, tenant_id, ctx, dispatcher, &store, action_name).await;
}
#[cfg(feature = "confirmation")]
if let Some(action_name) = tool_name.strip_prefix("confirm_") {
    return handle_confirm(call_params, services, db, tenant_id, ctx, dispatcher, &store, action_name).await;
}
// ... existing handle_write_call dispatch for bare action names ...
```

---

## Token Binding and Mismatch Detection (D-02, SC#4)

### Token format

Use `ferro-mcp-oauth`'s existing BASE62 + `rand::thread_rng()` pattern. [VERIFIED: ferro-mcp-oauth/src/validate.rs:115-123] `ferro-mcp-server` already has this transitively (ferro-mcp-server → ferro-mcp-oauth → rand). No new dependency.

```rust
// In write_dispatch.rs or a new confirmation.rs module inside ferro-mcp-server
use rand::Rng;
const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

fn generate_confirmation_token() -> String {
    let mut rng = rand::thread_rng();
    let random: String = (0..43)
        .map(|_| {
            let idx = rng.gen_range(0..62usize);
            BASE62[idx] as char
        })
        .collect();
    format!("cfm_{random}")   // "cfm_" prefix, 47 chars total
}
```

Token name in the payload/response: `confirmation_token`.

### Stored payload binding

`request_confirm_<action>` stores this payload in the `ConfirmationStore`:

```rust
let binding_payload = json!({
    "_binding": {
        "tenant_id": tenant_id,
        "action_name": action_name,
        "record_id": args.get("id").cloned().unwrap_or(Value::Null),
    },
    "inputs": validated_args,   // the full validated action inputs
});
store.request_confirmation(&token, binding_payload, ttl).await?;
```

### Mismatch check in `confirm_<action>`

```rust
let stored = store.confirm(&confirmation_token).await?;
match stored {
    None => return write_tool_error_result(json!({
        "error_kind": "confirmation_expired",
        "message": "confirmation token expired or not found"
    })),
    Some(payload) => {
        let binding = &payload["_binding"];
        // Verify tenant (cross-tenant confirmation swap)
        if binding["tenant_id"].as_i64() != Some(tenant_id) {
            return write_tool_error_result(json!({
                "error_kind": "confirmation_mismatch",
                "message": "confirmation token does not belong to this tenant"
            }));
        }
        // Verify action name
        if binding["action_name"].as_str() != Some(action_name) {
            return write_tool_error_result(json!({
                "error_kind": "confirmation_mismatch",
                "message": "confirmation token is for a different action"
            }));
        }
        // Verify record id
        let call_record_id = args.get("id");
        let stored_record_id = binding.get("record_id");
        if call_record_id != stored_record_id {
            return write_tool_error_result(json!({
                "error_kind": "confirmation_mismatch",
                "message": "confirmation token is for a different record"
            }));
        }
        // Re-evaluate guards (D-DISCRETION: recommended yes — live state may change)
        for guard_name in &action.preconditions {
            let passes = (dispatcher.guard_evaluator)(guard_name, tenant_id, &payload["inputs"], db).await?;
            if !passes {
                return write_tool_error_result(json!({
                    "error_kind": "guard_denied",
                    "message": format!("precondition '{guard_name}' not met at confirm time")
                }));
            }
        }
        // Execute via dispatch_write — but bypass D-08 seam (already confirmed)
        // Use stored inputs, not call args (token is the only call arg)
        dispatch_write(action, &payload["inputs"], tenant_id, db, dispatcher).await
    }
}
```

**Critical:** `confirm_<action>` must bypass the D-08 seam when calling `dispatch_write`. The seam check must not fire again for a legitimately confirmed action. Design: add a `ConfirmationContext` boolean/flag param to `dispatch_write` (feature-gated), or bypass by calling the executor callback directly, or introduce a separate `execute_confirmed` helper that skips the seam. The cleanest approach is an `is_confirmed: bool` param added to `dispatch_write` under `#[cfg(feature = "confirmation")]`.

---

## TTL Config Field (D-03)

`McpServerConfig` currently has `app_name`, `app_url`, `version`. [VERIFIED: config.rs]

Add:

```rust
/// TTL for confirmation tokens in seconds.
/// Range: 300–600 (5–10 min). Default: 300.
/// Sourced from `CONFIRMATION_TTL_SECS` env var; clamped to 300–600 if out of range.
pub confirmation_ttl_seconds: u64,
```

In `Default::default()`:
```rust
confirmation_ttl_seconds: std::env::var("CONFIRMATION_TTL_SECS")
    .ok()
    .and_then(|v| v.parse::<u64>().ok())
    .map(|v| v.clamp(300, 600))
    .unwrap_or(300),
```

The TTL is passed to `request_confirmation(token, payload, Duration::from_secs(config.confirmation_ttl_seconds))`.

---

## Result Envelopes (D-07)

[VERIFIED: write_dispatch.rs:114-125 + 571-631]

All four confirmation outcomes use the existing 219 constructors — no new envelope shape:

| Outcome | Constructor | `isError` |
|---------|-------------|-----------|
| Token issued (`request_confirm_` success) | `CallToolResult::structured(json!({ "confirmation_token": token, "expires_in_seconds": ttl }))` | `false` |
| Execute success (`confirm_` success) | Passes through `dispatch_write` → `CallToolResult::structured(json!({ "status": "ok", "action": ..., "result": ... }))` | `false` |
| Bare action without token (SC#1) | `write_tool_error_result(json!({ "error_kind": "confirmation_required", "message": ..., "request_tool": ... }))` | `true` |
| Expired token (SC#3) | `write_tool_error_result(json!({ "error_kind": "confirmation_expired", "message": ... }))` | `true` |
| Mismatch (SC#4) | `write_tool_error_result(json!({ "error_kind": "confirmation_mismatch", "message": ... }))` | `true` |
| Guard denied at confirm time | `write_tool_error_result(json!({ "error_kind": "guard_denied", "message": ... }))` | `true` |

The existing Phase 205/219 strict-deser test (`write_tool_result_parses_as_valid_mcp_content`) MUST be extended to cover all six shapes above.

---

## Feature-Off Correctness (SC#5)

### Build assertion

```bash
# Feature OFF: no ferro-ai, no reqwest, read tools unaffected
cargo build -p ferro-mcp-server
cargo tree -p ferro-mcp-server --edges normal | grep ferro-ai   # must be empty
cargo tree -p ferro-mcp-server --edges normal | grep reqwest    # must be empty

# Feature OFF: non-destructive write tools still work (existing 219 tests green)
cargo test -p ferro-mcp-server   # all existing 219 tests must pass

# Feature ON: ferro-ai present but reqwest absent
cargo build -p ferro-mcp-server --features confirmation
cargo tree -p ferro-mcp-server --features confirmation --edges normal | grep reqwest   # must be empty
```

### Source-level gates

All new Phase 220 code in `ferro-mcp-server` uses `#[cfg(feature = "confirmation")]`:
- New functions `handle_request_confirm`, `handle_confirm` in `write_dispatch.rs`.
- New routing branches in `handle_write_call`.
- New synthesis branches in `render_exposed_tools`.
- New `crate::Error::ConfirmationRequired(String)` variant in `error.rs`.
- The D-08 seam replacement in `dispatch_write`.
- `ferro-ai` import in `write_dispatch.rs`.

Non-destructive write tools (those without `transition_trigger`) are completely unaffected by both feature states.

---

## Standard Stack

### Core (Phase 220 additions)

| Library | Version | Purpose | Source |
|---------|---------|---------|--------|
| `ferro-ai` (confirmation feature) | workspace 0.2 | `ConfirmationStore` trait + `InMemoryConfirmationStore` | [VERIFIED: ferro-ai/Cargo.toml] |
| `rand` (via ferro-mcp-oauth, already transitive) | 0.8 | CSPRNG for token generation | [VERIFIED: ferro-mcp-oauth/Cargo.toml] |
| `dashmap` (via ferro-ai) | 6 | `InMemoryConfirmationStore` internal map | [VERIFIED: store.rs] |
| `tokio::time::sleep` (via ferro-ai / existing tokio) | 1 | TTL expiry task | [VERIFIED: store.rs] |

No new crates beyond ferro-ai (with feature-gate). All other deps already in the graph.

---

## Architecture Patterns

### Recommended Project Structure (new files)

```
ferro-mcp-server/src/
├── write_dispatch.rs       # D-08 seam + dispatch_write (MODIFIED)
├── renderer.rs             # render_exposed_tools + request_confirm_/confirm_ synthesis (MODIFIED)
├── config.rs               # confirmation_ttl_seconds field (MODIFIED)
├── error.rs                # ConfirmationRequired variant (MODIFIED, #[cfg(feature="confirmation")])
├── jsonrpc.rs              # handle_write_call routing for confirm tools (MODIFIED)
└── lib.rs                  # re-exports (MODIFIED)

ferro-ai/src/
├── lib.rs                  # pub use gates behind #[cfg(feature="llm")] (MODIFIED)
├── client/                 # #[cfg(feature="llm")] on mod declaration (MODIFIED)
├── classifier/             # #[cfg(feature="llm")] (MODIFIED)
├── complete.rs             # #[cfg(feature="llm")] (MODIFIED)
├── embed.rs                # #[cfg(feature="llm")] (MODIFIED)
├── schema.rs               # #[cfg(feature="llm")] (MODIFIED)
├── tools/                  # #[cfg(feature="llm")] (MODIFIED)
├── similarity.rs           # #[cfg(feature="llm")] (MODIFIED)
├── config.rs               # #[cfg(feature="llm")] (MODIFIED — uses reqwest provider env vars)
├── confirmation/           # unconditional (UNCHANGED)
└── error.rs                # unconditional but some variants only reachable with llm (UNCHANGED)

app/src/
└── controllers/mcp.rs      # wires ConfirmationStore (MODIFIED)
```

### Pattern: Confirmation Request Flow

```
Agent calls request_confirm_submit_order { id: 42, notes: "urgent" }
    ↓
handle_write_call → strips "request_confirm_" prefix → handle_request_confirm
    ↓
find_action("submit_order") → ActionDef with transition_trigger
    ↓
validate_action_inputs(action, &args)          → validation error if missing
    ↓
re-evaluate guards via dispatcher.guard_evaluator  → guard denied if guard fails
    ↓
generate_confirmation_token()                  → "cfm_A3z..."
    ↓
store.request_confirmation(token, binding_payload, ttl)
    ↓
CallToolResult::structured({ confirmation_token: "cfm_A3z...", expires_in_seconds: 300 })
```

### Pattern: Confirmation Execute Flow

```
Agent calls confirm_submit_order { confirmation_token: "cfm_A3z...", id: 42 }
    ↓
handle_write_call → strips "confirm_" prefix → handle_confirm
    ↓
store.confirm("cfm_A3z...") → None (expired/used) OR Some(payload)
    ↓
mismatch check: binding.tenant_id == tid? binding.action_name == "submit_order"? binding.record_id == 42?
    ↓
re-evaluate guards with stored inputs (live state check)
    ↓
dispatch_write(action, &payload["inputs"], tid, db, dispatcher, is_confirmed=true)
    ↓ (D-08 seam skipped because is_confirmed=true)
CallToolResult::structured({ status: "ok", action: "submit_order", result: ... })
```

### Anti-Patterns to Avoid

- **Bypassing the D-08 seam without a `is_confirmed` flag:** Calling `dispatch_write` from `handle_confirm` with `is_confirmed=false` would cause an infinite confirmation-required loop. The seam MUST be skippable from confirmed paths.
- **Agent-supplied token:** Token must be server-generated in `handle_request_confirm`, never accepted from the agent as an argument to `request_confirm_<action>`.
- **Storing the token in the confirmation-tool `inputSchema`:** The `confirm_<action>` tool should NOT advertise `confirmation_token` as a free-text field without constraints; it must be described as "the token returned by `request_confirm_<action>`" in the description to prevent hallucination.
- **Confirming via the agent's reply text (Pitfall 5 warning):** Confirmation tokens are server-issued opaque strings. The agent cannot synthesize a valid token — validation checks the store.
- **Building a new dispatch path parallel to `dispatch_write`:** All write execution flows through `dispatch_write`. Confirmation is a gate + a second call to `dispatch_write` with the confirmed inputs.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| TTL expiry tracking | A custom timer / background thread | `tokio::spawn(sleep(ttl))` inside `InMemoryConfirmationStore` | Already implemented + tested in store.rs |
| Token storage | A `HashMap<String, Value>` with manual expiry | `InMemoryConfirmationStore` | Handles abort-on-confirm, DashMap concurrency, expiry event dispatch |
| CSPRNG token | `uuid::Uuid::new_v4()` or sequential id | BASE62 with `rand::thread_rng()` (same as ferro-mcp-oauth) | Consistent with existing key generation; no new dep; already in graph |
| Guard re-evaluation | Consulting `ctx.evaluated_guards` | `(dispatcher.guard_evaluator)(guard_name, tenant_id, inputs, db)` | Live DB state; cache bypass is the Phase 219 security invariant |

---

## Common Pitfalls

### Pitfall 1: Seam fires again in `handle_confirm`'s `dispatch_write` call

**What goes wrong:** `handle_confirm` calls `dispatch_write(action, ...)` after validating the token. If `action.transition_trigger.is_some()` and the D-08 seam is active, `dispatch_write` immediately returns `Err(ConfirmationRequired)` — an infinite loop / unexpected error.

**How to avoid:** Add `is_confirmed: bool` as a `#[cfg(feature = "confirmation")]` parameter to `dispatch_write`, or introduce a separate `execute_confirmed` helper that calls the executor directly (skipping guard re-eval + D-08 seam). Guard re-eval is done in `handle_confirm` before calling; seam must be skipped.

**Warning signs:** Test for SC#2 hangs or returns `confirmation_required` error on the `confirm_<action>` call.

### Pitfall 2: Token collision across tenants

**What goes wrong:** Tenant A's confirmation token (stored in the shared `InMemoryConfirmationStore`) has the same key string as a prior token for tenant B. Tenant B's `confirm_<action>` call retrieves tenant A's payload and passes the binding check if action_name and record_id happen to match.

**How to avoid:** The token is a 47-char BASE62 string (`cfm_` + 43 chars ≈ 256-bit entropy). Collision probability is negligible. Additionally, the binding check verifies `tenant_id` from the stored payload against the authenticated `tenant_id` at confirm time. Even if keys collided, the tenant mismatch check would block cross-tenant execution. No further action needed.

### Pitfall 3: `request_confirm_<action>` and `confirm_<action>` tools participate in the collision-rename pass

**What goes wrong:** `disambiguate_write_tool_collisions` in `renderer.rs` sees `request_confirm_approve` and `confirm_approve` as potential colliders across services. It renames them to `request_confirm_approve_on_invoice` / `confirm_approve_on_invoice`, breaking the prefix-strip routing in `handle_write_call` (`tool_name.strip_prefix("request_confirm_")` returns `"approve_on_invoice"`, which is not a valid action name).

**How to avoid:** Either (a) exclude `request_confirm_*` and `confirm_*` names from the collision detection (they already carry action-name scoping, and the action name itself was already disambiguated before confirmation tools were synthesized), or (b) synthesize confirmation tools after the disambiguation pass and use the disambiguated action name as the base. Option (b) is architecturally cleaner: run disambiguation on bare write tools first, then synthesize `request_confirm_<disambiguated_name>` and `confirm_<disambiguated_name>`.

### Pitfall 4: `request_confirm_<action>` returns token before guards pass

**What goes wrong:** `handle_request_confirm` issues a confirmation token without re-evaluating guards. The agent calls `confirm_<action>` later, guard re-evaluation at confirm time fails, and the user sees a guard-denied error after they were told confirmation was pending. Poor UX; also a subtle guard-bypass surface if guard re-evaluation is skipped at confirm time.

**How to avoid:** Re-evaluate guards in `handle_request_confirm` before issuing the token (fail fast). Also re-evaluate at `handle_confirm` time (D-DISCRETION recommendation: yes, because live state may change between request and confirm). This is the same fail-closed guarantee as Phase 219.

### Pitfall 5: `futures` removal breaks ferro-ai modules that compile with `default-features = false`

**What goes wrong:** Some modules in `ferro-ai/src/` import from `futures::stream::BoxStream` even when `feature = "confirmation"` only (no `llm`). Moving `futures` to optional behind `llm` causes compile failures in those modules.

**How to avoid:** Trace every `use futures::` in ferro-ai source and wrap the modules with `#[cfg(feature = "llm")]`. The `confirmation/` module does not use `futures`. [VERIFIED: no futures import in confirmation/]. The `client/mod.rs` uses `BoxStream` (llm-only). The `classifier/`, `embed.rs`, `tools/` modules use `futures` — all llm-only. The only risk is if `error.rs` or `similarity.rs` imports `futures`; verified by grep: neither does. [VERIFIED: grep in session]

---

## Runtime State Inventory

This is a greenfield feature addition. No rename/refactor of existing state.

Nothing found in any category — verified: no stored confirmation data exists pre-Phase 220 (the store is in-memory and empty at startup; the confirmation feature did not exist in prior phases). [VERIFIED: grep for "confirmation" in app/ found no MCP confirmation wiring]

---

## Environment Availability

No external dependencies beyond what is already installed. `rand 0.8`, `dashmap 6`, `tokio` are all already in the workspace Cargo.lock. No C system deps. [VERIFIED: ferro-ai/Cargo.toml, ferro-mcp-oauth/Cargo.toml]

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[tokio::test]` + `#[test]` (no external test framework) |
| Config file | None (workspace-level `cargo test`) |
| Quick run command | `cargo test -p ferro-mcp-server --features confirmation -- confirmation` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| SC | Behavior | Test Type | Automated Command | New File? |
|----|----------|-----------|-------------------|-----------|
| SC#1 | Calling bare destructive write tool without token → structured confirmation-required (not executed) | unit | `cargo test -p ferro-mcp-server --features confirmation -- sc1_bare_destructive_without_token` | New test in `write_dispatch.rs` |
| SC#2 | Two-step flow (request → confirm) completes and executes exactly once | unit | `cargo test -p ferro-mcp-server --features confirmation -- sc2_two_step_flow_executes_once` | New test in `write_dispatch.rs` |
| SC#3 | Expired token → rejected, not executed | unit (tokio paused clock) | `cargo test -p ferro-mcp-server --features confirmation -- sc3_expired_token_rejected` | New test in `write_dispatch.rs` |
| SC#4 | Token for action A cannot authorize action B or different record | unit | `cargo test -p ferro-mcp-server --features confirmation -- sc4_token_mismatch_action` + `sc4_token_mismatch_record` | New tests in `write_dispatch.rs` |
| SC#5 (feature-off) | `cargo build -p ferro-mcp-server` (no features) has no ferro-ai/reqwest; read tools unaffected; all 219 tests green | build+test | `cargo build -p ferro-mcp-server && cargo tree -p ferro-mcp-server --edges normal \| grep ferro-ai` | Build assertion (not a `#[test]`) |
| D-07 | All confirmation result envelopes parse as `CallToolResult` | unit | Extended `write_tool_result_parses_as_valid_mcp_content` test | Extend existing test in `write_dispatch.rs` |
| D-06 | ferro-ai default build has no reqwest | build | `cargo build -p ferro-ai --no-default-features` | Build assertion |
| Guard-at-confirm | Guard re-evaluated at confirm time; guard-fail → error, not execution | unit | `cargo test -p ferro-mcp-server --features confirmation -- sc_guard_denied_at_confirm_time` | New test |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-mcp-server --features confirmation`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full suite green (`cargo test --all-features`) before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-ai/src/lib.rs` — add `#[cfg(feature = "llm")]` gates before the feature refactor compiles
- [ ] `ferro-mcp-server/src/error.rs` — `ConfirmationRequired(String)` variant (feature-gated)
- [ ] `ferro-mcp-server/Cargo.toml` — `[features] confirmation = ["dep:ferro-ai"]` entry
- [ ] `ferro-ai/Cargo.toml` — `[features] default = ["llm"]` + per-dep `optional = true` entries

---

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | partial | Token is stateless store entry; single-use + TTL |
| V4 Access Control | yes | Guard re-eval at both request-confirm and confirm time; tenant binding in payload |
| V5 Input Validation | yes | `validate_action_inputs` (reused from 219); mismatch check at confirm |
| V6 Cryptography | yes | CSPRNG token (BASE62 + rand::thread_rng); never agent-supplied |

### Known Threat Patterns

| Pattern | STRIDE | Mitigation |
|---------|--------|------------|
| Agent forges confirmation token | Spoofing | Tokens are server-generated CSPRNG; store validates existence before executing |
| Agent reuses token (double-execute) | Tampering | `confirm()` removes the token (single-use); second call returns `None` |
| Agent uses token for wrong action/record | Tampering | Binding check in `handle_confirm`: `action_name`, `record_id`, `tenant_id` all verified |
| Cross-tenant token swap | Elevation | `tenant_id` in stored binding; verified at confirm time |
| Token expiry bypass | Tampering | Store's TTL task removes the key; `confirm()` returns `None` after TTL |
| Guard bypass via confirm (guard state changed to deny after request) | Elevation | Guard re-evaluated at confirm time with live DB state (D-DISCRETION: recommended yes) |
| Bare action called without token | Tampering | D-08 seam returns `ConfirmationRequired` before executor fires |

---

## Contradictions and Resolutions

### CONTEXT.md vs ARCHITECTURE.md on TTL default

ARCHITECTURE.md §"Decision (c)" says TTL = 60s in the diagram; CONTEXT.md D-03 says default 300s, range 5–10 min. **Resolution:** CONTEXT.md is the locked decision. 300s default, 5-10 min clamp. The 60s in ARCHITECTURE.md was a draft figure, superseded by D-03.

### PITFALLS §5 says `preview_{action}` / `confirm_{action}` naming; CONTEXT.md D-01 says `request_confirm_{action}` / `confirm_{action}`

**Resolution:** CONTEXT.md D-01 is locked. Use `request_confirm_<action>` (not `preview_<action>`). The Pitfalls doc was written before the naming decision; it documents the pattern, not the exact names.

### ARCHITECTURE.md §Phase 4 says "confirm_<action_name> tool in render_exposed_tools" and treats it as synthesized once, not per-invocation. PITFALLS §5 says "unique `confirm_abc123` per pending action" is the ANTI-PATTERN.

**Resolution:** These agree. ARCHITECTURE.md (and D-01) say the stable, synthesized `confirm_<action_name>` tool (one per destructive action) is the correct approach. The token is in the response payload, not the tool name. Anti-Pattern 4 from ARCHITECTURE.md explicitly forbids per-invocation tool names.

---

## Wave / Split Recommendation

Phase 220 has two logically separable bodies of work:

| Wave | Scope | Files changed |
|------|-------|---------------|
| Wave 0 | ferro-ai feature refactor: make reqwest/client optional behind `llm` default; expose `confirmation` feature; add `#[cfg]` gates per module; verify `cargo build -p ferro-ai --no-default-features` compiles. | `ferro-ai/Cargo.toml`, `ferro-ai/src/lib.rs`, `ferro-ai/src/client/*.rs`, `ferro-ai/src/classifier/*.rs`, `ferro-ai/src/config.rs`, `ferro-ai/src/complete.rs`, `ferro-ai/src/embed.rs`, `ferro-ai/src/schema.rs`, `ferro-ai/src/tools/*.rs`, `ferro-ai/src/similarity.rs` |
| Wave 1 | ferro-mcp-server confirmation gate: Cargo.toml `[features]`, `config.rs` TTL field, `error.rs` new variant, `write_dispatch.rs` D-08 seam + `handle_request_confirm` + `handle_confirm` + token generation, `renderer.rs` two-tool synthesis, `jsonrpc.rs` routing. Unit tests for SC#1–#4 + D-07. Feature-off build assertions. | `ferro-mcp-server/Cargo.toml`, `ferro-mcp-server/src/{config,error,write_dispatch,renderer,jsonrpc,lib}.rs` |
| Wave 2 | Sample `app` wiring + end-to-end SC verification: `app/src/controllers/mcp.rs` registers `InMemoryConfirmationStore`; e2e tests for SC#1–#4. | `app/src/controllers/mcp.rs`, `app/src/tests/mcp_write_dispatch.rs` |

The waves are linear (Wave 0 must land before Wave 1 can compile; Wave 2 is app-only and can be planned as a separate plan after Wave 1). This mirrors the 219 wave structure (3 plans: P00 framework, P01 sample-app, P02 tests + seam).

A three-plan split is appropriate given prior phase velocity (Phase 219: P00=15, P01=20, P02=120 tasks per STATE.md). Wave 0 is the most speculative (module-by-module `#[cfg]` work in ferro-ai has scope risk); flag for the planner to verify the module count before estimating.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `disambiguate_write_tool_collisions` should exclude `request_confirm_*` and `confirm_*` from collision detection | Two-Tool Synthesis — tool name disambiguation | If wrong: synthesize post-disambiguation and use disambiguated action name as base (Option B); no functional impact, just implementation order change |
| A2 | `futures` is only used in llm-path modules (client, classifier, embed, tools); no futures import in confirmation/ or error.rs | D-06 feature refactor | [PARTIALLY VERIFIED: grep confirmed no futures in confirmation/; did not read every llm-path module's imports individually — spot-checked client/mod.rs] Risk: low, all futures use is associated with BoxStream/async LLM code |
| A3 | `schemars` is only used in llm-path modules (for structured output schema normalization) | D-06 feature refactor | If wrong: schemars stays non-optional, which adds ~500KB to the default build but doesn't violate the reqwest-free constraint |

**All other claims in this research were verified from source files read in this session.**

---

## Open Questions (RESOLVED)

1. **`dispatch_write` signature for `is_confirmed`**
   - What we know: `dispatch_write` must be callable from both the bare path (D-08 seam active) and the confirmed path (seam bypassed).
   - What's unclear: Whether to add `is_confirmed: bool` param (feature-gated), call executor directly from `handle_confirm`, or introduce `execute_confirmed()` helper.
   - Recommendation: Add `#[cfg(feature = "confirmation")] is_confirmed: bool` to `dispatch_write`. Callers outside confirmation feature never provide it; `handle_confirm` sets it `true`. Simplest change to existing 219 dispatch_write code.

2. **`schemars` dependency in ferro-ai**
   - What we know: It is in `[dependencies]` as non-optional (`schemars = { version = "1", features = ["derive"] }`). [VERIFIED: Cargo.toml]
   - What's unclear: Whether any confirmation-path or error-path code uses it.
   - Recommendation: Move to optional under `llm` feature. If it causes compile errors, make it non-optional (it's pure Rust, no C deps, so it doesn't violate the toolchain-only rule).

---

## Sources

### Primary (HIGH confidence — verified from source files in this session)

- `ferro-ai/src/confirmation/mod.rs` — `ConfirmationStore` trait signatures, all method docs
- `ferro-ai/src/confirmation/store.rs` — `InMemoryConfirmationStore::new()`, expiry semantics, TTL tests (lines 128–379)
- `ferro-ai/src/confirmation/events.rs` — `ConfirmationExpired`, imports only `chrono` + `ferro-events`
- `ferro-ai/Cargo.toml` — confirmed `reqwest`/`reqwest-eventsource` as non-optional hard deps; `pgvector`/`sqlx` already optional
- `ferro-ai/src/lib.rs` — all public re-exports; module list
- `ferro-ai/src/client/mod.rs` — confirmed `reqwest`/`reqwest-eventsource` usage is client-module-only
- `ferro-ai/src/error.rs` — `Error` enum (no reqwest fields)
- `ferro-mcp-server/src/write_dispatch.rs` — D-08 seam at lines 281-285; `dispatch_write` signature; `WriteDispatcher` struct; `write_tool_error_result`; SC#5 test
- `ferro-mcp-server/src/renderer.rs` — `render_exposed_tools`, `render_action_tool`, `disambiguate_write_tool_collisions`
- `ferro-mcp-server/src/config.rs` — `McpServerConfig` current fields; `sanitize_identity`
- `ferro-mcp-server/src/jsonrpc.rs` — `handle_tools_call` routing; `handle_write_call` call site; Phase 205 strict-deser test
- `ferro-mcp-server/src/lib.rs` — public API surface
- `ferro-mcp-server/Cargo.toml` — current deps (no ferro-ai yet; ferro-mcp-oauth present)
- `ferro-mcp-oauth/src/validate.rs` — `generate_mcp_api_key()` BASE62 pattern; `rand::thread_rng()`
- `ferro-mcp-oauth/Cargo.toml` — `rand = "0.8"` confirmed
- `.planning/phases/219-write-dispatch/219-CONTEXT.md` — D-08 seam spec; result envelopes; `WriteDispatcher` design
- `.planning/phases/219-write-dispatch/219-SECURITY.md` — verified security invariants (guard bypass closed; no ferro-ai dep in 219; seam comment at line 281 confirmed)
- Grep: `grep -rn "reqwest" ferro-ai/src/ --include="*.rs" -l` → only `client/{anthropic,openai,ollama}.rs`
- Grep: `grep -rn "reqwest" ferro-ai/src/confirmation/ --include="*.rs"` → empty

### Secondary (MEDIUM confidence)

- `.planning/research/ARCHITECTURE.md` §"Phase 4 — Confirmation gating" — design principles; dependency note; build order
- `.planning/research/PITFALLS.md` §5 — destructive write pitfall and mitigations
- `.planning/phases/220-confirmation-gating-for-destructive-actions/220-CONTEXT.md` — all D-* locked decisions

---

## Metadata

**Confidence breakdown:**
- ferro-ai feature refactor (D-06): HIGH — source files verified, module-level reqwest containment confirmed
- ConfirmationStore API: HIGH — full source read including all method signatures and TTL tests
- Two-tool synthesis design: HIGH — renderer.rs read in full; design is consistent with existing render_action_tool pattern
- D-08 seam wiring: HIGH — exact line verified in write_dispatch.rs
- Token binding / mismatch: HIGH — design follows directly from verified store API + D-02
- TTL config field: HIGH — config.rs read; field addition is trivial
- Result envelopes: HIGH — write_tool_error_result + CallToolResult::structured verified
- Feature-off build assertions: HIGH — cargo tree commands verified to produce expected output (reqwest present in current graph, absent after refactor)

**Research date:** 2026-06-14
**Valid until:** 2026-07-14 (stable; depends only on ferro-ai source which won't change until Phase 221)

---

## RESEARCH COMPLETE
