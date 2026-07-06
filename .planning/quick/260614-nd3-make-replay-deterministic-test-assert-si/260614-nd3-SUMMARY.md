---
quick_id: 260614-nd3
description: make replay_deterministic test assert single-execution via idempotency key and exec_count
status: complete
completed: 2026-06-14
commit: f14a4421
files_modified:
  - ferro-mcp-server/tests/intent_loop.rs
---

# Quick Task 260614-nd3: Strengthen `replay_deterministic` test — Summary

Converted the `replay_deterministic` test's constant executor into a shared
counting executor and injected an `idempotency_key` into the write fixture, so
the test now proves single-execution (idempotency replay) rather than the
trivial equality of two constant returns.

## What changed

`ferro-mcp-server/tests/intent_loop.rs` (one file, +52/-6):

1. Added `Clone` to the `IntentTurnFixture` derive (`#[derive(Debug, Clone, Deserialize, Serialize)]`) so each loop iteration can clone and mutate its fixture.
2. In `replay_deterministic`, renamed the loop variable to `src_fixture` and cloned it into a local `mut fixture` per iteration.
3. Injected `"idempotency_key": "replay-determinism-key"` into `fixture.recorded_selection["arguments"]` (via `as_object_mut`) **only for the write fixture** (`fixture.expected_tool != "list_order"`). The read fixture is left untouched to avoid passing an unknown filter to the list handler.
4. Created `let exec_count = Arc::new(AtomicUsize::new(0));` fresh per iteration, defined before `make_dispatcher` so the closure captures a clone that persists across both `make_dispatcher()` calls (run1 + run2).
5. Changed the `make_dispatcher` executor from the constant closure to a counting one: `count.fetch_add(1, Ordering::SeqCst)` before returning `Ok(json!({ "status": "approved" }))`.
6. Updated `single_fixture_provider(fixture)` → `single_fixture_provider(&fixture)` (now an owned local).
7. Added the single-execution assertion after the existing `assert_eq!(sc1, sc2, ...)`: expected `0` for `list_order` (read path never reaches the executor), `1` for the write path.

## Why the assertion is non-trivial

The write path executes exactly once across both runs because of idempotency
replay: run 1 executes the executor and stores the result under
`idempotency_key = "replay-determinism-key"` in `mcp_idempotency_keys`; run 2
hits the stored row in `write_dispatch.rs:302-334` and returns the stored
result **without invoking the executor**. Both runs share the same `db`
(created once per fixture iteration), so the stored key persists between them.

If the `idempotency_key` injection were removed, run 2 would miss the lookup and
re-execute, driving `exec_count` to `2` on the write path — failing
`assert_eq!(exec_count, 1)`. The assertion therefore directly verifies that
idempotency replay fires, which the prior constant-executor test did not.

## Verification

- `cargo test -p ferro-mcp-server --all-features replay_deterministic` — **pass** (1 passed; 0 failed).
- `cargo fmt --all` — no files changed outside the touched test.
- `cargo clippy -p ferro-mcp-server --all-targets --all-features -- -D warnings` — **clean** (finished, no warnings).

CPU operations were run serially, one at a time (project convention).

## Deviations from Plan

None — plan executed exactly as written.

## Notes

- `AtomicUsize`, `Ordering`, `Arc`, and `serde_json::json!` were already in scope (lines 320-321); no new imports needed.
- Only `ferro-mcp-server/tests/intent_loop.rs` was staged. The pre-existing phantom ` D planning/phases/158-.../158-REVIEW.md` deletion and untracked `.planning/` artifacts were left untouched. No `docs/protocol/schemas/*.json` churn occurred (the scoped test filter did not run the Phase 94 export test).

## Self-Check: PASSED

- FOUND: ferro-mcp-server/tests/intent_loop.rs
- FOUND commit: f14a4421
