---
phase: 129
slug: publish-workflow-refinement
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-09
---

# Phase 129 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust 2021, workspace) |
| **Config file** | `Cargo.toml` (workspace root) |
| **Quick run command** | `cargo test -p ferro-cli --lib deploy::` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~120 seconds full suite, ~15 seconds quick |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-cli --lib deploy::` (quick — covers the parser + rewriter modules touched by this phase)
- **After every plan wave:** Run `cargo clippy --all --all-targets -- -D warnings && cargo test -p ferro-cli`
- **Before `/gsd:verify-work`:** Full CI-equivalent suite must be green (fmt + clippy + test --all-features)
- **Max feedback latency:** 20 seconds (quick), 120 seconds (full)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 129-01-01 | 01 | 1 | REPORT §8 | shell/CI | `grep -q 'should_publish=none' .github/workflows/publish.yml` | ✅ | ⬜ pending |
| 129-01-02 | 01 | 1 | REPORT §8 | shell/CI | `grep -qE "if: needs.check-version.outputs.should_publish == 'bump' \|\| needs.check-version.outputs.should_publish == 'yes'" .github/workflows/publish.yml` | ✅ | ⬜ pending |
| 129-02-01 | 02 | 1 | REPORT §14 | unit | `cargo test -p ferro-cli --lib project::tests::parses_ferro_versions_override` | ❌ W0 | ⬜ pending |
| 129-02-02 | 02 | 1 | REPORT §14 | unit | `cargo test -p ferro-cli --lib project::tests::rejects_ferro_versions_wrong_type` | ❌ W0 | ⬜ pending |
| 129-02-03 | 02 | 2 | REPORT §14 | unit | `cargo test -p ferro-cli --lib deploy::rewrite_ferro_version::tests::preserves_ferro_versions_override_roundtrip` | ❌ W0 | ⬜ pending |
| 129-03-01 | 03 | 2 | REPORT §§8,14 | doc | `grep -q '## Version Model' PUBLISHING.md && grep -q '## Publish Gating' PUBLISHING.md` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

Note: `phase_req_ids` is null for Phase 129 (requirements are the two REPORT items absorbed — §8 and §14 — referenced canonically from `.planning/phases/126-deploy-experience-feedback/REPORT.md`).

---

## Wave 0 Requirements

- [ ] Add new `#[test]` functions to `ferro-cli/src/project.rs` tests module:
  - `parses_ferro_versions_override` — happy path
  - `rejects_ferro_versions_wrong_type` — non-string values error with existing wrong-type pattern
- [ ] Add new `#[test]` function to `ferro-cli/src/deploy/rewrite_ferro_version.rs` tests module:
  - `preserves_ferro_versions_override_roundtrip` — table survives `Cargo.docker.toml` rewrite intact
- [ ] No new framework install required — `cargo test` workspace harness already present.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Actual bump-skip on a docs-only push | REPORT §8 | Requires a real push to `master` against GitHub Actions; cannot be exercised from local `cargo test` | After merge: push a `docs/`-only change and confirm no `bump-version` / publish job runs in the Actions tab |
| Actual bump-trigger on a library crate change | REPORT §8 | Same as above | Push a trivial change to e.g. `framework/src/lib.rs`; confirm bump + publish waves run |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies (TDD tasks create their own tests inline)
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (self-creating tests within Plan 02 tasks)
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-09 (TDD self-creating test pattern)
