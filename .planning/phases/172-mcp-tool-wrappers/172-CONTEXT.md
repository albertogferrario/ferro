# Phase 172: MCP Tool Wrappers - Context

**Gathered:** 2026-06-08
**Status:** Ready for planning
**Mode:** `--auto` — all gray areas auto-selected, recommended option chosen per area (logged inline).

<domain>
## Phase Boundary

Expose two ferro-mcp tools so an in-process agent can invoke the Phase 171 killer-feature
logic without shelling out to the CLI:

- **`ai_scaffold`** — accepts `description: String`, returns a `ferro_projections::ServiceDef`
  as a JSON object. Same shape `ferro ai:make` produces. No `ScaffoldPlan` intermediary,
  no parallel surface, no multi-file scaffold.
- **`ai_explain`** — accepts `target: String` (route path, model name, or service name),
  returns the projection-framed explanation (`Intent`, `FieldMeaning`, `ActionDef`/`GuardDef`,
  `StateMachine`) as structured JSON when a `ServiceDef` exists for the target; plain prose
  fallback when none is found.

**In scope:** the two MCP tools, their registration + descriptions in `ferro-mcp`, and the
**extraction of the shared ServiceDef-producing / explanation core out of `ferro-cli` into
`ferro-mcp`** so the tools and the CLI commands run the same code path (SC#3). `ferro-mcp`
version bump. Full fmt/clippy/test gate.

**Not in scope (other phases):**
- `make:json-view` v2 (the first concrete `Renderer` over the `ServiceDef`) and the
  AICLI-06 projection-roundtrip test — **Phase 173**.
- Any change to the `ServiceDef` schema, the `ServiceDef`-aware normalizer, or the relevance
  algorithm itself — Phase 172 relocates that code unchanged, it does not redesign it.
- File-writing / `ServiceDef`→Rust-source emission — stays CLI-only (the MCP tool returns
  structured data, it never touches disk).
</domain>

<decisions>
## Implementation Decisions

### Shared-core location — the no-duplicate-implementation constraint (D-01)
- **D-01:** The dependency arrow is **`ferro-cli` → `ferro-mcp`** (`ferro-cli/Cargo.toml:45`;
  `ferro-mcp` does **not** depend on `ferro-cli`). `ferro-mcp` already depends on `ferro-ai`
  (`ferro-mcp/Cargo.toml:23`) and `ferro-projections` unconditionally (`:25`). Therefore the
  shared logic **must live in `ferro-mcp`**, and `ferro-cli` calls into it — the reverse
  would be a dependency cycle. Extract the Phase 171 core from
  `ferro-cli/src/commands/ai_make.rs::run` and `ai_explain.rs::run` into
  `ferro-mcp/src/tools/ai_scaffold.rs` and `ferro-mcp/src/tools/ai_explain.rs` as plain
  `async` functions returning typed `Result`s. The CLI `run` functions become thin
  presentation wrappers (env config, tokio bridge, stdout, file write, `process::exit`) that
  call the relocated core. The MCP tool methods call the same core.
  - **What moves into `ferro-mcp`:** the in-process introspection assembly (it already calls
    `ferro_mcp::tools::{list_models, list_routes, database_schema, generation_context,
    list_projections, inspect_projection}::execute` — these are *already in this crate*), the
    relevance filter (`ferro-cli/src/relevance.rs`, currently `pub(crate)` — relocate to
    `ferro-mcp`), prompt assembly, the `sanitize_description` prompt-injection guard, the
    `complete_with::<ServiceDef>()` call, and `ServiceDef::validate()`. For `ai:explain`:
    `resolve_target` (service → route → model) and the prose-completion path.
  - **What stays in `ferro-cli`:** the `ServiceDef`→Rust-builder-source emitter
    (`emit_service_def_source`, `ai_make.rs:30+`) and `mod.rs` registration — these are
    CLI-only file-output concerns the MCP tool does not need. `naming::to_snake_case` stays
    where it is unless the emitter is the only remaining consumer.
  - `[auto]` Shared core — chose **"extract core into `ferro-mcp`, CLI becomes thin wrapper"**
    over "new `ferro-ai-scaffold` shared crate" (extra publish surface, no benefit — `ferro-mcp`
    already has every dep) and over "duplicate the logic" (violates SC#3). Recommended:
    matches the existing dep graph, is the minimal structural change, and makes SC#3 a
    compile-time guarantee rather than a convention.

### `ai_scaffold` write semantics (D-02)
- **D-02:** `ai_scaffold` **never writes to disk** and returns the `ServiceDef` as a pretty
  JSON object only — equivalent to the CLI's `--dry-run` payload but as the tool's structured
  return value, not stdout text. File creation in `src/projections/<snake>.rs` and `mod.rs`
  registration remain exclusively in the CLI `ai:make` path. SC#1 says the tool "returns a
  `ServiceDef` JSON object" — returning, not persisting, is the correct MCP semantic; the
  agent decides what to do with the result.
  - `[auto]` Write semantics — chose **"return-only, no disk write"** over "write the file like
    the CLI does and also return it" (recommended: an MCP tool that silently mutates the
    project filesystem is surprising; keep the side-effecting write in the explicit CLI command).

### `ai_explain` output contract over MCP (D-03)
- **D-03:** The CLI `ai:explain` currently produces **prose only** (raw completion,
  `schema: None`, `ai_explain.rs`). SC#2 requires the MCP tool to return **structured
  projection JSON when a `ServiceDef` exists**. Reconcile as two branches sharing one
  resolution path:
  - **`ServiceDef` found** (target resolves to a service / a model or route backed by a
    `ServiceDef`): return the projection structure directly — `Intent` (from `intent_hints`),
    per-field `FieldMeaning`, `ActionDef`/`GuardDef`, `StateMachine` — as typed JSON. This is
    **deterministic and needs no LLM call** (the data is already what `inspect_projection`
    returns). No tokens spent.
  - **No `ServiceDef` found:** fall back to the LLM **prose** path — the exact code path the
    CLI uses (`resolve_target` → `build_*_prompt` → completion). Returned as a `{ "prose": "…" }`
    string field.
  - The **shared logic path with the CLI** (SC#3) is `resolve_target` + the prose-generation
    branch. The structured branch is the MCP-appropriate rendering of the already-resolved
    `ServiceDef`; the CLI may later adopt the same structured branch but that is out of scope
    here.
  - `[auto]` Explain contract — chose **"structured JSON from the resolved `ServiceDef` (no LLM),
    prose fallback via the shared CLI path"** over "always call the LLM and ask it to emit
    JSON" (recommended: the projection structure is already typed and free; reserve the LLM
    strictly for the no-`ServiceDef` prose fallback SC#2 describes).

### Error handling & runtime in MCP context (D-04)
- **D-04:** The relocated core returns `Result<T, E>` with **model-legible** error strings —
  no `std::process::exit`, no `console::style` coloring, no `eprintln!`. Those presentation
  concerns stay in the CLI wrapper. The MCP tool methods follow the existing crate pattern
  (`test_classifier` at `service.rs:1679`): build params, call the async core, serialize a
  result struct with a `success` flag + `error: Option<String>` (or serialize the `ServiceDef`
  directly on success) via `serde_json::to_string_pretty`, never panicking. `AiConfig::from_env`
  failure and LLM failure surface as tool errors, not crashes.
  - **Runtime:** MCP service methods are already `async` (rmcp runs inside tokio), so the core
    `async fn`s are awaited directly — the `tokio::runtime::Runtime::new()` + `block_on` bridge
    is **only** needed in the sync CLI `main` and stays in the CLI wrapper.
  - **Cost guard:** the Phase 171 `FERRO_AI_MAX_TOKENS_PER_COMMAND` env guard applies inside
    the relocated core, so it governs MCP invocations identically to CLI invocations.
  - `[auto]` Errors/runtime — chose **"`Result`-returning async core, structured tool-error
    serialization, bridge stays CLI-side"** over "let the core call `process::exit`/`eprintln`"
    (recommended: a library core must not terminate the MCP server process or write to its
    stderr; mirrors the established `test_classifier` tool shape).

### Tool naming, params & registration (D-05)
- **D-05:** MCP tool names are **`ai_scaffold`** and **`ai_explain`** (as named in AICLI-05
  and the roadmap) — a deliberate divergence from the CLI verbs `ai:make` / `ai:explain`
  (the requirement spells the scaffold tool `ai_scaffold`). Params:
  - `ai_scaffold` → `{ description: String }`
  - `ai_explain` → `{ target: String, type_override: Option<String> }` (mirrors the CLI's
    optional type disambiguator)
  - Register both with `#[tool(name = "…", description = "…")]` methods on the existing service
    impl in `ferro-mcp/src/service.rs`, alongside `test_classifier`. Descriptions must be
    self-sufficient for an agent (SC#4): when-to-use, that `ai_scaffold` returns a `ServiceDef`
    (not files) and `ai_explain` returns projection JSON or prose, the token-cost note (real
    LLM call), and a cross-link between the two and to `inspect_projection`/`list_projections`.
  - `[auto]` Naming — chose **"`ai_scaffold` / `ai_explain` (requirement-specified names)"** over
    "rename to match CLI `ai_make`" (recommended: the requirement and roadmap both write
    `ai_scaffold`; honor the contract, note the deliberate verb divergence in the tool doc).

### Versioning & gate (D-06)
- **D-06:** Bump the workspace version (`Cargo.toml:36`, currently `0.2.46`; `ferro-mcp` uses
  `version.workspace = true`) per SC#5. Run the full gate before commit:
  `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings &&
  cargo test --all-features`. Add unit tests for the relocated core in `ferro-mcp` and assert
  the CLI commands still pass their existing tests through the thin wrapper.

### Claude's Discretion
- Exact module layout inside `ferro-mcp/src/tools/` (one file per tool vs a shared `ai_scaffold`
  submodule housing the relocated relevance/prompt helpers) — planner's call, as long as the
  relevance filter and prompt assembly are reachable by both the tool and (transitively) the CLI.
- Whether `ai_explain`'s "model/route backed by a `ServiceDef`" detection reuses
  `list_projections` name-matching or adds a small resolver — either is fine if deterministic.
- Naming of the result wrapper structs and their serde shape, provided `success`/`error` and the
  `ServiceDef`/projection/prose payloads are present and round-trip.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope & requirements
- `.planning/ROADMAP.md` §"Phase 172: MCP Tool Wrappers" (lines ~1885–1895) — goal, 5 success criteria.
- `.planning/REQUIREMENTS.md` — **AICLI-05** (the single requirement; "no parallel surface").
- `.planning/phases/171-*/171-CONTEXT.md` — Phase 171 decisions this phase wraps (D-01 in-process
  introspection wiring, D-02 relevance filter, D-03 output artifact, D-04 `complete_with` + cost guard).

### Code to extract / relocate (the CLI core)
- `ferro-cli/src/commands/ai_make.rs` — `run()` (the sequence to split: introspection assembly →
  relevance filter → prompt + `sanitize_description` → `complete_with::<ServiceDef>()` → `validate()`).
  `emit_service_def_source` and the file-output path stay CLI-side.
- `ferro-cli/src/commands/ai_explain.rs` — `run()`, `resolve_target`, `build_service_prompt` /
  `build_route_prompt` / `build_model_prompt`, `ResolvedTarget` enum.
- `ferro-cli/src/relevance.rs` — `tokenize`, `Candidate`, `select_relevant`, `INPUT_BUDGET_CHARS`
  (`pub(crate)` today; relocate to `ferro-mcp`).

### MCP integration points (where the tools land)
- `ferro-mcp/src/service.rs` — `#[tool(...)]` registration pattern; copy the shape of
  `test_classifier` (`:1671–1688`) and `list_pending_confirmations` (`:1691–1705`).
- `ferro-mcp/src/tools/ai.rs` — existing AI tool module + `mod.rs` wiring conventions.
- `ferro-mcp/src/tools/{list_models,list_routes,database_schema,generation_context,list_projections,inspect_projection}.rs`
  — the introspection `execute()` functions the core already consumes (now same-crate).
- `ferro-mcp/Cargo.toml` (`ferro-ai` :23, `ferro-projections` :25) and `ferro-cli/Cargo.toml`
  (`ferro-mcp` :45) — confirm the dep direction that pins the core into `ferro-mcp`.

### Projection contract
- `ferro-projections/src/service.rs` — `ServiceDef` (derives `JsonSchema` at `:62`), `validate()`.
- `ferro-projections/src/intent.rs` — `Intent` enum; `ferro-projections/src/lib.rs` — `FieldMeaning`,
  `ActionDef`, `GuardDef`, `StateMachine`, `Cardinality` (the `ai_explain` structured payload shapes).

### Conventions
- `CLAUDE.md` §"Rendering Architecture" / "MCP" — MCP tool descriptions and accuracy are held to
  the same quality bar as the Rust API (relevant to SC#4).
- Workspace gate: `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`ferro-mcp` introspection `execute()` fns** — already the in-process surface Phase 171's CLI
  core calls; relocating the core into `ferro-mcp` makes these same-crate (no path change).
- **`test_classifier` tool** (`service.rs:1671`) — exact template for an AI MCP tool: `#[tool]`
  macro, `Parameters<…Params>`, async core call, `to_string_pretty` result, no panic on error.
- **`ferro_ai::complete_with::<ServiceDef>()`** (Phase 171 / `ferro-ai/src/complete.rs`) — the
  `ServiceDef`-aware structured-output entry; the core calls it unchanged.
- **`ferro-cli/src/relevance.rs`** — deterministic lexical relevance filter; relocate verbatim.

### Established Patterns
- MCP tools return `String` (pretty JSON); errors are encoded in the JSON, never thrown — the
  server process must not exit. CLI presentation (`console::style`, `eprintln!`, `process::exit`)
  belongs only in `ferro-cli`.
- `ferro-mcp` depends on `ferro-projections` **unconditionally**, so `ServiceDef` is always in
  scope here — no `feature = "projections"` gate is needed inside `ferro-mcp` (the gate exists
  only in `ferro-cli`).

### Integration Points
- Two new `#[tool]` methods on the service impl in `ferro-mcp/src/service.rs`.
- Two new modules in `ferro-mcp/src/tools/` (+ `mod.rs` registration).
- `ferro-cli/src/commands/ai_make.rs` and `ai_explain.rs` `run()` bodies replaced by thin calls
  into the relocated `ferro-mcp` core.
- Workspace version bump in root `Cargo.toml`.
</code_context>

<specifics>
## Specific Ideas

- SC#3 ("share the same logic path — no duplicate implementation") is treated as a **structural**
  requirement: the core has exactly one definition site (in `ferro-mcp`), so duplication is
  impossible by construction, not by discipline.
- `ai_explain`'s structured branch should spend **zero tokens** when a `ServiceDef` exists — the
  projection data is already typed introspection output.
</specifics>

<deferred>
## Deferred Ideas

- Having the CLI `ai:explain` also emit the structured projection JSON (currently prose-only) —
  the MCP tool gets it first; aligning the CLI is a later polish, not in this phase.
- Embedding-based relevance reranking (already deferred in 171-CONTEXT D-02) — unchanged here.

None of the above are blockers; discussion stayed within phase scope.

</deferred>

---

*Phase: 172-mcp-tool-wrappers*
*Context gathered: 2026-06-08*
