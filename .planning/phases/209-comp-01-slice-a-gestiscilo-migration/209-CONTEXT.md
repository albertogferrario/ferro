# Phase 209: COMP-01 Slice A — Gestiscilo Migration (Browse + Process + Summarize) - Context

**Gathered:** 2026-06-12
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults selected; review decisions below)

<domain>
## Phase Boundary

Migrate **three** gestiscilo entities — one Browse, one Process, one Summarize — from their existing `JsonUi::render_file` views to projection-driven rendering via `ServiceDef` + `JsonUiRenderer`. This is the first real-world validation signal for the projection/intent abstraction (v1.0 criterion #2, compressive beauty dimension).

**Repo split (the load-bearing boundary — D-09):** this ferro phase is **validation-only**. It does NOT modify, build, or merge the gestiscilo tree. The actual view migration (controller edits, `Cargo.toml` feature flag, branches, merges, server, screenshots) is a **gestiscilo-repo phase**, planned and executed in a gestiscilo GSD session from `GESTISCILO-MIGRATION-BRIEF.md`. Ferro 209 *consumes* that work's outputs.

**In scope (ferro 209):** the ferro-side intent-assertion tests in `ferro-projections/tests/catalog.rs` (the abstraction proof); a render-equivalence record per migrated entity (filled from the gestiscilo outputs); the "what the migration revealed" weakness note; the publish decision (single ferro publish at slice end only if forced). Two ferro plans: 01 (intent fixtures + equivalence stubs), 02 (evidence + weakness note + publish sign-off).

**In scope (gestiscilo phase, NOT here):** exactly 3 entity migrations; one-per-merge to gestiscilo master; the `projections` feature flag; before/after screenshots. Tracked via `GESTISCILO-MIGRATION-BRIEF.md`.

**Out of scope (do not expand):** the rest of the gestiscilo migration (130 views, 69 models — explicitly deferred past v13.0); any change to the seven-intent vocabulary (`intent.rs`); any new ferro renderer; speculative ferro API additions; ferro editing any gestiscilo file. Entity *selection* was deferred to plan-time per ROADMAP and is now resolved in RESEARCH §1.

</domain>

<decisions>
## Implementation Decisions

### Render-equivalence evidence
- **D-01:** Equivalence evidence per entity = **before/after screenshots captured via Chrome DevTools MCP**, paired with a short functional checklist (data fields shown, actions available, primary-use-case flow). HTML diffs are optional supplementary evidence, not the primary record. Rationale: this is a UI migration; visual + functional parity is the honest signal, and screenshots are the SC#2-named acceptable form.
- **D-02:** Equivalence bar = **functional parity for the primary use case**, not pixel-identity. Projection-driven rendering composes intent templates, so layout/markup will differ from the bespoke `render_file` HTML by design. Each migration documents intentional visual deltas; only a *functional* regression (missing data, missing action, broken primary flow) blocks the merge.

### Migration cadence & branch discipline
- **D-03:** Strictly **sequential, one entity per merge**. Each entity is its own short-lived gestiscilo branch, merged to gestiscilo master before the next entity's branch opens. No parallel migration branches. (Mandated by SC#1/#3.)
- **D-04:** **No ferro API changes on master while a gestiscilo migration branch is open** (SC#3). If a migration surfaces a ferro gap, the gap is recorded and worked around in gestiscilo — ferro is not edited mid-slice.

### Abstraction-gap handling (the honesty requirement)
- **D-05:** When a migration hits a `ServiceDef` field with no clean mapping or a renderer output needing a workaround: **note-and-workaround**. Record the gap in the weakness note, apply the smallest gestiscilo-side workaround, keep moving. Ferro fixes for discovered gaps are deferred to a follow-up phase (a subsequent v13.x slice), never folded into this slice. An empty weakness note fails the phase (SC#5) — finding nothing is a red flag, not a success.

### Publishing
- **D-06:** **Default expectation = zero ferro source changes** (the migration consumes the existing `ServiceDef` + `JsonUiRenderer` surface). Do not bump the ferro version speculatively. The "single publish at slice end" (SC#4) is exercised *only* if a discovered gap forces a minimal, safe ferro fix that all three slices can then be re-verified against; in that case the publish happens once, at the end, not mid-series.

### Entity-selection criteria (selection itself deferred to plan-time)
- **D-07:** Selection is resolved at **plan-time** by reading gestiscilo `src/models/` and `src/controllers/` (ROADMAP: do not pre-select now). The selection *criteria* are locked here: pick the **clearest exemplar** of each of Browse / Process / Summarize that (a) has a direct `JsonUi::render_file` call, (b) carries the least bespoke/one-off HTML (maximizes signal clarity and equivalence achievability), and (c) maps to a model whose shape exercises the intent's defining signals. Prefer representative CRUD/list/dashboard entities over edge cases.

### Deliverable location (cross-repo)
- **D-08:** All COMP-01 validation artifacts — the three render-equivalence records and the "what the migration revealed" weakness note — live in **this ferro phase directory** (`.planning/phases/209-comp-01-slice-a-gestiscilo-migration/`), each linking to the corresponding gestiscilo migration commit/PR. The executable code change lives in gestiscilo history.

### Repo boundary (validation-only ferro phase)
- **D-09:** Ferro 209 is **validation-only** and modifies **no gestiscilo file**. The migration code (controller swaps, `Cargo.toml` `projections` flag, branches, merges, the running server, screenshot capture) is a **gestiscilo-repo phase**, planned/executed in a gestiscilo GSD session from `GESTISCILO-MIGRATION-BRIEF.md`. Rationale: the gestiscilo repo has its own GSD planning system (its own milestone, roadmap, phase numbering) — driving its code or roadmap from a ferro session is the same boundary violation the split exists to prevent. Ferro 209's plans (01, 02) only touch `ferro-projections/tests/catalog.rs` and this phase directory. The dependency is one-directional: the gestiscilo phase migrates → ferro 209 records the evidence + runs intent tests + signs off COMP-01. Ferro 209's Plan 02 carries an `external_depends_on` on the gestiscilo migration.

### Claude's Discretion
- Exact screenshot tooling instance (chrome-devtools / -2 / -3), file naming for equivalence records, and the markdown shape of the weakness note are Claude's to decide at execution time, consistent with D-01/D-08.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase definition & requirements
- `.planning/ROADMAP.md` § "Phase 209: COMP-01 Slice A — Gestiscilo Migration" — the five success criteria, the depends-on (Phase 207), and the phase-time calibration note (entity selection deferred).
- `.planning/REQUIREMENTS.md` — COMP-01 line (Slice A scope: 3 entities, one-per-merge, render equivalence, single publish; full migration explicitly out of scope).

### Intent/projection baseline this migration validates against
- `.planning/phases/207-comp-02-synthetic-regression-catalog/207-CONTEXT.md` + `ferro-projections/tests/catalog.rs` — the verified intent vocabulary and per-intent structural invariants the migrated views are compared against (Phase 207 is COMP-01's dependency baseline).
- `ferro-projections/src/intent.rs` — the seven structural intents (Browse / Focus / Collect / Process / Summarize / Analyze / Track). **Read-only this phase — must not change.**
- `ferro-projections/src/derive.rs` — `derive_intents()` signal analyzers.
- `ferro-projections/src/lib.rs` — `ServiceDef` and `ServiceDef::from_model()` derivation bridge (v11.5/Phase 135).
- `ferro-json-ui/src/lib.rs` — `JsonUiRenderer` (the projection→HTML renderer the migration targets).

### Consumer-side migration surface (gestiscilo repo)
- `/Users/alberto/repositories/gestiscilo-it/app/src/controllers/` — 67 `JsonUi::render_file` call sites; the Browse/Process/Summarize candidates are selected from here at plan-time.
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/phases/202-adopt-ferro-crud-macros/202-EVIDENCE.md` — controller duplication/render survey (context for which views are representative).

### Related (do not conflate)
- v13.1 Phase 212 (`.planning/phases/212-crud-handler-proc-macros/212-CONTEXT.md`) — the CRUD-proc-macros phase formerly mis-numbered 209. Unrelated work; listed only so agents don't reconnect the two.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ServiceDef` + `ServiceDef::from_model()` (ferro-projections) — the migration's entry point: derive a projection from each chosen model.
- `JsonUiRenderer` (ferro-json-ui) — renders the `ServiceDef` to HTML, replacing the per-entity `JsonUi::render_file` call.
- `derive_intents()` (ferro-projections) — confirms each migrated entity derives the expected primary intent (Browse/Process/Summarize), tying the migration back to the Phase 207 catalog.
- Chrome DevTools MCP (`mcp__chrome-devtools*`) — captures before/after equivalence screenshots (D-01).

### Established Patterns
- gestiscilo renders views today via `JsonUi::render_file` (67 sites). Migration deletes the per-entity `render_file` call (SC#1) and routes through `ServiceDef` + `JsonUiRenderer` instead.
- gestiscilo has **no existing projection usage** — this is greenfield adoption, so the first migration also establishes the wiring pattern the other two follow.

### Integration Points
- Each migrated gestiscilo controller handler swaps its `JsonUi::render_file(...)` for a `ServiceDef`-derived `JsonUiRenderer` render call.
- The ferro version gestiscilo depends on (via crates.io) is the pin all three slices migrate against (D-06) — no local-path ferro for the validation, unless a forced fix requires it (then single publish at slice end).

</code_context>

<specifics>
## Specific Ideas

- The migration's *value* is the discovered weakness, not the line count. SC#5 makes "what the migration revealed" mandatory; the slice is a success only if it produces at least one real, named abstraction gap or friction point from working against a production codebase.
- Each equivalence record is a small, self-contained markdown file with the before/after screenshots and the functional checklist — readable as standalone evidence by anyone auditing the v1.0 criterion-#2 claim.

</specifics>

<deferred>
## Deferred Ideas

- **Full gestiscilo migration** (remaining ~127 views, 66 models) — explicitly out of v13.0 scope; revisit once Slice A validates the abstraction.
- **Ferro fixes for gaps discovered during the slice** — captured in the weakness note, addressed in a later v13.x phase, never mid-slice (D-04/D-05).
- **CRUD-handler proc macros** — relocated to v13.1 Phase 212 (formerly mis-numbered 209). Not part of this phase.

None of the above is acted on in Phase 209.

</deferred>

---

*Phase: 209-comp-01-slice-a-gestiscilo-migration*
*Context gathered: 2026-06-12*
