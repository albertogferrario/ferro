# Phase 126: Deploy experience feedback triage - Context

**Gathered:** 2026-04-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Analysis-only phase. Read `REPORT.md` (gestiscilo deploy field notes: 2 fixed bugs, 9 sharp edges, 6 DX items), cross-reference each numbered item (1–17) against existing phases 122–125, classify each as already-in-scope / new-phase / follow-up-plan / dropped, and produce `PROPOSAL.md` in this phase directory. No code changes. No ROADMAP edits. The user reviews PROPOSAL.md and decides what to promote via `/gsd:add-phase`.

</domain>

<decisions>
## Implementation Decisions

### PROPOSAL.md Structure
- **D-01:** Lead with a triage table — one row per REPORT item (1–17) with columns: item, one-line summary, classification, target phase (existing or proposed). Then a "Proposed New Phases" section with one block per phase.
- **D-02:** Items 1 and 2 are excluded from new work (already shipped in `70ad9ed4` / 0.2.1) but MUST still appear in the triage table marked as "shipped" so the table proves coverage of all 17 items.

### New-Phase Drafts
- **D-03:** Each proposed new phase block contains: working title, one-paragraph goal, list of REPORT item numbers it absorbs, dependencies on existing phases. Nothing more — no inline SCOPE.md, no plan breakdown. The user expands via `/gsd:add-phase` after approval.
- **D-04:** Group aggressively. If two items would land in the same phase anyway, they share a phase block. The user prefers concrete clustering over speculative roadmaps.

### Sequencing Recommendation
- **D-05:** Order proposed phases by user pain / real deploy friction first, not dependency graph or risk. Matches the "small phases, fast turnaround" pattern noted in SCOPE.md. Call out hard dependencies as a secondary constraint only when they force reordering.

### Cross-Reference Discipline
- **D-06:** For every "already in scope" classification, cite the specific phase (122 / 122.1 / 122.2 / 123 / 124 / 125) and ideally the SCOPE bullet that covers it. No vague "covered by 124" claims.
- **D-07:** Specifically check: does promoting `deploy_check` to a CLI command (likely a REPORT item) overlap with Phase 123's MCP tool, or with Phase 124's `ferro doctor`? Decide one home, don't double-book.

### Phase Numbering for Proposals
- **D-08:** Proposed phase numbers must not collide with the JSON-UI v2 milestone (115–121 already occupied). Pick numbers in the post-126 range. Note the gsd-tools collision bug from REPORT item 11 in PROPOSAL.md so the user can file it manually.

### Validation Against Real Apps
- **D-09:** Sanity-check each proposed phase against both reference apps (`../../gestiscilo-it/app` server-rendered, and `../../gestiscilo-it/mkmenu` frontend bundle). If a proposal only helps one shape, say so explicitly.

### Claude's Discretion
- Exact phase titles, exact phase numbers, and exact grouping of items into phases.
- Whether to include a short "dropped items rationale" appendix at the end of PROPOSAL.md.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary source
- `.planning/phases/126-deploy-experience-feedback/REPORT.md` — field notes, items 1–17, source of truth
- `.planning/phases/126-deploy-experience-feedback/SCOPE.md` — phase definition, process steps, success criteria

### Cross-reference targets (existing phases to check against)
- `.planning/phases/122*/SCOPE.md` — original deploy scaffold (122, 122.1, 122.2)
- `.planning/phases/123-deploy-mcp-tools/SCOPE.md` — Deploy MCP tools (overlap risk for `deploy_check` promotion)
- `.planning/phases/124-doctor-*/SCOPE.md` — `ferro doctor`, introspection, CI (overlap risk for `ferro deploy:check`)
- `.planning/phases/125-module-*/SCOPE.md` — module scaffolder + json-ui runtime split (likely unrelated; confirm)

### Roadmap context
- `.planning/ROADMAP.md` — current phase definitions, milestone boundaries
- `.planning/STATE.md` — Roadmap Evolution log, current milestone, recent decisions

### Code surface (for sanity-checking proposed work)
- `ferro-cli/src/deploy/` — current deploy command implementation
- `ferro-cli/src/templates/` — Dockerfile / deploy templates

### Reference apps (validation)
- `../../gestiscilo-it/app` — server-rendered, multi-bin, postgres, chromium
- `../../gestiscilo-it/mkmenu` — frontend bundle, single bin, deployed

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-cli/src/deploy/` already contains the deploy scaffold from Phase 122.2 — any "promote X to CLI" proposal builds on this surface, not greenfield.
- Recent fix `70ad9ed4` (path→version rewrite, real rust slim tag) lives in this surface and resolves REPORT items 1 and 2.

### Established Patterns
- Ferro CLI commands follow `ferro <area>:<verb>` (e.g. `deploy:check`). Any new command in proposals should match.
- The `ferro doctor` / introspection pattern (Phase 124) is the natural home for read-only diagnostic checks. Mutating / scaffolding work belongs in `deploy`.

### Integration Points
- `ferro mcp` exposes deploy tooling to agents (Phase 123). Anything proposed as a CLI command should also be considered for MCP exposure parity.

</code_context>

<specifics>
## Specific Ideas

- The user runs ferro improvements proactively during product work — proposed phases should be small enough to ship in a single focused session, not multi-week epics.
- "Concrete clustering over speculative roadmaps" — when in doubt, fewer larger-impact phases beat many tiny speculative ones.
- The 0.2.1 release context matters: the project just bumped from 0.1.88 in a large breaking-change release, so proposed phases can assume a clean slate.

</specifics>

<deferred>
## Deferred Ideas

- Filing the gsd-tools roadmap collision bug from REPORT item 11 — lives in another repo. PROPOSAL.md should note it so the user files manually.
- Implementing any of the suggested fixes — strictly out of scope for Phase 126.
- Editing ROADMAP.md to add new phases — the user does this via `/gsd:add-phase` after reviewing PROPOSAL.md.

</deferred>

---

*Phase: 126-deploy-experience-feedback*
*Context gathered: 2026-04-08*
