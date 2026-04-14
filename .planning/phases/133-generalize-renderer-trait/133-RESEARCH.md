# Phase 133: Generalize Renderer Trait - Research

**Researched:** 2026-04-14
**Domain:** Rust trait design — associated types, dependency graph surgery, feature flags
**Confidence:** HIGH

## Summary

Phase 133 is a focused internal refactor of `ferro-projections`. The current `Renderer` trait is hardcoded to `serde_json::Value` output and `&RenderContext` (which embeds `ThemeTemplates` from `ferro-theme`). This couples the core trait to the visual rendering stack. The goal is to replace both concrete types with associated types, split the monolithic `RenderContext` into a modality-agnostic `BaseContext` and a visual-only `VisualContext`, and remove the `ferro-projections → ferro-theme` hard dependency.

The architecture is well-understood. Decisions are locked in CONTEXT.md. The only non-trivial design question is how to handle `ferro-theme` in `ferro-projections/Cargo.toml` after the split: as an optional dependency behind a `visual` feature flag, or remove it entirely and have `json_ui.rs` declare it directly. Since `json_ui.rs` currently lives in `ferro-projections` and references `ThemeTemplates`, the `visual` feature flag approach is the cleanest path that compiles without breaking the downstream crates before Phase 134 moves `JsonUiRenderer` out.

The primary risk is `ferro-mcp`, which constructs `RenderContext` struct literals directly (not `::default()`) at six call sites. All six must be updated to use `VisualContext` in this phase (per D-14). `ferro-cli` has no direct usage.

**Primary recommendation:** Introduce associated types on the trait, split the context, gate `ferro-theme` behind a `visual` feature in `ferro-projections`, and update `ferro-mcp`'s six `RenderContext` construction sites to `VisualContext`.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `Renderer` trait gains `type Output` and `type Context: Default` associated types, replacing hardcoded `serde_json::Value` return and `&RenderContext` parameter.
- **D-02:** No trait objects needed. Renderers are always used as concrete types. Associated types are the correct Rust pattern.
- **D-03:** New trait signature:
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
- **D-04:** Create `BaseContext` struct retaining `intent_index: usize` and `current_state: Option<String>`.
- **D-05:** Remove `RenderMode` and `ThemeTemplates` from the base context.
- **D-06:** `JsonUiRenderer` uses `VisualContext` as its `Context` type, containing `BaseContext` fields plus `mode: RenderMode` plus `templates: Option<ThemeTemplates>`. Lives in `ferro-projections/src/render/json_ui.rs` for now.
- **D-07:** `TemplateRenderer` uses `BaseContext` directly as its `Context` type.
- **D-08:** `JsonUiRenderer::Output = serde_json::Value`.
- **D-09:** `TemplateRenderer::Output = serde_json::Value`.
- **D-10/D-11:** Remove `ferro-theme` as a hard dependency from `ferro-projections/Cargo.toml`. Make it optional behind a `visual` feature flag. `json_ui.rs` continues to reference `ferro_theme::ThemeTemplates` behind `#[cfg(feature = "visual")]`.
- **D-12:** Update all internal callers within ferro-projections (tests, renderer modules).
- **D-13:** Do NOT update ferro-mcp or ferro-cli imports in full — Phase 134 handles renderer relocation.
- **D-14:** ferro-mcp DOES need minimum compilation fixes: update `RenderContext` construction sites to use `VisualContext` (or equivalent). Keep fix minimal — just enough to compile.

### Claude's Discretion

- Naming: `BaseContext` vs `ProjectionContext` vs flat fields
- Whether `VisualContext` embeds `BaseContext` by composition or flattens the fields
- Whether `RenderMode` stays in `render/mod.rs` (as a shared concept) or moves to the visual context module
- Test structure and coverage approach

### Deferred Ideas (OUT OF SCOPE)

- Relocating `JsonUiRenderer` to `ferro-json-ui` — Phase 134
- `ServiceDef` derivation from models — Phase 135
- Crate consolidation — CONC-04 in v13.0
</user_constraints>

---

## Standard Stack

### Core (all already in use — no new dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `ferro-theme` | workspace (0.2) | `ThemeTemplates`, `IntentSlotTemplate`, `IntentModeTemplates` | Already the source of these types |
| `serde_json` | 1 | `Value` output type for both renderers | Already dep; both renderers output `Value` |
| `thiserror` | 1.0 | `Error` enum | Already crate pattern |

No new dependencies. This phase removes a hard dependency, it does not add any.

**Cargo.toml change required:**
```toml
# ferro-projections/Cargo.toml — BEFORE:
ferro-theme = { path = "../ferro-theme", version = "0.2" }

# AFTER:
[features]
visual = ["ferro-theme"]

[dependencies]
ferro-theme = { path = "../ferro-theme", version = "0.2", optional = true }
```

### Downstream dependency audit

| Crate | Imports from ferro-projections | Impact |
|-------|-------------------------------|--------|
| `ferro-mcp` | `JsonUiRenderer, RenderContext, RenderMode, Renderer` (render_projection.rs) | 6 `RenderContext` construction sites + 8 `RenderMode` references — all need `VisualContext` |
| `ferro-cli` | Nothing matching — confirmed no usage | No change needed |

---

## Architecture Patterns

### New module layout (render/mod.rs)

```
ferro-projections/src/render/
├── mod.rs            — Renderer trait (associated types), BaseContext, field_display_name, is_system_field
├── json_ui.rs        — JsonUiRenderer, VisualContext, RenderMode (visual concern)
├── template.rs       — TemplateRenderer (uses BaseContext directly)
├── field_map.rs      — (unchanged)
└── relationship_map.rs — (unchanged)
```

`RenderMode` moves from `render/mod.rs` into `render/json_ui.rs`. It was only ever consumed by `JsonUiRenderer` logic; having it at mod level was incidental.

### Pattern 1: Associated types on the Renderer trait

```rust
// Source: D-03 (CONTEXT.md locked decision)
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

The `Context: Default` bound is required so callers can construct a zero-config context with `VisualContext::default()` or `BaseContext::default()`.

### Pattern 2: Context split

```rust
// In render/mod.rs — modality-agnostic base
#[derive(Debug, Clone, Default)]
pub struct BaseContext {
    pub intent_index: usize,
    pub current_state: Option<String>,
}

// In render/json_ui.rs — visual renderer context
// (behind #[cfg(feature = "visual")] because it references ThemeTemplates)
#[cfg(feature = "visual")]
#[derive(Debug, Clone)]
pub struct VisualContext {
    pub intent_index: usize,
    pub current_state: Option<String>,
    pub mode: RenderMode,
    pub templates: Option<ferro_theme::ThemeTemplates>,
}

#[cfg(feature = "visual")]
impl Default for VisualContext {
    fn default() -> Self {
        Self {
            intent_index: 0,
            current_state: None,
            mode: RenderMode::Display,
            templates: None,
        }
    }
}
```

**Composition vs. flattening:** The Context.md grants discretion here. Flattening is simpler (no `.base.intent_index` indirection), matches the existing `RenderContext` layout, and avoids the `Deref` ceremony. Recommend flattening.

### Pattern 3: Renderer implementations

```rust
// TemplateRenderer — uses BaseContext
impl Renderer for TemplateRenderer {
    type Output = serde_json::Value;
    type Context = BaseContext;
    fn render(&self, service: &ServiceDef, intents: &[IntentScore], ctx: &BaseContext) -> Result<Value, Error> { ... }
}

// JsonUiRenderer — uses VisualContext
#[cfg(feature = "visual")]
impl Renderer for JsonUiRenderer {
    type Output = serde_json::Value;
    type Context = VisualContext;
    fn render(&self, service: &ServiceDef, intents: &[IntentScore], ctx: &VisualContext) -> Result<Value, Error> { ... }
}
```

### Pattern 4: Feature flag for visual renderers

The `visual` feature gates `json_ui.rs` module and `JsonUiRenderer` export. This prevents `ferro-theme` from being a required compile-time dependency of `ferro-projections` for consumers that only need `TemplateRenderer` or the trait itself.

```toml
# ferro-projections/Cargo.toml
[features]
visual = ["ferro-theme"]
```

```rust
// ferro-projections/src/lib.rs
pub use render::template::TemplateRenderer;
pub use render::{BaseContext, Renderer};

#[cfg(feature = "visual")]
pub use render::json_ui::{JsonUiRenderer, RenderMode, VisualContext};
```

Downstream crates that need `JsonUiRenderer` (ferro-mcp) must enable the feature:

```toml
# ferro-mcp/Cargo.toml
ferro-projections = { path = "../ferro-projections", version = "0.2", features = ["visual"] }
```

### Pattern 5: Minimum ferro-mcp fix (D-14)

All 6 `RenderContext { ... }` struct literal construction sites in `ferro-mcp/src/tools/render_projection.rs` switch to `VisualContext`. The `RenderMode` import path changes from `ferro_projections::RenderMode` to `ferro_projections::RenderMode` (still re-exported, just from a different module internally). Import line update:

```rust
// Before:
use ferro_projections::{JsonUiRenderer, RenderContext, RenderMode, Renderer, ...};

// After:
use ferro_projections::{JsonUiRenderer, VisualContext, RenderMode, Renderer, ...};
// All RenderContext { ... } sites become VisualContext { ... }
```

### Anti-patterns to Avoid

- **Do not make `Renderer` object-safe via `where Self::Output: ...` bounds.** The trait does not need to be `dyn Renderer` (D-02). Adding `Sized` constraints or boxing would complicate the design for no benefit.
- **Do not gate `BaseContext` behind the `visual` feature.** `BaseContext` is modality-agnostic and must be available unconditionally.
- **Do not remove `RenderMode` from public exports prematurely.** `ferro-mcp` references `RenderMode` directly. It must remain exported (from the `visual` feature path).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead |
|---------|-------------|-------------|
| Feature-flag conditional compilation | Custom proc macros | `#[cfg(feature = "...")]` — standard Rust |
| Context default values | Manual builder setup | `#[derive(Default)]` with `impl Default` override for `VisualContext` |

---

## Common Pitfalls

### Pitfall 1: TemplateRenderer tests reference `RenderContext` by name

**What goes wrong:** `template.rs` tests call `RenderContext::default()` and `TemplateRenderer` implements `Renderer` with the old signature. After the refactor, this becomes `BaseContext::default()`.

**How to avoid:** Update the `use super::{..., RenderContext, Renderer}` import in `template.rs` to `use super::{..., BaseContext, Renderer}` and replace `RenderContext::default()` in the test helper with `BaseContext::default()`.

**Also:** The doc comment example in `template.rs` (lines 44–62) references `RenderContext` and must be updated for `cargo test --doc` to pass.

### Pitfall 2: lib.rs re-exports the old types without updating

**What goes wrong:** `lib.rs` currently re-exports `RenderContext, RenderMode, Renderer` at line 21. After the refactor, `RenderContext` no longer exists, `RenderMode` moves behind the `visual` feature, and `BaseContext`/`VisualContext` are the new exports.

**How to avoid:** Update `lib.rs` re-exports in the same commit as `render/mod.rs`. Old exports of a removed type cause compile errors on the first `cargo check`.

### Pitfall 3: `ferro-mcp` Cargo.toml does not enable `visual` feature

**What goes wrong:** After `ferro-projections` gates `json_ui` behind `visual`, `ferro-mcp` will fail to find `JsonUiRenderer` even though it's listed as a dependency.

**How to avoid:** Add `features = ["visual"]` to `ferro-mcp/Cargo.toml`'s `ferro-projections` entry as part of the D-14 minimum fix.

### Pitfall 4: `render_context_default` test in `render/mod.rs` breaks

**What goes wrong:** The existing test at `render/mod.rs:116` is named `render_context_default` and tests `RenderContext::default()` fields including `mode` and `templates`. After the split, `RenderContext` is gone.

**How to avoid:** Replace with a `base_context_default` test that checks `intent_index == 0` and `current_state.is_none()`. Add a separate `visual_context_default` test (in `json_ui.rs`) verifying `mode == RenderMode::Display` and `templates.is_none()`.

### Pitfall 5: `render_mode_serde_round_trip` test in `render/mod.rs` breaks

**What goes wrong:** `RenderMode` moves to `json_ui.rs`. The tests in `render/mod.rs` that test `RenderMode` serde will fail to compile because `RenderMode` is no longer in scope.

**How to avoid:** Move the `render_mode_serde_round_trip` and `render_mode_display_serializes_snake_case` tests to `render/json_ui.rs` alongside the type.

### Pitfall 6: `json_ui.rs` imports `RenderContext` and `RenderMode` from `super`

**What goes wrong:** Line 17 of `json_ui.rs` does `use super::{..., RenderContext, RenderMode, Renderer}`. After the split, `RenderContext` no longer exists in `super`. If this import isn't updated the file won't compile.

**How to avoid:** Update the import to `use super::{..., Renderer}` only. `VisualContext` and `RenderMode` are defined locally in `json_ui.rs`.

---

## Code Examples

### Verified: Current RenderContext construction in ferro-mcp (6 sites)

```rust
// Source: ferro-mcp/src/tools/render_projection.rs:72-77
let ctx = RenderContext {
    intent_index: idx,
    current_state: None,
    mode: render_mode,
    templates: None,
};
```

After Phase 133, all 6 sites become:

```rust
let ctx = VisualContext {
    intent_index: idx,
    current_state: None,
    mode: render_mode,
    templates: None,
};
```

### Verified: TemplateRenderer ignores mode and templates

```rust
// Source: ferro-projections/src/render/template.rs:66-72
impl Renderer for TemplateRenderer {
    fn render(&self, service: &ServiceDef, _intents: &[IntentScore], _ctx: &RenderContext) -> Result<Value, Error> {
        // ctx.intent_index and ctx.current_state are never read either
    }
}
```

`TemplateRenderer` already ignores all context. With `BaseContext` it reads zero fields — that's correct per D-07.

### Verified: JsonUiRenderer reads ctx.intent_index, ctx.mode, ctx.templates, ctx.current_state

```rust
// Source: ferro-projections/src/render/json_ui.rs:71-83
let intent_score = intents.get(ctx.intent_index)...;
let template_override = ctx.templates.as_ref().and_then(|t| get_template_for_intent(t, &intent_score.intent, &ctx.mode));
```

All four fields accessed in `JsonUiRenderer::render` are visual concerns (`intent_index`, `current_state` move to `VisualContext` from `BaseContext`, `mode` and `templates` are visual-only). `current_state` is also read in `render_process` and `render_process_input` via `ctx.current_state`.

---

## Environment Availability

Step 2.6: SKIPPED (no external dependencies — this is a pure Rust refactor within the existing workspace).

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none (workspace default) |
| Quick run command | `cargo test -p ferro-projections --all-features` |
| Full suite command | `cargo test --all-features` |

### Phase Verification Gates

| Check | Command | Expected |
|-------|---------|----------|
| Format | `cargo fmt --all -- --check` | no diffs |
| Lint | `cargo clippy --all --all-targets -- -D warnings` | zero warnings |
| Tests | `cargo test --all-features` | all pass |
| Trait inversion confirmed | `grep -r "ferro-theme" ferro-projections/Cargo.toml` | only under `[dependencies]` with `optional = true` |
| Dependency check | `cargo tree -p ferro-projections --no-dedupe` | `ferro-theme` absent from non-visual tree |

### Wave 0 Gaps

None — the existing test infrastructure covers the affected code. Tests will be updated in place (renamed, not newly created). No new test files required.

---

## Sources

### Primary (HIGH confidence — direct code inspection)
- `ferro-projections/src/render/mod.rs` — full source, lines 1-180
- `ferro-projections/src/render/template.rs` — full source, lines 1-289
- `ferro-projections/src/render/json_ui.rs` — lines 1-150 (trait impl and first half of render dispatch)
- `ferro-projections/src/lib.rs` — re-export list
- `ferro-projections/Cargo.toml` — dependency list
- `ferro-mcp/src/tools/render_projection.rs` — lines 1-109 (all RenderContext construction sites)
- `ferro-mcp/Cargo.toml` — dependency list
- `ferro-theme/src/template.rs` — ThemeTemplates, IntentSlotTemplate definitions
- `.planning/phases/133-generalize-renderer-trait/133-CONTEXT.md` — all locked decisions

### Secondary (MEDIUM confidence)
- `ferro-projections/CLAUDE.md` — crate boundary rules confirming renderers should not add deps

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new deps, all existing types verified in source
- Architecture patterns: HIGH — derived directly from locked decisions + actual code inspection
- Pitfalls: HIGH — identified by tracing each import and test that references the types being moved/renamed

**Research date:** 2026-04-14
**Valid until:** N/A — pure internal refactor, not tied to any external API evolution
