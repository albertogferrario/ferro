# Pitfalls Research — v12.5 Projection Checkpoint

**Domain:** Agent-facing verification/checkpoint tool added to an existing introspection surface (ferro-mcp)
**Researched:** 2026-06-09
**Confidence:** HIGH — grounded in the existing codebase, the design spec, real friction data (F11–F14), and the regex-based `reconstruct_service_def` implementation already in `render_projection.rs`.

---

## Critical Pitfalls

### Pitfall 1: FALSE CONFIDENCE — `not_checked` collapsed into `pass`

**What goes wrong:**
A seam that cannot be checked because its prerequisite is absent (no rendered view yet, model source unresolvable, route registry empty) silently returns the same `pass` status as a seam that ran and verified clean. The agent reads the aggregate verdict as `pass` and treats the slice as safe. When the missing prerequisite is later provided, the previously "clean" slice turns out to be broken. The agent has been trained on a lie.

**Why it happens:**
The simplest aggregation implementation maps every non-`fail` outcome to either `pass` or `warn`. Developers reach for a boolean `ok/not_ok` without encoding the third epistemic state. The design spec explicitly names this risk but the implementation pressure is toward fewer states. A test suite that exercises only happy-path + failure-path cases will not catch the omission of `not_checked`.

**How to avoid:**
- The seam status enum must have four distinct variants: `pass`, `fail`, `warn`, `not_checked`. No implicit coercion to `pass`.
- The aggregate `status` field in the checkpoint verdict is computed as: `fail` if any seam is `fail`; `warn` if any seam is `warn` and none `fail`; `pass` only if every *checked* seam is `pass` **and** the unchecked list is explicitly surfaced.
- Per-seam prerequisite checks must be written as explicit guard returns of `not_checked` — not as `Ok(pass)` on the fallback path. Example: if `reconstruct_service_def` returns `Err`, seam 2 must return `not_checked`, not `pass`.
- Test case required: a projection with an unresolvable source model → seam 2 status must be exactly `not_checked`, not `pass`. This test must appear in P1 (tool + seam) and must fail before the guard is implemented.

**Warning signs:**
- Any match arm that maps an `Err` result from a prerequisite lookup to `pass` without going through `not_checked`.
- Aggregation logic using "all seams pass" without distinguishing "pass" from "not evaluated".
- Missing fixture: a projection that passes seam 1 but has no model source — if no test exists for this case, the `not_checked` path is untested.

**Phase to address:** P1 (tool + seam) — the `not_checked` variant and its test must be in the first deliverable. A checkpoint without this invariant cannot be released under any phasing.

---

### Pitfall 2: FIELD→COLUMN FALSE POSITIVE — legitimate non-column fields flagged as broken

**What goes wrong:**
The field→column check walks every `FieldDef` in the `ServiceDef` and tries to find a matching column in the entity/migration. Certain legitimate fields have no backing column: computed/virtual fields (totals derived at query time), relationship navigation fields (`customer_id` exists but `customer` is a `RelationshipDef` rendered as a navigation link, not a column), write-only aggregates, and projection-layer-only annotations. If the checker flags all of these as failures, agents receive false positives on every reasonably complex projection. One false positive is enough to destroy trust in the tool: agents learn to ignore findings, and the real defect (a column the migration never created) is buried in noise.

**Why it happens:**
The `ServiceDef` fields list intermixes column-backed fields with projection-layer fields. The `FieldMeaning` enum has no "virtual/computed" variant today. The regex-based `reconstruct_service_def` does not distinguish between `.field()`, `.read_only_field()`, and synthetic fields that were added by an agent as projection-layer annotations rather than column declarations.

**How to avoid:**
- Define the exemption set precisely before writing the checker. Exempt categories:
  1. Fields whose `FieldMeaning` maps to a relationship navigation role: `ForeignKey` fields that have a corresponding `RelationshipDef` with a matching `foreign_key` annotation should be checked against the FK column, not the relationship name.
  2. Relationship fields surfaced via `.has_many()` / `.belongs_to()` etc. appear in `ServiceDef.relationships`, not `ServiceDef.fields`, so they must never enter the field→column loop at all.
  3. `FieldMeaning::Custom("virtual")` or a new `FieldMeaning::Computed` variant (if added) — exempt by meaning.
  4. `id`, `created_at`, `updated_at` system fields — universally present in SeaORM entities; if the entity parse returns them, they will match; if the entity file is unreachable, seam 2 is `not_checked` (Pitfall 1 applies).
- The check resolves to `warn` (not `fail`) when a field has no column but there is an ambiguous reason (e.g. the entity file exists but the column parse is incomplete). `fail` is reserved for "entity file found, column definitively absent".
- Test fixture required: a projection with a `has_many` relationship plus a computed display field — neither should produce a finding.

**Warning signs:**
- The false-positive rate on the synthetic app catalog exceeds zero in any non-trivially complex projection (i.e., any projection with a relationship or a read-only aggregate).
- During dogfood acceptance (P3), the checker fires on projections that have been serving correctly in production.

**Phase to address:** P1 (field→column seam implementation). The exemption logic must be code-reviewed before any dogfood run. A failing dogfood (false positive on a known-clean projection) is a blocker, not a warning.

---

### Pitfall 3: FIELD→COLUMN FALSE NEGATIVE — real seam defect silently skipped

**What goes wrong:**
A projection field references a model attribute that the migration never created. The checker runs and produces no finding. The agent ships the projection. At runtime, SeaORM attempts to SELECT the missing column and produces an error. This is the exact F11-class failure the milestone exists to prevent — but now the checkpoint itself is culpable because it reported clean.

**Why it happens:**
Three distinct root causes:
1. **Column name mismatch tolerance.** A lenient normalizer might match a field named `startedAt` (camelCase, from a typo) to `started_at`, silently passing a real mismatch.
2. **Incomplete entity parse.** The regex-based `reconstruct_service_def` silently drops any builder call pattern it does not cover (e.g. a SeaORM entity using `column_name` attribute or `DeriveColumn` with a non-trivial enum). The column list is incomplete and the check has no evidence to fire on.
3. **Stale entity vs. renamed migration column.** A migration renamed a column; the projection was not updated; the entity file was regenerated. The checker compares projection field against the updated entity and correctly fires. But in the inverse: a projection uses the new column name while the entity file is stale — the checker sees "no such column" in the stale entity, producing a false negative from the entity's perspective (the column is in the DB but the entity file hasn't caught up).

**How to avoid:**
1. **Exact case-sensitive match after snake_case normalization only** — no fuzzy matching. A mismatch is a mismatch. The fix suggestion should say "add column X to migration or rename field in projection", not silently pass.
2. **Parse completeness signal.** When the entity parser uses regex and encounters a construct it cannot parse, it must return `not_checked` with a note rather than silently producing an incomplete list. Incomplete parse + "column not found" produces `warn`, not `fail`, because the absence could be a parse gap.
3. **Migration cross-reference as secondary signal.** If the entity file and the latest migration both lack column X, confidence in a `fail` is high. If only the entity lacks it, surface `warn: entity may need regeneration`.

**Warning signs:**
- The dogfood run against the synthetic catalog finds zero field→column failures in any projection (statistically implausible if any projection was hand-authored).
- The test suite has no fixture with a deliberately wrong field name.

**Phase to address:** P1 for the exact-match rule and parse completeness signal. The migration cross-reference can be P2 if it does not make P1 scope untenable.

---

### Pitfall 4: COLUMN NAME / TYPE MISMATCH SCOPE CREEP — verifying more than presence in P1

**What goes wrong:**
A projection field has `DataType::Integer` for a field that the entity maps to `String` (e.g. a status code that was once an integer and was migrated to a string enum). The checker attempts to verify type compatibility and fires on every such mismatch. Type mapping between `DataType` (projection-layer) and SeaORM `ColumnType` is lossy and non-invertible for several common cases (`ColumnType::String(None)` vs. `DataType::String`, `Text` vs. `String`, etc.). The checker produces false positives at a high rate and trust collapses before the presence check is established.

**Why it happens:**
`ServiceDef::from_model()` performs the `ColumnType` → `DataType` mapping. The reverse direction (verifying a hand-authored projection's `DataType` against the entity column) requires the same mapping to be invertible — which it is not fully. Attempting type verification before the mapping is calibrated produces noise.

**How to avoid:**
- Scope seam 2 to presence-only for P1. Type mismatch checking is a P2 enhancement, added only after the presence check is trusted against real projections.
- When type mismatch checking is added in P2, use `warn` not `fail` for all type mismatches except provably incompatible cases (e.g. `DataType::Integer` for a `Text` column). Ambiguous cases (e.g. `DataType::String` for a `Text` column) are silent.
- The P1 tool description must explicitly state "presence-only: type compatibility is not verified" so agents do not assume type safety from a `pass`.

**Warning signs:**
- A P1 implementation that compares `DataType` to entity column type strings.
- A test that fails because `DataType::String` does not match `Text`.

**Phase to address:** P1 scoping (presence-only), P2 for type mismatch as `warn`.

---

### Pitfall 5: OUTPUT ERGONOMICS — unranked findings, missing actionable fix, verbosity

**What goes wrong:**
The checkpoint produces a long, seam-ordered list of findings. An agent reading the output must determine which findings to act on first. Without a ranked `next_steps` list that names a specific file and action, the agent either acts on the wrong finding first, re-asks the human, or ignores the output entirely. Any of these outcomes defeats the purpose of the tool.

**Why it happens:**
The natural output of a multi-seam walk is seam-ordered (1, 2, 3, 4, 5), not priority-ordered. Seam 1 findings come before seam 2 findings in the raw output, even if seam 1 has a warning and seam 2 has a fail. Without an explicit ranking step, the agent reads the first finding as the most important one.

**How to avoid:**
- `next_steps` is a ranked, deduplicated list computed after all seams run. Ranking: `fail` findings before `warn` findings; within a rank, earlier seams first (seam 2 before seam 3, etc.).
- Each `next_steps` entry must name: the field/action/subject, the seam, and the specific action (e.g. "add column `starts_at` to migration `create_bookings_table`" not "fix field→column mismatch").
- The seam detail blocks are present for diagnostics; `next_steps` is what the agent acts on. The tool description must say this explicitly.
- Maximum `next_steps` per call: 5. An agent with 15 ranked items will act on 1–2 and re-run; a list of 15 looks like a project audit, not a targeted fix list.
- Test: a fixture with 3 seam failures and 2 warnings must produce a `next_steps` list that is ordered fail-first, seam-ordered within each tier, and contains no duplicates.

**Warning signs:**
- `next_steps` is built by appending each seam's findings in seam order without re-sorting.
- A seam 4 fail appears before a seam 2 fail in `next_steps`.
- A finding appears in `next_steps` as "field→column mismatch" without naming the specific field.

**Phase to address:** P1 for core ranking logic. P2 for fix specificity (e.g. naming the migration file). P3 for capping and deduplication stress testing during dogfood.

---

### Pitfall 6: SEAM COUPLING — checkpoint reimplements validator logic instead of delegating

**What goes wrong:**
The checkpoint implements its own version of "is this action's handler a registered route?" instead of calling `json_ui_verify_action`. The two implementations diverge. A route format the standalone validator accepts is flagged as unknown by the checkpoint, or vice versa. Agents get different answers from `json_ui_verify_action` and `checkpoint_projection` on the same projection.

**Why it happens:**
Calling into existing tool implementations requires threading `project_root` and function signatures through the checkpoint dispatcher. Reimplementing the check inline is faster and avoids the dependency. But the design spec is explicit: the checkpoint owns only the field→column seam and aggregation; all other seams are thin dispatches.

**How to avoid:**
- The five seams in `checkpoint_projection.rs` must import and call their respective validator functions (`execute_single` from `validate_projection`, `execute` from `json_ui_verify_action`, etc.), not reimplement the checks.
- The seam finding's `source` field carries the name of the producing validator (e.g. `"json_ui_verify_action"`). A code reviewer can verify this is populated with the actual function name, not a hardcoded string.
- Test: if `json_ui_verify_action` is updated to accept a new URL pattern, `checkpoint_projection` must automatically accept it too without a matching update. Write a fixture that would fail under a reimplementation but passes under delegation.

**Warning signs:**
- `checkpoint_projection.rs` contains route-parsing logic duplicated from `json_ui_verify_action.rs`.
- `source: "checkpoint"` appears on a seam other than seam 2.

**Phase to address:** P1 — architectural boundary must be enforced from the first commit. Seams 1, 3, 4, 5 are wrappers; seam 2 is the only owned check.

---

### Pitfall 7: DOGFOOD/ACCEPTANCE RISK — green-for-green's-sake

**What goes wrong:**
The dogfood gate runs the checkpoint against the synthetic app catalog and all projections pass. The tool ships. No one learns whether it catches anything real. Three months later, an agent introduces a dangling field reference and the checkpoint passes because the test catalog projections were authored to be correct. The tool has never caught a real defect in its life; it has only confirmed that correct projections are correct.

**Why it happens:**
The synthetic catalog projections are generated from model metadata via `ServiceDef::from_model()`, which by construction produces only fields that the model has — so field→column seam 2 will always pass on auto-derived projections. The dogfood gate passes not because the checker is correct but because the input was deliberately defect-free.

**How to avoid:**
- The synthetic app catalog must include at least one projection with a deliberately introduced seam defect: a field that references a column the entity does not have. This is a "poisoned" fixture. The acceptance criterion is: the poisoned fixture produces a `fail` on seam 2 with the correct field named.
- The live consumer acceptance criterion requires running the checkpoint against gestiscilo's projections. At least one finding (fail or warn) must surface. If zero findings appear, the design spec requires revisiting the checker before shipping.
- The dogfood acceptance test is a go/no-go gate for P3, not a nice-to-have.

**Warning signs:**
- All acceptance fixtures were authored by the same process that generates the ground truth (model-derived projections).
- No poisoned fixture exists in the test suite.
- The "at least one real finding" criterion is satisfied by a seam 1 structural warning unrelated to field→column.

**Phase to address:** P3 explicitly. The poisoned fixture must be written before the P3 acceptance run, not after.

---

### Pitfall 8: REGEX RECONSTRUCTION DRIFT — new builder patterns silently omitted

**What goes wrong:**
`reconstruct_service_def` in `render_projection.rs` is regex-based and silently drops any builder call it does not have a pattern for. If the field→column seam reuses this function and a new `FieldDef` builder variant is added after the regex was last updated, fields added via the new builder silently disappear from the reconstructed `ServiceDef`. The field never enters the field→column check, and a missing column goes undetected.

**Why it happens:**
Regex-based source parsing has no completeness guarantee. The parser is not aware of what it does not match. Adding a new builder variant requires a corresponding regex update; without it, fields added via that variant are invisible.

**How to avoid:**
- Add a "reconstruction completeness" assertion to the checkpoint: count the number of field-builder invocations in the raw source (any `.field(`, `.optional_field(`, `.read_only_field(`, `.write_only_field(`) and compare against `ServiceDef.fields.len()`. If they differ, surface `warn: reconstruction may be incomplete` on seam 2 rather than a clean pass.
- Treat any unrecognized `.XXX_field(` invocation as `not_checked` for the affected fields rather than silent omission.
- Any new field builder variant added to `ServiceDef` must be accompanied by a corresponding regex in `reconstruct_service_def`. Document this as an invariant in `ferro-mcp`'s CLAUDE.md.

**Warning signs:**
- A `.list_field()` call is present in a projection source, but the reconstructed `ServiceDef` has fewer fields than source builder invocations.
- The reconstruction count diverges from the builder invocation count in any test fixture.

**Phase to address:** P1 for the completeness assertion (directly affects whether seam 2 is trustworthy). P2 for hardening if new builder variants are introduced during v12.5.

---

### Pitfall 9: INLINE VERDICT NOISE — checkpoint appended to every generation call

**What goes wrong:**
The design spec closes the loop by appending the checkpoint verdict inline to `generate_projection` and `json_ui_generate` responses. If the projection was just created (empty fields, placeholder structure), the checkpoint fires with `not_checked` for most seams. The agent reads a wall of `not_checked` findings after every generation call, learns to ignore checkpoint output, and stops acting on it even when it carries a real `fail`.

**Why it happens:**
Immediately after generation, the project state cannot satisfy most seam prerequisites. The verdict is accurate but noise-producing in the exact context where the agent is least prepared to act on it.

**How to avoid:**
- Inline verdict after generation must be summarized, not full detail. Format: `"checkpoint": { "status": "not_checked", "reason": "newly generated — run checkpoint_projection after wiring model and routes" }` rather than the full seam breakdown.
- The full breakdown is returned by the standalone `checkpoint_projection` call, invoked when the agent is ready to verify.
- The inline verdict upgrades to a full breakdown only when at least one seam can actually run. Seam 1 can always run; if seam 1 fails immediately after generation, surface that in detail.

**Warning signs:**
- An agent that just ran `generate_projection` receives 5 `not_checked` seam entries and zero actionable next steps.
- The agent's subsequent messages do not reference the checkpoint output (signal it has been tuned out).

**Phase to address:** P2 (inline hook implementation). P1 delivers the standalone tool with full output; P2 must consciously design the inline summary format before wiring to `generate_projection`.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Return `pass` when source model is unresolvable | Simpler aggregation code | Trains agents to trust results when model is missing; silent F11-class pass | Never — `not_checked` is required |
| Reimplement route check inline instead of delegating | Avoids threading `project_root` | Two implementations diverge; inconsistent agent answers | Never — delegation is a design constraint |
| Skip reconstruction completeness assertion | Faster P1 delivery | Silent field omissions on any new builder pattern | Acceptable only if `not_checked` guard covers unrecognized builders |
| No cap on `next_steps` length | Fewer edge cases in P1 | Long lists are ignored by agents | Acceptable for P1 if capped to 10; must be 5 by P3 |
| Type mismatch check included in P1 | More complete from day one | High false-positive rate before mapping is calibrated; trust collapses | Defer to P2 — presence-only in P1 |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| `reconstruct_service_def` reuse | Treating any `Ok` as a complete reconstruction | Check field count vs. source builder invocation count; treat discrepancy as `not_checked` |
| `json_ui_verify_action` delegation | Calling with a bare action name rather than the full context it expects | Read the existing tool's signature; pass `project_root` and handler string exactly as the standalone tool does |
| `validate_contracts` delegation | Assuming it returns a result type compatible with the seam finding format | It returns its own result type; the checkpoint must translate, not alias |
| `application_info` status surfacing | Adding a new field to the response struct without updating MCP tool descriptions | Update tool descriptions alongside the struct change; MCP descriptions are part of the surface |

## "Looks Done But Isn't" Checklist

- [ ] **`not_checked` vs `pass` distinctness:** Unit test with unresolvable model source → seam 2 status is exactly `not_checked`, not `pass`.
- [ ] **Poisoned fixture exists:** At least one test fixture has a deliberately wrong field name (no matching entity column) and checker produces `fail` for exactly that field.
- [ ] **Relationship fields excluded from seam 2:** Projection with `belongs_to` relationship → zero findings for the relationship navigation field.
- [ ] **`next_steps` ranked correctly:** Fixture with seam 2 `fail` and seam 1 `warn` → `next_steps[0]` is the seam 2 finding.
- [ ] **Delegation, not reimplementation:** Code review confirms `checkpoint_projection.rs` imports and calls `validate_projection::execute_single`, `json_ui_verify_action::execute`, etc. — no inline route parsing.
- [ ] **Dogfood gate is not auto-pass:** Synthetic catalog contains at least one poisoned projection; acceptance run produces at least one `fail` finding.
- [ ] **Inline verdict is summary format:** After `generate_projection`, appended checkpoint does not contain 5 `not_checked` seam entries with empty `findings` arrays.
- [ ] **MCP tool description updated:** `checkpoint_projection` description accurately states presence-only scope (no type verification in P1).

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| FALSE CONFIDENCE (`not_checked` collapsed to `pass`) | P1 | Unit test: model-unresolvable projection → seam 2 is `not_checked` |
| Field→column false positive (virtual/relationship fields) | P1 | Unit test: projection with `belongs_to` + computed field → zero findings |
| Field→column false negative (missing column not caught) | P1 | Poisoned fixture → `fail` on exactly the dangling field |
| Type mismatch scope creep in P1 | P1 scoping | Tool description states presence-only; type check deferred to P2 |
| Output ergonomics, ranked `next_steps` | P1 (core ranking) + P2 (fix specificity) | Mixed-seam fixture → `next_steps` ordered fail-first |
| Seam coupling (reimplementation vs. delegation) | P1 | Code review: no route parsing logic in `checkpoint_projection.rs` |
| Dogfood acceptance green-for-green | P3 | Poisoned fixture fires + live consumer produces at least one finding |
| Regex reconstruction drift | P1 (completeness assertion) | Field count mismatch → `warn: reconstruction may be incomplete` |
| Inline verdict noise after generation | P2 | After `generate_projection`, appended verdict is summary format only |

## Sources

- Design spec: `docs/superpowers/specs/2026-06-09-projection-checkpoint-design.md`
- Existing field→column mapping implementation: `ferro-mcp/src/tools/projection_coverage.rs`
- Regex reconstruction: `ferro-mcp/src/tools/render_projection.rs` (`reconstruct_service_def`)
- Real friction evidence: `.planning/backlog/v12-runtime-friction-f11-f13.md` (F11: PageHeader.children silent drop; canonical seam failure example)
- `ferro-projections/src/service.rs` — `ServiceDef` structure, `FieldDef`, builder variants
- `ferro-mcp/src/tools/validate_projection.rs` — existing seam 1 validator (delegation target)

---
*Pitfalls research for: v12.5 Projection Checkpoint — agent-facing verification tool*
*Researched: 2026-06-09*
