---
phase: 254-props-contracts-touch-foundation-design-rules
plan: "03"
subsystem: ferro-json-ui/ferro-mcp
tags: [design-lint, pos, rules, drift-guard]
dependency_graph:
  requires: []
  provides: [pos-fill-viewport rule, pos-grid-fill rule, pos-cart-present rule, fill-viewport-layout-unknown rule]
  affects: [ferro-json-ui/src/design/rules.rs, ferro-mcp/src/tools/json_ui_catalog.rs, docs/src/design-system/patterns.md]
tech_stack:
  added: []
  patterns: [internal-presence-gate rules, all-intents design rules, three-fixture test structure]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/design/rules.rs
    - ferro-mcp/src/tools/json_ui_catalog.rs
    - docs/src/design-system/patterns.md
decisions:
  - Four POS rules use intents:&[] with internal presence/fill_viewport gates, not intent-keyed (D-11 — mirrors page-header pattern)
  - RULE_COMPONENTS maps pos-* rules to Grid (register-root builtin) and fill-viewport-layout-unknown to empty-slice; Phase 256 extends to ProductGrid/CartPanel/Numpad (D-14 handoff)
  - Component count stays at 47 — no BUILTIN_TYPES/dispatch/BUILTIN_SPECS changes (D-15)
metrics:
  duration: "330s (~6m)"
  completed: "2026-07-05"
  tasks: 2
  files: 3
---

# Phase 254 Plan 03: POS Design-Lint Rules Summary

Four POS design-lint rules with 12 fixtures shipped: `pos-fill-viewport`, `pos-grid-fill`, `pos-cart-present`, `fill-viewport-layout-unknown` — all-intents Warning-severity rules with internal gates, patterns.md sections, and RULE_COMPONENTS entries, both external drift guards green.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Four POS rules + RULE_REGISTRY + patterns.md + RULE_COMPONENTS | b0ab2f9b | rules.rs, patterns.md, json_ui_catalog.rs |
| 2 | Twelve fixtures (4 rules × violating/conforming/data-bound) | d58ac509 | rules.rs |

## What Was Built

### Task 1 — Four POS rules, patterns.md, RULE_COMPONENTS

Added four `DesignRule` entries to `RULE_REGISTRY` (11 → 15 rules):

- **`pos-fill-viewport`**: fires when any POS component type name (ProductGrid/CartPanel/Numpad) is present but `fill_viewport` is false — prevents silent whole-page scroll on register pages.
- **`pos-grid-fill`**: fires when `fill_viewport` is true but the spec's root element is a Grid lacking `fill:true` — prevents panes losing internal scroll.
- **`pos-cart-present`**: fires when a ProductGrid is present but no CartPanel exists anywhere — incomplete register composition.
- **`fill-viewport-layout-unknown`**: fires when `fill_viewport` is true but `layout` is outside `{app, dashboard}` — the ferro-fill CSS chain only supports those two; reuses `is_app_shell_layout()`.

All four use `intents: &[]` (all-intents) with internal presence/fill_viewport gates, mirroring the `page-header` pattern (D-11). All emit `Severity::Warning`.

Added `POS_TRIGGER_TYPES: &[&str]` constant for the presence check in `pos-fill-viewport`.

Added four sections to `docs/src/design-system/patterns.md` with the exact `` ## `rule-id` `` header format the D-09 drift guard parses. Each section includes title, rationale, intents, conforming example, violating example, and how-to-allow.

Added four entries to `RULE_COMPONENTS` in `ferro-mcp/src/tools/json_ui_catalog.rs`:
- `pos-fill-viewport` → `&["Grid"]`
- `pos-grid-fill` → `&["Grid"]`
- `pos-cart-present` → `&["Grid"]`
- `fill-viewport-layout-unknown` → `&[]` (no per-component guidance; empty-slice passes Direction 3 unconditionally)

Both external drift guards passed immediately:
- `patterns_md_matches_rule_registry` (D-09 bidirectional guard)
- `design_system_component_guidance_drift_guarded` (Direction 1/2/3)

### Task 2 — Twelve fixtures

Added 12 `#[test]` functions to `mod tests` in `rules.rs` — three per rule:

| Rule | Violating | Conforming | Data-bound |
|------|-----------|------------|------------|
| pos-fill-viewport | ProductGrid + no fill_viewport → 1 Warning | fill_viewport:true + ProductGrid → 0 | DataTable only (no POS types) → 0 |
| pos-grid-fill | fill_viewport + root Grid no fill → 1 Warning | fill_viewport + root Grid fill:true → 0 | fill_viewport + Grid fill:true + data-bound child → 0 |
| pos-cart-present | ProductGrid only → 1 Warning | ProductGrid + CartPanel → 0 | ProductGrid + CartPanel with $data props → 0 |
| fill-viewport-layout-unknown | fill_viewport + layout:auth → 1 Warning | fill_viewport + layout:app → 0 | fill_viewport + layout:app + data-bound element → 0 |

Every violating case asserts `findings.len() == 1` and `severity == Severity::Warning`. Every conforming and data-bound case asserts `findings.is_empty()`.

## Verification

- `cargo test -p ferro-json-ui design` — 62 passed (50 pre-existing + 12 new), 0 failed. Includes `patterns_md_matches_rule_registry`.
- `cargo test -p ferro-mcp design_system_component_guidance_drift_guarded` — 1 passed. Direction 1/2/3 all green.
- `cargo fmt --all -- --check` — clean.
- `cargo clippy -p ferro-json-ui -p ferro-mcp --all-targets -- -D warnings` — clean.
- Component count: 47 (unchanged, D-15 preserved).

## Deviations from Plan

None — plan executed exactly as written. `cargo fmt` required one post-edit run to normalize line wrapping in two `message:` continuations and `has_cart` chain formatting; no logic changes.

## Known Stubs

None. All four rules are fully implemented check functions with working fixture coverage. RULE_COMPONENTS maps to Grid (existing builtin) until Phase 256 registers ProductGrid/CartPanel/Numpad — this is a named handoff (D-14), not a stub.

## Threat Flags

None. Rules are pure diagnostic functions over parsed Spec fields; no network, no runtime, no end-user input path.

## Self-Check: PASSED

- ferro-json-ui/src/design/rules.rs — 4 new rules in RULE_REGISTRY, 4 check functions, 12 test fixtures: confirmed via grep and cargo test.
- docs/src/design-system/patterns.md — 4 new sections with correct header format: confirmed via grep (10 occurrences).
- ferro-mcp/src/tools/json_ui_catalog.rs — 4 RULE_COMPONENTS entries: confirmed via grep.
- Commits b0ab2f9b and d58ac509 verified in git log.
