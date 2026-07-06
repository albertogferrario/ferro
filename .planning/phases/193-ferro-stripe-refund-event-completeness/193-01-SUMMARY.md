---
phase: 193-ferro-stripe-refund-event-completeness
plan: "01"
subsystem: ferro-stripe
tags: [stripe, payments, webhook, refund, events, release]
dependency_graph:
  requires: [189-01, 189-02, 189-03]
  provides: [StripeChargeRefunded::refund_id, ferro-stripe 0.7.0 label, CHANGELOG.md]
  affects:
    - ferro-stripe/src/webhook/events.rs
    - ferro-stripe/tests/fixtures/stripe_events/charge_refunded.json
    - ferro-stripe/tests/parser_contract.rs
    - ferro-stripe/Cargo.toml
    - ferro-stripe/CHANGELOG.md
    - framework/Cargo.toml
tech_stack:
  added: []
  patterns: [Option<String> on typed event struct, charge.refunds.data[0].id accessor, Keep-a-Changelog format]
key_files:
  created:
    - ferro-stripe/CHANGELOG.md
  modified:
    - ferro-stripe/src/webhook/events.rs
    - ferro-stripe/tests/fixtures/stripe_events/charge_refunded.json
    - ferro-stripe/tests/parser_contract.rs
    - ferro-stripe/Cargo.toml
    - framework/Cargo.toml
decisions:
  - "refund_id parsed from charge.refunds.data[0].id inside the existing EventObject::Charge arm — NOT EventObject::Refund (a charge.refunded event carries a Charge)"
  - "Returns None for absent/empty refunds list — defensive guard, never panics"
  - "CHANGELOG starts at 0.7.0; earlier ferro-stripe history not reconstructed per D-07"
  - "framework/Cargo.toml pin updated from 0.5 to 0.7 (Rule 1 deviation — version bump broke the version constraint)"
metrics:
  duration: "~20 minutes"
  completed: "2026-06-09T18:20:00Z"
  tasks_completed: 2
  files_modified: 6
requirements: [STRIPE-REFUND-01, STRIPE-REFUND-02]
---

# Phase 193 Plan 01: ferro-stripe Refund Event Completeness Summary

One-line summary: `StripeChargeRefunded` gains `refund_id: Option<String>` parsed from `charge.refunds.data[0].id`, proven by fixture + parser-contract round-trip; ferro-stripe labelled 0.7.0 with CHANGELOG bundling Phase 189 manual-capture additions.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add refund_id field, parse from charge.refunds, prove with fixture + parser-contract test | 07e4450d | ferro-stripe/src/webhook/events.rs, charge_refunded.json, parser_contract.rs |
| 2 | Bump ferro-stripe to 0.7.0 and author the CHANGELOG, then run the full gate | 9eddbaee | ferro-stripe/Cargo.toml, ferro-stripe/CHANGELOG.md, framework/Cargo.toml |

## What Was Built

### Task 1 — refund_id field + parser + fixture + test (07e4450d)

`StripeChargeRefunded` gains a new field between `payment_intent_id` and `amount_refunded_cents`:

```rust
pub refund_id: Option<String>,
```

Parser expression in the existing `EventObject::Charge(charge)` arm of `from_raw`:

```rust
refund_id: charge
    .refunds
    .as_ref()
    .and_then(|list| list.data.first())
    .map(|r| r.id.to_string()),
```

`Charge::refunds` is `Option<List<Refund>>` and `Refund::id` is `RefundId` in async-stripe 0.41 — matched the plan's accessor expression exactly.

`charge_refunded.json` extended with a `refunds` object carrying one refund (`id: "re_test_refunded_001"`, `amount: 2000`, `currency: "usd"`, `created: 1700000003`) — the minimal shape async-stripe's `Refund` deserializer requires.

`parser_contract.rs` gains one assertion after `amount_refunded_cents`:

```rust
assert_eq!(typed.refund_id.as_deref(), Some("re_test_refunded_001"));
```

All 19 `parser_contract` tests pass including the updated `charge_refunded_parses_all_fields`.

### Task 2 — 0.7.0 version + CHANGELOG + full gate (9eddbaee)

`ferro-stripe/Cargo.toml` version bumped from `0.5.0` to `0.7.0`.

`ferro-stripe/CHANGELOG.md` created (Keep-a-Changelog format) with `## [0.7.0] - 2026-06-09` documenting:
- `StripeChargeRefunded::refund_id: Option<String>`
- `CheckoutBuilder::manual_capture()` (Phase 189)
- `ferro_stripe::payment_intent` module with `capture`, `cancel`, `retrieve` (Phase 189)
- `StripePaymentIntentAmountCapturableUpdated` and `StripePaymentIntentCanceled` (Phase 189)
- No 0.6.x rationale

Full gate results:
- `cargo fmt --all -- --check` — PASS
- `cargo clippy --all --all-targets -- -D warnings` — PASS
- `cargo test --all-features` — PASS (all tests pass, 0 failures)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated framework/Cargo.toml ferro-stripe version pin**
- **Found during:** Task 2, first clippy run
- **Issue:** `cargo clippy` failed with `"failed to select a version for the requirement ferro-stripe = "^0.5"` — `framework/Cargo.toml` pinned `version = "0.5"` but the crate is now `0.7.0`
- **Fix:** Updated `ferro-stripe = { ..., version = "0.5", ... }` to `version = "0.7"` in `framework/Cargo.toml`
- **Files modified:** `framework/Cargo.toml`
- **Commit:** 9eddbaee (included in Task 2 commit)

## Deferred Operator Step

ferro-stripe 0.7.0 is committed and ARMED for publish. Remaining operator-owned
step: push master -> GitHub Actions auto-publishes 0.7.0; then `cargo search
ferro-stripe --limit 1` returns 0.7.0 (ROADMAP SC6/SC7). Unblocks gestiscilo Phase 99.

## Threat Mitigations Applied

- **T-193-01** (Tampering): `refund_id` reads from an already signature-verified, serde-deserialized `stripe::Event`. No new input parsing surface, no string interpolation, no injection sink.
- **T-193-02** (DoS — empty refunds panicking): `.as_ref().and_then(|l| l.data.first()).map(...)` is total over a missing/empty list — returns `None` instead of indexing. Parser-contract test confirms the happy path; the None branch is structurally guaranteed by the Option chain.
- **T-193-03** (Information Disclosure): `refund_id` (`re_...`) is not PII or a secret; already in the webhook payload the consumer is authorized to receive.
- **T-193-04** (Spoofing): Out of scope — authenticity enforced upstream by existing HMAC check before `from_raw` runs.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes introduced.

## Known Stubs

None — `refund_id` is fully wired from the fixture through the parser to the consumer-facing field.

## Self-Check

- `grep -q 'pub refund_id: Option<String>' ferro-stripe/src/webhook/events.rs` — PASS
- `grep -q 'refunds' ferro-stripe/src/webhook/events.rs` — PASS
- `grep -q 're_test_refunded_001' ferro-stripe/tests/fixtures/stripe_events/charge_refunded.json` — PASS
- `grep -q 'refund_id' ferro-stripe/tests/parser_contract.rs` — PASS
- `grep -q 'version = "0.7.0"' ferro-stripe/Cargo.toml` — PASS
- `! grep -q 'version = "0.5.0"' ferro-stripe/Cargo.toml` — PASS
- `grep -q '## \[0.7.0\]' ferro-stripe/CHANGELOG.md` — PASS
- `cargo test -p ferro-stripe --test parser_contract` — PASS (19/19)
- `cargo fmt --all -- --check` — PASS
- `cargo clippy --all --all-targets -- -D warnings` — PASS
- `cargo test --all-features` — PASS
- Commit 07e4450d exists — PASS
- Commit 9eddbaee exists — PASS

## Self-Check: PASSED
