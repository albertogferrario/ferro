# Requirements: Ferro Framework

**Defined:** 2026-04-08
**Scope:** v13.0 Road to v1.0 — sustained investment program closing the gap between 0.2.0 and v1.0 across the four design dimensions.

## v13.0 Requirements

Organized by substance-first investment priority. Each maps to a planned phase in `ROADMAP.md`.

### Compressive — projection / intent validation

- [ ] **COMP-01**: Migrate `gestiscilo` to projection-driven rendering as the first real-world validation of the projection / intent abstraction.
- [ ] **COMP-02**: Build a synthetic catalog of canonical app classes covering the seven intents, with regression tests that run on every projection / intent change.
- [ ] **COMP-03**: Measure agent-success-rate — can an agent reading `ferro-mcp` introspection produce a working projection from a natural-language description?
- [ ] **COMP-04**: Time-to-working-app benchmark from `cargo new` to a running service with authentication, three entity types, and one background job.
- [ ] **COMP-05**: Intent vocabulary cross-modality sketch — take one intent (e.g. `Process`) and sketch how the same feature would be expressed as a mobile flow, a voice interaction, and a CLI command. Inform any future intent vocabulary revision.

### Operational — polish and documentation

- [ ] **OPER-01**: MCP integration documentation for Claude Code, Cursor, and other common AI development environments. Include copy-pasteable config snippets.
- [ ] **OPER-02**: Audit projection MCP tool descriptions (`list_projections`, `inspect_projection`, `render_projection`, `validate_projection`, `projection_coverage`) for completeness and accuracy.
- [ ] **OPER-03**: Projection authoring guide via MCP introspection — end-to-end walkthrough of authoring a projection using only MCP tool outputs.
- [ ] **OPER-04**: Agent-assisted deploy workflow walkthrough (`ferro docker:init` → `ferro do:init` → `ferro doctor` → publish), with the role MCP introspection plays at each step.
- [ ] **OPER-05**: Projection-driven starter template option for `ferro new`. Scaffolds a project that exercises the projection / intent system end-to-end as the primary example.
- [ ] **OPER-06**: Iteration loop ergonomics investigation. Quantify the change → rebuild cycle for projection-driven apps and identify concrete improvements.
- [ ] **OPER-07**: `ferro doctor` multi-bin support. `db_connection` and `migrations_pending` checks automatically pass `--bin <pkg>` for workspaces without `default-run`.

### Conceptual — coherence pass

- [ ] **CONC-01**: Systematic coherence audit across all 20 crates. First pass since Phase 113.
- [ ] **CONC-02**: Cross-cutting consistency audit covering naming, error patterns, middleware shapes, CLI verbs, file layouts, and module organization.
- [ ] **CONC-03**: Refactor outlier crates into prevailing patterns identified in CONC-01 and CONC-02.

### Aesthetic — incremental polish

- [ ] **AEST-01**: mdBook custom theme — colors, typography, code block styling.
- [ ] **AEST-02**: Crates.io README polish — shields, visual hierarchy, clear call-to-action.
- [ ] **AEST-03**: GitHub repo social preview image.
- [ ] **AEST-04**: Simple logo / wordmark. No full brand identity system — minimal visual mark only.

## v2 Requirements

Deferred to future releases. Tracked but not in the v13.0 scope.

### Additional rendering modalities

- **MULT-01**: Audio modality renderer — render projections as voice or conversational interfaces.
- **MULT-02**: Physical modality — haptic, gesture, and tangible rendering targets.
- **MULT-03**: Intent vocabulary revision if cross-modality sketch (COMP-05) reveals the current seven intents need reshaping for non-visual rendering.

## Out of Scope

Explicitly excluded from v13.0 to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Landing page at ferro-rs.dev | Dedicated multi-week effort; out of proportion to priority-4 aesthetic investment |
| Full brand identity system (logo + typography system + voice guidelines) | Premature; simple wordmark covered by AEST-04 is sufficient |
| Hero imagery or animated demos | Out of scope with landing page |
| JavaScript-powered client interactivity in JSON-UI | Server-authoritative model is correct; revisit only if validation surfaces a concrete gap |
| New major framework features (payments, subscriptions, IoT integrations, etc.) | v13.0 is consolidation and validation, not feature expansion |
| Audio or physical modality implementation | v2.0+ direction; COMP-05 is the probe, not the build |
| Builder / non-developer-facing tooling | Separate project, independent timeline |

## Traceability

Phase mappings populated during roadmap planning. Phases 115–121 (v12.0) run first and are tracked independently. v13.0 phases continue numbering after v12.0 completes.

| Requirement | Phase | Status |
|-------------|-------|--------|
| COMP-01 | v13.0 Phase (TBD) | Pending |
| COMP-02 | v13.0 Phase (TBD) | Pending |
| COMP-03 | v13.0 Phase (TBD) | Pending |
| COMP-04 | v13.0 Phase (TBD) | Pending |
| COMP-05 | v13.0 Phase (TBD) | Pending |
| OPER-01 | v13.0 Phase (TBD) | Pending |
| OPER-02 | v13.0 Phase (TBD) | Pending |
| OPER-03 | v13.0 Phase (TBD) | Pending |
| OPER-04 | v13.0 Phase (TBD) | Pending |
| OPER-05 | v13.0 Phase (TBD) | Pending |
| OPER-06 | v13.0 Phase (TBD) | Pending |
| OPER-07 | v13.0 Phase (TBD) | Pending |
| CONC-01 | v13.0 Phase (TBD) | Pending |
| CONC-02 | v13.0 Phase (TBD) | Pending |
| CONC-03 | v13.0 Phase (TBD) | Pending |
| AEST-01 | v13.0 Phase (TBD) | Pending |
| AEST-02 | v13.0 Phase (TBD) | Pending |
| AEST-03 | v13.0 Phase (TBD) | Pending |
| AEST-04 | v13.0 Phase (TBD) | Pending |

**Coverage:**
- v13.0 requirements: 19 total
- Mapped to phases: 0 (pending roadmap planning when v13.0 begins)
- Unmapped: 19 ⚠️

Phase mapping happens when v13.0 begins execution, after v12.0 completes.

---

*Requirements defined: 2026-04-08*
*Last updated: 2026-04-08 at v13.0 milestone creation*
