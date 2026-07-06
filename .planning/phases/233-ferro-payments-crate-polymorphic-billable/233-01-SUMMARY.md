---
phase: 233-ferro-payments-crate-polymorphic-billable
plan: "01"
subsystem: ferro-payments
tags: [crate-scaffold, sea-orm, payments, billing, workspace]
dependency_graph:
  requires: [ferro-orm]
  provides: [ferro-payments crate skeleton, PaymentIntentStatus, PaymentError, BillableKind]
  affects: [Cargo.toml (workspace), .github/workflows/publish.yml (Wave 1b)]
tech_stack:
  added: [ferro-payments, sea-orm-migration, thiserror, DeriveActiveEnum]
  patterns: [DeriveActiveEnum TEXT-backed enum, open-set newtype, thiserror error enum]
key_files:
  created:
    - ferro-payments/Cargo.toml
    - ferro-payments/README.md
    - ferro-payments/src/lib.rs
    - ferro-payments/src/error.rs
    - ferro-payments/src/intent/mod.rs
    - ferro-payments/src/intent/status.rs
  modified:
    - Cargo.toml
    - .github/workflows/publish.yml
decisions:
  - "version = \"0.1.0\" explicit (not version.workspace) per D-14"
  - "No ferro-stripe dependency per D-12 (avoids unused-dep clippy failure)"
  - "PaymentError ships only Db/StatusPrecondition/NotFound per D-13"
  - "ferro-payments placed in publish.yml Wave 1b (after ferro-orm in Wave 1a)"
  - "BillableKind newtype in lib.rs rather than a separate billable.rs stub (Claude's Discretion)"
  - "mod error declared before pub mod intent in lib.rs to satisfy rustfmt ordering"
metrics:
  duration_minutes: 2
  completed_date: "2026-06-17"
  tasks_completed: 2
  tasks_total: 2
  files_created: 6
  files_modified: 2
---

# Phase 233 Plan 01: ferro-payments Crate Scaffold Summary

**One-liner:** New `ferro-payments` workspace crate compiling with `PaymentIntentStatus` DeriveActiveEnum (TEXT, 5 variants), minimal `PaymentError` (Db/StatusPrecondition/NotFound), and `BillableKind` open-set newtype — registered in Cargo.toml members and publish.yml Wave 1b.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create crate manifest, register in workspace + publish pipeline | 44e6d232 | ferro-payments/Cargo.toml, ferro-payments/README.md, Cargo.toml, .github/workflows/publish.yml |
| 2 | Crate source skeleton — lib.rs, error.rs, intent/mod.rs, status enum | caaee82c | ferro-payments/src/lib.rs, ferro-payments/src/error.rs, ferro-payments/src/intent/mod.rs, ferro-payments/src/intent/status.rs, Cargo.lock |

## Verification Results

- `cargo build -p ferro-payments`: exit 0
- `cargo clippy -p ferro-payments --all-targets -- -D warnings`: exit 0 (no warnings)
- `cargo test -p ferro-payments status`: 1 passed, 0 failed (`status_string_values_round_trip`)
- `grep '"ferro-payments"' Cargo.toml`: matches (workspace member after ferro-reservation)
- `grep 'ferro-reservation ferro-payments' .github/workflows/publish.yml`: matches (Wave 1b)

## Decisions Made

- `version = "0.1.0"` is explicit, not `version.workspace` — D-14 is explicit on this.
- No `ferro-stripe` dependency — D-12; an unused dep would fail `clippy -D warnings`.
- `PaymentError` ships exactly three variants (Db, StatusPrecondition, NotFound) — D-13; `Stripe`/`Loader`/`AutoRefundTriggered` deferred to Phase 234.
- `ferro-payments` placed in `WAVE1B_CRATES` in publish.yml because it depends on `ferro-orm` (Wave 1a).
- `BillableKind` newtype defined inline in `lib.rs` rather than a separate `billable.rs` — Claude's Discretion; simpler, no separate stub needed at this wave.
- `mod error` declared before `pub mod intent` in `lib.rs` — rustfmt requires private modules before public modules.
- `intent/mod.rs` declares only `pub mod status;` — `entity` and `lifecycle` are added by Plans 02/03.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] rustfmt module ordering in lib.rs**
- **Found during:** Task 2, pre-commit `cargo fmt --check`
- **Issue:** `pub mod intent;` appeared before `mod error;`; rustfmt enforces private-before-public ordering.
- **Fix:** Swapped to `mod error; pub mod intent;`.
- **Files modified:** ferro-payments/src/lib.rs
- **Commit:** caaee82c (included in Task 2 commit after fix)

## Known Stubs

None — this plan is a compile-only skeleton. `intent/mod.rs` intentionally declares only `status` (no entity/lifecycle yet); this is documented in the file and tracked as out-of-scope for this plan.

## Threat Flags

None — pure crate scaffold, no network surface, no credentials, no user input.

## Self-Check

- ferro-payments/Cargo.toml: FOUND
- ferro-payments/README.md: FOUND
- ferro-payments/src/lib.rs: FOUND
- ferro-payments/src/error.rs: FOUND
- ferro-payments/src/intent/mod.rs: FOUND
- ferro-payments/src/intent/status.rs: FOUND
- Commit 44e6d232: FOUND
- Commit caaee82c: FOUND

## Self-Check: PASSED
