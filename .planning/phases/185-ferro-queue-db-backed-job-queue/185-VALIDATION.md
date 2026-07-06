---
phase: 185
slug: ferro-queue-db-backed-job-queue
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-07
---

# Phase 185 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (tokio async tests) |
| **Config file** | workspace Cargo.toml (existing) |
| **Quick run command** | `cargo test -p ferro-queue` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | quick ~60s; full ~10min |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-queue` (one CPU-intensive op at a time — never parallel with other cargo runs)
- **After every plan wave:** Run full suite command
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~600 seconds (full suite)

---

## Per-Task Verification Map

> Filled in by the planner per task. Anchors derived from phase success criteria:

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| race claim | TBD | TBD | QUEUE-F-02 | — | two concurrent WorkerLoops claim each job exactly once (shared temp-file SQLite; Postgres behind cfg gate) | integration | `cargo test -p ferro-queue race` | ❌ W0 | ⬜ pending |
| reaper | TBD | TBD | QUEUE-F-03 | — | stuck claimed job re-queued after visibility timeout; attempts incremented | integration | `cargo test -p ferro-queue reaper` | ❌ W0 | ⬜ pending |
| poison job | TBD | TBD | QUEUE-F-01 | — | job exceeding max_retries parked as failed with error; never blocks claims; panicking handle() isolated | integration | `cargo test -p ferro-queue poison` | ❌ W0 | ⬜ pending |
| backoff | TBD | TBD | QUEUE-F-01 | — | retry delay exponential with jitter, capped | unit | `cargo test -p ferro-queue backoff` | ❌ W0 | ⬜ pending |
| idempotency | TBD | TBD | QUEUE-F-01 | — | duplicate enqueue with same (job_type, idempotency_key) skipped while pending/claimed | unit | `cargo test -p ferro-queue idempotency` | ❌ W0 | ⬜ pending |
| shutdown drain | TBD | TBD | QUEUE-F-03 | — | SIGTERM/shutdown flag drains in-flight, re-queues unstarted claims | integration | `cargo test -p ferro-queue shutdown` | ❌ W0 | ⬜ pending |
| migration helper | TBD | TBD | QUEUE-F-04 | — | jobs table creates on SQLite + Postgres; no backend-specific SQL in migration | unit | `cargo test -p ferro-queue migration` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-queue` dev-dependencies: `tempfile` (shared temp-file SQLite for race test), `futures` (FutureExt::catch_unwind), `rand` (jitter) — verify or add
- [ ] cfg-gated Postgres test scaffold (`#[cfg(feature = "postgres-tests")]` or env-gated, matching workspace precedent)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| WorkerLoop starts inside app server | QUEUE-F-03 | needs a running app binary | run sample `app/`, enqueue a job, observe execution without separate worker process |
| ferro-mcp queue tools over DB | — | MCP server runtime | `cargo build`, restart MCP, call `queue_status`/`list_jobs`/`job_history` |
