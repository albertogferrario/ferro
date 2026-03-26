---
phase: 112
slug: agent-first-philosophy
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-26
---

# Phase 112 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | mdBook build + grep-based smoke checks |
| **Config file** | `docs/book.toml` |
| **Quick run command** | `mdbook build docs/ 2>&1 \| tail -5` |
| **Full suite command** | `mdbook build docs/` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `mdbook build docs/ 2>&1 | tail -5`
- **After every plan wave:** Run `mdbook build docs/`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 112-01-01 | 01 | 1 | PHIL-01 | smoke | `grep -c "agent-first" docs/src/introduction.md` | ✅ | ⬜ pending |
| 112-01-02 | 01 | 1 | PHIL-01 | smoke | `head -20 docs/src/introduction.md \| grep -c "MCP"` | ✅ | ⬜ pending |
| 112-02-01 | 02 | 1 | PHIL-02 | smoke | `grep -c "working-with-agents" docs/src/SUMMARY.md` | ❌ W0 | ⬜ pending |
| 112-02-02 | 02 | 1 | PHIL-02 | smoke | `test -f docs/src/getting-started/working-with-agents.md` | ❌ W0 | ⬜ pending |
| 112-02-03 | 02 | 1 | PHIL-04 | smoke | `grep -c "CLI" docs/src/getting-started/working-with-agents.md` | ❌ W0 | ⬜ pending |
| 112-02-04 | 02 | 1 | PHIL-03 | smoke | `grep -rl "## MCP Tools" docs/src/features/ \| wc -l` | ✅ (partial) | ⬜ pending |
| 112-02-05 | 02 | 1 | PHIL-03 | build | `mdbook build docs/` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `docs/src/getting-started/working-with-agents.md` — stub for PHIL-02, PHIL-04 (new file)

*All other required files exist; they are being edited, not created.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Agent callout box reads naturally | PHIL-01 | Subjective tone | Read introduction.md and verify callout doesn't feel "bolted on" |
| MCP section detail level matches tool complexity | PHIL-03 | Subjective judgment | Spot-check 3 feature pages — rich tools (database, projections) should have more detail than one-liners |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
