---
phase: 248-deployable-ferro-worker-runtime
fixed_at: 2026-08-14T20:46:29Z
review_path: .planning/phases/248-deployable-ferro-worker-runtime/248-REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 248: Code Review Fix Report

**Fixed at:** 2026-08-14T20:46:29Z
**Source review:** .planning/phases/248-deployable-ferro-worker-runtime/248-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 2 (WR-01, WR-02 — Critical + Warning; the review reported 0 Critical)
- Fixed: 2
- Skipped: 0

The three Info findings (IN-01, IN-02, IN-03) are out of scope for the
`critical_warning` fix pass. Note that the WR-01 fix incidentally narrows the
IN-03 drift it flagged: both the consumer `main.rs::get_database_connection` and
the framework `app.rs::get_database_connection` now surface the same
`Error:` / `Cause:` / `How to fix:` shape on DB failure, though they remain two
implementations (full consolidation was left to the optional IN-03 follow-up).

## Fixed Issues

### WR-01: `run_worker` boot-time DB-connect failure aborts before the actionable-error path

**Files modified:** `framework/src/app.rs`
**Commit:** 7a2ed4df
**Classification:** operational-quality regression (not a logic bug)

**Applied fix:** Replaced the two bare `.expect(...)` calls in
`Application::get_database_connection` (the `DATABASE_URL` env lookup and the
`sea_orm::Database::connect`) with `match` arms that print an operator-facing
`Error:` / `Cause:` / `How to fix:` block to stderr and `std::process::exit(1)`,
mirroring the consumer `app/src/main.rs::fail_with` remediation. Because both the
`serve` in-process path and the `worker` path route through
`run_common_boot → get_database_connection`, a misconfigured `DATABASE_URL` in a
worker deployment now produces the same guided diagnostic the rest of the CLI
provides instead of a panic plus backtrace.

The fix was applied inside the existing single `get_database_connection` helper,
so the shared `run_common_boot` seam is untouched and no boot logic is duplicated
(no-duplicate-control-surface respected). The intentional `None`-broadcaster
fallback (Phase 249.1) was not touched.

### WR-02: a queue typo silently idles the worker

**Files modified:** `framework/src/app.rs`
**Commit:** 38381b0b
**Classification:** silent operational failure (missing diagnostic)

**Applied fix:** In `Application::run_worker`, immediately after `effective_queues`
is computed, the requested queue names are compared against
`ferro_queue::Queue::registered_queue_names()`. Each requested queue absent from
the registered set emits a `tracing::warn!(queue = %q, …)` naming the queue and
stating it will idle. This is a warning, not a hard error: a queue may legitimately
carry zero local handlers when its jobs are enqueued by another service. The check
sits in `run_worker` where the framework already holds the registered set, so no
new control surface was introduced.

_Note: when `--queue` is omitted, `effective_queues` is exactly
`registered_queue_names()`, so the loop never warns on the all-queues default —
the warning fires only for explicitly-requested, unregistered queue names._

## Verification

Both fixes were verified together (they share one file) with the targeted,
thermally-scoped command set from the fix constraints — the full
`cargo test --all-features` was deliberately not run:

- `cargo build -p ferro-rs` — pass (framework package name is `ferro-rs`)
- `cargo build -p ferro-rs --features redis-transport` — pass
- `cargo build -p app` — pass
- `cargo clippy -p ferro-rs -p app --all-targets -- -D warnings` — pass, clean
- `cargo test -p ferro-rs --test worker_boot` — 1 passed
- `cargo test -p ferro-queue --test worker_runtime` — 1 passed
- `cargo fmt --all -- --check` — clean (re-checked after re-applying WR-02)
- `./target/debug/app worker --help` and `serve --help` — both parse

---

_Fixed: 2026-08-14T20:46:29Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
