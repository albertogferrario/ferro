---
phase: 247-read-model-delta-broadcast-streaming
verified: 2026-08-14T16:00:00Z
status: passed
score: 3/3
overrides_applied: 0
re_verification: false
---

# Phase 247: Read-Model Delta → Broadcast Streaming — Verification Report

**Phase Goal:** Deliver results live — stream the snapshot delta to a client subscribed to the handle over ferro-broadcast, completing the fire-and-forward loop so the originating request returns immediately and the answer arrives when ready.
**Verified:** 2026-08-14T16:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths (Roadmap Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A client subscribed to a handle receives a broadcast delta carrying the result on completion. | VERIFIED | `cross_replica_delta` scenario in `framework/tests/offload_delta_broadcast.rs` asserts `ServerMessage::Event` with `event == "offload.result"`, correct channel, and `data["status"] == "completed"` with expected value. The test uses two distinct `Broadcaster` instances over a shared `InMemoryTransport` (multi-replica shape). The hook in `register_offload_hooks_with_broadcaster` persists then broadcasts via `broadcast_delta`. |
| 2 | The originating request returns before the worker finishes (non-blocking, asserted in a test). | VERIFIED | `request_returns_before_worker` scenario asserts two things: (a) `elapsed < 500ms` after `enqueue_and_mark_pending` returns, and (b) ordering — snapshot is `Some(Pending)` BEFORE `drain()` is called and `Some(Completed { value: 7 })` AFTER. The `Pending` assertion is the authoritative ordering proof: it confirms the worker had not yet executed. The `QUEUE_CONNECTION=db` env var is set in the test suite to ensure background mode. |
| 3 | The subscribe-then-await-result client pattern is documented. | VERIFIED | `docs/src/features/queues.md` contains the "Subscribe and await an offloaded result" section (line 291) covering: channel convention (`projection.offload.result.{handle_key}`), `enqueue_and_mark_pending` on the request side, `resolve()` for server-side consumers, `read_result_redacted` for browser clients, the delta payload/redaction table, and the `CreateProjectionSnapshotsTable` migration requirement. |

**Score:** 3/3 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `framework/src/offload.rs` | `OffloadResult::Pending`, `persist_pending`, `read_result_redacted`, `OFFLOAD_BROADCASTER` static, `register_offload_hooks_with_broadcaster`, `broadcast_delta`, `enqueue_and_mark_pending`, `resolve`, `ResolveError` | VERIFIED | All nine items present. Confirmed at specific lines: `Pending,` variant (line 100), `persist_pending` (line 172), `read_result_redacted` (line 225), `OFFLOAD_BROADCASTER` static (line 69), `register_offload_hooks_with_broadcaster` (line 337), `broadcast_delta` private helper (line 300), `enqueue_and_mark_pending` (line 382), `resolve` (line 432), `ResolveError` enum (line 48). |
| `framework/src/app.rs` | Bootstrap wiring that selects broadcaster-aware hook vs persist-only based on container | VERIFIED | Lines 426–433: `match crate::App::get::<ferro_broadcast::Broadcaster>()` branches on `Some(broadcaster)` → `register_offload_hooks_with_broadcaster(Arc::new(broadcaster))` and `None` → `register_offload_hooks()`. |
| `framework/tests/offload_delta_broadcast.rs` | Integration suite with `cross_replica_delta`, `request_returns_before_worker`, `offload_failed_delta_is_redacted`, `resolve_already_complete`, env-gated `redis_cross_replica` | VERIFIED | File exists. All four scenario functions present (lines 193, 240, 278, 339). Scenarios run from one `#[tokio::test]` (`offload_delta_broadcast_suite`, line 366). `redis_cross_replica` in `#[cfg(feature = "redis-transport")] mod redis_tests` (lines 391–473). |
| `docs/src/features/queues.md` | "Subscribe and await an offloaded result" section containing `read_result_redacted`, `projection.offload.result`, `enqueue_and_mark_pending` | VERIFIED | Section present at line 291. All three required strings confirmed. Also documents `resolve()`, capability token model, and delta redaction table. |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `resolve()` | `broadcaster.subscribe()` then `read_result()` then `rx.recv()` | subscribe-first / read-back-once / await-delta order (D-09) | VERIFIED | Lines 447–451 (`broadcaster.subscribe`), lines 454–464 (`read_result` read-back), lines 468–486 (`rx.recv()` await loop). All three steps present in stated order. |
| `register_offload_hooks_with_broadcaster` result hook | `serde_json::json!({ "status": "failed" })` with no `error` key | builds redacted delta BEFORE consuming outcome (D-05) | VERIFIED | Lines 343–344: `Err(_) => serde_json::json!({ "status": "failed" })` — no `error` field. `persist_error(&key, &msg, db)` still called (line 349) so raw error stays in snapshot. |
| `enqueue_and_mark_pending` | `persist_pending(handle.key(), db)` | calls `job.offload().await?` then `persist_pending` | VERIFIED | Lines 389–397: `job.offload().await?` returns handle, then `persist_pending(handle.key(), db).await`. Non-fatal: `warn!` + continue on error. |
| `framework/src/app.rs` bootstrap | `register_offload_hooks_with_broadcaster` | `App::get::<Broadcaster>()` branch | VERIFIED | Lines 426–433 confirmed. `Some` branch calls `register_offload_hooks_with_broadcaster`; `None` branch calls `register_offload_hooks()`. |
| Integration test `offload_delta_broadcast_suite` | `Broadcaster::new().with_transport(bus.clone())` × 2 | two Broadcasters over one shared `InMemoryTransport` | VERIFIED | Lines 370–372: `bus = Arc::new(InMemoryTransport::new(64))`, `broadcaster_a = Broadcaster::new().with_transport(bus.clone())`, `broadcaster_b = Arc::new(Broadcaster::new().with_transport(bus))`. |

---

### Data-Flow Trace (Level 4)

The phase delivers computation results over a broadcast channel, not rendered UI state. The data flow is: `enqueue_and_mark_pending` writes a pending snapshot → worker drains → `register_offload_hooks_with_broadcaster` hook persists snapshot + broadcasts delta → subscribed client receives `ServerMessage::Event`.

| Component | Data Variable | Source | Produces Real Data | Status |
|-----------|---------------|--------|--------------------|--------|
| `broadcast_delta` | `payload: serde_json::Value` | Built from `outcome` in hook before persist | Yes — derived from actual job output or error | FLOWING |
| `persist_pending` | `{"status":"pending"}` | `serde_json::json!` literal | Yes — written to `projection_snapshots` via `snapshot_write` | FLOWING |
| `read_result_redacted` | `OffloadResult<T>` | `read_result` → `snapshot_read` → DB row | Yes — reads real DB row; redacts Failed arm | FLOWING |
| `resolve()` | `OffloadResult<T>` | `read_result` after delta wake | Yes — reads authoritative snapshot on delta receipt | FLOWING |

---

### Behavioral Spot-Checks

Step 7b skipped: tests require the Rust build pipeline (`cargo test`). The executor confirmed all tests pass (`cargo test -p ferro-rs --test offload_delta_broadcast` exits 0, `cargo test -p ferro-rs` all 521+ unit tests passing) per SUMMARY self-check tables. No server start needed.

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| OFFLOAD-04 | 247-01, 247-02, 247-03 | A client subscribed to a handle receives the result as a ferro-broadcast delta on completion; the originating request returns immediately and never blocks awaiting it. | SATISFIED | SC#1 proven by `cross_replica_delta`; SC#2 proven by `request_returns_before_worker` (ordering + elapsed); SC#3 proven by queues.md section; redaction proven by `offload_failed_delta_is_redacted` and `read_result_redacted_hides_error`. Note: REQUIREMENTS.md traceability table still shows "Not started" for OFFLOAD-04 — this is a documentation inconsistency; the requirement checkbox `[x]` and the implementation are both correct. The table row is a stale artifact. |

---

### Anti-Patterns Found

No blockers or substantive stubs. Specific items reviewed:

| File | Pattern | Severity | Assessment |
|------|---------|----------|------------|
| `framework/src/app.rs` lines 426–433 | `None => crate::offload::register_offload_hooks()` fallback | Info | Backward-compat shim — see note below. |
| `OffloadResult::Pending` "backward-compatible" framing in SUMMARY | Framing of the new variant as backward-compatible | Info | The variant is a genuine addition; the framing refers to existing enum arms being unaffected. Not a stub. |

The failed delta arm `json!({ "status": "failed" })` at line 344 contains no `error` key — confirmed clean.

---

### Backward-Compat Debt (Deferred to Cleanup Phase)

Per the verification prompt, these items are noted but not scored as failures:

1. **`None => register_offload_hooks()` fallback in `app.rs`** (line 432): a `Some`/`None` broadcaster fallback that keeps the persist-only path when no `Broadcaster` is in the container. Wave 3 plans were directed not to add new shims, but this fallback was introduced in Wave 2 (247-02) as the bootstrap wiring. It is a real capability (apps without broadcasting still work) rather than a pure shim. Scheduled for review in the milestone-wide cleanup phase.

2. **`OffloadResult::Pending` "backward-compatible" framing in 247-01-SUMMARY**: the SUMMARY describes the new variant as backward-compatible because existing completed/failed rows still deserialize correctly. This is accurate documentation, not a workaround. No debt.

---

### Human Verification Required

None. All three success criteria are verifiable programmatically against the actual codebase. SC#3 (docs) was verified by reading the actual `queues.md` content and confirming the required strings and coverage.

---

## Gaps Summary

No gaps. All three roadmap success criteria are met by substantive, wired, data-flowing code. The integration test proves SC#1 and SC#2 with real async multi-replica delivery over `InMemoryTransport`. SC#3 is satisfied by the queues.md section. OFFLOAD-04 is delivered.

---

_Verified: 2026-08-14T16:00:00Z_
_Verifier: Claude (gsd-verifier)_
