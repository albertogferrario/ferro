---
phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp
fixed_at: 2026-05-17T00:00:00Z
review_path: .planning/phases/160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp/160-REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 2
status: all_fixed
---

# Phase 160: Code Review Fix Report

**Fixed at:** 2026-05-17T00:00:00Z
**Source review:** .planning/phases/160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp/160-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 2 (both Warnings)
- Fixed: 2
- Skipped: 2 (both Info findings, out of scope per `fix_scope: critical_warning`)

`status: all_fixed` reflects the in-scope set (Critical + Warning). The two
Info findings are tracked below but were intentionally not fixed in this
iteration.

## Fixed Issues

### WR-01: Stale `MAX_NESTING_DEPTH = 3` literal in render walker doc-comment

**Files modified:** `ferro-json-ui/src/render/mod.rs`
**Commit:** 160f5645
**Applied fix:** Corrected the doc-comment on `render_element`'s depth
tripwire from `MAX_NESTING_DEPTH = 3` to `MAX_NESTING_DEPTH = 5`, matching
the actual constant defined in `ferro-json-ui/src/spec.rs:37` and the
other doc-comments in `spec.rs` (lines 1112, 1770, 1794, 1814) and the
runtime check at `spec.rs:937`.

Verification:
- Tier 1: re-read the modified section, confirmed the literal is `5`.
- Tier 2: `cargo clippy -p ferro-json-ui --all-targets -- -D warnings` passed.
- `cargo fmt --all -- --check` passed.

### WR-02: `docs/src/features/projections.md` references non-existent `RenderContext`, `DataType::Text`, and six `FieldMeaning` variants

**Files modified:** `docs/src/features/projections.md`
**Commit:** 17dcbdfa
**Applied fix:** Three-part sweep of the docs page:

1. **RenderContext → VisualContext.** Replaced every occurrence (import
   list, two struct literals in the Rendering section, the struct literal
   in the Complete Example, and the Reference-table row) with the real
   public type `VisualContext` (defined in
   `ferro-json-ui/src/projection/mod.rs:45`, re-exported as
   `ferro::VisualContext`). Updated the corresponding header
   "RenderContext fields:" → "VisualContext fields:" and tightened the
   `templates` field type from `Option<...>` to `Option<ThemeTemplates>`.

2. **DataType::Text dropped.** The `DataType` enum has no `Text` variant;
   `DataType::String` covers both short and long text. Removed the
   `Text` row from the DataType table and expanded the table with the
   three missing real variants (`Json`, `Binary`, `Uuid`) so the table
   now matches the 10-variant enum.

3. **FieldMeaning table rewritten against the real enum.** The six
   non-existent variants (`Description`, `Image`, `Timestamp`, `Count`,
   `Location`, `Generic`) were replaced with the actual 18 known
   variants plus `Custom(String)` from
   `ferro-projections/src/field.rs:35-56` — `Identifier`, `ForeignKey`,
   `EntityName`, `Email`, `Phone`, `Url`, `ImageUrl`, `Money`,
   `Percentage`, `Quantity`, `Status`, `Category`, `Boolean`, `FreeText`,
   `CreatedAt`/`UpdatedAt`/`DateTime` (one row), `Sensitive`, and
   `Custom(String)`.

Additional consistency fixes folded into the same commit:
- The signal-analyzer prose at line 132 referenced `Count` as an example
  meaning; replaced with `Quantity` (the real variant).
- The Complete Example used `FieldMeaning::Timestamp` for `created_at`;
  replaced with `FieldMeaning::CreatedAt`.
- The Rendering and Complete Example snippets bound the renderer output
  to a `json` variable; renamed to `spec` because `JsonUiRenderer::render`
  returns a `Spec`, not raw JSON.

Verification:
- Tier 1: re-read each modified section, confirmed no remaining
  `RenderContext`, `DataType::Text`, or removed `FieldMeaning` variants
  (`grep -E 'RenderContext|DataType::Text|FieldMeaning::(Description|Image[^U]|Timestamp|Count|Location|Generic)'` returns no matches).
- Tier 2: `mdbook build docs` completed cleanly (HTML book written).
  `mdbook test` was not run — the example snippets are illustrative
  (reference an undefined `service_def` binding in the Rendering
  section) and did not compile in isolation before this fix either;
  this fix corrects type names against the public API, not snippet
  runnability.

## Skipped Issues

### IN-01: `code_templates.rs` doc-comment still lists `migration` as a valid category

**File:** `ferro-mcp/src/tools/code_templates.rs:34`
**Reason:** Out of scope — `fix_scope: critical_warning` excludes Info
findings. The review itself classifies this as "Flagging for awareness
only; no action required."
**Original issue:** The doc-comment lists `migration` among accepted
category strings; the `migration_templates()` helper still exists and
emits genuine database migration templates (sea-orm-migration), not the
removed v1→v2 JSON-UI migration. Optional disambiguation suggested:
"`migration` (database schema migrations)".

### IN-02: Test plugin type name still carries "Phase116" branding

**File:** `ferro-json-ui/src/render/mod.rs:413, 449`
**Reason:** Out of scope — `fix_scope: critical_warning` excludes Info
findings. The review itself classifies this as cosmetic ("No behavioral
impact").
**Original issue:** Two test-only plugin component types are named
`FerroPhase116PluginDispatchTest` and `FerroPhase116AssetCollectTestPlugin`.
Optional rename to phase-neutral names (e.g. `BuiltinDispatchTestPlugin`
and `AssetCollectionTestPlugin`) suggested for consistency with the
broader Phase 160 sweep that removed phase provenance fragments.

---

_Fixed: 2026-05-17T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
