# Phase 246: Result → read-model snapshot - Context

**Gathered:** 2026-08-13
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults selected without interactive questioning)

<domain>
## Phase Boundary

Give offloaded work a **result path**. When the worker finishes running an
`#[offload]` method, it persists the method's return value into a
`ferro-projection` (singular — the live read-model runtime) snapshot keyed by the
handle, so the result is durably retrievable after completion without the
originating request having waited on it. A failed or panicking run records a
**terminal error state** on the handle rather than silently dropping.

In scope for 246 (OFFLOAD-03, ROADMAP Success Criteria 1–3):

1. On worker completion, the return value is persisted as a projection snapshot
   keyed by the handle.
2. The snapshot is retrievable by handle after completion (asserted in a test).
3. A failed/panicking offloaded method records a terminal error state on the
   handle (no silent drop).

Explicitly **out of scope** (later offload phases): the shared broadcast transport
for multi-replica delta delivery (246.1); read-model delta → broadcast streaming
and handle `.await`/`.subscribe()` resolve semantics (247); the deployable
`worker` subcommand runtime (248); `ferro-mcp` introspection + docs (249). 246
makes the result **retrievable by handle**; 247 makes it **stream to a subscribed
client**. This phase persists and reads back — it does not add a live push.

</domain>

<decisions>
## Implementation Decisions

### Persistence path — direct snapshot write, NOT the event-fold `Projection` trait
- **D-01:** The offload result is persisted through a **new direct snapshot
  write/read API added to `ferro-projection`**, decoupled from the `Projection`
  event-fold contract. The `ProjectionRuntime<P>` / `Projection` trait is built
  entirely around folding a *domain event* into per-key `State` via `apply()`
  (see `ferro-projection/src/runtime.rs:115` `apply_event`, `runtime.rs:238`
  `rebuild`). An offload result is a **one-shot, arbitrary-`T` value keyed by a
  UUID** — it has no `Event`, no `Default` state, and no incremental fold, so it
  does not fit that trait. The underlying `projection_snapshots` table
  (composite PK `(projection_name, key)`, `state: JsonValue`, `version`,
  `updated_at` — `ferro-projection/src/entity.rs`) fits it exactly.
- **D-02:** The new API is a low-level snapshot store over the existing
  `projection_snapshots` entity: an **upsert** (`OnConflict` on the composite PK,
  mirroring `apply_event` step 5 at `runtime.rs:158`) and a **read by
  `(name, key)`** (mirroring `read` at `runtime.rs:87`), taking a
  `&DatabaseConnection`, a projection name, a `ProjectionKey`, and a
  `serde_json::Value` state. It reuses the shipped `CreateProjectionSnapshotsTable`
  migration and entity — **no new table, no new migration**.
- **D-03:** This is an accepted **coherence-tax expansion of `ferro-projection`**
  (a public direct-write surface alongside the event-fold runtime), consistent
  with the precedent of Phase 244 expanding `ferro-queue`'s registration
  mechanism. The event-fold path is unchanged and remains the primary surface;
  the direct API is additive.

### Handle-key propagation — the load-bearing seam
- **D-04:** Today `.offload()` (`ferro-queue/src/offload.rs:118`) mints the
  `HandleKey` **after** dispatch and never embeds it in the enqueued payload, so
  the worker has no way to know which key to persist under. 246 reworks the
  enqueue path so the **handle key is minted before dispatch and travels with the
  job to the worker**; the worker persists the snapshot under that same key, which
  is the key the caller's `OffloadHandle` holds. Retrieval-by-handle (SC#2) is
  therefore against the identical key on both sides.
- **D-05:** The handle key travels as **job-execution metadata carried by the
  enqueue envelope**, NOT as a new field on the derived Job struct — so the
  serializable payload stays exactly the method's parameters (preserves 244 D-11:
  each non-`self` param becomes a field, nothing else). The exact `ferro-queue`
  mechanism (a `handle_key` column on the queue row threaded to the handler vs. a
  job-context slot passed into `Job::handle`) is a **planning/research decision**;
  the locked contract is: *the key the caller receives is the key the worker
  writes under, and the payload fields remain the method parameters only.*
- **D-06:** The fresh-UUID identity from 245 D-07 is preserved — the handle key
  stays **decoupled from `Job::idempotency_key()`**. 246 changes *when* the key is
  minted (before dispatch) and *that it is carried to the worker*, not *what it is*.

### Result envelope — tagged status discriminator
- **D-07:** The snapshot `state` (`JsonValue`) holds a **tagged result envelope**:
  a completed run stores `{ "status": "completed", "value": <Output-as-JSON> }`;
  a terminally failed run stores `{ "status": "failed", "error": "<message>" }`.
  A typed/untyped helper (e.g. an `OffloadResult<T>` read wrapper) is exposed so
  retrieval deserializes the success `value` back to `Output`, or surfaces the
  error string. `Output` is the 245 success type and is already
  `OffloadSerializable` (Serialize + DeserializeOwned), so `value` always
  round-trips.
- **D-08:** **No "pending" row is written at enqueue in 246.** SC#2 requires
  retrievable-*after-completion* only; the snapshot row appears at the terminal
  outcome (completed or failed). A pending/enqueued marker and the "unknown handle
  vs. not-done-yet" distinction are a 247-era subscription-lifecycle concern
  (deferred). Reading a handle whose work has not finished returns `None`
  (absent row), exactly like `ProjectionRuntime::read`.

### Terminal-error semantics vs. retries
- **D-09:** The **success snapshot is written when the derived `handle()` returns
  `Ok`** (after a successful method call — the value is captured instead of
  discarded via today's `.map(|_| ())` at `ferro-macros/src/offload.rs:251/255`).
  The **terminal-error snapshot is written when `ferro-queue` exhausts retries and
  invokes `Job::failed()`** — not on each transient attempt. This maps the two
  outcomes cleanly onto the two existing ferro-queue seams: `handle()` Ok →
  completed; `failed()` (retries exhausted) → failed. A transient failure that
  later succeeds ends in a **completed** snapshot (the terminal state reflects the
  true final outcome).
- **D-10:** The derived Job's **`failed()` is overridden by the macro** to persist
  the terminal-error envelope under the handle key; a panicking `handle()` is
  routed to the same `failed()` seam by ferro-queue's existing worker failure
  path, so panic and `Err` both land a terminal error (SC#3 "no silent drop"). The
  error message reuses 244 D-07's `Display`-stringified form already produced at
  `offload.rs:257/269` (`Error::job_failed(name, format!("{e}"))`).

### Write-back glue location — crate dependency direction
- **D-11:** The write-back glue lives in **`framework`** (the facade), which gains
  a **new `ferro-projection` dependency** (it currently depends only on
  `ferro-projections` *plural* — `framework/Cargo.toml:53`). The macro-emitted
  `handle()`/`failed()` call `::ferro::*` helpers (e.g.
  `::ferro::offload::persist_result(key, &value)` / `persist_error(key, &msg)`),
  matching the 244/245 convention that generated code emits **only `::ferro::*`
  paths**. `ferro-queue` does **not** gain a `ferro-projection` dependency — the
  queue stays free of the broadcast/events stack that `ferro-projection` pulls in.
- **D-12:** The write-back helper obtains its `&DatabaseConnection` from
  **`ferro-queue`'s existing global `Queue::connection()`** (`&'static
  DatabaseConnection`, used by the worker at `ferro-queue/src/worker.rs:247`) — the
  same database the worker already runs against. No new connection wiring, no
  container `App::make::<DatabaseConnection>` requirement for the result path.

### Reserved projection name + key convention
- **D-13:** Offload results use a **single reserved projection name**
  (`"offload.result"`) with **key = the handle UUID**. The handle UUID is already
  globally unique (245 D-07), so one namespace suffices and "keyed by the handle"
  maps directly to `key = handle`. The 247 broadcast channel derives as
  `projection.offload.result.{handle}` from the same `(name, key)`, so this choice
  also fixes the subscription key 247 will use — no per-method namespace.

### Claude's Discretion
- The exact `ferro-queue` mechanism for carrying the handle key to the worker
  (queue-row column threaded to the handler vs. a `Job::handle` context slot),
  provided D-04/D-05's contract holds.
- The precise name/shape of the new `ferro-projection` direct-write surface
  (free functions vs. a `SnapshotStore` struct vs. inherent methods) and whether
  it sets `version = 1` on first write or reuses the upsert-increment idiom.
- The exact tag/field names in the result envelope (`status`/`value`/`error` vs.
  an internally-tagged serde enum), as long as completed vs. failed is
  unambiguous and the success value deserializes back to `Output`.
- The name/module of the `::ferro::offload::*` write-back helpers and the read-back
  wrapper, and whether the read surface attaches to `OffloadHandle<T>` as an inert
  `resolve()`-style accessor now or waits for 247.
- Panic capture detail (whether ferro-queue's worker already converts a `handle()`
  panic into a `failed()` call, or a `catch_unwind`/join-error mapping is needed).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design anchor & phase spec
- `docs/superpowers/specs/2026-06-24-offload-work-distribution-design.md` — milestone
  anchor. §"Result path (fire-and-forward)" (~L75–88): the handle "identifies where
  the result will land" / "is the projection key the client subscribes to". §L29–30,
  L192–193: "the worker writes a `ferro-projection` snapshot … keyed by the handle".
  §L214: the round-trip acceptance shape (offloaded method runs on a worker, writes
  the projection snapshot).
- `.planning/ROADMAP.md` §"Phase 246: Result → read-model snapshot" (~L3353) — phase
  goal, dependency (245), and the three Success Criteria this phase must make TRUE.
- `.planning/REQUIREMENTS.md` — **OFFLOAD-03** (this phase's requirement); OFFLOAD-04
  (247, the immediate downstream consumer of the snapshot — shapes the channel/key).

### Predecessor phase context (locked decisions this phase builds on)
- `.planning/phases/245-typed-result-handle-serializable-enforcement/245-CONTEXT.md`
  — D-07 fresh-UUID handle key decoupled from idempotency; D-08 `OffloadHandle` inert
  (no resolve yet — 246 adds the *retrieve* half); D-09 `type Output` = success type;
  D-10 the worker's `handle()` **discards** the value in 245 (246 stops discarding).
- `.planning/phases/244-offload-macro-job-payload-derivation/244-CONTEXT.md` — D-07
  `Err` → job failure + `Display`-stringified message (reused for the error envelope);
  D-11 payload fields = non-`self` params (must stay so under D-05); D-14 `handle()`
  resolves the service from the container.

### The 245 offload types (extend, do not replace)
- `ferro-queue/src/offload.rs` — `OffloadHandle<T>` (L71), `HandleKey` (L41, UUID v4),
  `Offloadable::offload()` default (L118) that mints the key **after** dispatch and
  drops it — the D-04 rework target; `OffloadSerializable` (L31) bounding `Output`.

### The macro emission (where the value is captured / error persisted)
- `ferro-macros/src/offload.rs` — `emit_job_items` (L222); the `handle()` call
  expressions that currently discard the value via `.map(|_| ())` (L248–274, the four
  sync/async × Result arms); `type Output` emission (L326). 246 extends `handle()` to
  capture + persist and adds a `failed()` override (D-09/D-10).

### The snapshot store (the persistence target — new direct API here)
- `ferro-projection/src/runtime.rs` — `apply_event` upsert (step 5, L158, the
  `OnConflict` composite-PK idiom to mirror) and `read` (L87, the read-by-`(name,key)`
  idiom). The new direct write/read API sits alongside these, event-fold-free.
- `ferro-projection/src/entity.rs` — `projection_snapshots` `Model`
  (`projection_name`, `key`, `state: JsonValue`, `version`, `updated_at`) — the
  row the envelope is stored in.
- `ferro-projection/src/key.rs` — `ProjectionKey` (stringly-typed newtype; the
  handle UUID becomes the key).
- `ferro-projection/src/lib.rs` — the crate's public re-exports (where a new direct
  snapshot API must surface); `CreateProjectionSnapshotsTable` migration (reused).

### Worker DB + framework facade
- `ferro-queue/src/worker.rs:247` — `Queue::connection()` (`&'static
  DatabaseConnection`), the DB source the write-back helper reuses (D-12).
- `framework/Cargo.toml:53` — currently `ferro-projections` (plural) only; 246 adds
  `ferro-projection` (singular) (D-11).
- `framework/src/lib.rs:228` — the offload facade re-export block
  (`OffloadHandle`, `Offloadable`, …) where the write-back helpers / read wrapper
  surface as `::ferro::*`.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`projection_snapshots` table + entity + migration** (`ferro-projection/src/entity.rs`,
  `CreateProjectionSnapshotsTable`): the result snapshot store already exists —
  composite PK `(projection_name, key)`, `state: JsonValue`. 246 adds a direct
  write/read API over it; no schema work.
- **`apply_event` upsert idiom** (`runtime.rs:158`, `OnConflict::columns([ProjectionName, Key])`)
  and **`read`** (`runtime.rs:87`): the exact SeaORM patterns the new direct API mirrors.
- **`Queue::connection()`** (`ferro-queue/src/worker.rs:247`): a ready `&'static
  DatabaseConnection` inside the worker — the write-back's DB source (D-12).
- **245 offload types** (`ferro-queue/src/offload.rs`): `OffloadHandle<T>`, `HandleKey`,
  `Offloadable`, `OffloadSerializable` — 246 extends `offload()` (key-before-dispatch)
  and layers the retrieve/persist surface; the types are already `::ferro::*`-exported.
- **244 error stringification** (`offload.rs:257/269`, `Error::job_failed(name, format!("{e}"))`):
  the error message for the failed envelope is already produced.

### Established Patterns
- Generated code emits **only `::ferro::*` paths** (244/245) — the write-back helpers
  must be facade-exported and called as `::ferro::offload::*` from the macro.
- ferro-queue idiom: the Job struct **is** its serializable payload — the handle key
  must NOT become a payload field (D-05); it travels as enqueue metadata.
- ferro-projection is **single-instance / last-writer-wins** on concurrent same-key
  writes (`lib.rs` footgun #2). Each offload handle is a unique UUID, so same-key
  contention does not arise for results (one writer per handle) — a favorable fit.

### Integration Points
- `Offloadable::offload()` (`offload.rs:118`) — reworked to mint→carry→dispatch (D-04).
- `emit_job_items` `handle()` arms (`offload.rs:248`) — capture the value + persist on
  Ok; add a `failed()` override for the terminal-error write (D-09/D-10).
- `ferro-queue` enqueue/claim path — carries the handle key as job metadata to the
  worker (D-05); exact column/context slot is a planning decision.
- `framework` — new `ferro-projection` dep (D-11) + `::ferro::offload::*` write-back
  helpers over `Queue::connection()` (D-12).

</code_context>

<specifics>
## Specific Ideas

Target end-to-end shape (extends the 244/245 anchor example):

```rust
// Caller (web tier) — returns immediately, does not await the result:
let handle: OffloadHandle<Report> =
    ReportsBuildMonthlyJob { tenant_id, month }.offload().await?; // mints + carries key

// ... worker runs the method on another process/task ...

// After completion, the result is retrievable by handle:
let result: Option<OffloadResult<Report>> =
    ferro::offload::read_result::<Report>(handle.id()).await?;
// Some(Completed(report)) on success; Some(Failed("...")) on terminal failure; None if not done.
```

- The snapshot for handle `h` lands at `(projection_name = "offload.result", key = h)`
  — the same `(name, key)` from which 247 derives `projection.offload.result.{h}`.
- 246 delivers the **retrieve** half of the handle (SC#2); 247 adds the **subscribe/
  stream** half. The snapshot written here is exactly what 247 broadcasts as a delta.

</specifics>

<deferred>
## Deferred Ideas

- Shared-transport broadcast fan-out for multi-replica delta delivery — **Phase 246.1**.
- Read-model delta → broadcast streaming; `OffloadHandle::await`/`.subscribe()` resolve
  semantics; the caller receiving the result live without polling — **Phase 247**.
- A "pending"/enqueued snapshot row and the unknown-handle-vs-not-done distinction —
  247 subscription lifecycle (D-08).
- Reconciling a deduped job (same `idempotency_key`) that yielded multiple distinct
  handles — **Phase 246/247** (consequence of 245 D-07's random-UUID choice).
- Deployable `worker` subcommand runtime establishing DB/projection/broadcast context —
  **Phase 248** (246 reuses `Queue::connection()` in-process).
- `ferro-mcp` `list_services` offload introspection + result-path docs — **Phase 249**.
- Snapshot retention / TTL for completed offload results (results are not GC'd in 246) —
  future operational concern, not in the milestone scope.

None surfaced as scope creep; these are the already-planned downstream offload phases
and one noted operational follow-up.

</deferred>

---

*Phase: 246-result-read-model-snapshot*
*Context gathered: 2026-08-13*
