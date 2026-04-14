# Phase 134: Relocate Renderers to Output Crates - Research

**Researched:** 2026-04-15
**Domain:** Rust workspace crate reorganization — moving modules across crate boundaries, Cargo feature flags
**Confidence:** HIGH

## Summary

This phase is a pure relocation: move `JsonUiRenderer`, `VisualContext`, `RenderMode`, `field_map.rs`, and `relationship_map.rs` from `ferro-projections/src/render/` into a new `ferro-json-ui/src/projection/` module directory. No API surface changes. No behavior changes. All 62 tests in `json_ui.rs` and 29 tests in `field_map.rs` travel with the code and must pass after relocation.

The primary technical constraint is the `is_system_field` function: currently `pub(crate)` in `ferro-projections/src/render/mod.rs`, it is called 13 times in `json_ui.rs`. After relocation to a different crate, `pub(crate)` visibility no longer reaches `ferro-json-ui`. The function must be made `pub` in `ferro-projections` (it is already safe to expose — modality-agnostic predicate over `FieldMeaning`). `field_display_name` is already `pub` and causes no issue.

The second constraint is the Cargo dependency graph: `ferro-json-ui` currently has no dependency on `ferro-projections` or `ferro-theme`. Both must be added as optional dependencies behind a `projections` feature flag. `ferro-mcp` already depends on both crates; it needs the `projections` feature enabled on its `ferro-json-ui` entry.

**Primary recommendation:** Execute as three sequential steps: (1) relocate files + fix cross-crate visibility, (2) wire Cargo feature flags, (3) update downstream import paths. Compile-check after each step.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Create `ferro-json-ui/src/projection/` module directory. Do NOT merge with existing `render.rs` (HTML rendering of component trees). Projection rendering is a separate concern.
- **D-02:** Module structure: `projection/mod.rs` (JsonUiRenderer impl + VisualContext + RenderMode), `projection/field_map.rs`, `projection/relationship_map.rs`.
- **D-03:** Re-export `JsonUiRenderer`, `VisualContext`, `RenderMode` from `ferro-json-ui/src/lib.rs` behind `#[cfg(feature = "projections")]`.
- **D-04:** ferro-json-ui adds `ferro-projections` as optional dependency behind `projections` feature flag.
- **D-05:** ferro-json-ui also needs `ferro-theme` behind the `projections` feature flag.
- **D-06:** Clean break — remove all visual re-exports from `ferro-projections/src/lib.rs`. No deprecated re-exports.
- **D-07:** Remove the `visual` feature flag and `ferro-theme` optional dependency from `ferro-projections/Cargo.toml` entirely.
- **D-08:** Delete `ferro-projections/src/render/json_ui.rs`, `ferro-projections/src/render/field_map.rs`, `ferro-projections/src/render/relationship_map.rs` after relocation.
- **D-09:** ferro-mcp imports `JsonUiRenderer`, `RenderMode`, `VisualContext` from `ferro_json_ui` (not `ferro_projections`). Enable `projections` feature on ferro-mcp's ferro-json-ui dep.
- **D-10:** ferro-cli does not import visual types — no CLI changes required for visual types.
- **D-11:** Update doc comment example in relocated `projection/mod.rs` to use `ferro_json_ui::` import paths.
- **D-12:** Feature flag name: `projections`.
- **D-13:** ferro-json-ui Cargo additions:
  ```toml
  [features]
  projections = ["dep:ferro-projections", "dep:ferro-theme"]

  [dependencies]
  ferro-projections = { path = "../ferro-projections", version = "0.2", optional = true }
  ferro-theme = { path = "../ferro-theme", version = "0.2", optional = true }
  ```

### Claude's Discretion

- Internal module visibility (`pub` vs `pub(crate)`) for helper functions like `is_system_field` and `field_display_name` after relocation.
- Whether `field_map.rs` and `relationship_map.rs` tests move with the files or get rewritten.
- Whether to keep `render::field_map` and `render::relationship_map` as `pub mod` in ferro-projections `render/mod.rs` or remove them.

### Deferred Ideas (OUT OF SCOPE)

- ServiceDef derivation from models → Phase 135
- Crate consolidation audit → CONC-04 in v13.0
- WhatsApp renderer in ferro-whatsapp behind `projections` feature → v14.0+
</user_constraints>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Cargo features | workspace 0.2.2 | Optional dependency gating | Rust standard mechanism for conditional compilation |
| serde + serde_json | workspace | JSON serialization in relocated renderer | Already used throughout ferro-json-ui |
| ferro-theme | workspace 0.2 | `ThemeTemplates` used by `VisualContext` | Already a dep of ferro-projections; moves to ferro-json-ui |

No new libraries are introduced. This is a relocation, not a feature addition.

**Verification:** All packages are already in the workspace — no `npm install` equivalent needed. Cargo feature additions only.

## Architecture Patterns

### Recommended Project Structure (post-relocation)

```
ferro-projections/src/render/
├── mod.rs              # Renderer trait, BaseContext, field_display_name, is_system_field (now pub)
└── template.rs         # TemplateRenderer (stays)

ferro-json-ui/src/
├── projection/
│   ├── mod.rs          # JsonUiRenderer impl, VisualContext, RenderMode (was json_ui.rs)
│   ├── field_map.rs    # FieldMeaning → JSON-UI component mapping (was render/field_map.rs)
│   └── relationship_map.rs  # NavigationHint → component mapping (was render/relationship_map.rs)
└── lib.rs              # Adds: #[cfg(feature = "projections")] pub mod projection; + re-exports
```

### Pattern 1: Optional Cargo Feature for Cross-Crate Renderer

**What:** Gate a module behind a Cargo feature so the crate remains usable without pulling in renderer dependencies.

**When to use:** When an output crate needs types from a schema crate, but users who only need the schema types should not pay the dependency cost.

**Example (verified from ferro-cli/Cargo.toml, same pattern):**
```toml
# ferro-json-ui/Cargo.toml
[features]
projections = ["dep:ferro-projections", "dep:ferro-theme"]

[dependencies]
ferro-projections = { path = "../ferro-projections", version = "0.2", optional = true }
ferro-theme = { path = "../ferro-theme", version = "0.2", optional = true }
```

```rust
// ferro-json-ui/src/lib.rs
#[cfg(feature = "projections")]
pub mod projection;

#[cfg(feature = "projections")]
pub use projection::{JsonUiRenderer, RenderMode, VisualContext};
```

### Pattern 2: Cross-Crate Import Path Update in feature-gated module

**What:** Inside a `#[cfg(feature = "projections")]` module, reference types from the optional dep using the dep crate name.

**Example:**
```rust
// ferro-json-ui/src/projection/mod.rs
use ferro_projections::render::{field_display_name, is_system_field, Renderer};
use ferro_projections::error::Error;
use ferro_projections::field::FieldMeaning;
use ferro_projections::intent::{Intent, IntentScore};
use ferro_projections::relationship::NavigationHint;
use ferro_projections::service::ServiceDef;
use ferro_theme::{IntentSlotTemplate, ThemeTemplates};
```

Note: The `crate::` references in `json_ui.rs` become `ferro_projections::` references. The `super::` references to `field_map` and `relationship_map` become module-local `super::` references (they stay siblings in `projection/`).

### Pattern 3: Making `is_system_field` pub

**What:** `is_system_field` is currently `pub(crate)` in `ferro-projections/src/render/mod.rs`. After relocation, `json_ui.rs` (now `projection/mod.rs` in ferro-json-ui) calls it cross-crate. It must become `pub`.

**Why acceptable:** `is_system_field` is a pure, side-effect-free predicate over `FieldMeaning`. Making it `pub` does not expose any mutable state or internal complexity. It correctly belongs to the modality-agnostic render module, and external renderers legitimately need it.

**Change:**
```rust
// ferro-projections/src/render/mod.rs
// Before:
pub(crate) fn is_system_field(meaning: &FieldMeaning) -> bool { ... }
// After:
pub fn is_system_field(meaning: &FieldMeaning) -> bool { ... }
```

Also export from `ferro-projections/src/lib.rs`:
```rust
pub use render::is_system_field;
```

Or import directly via `ferro_projections::render::is_system_field` from the `projection/mod.rs` module. Either works; direct module path avoids adding to the public API surface of the crate root.

### Anti-Patterns to Avoid

- **Duplicating `is_system_field` logic in ferro-json-ui:** Creates divergence risk between renderers. The function is defined once, lives in ferro-projections, is called cross-crate.
- **Deprecated re-exports from ferro-projections:** D-06 mandates a clean break. No `#[deprecated]` shims.
- **Merging projection/ into render.rs:** D-01 explicitly forbids this. The render.rs in ferro-json-ui handles HTML rendering of component trees — a different concern.
- **Making `ferro-projections` depend on `ferro-json-ui`:** Creates a circular dependency. The dependency arrow is ferro-json-ui → ferro-projections, never the reverse.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Feature-gated optional deps | Custom build scripts or runtime detection | Cargo `[features]` + `optional = true` | Cargo handles all conditional compilation cleanly at zero cost |
| Cross-crate visibility | Duplicating functions | `pub` + cross-crate import | Duplication creates drift between renderers |
| Import path updates | Macro magic | Direct text edit of `use` statements | Simple, mechanical, auditable |

## Common Pitfalls

### Pitfall 1: `crate::` vs `ferro_projections::` in relocated code

**What goes wrong:** `json_ui.rs` uses `crate::field::FieldMeaning`, `crate::intent::Intent`, etc. After move to ferro-json-ui, `crate::` refers to ferro-json-ui which has none of these types. The file will not compile until every `crate::` reference is rewritten to `ferro_projections::`.

**Why it happens:** `crate::` is always relative to the crate containing the file, not the originating crate.

**How to avoid:** Grep for all `crate::` occurrences in the three relocated files before declaring the move complete. Replace each with `ferro_projections::`.

**Warning signs:** `error[E0433]: failed to resolve: use of undeclared crate or module` — means a `crate::` reference was missed.

### Pitfall 2: `super::` references in field_map.rs and relationship_map.rs

**What goes wrong:** Both files use `super::field_display_name` and `super::is_system_field` to reach functions in the parent `render/mod.rs`. After relocation to `projection/`, the parent is `projection/mod.rs` which no longer has these functions — they live in `ferro-projections`.

**Why it happens:** `super::` is also crate-relative, but cross-module. After move, `super::` in `field_map.rs` refers to `projection/mod.rs`, not `render/mod.rs`.

**How to avoid:** In `field_map.rs`, change `use super::field_display_name;` to `use ferro_projections::render::field_display_name;`. Same for `relationship_map.rs`. Alternatively, re-export them from `projection/mod.rs` and keep `super::` — valid if the relocated `projection/mod.rs` re-exports `field_display_name` from its own imports.

**Warning signs:** `error[E0432]: unresolved import` on `super::` lines.

### Pitfall 3: Tests in json_ui.rs use `crate::` types directly

**What goes wrong:** The 62 tests in `json_ui.rs` are `#[cfg(test)]` blocks using `use crate::field::*`, `use crate::service::*`, etc. After relocation, these also need to reference `ferro_projections::`.

**Why it happens:** Same `crate::` → `ferro_projections::` mechanical renaming, but in test code which is easy to miss.

**How to avoid:** Treat test code identically to production code for the import renaming pass. Do not skip `#[cfg(test)]` blocks.

**Warning signs:** `cargo test --features projections` fails in ferro-json-ui with unresolved imports inside test modules.

### Pitfall 4: ferro-projections default feature includes "visual"

**What goes wrong:** `ferro-projections/Cargo.toml` has `default = ["visual"]`. After removing the `visual` feature (D-07), any downstream that doesn't explicitly opt-in may get compile errors if they were relying on the default.

**How to avoid:**
1. Check all downstream Cargo.toml files for `ferro-projections` dependency entries.
2. ferro-mcp currently uses `features = ["visual"]` — this must be removed after the feature is deleted.
3. ferro-cli: check if it references `visual` feature anywhere.

**Warning signs:** `error[E0635]: unknown feature "visual"` in any downstream Cargo.toml.

### Pitfall 5: Missing `projections` feature on ferro-mcp's ferro-json-ui dep

**What goes wrong:** ferro-mcp imports `JsonUiRenderer`, `RenderMode`, `VisualContext` from `ferro_json_ui`. If the `projections` feature is not enabled in ferro-mcp's Cargo.toml, these types won't be compiled into ferro-json-ui and the import fails.

**How to avoid:** Update `ferro-mcp/Cargo.toml`:
```toml
ferro-json-ui = { path = "../ferro-json-ui", version = "0.2", features = ["projections"] }
```
And remove `features = ["visual"]` from the `ferro-projections` entry.

### Pitfall 6: ferro-projections `render/mod.rs` still declares `pub mod field_map` and `pub mod relationship_map`

**What goes wrong:** After deleting the files (D-08), if `render/mod.rs` still has `pub mod field_map` and `pub mod relationship_map` declarations, the crate will not compile (`file not found for module`).

**How to avoid:** Remove those two `pub mod` lines from `render/mod.rs` as part of the same commit that deletes the files. Also remove `#[cfg(feature = "visual")] pub mod json_ui`.

## Code Examples

### Relocated file import header (projection/mod.rs)

```rust
// Source: derived from ferro-projections/src/render/json_ui.rs after path rewrite

use ferro_projections::error::Error;
use ferro_projections::field::FieldMeaning;
use ferro_projections::intent::{Intent, IntentScore};
use ferro_projections::relationship::NavigationHint;
use ferro_projections::render::{field_display_name, is_system_field, Renderer};
use ferro_projections::service::ServiceDef;
use ferro_theme::{IntentSlotTemplate, ThemeTemplates};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::field_map::{field_to_column, field_to_display, field_to_input};
use super::relationship_map::relationship_to_component;
```

### ferro-projections/src/render/mod.rs cleanup

```rust
// Remove these lines:
pub mod field_map;
#[cfg(feature = "visual")]
pub mod json_ui;
pub mod relationship_map;

// Add pub to is_system_field (was pub(crate)):
pub fn is_system_field(meaning: &FieldMeaning) -> bool { ... }

// Keep these:
pub mod template;
pub use ...  // Renderer, BaseContext, field_display_name
```

### ferro-projections/src/lib.rs cleanup

```rust
// Remove:
#[cfg(feature = "visual")]
pub use render::json_ui::{JsonUiRenderer, RenderMode, VisualContext};

// Keep (no change needed):
pub use render::template::TemplateRenderer;
pub use render::{BaseContext, Renderer};
// Optionally add:
pub use render::is_system_field;  // now pub, so external renderers can use it
```

### ferro-json-ui/src/lib.rs additions

```rust
// Add at bottom (feature-gated):
#[cfg(feature = "projections")]
pub mod projection;

#[cfg(feature = "projections")]
pub use projection::{JsonUiRenderer, RenderMode, VisualContext};
```

### ferro-mcp render_projection.rs import update

```rust
// Before:
use ferro_projections::{
    derive_intents, ActionDef, Cardinality, DataType, FieldMeaning, GuardDef, InputDef, IntentHint,
    JsonUiRenderer, RenderMode, Renderer, ServiceDef, StateDef, StateMachine, Transition,
    VisualContext,
};

// After:
use ferro_json_ui::{JsonUiRenderer, RenderMode, VisualContext};
use ferro_projections::{
    derive_intents, ActionDef, Cardinality, DataType, FieldMeaning, GuardDef, InputDef, IntentHint,
    Renderer, ServiceDef, StateDef, StateMachine, Transition,
};
```

## Runtime State Inventory

Not applicable. This is a code relocation within a single workspace. No stored data, live service config, OS-registered state, secrets, or build artifacts reference these module paths at runtime.

## Environment Availability

Step 2.6: SKIPPED (no external dependencies — pure Rust workspace refactor, all tools already available in dev environment).

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none (workspace Cargo.toml) |
| Quick run command | `cargo test -p ferro-json-ui --features projections` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

This phase has no formal requirement IDs. The exit criterion is: all existing tests pass after relocation.

| Exit Criterion | Behavior | Test Type | Automated Command | Status |
|----------------|----------|-----------|-------------------|--------|
| JsonUiRenderer tests pass in new location | 62 tests from json_ui.rs run under ferro-json-ui | unit | `cargo test -p ferro-json-ui --features projections` | Travels with code |
| field_map tests pass in new location | 29 tests from field_map.rs run under ferro-json-ui | unit | `cargo test -p ferro-json-ui --features projections` | Travels with code |
| relationship_map tests pass in new location | Tests from relationship_map.rs run under ferro-json-ui | unit | `cargo test -p ferro-json-ui --features projections` | Travels with code |
| ferro-projections still compiles | No remaining references to removed modules | compile | `cargo build -p ferro-projections` | Verified by clean build |
| ferro-mcp still compiles and tests pass | render_projection.rs imports from new location | integration | `cargo test -p ferro-mcp` | Import path update |
| Workspace clean | No warnings under -D warnings | lint | `cargo clippy --all --all-targets -- -D warnings` | Required pre-commit |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-json-ui --features projections && cargo test -p ferro-projections && cargo test -p ferro-mcp`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`

### Wave 0 Gaps

None — no new test infrastructure needed. All tests travel with the relocated files. The `projection/` module directory is new but test harness is unchanged (cargo test, no external framework).

## Open Questions

1. **`field_display_name` and `is_system_field` crate root re-export**
   - What we know: `field_display_name` is already `pub` and accessible via `ferro_projections::render::field_display_name`. `is_system_field` needs to become `pub`.
   - What's unclear: Whether to also re-export `is_system_field` from the crate root (`ferro_projections::is_system_field`) for discoverability by future renderers.
   - Recommendation: Leave at render module path only (`ferro_projections::render::is_system_field`). Crate root re-export can happen in CONC-01 when the full API surface is audited. Pre-1.0, adding it later is zero cost.

2. **Tests in relocated files: `crate::` in test helper functions**
   - What we know: The `json_ui.rs` test module defines helper functions like `order_service()`, `browse_intent()` that themselves use `crate::*` types. These are inside `#[cfg(test)]` but still need the same `ferro_projections::` rewrite.
   - What's unclear: Whether any test uses ferro-json-ui types (from the same crate) that would create complexity.
   - Recommendation: During relocation, compile with `--features projections` immediately after the file move to catch all unresolved references in one pass.

## Sources

### Primary (HIGH confidence)
- Direct code inspection: `ferro-projections/src/render/mod.rs`, `json_ui.rs`, `field_map.rs`, `relationship_map.rs` — current state of all files to be relocated
- Direct code inspection: `ferro-json-ui/src/lib.rs`, `ferro-json-ui/Cargo.toml` — destination crate state
- Direct code inspection: `ferro-mcp/Cargo.toml`, `ferro-mcp/src/tools/render_projection.rs` — downstream consumer state
- `ferro-projections/Cargo.toml` — confirmed `default = ["visual"]` and `ferro-theme` as optional dep
- `134-CONTEXT.md` — all decisions D-01 through D-13 locked by user

### Secondary (MEDIUM confidence)
- Cargo documentation (training knowledge, stable since edition 2021): optional dependencies, feature flags, `dep:` prefix syntax — all stable Rust/Cargo features with no API drift risk

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new libraries, all existing workspace dependencies
- Architecture: HIGH — pattern already exists in ferro-cli (`projections` feature), direct code inspection confirms all file states
- Pitfalls: HIGH — all pitfalls derived from direct code inspection (grep confirmed `is_system_field` call sites, `crate::` usage, downstream Cargo.toml states)

**Research date:** 2026-04-15
**Valid until:** Stable until Phase 135 executes (next phase that touches ferro-projections). No external dependencies to become stale.
