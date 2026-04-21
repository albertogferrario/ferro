---
phase: 142
slug: ferro-mcp-parity
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-20
---

# Phase 142 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust built-in) |
| **Config file** | `ferro-mcp/Cargo.toml` |
| **Quick run command** | `cargo test -p ferro-mcp -- tools::stripe` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-mcp -- tools::stripe`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 142-01-01 | 01 | 1 | SC1 | — | N/A | unit | `cargo test -p ferro-mcp -- tools::stripe::tests::test_webhook_events` | ✅ exists | ⬜ pending |
| 142-01-02 | 01 | 1 | SC2 | — | N/A | unit | `cargo test -p ferro-mcp -- tools::stripe::tests::test_config_status` | ✅ exists | ⬜ pending |
| 142-01-03 | 01 | 1 | SC3 | — | N/A | unit | `cargo test -p ferro-mcp -- tools::stripe::tests::test_subscription_info` | ✅ exists | ⬜ pending |
| 142-01-04 | 01 | 2 | SC4,SC5 | — | N/A | build | `cargo build -p ferro-mcp` | ✅ exists | ⬜ pending |
| 142-01-05 | 01 | 2 | SC6 | — | N/A | build | `cargo clippy --all --all-targets -- -D warnings` | ✅ exists | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-mcp/src/tools/stripe.rs` — existing test suite; update `test_webhook_events_serializes` fixture (struct change: remove `listener`, add `line`)
- [ ] New test: `test_webhook_events_turbofish` — `.on::<TypeName` pattern detection
- [ ] New test: `test_config_status_capability_axis_fields` — four boolean capability-axis fields

Wave 0 = fix the compile-time breaking test + add the two new tests BEFORE implementing the feature changes.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| MCP tool descriptions accurate | SC4 | Requires MCP client to inspect | Run `cargo build -p ferro-mcp` and read generated JSON schema output |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
