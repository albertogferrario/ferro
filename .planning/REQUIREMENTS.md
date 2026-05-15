# Requirements: v12.1 AI — ferro-ai SDK & AI-Assisted Scaffolding

## Milestone Goal

Expand `ferro-ai` into a production-grade, provider-agnostic AI SDK and build AI-assisted scaffolding on top of it. The killer feature: `ferro ai:make <description>` uses live `ferro-mcp` introspection as context so the LLM generates code that fits the actual project, not a generic template.

---

## v1 Requirements

### AI SDK — ferro-ai Expansion

- [ ] **AISDK-01** — Developer can configure and use an LLM provider (Anthropic, OpenAI, Groq, Ollama) via env vars through a unified `LlmProvider` trait; existing `Classifier<T>` API is preserved or cleanly superseded
- [ ] **AISDK-02** — Developer can request typed responses via `ferro_ai::complete::<T>()` backed by a JSON Schema normalizer that resolves `schemars` `$ref`/`$defs` incompatibility with provider structured-output APIs
- [ ] **AISDK-03** — Developer can register Rust functions as AI tools; SDK dispatches tool-use calls automatically with a hard `max_iterations` guard
- [ ] **AISDK-04** — Developer can generate text embeddings and compute cosine similarity (pure Rust helpers, zero extra crates)
- [ ] **AISDK-05** — Developer can persist and query embeddings via pgvector (feature-gated `pgvector 0.4`, thin sqlx raw-query module)
- [ ] **AISDK-06** — `ferro-cli/src/ai.rs` blocking client deleted; ferro-cli depends on ferro-ai and routes all LLM calls through it

### SSE Streaming

- [ ] **AISSE-01** — Handler can return a streaming SSE response that pushes LLM tokens to the browser as they arrive; SSE routes are structurally excluded from any `CompressionLayer`
- [ ] **AISSE-02** — ferro-json-ui provides a `StreamText` component that connects to an SSE endpoint URL and renders a token stream in place

### AI CLI Commands

- [ ] **AICLI-01** — Developer can run `ferro ai:make <description>` to scaffold a complete feature (handler + model + routes + JSON-UI view) using live ferro-mcp introspection as context
- [ ] **AICLI-02** — `ferro ai:make` uses structured outputs to produce a typed `ScaffoldPlan`, then delegates to existing scaffold helpers (`generate_model`, `generate_migration`, `make:json-view`)
- [ ] **AICLI-03** — Developer can run `ferro ai:explain <route|model>` to get a plain-English explanation of an existing handler or model, with context loaded from ferro-mcp
- [ ] **AICLI-04** — `ferro make:json-view` upgraded to use structured outputs + ServiceDef introspection for schema-driven component selection *(deferred: depends on v12.0 JSON-UI v2 shipping)*
- [ ] **AICLI-05** — MCP tools `ai_scaffold` and `ai_explain` in ferro-mcp wrap the CLI command logic for in-process agent consumption

---

## Future Requirements

These are real capabilities deferred beyond v12.1:

- **Tool calling in streaming context** — multi-turn tool interactions with partial streaming; Ollama has a documented bug dropping tool calls when `stream: true`; defer until stabilized upstream
- **Conversation memory management** — per-session message history; out of scope for v12.1 (stateless completions are sufficient for scaffolding)
- **Multi-modal inputs** — image/audio input to LLM; v2.0+ direction per PROJECT.md
- **In-framework agent runtime** — `make:agent` scaffolding, built-in loop orchestration; no bundled agent UX is a ferro architecture decision

---

## Out of Scope

| Item | Reason |
|------|--------|
| In-framework agent runtime / `make:agent` | ferro-mcp + user's own agent is the supported workflow; bundled UX creates a competing abstraction |
| Conversation memory management | Stateless completions sufficient for v12.1 use cases |
| Multi-modal (image/audio) | v2.0+ direction; not needed for scaffolding commands |
| Groq as a distinct provider | Groq's API is OpenAI-compatible at `https://api.groq.com/openai/v1`; it's an `OpenAiProvider` config variant, not a separate impl |
| genai crate as provider abstraction | Missing tool calling + embeddings in v0.5.3; lowest-common-denominator API trap |
| async-openai crate | Leaks its types into ferro-ai's public API; hand-rolled reqwest pattern is cleaner |

---

## Traceability

| REQ-ID | Phase | Status |
|--------|-------|--------|
| AISDK-01 | — | Pending |
| AISDK-02 | — | Pending |
| AISDK-03 | — | Pending |
| AISDK-04 | — | Pending |
| AISDK-05 | — | Pending |
| AISDK-06 | — | Pending |
| AISSE-01 | — | Pending |
| AISSE-02 | — | Pending |
| AICLI-01 | — | Pending |
| AICLI-02 | — | Pending |
| AICLI-03 | — | Pending |
| AICLI-04 | — | Deferred (v12.0 gate) |
| AICLI-05 | — | Pending |
