---
phase: 164
plan: "12"
subsystem: planning
tags: [completed, v12.0, friction-loop-closure, audit, docs]
dependency_graph:
  requires: [164-01, 164-02, 164-03, 164-04, 164-05, 164-06, 164-07, 164-08, 164-09, 164-10, 164-11]
  provides: [COMPLETED.md, Phase-160-unblocked, Phase-161-CHANGELOG-input]
  affects: [Phase 160 (v1 deletion gate), Phase 161 (v12.0 merge + publish)]
tech_stack:
  added: []
  patterns: [completion-summary doc, audit-embed, five-section structure]
key_files:
  created:
    - .planning/phases/164-json-ui-improvements-batch-3-documenti-field-test-findings-m/COMPLETED.md
  modified: []
decisions:
  - "Embedded V1-DELETION-AUDIT.md table verbatim (25 rows) as Section 5 — no summarization"
  - "phrase 'placeholder' at line 34 is a technical term (URL placeholder substitution), not a document gap"
  - "LoadError::Catalog cleanup deferred item added beyond CONTEXT list — surfaced in Plan 07 SUMMARY"
  - "$if vs visible timing gap added to deferred list — architectural note from Plan 07"
metrics:
  duration: "~25 min"
  completed: "2026-05-17"
  tasks: 1
  files: 1
---

# Phase 164 Plan 12: COMPLETED.md — v12.0 Friction Loop Closure Summary

**One-liner:** `COMPLETED.md` authored with all five required sections, zero placeholders, zero trigger phrases, Phase 160 unblocked statement unambiguous; closes D-10 and D-11.

## What Was Built

`COMPLETED.md` at `.planning/phases/164-json-ui-improvements-batch-3-documenti-field-test-findings-m/COMPLETED.md`.

Single document summarizing every improvement shipped across Phases 162, 163, 163.1, and 164 — the full v12.0 JSON-UI friction loop. Five required sections per D-10:

### Section sizes (line counts)

| Section | Line range | Lines |
|---------|------------|-------|
| 1. Shipped across Phases 162–164 | 15 – 125 | 111 |
| 2. Runtime frictions resolved (F1–F10) | 126 – 142 | 17 |
| 3. Intentional gaps | 143 – 164 | 22 |
| 4. Deferred to future milestones | 165 – 178 | 14 |
| 5. v1 → v2 surface migration table | 179 – 218 | 40 |
| Handoff | 219 – 223 | 5 |
| **Total** | | **223** |

## Acceptance Criteria Results

| Check | Result |
|-------|--------|
| File exists | PASS |
| Five required sections present | PASS (5/5) |
| "Phase 160 (v1 deletion) is UNBLOCKED" present | PASS (2 occurrences) |
| F1 covered in Section 2 | PASS |
| F10 covered in Section 2 | PASS |
| Trigger phrases ("killer feature", "load-bearing weakness", etc.) | PASS — 0 matches |
| No document-gap placeholder text | PASS — 0 unintended placeholders (1 technical use of "placeholder" in DataTable URL description) |

## Deferred Items Added Beyond CONTEXT-Listed Set

The CONTEXT D-10 listed these deferred candidates:
- Host-based tenancy gap
- Codemod directory-recursive mode
- Advanced expression operators
- Granular Card props

Two additional items were added based on Plan 07 and Plan 07 SUMMARY findings:

- **`LoadError::Catalog` variant cleanup** — surfaced in Plan 07: the enum variant still exists from the `load_builtins` test helper; a deprecation pass is a natural follow-up once all hard-fail catalog producers are migrated.
- **`$if` evaluate-at-render-time vs `visible` timing gap** — architectural note from Plan 07: `$if` removes at resolve-time, `visible` hides at render-time; a unified directive is a possible v12.1 simplification.

## Phase 164 Final State

| Metric | Value |
|--------|-------|
| Plans | 12 (Waves 1–7) |
| Tests added (Phase 164) | ~60 across Plans 01–09 |
| New source files created | `ferro-mcp/src/tools/json_ui_validate_spec.rs`, `ferro-cli/tests/fixtures/migrate_v1/in_all_verbs.rs`, `ferro-json-ui/tests/fixtures/reject/six_level_nesting.json`, `framework/tests/pipeline_order.rs` |
| Planning artefacts created | `V1-DELETION-AUDIT.md`, `PLUGIN-SURFACE-AUDIT.md`, `COMPLETED.md` |
| v1 surface rows audited | 25 (23 MIGRATED, 2 INTENTIONAL_DROP, 0 BLOCKER) |
| Runtime frictions closed | 10/10 (F1–F10) |
| Docs files updated | 5 (`components.md`, `spec-construction.md`, `migration-v1-to-v2.md`, `expressions.md`, `plugins.md`) |

Phase 164 closes the v12.0 friction loop. All V7-RUNTIME-FRICTION.md items are resolved. The v1 deletion audit is clean.

## Handoff to Phase 160

Phase 160 (v1 deletion) is **UNBLOCKED**.

The gate condition for Phase 160 is "zero BLOCKER rows in the v1 deletion audit." That condition is met: the audit in `COMPLETED.md` Section 5 and in `V1-DELETION-AUDIT.md` both show Total BLOCKER rows = 0.

Phase 160 can proceed to delete `ferro-json-ui/src/view.rs` and all remaining v1 re-exports from `framework/src/lib.rs` and `ferro-json-ui/src/lib.rs`.

## Deviations from Plan

None — plan executed exactly as written.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | `45c63514` | docs(164-12): add COMPLETED.md — v12.0 friction loop closure summary |

## Self-Check: PASSED

- `COMPLETED.md` exists at correct path: CONFIRMED
- Five section headers (`## 1.`, `## 2.`, `## 3.`, `## 4.`, `## 5.`): CONFIRMED (grep -c = 5)
- "Phase 160 (v1 deletion) is UNBLOCKED" present: CONFIRMED (2 occurrences)
- F1 present in Section 2: CONFIRMED
- F10 present in Section 2: CONFIRMED
- Trigger phrases: 0 matches (CLEAN)
- Commit `45c63514` in git log: CONFIRMED
