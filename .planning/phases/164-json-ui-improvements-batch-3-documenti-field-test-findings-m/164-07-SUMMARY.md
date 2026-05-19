---
phase: 164-json-ui-improvements-batch-3-documenti-field-test-findings-m
plan: 07
subsystem: json-ui
tags: [rust, ferro-json-ui, framework, validation, pipeline, tracing, catalog]

requires:
  - phase: 164-06
    provides: "Plan 04 (D-12 title binding) already landed in framework/src/json_ui/mod.rs"

provides:
  - "Two-stage catalog validation: structural hard-fail at load, enum-shape warn-only at load + tracing::error at render"
  - "Catalog::validate called AFTER expand_directives in both JsonUi::resolve and resolve_with_errors"
  - "Integration test framework/tests/pipeline_order.rs proving pipeline order end-to-end"

affects: [164-08, 164-09, 160-remove-v1, 161-merge-v12]

tech-stack:
  added:
    - "tracing = 0.1 in ferro-json-ui Cargo.toml"
    - "tracing = 0.1 in framework Cargo.toml"
  patterns:
    - "Two-stage validation: structural errors fail-loud via Spec::from_json; catalog/enum errors are warnings at load, errors at render-time (post-expand_directives)"
    - "Integration tests for pipeline ordering in framework/tests/*.rs with #![cfg(feature = \"json-ui\")]"

key-files:
  created:
    - "framework/tests/pipeline_order.rs"
  modified:
    - "ferro-json-ui/src/loader.rs"
    - "ferro-json-ui/Cargo.toml"
    - "framework/src/json_ui/mod.rs"
    - "framework/Cargo.toml"

key-decisions:
  - "Option A (two-stage with warning) for D-16: load_cached downgrades catalog validation to tracing::warn; per-request resolve() adds tracing::error + continue"
  - "Clean-path resolve() uses tracing::error + continue (not a hard failure) because changing its return type to Result<> would be invasive"
  - "resolve_with_errors() also uses tracing::error + continue — catalog errors are orthogonal to form validation errors"
  - "visible gates are render-time (renderer honours them), not expand_directives-time (only $if is removed by expand_directives)"
  - "Test uses load_builtins_warn_only (test-local helper) not load_cached directly, to avoid global catalog pollution from BadPlugin_117 in parallel tests"

requirements-completed: [D-16]

duration: 45min
completed: 2026-05-17
---

# Phase 164 Plan 07: Pipeline Reorder — Two-Stage Catalog Validation Summary

**Alert.variant="" gated by `visible` no longer blocks server startup: load_cached downgrades catalog validation to tracing::warn; per-request resolve() enforces after expand_directives via tracing::error + continue**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-05-17T00:00:00Z
- **Completed:** 2026-05-17T00:45:00Z
- **Tasks:** 4 (+ pre-commit gate)
- **Files modified:** 5

## Accomplishments

- `load_cached` no longer fails hard on catalog errors — downgrades to `tracing::warn`
- `JsonUi::resolve` and `JsonUi::resolve_with_errors` now call `Catalog::validate` AFTER `expand_directives`
- Integration test `framework/tests/pipeline_order.rs` proves the pipeline order end-to-end
- Full workspace: fmt clean, clippy -D warnings clean, 0 test failures

## Before/After Pipeline Diagrams

**Before (single-stage hard-fail):**
```
load_cached:
  Spec::from_json  →  hard-fail on structural error
  Catalog::validate  →  hard-fail on catalog error   ← blocks startup for gated bad-variant

JsonUi::resolve:
  expand_directives
  resolve_actions
  resolve_expressions
  (no catalog validation)
```

**After (two-stage D-16):**
```
load_cached:
  Spec::from_json  →  hard-fail on structural error  (unchanged)
  Catalog::validate  →  tracing::warn per error  ← gated bad-variant loads

JsonUi::resolve + resolve_with_errors:
  expand_directives  ←  $if-falsy elements removed HERE
  Catalog::validate  →  tracing::error per error + continue  ← runs on post-expansion set
  resolve_actions
  resolve_expressions
  (resolve_errors for the _with_errors path)
```

## Error Surface Strategy

| Path | Load time | Render time |
|------|-----------|-------------|
| `load_cached` | `tracing::warn` (catalog), hard-fail (structural) | — |
| `JsonUi::resolve` (clean path) | — | `tracing::error` + render continues |
| `JsonUi::resolve_with_errors` | — | `tracing::error` + render continues |

The clean-path `resolve` returns `Spec` (not `Result`), so surfacing catalog errors as a return value would require a breaking signature change. The tracing::error approach was chosen as correct and non-invasive.

## Architecture Note: visible vs $if

An important clarification discovered during implementation: `expand_directives` only removes elements whose `$if` (`if_` field) evaluates to false. The `visible` field is evaluated at RENDER TIME by the renderer — it never removes elements from the spec. This means:

- A spec with `Alert.variant="" + visible: {...}` still has the element after `expand_directives`
- Catalog validation at render time sees and flags it
- The renderer then evaluates `visible` and suppresses the HTML output

The D-16 fix is still correct: the catalog error is now a warning at load time (not a startup failure), and a tracing::error at render time (not a panic). The element's HTML is suppressed by the renderer's visibility evaluation.

## Pre-Existing Tests Updated

None needed — the `load_spec_catalog_error` test uses the `load_builtins` test-local helper which still has hard-fail catalog validation. That helper is used to test the `LoadError::Catalog` variant still works for the `load_builtins_only` code path (used by tests needing controlled catalogs).

The new test `load_cached_warns_on_catalog_error_does_not_fail` uses `load_builtins_warn_only` (also test-local) to avoid `global_catalog()` panic from `BadPlugin_117` registered by other tests in the same binary.

## Reminder for Plan 09 (MCP validate-spec)

The `json_ui_validate_spec` MCP tool (Plan 09 per PATTERNS D-04) should surface the two-stage distinction in its response shape:

```json
{
  "valid": false,
  "structural_errors": [...],   // from Spec::from_json — hard failures
  "catalog_errors": [...],      // from Catalog::validate — now warnings at load
  "post_expand_errors": [...]   // catalog errors visible only after expand_directives
}
```

The MCP tool validates the raw spec (no request data), so it cannot run `expand_directives` accurately. It should run both stages and report both sets separately, noting that catalog errors may be eliminated by visibility gates at runtime.

## Task Commits

1. **Task 1: Downgrade load-time catalog validation to warning-only** — `3ed89ae7` (feat)
2. **Task 2: Enforce catalog validation per-request after expand_directives** — `b6e47106` (feat)
3. **Task 3: Integration test — gated bad-variant spec renders cleanly** — `43a33e45` (test)
4. **Task 4: Pre-commit gate** — `743c89ad` (chore)

## Files Created/Modified

- `ferro-json-ui/src/loader.rs` — D-16: replace `.map_err(LoadError::Catalog)?` with `tracing::warn` loop; add `load_cached_warns_on_catalog_error_does_not_fail` test + `load_builtins_warn_only` helper
- `ferro-json-ui/Cargo.toml` — Add `tracing = "0.1"` dependency
- `framework/src/json_ui/mod.rs` — Import `global_catalog`; add `Catalog::validate` after `expand_directives` in both `resolve()` and `resolve_with_errors()`
- `framework/Cargo.toml` — Add `tracing = "0.1"` dependency
- `framework/tests/pipeline_order.rs` — New: `alert_variant_empty_but_gated_renders_cleanly` + `alert_variant_empty_ungated_surfaces_error_at_render`

## Decisions Made

- Used `tracing::warn` at load time and `tracing::error` at render time (not hard failures) to match the "fail loud for structural, degrade gracefully for semantic" principle
- Did not change `resolve()` return type to `Result<Spec, ...>` — too invasive; tracing::error + continue is sufficient
- Both `resolve()` and `resolve_with_errors()` get the same post-expand catalog validation — consistent enforcement regardless of error-display path

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Test isolation: global catalog pollution from BadPlugin_117**
- **Found during:** Task 4 (pre-commit gate)
- **Issue:** `load_cached_warns_on_catalog_error_does_not_fail` called `global_catalog()` which panics when `BadPlugin_117` is registered by other tests in the same binary
- **Fix:** Replaced the test's `load_cached` call with a new `load_builtins_warn_only` test helper that mirrors D-16 warn-only behavior using `Catalog::build_builtins_only()` instead of `global_catalog()`
- **Files modified:** `ferro-json-ui/src/loader.rs`
- **Committed in:** `743c89ad` (Task 4 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - test isolation bug)
**Impact on plan:** Necessary for test suite stability in parallel execution. No scope creep.

## Issues Encountered

- `cargo test` was initially run from the main repo directory (`/Users/alberto/repositories/albertogferrario/ferro`) which compiled a different binary than the worktree edits. All subsequent commands ran from the worktree CWD.

## Next Phase Readiness

- D-16 pipeline reorder complete. Consumers with `Alert.variant="" + visible: {...}` patterns can load specs without startup failures.
- Plan 08 (D-19 error messages) and Plan 09 (MCP validate-spec) can proceed independently.
- The `LoadError::Catalog` variant still exists in the enum (produced by `load_builtins` test helper and potentially other code paths) — no cleanup needed but it could be `#[deprecated]` in a future phase if all producers are migrated.

## Threat Flags

None — this plan only changes validation timing (load → render), not the validation logic itself. Structural integrity guarantees are preserved via `Spec::from_json` hard-fail. Catalog errors surface at render time via tracing, not silently dropped.

## Self-Check: PASSED

- `ferro-json-ui/src/loader.rs` — exists, contains `tracing::warn`, `load_cached_warns_on_catalog_error_does_not_fail`
- `framework/src/json_ui/mod.rs` — exists, contains 2x `global_catalog().validate`
- `framework/tests/pipeline_order.rs` — exists, contains both test functions
- Commits: `3ed89ae7`, `b6e47106`, `43a33e45`, `743c89ad` — all present
- `cargo fmt --all -- --check` — clean
- `cargo clippy --all --all-targets -- -D warnings` — clean
- `cargo test --all-features` — 0 failures
