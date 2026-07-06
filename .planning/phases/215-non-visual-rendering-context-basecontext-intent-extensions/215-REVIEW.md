---
phase: 215-non-visual-rendering-context-basecontext-intent-extensions
reviewed: 2026-06-13T15:30:00Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - ferro-projections/src/render/mod.rs
  - ferro-projections/src/intent.rs
  - ferro-projections/src/error.rs
  - ferro-json-ui/src/projection/mod.rs
  - ferro-json-ui/src/projection/builder.rs
  - ferro-mcp/src/tools/render_projection.rs
  - ferro-mcp/src/tools/generate_projection.rs
  - ferro-mcp/src/tools/projection_coverage.rs
  - ferro-ai/tests/projection_roundtrip.rs
  - ferro-mcp/tests/agent_harness.rs
findings:
  critical: 0
  warning: 0
  info: 3
  total: 3
status: clean
---

# Phase 215: Code Review Report

**Reviewed:** 2026-06-13T15:30:00Z
**Depth:** standard
**Files Reviewed:** 10
**Status:** clean

## Summary

This phase added `evaluated_guards`/`verbosity` to `BaseContext`, the `Verbosity` enum, `Intent::label() -> &str`, and `Error::NoIntents` to `ferro-projections`; refactored `VisualContext` to embed `BaseContext`; migrated `builder.rs` field access to `ctx.base.*`; and switched 4 ferro-mcp label sites from `format!("{:?}", intent)` to `.label().to_string()`.

All reviewed files are correct. The label literals in `Intent::label()` match the serde snake_case output exactly for all 7 variants (verified by the `intent_label_known_variants` test and cross-checked against `#[serde(rename_all = "snake_case")]`). The `{:?}`-to-`.label()` migration is complete across all 4 targeted sites with no remaining label-producing `format!("{:?}", intent)` patterns in `ferro-mcp/src/tools/`. The `VisualContext` embed has no silent field-default losses: every struct-literal site uses `..Default::default()` on `BaseContext` to fill the two new fields, and every `VisualContext` literal that specifies all three fields correctly omits the update syntax. `evaluated_guards` and `verbosity` are defined but not wired into any current render path, which is correct for this phase (they are API surface for Phase 216). `Error::NoIntents` is defined and tested but not yet returned by any code path, which is also correct per the phase scope.

Three info-level observations are noted below; none affect correctness.

## Info

### IN-01: `Verbosity` not in top-level ferro-projections re-exports

**File:** `ferro-projections/src/lib.rs:20`
**Issue:** `ferro-projections::render::{BaseContext, Renderer}` are re-exported at the crate root, but `Verbosity` is not. Non-visual renderer implementors in external crates must import it as `ferro_projections::render::Verbosity`. The module is `pub mod render` so it is accessible, but it is asymmetric with `BaseContext`.
**Fix:** Add `Verbosity` to the re-export line when Phase 216 ships its first non-visual renderer and the type becomes part of the stable surface:
```rust
pub use render::{BaseContext, Renderer, Verbosity};
```
Deferring until Phase 216 is acceptable since no external consumer currently needs it.

### IN-02: `test_coverage_report_serialization` fixture uses PascalCase `"Browse"` while production now emits `"browse"`

**File:** `ferro-mcp/src/tools/projection_coverage.rs:210`
**Issue:** The test constructs a `ModelCoverage` struct directly with `primary_intent: Some("Browse".to_string())` and asserts `json_str.contains("Browse")`. The production code path (`derive_primary_intent` → `primary.intent.label().to_string()`) now returns `"browse"` (lowercase) after the `{:?}`-to-`.label()` migration. The test passes independently of the production path because it round-trips a manually constructed value — no failure, but the fixture no longer reflects what `execute()` would actually produce for that field.
**Fix:** Update the test fixture to use the lowercase form and strengthen the assertion if a real integration path becomes available:
```rust
primary_intent: Some("browse".to_string()),
// ...
assert!(json_str.contains("\"browse\""));
```

### IN-03: `Error::NoIntents` is defined but never returned by any current code path

**File:** `ferro-projections/src/error.rs:17`
**Issue:** `Error::NoIntents` is defined with a doc comment saying it is "returned by render entry points instead of a silent `\"unknown\"`". Currently no render entry point returns it — `JsonUiRenderer::render` maps the empty-intents condition to `Error::Render(...)` via `ProjectionError::EmptyIntents`, and no other caller emits it. The definition is correct as forward-looking API surface for Phase 216, but it is currently dead code.
**Fix:** No change needed for this phase. When Phase 216 wires its text renderer, it should emit `Error::NoIntents` at its entry point and add a test that confirms the variant is reachable. The existing unit test in `error.rs` pins the error message string, which is sufficient coverage for the definition itself.

---

_Reviewed: 2026-06-13T15:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
