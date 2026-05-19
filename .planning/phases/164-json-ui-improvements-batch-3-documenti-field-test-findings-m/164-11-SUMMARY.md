---
phase: 164
plan: "11"
subsystem: planning / docs
tags: [audit, v1-deletion, plugin-surface, d-01, d-02, d-03, d-06, d-07]
dependency_graph:
  requires: [164-01, 164-02, 164-03, 164-04, 164-05, 164-06, 164-07, 164-08, 164-09, 164-10]
  provides: [V1-DELETION-AUDIT.md, PLUGIN-SURFACE-AUDIT.md, Phase-160-readiness-gate]
  affects: [Phase 160 (v1 deletion gate), Phase 161 (merge + publish gate), Phase 12 (COMPLETED.md input)]
tech_stack:
  added: []
  patterns: [v1-deletion audit table, plugin paper audit, three-scenario walk-through]
key_files:
  created:
    - .planning/phases/164-json-ui-improvements-batch-3-documenti-field-test-findings-m/V1-DELETION-AUDIT.md
    - .planning/phases/164-json-ui-improvements-batch-3-documenti-field-test-findings-m/PLUGIN-SURFACE-AUDIT.md
  modified: []
decisions:
  - "25 v1 surface rows audited: 23 MIGRATED, 2 INTENTIONAL_DROP, 0 BLOCKER — Phase 160 is UNBLOCKED"
  - "D-06 plugin audit Outcome B: 2 minor gaps (render data param, init_script per-page-once) fixed in Plan 10 plugins.md"
  - "DetailFormProps and make_node helpers classified INTENTIONAL_DROP — documented in migration guide, no consumer blocked"
metrics:
  duration: "~3 minutes"
  completed: "2026-05-17T03:09:10Z"
  tasks_completed: 3
  files_modified: 2
---

# Phase 164 Plan 11: V1-Deletion Audit + Plugin Surface Audit Summary

V1-deletion readiness confirmed — 25 rows audited, zero BLOCKER rows. Plugin surface audit Outcome B: two minor gaps fixed by Plan 10, no escalation.

## Audit Results

### V1-Deletion Audit (D-01..D-03)

**File:** `V1-DELETION-AUDIT.md`

| Category | Count |
|----------|-------|
| MIGRATED | 23 |
| INTENTIONAL_DROP | 2 |
| BLOCKER | **0** |
| **Total** | **25** |

**BLOCKER count: 0. Phase 160 (v1 deletion) is UNBLOCKED.**

Resolution breakdown:

**23 MIGRATED rows** — every v1 public surface element has a confirmed v2 equivalent in the current source:
- Core types: `JsonUiView`, `Component` enum, `ComponentNode`, `PluginProps` (→ `RawHtml` + plugin registry)
- Container children: `CardProps.children`, `FormProps.fields`, `GridProps.children`, `CollapsibleProps.children`, `FormSectionProps.children`, `ButtonGroupProps.buttons`
- Re-added in Phase 162: `SwitchProps.compact`, `ImageProps.inline_svg`, `RichTextEditorProps` (as plugin)
- Phase 164 additions: `Spec.title` binding (D-12), `KanbanBoard.data_path` (D-13a), `MAX_NESTING_DEPTH` 3→5 (D-14), `Image.data_path` and `DescriptionList.data_path` (D-15), two-stage catalog validation (D-16), `CardVariant` (D-18), `Visibility` error message (D-19/F5), `PageHeader.actions` lax deserializer (D-19/F6)

**2 INTENTIONAL_DROP rows** — documented gaps accepted for v12.0:
1. `DetailFormProps` / `DetailField` / `EditMode` — v2-native pattern documented in `components.md` (Inline view/edit section) and migration guide. No consumer blocked.
2. `make_node` / `make_node_with_action` helpers — never part of ferro public API. Documented in migration guide.

**Grep evidence of v1 absence (captured in audit):**
- `JsonUiView`, `ComponentNode`, `PluginProps`: 2 matches, both `///` doc-comments (historical notes), 0 production code matches
- `Component::` enum variants: 0 matches
- `ferro-json-ui/src/view.rs`: does not exist
- `framework/src/lib.rs` and `ferro-json-ui/src/lib.rs` v1 re-exports: 0 matches

### Plugin Surface Audit (D-06..D-07)

**File:** `PLUGIN-SURFACE-AUDIT.md`

**Outcome: B — minor gaps, fixed inline**

Three scenarios walked against `docs/src/json-ui/plugins.md`:

| Scenario | Gaps | Fixed By |
|----------|------|----------|
| A — Stripe payment status widget | 1 (render `data` param undocumented) | Plan 10 commit `63529b33` |
| B — WhatsApp connection flow | 2 (render `data` param + `init_script` per-page-once semantics) | Plan 10 commit `63529b33` |
| C — Chart renderer | 0 | — |

**No load-bearing missing primitives.** No BLOCKER rows added to V1-DELETION-AUDIT.md.

Both fixes are in the `plugins.md` `ChartPlugin` example:
1. Comments in `render()` body explaining `props` vs `data` semantics
2. Prose after `init_script()` block explaining once-per-page injection and post-`js_assets()` ordering

### User Approval Status

**Pending user approval.** The checkpoint:human-verify task requires explicit user sign-off before Phase 160 (v1 deletion) is unblocked.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Grep-verify v1 surface absence | 1c8a853a | (evidence captured; no files created) |
| 2 | Author V1-DELETION-AUDIT.md | 1c8a853a | V1-DELETION-AUDIT.md |
| 3 | D-06/D-07 plugin surface paper audit | 826cb49a | PLUGIN-SURFACE-AUDIT.md |

(Tasks 1 and 2 are committed together — evidence gathering and document production are logically a single atomic operation.)

## Handoff to Plan 12 (COMPLETED.md)

The v1→v2 surface migration table in V1-DELETION-AUDIT.md (25 rows) can be embedded directly as the "v1 → v2 surface migration table" section of COMPLETED.md. The INTENTIONAL_DROP rows become the "Intentional gaps" section of COMPLETED.md with their associated rationale.

## Deviations from Plan

None — plan executed exactly as written. The D-06 audit was partially pre-empted by Plan 10 (which conducted the paper exercise and fixed the gaps during the docs pass). Plan 11 verified Plan 10's findings, confirmed Outcome B, and produced the formal PLUGIN-SURFACE-AUDIT.md artifact.

## Self-Check: PASSED

- `V1-DELETION-AUDIT.md` exists: FOUND
- Row count (MIGRATED + INTENTIONAL_DROP + BLOCKER in table): 25 (≥ 15 requirement met)
- BLOCKER rows in table: 0
- "Total BLOCKER rows: 0" in BLOCKER summary: FOUND
- Grep evidence section populated (no placeholder text): CONFIRMED
- `PLUGIN-SURFACE-AUDIT.md` exists: FOUND (Outcome B requires it)
- Commits `1c8a853a` and `826cb49a` present in git log: CONFIRMED
