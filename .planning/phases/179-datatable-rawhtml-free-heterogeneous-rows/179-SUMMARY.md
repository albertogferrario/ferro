---
phase: 179
slug: datatable-rawhtml-free-heterogeneous-rows
status: complete
completed: 2026-05-25
plans:
  - 179-01
key-files:
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/render/atoms.rs
    - ferro-json-ui/src/render/data.rs
    - Cargo.toml
    - Cargo.lock
  created: []
commits:
  - 6715a37c: feat(json-ui) ColumnFormat::Badge + DropdownMenuAction.visible_if
  - bdaad8ac: chore version bump 0.2.37 → 0.2.38
test_results:
  ferro_json_ui_lib: 560 passed, 0 failed
  new_tests_added: 8
---

# Phase 179: DataTable RawHtml-free heterogeneous rows — Complete

## What shipped

Two additive primitives in `ferro-json-ui` that let DataTable callers express per-row varying actions and typed status pills without raw HTML strings in cell values:

### 1. `ColumnFormat::Badge`

New enum variant in `component.rs`. Cell value is `{variant, label}`; renders the same `<span>` shape as the standalone `Badge` component via a shared `badge_inline_html(BadgeVariant, &str)` helper in `atoms.rs` (extracted from `render_badge` — single source of truth for badge markup so the two surfaces cannot drift).

Robustness behaviour:
- `null` cell value → empty cell (no diagnostic — null is "this row has no status").
- Non-object value (string, number, etc.) → HTML comment diagnostic `<!-- ferro-json-ui: invalid Badge cell value: ... -->`. The bad value does NOT render as live UI.
- Object with invalid `variant` enum value → HTML comment diagnostic with serde error message.
- Other column formats (Date / DateTime / Currency / Boolean / None) continue to render through the existing `html_escape(cell_string(...))` path — escaping behaviour for non-Badge cells is preserved (confirmed by the pre-existing `table_cell_value_is_html_escaped` test still passing).

### 2. `DropdownMenuAction.visible_if: Option<String>`

New optional field on `DropdownMenuAction` in `component.rs`. When set, the action item is only emitted for rows where `row[visible_if]` is truthy. Implemented via a private `action_visible_for_row` predicate that `template_actions` calls in a `.filter()` before its existing URL-substitution `.map()`. Truthy = `true` / non-zero number / non-empty string / non-empty array or object.

**Fail-closed design** (per threat T-179-S2): absent field, `null`, `false`, `0`, empty string, empty array, empty object all hide the item. A typo in the spec's `visible_if` field name therefore hides the action everywhere rather than exposing it everywhere — which is the safer default when the action might be privileged (e.g., "Elimina", "Invia link").

Backwards compatible: actions with no `visible_if` always show, exactly as before.

## Test coverage

8 new tests in `render::data::tests`:
- `data_table_badge_column_format_renders_pill` — Badge variant CSS class + label appear, no JSON leak
- `data_table_badge_column_format_invalid_value_emits_diagnostic` — non-object value emits comment, bad value does not render as live UI
- `data_table_badge_column_format_null_value_renders_empty_cell` — `null` is valid, no diagnostic
- `data_table_visible_if_keeps_action_when_truthy` — `true` shows action
- `data_table_visible_if_drops_action_when_falsy` — `false` hides action
- `data_table_visible_if_drops_action_when_field_missing` — fail-closed for typo / missing field
- `data_table_visible_if_absent_keeps_action` — backwards compat (no `visible_if` = always show)
- `data_table_visible_if_filters_per_row_independently` — the load-bearing scenario: two declared actions with different gates produce different action sets per row in the same table

All 8 green. Pre-existing `ferro-json-ui` lib suite: 552 → 560 passing, 0 failing.

## Downstream consumer

Gestiscilo Phase 172 (`unified-documenti-tab-on-booking-detail`) is the load-bearing consumer. Its `documenti_unified` table currently emits HTML strings into `stato`/`riferimento`/`azioni` cells and assumed a `RawHtml` cell variant existed — verifier flagged this as CR-01 ship-blocker. With these primitives, the gestiscilo gap-closure phase can:

- Drop the `build_*_actions_html` Rust helpers
- Emit `stato: {variant: "destructive", label: "Mancante"}` (typed, XSS-free)
- Emit per-row boolean flags (`can_invia_link`, `can_carica_ora`, `can_assegna_cliente`, `can_scarica`, `can_elimina`, `can_apri`) + URL fields
- Declare all 6 possible actions at the table level in `row_actions` with `visible_if: "can_X"` and templated URLs

## Version bump

Workspace `0.2.37 → 0.2.38` (patch level). Additive change only — new enum variant + new optional field, both `#[serde(default)]` with `skip_serializing_if`. No breaking change to existing consumers.

After `git push origin master`, the existing GitHub Actions auto-publish workflow (per `feedback_ferro_publish.md`) ships `ferro-json-ui = "0.2.38"` to crates.io. Gestiscilo can then `cargo update -p ferro-json-ui` and start consuming the new primitives in the gap-closure phase.
