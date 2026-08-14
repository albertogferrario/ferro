# Phase 247: Read-model delta → broadcast streaming — Research

**Researched:** 2026-08-14
**Domain:** ferro-queue / ferro-projection / ferro-broadcast — offload result hook, pending snapshot, race-safe resolve helper, redacted read-back
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** Delta emitted from the framework offload layer at the result-persist seam
(`framework::register_offload_hooks`). `ferro-projection::snapshot_write` stays write-only.

**D-02:** Persist-then-broadcast order (mirrors `runtime.rs:158`). Broadcast failure does not
roll back the snapshot; log at `tracing::warn!` and continue.

**D-03 (planning/research):** Exact mechanism for threading a `Broadcaster` into the
result-persist hook — left to research/planning (see D-03 Analysis below).

**D-04:** Broadcast channel is `projection.offload.result.{handle}` — the existing
`(OFFLOAD_PROJECTION_NAME, handle_key)` convention, identical to the event-fold format
`projection.{name}.{key}` in `runtime.rs:168`.

**D-05:** Completed delta carries the result value; failed delta carries a non-sensitive
terminal marker only (raw error NOT broadcast; it stays in the snapshot).

**D-06:** Delta is wakeup + client-safe payload; snapshot is authoritative store.

**D-07:** `{status:"pending"}` snapshot written at enqueue. Extends `OffloadResult<T>` with
a third tag.

**D-08 (planning):** Pending write seam — on-enqueue hook vs. framework wrapper around
`.offload()` — left to planning (see D-08 Analysis below).

**D-09:** `OffloadHandle<T>` gains a race-safe resolve helper encapsulating subscribe-first /
read-back-once / await-delta order. Timeout semantics are Claude's discretion.

**D-10:** `framework::offload::read_result_redacted` — mirrors delta redaction (value on
success, non-sensitive marker on failure). Existing `read_result` retained for server-side use.

**D-11:** Channel is public keyed by the unguessable UUID v4 handle (capability model).
Private-authorized channel is deferred.

**D-12:** Integration tests: worker persists on Broadcaster A → Broadcaster B subscriber
(via in-memory transport) receives redacted delta on
`projection.offload.result.{handle}`; plus SC#2 non-blocking assertion; plus env-gated
live-redis cross-process variant.

### Claude's Discretion

- Exact delta event-name string (recommend fixed `"offload.result"`).
- Resolve-helper signature + timeout semantics (recommend bounded await + optional timeout).
- Module home for `read_result_redacted` and resolve helper (keep under `::ferro::offload::*`).
- Precise plumbing for D-03 (Broadcaster → result hook) and D-08 (pending write seam).

### Deferred Ideas (OUT OF SCOPE)

- Private-authorized result channel (`private-projection.offload.result.{handle}`).
- Built-in framework read-back route.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| OFFLOAD-04 | A client subscribed to a handle receives the result as a `ferro-broadcast` delta on completion; the originating request returns immediately and never blocks awaiting it. | D-01/D-02: broadcast seam in `register_offload_hooks`; D-09 resolve helper; SC#2 non-blocking proof via test. |
</phase_requirements>

---

## Summary

Phase 247 closes the fire-and-forward loop by adding three capabilities to the existing
offload stack:

1. **Broadcast emission at result persist:** the result hook registered in
   `framework::offload::register_offload_hooks` (currently persists only) is extended to
   also broadcast a delta on `projection.offload.result.{handle}` immediately after the
   snapshot write.

2. **Pending marker at enqueue:** a `{status:"pending"}` snapshot is written when `.offload()`
   is called on the request side, extending `OffloadResult<T>` with a third variant and
   closing the unknown-handle / not-done ambiguity left by Phase 246 D-08.

3. **Race-safe resolve helper and redacted read-back:** `OffloadHandle<T>` gains
   `.resolve()` (subscribe-first / read-back / await-delta) and `framework::offload` gains
   `read_result_redacted` for the browser read-back leg.

All three capabilities are delivered through the framework layer, maintaining the
`ferro-queue` → `ferro-projection` / `ferro-broadcast` one-way boundary (D-11).

**Primary recommendation:** Thread the `Broadcaster` into the existing result hook by
changing `register_offload_hooks` from `fn` to `fn(Arc<Broadcaster>)` — the simplest,
lowest-risk approach consistent with all existing patterns. The pending write is best
implemented as a framework-level thin wrapper around `Offloadable::offload()` rather than
a second `OnceLock` hook, because the enqueue side has straightforward access to both
the handle key and the DB connection via the framework container.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Broadcast delta emission | Framework (app.rs / offload.rs) | `ferro-queue` (invokes hook) | Keeps `ferro-queue` dep-free of `ferro-broadcast` |
| Pending snapshot write | Framework (offload.rs wrapper) | — | Same boundary: enqueue side is framework-controlled |
| Race-safe resolve helper | `ferro-queue` (`offload.rs`) | Framework (calls snapshot_read) | `OffloadHandle` lives in `ferro-queue`; snapshot access is delegated via a callback or `DatabaseConnection` argument |
| Redacted read-back | Framework (`offload.rs`) | — | Mirrors `read_result`, lives in same module |
| Channel naming | Framework (`offload.rs`) | — | `OFFLOAD_PROJECTION_NAME` already there |
| Test harness | `framework/tests/` | — | Only location that can access both crates without a dep-cycle |

---

## Standard Stack

### Core (all already in workspace)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `ferro-broadcast` | workspace | `Broadcast` builder, `Broadcaster`, `InMemoryTransport` | The existing WebSocket broadcast layer |
| `ferro-projection` | workspace | `snapshot_write`, `snapshot_read`, `ProjectionKey` | Authoritative snapshot store |
| `ferro-queue` | workspace | `OffloadHandle`, `HandleKey`, `Offloadable`, hook registry | Offload primitive |
| `framework` | workspace | `offload.rs`, `app.rs` bootstrap, `register_offload_hooks` | Integration seam |
| `serde_json` | workspace | `json!` macro for delta payload | Standard |
| `tracing` | workspace | `warn!` on broadcast failure | Standard |

**No new dependencies.** Phase 246.1 shipped `InMemoryTransport` and `Broadcaster::with_transport`; this phase only uses them.

---

## Architecture Patterns

### System Architecture Diagram

```
[Request path]                          [Worker path]
     │                                       │
     ▼                                       ▼
framework::offload_wrapper              WorkerLoop::spawn_job
     │                                       │
     │  1. snapshot_write {pending}          │  2. handler executes
     │  2. Offloadable::offload()            │     (Job::handle_with_value)
     │  3. return OffloadHandle              │
     │                                       │
     ▼                                       ▼
 OffloadHandle<T>               persist_offload_outcome
 (capability token)                          │
                                 ┌─── Ok(value) ──────────────────┐
                                 │                                  │
                                 ▼                                  ▼
                         persist_result_raw                  persist_error
                         (snapshot: completed)              (snapshot: failed)
                                 │                                  │
                                 └──────────── both ───────────────┘
                                                 │
                                                 ▼
                                         broadcast_delta()
                                         channel: "projection.offload.result.{handle}"
                                         event:   "offload.result"
                                         data:    {status, value|marker}  [redacted]
                                                 │
                                    ┌────────────┴────────────┐
                                    │                          │
                              Broadcaster A              InMemoryTransport
                              (worker replica)                 │
                                                        Broadcaster B
                                                        (request replica)
                                                              │
                                                        subscribed client
                                                        OffloadHandle::resolve()
```

### Recommended Project Structure

No new files. All additions are in existing files:

```
framework/src/offload.rs         # extend: OffloadResult::Pending, register_offload_hooks(broadcaster),
                                 #         persist_pending, read_result_redacted, offload_wrapper
ferro-queue/src/offload.rs       # extend: OffloadHandle::resolve()
framework/tests/offload_delta_broadcast.rs  # new integration test file (D-12)
docs/src/features/queues.md      # extend: subscribe-then-await pattern (SC#3)
```

### Pattern 1: Persist-then-broadcast (template from `runtime.rs:158–199`)

The exact sequence in `ferro-projection/src/runtime.rs` at `apply_event`:

```rust
// Source: ferro-projection/src/runtime.rs:158–199
// Step 5: upsert (persist first)
Entity::insert(am).on_conflict(...).exec(&self.db).await?;

// Step 6: broadcast — failure does NOT roll back state
let channel_name = format!("projection.{}.{}", P::NAME, key.as_str());
let event_name = self.projection.broadcast_event_name();
let send_result = ferro_broadcast::Broadcast::new(self.broadcaster.clone())
    .channel(channel_name.clone())
    .event(event_name)
    .data(delta)
    .send()
    .await;

if let Err(e) = send_result {
    tracing::warn!(
        error = %e,
        channel = %channel_name,
        "projection broadcast failed; snapshot persisted"
    );
    // returns Err here in the projection path — for the offload hook, do NOT
    // fail the job; log and continue instead (D-02, Pitfall 5)
}
```

The offload adaptation: after `persist_result_raw` / `persist_error`, build the same
`Broadcast::new(broadcaster.clone()).channel(...).event("offload.result").data(...).send().await`
call. On failure: `tracing::warn!` and continue (do NOT return an error from the hook).

### Pattern 2: Hook registration via `OnceLock` (`dispatcher.rs`)

```rust
// Source: ferro-queue/src/dispatcher.rs:30–43
pub type OffloadResultHook = fn(
    String,                            // handle_key
    Result<serde_json::Value, String>, // outcome: Ok = completed / Err = error msg
    &'static sea_orm::DatabaseConnection,
) -> Pin<Box<dyn Future<Output = ()> + Send>>;

static OFFLOAD_RESULT_HOOK: OnceLock<OffloadResultHook> = OnceLock::new();

pub fn register_offload_result_hook(f: OffloadResultHook) {
    let _ = OFFLOAD_RESULT_HOOK.set(f);
}
```

**Constraint:** `OffloadResultHook` is a function pointer (`fn`), not a closure (`Fn`).
Function pointers cannot close over heap-allocated state like `Arc<Broadcaster>`. This is
the core challenge for D-03 (see D-03 Analysis).

### Pattern 3: Enqueue path (`offload.rs:118–126`)

```rust
// Source: ferro-queue/src/offload.rs:118–126
async fn offload(self) -> Result<OffloadHandle<Self::Output>, Error> {
    let key = HandleKey::new();
    crate::PendingDispatch::new(self)
        .with_handle_key(key.as_str().to_string())
        .dispatch()
        .await?;
    Ok(OffloadHandle::new(key))
}
```

The key is minted before the dispatch call. A framework wrapper calling
`snapshot_write` before (or immediately after) this `offload()` call has the key available.

### Anti-Patterns to Avoid

- **Modifying `OffloadResultHook` type to `Box<dyn Fn>` in `ferro-queue`:** would break
  the existing registration call site in `framework/src/offload.rs:183` and require
  `ferro-queue` to grow a dependency on something heap-allocated without context.
- **Broadcasting before persisting:** violates D-02 and breaks the authoritative-store
  guarantee — a subscriber who receives a delta and reads back might see `None`.
- **Returning `Err` from the hook on broadcast failure:** would fail the job; the non-fatal
  contract (D-02 / Pitfall 5) forbids this.
- **Putting the Broadcaster into `ferro-queue`:** violates D-11; any broadcast dependency
  belongs in the framework layer.

---

## D-03 Analysis: Threading the Broadcaster into the Result Hook

**The constraint (verified):** `OffloadResultHook` is defined in `ferro-queue/src/dispatcher.rs:30–34`
as a **function pointer** (`fn`):

```rust
pub type OffloadResultHook = fn(
    String,
    Result<serde_json::Value, String>,
    &'static sea_orm::DatabaseConnection,
) -> Pin<Box<dyn Future<Output = ()> + Send>>;
```

Function pointers cannot capture environment (no `Arc<Broadcaster>` in scope). The broadcaster
must be made available another way.

**Option A — Global static Broadcaster (OnceLock)**

Add a second `OnceLock<Arc<Broadcaster>>` in `framework/src/offload.rs`, registered at
bootstrap alongside `register_offload_hooks`. The hook function pointer reads the global
at call time:

```rust
// framework/src/offload.rs
static OFFLOAD_BROADCASTER: OnceLock<Arc<ferro_broadcast::Broadcaster>> = OnceLock::new();

pub fn register_offload_hooks_with_broadcaster(broadcaster: Arc<ferro_broadcast::Broadcaster>) {
    let _ = OFFLOAD_BROADCASTER.set(broadcaster);
    ferro_queue::register_offload_result_hook(|key, outcome, db| {
        Box::pin(async move {
            // ... persist ...
            if let Some(b) = OFFLOAD_BROADCASTER.get() {
                // ... broadcast ...
            }
        })
    });
}
```

This preserves the `fn` type of the hook. The function pointer reads a module-level static
at invocation time, identical to how `TENANT_ID_HOOK` (a `fn() -> Option<i64>`) works in
`ferro-queue/src/dispatcher.rs:13–23` without capturing state.

**Option B — Change `OffloadResultHook` to `Box<dyn Fn>`**

Change the type alias and the `OnceLock<OffloadResultHook>` in `ferro-queue` to use a trait
object. This makes the hook naturally closable over `Arc<Broadcaster>`. However it requires
modifying `ferro-queue` and is a more invasive change.

**Recommendation: Option A.** It requires zero changes to `ferro-queue`, is consistent with
the existing `TENANT_ID_HOOK` pattern (a static `fn` with no captured state), and matches the
project convention of keeping `ferro-queue` stable. The global broadcaster static is a
reasonable framework-level singleton — `App::get::<Broadcaster>()` in the app container is
already the canonical singleton.

At `app.rs` bootstrap (line 419), call
`crate::offload::register_offload_hooks_with_broadcaster(broadcaster)` instead of
`crate::offload::register_offload_hooks()`. The old name can be retained as a fallback (for
tests that do not need broadcast) or removed — one new function name is cleaner.

---

## D-08 Analysis: The Pending Snapshot Write Seam

**The constraint (verified):** `Offloadable::offload()` in `ferro-queue/src/offload.rs:118–126`
is a provided trait method. It mints the key, dispatches, and returns the handle. No hook
mechanism exists on the enqueue side (only on the result side via `register_offload_result_hook`).

**Option A — On-enqueue hook (symmetric to result hook)**

Add a second `OnceLock<OffloadEnqueueHook>` in `ferro-queue/src/dispatcher.rs`, called from
`Offloadable::offload()` after the dispatch succeeds. The hook receives `(handle_key, db)`
and writes the pending snapshot. This mirrors the result hook pattern exactly.

However, `Offloadable::offload()` currently returns `Result<OffloadHandle<..>, Error>` and
has no async context for the hook invocation — it would need to become async (it already is),
and the hook would need the DB connection. The DB connection is available via
`crate::db::Queue::connection()` (the static `&'static DatabaseConnection` already used in
`worker.rs:252`), so this is mechanically feasible.

**Option B — Framework-level wrapper around `.offload()`**

The framework exposes a function `framework::offload::offload_and_mark_pending(job, db)`
(or a method on a wrapper type) that:

1. Calls `job.offload().await?`
2. Immediately calls `persist_pending(handle.key(), db).await`
3. Returns the handle

The app code calls this wrapper instead of calling `.offload()` directly. The wrapper lives
entirely in the framework layer with direct access to `snapshot_write` — no new hook
machinery in `ferro-queue`.

**Recommendation: Option B.** Simpler and more contained — it requires zero changes to
`ferro-queue`. The macro-generated code for `#[offload]` methods would call
`::ferro::offload::enqueue_and_mark_pending(job, db)` rather than `job.offload().await`
directly. This is a macro-emission change (in the `ferro-macros` job-derivation logic, Phase
244) or a thin wrapper the planner can slot in without touching the macro if the app-side
calling convention already goes through the framework. Given that Phase 248 adds the
deployable worker (which must call the same enqueue path), this wrapper is the right
canonical form.

If the planner prefers to avoid a macro change, Option A is viable as a contained addendum
to `ferro-queue` — but Option B is recommended.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Cross-replica fan-out | Custom pub/sub | `InMemoryTransport` / `RedisTransport` (246.1) | Already shipped; `Broadcaster::with_transport` handles the fan-out loop |
| Subscribe-then-deliver | Custom async wait | `Broadcaster::subscribe` + `mpsc::Receiver` | Already the pattern in `runtime.rs` tests |
| Pending snapshot shape | Custom serialization | `serde_json::json!({ "status": "pending" })` + existing `snapshot_write` | Same as `persist_result_raw` / `persist_error` |
| Redaction | Custom string sanitizer | Simply omit the `error` field; use a fixed marker string | The delta has no "sanitize" work to do — just exclude the raw error |
| Race-safe ordering | Arbitrary logic | subscribe → read-back → await pattern (established in runtime tests) | The pattern eliminates the TOCTOU race deterministically |

---

## Current Code Surfaces (Verified, with file:line)

### `OffloadResult<T>` — `framework/src/offload.rs:49–62`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum OffloadResult<T> {
    Completed { value: T },
    Failed { error: String },
}
```

**Phase 247 extension:** add a `Pending` variant:

```rust
Pending,   // no fields — pending has no value yet
```

This is a backward-compatible serde addition (existing completed/failed records are
unaffected; `{"status":"pending"}` is a new tag that only new code writes).

### `register_offload_hooks` — `framework/src/offload.rs:182–198`

```rust
pub fn register_offload_hooks() {
    ferro_queue::register_offload_result_hook(|key, outcome, db| {
        Box::pin(async move {
            let res = match outcome {
                Ok(value) => persist_result_raw(&key, value, db).await,
                Err(msg) => persist_error(&key, &msg, db).await,
            };
            if let Err(e) = res {
                tracing::warn!(
                    handle_key = %key,
                    error = %e,
                    "offload result persist failed — result not stored"
                );
            }
        })
    });
}
```

**Phase 247 modification:** rename to `register_offload_hooks_with_broadcaster(broadcaster:
Arc<Broadcaster>)`. Inside the hook, after the match block, add the broadcast call (reading
`OFFLOAD_BROADCASTER` global). The `if let Err(e)` currently logs persist errors; broadcast
errors get their own warn log and are NOT returned.

### `OffloadResultHook` — `ferro-queue/src/dispatcher.rs:30–34`

```rust
pub type OffloadResultHook = fn(
    String,
    Result<serde_json::Value, String>,
    &'static sea_orm::DatabaseConnection,
) -> Pin<Box<dyn Future<Output = ()> + Send>>;
```

**Phase 247:** zero changes to this type. The hook function reads the broadcaster from a
module-level static in `framework/src/offload.rs`.

### `Offloadable::offload()` — `ferro-queue/src/offload.rs:118–126`

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

**Phase 247 (D-08 Option B):** unchanged. A framework wrapper calls `.offload()` and then
`persist_pending(handle.key(), db).await`.

### `OffloadHandle<T>` — `ferro-queue/src/offload.rs:71–96`

```rust
pub struct OffloadHandle<T> {
    key: HandleKey,
    #[serde(skip)]
    _phantom: PhantomData<fn() -> T>,
}
impl<T> OffloadHandle<T> {
    pub fn key(&self) -> &str { self.key.as_str() }
    pub fn id(&self) -> &HandleKey { &self.key }
}
```

**Phase 247 (D-09):** add `resolve` method. Because `OffloadHandle` lives in `ferro-queue`
but the resolve body needs `snapshot_read` (from `ferro-projection`) and `Broadcaster` (from
`ferro-broadcast`), the resolve helper cannot live in `ferro-queue` directly (D-11).

**Solution:** `resolve` lives in `framework/src/offload.rs` as a free function taking
`handle: &OffloadHandle<T>`, `broadcaster: &Broadcaster`, and `db: &DatabaseConnection`.
The `OffloadHandle` only needs to expose `.key()` — which it already does. No changes
to `ferro-queue/src/offload.rs`.

```rust
// framework/src/offload.rs (new)
pub async fn resolve<T: OffloadSerializable>(
    handle: &OffloadHandle<T>,
    broadcaster: &Arc<Broadcaster>,
    db: &DatabaseConnection,
    timeout: Option<Duration>,
) -> Result<OffloadResult<T>, ResolveError> {
    let channel = format!("projection.{}.{}", OFFLOAD_PROJECTION_NAME, handle.key());
    // 1. Subscribe first (prevents race: broadcast arrives before we subscribe)
    let (tx, mut rx) = mpsc::channel::<ServerMessage>(4);
    broadcaster.add_client(handle.key().to_string() + "-resolve", tx);
    broadcaster.subscribe(handle.key().to_string() + "-resolve", &channel, None, None).await?;
    // 2. Read back once — catches already-completed/failed
    if let Some(result) = read_result::<T>(handle.key(), db).await? {
        if !matches!(result, OffloadResult::Pending) {
            broadcaster.remove_client(handle.key().to_string() + "-resolve");
            return Ok(result);
        }
    }
    // 3. Await the delta with optional timeout
    let fut = async {
        while let Some(ServerMessage::Event(msg)) = rx.recv().await {
            if msg.event == "offload.result" {
                // Delta received — read back the authoritative snapshot
                broadcaster.remove_client(handle.key().to_string() + "-resolve");
                return read_result::<T>(handle.key(), db).await
                    .map(|opt| opt.unwrap_or(OffloadResult::Pending))
                    .map_err(ResolveError::from);
            }
        }
        Err(ResolveError::ChannelClosed)
    };
    match timeout {
        Some(d) => tokio::time::timeout(d, fut).await
            .map_err(|_| ResolveError::Timeout)?,
        None => fut.await,
    }
}
```

The planner should refine the exact client-id uniqueness and cleanup; this is the structural
shape for planning.

### `Broadcast` builder — `ferro-broadcast/src/broadcast.rs`

```rust
// ferro-broadcast/src/broadcast.rs:26–91 (verified)
pub struct Broadcast { broadcaster: Arc<Broadcaster> }
impl Broadcast {
    pub fn new(broadcaster: Arc<Broadcaster>) -> Self { Self { broadcaster } }
    pub fn channel(&self, name: impl Into<String>) -> BroadcastBuilder { ... }
}
impl BroadcastBuilder {
    pub fn event(mut self, name: impl Into<String>) -> Self { ... }
    pub fn data<T: Serialize>(mut self, data: T) -> Self { ... }
    pub fn except(mut self, socket_id: impl Into<String>) -> Self { ... }
    pub async fn send(self) -> Result<(), Error> { ... }
}
```

**Channel name (D-04):** `"projection.offload.result.{handle}"` — derived as
`format!("projection.{}.{}", OFFLOAD_PROJECTION_NAME, handle_key)` where
`OFFLOAD_PROJECTION_NAME = "offload.result"` (`framework/src/offload.rs:42`).

**Fan-out (246.1):** `Broadcaster::with_transport` (added in 246.1,
`ferro-broadcast/src/broadcaster.rs:90–120`) attaches a `BroadcastTransport`. When a
transport is set, `send_to_channel` → `fan_out` publishes `ServerMessage::Event` to the
bus (non-Event messages are not fanned out, `broadcaster.rs:400`). The subscribe loop on
other replicas delivers to local clients (origin-filtered). This is fully transparent to
the Phase 247 broadcast call — no Phase 247 code touches the transport.

### `Broadcaster::with_transport` — `ferro-broadcast/src/broadcaster.rs:90–120`

```rust
pub fn with_transport(self, transport: Arc<dyn BroadcastTransport + Send + Sync>) -> Self {
    // builds new inner with transport set, spawns subscribe+deliver tasks
    ...
}
```

**Phase 247 note:** `Broadcaster::new()` (no transport) is the in-process default. Tests
use `InMemoryTransport` to simulate a second replica without Redis. The live-redis variant
uses `RedisTransport` (feature-gated, shipped in 246.1).

### `runtime.rs:168` — channel name convention

```rust
// ferro-projection/src/runtime.rs:168
let channel_name = format!("projection.{}.{}", P::NAME, key.as_str());
```

For the offload delta: `P::NAME` = `"offload.result"` →
`format!("projection.{}.{}", "offload.result", handle_key)` =
`"projection.offload.result.{handle}"`. This is exactly D-04.

### `app.rs:419` — bootstrap call site

```rust
// framework/src/app.rs:419
crate::offload::register_offload_hooks();
```

Phase 247 changes this to:

```rust
if let Some(broadcaster) = App::try_get::<Broadcaster>() {
    crate::offload::register_offload_hooks_with_broadcaster(broadcaster.clone());
} else {
    crate::offload::register_offload_hooks();  // fallback: persist-only, no broadcast
}
```

Or, since the `Broadcaster` is always registered in `app.rs` before this point, simply:

```rust
let broadcaster = App::get::<Broadcaster>().clone();
crate::offload::register_offload_hooks_with_broadcaster(broadcaster);
```

The planner should confirm where `Broadcaster` is inserted into the App container and
whether `App::get::<Broadcaster>()` is available at bootstrap line 419.

### `drain_for_test` — `ferro-queue/src/worker.rs:393–433`

Already implemented in `WorkerLoop`. The 246-05 harness uses it successfully. Phase 247
tests reuse the same `drain_for_test` pattern from `framework/tests/offload_result_round_trip.rs`.

---

## Test Harness Templates (D-12)

### Template A: WorkerLoop drain + snapshot assert (from 246-05-PLAN.md)

```rust
// framework/tests/offload_result_round_trip.rs (246-05 harness)
// - sqlite::memory: with both CreateJobsTable + CreateProjectionSnapshotsTable
// - Queue::init(conn)
// - register_offload_hooks() (or register_offload_hooks_with_broadcaster for Phase 247)
// - #[offload]-derived job or hand-rolled handle_with_value job
// - worker.drain_for_test().await
// - ferro::offload::read_result::<T>(&key, db).await
```

### Template B: Env-gated live-redis (from 246.1-02-PLAN.md)

```rust
// ferro-broadcast/tests/redis_integration.rs (246.1 harness)
#![cfg(feature = "redis-transport")]
fn redis_url() -> Option<String> { std::env::var("REDIS_URL").ok().filter(|s| !s.is_empty()) }
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn redis_integration_cross_process_delivery() {
    let Some(url) = redis_url() else { eprintln!("REDIS_URL not set — skipping"); return; };
    let channel = format!("ferro:broadcast:test:{}", uuid::Uuid::new_v4());
    let bus_a = Arc::new(RedisTransport::new(&url, channel.clone()).await.unwrap());
    let bus_b = Arc::new(RedisTransport::new(&url, channel.clone()).await.unwrap());
    let a = Broadcaster::new().with_transport(bus_a);
    let b = Broadcaster::new().with_transport(bus_b);
    // subscribe b's client, sleep 150ms, broadcast from a, assert b receives
}
```

### Phase 247 test shape (D-12):

```rust
// framework/tests/offload_delta_broadcast.rs (new)
// Multi-replica shape:
//   bus = Arc::new(InMemoryTransport::new(64));
//   broadcaster_a = Broadcaster::new().with_transport(bus.clone()); // worker replica
//   broadcaster_b = Broadcaster::new().with_transport(bus.clone()); // client replica
//   register_offload_hooks_with_broadcaster(Arc::new(broadcaster_a));
//   // subscribe client to "projection.offload.result.{handle}" on broadcaster_b
//   // dispatch job → drain_for_test → assert client receives ServerMessage::Event
//   //   with event="offload.result", channel="projection.offload.result.{handle}",
//   //   data = {status:"completed", value:...}  (redacted: no raw error in failed case)
// SC#2 non-blocking:
//   let handle = job.offload().await?;
//   // assert job is NOT yet complete (queue has it)
//   // drain_for_test (simulates worker)
//   // assert result present AFTER drain
//   // (SC#2 proof: the return of offload() precedes worker execution)
```

---

## Common Pitfalls

### Pitfall 1: DashMap RefMut held across `.await`

**What goes wrong:** If the broadcaster's DashMap shard lock is held across an `.await` point,
the runtime cannot make progress on other tasks touching the same shard.

**Why it happens:** `self.inner.channels.get_mut()` returns a `RefMut` that must be dropped
before any `.await`. The `runtime.rs:123–128` pattern (clone the Arc, drop the RefMut before
await) is the canonical fix.

**How to avoid:** In the resolve helper, call `broadcaster.subscribe()` (which handles
locking internally) rather than touching the DashMap directly.

### Pitfall 2: Registering the offload hook twice in tests

**What goes wrong:** `OnceLock::set` silently ignores the second registration. If a test
registers `register_offload_hooks()` (persist-only) and then the code tries to register
`register_offload_hooks_with_broadcaster()`, the broadcaster is never registered.

**How to avoid:** Tests that need broadcast must register the broadcaster-aware hook first
(before any other hook registration in the same process). Use `serial_test::serial` on
integration tests that touch the global hook state.

### Pitfall 3: Broadcasting before subscribing

**What goes wrong:** The client subscribes to the channel after the worker completes and
broadcasts. The broadcast message is already gone; the client waits forever.

**Why it happens:** TOCTOU race in the subscribe-then-read-back pattern.

**How to avoid:** The resolve helper must subscribe FIRST (via `broadcaster.subscribe()`),
THEN read back the snapshot (to catch an already-completed handle), THEN await the channel.
This is the D-09 canonical order.

### Pitfall 4: Raw error exposed in the broadcast delta

**What goes wrong:** The `Display` form of the worker error (which may contain sensitive
values) is included in the broadcast payload — visible to any subscribed client.

**How to avoid:** D-05: for failed outcomes, the delta carries only
`{"status":"failed"}` (or a fixed non-sensitive marker). The raw error string lives only
in the snapshot (`persist_error` stores it) and the worker logs. `read_result_redacted`
returns `OffloadResult::Failed { error: "terminal error" }` (or an equivalent opaque marker).

### Pitfall 5: Hook failure causing job failure

**What goes wrong:** The broadcast call returns an `Err`; the hook propagates it; the
worker marks the job as failed and retries — triggering a second persist + broadcast.

**How to avoid:** The hook (like the Phase 246 persist hook) must catch all errors, log
via `tracing::warn!`, and return `()`. Broadcast failure is `warn!`-logged and swallowed.

### Pitfall 6: `ferro-queue` → `ferro-projection` / `ferro-broadcast` dependency

**What goes wrong:** Code that reads the snapshot or calls the broadcaster is placed inside
`ferro-queue`, introducing a dependency cycle.

**How to avoid:** All snapshot and broadcast calls live in `framework/src/offload.rs` (or
the hook closure registered from there). `ferro-queue` receives only the hook registration;
it never calls `snapshot_write` or `Broadcast::new` directly.

---

## Runtime State Inventory

Not applicable — this is a greenfield capability addition with no renamed or migrated state.
The `projection_snapshots` table (created by Phase 246 migration) is extended in-place with
a new `pending` tag on existing rows; no data migration is needed because pending rows are
written by new code and old rows (completed/failed) are unaffected by the new enum variant.

---

## Code Examples

### Emit the broadcast delta after persist (D-01, D-02)

```rust
// framework/src/offload.rs — inside the result hook closure
// Source pattern: ferro-projection/src/runtime.rs:168–196
async fn broadcast_delta(
    broadcaster: &Arc<ferro_broadcast::Broadcaster>,
    handle_key: &str,
    payload: serde_json::Value,
) {
    let channel = format!("projection.{}.{}", OFFLOAD_PROJECTION_NAME, handle_key);
    let send_result = ferro_broadcast::Broadcast::new(broadcaster.clone())
        .channel(channel.clone())
        .event("offload.result")
        .data(payload)
        .send()
        .await;
    if let Err(e) = send_result {
        tracing::warn!(
            handle_key = %handle_key,
            error = %e,
            channel = %channel,
            "offload delta broadcast failed; snapshot persisted"
        );
        // Do NOT propagate — broadcast failure is best-effort (D-02)
    }
}
```

### Pending snapshot shape (D-07)

```rust
// framework/src/offload.rs (new)
pub async fn persist_pending(
    handle_key: &str,
    db: &DatabaseConnection,
) -> Result<(), ProjectionError> {
    let envelope = serde_json::json!({ "status": "pending" });
    snapshot_write(db, OFFLOAD_PROJECTION_NAME, &ProjectionKey::new(handle_key), envelope).await
}
```

### Redacted read-back (D-10)

```rust
// framework/src/offload.rs (new)
pub async fn read_result_redacted<T: OffloadSerializable>(
    handle_key: &str,
    db: &DatabaseConnection,
) -> Result<Option<OffloadResult<T>>, ProjectionError> {
    match read_result::<T>(handle_key, db).await? {
        None => Ok(None),
        Some(OffloadResult::Completed { value }) => Ok(Some(OffloadResult::Completed { value })),
        Some(OffloadResult::Failed { .. }) => Ok(Some(OffloadResult::Failed {
            error: "terminal error".to_string(),  // non-sensitive marker (D-05)
        })),
        Some(OffloadResult::Pending) => Ok(Some(OffloadResult::Pending)),
    }
}
```

### Broadcast delta payload shapes (D-05)

```rust
// Completed — carries the value (client gets the answer in one message)
serde_json::json!({ "status": "completed", "value": value_json })

// Failed — non-sensitive marker only (raw error stays in snapshot)
serde_json::json!({ "status": "failed" })
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `OffloadResult<T>` with 2 variants | Extend to 3 variants (`Pending`) | Phase 247 | Backward-compatible; new serde tag |
| `register_offload_hooks()` (persist-only) | `register_offload_hooks_with_broadcaster(broadcaster)` (persist + broadcast) | Phase 247 | `app.rs:419` call site changes |
| `OffloadHandle<T>` inert | `OffloadHandle<T>` + `framework::offload::resolve()` | Phase 247 | First resolve/subscribe surface |

---

## Open Questions

1. **App container: is `Broadcaster` available at line 419 of `app.rs`?**
   - What we know: `app.rs:419` calls `crate::offload::register_offload_hooks()` after the
     bootstrap function runs. `Broadcaster` is injected into the `App` container during
     bootstrap (for WebSocket handling in `websocket.rs`).
   - What's unclear: whether `App::get::<Broadcaster>()` is called before or after the
     bootstrap function runs at line 404 in `app.rs`.
   - Recommendation: the planner reads `app.rs` lines 395–445 to confirm the ordering.
     If `Broadcaster` is not yet in the container at line 419, the planner can accept the
     `Broadcaster` as an argument to `Application::run` or defer registration to the
     bootstrap function itself.

2. **Resolve helper client-id uniqueness**
   - What we know: `Broadcaster::add_client` takes a `socket_id: String`. The resolve helper
     needs a unique id per call to avoid collisions.
   - Recommendation: `format!("{}-resolve-{}", handle.key(), uuid::Uuid::new_v4())` or
     simply `handle.key()` (the handle key is already globally unique per enqueue).

3. **`OffloadResult::Pending` and `serde_json::from_value` on old stored envelopes**
   - What we know: existing stored envelopes are `{"status":"completed",...}` or
     `{"status":"failed",...}`. Adding `Pending` as a new variant does not affect
     deserialization of those — they still match their existing arms.
   - What's unclear: whether the derive macro + internally-tagged serde enum will accept a
     missing-tag or unknown-tag gracefully.
   - Recommendation: add `#[serde(other)]` or `#[serde(rename_all = "snake_case")]` (already
     present) — the new `"pending"` tag will only be produced by new code. Verify with a
     `serde_json::from_str::<OffloadResult<()>>(r#"{"status":"pending"}"#)` unit test.

---

## Environment Availability

Step 2.6: SKIPPED — this phase adds no external dependencies beyond what Phases 246 and 246.1
already require (SQLite in-memory for unit tests; optional Redis for the env-gated integration
test variant). No new tools, services, or CLIs.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust / `#[tokio::test]` (workspace, no separate test runner) |
| Config file | `Cargo.toml` per crate |
| Quick run command | `cargo test -p ferro-rs --test offload_delta_broadcast` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| OFFLOAD-04 (SC#1) | Worker completes → subscribed client on Broadcaster B receives delta on `projection.offload.result.{handle}` | integration | `cargo test -p ferro-rs --test offload_delta_broadcast -- cross_replica_delta` | No — Wave 0 |
| OFFLOAD-04 (SC#2) | Originating request returns before worker finishes (non-blocking) | integration | `cargo test -p ferro-rs --test offload_delta_broadcast -- request_returns_before_worker` | No — Wave 0 |
| OFFLOAD-04 (SC#3) | `read_result_redacted` / subscribe-then-await pattern documented in `queues.md` | docs | manual review | queues.md exists, section missing |
| OFFLOAD-04 (D-05) | Failed delta carries non-sensitive marker only (no raw error) | unit | `cargo test -p ferro-rs -- offload_failed_delta_is_redacted` | No — Wave 0 |
| OFFLOAD-04 (D-07) | `persist_pending` writes `{status:"pending"}` retrievable by handle | unit | `cargo test -p ferro-rs -- offload_pending_round_trip` | No — Wave 0 |
| OFFLOAD-04 (D-09) | Race-safe resolve: subscribe first → read back once → await delta | integration | `cargo test -p ferro-rs --test offload_delta_broadcast -- resolve_already_complete` | No — Wave 0 |
| OFFLOAD-04 (live-redis) | Cross-process delivery over Redis | env-gated | `REDIS_URL=redis://... cargo test -p ferro-rs --test offload_delta_broadcast --features redis-transport -- redis_cross_replica` | No — Wave 0 |

### Observable Signals (per success criterion)

**SC#1 — subscriber receives delta:**
`rx.recv().await` on the Broadcaster B client's channel returns
`ServerMessage::Event(BroadcastMessage { event: "offload.result", channel: "projection.offload.result.{handle}", data: {status: "completed", ...} })`.
Asserted with `matches!` on the event name and channel.

**SC#2 — non-blocking:**
`let start = Instant::now(); let handle = job.offload().await?;
assert!(start.elapsed() < Duration::from_millis(100), "offload() must return before worker runs");`
(WorkerLoop is not yet drained at assertion time; the job is still in the queue.)

**SC#3 — documented pattern:**
Manual review of `docs/src/features/queues.md` section "Subscribe and await result".

**D-05 — no raw error in delta:**
Construct a hook result with `Err("sensitive-error-message".to_string())`;
call the broadcast path; intercept the delta; assert `data["error"]` is absent or equals
the opaque marker, not `"sensitive-error-message"`.

**D-07 — pending marker:**
Call `persist_pending("k1", &db).await`; `read_result::<()>("k1", &db).await` returns
`Some(OffloadResult::Pending)`.

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-rs --test offload_delta_broadcast` (fast, isolated)
- **Per wave merge:** `cargo test --all-features` (after disk-space check — see project fact
  `project_ferro_disk_full_test_gate.md`)
- **Phase gate:** full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- `framework/tests/offload_delta_broadcast.rs` — covers SC#1, SC#2, D-05, D-07, D-09,
  and the env-gated live-redis variant
- Unit tests for `persist_pending` and `read_result_redacted` can live in
  `framework/src/offload.rs` (same pattern as existing `offload_result_completed_round_trip`
  tests at lines 200–288)

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | partial | Capability model (D-11): unguessable UUID handle as the access token |
| V5 Input Validation | yes | Snapshot envelope deserialization via `serde_json::from_value` (strict) |
| V6 Cryptography | no | UUID v4 is not a cryptographic token; accepted caveat in D-11 |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Handle enumeration | Spoofing | UUID v4 (122 bits of randomness) — not guessable; documented accepted caveat in D-11 |
| Raw error exposure in delta | Information Disclosure | D-05: omit raw error from delta; use opaque marker |
| Hostile bus payload (Redis path) | Tampering | Strict `serde_json::from_str::<BusEnvelope>` — drop on parse error (inherited from 246.1 T-246.1-03) |
| Broadcast amplification / echo | Denial of Service | Origin-id echo suppression (246.1 D-03) — inherited |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `App::get::<Broadcaster>()` returns the app-configured `Broadcaster` at the point `register_offload_hooks` is called in `app.rs:419` | D-03 Analysis / Open Questions | If unavailable at that point, the bootstrap call site needs adjustment |
| A2 | `Broadcaster::add_client` / `Broadcaster::subscribe` are safe to call from a non-request context (the hook runs inside a spawned worker task) | Resolve helper shape | If tokio context requirements exist, wrapping in `tokio::spawn` may be needed |
| A3 | Adding `Pending` as a third `#[serde(tag = "status")]` variant to `OffloadResult<T>` does not break deserialization of existing `completed` / `failed` rows | OffloadResult extension | Minimal risk — serde internally-tagged enums are tag-discriminated; confirmed by existing test pattern |

---

## Sources

### Primary (HIGH confidence)

- `framework/src/offload.rs` — full file read; `register_offload_hooks`, `OffloadResult<T>`, `OFFLOAD_PROJECTION_NAME`, `persist_result_raw`, `persist_error`, `read_result`, `persist_pending` (to add)
- `ferro-queue/src/offload.rs` — full file read; `OffloadHandle<T>`, `HandleKey`, `Offloadable::offload()`
- `ferro-queue/src/dispatcher.rs` — full file read; `OffloadResultHook` type, `register_offload_result_hook`, `persist_offload_outcome`
- `ferro-queue/src/worker.rs` — full file read; `WorkerLoop::spawn_job`, `handle_failure`, `drain_for_test`
- `ferro-projection/src/runtime.rs` — full file read; `apply_event` steps 5–6 (persist-then-broadcast pattern at lines 158–199), channel naming at line 168
- `ferro-broadcast/src/broadcast.rs` — full file read; `Broadcast` builder surface
- `ferro-broadcast/src/broadcaster.rs` — lines 1–435 read; `with_transport`, `fan_out`, `send_to_channel`, `send_to_channel_local_only`
- `ferro-broadcast/src/transport/mod.rs` — full file read; `BroadcastTransport`, `BusEnvelope`
- `ferro-broadcast/src/transport/memory.rs` — full file read; `InMemoryTransport`
- `framework/src/app.rs:400–456` — `register_offload_hooks` call site at line 419
- `framework/src/lib.rs:215–258` — `ferro::broadcast::*` re-exports, `offload` module declaration

### Secondary (MEDIUM confidence)

- `.planning/phases/246-result-read-model-snapshot/246-05-PLAN.md` — WorkerLoop drain E2E harness shape; referenced for D-12 test template
- `.planning/phases/246.1-shared-transport-broadcast-fan-out-for-multi-replica-delta-d/246.1-02-PLAN.md` — env-gated live-redis integration test template; referenced for D-12 redis variant
- `docs/src/features/queues.md` — current queues documentation structure; SC#3 doc slot identified

---

## Metadata

**Confidence breakdown:**
- Hook plumbing (D-03 recommendation): HIGH — based on reading the actual `OffloadResultHook` type definition and established `OnceLock` patterns in the same codebase
- Pending write seam (D-08 recommendation): HIGH — based on reading `Offloadable::offload()` and confirming no enqueue hook exists
- Broadcast builder surface: HIGH — read the full `broadcast.rs` file
- Test harness shape: HIGH — read both referenced plan files in full

**Research date:** 2026-08-14
**Valid until:** This research is tied to the current codebase state. Valid until any of the
core files change: `framework/src/offload.rs`, `ferro-queue/src/dispatcher.rs`,
`ferro-queue/src/offload.rs`, `ferro-broadcast/src/broadcaster.rs`.
