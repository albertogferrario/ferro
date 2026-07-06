# Phase 134: Relocate Renderers to Output Crates - Context

**Gathered:** 2026-04-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Move `JsonUiRenderer` and its supporting modules (`field_map.rs`, `relationship_map.rs`) plus visual context types (`VisualContext`, `RenderMode`) from `ferro-projections/src/render/` to `ferro-json-ui`. ferro-projections retains: the `Renderer` trait, `BaseContext`, `derive_intents()`, `ServiceDef`, `IntentScore`, and `TemplateRenderer`. ferro-json-ui gains a dependency on ferro-projections (behind a `projections` feature flag) for the trait and types. All downstream consumers (ferro-mcp, ferro-cli) update imports to the new locations.

This establishes the pattern: each output crate provides its own `Renderer` implementation.

</domain>

<decisions>
## Implementation Decisions

### Module Organization
- **D-01:** Create a new `ferro-json-ui/src/projection/` module directory for the relocated code. Do NOT merge with the existing `render.rs` (which handles HTML rendering of component trees). Projection rendering (ServiceDef → JSON-UI spec) is a separate concern.
- **D-02:** Module structure: `projection/mod.rs` (JsonUiRenderer impl + VisualContext + RenderMode), `projection/field_map.rs`, `projection/relationship_map.rs`.
- **D-03:** Re-export `JsonUiRenderer`, `VisualContext`, `RenderMode` from `ferro-json-ui/src/lib.rs` behind `#[cfg(feature = "projections")]`.

### Dependency Direction
- **D-04:** ferro-json-ui adds `ferro-projections` as an optional dependency behind a `projections` feature flag. This keeps ferro-json-ui usable standalone for schema types without pulling in ferro-projections.
- **D-05:** ferro-json-ui also needs `ferro-theme` as a dependency (for `ThemeTemplates` used by `VisualContext`). This should also be behind the `projections` feature flag since it's only needed for projection rendering.

### Re-export Strategy
- **D-06:** Clean break — remove all visual re-exports from `ferro-projections/src/lib.rs` (`JsonUiRenderer`, `VisualContext`, `RenderMode`). No deprecated re-exports. Pre-1.0, breaking changes acceptable.
- **D-07:** Remove the `visual` feature flag and `ferro-theme` optional dependency from `ferro-projections/Cargo.toml` entirely. The `json_ui` module and its contents leave this crate.
- **D-08:** Delete `ferro-projections/src/render/json_ui.rs`, `ferro-projections/src/render/field_map.rs`, `ferro-projections/src/render/relationship_map.rs` after relocation.

### Downstream Consumer Updates
- **D-09:** ferro-mcp (`render_projection.rs`) currently imports `JsonUiRenderer`, `RenderMode`, `VisualContext` from `ferro_projections`. Update to import from `ferro_json_ui`. ferro-mcp already depends on both crates; enable the `projections` feature on its ferro-json-ui dependency.
- **D-10:** ferro-cli (`projection_check.rs`) does NOT import visual types — only `ServiceDef`, `ActionDef`, etc. from ferro-projections. No changes needed to ferro-cli imports for the visual types. However, ferro-cli has ferro-json-ui as a dependency and should enable `projections` if it needs projection rendering in the future.
- **D-11:** Update the doc comment example in the relocated `json_ui.rs` (now `projection/mod.rs`) to use `ferro_json_ui::` import paths instead of `ferro_projections::`.

### Feature Flag Design
- **D-12:** Feature flag name: `projections`. Matches the naming pattern ferro-cli already uses for its optional ferro-projections dependency.
- **D-13:** `ferro-json-ui/Cargo.toml` additions:
  ```toml
  [features]
  projections = ["dep:ferro-projections", "dep:ferro-theme"]

  [dependencies]
  ferro-projections = { path = "../ferro-projections", version = "0.2", optional = true }
  ferro-theme = { path = "../ferro-theme", version = "0.2", optional = true }
  ```

### Claude's Discretion
- Internal module visibility (`pub` vs `pub(crate)`) for helper functions like `is_system_field` and `field_display_name` after relocation
- Whether `field_map.rs` and `relationship_map.rs` tests move with the files or get rewritten
- Whether to keep `render::field_map` and `render::relationship_map` as `pub mod` in ferro-projections `render/mod.rs` or remove them (they may only be used by `json_ui.rs`)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Files to relocate (source)
- `ferro-projections/src/render/json_ui.rs` — JsonUiRenderer impl, VisualContext, RenderMode (2583 lines)
- `ferro-projections/src/render/field_map.rs` — Field-to-component mapping logic (554 lines)
- `ferro-projections/src/render/relationship_map.rs` — Relationship rendering helpers (106 lines)

### Files to modify (ferro-projections cleanup)
- `ferro-projections/src/render/mod.rs` — Remove `pub mod json_ui`, `pub mod field_map`, `pub mod relationship_map` declarations; keep `Renderer` trait, `BaseContext`, `field_display_name`, `is_system_field`, `template` module
- `ferro-projections/src/lib.rs` — Remove `#[cfg(feature = "visual")]` re-exports
- `ferro-projections/Cargo.toml` — Remove `visual` feature and `ferro-theme` optional dep

### Files to modify (ferro-json-ui destination)
- `ferro-json-ui/src/lib.rs` — Add `projection` module, conditional re-exports
- `ferro-json-ui/Cargo.toml` — Add `projections` feature, optional deps

### Downstream consumers to update
- `ferro-mcp/src/tools/render_projection.rs` — Lines 6-10: change `ferro_projections::{JsonUiRenderer, RenderMode, VisualContext}` to `ferro_json_ui::{JsonUiRenderer, RenderMode, VisualContext}`
- `ferro-mcp/Cargo.toml` — Enable `projections` feature on ferro-json-ui dep

### Architecture references
- `ferro-projections/CLAUDE.md` — Crate boundary rules (renderers don't add deps to this crate)
- `.planning/codebase/ARCHITECTURE.md` — Layer breakdown

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `field_display_name()` and `is_system_field()` in `render/mod.rs` — modality-agnostic utilities that stay in ferro-projections. The relocated `field_map.rs` may import these; check if they need to become `pub` (currently `pub` and `pub(crate)` respectively).
- `BaseContext` stays in ferro-projections — `VisualContext` in the relocated code composes with it.

### Established Patterns
- Feature-gated module pattern: `#[cfg(feature = "visual")] pub mod json_ui` already exists in ferro-projections — same pattern applies in ferro-json-ui with `#[cfg(feature = "projections")]`
- ferro-cli already uses optional ferro-projections dep behind a `projections` feature — same pattern for ferro-json-ui

### Integration Points
- ferro-mcp's `render_projection.rs` is the primary consumer of `JsonUiRenderer`. It builds a `ServiceDef`, calls `derive_intents()`, creates a `VisualContext`, and calls `renderer.render()`. Import paths change but the API surface is identical.
- `field_map.rs` and `relationship_map.rs` are internal to `json_ui.rs` — they are not imported by ferro-mcp or ferro-cli directly.

</code_context>

<specifics>
## Specific Ideas

- `field_map.rs` uses `is_system_field` from `render/mod.rs` which is `pub(crate)`. After relocation to ferro-json-ui, this becomes a cross-crate call. Either make `is_system_field` `pub` in ferro-projections and import it, or duplicate the simple `matches!()` expression in ferro-json-ui.
- `field_display_name` is already `pub` — no issue there.
- The `VisualContext` type currently uses `ferro_theme::ThemeTemplates`. After relocation, ferro-json-ui needs ferro-theme as a dependency (behind the `projections` feature).

</specifics>

<deferred>
## Deferred Ideas

- ServiceDef derivation from models → Phase 135
- Crate consolidation audit → CONC-04 in v13.0
- WhatsApp renderer in ferro-whatsapp behind `projections` feature → v14.0+

</deferred>

---

*Phase: 134-relocate-renderers-to-output-crates*
*Context gathered: 2026-04-14*
