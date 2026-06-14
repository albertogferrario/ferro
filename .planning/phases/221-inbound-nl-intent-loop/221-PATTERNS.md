# Phase 221: Inbound NL Intent Loop - Pattern Map

**Mapped:** 2026-06-14
**Files analyzed:** 6 new/modified files
**Analogs found:** 6 / 6

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-mcp-server/src/intent.rs` | service/module (turn core + `ToolSelection` + `render_tool_descriptions` + `ReplayClassificationProvider`) | request-response + event-driven | `ferro-mcp-server/src/write_dispatch.rs` (pipeline core), `ferro-mcp-server/src/renderer.rs` (`render_exposed_tools`), `ferro-ai/src/classifier/mod.rs` (`ConstProvider` pattern) | exact role-match across three composites |
| `ferro-mcp-server/tests/intent_loop/` (test + fixtures) | test (replay + live-gated) | request-response | `ferro-mcp/tests/agent_harness.rs` + `ferro-mcp/tests/fixtures/agent_harness/` | exact |
| `ferro-mcp-server/Cargo.toml` (add `ai`/`ai-live` features) | config | — | `ferro-mcp-server/Cargo.toml` existing `confirmation` feature wiring | exact |
| `ferro-ai/Cargo.toml` (no change needed — verified) | config | — | `ferro-ai/Cargo.toml` existing `llm`/`confirmation` split | exact — no modification required |
| `app/src/controllers/mcp_chat.rs` | controller (HTTP endpoint) | request-response | `app/src/controllers/mcp.rs` (`handle` + `exposed_services` + `make_write_dispatcher` + `confirmation_store`) | exact |

---

## Pattern Assignments

### `ferro-mcp-server/src/intent.rs` (new — turn core + ToolSelection + render_tool_descriptions)

**Three composites feed this file:**

**Composite 1 — ToolSelection type**
Analog: `ferro-ai/src/classifier/mod.rs`, in particular the `ClassificationResult<T>` struct (lines 46-58) and the test fixture `SampleOutput` struct (lines 191-194).
Rule: define `ToolSelection` in `ferro-mcp-server`, not in `ferro-ai` (D-01). Derive `Serialize + Deserialize` with `#[serde(rename_all = "snake_case")]` to match the JSON schema field names (Pitfall 5).

**ToolSelection type pattern** (modeled on `ClassificationResult<T>` lines 46-58 and `SampleOutput` lines 191-194 of `ferro-ai/src/classifier/mod.rs`):
```rust
use serde::{Deserialize, Serialize};
use serde_json::Map;

/// Classifier output for a single conversational turn.
/// Defined here (not in ferro-ai) because it is projection-specific (D-01).
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ToolSelection {
    pub tool_name: String,
    pub arguments: Map<String, serde_json::Value>,
    pub confidence: f64,
}
```

**Composite 2 — `render_tool_descriptions` helper**
Analog: `ferro-mcp-server/src/renderer.rs`, `render_exposed_tools` (lines 69-138).
Rule: call `render_exposed_tools(services, ctx)` and format the returned `Vec<Tool>` as text. Do NOT re-derive tool names/descriptions from `ServiceDef` independently (Pitfall 4 / PITFALLS §11).

**`render_tool_descriptions` pattern** (modeled on `render_exposed_tools` call site in renderer.rs lines 69-72):
```rust
// ferro-mcp-server/src/intent.rs — render_tool_descriptions
// Formats the guard-filtered tool list as a classifier system prompt.
// Calls render_exposed_tools (not a second projection renderer).
pub fn render_tool_descriptions(
    services: &[ServiceDef],
    ctx: &McpContext,
) -> Result<String, ferro_projections::Error> {
    let tools = render_exposed_tools(services, ctx)?;
    // Include name + description + input property names (not full types).
    // Concise enough for a system prompt; no type annotations to stay within context.
    let lines: Vec<String> = tools
        .iter()
        .map(|t| {
            let props: Vec<&str> = t
                .input_schema
                .get("properties")
                .and_then(|v| v.as_object())
                .map(|m| m.keys().map(|k| k.as_str()).collect())
                .unwrap_or_default();
            format!(
                "- {}: {} [args: {}]",
                t.name,
                t.description.as_deref().unwrap_or(""),
                props.join(", ")
            )
        })
        .collect();
    Ok(lines.join("\n"))
}
```

**Composite 3 — `process_nl_turn` core function**
Analog: `ferro-mcp-server/src/write_dispatch.rs`, `handle_write_call` (lines 362-412) and `dispatch_write` (lines 258-351) for the pipeline structure; `ferro-mcp-server/src/jsonrpc.rs`, `handle_tools_call` (lines 59-100) for the read/write routing branch.

**`process_nl_turn` signature pattern** (modeled on `handle_write_call` lines 362-372 of `write_dispatch.rs`):
```rust
// ferro-mcp-server/src/intent.rs (gated by #[cfg(feature = "ai")])
#[cfg(feature = "ai")]
pub async fn process_nl_turn(
    nl_message: &str,
    services: &[ServiceDef],
    db: &DatabaseConnection,
    tenant_id: Option<i64>,
    ctx: &McpContext,
    provider: Arc<dyn ferro_ai::ClassificationProvider>,
    classifier_config: ferro_ai::ClassifierConfig,
    dispatcher: &WriteDispatcher,
    #[cfg(feature = "confirmation")] store: &dyn ferro_ai::ConfirmationStore,
    #[cfg(feature = "confirmation")] config: &McpServerConfig,
) -> Value { ... }
```

**Read/write routing branch** (copy from `handle_tools_call` lines 69-100 of `jsonrpc.rs`):
```rust
// The existing routing idiom — mirror in process_nl_turn:
let is_write_tool = !tool_name.starts_with("list_");
if is_write_tool {
    // route to handle_write_call
} else {
    // route to handle_tools_call (read path)
}
```

**Low-confidence mapping to clarification** (modeled on `write_tool_error_result` pattern in `write_dispatch.rs` lines 113-125 and RESEARCH.md Pattern 2):
```rust
// Map Error::LowConfidence to a structured clarification response (D-03).
// Uses CallToolResult::structured — same envelope as all other turn outcomes (D-07).
Err(ferro_ai::Error::LowConfidence { best_guess, confidence }) => {
    let question = format!(
        "I'm not sure what you mean (confidence {:.0}%). Did you mean to {}? \
         Or could you be more specific?",
        confidence * 100.0,
        best_guess.get("tool_name").and_then(|v| v.as_str()).unwrap_or("do something")
    );
    // write_tool_error_result shape: { content:[{type,text}], isError, structuredContent }
    // For clarification: isError is FALSE (not an error, just ambiguity).
    serde_json::json!({
        "result": {
            "content": [{ "type": "text", "text": question }],
            "isError": false,
            "structuredContent": {
                "status": "needs_clarification",
                "question": question,
                "best_guess": best_guess
            }
        }
    })
}
```

---

### `ferro-mcp-server/tests/intent_loop/` (new — replay test + fixtures)

**Analog:** `ferro-mcp/tests/agent_harness.rs` + `ferro-mcp/tests/fixtures/agent_harness/`

**Transcript struct pattern** (copy from `agent_harness.rs` lines 110-140 — `TrialRecord` + `Transcript`):
```rust
// ferro-mcp-server/tests/intent_loop/mod.rs (or replay_test.rs)
use serde::{Deserialize, Serialize};

/// A single recorded NL turn for replay.
#[derive(Debug, Deserialize, Serialize)]
pub struct IntentTurnFixture {
    pub turn_id: String,
    pub nl_message: String,
    pub expected_tool: String,   // expected tool_name in the recorded_selection
    pub recorded_selection: serde_json::Value, // full ToolSelection JSON
}
```

**Fixture file shape** (modeled on `_fixture_valid.json` — simpler, single-turn):
```json
{
  "turn_id": "approve-order",
  "nl_message": "approve the order from Alice",
  "expected_tool": "approve",
  "recorded_selection": {
    "tool_name": "approve",
    "arguments": { "id": 42 },
    "confidence": 0.92
  }
}
```
Fixture files: `ferro-mcp-server/tests/fixtures/intent_loop/transcripts/<turn-id>.json`

**`ReplayClassificationProvider` pattern** (modeled on `ConstProvider` / `EchoProvider` in `ferro-ai/src/classifier/mod.rs` lines 196-211 and `provider.rs` lines 53-83):
```rust
// In #[cfg(test)] block or tests/intent_loop/mod.rs
use async_trait::async_trait;
use ferro_ai::{ClassificationProvider, ClassifierConfig};

pub struct ReplayClassificationProvider {
    /// Keyed by NL message string → recorded ToolSelection JSON.
    recordings: std::collections::HashMap<String, serde_json::Value>,
}

impl ReplayClassificationProvider {
    pub fn from_fixtures(fixtures: &[IntentTurnFixture]) -> Self {
        let recordings = fixtures
            .iter()
            .map(|f| (f.nl_message.clone(), f.recorded_selection.clone()))
            .collect();
        Self { recordings }
    }
}

#[async_trait]
impl ClassificationProvider for ReplayClassificationProvider {
    async fn classify_raw(
        &self,
        _system_prompt: &str,
        user_prompt: &str,
        _schema: &serde_json::Value,
        _config: &ClassifierConfig,
    ) -> Result<serde_json::Value, ferro_ai::Error> {
        self.recordings
            .get(user_prompt)
            .cloned()
            .ok_or_else(|| ferro_ai::Error::Provider {
                status: None,
                message: format!("no replay fixture for: {user_prompt}"),
            })
    }
}
```

**Deterministic replay test pattern** (copy from `agent_harness.rs` lines 636-683):
```rust
// Non-ignored: runs in default `cargo test` with no network.
#[tokio::test]
async fn intent_loop_replay_is_deterministic() {
    let raw = include_str!("fixtures/intent_loop/transcripts/approve-order.json");
    let fixture: IntentTurnFixture = serde_json::from_str(raw).expect("must parse");

    let provider = ReplayClassificationProvider::from_fixtures(&[fixture]);
    let classifier = ferro_ai::Classifier::<ToolSelection>::new(
        Arc::new(provider),
        ferro_ai::ClassifierConfig { confidence_threshold: 0.0, ..Default::default() },
    );
    // ... call process_nl_turn with classifier ...
    // assert result matches fixture.expected_tool
}
```

**Live-eval gate pattern** (copy from `agent_harness.rs` lines 890-893):
```rust
// Gated: skipped by default cargo test / CI.
#[tokio::test]
#[ignore]
async fn intent_loop_live_eval() {
    if std::env::var("FERRO_AI_LIVE_EVAL").as_deref() != Ok("1") {
        return;
    }
    // Announce cost BEFORE first API call (isolate-before-spend discipline).
    eprintln!(
        "FERRO_AI_LIVE_EVAL=1: running live classification \
         (~{N} turns × ~$0.005/call ≈ $X.XX)"
    );
    // ... AnthropicProvider::from_env(), live classify, assert vs fixture ...
}
```

---

### `ferro-mcp-server/Cargo.toml` (modify — add `ai` and `ai-live` features)

**Analog:** `ferro-mcp-server/Cargo.toml` existing `confirmation` feature (lines 30-33) and the optional `ferro-ai` dep declaration (line 20).

**Existing pattern** (lines 19-21 and 30-33 of `ferro-mcp-server/Cargo.toml`):
```toml
# Existing — do not change:
ferro-ai = { path = "../ferro-ai", version = "0.2", optional = true, default-features = false, features = ["confirmation"] }
rand = { version = "0.8", optional = true }

[features]
confirmation = ["dep:ferro-ai", "dep:rand"]
```

**Phase 221 extension** (add these two lines after `confirmation`):
```toml
[features]
confirmation = ["dep:ferro-ai", "dep:rand"]   # existing — unchanged
ai = ["dep:ferro-ai"]                         # NEW: replay-only loop path (no reqwest)
ai-live = ["ai", "ferro-ai/llm"]             # NEW: live provider (adds reqwest transitively)
```

The `ferro-ai` dep line is UNCHANGED — the optional dep with `features = ["confirmation"]` already applies when either `confirmation` or `ai` (or `ai-live`) activates it. The `llm` feature is only added when `ai-live` is active.

**Verification:** `ferro-ai/Cargo.toml` confirms `ClassificationProvider` requires only always-on deps (`async-trait`, `serde_json`, `tokio`) — not reqwest. The `ai` feature pulls no reqwest into ferro-mcp-server.

---

### `app/src/controllers/mcp_chat.rs` (new — thin `/mcp/chat` HTTP endpoint)

**Analog:** `app/src/controllers/mcp.rs` — the existing MCP endpoint handler.

**Imports pattern** (copy from `mcp.rs` lines 1-15):
```rust
use ferro::serde_json::{json, Value};
use ferro::ServiceDef;
use ferro::{handler, HttpResponse, Request, Response};
use ferro_mcp_server::{McpContext, McpServerConfig, WriteDispatcher};

#[cfg(feature = "confirmation")]
use std::sync::OnceLock;
```

**`OnceLock` confirmation store pattern** (copy from `mcp.rs` lines 16-28):
```rust
// Process-wide confirmation store — same pattern as in mcp.rs.
#[cfg(feature = "confirmation")]
static CONFIRMATION_STORE: OnceLock<ferro_ai::InMemoryConfirmationStore> = OnceLock::new();

#[cfg(feature = "confirmation")]
fn confirmation_store() -> &'static ferro_ai::InMemoryConfirmationStore {
    CONFIRMATION_STORE.get_or_init(ferro_ai::InMemoryConfirmationStore::new)
}
```

**`exposed_services()` pattern** (copy from `mcp.rs` lines 32-34):
```rust
fn exposed_services() -> Vec<ServiceDef> {
    vec![crate::projections::order::service_def()]
}
```

**Handler skeleton** (modeled on `mcp.rs` `handle` function lines 172-335, simplified for a single-turn NL endpoint):
```rust
/// POST /mcp/chat — single conversational turn.
/// Auth: same bearer middleware as /mcp.
/// Body: { "message": "<nl string>" }
/// Response: CallToolResult-shaped JSON with status/content/structuredContent.
#[handler]
pub async fn handle_chat(req: Request) -> Response {
    let config = McpServerConfig::from_env();

    // Origin check (mirrors mcp.rs lines 176-180).
    if let Some(origin) = req.header("Origin") {
        if !origin.starts_with(config.app_url.as_str()) {
            return Err(HttpResponse::new().status(403));
        }
    }

    // Retrieve principal (mirrors mcp.rs lines 187-198).
    let principal = req
        .get::<ferro::serde_json::Value>()
        .ok_or_else(|| HttpResponse::new().status(401))?;
    let _user_id: i64 = principal["sub"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| HttpResponse::new().status(400))?;
    let key_scope: Option<String> = principal
        .get("scope")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Parse body (mirrors mcp.rs lines 204-210 pattern).
    let body: Value = req.json().await.map_err(|e| {
        HttpResponse::json(json!({ "error": e.to_string() }))
    })?;
    let nl_message = body["message"].as_str().unwrap_or("").to_string();

    let db = ferro::DB::connection().map_err(|e| {
        HttpResponse::json(json!({ "error": e.to_string() }))
    })?;
    let tenant_id = ferro::current_tenant().map(|t| t.id);
    let ctx = McpContext {
        tenant_id,
        scope: key_scope,
        ..Default::default()
    };
    let services = exposed_services();
    let dispatcher = make_write_dispatcher(); // same factory as mcp.rs

    // Instantiate classifier provider (live or replay per feature flag / env var).
    // Wire process_nl_turn from ferro_mcp_server::intent.
    let result = ferro_mcp_server::intent::process_nl_turn(
        &nl_message,
        &services,
        db.inner(),
        tenant_id,
        &ctx,
        provider, // Arc<dyn ClassificationProvider>
        ferro_ai::ClassifierConfig::default(),
        &dispatcher,
        #[cfg(feature = "confirmation")] confirmation_store(),
        #[cfg(feature = "confirmation")] &config,
    )
    .await;

    Ok(HttpResponse::json(result))
}
```

**`make_write_dispatcher()` pattern** — copy verbatim from `mcp.rs` lines 68-134. The chat endpoint uses the same app-level dispatcher; do not duplicate the factory.

---

## Shared Patterns

### ClassificationProvider trait (the replay seam)
**Source:** `ferro-ai/src/classifier/provider.rs` lines 37-51
**Apply to:** `ReplayClassificationProvider` in test infrastructure, `AnthropicProvider` (live path behind `ai-live` feature)
```rust
#[async_trait]
pub trait ClassificationProvider: Send + Sync {
    async fn classify_raw(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        schema: &serde_json::Value,
        config: &ClassifierConfig,
    ) -> Result<serde_json::Value, Error>;
}
```
Object-safe — use `Arc<dyn ClassificationProvider>` as the seam parameter in `process_nl_turn`.

### Error envelopes (D-07 — every turn outcome)
**Source:** `ferro-mcp-server/src/write_dispatch.rs` lines 113-125
**Apply to:** all turn outcomes in `process_nl_turn` (dispatched result, needs_clarification, confirmation-required, guard-denied)
```rust
pub fn write_tool_error_result(payload: Value) -> Value {
    let message = payload.get("message").and_then(|v| v.as_str()).unwrap_or("error").to_string();
    json!({
        "content": [{ "type": "text", "text": message }],
        "isError": true,
        "structuredContent": payload
    })
}
// CallToolResult::structured from rmcp — used throughout jsonrpc.rs and write_dispatch.rs.
// For non-error turn outcomes (clarification, read result), use isError: false with structuredContent.
```

### Write dispatch pipeline (untrusted-args guarantee)
**Source:** `ferro-mcp-server/src/write_dispatch.rs` lines 258-351 (`dispatch_write`)
**Apply to:** every write intent dispatched from `process_nl_turn`
The full pipeline — guard re-eval (live DB, not `ctx.evaluated_guards`), idempotency, D-08 confirmation seam, executor, audit — runs unchanged inside `handle_write_call`. The turn core calls `handle_write_call` as-is; it adds zero new dispatch logic (D-02/SC#1).

### Feature gate pattern (`#[cfg(feature = "...")]` parameters)
**Source:** `ferro-mcp-server/src/write_dispatch.rs` lines 362-372 and `ferro-mcp-server/src/jsonrpc.rs` lines 59-68
**Apply to:** `process_nl_turn` signature, any module-level item in `intent.rs`
```rust
// Conditional parameter pattern — copy exactly from handle_write_call:
pub async fn handle_write_call(
    call_params: Value,
    services: &[ServiceDef],
    db: &DatabaseConnection,
    tenant_id: Option<i64>,
    ctx: &crate::McpContext,
    dispatcher: &WriteDispatcher,
    #[cfg(feature = "confirmation")] store: &dyn ferro_ai::ConfirmationStore,
    #[cfg(feature = "confirmation")] config: &crate::McpServerConfig,
) -> Value
```

### ConstProvider / EchoProvider test pattern
**Source:** `ferro-ai/src/classifier/mod.rs` lines 196-211 and `ferro-ai/src/classifier/provider.rs` lines 53-83
**Apply to:** `ReplayClassificationProvider` in `ferro-mcp-server/tests/`
```rust
// ConstProvider (from classifier/mod.rs lines 196-211) is the direct template:
struct ConstProvider { response: serde_json::Value }

#[async_trait]
impl ClassificationProvider for ConstProvider {
    async fn classify_raw(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
        _schema: &serde_json::Value,
        _config: &ClassifierConfig,
    ) -> Result<serde_json::Value, Error> {
        Ok(self.response.clone())
    }
}
// ReplayClassificationProvider replaces the fixed response with a HashMap keyed on user_prompt.
```

### `#[ignore]` + env-var gate for live tests
**Source:** `ferro-mcp/tests/agent_harness.rs` lines 890-896
**Apply to:** `intent_loop_live_eval` test in `ferro-mcp-server/tests/`
```rust
#[tokio::test]
#[ignore]
async fn intent_loop_live_eval() {
    if std::env::var("FERRO_AI_LIVE_EVAL").as_deref() != Ok("1") {
        return;
    }
    eprintln!("FERRO_AI_LIVE_EVAL=1: running live classification (~N calls × ~$X/call ≈ $X.XX)");
    // ... live path ...
}
```

---

## No Analog Found

None. All Phase 221 files have direct analogs in the codebase.

---

## Metadata

**Analog search scope:** `ferro-mcp-server/src/`, `ferro-ai/src/classifier/`, `ferro-mcp/tests/`, `app/src/controllers/`, `ferro-mcp-server/Cargo.toml`, `ferro-ai/Cargo.toml`
**Files scanned:** 11 source files read directly
**Pattern extraction date:** 2026-06-14
