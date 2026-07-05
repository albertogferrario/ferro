---
phase: 254-props-contracts-touch-foundation-design-rules
plan: 01
subsystem: ferro-json-ui
tags: [pos, component-api, serde, schema, contract]
dependency_graph:
  requires: []
  provides:
    - ProductTileProps additive fields (categories, image_url, color, stock_badge)
    - GridProps.row_weights
    - ProductGridProps, CartPanelProps, CategoryNavProps, QuantityStepperProps, NumpadProps
    - NumpadMode enum
    - 8 new tests (3 contract + 5 schema smoke)
  affects:
    - ferro-json-ui/src/component.rs
tech_stack:
  added: []
  patterns:
    - additive serde fields with skip_serializing_if (Vec::is_empty / Option::is_none)
    - schema smoke test pattern (assert_schema_nonempty_object in schema_smoke_tests module)
    - TDD RED/GREEN cycle for Rust compile-time field contract enforcement
key_files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
decisions:
  - D-01: categories is Vec<String> (plural) — multi-category products need Vec, one-element vec covers singular
  - D-02: image_url/color/stock_badge are Option<String> with skip_serializing_if for backward-compat
  - D-18: No CartRuntime hooks in any Props contract — deferred to Future Requirements
  - D-19: row_weights mirrors spans convention exactly (Vec<u8>, skip_serializing_if = Vec::is_empty)
metrics:
  duration_seconds: 422
  completed_date: "2026-07-05"
  tasks_completed: 2
  files_modified: 1
---

# Phase 254 Plan 01: Props Contracts — POS Component API Surface Summary

POS component API contracts locked in `ferro-json-ui/src/component.rs`: `ProductTileProps` extended with four additive serde-backward-compatible fields; `GridProps` gains `row_weights`; five new POS Props structs and `NumpadMode` enum declared with full derive set, rustdoc, and schema smoke tests — none registered.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 (RED) | Failing contract tests for ProductTileProps additive fields + GridProps.row_weights | 86541417 | ferro-json-ui/src/component.rs |
| 1 (GREEN) | ProductTileProps additive fields + GridProps.row_weights | 1e98b4dc | ferro-json-ui/src/component.rs |
| 2 | Five POS Props structs + NumpadMode + schema smoke tests | d60d3571 | ferro-json-ui/src/component.rs |

## What Was Built

### Task 1 — ProductTileProps additive fields + GridProps.row_weights

Four additive fields added to `ProductTileProps` (after `default_quantity`):

- `categories: Vec<String>` — `#[serde(default, skip_serializing_if = "Vec::is_empty")]`; space-separated `data-product-categories` attribute contract (Phase 255 filter runtime reads it; emission deferred to Phase 254 Plan 02)
- `image_url: Option<String>` — Phase 256 tile visual (D-03 named handoff)
- `color: Option<String>` — Phase 256 tile visual (D-03)
- `stock_badge: Option<String>` — Phase 256 tile visual (D-03)

`GridProps.row_weights: Vec<u8>` added after `spans`, mirroring the `spans` convention exactly. Render path (fractional `grid-template-rows` via inline style) deferred to Phase 256 (D-19).

Three contract tests in `product_tile_contract_tests` module:
- `product_tile_legacy_json_round_trips_unchanged` — legacy JSON without new fields round-trips without emitting any new keys (SC-1 / D-04)
- `product_tile_with_categories_serializes` — categories array appears in serialized output when set
- `grid_props_row_weights_round_trips` — empty row_weights omitted; non-empty round-trips unchanged

### Task 2 — Five POS Props structs + NumpadMode enum + schema smoke tests

Five new `pub struct` declarations (not registered, declaration-only per D-16):

- `ProductGridProps` — `data_path`, `form_id`, `categories_path?`, `columns?`, `search?` (D-17 `$each` iteration contract)
- `CartPanelProps` — `form_id`, `empty_message?`, `show_staff?`, `show_people?`
- `CategoryNavProps` — `items: Vec<String>`, `all_label?` (standalone builtin per operator decision)
- `QuantityStepperProps` — `field`, `min?`, `max?`, `step?`
- `NumpadMode` — enum `Quantity` (default) | `Price`, `#[serde(rename_all = "snake_case")]`
- `NumpadProps` — `target_field`, `mode: NumpadMode`

All structs use the full derive set: `Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema`. No `#[allow(dead_code)]` attributes — pub items in a lib crate produce no dead-code warnings (D-16 confirmed).

Five schema smoke tests added to `schema_smoke_tests` module (pattern: `assert_schema_nonempty_object::<T>`).

## Verification Results

```
cargo test -p ferro-json-ui component → 112 passed, 0 failed
cargo clippy -p ferro-json-ui --all-targets --all-features -- -D warnings → exit 0
grep -c 'register_component|BUILTIN_TYPES|BUILTIN_SPECS' component.rs → 0 (count stays 47, D-15)
grep -c 'data-cart-target|cart_target|cart_state' component.rs → 0 (D-18)
```

## Deviations from Plan

None — plan executed exactly as written.

The only procedural deviation was discovering that `kanban_board_props_empty_columns_skipped_on_serialize` lives in its own `kanban_board_props_tests` module (not in `schema_smoke_tests`), so the first smoke test insertion targeted the wrong closing brace. Fixed in the same task before committing.

## Downstream Handoffs

| Phase | What it consumes |
|-------|-----------------|
| 254-02 | `render_product_tile` gains `data-product-categories` emission from `ProductTileProps.categories` |
| 255 | `ProductGridProps.search`, `CategoryNavProps.items`, `NumpadProps.target_field` (runtime contracts) |
| 256 | All five Props structs (renderers); `ProductTileProps.image_url`/`color`/`stock_badge` (tile visual); `GridProps.row_weights` (fractional grid-template-rows) |
| 257 | `ProductGridProps.data_path` (`$each` iteration contract) |

## Self-Check: PASSED

- `ferro-json-ui/src/component.rs` exists and has expected content
- Commits 86541417, 1e98b4dc, d60d3571 present in git log
- 112 component tests pass, clippy --all-features clean
- Component count unchanged (47), no registration surface touched
