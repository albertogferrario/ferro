---
phase: 189-ferro-stripe-manual-capture
plan: "04"
subsystem: ferro-stripe
tags: [stripe, docs, manual-capture, ferro-reservation]
dependency_graph:
  requires: [189-01, 189-02, 189-03]
  provides: [Manual Capture documentation section in stripe.md]
  affects: [docs/src/features/stripe.md]
tech_stack:
  added: []
  patterns: [neutral architectural documentation voice, correspondence table with explicit no-compile-dependency framing]
key_files:
  created: []
  modified:
    - docs/src/features/stripe.md
decisions:
  - "Section inserted between Stripe Connect and Webhook Configuration per D-09"
  - "ferro-reservation correspondence table framed as semantic convention with explicit no compile-time dependency sentence per D-10"
  - "~7-day window hedged with 'approximately' per RESEARCH Secondary-source note (D-11)"
  - "capture idempotency caveat reinforced in prose per T-189-12 mitigation requirement"
metrics:
  duration: "~6 minutes"
  completed: "2026-06-07T15:39:26Z"
  tasks_completed: 1
  files_modified: 1
requirements: [STRIPE-MC-05]
---

# Phase 189 Plan 04: Manual Capture Documentation Summary

One-line summary: `docs/src/features/stripe.md` gains a `## Manual Capture` section covering the authorize-at-checkout to capture/cancel flow, webhook lifecycle, Connect composition, operational realities, and the ferro-reservation hold/commit/release correspondence table.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add the Manual Capture documentation section | f6c19ec3 | docs/src/features/stripe.md |

## What Was Built

### Task 1 — Manual Capture section (f6c19ec3)

Inserted a new `## Manual Capture` section at line 231 of `docs/src/features/stripe.md`, positioned between `## Stripe Connect` (line 175) and `## Webhook Configuration` (line 321). The section contains all seven required elements:

1. **Intro** — two sentences: authorizes without charging; useful for booking deposits.

2. **Authorize at checkout** (`###`) — full `CheckoutBuilder::new(Mode::Payment).manual_capture()` example with `LineItem` named "Booking deposit", `unit_amount_cents: 5000`, `idempotency_key("booking-deposit-42")`. Note: `manual_capture()` + `Mode::Subscription` returns `Error::ManualCaptureRequiresPaymentMode` before any network call.

3. **Capture and cancel** (`###`) — code block using `ferro_stripe::payment_intent` module showing all four calls: `capture(&id, None)`, `capture(&id, Some(2000))`, `cancel(&id)`, `retrieve(&id)`. Includes idempotency caveat (async-stripe 0.41 double-capture risk; DB unique constraint recommended) per T-189-12.

4. **Webhook lifecycle** (`###`) — table mapping `payment_intent.amount_capturable_updated` → `StripePaymentIntentAmountCapturableUpdated` and `payment_intent.canceled` → `StripePaymentIntentCanceled`. Dispatch note: same registration path as existing typed events.

5. **Operational realities** (D-11) — approximately 7-day hold window with hedged language; auto-cancellation surfaces as `payment_intent.canceled` with `cancellation_reason`; partial capture auto-releases remainder.

6. **Connect composition** — `manual_capture()` composes with `destination()`; authorization on platform account; capture/cancel are platform-scoped only.

7. **Correspondence with ferro-reservation** (`###`) — table: `hold()` ↔ authorize, `commit()` ↔ `capture(id, amount)`, `release()` ↔ `cancel(id)`. Closing sentence verbatim per D-10: "This is a documented semantic correspondence — a convention for pairing a reservation hold with a payment authorization. There is no compile-time dependency between the two crates."

## Phase-Close Gate

All three gates passed serially:

1. `cargo fmt --all -- --check` — **PASS** (no output)
2. `cargo clippy --all --all-targets -- -D warnings` — **PASS** (Finished dev profile, 0 warnings)
3. `cargo test --all-features` — **PASS** (0 failures across entire workspace)

## Deviations from Plan

None — plan executed exactly as written. All seven elements present, all acceptance criteria green.

## Threat Mitigations Applied

Per plan threat register:
- **T-189-11** (Information Disclosure — code examples): All examples use placeholder ids (`acct_xxx`, `"booking-deposit-42"`) and `app.example.com` URLs. No real keys or tenant identifiers.
- **T-189-12** (Tampering — idempotency caveat omission): Explicitly documented in the Capture and cancel subsection: async-stripe 0.41 does not forward per-request idempotency keys; DB unique constraint required to prevent double-capture on retry.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes. Documentation only.

## Known Stubs

None — the documentation section is fully wired to the shipped API from Plans 01-03.

## Self-Check

PASSED
- `grep -q "^## Manual Capture" docs/src/features/stripe.md` — PASS
- Section positioned between `## Stripe Connect` (line 175) and `## Webhook Configuration` (line 321) — PASS (Manual Capture at line 231)
- `grep -q "manual_capture()" docs/src/features/stripe.md` — PASS
- `grep -c "payment_intent::capture" docs/src/features/stripe.md` == 4 — PASS
- `grep -q "payment_intent::cancel" docs/src/features/stripe.md` — PASS
- `grep -q "payment_intent::retrieve" docs/src/features/stripe.md` — PASS
- `grep -q "ManualCaptureRequiresPaymentMode" docs/src/features/stripe.md` — PASS
- `grep -q "StripePaymentIntentAmountCapturableUpdated" docs/src/features/stripe.md` — PASS
- `grep -q "StripePaymentIntentCanceled" docs/src/features/stripe.md` — PASS
- `grep -q "payment_intent.amount_capturable_updated" docs/src/features/stripe.md` — PASS
- `grep -q "payment_intent.canceled" docs/src/features/stripe.md` — PASS
- `grep -qi "approximately 7 days\|7 days" docs/src/features/stripe.md` — PASS
- `grep -q "hold()" docs/src/features/stripe.md` — PASS
- `grep -q "commit()" docs/src/features/stripe.md` — PASS
- `grep -q "release()" docs/src/features/stripe.md` — PASS
- `grep -q "no compile-time dependency" docs/src/features/stripe.md` — PASS
- Commit `f6c19ec3` exists — confirmed
- Phase-close gate: fmt PASS, clippy PASS, tests PASS
