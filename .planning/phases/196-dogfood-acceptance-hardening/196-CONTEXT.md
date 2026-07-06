# Phase 196: Dogfood Acceptance + Hardening - Context

**Gathered:** 2026-06-10
**Status:** Ready for planning
**Mode:** --auto (recommended defaults selected; review decisions below)

<domain>
## Phase Boundary

The checkpoint must earn its place by finding a real seam defect in a real
project. This phase is an acceptance gate plus three hardening changes — not new
seam logic.

Four deliverables:
1. **Deliberately-poisoned synthetic fixture** — a projection with a field that
   has no backing migration column. Model-derived projections auto-pass seam 2,
   so without a planted dangling field the gate is vacuous. The acceptance test
   asserts the poisoned projection produces `status: "fail"` with the
   `field_to_column` seam finding naming *exactly* the planted field in `subject`
   and no other field (SC-1).
2. **Live-consumer run** — run `checkpoint_projection` against a real consumer
   application's projections; acceptance requires at least one finding (fail or
   warn on any seam). A run that finds nothing real fails acceptance and the
   design is revisited, not shipped (SC-2). This is the go/no-go gate.
3. **`next_steps` cap 10→5** — reduce the cap; a fixture with more than 5
   findings proves enforcement (SC-3).
4. **Zero-finding seam demotion** — any wrapper seam (1, 3, 4, 5) that produced
   zero findings across all dogfood inputs is documented as
   `not_checked`-by-default in the tool description, not silently omitted (SC-4).

**In scope:** the poisoned fixture + acceptance test; the live-consumer dogfood
run + recorded verdict; the cap reduction + over-cap test; empirical
per-seam-finding tally and demotion of zero-finding seams (code default +
documentation); a committed acceptance report capturing the go/no-go decision.

**Out of scope:** new seams; changing existing seam dispatch/normalization logic
(Phases 194–195); external consumers not reachable from this repo (e.g.
gestiscilo lives in a separate repo not checked out as a sibling — the live
consumer is the in-repo `app/` sample).
</domain>

<decisions>
## Implementation Decisions

### Poisoned synthetic fixture (D-01) — SC-1, CHK-10
- **D-01:** Build the poisoned projection as a **committed acceptance test
  fixture** using the existing test infrastructure in `checkpoint_projection.rs`
  (`project_with_projection(name, src)` + `add_model(tmp, name, src)`, lines
  ~858-873). The fixture defines a model with a known column set and a projection
  whose `FieldDef` list contains exactly one field absent from those columns. The
  test asserts `status == "fail"`, a `field_to_column` finding whose `subject`
  names that one planted field, and that no other field in the projection appears
  as a finding. Reproducible and exact-assertable — preferred over poisoning the
  `app/` sample (which would couple acceptance to a mutable sample app).
  - **[auto] recommended default** — chosen over (b) poison `app/` projection,
    (c) standalone catalog directory.

### Live consumer selection (D-02) — SC-2, CHK-10
- **D-02:** The live consumer is the in-repo **`app/` sample application**.
  gestiscilo (the external friction-loop consumer) is not checked out as a
  sibling of this repo and cannot be run reproducibly from the acceptance gate.
  `app/src/projections/` contains 9 projections (`api_key`, `feedback_form`,
  `order`, `product`, `revenue_dashboard`, `sales_analytics`, `todo`, `user`)
  while `app/src/models/` defines only 3 (`api_key`, `todos`, `users` + an
  `entities/` dir) — a real projection↔model surface mismatch likely to produce
  genuine findings. Run `checkpoint_projection::run_for` against each `app/`
  projection and record the aggregate.
  - **[auto] recommended default** — chosen over gestiscilo (unreachable from
    this repo).
  - **RESEARCH FLAG:** confirm which `app/` projections resolve a source model
    and which produce findings; if `app/` produces *zero* findings across all
    projections, acceptance is NOT satisfied — escalate (the gate is real, not a
    formality). Researcher should run the checkpoint against `app/` early to
    de-risk this.

### Acceptance evidence / go-no-go gate (D-03) — SC-1..SC-4
- **D-03:** Acceptance is recorded two ways:
  1. **Automated tests** in `checkpoint_projection.rs`: the poisoned-fixture test
     (D-01), and an over-cap test that builds >5 findings and asserts
     `next_steps.len() <= 5` (SC-3).
  2. **A committed acceptance report** at
     `.planning/phases/196-dogfood-acceptance-hardening/196-ACCEPTANCE.md`
     capturing: the live-consumer run output (which `app/` projections, which
     findings), the per-seam finding tally across all dogfood inputs (feeds
     D-04), and an explicit **GO / NO-GO** verdict. NO-GO means the design is
     revisited, not shipped.
  - **[auto] recommended default** — both, chosen over test-only or report-only.

### Zero-finding seam demotion (D-04) — SC-4, CHK-10
- **D-04:** Tally findings per wrapper seam (1 `projection_well_formed`,
  3 `action_to_route`, 4 `rendered_view`, 5 `props_to_contract`) across BOTH
  dogfood inputs (poisoned fixture + `app/` live run). For each wrapper seam with
  zero findings across all inputs:
  - change its default outcome to report `not_checked` with a reason explaining
    it is unproven against real inputs (rather than emitting a vacuous `pass`),
    and
  - document it as `not_checked`-by-default in the MCP tool description
    (`ferro-mcp/src/service.rs`) and in `docs/src/agents/checkpoint-projection.md`.
  Seam 2 (`field_to_column`) is exempt — the poisoned fixture proves it. Demotion
  is data-driven: only seams that actually found nothing are demoted; do not
  pre-emptively demote.
  - **[auto] recommended default** — code default → `not_checked` + documentation,
    chosen over documentation-only or a config flag.

### `next_steps` cap reduction (D-05) — SC-3 (mandated, not gray)
- **D-05:** Reduce the cap from 10 to 5 in `aggregate_next_steps`
  (`checkpoint_projection.rs:763`, `if result.len() == 10` → `== 5`) and update
  the doc comments that say "cap 10" (lines ~71, ~90, ~737) and the
  `aggregate_next_steps` docstring. The over-cap test (D-03) locks this.

### Claude's Discretion
- Exact name/location of the poisoned-fixture and over-cap test functions
  (follow existing `#[test]` naming in the module's test block).
- Wording of the `not_checked`-by-default reason strings and tool-description
  notes (D-04), within SC-4's "documented, not silently omitted" constraint.
- Structure of `196-ACCEPTANCE.md` beyond the required GO/NO-GO verdict + finding
  tally.
- Whether the cap becomes a named `const` (e.g. `MAX_NEXT_STEPS: usize = 5`)
  vs an inline literal — a `const` is mildly preferred for self-documentation.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design spec + requirements (authoritative)
- `docs/superpowers/specs/2026-06-09-projection-checkpoint-design.md` §"Testing"
  (dogfood acceptance bullet) and §"Implementation slice" (seams that do not
  catch a real defect may be deferred — the basis for D-04 demotion).
- `.planning/REQUIREMENTS.md` §CHK-10 (line 34) — the acceptance requirement.
- `.planning/ROADMAP.md` §"Phase 196: Dogfood Acceptance + Hardening"
  (lines 2409-2421) — goal + 4 success criteria.

### Prior phase output (the contract this phase exercises)
- `.planning/phases/194-core-checkpoint-tool/194-CONTEXT.md` — seam 2 design,
  output types, coverage-honesty invariant.
- `.planning/phases/195-close-the-loop-by-default/195-CONTEXT.md` — wrapper seam
  dispatch (D-02..D-05), canonical seam names, seam cascade.

### Code to touch
- `ferro-mcp/src/tools/checkpoint_projection.rs`:
  - `aggregate_next_steps` (lines ~734-770, cap at line 763) — D-05.
  - `run_for` and the per-seam dispatch functions — the surface the dogfood run
    exercises; locus for D-04 default-outcome changes.
  - test helpers `project_with_projection` (~861) and `add_model` (~871) — D-01
    poisoned fixture + D-03 over-cap test.
  - doc comments at lines ~71, ~90, ~737 ("cap 10") — D-05.
- `ferro-mcp/src/service.rs` — MCP `checkpoint_projection` tool description
  (D-04 `not_checked`-by-default documentation).
- `docs/src/agents/checkpoint-projection.md` — agent-facing doc (D-04
  documentation; keep seam example consistent with Phase 195 canonical names).

### Live-consumer target
- `app/src/projections/*.rs` (9 projections) and `app/src/models/*.rs`
  (3 models + `entities/`) — the projection↔model mismatch under test (D-02).
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `project_with_projection` + `add_model` tempdir test helpers
  (`checkpoint_projection.rs:858-873`) — exactly the infra the poisoned fixture
  (D-01) and over-cap fixture (D-03) need; no new test scaffolding required.
- `aggregate_next_steps` already ranks + dedups; only the cap constant changes
  (D-05).
- `run_for` returns the full `Verdict` with per-seam `SeamResult`s — the dogfood
  run reads finding counts per seam directly for the D-04 tally.

### Established Patterns
- `not_checked` is an existing distinct `SeamStatus` variant with a `reason`
  (Phase 194 coverage-honesty) — D-04 demotion reuses it, no new status type.
- One-tool-per-file; tool descriptions live in `service.rs`; agent docs in
  `docs/src/agents/` (CLAUDE.md: update MCP surface + docs when introspection
  changes).

### Integration Points
- The dogfood run is exploratory work that produces the D-04 tally — researcher
  should run the checkpoint against `app/` early; its findings determine which
  seams get demoted, so D-04 cannot be finalized before the live run.
</code_context>

<specifics>
## Specific Ideas

- The poisoned fixture must plant **exactly one** dangling field and the test
  must assert no *other* field is flagged — SC-1 is specific ("naming exactly the
  planted dangling field … and no other field in the same projection"). A
  one-field assertion is insufficient; assert the finding set size for that seam.
- The acceptance gate is real: SC-2 / CHK-10 explicitly say a checkpoint that
  finds nothing real in a real project FAILS acceptance and the design is
  revisited. The `196-ACCEPTANCE.md` GO/NO-GO verdict must reflect actual run
  output, not be assumed GO.
- Demotion is evidence-driven (D-04): document only the seams that actually found
  nothing across dogfood inputs — do not blanket-demote, and do not silently drop
  a seam.
</specifics>

<deferred>
## Deferred Ideas

- Running the checkpoint against an external consumer repo (gestiscilo) once it
  is reachable from a shared checkout — would strengthen the dogfood evidence
  beyond the in-repo `app/` sample. Future milestone.
- IN-02 from Phase 194 code review (surface the unrecognized `DataType` in the
  D-06 warn subject) — fold in only if this phase touches that path.

None blocking — discussion stayed within phase scope.
</deferred>

---

*Phase: 196-dogfood-acceptance-hardening*
*Context gathered: 2026-06-10*
