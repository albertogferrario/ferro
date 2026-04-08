---
phase: 126-deploy-experience-feedback
plan: 01
subsystem: planning
tags: [deploy, triage, analysis]
requires: []
provides: [PROPOSAL.md]
affects: []
tech-stack:
  added: []
  patterns: []
key-files:
  created:
    - .planning/phases/126-deploy-experience-feedback/PROPOSAL.md
    - .planning/phases/126-deploy-experience-feedback/126-01-SUMMARY.md
  modified: []
key-decisions:
  - "D-07 resolved: read-only deploy diagnostics live in `ferro doctor` (Phase 124), exposed via Phase 123 `deploy_check` MCP tool as a thin wrapper. Mutating/scaffolding stays in `ferro deploy:*`. One implementation, two surfaces."
  - "Item 18 (Dockerfile has no ENTRYPOINT/CMD) classified as deploy blocker and sequenced FIRST."
  - "Three new phases proposed (127, 128, 129); aggressive clustering per D-04."
requirements-completed: []
duration: "~10 min"
completed: 2026-04-08
---

# Phase 126 Plan 01: Deploy Experience Feedback Triage Summary

Triage analysis of 18 REPORT.md items into shipped / already-in-scope / new-phase / deferred buckets, with three proposed new phases (127, 128, 129) sequenced by user pain.

## Classification breakdown

- **Shipped (2):** items 1, 2 — already in commit `70ad9ed4` / 0.2.1
- **Already in scope (1):** item 12 — folds into Phase 124 `ferro doctor` per D-07
- **Deferred / external (1):** item 11 — gsd-tools repo, manual filing
- **New phase (14):** items 3, 4, 5, 6, 7, 8, 9, 10, 13, 14, 15, 16, 17, 18

## Proposed phases drafted

1. **Phase 127 — Generated artifact polish (deploy blocker fix + template hygiene)** — items 5, 6, 7, 9, 10, 16, 18
2. **Phase 128 — Deploy preflight (`ferro doctor` deploy checks + drift detection)** — items 3, 4, 13, 15, 17
3. **Phase 129 — Publish workflow refinement (gated bumps, per-crate version notes)** — items 8, 14

Each block contains the four required fields per D-03: working title, one-paragraph goal, absorbed REPORT item numbers, dependencies on existing phases. All numbers in the post-126 range, no collision with 115–121 (D-08).

## Recommended sequence (D-05, by user pain)

1. **Phase 127** first — item 18 is a hard blocker for any actual deploy (silent image exit, will also break DigitalOcean App Platform `web` services).
2. **Phase 128** second — preflight prevents the next deploy session from rediscovering the same five issues build-by-build. Depends on Phase 124's check registry.
3. **Phase 129** last — pure maintainer ergonomics, no end-user blocking.

## D-07 resolution (deploy_check overlap)

`deploy_check` does **not** become a third command. Honoring Phase 122.2's existing decision:

- Read-only diagnostics → **`ferro doctor`** (Phase 124 surface, single check registry)
- MCP exposure → Phase 123 **`deploy_check`** MCP tool, thin wrapper over the same registry
- Mutating/scaffolding → **`ferro deploy:*`** subcommands (Phase 122.2 surface)

Phase 128 adds new checks to the single registry, not a parallel command.

## Deferred / external

- REPORT item 11 (gsd-tools `phase add` collision + JSON-UI v2 number clash) — user files manually against gsd-tools repo. Both sub-bugs documented in PROPOSAL.md "Deferred / External" section.

## Verification

- `grep -E '^\| (1|...|18) ' PROPOSAL.md | wc -l` → 18 ✓
- Items 1 and 2 marked "shipped" ✓
- Triage Table / Proposed New Phases / Sequencing Recommendation sections all present ✓
- Each proposed phase block contains `**Goal:**`, `**Absorbs REPORT items:**`, `**Depends on:**` (and `**App applicability:**`) ✓
- Phase citations from {122, 122.1, 122.2, 123, 124, 125} present (25 occurrences) ✓
- Item 18 / ENTRYPOINT explicitly addressed ✓
- gsd-tools collision noted ✓
- ROADMAP.md untouched by this plan ✓

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## Next

User reviews `PROPOSAL.md` and decides which proposed phases (127, 128, 129) to promote via `/gsd:add-phase`. Phase 126 itself is complete after this triage; no further plans inside 126.

## Self-Check: PASSED
