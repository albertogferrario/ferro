---
phase: 128
slug: deploy-preflight
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-09
---

# Phase 128 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust built-in) |
| **Config file** | `ferro-cli/Cargo.toml` |
| **Quick run command** | `cargo test -p ferro-cli doctor::checks::` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~90s full, ~15s scoped |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-cli doctor::checks::`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 90 seconds

---

## Per-Task Verification Map

Populated by planner. Each task must have either an `<automated>` verify command or a Wave 0 dependency.

---

## Wave 0 Requirements

- Test fixtures for `.dockerignore` + `copy_dirs` combinations (temp dirs via `tempfile`)
- Test fixtures for `Cargo.toml` + `Cargo.docker.toml` version-skew scenarios
- Update `default_checks_returns_nine_in_declared_order` → eleven in declared order

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Interactive `ferro deploy:init` prompt flow | REPORT item 15 | `dialoguer` TTY interaction | Run `ferro deploy:init` in a sample project, answer prompts, verify Cargo.toml write |
| MCP `deploy_check` tool shape in real MCP client | Phase 123 tie-in | Requires live MCP client | Launch `ferro mcp`, call `deploy_check`, inspect response |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
