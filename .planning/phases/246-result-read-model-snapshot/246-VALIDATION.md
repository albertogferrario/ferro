---
phase: 246
slug: result-read-model-snapshot
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-08-13
---

# Phase 246 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from 246-RESEARCH.md §"Validation Architecture". Requirement: OFFLOAD-03.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` / `#[tokio::test]` (in-crate unit + `tests/` integration) |
| **Config file** | none — Cargo workspace toolchain only |
| **Quick run command** | `cargo test -p ferro-projection -p ferro-queue` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~90–180 seconds (workspace clippy + tests; per-crate quick run ~15–30s) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-projection -p ferro-queue` (add `-p ferro` / `framework` when the write-back helpers or facade re-exports are touched).
- **After every plan wave:** Run the full CI-exact gate `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`.
- **Before `/gsd-verify-work`:** Full suite must be green.
- **Max feedback latency:** ~30 seconds for the per-task quick run.

Note (from `feedback_one_cpu_op_at_a_time` / thermal): serialize `cargo` invocations — do not launch parallel workspace builds; reuse the prior step's green evidence rather than re-running the full gate needlessly.

---

## Per-Task Verification Map

> Task IDs are placeholders keyed to the plan waves the planner will produce; the concrete IDs are assigned in the PLAN.md files. Every row below maps a Success Criterion or supporting seam to an automated command.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| direct-write | ferro-projection | 1 | OFFLOAD-03 | — | Upsert is last-writer-wins; no torn read | unit | `cargo test -p ferro-projection direct_snapshot_round_trip` | ❌ W0 | ⬜ pending |
| direct-overwrite | ferro-projection | 1 | OFFLOAD-03 | T-246-04 (concurrent re-claim) | Second write wins, no error | unit | `cargo test -p ferro-projection direct_snapshot_overwrite` | ❌ W0 | ⬜ pending |
| absent-none | ferro-projection | 1 | OFFLOAD-03 (SC-none) | — | Absent row → `None`, not a fabricated pending state | unit | `cargo test -p ferro-projection snapshot_read_returns_none_for_absent` | ✅ (existing analog `read_returns_none_for_absent_key`) | ⬜ pending |
| unit-output | ferro-projection/framework | 1 | OFFLOAD-03 | T-246-03 (`()` → null) | `OffloadResult<()>` round-trips via JSON `null` | unit | `cargo test -p ferro offload_result_unit_output` | ❌ W0 | ⬜ pending |
| handle-key-carry | ferro-queue | 1 | OFFLOAD-03 (SC1/SC2) | — | Caller `OffloadHandle.key()` == worker write key | integration | `cargo test -p ferro-queue offload_handle_key_round_trip` | ❌ W0 | ⬜ pending |
| success-roundtrip | ferro-queue/framework | 2 | OFFLOAD-03 SC1+SC2 | T-246-05 (persist failure ≠ job failure) | Completed envelope persisted; retrievable by handle | integration | `cargo test -p ferro-queue offload_result_round_trip` | ❌ W0 | ⬜ pending |
| terminal-err | ferro-queue/framework | 2 | OFFLOAD-03 SC3 | T-246-02 (error-string disclosure) | `{"status":"failed"}` written when retries exhausted | integration | `cargo test -p ferro-queue offload_terminal_error_on_err` | ❌ W0 | ⬜ pending |
| terminal-panic | ferro-queue/framework | 2 | OFFLOAD-03 SC3 | — | Panic → failed envelope (no silent drop) | integration | `cargo test -p ferro-queue offload_terminal_error_on_panic` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-projection/src/direct.rs` — `snapshot_write` + `snapshot_read` free functions and their unit tests (`direct_snapshot_round_trip`, `direct_snapshot_overwrite`) using the `sqlite::memory:` + inline `TestMigrator` pattern from `runtime.rs`.
- [ ] `framework/src/offload.rs` — `persist_result`, `persist_error`, `read_result`, `OffloadResult<T>`, `OFFLOAD_PROJECTION_NAME`; a unit test for the `()` output round-trip.
- [ ] `ferro-queue/tests/offload_result_round_trip.rs` — integration harness registering BOTH `CreateJobsTable` and `ferro_projection::CreateProjectionSnapshotsTable`, draining via a real `WorkerLoop`, covering SC1/SC2/SC3a/SC3b. (Alternatively host the full round-trip in `framework/tests/` since framework depends on both crates — planner's call.)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| — | — | All Phase 246 behaviors have automated verification (in-process `sqlite::memory:` covers persist, retrieve, terminal-error, panic). | — |

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (direct.rs, framework/offload.rs, ferro-queue integration test)
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s (per-task quick run)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
