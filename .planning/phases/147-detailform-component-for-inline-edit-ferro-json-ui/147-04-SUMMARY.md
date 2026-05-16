---
phase: 147
plan: 04
subsystem: ferro-json-ui
tags: [resolver, url-resolution, validation-errors, detail-form]
wave: 1
depends_on: [01]
requires:
  - "Plan 01 resolver RED tests (resolve_detail_form_action, resolve_does_not_touch_edit_or_cancel_url, resolve_errors_propagates_into_detail_form_fields)"
  - "Plan 02 (Wave 1 parallel) adds Component::DetailForm variant + DetailFormProps/DetailField/EditMode types in ferro-json-ui/src/component.rs"
provides:
  - "Three Component::DetailForm resolver arms in ferro-json-ui/src/resolve.rs — URL resolution, unresolved collection, validation error mapping"
  - "D-16 invariant: edit_url and cancel_url are never mutated by the resolver"
affects:
  - "ferro-json-ui URL resolution pass for DetailForm.action"
  - "ferro-json-ui strict-mode unresolved handler detection for DetailForm.action"
  - "ferro-json-ui validation error propagation into DetailForm.fields[].input"
tech-stack:
  added: []
  patterns:
    - "Three-pass resolver participation (147-PATTERNS §Pattern S-1, 147-RESEARCH Pattern 5)"
    - "Container arm + leaf catch-all exclusion (Pitfall 1: silent absorption by catch-all)"
key-files:
  created: []
  modified:
    - ferro-json-ui/src/resolve.rs
decisions:
  - "D-15 applied: Component::DetailForm(props) participates in resolver pass like Component::Form — resolve_action on props.action, recurse into children"
  - "D-16 proven: edit_url and cancel_url never appear in any resolver arm; raw strings pass through unchanged"
  - "DetailField.input navigation: recursion target is &[mut] field.input (a ComponentNode), not field itself — one-line difference from FormProps.fields iteration"
  - "Pass 3 (resolve_errors_node) follows the Component::Form shape (recurse only), NOT the Component::KeyValueEditor shape (which writes to a component-level error slot) — DetailFormProps has no component-level error field"
metrics:
  duration: "~8 minutes"
  tasks_completed: "1/1"
  files_modified: 1
  lines_inserted: 17
  completed: "2026-04-23"
---

# Phase 147 Plan 04: Resolver arms for Component::DetailForm Summary

Three surgical insertions into `ferro-json-ui/src/resolve.rs`, each mirroring the
pre-existing `Component::Form` arm in the same match block. Turns the three
Plan 01 Task 3 RED resolver tests GREEN once Wave 1's Plan 02 lands the
`Component::DetailForm` variant in `component.rs`.

## One-liner

Adds Component::DetailForm arms to resolve_component_node, collect_unresolved_node, and resolve_errors_node — DetailForm participates in URL resolution and validation-error propagation like Form, with `edit_url` and `cancel_url` proven untouched per D-16.

## Inserted arms

| Pass | Function | Location in file | Recursion target | Action call |
|------|----------|-------------------|------------------|-------------|
| 1    | `resolve_component_node`   | inserted at L52 (immediately after the Form arm at L46–L51)     | `resolve_component_node(&mut field.input, resolver)` | `resolve_action(&mut props.action, resolver)` |
| 2    | `collect_unresolved_node`  | inserted at L231 (immediately after the Form arm at L225–L230)  | `collect_unresolved_node(&field.input, unresolved)`  | `collect_unresolved_action(&props.action, unresolved)` |
| 3    | `resolve_errors_node`      | inserted at L416 (immediately after the Form arm at L411–L415)  | `resolve_errors_node(&mut field.input, errors, all)` | none (D-05 / pass 3 does not resolve actions) |

### Exact emission

```rust
// Pass 1 — resolve_component_node (L52)
Component::DetailForm(props) => {
    resolve_action(&mut props.action, resolver);
    for field in &mut props.fields {
        resolve_component_node(&mut field.input, resolver);
    }
}

// Pass 2 — collect_unresolved_node (L231)
Component::DetailForm(props) => {
    collect_unresolved_action(&props.action, unresolved);
    for field in &props.fields {
        collect_unresolved_node(&field.input, unresolved);
    }
}

// Pass 3 — resolve_errors_node (L416)
Component::DetailForm(props) => {
    for field in &mut props.fields {
        resolve_errors_node(&mut field.input, errors, all);
    }
}
```

## Acceptance criteria results

| Criterion | Expected | Actual | Status |
|-----------|----------|--------|--------|
| Production arms of the shape `Component::DetailForm(props) => {` in resolve.rs (lines < 478, pre-test region) | exactly 3 | 3 (L52, L231, L416) | PASS |
| Pass 1 window L46–60 contains `resolve_component_node(&mut field.input` | exactly 1 | 1 | PASS |
| Pass 2 window L225–240 contains `collect_unresolved_node(&field.input`   | exactly 1 | 1 | PASS |
| Pass 3 window L411–425 contains `resolve_errors_node(&mut field.input`   | exactly 1 | 1 | PASS |
| DetailForm in any leaf `|`-chain catch-all                                | 0         | 0 | PASS |
| `props.edit_url` or `props.cancel_url` referenced in any resolver arm     | 0         | 0 | PASS (D-16 invariant — these fields appear only in test assertions, lines 1187+) |
| `cargo fmt --all -- --check`                                              | exit 0    | exit 0 | PASS |

## D-16 proof

The negative check `awk 'NR < 478' resolve.rs | grep -cE 'props\.edit_url|props\.cancel_url'`
returns `0`. The only occurrences of `edit_url` / `cancel_url` in the file are inside
the `#[cfg(test)] mod tests` block (lines 1187, 1188, 1217, 1224, 1225, 1240–1245, 1282, 1283)
— the production resolver code never touches them. Plan 01's
`resolve_does_not_touch_edit_or_cancel_url` test will assert this end-to-end once
the file compiles.

## Leaf-catch-all negative verification

`grep -E '^\s*\|?\s*Component::DetailForm\(_\)' ferro-json-ui/src/resolve.rs` returns no matches.
DetailForm is handled exclusively by the three explicit container arms above — never by
the trailing `|`-chain catch-all in any of the three functions. This protects against
Pitfall 1 (silent absorption without a compile error) identified in 147-RESEARCH.md.

## Critical gotchas honored

1. **Recursion target navigation.** `DetailFormProps.fields: Vec<DetailField>` — not
   `Vec<ComponentNode>` like `FormProps.fields`. Each recursive call therefore targets
   `&[mut] field.input` (the `ComponentNode` inside `DetailField`), not `field` itself.
   This is the one-line difference from the Form arm (147-PATTERNS.md §3a gotcha).

2. **Pass 3 shape.** `resolve_errors_node`'s DetailForm arm follows the `Component::Form`
   shape (recurse only), NOT the `Component::KeyValueEditor` shape at L472–474 (which
   calls `set_field_error` on a component-level `props.error` field). `DetailFormProps`
   has no component-level error slot; errors propagate into `field.input` via recursion,
   where `Input`/`Select`/`Checkbox`/`Switch` leaf arms own their own error handling.

3. **Parameter names matched.** The third function signature is
   `resolve_errors_node(node: &mut ComponentNode, errors: &HashMap<String, Vec<String>>, all: bool)`.
   The new arm uses `errors, all` — matching the existing Form arm (L399–403).

## Plan 01 Task 3 test status

The three Plan 01 RED tests that this plan flips GREEN:

| Test | Location | Flips GREEN when |
|------|----------|------------------|
| `resolve_detail_form_action`                         | L1162–1197 | this plan + Plan 02 both committed |
| `resolve_does_not_touch_edit_or_cancel_url`          | L1199–1233 | this plan + Plan 02 both committed |
| `resolve_errors_propagates_into_detail_form_fields`  | L1235–1296 | this plan + Plan 02 both committed |

**Compile status at time of this plan's completion:** `cargo check -p ferro-json-ui --lib` fails
with `E0599: no variant or associated item named 'DetailForm' found for enum Component` —
expected Wave 1 parallel state. The file compiles once Plan 02 (parallel Wave 1) lands the
`Component::DetailForm(DetailFormProps)` variant and the `DetailFormProps` / `DetailField`
/ `EditMode` types. Verification through `cargo test -p ferro-json-ui --lib resolve::tests`
must be deferred to the Wave 1 integration / Wave 2 verifier; it is not runnable from this
plan in isolation. This is the documented wave_1_green_expectation from the executor prompt:
"Until 147-02 lands the Component::DetailForm variant, your resolve.rs changes may not
compile in isolation. That's expected."

## Deviations from Plan

None — the plan executed exactly as written. Scope fence respected:
- Only `ferro-json-ui/src/resolve.rs` modified
- No changes to `component.rs` (Plan 02), `render.rs` (Plan 03), or docs / catalog (Plan 05)
- No new helper functions, no new imports
- Each arm is a mechanical mirror of the corresponding `Component::Form` arm

## Authentication gates

None encountered.

## Deferred Issues

None.

## Threat-register alignment

- **T-147-03 (Tampering — Action URL resolution):** Mitigated. Pass 1's DetailForm arm
  delegates URL resolution to `resolve_action(&mut props.action, resolver)` — the same
  function Form uses. No code path bypasses the resolver callback.
- **T-147-04 (Tampering — Error propagation path):** Mitigated. Pass 3's DetailForm arm
  recurses `resolve_errors_node(&mut field.input, errors, all)`. Each `field.input` owns
  its own error slot via the existing pipeline; no shared-mutable-state side-channel.
- **T-147-05 (Spoofing — raw edit_url/cancel_url):** Accepted per D-16. Preservation
  asserted by `resolve_does_not_touch_edit_or_cancel_url` (Plan 01).

No new threat surface introduced.

## Commits

| # | Hash       | Message |
|---|------------|---------|
| 1 | `bb4db79c` | feat(147-04): add Component::DetailForm arms to all three resolver passes |

## Self-Check: PASSED

- FOUND: ferro-json-ui/src/resolve.rs (modified)
- FOUND: commit bb4db79c in git log
- FOUND: 3 production arms at L52, L231, L416 — mirrored shape verified
- FOUND: 0 DetailForm occurrences in any leaf catch-all
- FOUND: 0 edit_url / cancel_url references in resolver production arms
- FOUND: cargo fmt --all -- --check exits 0
