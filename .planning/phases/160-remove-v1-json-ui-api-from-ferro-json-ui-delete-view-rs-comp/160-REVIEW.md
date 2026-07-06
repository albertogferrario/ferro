---
phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp
reviewed: 2026-05-17T05:52:03Z
depth: standard
files_reviewed: 17
files_reviewed_list:
  - ferro-json-ui/README.md
  - ferro-json-ui/src/layout.rs
  - ferro-json-ui/src/projection/builder.rs
  - ferro-json-ui/src/render/atoms.rs
  - ferro-json-ui/src/render/containers.rs
  - ferro-json-ui/src/render/data.rs
  - ferro-json-ui/src/render/form.rs
  - ferro-json-ui/src/render/mod.rs
  - ferro-mcp/src/tools/application_info.rs
  - ferro-mcp/src/tools/code_templates.rs
  - ferro-mcp/src/tools/json_ui_inspect.rs
  - docs/protocol/src/architecture.md
  - docs/protocol/src/rendering.md
  - docs/protocol/src/terminology.md
  - docs/src/features/projections.md
  - docs/src/reference/cli.md
findings:
  critical: 0
  warning: 2
  info: 2
  total: 4
status: issues_found
---

# Phase 160: Code Review Report

**Reviewed:** 2026-05-17T05:52:03Z
**Depth:** standard
**Files Reviewed:** 17
**Status:** issues_found

## Summary

Phase 160 is a deletion / cleanup phase. The Rust changes that carry behavior — `scan_json_ui_specs` rewrite in `ferro-mcp/src/tools/application_info.rs` (with four new unit tests), the deletion of `migration_v1_to_v2_templates()` in `ferro-mcp/src/tools/code_templates.rs`, and the test-fixture rename in `ferro-mcp/src/tools/json_ui_inspect.rs` — are clean. The rewritten `scan_json_ui_specs` correctly enumerates `*.json` files in `src/views/`, returns a typed `JsonUiSpecsStatus`, and exercises every branch in its new tests (no dir, empty dir, mixed dir, multi-file dir). The `code_templates.rs` deletion left no dangling references (no callers, no leftover `migration` category templates, `json_view_templates` still composes cleanly). The fixture rename in `json_ui_inspect.rs` is purely cosmetic and the surrounding test still asserts the right behavior (non-JSON files are ignored).

The doc-comment rewrites across `ferro-json-ui/src/render/{atoms,containers,data,form,mod}.rs`, `projection/builder.rs`, and `layout.rs` are well-scoped: the `// Phase 115/116 …`, `Port of v1 …`, and `render.rs:NN-MM` provenance fragments are gone; the present-tense voice in module heads and per-function docs reads like neutral architectural documentation. The `MAX_NESTING_DEPTH = 3` doc-comment literal in `render/mod.rs` is stale (now `5`) — see WR-01.

The `ferro-json-ui/README.md` rewrite is publish-ready for Phase 161: imports (`ferro::{handler, JsonUi, Request, Response}`, `ferro_json_ui::{Spec, Element}`), `JsonUi::render_file(path, data)` signature, and `Spec::builder().title(..).element(..).build()` chain all match the current public API; the "41 built-in components" claim matches `BUILTIN_TYPES.len() == 41` (pinned by a runtime test in `render/mod.rs`).

The protocol docs (`architecture.md`, `rendering.md`, `terminology.md`) cross-check cleanly against source — `FieldMeaning` 18-variant count, `DataType` 10-variant count, `RenderMode` two variants, `NavigationHint` five values, `Cardinality` four values, and the `Sensitive → Input::Password (no data_path)` mapping all line up with `ferro-projections/src/field.rs` and `ferro-json-ui/src/projection/component_map.rs`.

The framework-level docs page `docs/src/features/projections.md` has two factual-accuracy regressions against the actual public API (WR-02). The Quick Start example was updated in Phase 160-07, but the Rendering, Complete Example, and Reference sections were not — they still reference a non-existent `RenderContext` type and a `DataType::Text` variant that does not exist, plus six `FieldMeaning` variants (`Description`, `Image`, `Timestamp`, `Count`, `Location`, `Generic`) that do not exist in the enum. This contradicts the corrected Quick Start in the same file.

## Warnings

### WR-01: Stale `MAX_NESTING_DEPTH = 3` literal in render walker doc-comment

**File:** `ferro-json-ui/src/render/mod.rs:134`
**Issue:** The rewritten doc-comment on the depth tripwire says "Parse-time depth is capped at `MAX_NESTING_DEPTH = 3`", but the actual constant is `5` (`ferro-json-ui/src/spec.rs:37: pub const MAX_NESTING_DEPTH: usize = 5;`). The phase 160-01 commit (`c25e52a2`) reworded the surrounding prose but preserved the stale `= 3` literal that was carried over from when the cap was originally set in Phase 115. Other doc-comments in `spec.rs` (e.g. lines 1112, 1770, 1794, 1814) and the runtime check at `spec.rs:937` all use `5`. This is exactly the class of stale literal that survives prose rewrites — the present-tense framing makes the wrong number look more authoritative, not less.
**Fix:**
```rust
// (1) Depth tripwire. Parse-time depth is capped at `MAX_NESTING_DEPTH = 5`;
// this fires only for hand-mutated Specs that bypassed `Spec::from_json`.
if depth > MAX_NESTING_DEPTH + 1 {
```

### WR-02: `docs/src/features/projections.md` references non-existent `RenderContext`, `DataType::Text`, and six `FieldMeaning` variants

**File:** `docs/src/features/projections.md:104, 114-123, 164, 178-218, 256-263, 282-289`
**Issue:** Phase 160-07 corrected the Quick Start example to use the real public API (`VisualContext`, `ferro-json-ui/v2`, `spec.schema`), but did not propagate the fix into the rest of the file. As a result the page is internally inconsistent and several code snippets and tables document API surface that does not exist:

1. **`RenderContext` does not exist as a public type.** The actual struct is `VisualContext` (`ferro-json-ui/src/projection/mod.rs:45`), re-exported as `ferro_json_ui::VisualContext` and `ferro::VisualContext` (no `RenderContext` alias anywhere in `ferro-projections`, `ferro-json-ui`, or `framework/src/lib.rs`). The page still uses `RenderContext` in:
   - line 164: "If the derived intent is not what you want, use `IntentHint`" — fine, but
   - lines 178-218: "Use it with a `RenderContext` to control …", `use ferro::{JsonUiRenderer, RenderContext, RenderMode, Renderer};`, two struct literals `let display_ctx = RenderContext { … }` / `let input_ctx = RenderContext { … }`, and a "RenderContext fields" table.
   - lines 256-263: "let ctx = RenderContext { … }" in the Complete Example.
   - line 288: "`RenderContext` | Render parameters: intent index, current state, mode, template overrides" in the Reference table.
   None of these compile — `RenderContext` is not in scope. The corrected Quick Start at line 39 uses `VisualContext::default()` correctly.

2. **`DataType::Text` does not exist.** The `DataType` enum is `String, Integer, Float, Boolean, DateTime, Date, Json, Binary, Uuid, Enum` (ten variants — `ferro-projections/src/field.rs:10`). The page's DataType table at lines 96-105 lists `DataType::Text` as a separate variant for "Long-form prose". Use `DataType::String`; there is no separate long-text type.

3. **Six `FieldMeaning` variants do not exist.** The actual enum has 18 known variants plus `Custom(String)` — `Identifier, ForeignKey, EntityName, Email, Phone, Url, ImageUrl, Money, Percentage, Quantity, Status, Category, Boolean, FreeText, CreatedAt, UpdatedAt, DateTime, Sensitive` (`ferro-projections/src/field.rs:35-56`, also pinned by the `eighteen known variants` line in `docs/protocol/src/terminology.md:44-46`). The page's FieldMeaning table at lines 109-123 references:
   - `FieldMeaning::Description` — does not exist; closest is `FreeText`
   - `FieldMeaning::Image` — does not exist; the variant is `ImageUrl`
   - `FieldMeaning::Timestamp` — does not exist; use `CreatedAt` / `UpdatedAt` / `DateTime`
   - `FieldMeaning::Count` — does not exist; closest is `Quantity`
   - `FieldMeaning::Location` — does not exist
   - `FieldMeaning::Generic` — does not exist; use `Custom(String)`

These are not v1-framing leftovers per se, but they are exactly the "leftover stale facts about the actual `Spec` struct shape" the phase context calls out, and they undermine the Phase 160-07 fix — a reader who reads past Quick Start gets a contradicted, non-compiling API surface.

**Fix:** Three-part rewrite of `docs/src/features/projections.md`:

1. Replace every `RenderContext` occurrence with `VisualContext` (imports, struct literals, prose, and the Reference-table row). Example for line 178-202:
```rust
use ferro::{JsonUiRenderer, VisualContext, RenderMode, Renderer};

// Display mode: read-only view of data
let display_ctx = VisualContext {
    intent_index: 0,              // use primary intent
    current_state: None,          // no workflow state active
    mode: RenderMode::Display,    // read-only layout
    templates: None,              // use default layouts
};
```

2. Drop `DataType::Text` from the table; keep `DataType::String` as the entry for both short and long text.

3. Replace the FieldMeaning table with one keyed on the real enum:
```markdown
| `FieldMeaning::Identifier` | Primary key or unique ID |
| `FieldMeaning::EntityName` | Display name of the record |
| `FieldMeaning::Money` | Monetary amount |
| `FieldMeaning::FreeText` | Long descriptive text |
| `FieldMeaning::Status` | Current state or lifecycle value |
| `FieldMeaning::Email` | Email address |
| `FieldMeaning::Phone` | Phone number |
| `FieldMeaning::Url` | Web URL |
| `FieldMeaning::ImageUrl` | Image URL or path |
| `FieldMeaning::CreatedAt` / `FieldMeaning::UpdatedAt` / `FieldMeaning::DateTime` | Timestamp fields |
| `FieldMeaning::Quantity` | Aggregate count or numeric quantity |
| `FieldMeaning::Custom(String)` | Domain-specific meaning not covered above |
```

A scoped sweep of the file plus a `cargo doc --no-deps` or a `mdbook test` of the snippets would catch the rest in one pass.

## Info

### IN-01: `code_templates.rs` doc-comment still lists `migration` as a valid category

**File:** `ferro-mcp/src/tools/code_templates.rs:34`
**Issue:** The `execute(category: Option<&str>)` doc-comment lists `migration` among the accepted category strings: `"handler, model, migration, middleware, validation, json_view, rate_limiting, broadcasting, api"`. The `migration_templates()` helper still exists at line ~57 and emits genuine database migration templates (sea-orm-migration), not the v1→v2 JSON-UI migration that was removed — so the entry is technically correct. Flagging for awareness only: if a future reader confuses "migration" (database) with "migration" (v1→v2 templates), they may file a regression. No action required unless you want to disambiguate (e.g. "migration (database schema)").
**Fix:** Optional: tighten the doc-comment to "`migration` (database schema migrations)" to make the category unambiguous now that the v1→v2 migration category is gone.

### IN-02: Test plugin type name still carries "Phase116" branding

**File:** `ferro-json-ui/src/render/mod.rs:413, 449`
**Issue:** Two test-only plugin component types are named `FerroPhase116PluginDispatchTest` and `FerroPhase116AssetCollectTestPlugin`. The `c25e52a2` commit message says it dropped "Phase 115/116 phase references from test fixture docs" — and indeed the doc comments are clean — but the identifiers themselves still carry the phase number. These are test-internal identifiers (never exposed in compiled binaries or external surface), so this is cosmetic. Flagging because the per-phase narrative explicitly removed similar provenance and the rename in `json_ui_inspect.rs` (`old_view.rs` → `stale_artifact.rs`) shows the appetite for neutral identifiers.
**Fix:** Optional rename to phase-neutral names, e.g. `BuiltinDispatchTestPlugin` and `AssetCollectionTestPlugin`. No behavioral impact.

---

_Reviewed: 2026-05-17T05:52:03Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
