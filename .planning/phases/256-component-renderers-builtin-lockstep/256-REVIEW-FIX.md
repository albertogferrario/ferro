---
phase: 256-component-renderers-builtin-lockstep
fixed_at: 2026-07-06T03:05:00Z
review_path: .planning/phases/256-component-renderers-builtin-lockstep/256-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 256: Code Review Fix Report

**Fixed at:** 2026-07-06T03:05:00Z
**Source review:** .planning/phases/256-component-renderers-builtin-lockstep/256-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 5 (fix_scope: critical_warning — 0 Critical, 5 Warning; 4 Info findings out of scope)
- Fixed: 5
- Skipped: 0

**Verification:** Every fix was verified with `cargo fmt --all` + `cargo test -p ferro-json-ui` (or `-p ferro-mcp` for WR-03) before its commit. The full CI-exact gate ran green before the final commit: `cargo fmt --all -- --check`, `cargo clippy --all --all-targets --all-features -- -D warnings`, `cargo test --all-features` (exit 0, 135 suites, zero failures). No schema-export artifacts under `docs/protocol/schemas/` changed — component prop schemas are generated at runtime via `schema_for!`, not committed files, so there was nothing to fold in under the 255 V-07 precedent.

## Fixed Issues

### WR-01: `TileGridProps.form_id` is required but never used by the renderer

**Files modified:** `ferro-json-ui/src/component.rs`, `ferro-json-ui/src/render/containers.rs`
**Commit:** 7a0e46d3
**Applied fix:** `render_tile_grid` now emits `data-selection-form="{form_id}"` (escaped) on the grid root — the same scope attribute the SelectionPanel root carries — making the required prop functional and the TileGrid↔SelectionPanel pairing introspectable in markup. The `form_id` rustdoc was corrected to state the actual D-11 contract: the grid and its paired panel must both be descendants of `<form id="{form_id}">`, because the selection runtime scopes its queries and input-event listener to `document.getElementById(form_id)`; tiles placed outside the form neither submit nor appear in the panel. The renderer rustdoc documents the emitted attribute. New HTML assertion test `tile_grid_emits_selection_form_scope`. (Review option (a) — the HTML `form=` attribute on hidden inputs — was not taken: the inputs are emitted by `render_tile`, not `render_tile_grid`, and `form=` association would not make the runtime's subtree `querySelectorAll`/event-bubbling scoping work for out-of-form placement anyway; the D-11 form-ancestor composition is the locked contract, now documented truthfully.)

### WR-02: SelectionPanel inc/dec handlers lack the NaN guard used everywhere else

**Files modified:** `ferro-json-ui/src/runtime/selection.rs`
**Commit:** 0e415f44
**Applied fix:** Both delegated per-line handlers now guard `parseInt` with the same `|| 0` fallback used in `runtime/tiles.rs` and the reconciler: `(parseInt(input.value, 10) || 0) + 1` and `Math.max(0, (parseInt(input.value, 10) || 0) - 1)`. A non-numeric input value can no longer produce a literal `"NaN"` hidden-input value that would POST on confirm. ES5-only style preserved.

### WR-03: MCP `register-selection-present` rule mapping omits `TileGrid`

**Files modified:** `ferro-mcp/src/tools/json_ui_catalog.rs`
**Commit:** 7c088e3c
**Applied fix:** `RULE_COMPONENTS` entry for `register-selection-present` extended to `&["Grid", "TileGrid", "Numpad", "SelectionPanel"]`, so an agent fetching the TileGrid catalog entry now sees the composition rule its presence actually triggers. The three-direction drift guard (`design_system_component_guidance_drift_guarded`) and all 19 `json_ui_catalog` tests stay green.

### WR-04: User-visible English strings without override in a project-agnostic crate

**Files modified:** `ferro-json-ui/src/component.rs`, `ferro-json-ui/src/render/containers.rs`
**Commit:** 1c6eaef7
**Applied fix:** Per locked decision D-28, the neutral English defaults are kept and made overridable via additive props: `TileGridProps.all_label: Option<String>` (default "All", threaded into `render_filter_tab_strip` for the integrated category strip, mirroring the standalone `FilterTabsProps.all_label`) and `SelectionPanelProps.total_label: Option<String>` (default "Total", HTML-escaped, mirroring the existing `empty_message` pattern). Both props follow the additive-prop convention (`#[serde(default, skip_serializing_if = "Option::is_none")]` + rustdoc); the existing per-struct schema smoke tests cover the new fields automatically. New render tests `tile_grid_all_label_overridable` and `selection_panel_total_label_overridable` assert both the neutral defaults and the override path.

### WR-05: components.md is stale — five new builtins undocumented, Tile section contradicts its own migration row

**Files modified:** `docs/src/json-ui/components.md`
**Commit:** 3a83c521
**Applied fix (deliberately scoped):** The Tile section was rewritten to the tap-to-add contract — the whole tile is one `<button data-qty-inc>` tap surface, no on-tile steppers or qty display, quantity editing lives in the SelectionPanel — and its props table now lists all current props (`categories`, `image_url`, `color`, `stock_badge`, `price_cents` with its `data-unit-price` emission and running-total role) with an updated example and Form-placement note. **Scope decision:** the review's remaining asks — adding TileGrid/SelectionPanel/FilterTabs/QuantityStepper/Numpad to the overview table and body sections, and `numpad_mode` to the Component-Specific Enum Values list — were NOT done here because Phase 258 owns the full five-component documentation (props tables + examples) per the roadmap. This fix removes only the actively wrong content (the retired stepper description that contradicted the v16.6 migration row); the "documents every built-in component" claim at line 19 becomes true when Phase 258 lands the five sections.

## Skipped Issues

None.

---

_Fixed: 2026-07-06T03:05:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
