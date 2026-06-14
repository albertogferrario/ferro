---
phase: 225
slug: release-workflow-rustls-migration-and-e2e-cli-from-release-t
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-14
---

# Phase 225 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from 225-RESEARCH.md "## Validation Architecture".

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust stdlib runner) + `cargo tree` structural checks + GitHub Actions |
| **Config file** | none — uses existing workspace Cargo.toml profiles + .github/workflows |
| **Quick run command** | `cargo tree -p ferro-cli --edges no-dev -e features \| grep -E 'native-tls\|openssl-sys\|aws-lc' \| wc -l` (expect 0) |
| **Full suite command** | `cargo test --all-features && cargo deny check` |
| **Estimated runtime** | ~tree check < 5s; full workspace test build several minutes |

---

## Sampling Rate

- **After every task commit:** Run the structural `cargo tree -p ferro-cli ... \| grep -E 'native-tls\|openssl-sys\|aws-lc'` check (must stay 0 once the swap task lands) + `cargo build -p ferro-cli`.
- **After every plan wave:** Run `cargo test --all-features` + `cargo deny check`.
- **Before `/gsd-verify-work`:** Full suite green; release.yml/ci.yml YAML lints clean; e2e job present.
- **Max feedback latency:** structural checks < 10s; full gate bounded by workspace test build.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Decision | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|----------|------------|-----------------|-----------|-------------------|-------------|--------|
| 225-01-xx | 01 | 1 | D-01/D-02 | — | TLS still functions via ring; no openssl C linkage | structural | `cargo tree -p ferro-cli --edges no-dev -e features \| grep -E 'native-tls\|openssl-sys' \| wc -l` → 0 | ❌ W0 | ⬜ pending |
| 225-01-xx | 01 | 1 | D-02 | — | aws-lc-rs absent (no cmake/C build) | structural | `cargo tree -p ferro-cli --edges no-dev \| grep -E 'aws-lc-sys\|aws-lc-rs' \| wc -l` → 0 | ❌ W0 | ⬜ pending |
| 225-01-xx | 01 | 1 | D-02 | — | ring present | structural | `cargo tree -p ferro-cli --edges no-dev \| grep 'ring v'` → ≥1 line | ❌ W0 | ⬜ pending |
| 225-01-xx | 01 | 1 | D-01 | — | workspace compiles after backend swap | build | `cargo build --all-features` → exit 0 | ❌ W0 | ⬜ pending |
| 225-01-xx | 01 | 1 | D-01 | — | tests stay green | test | `cargo test --all-features` → exit 0 | ❌ W0 | ⬜ pending |
| 225-01-xx | 01 | 1 | D-05 | — | advisory/license surface stays green | deny | `cargo deny check` → exit 0 | ❌ W0 | ⬜ pending |
| 225-02-xx | 02 | 2 | D-04 | — | aarch64-linux built natively (no `cross`) with gcc cross-linker + ring CC env | release build | aarch64 matrix job exit 0 (observed in CI) | ❌ W0 | ⬜ pending |
| 225-03-xx | 03 | 2 | D-06/D-08 | — | real released binary scaffolds + builds COMP-04 sequence | e2e (CI) | e2e-from-release job exit 0 (`continue-on-error: true` initially per D-10) | ❌ W0 | ⬜ pending |
| 225-03-xx | 03 | 2 | D-09 | — | existing fast smoke unaffected | existing test | `cargo test -p ferro-cli scaffold_builds_against_workspace_ferro` → pass | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky. Task IDs `-xx` finalized by the planner.*

---

## Wave 0 Requirements

- No new test framework — `cargo test`, `cargo tree`, `cargo deny` are already present.
- The e2e job reuses `ferro-cli/tests/benchmark_new_project.rs` + `tests/fixtures/benchmark/` (existing apparatus).
- *Existing infrastructure covers all phase verifications; the structural `cargo tree` greps are new acceptance criteria, not new frameworks.*

---

## Manual-Only Verifications

| Behavior | Decision | Why Manual | Test Instructions |
|----------|----------|------------|-------------------|
| Clean-container `cargo install ferro-cli` with **no** `libssl-dev`/`pkg-config` | D-01 | Needs a fresh `debian:bookworm-slim` container; not run on every commit | Build the COMP-04 Dockerfile WITHOUT the `libssl-dev pkg-config` apt line; `cargo install ferro-cli`; expect success |
| aarch64-linux release artifact runs on real arm64 hardware | D-04 | CI builds but does not execute the foreign-arch binary | On an arm64 host, download the release artifact, `./ferro --version` |

---

## Validation Sign-Off

- [ ] All tasks have an `<automated>` verify (cargo tree / build / test / deny / CI job exit) or a Manual-Only entry
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (none — existing infra)
- [ ] No watch-mode flags
- [ ] Feedback latency acceptable (structural checks < 10s)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
