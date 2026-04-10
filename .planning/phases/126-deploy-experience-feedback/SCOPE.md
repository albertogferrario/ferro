# Phase 126 — Deploy experience feedback triage

## Context
After the first end-to-end deploy of a real Ferro app (gestiscilo) against
the Phase 122.2 deploy scaffold, a detailed field report was captured listing
2 fixed bugs, 9 still-present sharp edges, and 6 DX improvements. See
`REPORT.md` in this phase directory for the full report.

This phase is **not** an implementation phase. It is a triage phase: a future
agent should read `REPORT.md`, weigh items against existing phases (122–125
already cover some of this surface), decide which items are actionable now,
and propose a phase plan back to the user.

## Goal
Produce a concrete proposal — as a markdown document and/or one or more
follow-up phases — that turns `REPORT.md` into actionable Ferro work.

## Process

1. Read `REPORT.md` end to end. Treat it as the source of truth for what the
   gestiscilo deploy session actually surfaced.
2. Cross-reference each item against the existing roadmap:
   - Phases 122 / 122.1 / 122.2 (already shipped — what's still uncovered?)
   - Phase 123 (Deploy MCP tools) — does promoting `deploy_check` to a CLI
     command duplicate or extend it?
   - Phase 124 (Doctor, introspection, CI) — does `ferro deploy:check`
     overlap with `ferro doctor`?
   - Phase 125 (Module scaffolder + json-ui runtime split) — unrelated.
3. For each report item, classify as:
   - **Already in scope** of an existing phase → cite phase number, no new work.
   - **Should land as a new phase** → draft a one-paragraph SCOPE entry.
   - **Should be a follow-up plan inside an existing phase** → cite phase + plan.
   - **Dropped** with rationale (speculative, low value, out of scope).
4. Validate suggestions against the two real reference apps:
   - `../../gestiscilo-it/app` (server-rendered, multi-bin, postgres, chromium)
   - `../../gestiscilo-it/mkmenu` (frontend bundle, single bin, deployed)
5. Propose phase numbers that do not collide with the JSON-UI v2 milestone
   (which already occupies 115–121 — see `.planning/STATE.md` Roadmap
   Evolution and the gsd-tools bug noted in REPORT item 11).
6. Write the proposal to `PROPOSAL.md` in this phase directory. Stop there —
   do not implement. The user will review and approve before any new phases
   are added or any code is written.

## Out of scope
- Implementing any of the suggested fixes. This phase is analysis only.
- Filing the gsd-tools collision bug (REPORT item 11) — that lives in
  another repo. Note it in `PROPOSAL.md` so the user can file it manually.
- Adding the new phases to ROADMAP.md. The user runs `/gsd:add-phase` (or
  manually) after reviewing `PROPOSAL.md`.

## Success criteria
- `PROPOSAL.md` exists in this phase directory.
- Every numbered item in `REPORT.md` (1–17) is addressed in the proposal,
  even if the resolution is "dropped" or "already covered".
- For each new-phase suggestion, the proposal includes: a working title, a
  one-paragraph goal, the report items it absorbs, and dependencies on
  existing phases.
- The proposal makes a concrete sequencing recommendation (which phase first,
  why).

## Inputs
- `REPORT.md` (this directory) — primary source.
- `.planning/ROADMAP.md` — existing phase definitions.
- `.planning/STATE.md` — current milestone, recent decisions, Roadmap
  Evolution log.
- `.planning/phases/122*/`, `123-deploy-mcp-tools/`, `124-doctor-*/`,
  `125-module-*/` — adjacent SCOPE files for cross-reference.
- `ferro-cli/src/deploy/`, `ferro-cli/src/templates/` — current code surface.

## Notes for the analyzing agent
- The user prefers concrete clustering over speculative roadmaps. If two
  items would land in the same phase anyway, group them.
- Today's bug fixes (REPORT items 1 and 2) already shipped in commit
  `70ad9ed4` / ferro 0.2.1 — do not re-propose them.
- The user runs ferro improvements proactively during product work; this
  triage should reflect that pattern (small phases, fast turnaround).
