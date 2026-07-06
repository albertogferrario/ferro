---
phase: 159
slug: v12-0-end-to-end-browser-verification-and-docs-build-check
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-15
---

# Phase 159 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Chrome DevTools MCP + mdbook CLI + Bash assertions |
| **Config file** | docs/book.toml (mdbook), app/.env (server config) |
| **Quick run command** | `mdbook build docs/` |
| **Full suite command** | `mdbook build docs/ && echo "DOCS OK"` |
| **Estimated runtime** | ~10 seconds (docs build) |

---

## Sampling Rate

- **After every task commit:** Run `mdbook build docs/` (docs check) or Chrome MCP screenshot (browser check)
- **After every plan wave:** Run full verification suite
- **Before `/gsd-verify-work`:** Both checks (mdbook + Chrome MCP) must be green
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 159-01-01 | 01 | 1 | REQ-DOCS | — | mdbook build exits 0 | integration | `mdbook build docs/; echo $?` | ✅ | ⬜ pending |
| 159-01-02 | 01 | 1 | REQ-DOCS | — | No internal broken links; DOCS-CHECK.md verdict captured | integration | `grep -q "PASS" DOCS-CHECK.md` | ✅ | ⬜ pending |
| 159-02-01 | 02 | 2 | REQ-BROWSER | — | User starts server; agent waits for resume signal | manual | `echo "checkpoint:human-action"` | ✅ | ⬜ pending |
| 159-02-02 | 02 | 2 | REQ-BROWSER | — | HTTP 200, StatCard + DataTable visible, no panic text, screenshot saved | manual/MCP | `test -s .planning/phases/159-*/pagamenti-screenshot.png && grep -q "PASS" .planning/phases/159-*/BROWSER-CHECK.md` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements. No new test files needed — this is a verification phase using existing tools (mdbook CLI, Chrome MCP).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Chrome MCP browser test of /pagamenti | REQ-BROWSER | Requires live server (user must start `cd app && cargo run`); agent cannot start server per CLAUDE.md | Navigate to http://localhost:8080/pagamenti, screenshot, verify "Totale", "€ 1.245,00", "Data", "Descrizione", "Importo", "Stato" visible |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
