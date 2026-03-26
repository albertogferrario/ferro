---
phase: 109
slug: cli-reference-completeness
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-26
---

# Phase 109 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | None — documentation-only phase |
| **Config file** | N/A |
| **Quick run command** | `grep -c "^### " docs/src/reference/cli.md` |
| **Full suite command** | Manual review: verify 13 new sections present, each with synopsis/flags/description/example |
| **Estimated runtime** | ~2 seconds |

---

## Sampling Rate

- **After every task commit:** Run `grep -c "^### " docs/src/reference/cli.md`
- **After every plan wave:** Run full manual review
- **Before `/gsd:verify-work`:** Section count must be 50+ (37 existing + 13 new)
- **Max feedback latency:** 2 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 109-01-01 | 01 | 1 | CLIMCP-01 | manual | `grep -c "^### " docs/src/reference/cli.md` | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| 13 new CLI sections with correct format | CLIMCP-01 | Content accuracy cannot be tested automatically | Count sections, verify each has synopsis/flags/description/example matching source code |
| Command Summary table completeness | CLIMCP-01 | Table row accuracy requires human review | Verify all 13 commands appear in summary table |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 2s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
