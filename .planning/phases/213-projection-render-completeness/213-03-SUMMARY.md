---
phase: 213-projection-render-completeness
plan: "03"
subsystem: ferro-json-ui
tags: [json-ui, projection, render, statcard, data-binding, gap-c]
requirements: [GAP-C]

dependency_graph:
  requires: [213-02]
  provides: [statcard-value-binding]
  affects: [ferro-json-ui/src/component.rs, ferro-json-ui/src/render/atoms.rs, ferro-json-ui/src/projection/builder.rs]

tech_stack:
  added: []
  patterns:
    - "value_path: Option<String> on StatCardProps (mirrors ImageProps.data_path / DescriptionListProps.data_path)"
    - "resolve_path_string resolution in render_stat_card (same JSON-pointer convention as Image/DescriptionList)"

key_files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/render/atoms.rs
    - ferro-json-ui/src/projection/builder.rs

decisions:
  - "Single primary StatCard (first Money/Quantity readable field only) — multi-stat Grid deferred per Risk 1"
  - "value_path format: /data/{service.name}/{field.name} — same JSON-pointer convention as all other data_path fields"
  - "FieldMeaning added to non-test imports in builder.rs (was only present in test scope)"
  - "Orphan-metadata contract (statcard_metadata_is_orphan_element) preserved unchanged"

metrics:
  duration: "~20 minutes"
  completed: "2026-06-12"
  tasks_completed: 2
  files_modified: 3
---

# Phase 213 Plan 03: Gap C — StatCard Value Binding Summary

Bound `StatCard` displayed values to runtime handler data. Previously `emit_statcard_root` emitted `value: String::new()` — stat cards rendered labels with no numbers. After this plan, the primary Money/Quantity readable field drives a `value_path` prop that `render_stat_card` resolves against handler data at render time.

## What Was Built

**`StatCardProps.value_path: Option<String>`** — additive field with `#[serde(default, skip_serializing_if = "Option::is_none")]`. Existing serialized specs are unaffected.

**`render_stat_card` resolution** — renamed `_data` parameter to `data`, computes `display_value` via `resolve_path_string(data, path)` with fallback to `props.value`. Both `html_escape` call sites (SSE branch and plain branch) use `display_value` instead of `props.value`.

**`emit_statcard_root` binding** — scans `service.fields` for the first `readable` field with `FieldMeaning::Money | FieldMeaning::Quantity`. When found, emits `value_path: Some("/data/{service.name}/{field.name}")` and uses `field_display_name` as the label. When no such field exists, emits `value_path: None` with `resolve_title(service)` as the label (unchanged fallback behavior).

## Commits

| Hash | Description |
|------|-------------|
| `c495bf3d` | feat(213-03): add StatCardProps.value_path and resolve it in render_stat_card |
| `7b70cf63` | feat(213-03): emit value_path for primary stat field in emit_statcard_root |

## Tests Added

**atoms.rs:**
- `stat_card_value_path_resolves_from_data` — value_path resolves `€12,450` from handler data JSON
- `stat_card_value_path_fallback_to_static_value` — when `value_path` is None, static `value` renders

**builder.rs:**
- `statcard_root_binds_primary_stat_field` — `service_with_money_field()` (statistics/total_revenue Money field) emits `value_path: Some("/data/statistics/total_revenue")`
- `statcard_root_empty_when_no_stat_field` — service with only Identifier + FreeText fields emits `value_path: None`

**Preserved:**
- `statcard_metadata_is_orphan_element` — green; orphan-metadata contract unchanged
- `from_service_def_validates` — green; catalog validation passes
- `cargo test -p ferro-projections --test catalog` — all 22 frozen tests green

## Verification Results

```
cargo test -p ferro-json-ui --lib render::atoms          → 56 passed
cargo test -p ferro-json-ui --lib --features projections
  -- projection::builder                                  → 16 passed
cargo test -p ferro-projections --test catalog            → 22 passed
cargo fmt --all -- --check                                → clean
cargo clippy --all --all-targets -- -D warnings           → clean
```

## Deviations from Plan

None — plan executed exactly as written.

The one formatting deviation was `rustfmt` reformatting the `find` closure from a multi-line block to a chained iterator form. Applied immediately per `cargo fmt`.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. `value_path` resolution is bounded to the handler's own response body (T-213-07: accepted risk — renderer surfaces only what the handler already returned). The `f.readable` guard on field selection enforces T-213-06 (no non-readable field can be surfaced via `value_path`).

## Known Stubs

`value: String::new()` remains in the emitted `StatCardProps`. This is intentional — the actual value surfaces at render time via `value_path`. No stub removal needed; the empty `value` is the correct static fallback when `value_path` is absent or unresolvable.

## Self-Check: PASSED

- `ferro-json-ui/src/component.rs` — modified, contains `value_path`
- `ferro-json-ui/src/render/atoms.rs` — modified, contains `resolve_path_string`
- `ferro-json-ui/src/projection/builder.rs` — modified, contains `value_path` (9 occurrences)
- Commits `c495bf3d` and `7b70cf63` verified in git log
