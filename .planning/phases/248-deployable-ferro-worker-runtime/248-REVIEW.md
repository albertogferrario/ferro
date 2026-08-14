---
phase: 248-deployable-ferro-worker-runtime
reviewed: 2026-08-14T00:00:00Z
depth: standard
files_reviewed: 14
files_reviewed_list:
  - app/src/bootstrap.rs
  - app/src/main.rs
  - ferro-macros/src/offload.rs
  - ferro-macros/src/service.rs
  - ferro-macros/tests/ui/offload/fail/queue_unknown_arg.rs
  - ferro-macros/tests/ui/offload/pass/queue_arg.rs
  - ferro-queue/src/db.rs
  - ferro-queue/tests/offload_round_trip.rs
  - ferro-queue/tests/worker_runtime.rs
  - framework/src/app.rs
  - framework/src/lib.rs
  - framework/tests/offload_delta_broadcast.rs
  - framework/tests/offload_result_round_trip.rs
  - framework/tests/worker_boot.rs
findings:
  critical: 0
  warning: 2
  info: 3
  total: 5
status: issues_found
---

# Phase 248: Code Review Report

**Reviewed:** 2026-08-14
**Depth:** standard
**Files Reviewed:** 14
**Status:** issues_found

## Summary

Phase 248 introduces a deployable worker runtime: a `worker` CLI subcommand,
`serve --no-worker`, a shared `run_common_boot` / `run_worker` boot seam,
`#[offload(queue = "name")]` routing through the macro layer, and
`Queue::registered_queue_names()`. The implementation is coherent and the
integration test coverage is unusually thorough (queue-scoped claim isolation,
cross-replica broadcast, boot-step feature-gating).

The four domain-specific correctness properties requested all hold:

- **Macro emission is `::ferro::*`-only.** `offload.rs::emit_job_items` and
  `service.rs::service_impl` route every generated path — including `serde_json`,
  `async_trait`, `inventory`, and the `queue::*` types — through the `::ferro::`
  facade. No `::ferro_queue::` path is emitted.
- **Unknown `#[offload]` args are rejected** with a clear diagnostic
  (`service.rs:205-206`) and the matching trybuild `.stderr` snapshot is present
  (`queue_unknown_arg.stderr`), so the negative gate is enforced by CI.
- **The Redis-URL paths do not log the URL.** Both the connect-failure warn
  (`app.rs:460-463`) and the feature-off warn (`app.rs:491-495`) use a static
  message plus `error = %e`; the URL is never interpolated into the log.
- **Queue names reach a parameterized SQL filter.** `WorkerLoop::run` iterates
  `config.queues` and calls `claim(conn, queue, …)`, which binds the queue name
  via `Statement::from_sql_and_values` (`db.rs:387-394`, `439-451`). No
  string-interpolated queue name reaches SQL — no injection surface.

The `None`-broadcaster fallback in `app.rs:497-501` is intentionally retained
(Phase 249.1 removes it) and is not flagged.

No critical issues. Two warnings and three info items follow; none block the
phase, but WR-01 and WR-02 are worth resolving before the convergence sweep.

## Warnings

### WR-01: `run_worker` boot-time DB-connect failure aborts before the actionable-error path

**File:** `framework/src/app.rs:548-561` (via `run_common_boot` → `get_database_connection`, `app.rs:583-609`)
**Issue:** The `worker` subcommand routes through `Application::run_worker` →
`run_common_boot` → `Self::get_database_connection()`, which uses bare
`.expect("DATABASE_URL must be set")` and `.expect("Failed to connect to
database")` (`app.rs:584`, `608`). The consumer `app/src/main.rs` invests in a
rich `fail_with` helper with remediation steps for the `serve` path
(`main.rs:205-252`), but the `worker` path inside the framework bypasses it and
panics with a bare message plus a backtrace. A misconfigured `DATABASE_URL` in a
worker deployment (a common first-run failure mode for a *deployable* worker)
produces a panic rather than the guided diagnostic the rest of the CLI provides.
This is an operational-quality regression relative to the sibling `serve` path,
not a correctness bug.
**Fix:** Replace the `.expect(...)` calls in `Application::get_database_connection`
with a stderr message + `std::process::exit(1)` mirroring `main.rs::fail_with`, so
both `serve` (framework-internal path) and `worker` surface the same actionable
guidance. For example:

```rust
let database_url = match env::var("DATABASE_URL") {
    Ok(u) => u,
    Err(_) => {
        eprintln!("Error: DATABASE_URL not set");
        eprintln!("  Add DATABASE_URL to .env (e.g. sqlite://./database.db)");
        std::process::exit(1);
    }
};
// ... and likewise for Database::connect(...).
```

### WR-02: `worker` reaps orphan claims only for its own queue set, but consumes handlers for all — a queue typo silently idles

**File:** `framework/src/app.rs:548-556`; `ferro-queue/src/db.rs:84-99`
**Issue:** `run_worker` accepts an arbitrary `Vec<String>` of queue names from the
`--queue` flag and passes them verbatim into `WorkerConfig::new`
(`app.rs:552-555`) with no validation against `Queue::registered_queue_names()`.
A misspelled `--queue reprots` produces a worker that boots cleanly, logs
`queues = ["reprots"]`, reaps nothing, and claims nothing — an
indistinguishable-from-healthy idle loop. Because the queue name is only ever
compared inside a parameterized SQL `WHERE queue = $1`, the typo is not a safety
issue, but it is a silent operational failure with no diagnostic. The framework
already knows the set of declared queues, so the mismatch is detectable at boot.
**Fix:** After computing `effective_queues`, warn (do not hard-fail — a queue may
legitimately have zero registered handlers if jobs are enqueued by another
service) when a requested queue is absent from `Queue::registered_queue_names()`:

```rust
let known = ferro_queue::Queue::registered_queue_names();
for q in &effective_queues {
    if !known.contains(q) {
        tracing::warn!(queue = %q, "worker started for a queue with no registered job handlers — it will idle");
    }
}
```

## Info

### IN-01: `registered_queue_names` locks `JOB_REGISTRARS` twice per call

**File:** `ferro-queue/src/db.rs:88` (and the shared mutex used again at `db.rs:105` in `apply_registrars`)
**Issue:** `registered_queue_names` takes `JOB_REGISTRARS.lock()` for the
emptiness check at line 88, releasing it immediately. In the surrounding boot
path `from_registry` (`worker.rs:218`) re-locks the same mutex through
`apply_registrars`. This is correct — the lock is not held across an `await` and
there is no re-entrancy — but the single-line lock-check-then-drop pattern reads
as if it might race with a concurrent `Queue::register`. It does not affect
correctness (registration happens at boot, before any worker consumes the set),
so this is a readability note only.
**Fix:** Optional. A one-line comment noting that all `Queue::register` calls
complete during single-threaded bootstrap, before `registered_queue_names` is
consulted, would prevent a future reader from mistaking this for a TOCTOU.

### IN-02: `serve --no-migrate --no-worker` collapses two independent flags into one bound variable

**File:** `app/src/main.rs:145-151`
**Issue:** The third `Serve` match arm binds `no_worker` while matching
`no_migrate: true`. This is correct and exhaustive, but the arm ordering means
the `{no_migrate: true, no_worker: true}` and `{no_migrate: true, no_worker:
false}` cases are handled implicitly by the single binding. A reader scanning the
match must reason about the earlier two arms to confirm that
`{no_migrate: true, no_worker: false}` was not already consumed (it was not —
arm 1 requires `no_worker: false` *and* `no_migrate: false`). The logic is right;
the density is the only cost.
**Fix:** Optional. No change required; if clarity is desired, a short comment on
the third arm ("`no_migrate: true` — `no_worker` handled by the bound variable
for both values") documents the intent.

### IN-03: Two near-identical `get_database_connection` implementations drift risk

**File:** `app/src/main.rs:205-252` and `framework/src/app.rs:583-609`
**Issue:** The consumer `main.rs` and the framework `app.rs` each carry a
`get_database_connection` with the same SQLite-path-creation logic
(`trim_start_matches("sqlite://")`, `create_dir_all`, `File::create`,
`?mode=rwc`). They differ only in error handling (the consumer uses `fail_with`,
the framework uses `.expect`). Duplicated boot logic across the framework/consumer
boundary invites divergence — a future fix to the SQLite path handling in one
copy will not reach the other. Resolving WR-01 (routing both through one
actionable-error path) would be a natural point to also consolidate the SQLite
setup into a single shared helper the consumer can reuse.
**Fix:** Optional, and coupled to WR-01. Consider exposing the framework's
SQLite-URL normalization as a small `pub(crate)`/`pub` helper so the consumer
`main.rs` and `app.rs` share one implementation.

---

_Reviewed: 2026-08-14_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
