---
phase: 247
slug: read-model-delta-broadcast-streaming
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-08-14
validated: 2026-08-14
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

> Bound to executed plans (247-01/02/03). Every OFFLOAD-04 success criterion has an automated
> command except SC#3 (docs, manual review). Per the 247-03 deviation the four integration
> scenarios run inside one `#[tokio::test]` — see footnote ¹; a per-scenario `-- <fn>` filter
> matches zero tests, so those rows now invoke the whole binary.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| `cross_replica_delta` | 247-03 | W3 | OFFLOAD-04 (SC#1) | T-247-hostile-payload | Delta reaches a subscriber on a *second* Broadcaster via the shared transport | integration | `cargo test -p ferro-rs --test offload_delta_broadcast` ¹ | ✅ | ✅ green |
| `request_returns_before_worker` | 247-03 | W3 | OFFLOAD-04 (SC#2) | — | `enqueue_and_mark_pending()` returns before the worker runs; snapshot is `Pending` pre-drain | integration | `cargo test -p ferro-rs --test offload_delta_broadcast` ¹ | ✅ | ✅ green |
| `read_result_redacted_hides_error` (unit) · `offload_failed_delta_is_redacted` (integration) | 247-01 / 247-03 | W1 / W3 | OFFLOAD-04 (D-05) | T-247-info-disclosure | Failed delta carries a non-sensitive marker; raw error absent from delta, retained in snapshot | unit + integration | `cargo test -p ferro-rs --lib read_result_redacted_hides_error` · `cargo test -p ferro-rs --test offload_delta_broadcast` ¹ | ✅ | ✅ green |
| `offload_pending_round_trip` | 247-01 | W1 | OFFLOAD-04 (D-07) | — | `persist_pending` writes `{status:"pending"}` retrievable by handle; unknown handle = `None` | unit | `cargo test -p ferro-rs --lib offload_pending_round_trip` | ✅ | ✅ green |
| `resolve_already_complete` | 247-03 | W3 | OFFLOAD-04 (D-09) | — | Race-safe resolve: subscribe → read-back → await; already-complete handle short-circuits | integration | `cargo test -p ferro-rs --test offload_delta_broadcast` ¹ | ✅ | ✅ green |
| `redis_cross_replica` | 247-03 | W3 | OFFLOAD-04 (live-redis) | T-247-hostile-payload | Cross-process delivery over Redis | env-gated | `REDIS_URL=redis://... cargo test -p ferro-rs --test offload_delta_broadcast --features redis-transport -- redis_cross_replica` | ✅ | ⬜ env-gated |

*Status: ⬜ pending / env-gated · ✅ green · ❌ red · ⚠️ flaky*

> ¹ The four scenario functions (`cross_replica_delta`, `request_returns_before_worker`,
> `offload_failed_delta_is_redacted`, `resolve_already_complete`) run in sequence inside a single
> `#[tokio::test] offload_delta_broadcast_suite` (`Queue`/`OFFLOAD_BROADCASTER` `OnceLock` forbids
> parallel init). The binary reports **one** test; run the whole binary rather than a `-- <fn>` filter.
> First-hand result 2026-08-14: `1 passed; 0 failed; finished in 0.97s`.

### Observable signals (per success criterion)

- **SC#1 — subscriber receives delta:** `rx.recv().await` on the Broadcaster B client returns `ServerMessage::Event` with `event == "offload.result"`, `channel == "projection.offload.result.{handle}"`, `data == {status:"completed", value:…}`. Asserted with `matches!` on event + channel.
- **SC#2 — non-blocking:** `let start = Instant::now(); let handle = enqueue_and_mark_pending(job, db).await?; assert!(start.elapsed() < Duration::from_millis(500))`, plus the authoritative ordering proof — `read_result` returns `Some(Pending)` *before* `drain()` and `Some(Completed { value: 7 })` *after* — confirming the worker had not run at assertion time.
- **SC#3 — documented pattern:** manual review of `docs/src/features/queues.md` section covering subscribe → read-back → await.
- **D-05 — no raw error in delta:** hook result `Err("sensitive-error-message")` → intercept delta → assert `data["error"]` absent or equals the opaque marker, never the raw string.
- **D-07 — pending marker:** `persist_pending("k1", &db)` → `read_result::<()>("k1", &db)` returns `Some(OffloadResult::Pending)`.

---

## Wave 0 Requirements

- [x] `framework/tests/offload_delta_broadcast.rs` — integration file covering SC#1, SC#2, D-05, D-09, and the env-gated live-redis variant (two-Broadcaster `InMemoryTransport` harness; `#[serial_test::serial]`; temp-file SQLite so the WorkerLoop pool sees a shared DB)
- [x] Unit tests for `persist_pending` (D-07 → `offload_pending_round_trip`) and `read_result_redacted` (D-05 → `read_result_redacted_hides_error`) alongside `offload_result_completed_round_trip` in `framework/src/offload.rs`
- [x] Unit test: `{"status":"pending"}` round-trips to `OffloadResult::Pending` (research Open Question #3 → `offload_result_pending_round_trip`)

*Existing infrastructure (sqlite in-memory, `Queue::init`, `drain_for_test`, the 246-05 round-trip harness) covers the rest.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Subscribe-then-await client pattern is documented | OFFLOAD-04 (SC#3) | Prose/doc criterion — no automated assertion | Review `docs/src/features/queues.md`: a section shows subscribe → read-back (`read_result_redacted`) → await on `projection.offload.result.{handle}`, and states the delta is redacted while the snapshot is authoritative |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 15s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** validated 2026-08-14

---

## Validation Audit 2026-08-14

| Metric | Count |
|--------|-------|
| Gaps found | 1 |
| Resolved | 1 |
| Escalated | 0 |

**Finding (resolved — documentation accuracy, not missing coverage).** The six per-task rows carried
per-scenario filter commands (`… -- cross_replica_delta`, `… -- request_returns_before_worker`,
`… -- offload_failed_delta_is_redacted`, `… -- resolve_already_complete`). Those names are private
`async fn`, not `#[tokio::test]` functions, so each filter matched **zero** tests and exited 0 — a
silent false-green. Per the 247-03 deviation the four scenarios run inside one
`#[tokio::test] offload_delta_broadcast_suite`. Commands corrected to run the whole binary
(`cargo test -p ferro-rs --test offload_delta_broadcast`); the two unit rows scoped with `--lib`.

**First-hand evidence (2026-08-14).** `cargo test -p ferro-rs --test offload_delta_broadcast` →
`1 passed; 0 failed; finished in 0.97s` (the suite; covers SC#1, SC#2, D-05, D-09). Unit rows
(D-05 `read_result_redacted_hides_error`, D-07 `offload_pending_round_trip`, serde round-trip
`offload_result_pending_round_trip`) confirmed present as `#[tokio::test]` in `framework/src/offload.rs`
and reported green in the 247-01/247-02 self-checks. `redis_cross_replica` remains env-gated (skips when
`REDIS_URL` is unset). No MISSING coverage: every OFFLOAD-04 success criterion has an automated test
except SC#3 (docs), correctly recorded as manual-only. No test files generated; no auditor spawned.
