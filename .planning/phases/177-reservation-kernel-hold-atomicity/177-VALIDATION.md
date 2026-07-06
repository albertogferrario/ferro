---
phase: 177
slug: reservation-kernel-hold-atomicity
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-21
---

# Phase 177 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `tokio::test` (already in use) |
| **Config file** | none — uses `Cargo.toml` dev-deps |
| **Quick run command** | `cargo test -p ferro-reservation` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~45 seconds (ferro-reservation only); ~6 minutes (full workspace) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-reservation`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy -p ferro-reservation --all-targets -- -D warnings && cargo test -p ferro-reservation`
- **Before `/gsd-verify-work`:** Full workspace suite (`cargo test --all-features`) must be green; CI-equivalent clippy command must succeed
- **Max feedback latency:** 60 seconds for per-task verification

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 177-01-XX | 01 | 1 | SC-1 / D-06 | T-177-RACE | `held ≤ capacity` invariant holds under N-way concurrent `hold` | integration | `cargo test -p ferro-reservation hold_race` | ⚠️ Partial (concurrent_hold.rs exists; uses tokio::Mutex workaround that must be removed) | ⬜ pending |
| 177-01-XX | 01 | 1 | SC-2 | — | Non-overlapping windows on same key both succeed (boundary preservation, no false positives) | integration | `cargo test -p ferro-reservation non_overlapping_windows` | ❌ W0 | ⬜ pending |
| 177-01-XX | 01 | 1 | SC-5 / D-04 | T-177-AUDIT | Conflict-losing task does NOT write a `reservation.held` audit row | integration | `cargo test -p ferro-reservation audit_atomicity` | ❌ W0 | ⬜ pending |
| 177-01-XX | 01 | 1 | SC-3 | — | Existing single-writer kernel tests pass byte-identical | regression | `cargo test -p ferro-reservation` | ✅ (kernel.rs:494-560) | ⬜ pending |
| 177-02-XX | 02 | 2 | SC-1 (capacity > 1) | T-177-RACE | N+1 racing tasks on capacity=N produce exactly N Ok + 1 Insufficient | integration | `cargo test -p ferro-reservation race_capacity_n` | ❌ W0 | ⬜ pending |
| 177-02-XX | 02 | 2 | SC-1 (Postgres) | T-177-RACE | Postgres mirror of SC-1 race test under SERIALIZABLE isolation | integration (cfg-gated) | `cargo test -p ferro-reservation --features postgres-tests hold_race_postgres` | ❌ W0 | ⬜ pending |
| 177-02-XX | 02 | 2 | SC-1 (40001 translation) | — | Postgres `40001` serialization failure translates to `Err(Insufficient)` not `Err(Db)` | integration (cfg-gated) | `cargo test -p ferro-reservation --features postgres-tests sqlstate_40001_translation` | ❌ W0 | ⬜ pending |
| 177-03-XX | 03 | 3 | SC-6 | — | `docs/src/database/reservations.md` Consistency Model section reflects transaction-based fix (no Mutex advice) | doc review | manual — `grep -c "tokio::Mutex" docs/src/database/reservations.md` returns 0; `grep -c "serializable isolation" docs/src/database/reservations.md` returns ≥1 | ✅ (exists, stale) | ⬜ pending |
| 177-03-XX | 03 | 3 | SC-6 | — | `tests/concurrent_hold.rs` module doc no longer claims kernel cannot arbitrate | doc review | manual — read module doc block | ✅ (exists, stale) | ⬜ pending |
| 177-03-XX | 03 | 3 | SC-6 | — | kernel.rs module doc updated if it claims atomicity is caller responsibility | doc review | manual — grep kernel.rs head for stale claims | TBD — researcher to confirm | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*Task IDs are placeholders pending plan generation — the planner assigns final IDs.*

---

## Wave 0 Requirements

- [ ] `ferro-reservation/tests/concurrent_hold.rs` — rewrite to remove the `tokio::Mutex` workaround; extend to ≥50 iterations for SC-1; add functions for SC-2 (non-overlapping windows) and SC-5 (audit atomicity)
- [ ] `ferro-reservation/Cargo.toml` — add `[features] postgres-tests = []` section; add `sqlx-postgres` to dev-dep `sea-orm` features so SQLSTATE detection compiles in tests when the feature is on
- [ ] `ferro-reservation/tests/concurrent_hold_postgres.rs` (NEW, cfg-gated) — Postgres mirror of SC-1; requires `DATABASE_URL` or similar to point at a docker-compose Postgres; gated on `#[cfg(feature = "postgres-tests")]`

*No new test framework needed — `tokio::test` + `#[test]` is already configured. No new dev-deps beyond enabling existing optional sea-orm features.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `docs/src/database/reservations.md` Consistency Model section is technically accurate after the fix | SC-6 | Documentation correctness is a reading/judgment task, not a unit-testable property | Reviewer reads the Consistency Model section (around lines 145-148 and 363-382), confirms the "use a tokio::Mutex per resource key" advice is removed, and confirms the new transaction-based behavior is described accurately with `serializable isolation` mentioned |
| Phase 130/131/132 gestiscilo-it inventory tests still pass against the patched ferro-reservation | SC-3 | Cross-repo regression — requires checking out gestiscilo-it, pointing it at the local ferro path, and running its test suite | Reviewer (or release process) updates gestiscilo-it's `Cargo.toml` local path dep, runs `cargo test --workspace` in gestiscilo-it, asserts inventory tests pass byte-identical |
| Postgres `40001` retry behavior under high contention | SC-1 (Postgres) | Requires a Postgres instance and load — not part of the standard SQLite-primary suite | Reviewer spins up docker-compose Postgres, runs the cfg-gated `postgres-tests` feature, monitors for any flake across 50+ iterations |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies declared
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING test/feature/scaffolding references
- [ ] No watch-mode flags (no `cargo watch` in any task — CI must be deterministic single-run)
- [ ] Feedback latency < 60s per-task
- [ ] `nyquist_compliant: true` set in frontmatter after planner assigns final task IDs and the Per-Task Verification Map is filled in completely

**Approval:** pending
