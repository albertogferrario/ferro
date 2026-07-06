# Phase 171: ferro ai:make & ferro ai:explain CLI Commands - Context

**Gathered:** 2026-06-08
**Status:** Ready for planning
**Mode:** `--auto` — all gray areas auto-selected, recommended option chosen per area (logged inline).

<domain>
## Phase Boundary

Ship the two killer-feature CLI commands of the v12.1 milestone:

- **`ferro ai:make <description>`** — natural-language description → a single typed
  `ferro_projections::ServiceDef` (the universal projection contract). Live ferro-mcp
  introspection is loaded **in-process** (library call, not subprocess) and filtered to
  the items relevant to the description before prompt construction. The `ServiceDef` is
  produced via the Phase 166 `ServiceDef`-aware structured-output path.

- **`ferro ai:explain <route|model|service>`** — projection-framed explanation of an
  existing service using actual source loaded through ferro-mcp introspection. When a
  `ServiceDef` exists for the target, the explanation is framed in projection terms
  (`Intent`, `FieldMeaning`, `ActionDef`/`GuardDef`, `StateMachine`); plain code prose is
  the fallback only when no `ServiceDef` is found.

**In scope:** the two commands, in-process ferro-mcp wiring, relevance filtering of
context, the `ServiceDef`-aware completion path, a `ServiceDef` → Rust-builder source
emitter (new), target resolution for `ai:explain`, the `FERRO_AI_MAX_TOKENS_PER_COMMAND`
cost guard, `--dry-run` on both.

**Not in scope (other phases):**
- MCP tool wrappers `ai_scaffold`/`ai_explain` — **Phase 172** (thin layer over this CLI logic).
- `make:json-view` v2 as the first concrete `Renderer` over the `ServiceDef`, and the
  AICLI-06 projection-roundtrip test — **Phase 173**.
- Any multi-file scaffold bundle (handler / model / route / migration files). `ai:make`
  emits the `ServiceDef` **only**; downstream `make:*` helpers consume it (SC#3).
- No `ScaffoldPlan` intermediary type — structured outputs complete directly into `ServiceDef`.
</domain>

<decisions>
## Implementation Decisions

### In-process ferro-mcp introspection wiring (D-01)
- **D-01:** Call the ferro-mcp tool `execute` functions **directly in-process** — they are
  plain library functions returning typed structs, not JSON-RPC handlers. ferro-cli already
  depends on `ferro-mcp` (`ferro-cli/Cargo.toml:45`). No subprocess, no `ferro mcp` server
  spin-up. Concretely:
  - `ferro_mcp::tools::list_routes::execute(&project_root)` → `RoutesInfo` (async)
  - `ferro_mcp::tools::list_models::execute(&project_root)` → `Vec<ModelDetails>` (sync)
  - `ferro_mcp::tools::database_schema::execute(&project_root, None)` → `SchemaInfo` (async)
  - `ferro_mcp::tools::generation_context::execute()` → `GenerationContext` (sync, small/fixed — always included)
  - `ferro_mcp::tools::list_projections::execute(&project_root, None)` +
    `ferro_mcp::tools::inspect_projection::execute(&project_root, name)` → existing `ServiceDef`s
  - For `ai:explain`: `explain_route::execute` / `explain_model::execute` as prose fallback.
  - `[auto]` Wiring — chose "direct in-process `tools::*::execute()` calls" over "launch
    ferro-mcp server and call over JSON-RPC" (recommended: SC#1 says in-process; the tool
    fns are already the public introspection surface; zero IPC overhead).

### Selective context relevance filtering (D-02)
- **D-02:** Filter introspection results to items relevant to the description with a
  **deterministic lexical relevance pass** (token / identifier overlap between the
  description and each item's name + description + field names), ranked, top-N kept under an
  input budget. The fixed-size `generation_context` (conventions) is **always** included
  verbatim. This prevents context-window overflow on large projects (SC#1) with **no extra
  LLM round-trip and no embedding-provider dependency** — deterministic and unit-testable.
  - **SC#1 wording note ("semantically relevant"):** v1 satisfies this lexically, not with
    vector semantics. Embedding-based reranking (`ferro_ai::embed` + `cosine_similarity`,
    Phase 167) is the natural upgrade but adds an embed call per candidate item and requires
    provider embedding support → **deferred enhancement**, gated behind project size. The
    planner should annotate SC#1 as "lexically relevant in v1; embedding rerank deferred"
    rather than block on perfect semantic ranking.
  - `[auto]` Relevance — chose "deterministic lexical token-overlap filter + always-include
    generation_context" over "embedding cosine-similarity rerank" and over "LLM relevance
    pre-pass" (recommended: cost guard + determinism + filtering only needs overflow
    prevention, not perfect ranking).

### `ai:make` output artifact: format & destination (D-03)
- **D-03:** `ai:make` emits the produced `ServiceDef` as a **single commit-ready Rust
  builder source file** at `src/projections/<snake_name>.rs`, registered in
  `src/projections/mod.rs` — matching the established project convention that ServiceDefs
  are `pub fn <name>() -> ServiceDef` builder functions (this is exactly what
  `list_projections`/`inspect_projection` scan for, and what `make:projection` scaffolds).
  Reuse `make:projection`'s directory-creation + `mod.rs`-registration logic.
  - **New component required:** a `ServiceDef` → builder-source emitter. **None exists today**
    — `make:projection` only writes an *empty* `ServiceDef::new("…")` template; the Phase 135
    bridge derives `ServiceDef` *from* models but does not serialize one *to* Rust source.
    This emitter (typed `ServiceDef` value → idiomatic `.field(...).action(...).guard(...)`
    builder chain) is the central new implementation unit of this phase.
  - **`--dry-run`** prints the `ServiceDef` as pretty JSON (`serde_json::to_string_pretty`)
    to stdout and writes **nothing** (SC#2).
  - **Single artifact only** — no handler/model/route/migration files (SC#3). Those are
    produced later by existing `make:*` helpers and the Phase 173 renderer consuming this
    `ServiceDef`.
  - `[auto]` Output — chose "Rust builder file in src/projections/ (+ new ServiceDef→source
    emitter)" over "write a JSON file" and over "stdout-only" (recommended: keeps output
    discoverable by `list_projections`/`inspect_projection` and consumable by Phase 173,
    consistent with the only existing ServiceDef persistence convention).

### `ServiceDef`-aware completion path + cost guard (D-04)
- **D-04:** `ai:make` produces the `ServiceDef` through the Phase 166 **`ServiceDef`-aware
  schema-normalizer path**, which locks the LLM to valid projection shapes (`Intent`,
  `FieldMeaning`, `Cardinality` enums; `ActionDef`/`GuardDef`/`StateDef`). `ServiceDef`
  already derives `JsonSchema` (`ferro-projections/src/service.rs:62`), so
  `schemars::schema_for!(ServiceDef)` → `schema::for_structured_output(...)` is valid.
  - **`complete::<ServiceDef>()` gap:** the existing `ferro_ai::complete::<T>()` hardcodes
    `max_tokens: 4096` and `system: None`, `model_override: None`
    (`ferro-ai/src/complete.rs:64-77`). Phase 171 needs a **configurable `max_tokens`** (the
    cost guard, SC#5) and benefits from a system prompt for the large introspection context.
    **Decision:** add a small options-carrying variant `ferro_ai::complete_with::<T>(client,
    prompt, CompleteOptions { max_tokens, system, model_override })`; keep `complete::<T>()`
    as the zero-config wrapper delegating to it with defaults. `ai:make` calls
    `complete_with::<ServiceDef>()`. This preserves the `ServiceDef`-aware normalizer path
    (SC#2 reads "via `complete::<ServiceDef>()`" — satisfied: same typed entry, same
    normalizer, now parameterized) and is the minimal SDK surface change.
  - **`FERRO_AI_MAX_TOKENS_PER_COMMAND`** (new env) maps onto the request `max_tokens` for
    both commands (sensible defaults: `ai:make` ~8192, `ai:explain` ~2048). Read via the
    command, not baked into `AiConfig` (it is a per-command guard, not provider config).
  - `[auto]` Completion path — chose "`complete_with::<ServiceDef>()` options variant (adds
    configurable max_tokens, keeps ServiceDef-aware normalizer)" over "build raw
    `CompletionRequest` manually like Phase 170" and over "use `complete::<T>()` as-is with
    fixed 4096" (recommended: keeps the typed killer path intact while honoring the cost
    guard; minimal, justified SDK addition).

### `ai:explain` target resolution & projection framing (D-05)
- **D-05:** Single positional `<target>`. **Auto-detect** the kind by attempting ferro-mcp
  lookups in order — route path → model name → service/projection name — and use the first
  match. Optional `--type route|model|service` to disambiguate ambiguous names.
  - When a `ServiceDef` is found for the target (via `list_projections` +
    `inspect_projection`), return the **projection-framed** explanation: `Intent`s via
    `ferro_projections::derive_intents(&service)`, which `FieldMeaning`s drive rendering, the
    `ActionDef`s exposed under which `GuardDef`s, and `StateMachine` transitions if present
    (SC#4).
  - **Plain code prose fallback** via `explain_route`/`explain_model` only when no
    `ServiceDef` is found for the target.
  - The explanation prose is LLM-generated from the introspected facts; it must reference
    only what introspection reports — no generic templates, no invented fields (SC#6).
  - `[auto]` Resolution — chose "auto-detect route→model→service with optional `--type`
    override" over "require `--type` always" and over "prefixed target syntax" (recommended:
    lowest-friction agent/dev ergonomics; deterministic precedence).

### Command gating, AI-required failure mode & dry-run (D-06)
- **D-06:** Both `ai:make` and `ai:explain` **require** a configured provider via
  `AiConfig::from_env()`. Unlike `make:json-view` (which has a static-template fallback),
  these commands have **no non-AI path** — on `Err(_)` from `AiConfig::from_env()` they fail
  fast with a clear, actionable message naming `FERRO_AI_PROVIDER` / `FERRO_AI_API_KEY` /
  `FERRO_AI_MODEL`. No silent degradation.
  - **`ferro-projections` feature:** it is currently an **optional** ferro-cli dependency
    behind the `projections` feature (`ferro-cli/Cargo.toml:47,55`). `ai:make`/`ai:explain`
    need `ServiceDef`. **Decision:** the planner makes `projections` a **default** ferro-cli
    feature (the projection contract is now core to the CLI's killer surface), or gates the
    `ai:*` subcommands behind it with a clear error if built without it. Default-on is
    recommended for coherence with the milestone's projection-consumer thesis.
  - **`--dry-run` semantics:**
    - `ai:make --dry-run` → print the produced `ServiceDef` (pretty JSON), write nothing.
    - `ai:explain --dry-run` → print the **assembled context/prompt** that *would* be sent
      (lets devs inspect selected context and estimate cost) without making the LLM call.
  - `[auto]` Gating/failure — chose "AI-required, fail-fast with named env vars; default-on
    `projections` feature" over "silent fallback" and over "keep projections optional"
    (recommended: these are AI-native commands; honest failure beats degraded output; the
    projection contract is core surface now).

### Claude's Discretion
- Exact lexical-relevance scoring formula (token set overlap vs. weighted by field-name
  matches), top-N cutoff value, and input-token budget — planner/researcher decide,
  grounded in typical project sizes.
- Default `max_tokens` constants per command (the `ai:make` ~8192 / `ai:explain` ~2048
  figures above are starting points, not locked).
- Prompt wording / system-prompt structure for both commands.
- Whether `complete_with` carries `model_override` now or only `max_tokens` + `system`
  (carry all three if cheap; only `max_tokens` is strictly required by SC#5).
- CLI flag surface beyond `--dry-run` / `--type` (e.g. `--output <path>` override for
  `ai:make`) — add only if it falls out naturally.

### Folded Todos
None — no pending todos matched this phase.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase definition & requirements
- `.planning/ROADMAP.md` §"Phase 171: ferro ai:make & ferro ai:explain CLI Commands" — goal + 6 success criteria.
- `.planning/REQUIREMENTS.md` — `AICLI-01`, `AICLI-02`, `AICLI-03` (the requirements this phase closes).

### The projection contract (output type of `ai:make`)
- `ferro-projections/src/service.rs` — `ServiceDef` struct (`:63`), `FieldDef`, `ActionDef`, `GuardDef`, `RelationshipDef`, `IntentHint`, `StateMachine`. Derives `JsonSchema` (`:62`).
- `ferro-projections/src/intent.rs` §`Intent` enum (`:18`) — Browse / Focus / Collect / Process / Summarize / Analyze / Track.
- `ferro-projections/src/field.rs` §`FieldMeaning` enum (`:35`).
- `ferro-projections/src/relationship.rs` §`Cardinality` enum (`:10`).
- `ferro-projections/src/derive.rs` §`derive_intents(&ServiceDef)` (`:75`) — used by `ai:explain` projection framing.

### Structured-output SDK path (Phase 166)
- `ferro-ai/src/complete.rs` §`complete::<T>()` (`:57`) — the typed entry; note hardcoded `max_tokens: 4096` / `system: None` that D-04 extends.
- `ferro-ai/src/schema/mod.rs` §`for_structured_output` + the `ServiceDef`-aware specialization.
- `.planning/phases/166-structured-outputs-tool-calling-servicedef-aware-schema-normalizer/166-CONTEXT.md` — decisions behind the normalizer & ServiceDef-aware enum closing.

### In-process introspection (ferro-mcp tool fns)
- `ferro-mcp/src/tools/list_routes.rs` §`execute` (`:95`, async), `list_models.rs` §`execute` (`:165`, sync), `database_schema.rs` §`execute` (`:28`, async), `generation_context.rs` §`execute` (`:59`, sync), `list_projections.rs` §`execute` (`:31`), `inspect_projection.rs` §`execute` (`:48`), `explain_route.rs` §`execute` (`:35`), `explain_model.rs` §`execute` (`:50`).

### Output convention & emitter reference
- `ferro-cli/src/commands/make_projection.rs` — existing `src/projections/<name>.rs` scaffolding + `mod.rs` registration logic to reuse; the *empty* `ServiceDef::new(...)` template (`:367`, `:539`) that D-03's emitter populates.

### SDK migration precedent (transport / bridge patterns)
- `.planning/phases/170-ferro-cli-migration/170-CONTEXT.md` — async→sync bridge (D-01), `AiConfig::from_env()` gating (D-04), the `complete::<T>()`-vs-raw-`CompletionRequest` tradeoff (D-02) that this phase's D-04 resolves in the typed direction.

### Crate manifest
- `ferro-cli/Cargo.toml` (`:45-55`) — existing `ferro-mcp`, `ferro-ai` deps and the **optional** `ferro-projections` / `projections` feature gate D-06 addresses.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **ferro-mcp tool `execute` fns** — already the typed introspection surface; call directly in-process (D-01).
- **`make:projection` file/mod.rs writer** (`make_projection.rs`) — reuse its `src/projections/` directory + `mod.rs` registration path for `ai:make`'s output (D-03).
- **`ferro_ai::complete::<T>()` + ServiceDef-aware normalizer** (Phase 166) — the typed killer path; extend with options (D-04).
- **`derive_intents` + `FieldMeaning`/`Intent`/`StateMachine`** (ferro-projections) — the vocabulary for `ai:explain`'s projection framing (D-05).
- **Async→sync bridge pattern** from Phase 170 — `tokio::runtime::Runtime::new().block_on(...)` at the LLM/async-introspection call boundary (ferro-cli `main()` is sync).

### Established Patterns
- ServiceDefs are **Rust builder functions** in `src/projections/*.rs` (`pub fn name() -> ServiceDef`), discovered by regex scan — the output of `ai:make` must conform to this to be discoverable/consumable.
- Provider config flows exclusively through `FERRO_AI_*` env vars via `AiConfig::from_env()` (Phase 165/170) — no direct `ANTHROPIC_API_KEY` reads.
- `make:*` commands are sync; LLM calls are bridged with a local tokio runtime.

### Integration Points
- New `ai:make` / `ai:explain` subcommands registered in `ferro-cli/src/main.rs` (clap `#[command(name = "ai:make")]` / `"ai:explain"`, alongside the `make:*` family at `:77-288`; dispatch in the `match cli.command` at `:510`).
- `ai:make` output feeds Phase 173 `make:json-view` v2 (the first `Renderer`) and Phase 172 MCP wrappers — keep the `ServiceDef` the single shared shape (no parallel surface).
- `complete_with` (new) lands in `ferro-ai/src/complete.rs` + a re-export in `ferro-ai/src/lib.rs`.
</code_context>

<specifics>
## Specific Ideas

- The phase's conceptual proof is "AI is a first-class projection **consumer/producer**, not a
  parallel scaffolding system." Every decision above defends that: single `ServiceDef` output
  (not a file bundle), the typed `ServiceDef`-aware path (not free-form codegen), output that
  flows into the same renderer pipeline as hand-written projections.
- The central *new* unit of work is the **`ServiceDef` → Rust-builder source emitter** (D-03).
  It is the only genuinely new artifact-producing component; everything else is wiring
  existing ferro-mcp fns and the Phase 166 SDK path.
</specifics>

<deferred>
## Deferred Ideas

- **Embedding-based semantic relevance reranking** for context selection (`ferro_ai::embed` +
  `cosine_similarity`, Phase 167) — the true-semantic upgrade to D-02's lexical filter. Gate
  behind project size + provider embedding support. Belongs in a follow-up once large-project
  field testing shows lexical filtering is insufficient.
- **`ai_scaffold` / `ai_explain` MCP tool wrappers** — Phase 172 (explicitly out of scope here).
- **`make:json-view` v2 as the first concrete `Renderer`** over the produced `ServiceDef` +
  the AICLI-06 projection-roundtrip test — Phase 173.
- **`temperature` on `CompletionRequest`** (deterministic codegen) — carried over as a deferred
  SDK enhancement from Phase 170 D-05; would also benefit `ai:make` determinism but is not
  required to close this phase.

No reviewed-but-deferred todos — none matched.
</deferred>

---

*Phase: 171-ferro-ai-make-ferro-ai-explain-cli-commands*
*Context gathered: 2026-06-08*
