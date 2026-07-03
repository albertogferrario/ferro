---
phase: 252
plan: 04
subsystem: ferro-json-ui
tags: [design-lint, rules, tdd, design-module]
requirements: [DS-05]

dependency_graph:
  requires:
    - RULE_REGISTRY (5 rules from Plan 03)
    - ferro_json_ui::design::lint
    - ferro_json_ui::design::DesignRule
    - ferro_json_ui::design::Finding
    - ferro_json_ui::design::Severity
    - ferro_json_ui::spec::Spec
    - ferro_json_ui::action::Action (confirm field)
  provides:
    - RULE_REGISTRY (10 rules: all anchor-spec rules complete)
    - process-kanban, card-actions-in-menu, create-separate-page
    - form-default-values, destructive-confirmation
    - 44 design rule tests total (batch A 30 + batch B 14)
  affects:
    - ferro-json-ui/src/design/rules.rs (5 rules + 14 tests added)

tech_stack:
  added: []
  patterns:
    - TDD per-task (RED commit -> GREEN commit)
    - fn-pointer rule registry (same pattern as batch A)
    - FIELD_TYPES const slice for form-field type detection
    - labeled break ('outer) for de-duplicated props-embedded confirmation scan
    - $data binding detection via .get("$data") for edit-form heuristic

key_files:
  created: []
  modified:
    - ferro-json-ui/src/design/rules.rs

decisions:
  - "form-default-values uses $data-bound default_value as the edit-form signal;
    pure create forms (no field has a $data default_value) are exempt — login.json shape"
  - "destructive-confirmation de-duplicates per element: element-level Button check
    runs first, then props-embedded (row_actions/items); one Finding per element max"
  - "FIELD_TYPES const defined at module level for reuse clarity and easy extension"
  - "labeled break 'outer exits both key loop and entry loop after first violation
    in props-embedded scan — avoids double-finding per element"

metrics:
  duration: 325s (~5m)
  completed: 2026-07-03T18:41:00Z
  tasks: 2
  files: 1
---

# Phase 252 Plan 04: 5 batch-B design rules Summary

5 final anchor-spec rules registered in `RULE_REGISTRY`: `process-kanban`,
`card-actions-in-menu`, `create-separate-page`, `form-default-values`,
`destructive-confirmation`. RULE_REGISTRY now holds all 10 rules. The
`form-default-values` and `destructive-confirmation` rules encode the CLAUDE.md
form-default-value discipline and destructive-action confirmation as
machine-checkable code.

## Tasks Completed

| Task | Name | Commits | Files |
|------|------|---------|-------|
| 1 RED | Failing tests: process-kanban, card-actions-in-menu, create-separate-page | c3ef2aff | rules.rs |
| 1 GREEN | Implement 3 process/collect rules | f74e4983 | rules.rs |
| 2 RED | Failing tests: form-default-values, destructive-confirmation | bb1248e3 | rules.rs |
| 2 GREEN | Implement 2 rules; RULE_REGISTRY complete (10 rules) | 1cdcbf7f | rules.rs |

## Rules Registered (batch B)

| Rule ID | Intents | Key signal | De-duplication |
|---------|---------|------------|----------------|
| `process-kanban` | process | no KanbanBoard in spec.elements | n/a (one finding) |
| `card-actions-in-menu` | process | destructive row_action not at last index | one warning per offending KanbanBoard |
| `create-separate-page` | collect | Modal element with a Form child | one warning per offending Modal |
| `form-default-values` | collect | edit-form detected ($data default_value) + field missing default_value | one warning per offending field |
| `destructive-confirmation` | all | destructive Button w/o confirm; destructive row_action/item w/o confirm | one warning per element (element-level first, props-embedded second) |

## Verification

- `cargo test -p ferro-json-ui design` — 44 tests pass
- `grep -c "check: check_" ferro-json-ui/src/design/rules.rs` = 10
- `grep -c 'id: "' ferro-json-ui/src/design/rules.rs` = 10
- `cargo doc -p ferro-json-ui --no-deps` — zero warnings
- `cargo clippy -p ferro-json-ui --all-targets -- -D warnings` — clean
- `cargo fmt --all -- --check` — clean
- Pure-create-form guard: `form_default_values_conforming_pure_create_form_login_shape` test passes

## Deviations from Plan

None — plan executed exactly as written. All 5 rules implemented with the specified
check logic and verbatim metadata. TDD flow followed strictly (RED then GREEN per task).

## Known Stubs

None — all 10 rules are fully implemented with violating/conforming test coverage.
No placeholder findings or deferred check logic.

## Threat Flags

No new network endpoints, auth paths, or file access patterns. All check fns
are pure in-process computation on the `Spec` struct.

T-252-02 (DoS via malformed props) mitigated in all 5 new rules:
- `check_process_kanban`: no prop access, just element type scan
- `check_card_actions_in_menu`: `.get("row_actions").and_then(|v| v.as_array())` + index bound via `.len() - 1` comparison
- `check_create_separate_page`: element children Vec iteration with `.get()` guard
- `check_form_default_values`: `.get("default_value").and_then(|v| v.get("$data"))` chains
- `check_destructive_confirmation`: `.get("variant").and_then(|v| v.as_str())`, `.get("destructive").and_then(|v| v.as_bool())`, `.get("confirm").is_none()` — all graceful None paths

## Self-Check: PASSED

- `ferro-json-ui/src/design/rules.rs` — FOUND
- Commit c3ef2aff (Task 1 RED) — FOUND
- Commit f74e4983 (Task 1 GREEN) — FOUND
- Commit bb1248e3 (Task 2 RED) — FOUND
- Commit 1cdcbf7f (Task 2 GREEN) — FOUND
- `grep -c "check: check_" ferro-json-ui/src/design/rules.rs` = 10 — VERIFIED
- `grep -c 'id: "' ferro-json-ui/src/design/rules.rs` = 10 — VERIFIED
- 44 design tests pass — VERIFIED
