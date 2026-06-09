# Phase 195: Close the Loop by Default - Context

**Gathered:** 2026-06-10
**Status:** Ready for planning
**Mode:** --auto (recommended defaults selected; review decisions below)

<domain>
## Phase Boundary

Make checkpoint verification happen without the agent asking. Three deliverables:
1. **Wrapper seams 1, 3, 4, 5** — dispatch to existing validators and fold their
   output into the unified verdict via per-seam normalization. No validation logic
   reimplemented; each finding's `source` names the producing validator (CHK-09).
2. **Inline hook** — `generate_projection` and `json_ui_generate` embed the checkpoint
   verdict (summary format, not a five-seam breakdown) in their response (CHK-07).
3. **Ambient status surfacing** — `application_info` and `projection_coverage` expose
   per-projection checkpoint status (`clean`/`failing`/`unverified`) read from the
   `.ferro/checkpoints/{name}.json` cache, stale-ok, no recompute (CHK-08).

**In scope:** the four wrapper seams + their normalization functions; the seam cascade
(locked); inline summary embedding in both generators; ambient read-only surfacing in
two introspection tools; reconciling the non-canonical seam names introduced in Phase 194.

**Out of scope (Phase 196):** dogfood acceptance run; the deliberately-poisoned synthetic
fixture; demoting zero-finding wrapper seams to `not_checked`-by-default; `next_steps` cap
reduction from 10→5.
</domain>

<decisions>
## Implementation Decisions

### Seam-naming reconciliation — DISCREPANCY from Phase 194 (D-01)
Phase 194's executor named the four stub seams `schema_load`, `field_type_compat`,
`action_binding`, `render_target` — these **do not match the design-spec seam→validator
mapping**. They appear in `run_for` stubs, several tests, and `docs/src/agents/checkpoint-projection.md`.
- **D-01:** Phase 195 corrects the seam vocabulary to the design-spec canonical names and
  wires each to its validator. Fix, do not work around (per project discrepancy policy):
  | Seam | Canonical name | source (validator) | 194 wrong name |
  |------|----------------|--------------------|----------------|
  | 1 | `projection_well_formed` | `validate_projection` | `schema_load` |
  | 2 | `field_to_column` | `checkpoint` (already correct) | — |
  | 3 | `action_to_route` | `json_ui_verify_action` | `field_type_compat` |
  | 4 | `rendered_view` | `render_projection` (+ `json_ui_validate_spec`) | `action_binding` |
  | 5 | `props_to_contract` | `validate_contracts` | `render_target` |
  Update the stub-replacement code, all affected tests, and the docs example block in the
  same phase. Names are part of the output contract — settle them now before Phase 196
  documents seams as `not_checked`-by-default.

### Wrapper seam dispatch + normalization (D-02..D-05) — CHK-09
Each wrapper seam calls the existing validator and translates its heterogeneous output
into the uniform `Finding { subject, detail, fix }` via a per-seam normalization function
at the module boundary (the seam established in Phase 194, D-08). No validator logic is
reimplemented.
- **D-02 (seam 1, `projection_well_formed`):** call `validate_projection::execute_single(project_root, name)`
  (`validate_projection.rs:34`) → `ValidationResult`; normalize its errors/warnings to
  `Finding`s; `source: "validate_projection"`.
- **D-03 (seam 3, `action_to_route`):** dispatch to `json_ui_verify_action`. **RESEARCH FLAG:**
  this tool currently exposes only `find_handler` (`json_ui_verify_action.rs:52`, crate-private),
  not a full `execute`. The researcher must determine the correct entry: either call
  `find_handler` per `ActionDef` in the reconstructed `ServiceDef`, or add a thin
  `execute`-style wrapper in `json_ui_verify_action` that the seam consumes. Either way,
  `source: "json_ui_verify_action"`; no route-matching logic duplicated in the checkpoint.
- **D-04 (seam 4, `rendered_view`):** call `render_projection::execute(...)` (`render_projection.rs:36`),
  then feed the rendered spec JSON to `json_ui_validate_spec::execute(spec_json)`
  (`json_ui_validate_spec.rs:43`). Render failures → `source: "render_projection"`; spec
  validation findings → `source: "json_ui_validate_spec"`.
- **D-05 (seam 5, `props_to_contract`):** call `validate_contracts::execute(project_root, route_filter)`
  (`validate_contracts.rs:80`), scoping `route_filter` to this projection's route(s);
  normalize mismatches to `Finding`s; `source: "validate_contracts"`.

### Seam cascade (D-06) — locked in STATE.md
- **D-06:** Replace the stub block in `run_for` with cascade-aware dispatch:
  - seam 1 (`projection_well_formed`) fail → seams 4 and 5 report `not_checked` with
    `reason: "seam_1_failed"` (they need a valid ServiceDef parse).
  - seam 4 (`rendered_view`) fail → seam 5 reports `not_checked` with `reason: "seam_4_failed"`.
  - seams 2 (`field_to_column`) and 3 (`action_to_route`) run independently of seam 1.
  Coverage-honesty invariant from Phase 194 still holds: `not_checked` never coerced to `pass`.

### Inline verdict summary format (D-07, D-08) — CHK-07
- **D-07:** `generate_projection` and `json_ui_generate` call `checkpoint_projection::run_for`
  after generating and embed the result under a `checkpoint` key. One-way dependency:
  generators depend on the checkpoint; the checkpoint never imports the generators.
- **D-08:** The embedded value is a **summary**, not the full five-seam breakdown. SC-1
  forbids presenting five `not_checked` entries with empty findings. Recommended summary
  shape: `{ status, fail_seams: [names], warn_seams: [names], next_steps }` (or
  `{ status, counts: {fail, warn, not_checked}, next_steps }`). It MUST contain a top-level
  `status` and MUST NOT be the raw `Verdict.seams` array. Define a `VerdictSummary` type
  (e.g. `Verdict::summary()`) so both generators share it.
  - **RESEARCH FLAG (json_ui_generate anchor):** `json_ui_generate::execute(project_root, model, description)`
    returns a generation *context*, not a single named projection. Determine what projection
    name the checkpoint anchors on for the json_ui path (the model's derived projection
    function name, or skip the inline checkpoint when no projection anchor exists and say so
    rather than embedding a vacuous all-`not_checked` summary).

### Ambient status surfacing (D-09, D-10) — CHK-08
- **D-09 (`projection_coverage`):** add a `checkpoint_status` field to `ModelCoverage`
  (`projection_coverage.rs:22`), populated by reading `.ferro/checkpoints/{projection_name}.json`
  and returning its `ambient_status` (`clean`/`failing`); missing file → `"unverified"`.
  Keyed by the projection **function** name (`ModelCoverage.projection_name`), matching the
  cache key produced by Phase 194's `validate_name`-guarded write (the `name` param is the
  function name per the WR-03 doc fix).
- **D-10 (`application_info`):** add a `projection_checkpoint` summary
  `{ total_projections, clean, failing, unverified }` to `ApplicationInfo`, aggregating the
  same per-projection cache reads (SC-3).

### Stale-ok cache read (D-11) — locked freshness decision
- **D-11:** Ambient consumers (`projection_coverage`, `application_info`) read the cache file
  only — never call `run_for`/recompute (roadmap freshness decision: no live recompute on
  ambient queries). The inline hook (D-07) keeps the cache fresh on every generation.

### Claude's Discretion
- Exact field layout of `VerdictSummary` (within D-08 constraints).
- Whether seam 3 gets a thin `execute` wrapper vs per-action `find_handler` calls (research-led).
- Cache-read helper location (a shared `read_ambient_status(project_root, name)` in
  `checkpoint_projection.rs` consumed by both ambient tools is recommended).
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design spec + requirements (authoritative)
- `docs/superpowers/specs/2026-06-09-projection-checkpoint-design.md` — five-seam table,
  seam→validator mapping, coverage honesty, inline-hook + status-surfacing sections, output shape.
- `.planning/REQUIREMENTS.md` §CHK-07, CHK-08, CHK-09 (lines 28-30).
- `.planning/ROADMAP.md` §"Phase 195: Close the Loop by Default" (goal, 4 success criteria,
  locked seam cascade) and §STATE design-decisions block.

### Phase 194 output (the contract this phase extends)
- `.planning/phases/194-core-checkpoint-tool/194-CONTEXT.md` — D-07 output types, D-08
  normalization seam, D-11 cache.
- `ferro-mcp/src/tools/checkpoint_projection.rs` — `run_for` (lines 107-189, the stub block
  to replace), `Verdict`/`SeamResult`/`SeamStatus`/`Finding` types, `aggregate_status`/
  `aggregate_next_steps`, `write_cache` + `CacheEntry` (lines 407-423), the affected tests
  (lines ~851-933), `validate_name`.
- `docs/src/agents/checkpoint-projection.md` — the seam example block (line ~42) using the
  wrong seam name; update with canonical names.

### Validators to dispatch to (seam wiring)
- `ferro-mcp/src/tools/validate_projection.rs:34` — `execute_single(project_root, name)` (seam 1).
- `ferro-mcp/src/tools/json_ui_verify_action.rs:52` — `find_handler(...)` (seam 3 — entry TBD by research).
- `ferro-mcp/src/tools/render_projection.rs:36` — `execute(...)` (seam 4 render).
- `ferro-mcp/src/tools/json_ui_validate_spec.rs:43` — `execute(spec_json)` (seam 4 spec validation).
- `ferro-mcp/src/tools/validate_contracts.rs:80` — `execute(project_root, route_filter)` (seam 5).

### Consumers to extend (inline + ambient)
- `ferro-mcp/src/tools/generate_projection.rs:20,33,85` — `GenerateProjectionResult` + `execute` (inline hook).
- `ferro-mcp/src/tools/json_ui_generate.rs:104` — `execute(project_root, model, description)` (inline hook; anchor TBD).
- `ferro-mcp/src/tools/projection_coverage.rs:22` — `ModelCoverage` (add `checkpoint_status`).
- `ferro-mcp/src/tools/application_info.rs:12` — `ApplicationInfo` (add `projection_checkpoint` summary).
- `ferro-mcp/src/service.rs` — MCP tool result wiring for the above (descriptions reflect new fields).
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `checkpoint_projection::run_for` + the Phase 194 output types/aggregation — the spine the
  wrapper seams plug into.
- All five target validators already exist with stable `execute`-style entry points (except
  `json_ui_verify_action`, which needs a research-led entry decision).
- `CacheEntry`/`write_cache` already emit `ambient_status` — ambient consumers just read it.

### Established Patterns
- One-tool-per-file in `ferro-mcp/src/tools/`; result structs derive `Serialize`/`JsonSchema`;
  serde enums use `rename_all = "snake_case"`.
- Normalization-at-the-boundary pattern (Phase 194 D-08) — extend with one normalization fn
  per wrapper seam.

### Integration Points
- `run_for` stub block (checkpoint_projection.rs:139-167) is the single edit site for the
  wrapper seams + cascade.
- Two generators gain a `checkpoint` response field; two introspection tools gain ambient
  status fields; `service.rs` tool descriptions updated accordingly (CLAUDE.md: update MCP
  surface + docs when introspection changes).
</code_context>

<specifics>
## Specific Ideas

- SC-4 is the anti-reimplementation guard: assert in tests that `source == "checkpoint"`
  appears ONLY on `field_to_column` findings; every wrapper-seam finding names its delegating
  validator. This makes "no logic reimplemented" mechanically checkable.
- Keep the inline summary genuinely small — an agent reading a `generate_projection` response
  should see status + actionable next_steps, not a wall of `not_checked` noise (the F11-class
  signal-to-noise lesson).
</specifics>

<deferred>
## Deferred Ideas

- Dogfood acceptance, poisoned synthetic fixture, zero-finding-seam demotion, `next_steps`
  cap 10→5 — all Phase 196.
- IN-02 from Phase 194 code review (surface the unrecognized `DataType` in the D-06 warn
  subject) — minor polish; fold in only if Phase 195 touches that path, else leave for 196.
</deferred>

---

*Phase: 195-close-the-loop-by-default*
*Context gathered: 2026-06-10*
