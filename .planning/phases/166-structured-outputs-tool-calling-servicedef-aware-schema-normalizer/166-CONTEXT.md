# Phase 166: Structured Outputs, Tool Calling & ServiceDef-aware Schema Normalizer - Context

**Gathered:** 2026-06-08
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults selected; rationale logged per decision)

<domain>
## Phase Boundary

Build the typed structured-output layer on top of the Phase 165 `LlmClient` foundation:

1. `ferro_ai::complete::<T>()` — ergonomic typed completion; caller never touches schemars or `serde_json`.
2. `ferro_ai::schema::for_structured_output()` — generic JSON Schema normalizer that makes
   `schemars` 1.x output compatible with provider structured-output APIs (resolve `$ref`/`$defs`
   inline, add `additionalProperties: false`, strip provider-rejected constraints).
3. **`ServiceDef`-aware specialization** — the normalizer's projection path that *closes* the
   projection enums so the LLM is locked to valid projection shapes. This is the structural
   guarantee referenced by AISDK-02's projection-coherence clause.
4. `ToolDef` + `ToolRegistry` with a hard `max_iterations` cap and model-legible `ToolError`.

**Not in this phase:** embeddings / cosine similarity (Phase 167), ferro-cli migration onto the
SDK (Phase 170), `ai:make` / `ai:explain` commands (Phase 171), `make:json-view` v2 + the
projection-roundtrip test (Phase 173), and the Renderer-as-tool adapter (deferred — see Deferred Ideas).

**Current crate state (post-165):** `ferro-ai/src/client/{mod,anthropic,openai,ollama}.rs` ships the
`LlmClient` trait, three clients, `TokenStream`, and `CompletionRequest` — which already carries an
optional `schema: Option<serde_json::Value>` passthrough (D-11 of Phase 165). Phase 166 is the typed
ergonomic layer + normalizer + tool loop ON TOP of that passthrough. `ferro-ai/Cargo.toml` currently
depends only on `reqwest` (plus the 165 stream deps) — schemars, serde_json, ferro-projections, and a
JSON-Schema validator are new dependencies this phase introduces.

</domain>

<decisions>
## Implementation Decisions

### `complete::<T>()` API
- **D-01:** Public free function `ferro_ai::complete::<T>(client: &dyn LlmClient, prompt: &str) -> Result<T, Error>`
  where `T: schemars::JsonSchema + serde::DeserializeOwned`. This matches SC#1's literal form
  `ferro_ai::complete::<T>(client, prompt)`. Internal flow: `schema_for::<T>()` →
  `schema::for_structured_output(...)` → set `CompletionRequest.schema` → `client.complete(...)` →
  parse JSON response into `T`. The caller never calls schemars or `serde_json` directly (SC#1).
- **D-02:** A request-taking escape hatch (`complete_into::<T>(client, CompletionRequest)` or similar)
  for system prompt / `max_tokens` / model control is **Claude's discretion** — the small-core
  ergonomic `complete::<T>(client, prompt)` is the locked primary surface; the ergonomic form picks a
  sensible default `max_tokens` and no system prompt.

### Generic Schema Normalizer
- **D-03:** `ferro_ai::schema::for_structured_output(...) -> serde_json::Value`. Targets **schemars 1.x**
  output specifically (Draft 2020-12: `$defs` + `#/$defs/...` refs; defensively handle legacy
  `#/definitions/` too). It resolves all `$ref`/`$defs` inline (recursively, with a cycle guard),
  adds `additionalProperties: false` to every object schema, and strips the constraints Anthropic
  structured-output rejects (SC#2).
- **D-04:** Constraint policy — **PRESERVE** `type`, `properties`, `items`, `required`, `enum`,
  `additionalProperties`, and the `oneOf`/`anyOf` needed for tagged variants. **STRIP/transform** the
  keywords Anthropic structured-output rejects. The authoritative reject-list comes from Anthropic's
  documented constraints during research (candidates to confirm: `format`, `$schema`, `$id`, `title`,
  `default`, `examples`, and numeric/string bound keywords). **`enum` preservation is non-negotiable —
  it is the locking mechanism.** A unit test verifies the normalized output against Anthropic's
  documented constraints (SC#2).
- **D-05:** The normalizer's input parameter type (schemars `Schema` vs `serde_json::Value`) is
  **Claude's discretion**; the resolve-refs + `additionalProperties: false` + reject-strip behaviour
  is fixed.

### ServiceDef-aware Path — the structural guarantee
- **D-06 (central decision):** The projection enums `FieldMeaning` and `Intent` carry a
  `#[serde(untagged)] Custom(String)` escape hatch, so their **raw schemars schema accepts any string**.
  Generic normalization alone would let the LLM emit any value — directly contradicting SC#3. The
  `ServiceDef`-aware path therefore **closes** these enums: it emits a closed `enum` constraint of
  exactly the known snake_case variants and **drops the `Custom` untagged branch** from the
  LLM-facing schema. This is precisely why generic normalization is insufficient and the projection
  path is mandatory (per the AISDK-02 anti-requirement "Schema normalization is NOT
  projection-agnostic"). The Rust types keep `Custom` for deserialization; the LLM cannot produce a
  non-known value because the schema forbids it. (`Cardinality` has no `Custom` variant — already closed.)
- **D-07:** **Detection mechanism — single `complete::<T>()` entry, runtime `$defs` inspection.** The
  normalizer detects the projection case at runtime by inspecting the generated schema's `$defs` for
  the ferro-projections type names (`ServiceDef`, `FieldMeaning`, `Intent`, `Cardinality`, `ActionDef`,
  `GuardDef`, `StateDef`). No second public entry point, no stable-Rust specialization.
  *Alternative considered:* a marker trait with blanket + specific impls — rejected (impl conflict on
  stable Rust; runtime detection is simpler and equally explicit per "explicit over implicit").
- **D-08:** **Valid-value source of truth is `ferro-projections`, not a hardcoded list in ferro-ai.**
  Phase 166 adds a `ferro-projections` dependency; the closing pass derives the valid snake_case
  variant set from the projection enums' own schema output rather than duplicating the vocabulary
  (no-duplicate-control-surface convention). Exact extraction approach is a planning detail.
- **D-09:** **Structural-guarantee unit test (SC#3):** construct a `ServiceDef` JSON containing an
  invalid `FieldMeaning` (e.g. `"totally_bogus"`) and an invalid `Intent`, validate it against the
  `ServiceDef`-aware normalized schema with a real JSON-Schema validator, and assert validation
  **FAILS**; a valid `ServiceDef` passes. This needs a JSON-Schema validator dependency
  (recommend the `jsonschema` crate, dev-dependency).
- **D-10:** **Trade-off acknowledged:** closing the enums removes the LLM's ability to propose
  domain-specific custom `FieldMeaning`/`Intent` values. Per SC#3 and the structural-inseparability
  anti-requirement, this is intended for v12.1 — the LLM selects from the known projection vocabulary;
  custom meanings remain a human / trusted-path concern (the Rust `Custom` variant still
  deserializes from non-LLM sources).

### ToolRegistry & Tool Calling
- **D-11:** `ToolDef { name: String, description: String, parameters_schema: serde_json::Value, handler }`,
  where `parameters_schema` is normalized via `for_structured_output`. The handler is **async**
  (`Box<dyn Fn(serde_json::Value) -> BoxFuture<'static, Result<serde_json::Value, ToolError>> + Send + Sync>`)
  because tools commonly perform IO (SC#4).
- **D-12:** `max_iterations: u32` is **required at construction** (`ToolRegistry::new(max_iterations)`) —
  no `Default`, no zero-arg constructor, no override path to an unbounded loop. A warning is logged at
  iteration 5 and an error at the hard cap. The documented suggested value is 10 (a named convenience
  constructor that fills 10 is acceptable, but the value is never implicit/unbounded) (SC#5).
- **D-13:** `ToolError { message: String }` — model-legible. The dispatch loop surfaces tool failures
  to the LLM as this message, never as raw Rust panics, stack traces, or DB-constraint strings (SC#6).
- **D-14:** Tool-use **goes through the `LlmClient` layer**, not a parallel HTTP path. `CompletionRequest`
  currently carries only `schema`; this phase extends the client path with tool support so
  `ToolRegistry::dispatch(messages, client)` reuses the Phase 165 provider clients. Exact shape —
  a new `tools` field + a completion-response type that can carry tool-use blocks, vs a dedicated
  `complete_with_tools` method — is a **research/planning decision**, constrained by: (a) must work for
  both Anthropic tool-use and OpenAI function-calling, (b) `max_iterations` is enforced in ferro-ai
  not the provider, (c) single source of provider HTTP (reuse the 165 clients).

### Claude's Discretion
- Request-taking variant of `complete` (D-02) and its exact signature.
- Normalizer input parameter type (D-05).
- Internal module layout (`schema.rs`, `tools/` submodule, etc.).
- Exact JSON-Schema validator crate for the SC#3 test (`jsonschema` recommended).
- Exact client-layer tool-extension shape (D-14).
- Extraction technique for the projection valid-value set (D-08).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope & requirements
- `.planning/ROADMAP.md` §"Phase 166: Structured Outputs, Tool Calling & ServiceDef-aware Schema Normalizer" — goal + 7 Success Criteria (the boundary)
- `.planning/REQUIREMENTS.md` AISDK-02 (typed `complete::<T>()` + ServiceDef-aware normalizer), AISDK-03 (tool registration + `max_iterations`), and the **Anti-Requirements** section (no `ScaffoldPlan`; schema normalization is NOT projection-agnostic; no in-framework agent runtime)
- `.planning/phases/165-llmclient-trait-provider-implementations/165-CONTEXT.md` — predecessor decisions; especially D-11 (optional schema passthrough on `CompletionRequest` that 166 builds on) and D-13/D-14 (error enum shape)

### Existing crate (extended, not greenfield)
- `ferro-ai/src/client/mod.rs` — `LlmClient` trait, `CompletionRequest` (the `schema: Option<serde_json::Value>` passthrough 166 wraps; the request to extend for tools, D-14), `Message`/`Role`, `TokenStream`
- `ferro-ai/src/error.rs` — `Error` enum (add tool/schema variants as needed; `Error::Provider { status, message }` from 165)
- `ferro-ai/src/lib.rs` — public re-exports (add `complete`, `schema::for_structured_output`, `ToolDef`, `ToolRegistry`, `ToolError`)
- `ferro-ai/Cargo.toml` — add `schemars = "1"`, `serde_json`, `ferro-projections`, `futures` (for `BoxFuture`), and a JSON-Schema validator dev-dep (`jsonschema`)

### ferro-projections (the locked vocabulary — all already `#[derive(JsonSchema)]`)
- `ferro-projections/src/service.rs` — `ServiceDef` (the target type; fields/actions/guards/relationships/intent_hints/state_machine)
- `ferro-projections/src/field.rs` — `FieldMeaning` enum (**has `#[serde(untagged)] Custom(String)`** — must be closed, D-06), `FieldDef`, `DataType`
- `ferro-projections/src/intent.rs` — `Intent` enum (**has `#[serde(untagged)] Custom(String)`** — must be closed, D-06), `IntentHint`
- `ferro-projections/src/relationship.rs` — `Cardinality` (already closed), `RelationshipDef`, `NavigationHint`
- `ferro-projections/src/action.rs` — `ActionDef`, `GuardDef`
- `ferro-projections/src/state.rs` — `StateMachine`, `StateDef`

### Provider / library docs (fetch live during research — do not rely on training cutoff)
- Anthropic structured-output / tool-use API — the authoritative JSON-Schema constraint reject-list for D-04, and the tool-use request/response format for D-14 — via context7 / official docs
- OpenAI structured outputs + function-calling format (for D-14 cross-provider tool dispatch) — via context7
- `schemars` 1.x docs — `schema_for!`, `$defs` layout, Draft 2020-12 output — via context7
- `jsonschema` crate docs — validator for the SC#3 structural-guarantee test — via context7

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `CompletionRequest.schema: Option<serde_json::Value>` — the Phase 165 passthrough; `complete::<T>()` populates it (D-01). No new client plumbing needed for the structured-output path — only the typed wrapper + normalizer.
- All `ferro-projections` types already `#[derive(Serialize, Deserialize, JsonSchema)]` — the schema is generated, not hand-written; the SD-aware path operates on schemars output, not bespoke schemas.
- `Error` enum (thiserror, one-per-crate) — extend with schema/tool variants rather than introducing a new error type.

### Established Patterns
- `async_trait` already a dependency — async tool handlers (D-11) align with the existing async surface.
- Serde enums use `#[serde(rename_all = "snake_case")]` — the closed enum constraint values (D-06) are the snake_case variant names: e.g. `identifier`, `foreign_key`, `entity_name`, … for `FieldMeaning`; `browse`, `focus`, `collect`, `process`, `summarize`, `analyze`, `track` for `Intent`.
- The projection enums carry `#[schemars(description = "… Any other string is a custom domain-specific …")]` — that description reflects the OPEN type; the SD-aware LLM-facing schema deliberately diverges by closing it (D-06/D-10).

### Integration Points
- `ferro-ai` gains a `ferro-projections` dependency (D-08). Confirm workspace dependency direction: ferro-projections is a leaf the AI crate may depend on; verify no cycle (ferro-projections must not depend on ferro-ai).
- `.github/workflows/publish.yml` — if the dependency wave for `ferro-ai` changes due to the new `ferro-projections` dep, update the publish wave (workspace convention).
- `ToolRegistry::dispatch` extends the `LlmClient` path (D-14) — reuses the 165 Anthropic/OpenAI/Ollama clients; no new HTTP.

</code_context>

<specifics>
## Specific Ideas

- The structural guarantee's whole value is in D-06: **closing the `Custom(String)` escape hatch in the LLM-facing schema.** This is the one decision that makes SC#3's test (LLM cannot emit an invalid `FieldMeaning`/`Intent`) achievable. Plans must treat it as the core of the ServiceDef-aware path, not an afterthought.
- SC#2's "strips constraints Anthropic rejects" and SC#3's "lock to valid values" are in tension only if `enum` is treated as a strippable constraint. It is not — `enum` is preserved; `format`/bounds/etc. are stripped (D-04).

</specifics>

<deferred>
## Deferred Ideas

- **Renderer-as-tool adapter** (AISDK-03's "a `Renderer` IS a tool the LLM can invoke") — DEFERRED. Phase 166 SC#4/SC#5 specify closure-based tools only. The `Renderer` → `ToolDef` bridge belongs where Renderers materialize `ServiceDef`s into modalities (Phase 171+).
- **Tool calling in a streaming context** — DEFERRED (already a v12.1 future-requirement; Ollama drops tool calls when `stream: true`).
- **Conversation memory / multi-session history** — out of scope (stateless completions suffice for `ServiceDef` production).

</deferred>

---

*Phase: 166-structured-outputs-tool-calling-servicedef-aware-schema-normalizer*
*Context gathered: 2026-06-08*
