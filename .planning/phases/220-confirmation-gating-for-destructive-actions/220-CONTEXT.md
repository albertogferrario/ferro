# Phase 220: Confirmation Gating for Destructive Actions - Context

**Gathered:** 2026-06-14
**Status:** Ready for planning
**Mode:** `--auto` (decisions are recommended defaults grounded in the v15.0 research docs + phase success criteria; logged in `220-DISCUSSION-LOG.md`)

<domain>
## Phase Boundary

A destructive/irreversible action cannot execute in a single tool call. The server issues an explicit confirmation token (via `ferro-ai::ConfirmationStore`, with a TTL); a separate confirm step validates the token at dispatch time before executing. Wraps the Phase 219 D-08 seam in `dispatch_write`.

In scope (AMCP-05):
- For each destructive action (`transition_trigger.is_some()`), synthesize a two-tool flow: `request_confirm_<action>` (issues a token) and `confirm_<action>` (validates + executes).
- A bare destructive write-tool call without a valid token → structured "confirmation required" (not executed).
- Token bound to action + record + tenant; mismatch → error. TTL expiry → reject.
- A `confirmation` Cargo feature flag: consumers who don't enable it compile fine, read tools unaffected; only destructive write tools require it.

**Out of scope (later/deferred):** inbound NL classification loop — Phase 221 (AMCP-06); DB-backed confirmation store (production hardening — `InMemoryConfirmationStore` loses pending confirmations on restart) — deferred per REQUIREMENTS; a dedicated `requires_confirmation`/`irreversible` flag on `ActionDef` — 220 uses `transition_trigger.is_some()` as the destructive signal (no projection-layer change).

</domain>

<decisions>
## Implementation Decisions

### Tool surface (D-01)
- **D-01:** Per destructive action `<action>`, the renderer (when the `confirmation` feature is on) synthesizes **two** tools: `request_confirm_<action>` (inputs = the action's `ActionDef.inputs` incl. record identifier; validates inputs + re-evaluates guards exactly as 219's write path; issues a confirmation token; stores the validated-input payload via `ConfirmationStore::request_confirmation(token, payload, ttl)`; returns `{ confirmation_token, expires_in_seconds }`) and `confirm_<action>` (input = `{ confirmation_token }` + the record identifier for the mismatch check; `confirm(token)` → payload or `None`; on valid token → runs `dispatch_write` with the stored payload, exactly once). This matches SC#2's named flow.
- The bare destructive `<action>` write tool: if invoked directly through `dispatch_write` (the D-08 seam) without a valid confirmation context, it returns a structured **confirmation-required** response pointing the agent to `request_confirm_<action>` (SC#1) — it does NOT execute. Non-destructive write tools are unaffected (execute directly as in 219).

### Token binding (D-02)
- **D-02:** The confirmation token is bound to `(tenant_id, action_name, record_id)`. The stored payload includes these so `confirm_<action>` can verify the token matches the action and record being confirmed and the calling tenant. A token issued for action/record A used on a different action/record (or a different tenant) → mismatch error, not execution (SC#4). The token is a high-entropy server-generated string (not agent-supplied). `confirm()` consumes the token (single-use → exactly-once, SC#2).

### TTL / expiry (D-03)
- **D-03:** TTL is configurable via `McpServerConfig` (range 5–10 min per roadmap; default 300s) and passed to `request_confirmation(token, payload, ttl)`. Expiry is handled by the store's internal timer: a `confirm()` after the TTL returns `None` → `confirm_<action>` returns a "confirmation expired" structured error, no execution (SC#3). No separate expiry bookkeeping in ferro-mcp-server.

### Confirmation store wiring (D-04)
- **D-04:** Use `ferro_ai::InMemoryConfirmationStore` (the v15.0 walking-skeleton store; DB-backed deferred). The store is registered the way the 219 executor/guard-evaluator are — held by the confirmation-aware dispatch path (a field on a confirmation extension of `WriteDispatcher`, or a `&dyn ConfirmationStore` param threaded alongside the dispatcher). Exact placement is research/discretion; it must NOT leak into the non-`confirmation` build.

### Destructive detection (D-05)
- **D-05:** Destructive = `action.transition_trigger.is_some()` (reuse the 219/218 signal — state transitions are the irreversible class). No new `ActionDef` field in 220; a future explicit `requires_confirmation`/`irreversible` flag is deferred.

### Feature flag + dependency hygiene (D-06) — the central decision
- **D-06:** A `confirmation` Cargo feature on `ferro-mcp-server` gates: the `ferro-ai` dependency (as an **optional** dep), the synthesized confirm tools, and the D-08 seam interception. Feature OFF → ferro-mcp-server compiles with no `ferro-ai`/HTTP-client deps, read tools and non-destructive write tools work, destructive actions behave as in 219 (callable, no gate) — i.e. additive and backward-compatible (SC#5). Feature ON → confirmation gating active.
- **Dependency-hygiene problem to solve (RESEARCH-CRITICAL):** `ferro-ai/Cargo.toml` makes `reqwest` + `reqwest-eventsource` **non-optional** hard deps. Pulling `ferro-ai` into `ferro-mcp-server` (even feature-gated) would drag an HTTP client into the MCP server crate, which `ConfirmationStore` does not need. Per ARCHITECTURE build-order Phase 4, the cleanest fix is to **feature-gate `ferro-ai`** so the confirmation module can be used without the LLM/classification client: make `reqwest`/`reqwest-eventsource`/`llmclient` optional behind a default `llm` (or `classification`) feature, and expose a `confirmation` feature that excludes them. Then `ferro-mcp-server` depends on `ferro-ai` with `default-features = false, features = ["confirmation"]`. Research MUST verify the `ferro-ai` `confirmation` module (`src/confirmation/`) is transitively reqwest-free so this gating is clean; if it isn't, the fallback is extracting `ConfirmationStore` into a small `ferro-confirmation` crate. Pre-1.0 breaking changes to ferro-ai's feature surface are acceptable (CLAUDE.md). Whatever is chosen must keep `cargo build` of every `ferro-*` crate toolchain-only and not add C system deps to the default ferro-mcp-server graph.

### Result envelopes (D-07)
- **D-07:** All confirmation outcomes — confirmation-required, token issued, expired, mismatch, executed — use the 219 result envelopes: `CallToolResult::structured` for success/issued, `write_tool_error_result` (isError:true) for expired/mismatch/denied. No bare `content[]`. Reuse the 219 strict-deser guard so every new confirm-tool result parses as `rmcp::model::CallToolResult`.

### Seam insertion (D-08)
- **D-08:** The confirmation interception lives at the Phase 219 D-08 seam in `dispatch_write` (`write_dispatch.rs:281`): when `confirmation` is on AND `action.transition_trigger.is_some()` AND no valid confirmation context, short-circuit to confirmation-required before the executor. The two synthesized tools (`request_confirm_`/`confirm_`) wrap the same `dispatch_write` machinery — confirmation is a gate around the existing execution, not a parallel path (single source of truth).

### Claude's Discretion
- Token format/entropy and the exact `confirmation_token` field name.
- Whether the store is a new param vs a field on a confirmation-extended dispatcher.
- The exact `McpServerConfig` field name for the TTL.
- Whether `confirm_<action>` re-runs guard re-evaluation at execute time (recommended: yes — guards re-checked at confirm, since live state may have changed between request and confirm).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### v15.0 design
- `.planning/research/ARCHITECTURE.md` §"Decision (c): Inbound Intent Loop → Confirmation gating" + §"Build Order → Phase 4 — Confirmation gating" (the feature-flag / dependency-narrowing note is the D-06 basis).
- `.planning/research/PITFALLS.md` §5 (DESTRUCTIVE WRITE WITHOUT CONFIRMATION — the threat this phase closes; the two-step confirm + idempotency mechanisms).
- `.planning/research/FEATURES.md` — confirmation/safety rows; the "no second permission system" / single-source principle.
- `.planning/REQUIREMENTS.md` — AMCP-05 (the requirement this phase closes); the deferred "DB-backed confirmation store" note.

### Phase 219 foundation
- `.planning/phases/219-write-dispatch/219-CONTEXT.md` + `219-SECURITY.md` — the D-08 seam, `dispatch_write` pipeline, the 219 result envelopes confirmation reuses.

### Code touch-points (read before editing)
- `ferro-mcp-server/src/write_dispatch.rs` — the D-08 seam at ~line 281 (`transition_trigger` reference); `dispatch_write` is what confirmation wraps; `write_tool_error_result` for error envelopes.
- `ferro-mcp-server/src/renderer.rs` — `render_exposed_tools` / `render_action_tool` (synthesize `request_confirm_<action>` + `confirm_<action>` per destructive action, feature-gated).
- `ferro-mcp-server/src/config.rs` — `McpServerConfig` (TTL field).
- `ferro-mcp-server/src/jsonrpc.rs` — `handle_tools_call`/`handle_write_call` routing for the new confirm tools.
- `ferro-mcp-server/Cargo.toml` — the `confirmation` feature + optional `ferro-ai` dep.
- `ferro-ai/src/confirmation/mod.rs` — the `ConfirmationStore` trait (`request_confirmation`/`confirm`/`reject`/`get`/`list_pending`); `store.rs` — `InMemoryConfirmationStore::new()` + the internal TTL timer (expiry semantics).
- `ferro-ai/Cargo.toml` — the non-optional `reqwest`/`reqwest-eventsource` deps to feature-gate (D-06).
- `ferro-projections/src/action.rs` — `ActionDef.transition_trigger` (the destructive signal; read-only).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro_ai::ConfirmationStore` + `InMemoryConfirmationStore` (`ferro-ai/src/confirmation/`): `request_confirmation(key, payload, ttl)` stores + starts a TTL timer; `confirm(key)` returns the payload or `None` (expired/missing → SC#3); `reject`/`get`/`list_pending` available. TTL is per-call, not constructor-set.
- The Phase 219 D-08 seam (`write_dispatch.rs:281-285`): a clean comment placeholder exactly where confirmation intercepts — `transition_trigger.is_some()` already referenced.
- 219 `dispatch_write` + `write_tool_error_result` + the strict-deser test harness: confirmation reuses the execution machinery and result envelopes (single source of truth).
- `ActionDef.transition_trigger` (218/219): the destructive signal.

### Established Patterns
- Reuse the existing execution path; confirmation is a gate around `dispatch_write`, not a parallel implementation (FEATURES "no parallel handler" principle).
- Framework-owned config via `McpServerConfig`; the app registers stores/callbacks (219 executor/guard-evaluator precedent).
- Project-agnostic ferro-* crates; toolchain-only builds (the D-06 reqwest concern is exactly this principle applied to a transitive dep).

### Integration Points
- `dispatch_write` D-08 seam — where the confirmation check short-circuits.
- `render_exposed_tools` — where the two confirm tools are synthesized (feature-gated).
- `ferro-ai` feature surface — the optional-dep boundary that keeps the default ferro-mcp-server build HTTP-client-free.

</code_context>

<specifics>
## Specific Ideas

- "An unconfirmed, mismatched, or expired attempt does not mutate data" (AMCP-05) is the spine — every non-happy path must be a structured non-executing response, verified by tests for each (no-token, mismatch, expired).
- Re-evaluate guards at confirm time, not only at request time — live state may change between the two steps; this keeps the 219 fail-closed guarantee intact across the confirmation gap.
- The feature flag must be genuinely additive: a consumer that never enables `confirmation` must see zero new deps and identical read-tool behavior (SC#5) — this is a build-graph assertion, not just a runtime one.

</specifics>

<deferred>
## Deferred Ideas

- Inbound NL classification loop (`Classifier<ToolSelection>` → tool+args, confirmation-gated) + `FERRO_AI_LIVE_EVAL` replay/smoke — Phase 221 (AMCP-06).
- DB-backed/persistent `ConfirmationStore` (survives process restart) — production hardening, deferred per REQUIREMENTS.
- Explicit `requires_confirmation`/`irreversible` flag on `ActionDef` (beyond `transition_trigger`) — revisit if non-transition actions need confirmation.
- gestiscilo adoption of the confirm flow — consumer-repo follow-up.

</deferred>

---

*Phase: 220-confirmation-gating-for-destructive-actions*
*Context gathered: 2026-06-14*
