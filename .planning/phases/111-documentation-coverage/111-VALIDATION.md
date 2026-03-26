---
phase: 111
slug: documentation-coverage
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-26
---

# Phase 111 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust native) |
| **Config file** | workspace Cargo.toml |
| **Quick run command** | `cargo test --all-features -p ferro-projections` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --all-features -p ferro-projections`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 111-01-01 | 01 | 1 | DOC-01 | manual | `test -f docs/src/features/projections.md` | ❌ W0 | ⬜ pending |
| 111-01-02 | 01 | 1 | DOC-01 | manual | `grep projections docs/src/SUMMARY.md` | ✅ exists | ⬜ pending |
| 111-02-01 | 02 | 1 | DOC-02 | manual | `grep FerroModel docs/src/features/derive-macros.md` | ❌ W0 | ⬜ pending |
| 111-02-02 | 02 | 1 | DOC-03 | manual | `grep ValidateRules docs/src/features/derive-macros.md` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. This is a documentation-only phase — no new test files or fixtures needed. The existing `cargo test --all-features` suite serves as the regression gate to ensure doc changes don't break anything.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Projections page covers ServiceDef → IntentGraph → Renderer pipeline with worked example | DOC-01 | Content quality requires human review | Read projections.md; verify pipeline section exists and contains a complete code example |
| FerroModel has at least one complete usage example | DOC-02 | Content quality requires human review | Read derive-macros.md; verify FerroModel section with `#[derive(FerroModel)]` example |
| ValidateRules has at least one complete usage example | DOC-03 | Content quality requires human review | Read derive-macros.md; verify ValidateRules section with `#[rule(...)]` example |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
