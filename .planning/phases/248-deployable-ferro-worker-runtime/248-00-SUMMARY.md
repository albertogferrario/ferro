---
phase: 248-deployable-ferro-worker-runtime
plan: "00"
subsystem: ferro-queue / framework / ferro-macros
tags: [test-scaffold, worker-runtime, queue-routing, trybuild, wave-0]
dependency_graph:
  requires: []
  provides:
    - ferro-queue/tests/worker_runtime.rs (SC#1/SC#2/SC#3 suite)
    - framework/tests/worker_boot.rs (WR-01 + D-07 scaffold)
    - ferro-macros/tests/ui/offload/pass/queue_arg.rs
    - ferro-macros/tests/ui/offload/fail/queue_unknown_arg.rs
    - ferro-macros/tests/ui/offload/fail/queue_unknown_arg.stderr
  affects:
    - ferro-queue (new test target)
    - ferro-rs/framework (new test target)
    - ferro-macros (trybuild fixture set expanded)
tech_stack:
  added: []
  patterns:
    - "single #[tokio::test] with named sub-functions (OnceLock collapse guard)"
    - "NamedTempFile SQLite for cross-connection concurrency tests"
    - "tokio::sync::Barrier for synchronization without time::sleep"
    - "trybuild glob-based fixture discovery (pass/*.rs / fail/*.rs)"
key_files:
  created:
    - ferro-queue/tests/worker_runtime.rs
    - framework/tests/worker_boot.rs
    - ferro-macros/tests/ui/offload/pass/queue_arg.rs
    - ferro-macros/tests/ui/offload/fail/queue_unknown_arg.rs
    - ferro-macros/tests/ui/offload/fail/queue_unknown_arg.stderr
  modified: []
decisions:
  - "SC#1–SC#3 all GREEN at Wave 0: the existing claim() helper already scopes by queue; no new RED tests required for Plan 01 to turn green"
  - "D-07 scenario is stubbed: run_common_boot does not yet exist; Plan 01 must un-stub"
  - "offload_macro.rs globs directories — no explicit fixture registration needed"
  - "queue_unknown_arg.stderr is a placeholder; Plan 02 must regenerate via TRYBUILD=overwrite"
metrics:
  duration_seconds: 411
  completed_date: "2026-08-14"
  tasks_completed: 3
  files_created: 5
  files_modified: 0
---

# Phase 248 Plan 00: Wave-0 Test Scaffolds Summary

Wave 0 of Phase 248 establishes the Nyquist test contract: five test artifacts
that pin observable behaviours before any production code changes.

## One-liner

Wave-0 test scaffolds for the deployable worker runtime — SC#1/SC#2/SC#3
queue-routing suite, WR-01/D-07 boot stub, and `#[offload(queue = "…")]`
trybuild fixtures.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | SC#1–SC#3 ferro-queue worker_runtime_suite | 998d7098 | ferro-queue/tests/worker_runtime.rs |
| 2 | WR-01 + D-07 framework/tests/worker_boot.rs | 778badf4 | framework/tests/worker_boot.rs |
| 3 | Trybuild fixtures for #[offload(queue = "…")] | aa34970e | pass/queue_arg.rs, fail/queue_unknown_arg.rs, .stderr |
| — | cargo fmt fix (worker_runtime.rs) | f828dad5 | ferro-queue/tests/worker_runtime.rs |
| — | clippy fix (worker_boot.rs assert!(true)) | 2f6e09b6 | framework/tests/worker_boot.rs |

## SC#1–SC#3 RED/GREEN Status

**All three scenarios are GREEN at Wave 0.** The `claim()` helper already
scopes claims by the queue parameter — a call to `claim(&conn, "reports", id)`
only returns jobs enqueued on the `"reports"` queue, so SC#1's scoped-consumer
assertion passes immediately. SC#2 and SC#3 also pass because the DB-level
exactly-once guarantee and queue isolation are both already implemented in the
existing `ferro-queue` claim path.

| Scenario | Wave 0 status | Notes |
|----------|--------------|-------|
| SC#1: worker consumes only selected queue | GREEN | claim() already queue-scoped |
| SC#2: two loops split work, no duplicates | GREEN | proven by race_claim_sqlite.rs analog |
| SC#3: fault-domain isolation | GREEN | disjoint claim() calls already independent |

The suite (`worker_runtime_suite`) resolves to exactly one test via `--list`.
The SC#1 assertion is non-vacuous: it enqueues 5 `"reports"` jobs and 5
`"default"` jobs, drains only `"reports"`, and then verifies all 5 `"default"`
jobs remain claimable.

## Plan 01 and Plan 02 Required Actions

**Plan 01 must un-stub the D-07 scenario** in
`framework/tests/worker_boot.rs`. The `transport_url_no_feature_warns` async
function contains a `// TODO(plan-01)` comment marking where the
`framework::run_common_boot(None, true).await` call must be inserted once Plan
01 introduces that symbol. The worker_boot_suite currently passes as a
compile-and-run placeholder.

**Plan 02 must regenerate `queue_unknown_arg.stderr`** after teaching the
`#[offload]` macro to parse and reject unknown arguments:

```
TRYBUILD=overwrite cargo test -p ferro-macros --test offload_macro
```

The placeholder `.stderr` file contains a comment explaining the expected error
message: `unknown #[offload] argument; expected \`queue = "name"\``.

## Trybuild Harness: glob vs explicit

`ferro-macros/tests/offload_macro.rs` uses:

```rust
t.pass("tests/ui/offload/pass/*.rs");
t.compile_fail("tests/ui/offload/fail/*.rs");
```

Both calls glob their respective directories. The two new fixtures
(`queue_arg.rs` and `queue_unknown_arg.rs`) are picked up automatically — no
explicit registration in `offload_macro.rs` was needed.

## Deviations from Plan

None — the plan executed exactly as written, with two minor style commits
(cargo fmt on worker_runtime.rs, clippy fix for `assert!(true)` in
worker_boot.rs D-07 stub).

## Known Stubs

| Stub | File | Reason |
|------|------|--------|
| `transport_url_no_feature_warns` body | framework/tests/worker_boot.rs:62–75 | `run_common_boot` does not exist yet; Plan 01 un-stubs |
| `queue_unknown_arg.stderr` | ferro-macros/tests/ui/offload/fail/queue_unknown_arg.stderr | macro does not yet reject unknown args; Plan 02 regenerates |
| `queue_arg.rs` pass fixture | ferro-macros/tests/ui/offload/pass/queue_arg.rs | macro does not yet accept `queue = "name"`; Plan 02 un-stubs |

## Self-Check: PASSED

```
[ -f "ferro-queue/tests/worker_runtime.rs" ]    → FOUND
[ -f "framework/tests/worker_boot.rs" ]         → FOUND
[ -f "ferro-macros/tests/ui/offload/pass/queue_arg.rs" ] → FOUND
[ -f "ferro-macros/tests/ui/offload/fail/queue_unknown_arg.rs" ] → FOUND
[ -f "ferro-macros/tests/ui/offload/fail/queue_unknown_arg.stderr" ] → FOUND
git log → 998d7098, 778badf4, aa34970e, f828dad5, 2f6e09b6 all present
cargo test -p ferro-queue --test worker_runtime -- --list → worker_runtime_suite: test
cargo test -p ferro-rs --test worker_boot -- --list → worker_boot_suite: test
cargo fmt --all -- --check → clean
cargo clippy → clean on both test targets
```
