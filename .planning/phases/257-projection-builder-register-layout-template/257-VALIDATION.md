---
phase: 257
slug: projection-builder-register-layout-template
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-06
---

# Phase 257 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml (workspace) |
| **Quick run command** | `cargo test -p ferro-json-ui --features projections` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | quick ~60s · full ~10min |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui --features projections`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green + CI-exact gate (fmt --check, clippy --all-targets --all-features -D warnings, cargo doc)
- **Max feedback latency:** ~120 seconds (serialize CPU-heavy runs — one at a time)

---

## Per-Task Verification Map

*To be filled by the planner — one row per task, mapping to POS-10 success criteria.*

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| — | — | — | POS-10 | — | — | unit/integration | `cargo test -p ferro-json-ui --features projections` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements — `ferro-json-ui` has
established unit/integration test modules (projection builder tests with the
injected-catalog pattern, spec builder tests, design lint fixture tests) and
the `app` crate has controller test patterns. No new framework install needed.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `/cassa` visual quality on a tablet viewport (fill-viewport panes scroll independently, register feel) | POS-10 | Visual interaction quality is not assertable via HTML string tests | Run the app, open `/cassa` in Chrome DevTools MCP at tablet viewport, tap tiles, verify panel updates + no page scroll |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 600s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
