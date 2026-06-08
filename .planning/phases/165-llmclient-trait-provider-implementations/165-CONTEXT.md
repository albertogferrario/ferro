# Phase 165: LlmClient Trait & Provider Implementations - Context

**Gathered:** 2026-06-08
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults selected; rationale logged per decision)

<domain>
## Phase Boundary

Establish a provider-agnostic `LlmClient` trait in the existing `ferro-ai` leaf crate and ship
Anthropic, OpenAI (doubling as Groq via `base_url` override), and Ollama implementations, plus
`AiConfig::from_env()` for env-driven provider selection. Remove the hardcoded `ClassifierConfig`
default model by resolving it through `LlmClient::default_model()` per provider. The existing
`ClassificationProvider` / `AnthropicProvider` / `Classifier<T>` public surface is preserved; its
HTTP path is rewired onto the new client.

This phase delivers the foundation every later v12.1 AI phase builds on (structured outputs,
embeddings, SSE, CLI commands). It does NOT add typed `complete::<T>()`, the schema normalizer,
tool calling, or embeddings implementations — those are Phases 166-167.

Current crate state: `ferro-ai/src/` has `classifier/` (`mod.rs` with `ClassifierConfig` +
`Classifier<T>`, `anthropic.rs`, `provider.rs` with `ClassificationProvider`), `confirmation/`,
`error.rs`, `lib.rs`. Only dependency beyond serde/tokio is `reqwest`.
</domain>

<decisions>
## Implementation Decisions

### Trait & Naming
- **D-01:** The trait is named `LlmClient` (NOT `LlmProvider`). Roadmap SC and STATE.md decisions
  consistently use `LlmClient`; REQUIREMENTS.md AISDK-01 says `LlmProvider` — that is a stale wording
  to reconcile. Update REQUIREMENTS.md AISDK-01 to `LlmClient`.
- **D-02:** `LlmClient` lives in `ferro-ai/src/client/mod.rs` with `async fn complete(...)`,
  `async fn complete_stream(...)`, `async fn embed(...)`. `async_trait` retained (Rust stable
  async-fn-in-traits is not dyn-compatible; `Box<dyn LlmClient>` is required). Single trait — missing
  capabilities return `Err(Error::Unsupported)`, never panic, never a lowest-common-denominator collapse.
- **D-03:** `LlmClient::default_model() -> &str` resolves the per-provider default. `ClassifierConfig::default()`
  no longer hardcodes `"claude-sonnet-4-6"` — the model is resolved through the client's `default_model()`.

### Provider Set & Default Models
- **D-04:** Three implementations: `AnthropicClient`, `OpenAiClient` (doubles as Groq via `base_url`
  override — not a separate impl), `OllamaClient`. Each must be instantiable as `Box<dyn LlmClient>`.
- **D-05:** Default models, all overridable via `FERRO_AI_MODEL`:
  - Anthropic → `claude-sonnet-4-6` (preserves the current classifier default value; fast/cheap/capable
    for classification)
  - OpenAI → `gpt-4o`
  - Ollama → `llama3.1`

### Config
- **D-06:** `AiConfig::from_env()` reads `FERRO_AI_PROVIDER`, `FERRO_AI_MODEL`, `FERRO_AI_API_KEY`,
  `FERRO_AI_BASE_URL` and returns the correct provider as `Box<dyn LlmClient>`. Unknown provider names
  return a clear `Error::Config` at construction/startup — NOT at the first LLM call.
- **D-07:** Project-agnostic crate rule holds: only `FERRO_AI_*` env vars; no app identity hardcoded.
  (No `APP_NAME`/`APP_URL` needed for this phase — no generated artifact carries org identity here.)

### Streaming
- **D-08:** `complete_stream` is implemented for real in this phase for all three providers — not stubbed.
  Returns a ferro-ai-owned `TokenStream` (a `Stream<Item = Result<String, Error>>`-shaped type).
  Parse paths differ per provider: Anthropic + OpenAI use SSE via `reqwest-eventsource`; Ollama uses
  its line-delimited JSON (NDJSON) streaming. Rationale: SC#1 mandates the method and SC#6 explicitly
  declares `reqwest-eventsource` in provider modules — a signature-only stub would contradict both.
- **D-09:** `reqwest-eventsource 0.6` is a `pub(crate)` dependency used only inside provider modules.
  `TokenStream` is a public ferro-ai type, but `reqwest-eventsource` is NOT re-exported (SC#6). The
  wiring of `TokenStream` → framework `SseStream` happens in application handler code (Phase 168+);
  ferro-ai has no dependency on the framework SSE types.

### Old-Provider Convergence (no duplicate HTTP)
- **D-10:** The existing `AnthropicProvider` (`ClassificationProvider::classify_raw`) is reimplemented
  as a thin adapter that delegates its HTTP call to the new `AnthropicClient`. The duplicated Anthropic
  HTTP code in the old provider is deleted (single source of truth — aligns with the no-duplicate-
  control-surface convention and STATE.md "Classifier<T> delegates HTTP to AnthropicClient internally").
- **D-11:** To make D-10 possible BEFORE the Phase 166 schema normalizer exists, `AnthropicClient`'s
  request representation must carry an OPTIONAL structured-output schema/tool field from day one. The
  classifier passes its already-built JSON schema through this field. The ergonomic typed
  `complete::<T>()` + normalizer is a Phase 166 layer ON TOP of this — it is not required for the
  classifier bridge to work in 165.
- **D-12:** Public API is preserved exactly: `ClassificationProvider`, `AnthropicProvider`,
  `Classifier<T>`, `ClassifierConfig`, `ClassificationResult` keep their signatures and pass existing
  tests (SC#5). `ClassificationProvider` coexists with `LlmClient` — existing callers unchanged.

### Error Typing
- **D-13:** Add `Error::Unsupported` (mandatory — returned by capability methods a provider lacks,
  e.g. `AnthropicClient::embed()`).
- **D-14:** Upgrade `Error::Provider(String)` → `Error::Provider { status: Option<u16>, message: String }`.
  Retry logic switches from string-sniffing (`is_permanent_provider_error(msg: &str)` matching "400"/"401"/…)
  to status-based `is_retryable()` derived from the HTTP status. This is a breaking change to the `Error`
  enum — permitted in v12.1 AI (breaking changes explicitly allowed; not in production). Update the
  classifier retry path and any internal matchers accordingly.

### Claude's Discretion
- Exact signatures of `complete` / `complete_stream` / `embed` (request/response structs vs flat params) —
  planner decides, constrained by: `complete` must accept system + messages + max_tokens + the optional
  schema field (D-11); `complete_stream` returns `TokenStream`; `embed` returns `Vec<f32>`.
- Internal module layout under `client/` (one file per provider vs shared request/response module).
- Whether `AiConfig` selects providers via an enum dispatch or returns `Box<dyn LlmClient>` directly
  (D-06 only fixes the env-var contract and startup-time error behavior).
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope & requirements
- `.planning/ROADMAP.md` §"Phase 165: LlmClient Trait & Provider Implementations" — goal + 6 Success Criteria (the boundary)
- `.planning/ROADMAP.md` §"v12.1 AI — ferro-ai SDK & AI as Projection Consumer" — milestone goal, build order, new deps
- `.planning/REQUIREMENTS.md` AISDK-01 (the requirement; note the `LlmProvider`→`LlmClient` reconciliation, D-01) and the Anti-Requirements section (no `genai` crate, no lowest-common-denominator API)

### Existing crate (rewired, not greenfield)
- `ferro-ai/src/classifier/mod.rs` — `ClassifierConfig` (hardcoded model to remove, D-03), `Classifier<T>`, `is_permanent_provider_error` (string-sniffing to replace, D-14)
- `ferro-ai/src/classifier/provider.rs` — `ClassificationProvider` trait (preserved, object-safe)
- `ferro-ai/src/classifier/anthropic.rs` — existing `AnthropicProvider` (HTTP to be deduplicated onto `AnthropicClient`, D-10)
- `ferro-ai/src/error.rs` — `Error` enum (add `Unsupported`, restructure `Provider`, D-13/D-14)
- `ferro-ai/src/lib.rs` — public re-exports (add `LlmClient`, provider clients, `AiConfig`, `TokenStream`)
- `ferro-ai/Cargo.toml` — add `reqwest-eventsource 0.6` (and `futures`/`tokio-stream` as the stream plumbing requires)

### Conventions
- `CLAUDE.md` (project) §"Project-agnostic crates" — `FERRO_AI_*` env vars only, no app identity (D-07)
- Workspace conventions: `thiserror` one Error enum per crate; serde `rename_all = "snake_case"`; builder `with_*` consuming `self`

### Provider API docs (fetch live during research — do not rely on training cutoff)
- Anthropic Messages API (streaming SSE event format; tool-use for structured output) — via context7 / official docs
- OpenAI Chat Completions API (SSE streaming; `base_url` override for Groq compatibility)
- Ollama API (`/api/chat`, `/api/embeddings`; NDJSON streaming format)
- `reqwest-eventsource 0.6` crate docs — via context7

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ClassificationProvider` trait (object-safe `Arc<dyn>`) — pattern to mirror for `LlmClient` object safety
- `Classifier<T>` retry loop — reused; only its error-classification predicate changes (D-14)
- `Error` enum (thiserror) — extended, not replaced
- `confirmation/` module — untouched by this phase

### Established Patterns
- `async_trait` already a dependency — `LlmClient` uses the same pattern
- `from_env()` constructor pattern exists on `AnthropicProvider` — `AiConfig::from_env()` mirrors it
- String-based provider error + status-substring matching — the anti-pattern this phase removes (D-14)

### Integration Points
- `ferro-cli` currently has its own blocking Anthropic client (`ferro-cli/src/ai.rs`) — NOT touched here;
  Phase 170 migrates it onto this SDK. Phase 165 only establishes the SDK surface it will consume.
- `Classifier<T>` consumers across the workspace must keep compiling unchanged (D-12).
</code_context>

<specifics>
## Specific Ideas

- The `complete` request must carry an optional schema/tool field from the start (D-11) so the
  classifier bridge works before the Phase 166 normalizer lands. This is the load-bearing detail that
  lets convergence (D-10) happen in 165 rather than waiting for 166.
- Groq is NOT a fourth client — it is `OpenAiClient` with `FERRO_AI_BASE_URL` pointed at Groq's
  OpenAI-compatible endpoint and a Groq model id.
</specifics>

<deferred>
## Deferred Ideas

- **Typed `complete::<T>()` + schemars `$ref`/`$defs` normalizer + `ServiceDef`-aware path** → Phase 166.
- **`ToolRegistry` / tool calling with `max_iterations`** → Phase 166.
- **Embeddings implementation + cosine similarity + pgvector** → Phase 167. NOTE: Phase 167 SC#1 states
  "Anthropic, OpenAI, and Ollama providers implement `LlmClient::embed()`" — Anthropic has no embeddings
  endpoint, so `AnthropicClient::embed()` returns `Error::Unsupported` (D-13). Phase 167's success
  criterion must be reconciled to "OpenAI and Ollama implement `embed()`; Anthropic returns `Unsupported`."
  Flag this for Phase 167 planning.
- **Framework SSE primitives (`SseEvent`/`SseStream`/`HttpResponse::sse()`)** → Phase 168. ferro-ai's
  `TokenStream` stays framework-independent; wiring is application/handler code.

### Reviewed Todos (not folded)
None — no pending todos matched Phase 165 scope.
</deferred>

---

*Phase: 165-llmclient-trait-provider-implementations*
*Context gathered: 2026-06-08*
