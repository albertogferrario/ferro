---
phase: 149
slug: ferro-notifications-whatsapp-inapp-channels-mailmessage-attachment
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-28
---

# Phase 149 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Detailed test→behavior map lives in `149-RESEARCH.md` §Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (built-in, no external runner) |
| **Config file** | none — Cargo workspace handles discovery |
| **Quick run command** | `cargo test -p ferro-notifications` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | quick: ~5–10s; full: ~60–90s on this workspace |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-notifications` (quick — covers all new and modified test modules in this phase)
- **After every plan wave:** Run the full CI gate: `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 90s (full suite); 10s (quick)

---

## Per-Task Verification Map

The detailed behavior→test map is in `149-RESEARCH.md` §Validation Architecture. Each PLAN.md task surfaces concrete `automated:` commands; this section is filled in by the planner per-task.

| Task ID | Plan | Wave | Behavior | Test Type | Automated Command | Status |
|---------|------|------|----------|-----------|-------------------|--------|
| TBD | TBD | TBD | (filled by planner) | unit / integration | `cargo test ...` | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-notifications/src/channels/whatsapp.rs` — new file with at least `WhatsAppMessage::text` / `template` builder tests stubbed
- [ ] `ferro-notifications/src/channels/in_app.rs` — new file with `InAppMessage::new` / `severity` builder tests stubbed
- [ ] Mock `DatabaseNotificationStore` for `dispatcher` tests — defined inline in `dispatcher.rs` `#[cfg(test)] mod tests`
- [ ] Mailpit fixture for SMTP attachment integration test (per ROADMAP success criterion #5) — set up in `ferro-notifications/tests/` or via an integration-test feature flag

*Existing infrastructure covers the rest. Cargo test discovery is automatic.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Consumer-side smoke test in gestiscilo-it | ROADMAP criterion #7 | Cross-repo — `gestiscilo-it` consumes the published ferro-notifications version | After GH Actions publishes the new version, bump `Cargo.toml` in `gestiscilo-it`; `use ferro_notifications::{Channel, WhatsAppChannel, InAppChannel};` resolves; `MailMessage::new().attachment(...)` compiles. Out of scope for this phase's verification — gestiscilo-it Phase 120 owns this. |
| GH Actions publishes new ferro-notifications version | ROADMAP criterion #6 | Network / external service | Verify via crates.io after merge to master |
| Resend HTTP API attachment delivery (live) | ROADMAP criterion #5 (Resend driver) | Requires live API key + recipient address | Manual run with `RESEND_API_KEY` set + a test inbox; not part of CI |

*The Mailpit SMTP attachment round-trip IS automated via integration test — see Wave 0 above.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies (planner fills per-task in PLAN.md)
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (whatsapp.rs, in_app.rs, mock store, Mailpit fixture)
- [ ] No watch-mode flags in any test command
- [ ] Feedback latency < 90s for full suite
- [ ] `nyquist_compliant: true` set in frontmatter once all tasks land their `automated:` commands

**Approval:** pending — flips to approved YYYY-MM-DD once the planner completes per-task filling and all rows have `automated:` commands.
