# Phase 247: Read-model delta → broadcast streaming - Context

**Gathered:** 2026-08-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Close the fire-and-forward loop for `#[offload]`. When an offloaded worker persists its
result snapshot (Phase 246), also emit a `ferro-broadcast` delta on the handle's channel so a
client subscribed to the handle receives the result live — while the originating request has
already returned (non-blocking).

In scope: the broadcast emission at the result-persist seam; a pending marker at enqueue; a
race-safe resolve helper on `OffloadHandle`; a client-facing redaction-aware read-back helper;
documentation of the subscribe-then-await pattern; and integration tests proving the loop
(including the multi-replica shape).

Out of scope: the deployable `ferro worker` runtime (Phase 248), MCP introspection + the
scaling-model docs (Phase 249), any new broadcast transport (shipped in 246.1), private-channel
authorization (deferred), and a built-in framework read-back route (deferred).

**Requirement:** OFFLOAD-04. **Depends on:** Phases 246 (result snapshot) and 246.1 (shared
transport — without it the loop closes only at a single replica).

</domain>

<decisions>
## Implementation Decisions

### Broadcast seam — where the delta is emitted
- **D-01:** The result delta is emitted from the **framework offload layer**, at the same
  result-persist point that Phase 246 established (`framework::register_offload_hooks`, which
  today calls `persist_result_raw`/`persist_error`). `ferro-projection::snapshot_write` stays
  **write-only** — no Broadcaster is added to the direct snapshot API, preserving 246 D-01's
  deliberate decoupling of the direct store from broadcast. This also honors D-11
  (`ferro-queue` must not depend on `ferro-projection`); the framework layer already owns
  `OFFLOAD_PROJECTION_NAME` and the `ferro-broadcast` dependency, so it is the correct home.
- **D-02:** The persist-then-broadcast order mirrors the event-fold path
  (`ferro-projection/src/runtime.rs:158` upsert → `:168` broadcast): **persist the snapshot
  first, then broadcast**. A broadcast failure does **not** roll back the snapshot and does not
  fail the job — log at `tracing::warn!` and continue (consistent with 246.1 D-06 and the
  projection path's best-effort broadcast). The snapshot remains the authoritative record;
  subscribers reconcile by reading it back.
- **D-03 (planning):** The exact mechanism by which the worker-side `Broadcaster` reaches the
  result-persist hook — extending the hook signature (`register_offload_result_hook`) to carry
  a Broadcaster/broadcast sink vs. a worker-execution-context slot alongside the existing `db`
  — is a **planning/research decision**, mirroring how 246 D-05 left the exact `ferro-queue`
  key-propagation mechanism to planning. The locked contract: the delta is published by the
  framework layer, through the same `Broadcaster` (and therefore the same 246.1 shared
  transport) the app is configured with.

### Delta payload + error redaction
- **D-04:** The broadcast delta is published on channel **`projection.offload.result.{handle}`**
  — the convention already reserved by 246 (`(OFFLOAD_PROJECTION_NAME, handle_key)` →
  `projection.offload.result.{handle}`), matching the event-fold channel format
  `projection.{name}.{key}` so it reuses the existing subscribe path unchanged.
- **D-05:** A **completed** delta carries the result value (so a subscribed client gets the
  answer in one message, no mandatory HTTP round-trip). A **failed** delta carries the status
  plus a **non-sensitive terminal marker only** — the raw `Display`-stringified error is **not**
  broadcast to clients. The raw error remains in the snapshot `state` and the worker logs for
  authorized/in-process retrieval. This resolves the security note carried forward from Phase
  246 (`framework/src/offload.rs`: "Phase 247 must sanitize before any client-facing exposure").
- **D-06:** The delta is the **wakeup + client-facing payload**; the **snapshot is the
  authoritative store**. A server-side consumer with DB access reads the snapshot back on wake
  for the full envelope (including the raw error, since it is in-process/authorized); a browser
  client relies on the redacted delta + a redacted read-back (D-10). This split is what lets the
  delta be safely redacted without losing server-side diagnostic fidelity.

### Subscribe/completion race + pending marker
- **D-07:** A **`{status:"pending"}` snapshot is written at enqueue**, keyed by the handle.
  Phase 246 D-08 explicitly deferred this "pending marker + unknown-handle-vs-not-done
  distinction" to Phase 247. The pending row lets a read-back distinguish an **unknown handle**
  (no row) from **work not finished yet** (pending row), and lets the resolve helper fail fast
  on a bad handle. This extends 246's envelope with a third tag alongside `completed`/`failed`.
- **D-08:** The pending write happens on the **request/enqueue side and must go through the
  framework layer**, not `ferro-queue` (D-11: `ferro-queue` cannot depend on `ferro-projection`).
  The exact seam — a symmetric on-enqueue hook mirroring the result hook, vs. a framework
  offload wrapper around `.offload()` — is a **planning decision**. Locked contract: the pending
  snapshot exists after `.offload()` returns, under the same handle key the caller holds.
- **D-09:** `OffloadHandle<T>` gains a **race-safe resolve helper** (delivering the `.subscribe()`
  / `.await`-style resolve methods that 245 D-08 explicitly deferred to this phase). It
  encapsulates the correct order — **subscribe to the channel first, read the snapshot back once
  (catch an already-completed/failed handle), then await the delta** — so consumers cannot get
  the race wrong. On wake it reads the snapshot for the authoritative `OffloadResult<T>`
  envelope (D-06). Timeout/return semantics are Claude's discretion (see below).

### Client-facing read-back
- **D-10:** Ship a **redaction-aware read helper**, e.g. `framework::offload::read_result_redacted`,
  that mirrors the delta's redaction (value on success, non-sensitive marker on failure), for
  the browser read-back leg of the subscribe→read-back→await pattern. The existing
  `read_result` (full envelope, raw error) is retained for authorized/server-side use. The
  framework stays **route-agnostic** (ferro convention): the app mounts its own route calling
  the helper. A built-in framework read-back route is **deferred** (see Deferred Ideas).

### Channel privacy
- **D-11:** The channel is a **public channel keyed by the unguessable UUID v4 handle**
  (capability model), consistent with the existing public `projection.*` channel naming. The
  handle is minted server-side (245 D-07) and returned only to the caller, so it functions as a
  capability token. Accepted caveat: a leaked handle (logs, referrer) exposes that one result.
  A **private-authorized channel** (`private-projection.offload.result.{handle}` gated by the
  broadcast authorizer) is **deferred** — it requires handle→owner metadata on the snapshot and
  an app authorizer, which is meaningfully more scope than this phase.

### Test scope
- **D-12:** Integration tests assert the offload loop in the **multi-replica shape the phase
  exists for**: a worker persists on Broadcaster A → a subscriber attached to Broadcaster B (via
  the 246.1 **in-memory** transport) receives the **redacted** delta on
  `projection.offload.result.{handle}`. Plus **SC#2**: the originating request returns before
  the worker finishes (non-blocking, asserted). Plus an **env-gated live-redis** cross-process
  variant matching 246.1's redis integration-test style (`246.1-02-PLAN.md`). The single-process
  path is covered as the degenerate case of the cross-process harness.

### Claude's Discretion
- The exact delta **event-name string** (recommend a fixed `"offload.result"`; the fold path's
  `broadcast_event_name()` is event-fold-specific and does not apply to a one-shot result).
- The **resolve-helper signature + timeout semantics** (recommend a bounded await with an
  optional timeout; note that a terminally failed job records a `failed` snapshot + delta per
  246 D-09/D-10, so the only unbounded wait is a job that never runs).
- The **module home** for `read_result_redacted` and the resolve helper (keep generated/
  consumer paths resolving via `::ferro::offload::*`, per the 244/245/246 convention).
- The precise **plumbing** for D-03 (Broadcaster → result hook) and D-08 (pending write seam).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Offload design (authoritative spec)
- `docs/superpowers/specs/2026-06-24-offload-work-distribution-design.md` §"Result path
  (fire-and-forward)" (lines ~75–88) — the offload→queue→worker→snapshot→delta loop; the handle
  is the projection key the client subscribes to.
- `docs/superpowers/specs/2026-06-24-offload-work-distribution-design.md` §"Prerequisite:
  multi-replica broadcast" (lines ~90–105) — why 246.1 is a hard dependency of this phase.
- `docs/superpowers/specs/2026-06-24-offload-work-distribution-design.md` §"Phase decomposition"
  item 4 (lines ~194–196) — "Read-model delta → broadcast streaming; document the subscribe/
  await client pattern."

### Roadmap + requirements
- `.planning/ROADMAP.md` §"Phase 247: Read-model delta → broadcast streaming" — goal + the three
  success criteria (delta on completion; non-blocking request; documented client pattern).
- `.planning/REQUIREMENTS.md` OFFLOAD-04 — the requirement this phase satisfies.

### Prior-phase decisions this phase builds on
- `.planning/phases/245-typed-result-handle-serializable-enforcement/245-CONTEXT.md` D-07
  (fresh-UUID handle key), D-08 (`OffloadHandle` inert; resolve methods deferred to 247), and
  the Discretion note (handle is `Serialize` so it travels to the client as the subscription key).
- `.planning/phases/246-result-read-model-snapshot/246-CONTEXT.md` D-01/D-02/D-03 (direct
  snapshot API, decoupled from the fold path), D-04/D-05/D-06 (handle-key propagation), D-07
  (result envelope), **D-08 (pending marker + unknown-vs-not-done explicitly deferred here)**.
- `.planning/phases/246.1-shared-transport-broadcast-fan-out-for-multi-replica-delta-d/246.1-CONTEXT.md`
  D-01/D-02 (transport trait + in-process default), D-03 (origin-id loop prevention), D-04
  (fan out `ServerMessage::Event` — the projection-delta path), D-06 (best-effort publish).

### Docs home for the client pattern (SC#3)
- `docs/src/features/queues.md` — where the offload serializable contract was documented (245-03);
  the subscribe-then-await pattern documentation belongs here.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `framework/src/offload.rs` — `register_offload_hooks()` (the injection seam for the delta
  emission, D-01), `persist_result_raw`/`persist_error`, `read_result` (retain for server-side),
  `OffloadResult<T>` (extend with a `pending` tag for D-07), `OFFLOAD_PROJECTION_NAME` (the
  channel-name source).
- `ferro-queue` offload types — `register_offload_result_hook` (D-03 candidate seam),
  `OffloadHandle<T>` + `HandleKey` (add the resolve helper, D-09), `Offloadable::offload()`
  (the enqueue entrypoint the pending write must hang off, D-08).
- `ferro-projection` — `snapshot_write`/`snapshot_read` (direct API, stays write-only per D-01),
  `ProjectionKey`; the channel convention `projection.{name}.{key}` at `runtime.rs:168`.
- `ferro-broadcast` — `Broadcast::new(broadcaster).channel(..).event(..).data(..).send()`
  (`broadcast.rs`); `Broadcaster::with_transport()` (246.1) already fans `ServerMessage::Event`
  across processes — no transport work needed here.

### Established Patterns
- Persist-then-broadcast, broadcast-is-best-effort (`ferro-projection/src/runtime.rs:158–195`)
  is the template for D-02.
- The result-hook injection pattern (`register_offload_result_hook` registered by the framework)
  is the D-11-respecting way to cross the ferro-queue → ferro-projection/broadcast boundary; the
  pending write (D-08) should follow the same pattern.
- Env-gated live-redis integration test (`246.1-02-PLAN.md`) is the template for the D-12 redis
  cross-process variant; the WorkerLoop drain E2E (`246-05-PLAN.md`) is the template for the
  worker-completion harness.

### Integration Points
- Worker-side result persist (`framework::register_offload_hooks`) — add the delta emission.
- Enqueue path (`Offloadable::offload()`) — add the pending-snapshot write via the framework seam.
- `OffloadHandle<T>` — add the race-safe resolve helper.
- `docs/src/features/queues.md` — document the subscribe→read-back→await pattern (SC#3).

</code_context>

<specifics>
## Specific Ideas

- The delta and the snapshot play distinct roles: **delta = the live "it's ready" signal carrying
  a client-safe payload; snapshot = the authoritative, full-fidelity store.** The resolve helper
  treats a received delta as a wakeup and reads the snapshot for the authoritative result. This
  single idea reconciles redaction (D-05), server-side fidelity (D-06), and the subscribe race
  (D-09) coherently.
- The handle is treated as a **capability token** (D-11): unguessable UUID minted server-side,
  returned only to the caller. Public-channel-by-capability is the same trust model as an
  unguessable download URL.

</specifics>

<deferred>
## Deferred Ideas

- **Private-authorized result channel** — `private-projection.offload.result.{handle}` gated by
  the broadcast authorizer. Stronger for tenant-sensitive results, but needs handle→owner
  metadata on the snapshot and an app authorizer. Revisit when offload gains owner/tenant
  metadata on the handle.
- **Built-in framework read-back route** — a framework-registered handler returning the redacted
  result by handle, so the browser pattern needs zero app wiring. Deferred to keep 247
  route-agnostic; the app wires its own route over `read_result_redacted` (D-10) for now.

</deferred>

---

*Phase: 247-read-model-delta-broadcast-streaming*
*Context gathered: 2026-08-14*
