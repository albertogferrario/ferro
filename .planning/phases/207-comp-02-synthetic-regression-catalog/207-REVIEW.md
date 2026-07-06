---
phase: 207-comp-02-synthetic-regression-catalog
reviewed: 2026-06-12T00:00:00Z
depth: standard
files_reviewed: 2
files_reviewed_list:
  - ferro-projections/tests/catalog.rs
  - ferro-projections/Cargo.toml
findings:
  critical: 0
  warning: 1
  info: 3
  total: 4
status: issues_found
---

# Phase 207: Code Review Report

**Reviewed:** 2026-06-12T00:00:00Z
**Depth:** standard
**Files Reviewed:** 2
**Status:** issues_found

## Summary

Reviewed the new synthetic regression catalog (`catalog.rs`) and the `[dev-dependencies]`
addition to `ferro-projections/Cargo.toml`. The catalog is a high-quality regression
oracle: seven canonical fixtures each carry non-trivial structural content, every
fixture's primary-intent assertion is backed by independent structural invariants (field
counts, relationship cardinality, state-machine shape, signal presence), and the four
adversarial fixtures resolve genuine signal competitions rather than degenerate inputs.
The proptest invariants (non-empty, bounded confidence, sorted, duplicate-free) are sound
and exercise meaningful field variation. Signal-name strings (`datetime_numeric_cooccurrence`,
`linear_states`) and the `ServiceDef`/`IntentScore` API used were cross-checked against
`derive.rs` / `intent.rs` and match. The redacted snapshots correctly omit confidence
floats per D-04.

No fixture can pass on an empty/minimal `ServiceDef` (SC#2 holds): every canonical test
pairs the primary-intent equality check with structural-count asserts that a minimal
`ServiceDef` would fail. No critical issues found.

The one warning concerns a `validate()` assertion that is weaker than its message implies.
Info items cover proptest coverage gaps and minor maintainability notes.

## Warnings

### WR-01: `validate().is_ok()` does not assert structural cleanliness it implies

**File:** `ferro-projections/tests/catalog.rs:396-400` (and `:566-570`)
**Issue:** `svc.validate()` returns `Result<Vec<Warning>, Error>` (confirmed in
`service.rs:336`). The assertion `assert!(svc.validate().is_ok(), "process fixture must be
structurally valid: ...")` only verifies that no hard `Error` was returned — it accepts an
`Ok(vec![...])` carrying any number of warnings (e.g. `validate_warns_unused_guards`,
`validate_warns_transition_trigger_without_state_machine`). The message "must be
structurally valid" overstates what is checked; a fixture that accumulated structural
warnings (unused guard, dangling trigger) would still pass. For a regression oracle whose
job is to pin fixture shape, the warning vector is exactly the signal you want to assert
on.
**Fix:** Assert the warning vector is empty so the catalog catches drift that introduces
warnings:
```rust
let warnings = svc.validate().expect("process fixture must be structurally valid");
assert!(
    warnings.is_empty(),
    "process fixture produced structural warnings: {warnings:?}"
);
```
Apply the same to the `canonical_track` fixture at `:566`.

## Info

### IN-01: proptest never exercises read-only / write-only fields or relationships/state machines

**File:** `ferro-projections/tests/catalog.rs:872-882`
**Issue:** `arb_service_def` only generates writable plain `field()` entries (0..8). The
writability signals (`high_writable_ratio`, `mostly_read_only`), relationship signals
(`has_many_relationships`), and all state-machine signals (`linear_states`,
`guarded_transitions`) are therefore never driven by the property test. The four
robustness invariants (total/bounded/sorted/unique) consequently hold over only a slice of
the input space — the branches most likely to produce edge-case scores (all-read-only,
guarded branching) go unexercised by proptest, even though the canonical fixtures cover
them deterministically. This is a coverage gap, not a correctness defect.
**Fix:** Extend the strategy with a `prop_oneof!` over writability per field, and
optionally generate an `Option<StateMachine>`, so the engine invariants are asserted across
the writability and state-machine code paths too.

### IN-02: `ANALYZE_MARGIN = 0.04` is brittle and the file documents exactly why

**File:** `ferro-projections/tests/catalog.rs:60-61` (weakness note `:8-19`)
**Issue:** The Analyze margin is a 0.04 cushion over an observed 0.1429 gap — the tightest
in the catalog. The header note already explains that a single extra Money/Percentage field
or a >=0.31/field Summarize weight flips the winner. This is acknowledged and intentional,
so it is recorded as Info, not Warning. The risk is that a benign `derive.rs` tuning change
trips `canonical_analyze` with a margin failure that reads as a regression when it is
actually expected calibration drift.
**Fix:** No code change required; the inline documentation is the right mitigation. If this
proves flaky in CI, consider recalibrating from a re-observed run rather than loosening the
margin toward zero, and keep the weakness note in sync with the constant.

### IN-03: Margin asserts are skipped when `scores.len() == 1`

**File:** `ferro-projections/tests/catalog.rs:252`, `:302`, `:341`, `:412`, `:471`, `:530`, `:582`
**Issue:** Each canonical test guards the margin assertion with `if scores.len() > 1`. Per
the recorded observations every canonical fixture currently returns multiple ranked scores,
so the guard never short-circuits today. But if a future `derive.rs` change collapsed the
output to a single score, the margin assertion would silently be skipped rather than fail —
the test would still pass while the catalog stopped enforcing its separation guarantee.
**Fix:** Optionally assert the precondition explicitly (e.g. `assert!(scores.len() > 1,
"expected runner-up for margin check")`) for the canonical fixtures, which by construction
always produce competing intents. The proptest correctly keeps the non-empty-but-possibly-
single case general; this note applies only to the calibrated canonical tests.

---

_Reviewed: 2026-06-12T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
