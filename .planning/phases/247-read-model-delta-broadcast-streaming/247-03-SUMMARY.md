---
phase: 247-read-model-delta-broadcast-streaming
plan: "03"
subsystem: framework/offload
tags: [offload, broadcast, projection, resolve, integration-test, docs]
completed: "2026-08-14"
duration: "837s"

dependency_graph:
  requires:
    - "247-01 (OffloadResult::Pending, persist_pending, read_result_redacted)"
    - "247-02 (OFFLOAD_BROADCASTER, broadcast_delta, register_offload_hooks_with_broadcaster, enqueue_and_mark_pending)"
    - "ferro-broadcast (Broadcaster, InMemoryTransport, ServerMessage, with_transport)"
  provides:
    - "ResolveError — framework::offload::ResolveError"
    - "resolve() — framework::offload::resolve (race-safe subscribe-first helper)"
    - "framework/tests/offload_delta_broadcast.rs — four integration scenarios + env-gated redis variant"
    - "docs/src/features/queues.md — Subscribe and await an offloaded result section (SC#3)"
  affects:
    - "248 (deployable worker calls enqueue_and_mark_pending; resolve() is the client-side contract)"
    - "249 (ferro-mcp introspection docs reference the documented client pattern)"

tech_stack:
  added:
    - "redis-transport feature in framework/Cargo.toml (forwards to ferro-broadcast/redis-transport)"
  patterns:
    - "subscribe-first / read-back-once / await-delta order (D-09) — eliminates TOCTOU race"
    - "uuid v4 per-call client id — prevents resolve() collisions on the same handle"
    - "Channel-closed final read — degrades gracefully when broadcaster drops after completion"
    - "Single #[tokio::test] for all four scenarios — avoids Queue/OFFLOAD_BROADCASTER OnceLock races"

key_files:
  created:
    - "framework/tests/offload_delta_broadcast.rs"
  modified:
    - "framework/src/offload.rs"
    - "framework/Cargo.toml"
    - "docs/src/features/queues.md"

decisions:
  - "resolve() is a free function in framework/src/offload.rs, not a method on OffloadHandle — keeps ferro-queue dep-free of ferro-broadcast/ferro-projection (D-11)"
  - "Channel-closed path performs a final read_result attempt before returning ChannelClosed — allows completion-then-remove sequences to succeed"
  - "All four integration scenarios run inside one #[tokio::test] — mirrors offload_result_round_trip.rs; avoids OnceLock initialization races across concurrent test functions"
  - "redis-transport feature added to framework/Cargo.toml forwarding to ferro-broadcast — satisfies check-cfg; matches how 246.1 exposed it"
  - "Disk at 99% capacity — scoped test gate to cargo test -p ferro-rs (all scenarios pass); full --all-features gate deferred to operator with df note"

metrics:
  duration: "837s"
  completed: "2026-08-14"
  tasks_completed: 3
  files_modified: 4
---

# Phase 247 Plan 03: resolve(), Integration Suite, and queues.md Summary

**Race-safe `resolve()` helper, four-scenario integration suite proving the multi-replica broadcast loop, and the `queues.md` subscribe-then-await documentation section (SC#3).**

## What Was Built

### Task 1: `ResolveError` and `resolve()` (D-09)

Added to `framework/src/offload.rs`:

```rust
pub enum ResolveError { Projection(ProjectionError), Broadcast(String), ChannelClosed, Timeout }

pub async fn resolve<T: OffloadSerializable>(
    handle: &ferro_queue::OffloadHandle<T>,
    broadcaster: &Arc<ferro_broadcast::Broadcaster>,
    db: &DatabaseConnection,
    timeout: Option<std::time::Duration>,
) -> Result<OffloadResult<T>, ResolveError>
```

The three-step order (subscribe → read-back → await) is enforced in the implementation:

1. `broadcaster.add_client` + `broadcaster.subscribe` — buffers any delta that fires before the read-back
2. `read_result::<T>(key, db)` — short-circuits an already-terminal handle (returns immediately, no delta needed)
3. `rx.recv()` loop — awaits the delta; on receipt, reads the authoritative snapshot via `read_result`

Channel-closed path: if the mpsc receiver closes before a delta arrives, performs a final `read_result` attempt (in case the result landed and `remove_client` fired) before returning `ChannelClosed`.

`timeout: None` waits indefinitely; `Some(d)` wraps with `tokio::time::timeout`.

The client id is `format!("{}-resolve-{}", handle.key(), uuid::Uuid::new_v4())` — globally unique per call (Open Question #2 resolved per RESEARCH.md).

### Task 2: Integration suite (D-12)

Created `framework/tests/offload_delta_broadcast.rs` with four named scenario functions called from one `#[tokio::test]`:

| Scenario fn | VALIDATION row | What it proves |
|-------------|---------------|----------------|
| `cross_replica_delta` | OFFLOAD-04 SC#1 | Delta from Broadcaster A (worker) reaches a client on Broadcaster B (client) over `InMemoryTransport` |
| `request_returns_before_worker` | OFFLOAD-04 SC#2 | `enqueue_and_mark_pending` returns in < 500 ms; `Some(Pending)` snapshot exists before drain; `Some(Completed)` after drain |
| `offload_failed_delta_is_redacted` | OFFLOAD-04 D-05 | `data.get("error").is_none()` on failed delta; `"sensitive-secret-value"` absent from delta payload; present in `read_result` snapshot |
| `resolve_already_complete` | OFFLOAD-04 D-09 | `resolve()` returns `Completed` via read-back short-circuit; no delta needed |

The multi-replica shape: `bus = Arc::new(InMemoryTransport::new(64))`, `broadcaster_a = Broadcaster::new().with_transport(bus.clone())` (worker), `broadcaster_b = Broadcaster::new().with_transport(bus)` (client). `with_transport` appears 4 times in the file (two broadcasters per replica pair, redis module adds another pair).

The env-gated `redis_cross_replica` test lives in `#[cfg(feature = "redis-transport")] mod redis_tests` — compiles out without the feature, skips at runtime when `REDIS_URL` is unset.

### Task 3: queues.md docs (SC#3)

Added section "Subscribe and await an offloaded result" to `docs/src/features/queues.md` covering:

- Channel convention: `projection.offload.result.{handle_key}`
- Request side: `enqueue_and_mark_pending` (non-blocking, pending snapshot written immediately)
- Server-side consumer: `resolve()` with the three-step order explained in prose
- Browser / client-side: `read_result_redacted` for the redacted read-back
- Delta payload table (completed carries value; failed carries no `error` field)
- `CreateProjectionSnapshotsTable` migration requirement
- Capability token model (unguessable UUID v4 handle key, D-11)

## Task Commits

| Task | Description | Commit |
|------|-------------|--------|
| 1 | ResolveError + resolve() helper (D-09) | `52ba6905` |
| 2 | Integration suite + redis-transport feature (D-12) | `6fb60884` |
| 3 | queues.md subscribe-then-await section (SC#3) | `f551a8c7` |

## Deviations from Plan

### [Deviation] All four scenarios in one test function instead of four separate `#[tokio::test]`

- **Found during:** Task 2 implementation
- **Issue:** `Queue::init` uses a `OnceLock` that accepts only one call per process. `OFFLOAD_BROADCASTER` is also a `OnceLock`. When four separate `#[tokio::test]` functions run (even with `#[serial_test::serial]`), the second through fourth tests fail with "Queue already initialized" because they each call `Queue::init` and the temp file from the first test gets dropped (making the DB read-only for later tests).
- **Fix:** Mirror the exact pattern from `offload_result_round_trip.rs` — one `#[tokio::test]` function that calls four named scenario functions in sequence with `clear_tables` between them. The four function names (`cross_replica_delta`, `request_returns_before_worker`, `offload_failed_delta_is_redacted`, `resolve_already_complete`) are preserved in the file and bound to their VALIDATION rows via doc comments.
- **Impact:** The test binary reports one test (`offload_delta_broadcast_suite`) rather than four. All four scenario names still appear in the file (satisfying the `contains: "cross_replica_delta"` grep check). The functional coverage is identical.
- **Files modified:** `framework/tests/offload_delta_broadcast.rs`

### [Deviation] `redis-transport` feature added to framework/Cargo.toml

- **Found during:** Task 2/3 (the `#[cfg(feature = "redis-transport")]` triggered `unexpected_cfg` warning, which would fail CI `-D warnings`)
- **Fix:** Added `redis-transport = ["ferro-broadcast/redis-transport"]` to `framework/Cargo.toml`. This is additive, not a backward-compat shim.
- **Files modified:** `framework/Cargo.toml`

### [Note] Full `cargo test --all-features` gate scoped to `cargo test -p ferro-rs`

- **Reason:** Disk at 99% capacity (6.4 GiB free). `cargo test --all-features` builds every crate with all features and recurrently ENOSPC-fails on this machine. The full fmt + clippy (`--all --all-targets`) gate was run. The test gate was scoped to `cargo test -p ferro-rs` (the changed crate), which ran all unit tests + all integration tests including the new suite. No failures.
- **Recommended operator action:** Run `cargo test --all-features` after freeing disk space (clean `target/` of unused build artifacts).

## Threat Surface Scan

All mitigations from the plan's `<threat_model>` are implemented:

| Threat ID | Disposition | Verification |
|-----------|-------------|--------------|
| T-247-info-disclosure | mitigate | `offload_failed_delta_is_redacted` asserts `data.get("error").is_none()` AND the raw string absent from delta AND present in `read_result` snapshot |
| T-247-hostile-payload | mitigate | Inherited from 246.1 — `RedisTransport` uses strict `serde_json` parse; no new parse surface in Phase 247 |
| T-247-resolve-wakeup | mitigate | `resolve()` treats the delta only as a wakeup and reads the authoritative snapshot (`read_result`) on wake — a forged delta triggers only a DB read, never a fabricated value |
| T-247-handle-enum | accept | Documented accepted caveat (D-11 capability model); restated in queues.md |

No new threat surface beyond the plan's threat model.

## Known Stubs

None. All three additions are fully wired:
- `resolve()` calls real `broadcaster.subscribe`, real `read_result`, real `rx.recv`.
- The integration suite exercises the real drain loop, real hook, real snapshot writes.
- The docs section references real public API paths (`::ferro::offload::resolve`, `read_result_redacted`, `enqueue_and_mark_pending`).

## Self-Check: PASSED

| Item | Result |
|------|--------|
| `framework/src/offload.rs` contains `pub enum ResolveError` | FOUND |
| `framework/src/offload.rs` contains `pub async fn resolve` | FOUND |
| subscribe-first order: `broadcaster.subscribe` before `read_result` before `rx.recv` | FOUND |
| `matches!(result, OffloadResult::Pending)` short-circuit guard | FOUND |
| `framework/tests/offload_delta_broadcast.rs` exists | FOUND |
| All four scenario fn names in file | FOUND |
| `with_transport` appears >= 2 times | FOUND (4 times) |
| `data.get("error").is_none()` assertion | FOUND |
| `"sensitive-secret-value"` only in snapshot assertion | FOUND |
| `redis_cross_replica` and `REDIS_URL` in test file | FOUND |
| `docs/src/features/queues.md` contains `read_result_redacted` | FOUND |
| `docs/src/features/queues.md` contains `projection.offload.result` | FOUND |
| `docs/src/features/queues.md` contains `enqueue_and_mark_pending` | FOUND |
| Commit `52ba6905` (Task 1) | FOUND |
| Commit `6fb60884` (Task 2) | FOUND |
| Commit `f551a8c7` (Task 3) | FOUND |
| `cargo test -p ferro-rs --test offload_delta_broadcast` exit 0 | PASSED |
| `cargo fmt --all -- --check` exit 0 | PASSED |
| `cargo clippy --all --all-targets -- -D warnings` exit 0 | PASSED |
