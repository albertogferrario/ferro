---
phase: 175-json-ui-v2-runtime-patches-staff-domain-field-test
plan: "05"
subsystem: ferro-json-ui
tags: [json-ui, switch, documentation, regression-test, f4]
dependency_graph:
  requires: [175-01, 175-04, 175-06]
  provides: [switch-depth-8-regression-test, switch-docs]
  affects: [ferro-json-ui, docs]
tech_stack:
  added: []
  patterns: [depth-regression-test, Spec::builder-for-integration-test]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/render/form.rs
    - docs/src/json-ui/components.md
decisions:
  - "D-F4 = docs-only + depth-8 regression test (no new component surface, no variant field on CheckboxProps)"
  - "Use Spec::builder() in test so DepthExceeded fires at build() time if MAX_NESTING_DEPTH is reverted below 8"
metrics:
  duration: "~7min"
  completed: "2026-05-20"
  tasks_completed: 2
  files_modified: 2
---

# Phase 175 Plan 05: Switch Depth-8 Regression Test + Documentation Summary

One-liner: depth-8 Switch regression test pinning F1+F4 closure, plus Switch documentation with toggle semantics and Checkbox-styled-as-switch substitution path.

## Objective

Close F4 (Switch does not render in consumer's staff-detail surface) by confirming that F1's MAX_NESTING_DEPTH bump from 5 to 16 is sufficient — Switch was already registered and dispatched in v2; the consumer's symptom was depth-stripping, not a missing component. No new component surface was introduced.

## Tasks Completed

### Task 1: Add switch_at_depth_8_renders_role_switch regression test
**Commit:** fb6053cf

Added `switch_at_depth_8_renders_role_switch` to `ferro-json-ui/src/render/form.rs` tests module.

The test builds a Spec with seven chained Grid containers (depths 1–7) ending in a Switch leaf at depth 8 using `Spec::builder()`. Because `Spec::builder()` runs `validate_structure` at `.build()` time, a `DepthExceeded` error at that call site signals that MAX_NESTING_DEPTH was reverted below 8 — not just a runtime rendering failure. The test then calls `render_spec_to_html` and asserts:

- `role="switch"` is present (Switch rendered correctly)
- `"depth limit exceeded"` is absent (walker did not strip the element)
- `"cycle guard tripped"` is absent (no deprecated diagnostic emitted)

The test passes green because 175-01 already raised MAX_NESTING_DEPTH to 16.

### Task 2: Expand Switch documentation
**Commit:** 916cce58

Rewrote the Switch section in `docs/src/json-ui/components.md`:

- Description now names the semantic: state-flip toggle (on/off, open/closed, enabled/disabled), distinct from Checkbox (binary choice within a set of options). Notes that the renderer emits `role="switch"` and `aria-checked`.
- Props table expanded with `action` (auto-submit form wrap) and `compact` (scale-75 for dense grid layouts).
- Worked example updated to show `data_path`, `compact`, and `action` together (mirrors the consumer's per-day Orari toggle pattern).
- Added "Substitution: Checkbox styled as switch" subsection per D-F4-Switch decision: describes the visual-only substitution via Tailwind utilities, notes there is no `variant: "switch"` prop on Checkbox today.

## Deviations from Plan

None. Plan executed exactly as written.

- Switch was already in BUILTIN_TYPES (confirmed from 175-RESEARCH.md F4; no change made).
- No `variant` field added to CheckboxProps (scope guardrail intact per planner directive).
- Formatting fix applied: rustfmt reformatted the chained `.element()` calls across multiple lines; committed as part of Task 1 after fmt check flagged it.

## Verification

All acceptance criteria met:

```
grep -q 'fn switch_at_depth_8_renders_role_switch' ferro-json-ui/src/render/form.rs  # PASS
cargo test -p ferro-json-ui switch_at_depth_8_renders_role_switch                    # PASS (1/1)
grep -q 'role="switch"' docs/src/json-ui/components.md                               # PASS
grep -qi 'switch' docs/src/json-ui/components.md                                     # PASS
grep -q 'variant.*switch\|switch.*variant' docs/src/json-ui/components.md            # PASS
cargo fmt --all -- --check                                                            # PASS
cargo clippy --all --all-targets -- -D warnings                                       # PASS
cargo test --all-features                                                             # PASS (zero failures)
```

## Known Stubs

None.

## Threat Flags

None. No new network endpoints, auth paths, file access patterns, or schema changes introduced. The test exercises the existing public render API with a hand-built fixture.

## Self-Check: PASSED

- `ferro-json-ui/src/render/form.rs` — modified, contains `fn switch_at_depth_8_renders_role_switch`
- `docs/src/json-ui/components.md` — modified, contains `role="switch"` and `variant.*switch`
- Commits fb6053cf and 916cce58 exist in git log
