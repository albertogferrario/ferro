---
status: partial
phase: 236-ferro-payments-reapers-and-publish-0-1-0
source: [236-VERIFICATION.md]
started: 2026-06-21
updated: 2026-06-21
---

## Current Test

[awaiting human testing]

## Tests

### 1. End-to-end integration test against Stripe test mode
expected: With a live `STRIPE_TEST_SECRET_KEY` exported, running
`cargo test -p ferro-payments --test integration -- --ignored` drives a real
checkout → release_expired round-trip against ferro-stripe test mode and passes.
Offline the test only proves the clean env-guard skip; the live key exercises the
actual reaper path. Isolate-before-spending: this is the free gated path — a
crates.io publish is not required to run it.
result: PASS — 2026-07-28. `cargo test -p ferro-payments --test integration -- --ignored` with gestiscilo `sk_test_*` key passed (0.79s). Also surfaced and fixed a bug: `stripe::Currency::parse()` requires lowercase; uppercase `"EUR"` from `Billable::currency()` was rejected. Fixed by adding `.to_lowercase()` in `ferro-stripe/src/checkout.rs` before parsing.

## Summary

total: 1
passed: 1
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
