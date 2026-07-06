---
phase: 221-inbound-nl-intent-loop
plan: "03"
subsystem: app + ferro-mcp-server
tags: [ai, classification, intent-loop, http-endpoint, live-eval, sc4, amcp-06]
dependency_graph:
  requires: [221-02 (process_nl_turn, ReplayClassificationProvider, fixtures), ferro-ai/llm (AnthropicProvider), ferro-mcp-server/ai-live feature]
  provides: [POST /mcp/chat endpoint, intent_loop_live_eval test (SC#4)]
  affects: [app feature surface (ai-live), ferro-mcp-server test suite]
tech_stack:
  added: [ai-live feature in app/Cargo.toml (enables ferro-ai/llm + ferro-mcp-server/ai-live + confirmation)]
  patterns: [thin HTTP wrapper delegating to process_nl_turn, cfg-gated live provider instantiation, cost-announced ignore-gated live test]
key_files:
  created:
    - app/src/controllers/mcp_chat.rs
  modified:
    - app/src/controllers/mod.rs
    - app/src/routes.rs
    - app/Cargo.toml
    - app/src/controllers/mcp.rs
    - app/src/tests/mcp_write_dispatch.rs
    - ferro-mcp-server/tests/intent_loop.rs
decisions:
  - "mcp_chat.rs reuses make_write_dispatcher/exposed_services/confirmation_store from mcp.rs (pub(crate)) rather than duplicating them"
  - "key_scope extracted from principal before req.json() to avoid borrow-after-move; gated under #[cfg(feature = ai-live)] to avoid unused-variable warning in default builds"
  - "intent_loop_live_eval is #[cfg(feature = ai-live)] + #[ignore] + FERRO_AI_LIVE_EVAL=1 env guard; FERRO_AI_UPDATE_FIXTURES=1 for opt-in fixture update on mismatch"
  - "Ambiguous fixture (confidence 0.3) is expected to produce LowConfidence in live eval; all other LowConfidence results are treated as mismatches"
metrics:
  duration: "~27 minutes"
  completed: "2026-06-14"
  tasks: 2
  files: 7
requirements: [AMCP-06]
---

# Phase 221 Plan 03: NL Intent Loop — HTTP Endpoint + Live-Eval Gate Summary

Consumer-facing HTTP wiring: `POST /mcp/chat` in the sample `app` authenticates, resolves `tenant_id` from the principal, and delegates a single NL turn to `process_nl_turn` via the live `AnthropicProvider`. The `#[ignore]`-gated live-eval test announces estimated cost before the first API call and matches results against committed fixtures.

## What Was Built

### Task 1: POST /mcp/chat endpoint (app/src/controllers/mcp_chat.rs)

New controller `mcp_chat.rs` — a thin HTTP wrapper over `ferro_mcp_server::intent::process_nl_turn`:

1. `McpServerConfig::from_env()` for app identity (app_url for origin check, no hardcoded strings).
2. Origin check (mirrors `mcp.rs` lines 177-180): present but mismatched → 403.
3. Bearer principal lookup via `req.get::<serde_json::Value>()` + user_id validation.
4. `key_scope` extracted from principal **before** `req.json()` (borrow-after-move prevention); gated `#[cfg(feature = "ai-live")]` to avoid unused-variable warning in default builds.
5. Body parse: `{ "message": "<nl>" }`, 400 on empty message.
6. Under `ai-live`: db + tenant_id + McpContext + services + dispatcher + AnthropicProvider; single call to `process_nl_turn`; result returned as `HttpResponse::json`.

Three factory functions in `mcp.rs` made `pub(crate)` for reuse:
- `make_write_dispatcher()`, `exposed_services()`, `confirmation_store()`, `check_is_manager()`

Route registered inside the existing bearer+tenant protected group in `routes.rs`:
```
post!("/mcp/chat", controllers::mcp_chat::handle_chat).name("mcp.chat")
```

`app/Cargo.toml` new feature:
```toml
ai-live = ["ferro-mcp-server/ai-live", "ferro-mcp-server/confirmation", "dep:ferro-ai", "ferro-ai/llm", "confirmation"]
```

Pre-existing clippy issue fixed: `mcp_write_dispatch.rs` had `ferro_audit`, `AtomicUsize`, `Ordering`, and `Arc` imports at module level that are only used inside `#[cfg(not(feature = "confirmation"))]` tests. Gated those imports accordingly (same pattern as Plan 01 Deviation 2).

### Task 2: Live-eval gate with cost announcement (SC#4)

`ferro-mcp-server/tests/intent_loop.rs::intent_loop_live_eval` stub (from Plan 01) replaced with full implementation:

- `#[cfg(feature = "ai-live")]` — only compiles when live deps present; CI uses `--features ai,confirmation` and never sees this function.
- `#[ignore]` + `FERRO_AI_LIVE_EVAL=1` env guard — skipped by `cargo test` and CI.
- Cost `eprintln!` fires **before** `AnthropicProvider::from_env()` (isolate-before-spend discipline):
  ```
  FERRO_AI_LIVE_EVAL=1: running live classification (4 turns x ~$0.005/call = ~$0.02)
  ```
- Loads all four committed fixtures via `include_str!`.
- Instantiates `AnthropicProvider::from_env()` (requires `ANTHROPIC_API_KEY`).
- For each fixture: runs `Classifier::<ToolSelection>::new(provider, config).classify(...)` and asserts `tool_name` matches `fixture.expected_tool`.
- `FERRO_AI_UPDATE_FIXTURES=1` opt-in: prints a message indicating which fixture would need updating (manual step; never auto-rewrites committed JSON).
- Ambiguous fixture (confidence 0.3 < 0.7 threshold) expected to produce `LowConfidence` — not treated as a mismatch; all other `LowConfidence` results are.

## Verification Results

- `cargo build -p app --features ai-live`: exit 0
- `cargo build -p app` (no ai-live): exit 0
- `cargo clippy -p app --all-targets --features ai-live -- -D warnings`: exit 0
- `cargo test -p ferro-mcp-server --features ai,confirmation` (CI path): exit 0, `intent_loop_live_eval` NOT compiled
- `cargo test -p ferro-mcp-server --features ai-live,confirmation` (live path): exit 0, `intent_loop_live_eval` reported as `ignored` (FERRO_AI_LIVE_EVAL unset)
- `cargo fmt --all -- --check`: exit 0
- `cargo clippy --all --all-targets -- -D warnings`: exit 0
- `cargo test --all-features`: exit 0

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Unused-variable clippy errors in mcp_write_dispatch.rs under ai-live feature**
- **Found during:** Task 1 — `cargo clippy -p app --all-targets --features ai-live -- -D warnings`
- **Issue:** `ferro_audit::{history_for_target, AuditTarget}`, `std::sync::atomic::{AtomicUsize, Ordering}`, and `std::sync::Arc` are declared unconditionally at module level but only used inside `#[cfg(not(feature = "confirmation"))]` tests. When `ai-live` (which implies `confirmation`) is active, those tests are compiled out but imports remain — unused import warnings promoted to errors.
- **Fix:** Gated those three import lines behind `#[cfg(not(feature = "confirmation"))]`.
- **Files modified:** `app/src/tests/mcp_write_dispatch.rs`
- **Root cause:** Same pre-existing pattern as Plan 01 Deviation 2 (`McpServerConfig` import); `ai-live` feature exposes it because it transitively enables `confirmation`.

**2. [Rule 1 - Bug] Borrow-after-move in mcp_chat.rs when key_scope extracted inside ai-live block**
- **Found during:** Task 1 — `cargo build -p app --features ai-live`
- **Issue:** `principal` (a reference into `req`) was borrowed after `req.json().await` moved `req`. Initially `key_scope` was extracted inside the `#[cfg(feature = "ai-live")]` block (after `req.json()`), causing E0505.
- **Fix:** Extracted `key_scope` from `principal` before `req.json()`, gated the extraction itself under `#[cfg(feature = "ai-live")]` to prevent unused-variable warning in default builds.
- **Files modified:** `app/src/controllers/mcp_chat.rs`

## Known Stubs

None. All acceptance criteria are satisfied. The live-eval test is intentionally `#[ignore]`-gated — not a stub, by design.

## Threat Surface Scan

New HTTP endpoint `POST /mcp/chat` in `app/` (not in a `ferro-*` crate):

| Flag | File | Description |
|------|------|-------------|
| threat_flag: new-endpoint | app/src/controllers/mcp_chat.rs | POST /mcp/chat — new auth-protected endpoint |

Mitigations (per plan threat model):
- T-221-08 (spoofing): endpoint registered inside the bearer+tenant protected group; `tenant_id` from `ferro::current_tenant()`, never from body. Verified: no body-supplied tenant read in `mcp_chat.rs`.
- T-221-09 (DoS cost): live test is `#[cfg(feature = "ai-live")] #[ignore]` + env-gated; CI uses `ai,confirmation` only. Verified: `cargo test --features ai,confirmation` does not compile or run `intent_loop_live_eval`.
- T-221-10 (prompt injection): endpoint calls `process_nl_turn` which routes through `handle_write_call` (guard re-eval + tenant scoping + D-08 confirmation). No bypass in `mcp_chat.rs`. Verified by grep.
- T-221-11 (app identity leak): `mcp_chat.rs` is in `app/` (correct home); `McpServerConfig::from_env()` provides `app_url`. No hardcoded identity strings. Verified by grep.

## Self-Check

### Files Exist

- `/Users/alberto/repositories/albertogferrario/ferro/app/src/controllers/mcp_chat.rs`: FOUND
- `/Users/alberto/repositories/albertogferrario/ferro/app/src/controllers/mod.rs` (contains `pub mod mcp_chat;`): FOUND
- `/Users/alberto/repositories/albertogferrario/ferro/app/src/routes.rs` (contains `mcp/chat`): FOUND
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-mcp-server/tests/intent_loop.rs` (contains `intent_loop_live_eval`): FOUND

### Commits Exist

- ef510ed5: feat(221-03): POST /mcp/chat — thin NL turn endpoint wrapping process_nl_turn
- 99fc674a: feat(221-03): wire live-eval test with cost announcement (SC#4)
- 5da10983: style(221-03): apply cargo fmt and fix unused-variable warnings in mcp_chat.rs

## Self-Check: PASSED
