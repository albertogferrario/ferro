---
phase: 100
slug: ai-structured-classification-confirmation-primitives
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-22
---

# Phase 100 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` / `#[tokio::test]` |
| **Config file** | None (cargo test) |
| **Quick run command** | `cargo test -p ferro-ai` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-ai`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 100-01-01 | 01 | 0 | AI-01 | unit | `cargo test -p ferro-ai -- classifier::provider` | ❌ W0 | ⬜ pending |
| 100-01-02 | 01 | 0 | AI-02 | unit (mock) | `cargo test -p ferro-ai -- classifier::anthropic` | ❌ W0 | ⬜ pending |
| 100-01-03 | 01 | 0 | AI-03 | unit | `cargo test -p ferro-ai -- classifier::config` | ❌ W0 | ⬜ pending |
| 100-01-04 | 01 | 0 | CONF-01 | unit | `cargo test -p ferro-ai -- confirmation::store` | ❌ W0 | ⬜ pending |
| 100-01-05 | 01 | 0 | CONF-02 | unit (tokio::test) | `cargo test -p ferro-ai -- confirmation::store::ttl` | ❌ W0 | ⬜ pending |
| 100-01-06 | 01 | 0 | CONF-03 | unit | `cargo test -p ferro-ai -- confirmation::events` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-ai/Cargo.toml` — new crate manifest
- [ ] `ferro-ai/src/lib.rs` — crate root with module declarations
- [ ] `ferro-ai/src/error.rs` — Error enum (thiserror)
- [ ] `ferro-ai/src/classifier/mod.rs` — classifier module
- [ ] `ferro-ai/src/classifier/provider.rs` — ClassificationProvider trait
- [ ] `ferro-ai/src/classifier/anthropic.rs` — AnthropicProvider implementation
- [ ] `ferro-ai/src/confirmation/mod.rs` — confirmation module
- [ ] `ferro-ai/src/confirmation/store.rs` — ConfirmationStore trait + InMemoryConfirmationStore
- [ ] `ferro-ai/src/confirmation/events.rs` — ConfirmationExpired event

*All files are new — this is a new crate created from scratch.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| AnthropicProvider real API call | AI-02 | Requires API key + network | Set ANTHROPIC_API_KEY, run `cargo test -p ferro-ai -- --ignored anthropic_live` |

*All other behaviors have automated verification via mocked HTTP and in-memory stores.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
