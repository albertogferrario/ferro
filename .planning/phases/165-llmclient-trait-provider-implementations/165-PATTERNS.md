# Phase 165: LlmClient Trait & Provider Implementations - Pattern Map

**Mapped:** 2026-06-08
**Files analyzed:** 7 new/modified files
**Analogs found:** 7 / 7

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-ai/src/client/mod.rs` | trait-def + type-alias | request-response | `ferro-ai/src/classifier/provider.rs` | exact (async_trait object-safe trait) |
| `ferro-ai/src/client/anthropic.rs` | service / HTTP client | request-response + streaming | `ferro-ai/src/classifier/anthropic.rs` | exact (same provider, same HTTP shape) |
| `ferro-ai/src/client/openai.rs` | service / HTTP client | request-response + streaming | `ferro-ai/src/classifier/anthropic.rs` | role-match (same pattern, different provider) |
| `ferro-ai/src/client/ollama.rs` | service / HTTP client | request-response + streaming (NDJSON) | `ferro-ai/src/classifier/anthropic.rs` | role-match (same pattern, no-auth variant) |
| `ferro-ai/src/config.rs` | config / factory | request-response | `ferro-whatsapp/src/config.rs` + `ferro-storage/src/facade.rs` | role-match (from_env + driver dispatch) |
| `ferro-ai/src/error.rs` | error enum | — | `ferro-ai/src/error.rs` (self, extended) | exact (thiserror, same crate) |
| `ferro-ai/src/classifier/mod.rs` | service / retry | request-response | `ferro-ai/src/classifier/mod.rs` (self, modified) | exact (same file, targeted changes) |
| `ferro-ai/src/classifier/anthropic.rs` | adapter | request-response | `ferro-ai/src/classifier/anthropic.rs` (self, rewired) | exact (same file, thin-delegate refactor) |
| `ferro-ai/src/lib.rs` | re-export module | — | `ferro-ai/src/lib.rs` (self, extended) | exact |
| `ferro-ai/Cargo.toml` | manifest | — | `ferro-ai/Cargo.toml` (self, extended) | exact |

> Note: `config.rs` is a new file; the module layout in RESEARCH.md places `AiConfig` there.
> `ferro-ai/src/classifier/anthropic.rs` and `ferro-ai/src/classifier/mod.rs` are modifications to existing files.

---

## Pattern Assignments

### `ferro-ai/src/client/mod.rs` (trait-def, request-response)

**Analog:** `ferro-ai/src/classifier/provider.rs`

**Imports pattern** (lines 1–4 of provider.rs):
```rust
use crate::error::Error;
use async_trait::async_trait;

use super::ClassifierConfig;
```
Mirror for client/mod.rs:
```rust
use crate::error::Error;
use async_trait::async_trait;
use futures::stream::BoxStream;
```

**Object-safe async_trait pattern** (lines 37–51 of provider.rs):
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
The `LlmClient` trait follows this exact shape — `#[async_trait]`, `: Send + Sync`, `&self` receiver, `Result<_, Error>` return.

**Object-safety test pattern** (lines 75–82 of provider.rs):
```rust
#[test]
fn test_classification_provider_is_object_safe() {
    let provider = EchoProvider { response: serde_json::json!({"result": "ok"}) };
    // This must compile — verifies object safety
    let _: Arc<dyn ClassificationProvider> = Arc::new(provider);
}
```
Add an equivalent test: `let _: Box<dyn LlmClient> = Box::new(AnthropicClient::new(...));`

**TokenStream type alias** — net-new to workspace; no codebase analog exists. Follow RESEARCH.md Pattern 1:
```rust
/// Opaque stream of text tokens from a streaming LLM completion.
pub type TokenStream = BoxStream<'static, Result<String, Error>>;
```

---

### `ferro-ai/src/client/anthropic.rs` (service, request-response + streaming)

**Analog:** `ferro-ai/src/classifier/anthropic.rs`

**Struct + constructor pattern** (lines 15–37 of anthropic.rs):
```rust
pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("failed to build reqwest client");
        Self { client, api_key }
    }

    pub fn from_env() -> Result<Self, Error> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| Error::Config("ANTHROPIC_API_KEY not set".to_string()))?;
        Ok(Self::new(api_key))
    }
}
```
`AnthropicClient` copies this exactly. Rename struct, change env-var to `FERRO_AI_API_KEY` (with `ANTHROPIC_API_KEY` as fallback per D-06).

**Anthropic HTTP request pattern** (lines 94–109 of anthropic.rs — the core reqwest call):
```rust
let response = self
    .client
    .post("https://api.anthropic.com/v1/messages")
    .header("x-api-key", &self.api_key)
    .header("anthropic-version", "2023-06-01")
    .header("content-type", "application/json")
    .json(&body)
    .send()
    .await
    .map_err(|e| {
        if e.is_timeout() {
            Error::Timeout
        } else {
            Error::Provider(format!("request failed: {e}"))
        }
    })?;
```
After D-14, the error arm becomes `Error::Provider { status: None, message: e.to_string() }`.

**Status check + response parse pattern** (lines 111–142 of anthropic.rs):
```rust
let status = response.status().as_u16();

if is_permanent_error(status) {
    let text = response.text().await.unwrap_or_default();
    return Err(Error::Provider(format!("{status} {text}")));
}
// ... (collapse into single !is_success() check in AnthropicClient)

let json: serde_json::Value = response.json().await
    .map_err(|e| Error::Deserialization(e.to_string()))?;

let text = json["content"]
    .as_array()
    .and_then(|arr| arr.first())
    .and_then(|item| item["text"].as_str())
    .ok_or_else(|| Error::Deserialization(format!("unexpected response structure: {json}")))?;
```
After D-14, status check becomes: `Err(Error::Provider { status: Some(status), message: text })`.

**Request body builder pattern** (lines 44–66 of anthropic.rs):
```rust
pub(crate) fn build_request_body(
    system_prompt: &str,
    user_prompt: &str,
    schema: &serde_json::Value,
    config: &ClassifierConfig,
) -> serde_json::Value {
    serde_json::json!({
        "model": config.model,
        "max_tokens": config.max_tokens,
        "system": [{"type": "text", "text": system_prompt, "cache_control": {"type": "ephemeral"}}],
        "messages": [{"role": "user", "content": user_prompt}],
        "output_config": {
            "format": {"type": "json_schema", "schema": schema}
        }
    })
}
```
`AnthropicClient::build_body` extends this: takes `&CompletionRequest` instead of flat params; adds `"stream": bool`; wraps `output_config` in `if let Some(schema) = &request.schema`.

**Streaming** — net-new to this file; no codebase analog exists for SSE. Follow RESEARCH.md Pattern 2 (`stream::unfold` + `reqwest-eventsource`).

**Test pattern** (lines 146–228 of anthropic.rs) — unit tests for body builder, no tokio needed. For HTTP tests, mirror this shape:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_body_with_schema() {
        // verify body["output_config"]["format"]["type"] == "json_schema"
    }

    #[test]
    fn test_default_model() {
        let client = AnthropicClient::new("key".into(), None);
        assert_eq!(client.default_model(), "claude-sonnet-4-6");
    }
}
```

---

### `ferro-ai/src/client/openai.rs` (service, request-response + streaming)

**Analog:** `ferro-ai/src/classifier/anthropic.rs` (role-match — same HTTP pattern, different auth and endpoint)

**Key differences from AnthropicClient:**
- Auth header: `Authorization: Bearer {api_key}` (use `.bearer_auth()`)
- Endpoint: `{base_url}/v1/chat/completions` (base_url field on struct)
- Default base_url: `"https://api.openai.com"`
- Response extract: `json["choices"][0]["message"]["content"]` (non-stream)
- Stream sentinel: check `msg.data == "[DONE]"` before JSON parse
- `embed()` is implemented (not `Unsupported`)

**Struct pattern** — copy AnthropicProvider constructor shape, add `base_url: String` field:
```rust
pub struct OpenAiClient {
    client: reqwest::Client,
    api_key: String,
    model: Option<String>,
    base_url: String,
}
```

**reqwest call pattern** (same shape as anthropic.rs lines 94–109, but bearer auth):
```rust
let response = self.client
    .post(format!("{}/v1/chat/completions", self.base_url))
    .bearer_auth(&self.api_key)
    .json(&body)
    .send()
    .await
    .map_err(|e| Error::Provider { status: None, message: e.to_string() })?;
```

---

### `ferro-ai/src/client/ollama.rs` (service, request-response + NDJSON streaming)

**Analog:** `ferro-ai/src/classifier/anthropic.rs` (role-match — same HTTP client struct pattern, no auth)

**Key differences:**
- No `api_key` field
- `base_url` defaults to `"http://localhost:11434"`
- Endpoint: `{base_url}/api/chat` (non-stream and stream)
- Stream: `response.bytes_stream()` + manual newline splitting — NOT `reqwest-eventsource`
- `embed()` implemented: `{base_url}/api/embed`

**Struct pattern:**
```rust
pub struct OllamaClient {
    client: reqwest::Client,
    model: Option<String>,
    base_url: String,
}
```

**NDJSON stream pattern** — net-new to workspace; follow RESEARCH.md Pattern 2 (`async_stream::try_stream!`):
```rust
use async_stream::try_stream;

fn build_token_stream(response: reqwest::Response) -> TokenStream {
    Box::pin(try_stream! {
        let mut stream = response.bytes_stream();
        // line-by-line NDJSON: each line is {"message":{"content":"..."},"done":bool}
    })
}
```

---

### `ferro-ai/src/config.rs` (config / factory, request-response)

**Primary analog:** `ferro-whatsapp/src/config.rs` — for the `from_env()` → `Result<_, Error>` pattern with missing-var error at startup.

**from_env pattern** (lines 40–57 of ferro-whatsapp/src/config.rs):
```rust
pub fn from_env(is_owner: Box<dyn Fn(&str) -> bool + Send + Sync>) -> Result<Self, Error> {
    let app_secret = std::env::var("WHATSAPP_APP_SECRET")
        .map_err(|_| Error::Config("WHATSAPP_APP_SECRET not set".into()))?;
    // ...
    Ok(Self { app_secret, ... })
}
```
`AiConfig::from_env()` mirrors this `std::env::var(...).map_err(|_| Error::Config(...))?` idiom for each required var, but returns `Box<dyn LlmClient>` (not `Self`).

**Secondary analog:** `ferro-storage/src/facade.rs` — for the string-based driver dispatch (`match provider_string { "anthropic" => ..., "openai" => ..., unknown => Err(...) }`). The storage facade uses `match config.driver { DiskDriver::Local => Arc::new(LocalDriver::new(root)), ... }` (lines 200–215 of facade.rs). `AiConfig::from_env()` does the same with a string match on `FERRO_AI_PROVIDER`.

**Combined pattern for AiConfig:**
```rust
pub struct AiConfig;

impl AiConfig {
    /// Construct the configured LLM client from environment variables.
    ///
    /// Reads: FERRO_AI_PROVIDER, FERRO_AI_MODEL, FERRO_AI_API_KEY, FERRO_AI_BASE_URL.
    /// Returns Err(Error::Config) for unknown providers or missing required vars.
    pub fn from_env() -> Result<Box<dyn LlmClient>, Error> {
        let provider = std::env::var("FERRO_AI_PROVIDER")
            .unwrap_or_else(|_| "anthropic".to_string());
        let model = std::env::var("FERRO_AI_MODEL").ok();
        let api_key = std::env::var("FERRO_AI_API_KEY").ok();
        let base_url = std::env::var("FERRO_AI_BASE_URL").ok();

        match provider.to_lowercase().as_str() {
            "anthropic" => {
                let key = api_key
                    .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
                    .ok_or_else(|| Error::Config("FERRO_AI_API_KEY not set".into()))?;
                Ok(Box::new(AnthropicClient::new(key, model)))
            }
            "openai" => { ... }
            "groq" => { ... }
            "ollama" => { Ok(Box::new(OllamaClient::new(model, base_url))) }
            unknown => Err(Error::Config(format!("unknown FERRO_AI_PROVIDER: '{unknown}'")))
        }
    }
}
```

**Test pattern** (mirroring ferro-whatsapp/src/config.rs lines 69–96):
```rust
#[test]
fn from_env_fails_on_unknown_provider() {
    std::env::set_var("FERRO_AI_PROVIDER", "bogus");
    let result = AiConfig::from_env();
    assert!(matches!(result, Err(Error::Config(_))));
    std::env::remove_var("FERRO_AI_PROVIDER");
}
```

---

### `ferro-ai/src/error.rs` (error enum, modified)

**Analog:** `ferro-ai/src/error.rs` itself (same file, targeted additions per D-13 and D-14)

**Current shape** (lines 1–32 of error.rs):
```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("ai config error: {0}")]
    Config(String),

    #[error("ai provider error: {0}")]
    Provider(String),          // ← restructure to struct variant (D-14)

    #[error("low confidence classification (confidence: {confidence:.2})")]
    LowConfidence { best_guess: serde_json::Value, confidence: f64 },

    #[error("deserialization error: {0}")]
    Deserialization(String),

    #[error("classification request timed out after retries")]
    Timeout,

    #[error("confirmation store error: {0}")]
    StoreError(String),
}
```

**After D-13 + D-14**, add `Unsupported` and restructure `Provider`:
```rust
    /// HTTP provider error with optional status code for retry logic.
    #[error("ai provider error ({status:?}): {message}")]
    Provider { status: Option<u16>, message: String },   // breaking change — permitted in v12.1

    /// This provider does not implement the requested capability.
    #[error("capability not supported by this provider")]
    Unsupported,
```

**Add `is_retryable()` method** — replaces `is_permanent_provider_error` string-sniff (lines 176–182 of classifier/mod.rs):
```rust
impl Error {
    /// Permanent HTTP errors (400, 401, 403, 404, 422) are not retried.
    /// Transient (429, 500, 503, 529) and network errors (status: None) are retried.
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::Provider { status: Some(s), .. } => !matches!(s, 400 | 401 | 403 | 404 | 422),
            Error::Provider { status: None, .. } => true,
            _ => false,
        }
    }
}
```

**Test pattern** — mirror existing permanent/transient tests in anthropic.rs (lines 151–174):
```rust
#[test]
fn test_error_is_retryable() {
    assert!(!Error::Provider { status: Some(400), message: "".into() }.is_retryable());
    assert!(!Error::Provider { status: Some(401), message: "".into() }.is_retryable());
    assert!(Error::Provider { status: Some(429), message: "".into() }.is_retryable());
    assert!(Error::Provider { status: Some(500), message: "".into() }.is_retryable());
    assert!(Error::Provider { status: None, message: "".into() }.is_retryable());
    assert!(!Error::Unsupported.is_retryable());
    assert!(!Error::Timeout.is_retryable());
}
```

---

### `ferro-ai/src/classifier/mod.rs` (service, modified — retry logic)

**Analog:** `ferro-ai/src/classifier/mod.rs` itself (same file, targeted changes)

**Change 1 — D-03: ClassifierConfig::default() model field** (lines 33–43):
```rust
// Before:
model: "claude-sonnet-4-6".to_string(),

// After (Option A per RESEARCH.md):
model: String::new(),  // resolved from client.default_model() at call time
```

**Change 2 — D-14: retry match arm** (lines 149–151):
```rust
// Before (string-sniff pattern — the anti-pattern being replaced):
Err(Error::Provider(msg)) if is_permanent_provider_error(&msg) => {
    return Err(Error::Provider(msg));
}

// After (status-based is_retryable()):
Err(e) if !e.is_retryable() => {
    return Err(e);
}
```

**Change 3 — remove `is_permanent_provider_error` function** (lines 176–182). Delete this function; the `is_retryable()` method on `Error` replaces it.

**Change 4 — update test** `test_classifier_config_defaults` (lines 193–200):
```rust
// Before:
assert_eq!(config.model, "claude-sonnet-4-6");

// After:
assert!(config.model.is_empty());
```

**Change 5 — update CountingProvider tests** to emit `Error::Provider { status: Some(500), ... }` and `Error::Provider { status: Some(401), ... }` instead of string variants.

---

### `ferro-ai/src/classifier/anthropic.rs` (adapter, modified — D-10)

**Analog:** `ferro-ai/src/classifier/anthropic.rs` itself (same file, rewired)

The file becomes a thin adapter. The current `classify_raw` body (lines 94–143) is replaced by a delegation call to `AnthropicClient`.

**Before** (lines 83–144): full HTTP implementation in `classify_raw`.

**After pattern** — the new `AnthropicProvider` holds an `Arc<AnthropicClient>`:
```rust
pub struct AnthropicProvider {
    client: Arc<AnthropicClient>,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self { client: Arc::new(AnthropicClient::new(api_key, None)) }
    }

    pub fn from_env() -> Result<Self, Error> {
        let key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| Error::Config("ANTHROPIC_API_KEY not set".to_string()))?;
        Ok(Self::new(key))
    }
}

#[async_trait]
impl ClassificationProvider for AnthropicProvider {
    async fn classify_raw(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        schema: &serde_json::Value,
        config: &ClassifierConfig,
    ) -> Result<serde_json::Value, Error> {
        // Build CompletionRequest and delegate to AnthropicClient
        let request = CompletionRequest {
            system: Some(system_prompt.to_string()),
            messages: vec![Message { role: Role::User, content: user_prompt.to_string() }],
            max_tokens: config.max_tokens,
            schema: Some(schema.clone()),
            model_override: if config.model.is_empty() { None } else { Some(config.model.clone()) },
        };
        let text = self.client.complete(request).await?;
        serde_json::from_str(&text).map_err(|e| Error::Deserialization(e.to_string()))
    }
}
```

**Delete:** `is_permanent_error`, `is_transient_error`, `build_request_body` (HTTP moved to `AnthropicClient`).

**Keep/update tests:** `test_build_request_body_*` tests become `test_build_body_*` on `AnthropicClient` (moved to `client/anthropic.rs`). Update `test_build_request_body_contains_output_config` assertion from hardcoded `"claude-sonnet-4-6"` to a supplied model string.

---

### `ferro-ai/src/lib.rs` (re-export module, modified)

**Analog:** `ferro-ai/src/lib.rs` itself (lines 44–54)

**Current exports** (lines 44–54):
```rust
pub mod classifier;
pub mod confirmation;
pub mod error;

pub use classifier::anthropic::AnthropicProvider;
pub use classifier::provider::ClassificationProvider;
pub use classifier::{ClassificationResult, Classifier, ClassifierConfig};
pub use confirmation::events::ConfirmationExpired;
pub use confirmation::store::InMemoryConfirmationStore;
pub use confirmation::{ConfirmationStore, PendingActionInfo};
pub use error::Error;
```

**Add:**
```rust
pub mod client;
pub mod config;

pub use client::{AnthropicClient, LlmClient, OllamaClient, OpenAiClient, TokenStream};
pub use config::AiConfig;
```

---

### `ferro-ai/Cargo.toml` (manifest, modified)

**Analog:** `ferro-ai/Cargo.toml` itself (lines 1–26)

**Current deps:**
```toml
reqwest = { version = "0.12", features = ["json"] }
async-trait = "0.1"
thiserror = "2"
```

**Add:**
```toml
reqwest = { version = "0.12", features = ["json", "stream"] }   # add "stream" feature
reqwest-eventsource = { version = "0.6", default-features = false }
futures = { version = "0.3", default-features = false, features = ["std"] }
async-stream = "0.3"
```

`reqwest-eventsource` does not need `default-features` to strip anything critical here, but keeping it explicit aligns with workspace conventions for minimal dep footprint.

---

## Shared Patterns

### async_trait object-safe trait
**Source:** `ferro-ai/src/classifier/provider.rs` (entire file, 84 lines)
**Apply to:** `ferro-ai/src/client/mod.rs` (LlmClient trait definition)
```rust
#[async_trait]
pub trait ClassificationProvider: Send + Sync {
    async fn classify_raw(...) -> Result<serde_json::Value, Error>;
}
```

### reqwest Client construction (60-second timeout)
**Source:** `ferro-ai/src/classifier/anthropic.rs` lines 24–30
**Apply to:** `AnthropicClient::new`, `OpenAiClient::new`, `OllamaClient::new`
```rust
let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(60))
    .build()
    .expect("failed to build reqwest client");
```

### from_env() pattern for required env vars
**Source:** `ferro-whatsapp/src/config.rs` lines 40–57
**Apply to:** `ferro-ai/src/config.rs` (`AiConfig::from_env`), provider `from_env()` constructors
```rust
let var = std::env::var("VAR_NAME")
    .map_err(|_| Error::Config("VAR_NAME not set".into()))?;
```

### thiserror enum extension
**Source:** `ferro-ai/src/error.rs` lines 1–32 (current)
**Apply to:** `ferro-ai/src/error.rs` (D-13 + D-14 additions)
- `thiserror::Error` derive, one `Error` enum per crate
- Named struct variants for structured data: `LowConfidence { best_guess, confidence }` → same shape for `Provider { status, message }`

### Status check + error mapping
**Source:** `ferro-ai/src/classifier/anthropic.rs` lines 111–126
**Apply to:** All three provider `complete()` implementations
```rust
let status = response.status().as_u16();
if !response.status().is_success() {
    let text = response.text().await.unwrap_or_default();
    return Err(Error::Provider { status: Some(status), message: text });
}
```
(collapse the separate permanent/transient checks from the analog into a single `!is_success()` check)

### Retry loop
**Source:** `ferro-ai/src/classifier/mod.rs` lines 112–168
**Apply to:** `ferro-ai/src/classifier/mod.rs` (modified — replace `is_permanent_provider_error` predicate with `!e.is_retryable()`)

---

## Streaming — Net-New to Workspace

No existing ferro-* crate uses SSE client-side streaming or `BoxStream`. The SSE JS file (`ferro-json-ui/src/runtime/sse.rs`) is server-side browser JavaScript — not applicable. The streaming implementation must be written fresh per RESEARCH.md Patterns 2 and 3.

**Summary of streaming dependencies:**
- Anthropic + OpenAI: `reqwest-eventsource 0.6` (`RequestBuilderExt::eventsource()` → `Stream<Item = Result<Event, _>>`), filtered to `content_block_delta` / `choices[0].delta.content`
- Ollama: `reqwest::Response::bytes_stream()` + `async_stream::try_stream!` macro, NDJSON line-by-line
- `TokenStream` type alias: `futures::stream::BoxStream<'static, Result<String, Error>>`

| File | Stream Mechanism | Crate |
|---|---|---|
| `client/anthropic.rs` | `EventSource` from `reqwest-eventsource` | `reqwest-eventsource` |
| `client/openai.rs` | `EventSource` from `reqwest-eventsource` | `reqwest-eventsource` |
| `client/ollama.rs` | `response.bytes_stream()` + `try_stream!` | `async-stream` + `futures` |

---

## No Analog Found

No files are entirely without a codebase reference. All net-new streaming code has concrete patterns documented in RESEARCH.md (verified against `reqwest-eventsource 0.6.0` source in local cargo registry).

---

## Metadata

**Analog search scope:** `ferro-ai/`, `ferro-whatsapp/`, `ferro-storage/`, `ferro-cache/`, `ferro-broadcast/`, `ferro-json-ui/`, `ferro-cli/`
**Files scanned:** ~15 source files read; ~20 files grep-searched
**Key finding:** Streaming (SSE client-side + BoxStream) is net-new to the workspace. Every other pattern — async_trait object-safe trait, reqwest HTTP client, from_env() config, thiserror enum — has a direct codebase analog.
**Pattern extraction date:** 2026-06-08
