# Phase 166: Structured Outputs, Tool Calling & ServiceDef-aware Schema Normalizer - Pattern Map

**Mapped:** 2026-06-08
**Files analyzed:** 8 new/modified files
**Analogs found:** 8 / 8

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-ai/src/schema/mod.rs` | utility | transform | `ferro-json-ui/src/catalog.rs` (jsonschema usage) + research code examples | partial-match |
| `ferro-ai/src/complete.rs` | utility | request-response | `ferro-ai/src/classifier/anthropic.rs` | exact |
| `ferro-ai/src/tools/mod.rs` | service | request-response | `ferro-ai/src/confirmation/store.rs` (registry + Arc<DashMap>) | role-match |
| `ferro-ai/src/client/mod.rs` | model | request-response | self (extend existing) | self-extension |
| `ferro-ai/src/client/anthropic.rs` | service | request-response | self (extend `build_body`) | self-extension |
| `ferro-ai/src/client/openai.rs` | service | request-response | self (extend `build_body`) | self-extension |
| `ferro-ai/src/error.rs` | utility | — | self (extend existing `Error` enum) | self-extension |
| `ferro-ai/src/lib.rs` | config | — | self (extend re-exports) | self-extension |
| `ferro-ai/Cargo.toml` | config | — | self (add deps) | self-extension |

---

## Pattern Assignments

### `ferro-ai/src/schema/mod.rs` (utility, transform)

**Analog:** `ferro-json-ui/src/catalog.rs` for jsonschema API usage; RESEARCH.md §Code Examples for algorithm.

**Imports pattern** — follow the crate's `serde_json`-first import style (no `use serde_json::json!` at top-level unless needed; import `serde_json::{Map, Value}`):
```rust
use std::collections::{HashMap, HashSet};
use serde_json::{Map, Value};
```

**jsonschema 0.46 API pattern** (from `ferro-json-ui/src/catalog.rs` lines 1271–1276, 623–624):
```rust
// Compile once from a schema Value:
let validator = jsonschema::validator_for(&full_schema)
    .map_err(|e| CatalogError::BuildFailed(format!("compiling schema: {e}")))?;

// Draft 2020-12 meta-validation in tests:
use jsonschema::draft202012;
assert!(draft202012::meta::is_valid(schema));

// Validate an instance:
assert!(validator.is_valid(&instance));
// Or with error detail:
let result = compiled.validate(&instance);
assert!(result.is_err());
```

**Core normalizer entry point pattern** — pure function, `serde_json::Value` in/out (RESEARCH §Code Examples):
```rust
/// Normalize a schemars 1.x schema for Anthropic/OpenAI structured-output APIs.
///
/// Steps (order is mandatory — see Pitfall 2 in RESEARCH):
/// 1. Close projection enums in `$defs` first.
/// 2. Inline all `$ref` occurrences recursively (cycle-guarded).
/// 3. Remove `$defs` / `definitions` from root.
/// 4. Strip Anthropic-rejected keywords; add `additionalProperties: false` to objects.
pub fn for_structured_output(schema: Value) -> Value {
    // ... implementation
}
```

**Recursive Value mutation pattern** — immutable recursive descent + rebuild (Pitfall 3 in RESEARCH; avoids borrow conflicts). Build a new `Value` rather than mutating in place:
```rust
fn resolve_refs(schema: Value, defs: &Map<String, Value>, visited: &mut HashSet<String>) -> Value {
    match &schema {
        Value::Object(obj) if obj.len() == 1 && obj.contains_key("$ref") => {
            // extract name, guard cycle, recurse
        }
        Value::Object(_) => { /* rebuild map, recurse values */ }
        Value::Array(_) => { /* rebuild array, recurse elements */ }
        _ => schema,
    }
}
```

**Strip keyword pattern** — explicit allowlist (never denylist; Pitfall 1 in RESEARCH):
```rust
const STRIP_KEYWORDS: &[&str] = &[
    "$schema", "$id", "title", "examples",
    "minimum", "maximum", "multipleOf",
    "minLength", "maxLength",
    // "pattern" — strip unconditionally; see RESEARCH §Open Questions
];
```

**`additionalProperties: false` — only on objects with `properties`** (Pitfall 6 in RESEARCH):
```rust
// Only add when BOTH type=="object" AND "properties" key is present
if obj.get("type").and_then(|t| t.as_str()) == Some("object")
    && obj.contains_key("properties")
{
    obj.insert("additionalProperties".into(), Value::Bool(false));
}
```

**Unit test pattern** — follows `ferro-ai/src/error.rs` inline `#[cfg(test)]` block style (lines 66–125):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn schema_normalizer_strips_rejected_keywords() {
        let input = json!({
            "$schema": "...", "title": "Foo",
            "type": "object",
            "properties": { "x": { "type": "string", "minLength": 1 } },
            "required": ["x"]
        });
        let out = for_structured_output(input);
        assert!(out.get("$schema").is_none());
        assert!(out.get("title").is_none());
        assert_eq!(out["properties"]["x"].get("minLength"), None);
        assert_eq!(out["additionalProperties"], json!(false));
    }
}
```

---

### `ferro-ai/src/complete.rs` (utility, request-response)

**Analog:** `ferro-ai/src/classifier/anthropic.rs` (lines 38–63) — builds a `CompletionRequest`, calls `client.complete()`, parses JSON from the string response.

**Imports pattern** (mirror `classifier/anthropic.rs` lines 1–8):
```rust
use crate::client::{CompletionRequest, LlmClient, Message, Role};
use crate::error::Error;
use crate::schema;
```

**Core complete::<T>() pattern** — builds `CompletionRequest` with `schema: Some(normalized)`, calls `client.complete()`, deserializes (mirrors `classifier/anthropic.rs` lines 46–63):
```rust
/// Typed completion: generate a structured `T` from a prompt.
///
/// Internally: `schema_for::<T>()` → `schema::for_structured_output()` →
/// `CompletionRequest { schema: Some(...) }` → `client.complete()` →
/// `serde_json::from_str::<T>()`.
///
/// Callers never touch schemars or serde_json directly (SC#1).
pub async fn complete<T>(client: &dyn LlmClient, prompt: &str) -> Result<T, Error>
where
    T: schemars::JsonSchema + serde::de::DeserializeOwned,
{
    let raw_schema = schemars::schema_for!(T).to_value();
    let normalized = schema::for_structured_output(raw_schema);

    let request = CompletionRequest {
        system: None,
        messages: vec![Message { role: Role::User, content: prompt.to_string() }],
        max_tokens: 4096,
        model_override: None,
        schema: Some(normalized),
        tools: None,
        tool_choice: None,
    };

    let text = client.complete(request).await?;
    serde_json::from_str::<T>(&text).map_err(|e| Error::Deserialization(e.to_string()))
}
```

**Deserialization error pattern** — reuse `Error::Deserialization(e.to_string())` exactly as in `classifier/anthropic.rs` line 62 and `client/openai.rs` line 143:
```rust
serde_json::from_str::<T>(&text).map_err(|e| Error::Deserialization(e.to_string()))
```

**Unit test pattern** — mock `LlmClient` returning a fixed JSON string, assert `complete::<T>` returns the deserialized value (mirrors `classifier/mod.rs` `ConstProvider` pattern, lines 196–229):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde::Deserialize;
    use schemars::JsonSchema;
    use crate::client::{CompletionRequest, TokenStream};

    #[derive(Debug, Deserialize, JsonSchema)]
    struct MyOutput { value: String }

    struct ConstClient(String);

    #[async_trait]
    impl LlmClient for ConstClient {
        fn default_model(&self) -> &str { "test" }
        async fn complete(&self, _: CompletionRequest) -> Result<String, Error> {
            Ok(self.0.clone())
        }
        async fn complete_stream(&self, _: CompletionRequest) -> Result<TokenStream, Error> {
            Err(Error::Unsupported)
        }
        async fn embed(&self, _: &str) -> Result<Vec<f32>, Error> { Err(Error::Unsupported) }
    }

    #[tokio::test]
    async fn complete_returns_typed_result() {
        let client = ConstClient(r#"{"value":"hello"}"#.into());
        let result = complete::<MyOutput>(&client, "test prompt").await.unwrap();
        assert_eq!(result.value, "hello");
    }
}
```

---

### `ferro-ai/src/tools/mod.rs` (service, request-response)

**Analog:** `ferro-ai/src/confirmation/store.rs` for the registry-with-lookup pattern (HashMap + Arc); `ferro-ai/src/classifier/provider.rs` for async trait object pattern; `ferro-ai/src/classifier/mod.rs` for the iteration-with-early-return loop.

**Imports pattern** (follows confirmation/store.rs lines 1–10 and classifier/mod.rs lines 1–11):
```rust
use crate::client::{CompletionRequest, LlmClient, Message};
use crate::error::Error;
use futures::future::BoxFuture;
use std::collections::HashMap;
use tracing::{error, warn};
```

**ToolError pattern** — separate model-legible error type (NOT a variant of `Error`), plain struct:
```rust
/// Model-legible tool error. Surfaced to the LLM as a `tool_result` message,
/// never exposed to Rust callers as a panic or raw DB string (SC#6).
#[derive(Debug, Clone)]
pub struct ToolError {
    pub message: String,
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
```

**ToolDef pattern** — struct with async handler stored as boxed closure (D-11):
```rust
/// Tool definition for the `ToolRegistry`.
///
/// `parameters_schema` is already normalized via `schema::for_structured_output`.
/// The handler must own all captured state (no `&references`) to satisfy `'static`.
/// Wrap shared state in `Arc<T>`.
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters_schema: serde_json::Value,
    pub handler: Box<
        dyn Fn(serde_json::Value) -> BoxFuture<'static, Result<serde_json::Value, ToolError>>
            + Send
            + Sync,
    >,
}
```

**ToolRegistry construction pattern** — `max_iterations` required, no `Default` (D-12). Mirrors `ClassifierConfig` with required fields but NO `Default` impl:
```rust
/// Registry of named tools for the LLM dispatch loop.
///
/// `max_iterations` is required at construction — there is no zero-arg constructor
/// and no way to create an unbounded loop (SC#5). Suggested value: 10.
pub struct ToolRegistry {
    tools: HashMap<String, ToolDef>,
    max_iterations: u32,
}

impl ToolRegistry {
    pub fn new(max_iterations: u32) -> Self {
        Self { tools: HashMap::new(), max_iterations }
    }

    /// Convenience constructor with `max_iterations = 10`.
    pub fn with_default_iterations() -> Self {
        Self::new(10)
    }

    pub fn register(&mut self, tool: ToolDef) {
        self.tools.insert(tool.name.clone(), tool);
    }
}
```

**Dispatch loop pattern** — bounded iteration with warn/error at thresholds; mirrors `classifier/mod.rs` lines 112–169 (the retry loop with early return) but adapted for tool use:
```rust
pub async fn dispatch(
    &self,
    mut messages: Vec<Message>,
    client: &dyn LlmClient,
) -> Result<Vec<Message>, Error> {
    for iteration in 0..=self.max_iterations {
        if iteration == self.max_iterations {
            error!(max_iterations = self.max_iterations, "tool dispatch hit iteration limit");
            return Err(Error::ToolIterationLimit(self.max_iterations));
        }
        if iteration == 5 {
            warn!(iteration, max = self.max_iterations, "tool dispatch at iteration 5");
        }

        let request = self.build_request(messages.clone());
        let response = client.complete_with_tools(request).await?;

        match response {
            CompletionResponse::Text(text) => {
                messages.push(Message { role: Role::Assistant, content: text });
                return Ok(messages);
            }
            CompletionResponse::ToolUse(blocks) => {
                // Append assistant turn, dispatch each block, append results
                for block in &blocks {
                    let result = match self.tools.get(&block.name) {
                        None => Err(Error::ToolNotFound(block.name.clone())),
                        Some(tool) => (tool.handler)(block.input.clone())
                            .await
                            .map_err(|te| {
                                // Surface ToolError to LLM as tool_result content, not as Error
                                // Caller appends a tool_result message with te.message
                                te
                            }),
                    };
                    messages.push(result_to_message(&block.id, result));
                }
            }
        }
    }
    unreachable!()
}
```

**Unit test pattern** — mock `LlmClient` counting calls (mirrors `classifier/mod.rs` `CountingProvider`, lines 260–306):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, atomic::{AtomicU32, Ordering}};
    use async_trait::async_trait;

    struct LoopingClient { calls: Arc<AtomicU32>, stop_after: u32 }

    #[async_trait]
    impl LlmClient for LoopingClient {
        fn default_model(&self) -> &str { "test" }
        async fn complete(&self, _: CompletionRequest) -> Result<String, Error> { Err(Error::Unsupported) }
        async fn complete_stream(&self, _: CompletionRequest) -> Result<TokenStream, Error> { Err(Error::Unsupported) }
        async fn embed(&self, _: &str) -> Result<Vec<f32>, Error> { Err(Error::Unsupported) }
        async fn complete_with_tools(&self, _: CompletionRequest) -> Result<CompletionResponse, Error> {
            let n = self.calls.fetch_add(1, Ordering::SeqCst);
            if n >= self.stop_after {
                Ok(CompletionResponse::Text("done".into()))
            } else {
                Ok(CompletionResponse::ToolUse(vec![ /* fake block */ ]))
            }
        }
    }

    #[tokio::test]
    async fn tool_registry_enforces_max_iterations() {
        let registry = ToolRegistry::new(3);
        let calls = Arc::new(AtomicU32::new(0));
        let client = LoopingClient { calls, stop_after: 99 }; // never stops
        let result = registry.dispatch(vec![], &client).await;
        assert!(matches!(result, Err(Error::ToolIterationLimit(3))));
    }
}
```

---

### `ferro-ai/src/client/mod.rs` (self-extension)

**Analog:** Itself. Add new fields to `CompletionRequest` and new types/method to `LlmClient`.

**Extension to `CompletionRequest`** — add after `schema` field, keeping all existing fields unchanged (lines 51–69):
```rust
pub struct CompletionRequest {
    pub system: Option<String>,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    pub model_override: Option<String>,
    pub schema: Option<serde_json::Value>,  // existing
    // NEW in Phase 166:
    pub tools: Option<Vec<ToolRequest>>,
    pub tool_choice: Option<ToolChoice>,
}

pub struct ToolRequest {
    pub name: String,
    pub description: String,
    pub parameters_schema: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum ToolChoice { Auto, None }
```

**New `LlmClient` method** — add `complete_with_tools` as a separate method with a default impl returning `Unsupported`, so existing impls compile without changes:
```rust
// Add to the LlmClient trait after embed():
/// Run a completion that may invoke tools.
///
/// Returns `Err(Error::Unsupported)` for providers that do not implement tool calling.
async fn complete_with_tools(
    &self,
    request: CompletionRequest,
) -> Result<CompletionResponse, Error> {
    let _ = request;
    Err(Error::Unsupported)
}
```

**`CompletionResponse` type** — new enum, same file or re-exported from `tools/mod.rs`:
```rust
pub enum CompletionResponse {
    Text(String),
    ToolUse(Vec<ToolUseBlock>),
}

pub struct ToolUseBlock {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}
```

**Role extension** — add `Tool` variant to `Role` if needed for tool result messages sent back to provider:
```rust
pub enum Role {
    User,
    Assistant,
    Tool,  // tool result messages (Anthropic: role "user" with type "tool_result"; OpenAI: role "tool")
}
```

---

### `ferro-ai/src/client/anthropic.rs` (self-extension)

**Analog:** Itself. Extend `build_body` to include tools array; parse `stop_reason: "tool_use"` responses.

**Tools extension to `build_body`** — mirrors the existing `schema` field conditional at lines 86–94, same pattern:
```rust
// In build_body, after the schema block:
if let Some(tools) = &request.tools {
    let tools_json: Vec<serde_json::Value> = tools.iter().map(|t| serde_json::json!({
        "name": t.name,
        "description": t.description,
        "input_schema": t.parameters_schema,
    })).collect();
    body["tools"] = serde_json::Value::Array(tools_json);
    if let Some(choice) = &request.tool_choice {
        body["tool_choice"] = match choice {
            ToolChoice::Auto => serde_json::json!({"type": "auto"}),
            ToolChoice::None => serde_json::json!({"type": "none"}),
        };
    }
}
```

**`complete_with_tools` response parsing** — extends `complete()` (lines 120–164) to check `stop_reason`:
```rust
// After fetching json response (reuse existing HTTP pattern):
let stop_reason = json["stop_reason"].as_str().unwrap_or("");
if stop_reason == "tool_use" {
    let blocks = parse_tool_use_blocks(&json["content"]);
    return Ok(CompletionResponse::ToolUse(blocks));
}
// else: extract text as before
```

---

### `ferro-ai/src/client/openai.rs` (self-extension)

**Analog:** Itself. Extend `build_body` for OpenAI function-calling format; parse `finish_reason: "tool_calls"`.

**OpenAI tools format** — follows existing `response_format` block pattern (lines 83–95):
```rust
if let Some(tools) = &request.tools {
    let tools_json: Vec<serde_json::Value> = tools.iter().map(|t| serde_json::json!({
        "type": "function",
        "function": {
            "name": t.name,
            "description": t.description,
            "parameters": t.parameters_schema,
            "strict": true,
        }
    })).collect();
    body["tools"] = serde_json::Value::Array(tools_json);
    body["tool_choice"] = serde_json::json!("auto");
}
```

**OpenAI tool_calls response parsing** — mirrors `parse_openai_delta` style (lines 109–132); extract from `choices[0].message.tool_calls`:
```rust
pub(crate) fn parse_openai_tool_calls(json: &serde_json::Value) -> Vec<ToolUseBlock> {
    let tool_calls = json["choices"][0]["message"]["tool_calls"].as_array();
    tool_calls.map(|calls| calls.iter().filter_map(|c| {
        Some(ToolUseBlock {
            id: c["id"].as_str()?.to_string(),
            name: c["function"]["name"].as_str()?.to_string(),
            input: serde_json::from_str(c["function"]["arguments"].as_str()?).ok()?,
        })
    }).collect()).unwrap_or_default()
}
```

---

### `ferro-ai/src/error.rs` (self-extension)

**Analog:** Itself — add three variants following the exact `thiserror` pattern established at lines 1–47.

**Extension pattern** (copy the variant style from lines 6–47 exactly):
```rust
// Add after Deserialization variant, before the impl block:

/// Schema normalization failed (malformed schemars output or unexpected structure).
#[error("schema normalization error: {0}")]
SchemaError(String),

/// Tool dispatch loop exceeded the configured `max_iterations` without finishing.
#[error("tool dispatch exceeded max_iterations ({0})")]
ToolIterationLimit(u32),

/// A tool name referenced in a provider response is not registered.
#[error("tool not found: {0}")]
ToolNotFound(String),
```

**`is_retryable` extension** — `ToolIterationLimit` and `ToolNotFound` are permanent errors (non-retryable); no change to existing match arms needed since the wildcard `_ => false` covers them.

---

### `ferro-ai/src/lib.rs` (self-extension)

**Analog:** Itself — add module declarations and re-exports following the existing style (lines 44–58).

**Extension pattern** (mirror lines 44–58 exactly):
```rust
// New module declarations (add after existing pub mod lines):
pub mod complete;
pub mod schema;
pub mod tools;

// New re-exports (add after existing pub use lines):
pub use complete::{complete, complete_into};
pub use schema::for_structured_output;
pub use tools::{ToolDef, ToolError, ToolRegistry};
pub use client::{CompletionRequest, CompletionResponse, ToolChoice, ToolRequest, ToolUseBlock};
```

---

### `ferro-ai/Cargo.toml` (self-extension)

**Analog:** Itself — add deps following the existing style (lines 12–29).

**New dependency entries** (add under `[dependencies]`):
```toml
schemars = { version = "1", features = ["derive"] }
ferro-projections = { path = "../ferro-projections", version = "0.2" }
```

**New dev-dependency entry** (add under `[dev-dependencies]`):
```toml
jsonschema = { version = "0.46", default-features = false }
```

**Publish wave note:** `ferro-ai` is currently in WAVE1B alongside `ferro-projections`. Since `ferro-ai` now gains a dependency ON `ferro-projections`, `ferro-projections` must be in an earlier wave OR the same wave with ferro-projections listed first. Verify `.github/workflows/publish.yml` wave ordering before merging.

---

## Shared Patterns

### async_trait usage
**Source:** `ferro-ai/src/client/mod.rs` lines 78–100, `ferro-ai/src/confirmation/mod.rs` lines 41–75
**Apply to:** `LlmClient` trait extension (`complete_with_tools`), `ToolRegistry::dispatch` (uses `client: &dyn LlmClient`)
```rust
use async_trait::async_trait;

#[async_trait]
pub trait LlmClient: Send + Sync {
    // All async fn methods require the macro to be object-safe
}
```

### Error construction — `Error::Deserialization`
**Source:** `ferro-ai/src/client/openai.rs` line 143, `ferro-ai/src/classifier/anthropic.rs` line 62
**Apply to:** `complete.rs` deserialization site, any JSON parse failure in tools dispatch
```rust
serde_json::from_str(&text).map_err(|e| Error::Deserialization(e.to_string()))
resp.json().await.map_err(|e| Error::Deserialization(e.to_string()))
```

### Error construction — `Error::Provider`
**Source:** `ferro-ai/src/client/anthropic.rs` lines 137–150
**Apply to:** All new HTTP response error sites in `complete_with_tools` implementations
```rust
let status = resp.status().as_u16();
if !resp.status().is_success() {
    let text = resp.text().await.unwrap_or_default();
    return Err(Error::Provider { status: Some(status), message: text });
}
```

### `tracing` for structured logging
**Source:** `ferro-ai/src/classifier/mod.rs` lines 120–125, 154–155
**Apply to:** `ToolRegistry::dispatch` — warn at iteration 5, error at cap
```rust
use tracing::{error, info, warn};

// In dispatch loop:
tracing::warn!(iteration, max = self.max_iterations, "tool dispatch at iteration 5");
tracing::error!(max_iterations = self.max_iterations, "tool dispatch hit iteration limit");
```

### Mock `LlmClient` in unit tests
**Source:** `ferro-ai/src/classifier/mod.rs` lines 196–213, 260–281
**Apply to:** `complete.rs` tests, `tools/mod.rs` tests — build a `struct ConstClient(String)` or `CountingClient` that implements `LlmClient` via `async_trait`
```rust
struct ConstClient(String);

#[async_trait]
impl LlmClient for ConstClient {
    fn default_model(&self) -> &str { "test" }
    async fn complete(&self, _: CompletionRequest) -> Result<String, Error> { Ok(self.0.clone()) }
    async fn complete_stream(&self, _: CompletionRequest) -> Result<TokenStream, Error> { Err(Error::Unsupported) }
    async fn embed(&self, _: &str) -> Result<Vec<f32>, Error> { Err(Error::Unsupported) }
    async fn complete_with_tools(&self, _: CompletionRequest) -> Result<CompletionResponse, Error> { Err(Error::Unsupported) }
}
```

### `BoxFuture` for async closures
**Source:** `futures` crate (already a dependency per `ferro-ai/Cargo.toml` line 15)
**Apply to:** `ToolDef.handler` field type, `make_handler` helper
```rust
use futures::future::BoxFuture;

// Handler type:
Box<dyn Fn(serde_json::Value) -> BoxFuture<'static, Result<serde_json::Value, ToolError>> + Send + Sync>

// Registering a handler:
registry.register(ToolDef {
    name: "my_tool".into(),
    description: "...".into(),
    parameters_schema: normalized_schema,
    handler: Box::new(|input: serde_json::Value| -> BoxFuture<'static, _> {
        Box::pin(async move {
            // All captures must be owned or Arc-wrapped — no &references ('static bound)
            Ok(serde_json::json!({"result": "done"}))
        })
    }),
});
```

### SC#3 structural-guarantee test — `jsonschema::draft202012`
**Source:** `ferro-json-ui/src/catalog.rs` lines 1271–1276
**Apply to:** `ferro-ai/tests/projection_schema.rs` or inline `#[cfg(test)]` in `schema/mod.rs`
```rust
#[test]
fn servicedef_schema_rejects_invalid_field_meaning() {
    use ferro_projections::ServiceDef;
    let raw = schemars::schema_for!(ServiceDef).to_value();
    let normalized = ferro_ai::schema::for_structured_output(raw);
    let validator = jsonschema::draft202012::new(&normalized).unwrap();

    let invalid = serde_json::json!({
        "name": "order",
        "fields": [{"name": "total", "data_type": "float", "meaning": "totally_bogus"}]
    });
    assert!(validator.validate(&invalid).is_err(), "invalid FieldMeaning must fail");

    let valid = serde_json::json!({
        "name": "order",
        "fields": [{"name": "total", "data_type": "float", "meaning": "money"}]
    });
    assert!(validator.validate(&valid).is_ok(), "valid FieldMeaning must pass");
}
```

### Wave 0 structural probe test (resolve assumption A1/A2)
**Apply to:** `ferro-ai/src/schema/mod.rs` `#[cfg(test)]`
```rust
#[test]
fn field_meaning_schema_has_expected_any_of_shape() {
    use ferro_projections::FieldMeaning;
    let schema = schemars::schema_for!(FieldMeaning).to_value();
    // The schema should have anyOf with at least 2 branches
    let any_of = schema["anyOf"].as_array().expect("FieldMeaning schema must have anyOf");
    assert!(any_of.len() >= 2, "expected at least 2 anyOf branches");
    // First branch must be the closed enum
    let first = &any_of[0];
    assert_eq!(first["type"], "string");
    assert!(first["enum"].is_array(), "first anyOf branch must have enum");
    // Second branch must be the open string (no enum constraint)
    let second = &any_of[1];
    assert_eq!(second["type"], "string");
    assert!(second.get("enum").is_none(), "second branch must be open string");
}
```

---

## No Analog Found

All files have analogs in the workspace. However, the **schema normalizer algorithm** (`schema/mod.rs` core logic) has no pre-existing Rust implementation in the workspace — the RESEARCH.md code examples are the closest reference. The `jsonschema` API for validation is confirmed via `ferro-json-ui/src/catalog.rs`.

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| (none) | — | — | All files have workspace analogs or are self-extensions |

---

## Metadata

**Analog search scope:** `ferro-ai/src/**`, `ferro-json-ui/src/catalog.rs`, `ferro-projections/src/field.rs`, `ferro-projections/src/intent.rs`
**Files scanned:** 13 ferro-ai source files + catalog.rs + field.rs + intent.rs + Cargo.toml files
**Pattern extraction date:** 2026-06-08

**Key conventions confirmed from source:**
- `thiserror` derive on `Error` enum, one per crate — `error.rs` lines 2–3
- `async_trait` on all async trait definitions — `client/mod.rs` line 78
- `serde_json::Value` as the universal JSON type (not `schemars::Schema`) — all client files
- `Box::pin(stream::unfold(...))` for streams — `anthropic.rs` line 213
- Inline `#[cfg(test)] mod tests` at end of each file — all source files
- `use crate::error::Error` import style (crate-relative) — all source files
- No `unwrap()` in non-test production code — all errors propagated via `?` or `.map_err()`
- `pub(crate)` for internal helpers (`build_body`, `parse_anthropic_delta`) — `anthropic.rs` line 50, 104
