# Phase 172: MCP Tool Wrappers - Research

**Researched:** 2026-06-08
**Domain:** ferro-mcp tool registration; ferro-cli core extraction; ferro-projections ServiceDef serialization
**Confidence:** HIGH — all findings verified directly from source files in this session.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Core logic lives in `ferro-mcp`. `ferro-cli` calls into it. No reverse dep (would be a cycle). Move: relevance filter, prompt assembly, `sanitize_description`, `complete_with::<ServiceDef>()`, `validate()`, `resolve_target`, prose-completion path. Keep in CLI: `emit_service_def_source`, `render_output`, `mod.rs` registration.
- **D-02:** `ai_scaffold` MCP tool is return-only — never writes to disk.
- **D-03:** Structured projection JSON (zero LLM tokens) when a `ServiceDef` exists; `{ "prose": "..." }` field for the LLM fallback. Shared `resolve_target` + prose path with CLI (SC#3).
- **D-04:** Core returns `Result<T, E>` with model-legible errors. No `process::exit`, no `console::style`, no `eprintln!` in the core. MCP tool methods serialize a `success` flag + `error: Option<String>` and never panic. `tokio::runtime::Runtime::new()` + `block_on` bridge stays CLI-only. Cost guard `FERRO_AI_MAX_TOKENS_PER_COMMAND` governs both CLI and MCP invocations.
- **D-05:** Tool names are `ai_scaffold` (not `ai_make`) and `ai_explain`. Params: `ai_scaffold { description: String }`, `ai_explain { target: String, type_override: Option<String> }`. Register with `#[tool(name = "...", description = "...")]` on the service impl. Descriptions must be self-sufficient for an agent (SC#4).
- **D-06:** Bump workspace version (`Cargo.toml`, currently `0.2.46`). Run full gate: `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`. Unit tests for relocated core in `ferro-mcp`; existing CLI tests must pass through the thin wrapper.

### Claude's Discretion

- Exact module layout inside `ferro-mcp/src/tools/` (one file per tool vs shared submodule for relevance/prompt helpers).
- Whether `ai_explain`'s "model/route backed by a ServiceDef" detection reuses `list_projections` name-matching or adds a small resolver.
- Naming of result wrapper structs and their serde shape (provided `success`/`error` and the payloads are present).

### Deferred Ideas (OUT OF SCOPE)

- Having CLI `ai:explain` emit structured projection JSON (MCP tool gets it first).
- Embedding-based relevance reranking.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AICLI-05 | MCP tools `ai_scaffold` and `ai_explain` in `ferro-mcp` wrap the CLI command logic for in-process agent consumption. No parallel surface. | D-01 establishes the extraction plan; service.rs `#[tool]` pattern established; ServiceDef serializes cleanly; all introspection `execute()` fns are already in `ferro-mcp`. |
</phase_requirements>

---

## Summary

Phase 172 extracts the Phase 171 CLI core out of `ferro-cli` into `ferro-mcp` and exposes it as two new MCP tools: `ai_scaffold` and `ai_explain`. The dependency direction is `ferro-cli → ferro-mcp` (confirmed in `ferro-cli/Cargo.toml:45`); `ferro-mcp` does not and cannot depend on `ferro-cli`. The shared core therefore must live in `ferro-mcp`, making the "no duplicate implementation" constraint a compile-time guarantee.

The extraction boundary is clean. Everything reusable is already a well-factored function in the CLI command files: `sanitize_description`, `resolve_max_tokens`, `ai_config_error_message`, `build_service_prompt`, `build_route_prompt`, `build_model_prompt`, `resolve_target`, and the entire candidate-assembly + relevance-filter sequence. The emitter (`emit_service_def_source`, `render_output`) and file-output path remain CLI-only.

`ServiceDef` derives `Serialize`/`Deserialize`/`JsonSchema` (verified in `ferro-projections/src/service.rs:62`). The MCP tool can return it directly as `serde_json::to_string_pretty`. The `ai_explain` structured branch is zero-cost: `inspect_projection::execute()` already returns a `ProjectionDetail` whose fields are the exact projection vocabulary SC#2 requires. No LLM call is needed for that branch.

**Primary recommendation:** Extract CLI core into two new files `ferro-mcp/src/tools/ai_scaffold.rs` and `ferro-mcp/src/tools/ai_explain_core.rs` (or a shared `ferro-mcp/src/tools/ai_core/` submodule), relocate `relevance.rs` verbatim, add two `#[tool]` methods to the service impl, and replace both CLI `run()` bodies with thin wrappers that call the relocated core.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| ServiceDef production (NL → ServiceDef) | `ferro-mcp` (library core) | `ferro-cli` (thin presentation wrapper) | Dep direction mandates core in `ferro-mcp`; CLI owns only sync bridge + file write |
| Relevance filter (tokenize/select) | `ferro-mcp` | — | Shared by both MCP tool and CLI; must live in the crate both depend on |
| Prompt assembly + sanitization | `ferro-mcp` | — | Same reasoning as relevance filter |
| LLM completion (`complete_with`) | `ferro-ai` | called from `ferro-mcp` | Already public in `ferro-ai::complete_with`; `ferro-mcp` depends on `ferro-ai` unconditionally |
| Target resolution (route/model/service) | `ferro-mcp` | — | Calls same-crate introspection `execute()` fns |
| Structured projection JSON output | `ferro-mcp` (via `inspect_projection::execute`) | — | Zero-token branch; deterministic from the already-typed `ProjectionDetail` |
| File write / mod.rs registration | `ferro-cli` | — | D-02: MCP tool never writes disk; only CLI `ai:make` does |
| Error presentation (color, exit codes) | `ferro-cli` | — | D-04: `console::style`, `eprintln!`, `process::exit` are CLI-only |
| Tokio runtime bridge | `ferro-cli` | — | MCP service is already async; no bridge needed in `ferro-mcp` |

---

## Standard Stack

### Core

| Library | Version | Purpose | Notes |
|---------|---------|---------|-------|
| `rmcp` | 0.12 | MCP server runtime; provides `#[tool]`, `#[tool_router]`, `Parameters<T>` | [VERIFIED: ferro-mcp/Cargo.toml:13] |
| `ferro-ai` | workspace 0.2 | `AiConfig::from_env()`, `complete_with::<T>()`, `CompleteOptions`, `CompletionRequest` | [VERIFIED: ferro-mcp/Cargo.toml:23; ferro-ai/src/lib.rs] |
| `ferro-projections` | workspace 0.2 | `ServiceDef`, `FieldMeaning`, `Intent`, `ActionDef`, `GuardDef`, `StateMachine` | [VERIFIED: ferro-mcp/Cargo.toml:25 — unconditional dep] |
| `serde_json` | 1 | `to_string_pretty` for all tool returns | [VERIFIED: ferro-mcp/Cargo.toml] |
| `schemars` | 1 | `#[derive(JsonSchema)]` on `*Params` structs | [VERIFIED: ferro-mcp/Cargo.toml] |

### Already in ferro-mcp (no new deps needed)

`ferro-mcp` already depends on `ferro-ai` and `ferro-projections` unconditionally. No new crate entries needed in `ferro-mcp/Cargo.toml`. `ServiceDef` is unconditionally in scope — no `feature = "projections"` gate is needed inside `ferro-mcp` (the gate exists only in `ferro-cli` where `ferro-projections` is an optional dep).

---

## Architecture Patterns

### System Architecture Diagram

```
[ Agent (MCP client) ]
        |
        | MCP call: ai_scaffold { description }
        |           ai_explain  { target }
        v
[ FerroMcpService — service.rs ]
  #[tool_router] impl
    ai_scaffold()  ──────────────────────────────────────────────┐
    ai_explain()   ──────────────────────────────────────────┐   |
        |                                                    |   |
        v                                                    v   v
[ ferro-mcp/src/tools/ai_scaffold.rs ]       [ ferro-mcp/src/tools/ai_explain_core.rs ]
  scaffold_core(description, project_root)     explain_core(target, type_override, root)
        |                                            |
        |         ┌─────────────────────────────────┤
        |         | resolve_target()                |
        |         |   inspect_projection::execute() ← service branch
        |         |   explain_route::execute()       ← route branch
        |         |   explain_model::execute()       ← model branch
        |         |                                 |
        |         | structured branch (ServiceDef found):
        |         |   → return ProjectionDetail JSON directly (0 LLM tokens)
        |         |                                 |
        |         | prose fallback branch:
        |         |   build_{service,route,model}_prompt()
        |         |   AiConfig::from_env() → LlmClient
        |         |   client.complete(CompletionRequest { schema: None })
        |         |   → return { "prose": "..." }
        |         └──────────────────────────────────
        |
[ ferro-mcp/src/tools/relevance.rs ]  ← relocated from ferro-cli/src/relevance.rs verbatim
  tokenize(), Candidate, select_relevant(), INPUT_BUDGET_CHARS
        |
        v
[ introspection execute() fns — same crate ]
  list_models::execute(root)              ← sync
  generation_context::execute()           ← sync
  list_projections::execute(root, None)   ← sync
  list_routes::execute(root).await        ← async
  database_schema::execute(root, None).await ← async
        |
        v
[ ferro-ai ]
  AiConfig::from_env() → Box<dyn LlmClient>
  complete_with::<ServiceDef>(client, prompt, CompleteOptions)
        |
        v
[ LLM Provider (Anthropic/OpenAI/Groq/Ollama) ]

— — — — — — — — — — — — — — — — — — — — — —

[ ferro-cli/src/commands/ai_make.rs::run() ] (thin wrapper)
  tokio::runtime::Runtime::new()
  ferro_mcp::tools::ai_scaffold::scaffold_core(description, root)?
  render_output()  ← stays CLI-only
  console::style / eprintln! / process::exit ← stays CLI-only

[ ferro-cli/src/commands/ai_explain.rs::run() ] (thin wrapper)
  tokio::runtime::Runtime::new()
  ferro_mcp::tools::ai_explain_core::explain_core(target, type_override, root).await?
  println! / eprintln! / process::exit ← stays CLI-only
```

### Recommended Project Structure Changes

```
ferro-mcp/src/tools/
├── ai.rs                     # existing: test_classifier, list_pending_confirmations
├── ai_scaffold.rs            # NEW: scaffold_core() async fn + candidate assembly
├── ai_explain_core.rs        # NEW: explain_core() async fn, resolve_target, build_*_prompt
├── relevance.rs              # NEW: relocated verbatim from ferro-cli/src/relevance.rs
├── mod.rs                    # ADD: pub mod ai_scaffold; pub mod ai_explain_core; pub mod relevance;
...
ferro-mcp/src/service.rs
  # ADD: AiScaffoldParams, AiExplainParams structs (with #[derive(Debug,Clone,Deserialize,Serialize,JsonSchema)])
  # ADD: ai_scaffold() and ai_explain() tool methods in #[tool_router] impl

ferro-cli/src/commands/ai_make.rs
  # run() becomes thin wrapper calling ferro_mcp::tools::ai_scaffold::scaffold_core()
  # emit_service_def_source, render_output, resolve_projection_path stay

ferro-cli/src/commands/ai_explain.rs
  # run() becomes thin wrapper calling ferro_mcp::tools::ai_explain_core::explain_core()
  # build_*_prompt now live in ferro-mcp; CLI calls them via the thin wrapper result

ferro-cli/src/relevance.rs
  # DELETE (or keep as a re-export of ferro_mcp::tools::relevance if needed for CLI tests)
```

### Pattern 1: Tool Method Registration

The exact template from `ferro-mcp/src/service.rs:1671–1688`:

```rust
// Source: ferro-mcp/src/service.rs lines 1671-1688 [VERIFIED]

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AiScaffoldParams {
    /// Natural-language description of the service to scaffold.
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AiExplainParams {
    /// Target name: route path, model name, or service name.
    pub target: String,
    /// Optional type hint to skip auto-detect: "service", "route", or "model".
    pub type_override: Option<String>,
}

// In the #[tool_router] impl block:
#[tool(
    name = "ai_scaffold",
    description = "Generate a ferro_projections::ServiceDef from a natural-language \
        description using the project's live introspection as context.\n\n\
        **When to use:** Starting a new service; getting a typed ServiceDef to pass \
        to renderers or inspect_projection.\n\n\
        **Returns:** A ServiceDef JSON object (same shape as the CLI `ferro ai:make` \
        output). Does NOT write files — use `ferro ai:make` when you want the .rs file \
        written to src/projections/.\n\n\
        **Note:** Makes a real LLM API call. Costs tokens. Requires FERRO_AI_PROVIDER, \
        FERRO_AI_API_KEY, FERRO_AI_MODEL.\n\n\
        **Combine with:** `inspect_projection` to see existing projections, \
        `list_projections` to avoid naming collisions, `ai_explain` to understand \
        a generated ServiceDef."
)]
pub async fn ai_scaffold(&self, params: Parameters<AiScaffoldParams>) -> String {
    let result = tools::ai_scaffold::scaffold_core(
        &params.0.description,
        &self.project_root,
    ).await;
    match result {
        Ok(service_def) => serde_json::to_string_pretty(&service_def)
            .unwrap_or_else(|_| "{}".to_string()),
        Err(e) => serde_json::to_string_pretty(&serde_json::json!({
            "success": false,
            "error": e
        }))
        .unwrap_or_else(|_| r#"{"success":false}"#.to_string()),
    }
}
```

### Pattern 2: Async Core Function Signature

```rust
// Source: inferred from ferro-cli/src/commands/ai_make.rs::run() extraction plan [VERIFIED structure]

// ferro-mcp/src/tools/ai_scaffold.rs
pub async fn scaffold_core(
    description: &str,
    project_root: &std::path::Path,
) -> Result<ferro_projections::ServiceDef, String> {
    // 1. AiConfig::from_env() → Result, map Err to String
    // 2. Call sync introspection fns directly (no block_on needed — already async context)
    // 3. Await async introspection fns with .await
    // 4. Build candidates → call relevance::select_relevant()
    // 5. Assemble prompt, sanitize_description()
    // 6. complete_with::<ServiceDef>(...).await → map Err to String
    // 7. service.validate() → map Err to String
    // 8. Return Ok(service)
}
```

### Pattern 3: ai_explain Structured-vs-Prose Branch

```rust
// Source: D-03 from CONTEXT.md + inspect_projection.rs [VERIFIED]

pub async fn explain_core(
    target: &str,
    type_override: Option<&str>,
    project_root: &std::path::Path,
) -> Result<serde_json::Value, String> {
    let resolved = resolve_target(project_root, target, type_override).await;
    match resolved {
        ResolvedTarget::Service(detail) => {
            // Zero-token branch: return ProjectionDetail directly
            // ProjectionDetail derives Serialize — serde_json::to_value() works
            Ok(serde_json::to_value(&detail).map_err(|e| e.to_string())?)
        }
        ResolvedTarget::Route(r) => {
            // LLM prose branch
            let (sys, user) = build_route_prompt(&r);
            let prose = call_llm_prose(sys, user, project_root).await?;
            Ok(serde_json::json!({ "prose": prose }))
        }
        ResolvedTarget::Model(m) => {
            let (sys, user) = build_model_prompt(&m);
            let prose = call_llm_prose(sys, user, project_root).await?;
            Ok(serde_json::json!({ "prose": prose }))
        }
        ResolvedTarget::NotFound(msg) => Err(msg),
    }
}
```

**Key difference from CLI:** `resolve_target` is now `async` (no `tokio::runtime::Runtime` needed; the inner `explain_route::execute` and `explain_model::execute` are already async — previously bridged with `rt.block_on`, now simply awaited). `inspect_projection::execute()` is still sync and called directly.

### Anti-Patterns to Avoid

- **Tokio bridge in the core:** `Runtime::new()` and `block_on` must stay in the CLI `run()` wrappers only. Inside `ferro-mcp` everything is already async.
- **`process::exit` in the core:** The MCP server process must not exit on tool errors. Return `Result<_, String>` and encode errors in the JSON output.
- **`console::style` / `eprintln!` / colored output in the core:** These are CLI presentation concerns.
- **Writing files from MCP tool:** `ai_scaffold` returns the `ServiceDef` only. `render_output` and `emit_service_def_source` stay in the CLI.
- **Feature-gating `ServiceDef` in `ferro-mcp`:** `ferro-mcp` depends on `ferro-projections` unconditionally — no `#[cfg(feature = "projections")]` guard needed. The guard exists only in `ferro-cli`.
- **Duplicating `relevance.rs`:** Move it once to `ferro-mcp/src/tools/relevance.rs` and have `ferro-cli` call through `ferro_mcp::tools::relevance::*`. Do not maintain two copies.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| MCP tool registration | Custom dispatch table | `rmcp::tool` / `rmcp::tool_router` macros | Already used by all 50+ existing tools in the service |
| JSON Schema for params | Manual schema strings | `#[derive(JsonSchema)]` via `schemars` | Required by rmcp `Parameters<T>` |
| Structured LLM output | Manual JSON parsing | `ferro_ai::complete_with::<ServiceDef>()` | Already handles schema normalization, deserialization, `ServiceDef`-aware normalizer |
| Relevance filter | New algorithm | Relocate `ferro-cli/src/relevance.rs` verbatim | Proven, tested, matches CONTEXT D-02 |
| ServiceDef serialization | Custom serializer | `serde_json::to_string_pretty(&service_def)` | `ServiceDef` already derives `Serialize` |
| Projection detail extraction | New parser | `inspect_projection::execute()` | Already returns `ProjectionDetail` with all SC#2 fields |

---

## Extraction Boundary: What Moves Where

### Moves into ferro-mcp (verbatim or near-verbatim)

| Symbol | Current Location | Destination | Notes |
|--------|-----------------|-------------|-------|
| `sanitize_description()` | `ferro-cli/src/commands/ai_make.rs:473` | `ferro-mcp/src/tools/ai_scaffold.rs` | Already `pub(crate)`, no deps change |
| `resolve_max_tokens()` | `ferro-cli/src/commands/ai_make.rs:373` | `ferro-mcp/src/tools/ai_scaffold.rs` | Reads env; no deps |
| `resolve_max_tokens_with_default()` | `ferro-cli/src/commands/ai_explain.rs:267` | `ferro-mcp/src/tools/ai_explain_core.rs` | Reads env; no deps |
| `ai_config_error_message()` | `ferro-cli/src/commands/ai_make.rs:384` | `ferro-mcp/src/tools/ai_scaffold.rs` or shared | Depends on `ferro_ai::Error` — already a dep |
| Candidate assembly loop (models/routes/projections/schema) | `ai_make.rs::run()` lines 543-616 | `ferro-mcp/src/tools/ai_scaffold.rs::scaffold_core()` | Calls same-crate introspection |
| Prompt assembly (`gen_ctx_text`, `context_block`, `user_prompt`) | `ai_make.rs::run()` lines 619-647 | `ferro-mcp/src/tools/ai_scaffold.rs::scaffold_core()` | No external deps |
| `ResolvedTarget` enum | `ferro-cli/src/commands/ai_explain.rs:24` | `ferro-mcp/src/tools/ai_explain_core.rs` | Uses `ProjectionDetail`, `RouteExplanation`, `ModelExplanation` — all already in `ferro-mcp` |
| `resolve_kind_priority()` | `ferro-cli/src/commands/ai_explain.rs:47` | `ferro-mcp/src/tools/ai_explain_core.rs` | Pure logic, no deps |
| `resolve_target()` | `ferro-cli/src/commands/ai_explain.rs:78` | `ferro-mcp/src/tools/ai_explain_core.rs` | Calls `inspect_projection`, `explain_route`, `explain_model` — all same-crate in dest |
| `build_service_prompt()` | `ferro-cli/src/commands/ai_explain.rs:141` | `ferro-mcp/src/tools/ai_explain_core.rs` | Uses `ProjectionDetail` — same-crate in dest |
| `build_route_prompt()` | `ferro-cli/src/commands/ai_explain.rs:212` | `ferro-mcp/src/tools/ai_explain_core.rs` | Uses `RouteExplanation` — same-crate in dest |
| `build_model_prompt()` | `ferro-cli/src/commands/ai_explain.rs:235` | `ferro-mcp/src/tools/ai_explain_core.rs` | Uses `ModelExplanation` — same-crate in dest |
| `tokenize()`, `Candidate`, `select_relevant()`, `INPUT_BUDGET_CHARS` | `ferro-cli/src/relevance.rs` | `ferro-mcp/src/tools/relevance.rs` | Currently `pub(crate)` — must become `pub` for CLI re-use |

### Stays in ferro-cli

| Symbol | Location | Reason |
|--------|----------|--------|
| `emit_service_def_source()` | `ai_make.rs:30` | CLI-only: generates Rust builder source for file write |
| `render_output()` | `ai_make.rs:412` | CLI-only: manages file creation, mod.rs registration |
| `resolve_projection_path()` | `ai_make.rs:357` | Used only by `render_output` |
| `emit_*` helpers (`emit_data_type`, `emit_field_meaning`, etc.) | `ai_make.rs` | All feed into `emit_service_def_source` |
| `OutputResult` enum | `ai_make.rs:399` | Used by `render_output` |
| `naming::to_snake_case` | `ferro-cli/src/naming.rs` | Used by `emit_service_def_source` only — stays |
| `naming::is_valid_identifier` | `ferro-cli/src/naming.rs` | Used by `resolve_projection_path` only — stays |
| `run(description, dry_run)` | `ai_make.rs:490` | Thin wrapper: tokio bridge + call `scaffold_core` + `render_output` + presentation |
| `run(target, type_override, dry_run)` | `ai_explain.rs:284` | Thin wrapper: tokio bridge + call `explain_core` + `println!`/`process::exit` |

---

## Common Pitfalls

### Pitfall 1: resolve_target async transition
**What goes wrong:** The CLI's `resolve_target` accepts a `&tokio::runtime::Runtime` and uses `rt.block_on(explain_route::execute(...))` for async calls. In `ferro-mcp` the core is already in an async context.
**Why it happens:** The CLI is sync (`main` is not async); it needs the runtime bridge. The MCP service runs inside tokio.
**How to avoid:** In the relocated `resolve_target`, replace `rt.block_on(explain_route::execute(...))` with `explain_route::execute(...).await`. Remove the `rt` parameter entirely.
**Warning signs:** If you keep the `rt: &tokio::runtime::Runtime` parameter, you get a signature mismatch and the CLI thin wrapper needs to supply a runtime — adding needless complexity.

### Pitfall 2: Feature gate mismatch
**What goes wrong:** Copy-pasting `#[cfg(feature = "projections")]` guards from `ferro-cli` into `ferro-mcp`.
**Why it happens:** The CLI gates `ferro-projections` as an optional dep. In `ferro-mcp`, `ferro-projections` is unconditional.
**How to avoid:** No `#[cfg(feature = "projections")]` anywhere in `ferro-mcp`. The gate stays only in `ferro-cli` (the CLI thin wrappers that call into the relocated core already live inside the `#[cfg(feature = "projections")]` block).

### Pitfall 3: Visibility of relocated relevance types
**What goes wrong:** `tokenize`, `Candidate`, `select_relevant`, `INPUT_BUDGET_CHARS` are currently `pub(crate)` in `ferro-cli`. After relocation to `ferro-mcp`, the CLI thin wrapper imports from `ferro_mcp::tools::relevance` — these must be `pub`.
**How to avoid:** Change `pub(crate)` to `pub` when copying to `ferro-mcp/src/tools/relevance.rs`.

### Pitfall 4: ServiceDef validate() test in MCP context
**What goes wrong:** Writing tests for `scaffold_core` that call `service.validate()` without knowing its return type.
**Clarification (verified):** `ServiceDef::validate()` is defined in `ferro-projections/src/service.rs` and returns `Result<Vec<Warning>, _>` (the `Warning` type is in `ferro-projections/src/state.rs`). The core maps validation failure to `Err(String)`.

### Pitfall 5: ai_explain structured branch field names
**What goes wrong:** Assuming `ProjectionDetail` exposes `Intent` enum values or typed `ActionDef`s. It uses string representations.
**Actual shape (verified in `ferro-mcp/src/tools/inspect_projection.rs`):**
```
ProjectionDetail {
  name: String,
  file: String,
  service_name: String,
  display_name: Option<String>,
  fields: Vec<FieldInfo { name, data_type, meaning, readable, writable }>,
  relationships: Vec<String>,
  actions: Vec<String>,
  has_state_machine: bool,
  intent_hints: Vec<String>,   // e.g. "Primary(Browse)"
}
```
This is already serializable via `Serialize` derive. The "structured JSON" branch returns this directly with `serde_json::to_value(&detail)` — no typed `Intent` or `ActionDef` structs in the output, just the string vocabulary from `inspect_projection`. SC#2's requirement for "Intent, FieldMeaning, ActionDef/GuardDef, StateMachine" is satisfied by the field names in `ProjectionDetail`/`FieldInfo`.

### Pitfall 6: Tests for relocated core use ENV_LOCK
**What goes wrong:** Tests touching `FERRO_AI_MAX_TOKENS_PER_COMMAND` or other env vars race when run in parallel.
**How to avoid:** The CLI already uses `crate::commands::ENV_LOCK` (a `Mutex<()>`). Relocate this pattern to `ferro-mcp` tests — add an `ENV_LOCK` in `ferro-mcp` (e.g. `static ENV_LOCK: Mutex<()> = Mutex::new(())`) and use it in env-touching tests.

### Pitfall 7: Naming — ai_scaffold vs ai_make
**What goes wrong:** Using the CLI verb `ai_make` as the MCP tool name.
**Clarification:** D-05 specifies `ai_scaffold` (the AICLI-05 requirement spelling). The deliberate divergence from `ai:make` should be noted in the tool description.

---

## Code Examples

### Tool params struct (verified pattern from service.rs lines 332-348)

```rust
// Source: ferro-mcp/src/service.rs:332-348 [VERIFIED]
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct TestClassifierParams {
    pub system_prompt: String,
    pub user_prompt: String,
    pub schema_json: String,
    pub model: Option<String>,
}

// New params follow exactly the same shape:
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AiScaffoldParams {
    /// Natural-language description of the service to scaffold.
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AiExplainParams {
    /// Target: route path (e.g. "/orders/{id}"), model name, or service name.
    pub target: String,
    /// Optional type hint: "service", "route", or "model". Auto-detect if omitted.
    pub type_override: Option<String>,
}
```

### Introspection call sync/async summary (verified)

```rust
// Source: ferro-cli/src/commands/ai_make.rs:524-541 [VERIFIED]

// SYNC (call directly in async context — no .await):
let models = list_models::execute(root).unwrap_or_default();
let gen_ctx = generation_context::execute();
let projections = list_projections::execute(root, None);

// ASYNC (previously bridged with rt.block_on, now simply .await):
let routes = list_routes::execute(root).await.unwrap_or_else(|_| RoutesInfo { .. });
let schema = database_schema::execute(root, None).await.unwrap_or_else(|_| SchemaInfo { .. });
```

### AiConfig::from_env signature (verified)

```rust
// Source: ferro-ai/src/config.rs:44 [VERIFIED]
pub fn from_env() -> Result<Box<dyn LlmClient>, Error>
```

Returns `Box<dyn LlmClient>`. Call `.as_ref()` or `client.as_ref()` to get `&dyn LlmClient` for `complete_with`.

### complete_with signature (verified)

```rust
// Source: ferro-ai/src/complete.rs:79 [VERIFIED]
pub async fn complete_with<T>(
    client: &dyn LlmClient,
    prompt: &str,
    opts: CompleteOptions,
) -> Result<T, Error>
where
    T: schemars::JsonSchema + serde::de::DeserializeOwned,
```

`CompleteOptions { max_tokens: u32, system: Option<String>, model_override: Option<String> }`.

---

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| CLI `run()` owns all logic including introspection | Core in `ferro-mcp`; CLI is thin wrapper | Compile-time "no duplicate" guarantee |
| `rt.block_on` for async introspection in sync CLI | Direct `.await` in async MCP tool core | Cleaner; no nested runtime risk |
| CLI context only (no in-process MCP tool) | Both CLI and MCP tool call same core | Agent can invoke without shelling out |

---

## Assumptions Log

> All claims in this research were verified by reading source files directly in this session.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | — | — | — |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

---

## Open Questions (RESOLVED)

1. **`resolve_target` async signature — runtime parameter** — **RESOLVED**
   - What we know: CLI passes `rt: &Runtime` to bridge async calls. In `ferro-mcp` this becomes a direct `.await`.
   - What's unclear: Whether any CLI test directly calls `resolve_target` with a real runtime (would need the test to also change).
   - **RESOLVED:** Make `resolve_target` async in the relocated core (no `rt` param). Any existing CLI test that exercises `resolve_target` / `build_*_prompt` migrates to `ferro-mcp` under `#[tokio::test]` alongside the relocated implementation; the CLI keeps only thin-wrapper behavior tests. Adopted by Plan 02 (relocation) and Plan 04 Task 1 (CLI rewire + test migration).

2. **`relevance.rs` in CLI after relocation** — **RESOLVED**
   - What we know: `ferro-cli/src/relevance.rs` tests are in the same file and use `pub(crate)` items.
   - What's unclear: Whether to delete `ferro-cli/src/relevance.rs` entirely and import from `ferro_mcp::tools::relevance`, or keep the file as a thin re-export.
   - **RESOLVED:** Delete `ferro-cli/src/relevance.rs` (no re-export shim) and update `ferro-cli/src/commands/ai_make.rs` to import `ferro_mcp::tools::relevance::{tokenize, Candidate, select_relevant, INPUT_BUDGET_CHARS}`. The relevance tests move to `ferro-mcp` alongside the implementation. Adopted by Plan 01 (relocate, `pub`) and Plan 04 Task 1 (delete CLI copy + reimport).

---

## Environment Availability

Phase 172 is purely code changes within the workspace. No external tools, databases, or services beyond the existing project build chain.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` / `rustc` | Build gate | ✓ | workspace rust-version 1.88.0 | — |
| `FERRO_AI_*` env vars | AI tool execution (not build) | Not checked — runtime only | — | Tests use mock clients |

---

## Validation Architecture

`workflow.nyquist_validation` is absent in `.planning/config.json` — treat as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` / `#[tokio::test]` (tokio = "1", features = ["full"] in ferro-mcp/Cargo.toml) |
| Config file | Cargo workspace — no separate test config |
| Quick run command | `cargo test -p ferro-mcp --all-features` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AICLI-05 | `ai_scaffold` returns valid `ServiceDef` JSON (mock LLM) | unit | `cargo test -p ferro-mcp --all-features test_ai_scaffold` | ❌ Wave 0 |
| AICLI-05 | `ai_explain` returns structured JSON when ServiceDef found | unit | `cargo test -p ferro-mcp --all-features test_ai_explain_structured` | ❌ Wave 0 |
| AICLI-05 | `ai_explain` returns `{ "prose": ... }` fallback when no ServiceDef | unit | `cargo test -p ferro-mcp --all-features test_ai_explain_prose_fallback` | ❌ Wave 0 |
| AICLI-05 | Relevance filter `tokenize`/`select_relevant` (relocated) | unit | `cargo test -p ferro-mcp --all-features relevance` | ❌ Wave 0 |
| AICLI-05 | `sanitize_description` strips XML delimiters (relocated) | unit | `cargo test -p ferro-mcp --all-features sanitize` | ❌ Wave 0 |
| AICLI-05 | CLI `ai:make` thin wrapper still passes existing tests | unit | `cargo test -p ferro-cli --all-features` | ✅ (existing) |
| AICLI-05 | CLI `ai:explain` thin wrapper still passes existing tests | unit | `cargo test -p ferro-cli --all-features` | ✅ (existing) |
| AICLI-05 | Full gate green | integration | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | N/A |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-mcp --all-features && cargo test -p ferro-cli --all-features`
- **Per wave merge:** `cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full gate green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-mcp/src/tools/relevance.rs` — relocated relevance tests
- [ ] `ferro-mcp/src/tools/ai_scaffold.rs` — unit tests for `sanitize_description`, `resolve_max_tokens`, candidate assembly (mock introspection)
- [ ] `ferro-mcp/src/tools/ai_explain_core.rs` — unit tests for `build_*_prompt`, `resolve_kind_priority` (relocated from CLI), structured vs prose branch

Note: all existing CLI tests for `ai_make.rs` and `ai_explain.rs` (`sanitize_description`, `resolve_kind_priority`, `build_service_prompt`, `max_tokens_*`) migrate to `ferro-mcp` alongside the implementation. The CLI test files must be updated to call through `ferro_mcp::tools::*` or be replaced by the `ferro-mcp` tests directly.

---

## Security Domain

This phase relocates existing logic. No new attack surface is introduced.

| ASVS Category | Applies | Control |
|---------------|---------|---------|
| V5 Input Validation | yes | `sanitize_description` already strips XML delimiters (verified in `ai_make.rs:473`); this migrates intact to `ferro-mcp` |
| V2 Authentication | no | MCP server is local; no auth layer in scope |
| V6 Cryptography | no | No crypto operations |

The `sanitize_description` function (`IN-01` from Phase 171) is a load-bearing security control. It must relocate intact — not be re-implemented. Verified implementation strips `</description>` → `[/description]` and `<description>` → `[description]`.

---

## Sources

### Primary (HIGH confidence)

- `ferro-mcp/src/service.rs` — `#[tool_router]` pattern, `#[tool]` attribute, `Parameters<T>` usage, existing params structs (lines 332-376, 1671-1705) [VERIFIED in this session]
- `ferro-mcp/src/tools/ai.rs` — `TestClassifierResult` shape, async tool function pattern [VERIFIED in this session]
- `ferro-mcp/src/tools/inspect_projection.rs` — `ProjectionDetail`, `FieldInfo`, `InspectResult` exact fields and serialization [VERIFIED in this session]
- `ferro-mcp/src/tools/mod.rs` — module registration convention [VERIFIED in this session]
- `ferro-mcp/Cargo.toml` — dep edges: `ferro-ai:23`, `ferro-projections:25`, `rmcp = "0.12"` [VERIFIED in this session]
- `ferro-cli/Cargo.toml` — dep edge `ferro-mcp:45`; `ferro-projections` optional behind `projections` feature [VERIFIED in this session]
- `ferro-cli/src/commands/ai_make.rs` — full extraction boundary: `sanitize_description`, `resolve_max_tokens`, `ai_config_error_message`, `run()` body [VERIFIED in this session]
- `ferro-cli/src/commands/ai_explain.rs` — full extraction boundary: `ResolvedTarget`, `resolve_target`, `build_*_prompt`, `run()` body [VERIFIED in this session]
- `ferro-cli/src/relevance.rs` — `tokenize`, `Candidate`, `select_relevant`, `INPUT_BUDGET_CHARS` [VERIFIED in this session]
- `ferro-projections/src/service.rs` — `ServiceDef` derives `Serialize, Deserialize, JsonSchema` at line 62 [VERIFIED in this session]
- `ferro-ai/src/complete.rs` — `complete_with::<T>()` signature, `CompleteOptions` [VERIFIED in this session]
- `ferro-ai/src/config.rs` — `AiConfig::from_env() -> Result<Box<dyn LlmClient>, Error>` [VERIFIED in this session]
- `.github/workflows/publish.yml` — `ferro-mcp` is in Wave 2; no wave change needed [VERIFIED in this session]

---

## Metadata

**Confidence breakdown:**
- Extraction boundary: HIGH — all source files read directly
- MCP tool registration pattern: HIGH — exact template identified in service.rs
- Async/sync boundary for introspection fns: HIGH — verified from existing CLI usage
- ServiceDef serialization: HIGH — derives verified in source
- Wave/publish impact: HIGH — publish.yml confirmed, no new crates, no new dep edges

**Research date:** 2026-06-08
**Valid until:** 2026-07-08 (stable codebase; no external registry dependencies)
