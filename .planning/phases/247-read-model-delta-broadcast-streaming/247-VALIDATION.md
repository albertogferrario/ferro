---
phase: 247
slug: read-model-delta-broadcast-streaming
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-08-14
---

# Phase 247 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `247-RESEARCH.md` §"Validation Architecture".

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `#[tokio::test]` (workspace, no separate runner) |
| **Config file** | `Cargo.toml` per crate |
| **Quick run command** | `cargo test -p ferro-rs --test offload_delta_broadcast` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | quick ~15s · full suite several minutes (disk-gated — see `project_ferro_disk_full_test_gate.md`) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-rs --test offload_delta_broadcast` (fast, isolated)
- **After every plan wave:** Run `cargo test --all-features` (after a `df` disk-space check)
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~15 seconds (quick command)

---

## Per-Task Verification Map

> Task IDs are bound by the planner. Rows below are the requirement→signal targets the
> planner must attach a task to; every OFFLOAD-04 success criterion has an automated command
> except SC#3 (docs, manual review).

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD | — | — | OFFLOAD-04 (SC#1) | T-247-hostile-payload | Delta reaches a subscriber on a *second* Broadcaster via the shared transport | integration | `cargo test -p ferro-rs --test offload_delta_broadcast -- cross_replica_delta` | ❌ W0 | ⬜ pending |
| TBD | — | — | OFFLOAD-04 (SC#2) | — | `offload()` returns before the worker runs (non-blocking) | integration | `cargo test -p ferro-rs --test offload_delta_broadcast -- request_returns_before_worker` | ❌ W0 | ⬜ pending |
| TBD | — | — | OFFLOAD-04 (D-05) | T-247-info-disclosure | Failed delta carries a non-sensitive marker; raw error absent | unit | `cargo test -p ferro-rs -- offload_failed_delta_is_redacted` | ❌ W0 | ⬜ pending |
| TBD | — | — | OFFLOAD-04 (D-07) | — | `persist_pending` writes `{status:"pending"}` retrievable by handle | unit | `cargo test -p ferro-rs -- offload_pending_round_trip` | ❌ W0 | ⬜ pending |
| TBD | — | — | OFFLOAD-04 (D-09) | — | Race-safe resolve: subscribe → read-back → await; already-complete handle short-circuits | integration | `cargo test -p ferro-rs --test offload_delta_broadcast -- resolve_already_complete` | ❌ W0 | ⬜ pending |
| TBD | — | — | OFFLOAD-04 (live-redis) | T-247-hostile-payload | Cross-process delivery over Redis | env-gated | `REDIS_URL=redis://... cargo test -p ferro-rs --test offload_delta_broadcast --features redis-transport -- redis_cross_replica` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

### Observable signals (per success criterion)

- **SC#1 — subscriber receives delta:** `rx.recv().await` on the Broadcaster B client returns `ServerMessage::Event` with `event == "offload.result"`, `channel == "projection.offload.result.{handle}"`, `data == {status:"completed", value:…}`. Asserted with `matches!` on event + channel.
- **SC#2 — non-blocking:** `let start = Instant::now(); let handle = job.offload().await?; assert!(start.elapsed() < Duration::from_millis(100))` — WorkerLoop is not yet drained, so the job is still queued at assertion time.
- **SC#3 — documented pattern:** manual review of `docs/src/features/queues.md` section covering subscribe → read-back → await.
- **D-05 — no raw error in delta:** hook result `Err("sensitive-error-message")` → intercept delta → assert `data["error"]` absent or equals the opaque marker, never the raw string.
- **D-07 — pending marker:** `persist_pending("k1", &db)` → `read_result::<()>("k1", &db)` returns `Some(OffloadResult::Pending)`.

---

## Wave 0 Requirements

- [ ] `framework/tests/offload_delta_broadcast.rs` — new integration file covering SC#1, SC#2, D-05, D-09, and the env-gated live-redis variant (`InMemoryTransport` two-Broadcaster harness; `#[serial_test::serial]` for the global-hook state per Pitfall 2)
- [ ] Unit tests for `persist_pending` (D-07) and `read_result_redacted` (D-05) alongside the existing `offload_result_completed_round_trip` tests in `framework/src/offload.rs`
- [ ] Unit test: `serde_json::from_str::<OffloadResult<()>>(r#"{"status":"pending"}"#)` round-trips (research Open Question #3)

*Existing infrastructure (sqlite in-memory, `Queue::init`, `drain_for_test`, the 246-05 round-trip harness) covers the rest.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Subscribe-then-await client pattern is documented | OFFLOAD-04 (SC#3) | Prose/doc criterion — no automated assertion | Review `docs/src/features/queues.md`: a section shows subscribe → read-back (`read_result_redacted`) → await on `projection.offload.result.{handle}`, and states the delta is redacted while the snapshot is authoritative |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
