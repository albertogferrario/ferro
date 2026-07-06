# Phase 210: COMP-03 — Agent-Success-Rate Harness - Research

**Researched:** 2026-06-13
**Domain:** LLM-agent evaluation harness in Rust; rmcp 0.12 in-process transport; ferro projection/intent scoring pipeline
**Confidence:** HIGH (all scoring APIs read from source; transport pattern partially CITED from rmcp docs — see A-rmcp)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions (verbatim from CONTEXT.md `## Implementation Decisions`)

**Execution model**
- **D-01:** Hybrid execution. Live agent runs gated behind `FERRO_AGENT_EVAL=1` (normal `cargo test` / CI skip them — no API key, no network, no cost, no LLM flakiness). A gated run drives the agent live and writes/refreshes the committed **baseline** + per-task **transcripts**. A non-gated path replays the committed transcripts through the **same scorer**, so CI guards the scorer + tier logic deterministically without LLM calls. Mirrors the `FERRO_BENCH=1` gate pattern used for COMP-04 (Phase 211).
- **D-02:** Baseline model pinned to `claude-opus-4-8`, recorded verbatim in the baseline artifact alongside prompt version and per-tier rates. ~14×3 = 42 calls per full gated run.
- **D-03:** Reuse the **ferro-ai completion client** (`ferro-ai/src/complete.rs`, `ferro-ai/src/client/`) for the live call if it exposes a usable text/tool completion API; otherwise a minimal Anthropic client confined to the test target (`#[cfg(test)]` / gated), adding NO always-on dependency to ferro-mcp's default build. Researcher confirms the ferro-ai surface first.
- **D-04:** Agent runs as an **in-process rmcp client** over the dev tools (rmcp 0.12 in-process / async-rw transport), not `ferro-mcp-server`. ≥3 trials per task.

**Agent output + toolset**
- **D-05:** Agent's deliverable is a **`ServiceDef`** (structured JSON the harness deserializes into `ferro_projections::ServiceDef`). Success measured on the chain `ServiceDef` → `derive_intents` → `Spec::from_service_def` → `checkpoint_projection`. `generate_projection` derives a ServiceDef *from a SeaORM model*, not from NL — so the agent **hand-authors** the ServiceDef.
- **D-06:** Agent toolset = `generation_context` (authoring guidance), `json_ui_catalog` (component + intent vocabulary), `checkpoint_projection` (self-verify before submitting). `generate_projection` **excluded**.

**Tier pass definitions (cumulative; stated before any run)**
- **D-07:**
  - **T1 Structural validity** — ServiceDef deserializes AND `Spec::from_service_def` renders AND `Catalog::validate(&spec)` returns 0 errors.
  - **T2 Intent coverage** — `derive_intents(&service)[0].intent` equals the task's declared target intent.
  - **T3 Functional completeness** — rendered spec's primary content element is data-bound, not placeholder, per the Phase 213 bar: Browse/Track `DataTable` non-empty `columns` + `items_path`; Process `KanbanBoard` `columns` + `items_path` + `group_by`; Collect `Form` ≥1 field; Summarize `StatCard` `value_path`; Focus/Analyze primary fields bound. No empty/placeholder values.
  - **T4 Checkpoint pass** — `checkpoint_projection` returns a verdict with zero blocking findings.
- **D-08:** A task-trial passes tier N iff it passes tiers 1..N. Baseline reports per-tier pass rate across all 14 tasks × ≥3 trials.

**Corpus + contamination guard**
- **D-09:** Corpus = 14 hand-authored NL tasks (2 per intent, all 7 intents), each declaring its target intent (for T2). Committed as a fixtures file alongside the harness.
- **D-10:** Contamination guard = **invented synthetic domains** — entity / field / domain nouns NOT drawn from ferro docs, the sample app, gestiscilo, or the Phase 207 catalog; phrasing paraphrased so it does not quote ferro's own intent vocabulary.
- **D-11:** Tasks are realistic-but-novel business descriptions spanning the 7 intents.

### Claude's Discretion
- Exact prompt template + prompt-version string; transcript file format; trial aggregation (per-trial vs mean); the specific 14 invented domains (within the D-10 guard); baseline-artifact file layout.

### Deferred Ideas (OUT OF SCOPE)
- **Success-rate floor / CI pass threshold** — set AFTER the first committed baseline run; a follow-up, not this phase.
- **Expanding the corpus beyond 2/intent or adding multi-step agent tasks** — future hardening.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| COMP-03 | Agent-success-rate harness measuring whether an agent reading `ferro-mcp` introspection can produce a working projection from NL. Multi-tier pass criteria stated before runs; ≥3 trials/task; committed baseline (model, prompt version, per-tier rates); corpus spans all 7 intents; drives ferro-mcp dev tools as in-process client (not `ferro-mcp-server`); guards against training-data contamination. | T1–T4 scoring APIs documented below (§Scoring Pipeline); in-process rmcp pattern (§Architecture Patterns); gate idiom (§Gate Pattern); contamination guard (§Contamination Guard); transcript/baseline format (§Transcript & Baseline). |
</phase_requirements>

## Summary

This phase builds a single greenfield Rust integration test file, `ferro-mcp/tests/agent_harness.rs` (the `ferro-mcp/tests/` directory does NOT yet exist — confirmed, only `Cargo.toml`, `README.md`, `src/` are present). The harness has two execution paths sharing **one scorer**:

1. **Live path** (`FERRO_AGENT_EVAL=1`): stands up an in-process rmcp client+server over the existing `FerroMcpService` dev tools, runs an agentic tool-use loop against `claude-opus-4-8`, captures each trial's final `ServiceDef` JSON into a committed transcript, and writes a baseline artifact.
2. **Replay path** (default `cargo test`): deserializes the committed transcript ServiceDefs and runs them through the identical T1–T4 scorer — no LLM, no network. This is what keeps `cargo test` always-green and no-network (the project's load-bearing invariant).

The scoring chain is fully reusable from library APIs already in the workspace: `serde_json::from_value::<ServiceDef>` (T1 deserialize) → `derive_intents(&service)` (T2) → `Spec::from_service_def(&service, &intents, &ctx)` + the implicit `Catalog::validate` it runs (T1 render) → inspect the rendered `Spec.elements` for binding completeness (T3) → `checkpoint_projection` verdict (T4).

**Primary recommendation:** Build a **minimal gated Anthropic tool-use loop confined to the test target** rather than reusing `ferro_ai::complete`. The typed `complete::<T>()` surface is single-shot structured-output, not a multi-turn tool-use loop — but `LlmClient::complete_with_tools` (implemented on `AnthropicClient`) IS a real multi-turn primitive. Reuse `ferro_ai::AnthropicClient` + `complete_with_tools` for the loop, sourcing the key from `FERRO_AI_API_KEY` (fallback `ANTHROPIC_API_KEY`). ferro-ai is already a default dependency of ferro-mcp, so this adds no new always-on dependency. See §ferro-ai Client Surface for the reuse-vs-minimal decision.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| In-process MCP transport | Test harness (`tests/agent_harness.rs`) | rmcp 0.12 (`transport-async-rw`) | Drives the real `FerroMcpService` over duplex pipes; no process spawn, no socket |
| Agent tool-use loop | Test harness (gated) | `ferro_ai::AnthropicClient::complete_with_tools` | Multi-turn loop is the only place an LLM is invoked; lives entirely behind the gate |
| ServiceDef authoring | LLM agent | `generation_context` / `json_ui_catalog` tools | Agent hand-authors; introspection tools are the surface under test |
| T1 deserialize + render | Scorer (shared) | `ferro_projections::ServiceDef`, `Spec::from_service_def` | Pure functions; deterministic |
| T2 intent | Scorer (shared) | `ferro_projections::derive_intents` | Pure function over ServiceDef |
| T3 binding completeness | Scorer (shared) | inspect rendered `Spec.elements` props | Reads the rendered spec; no external state |
| T4 checkpoint | Scorer (shared) | `checkpoint_projection` (file-based) — see Pitfall 4 | Currently filesystem-coupled; needs adaptation |
| Baseline + transcript persistence | Test harness | serde_json files under phase/test dir | Committed artifacts |

## Resolving the Toolset Discrepancy (ROADMAP SC#1 vs CONTEXT D-06)

**Authoritative answer: CONTEXT.md D-06 is correct. The agent toolset is `generation_context`, `json_ui_catalog`, `checkpoint_projection`. `generate_projection` is excluded; `validate_projection`/`Catalog::validate` is an internal scorer mechanism, not an agent tool.** [VERIFIED: source read]

Confirmed by reading the tool sources:

- **`generate_projection`** (`ferro-mcp/src/tools/generate_projection.rs`): `execute(project_root, model_name)` — finds a SeaORM model by name via `list_models`, converts to `ModelMetadata`, calls `ServiceDef::from_model(&meta)`. **Its input is a model name, not NL.** It is pure model-derivation. Excluding it is correct: it would let the agent skip authoring entirely. [VERIFIED]
- **`generation_context`** (`generation_context.rs`): `execute()` takes **no arguments**, returns framework conventions (naming, file structure, common patterns, anti-patterns, import templates). It is generic authoring guidance — it does NOT contain ServiceDef examples for the synthetic domains, so it does not defeat the contamination guard. Note: it currently has NO ServiceDef/projection authoring guidance at all (its `common_patterns` cover handlers, validation, Inertia, json_ui_view files — not ServiceDef builder/JSON). This is a **gap the planner must note** (see Open Question Q1). [VERIFIED]
- **`json_ui_catalog`** (`json_ui_catalog.rs`): `execute(component: Option<&str>)` returns the component catalog (`components`, `plugin_components`, `builder_api`, `action_api`, `json_schema`, `component_schemas`, `directives`) sourced from `ferro_json_ui::global_catalog()`. This is the component + render vocabulary. [VERIFIED]
- **`checkpoint_projection`** (`checkpoint_projection.rs`): the T4 verdict source. [VERIFIED]

**ROADMAP SC#1's `list_projections` / `generate_projection` / `validate_projection` phrasing is stale.** It predates the discuss-phase. The planner MUST follow D-06. Document this resolution in PLAN so plan-checker does not re-introduce the wrong tools.

## Standard Stack

### Core (all already in the workspace — no new published crates)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rmcp` | 0.12 | In-process MCP client+server transport | Already ferro-mcp's dep; D-04 / REQUIREMENTS pin it (no upgrade — out of scope) [VERIFIED: ferro-mcp/Cargo.toml] |
| `ferro-projections` | path 0.2 | `ServiceDef`, `derive_intents`, `Intent` | T1/T2 scoring; already a ferro-mcp dep [VERIFIED] |
| `ferro-json-ui` | path 0.2 (feature `projections`) | `Spec::from_service_def`, `Catalog`, `global_catalog()` | T1/T3 scoring; already a ferro-mcp dep with `projections` feature [VERIFIED: ferro-mcp/Cargo.toml line 24] |
| `ferro-ai` | path 0.2 | `AnthropicClient`, `complete_with_tools`, `LlmClient` | Live agent loop; already a ferro-mcp dep [VERIFIED] |
| `tokio` | 1 (`full`) | async runtime + `tokio::io::duplex` for in-process transport | Already a dep [VERIFIED] |
| `serde_json` | 1 | ServiceDef (de)serialization, transcripts, baseline | Already a dep [VERIFIED] |

### Supporting (dev-dependencies to add to `ferro-mcp/Cargo.toml [dev-dependencies]`)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tempfile` | 3 | already present as dev-dep | T4 checkpoint needs a temp project layout (Pitfall 4) [VERIFIED] |
| `rmcp` (client + transport-async-rw features) | 0.12 | the in-process client side | dev-dep with `["client", "transport-async-rw", "server"]` — see Pitfall 1 / Open Question A-rmcp |
| `serde` (derive) | 1 | transcript/baseline structs | already a dep |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| In-process rmcp transport | child-process (`TokioChildProcess`, as ferro-api-mcp/tests/e2e.rs does) | D-04 and REQUIREMENTS explicitly forbid `ferro-mcp-server`/process spawn; in-process is mandated |
| `ferro_ai::complete_with_tools` reuse | minimal hand-rolled `reqwest` Anthropic client in the test | complete_with_tools already implements the Anthropic tool-use wire format; hand-rolling duplicates it. Reuse. |
| Driving tools over the MCP transport | calling `tools::*::execute()` directly | REQUIREMENTS mandates the agent drive tools "as an in-process client" — the transport IS the surface under test. Must go through rmcp. |

**Installation (Cargo.toml dev-dependencies delta):**
```toml
[dev-dependencies]
tempfile = "3"
rmcp = { version = "0.12", features = ["client", "server", "transport-async-rw"] }
serde = { version = "1", features = ["derive"] }
```
**Version verification:** rmcp 0.12 is already pinned in three workspace crates; do NOT bump (REQUIREMENTS "rmcp upgrade (≥1.5)" is explicitly out of scope). [VERIFIED: grep of Cargo.toml files]

## Architecture Patterns

### System Architecture Diagram

```
                            FERRO_AGENT_EVAL=1 (gated, live)
                            ─────────────────────────────────
  task corpus (14 NL  ──►  agent loop (claude-opus-4-8)  ──►  final ServiceDef JSON
  fixtures, declared       │  complete_with_tools                     │
  target intent)           │      ▲        │ tool_use                 │ capture
                           │      │        ▼                          ▼
                           │   tool_result  in-process rmcp client ──► transcript file
                           │                      │  call_tool        (per task × trial)
                           │                      ▼                          │
                           │            FerroMcpService (real dev tools:     │
                           │            generation_context, json_ui_catalog, │
                           │            checkpoint_projection) over duplex    │
                           └──────────────────────────────────────────────►  │
                                                                              │
  ══════════════════════════════ SHARED SCORER ══════════════════════════════
                                                                              ▼
  default cargo test  ──►  read committed transcripts ──► for each trial's ServiceDef:
  (replay, no LLM)                                          T1 deserialize+render+Catalog.validate
                                                            T2 derive_intents[0].intent == target
                                                            T3 rendered Spec primary element bound
                                                            T4 checkpoint_projection verdict
                                                                  │
                                                                  ▼
                                              per-tier pass rate ──► baseline artifact
                                              (live path writes it; replay path asserts
                                               replayed result == committed baseline)
```

The load-bearing design point: **the scorer is a pure function `fn score(service: &ServiceDef, target: Intent) -> TierResult`** called identically by both paths. The live path feeds it freshly-authored ServiceDefs; the replay path feeds it deserialized committed ones. Determinism of T1–T4 is what makes replay valid (see §Validation Architecture).

### Recommended file structure
```
ferro-mcp/tests/
├── agent_harness.rs            # the test entrypoint: both gated + replay tests
agent_harness/                  # (or inline mods) — keep it one file if it stays small
.planning/phases/210-comp-03-agent-success-rate-harness/
├── corpus.json                 # 14 task fixtures (NL + declared intent + named actions/guards)
├── transcripts/                # one file per task, each with ≥3 trial ServiceDefs
│   └── <task_id>.json
└── baseline.json               # model, prompt_version, per-tier rates, timestamp
```
Discretion (D): exact location of corpus/transcripts/baseline. Recommendation: put fixtures + artifacts under the **phase dir** (committed, discoverable), and have the harness locate them via `CARGO_MANIFEST_DIR` + relative path, OR co-locate under `ferro-mcp/tests/fixtures/agent_harness/`. The latter is more conventional for a `tests/` integration test and survives `cargo package`; recommend `ferro-mcp/tests/fixtures/agent_harness/`.

### Pattern 1: rmcp 0.12 in-process client+server over `tokio::io::duplex`
**What:** Stand up `FerroMcpService` as a server on one half of a duplex pipe and an rmcp client `()` on the other, then drive `call_tool`.
**When:** The live agent loop's tool dispatch.

The proven workspace pattern is in `ferro-api-mcp/tests/e2e.rs`, but it uses **child-process** transport (`TokioChildProcess` + `().serve(transport)` → `RunningService<RoleClient,()>`, then `client.peer().call_tool(CallToolRequestParam { name, arguments })`). The client-side call shape is the reusable part [VERIFIED: e2e.rs lines 191-200, 340-341]:

```rust
// Client side (reusable from e2e.rs):
let client = ().serve(transport).await?;          // RunningService<RoleClient, ()>
let result = client.peer().call_tool(CallToolRequestParam {
    name: "json_ui_catalog".into(),
    arguments: Some(serde_json::json!({}).as_object().unwrap().clone()),
}).await?;
```

For **in-process** (D-04, no child process), replace the transport with an in-memory duplex. The server side reuses `FerroMcpService::new(project_root)` (it is `Clone`, holds a `ToolRouter<Self>`, implements `ServerHandler` via `#[tool_handler]`) and `.serve(server_half)`. [VERIFIED: ferro-mcp/src/service.rs lines 14-28; server.rs shows `service.serve((stdin, stdout))` is the existing stdio invocation]

```rust
// SKETCH — verify exact rmcp 0.12 transport-async-rw API at plan time (A-rmcp):
let (client_io, server_io) = tokio::io::duplex(8 * 1024);
let service = FerroMcpService::new(project_root);
let _server = service.serve(server_io).await?;          // server task
let client = ().serve(client_io).await?;                // client handshake
// ... client.peer().call_tool(...) ...
```

`service.serve((stdin, stdout))` already works in `server.rs`, which proves `serve` accepts an `AsyncRead+AsyncWrite` pair. `tokio::io::DuplexStream` is `AsyncRead+AsyncWrite`, so a duplex pair should drop in with the `transport-async-rw` feature enabled. [CITED: rmcp transport-async-rw is the documented feature for serving over arbitrary async read/write — confirm exact `serve` signature for a single `DuplexStream` vs a `(read, write)` tuple at plan time.]

### Pattern 2: Agent tool-use loop via `complete_with_tools`
**What:** Multi-turn loop: send system+user prompt + tool defs → if `CompletionResponse::ToolUse`, dispatch each block through the rmcp client, push assistant + tool_result messages, repeat → on `CompletionResponse::Text`, parse the final ServiceDef JSON.
**Source APIs** [VERIFIED: ferro-ai/src/client/mod.rs + anthropic.rs]:

```rust
// ferro_ai::client (re-exported as ferro_ai::AnthropicClient):
pub enum CompletionResponse {
    Text(String),
    ToolUse { blocks: Vec<ToolUseBlock>, assistant_content: String },
}
pub struct ToolUseBlock { pub id: String, pub name: String, pub input: serde_json::Value }
pub struct ToolRequest { pub name: String, pub description: String, pub parameters_schema: serde_json::Value }
pub enum ToolChoice { Auto, None }

#[async_trait] pub trait LlmClient {
    async fn complete_with_tools(&self, request: CompletionRequest)
        -> Result<CompletionResponse, Error>;   // implemented on AnthropicClient
}
```
`AnthropicClient::complete_with_tools` POSTs to `https://api.anthropic.com/v1/messages` with `x-api-key`, parses `stop_reason == "tool_use"` into `ToolUse`, else extracts text. The loop reconstructs history per the doc-comment contract: **push an `Assistant` message with `assistant_content` BEFORE appending `Role::Tool` result messages**, with `tool_call_id = block.id`. [VERIFIED: anthropic.rs lines 273-329; mod.rs ToolUse doc-comment]

`AnthropicClient::new(api_key, model_override)` — pass `Some("claude-opus-4-8".into())` for D-02, or set `model_override` per `CompletionRequest`. Key from `std::env::var("FERRO_AI_API_KEY").or_else(|_| std::env::var("ANTHROPIC_API_KEY"))`. [VERIFIED: anthropic.rs line 35; config.rs lines 48-55]

### Anti-Patterns to Avoid
- **Calling `tools::*::execute()` directly instead of via the transport** — defeats the "drives ferro-mcp dev tools as an in-process client" requirement. The transport round-trip is the surface under test.
- **Collapsing the four tiers into a boolean** — D-08 requires per-tier rates. The scorer must return a struct with one field per tier, never `bool`.
- **Letting the live path's freshly-authored ServiceDef be scored differently from the replayed one** — both must go through the identical `score()` function. If they diverge, replay is meaningless.
- **Mid-loop ferro publishes / network in the default path** — violates always-green-no-network.

## Scoring Pipeline (T1–T4 exact APIs)

### T1 — Structural validity
1. **Deserialize:** `serde_json::from_value::<ferro_projections::ServiceDef>(agent_json)`. ServiceDef derives `Deserialize` [VERIFIED: service.rs line 62]. Failure → T1 fail.
2. **Derive intents** (needed before render): `let intents = ferro_projections::derive_intents(&service);` — always returns ≥1 score [VERIFIED: derive.rs line 75, always non-empty].
3. **Render:** `ferro_json_ui::Spec::from_service_def(&service, &intents, &ctx)` where `ctx: VisualContext` selects `intent_index` (use 0 for the top intent) and `mode` (`RenderMode::Display` for Browse/Process/Summarize/etc.; `RenderMode::Input` collapses to a Form — relevant for Collect). Returns `Result<Spec, ProjectionError>`. [VERIFIED: builder.rs lines 55-74]
4. **Catalog validation is implicit:** `from_service_def` internally calls `catalog.validate(&spec)` and, in **debug builds, PANICS** on invalid spec (`#[cfg(debug_assertions)] panic!(...)`); in release it returns `Err(ProjectionError::CatalogValidation(errors))`. [VERIFIED: builder.rs lines 112-122]. **This is Pitfall 3** — see below. For an explicit error-count, call `ferro_json_ui::global_catalog().validate(&spec) -> Result<(), Vec<CatalogError>>` directly and count `errors.len()`. [VERIFIED: catalog.rs line 670]

**T1 mechanism decision (resolving ROADMAP "validate_projection" vs CONTEXT "Catalog::validate"):** Use the **direct `Catalog::validate(&spec)` call** (CONTEXT D-07 phrasing), NOT the MCP `validate_projection` tool. `validate_projection` is a file-scanning tool (it reads projection source from `src/projections/`); the harness has an in-memory ServiceDef, not a source file. Direct `Catalog::validate` is the right T1 render-validity check. [VERIFIED: validate_projection used by checkpoint seam 1 operates on files]

### T2 — Intent coverage
```rust
let intents = derive_intents(&service);   // Vec<IntentScore>, sorted desc by confidence
let top = &intents[0].intent;             // IntentScore { intent: Intent, confidence: f64, matching_signals: Vec<String> }
let pass = *top == task.target_intent;
```
[VERIFIED: derive.rs returns sorted `Vec<IntentScore>`; IntentScore fields used throughout derive.rs tests]

**The 7 `Intent` variants** (plus `Custom`): `Browse, Focus, Collect, Process, Summarize, Analyze, Track, Custom(String)` [VERIFIED: derive.rs `intent_priority` match, lines 550-560]. `Intent` derives `PartialEq` (used in `==` throughout derive.rs tests). The corpus declares its target as one of the 7.

**T2 calibration note from COMP-02** (`ferro-projections/tests/catalog.rs`): Analyze↔Summarize margin is structurally thin — Analyze's `datetime_numeric_cooccurrence` is a flat 0.35 that does not scale, while each Money/Percentage/Quantity field adds 0.30 to Summarize. **Implication for the corpus:** an Analyze task must use exactly one numeric-aggregate field + a DateTime field; two numeric fields flip the winner to Summarize. The corpus author must understand the derivation signals (documented in §derive signals below) to author tasks whose *correct authoring* lands on the declared intent. [VERIFIED: catalog.rs doc-comment lines 12-19, 36-42]

**derive signals cheat-sheet (for corpus authoring; from derive.rs):**
- Browse: `EntityName` fields, `Category` fields, OneToMany/ManyToMany relationships, simple CRUD actions. Baseline +0.1 always.
- Focus: `FreeText`/`ImageUrl`/`Url` fields, OneToOne-inline / ManyToOne relationships, more-readable-than-writable. Baseline +0.1 always.
- Collect: >50% writable ratio, write-only fields, actions with >2 inputs.
- Process: state machine with **guards** + **branching**, actions with `transition_trigger` + `preconditions`.
- Track: state machine **linear** (>2 non-final states, no branching), `Status` field, unguarded progression, final states.
- Summarize: `Money`/`Percentage`/`Quantity` fields, mostly-read-only (>70% non-writable).
- Analyze: `DateTime` + numeric co-occurrence (flat 0.35). [VERIFIED: derive.rs analyzers]

Tie-break priority (lower wins): Process(0) > Track(1) > Collect(2) > Browse(3) > Focus(4) > Summarize(5) > Analyze(6). [VERIFIED: derive.rs intent_priority]

### T3 — Functional completeness (Phase 213 binding bar)
Inspect the rendered `Spec` (from T1). The Spec is a flat `elements` map (id → Element with `type` + `props`). Per D-07 and the Phase 213 root-fix, the primary content element must be **data-bound, not placeholder**:

| Declared intent | Primary element type | Bound iff (props present + non-empty) |
|-----------------|----------------------|----------------------------------------|
| Browse / Track | `DataTable` | `columns` non-empty AND `items_path` present (NOTE: Phase 213 **removed `data_path`** for kanban; DataTable still uses `items_path` per the structure/content split — verify the DataTable prop name in `component.rs` `DataTableProps` at plan time, see A2) |
| Process | `KanbanBoard` | `columns` non-empty AND `items_path` present AND `group_by` present |
| Collect | `Form` | ≥1 field element |
| Summarize | `StatCard` | `value_path` present |
| Focus / Analyze | (Focus: DescriptionList/Card; Analyze: chart/stat) | primary fields bound (non-empty field bindings) |

[VERIFIED: D-07; 213-06-SUMMARY confirms KanbanBoardProps = `columns` (always rendered) + `items_path` + `group_by` + card_*_key bindings, and that `data_path` was REMOVED as the blank-board root cause]

**How the scorer detects bound-vs-placeholder programmatically:** walk `spec.elements`, find the element whose `type` matches the expected primary component for the declared intent, then assert the relevant props exist and are non-empty in the element's `props` JSON object. The builder (`builder.rs`) imports the typed props structs (`DataTableProps`, `KanbanBoardProps`, `KanbanColumnProps`, `FormProps`, `StatCardProps`, `DescriptionListProps` — [VERIFIED: builder.rs lines 27-31]). The scorer can deserialize the element's `props` into the matching typed struct and check fields, OR inspect the raw JSON for the prop keys. **Recommend: inspect raw JSON props** (more robust to prop-struct churn; the binding keys are the contract). The planner should pin the exact prop key names by reading `ferro-json-ui/src/component.rs` for each `*Props` struct's serde field names (A2 — left unread this session; the 213-06 summary names them: `columns`, `items_path`, `group_by`, `card_title_key`, `card_description_key`, `row_actions`, `row_key` for KanbanBoard).

A "placeholder" is detectable as: the prop key absent, or present but empty (`columns: []`, `items_path: ""`/null, no field children). Before Phase 213, Process/Summarize/actions rendered placeholders — this harness is the standing regression guard that they stay content-complete (CONTEXT specifics).

### T4 — Checkpoint pass
`checkpoint_projection::execute(project_root: &Path, name: &str) -> Result<Verdict, String>` (and the timestamp-injectable `run_for(project_root, name, now)`). [VERIFIED: checkpoint_projection.rs lines 152-162]

**Input shape — CRITICAL constraint (Pitfall 4):** `checkpoint_projection` does **NOT** take a ServiceDef or Spec. It is **filesystem-coupled**: it calls `inspect_projection::execute(project_root, name)` to locate a projection **source file** under `src/projections/`, reads the file, reconstructs the ServiceDef from source text via `reconstruct_service_def`, resolves the backing model under `src/models/`, and walks 5 seams. [VERIFIED: checkpoint_projection.rs lines 163-235]

**Verdict output shape** [VERIFIED: lines 63-73]:
```rust
pub struct Verdict {
    pub status: SeamStatus,        // Pass | Warn | Fail | NotChecked
    pub projection: String,
    pub seams: Vec<SeamResult>,    // 5 seams: projection_well_formed, field_to_column, action_to_route, rendered_view, props_to_contract
    pub next_steps: Vec<String>,
}
```
"**Zero blocking findings**" (D-07 T4) = `verdict.status != SeamStatus::Fail`. `aggregate_status` returns `Fail` if any seam fails, `Warn` if any warns, else `Pass`; `NotChecked` never raises to Fail. [VERIFIED: lines 686-700]. Decision: T4 passes iff `status == Pass` (strictest) OR `status ∈ {Pass, Warn}` (zero *blocking* = not Fail). **Recommend T4 = `status != Fail`** to match "zero *blocking* findings" literally; document the choice in the tier definition so it is stated-before-runs.

**The filesystem coupling problem & the recommended adaptation:** to run T4 the harness must materialize the agent's ServiceDef as a `src/projections/<name>.rs` source file (and a matching `src/models/<name>.rs`) inside a `tempfile::tempdir()` project root, then call `checkpoint_projection::execute(tmp.path(), "<name>_service")`. The checkpoint tests in `checkpoint_projection.rs` already demonstrate this exact fixture pattern (`project_with_projection`, `add_model`). [VERIFIED: checkpoint_projection.rs lines 829-846, 980-1038]. **This is the single most complex part of the scorer** because the agent emits JSON but checkpoint reads Rust source — the harness must render the ServiceDef back into builder-call Rust source (or generate a faithful `src/projections` file). The planner should evaluate whether T4 can run against the in-memory ServiceDef by calling the underlying seam functions directly, OR whether the file-materialization round-trip is required. Given checkpoint is the mandated T4 mechanism (D-05/D-07), file-materialization is the safe path; budget a task for it. (See Open Question Q2.)

## The Agent's Deliverable Contract (D-05) — ServiceDef JSON schema

`ferro_projections::ServiceDef` serde shape [VERIFIED: service.rs lines 62-93]:

```jsonc
{
  "name": "string",                 // required
  "display_name": "string|absent",  // skip_serializing_if None
  "description": "string|absent",   // skip_serializing_if None
  "fields": [                       // required (may be []), Vec<FieldDef>
    {
      "name": "string",
      "data_type": "...",           // DataType enum
      "meaning": "...",             // FieldMeaning enum (drives intent derivation!)
      "required": true,
      "is_list": false,
      "readable": true,
      "writable": true
    }
  ],
  "actions": [ /* ActionDef */ ],          // default [], skip if empty
  "guards": [ /* GuardDef */ ],            // default [], skip if empty
  "relationships": [ /* RelationshipDef */ ], // default [], skip if empty
  "intent_hints": [ /* IntentHint */ ],    // default [], skip if empty — NOTE: lets agent cheat T2 (see below)
  "state_machine": { /* StateMachine */ }, // skip if None
  "mcp_exposed": false,                    // default false
  "tenant_column": "string|absent",
  "mcp_ability": "string|absent"
}
```

**Minimal valid ServiceDef:** `{"name":"x","fields":[]}` deserializes successfully [VERIFIED: service.rs test `mcp_exposed_defaults_false_when_absent` uses exactly this]. But an empty-fields ServiceDef will derive Browse/Focus baseline only and fail T3 (no bound content) — so a *passing* ServiceDef needs fields with meaningful `FieldMeaning`s, and for Process/Track a `state_machine`, for Collect writable fields, etc.

**Contamination caveat on `intent_hints`:** `IntentHint::Primary(intent)` forces `derive_intents()[0]` to that intent with confidence 1.0 [VERIFIED: derive.rs apply_hints lines 567-586]. An agent that emits `intent_hints: [{"Primary": "Process"}]` would auto-pass T2 without deriving anything structurally. **The planner must decide whether the prompt/contract forbids `intent_hints`** (recommended: forbid them, or treat their use as a T2 disqualifier) so T2 measures *structural* derivation, not a declared override. Document in the prompt + tier definition. (Open Question Q3.)

**What `generation_context` gives the agent for authoring:** currently NO ServiceDef-specific guidance (it covers handlers/models/views, not projections) [VERIFIED]. The agent learns the ServiceDef *component/render* vocabulary from `json_ui_catalog` and self-verifies via `checkpoint_projection`, but it has no canonical "here is how to write a ServiceDef JSON" reference from the tools. The harness's **prompt** must supply the ServiceDef JSON schema (it can include the schema from `schemars::schema_for!(ServiceDef)` — ServiceDef derives `JsonSchema` [VERIFIED: service.rs line 62] — without including domain examples that would defeat contamination). This is the clean way to teach shape without teaching answers.

## Gate Pattern (FERRO_AGENT_EVAL) — mirroring the established idiom

There is no `FERRO_BENCH` in the repo yet (COMP-04/Phase 211 is also pending — confirmed: `grep FERRO_BENCH` finds nothing) [VERIFIED]. The established gated-test idiom in the workspace is the **`#[ignore]` + env-var** pattern used by `framework/tests/async_validation_pg_gate.rs` and `constraint_map_pg_gate.rs` and `ferro-reservation/tests/concurrent_hold_postgres.rs` [VERIFIED]. Those use `#[ignore]` so the normal suite skips them and `DATABASE_URL` to locate the resource.

**Recommended idiom for the harness:**

```rust
// Live, gated test — skipped by default cargo test (no API key, no network).
#[tokio::test]
#[ignore = "live LLM eval; run with FERRO_AGENT_EVAL=1 and FERRO_AI_API_KEY set"]
async fn agent_eval_live_refresh_baseline() {
    if std::env::var("FERRO_AGENT_EVAL").is_err() {
        eprintln!("skipping: set FERRO_AGENT_EVAL=1 to run live eval");
        return;
    }
    // ... run 14×3 trials, write transcripts + baseline ...
}

// Replay test — runs in DEFAULT cargo test, no LLM, deterministic.
#[test]
fn agent_eval_replay_scores_match_baseline() {
    // read committed transcripts → score through identical scorer → assert per-tier
    // rates equal the committed baseline.json (determinism guard).
}
```

Use **both** belt-and-suspenders: `#[ignore]` (so even `cargo test -- --include-ignored` without the env var no-ops cleanly) AND the early-return env check. This matches the pg-gate precedent and guarantees CI green without an API key. The replay test carries the real CI value: it proves the scorer + tier logic on every run.

**Always-green invariant:** the replay path must NOT require network, an API key, a real model, or a writable global location — only reading committed transcript files and running pure scorer functions (plus a `tempfile` tempdir for T4 file-materialization). [VERIFIED against project invariant in CLAUDE.md + MEMORY.md]

## ferro-ai Client Surface — reuse-vs-minimal recommendation (D-03)

**Recommendation: REUSE `ferro_ai::AnthropicClient` + `LlmClient::complete_with_tools`. Do NOT hand-roll a client. Do NOT use `ferro_ai::complete::<T>()`.**

Rationale [all VERIFIED]:
- `ferro_ai::complete::<T>()` / `complete_with::<T>()` (complete.rs) is **single-shot structured output** — it builds one `CompletionRequest` with `tools: None, tool_choice: None`, calls `client.complete`, and deserializes one JSON response. It cannot do a multi-turn tool-use loop. Insufficient for an agent that must read `json_ui_catalog` then author.
- `LlmClient::complete_with_tools` (mod.rs) IS the multi-turn primitive, and `AnthropicClient` implements it fully against the real Anthropic Messages API including the `tool_use`/`tool_result` wire format (anthropic.rs lines 273-329). The harness writes the loop *around* it.
- ferro-ai is **already a non-optional dependency** of ferro-mcp (`ferro-ai = { path = "../ferro-ai", version = "0.2" }`), so reusing it adds **no new always-on dependency** — satisfying D-03's "NO always-on dependency to ferro-mcp's default build" because the client is only *constructed* inside the gated test, and the dependency already exists for src/tools/ai*. [VERIFIED: ferro-mcp/Cargo.toml line 23]
- API key: `FERRO_AI_API_KEY` (primary) / `ANTHROPIC_API_KEY` (fallback) [VERIFIED: config.rs].

**What the harness must build itself** (the gap): the **loop driver** — translate `ToolUseBlock { name, input }` into an rmcp `call_tool`, map the rmcp `CallToolResult` back into a `Role::Tool` message (`content` = the tool result text, `tool_call_id` = `block.id`), push the `Assistant(assistant_content)` turn first, and repeat until `CompletionResponse::Text`. Plus tool-definition construction: build `Vec<ToolRequest>` for the 3 allowed tools, with `parameters_schema` from each tool's input schema (generation_context takes no args; json_ui_catalog takes optional `component`; checkpoint_projection takes a projection `name`). The tool schemas can be lifted from `FerroMcpService`'s `ToolRouter` or hand-declared.

## Transcript & Baseline Format (Claude's discretion — researched constraints)

**Constraint:** the replay path must reproduce the exact per-tier result from the captured ServiceDef alone, with no LLM call. Therefore the transcript MUST capture, per trial, **the agent's final ServiceDef JSON** (the scorer input) and the **declared target intent** (or derive it from the corpus by task id). Capturing the full tool-call trace is optional/diagnostic; only the final ServiceDef is load-bearing for replay determinism.

**Recommended transcript shape** (one file per task under `tests/fixtures/agent_harness/transcripts/<task_id>.json`):
```jsonc
{
  "task_id": "process-telescope-slots",
  "target_intent": "Process",
  "model": "claude-opus-4-8",
  "prompt_version": "v1",
  "trials": [
    {
      "trial": 0,
      "service_def": { /* the agent's final ServiceDef JSON — scorer input */ },
      "tool_calls": [ /* optional: [{name, input, result_summary}] for audit */ ]
    }
    // ... ≥3 trials
  ]
}
```

**Recommended baseline shape** (`tests/fixtures/agent_harness/baseline.json`):
```jsonc
{
  "model": "claude-opus-4-8",
  "prompt_version": "v1",
  "generated_at": "2026-06-13T...Z",
  "tasks": 14,
  "trials_per_task": 3,
  "tier_rates": { "t1": 0.93, "t2": 0.79, "t3": 0.71, "t4": 0.64 },  // fraction of (task×trial) passing tier N (cumulative)
  "per_intent": { "Process": {"t1":..,"t2":..,"t3":..,"t4":..}, ... } // optional breakdown
}
```
Aggregation (D-discretion): recommend **per-(task×trial) cumulative pass fraction** for `tier_rates` (matches D-08 "per-tier pass rate across all 14 tasks × ≥3 trials"), plus an optional per-intent breakdown for diagnostics. Trials reported individually (not pre-meaned) keeps variance visible for the deferred floor decision.

**Replay assertion:** the replay test recomputes `tier_rates` from committed transcripts and asserts equality with committed `baseline.json` — this is the determinism guard (same transcript → same rates). A mismatch means either the scorer changed (intended — regenerate baseline) or non-determinism leaked in (bug). Recommend exact-equality on the rate fractions; if floating aggregation is fragile, store integer pass-counts instead of fractions.

## Contamination Guard Mechanics (D-10 / D-11)

**Domains already used in the codebase (the corpus must AVOID these nouns — reference only, do NOT seed from them):**
From `ferro-projections/tests/catalog.rs` (the 7 canonical COMP-02 fixtures) the recurring domain nouns across the repo's projection examples are: **order, product, invoice, booking, customer, user, shipment, line_item, payment, warehouse, category, profile, financials, dashboard, timeseries, catalog, content, orders, registration, secret, auth** [VERIFIED: catalog.rs imports + service.rs/derive.rs test fixtures + generate_projection.rs tests + checkpoint_projection.rs tests]. The 213-06 summary adds gestiscilo domains: **staff, orders (Italian: confermato/in_corso/rientrato/chiuso/annullato)**.

**Guard design:**
- Each corpus task uses an **invented synthetic domain** whose entity/field nouns appear nowhere in `ferro-projections/tests/catalog.rs`, the sample `app/`, gestiscilo, or the Phase 207 catalog. The CONTEXT D-11 example ("telescope observation-time slots: requested → scheduled → observed → archived") is the model — exotic enough that the nouns are not in training-adjacent ferro docs.
- **Paraphrase away ferro's intent vocabulary:** do not write "this is a Process intent" or "browse a list" or "collect a form"; describe the *behavior* ("staff move slots through stages with approval gates") and let the agent derive the intent.
- **Automated contamination check (a Validation Architecture signal):** a `#[test]` that loads the corpus and asserts no corpus domain/field noun appears in a denylist sourced from `catalog.rs` (and optionally a small curated list of ferro-doc nouns). This makes the guard a standing, checkable invariant rather than a one-time authoring discipline.

**Recommended corpus fixture shape** (`tests/fixtures/agent_harness/corpus.json`):
```jsonc
[
  {
    "id": "process-telescope-slots",
    "target_intent": "Process",          // for T2
    "description": "A registry of telescope observation-time slots that staff move through requested → scheduled → observed → archived, with approval before scheduling.",   // NL only, no intent vocabulary
    "expected_actions": ["schedule", "observe", "archive"],   // for T3 / checkpoint action seam
    "expected_guards": ["reviewer_approved"]                  // named guards for T3 / Process derivation
  }
  // ... 14 total, 2 per intent across Browse, Focus, Collect, Process, Summarize, Analyze, Track
]
```
The `expected_actions`/`expected_guards` give the scorer/diagnostics a reference for what a *correct* authoring contains, and help the T2/T3 author calibrate each task so its faithful authoring lands on the declared intent (recall the thin Analyze↔Summarize margin).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Anthropic tool-use wire format | a reqwest client | `ferro_ai::AnthropicClient::complete_with_tools` | already implements `tool_use`/`tool_result`, error mapping, model override [VERIFIED] |
| Intent derivation | re-implement scoring | `ferro_projections::derive_intents` | the system under test; re-implementing would not measure ferro |
| Spec validation | manual prop checks for T1 | `Catalog::validate(&spec)` / `Spec::from_service_def` | catalog is the authoritative validator [VERIFIED] |
| Checkpoint seams | re-derive the 5 seams | `checkpoint_projection::execute` | T4 must be the real tool (D-05/D-07) |
| MCP framing | hand-build JSON-RPC | rmcp 0.12 `serve` + `call_tool` | the transport is the surface under test |

**Key insight:** the harness is almost entirely *orchestration* of existing ferro APIs. The only genuinely new code is (a) the in-process transport wiring, (b) the tool-use loop driver, (c) the T3 binding inspector, (d) the T4 file-materialization adapter, (e) the corpus + transcript + baseline I/O. No new framework logic.

## Common Pitfalls

### Pitfall 1: rmcp client feature not enabled
**What goes wrong:** ferro-mcp's `Cargo.toml` only enables rmcp `["server", "transport-io"]`. The client side + in-process transport need `["client", "transport-async-rw"]` as **dev-dependencies**. Forgetting this → the in-process client/`serve(duplex)` won't compile.
**Avoid:** add the dev-dependency delta (see §Standard Stack). Do NOT add client/async-rw to the non-dev `[dependencies]` (keeps default build lean).

### Pitfall 2: always-on dependency creep
**What goes wrong:** putting the Anthropic loop or rmcp-client in non-dev deps pulls them into ferro-mcp's published surface (D-03 forbids always-on additions).
**Avoid:** everything LLM/client lives behind `#[cfg(test)]` in `tests/` and uses dev-dependencies only. `tests/*.rs` is compiled only for `cargo test`, never for the library build.

### Pitfall 3: debug-build panic on invalid spec
**What goes wrong:** `Spec::from_service_def` **panics** in debug builds when the catalog rejects the spec (`#[cfg(debug_assertions)] panic!`). `cargo test` is a debug build — so a malformed agent ServiceDef that renders an invalid spec would PANIC the test instead of being scored as a T1 fail. [VERIFIED: builder.rs lines 112-122]
**Avoid:** do NOT rely on `from_service_def` to report T1 render failure. Either (a) catch the panic with `std::panic::catch_unwind` (awkward across async), or (b) **score T1 by building the spec via the catalog path that returns errors**: call the render and validate steps in a way that returns `Result`, OR run the scorer logic that mirrors `from_service_def` but uses `from_service_def_with_catalog`/`global_catalog().validate` and treats `Err`/panic as T1 fail. **Recommend:** wrap the render in `catch_unwind` (it is sync) OR, cleaner, replicate the two render steps and call `global_catalog().validate(&spec)` explicitly so invalid specs return `Err(Vec<CatalogError>)` instead of panicking. The planner must pick one and verify it does not abort the test process. This is the subtlest correctness trap in the phase.

### Pitfall 4: checkpoint_projection is filesystem-coupled
**What goes wrong:** treating `checkpoint_projection` as if it accepts a ServiceDef/Spec. It does not — it reads a `src/projections/<name>.rs` source file and a `src/models/<name>.rs` model from a project root. [VERIFIED]
**Avoid:** materialize the agent's ServiceDef into a temp project (`tempfile::tempdir()` with `src/projections/<name>.rs` + `src/models/<name>.rs`), exactly as the checkpoint unit tests do, then call `execute(tmp.path(), "<name>_service")`. Budget a dedicated task for the ServiceDef→Rust-source renderer.

### Pitfall 5: schema-export test dirties the tree
**What goes wrong:** `cargo test` regenerates `docs/protocol/schemas/*.json` (Phase 94 export test) — unrelated churn. [project memory]
**Avoid:** `git checkout` those files; never fold into the phase commit.

### Pitfall 6: disk-full on `--all-features` test
**What goes wrong:** `cargo test --all-features` recurrently ENOSPC-fails (not a real defect). [project memory]
**Avoid:** check `df` and clean `target/` before the full gate; the harness's replay test is light, but the workspace gate is heavy.

## Validation Architecture

> nyquist_validation enabled (no `workflow.nyquist_validation` key in `.planning/config.json` → default enabled). This section triggers VALIDATION.md.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`#[test]` / `#[tokio::test]`) — workspace standard |
| Config file | none — `cargo test` |
| Quick run command | `cargo test -p ferro-mcp --test agent_harness` (replay only; no LLM) |
| Full suite command | `cargo test -p ferro-mcp` then workspace `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| Live (gated) command | `FERRO_AGENT_EVAL=1 FERRO_AI_API_KEY=… cargo test -p ferro-mcp --test agent_harness -- --ignored --nocapture` |

### Observable signals that prove the harness works
| Signal | Behavior proven | Automated check |
|--------|-----------------|-----------------|
| **Scorer determinism** | same transcript ServiceDef → same per-tier result every run | replay test recomputes `tier_rates` from committed transcripts and asserts `== baseline.json` |
| **Tier independence** | each tier reported separately, never collapsed to a boolean | unit test asserts the scorer returns a 4-field `TierResult`, and that a ServiceDef passing T1 but failing T2 is recorded as `{t1:pass, t2:fail, t3:fail, t4:fail}` (cumulative) |
| **CI-green-without-LLM** | replay path runs in default `cargo test`, no API key, no network | the replay `#[test]` is NOT `#[ignore]`d and has no env/network dependency; the live test IS `#[ignore]`d + env-gated |
| **Contamination guard** | no corpus noun appears in catalog.rs / ferro docs | `#[test]` loads corpus, asserts no corpus domain/field noun ∈ denylist derived from `ferro-projections/tests/catalog.rs` |
| **T1 invalid-spec safety** | malformed ServiceDef scores as T1 fail, does not panic the process | a fixture transcript containing a deliberately-invalid ServiceDef → scorer returns `t1:fail` (proves Pitfall 3 is handled) |
| **Honesty (REQUIREMENTS)** | the harness can FAIL / surface weakness | at least one committed transcript trial fails a tier; a "discovered weaknesses" note records what the agent got wrong |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| COMP-03 | replay scorer reproduces committed baseline | unit/integration | `cargo test -p ferro-mcp --test agent_harness::replay` | ❌ Wave 0 |
| COMP-03 | tier independence (no boolean collapse) | unit | `cargo test -p ferro-mcp --test agent_harness::tier_independence` | ❌ Wave 0 |
| COMP-03 | contamination denylist | unit | `cargo test -p ferro-mcp --test agent_harness::contamination` | ❌ Wave 0 |
| COMP-03 | T1 invalid spec → fail not panic | unit | `cargo test -p ferro-mcp --test agent_harness::t1_invalid` | ❌ Wave 0 |
| COMP-03 | live eval refreshes baseline (gated) | integration (ignored) | `FERRO_AGENT_EVAL=1 … -- --ignored` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-mcp --test agent_harness` (replay + unit signals; fast, no LLM).
- **Per wave merge:** `cargo test -p ferro-mcp` + `cargo clippy -p ferro-mcp --all-targets -- -D warnings`.
- **Phase gate:** full workspace `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` green; one committed baseline produced by a real `FERRO_AGENT_EVAL=1` run; "discovered weaknesses" section populated.

### Wave 0 Gaps
- [ ] `ferro-mcp/tests/agent_harness.rs` — does not exist; greenfield (confirmed `ferro-mcp/tests/` absent). Covers all COMP-03 signals.
- [ ] `ferro-mcp/tests/fixtures/agent_harness/corpus.json` — 14 task fixtures.
- [ ] `ferro-mcp/tests/fixtures/agent_harness/transcripts/*.json` — produced by the first gated run.
- [ ] `ferro-mcp/tests/fixtures/agent_harness/baseline.json` — produced by the first gated run.
- [ ] dev-dependency delta in `ferro-mcp/Cargo.toml` (rmcp client + transport-async-rw, serde derive).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rmcp 0.12 (server, transport-io) | existing ferro-mcp build | ✓ | 0.12 | — |
| rmcp client + transport-async-rw | in-process client (dev-dep) | ✗ (not yet enabled) | 0.12 | add dev-dep — no fallback needed |
| ferro-ai (AnthropicClient) | live agent loop | ✓ | path 0.2 | — |
| `claude-opus-4-8` via Anthropic API | live gated run only | ✗ at test time (no key in CI) | — | gate skips; replay path needs no model |
| `FERRO_AI_API_KEY` / `ANTHROPIC_API_KEY` | live gated run only | ✗ in CI (by design) | — | gate skips |

**Missing dependencies with no fallback:** none block the default/replay path.
**Missing dependencies with fallback:** the LLM + API key are absent in CI by design; the `#[ignore]` + env gate is the fallback (default suite skips live eval).

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| KanbanBoard `data_path` (column-replacing) | `columns` + `items_path` + `group_by` (structure/content split) | Phase 213 (0.2.55) | T3 Process check must look for `items_path`+`group_by`, NOT `data_path` |
| ROADMAP SC#1 toolset (`list_projections`/`generate_projection`/`validate_projection`) | CONTEXT D-06 toolset (`generation_context`/`json_ui_catalog`/`checkpoint_projection`) | discuss-phase 2026-06-13 | use D-06 |

**Deprecated/outdated:** ROADMAP SC#1 tool list (superseded by D-06). KanbanBoard `data_path` (removed Phase 213).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A-rmcp | rmcp 0.12 `serve` accepts a `tokio::io::DuplexStream` (or a `(read,write)` duplex tuple) for in-process transport with `transport-async-rw` | Pattern 1 | Medium — if `serve` needs a different in-memory transport type, the wiring changes (still in-process, but different glue). Confirm exact `serve` signature + the `transport-async-rw` API at plan time. `server.rs` proves `serve((stdin,stdout))` works, so an async-rw pair is highly likely to work. |
| A2 | DataTable's binding prop is `items_path` (not `data_path`) post-213, and the exact `*Props` serde keys are as the 213-06 summary names them | T3 | Low — read `ferro-json-ui/src/component.rs` for `DataTableProps`/`KanbanBoardProps`/`FormProps`/`StatCardProps` serde field names to pin T3 checks. |
| A3 | T4 "zero blocking findings" = `verdict.status != Fail` | T4 | Low — design choice; document in the stated-before-runs tier definition either way. |
| A4 | Forbidding `intent_hints` in the agent contract is the right call so T2 measures structural derivation | ServiceDef contract | Low — a policy decision for discuss/plan; either choice is defensible if stated up front. |
| A5 | `complete_with_tools` on AnthropicClient honors `model_override="claude-opus-4-8"` and the loop reconstruction contract as documented | Pattern 2 | Low — verified the impl reads `model_override` and parses tool_use; the multi-turn history reconstruction is per the documented contract. |

## Open Questions (RESOLVED)

1. **`generation_context` has no ServiceDef authoring guidance.** It returns handler/model/view conventions, not projection/ServiceDef guidance. The agent's only ServiceDef shape source is the harness prompt (recommend injecting `schemars::schema_for!(ServiceDef)`). Should the phase ALSO enrich `generation_context` with ServiceDef guidance? — Likely out of scope (would change a tool under test mid-measurement); recommend prompt-supplied schema only, and note the gap as a finding.
   - **RESOLVED:** Out of scope for Phase 210. Enriching `generation_context` would mutate a tool *under measurement* mid-baseline (violates the CONTEXT phase boundary: no toolset/renderer changes). Decision: the harness prompt supplies `schemars::schema_for!(ServiceDef)` only; the "tool gives no ServiceDef guidance" gap becomes a candidate SC#5 discovered-weakness finding (Plan 04 Task 3), not a code change this phase.
2. **T4 file-materialization vs direct-seam-call.** checkpoint_projection reads source files. Either render ServiceDef→Rust source into a temp project (faithful to the tool), or call the seam functions on the in-memory ServiceDef (faster, but bypasses the real tool entry point). Recommend file-materialization to honor "checkpoint_projection returns a verdict" (D-07) literally; plan a task for the ServiceDef→source renderer.
   - **RESOLVED:** File-materialization chosen. Plan 02 Task 2 materializes the agent ServiceDef into `src/projections/<name>.rs` inside `tempfile::tempdir()` before invoking `checkpoint_projection::execute`, honoring D-07's literal "checkpoint_projection returns a verdict" and exercising the real tool entry point.
3. **Forbid `intent_hints`?** They let the agent auto-pass T2. Recommend forbidding in the prompt + treating their presence as a T2 disqualifier; lock in discuss/plan.
   - **RESOLVED:** Forbidden. Plan 02 Task 1 treats the presence of `intent_hints` in the agent's ServiceDef as a T2 disqualifier (so T2 measures structural intent derivation, not a self-declared hint); Plan 03's prompt instructs the agent not to emit them.
4. **Exact in-process rmcp transport API** (A-rmcp) — confirm `serve(DuplexStream)` signature at plan time; this is the only mechanical unknown.
   - **RESOLVED (verify-at-execute):** Assumption A-rmcp stands at MEDIUM confidence — `server.rs` proves `serve((stdin,stdout))` works, so an async-rw duplex pair is highly likely to work with the `transport-async-rw` feature. Plan 03 Task 1 requires the executor to confirm the exact `serve` signature against the live rmcp 0.12 source / Context7 before coding. No further pre-execution research needed; the unknown is bounded to one wiring task.

## Sources

### Primary (HIGH confidence — read this session)
- `ferro-mcp/src/tools/{generate_projection,generation_context,checkpoint_projection,json_ui_catalog,mod}.rs` — tool surfaces, toolset discrepancy resolution, T4 verdict shape
- `ferro-mcp/src/{lib,server,service}.rs` — `FerroMcpService` server handler, `serve` usage
- `ferro-mcp/Cargo.toml` — rmcp features, dependency surface
- `ferro-ai/src/complete.rs`, `ferro-ai/src/client/mod.rs`, `ferro-ai/src/client/anthropic.rs`, `ferro-ai/src/config.rs` — completion + tool-use surface, API key env vars
- `ferro-projections/src/{derive,service}.rs` — `derive_intents`, `ServiceDef` schema, `Intent` variants
- `ferro-json-ui/src/projection/builder.rs`, `ferro-json-ui/src/catalog.rs` — `Spec::from_service_def`, `Catalog::validate`, debug panic behavior
- `ferro-projections/tests/catalog.rs` — used domains (contamination denylist), Analyze↔Summarize margin
- `ferro-api-mcp/tests/e2e.rs` — rmcp client `call_tool`/`peer`/`serve` shape (child-process variant)
- `framework/tests/async_validation_pg_gate.rs` — `#[ignore]` + env-var gate idiom
- `.planning/phases/213-projection-render-completeness/213-06-SUMMARY-gap-a-root-fix.md` — T3 binding bar
- `.planning/phases/210-…/210-CONTEXT.md`, `.planning/REQUIREMENTS.md` (§COMP-03), `.planning/STATE.md`

### Secondary (MEDIUM confidence)
- rmcp 0.12 `transport-async-rw` semantics (in-process duplex) — inferred from `serve((stdin,stdout))` working + feature name; CITE/confirm exact API at plan time.

## Metadata

**Confidence breakdown:**
- Toolset resolution: HIGH — read all four tool sources.
- Scoring pipeline (T1–T4): HIGH — read every API; T1 panic + T4 filesystem coupling are verified traps.
- Standard stack: HIGH — verified against Cargo.toml.
- In-process transport: MEDIUM — pattern proven for child-process; in-process duplex API needs one confirmation (A-rmcp).
- ServiceDef schema: HIGH — read serde struct + round-trip tests.
- Contamination guard / corpus: HIGH — denylist nouns verified from catalog.rs.

**Research date:** 2026-06-13
**Valid until:** ~30 days (workspace is pre-1.0 but these crates are stable internally; rmcp pin is frozen for v13.0)

## RESEARCH COMPLETE
