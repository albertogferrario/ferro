---
phase: 221-inbound-nl-intent-loop
plan: "02"
subsystem: ferro-mcp-server
tags: [ai, classification, intent-loop, process_nl_turn, replay-test, sc1, sc2, sc3, sc5]
dependency_graph:
  requires: [221-01 (ToolSelection, render_tool_descriptions, ReplayClassificationProvider, fixtures), Phase 220 confirmation gate, ferro-ai classifier-trait feature]
  provides: [process_nl_turn turn-core, non-ignored end-to-end replay test covering SC#1/SC#2/SC#3/SC#5]
  affects: [ferro-mcp-server ai feature surface, intent_loop.rs test suite]
tech_stack:
  added: []
  patterns: [classify-then-route pipeline, LowConfidence-to-clarification mapping, #[cfg(feature)] parameter threading]
key_files:
  created: []
  modified:
    - ferro-mcp-server/src/intent.rs
    - ferro-mcp-server/src/lib.rs
    - ferro-mcp-server/tests/intent_loop.rs
decisions:
  - "process_nl_turn composes handle_tools_call (read) and handle_write_call (write) with zero new dispatch/guard/confirm logic — the classifier output enters the identical tools/call pipeline as any direct call (SC#1)"
  - "LowConfidence maps to isError:false needs_clarification envelope without invoking any dispatch path (SC#5)"
  - "Destructive write (submit, transition_trigger.is_some()) routes through handle_write_call, which fires the D-08 seam and returns confirmation_required without executing (SC#2)"
  - "The ambiguous fixture (confidence 0.3 < default threshold 0.7) drives the LowConfidence branch in the replay test — no fixture change needed"
metrics:
  duration: "~12 minutes"
  completed: "2026-06-14"
  tasks: 2
  files: 3
---

# Phase 221 Plan 02: NL Intent Loop — process_nl_turn + End-to-End Replay Test Summary

Turn-core implementation and deterministic proof: `process_nl_turn` classifies an NL message via the replay provider, routes `list_*` to the read path and all other tool names to `handle_write_call`, maps `Error::LowConfidence` to a `needs_clarification` envelope, and the non-ignored replay test exercises all four SC branches (SC#1/SC#2/SC#3/SC#5) without network or LLM.

## What Was Built

### Task 1: process_nl_turn (ferro-mcp-server/src/intent.rs + lib.rs)

`process_nl_turn` gated by `#[cfg(feature = "ai")]`, mirroring `handle_write_call`'s signature with the `#[cfg(feature = "confirmation")]` parameter idiom:

```rust
pub async fn process_nl_turn(
    nl_message: &str,
    services: &[ServiceDef],
    db: &DatabaseConnection,
    tenant_id: Option<i64>,
    ctx: &McpContext,
    provider: Arc<dyn ferro_ai::ClassificationProvider>,
    classifier_config: ferro_ai::ClassifierConfig,
    dispatcher: &WriteDispatcher,
    #[cfg(feature = "confirmation")] store: &dyn ferro_ai::ConfirmationStore,
    #[cfg(feature = "confirmation")] config: &McpServerConfig,
) -> serde_json::Value
```

Pipeline body:
1. `render_tool_descriptions(services, ctx)` → system prompt (on Err: write_tool_error_result envelope, no panic)
2. JSON schema for `ToolSelection` (snake_case, three required fields)
3. `Classifier::<ToolSelection>::new(provider, classifier_config).classify(...)`
4. Match on result:
   - `Err(LowConfidence { best_guess, confidence })` → `needs_clarification` envelope, `isError: false`, no dispatch (SC#5)
   - `Err(other)` → `write_tool_error_result` error envelope
   - `Ok(result)` → build `call_params = { name, arguments }`, then:
     - `tool_name.starts_with("list_")` → `handle_tools_call(...)` (SC#1 read)
     - else → `handle_write_call(...)` (SC#1 write, SC#2)

Re-exported from `lib.rs` behind `#[cfg(feature = "ai")]`:
```rust
pub use intent::{process_nl_turn, render_tool_descriptions, ToolSelection};
```

Acceptance criteria verified:
- `intent.rs` contains `pub async fn process_nl_turn` ✓
- `handle_write_call` called 4 times (import + doc + read + write branch) ✓
- No `dispatch_write(` / `guard_evaluator` / `evaluated_guards` in `intent.rs` ✓
- `LowConfidence` match arm produces `"needs_clarification"` with no dispatch ✓
- No `confidence >=` gating dispatch ✓
- `cargo build -p ferro-mcp-server --features ai,confirmation` exits 0 ✓

### Task 2: Non-ignored deterministic replay test (ferro-mcp-server/tests/intent_loop.rs)

Six new `#[tokio::test]` functions (all non-ignored, under `#[cfg(feature = "ai")]`) added to the existing `intent_loop` module:

- **`read_turn`** (SC#1 read): `"show me the orders"` → `list_order` → read path → `isError:false`, executor NOT called (count=0)
- **`write_turn`** (SC#1 write): `"approve the order from Alice"` → `approve` → write path → executor called once, `isError:false`
- **`destructive_requires_confirm`** (SC#2, `#[cfg(feature = "confirmation")]`): `"submit order 7"` → `submit` (has `transition_trigger`) → D-08 seam → `confirmation_required` envelope, executor NOT called (count=0)
- **`low_confidence`** (SC#5): `"do the thing"` → confidence 0.3 < threshold 0.7 → `LowConfidence` → `needs_clarification` envelope, `isError:false`, no dispatch
- **`turn_result_valid_mcp`**: every turn result has `content[]` and `isError` bool (Phase 205 regression guard)
- **`replay_deterministic`** (SC#3): read+write turns run twice, `structuredContent` byte-identical

Test harness setup:
- In-memory SQLite with `orders`, `mcp_idempotency_keys`, `audit_log` tables
- `test_service()` ServiceDef with `approve` (non-destructive) and `submit` (destructive, has `transition_trigger`)
- `ReplayClassificationProvider` from existing Plan 01 infrastructure
- `AtomicUsize` executor call counter for SC#2/SC#5 assertions

## Verification Results

- `cargo build -p ferro-mcp-server --features ai,confirmation`: exit 0
- `cargo test -p ferro-mcp-server --features ai,confirmation`: 9 tests, 8 passed, 1 ignored (live_eval), 0 failed
- `cargo test -p ferro-mcp-server --features ai,confirmation replay_deterministic`: 1 passed
- `cargo fmt --all -- --check`: exit 0 (after applying fmt)
- `cargo clippy --all --all-targets -- -D warnings`: exit 0
- `cargo test --all-features`: exit 0

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Format] Applied cargo fmt after initial implementation**
- **Found during:** Pre-commit format check
- **Issue:** `cargo fmt --all -- --check` found formatting differences in `intent.rs` (import ordering) and `intent_loop.rs` (line wrapping in ActionDef builder calls and method chains)
- **Fix:** Ran `cargo fmt --all` — no logic changes, pure formatting
- **Files modified:** `ferro-mcp-server/src/intent.rs`, `ferro-mcp-server/tests/intent_loop.rs`

### Known Deviation: grep criterion for live eval references

The plan's acceptance criterion states:
> `grep -c "AnthropicProvider\|FERRO_AI_LIVE_EVAL\|reqwest" ferro-mcp-server/tests/intent_loop.rs` is `0`

The actual count is `5` because the `#[ignore]`-gated `intent_loop_live_eval` test stub (carried over from Plan 01) references `FERRO_AI_LIVE_EVAL` in comments and an env-var guard, and `AnthropicProvider` in a comment. This stub is `todo!()`-bodied and produces no live network calls. The six **new** replay tests from this plan contain zero such references. The KEY invariant — no active test constructs `AnthropicProvider` or calls the Anthropic API — is satisfied.

## Known Stubs

None. The `intent_loop_live_eval` `#[ignore]`-gated stub from Plan 01 is intentional (wired in Plan 03). It is not a stub in the current plan's goal — it is an intentionally scaffolded future path.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. `process_nl_turn` is a library function, not an HTTP endpoint. The threat mitigations from the plan's `<threat_model>` are satisfied by construction:

| Threat | Mitigation | Verified |
|--------|-----------|---------|
| T-221-04 (prompt injection via tool_name) | Write branch calls `handle_write_call` only; `find_action` rejects non-exposed names with -32601 | `destructive_requires_confirm` + `write_turn` tests |
| T-221-05 (prompt injection via arguments) | Classified args pass `validate_action_inputs` → LIVE-DB guard re-eval → tenant-scoped idempotency → executor | grep confirms no `evaluated_guards`/`guard_evaluator` in `intent.rs` |
| T-221-06 (write escalation) | Destructive actions hit D-08 seam in `handle_write_call` → `ConfirmationRequired`; no `confidence >=` bypass | `destructive_requires_confirm` asserts executor-count=0 |
| T-221-07 (cross-tenant) | `tenant_id` flows from principal param; classified-args tenant_id ignored by `handle_write_call` | covered by existing write_dispatch tenant tests |

## Self-Check

### Files Exist

- `/Users/alberto/repositories/albertogferrario/ferro/ferro-mcp-server/src/intent.rs`: contains `pub async fn process_nl_turn` at line 92
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-mcp-server/tests/intent_loop.rs`: contains all 6 new replay tests

### Commits Exist

- e5de9f98: feat(221-02): implement process_nl_turn — classify + read/write routing + low-confidence clarification
- 184fa2b2: test(221-02): non-ignored deterministic replay test covering all SC branches

## Self-Check: PASSED
