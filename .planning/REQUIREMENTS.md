# Requirements: v12.1 AI — ferro-ai SDK & AI as Projection Consumer

## Milestone Goal

Expand `ferro-ai` into a production-grade, provider-agnostic AI SDK and make AI a first-class consumer of the projection / intent core. The killer feature: `ferro ai:make <description>` produces a typed `ferro_projections::ServiceDef` — the universal projection contract. The existing rendering pipeline (`ferro-json-ui` renderer, `ferro-mcp` introspection renderer, future modality renderers) covers everything downstream. AI does NOT recreate the pre-projections multi-file scaffolding workflow; it generates the input that the projection layer already knows how to render.

Live `ferro-mcp` introspection is the in-process context source so generated `ServiceDef`s reference existing models, intents, and conventions in the project — not generic templates.

## Conceptual Coherence Anchor

v12.1 AI does NOT introduce a new abstraction parallel to projection / intent. Every AI surface either **consumes** or **produces** a `ServiceDef`:

- **Produces:** `ai:make` (NL → `ServiceDef`).
- **Consumes:** `ai:explain` (`ServiceDef` → NL explanation framed in projection terms).
- **Renders from:** `make:json-view` v2 (`ServiceDef` → JSON-UI spec, AICLI-04, now unblocked).
- **Future modalities** (`make:whatsapp-flow`, voice, etc.) follow the same shape — each is a `Renderer` over `ServiceDef`.

The structured-outputs schema normalizer (AISDK-02) is `ServiceDef`-aware: when the LLM is completing into a `ServiceDef` (or a type containing one), the normalizer constrains output to valid projection shapes (`FieldMeaning`, `Intent`, `Cardinality`, `ActionDef` / `GuardDef`, `StateMachine`). For non-projection `T`, behaviour matches generic normalization. This is the structural guarantee that AI cannot drift from the intent system.

`ferro-mcp` introspection (already framed as a projection renderer in memory `project_mcp_as_projection_renderer.md`) is the in-process context source for both `ai:make` (what's already in the project) and `ai:explain` (what to interpret).

---

## v1 Requirements

### AI SDK — ferro-ai Expansion

- [x] **AISDK-01** — Developer can configure and use an LLM provider (Anthropic, OpenAI, Groq via OpenAI-compatible endpoint, Ollama) via env vars through a unified `LlmClient` trait; existing `Classifier<T>` API is preserved or cleanly superseded.

- [ ] **AISDK-02** — Developer can request typed responses via `ferro_ai::complete::<T>()` backed by a JSON Schema normalizer that resolves `schemars` `$ref` / `$defs` incompatibility with provider structured-output APIs. **`ServiceDef`-aware:** when `T` is `ferro_projections::ServiceDef` (or contains one), the normalizer emits a constrained schema that locks the LLM to valid projection shapes — `FieldMeaning` values from the published enum, `Intent` values from the seven structural intents, `Cardinality` from the relationship enum, `ActionDef` / `GuardDef` / `StateDef` shapes derived from `ferro-projections`. This is what makes AI output structurally inseparable from the intent system.

- [ ] **AISDK-03** — Developer can register Rust functions as AI tools; SDK dispatches tool-use calls automatically with a hard `max_iterations` guard. Tool registration accepts both arbitrary closures (existing pattern) and `Renderer` implementations from `ferro-projections` (a `Renderer` IS a tool the LLM can invoke to materialize a `ServiceDef` into any modality during a multi-turn loop).

- [ ] **AISDK-04** — Developer can generate text embeddings and compute cosine similarity (pure Rust helpers, zero extra crates).

- [ ] **AISDK-05** — Developer can persist and query embeddings via pgvector (feature-gated `pgvector 0.4`, thin sqlx raw-query module).

- [ ] **AISDK-06** — `ferro-cli/src/ai.rs` blocking client deleted; ferro-cli depends on ferro-ai and routes all LLM calls through it.

### SSE Streaming

- [ ] **AISSE-01** — Handler can return a streaming SSE response that pushes LLM tokens to the browser as they arrive; SSE routes are structurally excluded from any `CompressionLayer`.

- [ ] **AISSE-02** — `ferro-json-ui` provides a `StreamText` component that connects to an SSE endpoint URL and renders a token stream in place. The component is a JSON-UI element produced by a `Renderer` — consistent with the projection rendering pipeline.

### AI CLI Commands (the killer-feature surface)

- [ ] **AICLI-01** — Developer can run `ferro ai:make <description>` to produce a typed `ferro_projections::ServiceDef` from a natural-language description. The output is a commit-ready `ServiceDef` definition — fields with `FieldMeaning`, `Intent` hints, `ActionDef`s with `GuardDef`s, `StateMachine` if stateful, `RelationshipDef`s with `Cardinality`. Live `ferro-mcp` introspection is loaded as context so the generated `ServiceDef` references existing models, established `Intent` patterns in the project, naming conventions, and tenant scoping rules. **Output unit is the `ServiceDef`, NOT a pre-scaffolded handler / model / route bundle.** The existing rendering pipeline (`ferro-json-ui` renderer per AICLI-04, `ferro-mcp` introspection renderer, future modality renderers) produces the downstream artifacts.

- [ ] **AICLI-02** — `ferro ai:make` uses structured outputs (AISDK-02) to produce the `ServiceDef` directly. No `ScaffoldPlan` intermediary type. The schema the LLM completes against IS the schema for `ServiceDef` itself, derived from the existing `#[derive(Serialize, Deserialize, JsonSchema)]` on the `ferro-projections` types and normalized by the `ServiceDef`-aware path in AISDK-02. Non-projection glue (registration on `App`, handler skeleton for `ServiceDef::handle(...)`, migration generation) is invoked **after** the `ServiceDef` is produced, by calling the existing `make:*` helpers — those helpers consume the `ServiceDef`, they don't compete with it.

- [ ] **AICLI-03** — Developer can run `ferro ai:explain <route|model|service>` to get a projection-framed explanation: the `Intent`s the service projects (Browse / Focus / Collect / Process / Summarize / Analyze / Track), which fields' `FieldMeaning`s drive the rendering, what `ActionDef`s are exposed under which `GuardDef`s, what state transitions exist via `StateMachine`. Plain code prose is the fallback only when no `ServiceDef` is found for the target. Context loaded from `ferro-mcp` introspection.

- [ ] **AICLI-04** — `ferro make:json-view` upgraded to use structured outputs + `ServiceDef` introspection for schema-driven component selection. **Now unblocked: v12.0 JSON-UI v2 shipped 2026-05-19.** This is the first concrete `Renderer` over the `ServiceDef` produced by `ai:make` and is the second AI surface to land. Together with AICLI-01, this closes the produce-then-render loop end-to-end.

- [ ] **AICLI-05** — MCP tools `ai_scaffold` and `ai_explain` in `ferro-mcp` wrap the CLI command logic for in-process agent consumption. Agents calling `ai_scaffold` over MCP get the same `ServiceDef` output as the CLI — no parallel surface.

- [ ] **AICLI-06** — `ferro ai:make` and `make:json-view` v2 share a single end-to-end test: from NL description → `ServiceDef` → rendered JSON-UI spec → renderable view. This is the structural proof that AI is a first-class projection consumer rather than a parallel scaffolding system. The test lives in `ferro-ai/tests/projection_roundtrip.rs`.

---

## Anti-Requirements (explicit non-goals to prevent scope drift)

The following framings are explicitly rejected and any plan reintroducing them must be challenged at discuss-phase:

- **`ai:make` does NOT produce a multi-file scaffold bundle as its primary output.** The unit of work is the `ServiceDef`. Downstream files are byproducts produced by `Renderer`s that consume the `ServiceDef`.
- **There is no `ScaffoldPlan` intermediary type.** Structured outputs complete directly into `ServiceDef`. An intermediary type would be a parallel abstraction to the projection contract.
- **Schema normalization is NOT projection-agnostic.** AISDK-02's `ServiceDef`-aware path is required, not optional. Generic normalization is the fallback for non-projection `T`.
- **`ai:explain` does NOT default to code prose.** It defaults to a projection-framed explanation; code prose is the fallback only when no `ServiceDef` is found.
- **There is no in-framework agent runtime.** `ferro-mcp` + user-owned agent remains the architecture (carried from original scope).

---

## Future Requirements

These are real capabilities deferred beyond v12.1:

- **Tool calling in streaming context** — multi-turn tool interactions with partial streaming; Ollama has a documented bug dropping tool calls when `stream: true`; defer until stabilized upstream.
- **Conversation memory management** — per-session message history; out of scope for v12.1 (stateless completions are sufficient for projection production).
- **Multi-modal inputs** — image / audio input to LLM; v2.0+ direction per PROJECT.md.
- **In-framework agent runtime** — `make:agent` scaffolding, built-in loop orchestration; no bundled agent UX is a ferro architecture decision (anti-requirement above).
- **Non-visual modality `Renderer`s for AI-produced `ServiceDef`s** (`make:whatsapp-flow`, voice, native) — same `ServiceDef` input, additional output crates. Tracked under v2.0+ multimodal direction.

---

## Out of Scope

| Item | Reason |
|------|--------|
| In-framework agent runtime / `make:agent` | `ferro-mcp` + user's own agent is the supported workflow; bundled UX creates a competing abstraction to projection / intent. |
| Conversation memory management | Stateless completions sufficient for `ServiceDef` production; conversation memory is an agent-runtime concern, not a projection concern. |
| Multi-modal (image / audio) | v2.0+ direction; not needed to produce `ServiceDef`. |
| Groq as a distinct provider | Groq's API is OpenAI-compatible at `https://api.groq.com/openai/v1`; it's an `OpenAiProvider` config variant, not a separate impl. |
| `genai` crate as provider abstraction | Missing tool calling + embeddings in v0.5.3; lowest-common-denominator API trap. |
| `async-openai` crate | Leaks its types into `ferro-ai`'s public API; hand-rolled `reqwest` pattern is cleaner. |
| `ScaffoldPlan` intermediary type | Anti-requirement above — structured outputs complete directly into `ServiceDef`. |
| Pre-projection multi-file scaffolding output for `ai:make` | Anti-requirement above — the projection / intent core handles downstream rendering. |

---

## Traceability

Phase numbers reflect the ROADMAP.md v12.1 AI milestone section (Phases 165-173).

| REQ-ID | Phase | Status |
|--------|-------|--------|
| AISDK-01 | Phase 165 (LlmClient Trait & Provider Implementations) | Complete |
| AISDK-02 | Phase 166 (Structured Outputs, Tool Calling & Schema Normalizer) | Pending |
| AISDK-03 | Phase 166 (Structured Outputs, Tool Calling & Schema Normalizer) | Pending |
| AISDK-04 | Phase 167 (Embeddings & pgvector) | Pending |
| AISDK-05 | Phase 167 (Embeddings & pgvector) | Pending |
| AISDK-06 | Phase 170 (ferro-cli Migration) | Pending |
| AISSE-01 | Phase 168 (Framework SSE Primitives) | Pending |
| AISSE-02 | Phase 169 (StreamText Component) | Pending |
| AICLI-01 | Phase 171 (ai:make & ai:explain CLI Commands) | Pending |
| AICLI-02 | Phase 171 (ai:make & ai:explain CLI Commands) | Pending |
| AICLI-03 | Phase 171 (ai:make & ai:explain CLI Commands) | Pending |
| AICLI-04 | Phase 173 (make:json-view v2) | Unblocked — v12.0 shipped 2026-05-19 |
| AICLI-05 | Phase 172 (MCP Tool Wrappers) | Pending |
| AICLI-06 | Phase 173 (make:json-view v2) | Unblocked — projection-roundtrip test ships with the second `Renderer` |
