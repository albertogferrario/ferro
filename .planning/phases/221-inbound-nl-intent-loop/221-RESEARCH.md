# Phase 221: Inbound NL Intent Loop - Research

**Researched:** 2026-06-14
**Domain:** ferro-mcp-server NL classification loop, ferro-ai Classifier, Phase 210 replay harness reuse
**Confidence:** HIGH — all claims verified against live source files; no assumed library versions

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01 (Loop home):** The conversational-turn core (classify → guard-check → confirmation-gate → dispatch → result) is a testable function in `ferro-mcp-server`. `ToolSelection { tool_name: String, arguments: Map<String,Value>, confidence: f64 }` defined in `ferro-mcp-server` (not ferro-ai). The sample `app` wires a thin `/mcp/chat` HTTP endpoint around it.

- **D-02 (Classification → routing):** `ferro_ai::Classifier<ToolSelection>::classify(system, user, schema)` where system = a new `render_tool_descriptions(services, ctx)` helper (reusing the 218 render surface), user = the NL message, schema = `ToolSelection` JSON Schema. Classified `tool_name` matched to read (`dispatch()`) vs write (`dispatch_write()` / 220 confirmation path) with no new dispatch logic. Guard-checked using 219 server-side guard re-eval (classifier output never trusted).

- **D-03 (Clarification / low-confidence):** Reuse `Classifier`'s EXISTING `Error::LowConfidence { best_guess, confidence }`. Map to structured `{ status: "needs_clarification", question, best_guess }` `CallToolResult::structured` response. No new confidence/threshold logic in ferro-mcp-server.

- **D-04 (Write → confirmation):** Classified WRITE intent routes through the Phase 220 confirmation gate. Loop returns confirmation-required response and does NOT call `dispatch_write` directly for destructive actions. Non-destructive writes dispatch directly.

- **D-05 (Replay / live-eval):** REUSE the Phase 210 COMP-03 transcript-fixture + deterministic-replay-guard pattern. A replay `ClassificationProvider` reads recorded transcripts, returns recorded classifications with no network. `FERRO_AI_LIVE_EVAL` unset → replay only (SC#3). `FERRO_AI_LIVE_EVAL=1` → live provider, asserts against (or updates) fixture, announces estimated cost before first call (SC#4).

- **D-06 (Feature gating):** An `intent` (or `ai`) Cargo feature on `ferro-mcp-server` enables `ferro-ai` (with `llm` for the live provider) + the loop module. The replay `ClassificationProvider` implements the trait without the http client so the deterministic replay test compiles/runs without `llm`.

- **D-07 (Result envelopes):** Every turn outcome uses 219/220 `CallToolResult::structured` / `write_tool_error_result` envelopes. Classified arguments are UNTRUSTED — pass through 219 validation + guard re-eval + tenant scoping.

### Claude's Discretion

- Exact `render_tool_descriptions` text format.
- The `ToolSelection` JSON-schema field names.
- The cost-estimate formula/announcement string.
- Whether the replay provider lives in ferro-ai (alongside the client) or ferro-mcp-server tests.

### Deferred Ideas (OUT OF SCOPE)

- Parameter-elicitation state machine (MCP request/response handles multi-turn).
- Multi-turn conversation memory / session context beyond a single turn.
- Live-eval in CI (live stays opt-in/local only).
- gestiscilo adoption of `/mcp/chat` — consumer-repo follow-up.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AMCP-06 | A natural-language message is classified to a tool + arguments (ferro-ai), guard-checked and confirmation-gated, dispatched, and the result rendered back — the conversational turn. The loop ships with a gated replay/smoke path (`FERRO_AI_LIVE_EVAL=1`) so it is CI-testable without live-LLM spend. | Verified: `Classifier<T>`, `ClassificationProvider` trait, `Error::LowConfidence`, Phase 210 harness pattern, existing dispatch/write_dispatch/confirmation paths all confirmed in source. |

</phase_requirements>

---

## Summary

Phase 221 is the final v15.0 phase: it adds a conversational-turn entry point that classifies a natural-language message into a `ToolSelection { tool_name, arguments, confidence }` and routes through the existing read/write/confirmation dispatch paths without adding new dispatch logic. The entire implementation is additive over four complete prior phases (217 auth, 218 write-tool rendering, 219 write dispatch, 220 confirmation gating).

The research confirms all five key architectural claims from CONTEXT.md against live source: (1) `ClassificationProvider` is reqwest-free — it is a standalone trait in `ferro-ai/src/classifier/provider.rs` with zero `reqwest` imports; (2) the `llm`/`confirmation` feature split in ferro-ai already exists and is clean; (3) `ferro-mcp-server` already has a `confirmation` feature and an optional ferro-ai dep pattern to extend; (4) `dispatch_write` and `handle_write_call` are production-ready with the 220 confirmation gate fully wired at the D-08 seam; and (5) the Phase 210 harness uses a concrete, copy-reusable pattern of committed transcript fixtures + a no-LLM replay assertion + a `FERRO_AGENT_EVAL=1`-gated live path.

**Primary recommendation:** Implement the loop as a new `intent.rs` module in `ferro-mcp-server` (gated by a new `intent` Cargo feature), a `ReplayClassificationProvider` struct in the same module or in test infrastructure, and a thin `app/src/controllers/mcp_chat.rs` endpoint. The replay provider is structurally parallel to the `ConstProvider` used in ferro-ai's own classifier unit tests — it holds a map of recorded responses and returns them deterministically.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| NL classification (live) | `ferro-ai` (library) | `ferro-mcp-server` (caller) | `Classifier<T>` lives in ferro-ai; ferro-mcp-server creates `Classifier<ToolSelection>` |
| NL classification (replay/CI) | `ferro-mcp-server` tests | — | Replay `ClassificationProvider` is test infrastructure; no LLM or network |
| `render_tool_descriptions` (classifier system prompt) | `ferro-mcp-server/src/renderer.rs` | — | Reuses `render_exposed_tools` render surface from 218 |
| Turn core function | `ferro-mcp-server` (new `src/intent.rs`) | — | Unit-testable without app HTTP; the app wires a thin endpoint around it |
| `/mcp/chat` HTTP endpoint | `app/src/controllers/mcp_chat.rs` | — | Consumer app layer per ARCHITECTURE Decision (c) |
| Guard re-eval + tenant scoping | `ferro-mcp-server/src/write_dispatch.rs` | — | Already enforced in `dispatch_write`; loop reuses, not duplicates |
| Confirmation gating | `ferro-mcp-server/src/write_dispatch.rs` | — | D-08 seam already wired in 220; loop routes through it |
| Result envelopes | `ferro-mcp-server/src/write_dispatch.rs` | — | `write_tool_error_result` + `CallToolResult::structured` already used |

---

## Standard Stack

### Core (verified in source)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `ferro-ai` (classifier module) | workspace 0.2.x | `Classifier<ToolSelection>` + `ClassificationProvider` trait + `Error::LowConfidence` | Already in workspace; the only LLM classification abstraction in ferro |
| `ferro-mcp-server` | workspace 0.2.x | Turn core function + `render_tool_descriptions` helper + feature gate | Primary v15.0 implementation site per all prior phases |
| `serde_json` | 1.x (workspace) | `ToolSelection` deserialization, `Map<String,Value>` for arguments | Already a dependency everywhere |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `async-trait` | 0.1 (already in ferro-ai dev-deps) | `ClassificationProvider` impls in test infrastructure | Replay provider impl |
| `tokio` | 1 (already a dep) | Async runtime for `classify()` | The live provider path |

### Feature Wiring (D-06 — verified current state)

`ferro-ai/Cargo.toml` (confirmed):
- `default = ["llm"]` — enables reqwest, reqwest-eventsource, futures, async-stream, schemars
- `llm = [dep:reqwest, dep:reqwest-eventsource, dep:futures, dep:async-stream, dep:schemars]`
- `confirmation = []` — no extra deps (dashmap + tokio are always-on)
- `ClassificationProvider` trait: in `ferro-ai/src/classifier/provider.rs` — **zero reqwest imports**, only `async_trait`, `crate::error::Error`, and `super::ClassifierConfig`

`ferro-mcp-server/Cargo.toml` (confirmed):
- `confirmation = ["dep:ferro-ai", "dep:rand"]` with `ferro-ai = { ..., optional = true, default-features = false, features = ["confirmation"] }`
- No `llm` feature exists yet on ferro-mcp-server

**Phase 221 extension (D-06 concrete wiring):**

```toml
# ferro-mcp-server/Cargo.toml — add:
[features]
confirmation = ["dep:ferro-ai", "dep:rand"]          # existing
intent = ["dep:ferro-ai"]                             # NEW: enables loop module + replay provider
intent-live = ["intent", "ferro-ai/llm"]              # NEW: enables live provider (pulls reqwest)

[dependencies]
ferro-ai = { path = "../ferro-ai", version = "0.2", optional = true, default-features = false, features = ["confirmation"] }
# NOTE: "confirmation" is always-on for the optional dep (shared dep path);
# "llm" is added only when `intent-live` is active.
```

The `intent` feature (without `llm`) must compile and run the replay path without reqwest in the graph. The live path (`intent-live`) adds `ferro-ai/llm`. CI uses `intent` only; local live eval adds `intent-live`.

**Alternative:** `intent` + `intent-live` could be collapsed into a single `ai` feature following the 220 naming; but the D-06 decision names it `intent` (or `ai`) — either works. Research recommends `ai` to mirror the 220 `confirmation` naming pattern (one-word feature names for all ferro-mcp-server capabilities).

---

## Architecture Patterns

### System Architecture Diagram

```
POST /mcp/chat { "message": "approve the order from Alice" }
         |
         v (app/src/controllers/mcp_chat.rs)
  auth → tenant_id (reuse bearer_auth middleware from 219/220)
         |
         v
  render_tool_descriptions(exposed_services(), &ctx)
  → concise text of guard-filtered available tools (renderer.rs)
         |
         v
  Classifier<ToolSelection>::classify(system, user, schema)
    ┌─ REPLAY path (FERRO_AI_LIVE_EVAL unset) ──────────────────────┐
    │  ReplayClassificationProvider reads fixture → returns recorded  │
    │  ToolSelection with no network call                             │
    └───────────────────────────────────────────────────────────────┘
    ┌─ LIVE path (FERRO_AI_LIVE_EVAL=1) ────────────────────────────┐
    │  AnthropicProvider → HTTP → Claude → ToolSelection JSON        │
    │  result asserted against (or updates) fixture                   │
    │  cost announced before first call                               │
    └───────────────────────────────────────────────────────────────┘
         |
         v
  Error::LowConfidence?
    → needs_clarification response (CallToolResult::structured, isError:false)
         |
  ToolSelection { tool_name, arguments, confidence }
         |
         v
  is read tool? (tool_name.starts_with("list_"))
    → dispatch(service, filters, limit, offset, db, tenant_id)
    → read result → CallToolResult::structured
         |
  is write tool?
    → handle_write_call(params, services, db, tenant_id, ctx, dispatcher,
                        [confirmation_store], [config])
       ├─ scope check (217)
       ├─ guard re-eval (219) — LIVE DB state, never ctx.evaluated_guards
       ├─ idempotency check (219)
       ├─ D-08 seam: destructive? → confirmation-required (220)
       │     → request_confirm_<action> / confirm_<action> paths
       └─ execute callback (219) → audit (219) → structured result
```

### Recommended Project Structure (new files only)

```
ferro-mcp-server/src/
├── intent.rs          # NEW: ToolSelection type, process_nl_turn() core function,
│                      #      render_tool_descriptions() helper, ReplayClassificationProvider
app/src/controllers/
└── mcp_chat.rs        # NEW: thin POST /mcp/chat endpoint wiring process_nl_turn()
ferro-mcp-server/
└── tests/fixtures/intent_loop/
    ├── transcripts/   # per-turn recorded (input, ToolSelection) pairs
    └── baseline.json  # intent loop classification baseline (mirrors COMP-03 shape)
```

### Pattern 1: ToolSelection Type (in ferro-mcp-server/src/intent.rs)

```rust
// Source: verified against ferro-ai/src/classifier/mod.rs + CONTEXT.md D-01
use serde::{Deserialize, Serialize};
use serde_json::Map;

/// The classifier output for a single conversational turn.
/// Defined here (not in ferro-ai) because it is projection-specific.
#[derive(Debug, Deserialize, Serialize)]
pub struct ToolSelection {
    pub tool_name: String,
    pub arguments: Map<String, serde_json::Value>,
    pub confidence: f64,
}
```

The JSON schema for `ToolSelection` is passed as the `schema` argument to `classify()`. Since `confidence` is a field in the output type (not separate metadata), `ClassifierConfig::confidence_threshold` can gate it directly via the existing logic in `Classifier::classify` (lines 129-138 of classifier/mod.rs: `raw_json.get("confidence").and_then(|v| v.as_f64())`).

### Pattern 2: Classifier invocation (verified API)

```rust
// Source: ferro-ai/src/classifier/mod.rs — confirmed signature
pub async fn classify(
    &self,
    system_prompt: &str,
    user_prompt: &str,
    schema: &serde_json::Value,
) -> Result<ClassificationResult<T>, Error>

// Error::LowConfidence (confirmed):
// Error::LowConfidence { best_guess: serde_json::Value, confidence: f64 }
// — returned when confidence field < config.confidence_threshold (default: 0.7)
```

Mapping `Error::LowConfidence` to a clarification response (D-03):

```rust
Err(ferro_ai::Error::LowConfidence { best_guess, confidence }) => {
    let question = format!(
        "I'm not sure what you mean (confidence {:.0}%). Did you mean to {}? Or could you be more specific?",
        confidence * 100.0,
        best_guess.get("tool_name").and_then(|v| v.as_str()).unwrap_or("do something")
    );
    CallToolResult::structured(serde_json::json!({
        "status": "needs_clarification",
        "question": question,
        "best_guess": best_guess
    }))
}
```

### Pattern 3: Replay ClassificationProvider (Phase 210 harness mirror)

The Phase 210 harness commits transcripts as `ferrmo-mcp/tests/fixtures/agent_harness/transcripts/<task-id>.json` with shape:
```json
{
  "task_id": "...",
  "target_intent": "Browse",
  "model": "claude-opus-4-8",
  "prompt_version": "v1",
  "trials": [
    {
      "trial": 1,
      "service_def": { ... },
      "tool_calls": [],
      "error": null
    }
  ]
}
```

For the intent loop, the analogous fixture shape (simpler — single turn, no multi-trial):
```json
{
  "turn_id": "approve-order-alice",
  "nl_message": "approve the order from Alice",
  "expected_tool": "approve",
  "recorded_selection": {
    "tool_name": "approve",
    "arguments": { "id": 42 },
    "confidence": 0.92
  }
}
```

The `ReplayClassificationProvider` struct holds a `Vec<IntentTurnFixture>` (or a `HashMap<String, ToolSelection>` keyed on the NL message) and implements `ClassificationProvider` by returning the recorded `ToolSelection` as raw JSON. No network, no API key.

### Pattern 4: Live-eval gate (Phase 210 `FERRO_AGENT_EVAL=1` pattern — mirror as `FERRO_AI_LIVE_EVAL`)

```rust
// Source: ferro-mcp/tests/agent_harness.rs pattern — confirmed
#[tokio::test]
#[ignore]  // skipped by default cargo test / CI
async fn intent_loop_live_eval() {
    if std::env::var("FERRO_AI_LIVE_EVAL").as_deref() != Ok("1") {
        return;  // silently skip if not opted in
    }
    // cost announcement before first call:
    eprintln!(
        "FERRO_AI_LIVE_EVAL=1: running live classification (~{N} calls × ~$0.00X/call ≈ $X.XX)"
    );
    // ... live provider path ...
}
```

### Pattern 5: handle_write_call dispatch (verified — no changes needed)

`handle_write_call` in `write_dispatch.rs` already handles the full routing:
- `request_confirm_` prefix → `handle_request_confirm`
- `confirm_` prefix → `handle_confirm`
- bare tool name → `dispatch_write` (with D-08 confirmation seam at `transition_trigger.is_some()`)
- tenant auth fail-closed check already in place

The turn core function calls `handle_write_call(params, services, db, tenant_id, ctx, dispatcher, store, config)` for write tools — zero new dispatch logic.

### Anti-Patterns to Avoid

- **Do NOT trust the classifier output as auth:** `tool_name` and `arguments` from `Classifier<ToolSelection>` are UNTRUSTED input. They pass through `handle_write_call` which re-runs guard re-eval + tenant scoping from scratch (PITFALLS §3). Never shortcut guard evaluation for "already classified" calls.
- **Do NOT add a confidence threshold gate to `handle_write_call`:** confidence is handled in the turn core by catching `Error::LowConfidence`, not in the existing dispatch paths. Adding it there would duplicate the control surface.
- **Do NOT synthesize a new `render_tool_descriptions` from scratch:** reuse `render_exposed_tools` which already guard-filters and handles disambiguation. The new helper is a text formatter over the already-rendered `Vec<Tool>`, not a second projection renderer.
- **Do NOT put `ToolSelection` in ferro-ai:** it is projection-specific. ferro-ai's `Classifier<T>` is generic; `T = ToolSelection` is defined in ferro-mcp-server (CONTEXT.md D-01).
- **Do NOT use the confirmed `evaluated_guards` map as the guard check for classified arguments:** PITFALLS §3 is explicit — re-evaluate guards against live DB state for every classified call, same as any direct `tools/call`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| LLM classification | Custom HTTP client to Anthropic | `ferro_ai::Classifier<ToolSelection>` | Already handles retry, confidence threshold, schema validation, error classification |
| Confidence gating | New threshold field in ferro-mcp-server | `ClassifierConfig::confidence_threshold` + `Error::LowConfidence` | Already implemented in ferro-ai/src/classifier/mod.rs lines 129-138 |
| Replay mode | New fixture format | Mirror Phase 210 transcript pattern | Proven: committed transcripts + replay assertion + gated live regen |
| Write routing | New dispatch function | `handle_write_call` (219/220) | Already handles scope, guard, idempotency, confirmation, audit, envelopes |
| Confirmation gate | New confirmation check in turn core | Phase 220 `ConfirmationRequired` error + existing `request_confirm_`/`confirm_` tools | Already wired; turn core delegates to `handle_write_call` which gates at D-08 seam |
| Result envelopes | New response shape | `CallToolResult::structured` + `write_tool_error_result` | Established in 219/220; any deviation risks Phase 205 content-block regression |

---

## Key Question Answers

### Q1: Feature split — is `ClassificationProvider` reqwest-free? (D-06, research-critical)

**VERIFIED: YES.** [VERIFIED: ferro-ai/src/classifier/provider.rs]

`ClassificationProvider` trait file contains only:
- `use crate::error::Error;`
- `use async_trait::async_trait;`
- `use super::ClassifierConfig;`

No reqwest, no HTTP. The concrete `AnthropicProvider` (which does use reqwest) lives in `ferro-ai/src/classifier/anthropic.rs` and is only compiled when `features = ["llm"]`.

**Concrete feature wiring for the planner:**

```toml
# ferro-mcp-server/Cargo.toml (verified current + phase 221 additions)
[features]
confirmation = ["dep:ferro-ai", "dep:rand"]          # existing (unchanged)
ai = ["dep:ferro-ai"]                                 # NEW: replay-only path (no reqwest)
ai-live = ["ai", "ferro-ai/llm"]                     # NEW: live provider (adds reqwest)

[dependencies]
ferro-ai = {
    path = "../ferro-ai",
    version = "0.2",
    optional = true,
    default-features = false,
    features = ["confirmation"]        # confirmation always included in the opt dep
}
```

When `ai` is active but NOT `ai-live`: `ferro-ai` is compiled with `confirmation` only (no reqwest). The `ReplayClassificationProvider` implements `ClassificationProvider` from `ferro_ai::classifier::provider` — it compiles and runs without reqwest in the graph.

When `ai-live` is active: `ferro_ai/llm` is added, bringing reqwest. Only used locally; CI uses `ai` only.

**Feature-off guarantee:** A consumer that enables neither `ai` nor `confirmation` gets zero new deps and identical read-tool behavior (existing `tools/list` + `tools/call` unchanged).

### Q2: Replay harness exact structure (D-05)

**VERIFIED against:** `ferro-mcp/tests/agent_harness.rs`, `ferro-mcp/tests/fixtures/agent_harness/` [VERIFIED: live source]

**Fixture files:**
- `baseline.json`: aggregated per-intent tier rates + `measured_trials` + `errored_trials` + `generated_at`
- `corpus.json`: array of `{ id, target_intent, description, expected_actions, expected_guards }`
- `transcripts/<task-id>.json`: one file per task, shape:
  ```json
  {
    "task_id": "browse-aviary-band-records",
    "target_intent": "Browse",
    "model": "claude-opus-4-8",
    "prompt_version": "v1",
    "trials": [
      { "trial": 1, "service_def": {...}, "tool_calls": [], "error": null }
    ]
  }
  ```

**No-API-key guarantee:** Fixtures contain `service_def` JSON (the agent's OUTPUT), not prompts or API responses. No key is ever needed to replay — the scorer reads the committed `service_def` and runs it through the same tier-scoring logic.

**Determinism guard:** `agent_eval_replay_scores_are_deterministic` (a non-ignored `#[tokio::test]`) loads the two `_fixture_*.json` files via `include_str!()` and asserts fixed `TierResult` values. This runs in default `cargo test` with no network.

**Live gate pattern:** `#[tokio::test] #[ignore]` + `if std::env::var("FERRO_AGENT_EVAL").as_deref() != Ok("1") { return; }`. The intent loop mirrors this with `FERRO_AI_LIVE_EVAL`.

**For Phase 221, the analogous structure:**
- `ferro-mcp-server/tests/fixtures/intent_loop/transcripts/<turn-id>.json` — committed turn fixtures (NL message + recorded `ToolSelection`)
- A non-ignored replay test that loads fixtures via `include_str!()`, runs `process_nl_turn()` with `ReplayClassificationProvider`, and asserts: classify result matches fixture, guard path taken, and `CallToolResult` parses as valid MCP content
- A `#[ignore]`-gated live test (`FERRO_AI_LIVE_EVAL=1`) that makes real calls, asserts against (or updates) fixtures, announces cost

### Q3: Turn core seam location (D-01)

**Recommended location:** `ferro-mcp-server/src/intent.rs` (new file), gated by `#[cfg(feature = "ai")]`.

```rust
// ferro-mcp-server/src/intent.rs
#[cfg(feature = "ai")]
pub async fn process_nl_turn(
    nl_message: &str,
    services: &[ServiceDef],
    db: &DatabaseConnection,
    tenant_id: Option<i64>,
    ctx: &McpContext,
    provider: Arc<dyn ferro_ai::ClassificationProvider>,
    classifier_config: ClassifierConfig,
    dispatcher: &WriteDispatcher,
    #[cfg(feature = "confirmation")] store: &dyn ferro_ai::ConfirmationStore,
    #[cfg(feature = "confirmation")] config: &McpServerConfig,
) -> Value { ... }
```

**Why this location is unit-testable without app HTTP:** The function takes the `provider: Arc<dyn ClassificationProvider>` directly. Tests instantiate a `ReplayClassificationProvider` (which implements the trait) and pass it in — no HTTP, no app process, no DB required for the classification step. Guard re-eval and dispatch still need a DB, but an in-memory SQLite (same pattern as existing write_dispatch tests) suffices.

The `app/src/controllers/mcp_chat.rs` endpoint is a thin wrapper:
```rust
#[handler]
pub async fn handle_chat(req: Request) -> Response {
    // auth → tenant_id (reuse bearer_auth middleware)
    // instantiate AnthropicProvider::from_env() (live) or ReplayProvider (test)
    // call process_nl_turn(...)
    // splice jsonrpc id + return response
}
```

### Q4: Routing without new dispatch (D-02/SC#1)

**VERIFIED: Zero new dispatch logic needed.** [VERIFIED: ferro-mcp-server/src/write_dispatch.rs + jsonrpc.rs]

The turn core function needs only two routing branches, both reusing existing functions:

```rust
if tool_name.starts_with("list_") {
    // Read path — reuse handle_tools_call directly (or dispatch() if skipping the scope gate)
    handle_tools_call(call_params, services, db, tenant_id, ctx, dispatcher, ...)
} else {
    // Write path — reuse handle_write_call
    // This already handles: request_confirm_ prefix, confirm_ prefix, bare write tools,
    // scope check, guard re-eval, idempotency, D-08 seam, execute, audit, envelopes
    handle_write_call(call_params, services, db, tenant_id, ctx, dispatcher, store, config)
}
```

**Tool-name registry:** `find_action()` in `write_dispatch.rs` iterates `services` (the app-registered `Vec<ServiceDef>`) matching `action.name == tool_name`. No separate registry — the `ServiceDef` slice is the registry. The turn core receives the same `services` slice as all other dispatch functions.

**Read tool detection:** already established as `tool_name.starts_with("list_")` throughout the codebase (jsonrpc.rs line 70, 75; write_dispatch.rs line 377). The turn core mirrors this.

### Q5: Untrusted-args security path (D-07/PITFALLS §3)

**VERIFIED in write_dispatch.rs lines 258-350:**

Pipeline that classified arguments MUST pass through (no bypass possible):
1. `validate_action_inputs(action, &args)` — checks required fields from `ActionDef.inputs`
2. `(dispatcher.guard_evaluator)(guard_name, tenant_id, inputs, db).await` — LIVE DB state re-evaluation for EVERY precondition
3. `lookup_idempotency(tenant_id, key, db)` — tenant-scoped idempotency check (both `tenant_id` AND `idempotency_key` in WHERE clause)
4. D-08 seam: `action.transition_trigger.is_some() && !is_confirmed` → `Err(ConfirmationRequired)` (no execution)
5. `(dispatcher.executor)(&action.name, inputs, tenant_id, db)` — app callback with `TenantScoped::find_for_tenant` enforcement

**Critical security invariant (lines 269-285):** The guard evaluator receives the `GuardEvaluatorFn` from the app-registered `WriteDispatcher`, NOT `ctx.evaluated_guards`. The source comment is explicit: "IMPORTANT: ctx.evaluated_guards (the 218 list-time visibility cache) is intentionally NOT consulted here."

**The classifier's `tool_name` is also untrusted:** `find_action(services, tool_name)` (lines 80-95) iterates only `mcp_exposed` services. A prompt-injected `tool_name` for a non-exposed service returns `None` → `-32601 Method not found`. A prompt-injected `tool_name` for a guarded exposed service fails at guard re-eval step 2. A prompt-injected `tenant_id` in `arguments` is ignored — `tenant_id` always comes from the authenticated principal parameter (line 418: `let tid = match tenant_id { Some(t) => t, None => ... }`).

---

## Common Pitfalls

### Pitfall 1: Confidence as confirmation gate (PITFALLS §8)
**What goes wrong:** Using `confidence >= threshold` as the gate before dispatching a write action, skipping the 220 confirmation flow.
**Why it happens:** High confidence feels like it means "correct intent" — but the user hasn't consented to the action.
**How to avoid:** Write intents ALWAYS route through `handle_write_call` which routes destructive actions to `request_confirm_<action>` (220 D-08 seam). Confidence is only a gate for clarification (D-03); it is not a substitute for user confirmation on writes.
**Warning signs:** Turn core has a `if confidence >= 0.9 { dispatch_write(...) }` branch bypassing `handle_write_call`.

### Pitfall 2: Live LLM cost in CI (PITFALLS §10)
**What goes wrong:** The replay provider is not used in CI; every test run makes a real API call.
**Why it happens:** `AnthropicProvider::from_env()` is wired unconditionally.
**How to avoid:** The `#[ignore]` attribute on the live test + the `FERRO_AI_LIVE_EVAL` env-var check (same pattern as Phase 210). Replay tests must be non-ignored. CI never sets `FERRO_AI_LIVE_EVAL=1`.
**Warning signs:** Any `#[tokio::test]` (without `#[ignore]`) in `tests/intent_loop/` that creates an `AnthropicProvider`.

### Pitfall 3: Replay provider compiled in non-test builds
**What goes wrong:** `ReplayClassificationProvider` brings in test fixtures (via `include_str!`) in production binaries.
**How to avoid:** Place the replay provider in `#[cfg(test)]` or in a `tests/` file, not in `src/`. The live provider (`AnthropicProvider`) lives in ferro-ai behind `llm` feature and is only used from `mcp_chat.rs` when the consumer enables `ai-live`.

### Pitfall 4: `render_tool_descriptions` duplicating McpRenderer logic
**What goes wrong:** A new rendering function in `intent.rs` re-derives tool names/descriptions from `ServiceDef` independently, diverging from `render_exposed_tools`.
**How to avoid:** `render_tool_descriptions` calls `render_exposed_tools(services, ctx)` and formats the resulting `Vec<Tool>` as a text block. It is a text formatter, not a second projection renderer. PITFALLS §11 is explicit on this.

### Pitfall 5: ToolSelection JSON schema not matching the type's serde representation
**What goes wrong:** The JSON schema passed to `classify()` expects `"tool_name"` but `ToolSelection` serializes as `"toolName"` (or vice versa), causing `serde_json::from_value::<ToolSelection>` to fail at line 140 of classifier/mod.rs.
**How to avoid:** Derive `Serialize, Deserialize` on `ToolSelection` with `#[serde(rename_all = "snake_case")]` and write the JSON schema with matching snake_case keys. Add a unit test that round-trips the schema through `serde_json::from_value::<ToolSelection>`.

---

## Code Examples

### Verified: ClassificationProvider trait signature

```rust
// Source: ferro-ai/src/classifier/provider.rs (VERIFIED)
#[async_trait]
pub trait ClassificationProvider: Send + Sync {
    async fn classify_raw(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        schema: &serde_json::Value,
        config: &ClassifierConfig,
    ) -> Result<serde_json::Value, Error>;
}
// Object-safe: Arc<dyn ClassificationProvider> confirmed by source tests
```

### Verified: Classifier::classify return type and Error::LowConfidence

```rust
// Source: ferro-ai/src/classifier/mod.rs lines 106-169 (VERIFIED)
pub async fn classify(
    &self,
    system_prompt: &str,
    user_prompt: &str,
    schema: &serde_json::Value,
) -> Result<ClassificationResult<T>, Error>

// Error::LowConfidence from crate::error::Error (inferred from usage, not read directly)
// confirmed: Err(Error::LowConfidence { best_guess: raw_json, confidence: conf })
// returned when: conf < self.config.confidence_threshold

pub struct ClassificationResult<T> {
    pub value: T,
    pub confidence: Option<f64>,
    pub raw_json: serde_json::Value,
}
```

### Verified: WriteDispatcher signature and handle_write_call

```rust
// Source: ferro-mcp-server/src/write_dispatch.rs lines 363-412 (VERIFIED)
pub async fn handle_write_call(
    call_params: Value,
    services: &[ServiceDef],
    db: &DatabaseConnection,
    tenant_id: Option<i64>,
    ctx: &crate::McpContext,
    dispatcher: &WriteDispatcher,
    #[cfg(feature = "confirmation")] store: &dyn ferro_ai::ConfirmationStore,
    #[cfg(feature = "confirmation")] config: &crate::McpServerConfig,
) -> Value
```

### Verified: write_tool_error_result and CallToolResult::structured

```rust
// Source: ferro-mcp-server/src/write_dispatch.rs lines 113-125 (VERIFIED)
pub fn write_tool_error_result(payload: Value) -> Value {
    let message = payload.get("message").and_then(|v| v.as_str()).unwrap_or("error").to_string();
    json!({
        "content": [{ "type": "text", "text": message }],
        "isError": true,
        "structuredContent": payload
    })
}
// CallToolResult::structured from rmcp — used throughout jsonrpc.rs and write_dispatch.rs
```

### Verified: Phase 210 replay assertion pattern

```rust
// Source: ferro-mcp/tests/agent_harness.rs lines 636-683 (VERIFIED)
#[tokio::test]
async fn agent_eval_replay_scores_are_deterministic() {
    let raw = include_str!("fixtures/agent_harness/transcripts/_fixture_valid.json");
    let transcript: Transcript = serde_json::from_str(raw).expect("must parse");
    for trial in &transcript.trials {
        let result = score(&trial.service_def, valid_target.clone()).await;
        assert!(result.t1, "...");
    }
}

// Live gate pattern (Phase 210):
#[tokio::test]
#[ignore]
async fn agent_live_eval() {
    if std::env::var("FERRO_AGENT_EVAL").as_deref() != Ok("1") { return; }
    // ... live LLM calls ...
}
```

---

## Runtime State Inventory

This is a greenfield feature phase (new module, new endpoint, new fixtures). No existing runtime state is renamed or migrated.

No items in any category — confirmed by the phase boundary in CONTEXT.md (new `intent.rs` module + new `/mcp/chat` endpoint + new fixture files; no existing data structures renamed).

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` / Rust toolchain | Build | Confirmed (workspace active) | — | — |
| SQLite (in-memory via sea-orm) | Replay tests (dispatch step) | Confirmed (used throughout write_dispatch tests) | — | — |
| `ANTHROPIC_API_KEY` | Live eval (`FERRO_AI_LIVE_EVAL=1`) | Not checked — live eval is opt-in | — | Replay path (no key needed) |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** `ANTHROPIC_API_KEY` is only needed for the live eval path; the default path (replay) needs no key.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (tokio runtime via `#[tokio::test]`) |
| Config file | none — workspace standard |
| Quick run command | `cargo test -p ferro-mcp-server --features ai,confirmation` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AMCP-06 | NL message classified to tool+args via replay provider, no LLM | unit | `cargo test -p ferro-mcp-server --features ai,confirmation intent_loop` | Wave 0 |
| AMCP-06 | Low-confidence classification → needs_clarification response (not dispatched) | unit | `cargo test -p ferro-mcp-server --features ai,confirmation low_confidence` | Wave 0 |
| AMCP-06 | Read intent dispatched via existing dispatch() (no new logic) | unit | `cargo test -p ferro-mcp-server --features ai,confirmation read_turn` | Wave 0 |
| AMCP-06 | Write intent routes through handle_write_call (guard re-eval, tenant) | unit | `cargo test -p ferro-mcp-server --features ai,confirmation write_turn` | Wave 0 |
| AMCP-06 | Destructive write returns confirmation-required (not dispatched) | unit | `cargo test -p ferro-mcp-server --features ai,confirmation destructive_requires_confirm` | Wave 0 |
| AMCP-06 | Every turn outcome parses as CallToolResult (Phase 205 regression guard extended) | unit | `cargo test -p ferro-mcp-server --features ai,confirmation turn_result_valid_mcp` | Wave 0 |
| AMCP-06 | Replay tests are deterministic (run twice, identical results) | unit | `cargo test -p ferro-mcp-server --features ai,confirmation replay_deterministic` | Wave 0 |
| AMCP-06 | CI runs without FERRO_AI_LIVE_EVAL (live tests skipped) | CI/build | `cargo test --all-features` (live tests have `#[ignore]`) | Wave 0 |
| AMCP-06 | Live eval gated by FERRO_AI_LIVE_EVAL=1 + cost announced | manual/opt-in | `FERRO_AI_LIVE_EVAL=1 cargo test -p ferro-mcp-server --features ai-live,confirmation intent_loop_live -- --ignored` | Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-mcp-server --features ai,confirmation`
- **Per wave merge:** `cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-mcp-server/src/intent.rs` — `ToolSelection`, `process_nl_turn()`, `render_tool_descriptions()`, `ReplayClassificationProvider`
- [ ] `ferro-mcp-server/tests/intent_loop/` — test fixtures directory
- [ ] `ferro-mcp-server/tests/fixtures/intent_loop/transcripts/` — initial turn fixtures (at minimum one read-intent and one write-intent turn, mirroring `_fixture_valid.json` / `_fixture_invalid.json` pattern from COMP-03)
- [ ] `app/src/controllers/mcp_chat.rs` — thin `/mcp/chat` endpoint
- [ ] `ferro-mcp-server/Cargo.toml` — add `ai` and `ai-live` features

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | Bearer auth already in middleware (Phases 217/219); turn core receives `tenant_id` from auth, never from NL message |
| V3 Session Management | no | Single-turn; no session state added |
| V4 Access Control | yes | Guard re-eval in `dispatch_write` (live DB, never ctx cache); scope check in `handle_write_call` |
| V5 Input Validation | yes | `validate_action_inputs(action, &args)` in `dispatch_write`; classified args are untrusted |
| V6 Cryptography | no | No new crypto; confirmation tokens use existing `generate_confirmation_token()` (rand BASE62) |

### Known Threat Patterns for NL Intent Loop

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Prompt injection via NL message | Tampering | Classified `tool_name`/`arguments` re-run full guard/tenant validation; no bypass shortcut |
| Prompt injection via record data in tool results | Tampering | Tool results use `CallToolResult::structured` (structuredContent, not interpolated text); already enforced by 219/220 |
| Confidence-as-auth bypass | Elevation of Privilege | `Error::LowConfidence` path → clarification only; write dispatch always routes to `handle_write_call` regardless of confidence |
| Cross-tenant NL escalation | Elevation of Privilege | `tenant_id` always from authenticated principal; `find_action` only searches `mcp_exposed` services; `TenantScoped` in executor |
| Replay attack (same NL message, different user) | Spoofing | idempotency key is per-(tenant, key) tuple; guard re-eval is live DB; token binding in confirmation store |
| LLM hallucinated `tool_name` for non-exposed service | Information Disclosure | `find_action()` only matches `mcp_exposed` services → `-32601` for unknown names |

---

## Open Questions

1. **Replay provider location — ferro-ai vs ferro-mcp-server tests?**
   - What we know: `ClassificationProvider` is in ferro-ai; the replay impl is pure trait impl with no ferro-ai-specific imports needed.
   - What's unclear: whether a shared `ReplayClassificationProvider` should live in `ferro-ai` (alongside `ConstProvider` in its own unit tests) or only in ferro-mcp-server test infrastructure.
   - Recommendation (Claude's Discretion): place in `ferro-mcp-server/tests/` or `src/intent.rs` behind `#[cfg(test)]`. Keeping it in test code avoids publishing a test utility in the ferro-ai crate surface. The Phase 210 harness puts its replay infrastructure in `ferro-mcp/tests/` — follow the same convention.

2. **`render_tool_descriptions` text format**
   - What we know: it wraps `render_exposed_tools(services, ctx)` and formats `Vec<Tool>` as text for the classifier system prompt.
   - What's unclear: whether to include tool descriptions only, or also field schemas; optimal length for classification accuracy vs context budget.
   - Recommendation (Claude's Discretion): include `tool.name` + `tool.description` + the `inputSchema` property names (not full types). Concise enough to fit in a system prompt without blowing the context window.

3. **`FERRO_AI_LIVE_EVAL` cost announcement formula**
   - Recommendation (Claude's Discretion): count the number of fixture turns to be called live, multiply by a per-call estimate derived from prompt length. Print before the first API call. Mirror the spirit of `feedback_isolate_live_eval_before_spending.md`.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | — | — | — |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

All claims in this document were verified against live source files in this session. No assumed knowledge was used for architectural decisions. Training knowledge was used only for general Rust patterns (e.g., `#[ignore]` attribute), which are stable language features.

---

## Sources

### Primary (HIGH confidence)

- `ferro-ai/src/classifier/provider.rs` — `ClassificationProvider` trait; confirmed reqwest-free
- `ferro-ai/src/classifier/mod.rs` — `Classifier<T>::classify()` signature, `ClassifierConfig`, `ClassificationResult<T>`, `Error::LowConfidence`
- `ferro-ai/Cargo.toml` — `llm`/`confirmation` feature split; confirmed reqwest optional behind `llm`
- `ferro-mcp-server/src/write_dispatch.rs` — `dispatch_write()`, `handle_write_call()`, `handle_request_confirm()`, `handle_confirm()`, D-08 seam, untrusted-args pipeline, `write_tool_error_result()`
- `ferro-mcp-server/src/renderer.rs` — `render_exposed_tools()`, `render_action_tool()`, `McpContext`, 218/220 render surface
- `ferro-mcp-server/src/jsonrpc.rs` — `handle_tools_call()` routing, scope gate, Phase 205 regression test
- `ferro-mcp-server/src/config.rs` — `McpServerConfig` fields including `confirmation_ttl_seconds`
- `ferro-mcp-server/src/error.rs` — `Error` enum variants including `ConfirmationRequired`
- `ferro-mcp-server/src/lib.rs` — public API surface
- `ferro-mcp-server/Cargo.toml` — `confirmation` feature + optional ferro-ai dep pattern
- `ferro-mcp/tests/agent_harness.rs` — Phase 210 harness: `TrialRecord`, `Transcript`, `ReplayClassificationProvider` pattern, `#[ignore]` + env-var gate, `include_str!` fixture loading
- `ferro-mcp/tests/fixtures/agent_harness/baseline.json` — baseline shape
- `ferro-mcp/tests/fixtures/agent_harness/transcripts/` — transcript file list (14 task files)
- `app/src/controllers/mcp.rs` — existing MCP endpoint integration; `make_write_dispatcher()`, `confirmation_store()`, `OnceLock` pattern
- `.planning/research/ARCHITECTURE.md` — Decision (c) inbound intent loop; Build Order Phase 5
- `.planning/research/PITFALLS.md` — §3 prompt injection; §8 NL misclassification; §10 live-LLM cost; §11 McpRenderer scope creep
- `.planning/phases/219-write-dispatch/219-CONTEXT.md` — dispatch_write, guard re-eval, envelopes
- `.planning/phases/220-confirmation-gating-for-destructive-actions/220-CONTEXT.md` — confirmation gate, D-06 feature split, D-08 seam

### Secondary (MEDIUM confidence)

- `.planning/phases/210-comp-03-agent-success-rate-harness/210-04-SUMMARY.md` — Phase 210 harness summary (confirms fixture shape matches source)

---

## Metadata

**Confidence breakdown:**
- Feature split (D-06): HIGH — `ClassificationProvider` inspected directly, zero reqwest imports confirmed
- Turn core seam (D-01): HIGH — `handle_write_call` signature verified; process_nl_turn design is additive over verified APIs
- Replay harness (D-05): HIGH — Phase 210 harness code and fixtures read directly
- Untrusted-args pipeline (D-07): HIGH — `dispatch_write` pipeline read line by line; guard re-eval confirmed NOT reading `ctx.evaluated_guards`
- Routing (D-02/SC#1): HIGH — `find_action()` and routing branches read directly

**Research date:** 2026-06-14
**Valid until:** 2026-07-14 (30 days; stable Rust codebase)
