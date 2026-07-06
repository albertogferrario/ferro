# Phase 209: COMP-01 Slice A — Gestiscilo Migration - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-12
**Phase:** 209-comp-01-slice-a-gestiscilo-migration
**Mode:** `--auto` (all gray areas auto-selected; recommended default chosen per area)
**Areas discussed:** Phase-identity collision (pre-discussion fix), Evidence format, Equivalence bar, Migration cadence, Abstraction-gap handling, Publish trigger, Entity-selection criteria, Deliverable location

---

## Pre-discussion: Phase 209 identity collision (user-resolved)

Before any gray area could be discussed, `init phase-op 209` resolved to a `209-crud-handler-proc-macros` directory whose CONTEXT.md (committed `dbaf32b5` by the gestiscilo Phase 202 evidence pass) defined Phase 209 as "CRUD handler proc macros" — contradicting ROADMAP.md, which owns 209 for "COMP-01 Slice A Gestiscilo Migration". Surfaced to the user rather than auto-proceeding.

| Option | Description | Selected |
|--------|-------------|----------|
| 209 = COMP-01 (roadmap wins), move CRUD macros | Relocate CRUD-macros work to a new number; 209 stays COMP-01 | |
| 209 = CRUD macros (repurpose), move COMP-01 | Update ROADMAP so 209 = CRUD macros | |
| Pause — user inspects | Stop, change nothing | |
| **Fix the two phases** (user free-text) | Keep both bodies of work; disentangle the number collision so each is clean | ✓ |

**Follow-up — where does the relocated CRUD-macros phase live?**

| Option | Description | Selected |
|--------|-------------|----------|
| New milestone v13.1, Phase 212 | Dedicated single-phase milestone; v13.0 stays purely Compressive Validation | ✓ |
| Append to v13.0 as Phase 212 | Simpler but mixes a framework feature into a validation milestone | |
| Park in backlog (999.x) | Defer until promoted | |

**Resolution:** 209 stays COMP-01; CRUD macros relocated to v13.1 Phase 212. Committed `4bbb2674`.

---

## Evidence format (render equivalence)

| Option | Description | Selected |
|--------|-------------|----------|
| Chrome MCP before/after screenshots + functional checklist | Visual + functional parity record per entity | ✓ (recommended) |
| HTML structural diff | Markup-level diff | |
| Both | Screenshots and HTML diff | |

**Auto-selected:** Screenshots via Chrome DevTools MCP + functional checklist; HTML diff optional supplementary. **Rationale:** UI migration — visual/functional parity is the honest, SC#2-named signal.

## Equivalence bar

| Option | Description | Selected |
|--------|-------------|----------|
| Functional parity for primary use case | Same data + actions + primary flow; layout may differ by design | ✓ (recommended) |
| Pixel-identity | Reproduce bespoke HTML exactly | |

**Auto-selected:** Functional parity; document intentional visual deltas. Pixel-identity is not a projection-rendering goal.

## Migration cadence

| Option | Description | Selected |
|--------|-------------|----------|
| Sequential, one entity per merge, short-lived branches | Each entity merged to gestiscilo master before the next opens | ✓ (recommended, mandated) |
| Parallel branches | Multiple migrations in flight | |

**Auto-selected:** Strictly sequential. Mandated by SC#1/#3.

## Abstraction-gap handling

| Option | Description | Selected |
|--------|-------------|----------|
| Note-and-workaround; defer ferro fixes post-slice | Record gap, smallest gestiscilo workaround, keep moving | ✓ (recommended, mandated) |
| Pause and fix ferro immediately | Edit ferro mid-slice | |

**Auto-selected:** Note-and-workaround. Mandated by SC#3 (no ferro master changes mid-branch); feeds the mandatory SC#5 weakness note.

## Publish trigger

| Option | Description | Selected |
|--------|-------------|----------|
| Zero-ferro-change default; batch forced fix into one slice-end publish | No speculative version bump | ✓ (recommended) |
| Pre-plan a ferro version bump | Bump regardless | |

**Auto-selected:** Zero-change default; single publish only if a discovered gap forces a minimal safe fix.

## Entity-selection criteria

| Option | Description | Selected |
|--------|-------------|----------|
| Lock criteria, defer selection to plan-time | Clearest Browse/Process/Summarize exemplars with direct render_file + least bespoke HTML | ✓ (recommended, mandated) |
| Pre-select entities now | Choose the 3 entities during discuss | |

**Auto-selected:** Criteria locked (D-07); selection deferred to plan-time per ROADMAP ("Do not pre-select now").

## Deliverable location

| Option | Description | Selected |
|--------|-------------|----------|
| ferro phase dir holds validation docs, links gestiscilo commits | COMP-01 is a ferro requirement | ✓ (recommended) |
| gestiscilo .planning holds docs | | |

**Auto-selected:** ferro phase directory holds equivalence records + weakness note, linking gestiscilo history.

## Claude's Discretion

- Screenshot MCP instance choice, equivalence-record file naming, weakness-note markdown shape.

## Deferred Ideas

- Full gestiscilo migration (beyond Slice A) — out of v13.0 scope.
- Ferro fixes for discovered gaps — later v13.x phase, never mid-slice.
- CRUD-handler proc macros — relocated to v13.1 Phase 212.
