---
phase: 248
slug: deployable-ferro-worker-runtime
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-08-14
---

# Phase 248 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `248-RESEARCH.md` § Validation Architecture. The Per-Task
> Verification Map is finalized once `*-PLAN.md` task IDs exist.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `tokio::test` + `cargo test` (Rust workspace) |
| **Config file** | None — Cargo.toml `[[test]]` sections per crate |
| **Quick run command** | `cargo test -p ferro-queue --test worker_runtime 2>&1 \| tail -5` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | Quick ~15s; full gate several minutes (serialize on this MacBook — one CPU op at a time) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-queue` (queue tests only, fast)
- **After every plan wave:** Run `cargo test -p ferro-queue && cargo test -p framework`
- **Before `/gsd-verify-work`:** Full gate green (`fmt --check && clippy -D warnings && test --all-features`)
- **Macro-touching tasks:** the trybuild UI gate + full changed-crates test are mandatory — per-crate `cargo test` misses broken `#[offload]` emission (only fixtures expand the attribute)
- **Max feedback latency:** ~15s (quick), full gate at wave boundaries

---

## Per-Task Verification Map

> Task IDs assigned during planning. Rows below are the Success-Criteria-level
> anchors each task must satisfy; the planner maps concrete `{N}-PP-TT` task IDs onto them.

| SC / Req | Behavior | Test Type | Automated Command | File Exists | Status |
|----------|----------|-----------|-------------------|-------------|--------|
| SC#1 | `<app-bin> worker --queue X` consumes only queue X (jobs on Y not claimed) | Integration (in-process) | `cargo test -p ferro-queue --test worker_runtime` | ❌ W0 | ⬜ pending |
| SC#2 | Two worker loops split work, no double-processing | Integration (in-process) | `cargo test -p ferro-queue --test worker_runtime` | ❌ W0 | ⬜ pending |
| SC#3 | Saturating one queue does not stall a disjoint one (fault-domain isolation) | Integration (in-process) | `cargo test -p ferro-queue --test worker_runtime` | ❌ W0 | ⬜ pending |
| SC#4 | No framework-managed autoscaling introduced | Structural / grep | `! grep -rqiE "autoscal\|scale_to_zero\|KEDA" framework/src/` | ✅ | ⬜ pending |
| WR-01 | `transport_redis_url` set → `RedisTransport` attached at framework boot | Env-gated integration | `cargo test -p framework --features redis-transport --test worker_boot` (skips without `REDIS_URL`) | ❌ W0 | ⬜ pending |
| D-07 | Feature-off + URL-set → `tracing::warn!`, in-process fallback, no panic | Unit | `cargo test -p framework --test worker_boot` | ❌ W0 | ⬜ pending |

> SC#1–SC#3 are named sub-functions of the single `#[tokio::test]` in `worker_runtime.rs` (test-collapse guard), so they are **not** individually filterable — the suite command above runs all three. Confirm resolution with `cargo test -p ferro-queue --test worker_runtime -- --list` (must list ≥1 test) before trusting a green. Likewise D-07 runs inside the `worker_boot` suite.
| OFFLOAD-05 | Deployable worker process runnable at N real replicas | Human UAT | see Manual-Only | — | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky · ❌ W0 = test file created in Wave 0*

---

## Test-Collapse Guard (mandatory)

SC#1–SC#3 tests live as **sub-functions of a single `#[tokio::test]`** in
`ferro-queue/tests/worker_runtime.rs` — not separate `#[tokio::test]` functions —
to avoid the OnceLock collision and the false-green test-collapse pitfall.
Every proposed test command MUST be proven to resolve to a real test:

```bash
cargo test -p ferro-queue --test worker_runtime -- --list   # must list ≥1 test name
```

Multi-connection concurrency tests use SQLite `NamedTempFile`, **not** `sqlite::memory:`.

---

## Wave 0 Requirements

- [ ] `ferro-queue/tests/worker_runtime.rs` — SC#1, SC#2, SC#3 as sub-functions of one `#[tokio::test]`
- [ ] `framework/tests/worker_boot.rs` — WR-01 bootstrap path (feature-gated) + D-07 warning behavior
- [ ] ferro-macros trybuild fixture updates for `#[offload(queue = "…")]` (macro-emission gate)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| N real replicas split shared-queue load without double-processing | OFFLOAD-05 / SC#2 | True multi-process behavior cannot be deterministically automated in `cargo test`; the in-process two-loop test is a proxy, and cross-process exactly-once is already proven by `race_claim_sqlite.rs` | Build the app binary; run `<app-bin> worker --queue default` in ≥2 shells + `<app-bin> serve --no-worker`; enqueue a batch; confirm each job processed exactly once across replicas |
| Cross-replica broadcast delivery over Redis transport | WR-01 / OFFLOAD-04 | Requires a live `redis-server` and ≥2 processes (unblocks the Phase 246.1 multi-replica UAT) | With `BROADCAST_REDIS_URL` set and `redis-transport` feature on, run 2 web replicas; a delta published on replica A reaches a client socketed on replica B |

---

## Validation Sign-Off

- [ ] All tasks have an `<automated>` verify or a Wave 0 dependency
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (test files above)
- [ ] No watch-mode flags in any command
- [ ] Every test command proven to resolve to a real test (`-- --list`)
- [ ] Feedback latency < 15s (quick loop)
- [ ] `nyquist_compliant: true` set in frontmatter once the map is task-complete

**Approval:** pending
