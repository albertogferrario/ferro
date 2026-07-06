---
phase: 96-stripe-integration
plan: "02"
subsystem: framework/tenant
tags: [stripe, multi-tenant, middleware, billing, subscription]
dependency_graph:
  requires: ["96-01"]
  provides: ["TenantContext.subscription", "RequiresPlan middleware", "TenantLookup.invalidate"]
  affects: ["framework/src/tenant/mod.rs", "framework/src/tenant/lookup.rs", "framework/src/lib.rs"]
tech_stack:
  added: ["ferro-stripe (optional, stripe feature flag)"]
  patterns: ["feature-gated re-export", "is_some_and delegation pattern", "moka cache invalidation"]
key_files:
  created:
    - framework/src/tenant/requires_plan.rs
  modified:
    - framework/Cargo.toml
    - framework/src/tenant/mod.rs
    - framework/src/tenant/lookup.rs
    - framework/src/tenant/context.rs
    - framework/src/tenant/middleware.rs
    - framework/src/tenant/resolver.rs
    - framework/src/tenant/scope.rs
    - framework/src/lib.rs
decisions:
  - "Clippy is_some_and over map_or(false, ...) for option boolean delegation"
  - "or(self.plan.as_deref()) over or_else closure for current_plan fallback"
  - "All ferro-stripe public types re-exported from framework lib.rs behind stripe feature"
metrics:
  duration_minutes: 10
  tasks_completed: 2
  files_modified: 8
  files_created: 1
  completed_date: "2026-03-11"
---

# Phase 96 Plan 02: TenantContext Subscription Enrichment Summary

Enrich TenantContext with subscription data and add RequiresPlan middleware for plan-based route access control. Connects billing state (from ferro-stripe) to the request pipeline via feature-gated fields and middleware.

## Tasks Completed

| Task | Description | Commit | Status |
|------|-------------|--------|--------|
| 1 | Enrich TenantContext with subscription and add cache invalidation | 816cbb2 | Done |
| 2 | RequiresPlan middleware | 9a8fff7 | Done |

## What Was Built

### Task 1: TenantContext Subscription Enrichment

**`framework/Cargo.toml`**
- Added `ferro-stripe = { path = "../ferro-stripe", version = "0.1", optional = true }` as optional dependency
- Added `stripe = ["dep:ferro-stripe"]` feature

**`framework/src/tenant/mod.rs`**
- Added `#[cfg(feature = "stripe")] pub subscription: Option<ferro_stripe::SubscriptionInfo>` to `TenantContext`
- Added `#[cfg(feature = "stripe")] impl TenantContext` with convenience methods:
  - `on_trial()` — delegates to `subscription.is_some_and(|s| s.on_trial())`
  - `subscribed()` — delegates to `subscription.is_some_and(|s| s.subscribed())`
  - `on_grace_period()` — delegates to `subscription.is_some_and(|s| s.on_grace_period())`
  - `current_plan()` — returns `subscription.plan` or falls back to legacy `plan` field
- Added `#[cfg(feature = "stripe")] mod stripe_tests` with 12 tests covering all helpers and serialization

**`framework/src/tenant/lookup.rs`**
- Added `fn invalidate(&self, _slug: &str, _id: i64) {}` default no-op to `TenantLookup` trait
- Implemented `DbTenantLookup.invalidate()` to call `self.cache.invalidate()` for both slug and id keys
- Added `invalidate_evicts_slug_and_id_cache_entries` test
- Added `default_invalidate_is_noop` test

**Updated all TenantContext construction sites** with `#[cfg(feature = "stripe")] subscription: None`:
- `framework/src/tenant/context.rs`
- `framework/src/tenant/middleware.rs`
- `framework/src/tenant/resolver.rs`
- `framework/src/tenant/scope.rs`
- `framework/src/tenant/lookup.rs`

### Task 2: RequiresPlan Middleware

**`framework/src/tenant/requires_plan.rs`** (new file)
- `pub struct RequiresPlan { required_plan: &'static str }`
- `RequiresPlan::new(plan: &'static str) -> Self`
- Implements `Middleware` trait:
  - No tenant context → 400 JSON error
  - No subscription → 403 JSON error with `required_plan` field
  - Subscription not active (canceled, etc.) → 403
  - Plan doesn't satisfy requirement via `ferro_stripe::plan_satisfies` → 403
  - All checks pass → `next(request).await`
- 7 tests: pro passes, enterprise satisfies pro, free blocked, no subscription, canceled, free passes free, no tenant context

**`framework/src/lib.rs`**
- Added `#[cfg(feature = "stripe")] pub use tenant::RequiresPlan`
- Added `#[cfg(feature = "stripe")] pub use ferro_stripe::{...}` re-exporting all public ferro-stripe types

## Verification Results

```
cargo build -p ferro-rs --all-features    ✓ compiled
cargo build -p ferro-rs                   ✓ compiled (backward compat)
cargo fmt --all -- --check                ✓ clean
cargo clippy -p ferro-rs --all-targets --all-features -- -D warnings  ✓ clean
cargo test -p ferro-rs --all-features -- tenant::   ✓ 60 passed
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Clippy] Replace map_or(false, ...) with is_some_and**
- **Found during:** Task 1 implementation
- **Issue:** Clippy `-D warnings` rejected `map_or(false, |s| s.on_trial())` pattern
- **Fix:** Changed to `is_some_and(|s| s.on_trial())` for all three delegation methods
- **Files modified:** `framework/src/tenant/mod.rs`
- **Commit:** 816cbb2

**2. [Rule 1 - Clippy] Replace or_else closure with or for current_plan**
- **Found during:** Task 1 implementation
- **Issue:** Clippy rejected `or_else(|| self.plan.as_deref())` as unnecessary lazy evaluation
- **Fix:** Changed to `.or(self.plan.as_deref())`
- **Files modified:** `framework/src/tenant/mod.rs`
- **Commit:** 816cbb2

## Decisions Made

- `is_some_and` delegation over `map_or(false, ...)` — Clippy-correct pattern for option boolean delegation
- `or(self.plan.as_deref())` over `or_else` closure — simpler, non-lazy fallback is sufficient
- All ferro-stripe types re-exported from `framework/src/lib.rs` — single import point for framework users

## Self-Check: PASSED

- `framework/src/tenant/requires_plan.rs` — FOUND
- `framework/src/tenant/mod.rs` — FOUND
- `framework/src/tenant/lookup.rs` — FOUND
- `816cbb2` — FOUND in git log
- `9a8fff7` — FOUND in git log
