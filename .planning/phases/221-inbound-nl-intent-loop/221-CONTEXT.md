# Phase 221: Inbound NL Intent Loop - Context

**Gathered:** 2026-06-14
**Status:** Ready for planning
**Mode:** `--auto` (decisions are recommended defaults grounded in the v15.0 research docs, the 5 SCs, and the existing `ferro-ai::Classifier` + Phase 210 replay harness; logged in `221-DISCUSSION-LOG.md`). The final phase of v15.0.

<domain>
## Phase Boundary

The full conversational turn: a natural-language message → classified to a tool + arguments (`ferro-ai::Classifier`) → guard-checked → confirmation-gated for writes → dispatched via the existing read/write paths → result rendered back. CI-testable without live-LLM spend via a replay/smoke path.

In scope (AMCP-06):
- A conversational-turn core that classifies NL → `ToolSelection { tool_name, arguments }`, guard-checks, routes to the **existing** `dispatch()` (read) / `dispatch_write()` (write) — no classification-specific dispatch logic.
- Write intents routed through the Phase 220 confirmation gate before execution.
- A gated replay/smoke path: `FERRO_AI_LIVE_EVAL` unset → runs from recorded transcript fixtures, no LLM, exercises all branches; `=1` → live LLM call matched against the fixture (or updates it), announcing estimated cost before the first call.
- Low-confidence classification → clarification response, not a wrong-tool dispatch.

**Out of scope (deferred):** parameter elicitation as a separate state machine (the MCP request/response loop handles multi-turn — ARCHITECTURE Decision (c)); multi-turn conversation memory; non-replay live-eval CI (live is opt-in only). This is the LAST v15.0 phase — after it, `/gsd-complete-milestone`.

</domain>

<decisions>
## Implementation Decisions

### Loop home (D-01)
- **D-01:** The conversational-turn **core** (classify → guard-check → confirmation-gate → dispatch → result) is a testable function in `ferro-mcp-server` (so SC#3/#4 are unit-testable with a replay provider, no app HTTP needed). The sample `app` wires a thin `/mcp/chat` HTTP endpoint around it (ARCHITECTURE Decision (c): the loop wires in the app; ferro-mcp-server provides the pieces). `ToolSelection { tool_name: String, arguments: Map<String,Value>, confidence: f64 }` is defined in `ferro-mcp-server` (projection-specific, not ferro-ai).

### Classification → routing (D-02)
- **D-02:** `ferro_ai::Classifier<ToolSelection>::classify(system, user, schema)` where system = a new `render_tool_descriptions(services, ctx)` helper (concise text of the guard-filtered available tools, reusing the 218 render surface), user = the NL message, schema = the `ToolSelection` JSON Schema. The returned `tool_name` is matched to a read tool (`list_*` → `dispatch()`) or a write tool (`dispatch_write()` / the 220 confirmation path) — **no new dispatch logic** (SC#1). Guard-checked using the 219 server-side guard re-eval (the agent's classification is never trusted).

### Clarification / low-confidence (D-03)
- **D-03:** Reuse the Classifier's EXISTING confidence handling: `classify()` already returns `Error::LowConfidence { best_guess, confidence }` when `confidence < config.confidence_threshold`. Map that to a structured `{ status: "needs_clarification", question, best_guess }` `CallToolResult::structured` response — do NOT dispatch (SC#5). No new confidence/threshold logic in ferro-mcp-server; configure the threshold via `ClassifierConfig`.

### Write → confirmation (D-04)
- **D-04:** A classified WRITE intent (destructive `transition_trigger.is_some()`) routes through the Phase 220 confirmation gate — the loop returns the confirmation-required response and does NOT call `dispatch_write` directly for destructive actions (SC#2). Non-destructive writes dispatch directly (219). Reuses 220; no parallel confirmation logic.

### Replay / live-eval (D-05) — reuse the proven harness
- **D-05:** REUSE the **Phase 210 COMP-03** transcript-fixture + deterministic-replay-guard pattern (`ferro-mcp/tests/fixtures/agent_harness/` — committed transcripts, a no-LLM replay assertion, a gated live-regen). A replay `ClassificationProvider` reads recorded transcripts and returns the recorded classification with no network. `FERRO_AI_LIVE_EVAL` unset → the loop runs entirely from fixtures, exercising classify/guard/confirm/dispatch/clarify branches (SC#3). `FERRO_AI_LIVE_EVAL=1` → a live provider makes the call; the result is asserted against (or updates) the fixture, and the live path **announces an estimated cost before the first call** (SC#4; honors the isolate-failures-before-spending discipline). No API keys in committed fixtures.

### Feature gating (D-06) — research-critical
- **D-06:** The intent loop pulls `ferro-ai::Classifier`. A **live** provider needs ferro-ai's `llm` feature (reqwest, from 220's split); the **replay** provider must be reqwest-free so CI runs the SC#3 path llm-free. Structure analogously to 220 D-06: an `intent` (or `ai`) Cargo feature on ferro-mcp-server enabling `ferro-ai` (with `llm` for the live provider) + the loop module; the replay `ClassificationProvider` implements the trait without the http client so the deterministic replay test compiles/runs without `llm`. Research must confirm the cleanest split (the `ClassificationProvider` trait is the seam — is it in the reqwest-free part of ferro-ai?) and that feature-off ferro-mcp-server is unaffected (SC: read tools work, no new default deps).

### Result envelopes (D-07)
- **D-07:** Every turn outcome — dispatched result, needs_clarification, confirmation-required, guard-denied — uses the 219/220 `CallToolResult::structured` / `write_tool_error_result` envelopes (reuse; no new shape). Classified arguments are UNTRUSTED (prompt-injection surface, PITFALLS §3) — they pass through the same 219 validation + guard re-eval + tenant scoping as any tool call; the classifier's `tool_name`/`arguments` are never trusted to bypass auth/guard/tenant checks.

### Claude's Discretion
- Exact `render_tool_descriptions` text format; the `ToolSelection` JSON-schema field names; the cost-estimate formula/announcement string; whether the replay provider lives in ferro-ai (alongside the client) or ferro-mcp-server tests.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `.planning/research/ARCHITECTURE.md` §"Decision (c): Inbound Intent Loop" (classification strategy, parameter elicitation, where the loop lives) + §"Build Order → Phase 5 — Inbound intent loop" (`render_tool_descriptions` re-export; loop in the app, not ferro-mcp-server).
- `.planning/research/PITFALLS.md` §3 (prompt injection — classified args/tool results are untrusted; structured content; the classifier never bypasses guard/tenant/auth) + any LLM-in-request-path reliability pitfall.
- `.planning/REQUIREMENTS.md` — AMCP-06 (the requirement this phase closes); the `FERRO_AI_LIVE_EVAL` gated-replay note.
- **Phase 210 COMP-03 replay harness (THE reuse target):** `.planning/phases/210-comp-03-agent-success-rate-harness/210-04-SUMMARY.md` + `ferro-mcp/tests/fixtures/agent_harness/` (transcripts, `agent_eval_replay_matches_baseline` determinism guard, gated regen). The D-05 replay pattern.
- Phase 219/220: `.planning/phases/219-write-dispatch/219-CONTEXT.md` (dispatch_write, guard re-eval, envelopes) + `.planning/phases/220-confirmation-gating-for-destructive-actions/220-CONTEXT.md` (the confirmation gate the loop routes writes through; the ferro-ai `llm`/`confirmation` feature split the intent feature extends).

### Code touch-points
- `ferro-ai/src/classifier/mod.rs` — `Classifier<T>::classify(system, user, schema) -> ClassificationResult<T>`; `ClassifierConfig` (`confidence_threshold`, `max_retries`, `model`); `Error::LowConfidence` (SC#5); `ClassificationProvider` trait (the replay seam, D-06).
- `ferro-mcp-server/src/{dispatch,write_dispatch,renderer,jsonrpc,config,error}.rs` — the existing dispatch paths the loop routes to; `render_tool_descriptions` is new (renderer); `ToolSelection` + the loop core are new.
- `ferro-mcp-server/Cargo.toml` + `ferro-ai/Cargo.toml` — the `intent`/`llm` feature wiring (D-06).
- `app/src/controllers/mcp.rs` — the `/mcp/chat` endpoint wiring.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro_ai::Classifier<T>::classify` — already does retry + confidence-threshold + `Error::LowConfidence` (D-03/SC#5) + schema-validated deser into `T`. The loop creates `Classifier<ToolSelection>`.
- The Phase 210 COMP-03 transcript/replay harness (`ferro-mcp/tests/fixtures/agent_harness/`) — the proven no-LLM deterministic-replay pattern to reuse for SC#3/#4.
- 219 `dispatch_write` + guard re-eval + 220 confirmation gate + the `CallToolResult::structured`/`write_tool_error_result` envelopes — the loop composes these; it adds NO new dispatch/confirm/guard logic.
- The 218 render surface (`render_exposed_tools`/`render_action_tool`) — basis for `render_tool_descriptions`.
- The 220 ferro-ai `llm`/`confirmation` feature split — the intent feature extends it (D-06).

### Established Patterns
- The classifier output is untrusted: every classified call re-runs 219 validation + guard re-eval + tenant scoping (no trust shortcut).
- Live-LLM is opt-in + cost-announced; CI is replay-only (the isolate-before-spend discipline).
- Reuse existing dispatch/confirm/envelope machinery; the loop is an entry point, not a parallel implementation (FEATURES "no parallel handler" principle).

### Integration Points
- `render_tool_descriptions(services, ctx)` → classifier system prompt.
- `Classifier<ToolSelection>` → existing `dispatch()`/`dispatch_write()`/220 confirmation.
- The replay `ClassificationProvider` → the SC#3 no-LLM CI path.
- The sample app `/mcp/chat` endpoint → hosts the loop.

</code_context>

<specifics>
## Specific Ideas

- "The loop is CI-testable without live-LLM spend" (AMCP-06) is the spine: the default test path is replay-only (no network, no key); live is `FERRO_AI_LIVE_EVAL=1` + cost-announced.
- The classifier never bypasses security: a classified write still hits 219 guard re-eval + tenant scoping + 220 confirmation. A prompt-injected tool_name/arguments cannot escalate.
- Reuse the Phase 210 harness rather than inventing a new replay mechanism — it already solved no-key fixtures + deterministic replay + gated regen.

</specifics>

<deferred>
## Deferred Ideas

- Parameter-elicitation state machine (MCP request/response handles multi-turn — ARCHITECTURE Decision (c)).
- Multi-turn conversation memory / session context beyond a single turn.
- Live-eval in CI (live stays opt-in/local only).
- gestiscilo adoption of `/mcp/chat` — consumer-repo follow-up; 221 ships framework capability + synthetic/replay validation.

</deferred>

---

*Phase: 221-inbound-nl-intent-loop*
*Context gathered: 2026-06-14*
