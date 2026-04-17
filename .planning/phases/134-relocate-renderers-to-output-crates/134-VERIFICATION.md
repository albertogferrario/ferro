---
phase: 134-relocate-renderers-to-output-crates
verified: 2026-04-17T16:30:00Z
status: passed
score: 9/9 must-haves verified
re_verification: false
gaps: []
---

# Phase 134: Relocate Renderers to Output Crates — Verification Report

**Phase Goal:** Move `JsonUiRenderer` and its supporting modules (`field_map.rs`, `relationship_map.rs`) from `ferro-projections/src/render/` to `ferro-json-ui`. ferro-projections retains: the `Renderer` trait, `derive_intents()`, `ServiceDef`, `IntentScore`, and `TemplateRenderer`. ferro-json-ui gains a dependency on ferro-projections for the trait and types.

**Verified:** 2026-04-17T16:30:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `JsonUiRenderer` importable from `ferro_json_ui` (not `ferro_projections`) | VERIFIED | `ferro-json-ui/src/lib.rs:94` re-exports `JsonUiRenderer` under `#[cfg(feature = "projections")]`; confirmed importable via `cargo test -p ferro-json-ui --features projections` (567 unit tests pass) |
| 2 | `VisualContext` and `RenderMode` importable from `ferro_json_ui` | VERIFIED | Same re-export line (`ferro-json-ui/src/lib.rs:94`); both types defined in `ferro-json-ui/src/projection/mod.rs:28,40` |
| 3 | All tests pass under ferro-json-ui | VERIFIED | `cargo test -p ferro-json-ui --features projections`: 567 unit tests + 6 doc tests, 0 failures |
| 4 | `is_system_field` is `pub` in ferro-projections and callable cross-crate | VERIFIED | `ferro-projections/src/render/mod.rs:84`: `pub fn is_system_field` (not `pub(crate)`) |
| 5 | ferro-projections no longer exports `JsonUiRenderer`, `VisualContext`, or `RenderMode` | VERIFIED | `ferro-projections/src/lib.rs` contains no mention of these types (grep returns empty) |
| 6 | ferro-projections has no `visual` feature flag or `ferro-theme` dependency | VERIFIED | `ferro-projections/Cargo.toml` contains no `[features]` section and no `ferro-theme` reference |
| 7 | `ferro-projections/src/render/` contains only `mod.rs` and `template.rs` | VERIFIED | `ls ferro-projections/src/render/` returns exactly `mod.rs` and `template.rs`; `json_ui.rs`, `field_map.rs`, `relationship_map.rs` deleted |
| 8 | ferro-mcp imports `JsonUiRenderer`, `RenderMode`, `VisualContext` from `ferro_json_ui` | VERIFIED | `ferro-mcp/src/tools/render_projection.rs:6`: `use ferro_json_ui::{JsonUiRenderer, RenderMode, VisualContext};` |
| 9 | Full workspace compiles and all tests pass | VERIFIED | `cargo fmt --all -- --check` (clean), `cargo clippy --all --all-targets -- -D warnings` (clean), `cargo test --all-features` (all pass) |

**Score:** 9/9 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-json-ui/src/projection/mod.rs` | `JsonUiRenderer`, `VisualContext`, `RenderMode` (min 2500 lines) | VERIFIED | 2588 lines; all three types present; no `use crate::` imports |
| `ferro-json-ui/src/projection/field_map.rs` | `field_to_column`, `field_to_display`, `field_to_input` (min 500 lines) | VERIFIED | 554 lines; all three functions exported as `pub fn` |
| `ferro-json-ui/src/projection/relationship_map.rs` | `relationship_to_component` (min 90 lines) | VERIFIED | 106 lines; function present as `pub fn` |
| `ferro-projections/src/lib.rs` | Clean exports without visual types | VERIFIED | No `JsonUiRenderer`, `VisualContext`, `RenderMode`, or `cfg(feature = "visual")` present |
| `ferro-projections/Cargo.toml` | No `visual` feature, no `ferro-theme` dep | VERIFIED | `[features]` section absent; `ferro-theme` absent |
| `ferro-mcp/src/tools/render_projection.rs` | Imports from `ferro_json_ui` | VERIFIED | Line 6: `use ferro_json_ui::{JsonUiRenderer, RenderMode, VisualContext};` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ferro-json-ui/src/projection/mod.rs` | `ferro-projections::render::Renderer` | `use ferro_projections::render::{field_display_name, is_system_field, Renderer}` | WIRED | Line 23 of projection/mod.rs |
| `ferro-json-ui/src/projection/field_map.rs` | `ferro-projections::FieldDef` | `use ferro_projections` (crate-root re-exports) | WIRED | Imports via crate-root per STATE.md convention |
| `ferro-json-ui/src/projection/mod.rs` | `ferro-theme::ThemeTemplates` | `use ferro_theme::{IntentSlotTemplate, ThemeTemplates}` | WIRED | Line 11 of projection/mod.rs |
| `ferro-mcp/src/tools/render_projection.rs` | `ferro-json-ui::JsonUiRenderer` | `use ferro_json_ui::{JsonUiRenderer, RenderMode, VisualContext}` | WIRED | Line 6; types used at lines 67, 72, 80 |

### Data-Flow Trace (Level 4)

Not applicable — this phase relocates renderer code, not runtime data pipelines. No component renders user-facing dynamic data that requires tracing through fetch/store cycles.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| ferro-json-ui tests pass with projections feature | `cargo test -p ferro-json-ui --features projections` | 567 unit + 6 doc tests: 0 failures | PASS |
| ferro-projections compiles without renderer code | `cargo build -p ferro-projections` | Finished `dev` profile (0.08s) | PASS |
| No formatting issues workspace-wide | `cargo fmt --all -- --check` | No output (clean) | PASS |
| No clippy warnings workspace-wide | `cargo clippy --all --all-targets -- -D warnings` | Finished cleanly, no warnings | PASS |
| Full workspace test suite | `cargo test --all-features` | All tests pass, 0 failures | PASS |

### Requirements Coverage

No requirement IDs were declared in the PLAN frontmatter (`requirements: []` in both plans). The phase is a pure structural refactor tracked by exit criteria in ROADMAP.md, not by requirement IDs. No orphaned requirement IDs found in REQUIREMENTS.md for this phase.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| — | — | — | — | — |

No anti-patterns found. Scanned all six modified/created files: no TODOs, no empty returns, no placeholder comments, no hardcoded stubs in rendering paths.

### Human Verification Required

None. All exit criteria are programmatically verifiable (import paths, file existence, test suite, compile checks). No visual, UX, or external service behavior to verify.

### Gaps Summary

No gaps. All nine observable truths verified against the codebase. The phase achieved its goal completely:

- `JsonUiRenderer` lives in `ferro-json-ui` behind the `projections` feature flag and is importable from there.
- `ferro-projections/src/render/` contains only `mod.rs` (Renderer trait, BaseContext, field_display_name, is_system_field) and `template.rs`.
- `ferro-mcp` and `framework` both import visual types from `ferro-json-ui`, not from `ferro-projections`.
- The entire workspace passes `fmt`, `clippy`, and `cargo test --all-features`.

---

_Verified: 2026-04-17T16:30:00Z_
_Verifier: Claude (gsd-verifier)_
