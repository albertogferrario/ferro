---
phase: 193
slug: ferro-stripe-refund-event-completeness
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-09
---

# Phase 193 — Validation Strategy

> Additive ferro-stripe change. Verification = a parser-contract test (fixture
> round-trip) + grep/version assertions. Publish (SC6/SC7) is an operator-owned
> deferred step, NOT verified in this code phase.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust) — extends `ferro-stripe/tests/parser_contract.rs` |
| **Quick run command** | `cargo test -p ferro-stripe --test parser_contract` |
| **Full suite command** | `cargo test --all-features && cargo clippy --all --all-targets -- -D warnings` |

---

## Per-Requirement Verification Map

| Req / SC | Behavior | Verify | Status |
|----------|----------|--------|--------|
| STRIPE-REFUND-01 / SC1 | `refund_id: Option<String>` field present between `payment_intent_id` and `amount_refunded_cents` | `grep -q 'pub refund_id: Option<String>' ferro-stripe/src/webhook/events.rs` | ⬜ |
| STRIPE-REFUND-01 / SC2 | parser populates from `charge.refunds.data[].id` (Charge object, not EventObject::Refund) | `grep -q 'refunds' ferro-stripe/src/webhook/events.rs`; parser-contract test green | ⬜ |
| STRIPE-REFUND-01 / SC3 | fixture has `refunds.data[].id`; test asserts `refund_id == Some("re_...")` | `grep -q 'refunds' ferro-stripe/tests/fixtures/stripe_events/charge_refunded.json` AND `cargo test -p ferro-stripe --test parser_contract` exits 0 | ⬜ |
| STRIPE-REFUND-02 / SC4 | version 0.5.0 → 0.7.0; CHANGELOG `## [0.7.0]` with the three documented items | `grep -q 'version = "0.7.0"' ferro-stripe/Cargo.toml` AND `grep -q '0.7.0' ferro-stripe/CHANGELOG.md` | ⬜ |
| — / SC5 | full gate green | `cargo test --all-features` + `cargo clippy --all --all-targets -- -D warnings` clean | ⬜ |

*Status: ⬜ pending · ✅ green*

---

## Wave 0 Requirements

- None — edits land in existing files (`events.rs`, the fixture, `parser_contract.rs`, `Cargo.toml`) + a new `CHANGELOG.md`.

---

## Manual-Only / Deferred Verifications

| Behavior | Requirement | Why Deferred | Instructions |
|----------|-------------|--------------|--------------|
| Publish ferro-stripe 0.7.0 to crates.io (ROADMAP SC6) + `cargo search` shows 0.7.0 (SC7) | STRIPE-REFUND-02 (publish) | Operator-owned outward action (user decision 2026-06-09: build code, stop before publish) | After this phase commits, run `git push` to master → GH Actions auto-publishes 0.7.0; then `cargo search ferro-stripe --limit 1` returns `0.7.0`. Unblocks gestiscilo Phase 99. |

---

## Validation Sign-Off

- [ ] SC1–SC5 grep + parser-contract test green
- [ ] `cargo test --all-features` + clippy clean
- [ ] Publish (SC6/SC7) explicitly recorded as the one remaining operator step in 193-VERIFICATION.md
- [ ] `nyquist_compliant: true` after code SCs pass

**Approval:** pending — set when plans pass the checker
