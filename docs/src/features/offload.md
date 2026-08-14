# Work Distribution (Offload)

Ferro turns a `#[service]` trait method into a distributable unit of work by marking it `#[offload]`: the framework derives a `ferro-queue` Job and a serializable payload from the method signature, runs it on a horizontally scalable worker, and streams the result back through the read-model and broadcast path — the originating request never blocks.

## Authoring an offloadable method

The `#[offload]` attribute derives a `ferro-queue` Job directly from a `#[service]` trait method
signature. Instead of writing a Job struct by hand and wiring an enqueue call, mark the method and
the macro handles the derivation — the trait method itself keeps its in-process signature; `#[offload]`
layers an enqueue entrypoint on top.

### Authoring surface

```rust
use ferro::prelude::*;

#[service(impl = ReportBuilder)]
#[async_trait]
pub trait ReportsService: Send + Sync {
    #[offload]
    async fn build_monthly(&self, tenant_id: i64, month: Month) -> Report;
    // ^ keeps its in-process signature; #[offload] is additive
}
```

The macro derives a Job whose name follows the pattern `<TraitPascalCase><MethodPascalCase>Job`.
For `trait ReportsService` + method `build_monthly`, the derived struct is
`ReportsServiceBuildMonthlyJob`. The struct fields mirror the method parameters, each mapped to an
owned serializable type (borrows become owned equivalents).

The derived Job gains an `.offload()` enqueue entrypoint:

```rust
let handle: ferro::queue::OffloadHandle<Report> =
    ReportsServiceBuildMonthlyJob { tenant_id, month }
        .offload()
        .await?;

let key = handle.key(); // read-only key; see "Typed handle" below
```

No separate Job struct, no manual `Queue::register` for the enqueue call — the trait declaration
is the single source of truth for both the in-process and the background execution contract.

### Typed handle

`.offload()` returns `Result<OffloadHandle<T>, Error>`, where `T` is the method's success type.
`OffloadHandle<T>` identifies where the result will eventually land — a typed, key-bearing handle
that carries the success type as a type parameter.

In the current release the handle is **inert**: it exposes `.key()` and `.id()` for reading the
handle's identity key, but it has no resolve or subscribe methods. Reading the result back and
streaming it to a client is a later result-path capability; the key returned by `.key()` is where
a subscriber will later attach.

### Success-type contract

`T` is the success type of the method — the type the worker produces when the job completes
without error.

| Method return | `OffloadHandle<T>` type |
|---------------|-------------------------|
| `-> Report` | `OffloadHandle<Report>` |
| `-> Result<Report, E>` | `OffloadHandle<Report>` |
| `-> ()` or no return | `OffloadHandle<()>` |

For `-> Result<Report, E>` the handle is `OffloadHandle<Report>`. The error type `E` is not
required to be serializable — when the job fails, `E` is recorded as a job failure via its
`Display` representation (string-serialized). Serializable enforcement targets the success type
and the parameters, not the error.

### Serializable contract as the isolation boundary

Every parameter type and every success return type crossing the offload boundary must implement
`Serialize + DeserializeOwned`. The framework enforces this at compile time.

This is framed as the isolation boundary because it is one: the payload of an offloaded job must
be fully described by serializable data so the work can travel to a background worker — potentially
in a separate process. A method whose inputs or output cannot serialize cannot be offloaded, and
the constraint is checked before the code runs. The serializable contract seals the module across
the boundary.

When a parameter or return type does not satisfy `Serialize + DeserializeOwned`, the compiler
emits an `E0277` error with a branded message naming the offending type. The `Offloadable`
supertrait bounds (inherited from `Serialize + DeserializeOwned`) fire first in the error stream
— serde's own `E0277` messages appear before the branded diagnostic. The branded line appears
later in the same compilation and names the type explicitly:

```
error[E0277]: `RawReport` crosses the #[offload] isolation boundary and must be `Serialize + DeserializeOwned`
  = note: offloaded parameters and return types travel as a queue payload; implement `Serialize` and `DeserializeOwned` for `RawReport` to seal the module across the isolation boundary
```

The fix is to derive or implement `Serialize` and `DeserializeOwned` (via `serde`) on the
offending type:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Report {
    pub id: i64,
    pub tenant_id: i64,
    // ...
}
```

Once the type satisfies the bound the compilation succeeds and the derived Job is available.

## Result path

When a method is marked `#[offload]`, the request side returns an `OffloadHandle<T>` immediately — the work runs in the background. A client that needs the result subscribes to the handle's channel and awaits the completion delta.

### Channel convention

Each handle has a dedicated broadcast channel:

```
projection.offload.result.{handle_key}
```

where `handle_key` is the UUID v4 returned by `handle.key()`. The key is minted server-side and returned only to the enqueuing caller, so it functions as a capability token: unguessable and single-use.

### Request side

Use `::ferro::offload::enqueue_and_mark_pending` as the request-side entrypoint. It enqueues the job and immediately writes a `{"status":"pending"}` snapshot under the handle key — so a read-back can distinguish an unknown handle (no snapshot) from work in progress (pending snapshot). The call returns before the worker executes.

```rust
use ferro::offload::enqueue_and_mark_pending;

// In a request handler:
let handle = enqueue_and_mark_pending(ReportsServiceBuildMonthlyJob { tenant_id, month }, db)
    .await?;

// Serialize the key and send it to the client — the client uses it as the subscription key.
let key = handle.key().to_string();
```

### Server-side consumer: race-safe resolve

For a server-side consumer (e.g. a handler polling on behalf of the client), use `::ferro::offload::resolve`. It encapsulates the correct subscribe → read-back → await order so the TOCTOU race is impossible:

```rust
use ferro::offload::{resolve, OffloadResult};
use std::sync::Arc;
use std::time::Duration;

// Reconstruct the handle from the key stored in the session/DB, then:
let result = resolve(&handle, &Arc::new(broadcaster), db, Some(Duration::from_secs(30))).await?;

match result {
    OffloadResult::Completed { value } => { /* use value */ }
    OffloadResult::Failed { error } => { /* log error, surface non-sensitive marker */ }
    OffloadResult::Pending => { /* still in progress (timeout path) */ }
}
```

`resolve` performs three steps internally:

1. **Subscribe first** — buffers any delta that fires before the read-back, preventing missed events.
2. **Read back once** — if the handle already reached a terminal state, returns immediately without awaiting a delta.
3. **Await the delta, read the authoritative snapshot on wake** — the delta is a redacted wakeup signal; `resolve` reads `read_result` (full envelope, raw error included) for the authoritative result.

Pass `timeout: None` to wait indefinitely, or `Some(Duration)` to bound the wait. A terminally failed job always produces a `failed` delta and snapshot, so the only unbounded wait is a job that never runs.

### Browser / client-side read-back

Browser clients receive the redacted delta via the WebSocket subscription and can reconcile by reading the handle's snapshot via an application route that calls `read_result_redacted`:

```rust
use ferro::offload::read_result_redacted;

// In a route handler that receives the handle key from the client:
let result = read_result_redacted::<Report>(&key, db).await?;
```

`read_result_redacted` replaces a failed result's raw error with the fixed non-sensitive marker `"terminal error"`. Completed values and the pending marker pass through unchanged. The raw error is retained in the snapshot and worker logs for authorized server-side access via `read_result`.

### Delta payload and redaction

The delta broadcast on `projection.offload.result.{handle_key}` carries:

| Outcome | Delta payload | `error` field |
|---------|--------------|---------------|
| Completed | `{"status":"completed","value":<T>}` | absent |
| Failed | `{"status":"failed"}` | **absent** — raw error never broadcast |

The raw error is stored only in the authoritative snapshot (`read_result`). This separation lets the delta be safely sent to any subscribed client without leaking internal diagnostic strings.

### Migration

The offload result path requires the `projection_snapshots` table. Register the migration alongside your application's own migrations:

```rust
use ferro_projection::CreateProjectionSnapshotsTable;

// In your Migrator::migrations():
Box::new(CreateProjectionSnapshotsTable),
```

## Scaling model

A Ferro application serves growing user load by running more worker replicas alongside a stateless web tier — all sharing the same queue backend and broadcast transport.

### Deploy recipe

- Web replicas run the application binary as `<app-bin> serve --no-worker` — the HTTP tier with the
  in-process worker loop disabled.
- Worker replicas run `<app-bin> worker --queue <class>` (the flag is repeatable for multiple
  queues; omitting `--queue` drains all registered queues). The worker is the application's own
  binary, not a separate `ferro` CLI binary — job handlers live in the app crate.
- Single-binary development runs `<app-bin> serve`, which retains the in-process worker loop and
  drains all queues.
- All replicas share the same queue backend and broadcast transport.
- Capacity scales by running more worker replicas; N is chosen by the operator, platform, or
  cluster scheduler. The framework provides no autoscaler.

### Worker classes and fault isolation

A worker class is its set of queues, selected via `--queue`. Each class is an independent fault domain: a saturated `media` class does not starve the `default` class, because they are consumed by separate worker replicas. The `#[offload]` attribute accepts an optional `queue` argument to route a method to a specific class:

```rust
#[offload(queue = "reports")]
async fn build_monthly(&self, tenant_id: i64, month: Month) -> Report;
```

When the argument is omitted, the method routes to the `default` queue.

### Honest limitations

- **Connection ceiling.** `DB_MAX_CONNECTIONS` defaults to 10 per process; total database
  connections are roughly `10 × (web replicas + worker replicas)`. Against a typical Postgres
  ceiling near 100, a modest deployment (e.g. 5 web + 5 workers) already approaches it. For
  scale-out, place a connection pooler such as PgBouncer between the replicas and Postgres.
- **No built-in metrics export.** Generated deployment manifests (DigitalOcean App Platform YAML,
  Docker Compose) do not include an OpenTelemetry collector or Prometheus scrape configuration.
  Monitoring worker throughput and queue depth requires a separately provisioned observability
  stack; the framework does not emit one.
- **Latency is worker-scheduling-bound.** Result latency depends on worker scheduling, so the
  offload path is unsuited to sub-second interactive computation that must complete before a
  response renders. It is the right shape for deferred-result work — report generation, imports,
  model inference.

## Non-goals (2.0 direction)

The following are out of scope for the current design and recorded here as possible future work,
not commitments:

- Elastic scale-to-zero driven by queue depth (e.g. a KEDA `ScaledObject`).
- Warm-start / checkpoint-restore for faster scale-from-zero cold starts.
- Non-Kubernetes actuation through a worker-fleet provider port (e.g. Nomad).
- WASM/WASI isolates as a lighter execution unit.
