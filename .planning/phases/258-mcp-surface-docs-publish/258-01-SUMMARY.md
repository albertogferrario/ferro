---
phase: 258-mcp-surface-docs-publish
plan: 01
subsystem: mcp
tags: [ferro-mcp, json-ui, generation-context, pos, register, design-rules]

requires:
  - phase: 257-projection-builder-register-layout-template
    provides: fill_viewport/each builder additions + register_template() helper + REGISTER_TRIGGER_TYPES
  - phase: 256-component-renderers-builtin-lockstep
    provides: TileGrid/SelectionPanel/FilterTabs/QuantityStepper/Numpad builtins (count=52)
  - phase: 255-pos-runtime-modules-double-submit-protection
    provides: runtime data-attribute vocabulary (data-qty-*, data-filter-*, data-numpad-*, data-disable-on-submit)
  - phase: 254-props-contracts-touch-foundation-design-rules
    provides: four register-* lint rules in design::rules() registry

provides:
  - BUILDER_API const documents fill_viewport(bool) and .each(path, as_) (Phase 257 additions)
  - RULE_COMPONENTS register-fill-viewport includes SelectionPanel and Numpad
  - RULE_COMPONENTS fill-viewport-layout-unknown maps to Grid
  - GenerationContext.register_composition field with six D-03 content items
  - RegisterCompositionGuidance and RegisterRuleRef public types (drift-guarded)
  - register_composition_drift_guard test pinning component names/rule ids/runtime attrs to authoritative sources
  - builder_api_mentions_fill_viewport_and_each test (Gap 1 guard)
  - SC-1 pre-existing evidence recorded (count=52, all five names, pre-satisfied in Phase 256)

affects:
  - 258-02 (docs/src component sections and register layout template docs)
  - 258-03 (CI-exact gate + publish)
  - gestiscilo register phase (pins ferro-rs 0.2.89 for agent-authoring context)

tech-stack:
  added: []
  patterns:
    - "derive lint rules from design::rules() by id-filter (not hand-copied) — RegisterRuleRef pattern"
    - "drift-guard via components_sorted() + design::rules() + FERRO_RUNTIME_JS.contains() assertions"
    - "static slice of annotated data-attribute strings for compact agent context"

key-files:
  created: []
  modified:
    - ferro-mcp/src/tools/json_ui_catalog.rs
    - ferro-mcp/src/tools/generation_context.rs

key-decisions:
  - "SC-1 recorded as pre-existing evidence (count=52 shipped in Phase 256) — no re-implementation"
  - "fill-viewport-layout-unknown maps to Grid (D-02 discretion) to surface app|dashboard constraint"
  - "components_sorted() used instead of .components (private field) for builtin lookup in drift guard"
  - "data_attributes as &'static [&'static str] with embedded role notes — simpler than DataAttributeInfo struct"

patterns-established:
  - "RegisterRuleRef derivation: filter design::rules() by id array, map to typed struct — mirrors IntentPattern"
  - "Drift guard for register guidance: three assertions (builtin names, rule ids, runtime attrs)"

requirements-completed: [POS-12]

duration: 25min
completed: 2026-07-06
---

# Phase 258 Plan 01: MCP Surface — json_ui_catalog + generation_context Register Guidance

**register_composition guidance on GenerationContext (six D-03 items, lint rules derived from design::rules()) + BUILDER_API fill_viewport/each additions + RULE_COMPONENTS SelectionPanel/Numpad gaps fixed, all drift-guarded**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-07-06T16:00:00Z
- **Completed:** 2026-07-06T16:25:00Z
- **Tasks:** 2
- **Files modified:** 2

## SC-1 Pre-Existing Evidence (World-State Correction 1)

`test_all_components_present` PASS recorded as pre-existing evidence — count=52, all five names
(TileGrid, FilterTabs, QuantityStepper, Numpad, SelectionPanel) shipped in Phase 256. This phase
did not re-implement the count assertion or name list. No count churn.

```
test tools::json_ui_catalog::tests::test_all_components_present ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 310 filtered out
```

## Accomplishments

- Extended `BUILDER_API` const string additively with `fill_viewport(bool) -> Self` (after `.build()`) and `.each(path, as_) -> Self` (after `.visible()`) — Phase 257 additions now documented for agent authoring
- Fixed `RULE_COMPONENTS`: `register-fill-viewport` gains SelectionPanel and Numpad (both are in REGISTER_TRIGGER_TYPES); `fill-viewport-layout-unknown` gains Grid with a clarifying comment
- Added `RegisterCompositionGuidance` and `RegisterRuleRef` structs to `generation_context.rs` (all fields carry `///` doc comments)
- Populated `register_composition` on `GenerationContext` with six D-03 content items: when_to_use, form_state_contract, data_attributes, fill_viewport_requirement, lint_rules (derived), template_helper
- Added `register_composition_drift_guard` test: component names vs builtins (`components_sorted()`), rule ids vs rule registry, key runtime attributes vs `FERRO_RUNTIME_JS`
- Added `builder_api_mentions_fill_viewport_and_each` test guarding Gap 1

## Task Commits

1. **Task 1: SC-1 evidence + BUILDER_API and RULE_COMPONENTS additive fixes** — `7a8f9472` (feat)
2. **Task 2: register_composition guidance on GenerationContext + drift guard** — `a105bdfd` (feat)

## Files Created/Modified

- `ferro-mcp/src/tools/json_ui_catalog.rs` — BUILDER_API + RULE_COMPONENTS additive fixes, new builder_api test
- `ferro-mcp/src/tools/generation_context.rs` — RegisterCompositionGuidance + RegisterRuleRef structs, register_composition field + derivation, extended section test + drift guard

## Decisions Made

- `SC-1` recorded as pre-existing evidence only — no re-implementation (D-01, world-state correction 1)
- `fill-viewport-layout-unknown` mapped to `["Grid"]` (D-02 discretion, Gap 3): Grid is the required root element in register/fill_viewport compositions; makes the app|dashboard layout constraint visible to authors
- Used `components_sorted()` instead of `.components` (private field) for the builtin-name lookup in the drift guard — [Rule 1] fix applied inline
- `data_attributes` typed as `&'static [&'static str]` with embedded role notes rather than a `DataAttributeInfo` struct — simpler and sufficient for compact inline agent context (D-04)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Used components_sorted() instead of private .components field**
- **Found during:** Task 2 (register_composition_drift_guard test compile)
- **Issue:** `ferro_json_ui::Catalog::components` is a private field; the drift guard test in the plan called it directly → compile error `E0616`
- **Fix:** Changed `global_catalog().components.iter()` to `global_catalog().components_sorted()` (public method returning `impl Iterator<Item = &ComponentSpec>`) and adjusted the `HashSet` type to `HashSet<String>`
- **Files modified:** ferro-mcp/src/tools/generation_context.rs
- **Verification:** `cargo test -p ferro-mcp -- register_composition_drift_guard` exits 0
- **Committed in:** a105bdfd (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 — compile error on private field access)
**Impact on plan:** No scope change; the fix is semantically identical (same component name set, same assertion).

## Issues Encountered

None beyond the private-field compile error documented above.

## Known Stubs

None. All six D-03 content items are populated with substantive content.

## Threat Flags

None. No new network endpoints, auth paths, file access patterns, or schema changes introduced. Both modified files are MCP tool output generation (read-only advisory context).

## Next Phase Readiness

- Plan 258-02 (docs/src component sections + register layout template docs) can proceed immediately
- `cargo doc --no-deps -p ferro-mcp` exits 0 with no missing-docs warnings (verified)
- `cargo fmt --all -- --check` exits 0

## Self-Check

---
*Phase: 258-mcp-surface-docs-publish*
*Completed: 2026-07-06*
