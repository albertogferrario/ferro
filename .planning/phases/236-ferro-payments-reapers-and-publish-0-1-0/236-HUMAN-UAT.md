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
result: [pending]

## Summary

total: 1
passed: 0
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps
