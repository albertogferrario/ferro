---
phase: 233
slug: ferro-payments-crate-polymorphic-billable
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-17
---

# Phase 233 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (`cargo test`) + in-memory SQLite via sea-orm |
| **Config file** | none — tests live in `ferro-payments/src/**/*.rs` (`#[cfg(test)]`) and/or `ferro-payments/tests/` |
| **Quick run command** | `cargo test -p ferro-payments` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~30–90 seconds (crate-scoped quick run faster) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-payments`
- **After every plan wave:** Run `cargo clippy -p ferro-payments --all-targets -- -D warnings && cargo test -p ferro-payments`
- **Before `/gsd-verify-work`:** Full suite (fmt + clippy --all + test --all-features) must be green
- **Max feedback latency:** ~90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 233-01-xx | 01 | 1 | PAY-POLY-DM-02 (enums) | — | N/A | unit | `cargo test -p ferro-payments status::` | ❌ W0 | ⬜ pending |
| 233-01-xx | 01 | 1 | PAY-POLY-DM-01 (entity/columns) | — | No FK to consumer tables; no PII in metadata | unit | `cargo test -p ferro-payments` | ❌ W0 | ⬜ pending |
| 233-02-xx | 02 | 2 | PAY-POLY-DM-04 (migration + partial unique index) | — | Partial-unique prevents double-active billable | unit | `cargo test -p ferro-payments migration::` | ❌ W0 | ⬜ pending |
| 233-02-xx | 02 | 2 | PAY-POLY-DM-04 (partial-unique enforcement) | — | Second active INSERT for same (kind,id) rejected; succeeds after release | unit (in-mem SQLite) | `cargo test -p ferro-payments partial_unique` | ❌ W0 | ⬜ pending |
| 233-03-xx | 03 | 3 | PAY-POLY-DM-03 (lifecycle transitions) | — | Guarded UPDATE: 0-row = no-op on stale precondition | unit (in-mem SQLite) | `cargo test -p ferro-payments lifecycle::` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*
*Exact task IDs assigned by the planner; plan/wave numbers are indicative.*

---

## Wave 0 Requirements

- [ ] `ferro-payments` crate skeleton must compile (`cargo build -p ferro-payments`) before any test runs
- [ ] In-memory SQLite test fixture: a helper that runs the migration against `sqlite::memory:` and returns a `DatabaseConnection`
- [ ] No new test framework needed — Rust built-in harness; `sea-orm` already a workspace dependency

*The crate does not yet exist — Wave 0 is effectively the scaffold (Plan 01) producing a compiling crate that later tests target.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Postgres partial unique index DDL applies | PAY-POLY-DM-04 | CI/unit path uses in-memory SQLite only; Postgres path is correct-by-construction + review | Apply migration against a Postgres instance; `\d payment_intents` shows `uq_..._active ... WHERE (status = ANY ...)` |
| MySQL generated-column UNIQUE permits multiple NULLs | PAY-POLY-DM-04 | No MySQL in CI; emulation path reviewed not unit-tested | Apply migration against MySQL 8.0; insert 2 non-active rows for same (kind,id) → both succeed; 2 active rows → second fails |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (crate must compile before tests)
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
