# Phase 194: Core Checkpoint Tool - Context

**Gathered:** 2026-06-10
**Status:** Ready for planning
**Mode:** --auto (recommended defaults selected; review decisions below)

<domain>
## Phase Boundary

Deliver the `checkpoint_projection { name }` MCP tool: load a `ServiceDef` from
`src/projections/`, run the **field→column seam** (the one new check no existing
validator covers), aggregate per-seam results into a single verdict
(`pass`/`warn`/`fail`), and return a ranked, deduplicated `next_steps` list. Also
establish the uniform `Finding { subject, detail, fix }` output contract and the
`.ferro/checkpoints/{name}.json` status-cache write.

**In scope (P1):** tool surface, field→column seam, `not_checked` coverage-honesty
invariant, reconstruction-completeness assertion, false-positive exemptions,
aggregation + `next_steps`, status cache write, output types reused by Phase 195.

**Out of scope (later phases):** wrapper seams 1/3/4/5 (Phase 195), inline
generator hook (Phase 195), ambient status surfacing in `application_info` /
`projection_coverage` (Phase 195), dogfood acceptance (Phase 196). The status
cache is *written* here but *read* by Phase 195.
</domain>

<decisions>
## Implementation Decisions

### Field→column matching rule (D-01)
- **D-01:** Source-model resolution reuses `projection_coverage`'s existing
  predicate — projection `service_name.to_lowercase()` == model `name.to_lowercase()`
  (`ferro-mcp/src/tools/projection_coverage.rs:75-79`). The column set comes from
  `list_models::execute` (`FieldInfo.name`), no running DB required (per CHK-02).
- **D-02:** Field-name comparison is exact against the model's column-name set.
  Projection `FieldDef.name` and entity `FieldInfo.name` are both snake_case;
  match is case-sensitive on the already-normalized snake_case strings. A field
  with no matching column produces a finding with `source: "checkpoint"`,
  `subject: "<field>"`, and `fix: "add column `<field>` to `<entity>` migration, or remove the field from the projection"`.
- **D-03:** When the source model cannot be resolved (no name-match), seam 2 is
  `not_checked` with `reason: "source_model_unresolved"` — never `pass`, and never
  elevates overall `status` to `fail` (CHK-03, success criterion 2).

### Computed/virtual field exemption — CHK-04 (D-04)
- **D-04:** Relationship navigation fields are exempt **by construction** — they
  live in `ServiceDef.relationships`, not `.fields`, so the seam only ever
  iterates real `FieldDef`s. No relationship is ever passed to the column check.
- **D-05:** Computed/virtual display fields with no backing column are exempt by
  the field-builder vocabulary: only fields added through the column-backed
  builders (`.field(`, `.optional_field(`, `.read_only_field(`) are subject to the
  column check. **RESEARCH FLAG:** confirm the exact builder/marker for
  computed/derived fields — `FieldDef` currently has no `computed`/`virtual` flag
  and `FieldMeaning` has no computed variant (`ferro-projections/src/field.rs:35-72`).
  The researcher must determine whether computed fields surface as
  `FieldMeaning::Custom(_)`, a distinct builder, or are simply absent from the
  reconstructed `.fields`. If no such marker exists, the exemption is satisfied by
  D-04 alone and CHK-04's "computed/virtual" clause needs the field model to grow
  an explicit marker — surface this as a coherence question, do not silently skip.

### Reconstruction-completeness detection — CHK-05 (D-06)
- **D-06:** Detect under-parsing by counting column-backed field-builder
  invocations in the projection source via regex (the same builder set parsed in
  `render_projection::reconstruct_service_def`, `render_projection.rs:113+`) and
  comparing to the reconstructed `ServiceDef.fields.len()`. If the source
  invocation count exceeds the reconstructed field count, the seam reports `warn`
  with `reason: "reconstruction_incomplete"` and a finding stating reconstruction
  may be incomplete — never a silent clean `pass` (success criterion 4, CHK-05).
  This is a real completeness check, not an assumption.

### Output types — uniform contract (D-07)
- **D-07:** All checkpoint output types live in
  `ferro-mcp/src/tools/checkpoint_projection.rs` as public types reused verbatim by
  Phase 195 wrapper seams (per STATE.md design decision). Shape:
  - `Finding { subject: String, detail: String, fix: String }`
  - `SeamStatus` enum: `Pass | Warn | Fail | NotChecked` with
    `#[serde(rename_all = "snake_case")]`.
  - `SeamResult { seam: String, status: SeamStatus, source: String, findings: Vec<Finding>, reason: Option<String> }`
    (`reason` populated for `not_checked`/`warn`).
  - Verdict `{ status, projection, seams: Vec<SeamResult>, next_steps: Vec<String> }`.
- **D-08:** Per-seam normalization functions translate heterogeneous sub-validator
  output into `Finding` at the module boundary (locked in STATE.md). In Phase 194
  only the checkpoint-owned field→column seam produces findings; the normalization
  seam is established here so Phase 195 plugs in without changing the contract.

### Overall verdict aggregation + next_steps — CHK-01, CHK-06 (D-09)
- **D-09:** `status` = `fail` if any seam fails, else `warn` if any warning, else
  `pass`. `not_checked` seams are listed but never raise `status` to `fail`
  (CHK-03).
- **D-10:** `next_steps` ranking: all failures before all warnings; within a rank,
  earlier seam number first (seam 1..5 order). Dedup by `(subject, fix)` tuple.
  Each entry is a single actionable string formatted
  `"<fix> (seam: <seam_name>)"` (success criterion 5, CHK-06).

### Status cache write (D-11)
- **D-11:** Every `run_for` call writes `.ferro/checkpoints/{name}.json` containing
  the full verdict plus a derived ambient status (`clean` = pass, `failing` =
  warn/fail, `unverified` = reserved for never-run) so Phase 195 can stale-ok read
  it without recompute (locked in roadmap/STATE.md freshness decision). Create the
  `.ferro/checkpoints/` directory if absent. Timestamp the cache entry — pass it in
  rather than reading wall-clock inside pure logic where testability matters.

### Claude's Discretion
- Exact regex for field-builder invocation counting (reuse/adapt
  `render_projection`'s existing patterns).
- Internal module layout within `checkpoint_projection.rs` (helper fn split).
- Test fixture file layout under `ferro-mcp` test tree.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design spec (authoritative)
- `docs/superpowers/specs/2026-06-09-projection-checkpoint-design.md` — full design:
  five-seam spine, coverage honesty, output shape, non-goals (not a compiler, not a
  generator), implementation slice order.

### Requirements
- `.planning/REQUIREMENTS.md` §CHK-01..CHK-06 (lines 19-24) — the six P1
  requirements this phase satisfies; CHK-07..10 are later phases.

### Roadmap (locked design decisions)
- `.planning/ROADMAP.md` §"Phase 194: Core Checkpoint Tool" (lines 2366-2381) —
  goal, success criteria, seam cascade, fix-string normalization, ambient freshness.

### Reuse targets (existing code the tool builds on)
- `ferro-mcp/src/tools/render_projection.rs:113+` — `reconstruct_service_def(service_name, display_name, content)` and the field-builder regex parsers.
- `ferro-mcp/src/tools/projection_coverage.rs:50-79` — model↔projection name-match predicate and `list_models::execute` usage.
- `ferro-mcp/src/tools/list_models.rs:13-26,165` — `ModelDetails`/`FieldInfo` (column set, no DB).
- `ferro-projections/src/field.rs:35-72` — `FieldMeaning` enum + `FieldDef` shape (relevant to D-05 exemption research).
- `ferro-mcp/src/tools/mod.rs` — tool module registration pattern (add `pub mod checkpoint_projection;` + wire into the MCP tool dispatch).
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `reconstruct_service_def` (render_projection.rs): regex-based ServiceDef
  reconstruction from source — reused for seam 2 and the completeness count (D-06).
- `projection_coverage` model-resolution predicate: the exact `src/projections/` ↔
  `src/models/` lowercase name-match the design spec mandates for seam 2 (D-01).
- `list_models::execute`: column set per model without a running DB (CHK-02 hard
  requirement).

### Established Patterns
- MCP tools live one-per-file in `ferro-mcp/src/tools/`, registered via
  `pub mod` in `mod.rs` and dispatched through the MCP tool registry.
- Tool result structs derive `Serialize` + `JsonSchema`; serde enums use
  `rename_all = "snake_case"` (matches the `pass`/`warn`/`fail`/`not_checked` wire
  format in the design spec).

### Integration Points
- New module `ferro-mcp/src/tools/checkpoint_projection.rs` + registration in
  `mod.rs` and the tool dispatcher.
- Status cache writes to a new `.ferro/checkpoints/` directory (read side is Phase 195).
</code_context>

<specifics>
## Specific Ideas

- The `PageHeader.children` silent-drop (v12.0 friction F11) is the canonical seam
  failure this whole feature targets — the dangling-field case is its field→column
  analogue. Keep the field→column finding's `fix` string concrete enough that an
  agent can act without a second introspection call.
- Acceptance bar (Phase 196, but informs design now): the tool must surface a
  *real* defect in a real project. Avoid over-fitting seam 2 to synthetic fixtures —
  the matching rule (D-02) must hold against real gestiscilo-class projections.
</specifics>

<deferred>
## Deferred Ideas

- Wrapper seams 1/3/4/5, inline generator hook, ambient status read-surfacing —
  Phase 195 (explicitly out of scope here; the output contract established in 194
  is the seam they plug into).
- Model-anchored fan-out (checkpoint every artifact touching a model) — deferred per
  design spec non-goals; anchor stays the projection name only.
- Growing `FieldDef`/`FieldMeaning` with an explicit computed/virtual marker — only
  if D-05 research shows no existing way to identify computed fields; would be a
  cross-cutting ferro-projections change, its own scope.
</deferred>

---

*Phase: 194-core-checkpoint-tool*
*Context gathered: 2026-06-10*
