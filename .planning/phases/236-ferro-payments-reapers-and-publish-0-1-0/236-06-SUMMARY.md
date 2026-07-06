---
phase: 236
plan: 06
subsystem: docs
tags: [docs, ferro-payments, mdbook, recovery-model]
dependency_graph:
  requires: [236-04]
  provides: [consumer-facing payments documentation, PAY-POLY-REAP-04]
  affects: [docs/src/features/payments.md, docs/src/SUMMARY.md, docs/src/features/stripe.md]
tech_stack:
  added: []
  patterns: [mdBook prose, rustdoc -D warnings gate]
key_files:
  created:
    - docs/src/features/payments.md
  modified:
    - docs/src/SUMMARY.md
    - docs/src/features/stripe.md
decisions:
  - "Code blocks in payments.md use ```rust,ignore — not doctests; they are mdBook prose that cannot compile under cargo test (no app context), so ,ignore is the correct annotation"
  - "Cross-link placed at the top of stripe.md as a blockquote note — discoverable without requiring readers to scroll to end"
metrics:
  duration_minutes: 4
  completed: "2026-06-21"
  tasks_completed: 1
  files_changed: 3
---

# Phase 236 Plan 06: Payments Documentation Summary

**One-liner:** Consumer-facing payments page documenting the three-step Quick Start (register migration, wire_dispatcher, schedule two reapers) and the self-healing recovery model with explicit double-refund guard.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Write payments.md, wire SUMMARY.md, cross-link stripe.md | f94558e0 | docs/src/features/payments.md (created), docs/src/SUMMARY.md, docs/src/features/stripe.md |

## Verification

All acceptance criteria passed:

- `docs/src/features/payments.md` exists (194 lines, > 50 minimum)
- `grep -q "wire_dispatcher" docs/src/features/payments.md` — PASS
- `grep -q "ReleaseExpiredPaymentIntents" docs/src/features/payments.md` — PASS
- `grep -q "ReconcileRefundsInFlight" docs/src/features/payments.md` — PASS
- `grep -qi "idempotent" docs/src/features/payments.md` — PASS (double-refund guard stated explicitly)
- `grep -q "features/payments.md" docs/src/SUMMARY.md` — PASS
- `grep -q "payments.md" docs/src/features/stripe.md` — PASS
- `RUSTDOCFLAGS="-D warnings" cargo doc -p ferro-payments --no-deps` — exit 0, no warnings
- `cargo clippy --all --all-targets -- -D warnings` — exit 0

## Decisions Made

- Code blocks annotated `rust,ignore` throughout: these are illustrative consumer snippets for an mdBook page; they reference consumer types (`MyBillableLoader`, `MyOrder`) that cannot compile under `cargo test` in the crate tree.
- Cross-link in stripe.md placed immediately after the opening bullet list as a blockquote, so readers encounter it before the Quick Start section.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. The documentation page references the real public API (`wire_dispatcher`, `ReleaseExpiredPaymentIntents::new`, `ReconcileRefundsInFlight::new`, `PaymentService::new`, `CreatePaymentIntentsTable`) as they exist in the committed crate source.

## Threat Flags

None. This plan touches only mdBook prose files under `docs/src/`; no new network endpoints, auth paths, or schema changes introduced.

## Self-Check: PASSED

- `docs/src/features/payments.md` — FOUND
- `docs/src/SUMMARY.md` — FOUND
- `docs/src/features/stripe.md` — FOUND
- Commit `f94558e0` — FOUND in git log
