---
phase: 256
slug: component-renderers-builtin-lockstep
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-06
---

# Phase 256 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml (workspace) |
| **Quick run command** | `cargo test -p ferro-json-ui` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | quick ~60s / full ~10min |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui`
- **After every plan wave:** Run `cargo test --all-features` (serialize — one CPU-heavy run at a time)
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| (filled by planner) | — | — | POS-01/03/04/05/06/09 | — | HTML-escape all prop-derived output | unit (HTML assertion) | `cargo test -p ferro-json-ui` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements — `ferro-json-ui` has in-crate
test modules (render HTML assertions, runtime inline-source tests, drift guards,
`variant_classes_use_semantic_tokens`, BUILTIN_SPECS render smoke) that new tests
extend. No new framework or fixtures needed.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live SelectionPanel feel (tap tile → line appears, total updates, EmptyState toggles) | POS-04 | Client-side JS behavior in a real browser; inline-source tests verify presence, not interaction | Serve a register spec page, tap tiles, edit quantities in the panel, verify total and remove-on-zero (Chrome MCP) |
| Touch quality on tablet-size viewport (44/56px targets, press states) | POS-01/03/06 | Visual/interaction quality | Chrome MCP device emulation over the sample spec |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 600s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
