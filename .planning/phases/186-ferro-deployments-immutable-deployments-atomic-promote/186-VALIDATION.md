---
phase: 186
slug: ferro-deployments-immutable-deployments-atomic-promote
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-07
---

# Phase 186 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust built-in + tokio::test) |
| **Config file** | none — workspace member `ferro-deployments/Cargo.toml` declares dev-deps (tokio, tempfile) |
| **Quick run command** | `cargo test -p ferro-deployments` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~60–120 seconds (workspace), ~10s for the crate alone |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-deployments`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 186-01-xx | 01 | 1 | DEPL-F-01 | — | migration up/down portable SQLite+Postgres; terminal rows immutable | unit | `cargo test -p ferro-deployments migration` | ❌ W0 | ⬜ pending |
| 186-02-xx | 02 | 2 | DEPL-F-02 | — | promote returns prior id; non-`ready` rejected; rollback = promote-of-previous | unit | `cargo test -p ferro-deployments promote` | ❌ W0 | ⬜ pending |
| 186-02-xx | 02 | 2 | DEPL-F-02 | — | two concurrent promotes serialize (LWW, no torn state) — SQLite always-on | integration | `cargo test -p ferro-deployments race_promote_sqlite` | ❌ W0 | ⬜ pending |
| 186-03-xx | 03 | 2 | DEPL-F-03 | — | DeploymentStorage S3-default delegates to ferro-storage; preview_url Option | unit | `cargo test -p ferro-deployments storage` | ❌ W0 | ⬜ pending |
| 186-04-xx | 04 | 3 | DEPL-F-01..03 | — | non-HTML (JSON) artifact stored through same API (criterion 5) | doc-test | `cargo test -p ferro-deployments --doc` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-deployments/Cargo.toml` — new workspace member with dev-deps (tokio multi-thread, tempfile)
- [ ] `ferro-deployments/tests/` — integration test files for promote race + status-transition rejection
- [ ] Postgres race test cfg-gated behind env (mirrors ferro-queue `race_claim_postgres.rs`); SQLite race test always-on

*Existing `cargo test` infrastructure covers the framework; the new crate brings its own test module + integration tests.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Postgres-backend race test | DEPL-F-02 | Requires a live Postgres (cfg-gated, not in default CI run) | Set `DATABASE_URL=postgres://…` + run `cargo test -p ferro-deployments --features pg-tests race_promote_postgres` |
| First crates.io publish | — | CI token is publish-update only; new crate needs bootstrap | One-time `cargo publish -p ferro-deployments` from local terminal before first CI push |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
