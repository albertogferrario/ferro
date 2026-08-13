# Phase 246: Result → read-model snapshot - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-08-13
**Phase:** 246-result-read-model-snapshot
**Mode:** `--auto` (recommended option auto-selected per gray area; no interactive questioning)
**Areas discussed:** Persistence path, Handle-key propagation, Result envelope, Terminal-error semantics, Write-back glue location, Projection name/key convention

---

## Persistence path — event-fold `Projection` vs. direct snapshot write

| Option | Description | Selected |
|--------|-------------|----------|
| Direct snapshot write/read API on `ferro-projection` | New low-level upsert/read over `projection_snapshots` by `(name, key)`, decoupled from the `Projection` event-fold trait | ✓ |
| Model the result as a synthetic `OffloadCompleted` event + a framework `Projection` impl | Reuses `apply_event`/broadcast wiring, but forces a one-shot heterogeneous `T` through Default-state + apply-fold semantics | |

**Auto-selected:** Direct snapshot write API (recommended).
**Notes:** The `ProjectionRuntime`/`Projection` trait folds a *domain event* into per-key
state (`runtime.rs:115` `apply_event`). An offload result is a one-shot, arbitrary-`T`
value keyed by a UUID — no Event, no Default, no fold. The `projection_snapshots` table
(composite PK, `state: JsonValue`) fits directly. Accepted coherence-tax expansion of
`ferro-projection`, mirroring 244's expansion of ferro-queue registration.

---

## Handle-key propagation to the worker

| Option | Description | Selected |
|--------|-------------|----------|
| Mint key before dispatch, carry it to the worker as job metadata | Payload params unchanged; worker persists under the same key the caller holds | ✓ |
| Reuse the ferro-queue job/row id as the handle key | Contradicts 245 D-07 (fresh UUID, decoupled from job identity) | |
| Add a `__offload_handle` field to the derived Job struct | Pollutes the serializable payload (244 D-11: payload = method params only) | |

**Auto-selected:** Mint-before-dispatch + carry as enqueue metadata (recommended).
**Notes:** Today `.offload()` (`offload.rs:118`) mints the key AFTER dispatch and drops
it — the worker cannot know it. This is the load-bearing seam 246 fixes. Exact ferro-queue
carrier mechanism (row column vs. `Job::handle` context slot) left to planning; the locked
contract is caller-key == worker-write-key, payload fields unchanged.

---

## Result envelope shape (status discriminator)

| Option | Description | Selected |
|--------|-------------|----------|
| Tagged `{status: completed, value}` / `{status: failed, error}`; no pending row | Row appears only at terminal outcome; retrieval deserializes `value` back to `Output` | ✓ |
| Write a `pending` row at enqueue, transition to completed/failed | Distinguishes unknown-handle from not-done, but is a 247 subscription-lifecycle concern | |

**Auto-selected:** Tagged envelope, completion-only (recommended).
**Notes:** SC#2 requires retrievable-after-completion only. Reading a not-yet-finished
handle returns `None` (absent row), like `ProjectionRuntime::read`. Pending marker deferred
to 247.

---

## Terminal-error semantics vs. retries

| Option | Description | Selected |
|--------|-------------|----------|
| Success on `handle()` Ok; terminal error on `Job::failed()` (retries exhausted) | Maps the two outcomes onto the two existing ferro-queue seams; transient-then-success ends completed | ✓ |
| Write an error snapshot on every failed attempt | Would overwrite/thrash across retries and misreport transient failures as terminal | |

**Auto-selected:** Success in `handle()`, terminal error in `failed()` (recommended).
**Notes:** 244 D-07 already maps method `Err` → job failure (retry + `failed()`). 246
overrides the derived `failed()` to persist the terminal-error envelope; panics route to the
same `failed()` seam (SC#3 "no silent drop"). Error message reuses the 244 `Display`-string.

---

## Write-back glue location (crate dependency direction)

| Option | Description | Selected |
|--------|-------------|----------|
| Glue in `framework`; add `ferro-projection` dep; `handle()` calls `::ferro::offload::*` | Keeps ferro-queue free of the broadcast/events stack; matches `::ferro::*`-only emission | ✓ |
| Glue in `ferro-queue`; ferro-queue depends on `ferro-projection` | Couples the queue to ferro-events + ferro-broadcast (a heavier, unwanted dependency) | |

**Auto-selected:** Glue in `framework` (recommended).
**Notes:** `framework` depends on `ferro-projections` (plural) but not `ferro-projection`
(singular) — 246 adds it (`Cargo.toml:53`). The helper takes its DB from ferro-queue's
`Queue::connection()` (`worker.rs:247`) — the same DB the worker uses; no new connection
wiring.

---

## Reserved projection name + key convention

| Option | Description | Selected |
|--------|-------------|----------|
| Single reserved name `"offload.result"`, key = handle UUID | Handle UUID is globally unique; 247 channel = `projection.offload.result.{handle}` | ✓ |
| Per-method namespace `offload.result.<Trait>.<method>`, key = handle | Redundant — the UUID already disambiguates; complicates the subscription key | |

**Auto-selected:** Single reserved name, key = handle (recommended).

---

## Claude's Discretion

- Exact ferro-queue carrier for the handle key (row column vs. `Job::handle` context slot).
- Shape/name of the new `ferro-projection` direct-write surface and its versioning on first write.
- Envelope tag/field names (`status`/`value`/`error` vs. an internally-tagged serde enum).
- Name/module of the `::ferro::offload::*` write-back helpers and the read-back wrapper;
  whether the read surface attaches to `OffloadHandle<T>` now or waits for 247.
- Panic-capture detail (whether ferro-queue already converts a `handle()` panic to `failed()`).

## Deferred Ideas

- Shared broadcast transport (246.1); delta streaming + handle resolve/subscribe (247);
  pending/enqueued row (247); deduped-job/multiple-handle reconciliation (246/247);
  deployable `worker` runtime context (248); ferro-mcp introspection + docs (249);
  snapshot retention/TTL for completed results (future operational).
