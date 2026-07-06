---
phase: 118-server-side-expressions
plan: 01
status: complete
---

## Plan 118-01: Expression Resolver Module

Implements the core Phase 118 primitive: `ferro_json_ui::resolve_expressions(&mut Spec)` with private helpers for `$data` and `$template` substitution. EXPR-01 and EXPR-02 are delivered end-to-end at the crate level; Plan 02 wires this into the framework pipeline.

## Files

| File | Change | LOC |
|------|--------|-----|
| `ferro-json-ui/src/expression.rs` | created | 424 (including inline `#[cfg(test)] mod tests`) |
| `ferro-json-ui/src/lib.rs` | +2 lines (module decl + re-export) | +2 |

No new dependencies added to `ferro-json-ui/Cargo.toml` (D-11 honored).

## Commits

| SHA | Scope |
|-----|-------|
| `2b0ee7a2` | `feat(118-01): add expression resolver module with 28 unit tests` — creates `expression.rs` + adds `pub mod expression;` to `lib.rs` |
| `1f40ce9f` | `feat(118-01): re-export resolve_expressions at ferro_json_ui crate root` — adds `pub use expression::resolve_expressions;` |

Task 1's commit carries the `pub mod expression;` line because the inline test gate cannot run until the module is part of the crate. Task 2 is scoped to the single-line re-export. Rustfmt placed the re-export alphabetically between `catalog` and `plugin` (lib.rs:69), which still satisfies the acceptance criterion that it precede `pub use resolve::...`.

## Decisions implemented

| Decision | Mechanism |
|----------|-----------|
| D-01 | Only `$data` and `$template` shipped; no `$if`/`$for`/`$state`/`$bind`/`$ref`/`$concat`/`$let` anywhere in touched files |
| D-02 | `$data` path resolves via `crate::data::resolve_path` preserving JSON type |
| D-03 | `$template` string produced via `crate::data::resolve_path_string`; placeholder syntax `{slash/path}`; escapes `\{`, `\}`, `\\` |
| D-04 | `resolve_expressions` walks only `el.props`; `spec.data`, `spec.title`, `spec.layout`, `el.children`, `el.action`, `el.visible` are untouched — verified by tests `does_not_touch_spec_data`, `does_not_touch_children`, `does_not_touch_visible` |
| D-05 | Missing `$data` path → `Value::Null`; missing `$template` placeholder → empty string |
| D-06 | Infallible resolver: malformed expressions (non-string value, sibling keys, null value) pass through literally — no panic, no log, no `Result` |
| D-07 | Single-pass: after replacing `*val`, walker does not recurse into the replacement — verified by `single_pass_no_recursion` |
| D-08 | Pipeline position documented in module doc-comment (runs after `resolve_actions`, before `Catalog::validate`); wiring is Plan 02's responsibility |
| D-11 | `resolve_path` / `resolve_path_string` remain `pub(crate)` — no visibility change, no new `pub` API, no new dependency |
| D-12 | All 28 test cases from the D-12 matrix implemented as individual `#[test]` fns |
| D-14 | Plugin-typed elements walk identically to built-in elements — verified by `plugin_props_walk_identically` |

## Test matrix (28 cases)

| # | Test fn | Domain | Status |
|---|---------|--------|--------|
| 1 | `data_simple_path` | `$data` success | pass |
| 2 | `data_nested_path` | `$data` success | pass |
| 3 | `data_array_index` | `$data` success | pass |
| 4 | `data_preserves_number` | `$data` type preservation | pass |
| 5 | `data_preserves_bool` | `$data` type preservation | pass |
| 6 | `data_preserves_object` | `$data` type preservation | pass |
| 7 | `data_preserves_array` | `$data` type preservation | pass |
| 8 | `data_missing_path` | `$data` missing → `Null` | pass |
| 9 | `data_non_string_value` | `$data` passthrough | pass |
| 10 | `data_sibling_keys` | `$data` passthrough | pass |
| 11 | `data_null_value` | `$data` passthrough | pass |
| 12 | `template_single_placeholder` | `$template` success | pass |
| 13 | `template_multiple_placeholders` | `$template` success | pass |
| 14 | `template_no_placeholder` | `$template` success | pass |
| 15 | `template_missing_placeholder` | `$template` missing → empty | pass |
| 16 | `template_whitespace_trimmed` | `$template` trim | pass |
| 17 | `template_escaped_open_brace` | `$template` escape | pass |
| 18 | `template_escaped_close_brace` | `$template` escape | pass |
| 19 | `template_escaped_backslash` | `$template` escape | pass |
| 20 | `template_unclosed_brace` | `$template` malformed | pass |
| 21 | `template_non_string_value` | `$template` passthrough | pass |
| 22 | `nested_in_array` | recursion | pass |
| 23 | `nested_in_object_values` | recursion | pass |
| 24 | `does_not_touch_spec_data` | scope | pass |
| 25 | `single_pass_no_recursion` | D-07 | pass |
| 26 | `does_not_touch_children` | scope | pass |
| 27 | `does_not_touch_visible` | scope | pass |
| 28 | `plugin_props_walk_identically` | D-14 | pass |

`cargo test --package ferro-json-ui --all-features -- expression::` → 28/28 pass.

## Gates

| Gate | Result |
|------|--------|
| `cargo test --package ferro-json-ui --all-features` | 390 unit + 11 + 8 + 5 doc all pass (pre-edit baseline plus 28 new expression tests — 362 pre-existing + 28 new = 390) |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --package ferro-json-ui --all-features --all-targets -- -D warnings` | clean |
| No forbidden sigils in touched files | `grep -nE '"\$(if\|for\|state\|bind\|ref\|concat\|let)"' ferro-json-ui/src/expression.rs ferro-json-ui/src/lib.rs` → empty |
| `resolve_path` / `resolve_path_string` visibility | still `pub(crate)` in `ferro-json-ui/src/data.rs` |
| `Cargo.toml` unchanged | `git diff ferro-json-ui/Cargo.toml` → empty |

## Deviations

None of substance. Rustfmt alphabetized the re-export line to position 69 (between `catalog` and `plugin`) rather than immediately above `resolve::...` as the plan text suggested; both placements satisfy the acceptance criterion that the line precede `pub use resolve::...`.

## Unblocks

- **Plan 118-02** — `framework/src/json_ui/mod.rs::JsonUi::resolve` and `JsonUi::resolve_with_errors` pipeline wiring.
- **Phase 119** — canonical parse → `resolve_actions` → `resolve_expressions` → `Catalog::validate` → `render` order for the page loader.
