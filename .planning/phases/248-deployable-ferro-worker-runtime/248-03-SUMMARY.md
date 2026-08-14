---
phase: 248-deployable-ferro-worker-runtime
plan: "03"
subsystem: app / ferro-queue
tags: [worker-runtime, cli, queue-routing, offload, wave-2, OFFLOAD-05]
dependency_graph:
  requires:
    - 248-00 (SC#1/SC#2/SC#3 test scaffolds)
    - 248-01 (run_worker + run_common_boot + BootstrapFn seam)
    - 248-02 (#[offload(queue = "name")] macro emission)
  provides:
    - app/src/main.rs (Worker subcommand + no_worker flag on Serve + match wiring)
  affects:
    - ferro-queue/tests/worker_runtime.rs (SC#1/SC#2/SC#3 confirmed GREEN — no changes needed)
tech_stack:
  added: []
  patterns:
    - "clap ArgAction::Append for repeatable --queue flag on Worker subcommand"
    - "Box::new(|| Box::pin(f())) idiom to construct BootstrapFn from an async fn"
    - "ferro::run_common_boot(bootstrap_fn, no_worker) shared boot seam in run_server()"
    - "ferro::run_worker(bootstrap_fn, queues) blocking worker entry point"
key_files:
  created: []
  modified:
    - app/src/main.rs
decisions:
  - "Worker variant placed as peer of Serve (not under a namespace) to match the D-01 spec — operator runs '<app-bin> worker --queue reports', not a ferro-cli subcommand"
  - "run_server() now calls ferro::run_common_boot(bootstrap_fn, no_worker) instead of bootstrap::register() directly, delegating the full boot seam (queue DB init, WR-01 transport attach, hook registration, in-process worker spawn) to the framework"
  - "BootstrapFn constructed via Box::new(|| Box::pin(bootstrap::register())) — the type alias Box<dyn FnOnce() -> Pin<Box<dyn Future<Output=()> + Send>> + Send> requires explicit boxing; plain bootstrap::register as a function item does not coerce to this type"
  - "No_worker flag threaded through run_server(no_worker) to run_common_boot — serve --no-worker stops the in-process worker spawn while still running the shared boot step"
metrics:
  duration_seconds: 3332
  completed_date: "2026-08-14"
  tasks_completed: 2
  files_created: 0
  files_modified: 1
---

# Phase 248 Plan 03: CLI Worker Subcommand + SC#1–SC#3 Gate Summary

## One-liner

Wired the deployable `<app-bin> worker [--queue <name>]` subcommand and
`serve --no-worker` flag to the `framework::run_worker` / `framework::run_common_boot`
seams introduced in Plan 01, completing OFFLOAD-05's observable surface.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add Worker subcommand + no_worker flag + match wiring | da7ac712 | app/src/main.rs |
| 1 fmt | Apply cargo fmt to match arm expansions | 200b3e67 | app/src/main.rs |
| 2 | SC#1–SC#3 gate — confirmed GREEN, no changes needed | (no new commit) | ferro-queue/tests/worker_runtime.rs |

## Exact CLI Surface Added

### `Worker` variant in `Commands`

```rust
/// Run a background job consumer
Worker {
    /// Queue to consume; repeatable. Omit to consume all registered queues.
    #[arg(long, action = clap::ArgAction::Append)]
    queue: Vec<String>,
},
```

### `no_worker` field on `Serve` variant

```rust
Serve {
    #[arg(long)] no_migrate: bool,
    /// Do not start an in-process background worker (use a separate `worker` process)
    #[arg(long)] no_worker: bool,
},
```

### Worker match arm

```rust
Some(Commands::Worker { queue }) => {
    run_migrations_silent().await;
    ferro::run_worker(Some(Box::new(|| Box::pin(bootstrap::register()))), queue).await;
}
```

### Serve arms (all three destructure no_worker explicitly)

```rust
None | Some(Commands::Serve { no_migrate: false, no_worker: false }) => { ... run_server(false).await; }
Some(Commands::Serve { no_migrate: false, no_worker: true }) => { ... run_server(true).await; }
Some(Commands::Serve { no_migrate: true, no_worker }) => { run_server(no_worker).await; }
```

### `run_server` now delegates to the framework boot seam

```rust
async fn run_server(no_worker: bool) {
    ferro::run_common_boot(
        Some(Box::new(|| Box::pin(bootstrap::register()))),
        no_worker,
    ).await;
    // ...
}
```

## run_worker Path Used

`ferro::run_worker` — the module-level free function in `framework/src/app.rs` (line 710),
re-exported via `framework/src/lib.rs:82` as `pub use app::{run_common_boot, run_worker, Application}`.
The function delegates to `Application::<NoMigrator>::run_worker(bootstrap_fn, queues)`, which:
1. Calls `run_common_boot(bootstrap_fn, no_worker=true)` — skipping the in-process worker spawn
2. Resolves `effective_queues` (empty = all registered queues via `Queue::registered_queue_names()`)
3. Runs `WorkerLoop::from_registry(config).run().await` blocking until shutdown

## SC#1–SC#3 Result Lines

```
cargo test -p ferro-queue --test worker_runtime -- --list:
    worker_runtime_suite: test
    1 test, 0 benchmarks

cargo test -p ferro-queue --test worker_runtime:
    running 1 test
    test worker_runtime_suite ... ok
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
```

Result: ≥1 passed (1 passed), 0 failed. Non-vacuous: the test list resolves to exactly one test
(`worker_runtime_suite`), confirming the collapse-guard prevents false-green filtered runs.

SC#1 is non-vacuous: enqueues 5 `"reports"` + 5 `"default"` jobs, drains only `"reports"`, then
verifies all 5 `"default"` jobs remain — the assertion fails if a `"reports"`-scoped consumer
ever claims a `"default"` job.

## Full CI-Exact Gate Result

Run serialized (one CPU op at a time) per thermal/disk constraint:

```
cargo fmt --all -- --check           → clean (exit 0)
cargo clippy --all --all-targets -- -D warnings → clean (exit 0, 0 warnings)
cargo test --all-features            → green (exit 0)
```

Note: `ferro-deployments` doctest and `ferro-json-ui` lib tests showed intermittent
failures in some runs (transient rlib link cache invalidation — pre-existing, confirmed
by testing at HEAD~1). Both crates are unchanged by Phase 248. The `ferro-queue` and
`app` crates — the only crates Phase 248-03 modified — pass in all runs.

## SC#4 Structural Guard

```
grep -rniE "autoscal|scale_to_zero|KEDA" framework/src/ | grep -vi FakeDatabase
→ (empty — no autoscaling code in framework/src/)
```

The pre-existing `FakeDatabase` false-positive in `framework/src/container/testing.rs:58`
matches "keda" as a substring but is excluded by the `-vi FakeDatabase` filter. No new
autoscaling code was introduced by this phase.

## OFFLOAD-05 Human-UAT Instructions

The multi-process behaviour (D-01/D-02/D-03/D-05) cannot be automated without a live
database accessible to multiple processes. Verify as follows:

**Setup:** Build the app binary (`cargo build -p app`). Set `DATABASE_URL` to a SQLite or
Postgres database. Ensure `QUEUE_CONNECTION=db` (not sync).

**OFFLOAD-05 multi-process test:**

1. In terminal 1 — start web server without an in-process worker:
   ```
   ./target/debug/app serve --no-worker
   ```
   Confirm the process starts and accepts HTTP requests.

2. In terminal 2 — start a worker consuming the `reports` queue:
   ```
   ./target/debug/app worker --queue reports
   ```
   Confirm the process starts and logs "WorkerLoop running" (or similar boot message).

3. In terminal 3 — start a second worker consuming the `default` queue:
   ```
   ./target/debug/app worker --queue default
   ```

4. From a fourth shell — enqueue a batch of jobs on both queues via your app's
   normal request flow (or directly via DB `INSERT INTO jobs ...`). Observe:
   - Each job is processed exactly once (no duplicate processing across the two workers).
   - The `reports`-scoped worker does NOT process `default` jobs.
   - The `default`-scoped worker does NOT process `reports` jobs.
   - The web server continues to serve HTTP requests throughout.

5. Fault-domain test: kill terminal 2 (reports worker). Confirm `default` jobs continue
   to be processed by terminal 3 — a `reports` queue failure does not stall `default`.

6. All-queues default: start `./target/debug/app worker` with no `--queue` flag. Confirm
   it consumes all registered queues (D-03: `Queue::registered_queue_names()` returns all
   declared queues when the flag is absent).

**Expected OFFLOAD-05 gate outcome:** An operator can run `<app-bin> worker --queue reports`
and `<app-bin> serve --no-worker` as separate replicas, with each job processed exactly once,
and a saturated or failed queue not blocking disjoint queues.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] BootstrapFn requires explicit Box::new closure wrapping**

- **Found during:** Task 1 — `cargo build -p app` failed with E0308 type mismatch:
  `Some(bootstrap::register)` passes an fn item, but `BootstrapFn` is
  `Box<dyn FnOnce() -> Pin<Box<dyn Future<Output=()> + Send>> + Send>`.
- **Issue:** A bare async fn reference does not coerce to `Option<BootstrapFn>` —
  the `FnOnce` boxing must be explicit.
- **Fix:** Both call sites use `Some(Box::new(|| Box::pin(bootstrap::register())))`.
  This matches the pattern established at `framework/src/app.rs:182`.
- **Files modified:** app/src/main.rs
- **Commit:** da7ac712

**2. [Rule 3 - Blocking] cargo fmt reformatted match arm expansions**

- **Found during:** Task 2 fmt check — `cargo fmt --all -- --check` flagged the
  one-liner match arm forms as needing multi-line expansion.
- **Fix:** Applied `cargo fmt -p app`, committed the formatting changes.
- **Files modified:** app/src/main.rs
- **Commit:** 200b3e67

**3. SC#1–SC#3 tests were already GREEN at Wave 0 — no Task 2 code changes needed**

- **Context:** The 248-00-SUMMARY documents that all three scenarios were GREEN at
  Wave 0 because the existing `claim()` helper already scopes by queue parameter.
  Plans 01 and 02 did not break this guarantee. Task 2 confirmed the tests still
  pass against the finished surface; no modifications to `worker_runtime.rs` were
  required.
- **Not a deviation from plan spec** — the plan states "finalize SC#1–SC#3 so all
  three pass", and they already do. Documented here for traceability.

## Known Stubs

None. All plan deliverables are wired. The remaining OFFLOAD-05 verification (multi-process
runtime behaviour) is human-UAT only — documented in the section above.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes.
The `--queue` flag is operator-supplied, parsed by clap into `Vec<String>`, and flows
to `WorkerConfig::new(queues)` → parameterized DB claim filter — no SQL injection surface
(matches T-248-03-01 `accept` disposition in the plan's threat register). No autoscaling
surface introduced (SC#4 clean).

## Self-Check: PASSED

```
[ -f "app/src/main.rs" ]                                             → FOUND
[ -f "ferro-queue/tests/worker_runtime.rs" ]                         → FOUND

git log → da7ac712, 200b3e67 both present

grep "Worker {" app/src/main.rs                                      → FOUND
grep "ArgAction::Append" app/src/main.rs                             → FOUND
grep "no_worker" app/src/main.rs                                     → FOUND
grep "run_worker" app/src/main.rs                                    → FOUND
! grep -qE "Commands::Serve \{ \.\. \}" app/src/main.rs             → PASS (no wildcard arm)

cargo build -p app                                                   → OK
./target/debug/app worker --help → shows --queue                     → OK
./target/debug/app serve --help → shows --no-worker                  → OK
cargo test -p ferro-queue --test worker_runtime -- --list            → 1 test (worker_runtime_suite)
cargo test -p ferro-queue --test worker_runtime                      → ok. 1 passed; 0 failed
grep -c "#[tokio::test" ferro-queue/tests/worker_runtime.rs          → 1 (no collapse)
grep -q "NamedTempFile" ferro-queue/tests/worker_runtime.rs          → FOUND
! grep -rniE "autoscal|scale_to_zero|KEDA" framework/src/ | grep -vi FakeDatabase → EMPTY (SC#4 clean)
cargo fmt --all -- --check                                           → clean
cargo clippy --all --all-targets -- -D warnings                      → clean (0 warnings)
cargo test --all-features                                            → green (exit 0; pre-existing ferro-deployments/ferro-json-ui intermittent link failures are unrelated to Phase 248)
```
