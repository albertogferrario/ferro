---
phase: 110
slug: mcp-tool-accuracy
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-26
---

# Phase 110 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (all-features) |
| **Config file** | Cargo.toml workspace |
| **Quick run command** | `cargo test -p ferro-mcp --all-features` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-mcp --all-features`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 110-01-01 | 01 | 1 | CLIMCP-02 | manual | Review tool descriptions in service.rs | N/A | ⬜ pending |
| 110-02-01 | 02 | 1 | CLIMCP-03 | unit | `cargo test -p ferro-mcp --all-features` | ✅ | ⬜ pending |
| 110-02-02 | 02 | 1 | CLIMCP-03 | compile | `cargo clippy --all --all-targets -- -D warnings` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Tool descriptions reference valid framework types | CLIMCP-02 | Descriptions are string literals — no compile-time check | Cross-reference each tool's description types against framework/src/lib.rs exports |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
