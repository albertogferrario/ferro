---
phase: 140
fixed_at: 2026-04-20T00:00:00Z
review_path: .planning/phases/140-core-reshape/140-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 140: Code Review Fix Report

**Fixed at:** 2026-04-20
**Source review:** .planning/phases/140-core-reshape/140-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 4
- Fixed: 4
- Skipped: 0

## Fixed Issues

### WR-01: Idempotency key accepted but silently ignored in `refund::create`

**Files modified:** `ferro-stripe/src/refund.rs`
**Commit:** fdbed41b
**Applied fix:** Added a doc note to the `create` function's `idempotency_key` parameter explaining that async-stripe 0.41 does not forward the key to the Stripe API, that Stripe-layer deduplication is not guaranteed, and that application-layer deduplication (e.g. a DB unique constraint on charge_id) is required to prevent duplicate refunds on retry.

### WR-02: `checkout.rs` `session.url` silently defaults to empty string on None

**Files modified:** `ferro-stripe/src/checkout.rs`
**Commit:** 8dfd8fd6
**Applied fix:** Replaced `session.url.unwrap_or_default()` with `session.url.ok_or_else(|| Error::Stripe("checkout session created but url field was absent".to_string()))?` so that an absent URL surfaces as an explicit error rather than silently redirecting to an empty string.

### WR-03: `TenantFailureMode::Custom` panics on clone — undocumented

**Files modified:** `framework/src/tenant/mod.rs`
**Commit:** ab09f781
**Applied fix:** Removed the entire `impl Clone for TenantFailureMode` block. Nothing in the codebase calls `.clone()` on this type — `TenantMiddleware` stores it by value and accesses it by reference — so removal is safe and makes the `Custom` variant's constraint correct at the type level.

### WR-04: `config.rs` test mutates process-global env vars without cleanup

**Files modified:** `ferro-stripe/src/config.rs`
**Commit:** a2fee925
**Applied fix:** The test now saves `STRIPE_SECRET_KEY` and `STRIPE_WEBHOOK_SECRET` before removing them, then restores any prior values after the assertion. This eliminates the env-var mutation hazard for parallel test execution.

---

_Fixed: 2026-04-20_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
