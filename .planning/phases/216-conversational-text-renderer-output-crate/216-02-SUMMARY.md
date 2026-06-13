---
phase: 216-conversational-text-renderer-output-crate
plan: "02"
subsystem: ferro-text
tags: [new-crate, renderer, text-channel, guard-filter, verbosity, insta-snapshots]
dependency_graph:
  requires: [216-01]
  provides: [TextRenderer, ferro-text crate, process-guard-snapshots]
  affects: [ferro-text, Cargo.toml]
tech_stack:
  added: [ferro-text (new output crate), insta snapshots]
  patterns: [renderer-per-output-crate, guard-filter-absent-key-renders, force_intent-test-helper]
key_files:
  created:
    - ferro-text/Cargo.toml
    - ferro-text/README.md
    - ferro-text/src/lib.rs
    - ferro-text/src/snapshots/ferro_text__tests__process_unfiltered.snap
    - ferro-text/src/snapshots/ferro_text__tests__process_filtered.snap
    - ferro-text/src/snapshots/ferro_text__tests__process_full.snap
    - ferro-text/src/snapshots/ferro_text__tests__process_brief.snap
    - ferro-text/src/snapshots/ferro_text__tests__browse_full.snap
    - ferro-text/src/snapshots/ferro_text__tests__collect_full.snap
    - ferro-text/src/snapshots/ferro_text__tests__summarize_full.snap
    - ferro-text/src/snapshots/ferro_text__tests__track_full.snap
    - ferro-text/src/snapshots/ferro_text__tests__focus_fallback.snap
    - ferro-text/src/snapshots/ferro_text__tests__analyze_fallback.snap
  modified:
    - Cargo.toml (workspace members)
decisions:
  - "TextRenderer dispatches on intent.label() (never format!(\"{:?}\")); Error::NoIntents on empty slice"
  - "force_intent() test helper constructs a single-entry IntentScore slice for Focus/Analyze tests — derive_intents does not score these as primary for the test fixtures used"
  - "image_url_none_hint_labels_not_raw tests render_field_value directly (not via render()) since Browse/Collect don't call render_field_value"
  - "render_focus and render_analyze ignore ctx.verbosity (fallback path; verbosity shapes defined only for the five cleanly-mapping intents)"
  - "approve/reject hidden when is_approver=false; submit/cancel remain — guard-leak-prevention behavior pinned by process_filtered snapshot"
metrics:
  duration: "354s"
  completed: "2026-06-13"
  tasks_completed: 3
  files_modified: 15
requirements: [CHAN-04, CHAN-03]
---

# Phase 216 Plan 02: ferro-text output crate Summary

`ferro-text` crate created with `TextRenderer` implementing `Renderer<Output=String, Context=BaseContext>` — seven per-intent strategies, guard filtering (absent-key-renders semantics), verbosity-aware output, and render_hint application; 13 tests green including both Process guard states snapshotted.

## What Was Built

Created `ferro-text`, the third renderer output crate in the pattern established by `ferro-json-ui` (`JsonUiRenderer`) and `ferro-mcp-server` (`McpRenderer`). `TextRenderer` projects a `ServiceDef` to deterministic conversational plain text:

**Per-intent strategies (5 cleanly-mapping + 2 fallback):**
- `render_browse`: entity name + domain field labels; Brief = primary EntityName only
- `render_collect`: writable domain fields as "Fields to fill in" with required markers; Brief = count
- `render_process`: current state + guard-passing actions only; Brief = headline + action verbs
- `render_summarize`: entity + Money/Percentage/Quantity/Status metric fields; Brief = entity + first metric
- `render_track`: state + terminal detection + next possible states (Full only)
- `render_focus`: field rendering with render_hint applied + limited-modality note (fallback)
- `render_analyze`: entity + domain field names + time-series note, NO fabricated statistics (fallback)

**Guard filtering (D-09, T-216-03):** `action_passes_guards` hides an action if ANY precondition maps to explicit `false`; absent key = render. The two process snapshots (`process_unfiltered`, `process_filtered`) pin this behavior.

**render_hint application (D-12):** `render_field_value` returns `None` for `Skip`, the alt string for `AltText`, and `"<label> (image)"` / `"<label> (link)"` for `ImageUrl`/`Url` with no hint.

## Tasks Completed

| # | Name | Commit | Key Files |
|---|------|--------|-----------|
| 1 | Scaffold ferro-text crate (Cargo.toml + README + skeleton) | a0856d5b | ferro-text/Cargo.toml, ferro-text/README.md, ferro-text/src/lib.rs, Cargo.toml |
| 2 | Implement seven per-intent strategies + guard filter + render_hint | d0e95830 | ferro-text/src/lib.rs |
| 3 | Test suite: anchor fixture + all 10 behaviors + snapshot baselines | 386f8d6f | ferro-text/src/lib.rs, ferro-text/src/snapshots/*.snap (10 files) |

## Verification

- `cargo test -p ferro-text` — 13/13 tests PASS
- `cargo clippy -p ferro-text --all-targets -- -D warnings` — clean
- `cargo doc --no-deps -p ferro-text` — clean
- SC-1: `grep -rn "impl Renderer" ferro-projections/src --include="*.rs" | grep -v "sketch|template"` — no output (TextRenderer is only in ferro-text)
- SC-2: process_unfiltered snap contains all 4 actions; process_filtered snap contains only submit+cancel
- SC-3: render_field_value tests for all three render_hint variants pass; focus_fallback + analyze_fallback snapshots committed
- Snapshot baselines: 10 `.snap` files in `ferro-text/src/snapshots/`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Test imports used private ferro-projections module paths**
- **Found during:** Task 3 (first clippy run)
- **Issue:** Test module used `ferro_projections::action::ActionDef`, `ferro_projections::derive::derive_intents` etc. — these are private modules; the crate only exports via its root re-exports
- **Fix:** Changed all test imports to use `ferro_projections::{derive_intents, ActionDef, DataType, ...}` (crate root)
- **Files modified:** ferro-text/src/lib.rs
- **Commit:** d0e95830

**2. [Rule 1 - Bug] Test 5 (ImageUrl None hint) asserted `(image)` in Browse/Collect output**
- **Found during:** Task 3 (first test run)
- **Issue:** `render_browse` and `render_collect` use `field_display_name()` for labels, not `render_field_value()`. Only `render_focus` and `render_analyze` call `render_field_value`. The test fixture's primary intent was not Focus, so `(image)` never appeared.
- **Fix:** Changed test 5 to test `render_field_value` directly (same pattern as tests 6 and 7 already used), which is the correct unit for this assertion
- **Files modified:** ferro-text/src/lib.rs
- **Commit:** 386f8d6f

**3. [Rule 1 - Bug] Tests 8/9 (focus/analyze fallback) fell to unwrap_or(0) intent index**
- **Found during:** Task 3 (first test run)
- **Issue:** `derive_intents` did not score `focus` or `analyze` as primary for the test fixtures (Profile with ImageUrl/Url scores as Collect; Sales Report with Money+Quantity scores as Summarize). `unwrap_or(0)` returned the wrong intent, so `render_analyze` was never called for the analyze test — the snapshot showed Summarize output instead.
- **Fix:** Added `force_intent(Intent)` helper that constructs a single-entry `IntentScore` vec; tests 8 and 9 now use it to force the desired intent without depending on `derive_intents` scoring
- **Files modified:** ferro-text/src/lib.rs
- **Commit:** 386f8d6f

## Known Stubs

None — all seven per-intent strategies are implemented and tested. The fallback note text for Focus and Analyze is intentionally minimal (D-13: "defined fallback", not full rendering).

## Threat Flags

None — no new network endpoints, auth paths, or trust-boundary crossings. The guard-filtering behavior (T-216-03) is verified by the `process_filtered` snapshot.

## Self-Check: PASSED

- `ferro-text/Cargo.toml` — FOUND
- `ferro-text/src/lib.rs` — FOUND (contains TextRenderer, impl Renderer, all 7 render_* fns, action_passes_guards, render_field_value, 13 tests)
- `ferro-text/src/snapshots/` — FOUND (10 .snap files)
- Commit a0856d5b — FOUND
- Commit d0e95830 — FOUND
- Commit 386f8d6f — FOUND
- `cargo test -p ferro-text` — 13/13 PASS
- `cargo clippy -p ferro-text --all-targets -- -D warnings` — clean
- SC-1 verified — no impl Renderer in ferro-projections (outside pre-existing sketch+template)
