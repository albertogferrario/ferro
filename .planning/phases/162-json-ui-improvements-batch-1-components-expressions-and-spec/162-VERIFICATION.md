---
phase: 162-json-ui-improvements-batch-1-components-expressions-and-spec
verified: 2026-05-16T22:00:00Z
status: passed
score: 25/25
verifier_model: sonnet
overrides_applied: 0
human_verification: []
gaps: []
deferred: []
resolved_findings:
  - finding: "plugins.md line 7 stated '39 built-in components' (stale; CheckboxList raised to 40)"
    resolution: "Updated to '40 built-in components' in commit a04a5f73"
  - finding: "validate_footer_ids inherently covers Modal but only Card had explicit test coverage"
    resolution: "Added from_json_rejects_missing_modal_footer_id in commit a04a5f73"
---

# Phase 162: JSON-UI Improvements Batch 1 Verification Report

**Phase Goal:** Absorb gestiscilo Phase 138 friction and deliver 25 decisions (D-01..D-25) spanning new components, render-time improvements, spec validation, MCP tooling, and documentation.
**Verified:** 2026-05-16T22:00:00Z
**Status:** human_needed (2 minor quality items need human decision; all 25 decisions are implemented)
**Re-verification:** No — initial verification

---

## Suite Results

| Command | Exit Code | Notes |
|---------|-----------|-------|
| `cargo fmt --all -- --check` | 0 | Clean |
| `cargo clippy --all --all-targets -- -D warnings` | 0 | No warnings |
| `cargo test --all-features` | 0 | All tests pass (no FAILED) |
| `mdbook build docs/` | 0 | HTML written to docs/book/ including migration-v1-to-v2.html |

---

## Decision Audit — D-01 through D-25

| # | Decision | Status | Evidence Path | Quality Verdict | Notes |
|---|----------|--------|---------------|-----------------|-------|
| D-01 | CheckboxList first-class component with field, options, options_path, selected_path, label, description, disabled, error | SHIPPED | `ferro-json-ui/src/component.rs:380`, `render/form.rs:463` | PASS | Props match CONTEXT spec. 5 unit tests: renders per option, selected_path pre-check, options_path dynamic resolve, HTML escape, disabled propagation |
| D-02 | CheckboxList catalog entry; existing Checkbox unchanged | SHIPPED | `catalog.rs:350`, `render/mod.rs:82,193` | PASS | BUILTIN_TYPES count 39→40 atomically with dispatch arm and BUILTIN_SPECS; Checkbox render untouched |
| D-03 | DataTable row_actions[i].action.url supports {row_key} interpolation at render time | SHIPPED | `render/data.rs:292-336` | PASS | template_actions generalizes column key substitution; {row_key} and {id} remain as legacy aliases |
| D-04 | Any column key placeholder ({label}, {slug_path}, {status}, …) substituted; missing keys left unsubstituted (no panic) | SHIPPED | `render/data.rs:314-325` | PASS | `row.as_object()` iteration with `String::replace` no-op for missing keys; 4 tests cover: single key, multiple keys, missing key (passthrough), legacy regression |
| D-05 | Auth layout card wrapper (`bg-card rounded-lg shadow-md p-8`) removed; layout is structural only | SHIPPED | `layout.rs:373`, test line 825 | PASS | Wrapper div deleted; `auth_layout_centers_content` asserts `!html.contains("bg-card rounded-lg shadow-md p-8")` |
| D-06 | No Fragment/Group borderless container added | NO-CODE | — | PASS | grep for Fragment/Group in BUILTIN_TYPES shows only ButtonGroup (pre-existing). D-06 explicitly rejects the feature. |
| D-07 | Spec validator emits SpecError::FooterMissing when footer-referenced element ID is absent | SHIPPED | `spec.rs:528,544`, test line 918 | PASS | validate_footer_ids() wired into validate_structure(); FooterMissing { element_id, footer_id } returned. Test asserts exact error variant and fields. |
| D-08 | Spec validator emits stderr warning when element ID is in both props.footer and children | SHIPPED | `spec.rs:549-555`, test line 945 | PASS | eprintln! for non-fatal warning; spec_warns_duplicate_footer_child test asserts parse succeeds. No tracing dependency added. |
| D-09 | json_ui_verify_action MCP tool: returns RouteInfo on match, Levenshtein candidate on miss | SHIPPED | `ferro-mcp/src/tools/json_ui_verify_action.rs`, `service.rs:1339,1350` | PASS | 5 unit tests: found, found with method filter, Levenshtein candidate, empty list, oversized input rejection (256-char cap). Wired into MCP dispatcher. |
| D-10 | No #[handler(name=...)] attribute added | NO-CODE | — | PASS | `git log --oneline master..HEAD -- ferro-macros/` is empty. D-10 explicitly rejects this feature. |
| D-11 | strum::AsRefStr on AlertVariant, BadgeVariant, ButtonVariant, ToastVariant, DialogVariant, NotifyVariant | SHIPPED | `component.rs:51,85,98,583`, `action.rs:13,47` | PASS | All 6 enums have #[derive(..., strum::AsRefStr)] and #[strum(serialize_all = "snake_case")]. strum 0.26 added to ferro-json-ui/Cargo.toml. |
| D-12 | JSON wire format unchanged by strum derive | NO-CODE | `component.rs:1277`, `action.rs:376` | PASS | `variant_enums_strum_matches_serde_wire_format` and `dialog_notify_variant_strum_matches_serde` round-trip tests pin strum.as_ref() == serde output. |
| D-13 | Migration banner at top of components.md linking to migration-v1-to-v2.md | SHIPPED | `docs/src/json-ui/components.md:1-5` | PASS | Banner present: "Migrating from v1? See the Migration v1 → v2 guide…" |
| D-14 | Worked example in components.md: Card with children IDs into flat elements map | SHIPPED | `docs/src/json-ui/components.md:101` | PASS | "v1→v2 children migration" section present with example |
| D-15 | "Inline view/edit" section in components.md; DetailFormProps not re-added | SHIPPED | `docs/src/json-ui/components.md:1202` | PASS | Section present at line 1202; no DetailFormProps in component.rs |
| D-16 | SwitchProps.compact: Option<bool>; emits scale-75 origin-left ONLY when Some(true) | SHIPPED | `component.rs:435`, `render/form.rs:576-580` | PASS | `if props.compact == Some(true)` guards the class suffix; test verifies Some(true) adds class, Some(false) and None do not |
| D-17 | ImageProps.inline_svg: Option<String>; NO img tag when set; alt html_escaped | SHIPPED | `component.rs:535`, `render/atoms.rs:373-377` | PASS | Early-return branch emits `<div aria-label="{alt}">{svg}</div>`; `html_escape` on alt; 2 tests: no-img and alt XSS escape; factory ImageProps::inline_svg(svg, alt) round-trips via serde |
| D-18 | RichTextEditor v2 plugin using Quill 2.0.3 via JsonUiPlugin surface; field/label html_escaped | SHIPPED | `plugins/rich_text_editor.rs` | PASS | Registered in global_plugin_registry. SRI sha384 hashes pinned (commit 2d7d80c4, RESOLVED per DEFERRED.md). XSS: field_esc, label_esc, html_escape applied. Test `rich_text_editor_plugin_assets_carry_sri_hashes` pins both css and js hashes. |
| D-19 | plugins.md documents JsonUiPlugin authoring surface | SHIPPED | `docs/src/json-ui/plugins.md` | CONCERN | 309-line guide covers trait, register_plugin, asset injection, Map and RichTextEditor examples. BUT line 7 still reads "The 39 built-in components" — should be 40 post-Phase 162. See Human Verification Required section. |
| D-20 | docs/src/json-ui/migration-v1-to-v2.md with 7 worked-example sections | SHIPPED | `docs/src/json-ui/migration-v1-to-v2.md` | PASS | 493 lines, all 7 sections present (render_file, depth-flattening, DataTable interpolation, read+edit, CheckboxList, strum round-trip, json_ui_verify_action). mdbook builds it successfully. |
| D-21 | Dual catalog+MCP update for every new/changed component | SHIPPED | `catalog.rs`, `render/mod.rs`, `json_ui_catalog.rs` | PASS | CheckboxList in BUILTIN_SPECS, BUILTIN_TYPES, dispatch arm, and MCP test expected list. RichTextEditor in plugin_components (separate from BUILTIN_SPECS — correct per plan). MCP test asserts 40 built-ins + 2 plugin components. |
| D-22 | code_templates surfaces 7 migration_v1_to_v2 templates | SHIPPED | `ferro-mcp/src/tools/code_templates.rs:1504` | PASS | migration_v1_to_v2_templates() registered via build_templates(); test asserts >= 7 templates all with category "migration_v1_to_v2" |
| D-23 | No cargo publish; workspace version unchanged at 0.2.35 | NO-CODE | `Cargo.toml:33` | PASS | `[workspace.package] version = "0.2.35"` confirmed; no 0.2.36 in CHANGELOG |
| D-24 | publish.yml not modified | NO-CODE | — | PASS | `git diff master -- .github/workflows/publish.yml` empty; `git log --oneline master..HEAD -- .github/workflows/publish.yml` empty |
| D-25 | CHANGELOG.md Unreleased section lists every Phase 162 decision | SHIPPED | `CHANGELOG.md:6-38` | PASS | "## [Unreleased] — ferro-json-ui / ferro-mcp (Phase 162)" block; Added: CheckboxList, Switch.compact, Image.inline_svg, RichTextEditor, FooterMissing, json_ui_verify_action, strum derives, migration guide, plugins.md, 7 code_templates; Changed: DataTable interpolation, D-08 warning, catalog shape, components.md; Removed: auth layout card wrapper; Notes: D-06 and D-10 explicitly not added, D-23/D-24 no publish. No 0.2.36 header. |

**Score: 24/25 decisions verified** (D-06, D-10, D-12, D-13–D-15, D-23, D-24 are no-code decisions, correctly absent from the codebase; all code decisions shipped with tests)

---

## Quality Gaps

### QG-1 (CONCERN): plugins.md stale "39 built-in components" count

**File:** `docs/src/json-ui/plugins.md`, line 7
**Text:** "The 39 built-in components cover most server-driven UI patterns."
**Actual:** 40 built-in components after Phase 162 (CheckboxList added in D-01/D-02).
**Impact:** Documentation only. No functional consequence — consumers using the plugin system are not misled about plugin behavior. The count is cosmetic context-setting.
**Category:** Warning (incomplete doc update)

### QG-2 (INFO): Modal footer validation tested only via Card element

**File:** `ferro-json-ui/src/spec.rs`, tests at line 918 and 945
**Issue:** D-07 requires validate_footer_ids to cover both Card and Modal footers. The implementation is fully generic (reads `props.get("footer")` from any element), which correctly covers Modal. However, only a Card element is tested explicitly.
**Impact:** The code is correct. The test coverage gap is minor — the code path is identical for Modal and Card at the spec level (both use the `footer: Vec<String>` JSON key). The generic walk means a Modal-specific test would exercise the same code path.
**Category:** Info (test coverage breadth, not a functional gap)

### QG-3 (RESOLVED): Quill 2.0.3 SRI hashes

**Status:** RESOLVED in Phase 162 (commit 2d7d80c4, documented in DEFERRED.md).
Hashes pinned: CSS `sha384-ecIckRi4QlKYya/...` and JS `sha384-utBUCeG4SYaCm4m7...`. Test `rich_text_editor_plugin_assets_carry_sri_hashes` pins both. Not a gap.

---

## Observable Truths Verification

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | CheckboxList component renders one checkbox per option with XSS-safe HTML | VERIFIED | render_checkbox_list() at form.rs:463; html_escape applied to field, value, label, checkbox_id, description, error; 5 tests pass |
| 2 | DataTable row_actions URL substitutes any column key; missing keys pass through | VERIFIED | template_actions() at data.rs:314; String::replace no-op for missing keys; 4 tests including explicit missing-key passthrough assertion |
| 3 | Auth layout emits no card chrome wrapper | VERIFIED | layout.rs:373 removes wrapper; test asserts `!html.contains("bg-card rounded-lg shadow-md p-8")` |
| 4 | Spec validator rejects footer references to missing element IDs | VERIFIED | validate_footer_ids() at spec.rs:528; SpecError::FooterMissing returned; test from_json_rejects_missing_footer_id passes |
| 5 | json_ui_verify_action MCP tool returns Levenshtein candidate on miss | VERIFIED | find_handler() at json_ui_verify_action.rs:52; strsim::levenshtein; test verify_action_not_found_returns_closest_levenshtein_candidate: "dashboar.show" → "dashboard.show" |
| 6 | Six variant enums expose .as_ref() returning snake_case matching JSON wire format | VERIFIED | strum::AsRefStr on all 6 enums; serialize_all = "snake_case"; round-trip tests in component.rs and action.rs |
| 7 | SwitchProps.compact emits scale-75 origin-left ONLY on Some(true) | VERIFIED | form.rs:576: `if props.compact == Some(true)`; test covers Some(true) adds class, Some(false) and None do not |
| 8 | ImageProps.inline_svg emits no img tag; alt is html_escaped in aria-label | VERIFIED | atoms.rs:373-377 early-return; html_escape(&props.alt); 2 tests: no-img and alt XSS escape |
| 9 | RichTextEditor plugin has SRI-pinned CDN assets | VERIFIED | QUILL_CSS_SRI and QUILL_JS_SRI constants; .integrity() called on each Asset; test asserts integrity is Some(sha384-...) |
| 10 | migration-v1-to-v2.md exists with 7 sections; linked from SUMMARY.md; mdbook builds | VERIFIED | 493-line file; H2 headings at sections 1-7 + summary; SUMMARY.md line 62; mdbook exit 0 |
| 11 | Workspace version is 0.2.35; no publish; publish.yml unchanged | VERIFIED | Cargo.toml:33; git diff master -- publish.yml is empty |
| 12 | CHANGELOG.md Unreleased block covers all shipped decisions with no 0.2.36 header | VERIFIED | Lines 6-38; grep -c 0.2.36 = 0; grep -c 162-0 = 11 |
| 13 | Full suite (fmt + clippy + test + mdbook) exits 0 | VERIFIED | All four commands exit 0 |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| CheckboxListProps | render_checkbox_list | BUILTIN_TYPES dispatch arm | WIRED | render/mod.rs:193 |
| CheckboxList | BUILTIN_SPECS catalog | catalog.rs:350 | WIRED | Import and entry present |
| CheckboxList | ferro-mcp test_all_components_present | expected list | WIRED | json_ui_catalog.rs:257 |
| template_actions | row.as_object() column keys | D-04 generalization | WIRED | data.rs:315-325 |
| validate_footer_ids | validate_structure | spec.rs:474 | WIRED | Called after validate_no_dangling |
| json_ui_verify_action.execute | MCP dispatcher | service.rs:1354 | WIRED | tool_router macro at line 1339 |
| strum::AsRefStr | all 6 enums | Cargo.toml dep + derive | WIRED | strum 0.26 in Cargo.toml; 6 derives confirmed |
| RichTextEditor plugin | global_plugin_registry | plugins/mod.rs | WIRED | Confirmed by test_plugin_components_present (2 plugins: Map + RichTextEditor) |
| migration-v1-to-v2.md | SUMMARY.md nav | docs/src/SUMMARY.md:62 | WIRED | "Migration v1 → v2" entry present |
| migration_v1_to_v2_templates | build_templates() | code_templates.rs:79 | WIRED | templates.extend(migration_v1_to_v2_templates()) |

---

## Data-Flow Trace (Level 4)

Phase 162 delivers components, rendering improvements, and documentation — not dashboard/data pipeline pages. Spot-checking is more appropriate than full data-flow traces. Selected flows:

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| render_checkbox_list | options / selected | resolve_path(data, options_path) / resolve_path(data, selected_path) | Yes — resolves from spec data JSON at render time | FLOWING |
| template_actions | url placeholders | row.as_object() iteration | Yes — iterates actual row data from spec data | FLOWING |
| validate_footer_ids | footer_ids | el.props.get("footer") | Yes — reads parsed spec elements map | FLOWING |
| find_handler (MCP) | routes | list_routes::execute(project_root) | Yes — reads from live app or static file parse | FLOWING |

---

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| CheckboxList struct compiles and schema generates | `cargo test -p ferro-json-ui schema_for_checkbox_list` | In full suite pass | PASS |
| DataTable missing-key leaves placeholder | `cargo test -p ferro-json-ui data_table_url_template_missing_key_leaves_placeholder` | In full suite pass | PASS |
| SwitchProps compact=false emits no scale-75 | `cargo test -p ferro-json-ui switch_compact_adds_scale_class` | In full suite pass | PASS |
| Quill SRI hash pinned | `cargo test -p ferro-json-ui rich_text_editor_plugin_assets_carry_sri_hashes` | In full suite pass | PASS |
| Levenshtein candidate on route miss | `cargo test -p ferro-mcp verify_action_not_found_returns_closest_levenshtein_candidate` | In full suite pass | PASS |
| 7 migration code templates | `cargo test -p ferro-mcp code_templates_returns_migration_patterns` | In full suite pass | PASS |
| MCP catalog = 40 built-ins + 2 plugin | `cargo test -p ferro-mcp test_all_components_present test_plugin_components_present` | In full suite pass | PASS |

---

## Human Verification Required

### 1. plugins.md stale "39 built-in components" count

**Test:** Open `docs/src/json-ui/plugins.md` line 7. Read: "The 39 built-in components cover most server-driven UI patterns."
**Expected:** Either confirm this is acceptable to leave until next doc pass, or update to "40 built-in components".
**Why human:** The count is factually wrong (Phase 162 added CheckboxList, making it 40), but the fix is trivial and the impact is documentation only. This is a developer call on whether to fix it in this phase or defer.

### 2. Modal footer validation test coverage

**Test:** Review `ferro-json-ui/src/spec.rs` validate_footer_ids (line 528). The function walks all elements generically — it will catch a missing footer ID on a Modal element the same way it catches it on a Card element. The existing tests only use Card.
**Expected:** Confirm that the generic implementation is sufficient for the D-07 coverage bar, or request a Modal-specific test.
**Why human:** The code is provably correct (reads props.get("footer") generically). Whether to add an explicit Modal test for clarity is a judgment call.

---

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|---------|--------|
| `docs/src/json-ui/plugins.md` | 7 | Stale count "39 built-in components" | Warning | Documentation only; no functional impact |

No TODO/FIXME/PLACEHOLDER markers found in implementation files (the Quill SRI TODO was resolved in commit 2d7d80c4 and is documented as RESOLVED in DEFERRED.md).

No empty implementations (`return null`, `return []`, `=> {}`), hardcoded empty data, or console.log-only handlers found in the Phase 162 implementation.

---

## Cross-Check: 162-11 Plan Audit Table vs Actual Findings

The 162-11 plan's human checkpoint (Task 3) presented expected grep counts. Verified:

| Claim | Expected | Actual | Match? |
|-------|----------|--------|--------|
| grep -c "CheckboxList" component.rs + catalog.rs + render/mod.rs + json_ui_catalog.rs | >= 1 each | All present | YES |
| grep -c "row.as_object\|data_table_url_template" render/data.rs | >= 1 | Both present | YES |
| grep -c "bg-card rounded-lg shadow-md p-8" layout.rs | 0 (removed) or 1 (if in test comment) | 0 — test asserts absence without repeating the string literally | YES (test at line 825 uses the string in an assert! which is technically 1 occurrence, but no emitted output) |
| grep -c "FooterMissing\|validate_footer_ids" spec.rs | >= 3 | 4 occurrences of FooterMissing, 3 of validate_footer_ids | YES |
| test -f ferro-mcp/src/tools/json_ui_verify_action.rs | OK | OK | YES |
| grep -c "strum::AsRefStr" component.rs + action.rs | 4 + 2 | 4 + 2 | YES |
| grep -c "pub compact: Option<bool>" component.rs | 1 | 1 | YES |
| grep -c "inline_svg" component.rs + render/atoms.rs | >= 1 each | Both present | YES |
| test -f ferro-json-ui/src/plugins/rich_text_editor.rs | OK | OK | YES |
| wc -l docs/src/json-ui/migration-v1-to-v2.md | >= 300 | 493 | YES |
| grep -c "migration_v1_to_v2_templates" code_templates.rs | 1 | 12 occurrences (function + calls + test) | YES |
| grep -c "TODO(162-04)" rich_text_editor.rs | 0 (resolved) | 0 (SRI pinned in commit 2d7d80c4) | YES |
| grep -n 'version = "0.2.35"' Cargo.toml | 1 | 1 at line 33 | YES |
| grep -c "162-0" CHANGELOG.md | >= 5 | 11 | YES |

**No discrepancies found between the 162-11 checkpoint claims and the actual codebase.**

---

## Overall Verdict

Phase 162 has delivered all 25 decisions to a high implementation bar:

**Shipped (code + tests):** D-01, D-02, D-03, D-04, D-05, D-07, D-08, D-09, D-11, D-16, D-17, D-18, D-19, D-20, D-21, D-22, D-25 — 17 decisions with implementation and automated tests.

**No-code (correctly absent):** D-06, D-10, D-12, D-13, D-14, D-15, D-23, D-24 — 8 decisions explicitly rejected or documentation-only; all verified absent or documented correctly.

**Quality bar:**
- XSS safety: html_escape applied to all user-surface strings in CheckboxList (field, value, label, checkbox_id, description, error), RichTextEditor (field, label, initial), Image inline_svg (alt in aria-label). Tests assert XSS escape behavior.
- SRI: Quill 2.0.3 CDN assets carry sha384 hashes; test pins both. RESOLVED in commit 2d7d80c4.
- Wire-format backward compat: strum::AsRefStr adds no serde behavior; round-trip tests confirm strum and serde agree on every snake_case string. DataTable placeholder changes are render-time only; wire format unchanged.
- Catalog count consistency: BUILTIN_TYPES == BUILTIN_SPECS == 40; MCP test_all_components_present asserts 40 built-ins; RichTextEditor in plugin_components (2 total: Map + RichTextEditor).
- Full suite (fmt + clippy + test + mdbook) exits 0.

**Two minor quality items require human decision before the phase is fully closed:**
1. `plugins.md` line 7 has a stale "39 built-in components" count (should be 40). Fix is one word; decision is whether to fix in Phase 162 or defer.
2. Modal footer validation has no Modal-specific test (code is generic and correct; coverage bar is a judgment call).

Neither item affects the correctness of the implementation. The phase goal — absorbing gestiscilo Phase 138 friction into 25 documented decisions — is achieved.

---

_Verified: 2026-05-16T22:00:00Z_
_Verifier: Claude (gsd-verifier, sonnet)_
