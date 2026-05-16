# Stack Research — v12.1 AI Milestone

**Project:** ferro-ai SDK expansion + AI-assisted scaffolding
**Researched:** 2026-05-15
**Confidence:** HIGH (versions verified against crates.io / docs.rs)

---

## Context

`ferro-ai` already exists with:
- `AnthropicProvider` — hand-rolled `reqwest` calls to `POST /v1/messages`
- `Classifier<T>` — structured JSON output via `output_config.format.type = "json_schema"`
- `InMemoryConfirmationStore` — TTL-gated confirmation state machine
- No streaming, no tool calling, no embeddings, no OpenAI/Groq/Ollama providers

The framework uses `hyper 1.x` directly (not axum). SSE must be implemented against hyper's
`http_body_util` + `futures-util` streaming body model, not axum's `Sse<>` wrapper.

---

## Capability 1 — Multi-Provider LLM Client

### Decision: stay hand-rolled per provider, do not adopt a multi-provider crate

`genai 0.5.3` (the best multi-provider option) does not support tool calling or embeddings as of May 2026 — both are roadmap items. Adopting a crate that lacks two of the four required capabilities forces a crate swap later or parallel dual-implementation.

`async-openai 0.38.2` covers OpenAI fully (streaming, tools, embeddings) but is OpenAI-only. It provides no Anthropic or Ollama support.

The existing `AnthropicProvider` pattern — implement `ClassificationProvider` per provider, use `reqwest` for HTTP — is correct. Extend it to `LlmProvider` (or rename the trait) with the three remaining providers.

Groq's API is OpenAI-compatible (`https://api.groq.com/openai/v1`). The OpenAI provider implementation repoints its base URL from an env var; Groq is a config variant, not a separate provider struct.

| Crate | Version | Purpose | Why |
|-------|---------|---------|-----|
| `reqwest` | `0.12` | HTTP client for all provider calls | Already in `ferro-ai`; no new dep |
| `serde` + `serde_json` | `1` | Request/response serialization | Already in `ferro-ai` |
| `async-trait` | `0.1` | Provider trait object safety | Already in `ferro-ai` |

**No new crates needed for multi-provider.** Implement `OllamaProvider`, `OpenAiProvider` (reused for Groq via base URL config) in `ferro-ai/src/providers/`.

### Provider–Capability Matrix

| Provider | Structured Output | Tool Calling | Embeddings | Streaming |
|----------|-------------------|--------------|------------|-----------|
| Anthropic | `output_config.format.json_schema` | Tool-use blocks | No (use OpenAI) | SSE event stream |
| OpenAI | `response_format.json_schema` | `tools` array | Yes (`/v1/embeddings`) | SSE `data: [DONE]` |
| Groq | Same as OpenAI, base URL override | Yes (OpenAI-compat) | Yes | Same as OpenAI |
| Ollama | `format: "json"` + schema | Yes (OpenAI-compat format) | Yes (`/api/embed`) | NDJSON stream |

---

## Capability 2 — Structured Outputs (`ferro_ai::complete::<T>()`)

The existing `Classifier<T>` already does this. The work is:
1. Expose a top-level `complete::<T>()` free function as ergonomic API
2. Generate the JSON Schema from `T: JsonSchema` at call site
3. Deserialize the provider response into `T` via `serde_json::from_value`

| Crate | Version | Purpose | Why |
|-------|---------|---------|-----|
| `schemars` | `1` | `#[derive(JsonSchema)]` → JSON Schema value | Already in `ferro-json-ui` and `ferro-projections`; workspace-consistent |

`schemars 1.2.1` is current (2026). It targets JSON Schema 2020-12. Anthropic, OpenAI, and Groq all accept JSON Schema in their structured-output endpoints. No alternative considered — `schemars` is the unambiguous standard in the Rust ecosystem.

**No new crates.** `schemars` is already a workspace dependency.

---

## Capability 3 — Tool / Function Calling

Tool calling requires:
- A `ToolDef` struct (name, description, JSON Schema for parameters)
- A `ToolHandler` trait or closure registry
- Provider-specific serialization (Anthropic tool-use blocks vs OpenAI `tools` array)
- Auto-dispatch loop: call LLM → if tool-use response, invoke handler → feed result back → repeat until text response

| Crate | Version | Purpose | Why |
|-------|---------|---------|-----|
| `schemars` | `1` | Parameter schema for `ToolDef` | Same as structured outputs |
| `serde_json` | `1` | Tool result serialization | Already in `ferro-ai` |

No dedicated "tool calling framework" crate is needed. The dispatch loop is ~50 lines; a crate would add an opaque layer between `ferro-ai` and the provider API shapes that differ per provider. Build it in `ferro-ai/src/tools.rs`.

---

## Capability 4 — Embeddings + Cosine Similarity + Optional pgvector

### Embeddings

Add `embed(text: &str) -> Vec<f32>` and `embed_batch(texts: &[&str]) -> Vec<Vec<f32>>` to the `LlmProvider` trait. Providers that lack embeddings (Anthropic) return `Err(Error::Unsupported)`.

No new crates. `reqwest` + `serde_json` is sufficient.

### Cosine Similarity

Cosine similarity between two `Vec<f32>` is a dot product divided by magnitudes — 10 lines of Rust. No crate needed. Expose as `ferro_ai::similarity::cosine(a: &[f32], b: &[f32]) -> f32`.

Do not pull in `ndarray` or `nalgebra` for a scalar function.

### pgvector

`pgvector` supports `sqlx` (which SeaORM is built on) but **SeaORM does not have native pgvector column support**. Integration requires raw SQL via `sea_orm::Statement` or a thin `sqlx` bypass layer.

| Crate | Version | Purpose | Why | Gate |
|-------|---------|---------|-----|------|
| `pgvector` | `0.4` | `Vector` type, `<=>` operator for Postgres | Official pgvector Rust client; supports sqlx 0.8 which SeaORM 1.x uses | `feature = "pgvector"` |

`pgvector 0.4.1` is current. Enable with `pgvector = { version = "0.4", optional = true, features = ["sqlx"] }` in `ferro-ai/Cargo.toml`.

The `pgvector` integration is a **thin helper module** (`ferro_ai::vector_store::PgVectorStore`) providing:
- `insert(key: &str, embedding: &[f32])` — stores vector via raw sqlx
- `search(embedding: &[f32], limit: usize) -> Vec<(String, f32)>` — cosine similarity query using `<=>` operator

This is not an ORM integration; it is a direct sqlx query wrapper. Keeps it maintainable when SeaORM evolves.

---

## Capability 5 — SSE Streaming from Handlers

### Framework context: hyper 1.x, not axum

Ferro uses `hyper 1.x` directly with its own `Request`/`Response` wrappers. Axum's `Sse<>` type is not available. SSE must be built against hyper's streaming body model.

Pattern: handler creates a `tokio::sync::mpsc` channel, spawns a task that calls the provider's streaming endpoint and sends tokens, returns a hyper `Response` whose body is a `StreamBody` that reads from the channel receiver.

| Crate | Version | Purpose | Why |
|-------|---------|---------|-----|
| `tokio` | `1` | `mpsc` channel, task spawn | Already in workspace |
| `futures-util` | `0.3` | `stream::unfold` / `StreamExt` for channel-to-stream | Already in `framework` and `ferro-broadcast` |
| `http-body-util` | `0.1` | `StreamBody` for hyper 1.x streaming response | Sibling of `hyper 1.x`; already in `hyper-util` dependency chain |
| `bytes` | `1` | `Bytes` chunks for SSE frames | Already in workspace (hyper 1.x dep) |

No `async-stream` or `tokio-stream` needed — `futures-util::stream::unfold` over a channel receiver is sufficient and avoids adding stream combinators for a single use case.

The SSE response format is plain text (`text/event-stream`), so the body is `data: <token>\n\n` byte chunks. No SSE-specific crate needed.

### LLM-side streaming (consuming provider SSE)

Anthropic and OpenAI stream tokens as SSE events. The provider implementations need to parse incoming `text/event-stream` responses.

| Crate | Version | Purpose | Why |
|-------|---------|---------|-----|
| `reqwest-eventsource` | `0.6` | Parse incoming SSE from LLM provider APIs | Used by `async-openai` and `genai`; correct abstraction layer |

`reqwest-eventsource 0.6.0` is current. It wraps a `reqwest::Response` and yields `Event` items. Anthropic's SSE format uses `event:` + `data:` lines; OpenAI uses `data:` only with `[DONE]` sentinel. Both are standard SSE and parsed correctly by this crate.

Ollama streams NDJSON (not SSE). For Ollama streaming, parse `response.bytes_stream()` directly — one JSON object per line.

---

## Integration Points with Existing ferro-ai

| New Capability | Integration Point | Notes |
|----------------|-------------------|-------|
| Multi-provider | Rename/extend `ClassificationProvider` → `LlmProvider` trait | Breaking but pre-1.0; `Classifier<T>` re-implemented on top |
| Structured outputs | `complete::<T>()` calls existing `classify_raw()` or new `complete_raw()` | `schemars` schema generation at call site |
| Tool calling | New `ToolRegistry` + dispatch loop in `ferro-ai/src/tools.rs` | Provider-agnostic; delegates to `LlmProvider::complete_raw()` |
| Embeddings | New method on `LlmProvider` trait | Providers that lack it return `Err(Error::Unsupported)` |
| Cosine similarity | `ferro_ai::similarity` module, pure Rust | No deps |
| pgvector | `ferro_ai::vector_store` module, feature-gated | Requires `sqlx` in scope via SeaORM |
| SSE streaming | New `ferro_ai::stream` module; handlers use `StreamBody` | Framework `Response` type must accept streaming body |
| LLM-side SSE | `reqwest-eventsource` in provider impls | Per-provider streaming adapters |

### ferro-cli integration

`ferro ai:make` and `ferro ai:explain` call `ferro-ai` through the same `LlmProvider` trait the SDK exposes. The CLI uses the blocking-compatible path (wrap in `tokio::runtime::Handle::current().block_on()`). No separate CLI AI client.

### ferro-json-ui integration

`ferro make:json-view` currently uses a hand-rolled Anthropic call in `ferro-cli/src/ai.rs`. Post-v12.1, it migrates to `ferro_ai::complete::<JsonUiSpec>()` using the structured-output path with a `JsonSchema`-derived spec type.

---

## Do Not Add

| What | Why Not |
|------|---------|
| `genai` crate | Missing tool calling and embeddings as of 0.5.3; adds a third-party abstraction over provider APIs that differ materially |
| `async-openai` as the OpenAI provider | Full crate for one provider; its types would leak into `ferro-ai`'s public API |
| `langchain-rust` or similar orchestration | Framework-level orchestration; adds heavy deps (`ort`, `candle`) inappropriate for a web framework crate |
| `ndarray` / `nalgebra` | Overkill for a scalar cosine similarity function |
| `ort` (ONNX Runtime) | Local model inference; Ollama already provides a clean REST interface for local models |
| `tokio-stream` | `futures-util` already in workspace; `tokio-stream` adds overlap for this use case |
| `async-stream` | `futures-util::stream::unfold` is sufficient; avoids another proc-macro dep |
| Full `axum` dependency in `ferro-ai` | ferro-ai must remain framework-agnostic; SSE helpers live in framework, not in `ferro-ai` |
| `candle` / `burn` / `tch` (ML frameworks) | Local inference is out of scope for v12.1; Ollama handles it |
| `openai-func-enums` | Proc-macro layer for tool calling; adds a build-time dep for functionality implementable in ~50 lines |

---

## Version Summary

| Crate | Version | Status | New? |
|-------|---------|--------|------|
| `reqwest` | `0.12` | Existing | No |
| `serde` / `serde_json` | `1` | Existing | No |
| `async-trait` | `0.1` | Existing | No |
| `schemars` | `1` (1.2.1 current) | Existing in workspace | No — already in `ferro-json-ui`, add to `ferro-ai` |
| `tokio` | `1` | Existing | No |
| `futures-util` | `0.3` | Existing in workspace | No — already in `framework`, add to `ferro-ai` |
| `bytes` | `1` | Existing (hyper dep) | No |
| `http-body-util` | `0.1` | Existing (hyper-util dep) | No — add to `framework` for SSE response builder |
| `reqwest-eventsource` | `0.6` (0.6.0 current) | New | YES — add to `ferro-ai` |
| `pgvector` | `0.4` (0.4.1 current) | New, optional | YES — add to `ferro-ai` with `features = ["sqlx"]`, optional |

Two new crates: `reqwest-eventsource` (required) and `pgvector` (optional feature). Everything else is already in the workspace.

---

## Sources

- async-openai 0.38.2: https://docs.rs/async-openai/latest/async_openai/
- genai 0.5.3: https://docs.rs/crate/genai/latest
- pgvector 0.4.1: https://docs.rs/pgvector/latest/pgvector/
- reqwest-eventsource 0.6.0: https://docs.rs/reqwest-eventsource/latest/reqwest_eventsource/
- schemars 1.2.1: https://docs.rs/schemars/latest/schemars/
- tokio-stream 0.1.18: https://docs.rs/tokio-stream/latest/tokio_stream/
- async-stream 0.3.6: https://docs.rs/async-stream/latest/async_stream/
- axum SSE (0.8.x): https://docs.rs/axum/latest/axum/response/sse/
- ollama-rs 0.3.4: https://lib.rs/crates/ollama-rs
- Groq OpenAI compatibility: https://console.groq.com/docs/openai
- pgvector + SeaORM pattern: https://cosminsanda.com/posts/using-pgvector-with-seaorm-in-rust/
