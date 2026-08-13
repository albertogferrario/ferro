# Phase 246: Result → read-model snapshot - Pattern Map

**Mapped:** 2026-08-13
**Files analyzed:** 8 (3 new, 5 modified)
**Analogs found:** 8 / 8

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-projection/src/direct.rs` | utility (persistence) | CRUD | `ferro-projection/src/runtime.rs` (apply_event + read) | exact: same entity, same SeaORM idioms |
| `ferro-projection/src/lib.rs` | config (re-exports) | — | itself (existing re-export block L83–93) | exact |
| `framework/src/offload.rs` | service (facade + envelope) | request-response | `framework/src/lib.rs` pub mod queue block (L224–232) | role-match |
| `framework/src/lib.rs` | config (re-exports) | — | itself (pub mod queue block L224–232) | exact |
| `framework/Cargo.toml` | config | — | itself (L53 `ferro-projections` optional dep line) | exact |
| `ferro-queue/src/offload.rs` | utility (enqueue) | request-response | itself (L118–122 `offload()` default) | exact |
| `ferro-queue/src/dispatcher.rs` | utility (dispatch) | request-response | itself (`PendingDispatch` struct + `for_tenant` + `captured_tenant_id`) | exact |
| `ferro-queue/src/db.rs` | utility (persistence) | CRUD | itself (`JobRow` struct + `parse_job_row` + `enqueue` both arms) | exact |
| `ferro-queue/src/worker.rs` | service (execution) | event-driven | itself (`spawn_job` L374–484 + `handle_failure` L488–512) | exact |
| `ferro-macros/src/offload.rs` | utility (codegen) | transform | itself (`emit_job_items` four `call_expr` arms L249–274) | exact |
| `ferro-queue/tests/offload_result_round_trip.rs` | test | CRUD + event-driven | `ferro-queue/tests/offload_round_trip.rs` + `ferro-projection/src/runtime.rs` inline tests | role-match |

---

## Pattern Assignments

### `ferro-projection/src/direct.rs` (new — utility, CRUD)

**Analog:** `ferro-projection/src/runtime.rs`

**Imports pattern** (runtime.rs L24–33 — copy these imports verbatim):
```rust
use chrono::Utc;
use sea_orm::{sea_query::OnConflict, ActiveValue, DatabaseConnection, EntityTrait};

use crate::entity::{ActiveModel, Column, Entity};
use crate::error::ProjectionError;
use crate::key::ProjectionKey;
```

**Core upsert pattern** (runtime.rs L147–165 — the `apply_event` step 5 block to mirror):
```rust
let now = Utc::now().naive_utc();
let am = ActiveModel {
    projection_name: ActiveValue::Set(P::NAME.to_string()),
    key: ActiveValue::Set(key.0.clone()),
    state: ActiveValue::Set(state_json.clone()),
    version: ActiveValue::Set(new_version),
    updated_at: ActiveValue::Set(now),
};

Entity::insert(am)
    .on_conflict(
        OnConflict::columns([Column::ProjectionName, Column::Key])
            .update_columns([Column::State, Column::Version, Column::UpdatedAt])
            .to_owned(),
    )
    .exec(&self.db)
    .await?;
```

**Adaptation for `snapshot_write`:** replace `ActiveValue::Set(P::NAME.to_string())` with `ActiveValue::Set(name.to_string())`, replace `ActiveValue::Set(new_version)` with `ActiveValue::Set(1_i64)` (fixed — no version increment for one-shot results, per D-02 and RESEARCH §Pattern 1), and drop `Column::Version` from `update_columns` so repeat writes do not overwrite version. Function signature takes `db: &DatabaseConnection, name: &str, key: &ProjectionKey, state: serde_json::Value`.

**Core read pattern** (runtime.rs L87–98 — the `read` method body):
```rust
let row = Entity::find_by_id((P::NAME.to_string(), key.0.clone()))
    .one(&self.db)
    .await?;
match row {
    None => Ok(None),
    Some(model) => {
        let state: P::State = serde_json::from_value(model.state)?;
        Ok(Some(state))
    }
}
```

**Adaptation for `snapshot_read`:** replace `P::NAME.to_string()` with `name.to_string()`, return `Ok(row.map(|m| m.state))` (return raw `JsonValue`, not typed `P::State` — callers deserialize). Signature: `db: &DatabaseConnection, name: &str, key: &ProjectionKey) -> Result<Option<serde_json::Value>, ProjectionError>`.

**Test pattern** (runtime.rs L378–391 — `TestMigrator` + `fresh_runtime` setup to mirror in unit tests):
```rust
struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![Box::new(crate::migration::Migration)]
    }
}

async fn fresh_db() -> sea_orm::DatabaseConnection {
    let conn = Database::connect("sqlite::memory:").await.expect("connect");
    TestMigrator::up(&conn, None).await.expect("migrate");
    conn
}
```
Unit tests for `direct.rs` go in `#[cfg(test)] mod tests` at the bottom of `direct.rs`, following the same pattern as `entity.rs` L46–132.

---

### `ferro-projection/src/lib.rs` (modified — re-exports)

**Analog:** itself, current re-export block (L75–93).

**Current public surface** (lib.rs L75–93 — add `direct` alongside existing modules):
```rust
mod entity;
mod error;
mod key;
mod listener;
mod migration;
mod projection;
mod runtime;

pub use error::ProjectionError;
pub use key::ProjectionKey;
pub use migration::Migration as CreateProjectionSnapshotsTable;
pub use projection::Projection;
pub use runtime::ProjectionRuntime;

// SeaORM entity re-exports for consumers needing native SeaORM query access.
pub use entity::{
    ActiveModel as ProjectionSnapshotActiveModel, Entity as ProjectionSnapshotEntity,
    Model as ProjectionSnapshotModel,
};
```

**Addition:** add `mod direct;` in the module block and `pub use direct::{snapshot_read, snapshot_write};` in the re-export block. Note: `entity::Column` is NOT currently re-exported — `direct.rs` uses `crate::entity::{Column, Entity}` internally (private import) and exposes only the two free functions, keeping `Column` private. This matches the existing convention in `runtime.rs` which also imports `Column` privately.

---

### `framework/src/offload.rs` (new — service/facade, request-response)

**Analog:** `framework/src/lib.rs` pub mod queue block (L224–232); `ferro-projection/src/runtime.rs` upsert pattern.

**Imports pattern** (synthesized from the queue block convention + projection entity shape):
```rust
use ferro_projection::{
    ProjectionKey, ProjectionSnapshotActiveModel, ProjectionSnapshotEntity,
};
use ferro_queue::OffloadSerializable;
use sea_orm::{sea_query::OnConflict, ActiveValue, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
```

Note: use `ferro_projection::snapshot_write` and `ferro_projection::snapshot_read` (the direct API added in `direct.rs`) rather than calling SeaORM directly here. This keeps the framework facade thin — it composes the `direct.rs` API.

**Constant and result enum pattern** (from RESEARCH §Pattern 2, D-07, D-13):
```rust
pub const OFFLOAD_PROJECTION_NAME: &str = "offload.result";

/// Typed result wrapper for a completed or terminally failed offloaded call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum OffloadResult<T> {
    Completed { value: T },
    Failed { error: String },
}
```
The `serde(tag = "status", rename_all = "snake_case")` pattern is standard in this codebase — see `JobState` in `ferro-queue/src/db.rs` L145–154 for the same `#[serde(rename_all = "snake_case")]` convention. The internally-tagged form matches D-07's `{"status":"completed","value":...}` envelope.

**persist_result / persist_error / read_result pattern** — call `ferro_projection::snapshot_write` / `ferro_projection::snapshot_read` internally; see RESEARCH §Code Examples (L722–775) for the concrete body. DB source is `ferro_queue::db::Queue::connection()` (worker.rs L247, always `&'static DatabaseConnection`).

**Error handling pattern:** call `persist_result(...).await.ok()` at the call site (non-fatal — log with `tracing::warn!`, do not fail the job). This mirrors `apply_event` step 6 broadcast failure behavior in runtime.rs L190–197: broadcast failure does not roll back state, it just logs. Same principle applies to snapshot persistence failure.

---

### `framework/src/lib.rs` (modified — re-exports)

**Analog:** itself, pub mod queue block (L224–232).

**Current offload re-export block** (lib.rs L224–232):
```rust
pub mod queue {
    pub use ferro_queue::{
        dispatch, dispatch_later, dispatch_to, register_tenant_capture_hook, CreateJobsTable,
        Error, FailedJobInfo, HandleKey, Job, JobInfo, JobPayload, JobRegistrarEntry, JobState,
        OffloadHandle, OffloadSerializable, Offloadable, PendingDispatch, Queue, QueueConfig,
        QueueStats, Queueable, SingleQueueStats, TenantScopeProvider, Worker, WorkerConfig,
        WorkerLoop,
    };
}
```

**Addition:** add `pub mod offload;` as a new top-level module (not nested under `queue`) and populate it in `framework/src/offload.rs`. Then `::ferro::offload::persist_result`, `::ferro::offload::persist_error`, `::ferro::offload::read_result`, and `::ferro::offload::OffloadResult` are all callable from macro-generated code without the `queue::` namespace — consistent with D-11's stated paths (`::ferro::offload::persist_result`).

---

### `framework/Cargo.toml` (modified — config)

**Analog:** itself, L53 `ferro-projections` optional dependency line.

**Current line** (Cargo.toml L53):
```toml
ferro-projections = { path = "../ferro-projections", version = "0.3", optional = true }
```

**Addition:** add `ferro-projection` (singular) as an **always-on** (non-optional) dependency:
```toml
ferro-projection = { path = "../ferro-projection", version = "0.3" }
```
Always-on because `ferro-queue` is already always-on and the offload result path is part of the core queue substrate (RESEARCH RQ6 recommendation). No feature gate needed. Verify there is no transitive cycle: `ferro-projection` depends on `ferro-events` + `ferro-broadcast`; `framework` already depends on both (lib.rs L244–248); no new cycle.

---

### `ferro-queue/src/offload.rs` (modified — utility, request-response)

**Analog:** itself, `offload()` default implementation (offload.rs L118–122).

**Current body** (offload.rs L118–122):
```rust
async fn offload(self) -> Result<OffloadHandle<Self::Output>, Error> {
    let key = HandleKey::new();
    crate::PendingDispatch::new(self).dispatch().await?;
    Ok(OffloadHandle::new(key))
}
```

**Required change (D-04):** mint key before dispatch, carry it via `with_handle_key()`:
```rust
async fn offload(self) -> Result<OffloadHandle<Self::Output>, Error> {
    let key = HandleKey::new();
    crate::PendingDispatch::new(self)
        .with_handle_key(key.as_str().to_string())
        .dispatch()
        .await?;
    Ok(OffloadHandle::new(key))
}
```
The `with_handle_key` method is new — see `ferro-queue/src/dispatcher.rs` pattern below.

---

### `ferro-queue/src/dispatcher.rs` (modified — utility, request-response)

**Analog:** itself — `PendingDispatch` struct (L26–32), `for_tenant` method (L65–68), `captured_tenant_id` (L73–76), and `dispatch_to_queue` (L131–156). The `handle_key` propagation is a complete mirror of the `tenant_id` path.

**`tenant_id` in `PendingDispatch` struct** (dispatcher.rs L26–32 — the field to mirror):
```rust
pub struct PendingDispatch<J> {
    job: J,
    queue: Option<&'static str>,
    delay: Option<Duration>,
    /// Explicit tenant ID override. When set, takes precedence over the auto-capture hook.
    tenant_id: Option<i64>,
}
```
Add `handle_key: Option<String>` field alongside `tenant_id: Option<i64>`.

**`for_tenant` builder method** (dispatcher.rs L65–68 — the `with_handle_key` method mirrors this exactly):
```rust
pub fn for_tenant(mut self, tenant_id: i64) -> Self {
    self.tenant_id = Some(tenant_id);
    self
}
```
New method:
```rust
pub fn with_handle_key(mut self, key: String) -> Self {
    self.handle_key = Some(key);
    self
}
```

**`dispatch_to_queue` enqueue call** (dispatcher.rs L131–156 — add `handle_key` as the final argument after `tenant_id`):
```rust
crate::db::enqueue(
    conn,
    queue,
    &job_type,
    &payload,
    max_retries,
    idempotency_key.as_deref(),
    tenant_id,
    available_at,
)
.await
```
Becomes:
```rust
crate::db::enqueue(
    conn,
    queue,
    &job_type,
    &payload,
    max_retries,
    idempotency_key.as_deref(),
    tenant_id,
    self.handle_key.as_deref(),  // NEW
    available_at,
)
.await
```

**`PendingDispatch::new` initialization** (dispatcher.rs L39–46 — add `handle_key: None` alongside `tenant_id: None`):
```rust
Self {
    job,
    queue: None,
    delay: None,
    tenant_id: None,
    handle_key: None,  // NEW
}
```

---

### `ferro-queue/src/db.rs` (modified — utility, CRUD)

**Analog:** itself — `JobRow` struct (L117–138), `parse_job_row` (L214–258), and `enqueue` both arms (L518–601).

**`tenant_id` in `JobRow`** (db.rs L133 — the field to mirror for `handle_key`):
```rust
/// Optional tenant scope.
pub tenant_id: Option<i64>,
```
Add after `tenant_id`:
```rust
/// Optional offload handle key (UUID string). Present for jobs dispatched via Offloadable::offload().
pub handle_key: Option<String>,
```

**`parse_job_row` — `tenant_id` parse** (db.rs L234–238 — the exact pattern to mirror for `handle_key`):
```rust
let tenant_id: Option<i64> = row
    .try_get_by::<Option<i64>, _>("tenant_id")
    .map_err(|e| Error::custom(format!("parse tenant_id: {e}")))?;
```
Mirror:
```rust
let handle_key: Option<String> = row
    .try_get_by::<Option<String>, _>("handle_key")
    .map_err(|e| Error::custom(format!("parse handle_key: {e}")))?;
```
Add `handle_key` to the returned `JobRow { … }` initializer (db.rs L246–257).

**`enqueue` idempotency-key arm SQL** (db.rs L544–548 — `tenant_id` column to mirror):
```sql
INSERT INTO jobs (queue, job_type, payload, status, attempts, max_retries,
 idempotency_key, tenant_id, available_at, created_at)
SELECT …
```
Add `handle_key` to the column list and add a corresponding `Value::String(...)` bound value. The binding follows the same pattern as `tenant_id`: `handle_key.map_or(Value::String(None), |k| Value::String(Some(Box::new(k.to_string()))))`.

**`enqueue` plain-insert arm SQL** (db.rs L579–583 — same pattern):
```sql
INSERT INTO jobs (queue, job_type, payload, status, attempts, max_retries,
 tenant_id, available_at, created_at)
```
Add `handle_key` column + binding.

**`enqueue` function signature** (db.rs L518–527 — add `handle_key: Option<&str>` after `tenant_id`):
```rust
pub async fn enqueue(
    conn: &DatabaseConnection,
    queue: &str,
    job_type: &str,
    payload: &str,
    max_retries: u32,
    idempotency_key: Option<&str>,
    tenant_id: Option<i64>,
    handle_key: Option<&str>,  // NEW
    available_at: DateTime<Utc>,
) -> Result<(), Error>
```
The `#[allow(clippy::too_many_arguments)]` attribute is already present at L517 — keep it.

**`claim` SQL SELECT** (db.rs L360–363 for Postgres, L411–416 for SQLite — add `handle_key` to the column list in both `claim_postgres` and `claim_sqlite`):
```sql
-- Postgres claim_postgres (L360–363):
SELECT id, job_type, payload, queue, attempts, max_retries, idempotency_key,
 tenant_id, available_at, created_at FROM jobs …
-- SQLite claim_sqlite (L411–416):
… RETURNING id, job_type, payload, queue, attempts, max_retries,
  idempotency_key, tenant_id, available_at, created_at
```
Both need `handle_key` added so `parse_job_row` can find the column.

**Migration — `CreateJobsTable` amendment** (migration.rs L69–70 — `TenantId` column to mirror for `HandleKey`):
```rust
.col(ColumnDef::new(Jobs::TenantId).big_integer().null())
```
Mirror:
```rust
.col(ColumnDef::new(Jobs::HandleKey).string().null())
```
Add `HandleKey` to the `Jobs` DeriveIden enum (migration.rs L147–164). The column goes after `TenantId` in the `Table::create()` builder. The column is nullable — existing rows get NULL (backward-compatible). Since ferro is not yet in production (MEMORY.md), amending the original migration is safe (RESEARCH RQ5 recommendation — no `AddHandleKeyToJobs` separate migration needed).

---

### `ferro-queue/src/worker.rs` (modified — service, event-driven)

**Analog:** itself — `spawn_job` (L374–484) and `handle_failure` (L488–512).

**`spawn_job` — current variables extracted from `job_row`** (worker.rs L403–406 — the `tenant_id` extraction to mirror for `handle_key`):
```rust
let job_id = job_row.id;
let job_type = job_row.job_type.clone();
let tenant_id = job_row.tenant_id;
let attempts = job_row.attempts;
let max_retries = job_row.max_retries;
```
Add:
```rust
let handle_key = job_row.handle_key.clone();  // NEW: Option<String>
```

**`spawn_job` — success path** (worker.rs L453–457 — add snapshot write before `delete_job`):
```rust
Ok((Ok(()), _)) => {
    debug!(job_id = %job_id, job_type = %job_type, "Job succeeded — deleting row");
    crate::db::delete_job(conn, job_id).await.ok();
}
```
The value to persist comes from a `JobHandler` return type extension (see below). After obtaining the `Option<serde_json::Value>` success value:
```rust
Ok((Ok(success_value), _)) => {
    if let (Some(ref key), Some(ref val)) = (&handle_key, &success_value) {
        if let Err(e) = ::ferro::offload::persist_result_raw(key, val.clone(), conn).await {
            tracing::warn!(job_id = %job_id, error = %e, "offload result persist failed");
        }
    }
    crate::db::delete_job(conn, job_id).await.ok();
}
```

**`spawn_job` — `Err` arm** (worker.rs L460–470 — call `handle_failure` which is extended to receive `handle_key`):
```rust
Ok((Err(e), retry_delay)) => {
    error!(job_id = %job_id, job_type = %job_type, error = %e, "Job handler returned error");
    handle_failure(conn, job_id, attempts, max_retries, &e.to_string(), retry_delay).await;
}
```
Becomes (pass `handle_key` to `handle_failure`):
```rust
Ok((Err(e), retry_delay)) => {
    error!(…);
    handle_failure(conn, job_id, attempts, max_retries, &e.to_string(), retry_delay, handle_key.as_deref()).await;
}
```

**`spawn_job` — panic arm** (worker.rs L474–481 — add persist_error call):
```rust
Err(_panic) => {
    error!(job_id = %job_id, job_type = %job_type, "Job handler panicked — counting as failure");
    let msg = "job handler panicked";
    let delay = default_jitter_delay(attempts);
    handle_failure(conn, job_id, attempts, max_retries, msg, delay).await;
}
```
Becomes (extend `handle_failure` call + persist_error in panic arm directly when retries exhausted — or let `handle_failure` handle it uniformly):
```rust
Err(_panic) => {
    error!(…);
    let msg = "job handler panicked";
    let delay = default_jitter_delay(attempts);
    handle_failure(conn, job_id, attempts, max_retries, msg, delay, handle_key.as_deref()).await;
}
```

**`handle_failure` — terminal-error snapshot write** (worker.rs L488–512 — extend to accept `handle_key: Option<&str>` and call `persist_error` when exhausted):
```rust
async fn handle_failure(
    conn: &'static DatabaseConnection,
    job_id: i64,
    attempts: u32,
    max_retries: u32,
    err_msg: &str,
    retry_delay: Duration,
) {
    if attempts + 1 >= max_retries {
        warn!(…);
        crate::db::fail_job(conn, job_id, err_msg).await.ok();
    } else {
        …
    }
}
```
Extend signature to `handle_key: Option<&str>` and add persist_error before `fail_job`:
```rust
if attempts + 1 >= max_retries {
    if let Some(key) = handle_key {
        if let Err(e) = ::ferro::offload::persist_error(key, err_msg, conn).await {
            tracing::warn!(job_id = %job_id, error = %e, "offload error persist failed");
        }
    }
    crate::db::fail_job(conn, job_id, err_msg).await.ok();
}
```

**`JobHandler` type alias extension** (worker.rs L108–112 — return type must carry the serialized success value):
```rust
// Current:
type JobHandler = Arc<
    dyn Fn(String, u32) -> Pin<Box<dyn Future<Output = (Result<(), Error>, Duration)> + Send>>
        + Send
        + Sync,
>;
```
Extend to:
```rust
// Extended:
type JobHandler = Arc<
    dyn Fn(String, u32) -> Pin<Box<dyn Future<Output = (Result<Option<serde_json::Value>, Error>, Duration)> + Send>>
        + Send
        + Sync,
>;
```
The `register::<J>` closure (worker.rs L181–197) must be updated to return `Ok(Some(serialized_value))` on success and `Err(...)` on failure (replacing `Ok(())`). The serialized value comes from `serde_json::to_value(&result_value)` on the job method return.

**Critical constraint:** The handler closure only sees the deserialized `J` and calls `job.handle().await`; `handle()` currently returns `Result<(), Error>`. Extending `JobHandler` means the `register` closure must call a new `job.handle_with_value().await` or the value must come from a side-channel. The RESEARCH §RQ3 final recommendation is to capture the value from within the `WorkerLoop::register` closure, not inside `Job::handle()`. The planner must decide the precise mechanism (e.g., a new `Job::handle_with_value()` provided method that the macro overrides, or extending the handler closure to call the derived `handle()` body directly). The pattern to follow for any signature change is the existing `register` closure body (worker.rs L181–197).

---

### `ferro-macros/src/offload.rs` (modified — codegen, transform)

**Analog:** itself — `emit_job_items` (L222–329) and the four `call_expr` arms (L249–274).

**Current four call_expr arms** (offload.rs L249–274 — these are the exact bodies to modify):
```rust
// async + non-Result:
(true, false) => quote! {
    let _ = svc.#method_ident( #( #field_args ),* ).await;
    Ok(())
},
// async + Result:
(true, true) => quote! {
    svc.#method_ident( #( #field_args ),* ).await
        .map(|_| ())
        .map_err(|e| ::ferro::queue::Error::job_failed(
            #job_ident_str,
            format!("{e}"),
        ))
},
// sync + non-Result:
(false, false) => quote! {
    let _ = svc.#method_ident( #( #field_args ),* );
    Ok(())
},
// sync + Result:
(false, true) => quote! {
    svc.#method_ident( #( #field_args ),* )
        .map(|_| ())
        .map_err(|e| ::ferro::queue::Error::job_failed(
            #job_ident_str,
            format!("{e}"),
        ))
},
```

**Adaptation:** if `JobHandler` return type is extended to `Result<Option<serde_json::Value>, Error>`, then the `.map(|_| ())` calls become `.map(|v| ::serde_json::to_value(&v).ok())`. For the non-Result arms, `Ok(Some(...))` or `Ok(None)` (for `()` output). The macro does NOT need to call `persist_result` — that happens in `spawn_job` using `job_row.handle_key`. The macro's job is only to not discard the value.

**`failed()` override for sync-mode path** (dispatcher.rs L117–127 calls `job.failed(&e).await` — the only sync-mode path). To persist the terminal-error envelope in sync mode, add a `failed()` override in `emit_job_items`:
```rust
async fn failed(&self, error: &::ferro::queue::Error) {
    // Sync-mode terminal-error path (dispatcher.rs:dispatch_immediately).
    // The real async worker path writes the snapshot from spawn_job/handle_failure.
    // This override handles the QUEUE_CONNECTION=sync test path.
    if let Some(ref key) = self.__offload_handle_key {
        // NOTE: __offload_handle_key on self is only available if the key
        // is stored in the Job struct — this conflicts with D-05.
        // Preferred: no-op here; rely on spawn_job for production path.
        // The sync-mode tests use dispatch_immediately which does not write snapshots (D-08 deferred).
    }
}
```
The planner must decide whether to add a `failed()` override or leave it as a no-op. Given D-10's statement that "overriding `failed()` in the macro" is the approach, but RESEARCH §RQ4 finds that `failed()` is not called in the async production path, the cleanest resolution is:
- No `failed()` override in the macro (or a no-op override).
- All snapshot writes happen from `spawn_job`/`handle_failure`.
- Sync-mode tests (`QUEUE_CONNECTION=sync`) do NOT assert snapshot writes (snapshot is only meaningful in the DB-backed worker path).

This avoids trying to carry `handle_key` inside the Job struct (which would violate D-05).

---

### `ferro-queue/tests/offload_result_round_trip.rs` (new — test)

**Analog 1:** `ferro-queue/tests/offload_round_trip.rs` (L1–129) — overall test file structure.

**Analog 2:** `ferro-projection/src/runtime.rs` inline tests (L307–775) — `TestMigrator` + `sqlite::memory:` + async test pattern.

**TestMigrator pattern** (runtime.rs L378–391 — copy for the integration test, running BOTH migrations):
```rust
struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![
            Box::new(ferro_queue::CreateJobsTable),
            Box::new(ferro_projection::CreateProjectionSnapshotsTable),
        ]
    }
}
```

**`sqlite::memory:` DB setup** (runtime.rs L387–390):
```rust
let conn = Database::connect("sqlite::memory:").await.expect("connect");
TestMigrator::up(&conn, None).await.expect("migrate");
```

**`WorkerLoop` drain pattern** (no existing analog — new for this phase):
```rust
Queue::init(conn.clone()).await.expect("init queue");
let worker = WorkerLoop::from_registry(WorkerConfig {
    stop_on_error: true,
    ..WorkerConfig::default()
});
// Dispatch a job, then run the worker until it drains (one tick).
// After the job runs, assert the snapshot row exists.
```

The test file goes in `framework/tests/offload_result_round_trip.rs` (preferred over `ferro-queue/tests/` to avoid the Cargo dev-dep cycle from `ferro-queue` tests importing `ferro-projection` — RESEARCH §RQ8 Pitfall 6 recommendation).

---

## Shared Patterns

### SeaORM OnConflict upsert (composite PK)
**Source:** `ferro-projection/src/runtime.rs` L158–165
**Apply to:** `ferro-projection/src/direct.rs` (snapshot_write), `framework/src/offload.rs` (write_envelope)
```rust
Entity::insert(am)
    .on_conflict(
        OnConflict::columns([Column::ProjectionName, Column::Key])
            .update_columns([Column::State, Column::UpdatedAt])
            .to_owned(),
    )
    .exec(db)
    .await?;
```
Note: omit `Column::Version` from `update_columns` in `snapshot_write` (fixed `version = 1`), unlike `apply_event` which increments it.

### `::ferro::*`-only path emission in macros
**Source:** `ferro-macros/src/offload.rs` L301–327 (all `::ferro::queue::*`, `::ferro::App`, `::ferro::async_trait`, `::ferro::inventory`)
**Apply to:** any new path emitted by `emit_job_items` — must use `::ferro::offload::persist_result`, `::ferro::offload::persist_error`, never `::ferro_projection::*` or `::ferro_queue::*` directly.

### `Option<T>` nullable column in `parse_job_row`
**Source:** `ferro-queue/src/db.rs` L234–238 (`tenant_id` parsing)
**Apply to:** `handle_key` parsing in `parse_job_row`
```rust
let handle_key: Option<String> = row
    .try_get_by::<Option<String>, _>("handle_key")
    .map_err(|e| Error::custom(format!("parse handle_key: {e}")))?;
```

### Nullable column in migration
**Source:** `ferro-queue/src/migration.rs` L69–70
**Apply to:** `handle_key` column in `CreateJobsTable`
```rust
.col(ColumnDef::new(Jobs::HandleKey).string().null())
```

### `tracing::warn!` on non-fatal async error
**Source:** `ferro-projection/src/runtime.rs` L190–197 (broadcast failure does not roll back state)
**Apply to:** `persist_result` and `persist_error` call sites in `spawn_job` / `handle_failure`
```rust
if let Err(e) = persist_result(...).await {
    tracing::warn!(job_id = %job_id, error = %e, "offload result persist failed — result not stored");
}
// Continue with delete_job/fail_job regardless.
```

### `sqlite::memory:` + `TestMigrator` pattern
**Source:** `ferro-projection/src/runtime.rs` L378–391
**Apply to:** unit tests in `ferro-projection/src/direct.rs` (single migration); integration test in `framework/tests/offload_result_round_trip.rs` (two migrations)

---

## No Analog Found

All files in scope have close or exact analogs in the codebase. No "no analog" entries.

---

## Metadata

**Analog search scope:** `ferro-projection/src/`, `ferro-queue/src/`, `ferro-queue/tests/`, `ferro-macros/src/`, `framework/src/`, `framework/Cargo.toml`
**Files read:** 12 source files
**Pattern extraction date:** 2026-08-13
