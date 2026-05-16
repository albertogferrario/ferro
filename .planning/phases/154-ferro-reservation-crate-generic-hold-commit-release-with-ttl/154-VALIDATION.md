---
phase: 154
slug: ferro-reservation-crate-generic-hold-commit-release-with-ttl
status: approved
nyquist_compliant: true
wave_0_complete: false
created: 2026-05-13
---

# Phase 154 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (`cargo test`) + `proptest 1` (dev-dep, new for this phase) |
| **Config file** | `ferro-reservation/Cargo.toml` (dev-dependencies block) |
| **Quick run command** | `cargo test -p ferro-reservation --lib` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~30s (unit + integration) + ~60s (proptest with default cases) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-reservation --lib`
- **After every plan wave:** Run full suite (fmt + clippy + test)
- **Before `/gsd-verify-work`:** Full suite must be green; `cargo doc -p ferro-reservation --no-deps` must succeed; rustdoc examples must compile (covered by `cargo test --doc -p ferro-reservation`)
- **Max feedback latency:** 30 seconds for unit tests; 90 seconds for full pre-push gate

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement (D-XX) | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|--------------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 154-01-* | 01 | 1 | D-01..D-04 | — | crate scaffold + Cargo.toml + workspace member registration | build | `cargo build -p ferro-reservation` | ❌ W0 | ⬜ pending |
| 154-02-* | 02 | 1→2 | D-05..D-08 | — | `Resource` trait surface compiles with `async_trait`; generic over `Key`/`Window` | unit | `cargo test -p ferro-reservation --lib resource::` | ❌ W0 | ⬜ pending |
| 154-03-* | 03 | 2 | D-38..D-42 | — | SeaORM migration creates `reservations` + 2 indexes; SQLite + Postgres-compatible DDL | unit | `cargo test -p ferro-reservation --lib migration::` | ❌ W0 | ⬜ pending |
| 154-04-* | 04 | 2 | D-43..D-46 | — | `ReservationError` enum with `From<GuardedError>` / `From<AuditError>` / `From<DbErr>` / `From<serde_json::Error>` | unit | `cargo test -p ferro-reservation --lib error::` | ❌ W0 | ⬜ pending |
| 154-05-* | 05 | 2 | D-25, D-26, D-27 | — | `ReservationEvent` implements `ferro_events::Event`; serde round-trip | unit | `cargo test -p ferro-reservation --lib event::` | ❌ W0 | ⬜ pending |
| 154-06-* | 06 | 2 | D-29, D-34, D-35 | — | `ReservationContext` builder + `ReservationHandle` serde round-trip | unit | `cargo test -p ferro-reservation --lib context::` `cargo test -p ferro-reservation --lib handle::` | ❌ W0 | ⬜ pending |
| 154-07-* | 07 | 3 | D-09..D-15, D-28..D-30 | T-154-01 (race-free hold) / T-154-02 (audit-failure surfaces) | `ReservationKernel::hold/commit/release/extend` race-free + auto-audit | unit | `cargo test -p ferro-reservation --lib kernel::` | ❌ W0 | ⬜ pending |
| 154-08-* | 08 | 3 | D-21..D-24 | T-154-03 (sweeper idempotency under concurrent sweepers) | `run_sweep_once` transitions expired holds; `exec_at_most_one` tolerates 0 rows | unit | `cargo test -p ferro-reservation --lib sweeper::` | ❌ W0 | ⬜ pending |
| 154-09-* | 09 | 4 | D-47 (the 12 unit-test cases) | — | All 12 unit-test cases pass | unit | `cargo test -p ferro-reservation --lib` | ❌ W0 | ⬜ pending |
| 154-10-* | 10 | 4 | D-48 | T-154-01 | concurrent_hold integration test: N=20 vs capacity=5 → exactly 5 succeed | integration | `cargo test -p ferro-reservation --test concurrent_hold` | ❌ W0 | ⬜ pending |
| 154-11-* | 11 | 4 | D-49 | T-154-01, T-154-04 (state-machine validity) | proptest property tests: capacity invariant + state-machine validity | property | `cargo test -p ferro-reservation --test property_invariants` | ❌ W0 | ⬜ pending |
| 154-12-* | 12 | 4 | D-50 | — | cross-crate integration: 152+153+154 compose; events dispatch; audit replay reconstructs final state | integration | `cargo test -p ferro-reservation --test integration_with_audit_and_events` | ❌ W0 | ⬜ pending |
| 154-13-* | 13 | 5 | D-53, D-54, D-55 | — | rustdoc examples compile; user-facing doc page exists with all sections | doc | `cargo test --doc -p ferro-reservation` + grep for required sections in `docs/src/database/reservations.md` | ❌ W0 | ⬜ pending |
| 154-14-* | 14 | 5 | D-56, D-57, D-58 | — | workspace version bumped to 0.2.32; `ferro-reservation` listed in WAVE1B_CRATES; CHANGELOG entry exists | grep | `grep '"0.2.32"' Cargo.toml` + `grep 'ferro-reservation' .github/workflows/publish.yml` + `grep '## ferro-reservation' CHANGELOG.md` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*Task IDs are placeholders; planner produces final IDs of the form `154-NN-MM` once plan files are emitted.*

---

## Wave 0 Requirements

- [ ] `ferro-reservation/Cargo.toml` — crate scaffold with `[dev-dependencies] proptest = "1"` (first appearance in workspace)
- [ ] `ferro-reservation/tests/concurrent_hold.rs` — integration test stub for D-48
- [ ] `ferro-reservation/tests/property_invariants.rs` — proptest stub for D-49
- [ ] `ferro-reservation/tests/integration_with_audit_and_events.rs` — cross-crate test stub for D-50
- [ ] In-memory SQLite testing harness re-derived inline (no `framework` dep — matches Phase 153 D-34)
- [ ] Root `Cargo.toml` adds `"ferro-reservation"` to `[workspace.members]`

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| First-publish bootstrap from local terminal | D-04, D-57 | CI publish token has `publish-update` only — new-crate first publish requires personal `publish-new` token; cannot be automated | Run `cargo publish -p ferro-reservation --token $PERSONAL_PUBLISH_NEW_TOKEN` from local terminal AFTER workspace version bump merges to master and Wave 1a's auto-publish completes. Same operational reality as Phase 151 / 152 / 153 — captured in `project_ferro_publish_token_scoping.md`. |
| Postgres dialect verification | D-19, D-51 (deferred) | Postgres CI integration tests deferred per CONTEXT.md; SQLite-only in CI | Optional out-of-CI verification: run the test suite against a local Postgres via `DATABASE_URL=postgres://… cargo test -p ferro-reservation --features postgres` (feature gate not in v0 — relies on swapping sea-orm features locally) |

---

## Validation Sign-Off

- [ ] All 14 task groups have automated verification commands (no manual-only required tasks)
- [ ] Sampling continuity: every wave has at least one automated `cargo test` invocation; no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING test infrastructure (proptest dev-dep, three test files, harness inline, workspace member registration)
- [ ] No watch-mode flags (`cargo test` runs once-and-exits; no `cargo watch` in CI)
- [ ] Feedback latency < 90s for full suite, < 30s for per-task unit run
- [ ] `nyquist_compliant: true` set in frontmatter after planner emits PLAN.md files mapping each task to a verify command

**Approval:** pending
