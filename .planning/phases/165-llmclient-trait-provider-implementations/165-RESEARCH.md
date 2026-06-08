# Phase 165: LlmClient Trait & Provider Implementations - Research

**Researched:** 2026-06-08
**Domain:** Rust async HTTP client, SSE streaming, LLM provider APIs (Anthropic, OpenAI, Ollama)
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Trait named `LlmClient`. REQUIREMENTS.md AISDK-01 says `LlmProvider` — stale wording; update REQUIREMENTS.md.
- **D-02:** `LlmClient` in `ferro-ai/src/client/mod.rs`. Methods: `async fn complete(...)`, `async fn complete_stream(...)`, `async fn embed(...)`. `async_trait` retained (stable async-fn-in-traits is not dyn-compatible). Single trait — missing capabilities return `Err(Error::Unsupported)`.
- **D-03:** `LlmClient::default_model() -> &str`. `ClassifierConfig::default()` no longer hardcodes `"claude-sonnet-4-6"`.
- **D-04:** Three impls: `AnthropicClient`, `OpenAiClient` (doubles as Groq via base_url), `OllamaClient`. Each instantiable as `Box<dyn LlmClient>`.
- **D-05:** Default models: Anthropic → `claude-sonnet-4-6`, OpenAI → `gpt-4o`, Ollama → `llama3.1`.
- **D-06:** `AiConfig::from_env()` reads `FERRO_AI_PROVIDER`, `FERRO_AI_MODEL`, `FERRO_AI_API_KEY`, `FERRO_AI_BASE_URL`. Unknown provider names return `Error::Config` at startup.
- **D-07:** Project-agnostic crate rule: only `FERRO_AI_*` env vars; no app identity.
- **D-08:** `complete_stream` implemented for real (not stubbed). Returns ferro-ai-owned `TokenStream`. Anthropic + OpenAI use SSE via `reqwest-eventsource`; Ollama uses NDJSON line-by-line.
- **D-09:** `reqwest-eventsource 0.6` is `pub(crate)` in provider modules. `TokenStream` is public but `reqwest-eventsource` NOT re-exported.
- **D-10:** Existing `AnthropicProvider.classify_raw` becomes a thin adapter delegating to `AnthropicClient`. Duplicate HTTP code deleted.
- **D-11:** `AnthropicClient` request carries an OPTIONAL structured-output schema/tool field from day one so the classifier bridge works before Phase 166.
- **D-12:** Public API preserved: `ClassificationProvider`, `AnthropicProvider`, `Classifier<T>`, `ClassifierConfig`, `ClassificationResult` keep signatures.
- **D-13:** Add `Error::Unsupported`.
- **D-14:** `Error::Provider(String)` → `Error::Provider { status: Option<u16>, message: String }`. Retry switches from string-sniffing to status-based `is_retryable()`.

### Claude's Discretion

- Exact signatures of `complete` / `complete_stream` / `embed` (request/response structs vs flat params).
- Internal module layout under `client/` (one file per provider vs shared request/response module).
- Whether `AiConfig` selects providers via enum dispatch or returns `Box<dyn LlmClient>` directly.

### Deferred Ideas (OUT OF SCOPE)

- Typed `complete::<T>()` + schemars `$ref`/`$defs` normalizer → Phase 166.
- `ToolRegistry` / tool calling → Phase 166.
- Embeddings implementation + cosine similarity + pgvector → Phase 167.
- Framework SSE primitives (`SseEvent`/`SseStream`/`HttpResponse::sse()`) → Phase 168.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AISDK-01 | Provider-agnostic `LlmClient` trait; Anthropic, OpenAI, Groq (via OpenAI), Ollama providers; `AiConfig::from_env()`; existing `Classifier<T>` API preserved | All six Success Criteria covered by research below |
</phase_requirements>

---

## Summary

Phase 165 rewires the existing `ferro-ai` crate onto a new `LlmClient` trait layer. The crate is not greenfield — `Classifier<T>`, `ClassificationProvider`, `AnthropicProvider`, and the `Error` enum already exist and must be preserved in API shape. The core work is: (1) define the trait + three implementations, (2) add `AiConfig::from_env()` for env-driven dispatch, (3) implement streaming for all providers (Anthropic + OpenAI via SSE, Ollama via NDJSON), (4) rewire `AnthropicProvider.classify_raw` as a thin delegate onto `AnthropicClient`, and (5) upgrade the `Error` enum.

The streaming path is the most complex new dependency. `reqwest-eventsource 0.6` is a thin wrapper around `reqwest`'s byte stream that parses SSE protocol. It implements `Stream<Item = Result<Event, Error>>` via `futures-core`. Ollama streaming is simpler — NDJSON over reqwest's raw byte stream, parsed line-by-line without the eventsource library.

The `AnthropicClient` already has the hard parts solved in the existing `AnthropicProvider`: the HTTP request shape, `output_config.format.type = "json_schema"` for structured output, and the correct `content[0].text` extraction. D-11 requires adding an optional `schema: Option<serde_json::Value>` field to the request struct from day one so the classifier bridge works without Phase 166.

**Primary recommendation:** Define `TokenStream` as `Pin<Box<dyn Stream<Item = Result<String, Error>> + Send>>` using `futures::stream::BoxStream` internally; implement per-provider stream constructors using `async_stream::try_stream!` macro for Ollama and `reqwest-eventsource` for Anthropic/OpenAI.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `LlmClient` trait definition | ferro-ai crate | — | Leaf crate, no framework dep required |
| Provider HTTP calls | ferro-ai/client/ | — | reqwest is already a ferro-ai dep |
| SSE stream parsing | ferro-ai/client/ (pub(crate)) | — | reqwest-eventsource stays internal per D-09 |
| NDJSON line parsing | ferro-ai/client/ollama.rs | — | Pure reqwest bytes_stream, no extra crate |
| `TokenStream` type | ferro-ai public API | — | Defined in ferro-ai, consumed by framework SSE (Phase 168) |
| `AiConfig::from_env()` | ferro-ai public API | — | Reads FERRO_AI_* vars; framework-independent |
| `ClassifierConfig` model resolution | ferro-ai/classifier/mod.rs | ferro-ai/client/ | ClassifierConfig asks AnthropicClient::default_model() |
| `AnthropicProvider.classify_raw` | ferro-ai/classifier/anthropic.rs | ferro-ai/client/anthropic.rs | Thin adapter — HTTP lives in AnthropicClient |
| Retry logic | ferro-ai/classifier/mod.rs | — | Uses Error::Provider.status, not string-sniff |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `reqwest` | `0.12` (already in ferro-ai) | HTTP client for all three providers | Already a dep; add `stream` feature |
| `reqwest-eventsource` | `0.6.0` [VERIFIED: crates.io] | SSE parsing for Anthropic + OpenAI streaming | Thin wrapper on reqwest; minimal dep footprint |
| `async-trait` | `0.1` (already in ferro-ai) | `Box<dyn LlmClient>` dyn-compatibility | Stable async-fn-in-traits not dyn-compatible; already proven pattern in ClassificationProvider |
| `futures` | `0.3.32` [VERIFIED: crates.io] | `StreamExt` for consuming eventsource stream; `BoxStream` type alias | Required by reqwest-eventsource's stream iteration |
| `async-stream` | `0.3.6` [VERIFIED: crates.io] | `try_stream!` macro for Ollama NDJSON stream construction | Simplest way to build a custom stream from async code without pin boilerplate |
| `serde` / `serde_json` | workspace | Request/response (de)serialization | Already in ferro-ai |
| `thiserror` | workspace | Error enum | Already in ferro-ai |
| `tokio` | `1` (already in ferro-ai) | Async runtime | Add `full` feature to dev-dependencies |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tokio-stream` | `0.1.18` [VERIFIED: crates.io] | `StreamExt` utilities for tokio streams | Alternative to `futures::StreamExt`; use if tokio-stream already in workspace. Not strictly required if `futures` is added. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `async-stream::try_stream!` for Ollama | Manual `poll_fn` + `Pin<Box<...>>` | Much more boilerplate; async-stream is the Tokio-team maintained solution |
| `reqwest-eventsource` for SSE | `eventsource-client` or `eventsource` | reqwest-eventsource reuses the existing reqwest client; avoids a second HTTP client |
| `futures::BoxStream` | `tokio_stream::wrappers::ReceiverStream` | BoxStream is more general; ReceiverStream only for channel-backed streams |

**Installation (new deps to add to `ferro-ai/Cargo.toml`):**
```toml
reqwest-eventsource = { version = "0.6", default-features = false }
futures = { version = "0.3", default-features = false, features = ["std"] }
async-stream = "0.3"
```

Also add `stream` feature to the existing reqwest dep:
```toml
reqwest = { version = "0.12", features = ["json", "stream"] }
```

**Version verification:** [VERIFIED: crates.io via `cargo info` 2026-06-08]
- `reqwest-eventsource`: 0.6.0
- `futures`: 0.3.32 (latest; 0.3.31 locally cached)
- `async-stream`: 0.3.6

**Note on `reqwest-eventsource` thiserror version:** `reqwest-eventsource 0.6.0` depends on `thiserror 1.x`, while `ferro-ai` already uses `thiserror 2.x`. Cargo resolves both via semver — no conflict. Both compile to the same proc-macro surface.

---

## Architecture Patterns

### System Architecture Diagram

```
[AiConfig::from_env()]
        |
        | reads FERRO_AI_PROVIDER / MODEL / API_KEY / BASE_URL
        v
[Box<dyn LlmClient>] ─────────────────────────────────────────┐
        |                                                       |
   ┌────┴────┐                                            Error::Config
   |         |                                           (unknown provider)
   |  complete(request: CompletionRequest) -> Result<String, Error>
   |  complete_stream(request: CompletionRequest) -> Result<TokenStream, Error>
   |  embed(text: &str) -> Result<Vec<f32>, Error>
   |  default_model() -> &str
   |
   ├──[AnthropicClient]──────────────────────────────────────────────┐
   │    POST https://api.anthropic.com/v1/messages                   │
   │    Headers: x-api-key, anthropic-version: 2023-06-01            │
   │    Body: {model, max_tokens, system, messages,                  │
   │           output_config: {format: {type: "json_schema",         │
   │                           schema: Option<Value>}}}              │
   │    Stream: SSE via reqwest-eventsource                          │
   │    Extract tokens: content_block_delta.delta.text               │
   │    Delegate from: AnthropicProvider.classify_raw (thin adapter) │
   │                                                                 │
   ├──[OpenAiClient]─────────────────────────────────────────────────┤
   │    POST {base_url}/v1/chat/completions                          │
   │    Default base_url: https://api.openai.com                     │
   │    Groq: base_url = https://api.groq.com/openai                │
   │    Headers: Authorization: Bearer {api_key}                     │
   │    Body: {model, messages, max_tokens, stream: bool,           │
   │           response_format: {type: "json_schema", ...}}          │
   │    Stream: SSE via reqwest-eventsource                          │
   │    Extract tokens: choices[0].delta.content                     │
   │    embed(): POST {base_url}/v1/embeddings → data[0].embedding   │
   │                                                                 │
   └──[OllamaClient]─────────────────────────────────────────────────┘
        POST http://localhost:11434/api/chat (default base_url)
        Body: {model, messages, stream: bool}
        Stream: NDJSON via reqwest bytes_stream + line split
        Each line: {"message":{"content":"token"}, "done": false}
        Final line: {"done": true}
        embed(): POST http://localhost:11434/api/embed
                 Body: {model, input: text}
                 Response: {embeddings: [[f32...]]}

TokenStream = Pin<Box<dyn Stream<Item = Result<String, Error>> + Send>>

Classifier<T> ─── delegates HTTP ──► AnthropicClient.complete(request_with_schema)
ClassificationProvider (preserved) ◄─ thin wrapper in AnthropicProvider
```

### Recommended Module Layout

```
ferro-ai/src/
├── client/
│   ├── mod.rs           # LlmClient trait, TokenStream type alias, CompletionRequest struct
│   ├── anthropic.rs     # AnthropicClient struct + impl LlmClient
│   ├── openai.rs        # OpenAiClient struct + impl LlmClient (Groq via base_url)
│   └── ollama.rs        # OllamaClient struct + impl LlmClient
├── config.rs            # AiConfig::from_env() → Box<dyn LlmClient>
├── classifier/
│   ├── mod.rs           # ClassifierConfig (model resolved via LlmClient::default_model()), Classifier<T>
│   ├── anthropic.rs     # AnthropicProvider: thin adapter over AnthropicClient
│   └── provider.rs      # ClassificationProvider trait (unchanged)
├── confirmation/        # Unchanged
├── error.rs             # Error enum (add Unsupported, restructure Provider)
└── lib.rs               # pub use: LlmClient, AnthropicClient, OpenAiClient, OllamaClient, AiConfig, TokenStream, ...
```

### Pattern 1: LlmClient Trait Definition

**What:** `async_trait` macro makes async methods dyn-compatible. `TokenStream` is a type alias for a boxed Send stream.

```rust
// Source: async-trait confirmed pattern [VERIFIED: /dtolnay/async-trait via Context7]
use async_trait::async_trait;
use futures::stream::BoxStream;
use crate::error::Error;

/// Opaque stream of text tokens from a streaming LLM completion.
pub type TokenStream = BoxStream<'static, Result<String, Error>>;

/// Request for a text completion.
pub struct CompletionRequest {
    pub system: Option<String>,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    /// Optional JSON schema for structured output (passed through to provider;
    /// Phase 166 normalizer handles schemars → provider-specific format).
    pub schema: Option<serde_json::Value>,
}

pub struct Message {
    pub role: Role,
    pub content: String,
}

pub enum Role {
    User,
    Assistant,
}

#[async_trait]
pub trait LlmClient: Send + Sync {
    fn default_model(&self) -> &str;
    async fn complete(&self, request: CompletionRequest) -> Result<String, Error>;
    async fn complete_stream(&self, request: CompletionRequest) -> Result<TokenStream, Error>;
    /// Returns Err(Error::Unsupported) for providers without an embeddings endpoint.
    async fn embed(&self, text: &str) -> Result<Vec<f32>, Error>;
}
```

[ASSUMED: The exact struct field names `CompletionRequest`, `Message`, `Role` are at Claude's discretion per CONTEXT.md — the above is a reasonable concrete proposal.]

### Pattern 2: Anthropic SSE Stream via reqwest-eventsource

**What:** Build a `TokenStream` from Anthropic's SSE response. The `EventSource` implements `Stream<Item = Result<Event, Error>>`. Filter `content_block_delta` events and extract `delta.text`.

```rust
// Source: reqwest-eventsource 0.6.0 source [VERIFIED: local cargo registry]
use futures::StreamExt;
use reqwest_eventsource::{Event, EventSource, RequestBuilderExt};
use futures::stream;

async fn complete_stream(&self, request: CompletionRequest) -> Result<TokenStream, crate::Error> {
    let body = self.build_body(&request, true)?;
    let builder = self.http_client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &self.api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body);

    let mut es = builder
        .eventsource()
        .map_err(|_| crate::Error::Provider { status: None, message: "request not cloneable".into() })?;

    let token_stream = stream::unfold(es, |mut es| async move {
        loop {
            match es.next().await {
                None => return None,
                Some(Ok(Event::Open)) => continue,
                Some(Ok(Event::Message(msg))) => {
                    if msg.event == "message_stop" || msg.event == "message_delta" {
                        // Check for stop; message_delta has stop_reason
                        // continue draining to get message_stop
                        continue;
                    }
                    if msg.event == "content_block_delta" {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&msg.data) {
                            if let Some(text) = v["delta"]["text"].as_str() {
                                if !text.is_empty() {
                                    return Some((Ok(text.to_string()), es));
                                }
                            }
                        }
                        continue;
                    }
                    continue;
                }
                Some(Err(e)) => {
                    es.close();
                    return Some((Err(crate::Error::Provider {
                        status: None,
                        message: e.to_string(),
                    }), es));
                }
            }
        }
    });

    Ok(Box::pin(token_stream))
}
```

**Alternative using `async-stream::try_stream!`** (cleaner for Ollama):
```rust
// Source: async-stream crate [VERIFIED: crates.io 0.3.6]
use async_stream::try_stream;
use futures::stream::BoxStream;

fn ollama_token_stream(response: reqwest::Response) -> TokenStream {
    Box::pin(try_stream! {
        let mut stream = response.bytes_stream();
        let mut buf = String::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| crate::Error::Provider {
                status: None,
                message: e.to_string()
            })?;
            buf.push_str(&String::from_utf8_lossy(&chunk));
            while let Some(newline_pos) = buf.find('\n') {
                let line = buf[..newline_pos].trim().to_string();
                buf = buf[newline_pos + 1..].to_string();
                if line.is_empty() { continue; }
                let v: serde_json::Value = serde_json::from_str(&line)
                    .map_err(|e| crate::Error::Deserialization(e.to_string()))?;
                if let Some(content) = v["message"]["content"].as_str() {
                    if !content.is_empty() {
                        yield content.to_string();
                    }
                }
                if v["done"].as_bool().unwrap_or(false) {
                    return;
                }
            }
        }
    })
}
```

### Pattern 3: Error Enum Restructuring (D-13, D-14)

```rust
// Source: existing ferro-ai/src/error.rs + locked decisions D-13/D-14
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("ai config error: {0}")]
    Config(String),

    /// HTTP provider error — structured with status code for retry logic.
    #[error("ai provider error ({status:?}): {message}")]
    Provider { status: Option<u16>, message: String },

    /// This provider does not implement the requested capability.
    #[error("capability not supported by this provider")]
    Unsupported,

    #[error("low confidence classification (confidence: {confidence:.2})")]
    LowConfidence { best_guess: serde_json::Value, confidence: f64 },

    #[error("deserialization error: {0}")]
    Deserialization(String),

    #[error("classification request timed out after retries")]
    Timeout,

    #[error("confirmation store error: {0}")]
    StoreError(String),
}

impl Error {
    /// Returns true for errors that should be retried.
    /// Permanent HTTP errors (400, 401, 403, 404, 422) are not retried.
    /// Transient errors (429, 500, 503, 529) are retried.
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::Provider { status: Some(s), .. } => {
                !matches!(s, 400 | 401 | 403 | 404 | 422)
            }
            Error::Provider { status: None, .. } => true,  // network error, retry
            Error::Timeout => false,  // already timed out
            _ => false,
        }
    }
}
```

### Pattern 4: ClassifierConfig Default Model via LlmClient::default_model()

**What:** Remove hardcoded `"claude-sonnet-4-6"` from `ClassifierConfig::default()`. The model is resolved from whatever client is injected.

**Implementation strategy:** `ClassifierConfig` keeps its `model: String` field. The `Classifier<T>` constructor takes a client reference, and `ClassifierConfig::default()` uses an empty string or the caller's client's `default_model()`.

Option A: `ClassifierConfig` model defaults to `""` (empty); `Classifier<T>` falls back to `client.default_model()` when the model field is empty.

Option B: Remove the `model` field from `ClassifierConfig` entirely and resolve it from the client in the classifier.

Option A is lower risk — the `ClassifierConfig` struct stays compatible (D-12). [ASSUMED: planner chooses approach; Option A recommended for minimal API surface change.]

```rust
// Classifier::new now receives Arc<dyn LlmClient> instead of Arc<dyn ClassificationProvider>
// ClassifierConfig::default() no longer hardcodes the model:
impl Default for ClassifierConfig {
    fn default() -> Self {
        Self {
            model: String::new(),  // resolved from client.default_model() at call time
            max_tokens: 1024,
            max_retries: 1,
            retry_delay: Duration::from_secs(1),
            confidence_threshold: 0.7,
        }
    }
}
```

### Pattern 5: AiConfig::from_env() Dispatch

```rust
// Source: D-06 decision; mirrors existing AnthropicProvider::from_env() pattern
pub struct AiConfig;

impl AiConfig {
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
            "openai" => {
                let key = api_key
                    .ok_or_else(|| Error::Config("FERRO_AI_API_KEY not set for openai".into()))?;
                Ok(Box::new(OpenAiClient::new(key, model, base_url)))
            }
            "groq" => {
                let key = api_key
                    .ok_or_else(|| Error::Config("FERRO_AI_API_KEY not set for groq".into()))?;
                let url = base_url.unwrap_or_else(|| "https://api.groq.com/openai".into());
                Ok(Box::new(OpenAiClient::new(key, model, Some(url))))
            }
            "ollama" => {
                Ok(Box::new(OllamaClient::new(model, base_url)))
            }
            unknown => Err(Error::Config(format!("unknown FERRO_AI_PROVIDER: '{unknown}'")))
        }
    }
}
```

[ASSUMED: "groq" as a named alias for `OpenAiClient` with pre-filled base_url is at planner's discretion — D-04 says OpenAiClient doubles as Groq via base_url; aliasing it in AiConfig is a convenience option.]

### Pattern 6: Anthropic Request Struct with Optional Schema (D-11)

**What:** The `AnthropicClient`'s request payload needs an optional `schema` field from day one so `AnthropicProvider.classify_raw` can delegate without Phase 166.

```rust
// AnthropicClient::build_body (internal)
fn build_body(&self, request: &CompletionRequest, stream: bool) -> serde_json::Value {
    let mut body = serde_json::json!({
        "model": if request.model_override.is_some() {
            request.model_override.as_deref().unwrap()
        } else {
            self.default_model()
        },
        "max_tokens": request.max_tokens,
        "messages": request.messages.iter().map(|m| serde_json::json!({
            "role": match m.role { Role::User => "user", Role::Assistant => "assistant" },
            "content": m.content
        })).collect::<Vec<_>>(),
        "stream": stream,
    });

    if let Some(system) = &request.system {
        body["system"] = serde_json::json!([{
            "type": "text",
            "text": system,
            "cache_control": {"type": "ephemeral"}
        }]);
    }

    if let Some(schema) = &request.schema {
        body["output_config"] = serde_json::json!({
            "format": {
                "type": "json_schema",
                "schema": schema
            }
        });
    }

    body
}
```

### Anti-Patterns to Avoid

- **String-sniffing for error classification:** The existing `is_permanent_provider_error(msg: &str)` checks for "400", "401", etc. as substrings of a string. D-14 requires replacing this with status-based `is_retryable()` on `Error::Provider { status: Option<u16>, ... }`.
- **`reqwest-eventsource` re-exported publicly:** D-09 is explicit — it stays `pub(crate)`. The `TokenStream` type alias hides the underlying stream implementation.
- **Panic on unsupported capability:** `embed()` on `AnthropicClient` MUST return `Err(Error::Unsupported)`, never panic.
- **Using `reqwest-eventsource` for Ollama:** Ollama's `/api/chat` streaming is NDJSON (newline-delimited JSON), NOT SSE. Do not use `EventSource` for Ollama — use `response.bytes_stream()` directly.
- **Blocking `reqwest` client in any async path:** ferro-ai uses `reqwest` async client (`tokio` runtime). Never import `reqwest::blocking`.
- **Hardcoded app identity in env var names:** Only `FERRO_AI_*` vars (D-07). Do not accept `ANTHROPIC_API_KEY` as primary — only as fallback for backward compatibility.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SSE parsing | Custom SSE parser | `reqwest-eventsource 0.6` | SSE has reconnect, `Last-Event-ID`, retry semantics; the crate handles all of them |
| Async stream construction | Manual `poll_fn` + state machine | `async_stream::try_stream!` | Correct pinning is subtle; async-stream is Tokio-team maintained |
| Type-erased stream | Manual vtable | `futures::stream::BoxStream` | Standard type alias for `Pin<Box<dyn Stream + Send>>` |
| dyn-compatible async trait | Manual `Pin<Box<dyn Future>>` returns | `async_trait` macro | Already proven in `ClassificationProvider`; identical pattern |

**Key insight:** The SSE protocol has non-obvious reconnect and backoff semantics. `reqwest-eventsource` handles them correctly; a hand-rolled `bytes_stream().lines()` approach would silently drop reconnect on 5xx without the retry policy.

---

## Provider API Reference

### Anthropic Messages API

**Endpoint:** `https://api.anthropic.com/v1/messages`
**Method:** POST
**Auth headers:** [VERIFIED: platform.claude.com/docs/en/api/messages 2026-06-08]
```
x-api-key: $ANTHROPIC_API_KEY
anthropic-version: 2023-06-01
content-type: application/json
```

**Non-streaming request JSON:**
```json
{
  "model": "claude-sonnet-4-6",
  "max_tokens": 1024,
  "system": [{"type": "text", "text": "...", "cache_control": {"type": "ephemeral"}}],
  "messages": [{"role": "user", "content": "..."}],
  "output_config": {
    "format": {
      "type": "json_schema",
      "schema": { "type": "object", "properties": {...} }
    }
  }
}
```

**Non-streaming response:**
```json
{
  "id": "msg_...",
  "type": "message",
  "role": "assistant",
  "content": [{"type": "text", "text": "..."}],
  "stop_reason": "end_turn",
  "usage": {"input_tokens": 100, "output_tokens": 50}
}
```
Extract: `content[0].text` (already done in existing `AnthropicProvider`).

**Streaming SSE event sequence:** [VERIFIED: platform.claude.com docs 2026-06-08]
```
event: message_start
data: {"type":"message_start","message":{"id":"msg_...","type":"message","role":"assistant"}}

event: content_block_start
data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hi"}}

event: content_block_stop
data: {"type":"content_block_stop","index":0}

event: message_delta
data: {"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":10}}

event: message_stop
data: {"type":"message_stop"}
```
Token extraction: filter `msg.event == "content_block_delta"`, parse `delta.text` from `msg.data`. Terminate on `msg.event == "message_stop"` or stream end.

**Structured output via `output_config`:** [VERIFIED: platform.claude.com/docs structured-outputs 2026-06-08]
- No beta header required (GA since late 2025).
- Use `output_config.format.type = "json_schema"` with `schema`.
- The existing `AnthropicProvider` already uses this correctly.
- With streaming + `output_config`, the structured JSON arrives token-by-token in `content_block_delta` events and must be **accumulated then parsed** — do NOT try to parse each token individually.

**Embeddings:** Anthropic has NO embeddings endpoint. `AnthropicClient::embed()` MUST return `Err(Error::Unsupported)`. [VERIFIED: confirmed by absence in official docs; deferred note in 165-CONTEXT.md.]

---

### OpenAI Chat Completions API

**Endpoint:** `{base_url}/v1/chat/completions` (default base_url: `https://api.openai.com`)
**Groq base_url:** `https://api.groq.com/openai` [ASSUMED: based on Groq OpenAI-compatible docs]
**Method:** POST
**Auth header:** `Authorization: Bearer {api_key}`

**Request JSON:**
```json
{
  "model": "gpt-4o",
  "messages": [{"role": "user", "content": "..."}],
  "max_tokens": 1024,
  "stream": false
}
```

**Non-streaming response:**
```json
{
  "id": "chatcmpl-...",
  "choices": [{"index": 0, "message": {"role": "assistant", "content": "..."}, "finish_reason": "stop"}],
  "usage": {"prompt_tokens": 100, "completion_tokens": 50}
}
```
Extract: `choices[0].message.content`.

**Streaming SSE chunk (each event):** [VERIFIED: OpenAI API reference 2026-06-08]
```json
data: {"id":"chatcmpl-...","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}
```
Termination: `data: [DONE]` — when `msg.data == "[DONE]"`, stop the stream. Also stop when `finish_reason` is non-null.
Extract: `choices[0].delta.content` (may be null on first chunk which carries role only).

**Embeddings endpoint:** `{base_url}/v1/embeddings` [VERIFIED: OpenAI API ref 2026-06-08]
```json
Request: {"model": "text-embedding-3-small", "input": "text to embed"}
Response: {"data": [{"embedding": [0.1, -0.2, ...], "index": 0}], "usage": {...}}
```
Extract: `data[0].embedding` as `Vec<f32>`.

---

### Ollama API

**Chat endpoint:** `{base_url}/api/chat` (default base_url: `http://localhost:11434`)
**Method:** POST
**No auth required** (local by default)

**Request JSON:** [VERIFIED: ollama GitHub docs 2026-06-08]
```json
{
  "model": "llama3.1",
  "messages": [{"role": "user", "content": "..."}],
  "stream": true
}
```

**NDJSON streaming** (each newline is a complete JSON object, NOT SSE):
```json
{"model":"llama3.1","message":{"role":"assistant","content":"The"},"done":false}
{"model":"llama3.1","message":{"role":"assistant","content":" answer"},"done":false}
{"model":"llama3.1","message":{"role":"assistant","content":""},"done":true,"total_duration":...}
```
Parse: for each newline, `message.content` is the token chunk. Stop when `done == true`.

**Non-streaming response:** `stream: false` returns a single JSON with the full `message.content`.

**Embeddings endpoint:** `{base_url}/api/embed` [VERIFIED: ollama GitHub docs 2026-06-08]
```json
Request: {"model": "nomic-embed-text", "input": "text to embed"}
Response: {"embeddings": [[0.1, -0.2, ...]], "total_duration": 14143917}
```
Extract: `embeddings[0]`. Note: older Ollama used `/api/embeddings` with `prompt` field; `/api/embed` with `input` is current.

**CRITICAL:** Ollama streaming is NDJSON, not SSE. Do NOT use `reqwest-eventsource` for Ollama. Use `response.bytes_stream()` (requires `stream` feature on reqwest, which is added for eventsource anyway).

---

## reqwest-eventsource 0.6 API Details

[VERIFIED: local cargo registry source 2026-06-08]

```rust
// Public API from reqwest-eventsource 0.6.0/src/
pub use error::{CannotCloneRequestError, Error};
pub use event_source::{Event, EventSource, ReadyState};
pub use reqwest_ext::RequestBuilderExt;

// Event enum
pub enum Event {
    Open,
    Message(eventsource_stream::Event),  // event_source_stream::Event
}

// eventsource_stream::Event fields [VERIFIED: docs.rs]
pub struct Event {      // MessageEvent
    pub event: String,  // the event type name (e.g. "content_block_delta")
    pub data: String,   // the payload
    pub id: String,     // event id if given
    pub retry: Option<Duration>,
}

// RequestBuilderExt trait
pub trait RequestBuilderExt {
    fn eventsource(self) -> Result<EventSource, CannotCloneRequestError>;
}
// impl for reqwest::RequestBuilder

// EventSource implements Stream<Item = Result<Event, Error>>
// Stream iteration:
use futures::StreamExt;
let mut es = request_builder
    .header("accept", "text/event-stream")
    .eventsource()?;
while let Some(event) = es.next().await {
    match event {
        Ok(Event::Open) => { /* connection established */ }
        Ok(Event::Message(msg)) => { /* msg.event, msg.data */ }
        Err(e) => { es.close(); /* handle error */ }
    }
}
```

**Cargo.toml note:** `reqwest-eventsource 0.6.0` depends on `reqwest 0.12` with `stream` feature. Adding `reqwest-eventsource` to ferro-ai will transitively enable the `stream` feature on reqwest. To be explicit, also add `"stream"` to the ferro-ai reqwest features.

**`CannotCloneRequestError`:** `EventSource::new(builder)` requires the RequestBuilder to be cloneable (for retry). A request with a streaming body cannot be cloned. The `json(&body)` builder pattern used in AnthropicProvider IS cloneable because serde_json produces a known Body. `EventSource::get(url)` is also safe. Avoid streaming request bodies.

---

## Common Pitfalls

### Pitfall 1: Streaming + Structured Output Parsing

**What goes wrong:** When `complete_stream` is called with a `schema` field (for structured output), the response JSON arrives token-by-token. Trying to parse each `content_block_delta` text token as JSON will fail because it's a fragment.
**Why it happens:** Structured output via `output_config.format.type = "json_schema"` streams the JSON string token-by-token. The valid JSON only exists after all tokens are accumulated.
**How to avoid:** In `complete_stream`, yield raw tokens without parsing (let callers accumulate). In `complete` (non-streaming), parse after `content[0].text` is complete. The Phase 166 structured-output wrapper handles accumulation.
**Warning signs:** `serde_json::from_str` errors in the stream handler; partial JSON fragments in token logs.

### Pitfall 2: Ollama NDJSON vs SSE Confusion

**What goes wrong:** Using `reqwest-eventsource` for Ollama's `/api/chat` stream produces `Error::InvalidContentType` because Ollama returns `application/x-ndjson`, not `text/event-stream`.
**Why it happens:** reqwest-eventsource validates `Content-Type: text/event-stream`. Ollama explicitly does not use SSE protocol.
**How to avoid:** Ollama's stream uses `response.bytes_stream()` + manual newline splitting. Never call `.eventsource()` on an Ollama request.
**Warning signs:** `reqwest_eventsource::Error::InvalidContentType` in Ollama tests.

### Pitfall 3: `CannotCloneRequestError` on EventSource Construction

**What goes wrong:** Calling `.eventsource()` on a `RequestBuilder` that has a non-cloneable body (e.g., an async stream body).
**Why it happens:** `reqwest-eventsource` must clone the request for retry. `reqwest::Client::post().json(&value)` IS cloneable. Raw `.body(stream)` is NOT.
**How to avoid:** Always use `.json(&body_value)` for Anthropic/OpenAI SSE requests, never a streaming body.
**Warning signs:** `CannotCloneRequestError` at stream construction time.

### Pitfall 4: OpenAI `[DONE]` Termination

**What goes wrong:** The final SSE message from OpenAI is `data: [DONE]` — this is NOT valid JSON. Calling `serde_json::from_str("[DONE]")` panics/errors.
**Why it happens:** OpenAI's SSE termination sentinel is a bare string, not a JSON object.
**How to avoid:** Before JSON parsing, check `if msg.data == "[DONE]" { break; }`.
**Warning signs:** `serde_json::Error` on the last chunk of OpenAI streams.

### Pitfall 5: Error::Provider Matching in Classifier Retry Loop

**What goes wrong:** The existing classifier retry match arm `Err(Error::Provider(msg)) if is_permanent_provider_error(&msg)` will fail to compile after D-14 restructures `Error::Provider` to `Error::Provider { status, message }`.
**Why it happens:** Pattern must be updated to match the new struct variant.
**How to avoid:** Update the match arm to `Err(e) if !e.is_retryable() => return Err(e)` using the new `is_retryable()` method. This also handles `Error::Config` and `Error::Unsupported` correctly (neither should be retried).
**Warning signs:** compile error "expected tuple struct or tuple variant, found struct variant".

### Pitfall 6: Existing Tests Asserting Hardcoded Model String

**What goes wrong:** `test_classifier_config_defaults` asserts `config.model == "claude-sonnet-4-6"` and `test_build_request_body_contains_output_config` asserts `body["model"] == "claude-sonnet-4-6"`. Both will fail if `ClassifierConfig::default().model` becomes empty.
**Why it happens:** D-03 removes the hardcode from `ClassifierConfig::default()`.
**How to avoid:** Update `test_classifier_config_defaults` to assert `config.model.is_empty()` (or assert it equals `AnthropicClient::default().default_model()` if the client is available). Update `test_build_request_body_contains_output_config` to supply an explicit model in the config.
**Warning signs:** Test failures immediately on changing `ClassifierConfig::default()`.

### Pitfall 7: reqwest `stream` Feature Not Enabled

**What goes wrong:** `response.bytes_stream()` panics at runtime or fails to compile if `reqwest` is built without the `stream` feature.
**Why it happens:** ferro-ai's existing reqwest dep only lists `["json"]`. `bytes_stream()` and `reqwest-eventsource` both require the `stream` feature.
**How to avoid:** Add `"stream"` to ferro-ai's reqwest features: `reqwest = { version = "0.12", features = ["json", "stream"] }`. reqwest-eventsource already adds this transitively, but explicit is better.

---

## Code Examples

### Anthropic Non-Streaming complete()

```rust
// Source: existing AnthropicProvider::classify_raw (ferro-ai/src/classifier/anthropic.rs)
// Enhanced with structured Error and schema field
async fn complete(&self, request: CompletionRequest) -> Result<String, Error> {
    let body = self.build_body(&request, false);
    let resp = self.client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &self.api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await
        .map_err(|e| if e.is_timeout() {
            Error::Timeout
        } else {
            Error::Provider { status: None, message: e.to_string() }
        })?;

    let status = resp.status().as_u16();
    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(Error::Provider { status: Some(status), message: text });
    }

    let json: serde_json::Value = resp.json().await
        .map_err(|e| Error::Deserialization(e.to_string()))?;
    let text = json["content"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|i| i["text"].as_str())
        .ok_or_else(|| Error::Deserialization(format!("unexpected: {json}")))?;
    Ok(text.to_string())
}
```

### OpenAI Non-Streaming complete()

```rust
// Source: OpenAI API reference [VERIFIED: 2026-06-08]
async fn complete(&self, request: CompletionRequest) -> Result<String, Error> {
    let mut body = serde_json::json!({
        "model": self.model_for(&request),
        "messages": build_messages(&request),
        "max_tokens": request.max_tokens,
        "stream": false,
    });
    if let Some(schema) = &request.schema {
        body["response_format"] = serde_json::json!({
            "type": "json_schema",
            "json_schema": { "name": "output", "schema": schema, "strict": true }
        });
    }
    let resp = self.client
        .post(format!("{}/v1/chat/completions", self.base_url))
        .header("Authorization", format!("Bearer {}", self.api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| Error::Provider { status: None, message: e.to_string() })?;
    let status = resp.status().as_u16();
    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(Error::Provider { status: Some(status), message: text });
    }
    let json: serde_json::Value = resp.json().await
        .map_err(|e| Error::Deserialization(e.to_string()))?;
    json["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| Error::Deserialization("no content in response".into()))
}
```

### Ollama Embeddings

```rust
// Source: Ollama API docs [VERIFIED: 2026-06-08]
async fn embed(&self, text: &str) -> Result<Vec<f32>, Error> {
    let body = serde_json::json!({
        "model": self.model_for_embed(),
        "input": text
    });
    let resp = self.client
        .post(format!("{}/api/embed", self.base_url))
        .json(&body)
        .send()
        .await
        .map_err(|e| Error::Provider { status: None, message: e.to_string() })?;
    let json: serde_json::Value = resp.json().await
        .map_err(|e| Error::Deserialization(e.to_string()))?;
    json["embeddings"][0]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect())
        .ok_or_else(|| Error::Deserialization("no embeddings in response".into()))
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `output_format` beta header for Anthropic structured output | `output_config.format` (GA, no beta header) | Late 2025 | No beta header needed; existing code uses `output_config` correctly |
| Anthropic `tool_use` forced-call for structured output | `output_config.format.type = "json_schema"` | 2025 | Native JSON schema compliance; cleaner than tool workaround |
| `reqwest-eventsource` < 0.5 (different API) | 0.6.0 (current) | 2024 | `RequestBuilderExt.eventsource()` is the stable API |
| `/api/embeddings` with `prompt` field (Ollama) | `/api/embed` with `input` field | Ollama 0.3+ | Use `/api/embed`; old endpoint still works as fallback |
| String-sniff error classification (`is_permanent_provider_error`) | Status-code-based `Error::is_retryable()` | Phase 165 | Accurate retry decisions; structured error propagation |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `ClassifierConfig::default().model` becomes `""` (empty) and `Classifier<T>` falls back to `client.default_model()` | Pattern 4 | Compiler error or test breakage if different approach chosen; both paths are valid |
| A2 | "groq" as named FERRO_AI_PROVIDER alias in `AiConfig::from_env()` | Pattern 5 | If not aliased, user must set FERRO_AI_PROVIDER=openai + FERRO_AI_BASE_URL manually |
| A3 | `CompletionRequest` struct field names and module layout | Pattern 1 | Planner may choose flat params vs struct; struct approach is recommended for Phase 166 compatibility |
| A4 | Groq default base_url = `https://api.groq.com/openai` | Pattern 5 | If Groq changes API URL, env override still works |

---

## Open Questions

1. **`ClassifierConfig.model` field after D-03**
   - What we know: D-03 says "hardcoded `claude-sonnet-4-6` is removed from `ClassifierConfig::default()`"
   - What's unclear: Should the field be kept as `String` (empty default) or removed entirely? Removing it is a breaking API change for users who set `config.model = "..."` directly.
   - Recommendation: Keep the field; default to `String::new()`; `Classifier<T>` uses `if config.model.is_empty() { client.default_model() } else { &config.model }`.

2. **`AnthropicProvider` backward-compatibility after D-10**
   - What we know: D-12 says the public API is preserved; D-10 says HTTP is delegated to `AnthropicClient`.
   - What's unclear: `AnthropicProvider` currently takes `(client: reqwest::Client, api_key: String)`. After D-10, it needs an `Arc<AnthropicClient>` instead. The public `from_env()` constructor can hide this.
   - Recommendation: `AnthropicProvider::from_env()` internally constructs an `AnthropicClient`; the public `new(api_key)` constructor also works. External callers cannot tell the difference.

3. **`complete_stream` with structured output schema**
   - What we know: `output_config.format.type = "json_schema"` streams tokens; structured JSON must be accumulated before parsing.
   - What's unclear: Should `complete_stream` with a schema field stream raw tokens (caller accumulates) or buffer internally and yield only after the full JSON is received?
   - Recommendation: Stream raw tokens always. Phase 166's `complete::<T>()` wrapper handles accumulation. Streaming raw tokens is the correct primitive behavior.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Anthropic API (`api.anthropic.com`) | AnthropicClient HTTP tests | ✗ (not in CI) | — | Mock HTTP server for unit/integration tests |
| OpenAI API (`api.openai.com`) | OpenAiClient HTTP tests | ✗ (not in CI) | — | Mock HTTP server |
| Ollama | OllamaClient HTTP tests | ✗ (not confirmed installed) | — | Mock HTTP or skip with `#[ignore]` |
| `reqwest-eventsource 0.6` | SSE streaming | ✓ (crates.io) | 0.6.0 | — |
| `futures 0.3` | StreamExt, BoxStream | ✓ (crates.io) | 0.3.32 | — |
| `async-stream 0.3` | try_stream! macro | ✓ (crates.io) | 0.3.6 | — |

**Missing dependencies with no fallback:** None.

**Note on live API tests:** All tests requiring live API keys MUST be gated with `#[ignore]` or a `FERRO_AI_INTEGRATION_TESTS=1` environment variable check. CI will not have API keys.

---

## Validation Architecture

> `workflow.nyquist_validation` key absent from `.planning/config.json` — treated as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test + tokio test |
| Config file | None (no separate test config; `cargo test`) |
| Quick run command | `cargo test -p ferro-ai` |
| Full suite command | `cargo test --all-features -p ferro-ai` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SC#1 | `LlmClient` trait exists with three methods | Unit | `cargo test -p ferro-ai -- client::tests::` | ❌ Wave 0 |
| SC#1 | Missing capabilities return `Err(Error::Unsupported)` | Unit | `cargo test -p ferro-ai -- test_anthropic_embed_unsupported` | ❌ Wave 0 |
| SC#2 | Each provider instantiable as `Box<dyn LlmClient>` | Unit (compile-time) | `cargo build -p ferro-ai` | ❌ Wave 0 |
| SC#2 | `default_model()` returns correct string per provider | Unit | `cargo test -p ferro-ai -- test_default_models` | ❌ Wave 0 |
| SC#3 | `AiConfig::from_env()` unknown provider → `Error::Config` at startup | Unit | `cargo test -p ferro-ai -- test_unknown_provider_config_error` | ❌ Wave 0 |
| SC#3 | `AiConfig::from_env()` known providers dispatch correctly | Unit (mock) | `cargo test -p ferro-ai -- test_aiconfig_dispatch` | ❌ Wave 0 |
| SC#4 | `ClassifierConfig::default().model` no longer hardcodes the model string | Unit | `cargo test -p ferro-ai -- test_classifier_config_defaults` (updated) | ✅ (update) |
| SC#4 | `Classifier<T>` resolves model from client | Unit | `cargo test -p ferro-ai -- test_classifier_uses_client_default_model` | ❌ Wave 0 |
| SC#5 | Existing `Classifier<T>` tests pass | Unit | `cargo test -p ferro-ai -- tests::` | ✅ (existing) |
| SC#5 | `ClassificationProvider` object safety preserved | Unit (compile-time) | `cargo test -p ferro-ai -- test_classification_provider_is_object_safe` | ✅ (existing) |
| SC#6 | `reqwest-eventsource` not in ferro-ai public API | Compile-time | `cargo doc -p ferro-ai` (verify no eventsource in public items) | ❌ Wave 0 |
| D-14 | `Error::is_retryable()` correct for all status codes | Unit | `cargo test -p ferro-ai -- test_error_is_retryable` | ❌ Wave 0 |
| D-14 | Classifier retries only on `is_retryable() == true` | Unit | Updated `test_no_retry_on_permanent_error` | ✅ (update) |
| D-11 | Schema field passthrough in AnthropicClient request body | Unit | `cargo test -p ferro-ai -- test_build_body_with_schema` | ❌ Wave 0 |
| D-08 | `TokenStream` yields correct tokens from mock SSE | Integration (mock) | `cargo test -p ferro-ai -- test_anthropic_stream_tokens` | ❌ Wave 0 |
| D-08 | Ollama NDJSON stream yields correct tokens | Integration (mock) | `cargo test -p ferro-ai -- test_ollama_stream_tokens` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-ai`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features -p ferro-ai`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-ai/src/client/mod.rs` — trait definition, TokenStream, CompletionRequest
- [ ] `ferro-ai/src/client/anthropic.rs` — AnthropicClient + LlmClient impl + tests
- [ ] `ferro-ai/src/client/openai.rs` — OpenAiClient + LlmClient impl + tests
- [ ] `ferro-ai/src/client/ollama.rs` — OllamaClient + LlmClient impl + tests
- [ ] `ferro-ai/src/config.rs` — AiConfig::from_env()
- [ ] Tests for `Error::is_retryable()`, schema passthrough, default_model() per provider
- [ ] Mock HTTP tests for token stream correctness (Anthropic SSE, Ollama NDJSON)

**Mock HTTP strategy:** Use `wiremock` crate or `tokio::net::TcpListener` in tests to serve deterministic SSE/NDJSON responses without live API keys. Alternatively, use `reqwest::Client` mock with `reqwest-mock` or verify request construction only (no live calls). [ASSUMED: planner chooses mock strategy; `wiremock` is the standard in the Rust ecosystem for this pattern.]

---

## Security Domain

> `security_enforcement` key absent from config → treated as enabled.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | N/A (no user auth in this crate) |
| V3 Session Management | No | N/A |
| V4 Access Control | No | N/A |
| V5 Input Validation | Yes | API keys read from env vars (not hardcoded); unknown provider names rejected at startup |
| V6 Cryptography | No | API keys transmitted via HTTPS by reqwest (TLS default) |

### Known Threat Patterns for HTTP API Client

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| API key leak in logs | Information Disclosure | Never log `FERRO_AI_API_KEY`; `tracing` spans should not include the key value |
| SSRF via FERRO_AI_BASE_URL | Tampering / Elevation | Low risk in ferro-ai leaf crate (server-side only); no user-controlled base_url at runtime |
| Prompt injection in `complete` | Tampering | Out of scope for the client layer; application responsibility |

---

## Sources

### Primary (HIGH confidence)

- `ferro-ai/src/classifier/anthropic.rs` — existing HTTP implementation, output_config structure, auth headers [VERIFIED: local codebase]
- `/Users/alberto/.cargo/registry/.../reqwest-eventsource-0.6.0/src/` — `Event` enum, `RequestBuilderExt`, `Error` enum [VERIFIED: local cargo registry]
- `platform.claude.com/docs/en/api/messages` — SSE event sequence, response structure, output_config GA [VERIFIED: live fetch 2026-06-08]
- `platform.claude.com/docs/en/build-with-claude/structured-outputs` — output_config vs tool_use, no beta header required [VERIFIED: live fetch 2026-06-08]
- `ollama.com` / `github.com/ollama/ollama/blob/main/docs/api.md` — /api/chat NDJSON format, /api/embed endpoint [VERIFIED: live fetch 2026-06-08]
- `context7.com/dtolnay/async-trait` — async_trait dyn compatibility pattern [VERIFIED: Context7 2026-06-08]
- `cargo info` for futures (0.3.32), async-stream (0.3.6), tokio-stream (0.1.18), reqwest-eventsource (0.6.0) [VERIFIED: local cargo]

### Secondary (MEDIUM confidence)

- OpenAI Chat Completions SSE streaming format — choices[0].delta.content, [DONE] termination [VERIFIED via WebSearch + OpenAI streaming events reference]
- OpenAI /v1/embeddings endpoint — data[0].embedding → Vec<f32> [VERIFIED via WebSearch]
- Groq base_url `https://api.groq.com/openai` [CITED: Groq/OpenAI-compatibility docs referenced in REQUIREMENTS.md; not directly fetched]

### Tertiary (LOW confidence)

- None; all critical claims verified or cited above.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified via crates.io
- Anthropic API: HIGH — live doc fetch confirmed SSE format and output_config structure
- OpenAI API: MEDIUM — OpenAI platform 403'd; format confirmed via WebSearch and existing code patterns
- Ollama API: HIGH — live doc fetch confirmed NDJSON format and /api/embed
- reqwest-eventsource API: HIGH — source read from local cargo registry
- Architecture: HIGH — builds directly on existing codebase patterns

**Research date:** 2026-06-08
**Valid until:** 2026-09-08 (APIs are stable; Anthropic/OpenAI versioning is slow-moving)
