---
plan: 116-05
phase: 116-flat-element-renderer
wave: 3
status: complete
completed: 2026-04-18
requirements: [RENDER-01, RENDER-03]
tags: [renderer, form-controls, data-displays, v1-port, data-path, url-templating]
dependency_graph:
  requires: [116-02]
  provides: [real-form-renderers, real-data-renderers, data-path-consumption]
  affects: [plan-116-06]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/render/form.rs
    - ferro-json-ui/src/render/data.rs
    - ferro-json-ui/src/data.rs
metrics:
  duration_minutes: ~35
  tasks_completed: 2
  files_modified: 3
  loc_written: ~1613
  tests_added: 32
---

# Plan 116-05 Summary — Form Controls and Data Displays

## One-liner

Verbatim v1 port of 5 form-control and 2 data-display renderers into the Phase 116 walker; `ferro-json-ui/src/data.rs` resolvers now compile without dead-code attributes because form.rs and data.rs consume them.

## What Landed

### Task 1 — `render/form.rs` (5 renderers, 1008 LOC, 20 tests)

Ports v1 `render.rs:961–1711` covering `Form`, `Input`, `Select`, `Checkbox`, `Switch`.

| v1 function | v1 range | v2 adaptation |
|---|---|---|
| `render_form` | 961–1015 | Fields come from `Element.children` (IDs) instead of removed `FormProps.fields` per D-05. Action URL: `Some → action=url`; `None → action="#"` + D-16 diagnostic comment. |
| `render_input` | 1289–1436 | Unchanged HTML; `default_value > data_path` precedence preserved verbatim. |
| `render_select` | 1438–1534 | Unchanged HTML; same precedence as Input. |
| `render_checkbox` | 1536–1601 | Unchanged HTML; shared `resolve_checked` helper with Switch. |
| `render_switch` | 1603–1711 | Unchanged HTML including `onchange="this.closest('form').submit()"` auto-submit. D-16 diagnostic applied when `action.url = None`. |

### Task 2 — `render/data.rs` (2 renderers, 605 LOC, 12 tests) + `data.rs` (`#[allow(dead_code)]` removal)

Ports v1 `render.rs:1017–1285` covering `Table` and `DataTable`.

| v1 function | v1 range | v2 adaptation |
|---|---|---|
| `render_table` | 1017–1102 | Unchanged HTML including `Azioni` action column, anchor-link row actions. |
| `render_data_table` | 1104–1285 | Unchanged HTML except row actions wrapped in inline `<details>` dropdown (see Deviations) instead of Plan 03's `render_dropdown_menu` — keeps Plan 05 independent of Plan 03. |

`ferro-json-ui/src/data.rs`: both `#[allow(dead_code)]` attributes removed (lines 18 and 54). `resolve_path` and `resolve_path_string` are now consumed by form.rs (Input/Select/Checkbox/Switch data_path pre-fill) and data.rs (Table/DataTable row resolution).

DescriptionList and Pagination stay in atoms.rs per plan's recommendation and CONTEXT "Claude's Discretion" — Plan 116-03 owns their implementation.

## Non-obvious v1 behaviors preserved

- **Switch auto-form wrap (SwitchProps.action).** When `SwitchProps.action` is `Some`, the switch renders inside a `<form>` whose child `<input type="checkbox" role="switch">` carries `onchange="this.closest('form').submit()"`. Method spoofing applies for PUT/PATCH/DELETE (hidden `_method` input). (v1 render.rs:1603–1711.)
- **Input/Select `default_value > data_path` precedence.** Spec-author-provided default overrides the resolved data path. This matches v1 verbatim and differs from the plan text which suggested the opposite precedence — the plan text was in error; D-21 mandates v1 semantics.
- **Checkbox/Switch truthiness match.** `Value::Bool` returns its own value; `Value::Number` is truthy iff nonzero; `Value::String` is truthy iff non-empty AND not the literal `"false"` / `"0"`; `Value::Null` is always falsy; arrays/objects are always truthy. (v1 render.rs:1538–1552, 1605–1619.)
- **DataTable URL templating.** v1 substitutes `{row_key}` against `props.row_key`'s value on each row (fallback: row index). This plan adds `{id}` as a convenience placeholder resolved against `row["id"]` when present. Both run per-row before `html_escape`.
- **Form method spoofing.** PUT/PATCH/DELETE render as `method="post"` + hidden `<input name="_method" value="PUT|PATCH|DELETE">`. (v1 render.rs:970–1001.)
- **Form max-width wrapper.** `FormMaxWidth::Narrow` → `<div class="max-w-2xl mx-auto">`, `Wide` → `<div class="max-w-4xl mx-auto">` per v1 FIX-02.
- **Input A11Y-07 hidden inputs.** Hidden inputs emit a single `<input type="hidden">` with no label or wrapper div.
- **Input datalist.** `props.list = Some(id)` emits a companion `<datalist id="...">` whose options are pulled from `data[id]` (flat root lookup, v1 verbatim).

## Deviations from Plan

### Auto-fixed issues

**1. [Rule 1 — Plan wrong about value precedence] Input/Select default_value > data_path.**
- **Found during:** Task 1 v1 source review
- **Issue:** Plan's `<behavior>` stated "data_path → value (overrides default_value)". v1 render.rs:1289–1297 and 1438–1446 use the opposite precedence: `default_value` wins, `data_path` is the fallback.
- **Fix:** Implemented v1 precedence per D-21 verbatim-port rule. Tests `input_default_value_wins_over_data_path` and `select_default_value_wins_over_data_path` assert this.
- **Rationale:** D-21 mandates v1 HTML semantics; the plan instruction was an aspirational recollection that didn't match the canonical source.

**2. [Rule 1 — Plan wrong about switch auto-form markup] `onchange` not `data-auto-submit`.**
- **Found during:** Task 1 v1 source review
- **Issue:** Plan's `<action>` pattern suggested wrapping with `<form ... data-auto-submit>`. v1 render.rs:1621–1694 emits `onchange="this.closest('form').submit()"` on the inner checkbox; no `data-auto-submit` attribute exists in v1.
- **Fix:** Implemented v1 verbatim. Test `switch_with_action_wraps_in_form` asserts the `onchange` attribute is present.

**3. [Rule 1 — Plan wrong about switch action location] `SwitchProps.action` not `Element.action`.**
- **Found during:** Task 1 prop struct review
- **Issue:** Plan's test snippet used `el.action = Some(Action{...})`. Actual `SwitchProps` (component.rs:369) holds `action: Option<Action>` inside the props, not on the `Element`.
- **Fix:** Tests pass the action inside props (`json!({... "action": action})`). Semantically equivalent — the wrap condition triggers on `props.action.is_some()`.

**4. [Rule 1 — Plan wrong about DataTable placeholder] `{row_key}` is v1's actual placeholder.**
- **Found during:** Task 2 v1 source review
- **Issue:** Plan test and CONTEXT non-obvious-behaviors list reference `{id}` substitution. v1 render.rs:1193 and 1264 actually substitute `{row_key}` against `props.row_key`'s per-row value (fallback: row index).
- **Fix:** Implemented both placeholders. `{row_key}` preserves v1 verbatim per D-21. `{id}` is an additional convenience shortcut resolved against `row["id"]` when present — satisfies the plan test and stays backward-compatible for any existing v1 callers using `{row_key}`. Both placeholders are substituted before `html_escape`.
- **Documented:** module-level doc in `render/data.rs` explicitly states both placeholders and their origin.

**5. [Rule 3 — Cross-wave isolation] Inline `<details>` dropdown instead of `render_dropdown_menu`.**
- **Found during:** Task 2 v1 source review
- **Issue:** v1 `render_data_table` calls `render_dropdown_menu(&DropdownMenuProps{...})` for per-row actions. `render_dropdown_menu` is a stub in `render/atoms.rs` until Plan 116-03 fills it. Plans 03, 04, 05 run in parallel so Plan 05 cannot call a Plan-03-owned renderer.
- **Fix:** Emit an inline `<details>` + `<summary>` dropdown structure self-contained in `render/data.rs`. Preserves v1's dropdown affordance (same U+22EE kebab trigger) and keeps URL templating identical. Plan 116-06 integration can swap to `atoms::render_dropdown_menu` once all three waves merge.
- **Impact:** Visual markup differs slightly from v1 for DataTable row actions (inline `<details>` vs. styled dropdown), but the contract tested by `data_table_url_template_replaces_id` and siblings — that `{id}` and `{row_key}` substitution works and escape discipline holds — is preserved byte-exact.

None of the deviations change success criteria or public API. All are documented inline with comments pointing to v1 line ranges.

## Success Criteria

| # | Criterion | Status | Evidence |
|---|---|---|---|
| 1 | 5 form-control renderer bodies real | PASS | form.rs 1008 LOC; no `stub_renderer!` |
| 2 | 2+ data-display renderer bodies real | PASS | data.rs 605 LOC; Table + DataTable implemented; no `stub_renderer!` |
| 3 | `data_path` pre-fill works for Input/Select/Checkbox/Switch | PASS | 4 tests: input_data_path_prefills_value, select_data_path_marks_option_selected, checkbox_data_path_truthy_renders_checked, switch resolve_checked path |
| 4 | DataTable `{id}` URL templating preserved | PASS | data_table_url_template_replaces_id test |
| 5 | DataTable `{row_key}` URL templating preserved | PASS | data_table_url_template_replaces_row_key test (additional v1 parity) |
| 6 | Switch auto-form wrap preserved | PASS | switch_with_action_wraps_in_form, switch_without_action_no_form_wrap |
| 7 | Form action URL: Some → url, None → "#" + diagnostic | PASS | form_action_url_resolved_in_action_attr, form_action_url_unresolved_falls_back_with_diagnostic |
| 8 | `#[allow(dead_code)]` removed from data.rs | PASS | `grep -c '#\[allow(dead_code)\]' ferro-json-ui/src/data.rs = 0` |
| 9 | `cargo test -p ferro-json-ui --lib` passes | PASS | 244 tests, 0 failed |
| 10 | `cargo clippy -p ferro-json-ui --lib --all-features -- -D warnings` clean | PASS | no dead-code warnings after removal |
| 11 | Each task committed with `--no-verify` | PASS | commits 1783c6c2, 7316677e |
| 12 | SUMMARY.md created | PASS | this file |

## Tests Added (32 total)

### form.rs (20)

Input (5): `input_data_path_prefills_value`, `input_default_value_wins_over_data_path`, `input_hidden_emits_no_label`, `input_xss_in_value_is_escaped`, `input_error_emits_aria_describedby`.

Select (2): `select_data_path_marks_option_selected`, `select_default_value_wins_over_data_path`.

Checkbox (3): `checkbox_data_path_truthy_renders_checked`, `checkbox_data_path_false_omits_checked`, `checkbox_with_value_scopes_id`.

Switch (4): `switch_with_action_wraps_in_form`, `switch_without_action_no_form_wrap`, `switch_action_without_url_emits_diagnostic`, `switch_put_method_spoofs_post`.

Form (4): `form_recurses_children_as_fields`, `form_action_url_resolved_in_action_attr`, `form_action_url_unresolved_falls_back_with_diagnostic`, `form_put_method_spoofs_post_with_hidden_input`.

Decode diagnostics (2): `input_props_decode_failure_emits_diagnostic`, `form_props_decode_failure_emits_diagnostic`.

### data.rs (12)

Table (5): `table_renders_rows_from_data_path`, `table_empty_rows_emits_empty_message`, `table_missing_path_emits_empty_message_when_provided`, `table_cell_value_is_html_escaped`, `table_props_decode_failure_emits_diagnostic`.

DataTable (7): `data_table_url_template_replaces_id`, `data_table_url_template_replaces_row_key`, `data_table_empty_renders_empty_message`, `data_table_default_empty_message_used_when_absent`, `data_table_url_template_substitution_is_escaped`, `data_table_renders_desktop_and_mobile_markup`, `data_table_props_decode_failure_emits_diagnostic`.

## Gates

- `cargo build -p ferro-json-ui --lib`: green
- `cargo test -p ferro-json-ui --lib`: 244 passed, 0 failed
- `cargo clippy -p ferro-json-ui --lib --all-features -- -D warnings`: clean
- `cargo clippy -p ferro-json-ui --lib --all-targets --all-features -- -D warnings`: clean
- `cargo fmt -p ferro-json-ui -- --check`: clean

Per plan disk budget, workspace-wide gates (`cargo test --all-features`, `cargo clippy --all --all-targets`) were NOT run in this worktree.

## Commits

- `1783c6c2` — `feat(116-05): port 5 form controls into render/form.rs`
- `7316677e` — `feat(116-05): port Table + DataTable renderers and consume data::resolve_path`

## DescriptionList / Pagination Disposition

Both stay in `render/atoms.rs` per Plan 03's ownership (CONTEXT "Claude's Discretion" line 119). Plan 05 did NOT move them into `render/data.rs` — the dispatch match in `render/mod.rs` continues to route `DescriptionList` and `Pagination` to `atoms::render_description_list` and `atoms::render_pagination`. Wave 3 Plan 03 fills those stubs.

## Threat Flags

Scan for new security surface — none. All user-supplied strings (labels, values, placeholders, templated URLs) pass through `super::html_escape` before emission. `data::resolve_path` remains a pure JSON-tree lookup (no filesystem, no network). Test `data_table_url_template_substitution_is_escaped` confirms attribute-breakout is impossible even when a row's `id` contains `"><script>`.

## Hand-off to Plan 116-06

Framework-level integration tests (`framework/src/json_ui/mod.rs`) can now assert real HTML output for form and data-display specs — all assertions currently stubbed against the Phase 115 placeholder marker can be rewritten against concrete v1-compatible markup (`<form action=...>`, `<input value=...>`, `<table class="min-w-full...">`, row-action anchors with templated URLs).

Plan 116-06 integration notes:
- If Plan 03 merges a different dropdown markup than the `<details>` inline emitted here for DataTable row actions, Plan 06 should decide whether to unify (call `atoms::render_dropdown_menu` from `data.rs`) or keep the two dropdowns specialized.
- The `{id}` placeholder added here is an extension beyond v1. Plan 06 or Phase 117 catalog docs should document both `{id}` and `{row_key}` as valid templating placeholders for DataTable row_actions.

## Self-Check: PASSED

Files:
- `ferro-json-ui/src/render/form.rs` — FOUND (1008 LOC)
- `ferro-json-ui/src/render/data.rs` — FOUND (605 LOC)
- `ferro-json-ui/src/data.rs` — FOUND (`#[allow(dead_code)]` removed at both sites)

Commits:
- `1783c6c2` — FOUND in `git log`
- `7316677e` — FOUND in `git log`

Gates:
- stub count in form.rs: 0 (expected 0)
- stub count in data.rs: 0 (expected 0)
- `#[allow(dead_code)]` count in ferro-json-ui/src/data.rs: 0 (expected 0)
- test count delta: +32 (212 → 244)
