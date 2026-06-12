---
phase: 209-comp-01-slice-a-gestiscilo-migration
plan: "01"
subsystem: ferro-projections/tests
tags: [projections, intent-derivation, validation-scaffold, gestiscilo-migration]
dependency_graph:
  requires: [207-01]
  provides: [real_world_slice_a intent fixtures, EQUIV stubs for Plan 02]
  affects: [ferro-projections/tests/catalog.rs]
tech_stack:
  added: []
  patterns: [intent_hint_override, guarded_branching_state_machine]
key_files:
  created:
    - .planning/phases/209-comp-01-slice-a-gestiscilo-migration/EQUIV-staff-browse.md
    - .planning/phases/209-comp-01-slice-a-gestiscilo-migration/EQUIV-orders-process.md
    - .planning/phases/209-comp-01-slice-a-gestiscilo-migration/EQUIV-stats-summarize.md
  modified:
    - ferro-projections/tests/catalog.rs
decisions:
  - "IntentHint::Primary(Intent::Browse) required for Staff — bio+avatar_url pull toward Focus without the hint (weak-signal finding, aligns with RESEARCH Risk 1)"
  - "Orders state machine includes both branching states (confermato, in_corso) to satisfy >=2 branching states invariant and fire guarded_transitions + branching_states signals"
  - "stats_summarize uses only the three SummaryStats fields (no id, no created_at) — all read-only Money+Quantity, no hint needed"
metrics:
  duration: "~3 minutes (202 seconds)"
  completed: "2026-06-12"
  tasks_completed: 2
  files_modified: 4
---

# Phase 209 Plan 01: Slice A Validation Scaffold Summary

Three gestiscilo entity `ServiceDef` fixtures with `derive_intents()` assertions added to `catalog.rs`; three EQUIV equivalence-record stub files filed in the phase directory.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add real_world_slice_a intent-assertion fixtures to catalog.rs | a9d6a36a | ferro-projections/tests/catalog.rs |
| 2 | Create three equivalence-record stubs in the phase directory | 00170248 | EQUIV-staff-browse.md, EQUIV-orders-process.md, EQUIV-stats-summarize.md |

## What Was Built

**Task 1:** Added `pub mod real_world_slice_a` to `ferro-projections/tests/catalog.rs` with:
- `IntentHint` added to the `use ferro_projections::{...}` import list
- `staff_browse()` fixture: 7 fields mirroring the gestiscilo `Staff` model shape, with `IntentHint::Primary(Intent::Browse)` override (required because `bio` FreeText + `avatar_url` ImageUrl pull toward Focus)
- `orders_process()` fixture: 5 fields + a 5-state branching guarded state machine (`confermato/in_corso/rientrato/chiuso/annullato`) with 6 guarded transitions; two branching states fire the Process signals
- `stats_summarize()` fixture: 3 read-only fields (`total_revenue_cents` Money, `order_count` Quantity, `average_order_cents` Money); Summarize wins decisively without a hint
- Three `#[test]` functions asserting `[0].intent` for each fixture; all three pass

**Task 2:** Created three EQUIV stub markdown files in the phase directory, each containing the five-assertion functional checklist template from VALIDATION.md and the correct gestiscilo source paths per the RESEARCH §1 entity selection table. These stubs are filled in Plan 02 after the gestiscilo migration executes.

## Deviations from Plan

None — plan executed exactly as written. The `optional_field`, `display_name`, and `intent_hint` builder methods were all present at 0.2.54 as documented in the plan interfaces. The `IntentHint::Primary(Intent::Browse)` override worked as predicted by RESEARCH Risk 1 / Pitfall 3.

## Key Decisions

1. `IntentHint::Primary(Intent::Browse)` required for Staff — bio+avatar_url pull toward Focus without the hint. This is itself an abstraction signal worth noting in the Plan 02 weakness note.
2. Orders state machine includes both `confermato` and `in_corso` as branching states to ensure the `guarded_transitions` and `branching_states` signals both fire cleanly.
3. `stats_summarize` contains only the three `SummaryStats` fields with no `id` or `created_at` — all read-only Money+Quantity, no hint needed; Summarize wins decisively.

## Verification

- `cargo test -p ferro-projections --test catalog real_world_slice_a` — 3/3 PASS
- `cargo fmt --all -- --check` — PASS (rustfmt applied, then re-checked clean)
- `cargo clippy --all --all-targets -- -D warnings` — PASS (no warnings)
- `git diff --name-only ferro-projections/src/` — empty (derive.rs/intent.rs unchanged)
- No files under `/Users/alberto/repositories/gestiscilo-it/` modified

## Known Stubs

The three EQUIV files are intentional stubs by design — they are the D-08 placeholders for Plan 02 to fill from gestiscilo migration outputs. The plan explicitly marks them as `Status: stub (filled in Plan 02...)`. This does not block the plan's goal, which is to *create* the stub templates (not fill them).

## Threat Flags

None — this plan adds only `ferro-projections` test fixtures and planning documents. No request path, no input parsing, no auth or data-scoping surface is touched.
