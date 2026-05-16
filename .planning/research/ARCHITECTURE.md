# Architecture: ferro v12.1 AI Milestone

**Domain:** Multi-provider LLM SDK + AI-assisted scaffolding CLI
**Researched:** 2026-05-15
**Confidence:** HIGH — based on direct codebase reading

---

## Component Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         USER / AGENT                                │
│               ferro ai:make "..."   ferro ai:explain ...            │
└──────────────────────────────┬──────────────────────────────────────┘
                               │ invokes
┌──────────────────────────────▼──────────────────────────────────────┐
│                         ferro-cli                                   │
│  src/commands/ai_make.rs        src/commands/ai_explain.rs          │
│  src/commands/make_json_view.rs (modified: use SDK instead of ai.rs)│
│                                                                     │
│  src/ai.rs  <-- REPLACED by direct ferro-ai SDK calls              │
└────────────────────┬────────────────────────────────────────────────┘
                     │ depends on
┌────────────────────▼────────────────────────────────────────────────┐
│                         ferro-ai  (EXPANDED)                        │
│                                                                     │
│  src/client/          <- NEW: provider-agnostic LLM client          │
│    mod.rs             (LlmClient trait)                             │
│    anthropic.rs       (existing classify_raw promoted here)         │
│    openai.rs          <- NEW                                        │
│    groq.rs            <- NEW (reuses openai wire format)            │
│    ollama.rs          <- NEW                                        │
│    config.rs          <- NEW: AiConfig (from_env, provider select)  │
│                                                                     │
│  src/complete.rs      <- NEW: ferro_ai::complete::<T>() entry point │
│  src/tools.rs         <- NEW: tool-calling register & dispatch      │
│  src/embeddings.rs    <- NEW: embed() + cosine_similarity()         │
│  src/stream.rs        <- NEW: TokenStream (async stream of tokens)  │
│                                                                     │
│  src/classifier/      <- KEPT unchanged                             │
│  src/confirmation/    <- KEPT unchanged                             │
│  src/error.rs         <- MODIFIED: new Stream/Tool/Embed variants   │
└────────────────────┬────────────────────────────────────────────────┘
                     │ introspection context via
┌────────────────────▼────────────────────────────────────────────────┐
│                         ferro-mcp                                   │
│                                                                     │
│  src/tools/ai.rs       <- MODIFIED: add ai_scaffold, ai_explain     │
│                           MCP tools calling the new SDK             │
│  (existing generation_context, list_routes, list_models unchanged)  │
└────────────────────┬────────────────────────────────────────────────┘
                     │ SSE response via
┌────────────────────▼────────────────────────────────────────────────┐
│                    framework (ferro-rs)  (MODIFIED)                 │
│                                                                     │
│  src/http/sse.rs        <- NEW: SseEvent, SseStream                 │
│  src/http/response.rs   <- MODIFIED: HttpResponse::sse() factory    │
│  src/lib.rs             <- MODIFIED: re-export SseStream, SseEvent  │
│  Cargo.toml [ai] feature <- already optional; unchanged             │
└────────────────────┬────────────────────────────────────────────────┘
                     │ renders streaming text via
┌────────────────────▼────────────────────────────────────────────────┐
│                    ferro-json-ui  (MODIFIED)                        │
│                                                                     │
│  src/components/stream_text.rs  <- NEW: StreamText component        │
│    renders <div data-ferro-stream-url="..."> + inline EventSource   │
│  src/render.rs          <- MODIFIED: handle StreamText variant      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## New vs Modified Components

### NEW — ferro-ai modules

| Module | Path | What it adds |
|--------|------|--------------|
| `LlmClient` trait | `ferro-ai/src/client/mod.rs` | Provider-agnostic `complete_raw`, `complete_structured`, `stream_raw`, `embed_raw` |
| `AnthropicClient` | `ferro-ai/src/client/anthropic.rs` | Promote existing reqwest logic; add streaming and plain completions |
| `OpenAiClient` | `ferro-ai/src/client/openai.rs` | OpenAI-compatible provider |
| `GroqClient` | `ferro-ai/src/client/groq.rs` | Groq via OpenAI wire format with different base URL |
| `OllamaClient` | `ferro-ai/src/client/ollama.rs` | Ollama local provider |
| `AiConfig` | `ferro-ai/src/client/config.rs` | `from_env()` reads `FERRO_AI_PROVIDER`; returns `Box<dyn LlmClient>` |
| `complete::<T>()` | `ferro-ai/src/complete.rs` | One-shot typed completion via JSON Schema; wraps `LlmClient` |
| `Tool` + `ToolRegistry` | `ferro-ai/src/tools.rs` | Register Rust fns as AI tools; dispatch tool-use calls from provider response |
| `embed()` + `cosine_similarity()` | `ferro-ai/src/embeddings.rs` | Embedding helpers; optional `pgvector` cargo feature |
| `TokenStream` | `ferro-ai/src/stream.rs` | `Pin<Box<dyn Stream<Item=Result<String, Error>>>>` from streaming provider |

### NEW — ferro-cli commands

| Command | File | Depends on |
|---------|------|------------|
| `ferro ai:make <description>` | `ferro-cli/src/commands/ai_make.rs` | `ferro-ai` SDK + ferro-mcp context fns in-process |
| `ferro ai:explain <route\|model>` | `ferro-cli/src/commands/ai_explain.rs` | `ferro-ai` SDK + file scanning |

### NEW — framework

| Component | Path | What it adds |
|-----------|------|--------------|
| `SseEvent` | `framework/src/http/sse.rs` | Single SSE frame with `data:`, optional `id:`, optional `event:` |
| `SseStream` | `framework/src/http/sse.rs` | `mpsc::Sender` + `StreamBody`-backed hyper response |
| `HttpResponse::sse()` factory | `framework/src/http/response.rs` | Returns `(SseStream, SseSender)` pair |

### NEW — ferro-json-ui

| Component | Path | What it adds |
|-----------|------|--------------|
| `StreamText` variant | `ferro-json-ui/src/components/stream_text.rs` | Emits `<div>` with `data-ferro-stream-url` attribute and inline `EventSource` JS snippet |

### MODIFIED — existing components

| Component | Change |
|-----------|--------|
| `ferro-ai/src/error.rs` | Add `Stream`, `Tool`, `Embed` error variants |
| `ferro-ai/src/classifier/anthropic.rs` | Extract shared HTTP logic into `client/anthropic.rs`; `classify_raw` delegates to new client |
| `ferro-ai/src/lib.rs` | Export new public surface: `complete`, `LlmClient`, `AiConfig`, `TokenStream`, `embed`, `Tool` |
| `ferro-ai/Cargo.toml` | Add `tokio-stream` for `AsyncStream`; optional `pgvector` feature |
| `framework/src/lib.rs` | Re-export `SseStream`, `SseEvent` under `ai` feature flag |
| `ferro-cli/src/ai.rs` | Delete and replace with `ferro-ai` SDK calls |
| `ferro-cli/Cargo.toml` | Add `ferro-ai = { path = "../ferro-ai", version = "0.2" }` (currently absent) |
| `ferro-cli/src/commands/make_json_view.rs` | Replace `ai::call_anthropic` with `ferro_ai::complete::<String>()` |
| `ferro-cli/src/commands/mod.rs` | Register `ai_make`, `ai_explain` |
| `ferro-cli/src/main.rs` | Add `AiMake`, `AiExplain` variants to `Commands` enum |
| `ferro-mcp/src/tools/ai.rs` | Add `ai_scaffold` and `ai_explain` MCP tool implementations |
| `ferro-json-ui/src/components/` | Add `StreamText` to the `Component` enum |
| `ferro-json-ui/src/render.rs` | Handle `StreamText` variant in HTML renderer |

---

## Data Flow Descriptions

### Flow 1: `ferro ai:make "a product catalog with filters"`

```
1. ferro-cli/ai_make.rs
   |-- Loads .env (dotenvy — already used in existing CLI commands)
   |-- Calls ferro-mcp context builders in-process:
   |     generation_context::execute()        -> naming conventions, patterns
   |     list_routes::execute(project_root)   -> existing route list
   |     list_models::execute(project_root)   -> existing model shapes
   |     application_info::execute(...)       -> app name, installed crates
   |-- Assembles system prompt: context + description
   `-- Calls ferro_ai::complete::<ScaffoldPlan>(system, user)
         |
2. ferro-ai/complete.rs
   |-- AiConfig::from_env() selects provider
   |-- provider.complete_structured(system, user, json_schema::<ScaffoldPlan>())
   `-- Returns ScaffoldPlan { handlers, model, routes, view }
         |
3. ferro-cli/ai_make.rs
   |-- Interprets ScaffoldPlan
   |-- Calls existing scaffold helpers (same functions as make:scaffold):
   |     generate_migration(), generate_model(), generate_controller()
   |     make_json_view::run() for the view
   `-- Writes files, prints summary
```

### Flow 2: `ferro ai:explain users/show`

```
1. ferro-cli/ai_explain.rs
   |-- Resolves route or model identifier to a source file path
   |-- Reads source content
   |-- Calls ferro_ai::complete::<String>(explain_system_prompt, source_code)
         |
2. ferro-ai/complete.rs -> plain text explanation string
         |
3. ferro-cli/ai_explain.rs
   `-- Prints explanation to stdout
```

### Flow 3: SSE streaming in an application handler

```
Application handler (written by developer or generated by ai:make):

  pub async fn chat_stream(req: Request) -> SseResponse {
      let (sse, tx) = SseStream::new();
      tokio::spawn(async move {
          let mut tokens = ferro_ai::stream(system, user, AiConfig::from_env()?).await?;
          while let Some(token) = tokens.next().await {
              tx.send(SseEvent::data(token?)).await.ok();
          }
      });
      Ok(sse.into_response())
  }

Browser side:
  <div data-ferro-stream-url="/chat/stream">Loading...</div>
  (rendered by StreamText component in a JSON-UI spec)
  |
  EventSource("/chat/stream")
  |-- receives: data: <token>\n\n
  `-- appends tokens to element textContent
```

### Flow 4: `ferro make:json-view` improvement

```
Current path:
  ferro-cli/src/ai.rs::call_anthropic()
    -> blocking reqwest::Client (not reusable, not provider-agnostic)

After v12.1:
  ferro_ai::complete::<String>(system, user)
    -> AiConfig::from_env() (provider-agnostic, respects FERRO_AI_PROVIDER)
    -> same result, no behavioral change for users
```

---

## Integration Points

### ferro-cli depends on ferro-ai (gap to close)

ferro-cli currently has its own AI client in `src/ai.rs`: a blocking `reqwest::Client`, hardcoded to Anthropic, with no retry logic and no reuse across commands. This is a direct duplicate of part of `ferro-ai`. The integration point:

- Add `ferro-ai = { path = "../ferro-ai", version = "0.2" }` to `ferro-cli/Cargo.toml`
- Delete `ferro-cli/src/ai.rs` and replace call sites with `ferro_ai::complete::<T>()` and `AiConfig::from_env()`
- `ferro-cli` already uses `tokio = { features = ["full"] }`, so async usage requires no new dep

### ferro-cli reads ferro-mcp context in-process

`ferro-cli/Cargo.toml` already lists `ferro-mcp = { path = "../ferro-mcp" }`. The `ai:make` command can call the same context-gathering functions that ferro-mcp exposes for its MCP tools. These functions accept `project_root: &Path` and return serializable structs — no MCP transport, no subprocess.

Functions called in-process by `ai:make`:
- `ferro_mcp::tools::generation_context::execute()`
- `ferro_mcp::tools::list_routes::execute(project_root)`
- `ferro_mcp::tools::list_models::execute(project_root)`
- `ferro_mcp::tools::application_info::execute(project_root, config)`

### ClassificationProvider is kept alongside LlmClient

`ClassificationProvider` covers structured-output classification only. `LlmClient` is broader (plain text, streaming, embeddings). The two coexist: `AnthropicProvider` (and any future provider) implements both traits. `Classifier<T>` continues to work unchanged by calling `ClassificationProvider::classify_raw`, which the new `AnthropicClient` delegates to `complete_structured`. No callers of the existing API break.

### Framework SSE has no dependency on ferro-ai

`SseStream` is a pure HTTP primitive: `mpsc::Sender<SseEvent>` + a `StreamBody` hyper response. It does not depend on `ferro-ai`. The wiring of `TokenStream` into `SseStream` is done inside application handler code, not inside either library. This keeps both crates decoupled and separately testable.

### AiConfig provider selection

```
FERRO_AI_PROVIDER=anthropic   -> AnthropicClient (default when unset)
FERRO_AI_PROVIDER=openai      -> OpenAiClient     (reads OPENAI_API_KEY)
FERRO_AI_PROVIDER=groq        -> GroqClient       (reads GROQ_API_KEY, OpenAI wire)
FERRO_AI_PROVIDER=ollama      -> OllamaClient     (reads OLLAMA_BASE_URL)
FERRO_AI_MODEL=<model-id>     -> overrides default model for selected provider
```

`AiConfig::from_env()` returns `Box<dyn LlmClient>`. Call sites do not branch on provider. This follows the pattern of `StorageConfig::from_env()` in ferro-storage and `CacheConfig::from_env()` in ferro-cache.

---

## Build Order

Dependencies flow in one direction: ferro-ai is a leaf crate; framework, ferro-cli, ferro-mcp all depend on it. The correct build order is:

```
Wave 1 — ferro-ai expansion (no dependents change yet)
  Step 1: LlmClient trait + provider modules
          [ferro-ai/src/client/]
          Rationale: All subsequent steps depend on this trait.
          Risk: Refactoring AnthropicProvider must not break the
                existing ClassificationProvider usage.

  Step 2: complete::<T>(), tools.rs, embeddings.rs
          [ferro-ai/src/complete.rs, tools.rs, embeddings.rs]
          Rationale: Builds on LlmClient. Fully testable in isolation
                     before any consumer crate changes.

  Step 3: TokenStream (stream.rs)
          [ferro-ai/src/stream.rs]
          Rationale: Streaming requires careful async cancellation
                     handling. Separate step to keep it isolated.
                     Consumers (framework SSE, CLI) depend on the
                     type shape established here.

Wave 2 — framework SSE (depends only on hyper/mpsc, not on ferro-ai)
  Step 4: SseEvent + SseStream
          [framework/src/http/sse.rs]
          Rationale: Can be built and tested before Wave 1 is complete
                     because it has no ferro-ai dependency. Delivers
                     streaming response infrastructure independently.

Wave 3 — ferro-json-ui StreamText (depends on SSE URL convention)
  Step 5: StreamText component + renderer
          [ferro-json-ui/src/components/stream_text.rs + render.rs]
          Rationale: The URL attribute convention is established by
                     Step 4. This step is a pure JSON-UI concern.

Wave 4 — ferro-cli commands (depends on Waves 1-3)
  Step 6: Replace src/ai.rs; wire ferro-ai SDK into make:json-view
          Rationale: Validates the SDK against an existing workflow
                     before building new commands on top.
                     Reduces risk for Steps 7-8.

  Step 7: ai:make command
          Rationale: The primary CLI command; highest value.
                     Uses ferro-mcp in-process context + SDK.
                     Depends on Step 6 (SDK wired in).

  Step 8: ai:explain command
          Rationale: Simpler than ai:make. Can be developed in
                     parallel with Step 7 but has no dependency on it.

  Step 9: Improved make:json-view (v2 spec + ServiceDef introspection)
          Rationale: Depends on v12.0 (JSON-UI v2) having shipped.
                     Listed last to clarify the ordering constraint
                     relative to v12.0.

Wave 5 — ferro-mcp tools (depends on Wave 4 being proven)
  Step 10: ai_scaffold and ai_explain MCP tools
           Rationale: MCP tools are thin wrappers over the same logic
                      as the CLI commands. Come last because the CLI
                      validates the underlying functions first.
```

---

## Key Architecture Decisions

| Decision | Rationale |
|----------|-----------|
| New `LlmClient` trait alongside `ClassificationProvider` | `ClassificationProvider` is narrow (structured JSON only); `LlmClient` is broader. Keeping both avoids a breaking change to existing `Classifier<T>` callers. |
| `AiConfig::from_env()` returns `Box<dyn LlmClient>` | Matches the provider-selection pattern in ferro-storage and ferro-cache. Call sites stay provider-agnostic. |
| Delete `ferro-cli/src/ai.rs` rather than wrap it | It is a partial duplicate of AnthropicClient in ferro-ai. Pre-1.0, there is no backward compatibility constraint. Deletion is cleaner than maintaining two implementations. |
| SSE as a framework HTTP primitive, not a ferro-ai type | `SseStream` is an HTTP concern. `TokenStream` is a Rust async stream. The conversion happens in the application handler. Neither crate depends on the other. |
| Groq shares OpenAI wire format | Groq's API accepts OpenAI-format requests with a different base URL and `Authorization: Bearer` header. `GroqClient` composes `OpenAiClient` with config overrides rather than duplicating HTTP logic. |
| Tool calling dispatch in ferro-ai | Tool dispatch is a general SDK capability used in application handlers as well as the CLI. Keeping it in ferro-ai ensures one implementation. |
| `ai:make` reads ferro-mcp in-process | ferro-cli already depends on ferro-mcp. Direct function calls are faster, simpler, and have no transport overhead compared to spawning a subprocess. |
| Introspection context is assembled before the LLM call | The system prompt (conventions, routes, models) is built entirely from file-system reads and serialized to a string before any API call. This makes the prompt deterministic and testable. |

---

## What Does Not Change

- `ClassificationProvider` and `Classifier<T>` — kept, not deleted
- `InMemoryConfirmationStore` and `ConfirmationStore` — unchanged
- `framework/Cargo.toml` `ai` feature flag — remains optional
- The `ferro-mcp` MCP server binary and its existing 35+ tools
- `ferro-cli/src/commands/make_scaffold.rs` — `ai:make` calls its helpers, does not replace it
- All Inertia/React-based scaffolding paths — unrelated to this milestone
- `ferro-ai` crate name and crates.io identity

---

## Sources

All findings from direct reading of the codebase:

- `ferro-ai/src/lib.rs`, `ferro-ai/src/classifier/`, `ferro-ai/src/error.rs`, `ferro-ai/Cargo.toml`
- `ferro-cli/src/ai.rs`, `ferro-cli/Cargo.toml`, `ferro-cli/src/main.rs`, `ferro-cli/src/commands/mod.rs`
- `ferro-cli/src/commands/make_json_view.rs`, `ferro-cli/src/commands/make_scaffold.rs`
- `ferro-mcp/src/tools/ai.rs`, `ferro-mcp/src/tools/generation_context.rs`
- `framework/Cargo.toml`, `framework/src/http/response.rs`, `framework/src/websocket.rs`, `framework/src/lib.rs`
- `.planning/PROJECT.md` (v12.1 target features section)
