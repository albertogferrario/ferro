---
phase: 252
plan: 03
subsystem: ferro-json-ui
tags: [design-lint, rules, tdd, design-module]
requirements: [DS-05]

dependency_graph:
  requires:
    - ferro_json_ui::design::lint         # Plan 01
    - ferro_json_ui::design::DesignRule   # Plan 01
    - ferro_json_ui::design::Finding      # Plan 01
    - ferro_json_ui::design::Severity     # Plan 01
    - ferro_json_ui::spec::Spec           # Plan 01
  provides:
    - RULE_REGISTRY (5 rules: page-header, prefer-data-table, list-empty-state,
      row-actions-grouped, breadcrumb-on-subpages)
    - is_app_shell_layout() layout-gate helper
    - 30 design rule tests (10 violating + 10 conforming + 10 layout/intent guard cases)
  affects:
    - ferro-json-ui/src/design/rules.rs (populated from empty stub)
    - ferro-json-ui/src/design/mod.rs (engine test helpers updated)

tech_stack:
  added: []
  patterns:
    - TDD per-task (RED commit → GREEN commit)
    - fn-pointer rule registry (zero-cost iteration, static lifetimes)
    - layout-gated check functions (is_app_shell_layout helper)
    - intent-keyed rules via RULE_REGISTRY.intents slices

key_files:
  created: []
  modified:
    - ferro-json-ui/src/design/rules.rs
    - ferro-json-ui/src/design/mod.rs

decisions:
  - "5 rules implemented as fn-pointer check fns registered in RULE_REGISTRY;
    is_app_shell_layout() gates page-header and breadcrumb-on-subpages"
  - "All check fns are defensive: .get()/.and_then()/.unwrap_or() — never panic
    on arbitrary spec props (T-252-02 threat mitigation)"
  - "Auth layout + layout-absent specs exempt from page-header and
    breadcrumb-on-subpages by is_app_shell_layout check (D-14)"
  - "breadcrumb-on-subpages accepts either a Breadcrumb element OR a PageHeader
    with a non-empty props.breadcrumb array"
  - "Engine test helpers in mod.rs updated to use conforming specs (DataTable
    with empty_message) after list-empty-state was registered — Rule 1 auto-fix"

metrics:
  duration: 10m
  completed: 2026-07-03T18:20:00Z
  tasks: 2
  files: 2
---

# Phase 252 Plan 03: 5 batch-A design rules Summary

5 intent-keyed composition rules registered in `RULE_REGISTRY` with check fns
and violating/conforming test pairs. The layout-gated rules (`page-header`,
`breadcrumb-on-subpages`) fire only on app-shell layouts so auth pages stay clean.

## Tasks Completed

| Task | Name | Commits | Files |
|------|------|---------|-------|
| 1 RED | Failing tests: page-header, prefer-data-table, list-empty-state | dada250d | rules.rs |
| 1 GREEN | Implement 3 rules + fix engine tests | 92a53371 | rules.rs, mod.rs |
| 2 RED | Failing tests: row-actions-grouped, breadcrumb-on-subpages | e6491f6d | rules.rs |
| 2 GREEN | Implement 2 rules | 4b289f28 | rules.rs |

## Rules Registered (batch A)

| Rule ID | Intents | Layout gate | Key signal |
|---------|---------|-------------|------------|
| `page-header` | all | dashboard/app only | missing PageHeader or PageHeader without title |
| `prefer-data-table` | browse | none | raw `Table` element present |
| `list-empty-state` | browse | none | DataTable/MediaCardGrid without `empty_message` and no EmptyState element |
| `row-actions-grouped` | browse, process | none | element with ≥2 Button children |
| `breadcrumb-on-subpages` | collect, focus | dashboard/app only | no Breadcrumb element and no PageHeader with `breadcrumb` array |

## Verification

- `cargo test -p ferro-json-ui design` — 30 tests pass (10 violating + 10 conforming
  + engine tests + inference tests)
- `cargo test -p ferro-json-ui` — 696 total tests, 0 failures
- `cargo fmt --all -- --check` — clean
- `cargo clippy -p ferro-json-ui --all-targets --all-features -- -D warnings` — clean
- `cargo doc -p ferro-json-ui --no-deps` — zero warnings

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] 4 engine tests in mod.rs assumed empty RULE_REGISTRY**
- **Found during:** Task 1 GREEN
- **Issue:** `undeclared_intent_with_data_table_emits_info_browse`,
  `unknown_declared_intent_emits_warning`, `unknown_allow_id_emits_warning`,
  `valid_declared_intent_with_data_table_zero_findings` all used DataTable specs
  without `empty_message`, which now fire `list-empty-state` → wrong finding counts
- **Fix:** Updated `spec_with` helper to include `empty_message` for DataTable
  case; updated `spec_with_design` helper similarly; changed
  `unknown_allow_id_emits_warning` to use Form + focus intent (no browse-keyed rules)
  to isolate the allow-id mechanism; rewrote `valid_declared_intent_with_data_table_zero_findings`
  inline with conforming DataTable
- **Files modified:** ferro-json-ui/src/design/mod.rs
- **Commit:** 92a53371

**2. [Rule 1 - Bug] Premature all-5-rules implementation reversed for TDD compliance**
- **Found during:** Task 1 GREEN — accidentally added rules 4-5 before Task 2 RED tests existed
- **Fix:** Reverted to 3-rule registry for Task 1 GREEN commit; removed the two
  premature check functions; restored proper TDD flow (Task 2 RED tests fail → Task 2
  GREEN adds rules)
- **Files modified:** ferro-json-ui/src/design/rules.rs
- **Commits:** 92a53371, e6491f6d, 4b289f28

## Known Stubs

None — all 5 rules are fully implemented with violating/conforming coverage. No
placeholder findings, hardcoded empty values, or deferred check logic.

## Threat Flags

No new network endpoints, auth paths, or file access patterns. All check fns
are pure in-process computation on the `Spec` struct.

T-252-02 (DoS via malformed props) mitigated: every prop access uses
`.get()/.and_then()/.as_str()/.as_array()` chains with `.unwrap_or()` defaults —
zero panicking access paths.

## Self-Check: PASSED

- `ferro-json-ui/src/design/rules.rs` — FOUND
- `ferro-json-ui/src/design/mod.rs` — FOUND (engine tests updated)
- Commit dada250d (Task 1 RED) — FOUND
- Commit 92a53371 (Task 1 GREEN) — FOUND
- Commit e6491f6d (Task 2 RED) — FOUND
- Commit 4b289f28 (Task 2 GREEN) — FOUND
- `grep -c "check: check_" ferro-json-ui/src/design/rules.rs` = 5 — VERIFIED
- `grep -q "fn is_app_shell_layout"` — VERIFIED
- 30 design tests pass — VERIFIED
