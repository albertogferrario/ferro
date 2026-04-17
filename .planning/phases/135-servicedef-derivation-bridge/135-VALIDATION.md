---
phase: 135
slug: servicedef-derivation-bridge
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-17
---

# Phase 135 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml (workspace) |
| **Quick run command** | `cargo test -p ferro-projections --all-features` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-projections --all-features`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 135-01-01 | 01 | 1 | ModelMetadata + from_model | unit | `cargo test -p ferro-projections test_from_model` | ❌ W0 | ⬜ pending |
| 135-01-02 | 01 | 1 | DataType::from_column_type | unit | `cargo test -p ferro-projections test_from_column_type` | ❌ W0 | ⬜ pending |
| 135-02-01 | 02 | 2 | generate_projection MCP tool | integration | `cargo test -p ferro-mcp test_generate_projection` | ❌ W0 | ⬜ pending |
| 135-02-02 | 02 | 2 | Round-trip test | integration | `cargo test -p ferro-mcp test_round_trip` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Tests created as part of implementation tasks (TDD not required — test after implementation per project convention)
- [ ] Existing test infrastructure covers all needs — no new framework install required

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
