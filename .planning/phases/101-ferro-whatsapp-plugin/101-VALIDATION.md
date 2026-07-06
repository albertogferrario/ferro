---
phase: 101
slug: ferro-whatsapp-plugin
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-23
---

# Phase 101 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `#[tokio::test]` |
| **Config file** | None (workspace-level) |
| **Quick run command** | `cargo test -p ferro-whatsapp` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-whatsapp`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 101-01-01 | 01 | 1 | WA-01 | unit (mock HTTP) | `cargo test -p ferro-whatsapp test_send_text` | ❌ W0 | ⬜ pending |
| 101-01-02 | 01 | 1 | WA-01 | unit | `cargo test -p ferro-whatsapp test_send_template_payload` | ❌ W0 | ⬜ pending |
| 101-01-03 | 01 | 1 | WA-01 | unit | `cargo test -p ferro-whatsapp test_error_mapping` | ❌ W0 | ⬜ pending |
| 101-02-01 | 02 | 1 | WA-02 | unit | `cargo test -p ferro-whatsapp verify_webhook_valid` | ❌ W0 | ⬜ pending |
| 101-02-02 | 02 | 1 | WA-02 | unit | `cargo test -p ferro-whatsapp verify_webhook_tampered` | ❌ W0 | ⬜ pending |
| 101-02-03 | 02 | 1 | WA-02 | unit | `cargo test -p ferro-whatsapp verify_webhook_wrong_secret` | ❌ W0 | ⬜ pending |
| 101-02-04 | 02 | 1 | WA-02 | unit | `cargo test -p ferro-whatsapp verify_webhook_bad_prefix` | ❌ W0 | ⬜ pending |
| 101-03-01 | 03 | 1 | WA-03 | unit | `cargo test -p ferro-whatsapp dedup_first_insert` | ❌ W0 | ⬜ pending |
| 101-03-02 | 03 | 1 | WA-03 | unit | `cargo test -p ferro-whatsapp dedup_duplicate` | ❌ W0 | ⬜ pending |
| 101-03-03 | 03 | 1 | WA-03 | unit (tokio time) | `cargo test -p ferro-whatsapp dedup_ttl_expiry` | ❌ W0 | ⬜ pending |
| 101-04-01 | 04 | 1 | WA-04 | unit | `cargo test -p ferro-whatsapp sender_identity_owner` | ❌ W0 | ⬜ pending |
| 101-04-02 | 04 | 1 | WA-04 | unit | `cargo test -p ferro-whatsapp sender_identity_customer` | ❌ W0 | ⬜ pending |
| 101-05-01 | 05 | 2 | WA-05 | unit | `cargo test -p ferro-cli make_whatsapp_generates_files` | ❌ W0 | ⬜ pending |
| 101-05-02 | 05 | 2 | WA-05 | unit | `cargo test -p ferro-cli make_whatsapp_no_overwrite` | ❌ W0 | ⬜ pending |
| 101-05-03 | 05 | 2 | WA-05 | unit | `cargo test -p ferro-mcp whatsapp_config_status_missing` | ❌ W0 | ⬜ pending |
| 101-05-04 | 05 | 2 | WA-05 | unit | `cargo test -p ferro-mcp whatsapp_webhook_events_parsed` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-whatsapp/Cargo.toml` — new crate workspace member
- [ ] `ferro-whatsapp/src/lib.rs` — crate root with module declarations
- [ ] Update `/Cargo.toml` `members` array to include `"ferro-whatsapp"`
- [ ] Update `framework/Cargo.toml` to add optional `ferro-whatsapp` dependency and `whatsapp` feature flag
- [ ] Update `.github/workflows/publish.yml` Wave 1 CRATES list

*All test files created as part of Wave 0 crate setup.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Meta webhook challenge responds correctly | WA-02 | Requires real Meta webhook verification request | Use `curl` to simulate GET with `hub.verify_token` + `hub.challenge` params |
| End-to-end WhatsApp message delivery | WA-01 | Requires Meta Business account + phone number | Send test message via sandbox number, verify delivery |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
