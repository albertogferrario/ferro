---
phase: 108
slug: p0-accuracy-fixes
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-26
---

# Phase 108 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | grep (shell) — no Rust test suite for docs-only changes |
| **Config file** | none |
| **Quick run command** | `grep -rn "ferro_rs::" docs/src/` |
| **Full suite command** | See Per-Task Verification Map below (all 5 smoke greps) |
| **Estimated runtime** | ~1 second |

---

## Sampling Rate

- **After every task commit:** Run the smoke grep for that task's requirement
- **After every plan wave:** Run all 5 smoke greps
- **Before `/gsd:verify-work`:** Full suite must be green (all greps return zero matches)
- **Max feedback latency:** 1 second

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 108-01-01 | 01 | 1 | ACC-01 | smoke | `grep -rn "ferro_rs::" docs/src/` → no output | N/A (shell) | ⬜ pending |
| 108-02-01 | 02 | 1 | ACC-02 | smoke | `grep -n "// TODO: Implement" docs/src/reference/cli.md` → no output | N/A (shell) | ⬜ pending |
| 108-02-02 | 02 | 1 | ACC-03 | smoke | `grep -n "Work in Progress" README.md` → no output | N/A (shell) | ⬜ pending |
| 108-02-03 | 02 | 1 | ACC-04 | smoke | `grep -n "coming soon" docs/src/features/storage.md` → no output | N/A (shell) | ⬜ pending |
| 108-02-04 | 02 | 1 | ACC-05 | manual | Verify any count claim matches `grep -c "#\[tool(" ferro-mcp/src/service.rs` → 65 | N/A (shell) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. No test files need creation.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| README accuracy audit | ACC-03 | Subjective — must verify no other false claims remain | Read README.md end-to-end after edits; confirm all feature claims match shipped state |
| CLI stub quality | ACC-02 | Must verify examples show real logic, not just removed comments | Read each replaced stub; confirm 1-2 lines of idiomatic logic present |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 1s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
