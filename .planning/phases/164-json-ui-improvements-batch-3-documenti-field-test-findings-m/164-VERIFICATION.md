---
phase: 164-json-ui-improvements-batch-3
verified: 2026-05-17T03:34:57Z
status: human_needed
score: 9/10
overrides_applied: 0
human_verification:
  - test: "Visual diff of CardVariant::Bordered vs CardVariant::Elevated rendering"
    expected: "Bordered renders dashboard card chrome (border + shadow-sm + p-4); Elevated renders auth page chrome (shadow-md + p-8, no border)"
    why_human: "Unit tests assert CSS class strings. Visual correctness on actual rendered output requires browser inspection against reference screenshots (V7-RUNTIME-FRICTION.md login-prod.png)."
---

# Phase 164: JSON-UI Improvements Batch 3 Verification Report

**Phase Goal:** Absorb V7-RUNTIME-FRICTION.md (F1–F10) and residual Phase 138 friction into the closing batch of the v12.0 loop; produce COMPLETED.md summarising all improvements; unblock Phase 160 (v1 deletion).
**Verified:** 2026-05-17T03:34:57Z
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | F4: MAX_NESTING_DEPTH raised from 3 to 5; depth-5 specs valid; depth-6 rejected | VERIFIED | `grep` confirms `pub const MAX_NESTING_DEPTH: usize = 5` in `ferro-json-ui/src/spec.rs`. Tests `nested_builder_accepts_depth_five` and `nested_builder_rejects_depth_six` exist. Integration test fixture `six_level_nesting.json` present. |
| 2 | F2: Codemod emits uppercase HTTP methods; regression test prevents reversion | VERIFIED | Fixture `ferro-cli/tests/fixtures/migrate_v1/in_all_verbs.rs` and test `codemod_emits_uppercase_http_methods` in `ferro-cli/tests/json_ui_migrate_v1.rs` both exist. Commit dd890ff7 verified. |
| 3 | F7: Image.data_path and DescriptionList.data_path ship with tests | VERIFIED | Both `ImageProps` (line 538) and `DescriptionListProps` (line 472) in `component.rs` carry `data_path: Option<String>`. Render tests confirmed in Plan 03 SUMMARY. |
| 4 | F9: RawHtml component added to catalog; closes "type: Plugin" runtime block | VERIFIED | `pub struct RawHtmlProps` exists in `component.rs`. Dispatch arm in `render/mod.rs`. BUILTIN_TYPES count asserted = 41. `RawHtmlProps` re-exported from `ferro-json-ui/src/lib.rs`. |
| 5 | F10: CardVariant enum (Bordered/Elevated) added to CardProps | VERIFIED | `pub enum CardVariant` exists in `component.rs`. `CardProps.variant` field with `#[serde(default)]`. Render match in `containers.rs` emits correct CSS classes. `CardVariant` re-exported from lib.rs. |
| 6 | F1: Spec.title accepts literal String or {"$data": "/path"} binding | VERIFIED | `pub enum TitleBinding` in `spec.rs`. `DataRef` struct with `#[serde(rename = "$data")]`. Framework renderer resolves at response-build time in `framework/src/json_ui/mod.rs`. `TitleBinding` and `DataRef` re-exported. |
| 7 | F3: KanbanBoard.data_path added; runtime column resolution works | VERIFIED | `KanbanBoardProps.data_path: Option<String>` at line 928 of `component.rs`. `columns` made `#[serde(default, skip_serializing_if)]`. Render branch in `containers.rs`. 4 render tests in Plan 06 SUMMARY. |
| 8 | F8: Two-stage validation — catalog validation deferred to post-expand_directives | VERIFIED | `loader.rs` downgrades to `tracing::warn` at load time (D-16). `framework/src/json_ui/mod.rs` calls `global_catalog().validate` after `expand_directives` in both `resolve()` and `resolve_with_errors()`. Integration tests `alert_variant_empty_but_gated_renders_cleanly` and `alert_variant_empty_ungated_surfaces_error_at_render` in `framework/tests/pipeline_order.rs`. |
| 9 | F5/F6: Visibility error message improved; PageHeader.actions accepts null/""/[] | VERIFIED | Hand-rolled `impl Deserialize for Visibility` in `visibility.rs` (key-presence dispatch, names all four accepted shapes). `deserialize_actions_lax` function in `component.rs`, applied via `#[serde(deserialize_with)]` on `PageHeaderProps.actions`. |
| 10 | v1 deletion audit produces zero BLOCKER rows; COMPLETED.md and V1-DELETION-AUDIT.md written | VERIFIED | `V1-DELETION-AUDIT.md` exists with "Total BLOCKER rows: 0". `COMPLETED.md` exists with 5 required sections. All v1 types (`JsonUiView`, `ComponentNode`, `PluginProps`, `Component::*` variants) absent from production source (grep confirms zero matches outside `///` doc-comments). `view.rs` file absent from `ferro-json-ui/src/`. |

**Score:** 9/10 truths verified (one requires human visual check — Truth 5 partial: CSS class strings asserted but visual rendering not confirmed)

---

### Deferred Items

No items deferred to later phases.

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-json-ui/src/spec.rs` | MAX_NESTING_DEPTH=5; TitleBinding enum; depth tests | VERIFIED | All three present and substantive |
| `ferro-json-ui/src/component.rs` | RawHtmlProps, CardVariant, data_path on Image/DescList/KanbanBoard, Visibility custom Deserialize, PageHeader.actions lax deserializer | VERIFIED | All present at correct line ranges |
| `ferro-json-ui/src/render/containers.rs` | CardVariant match; KanbanBoard data_path branch | VERIFIED | Both branches present |
| `ferro-json-ui/src/render/atoms.rs` | Image/DescList data_path resolution; RawHtml render | VERIFIED | Modified per Plan 03 SUMMARY |
| `ferro-json-ui/src/visibility.rs` | Hand-rolled Deserialize impl | VERIFIED | impl exists; dispatches by key presence |
| `ferro-json-ui/src/loader.rs` | tracing::warn instead of hard-fail on catalog errors | VERIFIED | `tracing::warn` loop at ~line 155; test `load_cached_warns_on_catalog_error_does_not_fail` present |
| `framework/src/json_ui/mod.rs` | TitleBinding match; post-expand catalog validation | VERIFIED | Both changes at lines ~50-54 and ~89; render title tests exist |
| `framework/tests/pipeline_order.rs` | Two pipeline order integration tests | VERIFIED | Both `alert_variant_empty_*` tests exist |
| `ferro-mcp/src/tools/json_ui_validate_spec.rs` | MCP validate tool (two-stage structural + catalog) | VERIFIED | File exists; registered in service.rs and mod.rs |
| `ferro-cli/tests/fixtures/migrate_v1/in_all_verbs.rs` | Five-verb codemod fixture | VERIFIED | File exists |
| `ferro-cli/tests/json_ui_migrate_v1.rs` | codemod_emits_uppercase_http_methods test | VERIFIED | Test function present |
| `ferro-json-ui/tests/fixtures/reject/six_level_nesting.json` | Depth-6 rejection fixture | VERIFIED | File exists |
| `.planning/phases/164-.../V1-DELETION-AUDIT.md` | 25-row table; 0 BLOCKER rows | VERIFIED | 25 rows (23 MIGRATED, 2 INTENTIONAL_DROP, 0 BLOCKER); "Total BLOCKER rows: 0" present |
| `.planning/phases/164-.../PLUGIN-SURFACE-AUDIT.md` | D-06 paper audit; Outcome B | VERIFIED | File exists; Outcome B confirmed; 2 gaps fixed inline |
| `.planning/phases/164-.../COMPLETED.md` | 5 required sections; Phase 160 unblocked statement | VERIFIED | All 5 sections present; "Phase 160 (v1 deletion) is UNBLOCKED" appears twice |
| `docs/src/json-ui/components.md` | CardVariant, RawHtml, data_path sections, lax actions | VERIFIED | CardVariant section at ~line 138; RawHtml at line 1340; data_path fields present (27 occurrences); lax actions section present |
| `docs/src/json-ui/spec-construction.md` | MAX_NESTING_DEPTH=5; Spec.title binding section | VERIFIED | Depth-5 documented at line 157; `## Spec.title binding` section at line 163 |
| `docs/src/json-ui/expressions.md` | $each-for-kanban example | VERIFIED | `### Example: kanban cards from a data array` at line 179 |
| `docs/src/json-ui/plugins.md` | D-06 gaps fixed (render data param; init_script semantics) | VERIFIED | Commit 63529b33 applied fixes; PLUGIN-SURFACE-AUDIT.md confirms |
| `docs/src/json-ui/migration-v1-to-v2.md` | v1→v2 cheat sheet (10-row table) | INTENTIONAL_DELETE | File was created (Plan 10), cheat sheet added (commit cf6a807f), then deliberately deleted by developer in commit 52e38ab3 with explicit rationale: "There is no public migration story; the only consumer is gestiscilo, which migrates against local-path ferro." This is a deliberate deviation, not an omission. Content is preserved in COMPLETED.md Section 5. |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `Spec.title: Option<TitleBinding>` | `framework/src/json_ui/mod.rs` title resolution | `TitleBinding` match on `spec.title` | WIRED | Match arm in renderer; fallback to "Ferro" when binding path missing |
| `KanbanBoardProps.data_path` | `render_kanban_board` in containers.rs | resolve_path import; branch on `props.data_path` | WIRED | `data_path` wins over static `columns` when both set |
| `ImageProps.data_path` | `render/atoms.rs` | resolve_path + fallback to static src | WIRED | Falls back to `src` when data_path missing or non-string |
| `CardVariant` enum | `render_card` in containers.rs | match `props.variant` → CSS class selection | WIRED | Bordered → `shadow-sm + border + p-4`; Elevated → `shadow-md + p-8` |
| `loader.rs` load_cached | `tracing::warn` on catalog errors (D-16) | replaced `.map_err(LoadError::Catalog)?` | WIRED | D-16 pipeline reorder confirmed in loader.rs |
| `global_catalog().validate` after `expand_directives` | framework `resolve()` and `resolve_with_errors()` | import + call site AFTER `expand_directives` call | WIRED | Both resolve paths updated; integration tests confirm |
| `json_ui_validate_spec` MCP tool | `ferro-mcp/src/service.rs` registration | `tools/mod.rs` export + service.rs register | WIRED | 3 occurrences in service.rs; 1 in mod.rs |
| `Visibility` hand-rolled Deserialize | `PageHeaderProps` and all Visibility consumers | serde `Deserialize` impl on `Visibility` | WIRED | Serialize direction unchanged; all four variant shapes covered |

---

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|-------------------|--------|
| `render_card` in containers.rs | `props.variant` | `CardProps::variant` field (serde default=Bordered) | Yes — enum variant drives CSS selection | FLOWING |
| `render_kanban_board` | `props.data_path` | resolved via `resolve_path(&spec.data, path)` from handler data | Yes — resolves from real spec.data payload | FLOWING |
| `render_image` | `props.data_path` | resolved via `resolve_path` fallback chain | Yes — falls back to static src when missing | FLOWING |
| `framework title resolution` | `spec.title: Option<TitleBinding>` | `spec.data.pointer(&r.data)` for binding variant | Yes — JSON Pointer into handler data | FLOWING |

---

### Behavioral Spot-Checks

Step 7b: Partially applicable — the phase produces library code, not a standalone runnable entry point. Spot-checks run on structural verifiability only.

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| MAX_NESTING_DEPTH = 5 in source | `grep -c 'pub const MAX_NESTING_DEPTH: usize = 5' ferro-json-ui/src/spec.rs` | 1 | PASS |
| MAX_NESTING_DEPTH = 3 absent | `grep -c 'pub const MAX_NESTING_DEPTH: usize = 3' ferro-json-ui/src/spec.rs` | 0 | PASS |
| RawHtmlProps struct present | `grep -c 'pub struct RawHtmlProps' ferro-json-ui/src/component.rs` | 1 | PASS |
| CardVariant enum present | `grep -c 'pub enum CardVariant' ferro-json-ui/src/component.rs` | 1 | PASS |
| TitleBinding enum present | `grep -c 'pub enum TitleBinding' ferro-json-ui/src/spec.rs` | 1 | PASS |
| BUILTIN_TYPES asserts 41 | `grep -n 'assert_eq.*BUILTIN_TYPES.len' ferro-json-ui/src/render/mod.rs` | line 532: `assert_eq!(BUILTIN_TYPES.len(), 41)` | PASS |
| json_ui_validate_spec registered | `grep -c 'json_ui_validate_spec' ferro-mcp/src/service.rs` | 3 | PASS |
| view.rs absent | `ls ferro-json-ui/src/view.rs` | No such file | PASS |
| v1 types absent from prod source | `grep -rE '(JsonUiView|ComponentNode|PluginProps)' ferro-json-ui/src/*.rs framework/src/*.rs` (non-doc) | 0 production matches | PASS |
| BLOCKER rows = 0 in audit | `grep 'Total BLOCKER rows' V1-DELETION-AUDIT.md` | "Total BLOCKER rows: 0" | PASS |
| COMPLETED.md exists with 5 sections | file exists; section count | 5 sections confirmed | PASS |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| D-14 / F4 (MAX_NESTING_DEPTH 3→5) | Plan 01 | Raise depth limit; tests for pass/fail | SATISFIED | Constant=5; two new depth tests; reject fixture |
| D-19/F2 (codemod uppercase methods) | Plan 02 | Regression test locking uppercase emission | SATISFIED | `codemod_emits_uppercase_http_methods` test; fixture `in_all_verbs.rs` |
| D-15 / F7 (Image/DescList data_path) | Plan 03 | data_path override on both props | SATISFIED | Both structs have `data_path: Option<String>`; render tests present |
| D-17a / F9 (RawHtml component) | Plan 03 | Closes "type: Plugin" block | SATISFIED | `RawHtmlProps` struct; catalog count 41; render dispatch arm |
| D-12 / F1 (Spec.title binding) | Plan 04 | TitleBinding enum; renderer resolves at request time | SATISFIED | `TitleBinding`, `DataRef` in spec.rs; framework title resolution match |
| D-18 / F10 (CardVariant) | Plan 05 | Bordered/Elevated enum; backward compat default | SATISFIED | `CardVariant` enum; `#[serde(default)]`; render match; 9 tests |
| D-13a / F3 (KanbanBoard.data_path) | Plan 06 | Runtime column resolution | SATISFIED | `KanbanBoardProps.data_path`; render branch; 4 render tests |
| D-16 / F8 (two-stage validation) | Plan 07 | Structural hard-fail at load; catalog warn-then-enforce post-expand | SATISFIED | `tracing::warn` in loader; `global_catalog().validate` after `expand_directives` in framework; pipeline integration tests |
| D-19/F5 (Visibility error message) | Plan 08 | Hand-rolled Deserialize naming accepted shapes | SATISFIED | `impl<'de> Deserialize<'de> for Visibility` in visibility.rs; key-presence dispatch |
| D-19/F6 (PageHeader.actions lax) | Plan 08 | Accept null/""/[] without error | SATISFIED | `deserialize_actions_lax` function; `#[serde(deserialize_with)]` on actions field |
| D-04 (MCP validate-spec tool) | Plan 09 | Two-stage MCP tool surfaces structural + catalog errors | SATISFIED | `json_ui_validate_spec.rs` exists; registered in service.rs; 4 tests in tool file |
| D-05 case 4 (children ref to $if-gated element allowed) | Plan 09 | Confirm and regression-test existing behavior | SATISFIED | `validate_allows_children_ref_to_if_gated_element` test in spec.rs |
| D-08/D-09 (documentation pass) | Plan 10 | All v12 components documented; $each kanban example | SATISFIED | components.md (CardVariant, RawHtml, data_path, lax actions); spec-construction.md (Spec.title binding, depth 5); expressions.md ($each kanban example). Note: cheat sheet in migration guide created then intentionally deleted by developer. |
| D-01..D-03 (v1-deletion audit) | Plan 11 | V1-DELETION-AUDIT.md; 0 BLOCKER rows | SATISFIED | File exists; 25 rows; 0 BLOCKER; grep evidence documented |
| D-06..D-07 (plugin surface audit) | Plan 11 | Paper audit; Outcome B; no BLOCKER escalation | SATISFIED | PLUGIN-SURFACE-AUDIT.md exists; Outcome B; 2 gaps fixed inline in plugins.md |
| D-10..D-11 (COMPLETED.md) | Plan 12 | 5 sections; Phase 160 unblocked statement | SATISFIED | COMPLETED.md exists with all 5 sections; "Phase 160 (v1 deletion) is UNBLOCKED" stated |

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|---------|--------|
| `ferro-json-ui/src/render/mod.rs` | 528 | Comment says "BUILTIN_TYPES must be 40 entries" but assertion checks 41 | Warning (WR-01 from code review — not fixed) | Misleads the next engineer adding a component: they may update the assertion to 42 but leave the comment at 40, or vice versa. The assertion itself is correct; only the comment is stale. Non-blocking. |

No STUB patterns, no TODO/FIXME in production paths, no hardcoded empty data flows, no orphaned artifacts.

---

### Human Verification Required

#### 1. CardVariant Visual Rendering

**Test:** Build and open a page that renders `"variant": "bordered"` Card vs `"variant": "elevated"` Card in a browser.
**Expected:** Bordered shows dashboard chrome (1px border, subtle shadow, 1rem padding); Elevated shows no border, stronger shadow (shadow-md), larger padding (2rem). Auth/login pages should no longer have dashboard-style chrome.
**Why human:** Unit tests assert CSS class strings (`border-border`, `shadow-sm`, `shadow-md`, `p-4`, `p-8`). Actual visual rendering — color correctness, shadow depth perception, padding proportions — requires browser inspection. Reference: V7-RUNTIME-FRICTION.md F10, gestiscilo `auth/login.json` context.

---

### Gaps Summary

No blocking gaps found. All eight V7-RUNTIME friction items (F1–F10, with F5/F6 also getting ferro-side improvements) are implemented with tests and committed. The v1 deletion audit shows 0 BLOCKER rows. COMPLETED.md is written with all required sections.

**Open items (non-blocking):**
- WR-01 from code review: stale comment in `ferro-json-ui/src/render/mod.rs:528` says "BUILTIN_TYPES must be 40" but assertion is 41. One-liner fix: update comment to "41". Not blocking Phase 160 or Phase 161.
- `migration-v1-to-v2.md` was created with cheat sheet (D-09 fulfilled), then intentionally deleted by the developer (commit 52e38ab3) as the public doc set describes JSON-UI as the only version. The cheat sheet content lives in COMPLETED.md Section 5 (v1→v2 surface migration table). No gap for Phase 160/161.

The single human verification item (CardVariant visual rendering) is the only reason status is `human_needed` rather than `passed`.

---

_Verified: 2026-05-17T03:34:57Z_
_Verifier: Claude (gsd-verifier)_
