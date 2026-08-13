---
phase: 246-result-read-model-snapshot
plan: "04"
subsystem: ferro-queue / ferro-macros / framework
tags: [offload, hook, projection, snapshot, worker, macro]
dependency_graph:
  requires:
    - ferro_projection::snapshot_write (246-01)
    - ferro::offload::persist_result / persist_error (246-02)
    - job_row.handle_key (246-03)
  provides:
    - Job::handle_with_value (ferro-queue)
    - OffloadResultHook + register_offload_result_hook (ferro-queue)
    - persist_offload_outcome crate-internal helper (ferro-queue)
    - handle_with_value override in macro-emitted Job impl (ferro-macros)
    - persist_result_raw + register_offload_hooks (framework/offload.rs)
    - framework boot calls register_offload_hooks() (framework/app.rs)
  affects:
    - ferro-queue public API (OffloadResultHook, register_offload_result_hook re-exported)
    - framework/src/offload.rs (new functions)
    - framework/src/app.rs (hook registration at boot)
    - ferro-macros/src/offload.rs (handle_with_value override emitted)
tech_stack:
  added: []
  patterns:
    - OnceLock fn-pointer hook injection (mirrors TENANT_ID_HOOK pattern)
    - Log-not-fail on persistence error (T-246-05, tracing::warn!)
    - handle_with_value provided method with default = discard via handle()
    - macro emits four value-capture arms (async/sync x Result/non-Result)
    - extract-handle_key-before-spawn pattern in spawn_job
key_files:
  created: []
  modified:
    - ferro-queue/src/job.rs
    - ferro-queue/src/worker.rs
    - ferro-queue/src/dispatcher.rs
    - ferro-queue/src/lib.rs
    - ferro-macros/src/offload.rs
    - framework/src/offload.rs
    - framework/src/app.rs
    - framework/src/lib.rs
decisions:
  - "handle_with_value provided method on Job trait (default: self.handle().await.map(|_| None)); macro overrides it — keeps handle() fixed at Result<(), Error>"
  - "No failed() override emitted by macro (D-10 correction): async worker path persists errors from spawn_job/handle_failure, not Job::failed()"
  - "OffloadResultHook is a fn-pointer OnceLock (mirrors TENANT_ID_HOOK) — ferro-queue gains no new crate dependency (D-11)"
  - "persist_offload_outcome is crate-internal pub(crate) — only worker.rs calls it"
  - "Terminal-error envelope persisted only when attempts+1 >= max_retries, not on transient failures (D-09)"
  - "register_offload_hooks() called in app.rs run_server_internal after Queue::init, before WorkerLoop spawn"
  - "Tenant-scope arm adapted: value_cell/delay_cell Arc<Mutex> shuttle the Option<Value> and Duration out of the with_scope boundary (which expects Future<Output=Result<(), Error>>)"
metrics:
  duration: ~30min
  completed: "2026-08-13T22:01:20Z"
  tasks_completed: 3
  tasks_total: 3
  files_modified: 8
requirements: [OFFLOAD-03]
---

# Phase 246 Plan 04: Value Capture and Result Write-Back — Summary

**One-liner:** Worker captures the success value from `Job::handle_with_value()` (macro-overridden to serialize the return), persists it via an injected `OffloadResultHook` on success and on terminal failure, with `ferro-queue` gaining no new crate dependency.

## What Was Built

### Task 1: `Job::handle_with_value` + `JobHandler` extension

`ferro-queue/src/job.rs` — new provided trait method:

```rust
async fn handle_with_value(&self) -> Result<Option<serde_json::Value>, Error> {
    self.handle().await.map(|_| None)
}
```

Default discards the value (`None`). Derived jobs override this; sync-mode `dispatch_immediately` still calls `handle()` unchanged.

`ferro-queue/src/worker.rs` — `JobHandler` type extended:

```rust
type JobHandler = Arc<
    dyn Fn(String, u32) -> Pin<
        Box<dyn Future<Output = (Result<Option<serde_json::Value>, Error>, Duration)> + Send>
    > + Send + Sync,
>;
```

The `register` closure now calls `job.handle_with_value().await` instead of `job.handle().await`.

`handle_key` is extracted from `job_row` alongside `tenant_id` in `spawn_job` for use at the write-back sites.

The tenant-scope arm was adapted to bridge the `with_scope` boundary (which expects `Future<Output=Result<(), Error>>`) using `Arc<Mutex>` cells to shuttle the `Option<Value>` and `Duration` out.

### Task 2: `OffloadResultHook` + `persist_offload_outcome` + worker write-back

`ferro-queue/src/dispatcher.rs` — hook type, static, registration fn, and crate-internal invoke helper:

```rust
pub type OffloadResultHook = fn(
    String,
    Result<serde_json::Value, String>,
    &'static sea_orm::DatabaseConnection,
) -> Pin<Box<dyn Future<Output = ()> + Send>>;

static OFFLOAD_RESULT_HOOK: OnceLock<OffloadResultHook> = OnceLock::new();
pub fn register_offload_result_hook(f: OffloadResultHook) { let _ = OFFLOAD_RESULT_HOOK.set(f); }

pub(crate) async fn persist_offload_outcome(
    handle_key: Option<&str>,
    outcome: Result<serde_json::Value, String>,
    db: &'static DatabaseConnection,
) { … }
```

`OffloadResultHook` and `register_offload_result_hook` re-exported from `ferro-queue/src/lib.rs`.

Worker write-back call sites in `spawn_job`:

- **Success arm:** `if let Some(val) = success_value { persist_offload_outcome(handle_key.as_deref(), Ok(val), conn).await; }`
- **Err arm:** passes `handle_key.as_deref()` to `handle_failure`
- **Panic arm:** passes `handle_key.as_deref()` to `handle_failure`

`handle_failure` extended with `handle_key: Option<&str>`. Terminal-error persistence fires only inside `if attempts + 1 >= max_retries` (D-09 — transient failures are not persisted):

```rust
if attempts + 1 >= max_retries {
    crate::dispatcher::persist_offload_outcome(handle_key, Err(err_msg.to_string()), conn).await;
    crate::db::fail_job(conn, job_id, err_msg).await.ok();
}
```

### Task 3: Macro override + framework hook registration

`ferro-macros/src/offload.rs` — `emit_job_items` now emits a `handle_with_value` override in the `impl ::ferro::queue::Job` block, after the existing `handle()`. Four value-capture arms mirror the four `call_expr` arms:

```rust
async fn handle_with_value(&self)
    -> Result<Option<::serde_json::Value>, ::ferro::queue::Error>
{
    let svc = ::ferro::App::make::<dyn #trait_ident>()
        .map_err(|e| ::ferro::queue::Error::job_failed(...))?;
    // async+Result: svc.method(args).await.map(|v| ::serde_json::to_value(&v).ok()).map_err(...)
    // async+non-Result: let v = svc.method(args).await; Ok(::serde_json::to_value(&v).ok())
    // sync+Result / sync+non-Result: analogous
}
```

No `failed()` override is emitted (D-10 correction).

`framework/src/offload.rs` — two new functions:

```rust
pub async fn persist_result_raw(handle_key: &str, value: serde_json::Value, db: &DatabaseConnection)
    -> Result<(), ProjectionError>

pub fn register_offload_hooks()  // registers the closure calling persist_result_raw / persist_error
```

The hook closure:
```rust
ferro_queue::register_offload_result_hook(|key, outcome, db| Box::pin(async move {
    let res = match outcome {
        Ok(value) => persist_result_raw(&key, value, db).await,
        Err(msg)  => persist_error(&key, &msg, db).await,
    };
    if let Err(e) = res {
        tracing::warn!(handle_key = %key, error = %e, "offload result persist failed — result not stored");
    }
}));
```

`framework/src/app.rs` — `register_offload_hooks()` called in `run_server_internal` immediately after `Queue::init()`:

```rust
crate::offload::register_offload_hooks();
```

## Note for Plan 05

The hook is registered by `App::run` (via `run_server_internal`). The Plan 05 integration test must call `crate::offload::register_offload_hooks()` explicitly in its test harness if it does not go through full framework boot (e.g., a bare `WorkerLoop` test with `sqlite::memory:`).

## Test Evidence

```
cargo build -p ferro-queue     exit 0
cargo build -p ferro-rs        exit 0
cargo test -p ferro-queue      56 tests (unit + integration), all passed
cargo test -p ferro-rs         5 tests passed, 364 ignored
cargo fmt --all -- --check     clean
cargo clippy -p ferro-rs -p ferro-macros -p ferro-queue --all-targets -- -D warnings   clean
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Tenant-scope arm type mismatch after JobHandler extension**

- **Found during:** Task 1 compile run
- **Issue:** The tenant-scope arm in `spawn_job` fed `handler()` result through `scope.with_scope(id, fut)`, which returns `Result<(), Error>`. After `JobHandler` changed to return `Result<Option<Value>, Error>`, the types mismatched and type inference failed for the outer `result` binding.
- **Fix:** Added explicit type annotation `let result: Result<(Result<Option<serde_json::Value>, Error>, Duration), _>` and adapted the tenant-scope arm to use `Arc<Mutex>` cells to shuttle `Option<Value>` and `Duration` out of the `with_scope` future (which must stay `Result<(), Error>`). The non-tenant-scoped arm passes through unchanged.
- **Files modified:** `ferro-queue/src/worker.rs`
- **Commit:** `b4dfc6c4`

**2. [Rule 1 - Style] rustfmt reformatted `JobHandler` type alias and inline comments**

- **Found during:** Task 3 pre-commit `cargo fmt --check`
- **Issue:** The long `JobHandler` type alias line exceeded rustfmt's line width; inline alignment padding in the `OffloadResultHook` type definition was also non-standard.
- **Fix:** Applied `cargo fmt --all`.
- **Files modified:** `ferro-queue/src/worker.rs`, `ferro-queue/src/dispatcher.rs`, `ferro-queue/src/lib.rs`
- **Commit:** `aae642be` (included in Task 3 commit)

## Commits

| Task | Hash | Message |
|------|------|---------|
| 1 | `b4dfc6c4` | feat(246-04): add Job::handle_with_value and extend JobHandler to carry Option<Value> |
| 2 | `4d873389` | feat(246-04): add OffloadResultHook OnceLock and persist_offload_outcome to ferro-queue |
| 3 | `aae642be` | feat(246-04): macro handle_with_value override + framework hook registration |

## Known Stubs

None. All three tasks are fully implemented. The value-capture and write-back paths are wired but untested end-to-end — Plan 05 provides the integration test that asserts the round-trip.

## Threat Surface Scan

No new network endpoints or auth paths introduced. The `OffloadResultHook` is an in-process fn-pointer registered at boot; the value that flows through it came from an in-process `OffloadSerializable` return (T-246-tamper, mitigated). T-246-05 (persistence failure silences the job outcome) is mitigated structurally: the hook signature returns `()` and all persistence errors are logged via `tracing::warn!`.

## Self-Check: PASSED

- [x] `ferro-queue/src/job.rs` — `handle_with_value` provided method
- [x] `ferro-queue/src/worker.rs` — `JobHandler` extended, `handle_with_value()` called, `handle_key` extracted, 2 `persist_offload_outcome` calls, `handle_failure` extended
- [x] `ferro-queue/src/dispatcher.rs` — `OffloadResultHook`, `OFFLOAD_RESULT_HOOK`, `register_offload_result_hook`, `persist_offload_outcome`
- [x] `ferro-queue/src/lib.rs` — `register_offload_result_hook` and `OffloadResultHook` re-exported
- [x] `ferro-macros/src/offload.rs` — `handle_with_value` override emitted; no `failed()` override; `to_value` in 6 places
- [x] `framework/src/offload.rs` — `persist_result_raw`, `register_offload_hooks`, `tracing::warn!`
- [x] `framework/src/app.rs` — `crate::offload::register_offload_hooks()` at boot
- [x] `framework/src/lib.rs` — `register_offload_hooks` referenced in doc comment
- [x] Commits `b4dfc6c4`, `4d873389`, `aae642be` exist in git log
- [x] `cargo build -p ferro-rs` exit 0
- [x] `cargo test -p ferro-queue` all passed
- [x] `cargo clippy … -- -D warnings` clean
- [x] `cargo fmt --all -- --check` clean
- [x] D-11 guard: no `use ferro_projection` / `use framework` / `::ferro::offload` in `ferro-queue/src/` (doc comment mention only)
- [x] D-10 guard: `! grep -q "async fn failed" ferro-macros/src/offload.rs`
