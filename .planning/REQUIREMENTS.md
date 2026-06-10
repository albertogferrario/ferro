# Requirements: v12.5 Projection Checkpoint

## Milestone Goal

Close the agent write→verify loop. Ferro's write side is rich (generators, templates); the verify side is fragmented across single-purpose validators an agent must know to call and sequence itself. `checkpoint_projection` is one projection-anchored MCP call that walks the intent-slice spine, dispatches to the existing validators at each seam, runs the one seam no validator covers today (projection field → model column), and returns a single structured verdict with ranked next steps — honest about coverage, and closing by default after generation.

Killer feature: an agent that adds a projection field referencing a model attribute the migration never created learns it statically, in one call, instead of at runtime — the silent F11-class seam becomes a ranked, actionable next step.

## Conceptual Coherence Anchor

v12.5 introduces no new abstraction. The unit of verification is the **intent slice**, anchored on the projection / `ServiceDef`, which already ties model → intent → derived view. The tool is a pure orchestrator: it owns exactly one new check (field→column) plus aggregation, and **delegates every other seam to an existing validator** (`validate_projection`, `json_ui_verify_action`, `render_projection` + `json_ui_validate_spec`, `validate_contracts`) — no duplicate control surface. It is read-only and introspective: no `cargo`/compile; it reads source, the route registry, and DB schema, reusing primitives already in the `ferro-mcp` crate graph (`list_models::execute`, the `projection_coverage` model-resolution predicate, `reconstruct_service_def`). Zero new dependencies.

The load-bearing trust invariant: **`not_checked` never collapses into `pass`.** A checkpoint that reports `pass` on something it did not actually verify trains the agent to trust a lie, which is worse than no checkpoint.

## v1 Requirements

### Core Checkpoint (P1)

- [x] **CHK-01** — An agent can call `checkpoint_projection { name }` and receive a single structured verdict: top-level `status` (`pass`/`warn`/`fail`), a per-seam result list, and a ranked, deduplicated `next_steps` list of actionable fixes. Each seam finding names its producing validator in `source` (provenance).
- [x] **CHK-02** — The field→column seam flags every projection field with no backing entity column. It resolves the projection to its source model via the same `src/projections/` ↔ `src/models/` name-match `projection_coverage` uses, reconstructs the `ServiceDef` via `reconstruct_service_def`, and compares field names against the model's columns (`list_models::execute`, no running DB required).
- [x] **CHK-03** — Every seam reports its state as one of `pass` / `fail` / `warn` / `not_checked`, distinctly. `not_checked` is used when a prerequisite is absent (no source model resolved, no rendered view exists, reconstruction incomplete) and is **never** coerced to `pass`. Unchecked seams do not raise overall `status` to `fail` but are listed. (Enforced by a dedicated test.)
- [x] **CHK-04** — The field→column seam never raises a false positive on a field that legitimately has no column: relationship navigation fields (carried in `ServiceDef.relationships`, not `.fields`) and computed/virtual fields are exempted by construction, not flagged.
- [x] **CHK-05** — When `reconstruct_service_def` cannot fully parse the projection source (a builder pattern it does not cover), the field→column seam reports `not_checked` with a reason rather than a false `pass` — verified by a completeness check, not assumed.
- [x] **CHK-06** — `next_steps` is ranked (failures above warnings; within a rank, earlier seams first) and deduplicated, and each entry is actionable (names the subject, the problem, and a concrete fix path).

### Close the Loop by Default (P2)

- [x] **CHK-07** — `generate_projection` and `json_ui_generate` return the checkpoint verdict inline in their response after generating, so the agent receives it whether or not it issues a separate call. The dependency is one-way: the generators depend on the checkpoint; the checkpoint does not depend on the generators.
- [x] **CHK-08** — `application_info` and `projection_coverage` surface a per-projection checkpoint status (`clean` / `failing` / `unverified`) as read-only consumers, so an agent surveying the project sees verification debt without probing for it.
- [x] **CHK-09** — Seams 1, 3, 4, and 5 dispatch to the existing validators (`validate_projection`, `json_ui_verify_action`, `render_projection` + `json_ui_validate_spec`, `validate_contracts`) and aggregate their output into the unified verdict. No validation logic for these seams is reimplemented in the checkpoint; each finding's `source` names the producing validator.

### Dogfood Acceptance (P3)

- [ ] **CHK-10** — The checkpoint is run across the synthetic app catalog — which must include at least one **deliberately poisoned** projection (a field with no backing column), since model-derived projections auto-pass seam 2 and would make the gate vacuous — and against one live consumer application. Acceptance requires it to surface at least one real seam defect; a checkpoint that finds nothing real in a real project fails acceptance and the design is revisited, not shipped.

## Design Decisions To Resolve In Planning

Surfaced by research as underspecified in the design spec; the phase planner must resolve them (not user-facing requirements):

- **Seam cascade** (P1): when an upstream seam fails (e.g. projection malformed), do dependent downstream seams report `not_checked` with reason, or run anyway? Research recommends cascade-to-`not_checked`.
- **Fix-string normalization** (decide P1, ship P2): sub-validators for seams 1/3/4/5 return heterogeneous shapes; decide whether the checkpoint normalizes them into the uniform finding shape or passes through with a documented caveat. The output contract must commit to a shape in P1.
- **Ambient status freshness** (P2): is the `application_info`/`projection_coverage` status a cached last-run result or an always-fresh lightweight recompute? Two materially different implementation costs.

## Anti-Requirements (explicit non-goals to prevent scope drift)

- No auto-fix / mutation — the checkpoint reports what to fix; it never edits code. The read-only contract preserves the agent's review step.
- No parallel validation engine — every seam except field→column reuses an existing validator; a second implementation would create two sources of truth.
- No mega-verdict — one call verifies one projection; whole-project status is surfaced via `projection_coverage`, not an aggregate of every projection.
- No `cargo`/compile invocation — the checkpoint stays introspective/read-only.

## Future Requirements (deferred)

- Model-anchored fan-out: checkpoint every projection/route/view touching a given model (projection anchor only for v12.5).
- A `cargo check`-backed compile seam.
- A method-threaded seam-3 check (verify the action's HTTP method, not just handler registration).

## Out of Scope

- Model-anchored and compile seams (see Future) — explicitly deferred to keep the spine walk bounded and the loop fast.
- Client-side surfacing of the verdict (IDE panel, etc.) — the consumer is the agent via MCP.

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CHK-01 | Phase 194 | Complete |
| CHK-02 | Phase 194 | Complete |
| CHK-03 | Phase 194 | Complete |
| CHK-04 | Phase 194 | Complete |
| CHK-05 | Phase 194 | Complete |
| CHK-06 | Phase 194 | Complete |
| CHK-07 | Phase 195 | Complete |
| CHK-08 | Phase 195 | Complete |
| CHK-09 | Phase 195 | Complete |
| CHK-10 | Phase 196 | Pending |

*(Phase column filled by the roadmapper.)*
