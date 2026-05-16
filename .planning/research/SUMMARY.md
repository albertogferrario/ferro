# Research Summary — ferro v12.1 AI Milestone

**Domain:** ferro-ai SDK expansion + AI-assisted scaffolding CLI
**Synthesized:** 2026-05-15
**Confidence:** HIGH (all four research files: HIGH confidence, based on live codebase inspection + verified external sources)

---

## Executive Summary

v12.1 expands `ferro-ai` from a single-provider classification utility into a production-grade, multi-provider LLM SDK, and ships two new CLI commands (`ferro ai:make`, `ferro ai:explain`) that use live `ferro-mcp` introspection to generate and explain project-specific code. The milestone requires zero new crates for most capabilities — the workspace already contains `reqwest`, `serde_json`, `schemars`, `tokio`, `futures-util`, and `bytes`. Only two new dependencies are needed: `reqwest-eventsource 0.6` (incoming SSE from LLM providers) and optionally `pgvector 0.4` (feature-gated vector storage). No multi-provider abstraction crate (`genai`, `async-openai`) is adopted; each provider is implemented directly against its own wire format.

The recommended approach is a five-wave build order: expand `ferro-ai` first (the leaf crate everything else depends on), then add SSE HTTP primitives to `framework`, then `StreamText` to `ferro-json-ui`, then the CLI commands, and finally thin MCP tool wrappers. The `ferro-cli/src/ai.rs` blocking Anthropic client is deleted in full during Wave 4 and replaced by direct `ferro_ai::complete::<T>()` calls. The two new CLI commands are not generic AI assistants — they read actual project models, routes, and handler source via `ferro-mcp` functions called in-process, so generated scaffolds match the real project rather than a synthetic template.

The primary risk is not in the LLM integration itself but in five compounding complexity areas: (1) JSON Schema normalization before each structured-output call (Anthropic rejects `schemars` default output); (2) unbounded tool-calling loops requiring a hard `max_iterations` cap from day one; (3) SSE compression middleware silently buffering token streams; (4) context window overflow when the full `ferro-mcp` introspection output is used as a prompt without filtering; and (5) the `ClassifierConfig` default model being wrong for non-Anthropic providers. All five are preventable if addressed at the phase where they first become relevant.

---

## Key Findings

### Stack — What Is New vs Already Present

Only two crates are genuinely new to the workspace. Everything else is already available.

| Crate | Version | Status | Reason |
|-------|---------|--------|--------|
| `reqwest-eventsource` | `0.6.0` | **NEW** | Parse incoming SSE from Anthropic/OpenAI/Groq provider streams |
| `pgvector` | `0.4.1` | **NEW, optional** | `feature = "pgvector"` on `ferro-ai`; direct sqlx wrapper for vector storage |
| `schemars` | `1.2.1` | Exists in workspace; add to `ferro-ai` | JSON Schema generation for `complete::<T>()` and tool parameter schemas |
| `futures-util` | `0.3` | Exists in workspace; add to `ferro-ai` | `StreamExt` for token stream iteration |
| `http-body-util` | `0.1` | Exists as hyper-util dep; add to `framework` | `StreamBody` for hyper 1.x SSE responses |

The decision not to adopt `genai 0.5.3` is load-bearing: that crate lacks tool calling and embeddings as of May 2026. Adopting it now forces either a crate swap or a dual implementation later.

Groq is implemented as a config variant of `OpenAiClient` (same wire format, different base URL), not a separate provider struct.

### Features — Table Stakes, Differentiators, Anti-Features

**Must ship for v12.1 to be useful (table stakes):**

- Multi-provider async `LlmClient` trait with Anthropic, OpenAI, Groq, Ollama implementations
- `ferro_ai::complete::<T>()` structured output via `schemars::JsonSchema` derive (generalizes existing `Classifier<T>`)
- SSE streaming: `TokenStream` in `ferro-ai`; `SseStream`/`SseEvent` HTTP primitives in `framework`
- `ferro ai:make <description>` — context-aware scaffolding using live `ferro-mcp` introspection
- `ferro ai:explain <route|model>` — LLM-backed explanation from actual handler source, not rule-based heuristics
- `FERRO_AI_PROVIDER` + `FERRO_AI_MODEL` env vars centralized in `AiConfig::from_env()`; consistent across all AI commands
- `--dry-run` flag and `FERRO_AI_MAX_TOKENS_PER_COMMAND` env var on AI CLI commands (cost control)

**Differentiators (what makes v12.1 worth shipping):**

- `ai:make` assembles live context from `ferro-mcp` (`list_routes`, `list_models`, `db_schema`, `generation_context`) and generates a structured `ScaffoldPlan` before any file is written — this is what makes it real scaffolding, not a template expander
- `ai:explain` calls `get_handler` MCP tool to read actual handler source, then explains business logic including side effects (events, jobs) — not metadata reformatting
- `StreamText` JSON-UI component renders `<div data-ferro-stream-url>` + inline `EventSource` JS — server-driven streaming UI with zero client-side framework
- Schema normalization module (`ferro_ai::schema::for_structured_output`) that resolves `$ref`, adds `additionalProperties: false`, and strips unsupported constraints per provider — makes structured output portable across providers

**Explicitly out of scope (anti-features):**

- Bundled agent runtime or `make:agent` command — ferro's agent is external (Claude Code, Cursor via `ferro-mcp`); a second in-framework agent model is the wrong abstraction
- Conversation memory / session management in the SDK — application concern; callers supply `Vec<Message>` history
- Multi-modal generation (image, TTS, STT) — deferred post-v1.0
- Vector store integrations beyond optional `pgvector` — application infrastructure choice
- Provider failover / automatic fallback — explicit `Result<_, AiError>` is correct; silent failover masks misconfiguration
- Generic AI CLI for non-ferro questions — all AI CLI commands are scoped to ferro artifacts only
- `useChat`/`useCompletion` React hooks — frontend concern; SSE server support is the framework's responsibility

### Architecture — What Changes, What Stays

**Deleted:**
- `ferro-cli/src/ai.rs` — blocking `reqwest::Client`, Anthropic-only, duplicates `ferro-ai` logic; replaced wholesale by `ferro_ai::complete::<T>()` and `AiConfig::from_env()`

**New modules in `ferro-ai`:**

```
src/client/         LlmClient trait + 4 provider impls (Anthropic, OpenAI, Groq, Ollama)
src/client/config.rs  AiConfig::from_env() -> Box<dyn LlmClient>
src/complete.rs     ferro_ai::complete::<T>() entry point
src/tools.rs        ToolDef, ToolRegistry, dispatch loop with max_iterations guard
src/embeddings.rs   embed() + cosine_similarity() + optional pgvector::PgVectorStore
src/stream.rs       TokenStream = Pin<Box<dyn Stream<Item=Result<String, Error>>>>
src/schema.rs       for_structured_output() normalizer (resolve $ref, add additionalProperties)
```

**New in `framework`:**
```
src/http/sse.rs     SseEvent, SseStream (mpsc + StreamBody); HttpResponse::sse() factory
```

**New in `ferro-json-ui`:**
```
src/components/stream_text.rs  StreamText variant; render.rs updated
```

**New in `ferro-cli`:**
```
src/commands/ai_make.rs      ferro ai:make
src/commands/ai_explain.rs   ferro ai:explain
```

**Unchanged:** `ClassificationProvider`, `Classifier<T>`, `InMemoryConfirmationStore`, all 35+ `ferro-mcp` tools, `make:scaffold` helpers, `framework` `ai` feature flag, all Inertia paths.

**Key structural decision:** `ClassificationProvider` is kept alongside `LlmClient`. They coexist without breaking existing callers. `Classifier<T>` delegates its HTTP work to the new `AnthropicClient` internally.

**Key decoupling decision:** `SseStream` has no dependency on `ferro-ai`. The wiring of `TokenStream` -> `SseStream` happens in application handler code, not in either library. Both crates are independently testable.

### Build Order

```
Wave 1 — ferro-ai (leaf; no consumer changes yet)
  Step 1: LlmClient trait + 4 provider modules  [all subsequent steps depend on this]
  Step 2: complete::<T>(), tools.rs, embeddings.rs, schema.rs
  Step 3: TokenStream (stream.rs)  [async cancellation needs isolated step]

Wave 2 — framework SSE (no ferro-ai dep; can run in parallel with Wave 1)
  Step 4: SseEvent + SseStream + HttpResponse::sse()

Wave 3 — ferro-json-ui StreamText (depends on SSE URL convention from Step 4)
  Step 5: StreamText component + render.rs handler

Wave 4 — ferro-cli (depends on Waves 1-3)
  Step 6: Delete src/ai.rs; wire ferro-ai SDK into make:json-view  [validates SDK against existing command]
  Step 7: ferro ai:make  [primary CLI command; uses ferro-mcp in-process + SDK]
  Step 8: ferro ai:explain  [simpler; can parallelize with Step 7]
  Step 9: Improved make:json-view via ServiceDef  [gated on v12.0 catalog.prompt() shipping]

Wave 5 — ferro-mcp tools (thin wrappers; validated by Wave 4 first)
  Step 10: ai_scaffold + ai_explain MCP tools
```

Rationale: `ferro-ai` is a leaf crate; everything builds on the `LlmClient` trait established in Step 1. Wave 2 can start immediately in parallel because `SseStream` has no `ferro-ai` dependency. Step 9 is gated on v12.0 shipping; do not block v12.1 on it.

---

## Watch Out For

### 1. JSON Schema Normalization (Wave 1, Step 2)

`schemars` emits `$ref` references for complex types and does not add `additionalProperties: false`. Anthropic's structured output endpoint rejects both. The error returned is a 400 with no echo of the rejected schema, making it hard to debug.

**Prevention:** Implement `ferro_ai::schema::for_structured_output()` before any provider integration test. It resolves `$ref` inline, adds `additionalProperties: false` to every object, and strips unsupported constraints. This is the only path for generating schemas for structured output calls. Include a test that verifies output against Anthropic's documented constraints.

---

### 2. Unbounded Tool-Calling Loops (Wave 1, Step 2 / tools.rs)

LLM tool-calling loops can run indefinitely — documented real-world incident: 1.67B tokens in 5 hours. Tool errors that return non-actionable messages (stack traces, DB constraint errors) cause the model to retry the same call with the same arguments.

**Prevention:** Hard `max_iterations: u32` cap in `ToolRegistry` dispatch loop, default 10, never unbounded. Define `ToolError { message: String }` with model-legible descriptions. Log warnings at 5, errors at 10. Must be in the initial implementation, not added after observing runaway behavior.

---

### 3. SSE Compression Middleware (Wave 2, Step 4)

`tower-http`'s `CompressionLayer` buffers the response body before compressing. SSE responses buffered this way deliver all tokens at once after a long pause. The middleware does not warn.

**Prevention:** Exclude SSE routes from `CompressionLayer`. Add an integration test that verifies token-by-token delivery, not just final output correctness. Default keep-alive at 15-second intervals (`:ping\n\n`) prevents reverse proxy idle-timeout disconnects on long reasoning steps.

---

### 4. Context Window Overflow in `ai:make` (Wave 4, Step 7)

For a large application, full `ferro-mcp` introspection output easily exceeds 50K tokens as a system prompt. Quality degrades before the context limit is reached. The full JSON-UI component catalog (40-80 KB schema) compounds this.

**Prevention:** Apply selective context loading: filter models and routes to those semantically relevant to the user's description using string matching. Use per-component schemas (v12.0 direction), not the full catalog. Build context selection logic before prompt construction; do not add it after observing cost or quality problems.

---

### 5. `ClassifierConfig` Default Model Wrong for Non-Anthropic Providers (Wave 1, Step 1)

`ClassifierConfig::default()` hardcodes `model: "claude-sonnet-4-6"`. When a user configures OpenAI or Groq, the default model is wrong and surfaces only as a 400 at runtime.

**Prevention:** Add `fn default_model(&self) -> &str` to `ClassificationProvider`. Remove the hardcoded default from `ClassifierConfig` (or make it `Option<String>` resolved through the provider). Must be fixed before any provider other than Anthropic is added.

---

## Open Questions

These need a decision before or during Phase 1 (SDK foundation):

1. **Capability trait split vs single `LlmClient` trait.** PITFALLS.md recommends splitting into `CompletionProvider`, `StreamingProvider`, `EmbeddingProvider`, `ToolProvider`. ARCHITECTURE.md describes a single `LlmClient` with four methods. Recommendation: single `LlmClient` trait with `Err(Error::Unsupported)` for capabilities a provider lacks — preserves ergonomic dispatch without lowest-common-denominator collapse.

2. **`async_trait` retention.** Rust 1.75+ has stable async fn in traits, but native async traits are not dyn-compatible. `async_trait` is still required for `dyn LlmClient`. Keep it; document the constraint; do not migrate speculatively.

3. **`ScaffoldPlan` type design.** The two-step `ai:make` approach (generate `ScaffoldPlan` via structured output, then expand per file) requires designing the `ScaffoldPlan` struct before Step 7 begins. This type must be designed during Step 7 planning, not during implementation.

4. **v12.0 gate for Step 9.** Improved `make:json-view` via `ServiceDef` and `catalog.prompt()` is gated on v12.0 JSON-UI v2. Confirm v12.0 status before scoping Step 9; if not shipped, Step 9 moves to v12.2.

5. **`reqwest-eventsource` visibility.** Needed inside provider implementations but must not be a public API surface. Confirm it is `pub(crate)` in all provider modules before the first PR.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Versions verified against crates.io 2026-05-15; rejection of `genai`/`async-openai` substantiated with documented capability gaps |
| Features | HIGH | Verified against Laravel AI SDK, Vercel AI SDK 6, Spring AI, Rig v0.37.0 live sources; existing ferro-ai capabilities from direct codebase read |
| Architecture | HIGH | All component boundaries and integration points from direct codebase inspection; no speculation |
| Pitfalls | HIGH | All pitfalls backed by documented incidents, provider bug reports, and confirmed codebase issues |

**Overall: HIGH**

**Gaps:**
- `ScaffoldPlan` type design is not in research; must be done during Step 7 planning
- `ferro-mcp` in-process function signatures assumed stable; verify against current `ferro-mcp/src/tools/` before Step 7
- Step 9 timing depends on v12.0 shipping; treat as out of scope until confirmed

---

## Sources

Full source lists are in the individual research files. Key sources:

**HIGH confidence (live, verified 2026-05-15):**
- pgvector 0.4.1: https://docs.rs/pgvector
- reqwest-eventsource 0.6.0: https://docs.rs/reqwest-eventsource
- schemars 1.2.1: https://docs.rs/schemars
- Laravel AI SDK 12.x/13.x: https://laravel.com/docs/12.x/ai-sdk
- Vercel AI SDK 6: https://vercel.com/blog/ai-sdk-6
- Rig v0.37.0: https://github.com/0xPlaygrounds/rig
- Anthropic structured outputs: https://platform.claude.com/docs/en/build-with-claude/structured-outputs
- Ollama streaming+tool bug: https://github.com/ollama/ollama/issues/12557
- Groq finish_reason streaming bug: https://community.groq.com/t/groq-api-bug-report-missing-finish-reason-in-streaming-responses/775
- Direct codebase: `ferro-ai/src/`, `ferro-cli/src/ai.rs`, `ferro-mcp/src/tools/`, `framework/src/http/`

**MEDIUM confidence:**
- PostHog LLM code generation retrospective: https://posthog.com/blog/correct-llm-code-generation
- LLM tool loop failure modes (documented $50K incident): https://medium.com/@komalbaparmar007/llm-tool-calling-in-production-rate-limits-retries-and-the-infinite-loop-failure-mode-you-must-2a1e2a1e84c8

---

*Research synthesized: 2026-05-15*
*Ready for roadmap: yes*
