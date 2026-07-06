---
phase: 207
slug: comp-02-synthetic-regression-catalog
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-12
---

# Phase 207 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust integration tests) + `proptest = "1"` + `insta = "1"` (yaml) |
| **Config file** | none — Wave 0 adds `[dev-dependencies]` to `ferro-projections/Cargo.toml` |
| **Quick run command** | `cargo test -p ferro-projections --test catalog` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~15–40 seconds (proptest cases dominate) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-projections --test catalog`
- **After every plan wave:** Run `cargo test --all-features` + `cargo clippy --all --all-targets -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite green AND `cargo fmt --all -- --check`
- **Max feedback latency:** ~40 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 207-01-01 | 01 | 1 | COMP-02 | — | N/A (test-only crate code) | unit | `cargo test -p ferro-projections --test catalog` | ❌ W0 | ⬜ pending |
| 207-01-02 | 01 | 1 | COMP-02 | — | N/A | unit | `cargo test -p ferro-projections --test catalog::canonical` | ❌ W0 | ⬜ pending |
| 207-01-03 | 01 | 1 | COMP-02 | — | N/A | unit | `cargo test -p ferro-projections --test catalog::adversarial` | ❌ W0 | ⬜ pending |
| 207-01-04 | 01 | 1 | COMP-02 | — | N/A | property | `cargo test -p ferro-projections --test catalog::proptest` | ❌ W0 | ⬜ pending |
| 207-01-05 | 01 | 1 | COMP-02 | — | N/A | snapshot | `cargo test -p ferro-projections --test catalog` (insta) | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*Exact task IDs are finalized by the planner; the catalog is a single file so all tasks share the
one `cargo test --test catalog` entrypoint with module-path filters.*

---

## Wave 0 Requirements

- [ ] `ferro-projections/Cargo.toml` — add `[dev-dependencies]`: `insta = { version = "1", features = ["yaml"] }`, `proptest = "1"`
- [ ] `ferro-projections/tests/catalog.rs` — new integration test file (system under test is `derive_intents()`, read-only)
- [ ] `ferro-projections/tests/snapshots/` — insta snapshot directory (created on first `cargo insta` accept)

*Calibration gate: a first `cargo test -p ferro-projections --test catalog -- --nocapture` run records observed
confidences before any floor/margin assertion is hardened (CONTEXT D-07). This is a Wave-1 calibration task, not Wave 0.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| "Discovered weaknesses" note names ≥1 real limitation (SC#5) | COMP-02 | Qualitative judgement — cannot be grep-asserted | Phase verification reads the note; an empty/boilerplate section fails phase close |
| Structural-invariant asserts outnumber insta snapshot asserts (SC#2) | COMP-02 | Requires counting two assertion classes in the source | `grep -c 'assert' catalog.rs` vs `grep -c 'assert_.*_snapshot' catalog.rs` — first must exceed second |

---

## Validation Sign-Off

- [ ] All tasks have automated `cargo test` verification or Wave 0 dependencies
- [ ] Sampling continuity: every task verified by the catalog test entrypoint (no 3 consecutive unverified tasks)
- [ ] Wave 0 covers the missing `[dev-dependencies]` and the new test file
- [ ] No watch-mode flags (single-shot `cargo test`)
- [ ] Feedback latency < 40s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
