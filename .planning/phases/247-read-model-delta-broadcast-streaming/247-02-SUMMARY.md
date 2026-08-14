---
phase: 247-read-model-delta-broadcast-streaming
plan: "02"
subsystem: framework/offload
tags: [offload, queue, broadcast, projection, hooks, pending, delta, redaction]
completed: "2026-08-14"
duration: "259s"

dependency_graph:
  requires:
    - "247-01 (OffloadResult::Pending, persist_pending, read_result_redacted, persist_result_raw)"
    - "ferro-broadcast (Broadcast builder, Broadcaster, OnceLock static pattern)"
    - "ferro-queue (OffloadResultHook fn-pointer type, register_offload_result_hook, Offloadable::offload)"
  provides:
    - "OFFLOAD_BROADCASTER static — OnceLock<Arc<Broadcaster>> read by the fn-pointer result hook"
    - "broadcast_delta() — private async helper: best-effort send on projection.offload.result.{handle}"
    - "register_offload_hooks_with_broadcaster(Arc<Broadcaster>) — persist-then-broadcast result hook"
    - "enqueue_and_mark_pending<J>(job, db) — request-side wrapper: offload() + persist_pending"
    - "app.rs bootstrap branch — App::get::<Broadcaster>() selects broadcaster-aware vs persist-only hook"
  affects:
    - "247-03 (integration tests consume all five artifacts above)"
    - "248 (deployable worker calls ::ferro::offload::enqueue_and_mark_pending)"

tech_stack:
  added: []
  patterns:
    - "OnceLock<Arc<T>> module-level static as fn-pointer-compatible broadcaster slot (D-03 Option A, mirrors TENANT_ID_HOOK)"
    - "Persist-then-broadcast order with best-effort broadcast (D-02, mirrors ferro-projection/src/runtime.rs:158-199)"
    - "Redacted delta: Err arm builds {status:failed} before consuming outcome, never includes raw error (D-05)"
    - "Framework enqueue wrapper: job.offload().await? then persist_pending — zero changes to ferro-queue (D-08 Option B)"

key_files:
  created: []
  modified:
    - "framework/src/offload.rs"
    - "framework/src/app.rs"

key_decisions:
  - "D-03 Option A chosen: OFFLOAD_BROADCASTER OnceLock static (fn-pointer hook cannot close over Arc<Broadcaster>)"
  - "D-08 Option B chosen: enqueue_and_mark_pending framework wrapper, zero ferro-queue changes"
  - "Delta built from &outcome before outcome is moved into the persist match — avoids double-borrow of a consumed Result"
  - "Persist failure aborts broadcast (return early): no delta for a snapshot that never landed"
  - "App::get::<Broadcaster>() returns owned Broadcaster (Clone), so Arc::new(broadcaster) is correct"

requirements-completed: [OFFLOAD-04]

metrics:
  duration: "259s"
  completed: "2026-08-14"
  tasks_completed: 3
  files_modified: 2
---

# Phase 247 Plan 02: Broadcaster-aware result hook, broadcast_delta, and enqueue_and_mark_pending Summary

**Offload result hook extended to persist-then-broadcast a redacted delta on `projection.offload.result.{handle}` via a module-level `OnceLock<Arc<Broadcaster>>` static; `enqueue_and_mark_pending` wires the pending snapshot at the request-side enqueue call; bootstrap branches on `App::get::<Broadcaster>()` to select the broadcaster-aware or persist-only hook.**

## Performance

- **Duration:** 259s (~4 min)
- **Started:** 2026-08-14T14:33:31Z
- **Completed:** 2026-08-14T14:37:50Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments

- `OFFLOAD_BROADCASTER` static and `register_offload_hooks_with_broadcaster` close the broadcast half of the OFFLOAD-04 loop: when the worker finishes, a delta lands on `projection.offload.result.{handle}` immediately after the snapshot persists.
- `broadcast_delta` helper encapsulates the channel format (`projection.offload.result.{handle}`), event name (`"offload.result"`), and the best-effort/warn-and-swallow contract so Plan 03's test harness can assert against the exact strings.
- `enqueue_and_mark_pending` gives consumers a single call that enqueues and marks pending — the request stays non-blocking because `job.offload()` dispatches without awaiting the result.
- Bootstrap in `app.rs` auto-selects the broadcaster-aware hook when `Broadcaster` is in the container and falls back to persist-only otherwise — no consumer configuration change required.

## Task Commits

1. **Task 1: OFFLOAD_BROADCASTER static, broadcast_delta, register_offload_hooks_with_broadcaster** — `ad025f04` (feat)
2. **Task 2: enqueue_and_mark_pending framework wrapper** — `3422b120` (feat)
3. **Task 3: Wire broadcaster-aware hook at bootstrap** — `1c22851c` (feat)

## Files Created/Modified

- `framework/src/offload.rs` — Added `OFFLOAD_BROADCASTER` static (line 48), `broadcast_delta` private async helper, `register_offload_hooks_with_broadcaster`, and `enqueue_and_mark_pending`. All existing functions and tests unchanged.
- `framework/src/app.rs` — Replaced bare `register_offload_hooks()` call at the former line 419 with an `App::get::<Broadcaster>()` branch selecting the broadcaster-aware or persist-only hook.

## Key Signatures (for Plan 03 test harness)

```rust
// Channel and event strings — assert these exactly in integration tests
let channel = format!("projection.{}.{}", OFFLOAD_PROJECTION_NAME, handle_key);
// == "projection.offload.result.{handle}"
let event = "offload.result";

// Delta payloads
// Completed: {"status":"completed","value":<v>}
// Failed:    {"status":"failed"}   <-- NO error field (D-05)

// Bootstrap acquires Broadcaster as an owned clone (Broadcaster: Clone)
// then wraps in Arc:
match crate::App::get::<ferro_broadcast::Broadcaster>() {
    Some(broadcaster) => register_offload_hooks_with_broadcaster(Arc::new(broadcaster)),
    None             => register_offload_hooks(),
}

// Enqueue wrapper signature
pub async fn enqueue_and_mark_pending<J>(
    job: J,
    db: &DatabaseConnection,
) -> Result<ferro_queue::OffloadHandle<J::Output>, ferro_queue::Error>
where
    J: ferro_queue::Offloadable;
```

## Decisions Made

- **D-03 Option A** — `OnceLock<Arc<Broadcaster>>` static. The `OffloadResultHook` type is a bare `fn` pointer that cannot capture heap state; the static mirrors the `TENANT_ID_HOOK` pattern already in `ferro-queue/src/dispatcher.rs`.
- **D-08 Option B** — Framework wrapper around `Offloadable::offload()`. No changes to `ferro-queue` required; `enqueue_and_mark_pending` is the canonical request-side entrypoint Phase 248's deployable worker will also call.
- **Delta built before consuming `outcome`** — `let delta = match &outcome { ... }` uses a shared reference so the value can be moved into the persist match on the next line without a borrow conflict.
- **Persist failure aborts broadcast** — If the snapshot write fails, the hook returns early without broadcasting. Broadcasting a delta that no snapshot backs would mislead a read-back on the authoritative store (D-02 / D-06).
- **`Arc::new(broadcaster)` at the call site** — `App::get::<Broadcaster>()` clones the stored `Broadcaster` (not an `Arc<Broadcaster>`); wrapping it in `Arc::new` at the bootstrap site matches the `register_offload_hooks_with_broadcaster` signature without changing the container's stored type.

## Deviations from Plan

None — plan executed exactly as written. All three tasks implemented verbatim per the plan's `<action>` blocks; no bugs, missing functionality, or blocking issues were encountered.

## Issues Encountered

None.

## Threat Surface Scan

No new network endpoints or auth paths introduced. The plan's threat model covers all relevant surfaces:

| Threat ID | Disposition | Status |
|-----------|-------------|--------|
| T-247-info-disclosure | mitigate | `Err` delta arm is `json!({ "status": "failed" })` — no `error` field; `persist_error` still stores the raw error in the snapshot only |
| T-247-hook-failfail | mitigate | Hook swallows both persist and broadcast errors (`warn!`, returns `()`); broadcast failure cannot trigger a job retry |
| T-247-handle-enum | accept | Capability model unchanged; no new control this plan |
| T-247-hostile-payload | mitigate | Inherited from 246.1 — delta rides the existing `ServerMessage::Event` fan-out; no new bus-parsing surface |

## Known Stubs

None. All five artifacts are fully wired:
- `OFFLOAD_BROADCASTER.get()` reads a real `Arc<Broadcaster>` set at bootstrap.
- `broadcast_delta` calls `ferro_broadcast::Broadcast::new(..).send().await`.
- `register_offload_hooks_with_broadcaster` registers a real hook via `ferro_queue::register_offload_result_hook`.
- `enqueue_and_mark_pending` calls real `job.offload().await?` and real `persist_pending`.
- Bootstrap branch reads the real container.

Integration assertions (Plan 03) will prove the full loop end-to-end.

## Next Phase Readiness

Plan 247-03 integration tests can now assert:
- A completed worker result delivers `ServerMessage::Event` on `projection.offload.result.{handle}` with `event == "offload.result"` and `data["status"] == "completed"`.
- A failed worker result delivers `data == {"status":"failed"}` with no `data["error"]` key.
- `data["error"]` is absent from failed broadcast deltas (T-247-info-disclosure D-05 assertion).
- The pending snapshot exists after `enqueue_and_mark_pending` returns (SC#2 non-blocking proof).
- The multi-replica shape (Broadcaster A worker → InMemoryTransport → Broadcaster B subscriber) delivers the delta across replicas.

## Self-Check: PASSED

| Item | Result |
|------|--------|
| `framework/src/offload.rs` exists | FOUND |
| `framework/src/app.rs` exists | FOUND |
| `static OFFLOAD_BROADCASTER` at line 48 | FOUND |
| `pub fn register_offload_hooks_with_broadcaster` at line 316 | FOUND |
| channel format `format!("projection.{}.{}", OFFLOAD_PROJECTION_NAME, handle_key)` | FOUND (line 284) |
| event literal `"offload.result"` | FOUND (line 287) |
| Failed delta arm `json!({ "status": "failed" })` with no `error` key | FOUND (line 323) |
| `persist_error(&key, &msg, db)` still called in broadcaster hook | FOUND (line 328) |
| `pub async fn enqueue_and_mark_pending` | FOUND (line 361) |
| Body: `job.offload().await?` then `persist_pending(handle.key(), db)` | FOUND (lines 368-369) |
| `app.rs` branches on `App::get::<Broadcaster>()` | FOUND (line 425) |
| `register_offload_hooks_with_broadcaster` in `app.rs` | FOUND (line 428) |
| `None => register_offload_hooks()` fallback in `app.rs` | FOUND (line 432) |
| Commit `ad025f04` (Task 1) | FOUND |
| Commit `3422b120` (Task 2) | FOUND |
| Commit `1c22851c` (Task 3) | FOUND |
| All 7 offload unit tests pass | PASSED |
| 521 ferro-rs unit tests pass, 0 failed | PASSED |
| `cargo build -p ferro-rs` exit 0 | PASSED |
| `cargo clippy -p ferro-rs --all-targets -D warnings` exit 0 | PASSED |
