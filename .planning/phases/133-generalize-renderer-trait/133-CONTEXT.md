# Phase 133: Generalize Renderer Trait - Context

**Gathered:** 2026-04-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Refactor the `Renderer` trait in ferro-projections from visual-only to modality-agnostic. Change the trait signature to use associated types for output and context. Update both existing renderers (JsonUiRenderer, TemplateRenderer) to implement the new trait. Remove the `ferro-projections → ferro-theme` dependency. Do NOT relocate renderers to other crates — that is Phase 134.

</domain>

<decisions>
## Implementation Decisions

### Trait Design
- **D-01:** `Renderer` trait gains `type Output` and `type Context: Default` associated types, replacing hardcoded `serde_json::Value` return and `&RenderContext` parameter.
- **D-02:** No trait objects needed. Renderers are always used as concrete types (`JsonUiRenderer`, `TemplateRenderer`), never as `dyn Renderer`. Associated types are the correct Rust pattern.
- **D-03:** The new trait signature:
  ```rust
  pub trait Renderer: Send + Sync {
      type Output;
      type Context: Default;
      fn render(
          &self,
          service: &ServiceDef,
          intents: &[IntentScore],
          ctx: &Self::Context,
      ) -> Result<Self::Output, Error>;
  }
  ```

### Context Type Hierarchy
- **D-04:** Create a `BaseContext` struct retaining `intent_index: usize` and `current_state: Option<String>` — these are modality-agnostic (every renderer needs to know which intent and what entity state).
- **D-05:** Remove `RenderMode` (Display/Input) and `ThemeTemplates` from the base. These are visual concerns.
- **D-06:** `JsonUiRenderer` uses `VisualContext` (or similar) as its `Context` type, containing `BaseContext` fields plus `mode: RenderMode` plus `templates: Option<ThemeTemplates>`. This context type lives in `ferro-projections/src/render/json_ui.rs` for now (moves to ferro-json-ui in Phase 134).
- **D-07:** `TemplateRenderer` uses `BaseContext` directly as its `Context` type (it doesn't need mode or themes).

### Output Types
- **D-08:** `JsonUiRenderer::Output = serde_json::Value` (JSON-UI component tree spec).
- **D-09:** `TemplateRenderer::Output = serde_json::Value` (generic structured template context).

### Dependency Removal
- **D-10:** Remove `ferro-theme` from `ferro-projections/Cargo.toml` dependencies. `ThemeTemplates` import moves into `json_ui.rs` where `VisualContext` is defined — but since ferro-theme is still needed there, it becomes an optional dependency behind a feature flag (e.g., `visual`), or the json_ui module uses a re-export. The cleanest path: `VisualContext` references `ferro_theme::ThemeTemplates` behind `#[cfg(feature = "visual")]` so ferro-projections can compile without ferro-theme.
- **D-11:** Alternative: since Phase 134 moves JsonUiRenderer to ferro-json-ui anyway, the simplest Phase 133 approach is to keep ferro-theme as an optional dep behind a `visual` feature that json_ui.rs requires. This avoids two rounds of dependency surgery.

### Migration Scope
- **D-12:** Update all internal callers within ferro-projections (tests, renderer modules).
- **D-13:** Do NOT update ferro-mcp or ferro-cli imports in this phase — they will break but are fixed in Phase 134 when renderers relocate. Mark this as a known breakage in the summary.
- **D-14:** Actually — ferro-mcp and ferro-cli DO import from ferro-projections and need to compile. Minimum fix: update their `RenderContext::default()` calls to use the new `VisualContext::default()` (or equivalent). Keep the fix minimal — just enough for compilation.

### Claude's Discretion
- Naming: `BaseContext` vs `ProjectionContext` vs just keeping fields flat
- Whether `VisualContext` embeds `BaseContext` by composition or flattens the fields
- Whether `RenderMode` stays in ferro-projections (as a general concept) or moves to the visual context module
- Test structure and coverage approach

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Core files to modify
- `ferro-projections/src/render/mod.rs` — Current Renderer trait, RenderContext, RenderMode, is_system_field, field_display_name
- `ferro-projections/src/render/json_ui.rs` — JsonUiRenderer impl, uses RenderContext and ThemeTemplates
- `ferro-projections/src/render/template.rs` — TemplateRenderer impl, uses RenderContext
- `ferro-projections/src/lib.rs` — Re-exports RenderContext, RenderMode, Renderer
- `ferro-projections/Cargo.toml` — ferro-theme dependency to make optional or remove

### Downstream consumers (minimum compilation fix)
- `ferro-mcp/src/tools/` — imports JsonUiRenderer and RenderContext from ferro-projections
- `ferro-cli/src/commands/` — imports from ferro-projections

### Reference
- `ferro-theme/src/template.rs` — ThemeTemplates, IntentSlotTemplate definitions
- `ferro-projections/CLAUDE.md` — Crate boundary rules (renderers don't add deps to this crate)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `RenderContext` at render/mod.rs:30-51 — fields to split into base vs visual
- `RenderMode` at render/mod.rs:20-27 — moves to visual context
- `field_display_name()` and `is_system_field()` at render/mod.rs:86-109 — stay in mod.rs (modality-agnostic utilities)

### Established Patterns
- `#[async_trait]` not needed — Renderer::render is sync
- All types derive `Serialize, Deserialize, Debug, Clone` per crate conventions
- `Default` impl on context types (RenderContext has one at lines 42-51)

### Integration Points
- `ferro-projections/src/render/json_ui.rs:64-77` — JsonUiRenderer::render() uses `ctx.intent_index`, `ctx.current_state`, `ctx.mode`, `ctx.templates`
- `ferro-projections/src/render/template.rs` — TemplateRenderer::render() uses `ctx.intent_index`, `ctx.current_state` only (never touches mode or templates)
- `ferro-theme` imported at render/mod.rs:16 for `ThemeTemplates` in RenderContext

</code_context>

<specifics>
## Specific Ideas

- The TemplateRenderer already only uses intent_index and current_state — it's a natural validation that the base context split is correct
- `RenderMode` enum could arguably stay in render/mod.rs as a shared concept (other renderers might have display/input modes), but per the architecture discussion it's visual-specific. Move it to the visual context.

</specifics>

<deferred>
## Deferred Ideas

- Relocating JsonUiRenderer to ferro-json-ui → Phase 134
- ServiceDef derivation from models → Phase 135
- Crate consolidation → CONC-04 in v13.0

</deferred>

---

*Phase: 133-generalize-renderer-trait*
*Context gathered: 2026-04-14*
