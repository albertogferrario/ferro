# Phase 170: ferro-cli Migration - Context

**Gathered:** 2026-06-08
**Status:** Ready for planning
**Mode:** `--auto` — gray areas auto-selected, recommended option chosen per area (logged below).

<domain>
## Phase Boundary

Delete the blocking Anthropic-only `ferro-cli/src/ai.rs` client and route every LLM
call in ferro-cli through the `ferro-ai` SDK (`AiConfig::from_env()` →
`Box<dyn LlmClient>`). The only LLM consumer in ferro-cli today is the
`make:json-view` command's two-pass generation. After the migration that command
works against any configured provider (Anthropic / OpenAI / Groq / Ollama), not just
Anthropic, with no behavior regression.

**In scope:** removing `ai.rs`, adding the `ferro-ai` dependency, the async→sync
bridge, env-var/provider gating, preserving the two-pass generation behavior.

**Not in scope:** new AI commands (`ai:make`/`ai:explain` are Phase 171), the
`ServiceDef` killer feature, `make:json-view` v2 redesign (Phase 173), expanding the
SDK's `CompletionRequest` surface (temperature/cache_control — noted as deferred SDK
enhancements, not done here).
</domain>

<decisions>
## Implementation Decisions

### Async→Sync Bridge (D-01)
- **D-01:** ferro-cli `main()` is **synchronous** (`fn main()` at `ferro-cli/src/main.rs:507`,
  no `#[tokio::main]`), and `make_json_view::run()` is sync. The `ferro-ai` SDK is async
  (`LlmClient::complete` is `async fn`). Bridge with a local tokio runtime at the LLM-call
  boundary: construct `tokio::runtime::Runtime::new()` (tokio "full" is already a ferro-cli
  dependency) and `block_on(...)` the async calls inside `generate_with_ai`. Keep the sync
  CLI surface — do **not** convert `run()`/`main()` to async. Minimal blast radius.
  - `[auto]` Async bridge — chose "local `Runtime::new().block_on()` at the call site"
    over "make the command async" (recommended: smallest change, no main() rewrite).

### SDK Entry Point (D-02) — the load-bearing decision
- **D-02:** Route both passes through `AiConfig::from_env()` → `Box<dyn LlmClient>` →
  `client.complete(CompletionRequest { .. })` (the low-level trait method), **not** the
  generic `ferro_ai::complete::<T>()` wrapper.
  - **Rationale / discrepancy with ROADMAP SC#2:** ROADMAP Phase 170 SC#2 literally says
    "all LLM calls go through `ferro_ai::complete::<T>()`". That wording does not fit
    `make:json-view`:
    1. Pass 1 is **plain text** (a component plan) with no schema — `complete::<T>()` always
       attaches a schema, so it is the wrong tool for Pass 1.
    2. Pass 2 is constrained by the **catalog's runtime-built schema**
       (`global_catalog().json_schema()`, a `serde_json::Value` encoding per-component
       `oneOf` constraints). `complete::<T>()` derives its schema from `schemars::schema_for!(T)`;
       `ferro_json_ui::Spec` does not derive `JsonSchema`, and even if it did, a schemars
       schema cannot reproduce the catalog's component constraints. The catalog schema is the
       validation source of truth (`catalog.validate(&spec)`), so the request must carry
       *that* schema via the raw `CompletionRequest.schema: Option<serde_json::Value>` field
       (documented as "passed through to the provider as-is").
  - **Resolution:** Read SC#2 as "through the ferro-ai SDK" (satisfies AISDK-06: blocking
    client deleted, ferro-cli depends on ferro-ai, all calls go through it). `complete::<T>()`
    is reserved for genuine typed-output cases (Phase 171 `ai:make` → `ServiceDef`). The
    planner should update/annotate SC#2 wording to "through the ferro-ai SDK / `LlmClient`"
    rather than the generic free function. **Do not** force `Spec` to derive `JsonSchema`
    just to satisfy the literal wording — that would diverge the generated schema from the
    catalog validator and is a real correctness hazard.
  - `[auto]` SDK entry point — chose "low-level `client.complete()` with catalog schema"
    over "`complete::<T>()` (requires Spec: JsonSchema, diverges from catalog schema)"
    (recommended: preserves the catalog as single schema source of truth).

### Two-Pass Behavior Preservation (D-03)
- **D-03:** Preserve the existing two-pass flow exactly — Pass 1 plain-text component plan,
  Pass 2 structured spec against `catalog.json_schema()`, then `Spec::from_json` +
  `catalog.validate` with static-template fallback on any failure. This is a plumbing-only
  migration (swap the transport), not a generation redesign. The v2 redesign is Phase 173.
  - `[auto]` Generation shape — chose "preserve two-pass" over "collapse to single
    structured call" (recommended: SC#3 requires existing behavior preserved).
- **D-03b:** `make_json_view.rs` keeps the same prompt-builder functions and fallback
  control flow; only `ai::call_anthropic_plain` / `ai::call_anthropic_structured` call sites
  are replaced with SDK calls through the bridged client. The prompt-building helpers
  (`build_json_view_pass1/2`, `scan_models`, `scan_routes`) move out of the deleted `ai.rs`
  into the command module (or a small prompt module) — they have no Anthropic coupling.

### Provider Gating & Env Vars (D-04)
- **D-04:** Replace the AI-vs-static gate. Today the code branches on
  `std::env::var("ANTHROPIC_API_KEY")` presence. After migration, gate on
  `AiConfig::from_env()`: `Ok(client)` → AI path; `Err(_)` → static template with the same
  informational stderr message. The `--no-ai` flag is preserved and short-circuits before
  any client construction.
  - `[auto]` Gating — chose "gate on `AiConfig::from_env()` success" over "keep checking
    `ANTHROPIC_API_KEY`" (recommended: SC#4 — `FERRO_AI_*` vars must control the provider).
- **D-04b:** Provider/model/key are controlled by `FERRO_AI_PROVIDER`, `FERRO_AI_MODEL`,
  `FERRO_AI_API_KEY` (read by `AiConfig::from_env()`). The old direct reads of
  `ANTHROPIC_API_KEY` and `FERRO_AI_MODEL` inside `ai.rs` are removed. `FERRO_AI_MODEL` is
  consistent — both the old client and `AiConfig` read it, so model selection is unchanged
  for existing users on Anthropic. Note: the old default model string `claude-sonnet-4-5`
  is dropped; the per-provider default now comes from `LlmClient::default_model()`
  (Phase 165 D-fix), which avoids a hardcoded model in ferro-cli.

### Request Knob Parity (D-05)
- **D-05:** `CompletionRequest` exposes `system`, `messages`, `max_tokens`,
  `model_override`, `schema` — it has **no `temperature` and no `cache_control`** fields.
  The old client set `temperature: 0.2` (deterministic codegen) and `cache_control:
  ephemeral` on the system prompt.
  - Map what exists: set `max_tokens` per pass via the request field (Pass 1 ~1024,
    Pass 2 ~4096, matching the old values); pass the system prompt via
    `CompletionRequest.system`.
  - Accept the loss of explicit `temperature` and prompt `cache_control` for this phase.
    Do **not** expand the SDK request surface inside Phase 170 (scope guard). The loss of
    `temperature: 0.2` is a real, if minor, determinism regression for codegen output —
    captured as a deferred SDK enhancement (see Deferred Ideas), not blocking.
  - `[auto]` Request knobs — chose "accept SDK defaults, map max_tokens + system" over
    "extend `CompletionRequest` with temperature now" (recommended: avoids cross-provider
    scope creep in a migration phase).

### reqwest `blocking` Feature (D-06)
- **D-06:** Do **not** drop the `blocking` feature from ferro-cli's `reqwest` dependency.
  `ferro-cli/src/commands/api_check.rs` still uses `reqwest::blocking`. SC#1 ("no
  `reqwest::blocking::Client` remains in ferro-cli") must be read as scoped to the deleted
  AI client — the planner should verify SC#1 against `api_check.rs` and not over-delete.
  Only `ai.rs`'s blocking usage is removed.
  - `[auto]` Feature cleanup — chose "keep `blocking` (api_check.rs uses it)" over
    "remove `blocking` feature" (recommended: honest dependency check; removal would break
    `api_check`).

### Claude's Discretion
- Where the relocated prompt-builder/scan helpers live (inline in `make_json_view.rs` vs a
  small `commands/make_json_view_prompts.rs` module) — planner's call; either is fine.
- Exact runtime-construction pattern (one `Runtime` for the whole command vs per-call) —
  prefer one runtime built once in `generate_with_ai` and reused across both passes.
- Whether to add a regression test that asserts the static-template fallback path still
  produces a catalog-valid spec when `AiConfig::from_env()` errors.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase definition & requirement
- `.planning/ROADMAP.md` — Phase 170 details (Goal, the 5 Success Criteria, build order Wave 3)
- `.planning/REQUIREMENTS.md` — `AISDK-06` (the single requirement this phase closes)

### Code to delete / migrate
- `ferro-cli/src/ai.rs` — the blocking Anthropic client to delete (411 lines:
  `call_anthropic_plain`, `call_anthropic_structured`, `build_json_view_pass1/2`,
  `generate_json_view`, `scan_models`, `scan_routes`). Prompt builders + scanners are
  Anthropic-agnostic and must be relocated, not deleted.
- `ferro-cli/src/commands/make_json_view.rs` — the consumer; `generate_with_ai()` holds the
  two-pass + fallback control flow that needs its call sites rewired.
- `ferro-cli/src/lib.rs`, `ferro-cli/src/templates/mod.rs` — also reference `ai::` (grep hits);
  verify and update module wiring.
- `ferro-cli/src/commands/api_check.rs` — uses `reqwest::blocking` (the reason D-06 keeps the feature).
- `ferro-cli/Cargo.toml` — add `ferro-ai` dependency; keep `reqwest` `blocking` feature.

### ferro-ai SDK surface to call into
- `ferro-ai/src/config.rs` §`AiConfig::from_env()` — returns `Result<Box<dyn LlmClient>, Error>`
  reading `FERRO_AI_PROVIDER` / `FERRO_AI_MODEL` / `FERRO_AI_API_KEY` / `FERRO_AI_BASE_URL`.
- `ferro-ai/src/client/mod.rs` — `LlmClient` trait, `CompletionRequest` (fields:
  `system`, `messages`, `max_tokens`, `model_override`, `schema`, `tools`, `tool_choice`),
  `Message`, `Role`, `TokenStream`. The `schema: Option<serde_json::Value>` field is the
  pass-through used for Pass 2's catalog schema.
- `ferro-ai/src/complete.rs` — `complete::<T>()` typed wrapper. Read to understand WHY it is
  **not** used here (schemars-derived schema vs catalog runtime schema — see D-02).
- `ferro-json-ui` — `global_catalog()`, `Catalog::prompt()`, `Catalog::json_schema()`,
  `Spec::from_json`, `Catalog::validate` (the validation contract Pass 2 must satisfy).

### Prior-phase context (SDK decisions this phase consumes)
- `.planning/phases/165-*/165-CONTEXT.md` — `LlmClient` trait, `AiConfig::from_env()`,
  `default_model()` per provider, removal of hardcoded model string.
- `.planning/phases/166-*/166-CONTEXT.md` — `complete::<T>()` + schema normalizer +
  `ServiceDef`-aware path (context for why typed-complete is reserved for Phase 171).
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `AiConfig::from_env() -> Result<Box<dyn LlmClient>, Error>` — single construction point;
  replaces all bespoke key/model/HTTP-client setup from `ai.rs`.
- `CompletionRequest` carries a raw `schema: Option<serde_json::Value>` pass-through — lets
  Pass 2 send `global_catalog().json_schema()` unchanged, preserving validation parity.
- `build_json_view_pass1/2`, `scan_models`, `scan_routes` in `ai.rs` are transport-agnostic
  and reusable verbatim after relocation.
- The two-pass + static-template fallback control flow in `make_json_view::generate_with_ai`
  stays intact; only the two call sites change.

### Established Patterns
- ferro-cli is a **synchronous** CLI (`fn main()`); async SDK calls require an explicit
  `tokio::runtime` bridge. tokio "full" is already a dependency.
- `reqwest` `blocking` feature is still load-bearing for `api_check.rs` — not removable.
- Provider model defaults come from `LlmClient::default_model()`, not hardcoded strings
  (Phase 165 convention) — keep ferro-cli free of model literals.

### Integration Points
- `make:json-view` is the only LLM call path in ferro-cli — the migration's blast radius is
  one command plus the deleted module and its re-exports.
- Output contract is unchanged: a `src/views/{name}.json` JSON-UI v2 spec validated against
  the catalog, with static-template fallback. Handlers still call
  `JsonUi::render_file("views/{name}.json", data)`.
</code_context>

<specifics>
## Specific Ideas

- The honest reading of SC#2 ("all calls through `complete::<T>()`") is the single most
  important thing to carry into planning: it is **not literally achievable** for
  `make:json-view` without breaking the catalog-schema-as-source-of-truth invariant. The
  planner should adjust the SC wording to "through the ferro-ai SDK / `LlmClient`" and route
  via `client.complete()` with the raw catalog schema. Flag, don't silently work around.
- Preserve the exact stderr UX of the static fallback (yellow warnings, "Falling back to
  static template.") so existing users see no behavioral surprise when no provider is configured.
</specifics>

<deferred>
## Deferred Ideas

- **`temperature` on `CompletionRequest`** — the old client used `temperature: 0.2` for
  deterministic codegen; the SDK request type has no temperature field. Adding
  `temperature: Option<f32>` (plus per-provider wiring) is an SDK surface change that belongs
  in a ferro-ai SDK enhancement, not this migration phase. Note for a future ferro-ai phase.
- **Prompt `cache_control` (ephemeral) on system prompts** — the old client set Anthropic
  prompt caching on the system block. The SDK does not expose a cache-control knob. Future
  SDK enhancement (provider-specific; design carefully to stay provider-agnostic).
- **`make:json-view` v2 redesign** — schema-driven component selection / `ServiceDef`
  consumption is Phase 173, explicitly out of scope here.

### Reviewed Todos (not folded)
None — no pending todos matched this phase.
</deferred>

---

*Phase: 170-ferro-cli-migration*
*Context gathered: 2026-06-08*
