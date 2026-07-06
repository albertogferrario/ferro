---
phase: 96
slug: stripe-integration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-11
---

# Phase 96 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test framework + `tokio::test` |
| **Config file** | none — uses `cargo test` conventions |
| **Quick run command** | `cargo test -p ferro-stripe` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-stripe`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 96-01-01 | 01 | 1 | STRIPE-01 | unit | `cargo test -p ferro-stripe subscription::` | ❌ W0 | ⬜ pending |
| 96-01-02 | 01 | 1 | STRIPE-02 | unit | `cargo test -p ferro-stripe subscription::` | ❌ W0 | ⬜ pending |
| 96-01-03 | 01 | 1 | STRIPE-03 | unit | `cargo test -p ferro-stripe webhook::` | ❌ W0 | ⬜ pending |
| 96-01-04 | 01 | 1 | STRIPE-04 | unit | `cargo test -p ferro-stripe webhook::` | ❌ W0 | ⬜ pending |
| 96-01-05 | 01 | 1 | STRIPE-05 | unit | `cargo test -p ferro-stripe middleware::` | ❌ W0 | ⬜ pending |
| 96-01-06 | 01 | 1 | STRIPE-06 | unit | `cargo test -p ferro-stripe middleware::` | ❌ W0 | ⬜ pending |
| 96-01-07 | 01 | 1 | STRIPE-07 | unit | `cargo test -p ferro-stripe` | ❌ W0 | ⬜ pending |
| 96-01-08 | 01 | 1 | STRIPE-08 | unit | `cargo test -p ferro-rs tenant::` | ❌ W0 | ⬜ pending |
| 96-01-09 | 01 | 1 | STRIPE-09 | unit | `cargo test -p ferro-cli make_stripe::` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-stripe/src/lib.rs` — crate entry point (does not exist yet)
- [ ] `ferro-stripe/Cargo.toml` — new crate (does not exist yet)
- [ ] Add `ferro-stripe` to `Cargo.toml` workspace members
- [ ] Add `ferro-stripe` to Wave 1 in `.github/workflows/publish.yml`
- [ ] Add `stripe` feature to `framework/Cargo.toml` optional deps

*Existing test infrastructure covers test execution — Wave 0 creates the crate scaffold only.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Stripe Checkout redirect works end-to-end | STRIPE-CONNECT | Requires live/test Stripe keys | Use Stripe test mode, create checkout session, verify redirect URL |
| Billing Portal redirect works | STRIPE-PORTAL | Requires live/test Stripe keys | Use Stripe test mode, create portal session, verify redirect |
| Webhook delivery from Stripe CLI | STRIPE-WEBHOOK | External dependency | `stripe listen --forward-to localhost:8080/stripe/webhook` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
