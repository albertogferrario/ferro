# Phase 164: JSON-UI improvements batch 3 — documenti field-test findings — Context

**Status:** Awaiting friction file. Do not plan yet.

## Phase Boundary

Phase 164 consumes the FRICTION.md files produced by gestiscilo Phases 142 (documenti) and 143 (final cleanup). Documenti is the most form-intensive module: multi-step flows, conditional field visibility, and PDF-preview routing. This is the last improvement batch before v1 deletion (Phase 160).

## Expected friction sources (when ready)

- `/Users/alberto/repositories/gestiscilo-it/app/.planning/phases/142-*/FRICTION.md` — documenti migration (create, edit, preview, emessi/ricevuti detail).
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/phases/143-*/FRICTION.md` — final cleanup and any residual sites that did not migrate in 140–142.

## Expected scope (pre-friction-file estimate, will be revised)

Likely areas, based on documenti's structure and Phase 138's blast-radius hints:

- Multi-step form patterns — wizard flows with state preserved across steps, where each step is its own spec but they share form state. Likely needs a documented convention rather than a new component.
- `visible` rule expressiveness at depth — conditional field visibility on nested forms; whether the current `visible` operator covers all needed cases.
- PDF preview routing — separate concern from JSON-UI; may surface request/response wiring needs.
- `RichTextEditor` (D-18 from Phase 162) — documenti templates require this; Phase 162 lands the API, Phase 164 uses it in field test.
- `DetailForm` v2 replacement pattern (D-15 from Phase 162) — documenti edit flows are the largest consumer of the read+edit pattern; Phase 162 documents the v2 approach, Phase 164 validates it under real load.

## Phase-completion artefact

Produce `COMPLETED.md` summarising every improvement shipped across Phases 162–164 and any intentional gaps retained for future milestones. This artefact is the input to Phase 160's gate (v1 deletion may proceed) and to the v12.0 closing argument in Phase 161 (merge v12.0/json-ui-v2 → master).

## Predecessor

Phase 163 (cassa/calendario). Phase 164 does not depend on Phase 163 completing for friction-file collection but does inherit the iteration-directive and SpecBuilder decisions made there.

## Planning gate

Do not run `/gsd-plan-phase 164` until both 142-FRICTION.md and 143-FRICTION.md exist. The planner should read both, classify every entry, confirm Phase 162's D-15 / D-18 docs survived contact with the documenti migration, and rewrite this CONTEXT with locked decisions before plan-creation begins.
