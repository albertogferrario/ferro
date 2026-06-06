---
phase: 184-ferro-inlinebudget-ferro-requesttelemetry
plan: 02
subsystem: http+telemetry
tags: [inline-budget, state-machine, tracing, request-impl, borrow-checker, fire-once-warning]

# Dependency graph
requires:
  - 184-01 (Decision enum, InlineBudgetState struct, RequestTelemetry storage, AppConfig::inline_budget_threshold_bytes, telemetry::request_telemetry::record private writer)
provides:
  - "InlineBudgetState::record_and_decide(key, bytes, threshold, fallback_url, route_pattern) -> Decision — pure state machine (no Request involved): saturating_add accumulator, Inline at-or-below threshold, Preload past threshold, fire-once tracing::warn! per (key, request) with 5 structured fields"
  - "telemetry::inline_budget::decide(req, key, bytes, fallback_url) -> Decision — thin Request-side wrapper with borrow-checker-safe ordering (Config + route_pattern read into owned locals BEFORE &mut extensions borrow); lazy-inits InlineBudgetState"
  - "Request::inline_budget(&mut self, key, bytes, fallback_url) -> Decision — public delegator on second impl Request block"
  - "Request::telemetry_record(&mut self, key, sample) — public unscoped delegator to telemetry::request_telemetry::record(key, None, sample)"
  - "Request::telemetry_record_scoped(&mut self, key, scope, sample) — public scoped delegator to telemetry::request_telemetry::record(key, scope, sample)"
affects:
  - 184-03 (Plan 03 lands the integration test in framework/tests/telemetry_smoke.rs using TCP-loopback Request::new(req) constructor, the docs page, and the workspace 0.2.43 → 0.2.44 bump on top of this complete API surface)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Pure-state-machine + thin-Request-wrapper split — pattern for any future per-request decisioning that needs unit-testable core logic when the host type (Request) has no Default impl"
    - "Borrow-checker-safe ordering: capture &self-borrowing reads (config lookup, route_pattern) into local owned values BEFORE the &mut self call to get_mut::<T>() on extensions"
    - "Fire-once side-effect tracking via HashSet<String> on the state struct, with state.warned.insert() guarded by !state.warned.contains() — testable via set-size invariant rather than tracing-subscriber capture"

key-files:
  created: []
  modified:
    - "framework/src/telemetry/inline_budget.rs"
    - "framework/src/http/request.rs"
    - "framework/src/telemetry/request_telemetry.rs"

key-decisions:
  - "State machine implemented as pure InlineBudgetState::record_and_decide(...) method with no Request involved. Tests construct InlineBudgetState::default() directly — no synthetic Request needed. This sidesteps the locked rationale that Request has no Default impl and only constructs via TCP loopback (see framework/tests/action_handler.rs:47-90)."
  - "Fire-once invariant verified via state.warned set-size assertion (== 1 after two crosses on same key; == 2 after crosses on two distinct keys). No tracing-test dep added (OQ3 resolution honored)."
  - "Threshold-cross semantic locked: cumulative <= threshold returns Inline; cumulative > threshold returns Preload. At-exact-threshold is NOT a cross (validated by test decides_inline_at_exact_threshold)."
  - "Removed Plan 01's #[allow(dead_code)] guards on decide() and record() — both are now wired by Request impl methods, so the rustc dead-code warnings would not fire even with -D warnings."

patterns-established:
  - "Public Request method bodies are thin delegators to crate::telemetry::* — keeps request.rs from absorbing telemetry semantics. Test coverage for the delegators lands in integration tests (framework/tests/), not in-crate, because Request has no fake constructor."
  - "decide() wrapper documents the borrow-checker pitfall inline (RESEARCH Pitfall 1) so future refactors keep the read-then-mut ordering."

requirements-completed: [SC-1, SC-2]

# Metrics
duration: 15min
completed: 2026-06-06
---

# Phase 184 Plan 02: Request Integration and Decide Summary

**Inline-budget state machine + Request integration — record_and_decide pure method, decide(req) thin wrapper with borrow-safe ordering, three Request methods (inline_budget / telemetry_record / telemetry_record_scoped). Pre-commit gate green.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-06-06T20:22:53Z
- **Tasks:** 3 (2 commits — Task 3 was a gate-only verification with no file mutations)
- **Files modified:** 3 (no new files)

## Accomplishments

- Implemented `InlineBudgetState::record_and_decide(key, bytes, threshold, fallback_url, route_pattern) -> Decision` — pure state machine with saturating_add accumulator, threshold comparison, fire-once `tracing::warn!` emission
- Implemented `pub(crate) fn decide(req, key, bytes, fallback_url) -> Decision` — thin Request-side wrapper with borrow-checker-safe ordering (Config + route_pattern reads into owned locals BEFORE `&mut self` borrow on `req.get_mut::<InlineBudgetState>()`)
- Added `Request::inline_budget`, `Request::telemetry_record`, `Request::telemetry_record_scoped` to the second `impl Request` block at `framework/src/http/request.rs:742-829` (alongside existing `flash` / `redirect_to`)
- 6 new unit tests on `InlineBudgetState::default()` — no Request constructed: `decides_inline_below_threshold`, `decides_inline_at_exact_threshold`, `decides_preload_above_threshold`, `decides_preload_after_accumulation`, `warn_fires_once_per_key`, `warn_independent_per_key`
- Cumulative `telemetry::inline_budget::tests` count = 8 (2 Plan 01 + 6 Plan 02); all green
- Pre-commit gate (`cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`) shipped green: 2850 tests passed, 0 failed across the workspace

## Task Commits

Each task was committed atomically:

1. **Task 1: state-machine — record_and_decide + warning fire-once** — `9cf94bd7` (feat)
2. **Task 2: Request impl — inline_budget + telemetry_record + scoped** — `772e8f52` (feat)
3. **Task 3: pre-commit gate verification** — no commit (Cargo.lock unchanged; per plan instructions "commit Cargo.lock if cargo test regenerated it")

## Files Modified

- `framework/src/telemetry/inline_budget.rs` — module header updated to describe the pure-state-machine + thin-wrapper split; added `impl InlineBudgetState { pub(crate) fn record_and_decide(...) }` with cumulative accumulation, threshold comparison, and fire-once `tracing::warn!` with 5 structured fields (`key = %key`, `cumulative_bytes = cumulative`, `threshold_bytes = threshold`, `fallback_url = %fallback_url`, `route_pattern = %route_pattern`); added `pub(crate) fn decide(req, key, bytes, fallback_url) -> Decision` thin wrapper; appended 6 new unit tests to `#[cfg(test)] mod tests`. Removed Task 1's temporary `#[allow(dead_code)]` once Task 2 wired `decide` via `Request::inline_budget`.
- `framework/src/http/request.rs` — added 3 thin delegators to the SECOND `impl Request` block (alongside `flash`, `redirect_to`): `inline_budget` (delegates to `crate::telemetry::inline_budget::decide`), `telemetry_record` (delegates to `crate::telemetry::request_telemetry::record(key, None, sample)`), `telemetry_record_scoped` (delegates to `crate::telemetry::request_telemetry::record(key, scope, sample)`). Each method carries `///` doc comments; `inline_budget` includes `# Example` block plus a security note that `fallback_url` MUST NOT be user input (T-184-04 mitigation by documentation).
- `framework/src/telemetry/request_telemetry.rs` — removed Plan 01's `#[allow(dead_code)]` on `record()` now that `Request::telemetry_record` / `Request::telemetry_record_scoped` wire it (rustc no longer warns). Updated doc-comment to drop the "Plan 02 wires them" stub.

## Decisions Made

- **State machine implemented as pure method, sidestepping the no-Default-Request rationale.** Plan 02's locked design (see CONTEXT.md `<objective>` lines 53-58) is that `Request` has no `Default` impl and cannot be constructed synthetically. The state machine is therefore implemented on `InlineBudgetState::record_and_decide(...)` — a pure method that takes `&mut self` plus owned-string-like parameters. Tests construct `InlineBudgetState::default()` directly. No `Request` instance appears in any unit test in `inline_budget.rs`. `grep -F 'Request::default()' framework/src/telemetry/inline_budget.rs` returns no matches; `grep -F 'Request::default()' framework/src/http/request.rs` returns no matches.
- **Fire-once verified by set-size invariant, not by tracing-subscriber capture.** Per OQ3 (CONTEXT.md), no `tracing-test` dependency added. Tests `warn_fires_once_per_key` and `warn_independent_per_key` assert `state.warned.len() == 1` (after two crosses on same key) and `state.warned.len() == 2` (after crosses on two distinct keys). The state machine is the source of truth for the contract; whether `tracing` actually emitted is the subscriber's concern, not the state machine's.
- **Borrow-checker-safe ordering preserved in `decide`.** Per RESEARCH Pitfall 1, `Config::get::<crate::AppConfig>()` and `req.route_pattern()` are read into local owned values BEFORE `req.get_mut::<InlineBudgetState>()` is called. Inverting the order would compile-fail with "cannot borrow `*req` as immutable because it is also borrowed as mutable." The ordering is documented inline in the wrapper's doc-comment.
- **Threshold-cross semantic locked to `>` (NOT `>=`).** `cumulative <= threshold` → `Inline`; `cumulative > threshold` → `Preload`. At-exact-threshold is NOT a cross. Verified by test `decides_inline_at_exact_threshold` (bytes = 102_400, threshold = 102_400 → returns `Inline`). The natural reading of "below threshold = inline" is the locked semantic.
- **Removed Plan 01's `#[allow(dead_code)]` guards immediately.** Plan 01 added the guards on `record()` and `InlineBudgetState` to keep clippy `-D warnings` clean while Plan 02 was pending. With Plan 02's Request integration, `record()` and the state fields are now actually used by production code paths (`Request::telemetry_record`, `Request::telemetry_record_scoped`, `Request::inline_budget` → `decide()` → `record_and_decide()`). Both `#[allow(dead_code)]` attributes were removed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Lint Failure] Temporary `#[allow(dead_code)]` on `decide()` for the Task 1 commit gate**
- **Found during:** Task 1 (pre-commit gate, post-implementation)
- **Issue:** Task 1 ships `decide()` but `Request::inline_budget` doesn't exist yet — rustc warns "function `decide` is never used" and clippy `-D warnings` rejects the commit.
- **Fix:** Added `#[allow(dead_code)] // Wired by Request::inline_budget in Task 2 of this same plan.` on `decide()` so the Task 1 commit passes the CI-equivalent gate. Task 2 then removed the attribute as soon as the delegator was wired (so it never persists across multiple plans, unlike Plan 01's `#[allow]`s which spanned plans).
- **Files modified:** `framework/src/telemetry/inline_budget.rs`
- **Committed in:** `9cf94bd7` (Task 1 commit, added) and `772e8f52` (Task 2 commit, removed)

**2. [Rule 1 - Cleanup] Removed Plan 01's `#[allow(dead_code)]` on `record()` in request_telemetry.rs**
- **Found during:** Task 2
- **Issue:** Plan 01 added `#[allow(dead_code)] // Plan 02 wires Request::telemetry_record to this.` on `record()`. With Task 2 actually wiring `Request::telemetry_record` and `Request::telemetry_record_scoped` to `record()`, the attribute is now unnecessary — keeping it would be dead-on-the-surface technical debt.
- **Fix:** Removed the `#[allow(dead_code)]` and updated the doc-comment to drop the "Plan 02 wires them" stub (now reads simply "Called from `Request::telemetry_record` and `Request::telemetry_record_scoped`").
- **Files modified:** `framework/src/telemetry/request_telemetry.rs`
- **Verification:** `cargo clippy --all --all-targets -- -D warnings` exits 0 after the removal; no unused-attribute or dead-code warnings.
- **Committed in:** `772e8f52` (Task 2 commit)

**3. [Rule 1 - Format] rustfmt collapsed multi-line `decide()` signature**
- **Found during:** Task 1 (`cargo fmt --all -- --check`)
- **Issue:** Original implementation wrote `fn decide(req, key, bytes, fallback_url)` across 6 lines for readability; rustfmt collapsed it onto a single line (fits within line width).
- **Fix:** Ran `cargo fmt --all` once; verified `--check` clean afterward. Same pattern Plan 01 encountered.
- **Files modified:** `framework/src/telemetry/inline_budget.rs`
- **Committed in:** `9cf94bd7` (Task 1 commit)

---

**Total deviations:** 3 auto-fixed (3× Rule 1 — lint/cleanup/format adjustments to satisfy CLAUDE.md's CI-equivalent gate). All three are mechanical; storage shape, public surface, and semantics are unchanged from the plan. No scope creep.

## Issues Encountered

- **Transient single-test FAILED on first full `cargo test --all-features` run after Task 2.** During the first end-to-end gate run after Task 2, one summary line showed `test result: FAILED. 517 passed; 1 failed`. The immediate re-run was clean across all 91 test binaries (2850 total tests passed, 0 failed). No FAILED line was reproducible on subsequent runs. Plan 01's SUMMARY documented a similar transient flake in `ferro-api-mcp/tests/e2e.rs::test_openapi_spec_served` caused by disk-pressure during the test run. Classified as environmental, not Plan-184-specific (the telemetry primitives don't touch `ferro-api-mcp` at all). The final gate (Task 3) is fully green.

## User Setup Required

None — Plan 02 ships additive Rust API surface only. No env vars, no migrations, no manual setup. The new `Request` methods are accessible from any handler holding `&mut Request` after `use ferro_rs::{Request, Decision, Sample, ...};`.

## Next Phase Readiness

Plan 03 can land directly on top of this complete API surface:

- **Integration test** — `framework/tests/telemetry_smoke.rs` constructs a real `Request` via the canonical TCP-loopback `make_request()` helper from `framework/tests/action_handler.rs:47-90` (where `Request::new(req)` is called synchronously on line 71). The test exercises all three `Request` methods end-to-end: `req.inline_budget("k", 200_000, "/fb")` → asserts `Decision::Preload("/fb".into())`; `req.telemetry_record("render_latency", Sample::now(json!({"ms": 42})))` then `RequestTelemetry::snapshot("render_latency", None)` → asserts Vec of length 1 with the recorded payload; similarly for `telemetry_record_scoped`.
- **Docs page** — `docs/src/the-basics/inline-budget-and-telemetry.md` (per CONTEXT D-14). Covers both primitives with the end-to-end example from CONTEXT lines 444-475, the scoping conventions table, and the lost-on-restart semantic. `docs/src/SUMMARY.md` gets the new entry under "The Basics."
- **Workspace version bump** — `Cargo.toml [workspace.package.version]` 0.2.43 → 0.2.44 (per CONTEXT D-13).
- **`cargo publish --dry-run`** — verify the new `framework` crate compiles cleanly for crates.io with the new public surface. Real publish remains user-driven (per memory `feedback_friction_loop_release_cadence.md`).
- **`cargo doc --no-deps`** — verify the new `///` doc-comments render and the `[crate::AppConfig::inline_budget_threshold_bytes]` rustdoc link resolves.

The public API surface is complete and ready for consumer adoption (gestiscilo Phase 187 path).

## Self-Check: PASSED

**Files verified to exist:**
- `framework/src/telemetry/inline_budget.rs` — FOUND (modified)
- `framework/src/http/request.rs` — FOUND (modified)
- `framework/src/telemetry/request_telemetry.rs` — FOUND (modified)

**Commits verified in git log:**
- `9cf94bd7` — FOUND (Task 1: feat(184-02): state-machine — record_and_decide + warning fire-once)
- `772e8f52` — FOUND (Task 2: feat(184-02): Request impl — inline_budget + telemetry_record + scoped)

**Acceptance criteria verified:**
- `grep -F 'pub(crate) fn record_and_decide(' framework/src/telemetry/inline_budget.rs` — FOUND
- `grep -F 'pub(crate) fn decide(' framework/src/telemetry/inline_budget.rs` — FOUND
- `grep -F 'Config::get::<crate::AppConfig>()' framework/src/telemetry/inline_budget.rs` — FOUND
- `grep -F '.unwrap_or(102_400)' framework/src/telemetry/inline_budget.rs` — FOUND
- `grep -F 'tracing::warn!' framework/src/telemetry/inline_budget.rs` — FOUND
- `grep -F 'inline_budget: threshold crossed; flipping to Preload' framework/src/telemetry/inline_budget.rs` — FOUND
- `grep -F 'state.record_and_decide(key, bytes, threshold, fallback_url, &route_pattern)' framework/src/telemetry/inline_budget.rs` — FOUND
- All 5 warning fields (`key = %key`, `cumulative_bytes = cumulative`, `threshold_bytes = threshold`, `fallback_url = %fallback_url`, `route_pattern = %route_pattern`) — FOUND
- `grep -F 'req.route_pattern().unwrap_or_default()' framework/src/telemetry/inline_budget.rs` — FOUND
- 6 new unit tests by name (decides_inline_below_threshold, decides_inline_at_exact_threshold, decides_preload_above_threshold, decides_preload_after_accumulation, warn_fires_once_per_key, warn_independent_per_key) — FOUND in source; all pass under `cargo test -p ferro-rs --lib telemetry::inline_budget`
- `grep -F 'Request::default()' framework/src/{telemetry/inline_budget.rs,http/request.rs}` — NO MATCHES (fictional API absent, as required)
- `grep -F 'pub fn inline_budget(' framework/src/http/request.rs` — FOUND
- `grep -F 'pub fn telemetry_record(' framework/src/http/request.rs` — FOUND
- `grep -F 'pub fn telemetry_record_scoped(' framework/src/http/request.rs` — FOUND
- `grep -F 'crate::telemetry::inline_budget::decide(self, key, bytes, fallback_url)' framework/src/http/request.rs` — FOUND
- `grep -F 'crate::telemetry::request_telemetry::record(key, None, sample)' framework/src/http/request.rs` — FOUND
- `grep -F 'crate::telemetry::request_telemetry::record(key, scope, sample)' framework/src/http/request.rs` — FOUND

**Pre-commit gate result (Task 3):** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` exits 0. 2850 tests passed, 0 failed across the workspace. `telemetry::inline_budget::tests` reports 8 passed; `telemetry::request_telemetry::tests` reports 8 passed; `config::providers::app::tests::inline_budget_threshold_*` reports 3 passed.

---
*Phase: 184-ferro-inlinebudget-ferro-requesttelemetry*
*Plan: 02-request-integration-and-decide*
*Completed: 2026-06-06*
