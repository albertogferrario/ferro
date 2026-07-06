# Phase 210: COMP-03 — Agent-Success-Rate Harness - Pattern Map

**Mapped:** 2026-06-13
**Files analyzed:** 5 (1 Rust test file + 4 committed fixture/data files)
**Analogs found:** 5 / 5

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-mcp/tests/agent_harness.rs` | integration-test | request-response + event-driven (tool-use loop) | `ferro-api-mcp/tests/e2e.rs` | role-match (child-process → in-process) |
| `ferro-mcp/tests/agent_harness.rs` (gate idiom) | test | request-response | `framework/tests/async_validation_pg_gate.rs` | exact |
| `ferro-mcp/tests/fixtures/agent_harness/corpus.json` | fixture/config | batch | `ferro-projections/tests/catalog.rs` (fixtures module) | domain-match (contamination reference) |
| `ferro-mcp/tests/fixtures/agent_harness/transcripts/<task_id>.json` | artifact | batch | none — greenfield format | no analog |
| `ferro-mcp/tests/fixtures/agent_harness/baseline.json` | artifact | batch | none — greenfield format | no analog |
| `ferro-mcp/Cargo.toml` (dev-dep delta) | config | — | existing `[dev-dependencies]` block | exact (additive) |

---

## Pattern Assignments

### 1. `ferro-mcp/tests/agent_harness.rs` — in-process rmcp transport

**Analog:** `ferro-api-mcp/tests/e2e.rs`
**Data flow:** The child-process transport pattern is the proven analog; swap `TokioChildProcess` for `tokio::io::duplex`.

**Imports pattern** (`ferro-api-mcp/tests/e2e.rs` lines 13-19):
```rust
use rmcp::model::CallToolRequestParam;
use rmcp::service::RunningService;
use rmcp::ServiceExt;
use serde_json::json;
```

**Client construction + tool call pattern** (`ferro-api-mcp/tests/e2e.rs` lines 191-200, 340-357):
```rust
// Child-process variant (proven workspace pattern):
let transport = TokioChildProcess::new(cmd).expect("failed to spawn ferro-api-mcp");
let client: RunningService<rmcp::RoleClient, ()> = ().serve(transport).await.expect("MCP handshake failed");

// Tool call shape (identical for in-process and child-process variants):
let result = client.peer().call_tool(CallToolRequestParam {
    name: store_name.into(),
    arguments: Some(
        json!({ "body": { "name": "E2E Test User" } })
            .as_object()
            .unwrap()
            .clone(),
    ),
}).await.expect("store tool call failed");

// Extract text content from result:
let text = result
    .content
    .iter()
    .filter_map(|c| c.raw.as_text())
    .map(|t| t.text.as_str())
    .collect::<Vec<_>>()
    .join("\n");
```

**In-process adaptation** — swap transport only; call shape is unchanged.
`ferro-mcp/src/server.rs` lines 21-32 prove that `FerroMcpService::new(root).serve((stdin, stdout))` accepts an `AsyncRead+AsyncWrite` pair. `tokio::io::duplex` produces `DuplexStream` which is `AsyncRead+AsyncWrite`. Pattern:
```rust
// Adapt the server.rs proven pattern (lines 22-28) to use a duplex:
let (client_stream, server_stream) = tokio::io::duplex(64 * 1024);
let service = FerroMcpService::new(project_root.clone());
let _server_handle = service.serve(server_stream).await?;    // server half
let client: RunningService<rmcp::RoleClient, ()> =
    ().serve(client_stream).await?;                          // client handshake
// then: client.peer().call_tool(CallToolRequestParam { ... })
```

`FerroMcpService` construction: `ferro-mcp/src/service.rs` lines 15-27:
```rust
#[derive(Clone)]                    // Clone is derived — can be moved into server task
pub struct FerroMcpService {
    project_root: PathBuf,
    tool_router: ToolRouter<Self>,  // constructed via Self::tool_router()
}

impl FerroMcpService {
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            tool_router: Self::tool_router(),
        }
    }
}
// Implements ServerHandler via #[tool_handler] + #[tool_router] proc-macro attrs
```

**Cancellation pattern** (`ferro-api-mcp/tests/e2e.rs` line 294):
```rust
client.cancel().await.ok();   // shut down the client cleanly after each test
```

---

### 2. `ferro-mcp/tests/agent_harness.rs` — gated env-var test idiom

**Analog:** `framework/tests/async_validation_pg_gate.rs`
**Match quality:** exact — same workspace project, same `#[ignore]` + `std::env::var` belt-and-suspenders pattern.

**Full gate pattern** (`framework/tests/async_validation_pg_gate.rs` lines 51-54):
```rust
#[tokio::test]
#[serial]
#[ignore = "requires a live Postgres (set DATABASE_URL); run with -- --ignored"]
async fn pg_unique_rule_placeholder_and_quoting_path() {
    // no env-var early-return needed when #[ignore] is used;
    // but RESEARCH recommends belt-and-suspenders: add a guard at the top too
```

**Adapted idiom for `agent_harness.rs`** (combine `#[ignore]` + env guard):
```rust
// Live, gated — skipped by default cargo test (no API key, no network, no cost).
#[tokio::test]
#[ignore = "live LLM eval; run with FERRO_AGENT_EVAL=1 and FERRO_AI_API_KEY set"]
async fn agent_eval_live_refresh_baseline() {
    if std::env::var("FERRO_AGENT_EVAL").is_err() {
        eprintln!("skipping: set FERRO_AGENT_EVAL=1 to run live eval");
        return;
    }
    // ... 14×3 trials, write transcripts + baseline ...
}

// Replay test — runs in DEFAULT cargo test; no LLM, deterministic.
#[test]
fn agent_eval_replay_scores_match_baseline() {
    // read committed transcripts → score identical scorer → assert tier_rates == baseline.json
}
```

**Key:** the replay test must NOT carry `#[ignore]` — it is the CI-valuable half. Only the live test is `#[ignore]`d.

---

### 3. `ferro-mcp/tests/agent_harness.rs` — ferro-ai completion client

**Analog:** `ferro-ai/src/client/mod.rs` + `ferro-ai/src/client/anthropic.rs`

**Types to import** (`ferro-ai/src/client/mod.rs` lines 33-191):
```rust
use ferro_ai::client::{
    AnthropicClient,
    CompletionRequest,
    CompletionResponse,   // Text(String) | ToolUse { blocks, assistant_content }
    LlmClient,            // trait with complete_with_tools
    Message,
    Role,                 // User | Assistant | Tool
    ToolChoice,
    ToolRequest,
    ToolUseBlock,         // { id: String, name: String, input: serde_json::Value }
};
```

**Client construction** (`ferro-ai/src/client/anthropic.rs` lines 35-45):
```rust
// Key sourcing (config.rs lines 48-56):
let api_key = std::env::var("FERRO_AI_API_KEY")
    .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
    .expect("FERRO_AI_API_KEY or ANTHROPIC_API_KEY must be set for live eval");

// Client instantiation with model pin (D-02):
let client = AnthropicClient::new(
    api_key,
    Some("claude-opus-4-8".into()),   // D-02: pinned model
);
```

**`complete_with_tools` signature** (`ferro-ai/src/client/mod.rs` lines 184-191):
```rust
async fn complete_with_tools(
    &self,
    request: CompletionRequest,
) -> Result<CompletionResponse, Error>
```

**`complete_with_tools` implementation contract** (`ferro-ai/src/client/anthropic.rs` lines 273-331):
The method POSTs to `https://api.anthropic.com/v1/messages`, checks `stop_reason`:
- `"tool_use"` → returns `CompletionResponse::ToolUse { blocks: Vec<ToolUseBlock>, assistant_content: String }`
- any other → returns `CompletionResponse::Text(String)`

**Multi-turn history reconstruction rule** (from `ferro-ai/src/client/mod.rs` ToolUse doc-comment, lines 107-114):
```rust
// BEFORE appending tool result messages, push the assistant's turn:
messages.push(Message {
    role: Role::Assistant,
    content: assistant_content.clone(),   // the raw assistant_content from ToolUse
    tool_call_id: None,
});
// THEN push one Tool message per block:
for block in &blocks {
    let result_text = dispatch_tool_via_rmcp(&block.name, &block.input, &client).await;
    messages.push(Message {
        role: Role::Tool,
        content: result_text,
        tool_call_id: Some(block.id.clone()),   // Anthropic: tool_use_id
    });
}
```

**ToolRequest construction** (for registering the 3 allowed tools from D-06):
```rust
// generation_context — no args; json_ui_catalog — optional component; checkpoint_projection — name
let tools = vec![
    ToolRequest {
        name: "generation_context".into(),
        description: "...".into(),
        parameters_schema: serde_json::json!({ "type": "object", "properties": {} }),
    },
    ToolRequest {
        name: "json_ui_catalog".into(),
        description: "...".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": { "component": { "type": "string" } }
        }),
    },
    // checkpoint_projection similarly
];
```

---

### 4. `ferro-mcp/tests/agent_harness.rs` — scoring pipeline (T1–T4)

#### T1 + T2: `derive_intents` and `Spec::from_service_def`

**Analog for T1 deserialization:** `ferro-projections/src/service.rs` lines 62-93 (`ServiceDef` derives `Deserialize`).

**T1 + T2 scorer pattern:**
```rust
use ferro_projections::{ServiceDef, derive_intents, Intent};
use ferro_json_ui::projection::{RenderMode, VisualContext};
use ferro_json_ui::catalog::global_catalog;

// T1a — deserialize
let service: ServiceDef = serde_json::from_value(agent_json.clone())?;

// T1b — derive intents (needed for render)
let intents = derive_intents(&service);   // Vec<IntentScore>, always non-empty

// T1c — render + implicit validate (PITFALL 3: panics in debug builds on invalid spec)
// Use from_service_def_with_catalog via the pub(crate) path, OR wrap in catch_unwind:
let ctx = VisualContext {
    intent_index: 0,
    current_state: None,
    mode: RenderMode::Display,
    templates: None,
};
let render_result = std::panic::catch_unwind(|| {
    ferro_json_ui::Spec::from_service_def(&service, &intents, &ctx)
});
let t1_pass = render_result.is_ok() && render_result.unwrap().is_ok();

// T1d — explicit catalog validate for error count (builder.rs line 112, catalog.rs line 670):
// Call only after confirming from_service_def did not panic:
// let errors = global_catalog().validate(&spec).err().unwrap_or_default();
```

**`derive_intents` function signature** (`ferro-projections/src/derive.rs` lines 75-113):
```rust
pub fn derive_intents(service: &ServiceDef) -> Vec<IntentScore>
// Vec<IntentScore> is always non-empty (fallback Focus 0.5 if no signals)
// IntentScore { intent: Intent, confidence: f64, matching_signals: Vec<String> }
// sorted descending by confidence; [0] is the primary intent
```

**T2 check:**
```rust
// T2 — intent coverage
let top_intent = &intents[0].intent;
let t2_pass = t1_pass && (*top_intent == task.target_intent);
// Intent derives PartialEq — == works directly
// 7 variants: Browse, Focus, Collect, Process, Summarize, Analyze, Track, Custom(String)
```

**`Spec::from_service_def` signature** (`ferro-json-ui/src/projection/builder.rs` lines 55-73):
```rust
pub fn from_service_def(
    service: &ServiceDef,
    intents: &[IntentScore],
    ctx: &VisualContext,
) -> Result<Spec, ProjectionError>
// VisualContext { intent_index: usize, current_state: Option<String>, mode: RenderMode, templates: Option<ThemeTemplates> }
// RenderMode::Display for Browse/Process/Summarize/Track/Focus/Analyze
// RenderMode::Input for Collect
```

**CRITICAL — debug panic** (`ferro-json-ui/src/projection/builder.rs` lines 112-122):
```rust
match catalog.validate(&spec) {
    Ok(()) => Ok(spec),
    Err(errors) => {
        #[cfg(debug_assertions)]
        panic!("Projector emitted invalid spec: {errors:?}");  // panics in cargo test!
        #[cfg(not(debug_assertions))]
        Err(ProjectionError::CatalogValidation(errors))
    }
}
```
Wrap the render call in `std::panic::catch_unwind` — malformed agent ServiceDefs must score as T1 fail, not crash the test process.

#### T3: binding completeness inspector

No analog — inspect `spec.elements` raw JSON props. Key props per intent (from RESEARCH D-07 + 213-06 summary):

| Intent | Element type | Bound-iff props present and non-empty |
|--------|-------------|---------------------------------------|
| Browse / Track | `DataTable` | `items_path` (String, non-empty), `columns` (Array, non-empty) |
| Process | `KanbanBoard` | `items_path`, `columns`, `group_by` all non-empty |
| Collect | `Form` | ≥1 child field element |
| Summarize | `StatCard` | `value_path` non-empty |
| Focus / Analyze | `DescriptionList` / chart | primary fields non-empty |

Pattern for T3:
```rust
// walk spec.elements (flat id→Element map), find the element whose `type` matches
// the declared intent's primary component, then inspect `.props`:
let t3_pass = t2_pass && {
    let primary_elem = spec.elements.values()
        .find(|e| e.element_type == expected_primary_type(&task.target_intent));
    primary_elem.map(|e| is_bound(&e.props, &task.target_intent)).unwrap_or(false)
};
```

#### T4: `checkpoint_projection` (filesystem-coupled)

**Analog for tempdir fixture:** `ferro-mcp/src/tools/checkpoint_projection.rs` lines 829-846 (internal test helpers):
```rust
// Copy this exact helper pattern into agent_harness.rs for T4 materialization:
fn project_with_projection(name: &str, projection_src: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let proj_dir = tmp.path().join("src/projections");
    std::fs::create_dir_all(&proj_dir).unwrap();
    std::fs::write(proj_dir.join(format!("{name}.rs")), projection_src).unwrap();
    tmp
}

fn add_model(tmp: &tempfile::TempDir, name: &str, model_src: &str) {
    let models_dir = tmp.path().join("src/models");
    std::fs::create_dir_all(&models_dir).unwrap();
    std::fs::write(models_dir.join(format!("{name}.rs")), model_src).unwrap();
}
```

**`checkpoint_projection::execute` signature** (`ferro-mcp/src/tools/checkpoint_projection.rs` lines 152-153):
```rust
pub async fn execute(project_root: &Path, name: &str) -> Result<Verdict, String>
```

**`Verdict` shape** (lines 64-73):
```rust
pub struct Verdict {
    pub status: SeamStatus,       // Pass | Warn | Fail | NotChecked
    pub projection: String,
    pub seams: Vec<SeamResult>,   // 5 seams
    pub next_steps: Vec<String>,
}
// T4 passes iff: verdict.status != SeamStatus::Fail   (per RESEARCH A3 recommendation)
```

**T4 usage pattern** (requires ServiceDef→Rust source renderer, then materialize into tempdir):
```rust
// 1. Render the agent's ServiceDef to Rust source (custom code in harness)
let projection_src = render_service_def_to_rust_source(&service);
// 2. Materialize into tempdir
let tmp = project_with_projection("harness_service", &projection_src);
// (add_model for a minimal stub model if seam2 is to be checked)
// 3. Call checkpoint
let verdict = ferro_mcp::tools::checkpoint_projection::execute(
    tmp.path(),
    "harness_service",
).await.expect("checkpoint execute failed");
// 4. Score
let t4_pass = t3_pass && (verdict.status != SeamStatus::Fail);
```

---

### 5. `ferro-mcp/tests/fixtures/agent_harness/corpus.json` — contamination guard reference

**Analog (contamination denylist source):** `ferro-projections/tests/catalog.rs` lines 70-80+
The canonical COMP-02 fixtures define the domains the corpus MUST NOT use. Denylist nouns include:
`product_catalog, order, invoice, booking, customer, user, shipment, line_item, payment, warehouse, category, profile, financials, dashboard, timeseries, registration`

**Corpus fixture shape** (from RESEARCH D-09 / D-11):
```jsonc
[
  {
    "id": "process-telescope-slots",
    "target_intent": "Process",
    "description": "A registry of telescope observation-time slots that staff move through requested → scheduled → observed → archived, with approval before scheduling.",
    "expected_actions": ["schedule", "observe", "archive"],
    "expected_guards": ["reviewer_approved"]
  }
  // ... 14 total, 2 per intent: Browse, Focus, Collect, Process, Summarize, Analyze, Track
]
```

**Contamination check test pattern:**
```rust
#[test]
fn corpus_contamination_guard() {
    let denylist = ["product", "order", "invoice", "booking", "customer", "user",
                    "shipment", "payment", "warehouse", "category", "profile",
                    "financials", "timeseries", "registration", "gestiscilo",
                    "staff", "confermato", "chiuso", "annullato"];
    let corpus: Vec<serde_json::Value> = serde_json::from_str(
        include_str!("fixtures/agent_harness/corpus.json")
    ).unwrap();
    for task in &corpus {
        let desc = task["description"].as_str().unwrap_or("").to_lowercase();
        for noun in &denylist {
            assert!(!desc.contains(noun),
                "corpus task '{}' contains denylist noun '{}'",
                task["id"].as_str().unwrap_or("?"), noun);
        }
    }
}
```

---

### 6. `ferro-mcp/Cargo.toml` — dev-dependency delta

**Analog:** existing `[dev-dependencies]` block (line 39-40). Currently only `tempfile = "3"`.

**Required additions** (additive, dev-only per Pitfall 2):
```toml
[dev-dependencies]
tempfile = "3"   # already present
rmcp = { version = "0.12", features = ["client", "server", "transport-async-rw"] }
serde = { version = "1", features = ["derive"] }
```

DO NOT add `rmcp` client/transport-async-rw features to `[dependencies]` — keep them dev-only so ferry-mcp's published surface stays lean (D-03, Pitfall 2).

---

## Shared Patterns

### ServiceDef Serde Shape (agent deliverable contract — T1 deserialize)

**Source:** `ferro-projections/src/service.rs` lines 62-93
**Apply to:** Scorer deserialization step, prompt template (JSON schema injection), transcript storage.

```rust
// Minimal valid (deserializes, but fails T3 — no bound content):
// {"name":"x","fields":[]}

// Fields that drive intent derivation (corpus authors must include these):
// - fields[*].meaning: FieldMeaning enum value (drives T2 derive signals)
// - state_machine: required for Process/Track
// - actions[*]: required for Process (transition_trigger + preconditions → state machine guard signals)
// - guards[*]: required for Process (branching)

// FORBIDDEN in agent output (T2 contamination guard, Q3):
// - intent_hints: [{"Primary": "Process"}]  — forces T2 pass without structural derivation
```

Key serde attributes on `ServiceDef`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ServiceDef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")] pub display_name: Option<String>,
    pub fields: Vec<FieldDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")] pub actions: Vec<ActionDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")] pub guards: Vec<GuardDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")] pub intent_hints: Vec<IntentHint>,
    #[serde(skip_serializing_if = "Option::is_none")] pub state_machine: Option<StateMachine>,
    #[serde(default)] pub mcp_exposed: bool,
    // ...
}
```

`ServiceDef` derives `JsonSchema` — the harness prompt can inject `schemars::schema_for!(ServiceDef)` as the shape reference without including domain examples.

### TierResult struct (scorer output — must be 4-field, never bool)

No analog exists — new type. Must be:
```rust
#[derive(Debug, Serialize)]
struct TierResult {
    t1: bool,   // structural validity
    t2: bool,   // intent coverage (cumulative: false if t1 false)
    t3: bool,   // binding completeness (cumulative)
    t4: bool,   // checkpoint pass (cumulative)
}
```
Cumulative: `t2 = t1 && <t2 check>`, `t3 = t2 && <t3 check>`, `t4 = t3 && <t4 check>`.

### Transcript and Baseline Format (per RESEARCH §Transcript & Baseline)

**Transcript shape** (`tests/fixtures/agent_harness/transcripts/<task_id>.json`):
```jsonc
{
  "task_id": "process-telescope-slots",
  "target_intent": "Process",
  "model": "claude-opus-4-8",
  "prompt_version": "v1",
  "trials": [
    {
      "trial": 0,
      "service_def": { /* agent's final ServiceDef JSON — the scorer input */ },
      "tool_calls": [ /* optional audit trace */ ]
    }
  ]
}
```

**Baseline shape** (`tests/fixtures/agent_harness/baseline.json`):
```jsonc
{
  "model": "claude-opus-4-8",
  "prompt_version": "v1",
  "generated_at": "2026-06-13T...Z",
  "tasks": 14,
  "trials_per_task": 3,
  "tier_rates": { "t1": 0.0, "t2": 0.0, "t3": 0.0, "t4": 0.0 },
  "per_intent": { "Process": {"t1":0.0,"t2":0.0,"t3":0.0,"t4":0.0} }
}
```
Store integer pass-counts rather than fractions if float equality in replay assertions proves fragile.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `tests/fixtures/agent_harness/transcripts/<task_id>.json` | artifact | batch write/read | No existing transcript format in the workspace; shape defined by RESEARCH |
| `tests/fixtures/agent_harness/baseline.json` | artifact | batch | No existing eval-baseline format; shape defined by RESEARCH |
| ServiceDef→Rust source renderer (inline helper in harness) | utility | transform | No existing ServiceDef→source renderer; custom code required for T4 materialization |

---

## Key Pitfalls (for planner to propagate into plan tasks)

1. **rmcp dev-dep features** — `[dev-dependencies]` must add `rmcp` with `["client", "server", "transport-async-rw"]`; the library `[dependencies]` already has `rmcp = { version = "0.12", features = ["server", "transport-io"] }` — do not modify that.
2. **Debug panic in `from_service_def`** — builder.rs panics on invalid spec in debug builds; wrap in `std::panic::catch_unwind` for T1 scoring.
3. **T4 filesystem coupling** — `checkpoint_projection::execute` reads `src/projections/<name>.rs` from disk; use the `project_with_projection` / `add_model` tempdir helper pattern from checkpoint_projection.rs lines 832-846.
4. **`intent_hints` contamination** — forbid `intent_hints` in agent output via the prompt contract; treat their presence as a T2 disqualifier. This is a policy choice that must be stated-before-runs.
5. **`generation_context` gap** — the tool returns handler/model/view conventions, not ServiceDef authoring guidance. The harness prompt must supply the `schemars::schema_for!(ServiceDef)` JSON schema as the shape reference.
6. **Schema export test dirties tree** — `cargo test` regenerates `docs/protocol/schemas/*.json`; `git checkout` those files, never fold into the phase commit.

---

## Metadata

**Analog search scope:** `ferro-api-mcp/tests/`, `framework/tests/`, `ferro-mcp/src/`, `ferro-ai/src/`, `ferro-projections/src/`, `ferro-projections/tests/`, `ferro-json-ui/src/`
**Files scanned:** 12 (e2e.rs, async_validation_pg_gate.rs, client/mod.rs, client/anthropic.rs, config.rs, service.rs, server.rs, checkpoint_projection.rs lines 55-846, derive.rs lines 60-160, builder.rs lines 1-124, projection/mod.rs lines 30-65, Cargo.toml, catalog.rs lines 1-80)
**Pattern extraction date:** 2026-06-13
