---
phase: 140
plan: 01
subsystem: ferro-stripe
tags: [stripe, foundation, client, error, dashmap]
requires: []
provides: [dashmap-dep, MissingIdempotencyKey, Stripe::with]
affects: [ferro-stripe/Cargo.toml, ferro-stripe/src/error.rs, ferro-stripe/src/client.rs]
tech_stack:
  added: [dashmap = "6"]
  patterns: [thiserror variant extension, OnceLock scoped bypass, TDD red-green]
key_files:
  modified:
    - ferro-stripe/Cargo.toml
    - ferro-stripe/src/error.rs
    - ferro-stripe/src/client.rs
decisions:
  - dashmap = "6" inserted alphabetically between chrono and async-trait in [dependencies]
  - MissingIdempotencyKey is a unit variant (no payload); error message uses imperative fix instruction
  - Stripe::with placed after config() in impl block; doc comment makes per-tenant use case explicit
  - TDD cycle followed: RED commit (bde63492) then GREEN commit (169dfadc)
metrics:
  duration: ~8min
  completed: 2026-04-20
  tasks: 2
  files: 3
---

# Phase 140 Plan 01: Foundation (dashmap + MissingIdempotencyKey + Stripe::with) Summary

Additive foundation plan landing three prerequisites that unblock parallel execution of plans 02 and 03: `dashmap = "6"` dependency, `Error::MissingIdempotencyKey` variant, and `Stripe::with(api_key)` scoped-client constructor.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add dashmap dep and MissingIdempotencyKey variant | ba34c8f4 | ferro-stripe/Cargo.toml, ferro-stripe/src/error.rs |
| 2 (RED) | Failing tests for Stripe::with | bde63492 | ferro-stripe/src/client.rs |
| 2 (GREEN) | Implement Stripe::with | 169dfadc | ferro-stripe/src/client.rs |

## Diff Summary

### ferro-stripe/Cargo.toml
```toml
+ dashmap = "6"
```
Inserted between `chrono` and `async-trait` in `[dependencies]` (alphabetical order).

### ferro-stripe/src/error.rs
```rust
+     /// Idempotency key not set on CheckoutBuilder before calling create().
+     #[error("idempotency key required: call .idempotency_key() before .create()")]
+     MissingIdempotencyKey,
```
Added after `EventAlreadyProcessed` variant. All five existing variants unchanged.

### ferro-stripe/src/client.rs
```rust
+     /// Returns a scoped Stripe client for the given API key.
+     ///
+     /// Use for per-tenant direct-charges scenarios where a different
+     /// Stripe account key is needed per request.
+     /// Does not affect the global static client initialized by [`Stripe::init`].
+     pub fn with(api_key: &str) -> stripe::Client {
+         stripe::Client::new(api_key)
+     }
```
Added after `config()` inside `impl Stripe`. Two new tests in `#[cfg(test)] mod tests`:
- `with_does_not_populate_global_static` — asserts `STRIPE_CLIENT.get().is_none()` after calling `Stripe::with`
- `with_returns_independent_client_values` — asserts two independent `stripe::Client` values are returned by value

## Verification Results

```
cargo check -p ferro-stripe         → Finished (exit 0)
cargo test -p ferro-stripe --lib    → 36 passed, 0 failed
cargo fmt -p ferro-stripe -- --check → clean (exit 0)
cargo clippy -p ferro-stripe --all-targets -- -D warnings → clean (exit 0)
```

## Deviations from Plan

None. Plan executed exactly as written.

The acceptance criterion `grep -n 'MissingIdempotencyKey' ferro-stripe/src/error.rs returns 2 lines` appears to contain a documentation error — the `#[error(...)]` attribute line does not contain the text `MissingIdempotencyKey`, so the grep returns 1 line. The substance is correct: both the `#[error]` attribute and the variant name are present.

## TDD Gate Compliance

- RED gate: commit `bde63492` — `test(140-01): add failing tests for Stripe::with scoped client`
- GREEN gate: commit `169dfadc` — `feat(140-01): implement Stripe::with scoped client constructor`
- REFACTOR gate: not needed (implementation is minimal and clean)

## Self-Check: PASSED

- ferro-stripe/Cargo.toml: contains `dashmap = "6"` ✓
- ferro-stripe/src/error.rs: contains `MissingIdempotencyKey` ✓
- ferro-stripe/src/client.rs: contains `pub fn with` and `stripe::Client::new(api_key)` ✓
- Commits ba34c8f4, bde63492, 169dfadc all present in git log ✓
