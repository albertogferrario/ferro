---
phase: 221-inbound-nl-intent-loop
verified: 2026-06-14T10:00:00Z
status: gaps_found
score: 4/5 must-haves verified
overrides_applied: 0
gaps:
  - truth: "A natural-language message is classified to a ToolSelection via ferro-ai::Classifier; the result is guard-checked and routed to the existing dispatch/handle_tools_call (read) or dispatch_write/handle_write_call (write) path — no separate classification-specific dispatch logic."
    status: partial
    reason: "SC#1's 'guard-checked' guarantee is NOT met for the read path on the NL surface. The direct /mcp handler applies Gate::authorize_for + mcp_ability fail-closed check before dispatching any list_* read tool (mcp.rs:251-301). The /mcp/chat handler delegates directly to process_nl_turn, which routes list_* to handle_tools_call — a library function that applies no Gate::authorize_for check. A user denied the 'view-orders' ability (or any service with mcp_ability = None) is blocked on the direct /mcp path but succeeds on the /mcp/chat NL path by phrasing the same request in natural language. This is the WR-01 authorization bypass confirmed by the code review."
    artifacts:
      - path: "app/src/controllers/mcp_chat.rs"
        issue: "handle_chat delegates to process_nl_turn without calling Gate::authorize_for or checking service.mcp_ability — no fail-closed guard for read tools"
      - path: "ferro-mcp-server/src/intent.rs"
        issue: "process_nl_turn routes list_* to handle_tools_call; handle_tools_call is a library function that does not perform Gate::authorize_for; the library cannot call the app-level Gate (and should not)"
      - path: "ferro-mcp-server/src/jsonrpc.rs"
        issue: "handle_tools_call contains no Gate::authorize_for call — authorization is app-layer concern; the app (mcp.rs) applies it before calling handle_tools_call on the direct path, but mcp_chat.rs does not"
    missing:
      - "Gate::authorize_for check in handle_chat (mcp_chat.rs) applied after classification (tool_name known) and before routing to process_nl_turn, or passed as a callback/hook into process_nl_turn. Must mirror the fail-closed logic at mcp.rs:251-301: (a) strip 'list_' to find the service, (b) deny if service.mcp_ability is None, (c) call Gate::authorize_for with a loaded User, deny on Err."
      - "User must be loaded from the principal's user_id before process_nl_turn is called (same Pitfall-7 pattern as mcp.rs:268-276), since Gate::authorize_for requires a concrete User."
deferred: []
---

# Phase 221: Inbound NL Intent Loop — Verification Report

**Phase Goal:** A natural-language message is classified to a tool and arguments, guard-checked, confirmation-gated for write intents, dispatched, and the result returned — the full conversational turn. The loop is CI-testable without live-LLM spend.
**Verified:** 2026-06-14T10:00:00Z
**Status:** gaps_found
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | NL message classified to `ToolSelection` via `ferro-ai::Classifier`, guard-checked, routed to existing read/write paths — no separate dispatch logic | PARTIAL | `process_nl_turn` routes correctly via `handle_tools_call` / `handle_write_call` with no new dispatch logic. Guard check for WRITE path: inherited from `handle_write_call` (live-DB guard re-eval, scope gate). Guard check for READ path: MISSING — `mcp_chat.rs::handle_chat` calls `process_nl_turn` without `Gate::authorize_for`, whereas direct `/mcp` applies it at `mcp.rs:292`. Authorization bypass on the NL read surface. |
| 2 | A classified WRITE intent routes through the confirmation gate before execution | VERIFIED | `process_nl_turn` calls `handle_write_call` for non-`list_*` tools. `handle_write_call` checks `transition_trigger.is_some()` and returns `confirmation_required` without executing. Test `destructive_requires_confirm` asserts `exec_count == 0` and `error_kind == "confirmation_required"`. |
| 3 | `FERRO_AI_LIVE_EVAL` unset → loop runs from fixtures, exercises all paths, no LLM call; replay test is non-ignored, compiles under `ai` feature | VERIFIED | `replay_deterministic`, `read_turn`, `write_turn`, `low_confidence`, `destructive_requires_confirm`, `turn_result_valid_mcp`, `fixtures_parse_and_replay_returns_recorded` are all `#[tokio::test]` (no `#[ignore]`). `grep -c "AnthropicProvider\|FERRO_AI_LIVE_EVAL\|reqwest"` outside the `ai-live` block returns matches only within the `#[cfg(feature = "ai-live")]` live eval function. `ReplayClassificationProvider` contains no reqwest. `ferro-mcp-server/Cargo.toml`: `ai = ["dep:ferro-ai"]` with `ferro-ai` pinned `default-features=false, features=["confirmation", "classifier-trait"]` — reqwest absent. |
| 4 | `FERRO_AI_LIVE_EVAL=1` → live LLM call, result matches/updates fixture, cost announced BEFORE first call; gated `#[ignore]` + `ai-live` feature | VERIFIED | `intent_loop_live_eval` is `#[cfg(feature = "ai-live")] #[tokio::test] #[ignore]`. Early-return guard at line 193: `if std::env::var("FERRO_AI_LIVE_EVAL").as_deref() != Ok("1") { return; }`. Cost `eprintln!` at line 215 is textually before `ferro_ai::AnthropicProvider::from_env()` at line 223. `FERRO_AI_UPDATE_FIXTURES` gate present. |
| 5 | Low-confidence classification → clarification response, NOT a wrong-tool dispatch | VERIFIED | `process_nl_turn` matches `Err(ferro_ai::Error::LowConfidence { .. })` and returns `needs_clarification` envelope with `isError:false`, no dispatch. Test `low_confidence` asserts `exec_count == 0` and `structuredContent.status == "needs_clarification"`. Ambiguous fixture has `confidence: 0.3`, below default threshold `0.7`. |

**Score: 4/5 truths verified** (SC#1 PARTIAL due to read-path authorization bypass)

---

## Artifact Verification

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-mcp-server/Cargo.toml` | `ai` and `ai-live` feature definitions | VERIFIED | `ai = ["dep:ferro-ai"]` and `ai-live = ["ai", "ferro-ai/llm"]` present; `ferro-ai` dep has `default-features = false, features = ["confirmation", "classifier-trait"]` |
| `ferro-mcp-server/src/intent.rs` | `ToolSelection` type + `render_tool_descriptions` + `process_nl_turn` | VERIFIED | All three present. `ToolSelection` has `#[serde(rename_all = "snake_case")]`. `render_tool_descriptions` calls `render_exposed_tools`. `process_nl_turn` routes `list_*` to `handle_tools_call` and others to `handle_write_call`. |
| `ferro-mcp-server/tests/intent_loop.rs` | `ReplayClassificationProvider` + fixture loader + `process_nl_turn` tests | VERIFIED | `ReplayClassificationProvider` implements `ClassificationProvider` via `HashMap<String, serde_json::Value>`, reqwest-free. Six non-ignored tests, one `#[ignore]` live-eval test under `ai-live`. |
| `ferro-mcp-server/tests/fixtures/intent_loop/transcripts/approve-order.json` | Non-destructive write turn fixture with `recorded_selection` | VERIFIED | Contains `tool_name: "approve"`, `arguments: {"id": 42}`, `confidence: 0.92`. No secrets. |
| `ferro-mcp-server/tests/fixtures/intent_loop/transcripts/list-orders.json` | Read turn fixture | VERIFIED | `tool_name: "list_order"`, `confidence: 0.95`. |
| `ferro-mcp-server/tests/fixtures/intent_loop/transcripts/cancel-order.json` | Destructive write fixture (mapped to `submit` in actual test service) | VERIFIED | `expected_tool: "submit"`, `confidence: 0.9`. Note: filename says "cancel" but the test service uses `submit` as the destructive action — the test asserts `expected_tool == "submit"` correctly. |
| `ferro-mcp-server/tests/fixtures/intent_loop/transcripts/ambiguous.json` | Low-confidence fixture | VERIFIED | `confidence: 0.3`, below 0.7 threshold. |
| `app/src/controllers/mcp_chat.rs` | Thin POST /mcp/chat endpoint calling `process_nl_turn` | PARTIAL | Calls `process_nl_turn` correctly. App identity from `McpServerConfig::from_env()`. Tenant from `ferro::current_tenant()`. No hardcoded strings. Route is unconditionally registered (WR-02). **Missing: Gate::authorize_for check before routing read tools via NL (WR-01 — the authorization bypass).** |
| `app/src/controllers/mod.rs` | `pub mod mcp_chat;` | VERIFIED | Present at line 5. |
| `app/src/routes.rs` | `/mcp/chat` route registration | VERIFIED | `post!("/mcp/chat", controllers::mcp_chat::handle_chat).name("mcp.chat")` in the same bearer-auth-protected group as `/mcp`. |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `intent.rs::render_tool_descriptions` | `renderer.rs::render_exposed_tools` | function call over `Vec<Tool>` | VERIFIED | Line 54: `let tools = render_exposed_tools(services, ctx)?;` |
| `ReplayClassificationProvider` | `ferro_ai::ClassificationProvider` | `async-trait impl` | VERIFIED | `impl ClassificationProvider for ReplayClassificationProvider` present, no reqwest |
| `intent.rs::process_nl_turn` (write branch) | `write_dispatch.rs::handle_write_call` | direct call | VERIFIED | Lines 193-205: `handle_write_call(call_params, ...)` called for non-`list_*` tools |
| `intent.rs::process_nl_turn` (read branch) | `jsonrpc.rs::handle_tools_call` | direct call for `list_*` | VERIFIED | Lines 176-187: `handle_tools_call(call_params, ...)` called for `list_*` tools |
| `intent.rs::process_nl_turn` (low-confidence) | `ferro_ai::Error::LowConfidence` | match arm | VERIFIED | Lines 131-155: `Err(ferro_ai::Error::LowConfidence)` → `needs_clarification`, no dispatch |
| `mcp_chat.rs::handle_chat` | `ferro_mcp_server::intent::process_nl_turn` | single call after auth/tenant | VERIFIED | Line 92: `ferro_mcp_server::intent::process_nl_turn(...)` |
| `routes.rs` | `controllers::mcp_chat::handle_chat` | `post!("/mcp/chat", ...)` | VERIFIED | Line 60 of routes.rs |
| **`mcp_chat.rs` Gate check** | **`Gate::authorize_for` before NL read dispatch** | **app-level ability gate** | **NOT WIRED** | **`handle_chat` applies no `Gate::authorize_for` or `mcp_ability` check before delegating to `process_nl_turn`. The direct `/mcp` path does (mcp.rs:251-301). This is the WR-01 bypass.** |

---

## Authorization Bypass: WR-01 — Independent Verification

The code review's WR-01 finding is confirmed by independent inspection.

**Direct /mcp path (mcp.rs, lines 251-301):**
```
if let Some(service_name) = tool_name.strip_prefix("list_") {
    // 1. Find mcp_exposed service
    // 2. Load User from user_id (Pitfall-7 workaround)
    // 3. Fail-closed: service.mcp_ability == None → deny
    // 4. Gate::authorize_for(&user, ability, None) → deny on Err
}
// Only reaches handle_tools_call if gate passed
```

**NL /mcp/chat path (mcp_chat.rs, lines 72-106 → intent.rs, lines 174-187):**
```
// mcp_chat.rs: no Gate check
process_nl_turn(&nl_message, ...).await
// intent.rs: if list_* → handle_tools_call directly (no Gate)
```

**Consequence:** A tenant user with the `view-orders` ability denied (or a service with `mcp_ability = None`) is blocked on `/mcp` but can retrieve the same data via `/mcp/chat` with a natural-language query like "show me the orders". The WRITE path is not affected — `handle_write_call` enforces scope gate + live-DB guard re-eval + idempotency + confirmation; those layers are present on both paths.

**Severity for SC#1:** SC#1 requires the classified result to be "guard-checked" with "no trust shortcut". The read-path Gate is the app-level policy layer (AMCP-11, D-04) that controls which tenants can see which projections. Bypassing it means the NL read path has a lower authorization bar than the direct read path, violating the "no trust shortcut" guarantee. This is a real gap, not a deferred item — there are no later milestone phases that address it.

---

## Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| `app/src/controllers/mcp_chat.rs:109-118` | Non-`ai-live` fallback handler returns HTTP 200 with `isError:true` JSON; route is registered unconditionally | Warning | Authenticated users can probe for feature availability; leaks `"NL intent loop requires the ai-live feature"` string. Not a blocker for the goal but the comment "the route is not exposed" at line 109-111 is factually incorrect — the route IS registered. |
| `ferro-mcp-server/tests/intent_loop.rs:766-838` | `replay_deterministic` runs the `approve` write path twice without an idempotency key; asserts structuredContent equality but does not assert `exec_count == 1` for the second run | Info | The test name implies idempotent replay but only proves two independent mock executions produce equal output. The mock executor is a constant-returning closure so this passes trivially. Not a blocker. |

---

## Behavioral Spot-Checks

Step 7b is SKIPPED for live server integration checks. The implementation depends on DB + Anthropic API for the live path. The replay path is exercised by the non-ignored test suite.

Compile-time checks (inferred from code structure):
- `cargo build -p ferro-mcp-server` (no features): Verified by artifact existence and feature gating — `intent.rs` is `#[cfg(feature = "ai")]`; no intent code compiles without the feature.
- `ai` feature: `ferro-ai` dep with `default-features = false, features = ["confirmation", "classifier-trait"]` — reqwest absent per Cargo.toml.
- `ai-live` feature: adds `ferro-ai/llm` (reqwest); used only by `intent_loop_live_eval` gated `#[cfg(feature = "ai-live")]`.

---

## Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| AMCP-06 | 221-01, 221-02, 221-03 | NL message classified to tool+args, guard-checked, confirmation-gated, dispatched — loop CI-testable without live LLM | PARTIAL | SC#2, SC#3, SC#4, SC#5 satisfied. SC#1 partial: write path guard-checked (handle_write_call), read path Gate check missing on NL surface. |

---

## Human Verification Required

None. The gaps identified are programmatically verifiable from the code.

---

## Gaps Summary

One gap blocks full goal achievement. The phase successfully delivers: ToolSelection type, render_tool_descriptions, ReplayClassificationProvider, four committed fixtures, process_nl_turn with correct read/write routing, write-path guard re-eval (inherited from handle_write_call), confirmation gate for destructive writes, low-confidence clarification, all non-ignored replay tests, and the /mcp/chat endpoint skeleton.

The single gap is the read-path authorization bypass (WR-01): the `/mcp/chat` NL endpoint routes `list_*` tool names to `handle_tools_call` without first calling `Gate::authorize_for` + `mcp_ability` fail-closed check. The direct `/mcp` path applies this check before dispatching. The fix requires adding the Gate check inside `handle_chat` after classification (tool_name is known after `process_nl_turn` classifies, but the Gate needs to fire before dispatch). One workable pattern: call `process_nl_turn` and inspect the `tool_name` classification result before committing to dispatch — or accept a pre-dispatch hook parameter in `process_nl_turn` that fires after classification. The simplest safe approach mirrors mcp.rs lines 251-276: load the User from `user_id` (already parsed), find the service by stripping `list_`, fail-closed on `mcp_ability = None`, call `Gate::authorize_for`.

The secondary issues (WR-02 unconditional route registration, WR-03 misleading `replay_deterministic` test) are warnings that do not block the phase goal but should be addressed before shipping the endpoint.

---

_Verified: 2026-06-14T10:00:00Z_
_Verifier: Claude (gsd-verifier)_
