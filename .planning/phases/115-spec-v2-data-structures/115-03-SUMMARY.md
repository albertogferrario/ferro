---
phase: 115-spec-v2-data-structures
plan: 03
subsystem: framework-integration
tags: [json-ui, spec-v2, framework-facade, caller-migration, lib-re-exports]

# Dependency graph
requires:
  - phase: 115-spec-v2-data-structures
    plan: 02
    provides: Spec/Element/SpecBuilder types, render_spec_to_html_with_plugins, resolve_actions/resolve_errors on &mut Spec, v2 re-exports from ferro-json-ui
provides:
  - framework::JsonUi facade accepting &Spec across all 11 public/private methods
  - framework crate lib.rs re-exports of Spec, Element, SpecBuilder, ElementBuilder, SpecError, MAX_NESTING_DEPTH
  - Ferro app handlers compile against Spec::builder() instead of JsonUiView::new()
affects:
  - 115-04-ferro-mcp-caller-migration (Plan 04 runs in parallel; now has framework v2 re-exports to consume)
  - 116-flat-element-renderer (will replace the placeholder renderer; framework surface is already v2)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "JsonUi::render(&Spec, &Value) facade — mirrors the v1 JsonUi::render(&JsonUiView, &Value) surface with only the type swap"
    - "JSON payload key \"view\" renamed to \"spec\" in render_json / render_json_with_errors responses"
    - "Placeholder renderer limitation surfaced via single #[ignore] tag — no backward-compat shim"

key-files:
  created: []
  deleted: []
  modified:
    - framework/src/json_ui/mod.rs
    - framework/src/lib.rs

key-decisions:
  - "empty_view_renders_valid_html renamed to empty_spec_renders_valid_html and rewritten to Spec::builder().element(\"root\", Element::new(\"Text\")). A zero-element Spec is not representable under v2 structural validation (SpecBuilder::build returns RootMissing when no element was added). Preserving the test's intent — \"an otherwise-empty spec still renders a valid HTML shell with default title\" — required the single-element minimum."
  - "render_with_errors_uses_layout mutates spec.layout post-build (`spec.layout = Some(\"auth\".to_string())`). SpecBuilder has no post-hoc layout setter on the built Spec, and rebuilding the form spec just to swap layouts would double the test fixture. Direct mutation on an owned-in-test clone is acceptable and exercises the same code path."
  - "Plugin test test_plugin_component_renders_in_full_page is the only #[ignore]'d test. The nested theme_tests module was NOT tagged — theme CSS injection happens in build_response before the placeholder renderer runs, so those four theme tests still pass on the placeholder path. This matches the plan's guidance: \"Tests that check theme CSS head injection … should still pass — theme injection happens in build_response before the placeholder renderer runs.\""
  - "LayoutContext.view_json field name and the data-view HTML attribute are preserved (not renamed to data-spec). Per plan, layout compatibility is kept; only the response JSON payload key changes."

patterns-established:
  - "Framework callers of ferro-json-ui reference Spec / Element / SpecBuilder through ferro_rs::{...} re-exports — no direct ferro_json_ui import needed downstream."
  - "Test fixtures construct specs via Spec::builder() + Element::new().prop().action().child() chains. Action is attached to the element that owns it, not to a ComponentNode wrapper."

requirements-completed: [SPEC-04]

# Metrics
duration: ~30min
completed: 2026-04-18
---

# Phase 115 Plan 03: Framework Caller Migration to Spec v2

**Migrate the framework's JSON-UI integration from v1 (`JsonUiView` + `ComponentNode` + `Component` enum) to v2 (`Spec` + `Element`), port all 30 inline tests in `framework/src/json_ui/mod.rs`, and swap re-exports in `framework/src/lib.rs`. After this plan, any handler importing `ferro_rs::{JsonUi, Spec, Element}` compiles and renders against the v2 surface.**

## Performance

- **Tasks:** 2 (both completed)
- **Files modified:** 2
  - `framework/src/json_ui/mod.rs` (1020 → 1012 LoC; the src portion shrank by ~4 lines, the tests are roughly equal LoC but now express Spec::builder shapes instead of ComponentNode literals)
  - `framework/src/lib.rs` (re-export list: removed 3 names, added 6 names)
- **Tests ported vs ignored:**
  - 30 total `#[test]` / `#[tokio::test]` in `framework/src/json_ui/mod.rs`
  - **29 ported to v2 and passing** (25 under `--features json-ui` + 4 under `--features "json-ui,theme"`)
  - **1 `#[ignore]`'d** with `TODO(Phase 116)` comment (`test_plugin_component_renders_in_full_page`)

## Accomplishments

**JsonUi facade (Task 1, commit `fee225f8`):**
- All 11 methods on `impl JsonUi` migrated from `&JsonUiView` → `&Spec`
  - Public: `render`, `render_with_config`, `render_json`, `render_with_errors`, `render_json_with_errors`, `render_validation_error`, `render_json_validation_error`
  - Private: `resolve`, `build_response`, `resolve_with_errors`, `render_with_errors_config`
- Local variables renamed `view` → `spec`, `resolved` → `resolved_spec` for consistency
- Import block swap: `render_to_html_with_plugins` → `render_spec_to_html_with_plugins`; `JsonUiView` dropped; `Spec` added
- `resolved.errors = Some(errors.clone())` line deleted (D-06: Spec has no errors field). The errors map still threads through `resolve_errors(&mut spec, errors)` which populates `props.errors` on matching form elements — Phase 116 decides how the renderer surfaces the full bag.
- `render_json` / `render_json_with_errors` rename the JSON payload key from `"view"` to `"spec"` to match v2 terminology. `LayoutContext.view_json` keeps its name for layout compatibility (layouts emit `data-view=` attributes; that HTML attribute name is a stable surface).
- Module rustdoc example rewritten to `Spec::builder().element(...).build().expect(...)` syntax

**Framework re-exports (Task 1):**
- Dropped: `Component`, `ComponentNode`, `JsonUiView`
- Added: `Spec`, `Element`, `SpecBuilder`, `ElementBuilder`, `SpecError`, `MAX_NESTING_DEPTH`
- Kept: all 40+ Props structs, enums, layout types, `JsonUiConfig`, `SCHEMA_VERSION` (now referring to the v2 constant)
- Projection re-exports (`JsonUiRenderer`, `RenderMode`, `VisualContext`) unchanged

**Test suite port (Task 2, commit `26fa9737`):**
- Helper functions: `sample_view() → sample_spec()`, `form_view_with_inputs() → form_spec_with_inputs()`, added `sample_spec_with_layout(&str)`
- Fixture pattern rewrite (30 tests touching ~20+ fixtures): `JsonUiView::new().component(ComponentNode { key, component: Component::X(XProps { ... }), action, visibility })` becomes `Spec::builder().element(id, Element::new(type).prop(k, v).action(a).child(c)).build().unwrap()`
- Per-test adaptations:
  - `render_json_returns_json`: assertion `body.contains("view")` → `body.contains("spec")`
  - `render_resolves_action_urls`: mutation check against `spec.elements.get("btn").unwrap().action.as_ref().unwrap().url` (v1 used `view.components[0]` positional access)
  - `render_with_errors_uses_layout`: direct mutation of `spec.layout` post-build
- Theme-feature tests (4): rewrote inner `sample_view()` helper inside `theme_tests` module to return `Spec`. All four still pass on the placeholder renderer path because theme CSS injection happens in `build_response` before `render_spec_to_html_with_plugins` is called.
- Single ignored test:
  ```rust
  #[ignore = "TODO(Phase 116): placeholder renderer does not collect plugin assets"]
  fn test_plugin_component_renders_in_full_page()
  ```

## Task Commits

1. **Task 1: JsonUi facade + framework re-exports** — `fee225f8` (refactor)
2. **Task 2: Port inline tests to Spec::builder** — `26fa9737` (test)

## Files Modified

### Modified
- `framework/src/json_ui/mod.rs` — 1020 → 1012 LoC. src portion rewritten (Task 1); ~20 test fixtures ported (Task 2); 1 plugin test `#[ignore]`'d.
- `framework/src/lib.rs` — re-export list swapped (removed 3 v1 names, added 6 v2 names, kept everything else).

## Ignored tests (for Phase 116 follow-up)

| Test function | Module | Reason |
|--|--|--|
| `test_plugin_component_renders_in_full_page` | `framework::json_ui::tests` | Asserts Leaflet CSS/JS link + `data-ferro-map` container + `DOMContentLoaded` init script. Placeholder renderer does not walk the element graph so it collects no plugin assets. Test body compiles cleanly against v2 Spec types — Phase 116 only needs to remove the `#[ignore]` once the real walker lands. |

Zero theme tests were ignored — all 4 pass on the placeholder path because theme CSS injection is independent of element rendering.

## Deviations from Plan

None of substance. Three minor execution notes:

1. **`cargo fmt -p ferro-rs` reformatted `build_response`** from a multi-line signature (`fn build_response(\n    spec: &Spec,\n    data: &serde_json::Value,\n    config: &JsonUiConfig,\n) -> Response {`) to a single-line signature. This is expected rustfmt behavior on short-enough parameter lists and was included in the Task 1 commit via the linter hook.
2. **`empty_view_renders_valid_html` renamed to `empty_spec_renders_valid_html`** and its body changed from `JsonUiView::new()` (zero components allowed in v1) to `Spec::builder().element("root", Element::new("Text")).build().expect(...)` — SpecBuilder requires at least one element. The test still asserts the same things: `<!DOCTYPE html>` present and default title `"Ferro"` when none set.
3. **`render_with_errors_uses_layout` mutates `spec.layout` post-build** rather than re-running `form_spec_with_inputs` with a layout parameter. This is a cheaper fixture adaptation and exercises the same code path inside `build_response`.

## Issues Encountered

- **Environmental: workspace disk 100% full during final `cargo build -p ferro-rs --all-targets --all-features`**. Separate `cargo clippy -p ferro-rs --all-targets --all-features -- -D warnings` had completed successfully prior to the full build attempt, and the combined incremental artifacts pushed the disk over. This is outside the scope of Plan 03 and does not affect the correctness of the migration — every required gate (lib build, json_ui test suite under both feature combinations, clippy --all-targets --all-features, fmt check) passed independently. Flagged in "Deferred Items" for the phase owner.

## Self-Check: PASSED

**Files verified:**
- `framework/src/json_ui/mod.rs` — FOUND (1012 LoC)
- `framework/src/lib.rs` — FOUND (re-export block updated)
- `.planning/phases/115-spec-v2-data-structures/115-03-SUMMARY.md` — FOUND (this file)

**Commits verified:**
- `fee225f8` (Task 1 refactor) — FOUND in git log
- `26fa9737` (Task 2 test) — FOUND in git log

**Acceptance gates (run 2026-04-18):**
- `cargo build -p ferro-rs --lib` → 0
- `cargo test -p ferro-rs --features json-ui --lib json_ui::tests` → 25 passed, 0 failed, 1 ignored
- `cargo test -p ferro-rs --features "json-ui,theme" --lib json_ui::tests` → 29 passed, 0 failed, 1 ignored
- `cargo clippy -p ferro-rs --all-targets --all-features -- -D warnings` → 0
- `cargo fmt --all -- --check` → 0

**Grep invariants verified:**
- `grep -q "JsonUiView::new" framework/src/json_ui/mod.rs` → empty (removed)
- `grep -q "ComponentNode {" framework/src/json_ui/mod.rs` → empty (removed)
- `grep -qE "\bComponent::(Button|Card|Input|Alert|Text|Table|Form|Modal|Select|Checkbox|Switch|Badge|Tab|Tabs|Map|Plugin)\b" framework/src/json_ui/mod.rs` → empty (removed)
- `grep -q "JsonUiView" framework/src/lib.rs` → empty (removed)
- `grep -q "ComponentNode" framework/src/lib.rs` → empty (removed)
- `grep -q "Spec::builder" framework/src/json_ui/mod.rs` → present
- `grep -q "Element::new" framework/src/json_ui/mod.rs` → present
- `grep -q "TODO(Phase 116)" framework/src/json_ui/mod.rs` → present (single occurrence)
- `grep -q "resolved.errors = Some" framework/src/json_ui/mod.rs` → empty (D-06 enforcement)
- Re-exports `Spec`, `Element`, `SpecBuilder`, `ElementBuilder`, `SpecError`, `MAX_NESTING_DEPTH` all grep-confirmed in `framework/src/lib.rs`

## Workspace Status

`ferro-rs` (framework crate) is v2-only and builds standalone with all `json_ui::tests` passing. Plan 04 now has the framework's v2 re-exports to lean on for `ferro-mcp` / `ferro-cli` migration.

## Next Phase Readiness

- **Plan 04** (ferro-mcp / ferro-cli caller migration) is unblocked. Framework now re-exports `Spec`, `Element`, `SpecBuilder` under `ferro_rs::{...}` for any downstream consumer.
- **Phase 116** (flat element renderer) inherits:
  - A fully-compiling framework that accepts `&Spec` at the top of every JSON-UI rendering call
  - The single ignored plugin test (`test_plugin_component_renders_in_full_page`) as a precise target — removing the `#[ignore]` becomes a regression gate for the new walker's plugin-asset collection
  - Theme CSS injection path already validated on the placeholder (4 theme tests green)

---
*Phase: 115-spec-v2-data-structures*
*Plan: 03*
*Completed: 2026-04-18*
