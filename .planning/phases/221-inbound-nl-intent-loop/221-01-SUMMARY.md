---
phase: 221-inbound-nl-intent-loop
plan: "01"
subsystem: ferro-mcp-server
tags: [ai, classification, intent-loop, replay, fixtures, feature-gating]
dependency_graph:
  requires: [Phase 220 confirmation gate, ferro-ai classifier module]
  provides: [ToolSelection type, render_tool_descriptions, ReplayClassificationProvider, turn fixtures]
  affects: [ferro-mcp-server ai feature graph, ferro-ai classifier-trait feature]
tech_stack:
  added: [ferro-ai/classifier-trait feature, async-trait dev-dep in ferro-mcp-server]
  patterns: [reqwest-free feature split, replay classification provider, include_str! fixture loading]
key_files:
  created:
    - ferro-mcp-server/src/intent.rs
    - ferro-mcp-server/tests/intent_loop.rs
    - ferro-mcp-server/tests/fixtures/intent_loop/transcripts/list-orders.json
    - ferro-mcp-server/tests/fixtures/intent_loop/transcripts/approve-order.json
    - ferro-mcp-server/tests/fixtures/intent_loop/transcripts/cancel-order.json
    - ferro-mcp-server/tests/fixtures/intent_loop/transcripts/ambiguous.json
  modified:
    - ferro-ai/Cargo.toml
    - ferro-ai/src/lib.rs
    - ferro-ai/src/classifier/mod.rs
    - ferro-mcp-server/Cargo.toml
    - ferro-mcp-server/src/lib.rs
    - app/src/tests/mcp_tenant_isolation.rs
    - app/src/tests/mcp_write_dispatch.rs
decisions:
  - "Added classifier-trait feature to ferro-ai to expose ClassificationProvider + Classifier without reqwest (reqwest only in llm feature)"
  - "Fixture tool names adjusted to match app/src/projections/order.rs actual ActionDef names: list_order, approve, submit (not list_orders, cancel)"
  - "cancel-order.json uses submit action (has transition_trigger) as the destructive turn — no cancel ActionDef exists in order projection"
metrics:
  duration: "~17 minutes"
  completed: "2026-06-14"
  tasks: 3
  files: 13
---

# Phase 221 Plan 01: NL Intent Loop — Wave-0 Spine Summary

Wave-0 deterministic spine: `ai`/`ai-live` feature wiring in ferro-ai + ferro-mcp-server, `ToolSelection` type, `render_tool_descriptions` helper, `ReplayClassificationProvider`, and four committed turn fixtures.

## What Was Built

### Task 1: Feature Wiring (ai / ai-live)

Added a new `classifier-trait` feature to `ferro-ai` that exposes `ClassificationProvider`, `Classifier`, and `ClassifierConfig` without the `llm` feature (no reqwest). This required:
- Gating `pub mod anthropic;` in `ferro-ai/src/classifier/mod.rs` behind `#[cfg(feature = "llm")]`
- Exposing the classifier module under `any(feature = "llm", feature = "classifier-trait")` in `ferro-ai/src/lib.rs`
- Adding `classifier-trait = []` to `ferro-ai/Cargo.toml`

In `ferro-mcp-server/Cargo.toml`:
- `ai = ["dep:ferro-ai"]` — replay/CI path (ferro-ai pulled with `confirmation + classifier-trait`, no reqwest from ferro-ai)
- `ai-live = ["ai", "ferro-ai/llm"]` — live path (adds reqwest via ferro-ai/llm)
- Updated ferro-ai dep to `features = ["confirmation", "classifier-trait"]`

`ferro-mcp-server/src/lib.rs` gains `#[cfg(feature = "ai")] pub mod intent;` and re-exports.

### Task 2: ToolSelection + render_tool_descriptions

`ferro-mcp-server/src/intent.rs` (new file):
- `ToolSelection { tool_name: String, arguments: Map<String, Value>, confidence: f64 }` with `#[serde(rename_all = "snake_case")]`
- `render_tool_descriptions(services, ctx)` — text formatter over `render_exposed_tools` (NOT a second projection renderer, per Pitfall 4/PITFALLS §11)
- Four unit tests covering snake_case round-trip, camelCase rejection, tool name presence, args listing

### Task 3: ReplayClassificationProvider + Fixtures

Four turn fixtures in `ferro-mcp-server/tests/fixtures/intent_loop/transcripts/`:
- `list-orders.json` — read turn (confidence 0.95)
- `approve-order.json` — non-destructive write (confidence 0.92)
- `cancel-order.json` — destructive write via `submit` action (has `transition_trigger`, confidence 0.9)
- `ambiguous.json` — low-confidence turn (0.3, below default threshold 0.7)

`ferro-mcp-server/tests/intent_loop.rs`:
- `IntentTurnFixture` struct (turn_id, nl_message, expected_tool, recorded_selection)
- `ReplayClassificationProvider`: `HashMap<nl_message, Value>` implementing `ClassificationProvider` via async_trait, zero reqwest
- `fixtures_parse_and_replay_returns_recorded`: loads all 4 fixtures, asserts replay returns correct tool_name
- `replay_provider_returns_error_on_miss`: verifies `Error::Provider` on unknown message
- `intent_loop_live_eval`: `#[ignore]` + `FERRO_AI_LIVE_EVAL=1` gate stub (wired in Plan 03)

## Verification Results

- `cargo build -p ferro-mcp-server` (no features): exit 0
- `cargo build -p ferro-mcp-server --features ai`: exit 0
- `cargo build -p ferro-mcp-server --features ai-live`: exit 0
- `cargo test -p ferro-mcp-server --features ai`: 54 tests passed, 0 failed
- `cargo test --all-features`: all tests passed, 0 failed
- `cargo fmt --all -- --check`: exit 0
- `cargo clippy --all --all-targets -- -D warnings`: exit 0

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical Feature] Added classifier-trait feature to ferro-ai**
- **Found during:** Task 1 implementation
- **Issue:** `ClassificationProvider`, `Classifier`, and `ClassifierConfig` in ferro-ai were gated behind `#[cfg(feature = "llm")]` which pulls reqwest. The plan's `ai` feature wiring required these types without reqwest, but no mechanism existed to expose them that way.
- **Fix:** Added `classifier-trait = []` feature to ferro-ai; exposed classifier module and its reqwest-free types under `any(feature = "llm", feature = "classifier-trait")`; gated `pub mod anthropic` (the reqwest-using module) behind `#[cfg(feature = "llm")]` only. Updated ferro-mcp-server dep to include `classifier-trait` in base features.
- **Files modified:** `ferro-ai/Cargo.toml`, `ferro-ai/src/lib.rs`, `ferro-ai/src/classifier/mod.rs`, `ferro-mcp-server/Cargo.toml`
- **Impact:** The plan's `ai` feature now works exactly as specified — ferro-ai doesn't add reqwest under `ai` (only the pre-existing transitive dep from ferro-mcp-oauth/ferro-whatsapp appears, which was there before Phase 221)

**2. [Rule 1 - Bug] Fixed pre-existing clippy -D warnings failure in app tests**
- **Found during:** Final gate run (cargo clippy --all --all-targets -- -D warnings)
- **Issue:** `app/src/tests/mcp_tenant_isolation.rs` and `mcp_write_dispatch.rs` imported `McpServerConfig` unconditionally, but it's only used inside `#[cfg(feature = "confirmation")]` blocks — causing unused-import warnings promoted to errors.
- **Fix:** Moved `use ferro_mcp_server::McpServerConfig;` behind `#[cfg(feature = "confirmation")]` in both files.
- **Root cause:** Pre-existing from Phase 220 (last modified by `feat(220-02)`).
- **Files modified:** `app/src/tests/mcp_tenant_isolation.rs`, `app/src/tests/mcp_write_dispatch.rs`
- **Commit:** 0bb012fc

**3. [Rule 3 - Deviation] Fixture tool names adjusted to match actual projection**
- **Found during:** Task 3 — plan said `list_orders`, `cancel`; actual ActionDefs in `app/src/projections/order.rs` are `list_order` (singular), `approve`, `submit`, `ship`; no `cancel` ActionDef exists
- **Fix:** Used `list_order` (not `list_orders`), `submit` as the destructive action (has `transition_trigger`), kept `approve` correct. The plan's note said "adjust to real action names if they differ" — applied exactly.
- **Files modified:** all 4 fixture JSON files

### reqwest Acceptance Criterion Note

The plan's criterion `cargo tree --features ai | grep -q reqwest returns NON-zero (reqwest NOT pulled by ai feature)` cannot be satisfied literally because `ferro-mcp-server` already has reqwest transitively via `ferro-mcp-oauth → ferro-notifications → ferro-whatsapp` — present with NO features enabled. The KEY invariant IS satisfied: **ferro-ai does not add reqwest under the `ai` feature** (`cargo tree -p ferro-ai --no-default-features --features classifier-trait | grep reqwest` returns nothing). The reqwest count is identical (1) under both `ai` and `ai-live`, confirming ferro-ai/llm adds no new reqwest instance in this codebase.

## Known Stubs

`ferro-mcp-server/tests/intent_loop.rs:195` — `todo!("live eval wired in Plan 03 when process_nl_turn is available")` inside the `#[ignore]`-gated `intent_loop_live_eval` test. This is intentional: the live path requires `process_nl_turn` which is implemented in Plan 02, and `AnthropicProvider` wiring in Plan 03.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes. All additions are:
- A source module in ferro-mcp-server (feature-gated, reqwest-free under `ai`)
- Static fixture files loaded via `include_str!` at compile time
- Test infrastructure (no production code path changes)

T-221-01 (no API keys in fixtures): verified — `grep -rl "api[_-]key\|sk-ant\|ANTHROPIC" tests/fixtures/intent_loop/` returns nothing.
T-221-02 (reqwest gating): ferro-ai does not add reqwest under `ai`; see deviation note above.
T-221-03 (ReplayClassificationProvider): reqwest-free test infrastructure, no untrusted input at build time.

## Self-Check

### Files Exist
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-mcp-server/src/intent.rs`: FOUND
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-mcp-server/tests/intent_loop.rs`: FOUND
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-mcp-server/tests/fixtures/intent_loop/transcripts/list-orders.json`: FOUND
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-mcp-server/tests/fixtures/intent_loop/transcripts/approve-order.json`: FOUND
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-mcp-server/tests/fixtures/intent_loop/transcripts/cancel-order.json`: FOUND
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-mcp-server/tests/fixtures/intent_loop/transcripts/ambiguous.json`: FOUND

### Commits Exist
- 77fd4abd: feat(221-01): add ai / ai-live features and declare intent module
- 43baf6e8: feat(221-01): ToolSelection type and render_tool_descriptions helper
- 7d0c0025: feat(221-01): ReplayClassificationProvider + committed turn fixtures
- 3cfe025b: style(221-01): apply cargo fmt formatting
- 0bb012fc: fix(221-01): gate McpServerConfig import behind confirmation feature

## Self-Check: PASSED
