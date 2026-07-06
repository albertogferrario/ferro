---
phase: 98
slug: ferro-json-ui-stable-release
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-11
---

# Phase 98 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` (cargo test) |
| **Config file** | none — standard cargo test runner |
| **Quick run command** | `cargo test -p ferro-json-ui` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test -p ferro-json-ui`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 98-01-01 | 01 | 1 | API-01 | unit | `cargo test -p ferro-json-ui` | ✅ | ⬜ pending |
| 98-01-02 | 01 | 1 | API-02 | unit | `cargo test -p ferro-json-ui` | ✅ | ⬜ pending |
| 98-02-01 | 02 | 1 | API-05 | unit | `cargo test -p ferro-json-ui` | ✅ | ⬜ pending |
| 98-02-02 | 02 | 1 | API-06 | unit | `cargo test -p ferro-json-ui` | ✅ | ⬜ pending |
| 98-02-03 | 02 | 1 | API-07 | unit | `cargo test -p ferro-json-ui` | ✅ | ⬜ pending |
| 98-02-04 | 02 | 1 | API-08 | unit | `cargo test -p ferro-json-ui` | ✅ | ⬜ pending |
| 98-03-01 | 03 | 2 | API-03 | unit | `cargo test -p ferro-json-ui` | ✅ | ⬜ pending |
| 98-03-02 | 03 | 2 | API-04 | compile | `cargo build -p ferro-json-ui` | ✅ | ⬜ pending |
| 98-04-01 | 04 | 3 | API-09 | unit | `cargo test -p ferro-json-ui` | ✅ | ⬜ pending |
| 98-04-02 | 04 | 3 | API-10 | suite | `cargo test -p ferro-json-ui` | ✅ | ⬜ pending |
| 98-05-01 | 05 | 4 | DOCS-01 | doc | `cargo doc -p ferro-json-ui --no-deps` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. No new test framework or fixture setup needed.

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| JS runtime SSE connection + toast stacking | API-06/API-08 | Requires running browser with SSE server | Start dev server, open dashboard, send SSE event, verify toast appears and auto-dismisses |
| Mobile sidebar collapse | API-05 | Requires viewport resize testing | Resize browser to <768px, verify sidebar becomes hamburger menu |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
