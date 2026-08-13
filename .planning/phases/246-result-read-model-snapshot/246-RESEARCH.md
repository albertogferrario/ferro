# Phase 246: Result → read-model snapshot - Research

**Researched:** 2026-08-13
**Domain:** ferro-queue × ferro-projection × ferro-macros × framework integration
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01/D-02:** Persistence via a new direct snapshot write/read API on `ferro-projection`,
  decoupled from the `Projection` event-fold trait. Reuses `projection_snapshots` entity —
  no new table/migration.
- **D-03:** Coherence-tax expansion of `ferro-projection`; the event-fold path is unchanged.
- **D-04:** Handle key is minted BEFORE dispatch, so both caller and worker hold the same key.
- **D-05:** Handle key travels as job-execution metadata (NOT a payload field). The exact
  carrier mechanism is a planning decision.
- **D-06:** Handle key stays decoupled from `Job::idempotency_key()`.
- **D-07:** Snapshot `state` holds a tagged envelope: `{"status":"completed","value":<T>}` or
  `{"status":"failed","error":"<msg>"}`.
- **D-08:** No "pending" row at enqueue in 246. `None` = not yet done.
- **D-09:** Success snapshot written when derived `handle()` returns `Ok`. The `.map(|_| ())`
  call is replaced by value capture + persist.
- **D-10:** Terminal-error snapshot written when retries exhausted and `Job::failed()` is called.
  The derived job's `failed()` is overridden by the macro.
- **D-11:** Write-back glue lives in `framework`. Macro emits `::ferro::*` paths only.
- **D-12:** `Queue::connection()` (`&'static DatabaseConnection`) is the DB source for write-back.
- **D-13:** Projection name `"offload.result"`, key = handle UUID.

### Claude's Discretion
- Exact `ferro-queue` mechanism for carrying the handle key (queue-row column vs. job-context slot).
- Name/shape of the `ferro-projection` direct-write surface (free functions vs. struct).
- Exact envelope tag/field names and serde representation.
- Name/module of `::ferro::offload::*` write-back helpers and read-back wrapper.
- Whether the read surface attaches to `OffloadHandle<T>` now or waits for 247.
- Panic capture detail (whether worker's existing path already routes to `failed()`).

### Deferred Ideas (OUT OF SCOPE)
- Shared broadcast transport for multi-replica delta delivery (Phase 246.1).
- Read-model delta → broadcast streaming; handle `.await`/`.subscribe()` (Phase 247).
- Pending/enqueued marker and unknown-handle-vs-not-done distinction (Phase 247).
- Deployable `worker` subcommand runtime (Phase 248).
- `ferro-mcp` introspection + docs (Phase 249).
- Snapshot retention/TTL for completed results.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| OFFLOAD-03 | An offloaded method's return value is persisted as a `ferro-projection` snapshot keyed by the handle, retrievable after completion; a failed run records a terminal error state (no silent drop). | Addressed in full: persistence path (RQ1–RQ3), direct-write API (RQ2), terminal-error seam gap (RQ4 critical finding), crate deps (RQ6), test strategy (RQ8). |
</phase_requirements>

---

## Summary

Phase 246 wires the result path of the offload substrate: after the worker finishes a derived
`Job::handle()`, it persists the return value into the `projection_snapshots` table under
`(projection_name="offload.result", key=<handle-UUID>)`, and exposes a read-back helper so a
caller can retrieve the result by handle. A terminally failed run writes a `{"status":"failed",
"error":"<msg>"}` envelope instead of silently dropping.

The research verified all canonical source files listed in 246-CONTEXT.md. Five findings have
direct impact on the plan, one of which is a gap versus the CONTEXT.md's stated D-10 assumption.

**Primary recommendation:** The four changes are: (1) add a direct write/read API to
`ferro-projection`; (2) rework `Offloadable::offload()` to mint-before-dispatch and embed the
key via `PendingDispatch`; (3) extend the macro's `handle()` arms to capture and persist the
value; (4) wire the terminal-error path by extending `WorkerLoop::handle_failure` to call
`Job::failed()` on the deserialized job after retries are exhausted — because `Job::failed()`
is NOT called in the current async worker path.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Direct snapshot write/read | `ferro-projection` | — | The `projection_snapshots` entity and SeaORM upsert pattern already live there; adding a bypass API over the same entity is the minimal coherence-tax change. |
| Handle-key propagation to worker | `ferro-queue` (PendingDispatch + DB row) | `ferro-macros` (emit the key field) | The key must survive serialization/deserialization across the enqueue→claim boundary; it belongs in the jobs row, not in the JSON payload. |
| Value capture in derived `handle()` | `ferro-macros` | `framework` (helper fn) | The macro generates the `handle()` body; the actual persistence call goes through a `::ferro::offload::*` helper to stay `::ferro::*`-only. |
| Terminal-error persistence | `ferro-queue` (WorkerLoop) | `ferro-macros` (Job::failed override) | See critical finding: `Job::failed()` is not called in the real async worker path today; the worker needs extending first. |
| DB source for write-back | `ferro-queue` (Queue::connection()) | — | Already a `&'static DatabaseConnection` available inside the worker's spawned task. |
| Framework facade re-exports | `framework/src/lib.rs` | — | All `::ferro::offload::*` helpers and `OffloadResult<T>` surface here, consistent with 244/245 convention. |

---

## Standard Stack

### Core (verified from source)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `sea-orm` | 1.0 | SeaORM upsert + find_by_id for snapshot reads/writes | Already used in ferro-projection and framework; the `OnConflict` idiom is proven in `apply_event`. |
| `serde_json` | 1 | Envelope serialization (`Value`) | In `projection_snapshots.state: JsonValue`; the envelope is a `serde_json::Value` in both write and read paths. |
| `chrono` | 0.4 | `Utc::now().naive_utc()` for `updated_at` | Consistent with every other snapshot write in `apply_event`. |
| `uuid` | 1 | Handle key identity | Already used for `HandleKey::new()` in `ferro-queue/src/offload.rs`. |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `async-trait` | 0.1 | Required for `Job::failed(&self)` override (async trait method) | The `Job` trait is `#[async_trait]`; overriding `failed()` needs it. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Embedding the handle key in a new `jobs` table column | Embedding in the JSON payload as a reserved field | A new column is the clean approach (payload = method params only, D-05); payload embedding would break D-05 and make the envelope visible to serde. |
| Free functions for direct-write API | A `SnapshotStore` struct wrapping `&DatabaseConnection` | Free functions are simpler and sufficient for a single write; the struct adds flexibility if multiple callers need different DBs (unlikely here). |

---

## Architecture Patterns

### System Architecture Diagram

```
Caller (web tier)
  │
  │  ReportsBuildMonthlyJob { .. }.offload().await?
  │     └── Offloadable::offload()                 ← reworked in 246
  │           ├── 1. mint HandleKey (UUID v4)
  │           ├── 2. PendingDispatch::new(self)
  │           │         .with_handle_key(key.clone())   ← NEW: carry key via PendingDispatch field
  │           │         .dispatch().await               → INSERT INTO jobs (payload, handle_key, …)
  │           └── 3. return OffloadHandle::new(key)     ← caller holds the key
  │
  ╔════════════════════════════════════════════╗
  ║       jobs table row                       ║
  ║  payload = {tenant_id, month}              ║
  ║  handle_key = "<UUID>"   ← new column      ║
  ╚════════════════════════════════════════════╝
           │
           │   WorkerLoop::claim() → JobRow {handle_key, payload, …}
           ▼
  WorkerLoop::spawn_job()
    └── handler(job_row.payload, attempts)   ← existing closure
          ├── deserialize Job from payload
          │
          ├── (SUCCESS PATH) job.handle().await → Ok(value)
          │      └── ::ferro::offload::persist_result(key, &value, db).await
          │              → upsert projection_snapshots
          │                (name="offload.result", key=handle_uuid,
          │                 state={"status":"completed","value":<T>}, version=1)
          │
          └── (FAILURE PATH after retries exhausted)
                 handle_failure → job.failed(&err).await     ← worker must call this
                        └── ::ferro::offload::persist_error(key, &msg, db).await
                                → upsert projection_snapshots
                                  (state={"status":"failed","error":"<msg>"})

  ─────────────────────────────────────────────

  Later: caller reads by handle
    ::ferro::offload::read_result::<Report>(handle, db).await
      → Entity::find_by_id(("offload.result", handle_uuid))
      → deserialize envelope → Some(Completed(report)) | Some(Failed("…")) | None
```

### Recommended Project Structure (additions only)

```
ferro-projection/src/
├── direct.rs       # new: snapshot_write() + snapshot_read() free functions
├── lib.rs          # export pub use direct::{snapshot_write, snapshot_read}

ferro-queue/src/
├── offload.rs      # extend Offloadable::offload() for mint-before-dispatch
├── dispatcher.rs   # add handle_key field to PendingDispatch + dispatch_to_queue
├── db.rs           # add handle_key column to enqueue + JobRow; extend fail_job path

framework/src/
├── offload.rs      # new module: persist_result(), persist_error(), read_result(), OffloadResult<T>
├── lib.rs          # pub mod offload; re-export at ::ferro::offload::*
```

### Pattern 1: Direct Snapshot Write (new `ferro-projection/src/direct.rs`)

```rust
// Source: mirrors apply_event upsert at ferro-projection/src/runtime.rs:158
use chrono::Utc;
use sea_orm::{sea_query::OnConflict, ActiveValue, DatabaseConnection, EntityTrait};
use serde_json::Value as JsonValue;

use crate::entity::{ActiveModel, Column, Entity};
use crate::error::ProjectionError;
use crate::key::ProjectionKey;

/// Write a snapshot directly, bypassing the event-fold Projection trait.
/// Uses an upsert (OnConflict on composite PK) so repeat writes are idempotent.
/// Sets version = 1 on first write; overwrites on subsequent (last-writer-wins).
pub async fn snapshot_write(
    db: &DatabaseConnection,
    name: &str,
    key: &ProjectionKey,
    state: JsonValue,
) -> Result<(), ProjectionError> {
    let now = Utc::now().naive_utc();
    let am = ActiveModel {
        projection_name: ActiveValue::Set(name.to_string()),
        key: ActiveValue::Set(key.0.clone()),
        state: ActiveValue::Set(state),
        version: ActiveValue::Set(1),
        updated_at: ActiveValue::Set(now),
    };
    Entity::insert(am)
        .on_conflict(
            OnConflict::columns([Column::ProjectionName, Column::Key])
                .update_columns([Column::State, Column::UpdatedAt])
                .to_owned(),
        )
        .exec(db)
        .await?;
    Ok(())
}

/// Read a snapshot by (name, key). Returns Ok(None) if the row is absent.
pub async fn snapshot_read(
    db: &DatabaseConnection,
    name: &str,
    key: &ProjectionKey,
) -> Result<Option<JsonValue>, ProjectionError> {
    let row = Entity::find_by_id((name.to_string(), key.0.clone()))
        .one(db)
        .await?;
    Ok(row.map(|m| m.state))
}
```

**Note:** The version field is set to 1 on every `snapshot_write` call; the `OnConflict` clause
only updates `state` and `updated_at`, leaving `version` at 1. This differs from `apply_event`'s
version-increment idiom: for one-shot results, incrementing the version on re-write is not
meaningful and adds complexity. If overwrite-version tracking is later needed, it can be
added to the `OnConflict` clause. This is a discretion call for the planner.

### Pattern 2: Result Envelope in `framework/src/offload.rs`

```rust
// Source: mirrors D-07 tagged envelope; serde internally-tagged for unambiguous discrimination
use serde::{Deserialize, Serialize};

pub const OFFLOAD_PROJECTION_NAME: &str = "offload.result";

/// Typed result of a completed or terminally failed offloaded call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum OffloadResult<T> {
    Completed { value: T },
    Failed { error: String },
}

/// Tagged envelope stored in projection_snapshots.state.
/// `T: OffloadSerializable` ensures round-trip.
#[derive(Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum Envelope {
    Completed { value: serde_json::Value },
    Failed { error: String },
}
```

### Pattern 3: Handle Key in `PendingDispatch` (carrying the key to the worker)

The cleanest mechanism (see Research Question 1 for the full analysis) is a new `handle_key`
field on `PendingDispatch` + a new `handle_key` column in the `jobs` table + reading it back
in `JobRow`. This keeps the payload clean and mirrors how `tenant_id` is already propagated:

```rust
// ferro-queue/src/dispatcher.rs
pub struct PendingDispatch<J> {
    job: J,
    queue: Option<&'static str>,
    delay: Option<Duration>,
    tenant_id: Option<i64>,
    handle_key: Option<String>,  // NEW
}

impl<J> PendingDispatch<J> where J: Job + Serialize + DeserializeOwned {
    /// Attach an offload handle key to the pending dispatch.
    pub fn with_handle_key(mut self, key: String) -> Self {
        self.handle_key = Some(key);
        self
    }
}
```

The `enqueue` function in `db.rs` receives the optional `handle_key` and stores it in the jobs
row. `parse_job_row` reads it back into `JobRow`. The worker's handler closure passes it to the
success/failure paths.

### Anti-Patterns to Avoid

- **Embedding `handle_key` in the JSON payload:** Breaks D-05 (payload = method params only).
  The derive-emitted `Serialize/Deserialize` on the Job struct would include the field, and
  the `OffloadSerializable` bound would wrongly require the key type to be serializable from
  the method contract's perspective. Use a table column, same as `tenant_id`.
- **Setting `version` from the existing row + 1 in `snapshot_write`:** Requires a read before
  the upsert, adding a round-trip. Since offload handles are single-writer, version tracking
  has no operational value and the read-modify-write adds concurrency risk (race between read
  and upsert). Use a fixed value of 1.
- **Calling `persist_result` inside the `register` handler closure:** The handler closure
  captures `String` payload, not the deserialized job; the handle key must travel from
  `JobRow` into the spawned task separately (via `job_row.handle_key`).
- **Adding `ferro-projection` as a dependency of `ferro-queue`:** Violates D-11 and the crate
  isolation principle — `ferro-projection` depends on `ferro-events` + `ferro-broadcast`, which
  would pull the entire broadcast stack into `ferro-queue`. The write-back helpers go in
  `framework` and are called as `::ferro::offload::*` from the macro.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Upsert on composite PK | Custom SQL `INSERT OR REPLACE` | `Entity::insert(...).on_conflict(OnConflict::columns([…]).update_columns([…]))` | Proven idiom from `apply_event`; portable across SQLite/Postgres. |
| Result deserialization | Custom JSON parsing | `serde_json::from_value::<T>(state)` after extracting the `"value"` field | `T: OffloadSerializable` guarantees round-trip. |
| Panic capture | Manual `std::panic::catch_unwind` in the macro-emitted body | The worker already wraps `handler(…).catch_unwind()` (worker.rs:449); panics arrive as `Err(_panic)` in the `result` match | Double-wrapping is unnecessary; extending `handle_failure` is the correct hook point. |

**Key insight:** The persistence and error-handling infrastructure already exists in
`ferro-projection` and `ferro-queue`. Phase 246 is almost entirely wiring, not new capability.

---

## Research Questions — Answered

### RQ1: Handle-key propagation — the load-bearing seam

**Verified state of `Offloadable::offload()`** (`ferro-queue/src/offload.rs:118`): the current
implementation mints the key AFTER dispatch and the key is lost immediately:

```rust
async fn offload(self) -> Result<OffloadHandle<Self::Output>, Error> {
    let key = HandleKey::new();
    crate::PendingDispatch::new(self).dispatch().await?;  // key NOT carried
    Ok(OffloadHandle::new(key))
}
```

**Recommended mechanism:** Add a `handle_key: Option<String>` field to `PendingDispatch` (same
pattern as the existing `tenant_id: Option<i64>`). Add a `handle_key` column to the `jobs` table
(nullable `TEXT`, default NULL — existing rows are unaffected). Add `handle_key: Option<String>`
to `JobRow`. Thread the value through `db::enqueue(…, handle_key: Option<&str>, …)`.

The `WorkerLoop::register::<J>` handler closure already receives `job_row.payload: String` and
`attempts: u32` as arguments; it does not currently receive `JobRow` directly. The fix is to pass
`job_row.handle_key.clone()` into the spawned task alongside `job_row.payload`.

**No change to `Job::handle(&self)` signature required.** The handle key is available in
`spawn_job` from `job_row.handle_key` and can be passed to `persist_result`/`persist_error`
directly after the handler future completes, bypassing `handle()` entirely for the write-back.

This approach needs a DB migration for the new `handle_key` column. **This is a schema change.**
The CONTEXT.md says "no new table, no new migration" for the ferro-projection side (D-02), but
is silent about the `jobs` table. Adding a nullable column to `jobs` is compatible with
existing rows and the `CreateJobsTable` migration is a separate, earlier migration.

**Alternative: encode the key in the Job's name or idempotency_key:** Prohibited by D-06.

**Alternative: pass key as part of the job-context `App` container:** Overly complex; the container
is per-request, not per-job-execution.

[VERIFIED: reading ferro-queue/src/offload.rs:118, dispatcher.rs:131-156, worker.rs:374-484]

### RQ2: Direct snapshot write/read API on `ferro-projection`

**Verified:** The `projection_snapshots` entity (entity.rs), the `CreateProjectionSnapshotsTable`
migration (re-exported from lib.rs), and `ProjectionKey` (key.rs) are all exactly as described in
CONTEXT.md. The upsert idiom in `apply_event` (runtime.rs:158) uses:

```rust
Entity::insert(am)
    .on_conflict(
        OnConflict::columns([Column::ProjectionName, Column::Key])
            .update_columns([Column::State, Column::Version, Column::UpdatedAt])
            .to_owned(),
    )
    .exec(&self.db)
    .await?;
```

The `read` pattern at runtime.rs:87 uses `Entity::find_by_id((name, key))`.

The `ferro-projection/src/lib.rs` currently exports: `ProjectionError`, `ProjectionKey`,
`CreateProjectionSnapshotsTable`, `Projection`, `ProjectionRuntime`, plus entity re-exports
(`ProjectionSnapshotActiveModel`, `ProjectionSnapshotEntity`, `ProjectionSnapshotModel`).

**Recommended surface:** add a new `direct.rs` module with two free functions
(`snapshot_write`, `snapshot_read`) and re-export them from `lib.rs`.

**Version behavior:** set `version = 1` on first write; the `OnConflict` update clause does NOT
update `version` (different from `apply_event` which increments it). For one-shot results,
overwriting with a fixed version is fine because there is one writer per handle key.

[VERIFIED: reading ferro-projection/src/runtime.rs, entity.rs, key.rs, lib.rs]

### RQ3: Value capture in the derived `handle()` arms

**Verified current macro code** (`ferro-macros/src/offload.rs:249-274`): four arms,
all via `.map(|_| ())`:

```rust
// async + Result:
svc.method(args).await.map(|_| ()).map_err(|e| Error::job_failed(…))
// async + non-Result:
let _ = svc.method(args).await; Ok(())
// sync + Result:
svc.method(args).map(|_| ()).map_err(…)
// sync + non-Result:
let _ = svc.method(args); Ok(())
```

**Required change:** capture the value before discarding and call a helper:

```rust
// async + Result (illustrative):
match svc.#method_ident( #(#field_args),* ).await {
    Ok(value) => {
        let key_str = self.__offload_handle_key.as_deref()
            .unwrap_or_default();
        ::ferro::offload::persist_result(
            key_str,
            &value,
            ::ferro::queue::Queue::connection(),
        ).await.ok(); // non-fatal: log on error, do not fail the job
        Ok(())
    }
    Err(e) => Err(::ferro::queue::Error::job_failed(#job_ident_str, format!("{e}"))),
}
```

**How the key reaches the emitted body:** the handle key is available in `JobRow.handle_key`,
not in `self` (the deserialized job). Two options: (a) add a `__offload_handle_key: Option<String>`
field to the derived Job struct (breaks D-05 — payload includes it); (b) receive the key as a
method argument — but `Job::handle(&self)` has a fixed signature. Option (c): the write-back
does NOT happen inside `Job::handle()` at all — it happens in `spawn_job` after the handler
future completes, using `job_row.handle_key` directly. This is the correct approach.

**Revised architecture for value capture:** the `handler` closure stored in `WorkerLoop` runs
`job.handle().await`; its return type is `(Result<(), Error>, Duration)`. To capture the
value and persist it, the handler closure must be extended to accept the handle key as a third
argument, or `spawn_job` calls a separate write-back step after the handler returns `Ok`.

The simplest approach: modify `spawn_job` to call `::ferro::offload::persist_result(key, raw_value, conn)` where `raw_value` comes from a parallel "value channel" that the derived `handle()` deposits the serialized result into before returning `Ok(())`. A `tokio::sync::oneshot` channel or a thread-local could be used, but those add complexity.

**Cleanest final approach:** extend the `JobHandler` type alias to return
`(Result<Option<serde_json::Value>, Error>, Duration)` — the `Some(value)` is the serialized
success value. The derived `handle()` body becomes:

```rust
// stored in a task-local OnceLock or returned directly from the handler closure
```

Actually, the cleanest approach given the current architecture is to serialize the return value
in the handler closure (inside the `WorkerLoop::register` registration closure) rather than in
`handle()`. The handler closure currently deserializes the job from the raw JSON payload; it
could also return the serialized output alongside the `Result<(), Error>`.

**Recommended plan decision:** Extend `JobHandler` return type to include
`Option<serde_json::Value>` (the serialized success value). The derived `handle()` no longer
discards via `.map(|_| ())`; instead the handler closure serializes `Ok(value)` → stores in the
return. `spawn_job` writes the result snapshot after `Ok(Some(v))`.

This keeps the handle key entirely outside `Job::handle()`'s signature.

[VERIFIED: reading ferro-macros/src/offload.rs, worker.rs:374-484]

### RQ4: Terminal error via `Job::failed()` — critical gap

**Critical finding:** `Job::failed()` is **NOT called** by `WorkerLoop::spawn_job` in the real
async DB path. The `handle_failure` function at worker.rs:488-511 only calls `db::fail_job` (SQL
update to `status='failed'`). `Job::failed()` is called ONLY in `dispatcher.rs:124` inside
`dispatch_immediately()` — the sync mode path. This gap means D-10 as stated ("override `failed()`
in the macro") will not fire in production.

**Evidence:**

```rust
// worker.rs:488 — handle_failure DOES NOT call Job::failed():
async fn handle_failure(conn, job_id, attempts, max_retries, err_msg, retry_delay) {
    if attempts + 1 >= max_retries {
        crate::db::fail_job(conn, job_id, err_msg).await.ok();  // no Job::failed() call
    } else {
        crate::db::release_job(conn, job_id, attempts + 1, available_at).await.ok();
    }
}

// dispatcher.rs:124 — only sync mode calls Job::failed():
async fn dispatch_immediately(self) -> Result<(), Error> {
    match self.job.handle().await {
        Ok(()) => Ok(()),
        Err(e) => {
            self.job.failed(&e).await;  // only here
            Err(e)
        }
    }
}
```

**Options for the plan:**

A. **Extend `WorkerLoop::handle_failure` to call `Job::failed(&err)` on the deserialized job.**
   Requires passing the deserialized `J` instance (or a boxed `dyn Job`) through the failure
   path, which the current code does not do (it only passes `job_id`, `err_msg`, `attempts`).
   The handler closure captures `J` by value before executing `handle()`; after a panic it is
   gone, so `failed()` cannot be called after a panic anyway (consistent with the current panic
   arm at worker.rs:474). For the non-panic `Err` arm, passing the deserialized job to
   `handle_failure` is feasible.

B. **Write the terminal-error snapshot from `handle_failure` using the job_id + handle_key
   passed in, WITHOUT calling `Job::failed()`.** The handler closure already knows `job_row.handle_key`;
   it can call `::ferro::offload::persist_error(key, err_msg, conn)` directly when
   `attempts + 1 >= max_retries`, bypassing `Job::failed()` entirely. This is simpler and does not
   require changing `handle_failure`'s signature.

C. **Extend `handle_failure` to accept an optional terminal-error callback closure** that fires
   only when retries are exhausted.

**Recommended approach (Option B):** persist the terminal-error envelope from `spawn_job`
directly, not from `Job::failed()`. This avoids changing `handle_failure`'s signature and does
not require passing a deserialized `J` through the failure path. The `Job::failed()` method
on the derived struct can still be overridden by the macro for correctness in sync-mode tests,
but the production path goes through `spawn_job`.

Panic case: `job_row.handle_key` is available before the panic; `spawn_job` can call
`persist_error` in the panic arm at worker.rs:474 with the key and "job handler panicked".

[VERIFIED: reading worker.rs:452-511, dispatcher.rs:117-128]

### RQ5: Result envelope and read-back wrapper

**Serde representation:** an internally-tagged enum works cleanly:

```rust
#[serde(tag = "status", rename_all = "snake_case")]
enum OffloadResult<T> {
    Completed { value: T },
    Failed { error: String },
}
```

This serializes as `{"status":"completed","value":{…}}` / `{"status":"failed","error":"…"}`,
matching the D-07 spec. The internally-tagged form is unambiguous and standard in serde.

**Read-back helper:**

```rust
pub async fn read_result<T: OffloadSerializable>(
    handle: &OffloadHandle<T>,
    db: &DatabaseConnection,
) -> Result<Option<OffloadResult<T>>, SnapshotError> {
    let key = ProjectionKey::new(handle.key());
    let state = snapshot_read(db, OFFLOAD_PROJECTION_NAME, &key).await?;
    match state {
        None => Ok(None),
        Some(v) => {
            let envelope: OffloadResult<T> = serde_json::from_value(v)?;
            Ok(Some(envelope))
        }
    }
}
```

Returning `None` when the row is absent (D-08) is the natural behavior of `snapshot_read`.

[VERIFIED: ferro-projection/src/runtime.rs:87, entity.rs model shape, 246-CONTEXT.md D-07/D-08]

### RQ6: Crate dependency direction

**Verified:**

- `framework/Cargo.toml` currently lists `ferro-projections` (plural, optional under `projections`
  feature) but does NOT list `ferro-projection` (singular). Adding `ferro-projection` as a new
  dependency is required (D-11). [VERIFIED: framework/Cargo.toml:53]
- `ferro-projection/Cargo.toml` depends on `ferro-events` and `ferro-broadcast` only — it does
  NOT depend on `framework`. No cycle. [VERIFIED: ferro-projection/Cargo.toml]
- `ferro-queue/Cargo.toml` must NOT gain `ferro-projection` as a dependency (D-11). The write-back
  helpers live in `framework` only.
- `framework` already depends on `ferro-queue` and `ferro-broadcast` and `ferro-events` — adding
  `ferro-projection` adds no new transitive cycle risk.
- `ferro-projection` should be an **always-on** dependency in `framework` (not optional), since
  the offload result path is part of the core queue substrate that ships with the framework.
  Alternatively it can be feature-gated behind a new `offload-projection` feature, but given
  that `ferro-queue` is already always-on, this seems unnecessary.

**Emit convention check:** `ferro-macros/src/offload.rs` already emits only `::ferro::*` paths
(e.g. `::ferro::queue::Job`, `::ferro::queue::Error::job_failed`, `::ferro::queue::OffloadSerializable`,
`::ferro::App::make`, `::ferro::async_trait`, `::ferro::inventory::submit!`). New paths must follow
the same convention: `::ferro::offload::persist_result`, `::ferro::offload::persist_error`.

[VERIFIED: framework/Cargo.toml, ferro-projection/Cargo.toml, ferro-macros/src/offload.rs]

### RQ7: Reserved projection name and collision risk

`"offload.result"` is a new projection name not present in any existing code. The naming
convention in the codebase uses `"test.counter"`, `"test.keyed_counter"`, `"inventory.dashboard"`
patterns — dotted namespaces. `"offload.result"` fits the convention and is unlikely to collide
with application-defined projections, which should use domain nouns (the `feedback_catalog_vocabulary_structural_nouns`
rule in CLAUDE.md applies here: `ferro-*` crates use structural nouns). The 247 broadcast channel
derives as `projection.offload.result.{handle}`, which matches the `projection.{NAME}.{key}` template
used by `apply_event` (runtime.rs:168).

[VERIFIED: runtime.rs:168 channel naming, lib.rs exports]

### RQ8: Testing approach

**Existing patterns:**

- `ferro-queue/tests/offload_round_trip.rs`: uses `QUEUE_CONNECTION=sync` — no DB needed. Suitable
  for unit-level value-capture and `failed()` tests, but does NOT exercise the real `WorkerLoop`
  claim/exec path where the terminal-error write-back lives.
- `ferro-projection/src/runtime.rs` (inline tests): uses `sqlite::memory:` with a fresh DB + migration
  inline. Provides the pattern for testing the direct write/read API in isolation.

**Recommended test structure for this phase:**

| Seam | Test Type | Strategy |
|------|-----------|----------|
| `snapshot_write` + `snapshot_read` round-trip | Unit (in ferro-projection/src/direct.rs) | `sqlite::memory:` + `TestMigrator` inline; write completed envelope, read back and deserialize |
| `snapshot_write` idempotency (overwrite with same key) | Unit (same file) | Write twice; read back = second value |
| Success path round-trip (enqueue → worker drains → snapshot retrievable) | Integration (ferro-queue/tests/) | `sqlite::memory:` DB with both `CreateJobsTable` + `CreateProjectionSnapshotsTable` migrations; use `WorkerLoop` drain in test |
| Terminal-error path (method returns Err, retries exhausted) | Integration | Same harness; trigger failure by returning `Err` from `handle()` until `max_retries` exhausted |
| Panic path (method panics → failed envelope) | Integration | Same harness; trigger panic in `handle()` |
| No-row = not done yet | Unit | Read before write; assert `None` |

**Migration wiring in integration tests:** the test DB must run BOTH migrations. The `CreateJobsTable`
migration is in `ferro-queue`; `CreateProjectionSnapshotsTable` is in `ferro-projection`. An
integration test in `ferro-queue/tests/` would need to import `ferro-projection` — which would
create a direct dev-dependency from `ferro-queue/tests` to `ferro-projection`. This is acceptable
(dev-dependencies don't affect the crate's published dep graph). Alternatively, integration tests
for the full round-trip can live in `framework/tests/` since framework already depends on both.

[VERIFIED: offload_round_trip.rs, runtime.rs inline tests]

---

## Common Pitfalls

### Pitfall 1: `Job::failed()` not called in the real async worker path

**What goes wrong:** The plan assumes overriding `Job::failed()` in the macro is sufficient to
write the terminal-error snapshot. In sync mode this works; in async mode it does not fire.

**Why it happens:** `WorkerLoop::handle_failure` (worker.rs:488) calls `db::fail_job` directly
without deserializing the job or calling `Job::failed()`. The `dispatch_immediately` sync path
is the only place `failed()` is called.

**How to avoid:** Write the terminal-error snapshot from `spawn_job`, not from `failed()`. The
handle key is available in `job_row.handle_key` before the panic, so both the `Err` arm and the
panic arm can call `::ferro::offload::persist_error(key, msg, conn)` directly.

**Warning signs:** Tests running in `QUEUE_CONNECTION=sync` pass; real-worker integration tests
show no `{"status":"failed"}` row written after retries exhausted.

### Pitfall 2: `handle_key` column migration must be idempotent / backward-compatible

**What goes wrong:** Adding a `handle_key TEXT NULL` column to the `jobs` table in a migration
that already ran breaks existing deployed apps if the migration is applied twice or if the column
already exists.

**Why it happens:** SeaORM migrations are applied once; running `up()` twice on an existing DB
raises an error unless guarded. The column must be nullable (existing rows have no handle key).

**How to avoid:** The migration adds `handle_key TEXT NULL DEFAULT NULL`. Parse it in
`parse_job_row` with `try_get_by::<Option<String>, _>("handle_key")`. Offload-derived jobs that
lack a handle key (unlikely, but possible for manually dispatched jobs) produce `handle_key = None`
and the write-back is a no-op.

### Pitfall 3: `serde_json::to_value` for non-unit `()` output type

**What goes wrong:** For a `-> ()` return (`Output = ()`), `serde_json::to_value(())` yields
`serde_json::Value::Null`. Storing `{"status":"completed","value":null}` is valid but must
deserialize back to `()`.

**Why it happens:** `()` serializes as JSON `null` in serde.

**How to avoid:** This is actually correct: `serde_json::from_value::<()>(Value::Null)` succeeds.
No special case needed. Verify with a unit test.

### Pitfall 4: `snapshot_write` from `spawn_job` races a concurrent second call for the same handle

**What goes wrong:** Two workers race to write the same handle key's result (e.g., a job that was
re-claimed after a timeout while the first worker actually completed it).

**Why it happens:** A visibility-timeout reaper can re-expose a job to a second worker if the
first worker is slow. Both workers can reach the success path and both call `persist_result`.

**How to avoid:** The `OnConflict` upsert is last-writer-wins — both writes succeed (no error);
the final state is the last writer's result. Since both workers ran the same method, the outputs
should be identical (assuming deterministic `Output`). For non-deterministic outputs, this is an
accepted tradeoff (labeled in `ferro-projection/src/lib.rs` as the single-instance footgun).

### Pitfall 5: `persist_result` failure silently suppresses the success path

**What goes wrong:** `persist_result` fails (DB error); the returned error causes `spawn_job` to
treat the job as failed and schedule a retry, even though the method succeeded.

**How to avoid:** Call `persist_result(…).await.ok()` (ignore the error with a `tracing::warn!`)
and still return `Ok(())` from the job. This is consistent with `apply_event`'s step 6 broadcast
failure behavior: the broadcast failure does not roll back the state (D-21 in ferro-projection).
Log the failure; the row is absent rather than stale.

### Pitfall 6: Cargo cycle via test dev-dependency

**What goes wrong:** Integration tests in `ferro-queue/tests/` import `ferro-projection` as a
dev-dependency. Cargo resolves dev-dependencies in the same dependency graph, so a circular
dev-dep can silently compile if the cycle only involves dev code, but Cargo will still error if
the production dependency graph has a cycle.

**How to avoid:** `ferro-projection` does NOT depend on `ferro-queue` in production. The
dev-dependency direction (`ferro-queue` test → `ferro-projection`) is acyclic. Cargo allows this.
However, the cleanest choice is to put the round-trip integration test in `framework/tests/` or
in a separate integration test crate.

---

## Code Examples

### Write-back helper (framework/src/offload.rs sketch)

```rust
// Source: synthesized from runtime.rs:158 OnConflict idiom + entity.rs model shape
use ferro_projection::{ProjectionKey, ProjectionSnapshotActiveModel, ProjectionSnapshotEntity};
use ferro_queue::OffloadSerializable;
use sea_orm::{sea_query::OnConflict, ActiveValue, DatabaseConnection, EntityTrait};
use serde_json::Value as JsonValue;

pub const OFFLOAD_PROJECTION_NAME: &str = "offload.result";

pub async fn persist_result<T: OffloadSerializable>(
    handle_key: &str,
    value: &T,
    db: &DatabaseConnection,
) -> Result<(), crate::OffloadError> {
    let state = serde_json::to_value(value)?;
    let envelope = serde_json::json!({"status": "completed", "value": state});
    write_envelope(handle_key, envelope, db).await
}

pub async fn persist_error(
    handle_key: &str,
    error: &str,
    db: &DatabaseConnection,
) -> Result<(), crate::OffloadError> {
    let envelope = serde_json::json!({"status": "failed", "error": error});
    write_envelope(handle_key, envelope, db).await
}

async fn write_envelope(
    handle_key: &str,
    envelope: JsonValue,
    db: &DatabaseConnection,
) -> Result<(), crate::OffloadError> {
    use ferro_projection::entity::{Column, Entity};
    let now = chrono::Utc::now().naive_utc();
    let am = ferro_projection::ProjectionSnapshotActiveModel {
        projection_name: ActiveValue::Set(OFFLOAD_PROJECTION_NAME.to_string()),
        key: ActiveValue::Set(handle_key.to_string()),
        state: ActiveValue::Set(envelope),
        version: ActiveValue::Set(1),
        updated_at: ActiveValue::Set(now),
    };
    Entity::insert(am)
        .on_conflict(
            OnConflict::columns([Column::ProjectionName, Column::Key])
                .update_columns([Column::State, Column::UpdatedAt])
                .to_owned(),
        )
        .exec(db)
        .await?;
    Ok(())
}
```

Note: `ferro-projection` already re-exports `ProjectionSnapshotActiveModel` and
`ProjectionSnapshotEntity` from `lib.rs:90-93`. However `entity::Column` is not re-exported.
The plan must either re-export `Column` from `ferro-projection/src/lib.rs`, or add the direct
API to `ferro-projection/src/direct.rs` and call it from `framework` via the public function.
The latter is cleaner (avoids leaking internal SeaORM column types through the public API).

### In-worker success write-back (in `spawn_job`)

```rust
// After the handler future returns Ok((Ok(()), _)):
if let Some(ref key) = job_row.handle_key {
    // persist_result receives the serialized Value from the handler closure
    // (requires JobHandler to return Option<serde_json::Value> alongside Result<(), Error>)
    if let Some(serialized_value) = success_value {
        if let Err(e) = ::ferro::offload::persist_result_raw(key, serialized_value, conn).await {
            tracing::warn!(job_id = %job_id, error = %e, "offload result persist failed — result not stored");
        }
    }
}
crate::db::delete_job(conn, job_id).await.ok();
```

### In-worker terminal-error write-back (in `spawn_job`)

```rust
// In handle_failure when attempts + 1 >= max_retries, before db::fail_job:
if let Some(ref key) = handle_key {
    if let Err(e) = ::ferro::offload::persist_error(key, err_msg, conn).await {
        tracing::warn!(job_id = %job_id, error = %e, "offload error persist failed");
    }
}
crate::db::fail_job(conn, job_id, err_msg).await.ok();
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `Offloadable::offload()` mints key after dispatch, discards it | Mint before dispatch, carry via `PendingDispatch.handle_key` | Phase 246 | Caller's `OffloadHandle.key()` equals the worker's write key |
| `handle()` discards return value via `.map(\|_\| ())` | `handle()` captures value, worker persists snapshot after `Ok` | Phase 246 | OFFLOAD-03 SC#1 |
| No terminal-error row | `handle_failure` writes `{"status":"failed"}` snapshot when retries exhausted | Phase 246 | OFFLOAD-03 SC#3 |

**Deprecated/outdated:**

- The current `Offloadable::offload()` default implementation (offload.rs:118-122): replaced by the
  mint-before-dispatch version in this phase.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Adding `handle_key TEXT NULL` to the `jobs` table is a backward-compatible migration (existing rows get NULL; `parse_job_row` handles `Option<String>`) | RQ1 | Schema migration fails or existing rows are corrupted — low risk given nullable column convention |
| A2 | `ferro-projection` as an always-on (non-feature-gated) dependency of `framework` is acceptable | RQ6 | May pull `ferro-broadcast` + `ferro-events` into consumers that don't want them — investigate if a feature gate is preferred |
| A3 | `serde_json::to_value(())` yielding `null` round-trips correctly to `()` via `serde_json::from_value` | RQ5 | `OffloadResult<()>` fails to deserialize — verify with a unit test in Wave 0 |
| A4 | `version = 1` (fixed, not incremented) on `snapshot_write` is acceptable for offload results | RQ2 | Downstream Phase 247 reads `version` for delta comparison — confirm 247's requirements before locking |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation
needed. The table is not empty: A2 and A4 require planner confirmation.

---

## Open Questions

1. **`ferro-projection` as always-on vs. feature-gated dependency in `framework`**
   - What we know: `ferro-projection` depends on `ferro-broadcast` + `ferro-events`. `framework`
     already depends on both. Adding `ferro-projection` is not a cycle. But making it optional
     (e.g. `#[cfg(feature = "projection")]`) preserves the existing layering discipline.
   - What's unclear: whether downstream apps that pull `ferro-rs` without `projection` features
     should be forced to carry `ferro-projection`.
   - Recommendation: make it always-on (no feature gate) since `ferro-queue` is already always-on
     and the result path is part of the queue's core contract.

2. **`jobs` table migration: new migration file vs. amendment**
   - What we know: `CreateJobsTable` migration already exists. Adding a column requires either
     a new `AddHandleKeyToJobs` migration or amending the original (only safe before first deploy).
   - What's unclear: whether the `jobs` table has ever shipped to a production environment. Given
     that ferro is not yet in production (MEMORY.md), amending the original is safe.
   - Recommendation: amend the original `CreateJobsTable` migration — simpler, no migration chain.

3. **`JobHandler` return type extension impact on existing Job impls**
   - What we know: changing `JobHandler` from returning `(Result<(), Error>, Duration)` to
     `(Result<Option<serde_json::Value>, Error>, Duration)` affects ALL registered handlers in
     `WorkerLoop::register::<J>()`.
   - What's unclear: whether there are external consumers of the `register` API that would break.
     Given ferro is not yet in production, this is safe.
   - Recommendation: extend the return type; the change is internal to `ferro-queue`.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test + tokio::test |
| Config file | none |
| Quick run command | `cargo test -p ferro-projection -p ferro-queue direct snapshot offload` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| OFFLOAD-03-SC1 | Worker persists completed envelope after `handle()` returns `Ok` | Integration | `cargo test -p ferro-queue offload_result_round_trip` | ❌ Wave 0 |
| OFFLOAD-03-SC2 | Result is retrievable by handle key after worker completes | Integration | `cargo test -p ferro-queue retrieve_by_handle_after_complete` | ❌ Wave 0 |
| OFFLOAD-03-SC3a | Terminal-error envelope written when `handle()` returns `Err` and retries exhausted | Integration | `cargo test -p ferro-queue offload_terminal_error_on_err` | ❌ Wave 0 |
| OFFLOAD-03-SC3b | Terminal-error envelope written when `handle()` panics and retries exhausted | Integration | `cargo test -p ferro-queue offload_terminal_error_on_panic` | ❌ Wave 0 |
| OFFLOAD-03-SC3c | No snapshot row exists before worker runs (no silent pending row) | Unit | `cargo test -p ferro-projection snapshot_read_returns_none_for_absent` | ✅ (existing `read_returns_none_for_absent_key`) |
| OFFLOAD-03-direct | `snapshot_write` + `snapshot_read` round-trip with completed envelope | Unit | `cargo test -p ferro-projection direct_snapshot_round_trip` | ❌ Wave 0 |
| OFFLOAD-03-direct | `snapshot_write` idempotency (overwrite is safe) | Unit | `cargo test -p ferro-projection direct_snapshot_overwrite` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-projection && cargo test -p ferro-queue`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-projection/src/direct.rs` — `snapshot_write` + `snapshot_read` + unit tests (covers `OFFLOAD-03-direct` rows)
- [ ] `ferro-queue/tests/offload_result_round_trip.rs` — integration harness with both migrations, WorkerLoop drain, and all three success/failure/panic seams (covers `OFFLOAD-03-SC1` through `SC3b`)
- [ ] `framework/src/offload.rs` — `persist_result`, `persist_error`, `read_result`, `OffloadResult<T>` (needed by the integration test via `::ferro::offload::*`)

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | yes (handle-key disclosure) | Handle key is a UUID v4 — non-guessable by brute force but NOT cryptographically access-controlled; Phase 247 subscribe semantics must add access control |
| V5 Input Validation | yes | `serde_json::from_value` for the output value — bounds enforced by `OffloadSerializable` |
| V6 Cryptography | no | UUID v4 for handle key is sufficient for uniqueness; not a secret |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Unbounded snapshot growth (no TTL) | Denial of service | Noted as deferred (CONTEXT.md Deferred); the planner should add a monitoring note — no row is ever deleted from `projection_snapshots` for offload results in Phase 246 |
| Error string leaking internal details into the failed envelope | Information disclosure | `format!("{e}")` for the error message (244 D-07) — the `Display` of a framework error may include internal DB error strings; consider truncating or sanitizing the error message before storing in the envelope |
| Handle key guessability → result disclosure | Elevation of privilege | UUID v4 is 122 bits of entropy, effectively non-guessable; this is the accepted stance in Phase 245 D-07. Phase 247 must add proper subscriber authentication. |
| Payload deserialization of untrusted job row | Tampering | The `jobs` table is an internal DB table; no external input reaches `parse_job_row`. Standard SQL injection risk is already mitigated by `Statement::from_sql_and_values` (db.rs:1). |

---

## Environment Availability

Step 2.6: SKIPPED — Phase 246 is a pure code/config change. No new external dependencies.
All runtime services (SQLite/Postgres, the queue, the projection table) are already established
by the predecessor phases. Integration tests use `sqlite::memory:`.

---

## Sources

### Primary (HIGH confidence)

- [VERIFIED: ferro-queue/src/offload.rs] — `Offloadable`, `HandleKey`, `OffloadHandle<T>`, current `offload()` default body
- [VERIFIED: ferro-macros/src/offload.rs] — `emit_job_items`, four `handle()` arms, `::ferro::*` emit convention
- [VERIFIED: ferro-queue/src/worker.rs] — `WorkerLoop::spawn_job`, `handle_failure`, absence of `Job::failed()` call in async path
- [VERIFIED: ferro-queue/src/dispatcher.rs] — `dispatch_immediately()`, presence of `Job::failed()` call in sync path
- [VERIFIED: ferro-queue/src/job.rs] — `Job::failed(&self, error: &Error)` signature, async trait method
- [VERIFIED: ferro-queue/src/db.rs] — `JobRow` struct, `fail_job`, `Queue::connection()`, `JobRegistrarEntry`
- [VERIFIED: ferro-projection/src/runtime.rs] — `apply_event` upsert idiom, `read` pattern
- [VERIFIED: ferro-projection/src/entity.rs] — `projection_snapshots` model shape, composite PK
- [VERIFIED: ferro-projection/src/key.rs] — `ProjectionKey` newtype
- [VERIFIED: ferro-projection/src/lib.rs] — public re-exports, entity re-exports
- [VERIFIED: ferro-projection/Cargo.toml] — depends on ferro-events + ferro-broadcast; NOT on framework or ferro-queue
- [VERIFIED: framework/Cargo.toml] — `ferro-projections` (plural, optional) present; `ferro-projection` (singular) absent
- [VERIFIED: framework/src/lib.rs:224-232] — existing `pub mod queue` re-export block including `OffloadHandle`, `Offloadable`, etc.
- [VERIFIED: ferro-queue/tests/offload_round_trip.rs] — 244/245 test pattern with `sqlite::memory:` + `QUEUE_CONNECTION=sync`

### Secondary (MEDIUM confidence)

- [CITED: 246-CONTEXT.md D-01..D-13] — all locked decisions confirmed against actual source code

### Tertiary (LOW confidence)

- [ASSUMED] A3: `serde_json::from_value::<()>(Value::Null)` succeeds — standard serde behavior but not explicitly tested in this codebase
- [ASSUMED] A4: `version = 1` fixed on snapshot_write is acceptable to Phase 247 — depends on 247's read-model delta requirements

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries verified from source
- Architecture: HIGH — patterns verified from actual code; critical gap (Job::failed not called) verified
- Pitfalls: HIGH — gap at Pitfall 1 verified directly from worker.rs source

**Research date:** 2026-08-13
**Valid until:** 2026-09-13 (stable codebase; valid until next phase execution)
