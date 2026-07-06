---
phase: 215-non-visual-rendering-context-basecontext-intent-extensions
plan: "01"
subsystem: ferro-projections
tags: [rendering, context, intent, error, non-visual]
requirements: [CHAN-01, CHAN-02]

dependency_graph:
  requires: []
  provides:
    - BaseContext.evaluated_guards (HashMap<String, bool>)
    - BaseContext.verbosity (Verbosity enum)
    - Verbosity enum (Full/Brief, no serde)
    - Intent::label() -> &str
    - Error::NoIntents variant
  affects:
    - ferro-json-ui (Plan 02 will embed BaseContext in VisualContext)
    - ferro-mcp (Plan 02 will migrate {:?} label sites to .label())

tech_stack:
  added: []
  patterns:
    - "#[derive(Default)] with #[default] on enum variant (Verbosity::Full)"
    - "HashMap<String, bool> for guard evaluation map (std, no new dep)"
    - "thiserror unit variant with #[error(...)] message (NoIntents)"

key_files:
  created: []
  modified:
    - ferro-projections/src/render/mod.rs
    - ferro-projections/src/intent.rs
    - ferro-projections/src/error.rs

decisions:
  - "Verbosity carries no serde (BaseContext has no serde — kept consistent, D-05)"
  - "Error::NoIntents is a unit variant; not wired into visual render path (D-09)"
  - "Intent::label() returns &str (not &'static str) so Custom(s) arm can borrow from self"
  - "evaluated_guards absent key = render action (D-04)"

metrics:
  duration: "200s"
  completed: "2026-06-13"
  tasks_completed: 2
  files_modified: 3
---

# Phase 215 Plan 01: BaseContext + Intent extensions Summary

Extends `ferro-projections` with the three type-surface additions required for a
future non-visual renderer (Phase 216), satisfying CHAN-01 and CHAN-02 at the
crate boundary without adding any renderer to `ferro-projections`.

## What Was Built

### Task 1 — BaseContext extended, Verbosity enum added

**File:** `ferro-projections/src/render/mod.rs`

Added `Verbosity { Full, Brief }` enum:
- Derives `Debug, Clone, Copy, PartialEq, Eq, Default` — no serde (consistent with `BaseContext`)
- `#[default]` on `Full` — backward-compatible with all existing visual rendering

Added two fields to `BaseContext`:
- `evaluated_guards: HashMap<String, bool>` — guard-name → evaluation result; absent key = render the action (D-04); default = empty map = render everything
- `verbosity: Verbosity` — defaults to `Full` via `Verbosity`'s `#[default]`; `BaseContext::default()` is therefore fully backward-compatible

`use std::collections::HashMap` added at top of file (stdlib, no new Cargo dep).

Unit tests extended/added:
- `base_context_default` — extended with two assertions (`evaluated_guards.is_empty()`, `verbosity == Full`)
- `verbosity_default_is_full` — new test

**Commit:** `6ae3a8e5`

### Task 2 — Intent::label() and Error::NoIntents

**File:** `ferro-projections/src/intent.rs`

Added `impl Intent { pub fn label(&self) -> &str }`:
- Returns stable snake_case strings for all 7 known variants: "browse", "focus", "collect", "process", "summarize", "analyze", "track"
- `Custom(s)` arm returns `s.as_str()` (borrows from self)
- Return type is `&str` (not `&'static str`) to unify known-variant `'static` literals with the `Custom` borrow lifetime
- Seven-intent vocabulary (Browse/Focus/Collect/Process/Summarize/Analyze/Track + Custom) unchanged

**File:** `ferro-projections/src/error.rs`

Added `Error::NoIntents` unit variant:
- Message: "cannot render service with no intents"
- Not wired into the visual render path — that path keeps `ProjectionError::EmptyIntents` (D-09)
- Test module created in error.rs with `no_intents_error_message` test

Unit tests added:
- `intent_label_known_variants` — all 7 known variants checked
- `intent_label_custom_returns_inner_string` — `Custom("reporting")` returns "reporting"
- `no_intents_error_message` — `Error::NoIntents.to_string()` matches expected message

**Commit:** `4b543669`

## Verification Results

- `cargo test -p ferro-projections`: 241 unit + 22 catalog + 1 schema + 8 doc = **272 tests, all green**
- No serde added to `Verbosity`: `grep -c "Serialize" ferro-projections/src/render/mod.rs` = 0
- Seven-intent vocabulary count: 7 (unchanged)
- No new `[dependencies]` in `ferro-projections/Cargo.toml` (`HashMap` is stdlib)
- `cargo clippy -p ferro-projections --all-targets -- -D warnings`: clean
- `cargo fmt --all -- --check`: clean (pre-existing fmt drift in `ferro-queue` fixed in a separate `style` commit)

## Note for Plan 02

`BaseContext` now has two extra fields (`evaluated_guards`, `verbosity`). Every
`VisualContext` struct-literal site that constructs `BaseContext` fields inline needs
`base: BaseContext { ..Default::default() }` wrapping or a `..Default::default()`
fill-in. The compiler will catch all missed sites with "no field `intent_index` on type
`VisualContext`" after D-02 embedding is applied.

The three `ferro-mcp` call sites using `format!("{:?}", intent)` for labels
(`render_projection.rs:94/:102`, `generate_projection.rs:89`,
`projection_coverage.rs:173`) are ready to migrate to `.label()` in Plan 02.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Pre-existing rustfmt drift in ferro-queue**
- **Found during:** Task 2 pre-commit gate (`cargo fmt --all -- --check`)
- **Issue:** `ferro-queue/src/db.rs` and `ferro-queue/src/worker.rs` had pre-existing
  formatting drift (long function call chains not split per rustfmt style). Unrelated
  to this plan's changes but blocked the fmt gate.
- **Fix:** Applied `cargo fmt --all` and committed the ferro-queue changes as a separate
  `style` commit before the Task 2 feat commit.
- **Files modified:** `ferro-queue/src/db.rs`, `ferro-queue/src/worker.rs`
- **Commit:** `a24cb153`

## Known Stubs

None — no stubs introduced. All added types are fully implemented with unit tests.

## Threat Flags

None — no new trust boundaries introduced. `evaluated_guards` is populated by
trusted in-process callers, not by request data (T-215-01 accepted per threat model).
`Intent::label()` exposes the same snake_case strings already produced by serde
(T-215-02 accepted).

## Self-Check: PASSED

- `ferro-projections/src/render/mod.rs` — FOUND
- `ferro-projections/src/intent.rs` — FOUND
- `ferro-projections/src/error.rs` — FOUND
- Commit `6ae3a8e5` (Task 1) — FOUND
- Commit `4b543669` (Task 2) — FOUND
