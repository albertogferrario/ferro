# Phase 220: Confirmation Gating for Destructive Actions - Pattern Map

**Mapped:** 2026-06-14
**Files analyzed:** 8 modified files across ferro-ai and ferro-mcp-server
**Analogs found:** 8 / 8

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-ai/Cargo.toml` | config | — | `ferro-ai/Cargo.toml` `[features]` pgvector block (in-file); `ferro-storage/Cargo.toml` `[features]` s3 block | exact (same file, same feature-gating pattern) |
| `ferro-ai/src/lib.rs` | config | — | `ferro-ai/src/lib.rs` `#[cfg(feature = "pgvector")]` blocks (in-file) | exact |
| `ferro-ai/src/client/*.rs`, `src/classifier/*.rs`, etc. | service | request-response | `ferro-ai/src/lib.rs` + `ferro-storage` feature-gated modules | role-match |
| `ferro-mcp-server/Cargo.toml` | config | — | `ferro-storage/Cargo.toml` optional dep behind feature | role-match |
| `ferro-mcp-server/src/config.rs` | config | — | existing `McpServerConfig` env-var field pattern (in-file) | exact |
| `ferro-mcp-server/src/error.rs` | utility | — | existing `GuardFailed`/`ActionNotFound` variants (in-file) | exact |
| `ferro-mcp-server/src/write_dispatch.rs` | service | request-response | existing `dispatch_write` + `handle_write_call` + `write_tool_error_result` (in-file) | exact |
| `ferro-mcp-server/src/renderer.rs` | utility | request-response | existing `render_action_tool` + `render_exposed_tools` + `disambiguate_write_tool_collisions` (in-file) | exact |

---

## Pattern Assignments

### `ferro-ai/Cargo.toml` — optional-dep feature gates

**Analog:** `ferro-ai/Cargo.toml` lines 32-46 (pgvector optional deps, already in file) + `ferro-storage/Cargo.toml` lines 23-29 (s3 optional deps)

**Existing optional-dep pattern** (`ferro-ai/Cargo.toml` lines 32-33, 40-46):
```toml
pgvector = { version = "0.4", features = ["sqlx"], optional = true }
sqlx     = { version = "0.8", features = ["postgres", "runtime-tokio"], optional = true }

[features]
pgvector       = ["dep:pgvector", "dep:sqlx"]
postgres-tests = ["pgvector"]
```

**Existing optional-dep pattern** (`ferro-storage/Cargo.toml` lines 23-29):
```toml
aws-sdk-s3              = { version = "1", optional = true }
aws-config              = { version = "1", optional = true }
aws-credential-types    = { version = "1", features = ["hardcoded-credentials"], optional = true }

[features]
default = []
s3 = ["aws-sdk-s3", "aws-config", "aws-credential-types"]
s3-tests = ["s3"]
cdn-bunny = []
cdn-cloudflare = []
```

**Copy pattern for Phase 220** — make the LLM-only deps optional and add `llm`/`confirmation` features:
```toml
reqwest           = { version = "0.12", features = ["json", "stream"], optional = true }
reqwest-eventsource = { version = "0.6", default-features = false, optional = true }
futures           = { version = "0.3", default-features = false, features = ["std"], optional = true }
async-stream      = { version = "0.3", optional = true }
schemars          = { version = "1", features = ["derive"], optional = true }

[features]
default = ["llm"]
llm = ["dep:reqwest", "dep:reqwest-eventsource", "dep:futures", "dep:async-stream", "dep:schemars"]
confirmation = []    # no extra deps; dashmap/tokio/ferro-events already non-optional
pgvector       = ["dep:pgvector", "dep:sqlx"]
postgres-tests = ["pgvector"]
```

---

### `ferro-ai/src/lib.rs` — `#[cfg(feature)]` module gates

**Analog:** `ferro-ai/src/lib.rs` lines 55-56, 76-77 (existing pgvector cfg gate):
```rust
#[cfg(feature = "pgvector")]
pub mod pgvector;

#[cfg(feature = "pgvector")]
pub use pgvector::{Neighbor, PgVectorStore};
```

**Current unconditional module declarations** (`ferro-ai/src/lib.rs` lines 44-53):
```rust
pub mod classifier;
pub mod client;
pub mod complete;
pub mod config;
pub mod confirmation;
pub mod embed;
pub mod error;
pub mod schema;
pub mod similarity;
pub mod tools;
```

**Copy pattern for Phase 220** — add `#[cfg(feature = "llm")]` to the LLM-only modules; leave `confirmation` and `error` unconditional:
```rust
#[cfg(feature = "llm")]
pub mod classifier;
#[cfg(feature = "llm")]
pub mod client;
#[cfg(feature = "llm")]
pub mod complete;
#[cfg(feature = "llm")]
pub mod config;
pub mod confirmation;           // always available
#[cfg(feature = "llm")]
pub mod embed;
pub mod error;                  // always available
#[cfg(feature = "llm")]
pub mod schema;
#[cfg(feature = "llm")]
pub mod similarity;
#[cfg(feature = "llm")]
pub mod tools;

#[cfg(feature = "pgvector")]
pub mod pgvector;

// pub use re-exports — gate LLM ones behind #[cfg(feature = "llm")]:
#[cfg(feature = "llm")]
pub use classifier::anthropic::AnthropicProvider;
// ... etc

// confirmation re-exports — always available:
pub use confirmation::events::ConfirmationExpired;
pub use confirmation::store::InMemoryConfirmationStore;
pub use confirmation::{ConfirmationStore, PendingActionInfo};
pub use error::Error;
```

---

### `ferro-mcp-server/Cargo.toml` — optional ferro-ai dep behind `confirmation` feature

**Analog:** `ferro-storage/Cargo.toml` lines 23-29 (optional aws deps behind `s3` feature); `ferro-ai/Cargo.toml` lines 32-33 (optional pgvector deps)

**Copy pattern for Phase 220:**
```toml
[dependencies]
# ... existing deps (ferro-projections, ferro-mcp-oauth, ferro-audit, rmcp, etc.) ...
ferro-ai = { path = "../ferro-ai", version = "0.2", optional = true, default-features = false, features = ["confirmation"] }

[features]
confirmation = ["dep:ferro-ai"]
```

The key: `default-features = false` prevents pulling in `reqwest` (the `llm` feature is default in ferro-ai but not wanted here). `features = ["confirmation"]` activates only the store — which has zero HTTP-client deps.

---

### `ferro-mcp-server/src/config.rs` — TTL field addition

**Analog:** `ferro-mcp-server/src/config.rs` lines 9-16, 29-41 (existing env-var field pattern):
```rust
pub struct McpServerConfig {
    pub app_name: String,
    pub app_url: String,
    pub version: String,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            app_name: sanitize_identity(
                std::env::var("APP_NAME").unwrap_or_else(|_| "Ferro".to_string()),
            ),
            app_url: sanitize_identity(
                std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost".to_string()),
            ),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}
```

**Copy pattern for Phase 220** — add TTL field following the env-var + fallback convention:
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

---

### `ferro-mcp-server/src/error.rs` — `ConfirmationRequired` variant

**Analog:** `ferro-mcp-server/src/error.rs` lines 25-35 (existing Phase 219 variants):
```rust
#[derive(Error, Debug)]
pub enum Error {
    // ...
    /// The resolved action name is not found in any mcp_exposed ServiceDef.
    /// Maps to JSON-RPC -32601 (method not found) at the jsonrpc layer.
    #[error("action not found: {0}")]
    ActionNotFound(String),
    /// A precondition guard returned false or errored at execution time.
    /// Maps to a structured tool error result (isError:true), NOT a -32603.
    /// Never discloses which guard or what state it checked.
    #[error("guard failed: {0}")]
    GuardFailed(String),
    /// Input validation failed (required field missing, wrong type, etc.).
    /// Maps to a structured tool error result (isError:true).
    #[error("validation error: {0}")]
    Validation(String),
}
```

**Copy pattern for Phase 220** — add the new variant behind `#[cfg(feature = "confirmation")]`:
```rust
/// A destructive action was called without a valid confirmation token.
/// Maps to a structured tool error result (isError:true) pointing the agent
/// to `request_confirm_<action>`.
/// Feature-gated: only reachable when the `confirmation` feature is enabled.
#[cfg(feature = "confirmation")]
#[error("confirmation required for action: {0}")]
ConfirmationRequired(String),
```

The match arm in `handle_write_call` follows the same pattern as `GuardFailed` — map to `write_tool_error_result` with `isError:true`, never to a JSON-RPC `-32603`.

---

### `ferro-mcp-server/src/write_dispatch.rs` — D-08 seam + confirmation handlers + token gen

**Analog (seam):** `ferro-mcp-server/src/write_dispatch.rs` lines 281-285 (the existing seam comment):
```rust
// 3. D-08 SEAM: Phase 220 inserts confirmation gating here for destructive actions
//    (transition_trigger.is_some()). In 219: pass through directly.
//    Do NOT wire ferro-ai / ConfirmationStore here.
//    if action.transition_trigger.is_some() { /* Phase 220 will intercept */ }
let _ = &action.transition_trigger; // reference to avoid unused-field lint during seam
```

**Phase 220 seam replacement pattern:**
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

**Analog (error match arm):** `ferro-mcp-server/src/write_dispatch.rs` lines 375-407 (existing `handle_write_call` error matching):
```rust
Err(crate::Error::GuardFailed(ref msg)) => {
    // Audit the denial for forensic trail
    let record_id = args.get("id").map(|v| v.to_string()).unwrap_or_default();
    let _ = AuditEntry::record(format!("mcp.action.{}", action.name))
        .tenant(tid.to_string())
        // ...
        .write(db)
        .await;
    json!({ "result": write_tool_error_result(json!({
        "error_kind": "guard_denied",
        "message": msg
    })) })
}
Err(ref e @ crate::Error::Validation(_))
| Err(ref e @ crate::Error::ActionNotFound(_)) => {
    json!({ "result": write_tool_error_result(json!({
        "error_kind": "execution_error",
        "message": e.to_string()
    })) })
}
```

**Copy pattern for Phase 220** — add ConfirmationRequired arm:
```rust
#[cfg(feature = "confirmation")]
Err(crate::Error::ConfirmationRequired(ref action_name)) => {
    json!({ "result": write_tool_error_result(json!({
        "error_kind": "confirmation_required",
        "message": format!("use request_confirm_{action_name} first"),
        "request_tool": format!("request_confirm_{action_name}")
    })) })
}
```

**Analog (token generation):** `ferro-mcp-oauth/src/validate.rs` lines 30, 115-126 (`generate_mcp_api_key`):
```rust
const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub fn generate_mcp_api_key() -> (String, String) {
    let mut rng = rand::thread_rng();
    let random: String = (0..43)
        .map(|_| {
            let idx = rand::Rng::gen_range(&mut rng, 0..62usize);
            BASE62[idx] as char
        })
        .collect();
    let raw_key = format!("ferro_{random}");
    let key_hash = hash_mcp_api_key(&raw_key);
    (raw_key, key_hash)
}
```

**Copy pattern for Phase 220** — generate confirmation token (same BASE62/rand, different prefix):
```rust
#[cfg(feature = "confirmation")]
fn generate_confirmation_token() -> String {
    use rand::Rng;
    const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
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

Note: `rand` is already in the graph via `ferro-mcp-server` → `ferro-mcp-oauth` → `rand = "0.8"`. No new dep.

**Analog (WriteDispatcher and callback types):** `ferro-mcp-server/src/write_dispatch.rs` lines 38-73 (existing `ExecutorFn`/`GuardEvaluatorFn`/`WriteDispatcher`):
```rust
pub type ExecutorFn = Box<
    dyn Fn(
            &str,
            &Value,
            i64,
            &DatabaseConnection,
        ) -> Pin<Box<dyn Future<Output = crate::Result<Value>> + Send>>
        + Send
        + Sync,
>;

pub struct WriteDispatcher {
    pub executor: ExecutorFn,
    pub guard_evaluator: GuardEvaluatorFn,
}
```

**Copy pattern for Phase 220** — the confirmation store is passed as a parameter to `handle_request_confirm` and `handle_confirm` (not added to `WriteDispatcher`), keeping Option A from the research:
```rust
#[cfg(feature = "confirmation")]
pub async fn handle_request_confirm(
    call_params: Value,
    services: &[ServiceDef],
    db: &DatabaseConnection,
    tenant_id: Option<i64>,
    ctx: &crate::McpContext,
    dispatcher: &WriteDispatcher,
    store: &dyn ferro_ai::ConfirmationStore,
    action_name: &str,
    config: &crate::McpServerConfig,
) -> Value { ... }
```

**Analog (dispatch_write call from tests):** `ferro-mcp-server/src/write_dispatch.rs` lines 498-511 (test fixture showing dispatch_write signature and how `is_confirmed` is added):
```rust
let result = dispatch_write(&approve_action(), &json!({"id": 1}), 1, &db, &dispatcher).await;
```

Phase 220 adds `is_confirmed: bool` as a `#[cfg(feature = "confirmation")]` parameter to skip the seam in the confirmed call path.

---

### `ferro-mcp-server/src/renderer.rs` — two-tool synthesis

**Analog:** `ferro-mcp-server/src/renderer.rs` lines 63-96 (`render_exposed_tools` loop + existing write-tool call):
```rust
pub fn render_exposed_tools(
    services: &[ServiceDef],
    ctx: &McpContext,
) -> std::result::Result<Vec<Tool>, ProjError> {
    let renderer = McpRenderer;
    let mut tagged: Vec<(String, Tool)> = Vec::new();

    for service in services.iter().filter(|s| s.mcp_exposed) {
        let read_tool =
            renderer.render(service, &ferro_projections::derive_intents(service), ctx)?;
        tagged.push((service.name.clone(), read_tool));

        for action in &service.actions {
            if let Some(tool) = render_action_tool(service, action, ctx)? {
                tagged.push((service.name.clone(), tool));
            }
        }
    }

    disambiguate_write_tool_collisions(&mut tagged);
    Ok(tagged.into_iter().map(|(_, t)| t).collect())
}
```

**Analog:** `ferro-mcp-server/src/renderer.rs` lines 131-175 (`render_action_tool` showing `Tool::new` + `ToolAnnotations` + schema):
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
    let name = action.name.clone();
    let description = action
        .description
        .clone()
        .or_else(|| action.display_name.clone())
        .unwrap_or_else(|| format!("{} {}", action.name, service.name));
    let schema_value = crate::schema::build_action_input_schema(action, service)
        .map_err(|e| ProjError::Render(e.to_string()))?;
    let schema_map = match schema_value {
        serde_json::Value::Object(m) => m,
        _ => return Err(ProjError::Render("action inputSchema must be an object".into())),
    };
    let annotations = ToolAnnotations::new()
        .read_only(false)
        .destructive(action.transition_trigger.is_some());
    Ok(Some(Tool::new(name, description, Arc::new(schema_map)).annotate(annotations)))
}
```

**Copy pattern for Phase 220** — inside the `render_exposed_tools` loop, after pushing the bare write tool, add a `#[cfg(feature = "confirmation")]` block that synthesizes the two extra tools:
```rust
for action in &service.actions {
    if let Some(tool) = render_action_tool(service, action, ctx)? {
        tagged.push((service.name.clone(), tool));
    }

    // Phase 220: synthesize request_confirm_<action> + confirm_<action>
    // for destructive actions when the confirmation feature is on.
    #[cfg(feature = "confirmation")]
    if action.transition_trigger.is_some() {
        // request_confirm_<action>: same schema as the bare action tool;
        // destructiveHint=false (this step only issues a token).
        if let Some(req_tool) = render_request_confirm_tool(service, action, ctx)? {
            tagged.push((service.name.clone(), req_tool));
        }
        // confirm_<action>: schema = { confirmation_token: string } + record id;
        // destructiveHint=true (this step executes).
        if let Some(cfm_tool) = render_confirm_tool(service, action, ctx)? {
            tagged.push((service.name.clone(), cfm_tool));
        }
    }
}
```

**Analog (disambiguation):** `ferro-mcp-server/src/renderer.rs` lines 98-129 (`disambiguate_write_tool_collisions` showing the `list_` prefix skip):
```rust
fn disambiguate_write_tool_collisions(tagged: &mut [(String, Tool)]) {
    let mut name_to_services: HashMap<String, HashSet<String>> = HashMap::new();
    for (service_name, tool) in tagged.iter() {
        if !tool.name.starts_with("list_") {
            name_to_services
                .entry(tool.name.to_string())
                .or_default()
                .insert(service_name.clone());
        }
    }
    for (service_name, tool) in tagged.iter_mut() {
        if !tool.name.starts_with("list_")
            && name_to_services.get(tool.name.as_ref()).map_or(0, |s| s.len()) > 1
        {
            let new_name = format!("{}_on_{}", tool.name, service_name);
            tool.name = new_name.into();
        }
    }
}
```

**Copy pattern for Phase 220** — synthesize confirmation tools AFTER the disambiguation pass, using the post-disambiguation action name as the base. This avoids the Pitfall 3 routing break (`strip_prefix("request_confirm_")` returning `"approve_on_invoice"` instead of `"approve"`).

Architecture: run `disambiguate_write_tool_collisions` on the bare write tools, then iterate the tagged vec again to push `request_confirm_<disambiguated_name>` and `confirm_<disambiguated_name>`. These confirmation tool names inherit the already-disambiguated action name and need no second rename pass.

---

### `ferro-mcp-server/src/jsonrpc.rs` — routing for confirmation tools

**Analog:** `ferro-mcp-server/src/jsonrpc.rs` lines 63-86 (`handle_tools_call` scope check + write routing):
```rust
pub async fn handle_tools_call(
    call_params: Value,
    // ...
    dispatcher: &WriteDispatcher,
) -> Value {
    let tool_name = call_params["name"].as_str().unwrap_or("");
    let is_write_tool = !tool_name.starts_with("list_");
    let key_scope = ctx.scope.as_deref().unwrap_or("read_write");
    if is_write_tool && key_scope == "read" {
        return json!({
            "error": { "code": -32603, "message": "scope insufficient: ..." }
        });
    }

    if is_write_tool {
        return handle_write_call(call_params, services, db, tenant_id, ctx, dispatcher).await;
    }
    // ... read-tool path ...
}
```

**Copy pattern for Phase 220** — inside `handle_write_call` (before the `find_action` call), add `#[cfg(feature = "confirmation")]` prefix-match routing. Confirmation tools are write-scope (same scope gate applies):
```rust
pub async fn handle_write_call(
    call_params: Value,
    services: &[ServiceDef],
    db: &DatabaseConnection,
    tenant_id: Option<i64>,
    ctx: &crate::McpContext,
    dispatcher: &WriteDispatcher,
    // Option A: store only present when feature is on
    #[cfg(feature = "confirmation")]
    store: &dyn ferro_ai::ConfirmationStore,
    #[cfg(feature = "confirmation")]
    config: &crate::McpServerConfig,
) -> Value {
    let tool_name = call_params["name"].as_str().unwrap_or("");

    #[cfg(feature = "confirmation")]
    if let Some(action_name) = tool_name.strip_prefix("request_confirm_") {
        return handle_request_confirm(
            call_params, services, db, tenant_id, ctx, dispatcher, store, action_name, config,
        ).await;
    }
    #[cfg(feature = "confirmation")]
    if let Some(action_name) = tool_name.strip_prefix("confirm_") {
        return handle_confirm(
            call_params, services, db, tenant_id, ctx, dispatcher, store, action_name,
        ).await;
    }
    // ... existing find_action + validate + dispatch_write path ...
}
```

---

## Shared Patterns

### `write_tool_error_result` — all confirmation error outcomes use this

**Source:** `ferro-mcp-server/src/write_dispatch.rs` lines 114-125
```rust
pub fn write_tool_error_result(payload: Value) -> Value {
    let message = payload
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("error")
        .to_string();
    json!({
        "content": [{ "type": "text", "text": message }],
        "isError": true,
        "structuredContent": payload
    })
}
```

**Apply to:** ALL confirmation error outcomes — `confirmation_required`, `confirmation_expired`, `confirmation_mismatch`, `guard_denied` at confirm time. Never return bare `content[]` without `isError` and `structuredContent` (D-07).

---

### `CallToolResult::structured` — confirmation success outcomes

**Source:** `ferro-mcp-server/src/write_dispatch.rs` lines 365-374:
```rust
Ok(result) => {
    let payload = json!({
        "status": "ok",
        "action": action.name,
        "result": result
    });
    let tool_result = CallToolResult::structured(payload);
    json!({ "result": tool_result })
}
```

**Apply to:** `request_confirm_<action>` success (returns `{ confirmation_token, expires_in_seconds }`); `confirm_<action>` success (passes through `dispatch_write` result). Both wrap in `json!({ "result": CallToolResult::structured(...) })`.

---

### `ConfirmationStore` TTL test pattern — SC#3 test

**Source:** `ferro-ai/src/confirmation/store.rs` lines 224-256 (`#[tokio::test(start_paused = true)]`):
```rust
async fn yield_to_register_timer() {
    tokio::task::yield_now().await;
}

async fn yield_after_advance() {
    for _ in 0..5 {
        tokio::task::yield_now().await;
    }
}

#[tokio::test(start_paused = true)]
async fn test_entry_removed_after_ttl_expires() {
    let s = store();
    s.request_confirmation("ttl-key", payload("expire-me"), Duration::from_millis(100))
        .await
        .unwrap();
    yield_to_register_timer().await;
    assert!(s.get("ttl-key").await.unwrap().is_some());

    tokio::time::advance(Duration::from_millis(150)).await;
    yield_after_advance().await;

    assert_eq!(s.get("ttl-key").await.unwrap(), None);
}
```

**Apply to:** SC#3 test in `ferro-mcp-server/src/write_dispatch.rs` — use `#[tokio::test(start_paused = true)]` with the same `yield_to_register_timer` + `tokio::time::advance` + `yield_after_advance` protocol. The test sets up a `handle_request_confirm` call, then advances clock past TTL, then calls `handle_confirm` and asserts it returns `confirmation_expired` error (not execution).

---

### `#[cfg(feature = "confirmation")]` gate convention

**Source:** `ferro-ai/src/lib.rs` lines 55-77 (pgvector gate — the closest in-file precedent):
```rust
#[cfg(feature = "pgvector")]
pub mod pgvector;

#[cfg(feature = "pgvector")]
pub use pgvector::{Neighbor, PgVectorStore};
```

**Apply to:** every new symbol in Phase 220:
- `ferro-ai/src/lib.rs` — module declarations and re-exports for LLM-only modules
- `ferro-mcp-server/src/error.rs` — `ConfirmationRequired` variant
- `ferro-mcp-server/src/write_dispatch.rs` — D-08 seam block, `handle_request_confirm`, `handle_confirm`, token generator, `is_confirmed` param
- `ferro-mcp-server/src/renderer.rs` — confirmation-tool synthesis branches
- `ferro-mcp-server/src/jsonrpc.rs` — prefix-match routing branches

The feature-off path MUST compile with zero dead-code warnings. Use `#[cfg(not(feature = "confirmation"))]` for the fallback `let _ = &action.transition_trigger;` at the seam (maintains Phase 219 behavior with no lint).

---

### Guard re-evaluation at confirm time

**Source:** `ferro-mcp-server/src/write_dispatch.rs` lines 237-255 (guard loop in `dispatch_write`):
```rust
for guard_name in &action.preconditions {
    let passes = (dispatcher.guard_evaluator)(guard_name, tenant_id, inputs, db)
        .await
        .map_err(|e| crate::Error::GuardFailed(format!("{guard_name}: {e}")))?;
    if !passes {
        return Err(crate::Error::GuardFailed(format!(
            "precondition '{guard_name}' not met"
        )));
    }
}
```

**Apply to:** BOTH `handle_request_confirm` (fail fast — do not issue a token if the guard is already denied) AND `handle_confirm` (re-evaluate at execute time — live state may have changed between request and confirm). This is the Phase 219 fail-closed guarantee extended across the confirmation gap.

---

### `dispatch_write` invocation from `handle_confirm`

**Source:** `ferro-mcp-server/src/write_dispatch.rs` lines 287-311 (post-seam execute path):
```rust
let result = (dispatcher.executor)(&action.name, inputs, tenant_id, db).await?;

if let Some(key) = idempotency_key {
    store_idempotency(tenant_id, key, &result, db).await?;
}

AuditEntry::record(format!("mcp.action.{}", &action.name))
    .tenant(tenant_id.to_string())
    // ...
    .write(db)
    .await
    .map_err(|e| crate::Error::Database(e.to_string()))?;
```

**Apply to:** `handle_confirm` calls `dispatch_write` with `is_confirmed = true` (the `#[cfg(feature = "confirmation")]` flag that bypasses the D-08 seam). The stored inputs from `ConfirmationStore::confirm()` are the `inputs` argument — NOT the `confirm_<action>` tool call arguments (which contain only the token and record id).

---

## No Analog Found

All Phase 220 files have in-file or workspace analogs. No file is without a pattern source.

---

## Metadata

**Analog search scope:** `ferro-ai/`, `ferro-mcp-server/`, `ferro-mcp-oauth/`, `ferro-storage/`, `ferro-stripe/`
**Files read:** 14 source files
**Pattern extraction date:** 2026-06-14

---

## PATTERN MAPPING COMPLETE

**Phase:** 220 - Confirmation Gating for Destructive Actions
**Files classified:** 8
**Analogs found:** 8 / 8

### Coverage
- Files with exact analog (in-file): 6
- Files with role-match analog (sibling crate): 2
- Files with no analog: 0

### Key Patterns Identified
- Optional-dep feature gating: `dep:reqwest` behind `llm` default feature — copy `ferro-ai` pgvector block + `ferro-storage` s3 block shape
- `#[cfg(feature = "llm")]` module gates: copy `#[cfg(feature = "pgvector")]` in `ferro-ai/src/lib.rs`; one gate per `pub mod` and `pub use` for LLM-only modules
- BASE62 CSPRNG token generation: copy `generate_mcp_api_key` from `ferro-mcp-oauth/src/validate.rs` lines 115-126, change prefix from `ferro_` to `cfm_`
- D-08 seam replacement: `#[cfg(feature = "confirmation")] if action.transition_trigger.is_some() { return Err(ConfirmationRequired(...)); }` — exact location `write_dispatch.rs:281`
- Error variant: `ConfirmationRequired(String)` behind `#[cfg(feature = "confirmation")]`, matches `GuardFailed`/`ActionNotFound` shape in `error.rs`
- Two-tool synthesis: inside `render_exposed_tools` loop after `disambiguate_write_tool_collisions`, using post-disambiguated action name as base; follows `render_action_tool` shape
- TTL tests: `#[tokio::test(start_paused = true)]` + `yield_to_register_timer` + `tokio::time::advance` + `yield_after_advance` — copy from `ferro-ai/src/confirmation/store.rs:224-379`
- All result envelopes: `write_tool_error_result(json!({ "error_kind": ..., "message": ... }))` for errors; `CallToolResult::structured(json!({ ... }))` for success — no new envelope shapes

### File Created
`/Users/alberto/repositories/albertogferrario/ferro/.planning/phases/220-confirmation-gating-for-destructive-actions/220-PATTERNS.md`

### Ready for Planning
Pattern mapping complete. Planner can reference analog patterns in PLAN.md files.
