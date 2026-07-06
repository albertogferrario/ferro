---
phase: 258-mcp-surface-docs-publish
fixed_at: 2026-07-06T18:20:48Z
review_path: .planning/phases/258-mcp-surface-docs-publish/258-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 258: Code Review Fix Report

**Fixed at:** 2026-07-06T18:20:48Z
**Source review:** .planning/phases/258-mcp-surface-docs-publish/258-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 5 (0 Critical, 5 Warning; fix_scope: critical_warning)
- Fixed: 5
- Skipped: 0

Post-publish fixes on `master` (0.2.89 already shipped) — no version bump, no push. Verification: `cargo fmt --all -- --check` clean; `cargo test -p ferro-mcp` green (313 lib tests + integration suites, `register_composition_drift_guard` passing with the hardened checks); `mdbook build docs` exits 0 after each docs edit.

## Fixed Issues

### WR-01: Builder-API doc example does not compile (wrong import path + private constructor)

**Files modified:** `docs/src/json-ui/spec-construction.md`
**Commit:** e7842bf6
**Applied fix:** Both examples (heterogeneous-construction at ~line 116 and the register builder example at ~line 145) now use the crate-root import and the public constructor: `use ferro::{Element, Spec};` + `Spec::builder()`. Verified against sources: `Spec::builder()` is public (`ferro-json-ui/src/spec.rs:265`), `SpecBuilder::new()` is private (`spec.rs:369`), and the types are re-exported at the `ferro` crate root only (`framework/src/lib.rs:85`) — `framework/src/json_ui/mod.rs` re-exports none of them. Matches what `forms.md` and the MCP `BUILDER_API` string document. Deviation from the review's literal snippet: `SpecBuilder` was dropped from the import list since the fixed code never names the type — importing it would trigger an `unused_imports` warning when copied verbatim.

### WR-02: Attribute drift guard covers only 5 of 13 REGISTER_DATA_ATTRIBUTES

**Files modified:** `ferro-mcp/src/tools/generation_context.rs`
**Commit:** 208cc3aa
**Applied fix:** Check 3 of `register_composition_drift_guard` now derives its inputs from the published `ctx.register_composition.data_attributes` array — the attribute name is parsed as the token before the first `=` or space — and asserts every one of the 13 entries appears in `ferro_json_ui::FERRO_RUNTIME_JS`. Test passes, confirming all 13 attributes exist in the assembled runtime bundle.

### WR-03: Rule-id drift guard check is vacuous by construction

**Files modified:** `ferro-mcp/src/tools/generation_context.rs`
**Commit:** 305a509e
**Applied fix:** Check 2 now asserts the four expected literal rule ids (`register-fill-viewport`, `register-grid-fill`, `register-selection-present`, `fill-viewport-layout-unknown`) are present in BOTH the rule registry (`design::rules()`) and the derived `lint_rules` set, with distinct failure messages ("registry lost rule" vs "guidance failed to derive rule"). The expected list is intentionally duplicated in the test (with a comment explaining why) rather than extracted to a shared const — a drift guard that shares its source with `execute()` would silently track renames instead of catching them.

### WR-04: Button props table omits `disable_on_submit`, which the register docs require

**Files modified:** `docs/src/json-ui/components.md`
**Commit:** a972cbf7
**Applied fix:** Added `form` (`string | null`, HTML5 `form` attribute) and `disable_on_submit` (`boolean | null`, emits `data-disable-on-submit` double-submit guard) rows to the Button props table, in struct field order. Names and types verified against `ButtonProps` (`ferro-json-ui/src/component.rs:328,334`).

### WR-05: Component Overview table missing the five components documented in this phase

**Files modified:** `docs/src/json-ui/components.md`
**Commit:** 25b7dd96
**Applied fix:** Extended the overview table's Commerce row per the review's suggested markdown: `| **Commerce / Register** | Tile, TileGrid, SelectionPanel, FilterTabs, QuantityStepper, Numpad |`. Grepped docs for other references to the old `**Commerce**` label — none exist. Pre-existing overview gaps (StreamText, DetailPage, MediaCardGrid, SegmentedControl, SidebarLayout) left as-is per the finding's scoping.

---

_Fixed: 2026-07-06T18:20:48Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
