---
phase: 221-inbound-nl-intent-loop
verified: 2026-06-14T12:00:00Z
status: passed
score: 5/5 must-haves verified
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: 4/5
  gaps_closed:
    - "SC#1 read-path authorization bypass (WR-01): process_nl_turn now accepts authorize_read closure; app closure is fail-closed (None -> deny, Some(a) -> Gate::authorize_for); read_denied_by_ability_gate regression test added and non-ignored"
  gaps_remaining: []
  regressions: []
---

# Phase 221: Inbound NL Intent Loop — Verification Report

**Phase Goal:** A natural-language message is classified to a tool and arguments, guard-checked, confirmation-gated for write intents, dispatched, and the result returned — the full conversational turn. The loop is CI-testable without live-LLM spend.
**Verified:** 2026-06-14T12:00:00Z
**Status:** passed
**Re-verification:** Yes — after gap closure (commit 13378c0a)

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | NL message classified to `ToolSelection` via `ferro-ai::Classifier`, guard-checked, routed to existing read/write paths — no separate dispatch logic | VERIFIED | `process_nl_turn` now accepts `authorize_read: &(dyn Fn(Option<&str>) -> bool + Sync)`. After classification resolves a `list_*` tool to its `ServiceDef`, it calls `authorize_read(service.mcp_ability.as_deref())` BEFORE dispatch (`intent.rs:203`). On `false`: returns `access_denied` envelope (`isError:true`, `structuredContent.status=="access_denied"`) with no dispatch (`intent.rs:205-214`). Unknown service returns `method_not_found` (`intent.rs:195-200`). `mcp_chat.rs:99-107` loads the concrete `User` from `user_id` (from principal, not body) and builds a fail-closed closure: `None => false`, `Some(a) => Gate::authorize_for(&user, a, None).is_ok()`. This mirrors `mcp.rs:251-301` exactly. Write branch unchanged: `handle_write_call` called without consulting `authorize_read` (`intent.rs:236-248`). |
| 2 | A classified WRITE intent routes through the confirmation gate before execution | VERIFIED | `process_nl_turn` calls `handle_write_call` for non-`list_*` tools. `handle_write_call` checks `transition_trigger.is_some()` and returns `confirmation_required` without executing. Test `destructive_requires_confirm` asserts `exec_count == 0` and `error_kind == "confirmation_required"`. |
| 3 | `FERRO_AI_LIVE_EVAL` unset → loop runs from fixtures, exercises all paths, no LLM call; replay test is non-ignored, compiles under `ai` feature | VERIFIED | `replay_deterministic`, `read_turn`, `write_turn`, `low_confidence`, `destructive_requires_confirm`, `turn_result_valid_mcp`, `fixtures_parse_and_replay_returns_recorded`, `read_denied_by_ability_gate` are all `#[tokio::test]` with no `#[ignore]`. Only `intent_loop_live_eval` carries `#[ignore]` (`intent_loop.rs:191`). `ReplayClassificationProvider` contains no reqwest. `ferro-mcp-server/Cargo.toml`: `ai = ["dep:ferro-ai"]` with `ferro-ai` `default-features=false, features=["confirmation", "classifier-trait"]` — reqwest absent. |
| 4 | `FERRO_AI_LIVE_EVAL=1` → live LLM call, result matches/updates fixture, cost announced BEFORE first call; gated `#[ignore]` + `ai-live` feature | VERIFIED | `intent_loop_live_eval` is `#[cfg(feature = "ai-live")] #[tokio::test] #[ignore]`. Early-return guard at line 193. Cost `eprintln!` at line 215 is textually before `ferro_ai::AnthropicProvider::from_env()` at line 223. `FERRO_AI_UPDATE_FIXTURES` gate present. |
| 5 | Low-confidence classification → clarification response, NOT a wrong-tool dispatch | VERIFIED | `process_nl_turn` matches `Err(ferro_ai::Error::LowConfidence { .. })` and returns `needs_clarification` envelope with `isError:false`, no dispatch. Test `low_confidence` asserts `exec_count == 0` and `structuredContent.status == "needs_clarification"`. Ambiguous fixture has `confidence: 0.3`, below default threshold `0.7`. |

**Score: 5/5 truths verified**

---

## SC#1 Gap Closure — Independent Verification

The WR-01 authorization bypass (previous gap) is closed. Evidence by file:line:

**`ferro-mcp-server/src/intent.rs`**
- Line 108: `authorize_read: &(dyn Fn(Option<&str>) -> bool + Sync)` parameter added to `process_nl_turn`
- Lines 185-217: after classification, `list_*` branch resolves `ServiceDef` then:
  - Line 191-200: unknown service → `method_not_found` (no dispatch)
  - Line 203: `if !authorize_read(service.mcp_ability.as_deref())` → deny before dispatch
  - Lines 205-214: `access_denied` envelope (`isError:true`, `structuredContent.status=="access_denied"`)
  - Line 219: `handle_tools_call` reached only after gate passes
- Lines 236-248: write branch calls `handle_write_call` — `authorize_read` never consulted for writes

**`app/src/controllers/mcp_chat.rs`**
- Lines 99-102: `User::find_by_id(user_id)` loads concrete User from principal (not body)
- Lines 103-107: fail-closed closure: `None => false`, `Some(a) => Gate::authorize_for(&user, a, None).is_ok()`
- Line 110-126: closure passed as `&authorize_read` to `process_nl_turn`

**`ferro-mcp-server/tests/intent_loop.rs`**
- Line 500: `#[tokio::test]` (no `#[ignore]`)
- Line 501: `async fn read_denied_by_ability_gate()`
- Line 535: `&|_| false` — deny-all closure
- Lines 547-551: asserts `exec_count == 0`
- Lines 553-558: asserts `isError == true`
- Lines 559-563: asserts `structuredContent.status == "access_denied"`

**Parity with direct `/mcp` path (`mcp.rs:251-301`):**
- Direct path: strip `list_` → find service → deny if `mcp_ability == None` → `Gate::authorize_for` → dispatch
- NL path: strip `list_` → find service → `authorize_read(service.mcp_ability.as_deref())` where the closure is `None => false, Some(a) => Gate::authorize_for(...)` → dispatch

The logic is equivalent. The seam is correctly placed at the library boundary: `process_nl_turn` accepts a closure rather than calling the app-layer Gate directly, which keeps the library project-agnostic (CLAUDE.md rule).

---

## Artifact Verification

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-mcp-server/Cargo.toml` | `ai` and `ai-live` feature definitions | VERIFIED | `ai = ["dep:ferro-ai"]` and `ai-live = ["ai", "ferro-ai/llm"]` present; `ferro-ai` dep has `default-features = false, features = ["confirmation", "classifier-trait"]` |
| `ferro-mcp-server/src/intent.rs` | `ToolSelection` type + `render_tool_descriptions` + `process_nl_turn` with `authorize_read` gate | VERIFIED | All present. `authorize_read` parameter added; gate fires before dispatch on `list_*` path; write path unchanged. |
| `ferro-mcp-server/tests/intent_loop.rs` | `ReplayClassificationProvider` + fixture tests + `read_denied_by_ability_gate` non-ignored test | VERIFIED | `ReplayClassificationProvider` implements `ClassificationProvider`, reqwest-free. Eight non-ignored tests including `read_denied_by_ability_gate`. One `#[ignore]` live-eval test under `ai-live`. |
| `ferro-mcp-server/tests/fixtures/intent_loop/transcripts/approve-order.json` | Non-destructive write turn fixture | VERIFIED | `tool_name: "approve"`, `confidence: 0.92` |
| `ferro-mcp-server/tests/fixtures/intent_loop/transcripts/list-orders.json` | Read turn fixture | VERIFIED | `tool_name: "list_order"`, `confidence: 0.95` |
| `ferro-mcp-server/tests/fixtures/intent_loop/transcripts/cancel-order.json` | Destructive write fixture | VERIFIED | `expected_tool: "submit"`, `confidence: 0.9` |
| `ferro-mcp-server/tests/fixtures/intent_loop/transcripts/ambiguous.json` | Low-confidence fixture | VERIFIED | `confidence: 0.3`, below 0.7 threshold |
| `app/src/controllers/mcp_chat.rs` | POST /mcp/chat with fail-closed read gate | VERIFIED | Loads `User` from `user_id` (principal), builds fail-closed closure, passes to `process_nl_turn`. Route unconditionally registered (WR-02 — non-blocking follow-up). |
| `app/src/controllers/mod.rs` | `pub mod mcp_chat;` | VERIFIED | Present |
| `app/src/routes.rs` | `/mcp/chat` route registration | VERIFIED | `post!("/mcp/chat", controllers::mcp_chat::handle_chat).name("mcp.chat")` in bearer-auth-protected group |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `intent.rs::render_tool_descriptions` | `renderer.rs::render_exposed_tools` | function call over `Vec<Tool>` | VERIFIED | Line 54: `let tools = render_exposed_tools(services, ctx)?;` |
| `ReplayClassificationProvider` | `ferro_ai::ClassificationProvider` | `async-trait impl` | VERIFIED | `impl ClassificationProvider for ReplayClassificationProvider` present, reqwest-free |
| `intent.rs::process_nl_turn` (write branch) | `write_dispatch.rs::handle_write_call` | direct call | VERIFIED | Lines 236-248: `handle_write_call(call_params, ...)` called for non-`list_*` tools |
| `intent.rs::process_nl_turn` (read branch) | `authorize_read` closure | gate call before dispatch | VERIFIED | Line 203: `authorize_read(service.mcp_ability.as_deref())` fires before `handle_tools_call` |
| `intent.rs::process_nl_turn` (read branch) | `jsonrpc.rs::handle_tools_call` | direct call for `list_*` after gate | VERIFIED | Lines 219-231: `handle_tools_call(call_params, ...)` reached only after `authorize_read` passes |
| `intent.rs::process_nl_turn` (low-confidence) | `ferro_ai::Error::LowConfidence` | match arm | VERIFIED | Lines 141-165: `Err(ferro_ai::Error::LowConfidence)` → `needs_clarification`, no dispatch |
| `mcp_chat.rs::handle_chat` | `Gate::authorize_for` before NL read dispatch | fail-closed `authorize_read` closure | VERIFIED | Lines 103-107: `None => false`, `Some(a) => Gate::authorize_for(&user, a, None).is_ok()` |
| `mcp_chat.rs::handle_chat` | `ferro_mcp_server::intent::process_nl_turn` | single call after auth/tenant/gate-closure | VERIFIED | Line 110: `ferro_mcp_server::intent::process_nl_turn(...)` |
| `routes.rs` | `controllers::mcp_chat::handle_chat` | `post!("/mcp/chat", ...)` | VERIFIED | Line 60 of routes.rs |

---

## Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| `app/src/controllers/mcp_chat.rs` | Route registered unconditionally; non-`ai-live` handler returns HTTP 200 with `isError:true` JSON | Warning (WR-02) | Authenticated users can probe for feature availability. Non-blocking follow-up. |
| `ferro-mcp-server/tests/intent_loop.rs:842-938` | `replay_deterministic` does not assert `exec_count == 1` for the second write run; the mock executor is constant-returning so "determinism" passes trivially | Info (WR-03) | Test name implies idempotent-replay but only proves equal output from two independent mock executions. Non-blocking follow-up. |

---

## Behavioral Spot-Checks

Step 7b SKIPPED for live server integration. The replay path is exercised by the non-ignored test suite, including the new `read_denied_by_ability_gate` test.

---

## Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| AMCP-06 | 221-01, 221-02, 221-03 | NL message classified to tool+args, guard-checked, confirmation-gated, dispatched — loop CI-testable without live LLM | SATISFIED | All 5 SCs met. SC#1 read-path gate closed by commit 13378c0a. SC#2 confirmation gate verified. SC#3 replay tests non-ignored and CI-runnable. SC#4 live eval gated. SC#5 low-confidence clarification verified. |

---

## Human Verification Required

None. All gaps were programmatically verifiable and are now closed.

---

## Non-Blocking Follow-Ups

These items do not block the phase goal or AMCP-06 satisfaction. They are recorded for future phases.

**WR-02:** The `/mcp/chat` route is unconditionally registered in `routes.rs`. When the `ai-live` feature is absent, the handler returns a 200 with `isError:true` and leaks `"NL intent loop requires the ai-live feature"`. Authenticated users can probe for feature presence. Fix: gate the route registration under `#[cfg(feature = "ai-live")]`, or return 404/501 instead of 200.

**WR-03:** `replay_deterministic` runs the write path twice with independent mock executors. The mock always returns the same constant, so the two outputs are trivially equal. The test does not assert single-execution semantics (exec_count per run). If idempotency under repeated NL invocations is a correctness requirement, a dedicated test asserting `exec_count == 1` on the second call (with a shared idempotency key store) would provide that guarantee.

---

_Verified: 2026-06-14T12:00:00Z_
_Verifier: Claude (gsd-verifier)_
