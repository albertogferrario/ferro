---
phase: 207-comp-02-synthetic-regression-catalog
plan: "01"
subsystem: ferro-projections
tags: [testing, regression-catalog, derive_intents, proptest, insta, COMP-02]
dependency_graph:
  requires: []
  provides:
    - ferro-projections/tests/catalog.rs (canonical fixtures + all test types)
    - ferro-projections/tests/snapshots/catalog__canonical_*.snap (7 committed baselines)
  affects:
    - ferro-projections (dev-dependencies added)
tech_stack:
  added:
    - insta = { version = "1", features = ["yaml"] }
    - proptest = "1"
  patterns:
    - calibrated-floor/margin confidence assertions (D-07)
    - IntentSignals redacted snapshot struct (D-04)
    - arb_service_def proptest Strategy over ServiceDef builder
key_files:
  created:
    - ferro-projections/tests/catalog.rs
    - ferro-projections/tests/snapshots/catalog__canonical_browse.snap
    - ferro-projections/tests/snapshots/catalog__canonical_focus.snap
    - ferro-projections/tests/snapshots/catalog__canonical_collect.snap
    - ferro-projections/tests/snapshots/catalog__canonical_process.snap
    - ferro-projections/tests/snapshots/catalog__canonical_summarize.snap
    - ferro-projections/tests/snapshots/catalog__canonical_analyze.snap
    - ferro-projections/tests/snapshots/catalog__canonical_track.snap
  modified:
    - ferro-projections/Cargo.toml
decisions:
  - "analyze_timeseries fixture uses 50% writable ratio (2 writable + 2 read_only DateTime/Money) to avoid Collect signal trap while keeping Analyze (0.35) above Summarize (0.30)"
  - "All primary confidences normalize to 1.0; floor=0.85 for all 7; ANALYZE_MARGIN=0.04 is the binding constraint"
  - "IntentSignals redacted struct (no confidence field) prevents snapshot thrash on derive.rs re-tuning"
metrics:
  duration: "566s"
  completed: "2026-06-12"
  tasks_completed: 3
  files_created: 9
  files_modified: 1
---

# Phase 207 Plan 01: Synthetic Regression Catalog — COMP-02 Summary

Permanent regression catalog for `derive_intents()` in `ferro-projections/tests/catalog.rs`. Seven canonical `ServiceDef` fixtures (one per structural intent), calibrated confidence assertions, structural invariants, 4 adversarial competing-signal fixtures, 256-case proptest suite, 7 committed `insta` YAML snapshots, and a discovered-weaknesses note.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Scaffold 7 fixtures + structural-invariant tests | cd1f8c74 | Cargo.toml, catalog.rs |
| 2 | Calibrate confidence floors/margins (D-07) | 52b2fe67 | catalog.rs |
| 3 | Adversarial tests, proptest, snapshots, weakness note | b57578b4 | catalog.rs, 7 .snap files |

## What Was Built

**`ferro-projections/tests/catalog.rs`** (923 lines):

- `mod fixtures` — 7 named `ServiceDef` builder functions: `browse_catalog`, `focus_detail`, `collect_form`, `process_workflow`, `summarize_dashboard`, `analyze_timeseries`, `track_timeline`
- 7 `canonical_*` tests — each asserts: primary-intent identity, calibrated confidence floor (0.85), calibrated margin, and ≥1 structural invariant on the fixture shape
- 4 `adversarial_*` tests — confusable pairs with inline `// competing:` rationale comments
- `engine_never_panics_returns_valid_scores` proptest — 256 cases, 4 invariants (non-empty, confidence ∈ [0,1], sorted descending, no duplicate Intent)
- 7 `snapshot_canonical_*` tests — `IntentSignals` struct redacts confidence floats; snapshots capture ranked `(intent, signals)` only
- 59 structural/prop assertions vs 7 snapshot assertions (SC#2 ratio: 8.4×)

**`ferro-projections/tests/snapshots/`** — 7 committed `.snap` files; each contains intent names and signal strings only, no floats.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed analyze_timeseries fixture: Collect won instead of Analyze**
- **Found during:** Task 1 first test run
- **Issue:** Research assumption A1 was wrong about the fixture. The 3 writable DateTime fields produced a 75% writable ratio (>50%), triggering the `high_writable_ratio` Collect signal (+0.35). Collect (0.35) beat Analyze (0.35) by tie-break priority.
- **Fix:** Changed `analyze_timeseries` to use 2 writable + 1 read_only DateTime + 1 read_only Money: exactly 50% writable ratio (not >50%) prevents the Collect signal; non_writable_ratio=50% (not >70%) prevents `mostly_read_only` Summarize boost. Analyze raw (0.35) > Summarize raw (0.30) — Analyze wins.
- **Files modified:** `ferro-projections/tests/catalog.rs` (fixture only, never `derive.rs`)
- **Commit:** cd1f8c74

**2. [Rule 1 - Bug] Fixed clippy uninlined_format_args violations**
- **Found during:** Task 1 clippy gate
- **Issue:** 11 `assert!` calls used `assert!(cond, "msg {}", var)` instead of `assert!(cond, "msg {var}")` — rejected by `-D warnings`
- **Fix:** Rewrote all assert format strings to use inline captured variable names
- **Files modified:** `ferro-projections/tests/catalog.rs`
- **Commit:** cd1f8c74

## Calibration Results (D-07)

Observed from first real run (all primaries normalize to 1.0):

| Intent | Runner-up | Gap | FLOOR | MARGIN |
|--------|-----------|-----|-------|--------|
| Browse | 0.2414 | 0.7586 | 0.85 | 0.66 |
| Focus | 0.2593 | 0.7407 | 0.85 | 0.64 |
| Collect | 0.4000 | 0.6000 | 0.85 | 0.50 |
| Process | 0.1842 | 0.8158 | 0.85 | 0.72 |
| Summarize | 0.1000 | 0.9000 | 0.85 | 0.80 |
| Analyze | 0.8571 | 0.1429 | 0.85 | **0.04** |
| Track | 0.4667 | 0.5333 | 0.85 | 0.43 |

Analyze has the tightest margin (0.04). See Discovered weaknesses note in catalog.rs module doc.

## Discovered Weaknesses (SC#5)

**Analyze↔Summarize margin is structurally thin.** The `datetime_numeric_cooccurrence` signal contributes a flat 0.35 raw weight regardless of how many DateTime fields are present. With 1 Money field, Analyze (0.35) beats Summarize (0.30) by only 0.1429 normalized. Adding a second Money field immediately flips the winner to Summarize. The calibrated `ANALYZE_MARGIN` is 0.04 — any `derive.rs` change that raises the Summarize Money weight even slightly would break this test. This is a genuine derivation limitation: the engine cannot distinguish "time-series with one KPI" from "dashboard with one KPI and a date" at the signal level.

## Self-Check: PASSED

- [x] `ferro-projections/tests/catalog.rs` — FOUND (923 lines)
- [x] `ferro-projections/tests/snapshots/catalog__canonical_browse.snap` — FOUND
- [x] `ferro-projections/tests/snapshots/catalog__canonical_focus.snap` — FOUND
- [x] `ferro-projections/tests/snapshots/catalog__canonical_collect.snap` — FOUND
- [x] `ferro-projections/tests/snapshots/catalog__canonical_process.snap` — FOUND
- [x] `ferro-projections/tests/snapshots/catalog__canonical_summarize.snap` — FOUND
- [x] `ferro-projections/tests/snapshots/catalog__canonical_analyze.snap` — FOUND
- [x] `ferro-projections/tests/snapshots/catalog__canonical_track.snap` — FOUND
- [x] Commit cd1f8c74 — FOUND
- [x] Commit 52b2fe67 — FOUND
- [x] Commit b57578b4 — FOUND
- [x] `cargo test -p ferro-projections --test catalog` — 19/19 PASS
- [x] `git diff --stat ferro-projections/src/intent.rs ferro-projections/src/derive.rs` — empty (no changes)
- [x] `grep -c '#\[ignore\]' catalog.rs` — 0
- [x] SC#2: 59 structural/prop assertions > 7 snapshot assertions
