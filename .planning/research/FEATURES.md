# Feature Research

**Domain:** Agent-facing verification tool — `checkpoint_projection` MCP call for the ferro write→verify loop (v12.5)
**Researched:** 2026-06-09
**Confidence:** HIGH (design spec and all existing validator source read directly; prior art from LSP spec, MCP tool design literature, and schema/contract drift research corroborates findings)

---

## Prior Art Summary

**LSP diagnostics** (closest industrial analog): four severity levels — Error/Warning/Information/Hint. Each diagnostic carries: subject range, severity, code, source (the producing tool), and message. The LSP has no concept of "not checked" — it silently omits diagnostics for regions it did not inspect. That silent omission is the exact trap `checkpoint_projection` must avoid: an agent reading a response with no errors on seams 4–5 must not conclude those seams passed.

**Agent-aware MCP tool design** (10-pattern survey, 2025 community research): agents trust tools that embed `next_actions` directly in the response, expose confidence thresholds with explicit recovery paths, document their own capability limits ("capability advertisement"), and include audit-trail provenance (sources, timestamps, model versions). Agents stall or hallucinate on: opaque pass/fail booleans with no fix hint, verbose raw dumps without a summary, and outputs that conflate "not checked" with "clean." The cited research finding: "Confidence: 0.51, below threshold — call `get_additional_context`" creates trust through transparency. Opaque numerical outputs create indecision loops.

**Schema and contract drift detectors** (dbt, data contracts, OpenAPI drift tools): standard pattern is baseline→current diff with per-field finding and provenance (which rule, which schema version produced this finding). Actionable drift classification distinguishes breaking from additive change. Best tools annotate every finding with the repair step, not just the detected delta. The 2025 community consensus: distinguishing actionable drift from benign variation reduces alert fatigue; alerts that include context (the failing feature, suggested rollback step) get acted on; alerts without context get filtered.

**Verification tool trust research**: tools that minimize false negatives (missed real failures) earn agent trust faster than tools that find more issues. A checkpoint that returns `pass` on a broken slice is the worst outcome — agents stop calling it after the first missed defect. False positives (noise findings on valid code) are the second failure mode: agents start ignoring output or filtering the tool out of the loop.

---

## Feature Landscape

### Table Stakes (Agents Expect These)

Features an agent assumes a checkpoint tool provides. Missing any = the tool is ignored or misused.

| Feature | Why Expected | Seam | Complexity | Spec Coverage |
|---------|--------------|------|------------|---------------|
| Single-call entry point anchored on projection name | Agents must not orchestrate multi-tool verification. One call: `checkpoint_projection { name: "Booking" }` | All seams | LOW | YES |
| Top-level `status: pass / warn / fail` | Agent needs one token to branch: continue vs. stop vs. review. Without it, the agent must read all seam details before deciding | Aggregated | LOW | YES |
| Per-seam status with distinct `not_checked` | "not checked" must never collapse into "pass." An agent reading `pass` on seam 5 when it was never evaluated will confidently ship a broken props contract. This is the load-bearing invariant that makes the tool trustworthy | All seams | LOW | YES — coverage honesty section |
| `next_steps` ranked list | Agents act on ordered imperatives. Failures rank before warnings; within a rank, earlier seams before later seams. Without ranking, the agent must read all findings to determine what to do first | Aggregated | LOW | YES |
| `source` provenance per finding | Agent must know which validator produced a finding to understand its reliability and to call that validator directly for detail. Without provenance, the agent cannot distinguish a checkpoint-owned finding from a sub-validator finding | All seams | LOW | YES — each seam carries `source` field |
| `subject` + `detail` per finding | Subject identifies the artifact (field name, action name); detail is the human-readable description. Both are required — LSP shape minimum. A detail without a subject is unactionable; a subject without a detail is unintelligible | All seams | LOW | YES — shown in output example |
| `fix` suggestion per finding | Actionable repair step alongside the symptom. "add column in migration, or remove field from projection" is the minimal acceptable form. An agent without a fix hint must reason about the repair from first principles — this is where hallucination enters | All seams | MEDIUM | PARTIAL — fix shown for seam 2 only. Seams 1/3/4/5 delegate to sub-validators; fix quality depends on those tools' output shapes, which are not standardized. Gap. |
| Field→column seam (seam 2) | The one gap nothing checks today. A projection field referencing a non-existent migration column is a silent runtime failure — the rendered view drops the field with no diagnostic. Agents cannot discover this without running the app. This is the F11-class seam the milestone was created to close | field→column | MEDIUM | YES — primary new check, checkpoint-owned |
| Prerequisite-gated seam execution | Seams 4–5 require a rendered view to exist. Seam 2 requires the projection to be well-formed. If a prerequisite is absent, the seam must report `not_checked` rather than silently skipping. The reason for `not_checked` must be surfaced | rendered_view, props→contract, field→column | LOW | YES — implied by coverage honesty; not fully specified per seam |
| Inline return from generate tools | The loop closes automatically: `generate_projection` and `json_ui_generate` embed the verdict in their response. The agent receives verification without a separate call, which means verification happens even when the agent is not explicitly disciplined about it | All seams | LOW | YES |

### Differentiators (What Makes This Tool Trusted Rather Than Ignored)

| Feature | Value Proposition | Seam | Complexity | Spec Coverage |
|---------|-------------------|------|------------|---------------|
| Dogfood acceptance gate | The tool must surface a real seam defect in a real consumer app before shipping. A checkpoint that finds nothing real in real apps fails acceptance and the design is revisited. This is the mechanism that prevents shipping a tool that always returns `pass` — the primary trust-destroying failure mode | All seams | LOW (process gate) | YES — testing section |
| Ranked deduplication in `next_steps` | Multiple seams can produce overlapping repair steps — for example, seam 2 and seam 4 may both flag the same missing field in different vocabulary. Deduplication prevents the agent from seeing the same action twice and having to reason about whether they are the same | Aggregated | MEDIUM | YES — "ranked, deduplicated" stated; dedup algorithm not designed. Gap: exact-string-match vs. semantic dedup distinction needed. |
| `application_info` / `projection_coverage` surface checkpoint status | Agents surveying the project see per-projection verification debt (`unverified` / `failing` / `clean`) without probing each projection individually. Converts a point-in-time tool into an ambient health signal — the agent does not need to discover that verification exists | All seams | MEDIUM | YES — specified; freshness strategy (cached vs. fresh) not decided. Gap. |
| Seam dependency topology: upstream failure → downstream `not_checked` with reason | If seam 1 fails (projection not well-formed), field names in seam 2 are unreliable. If seam 2 fails (column missing), the rendered view in seam 4 may render incorrectly. Propagating `not_checked` with `reason: "seam_1_failed"` rather than a silent skip is what allows the agent to correctly attribute blame and repair the right thing first | Aggregated | MEDIUM | NOT EXPLICIT. The spec lists seams in order and states that absent prerequisites yield `not_checked`, but does not specify that a seam failure (as distinct from absent prerequisite) propagates `not_checked` downstream. Gap. |
| Method filter threading in seam 3 | `json_ui_verify_action` supports an optional HTTP method filter. If `ActionDef` carries a declared method, the checkpoint should thread it through to avoid a false negative when GET and POST handlers share the same name | action→route | LOW | NOT EXPLICIT. Spec says "reuse `json_ui_verify_action`" without specifying whether ActionDef method is threaded. Gap. |
| Intent + confidence surfaced in seam 4 finding | The rendered view uses intent-ranked rendering. Surfacing intent name + confidence in the verdict gives the agent context to understand why a particular layout was chosen and whether a lower-confidence intent would produce a different rendering | rendered_view | LOW | PARTIAL — `render_projection` already returns intent+confidence; checkpoint output spec does not include it in seam 4 finding. Gap. |

### Anti-Features (Avoid Building These)

| Anti-Feature | Why Requested | Why Problematic | Seam | Alternative |
|--------------|---------------|-----------------|------|-------------|
| Compile / `cargo check` invocation | Agents want "does this build?" as part of verification | Slow (seconds to minutes), breaks the read-only fast contract every other MCP tool maintains. Makes the tool unusable in a tight generate→verify loop. Also redundant: the agent will invoke cargo as a separate step anyway | All | Keep as separate agent step. Spec correctly excludes this. |
| Collapsing `not_checked` into `pass` | Simpler status enum; fewer states for the agent to handle | False confidence. One false `pass` on a seam the agent later discovers was unchecked destroys trust in all future `pass` results. The LSP precedent (silent omission of unexamined regions) makes this worse, not better: the LSP has no "not checked" concept and this is documented as a known LSP weakness | All | Distinct `not_checked` with an explicit `reason` field |
| Full spec dump per seam in the verdict | Agents debugging a render failure want to see the spec | Overwhelms context window. Agents receiving kilobytes of raw JSON in a verdict stop reading after the first seam. Progressive disclosure is the correct pattern: summary verdict in-band, full detail available on demand via existing tools (`render_projection`, `json_ui_validate_spec`) | rendered_view, props→contract | Return finding counts + first N findings per seam; agent calls the sub-validator for full output |
| Aggregating all projections in one call | "Check everything" is convenient | O(n) I/O across the entire projection directory. With 20+ projections, response time and output size make the tool unusable in a loop. The ambient `projection_coverage` / `application_info` status already covers the "survey" use case at lower cost | All | Single-projection anchor per call; ambient status for bulk view |
| Type-level diff in props→contract seam (seam 5) | Full Rust↔TypeScript type mismatch detection feels more complete | `validate_contracts` already has regex-based type comparison that produces false positives on complex types (generics, nested structs, enums). Re-implementing or extending type comparison in the checkpoint creates a second source of truth. The "no duplicate control surface" rule applies directly | props→contract | Thread `validate_contracts` findings through as-is with provenance. Type comparison improvements belong in `validate_contracts`, not in the checkpoint. |
| Fix suggestions that mutate code | Agents want push-button repair | The checkpoint is a read-only introspection tool. Mutation belongs in generator tools. A tool that sometimes reads and sometimes writes is harder to reason about, harder to test, and harder to trust | All | Return `fix` as an imperative string describing what to do; separate generator tools handle actual mutation |
| Severity levels beyond pass/warn/fail/not_checked | LSP uses four levels (Error/Warning/Information/Hint); mirroring seems natural | `Information` and `Hint` produce output agents discount. The tool's job is to drive repair actions, not to be informative. Three outcome states (fail/warn/pass) plus one coverage state (not_checked) is sufficient. Adding hint-level findings inflates output without changing agent behavior | All | Use `warn` for near-misses that do not block a working slice; reserve `fail` for seams that are definitively broken |
| Caching verdict across runs | Repeated calls to the same projection are expensive | Source files change between calls. A stale cache produces false positives or false negatives depending on direction of change. The tool is fast precisely because it reads source without compiling — caching removes the last reason to tolerate staleness | All | Accept that the tool is always fresh; optimize the read path instead of caching |
| Verdict mutation (checkpoint that suggests and applies fixes in one call) | Reduces round trips | A tool that both diagnoses and repairs collapses the read→write boundary. The agent loses the ability to review and approve the fix. Agents that approve their own repairs silently are the primary source of compounding errors in agent coding loops | All | Read-only verdict; agent calls a generator tool with the fix information |

---

## Feature Dependencies

```
seam_1_well_formed
    └──required_before──> seam_2_field_to_column
                              └──required_before──> seam_4_rendered_view (meaningful)
                                                        └──required_before──> seam_5_props_contract

seam_3_action_to_route  (independent — can run in parallel with seam 2)

ranked_deduped_next_steps
    └──requires──> all seam findings collected and aggregated

inline_verdict_on_generate_projection
    └──requires──> checkpoint_projection tool exists

inline_verdict_on_json_ui_generate
    └──requires──> checkpoint_projection tool exists

ambient_status_in_projection_coverage
    └──requires──> checkpoint_projection tool exists
    └──requires──> freshness strategy decision (cached vs. fresh lightweight check)

ambient_status_in_application_info
    └──requires──> ambient_status_in_projection_coverage (same underlying data)
```

### Dependency Notes

- **Seam 1 failure propagates not_checked downstream.** If the projection is not well-formed, field names are unreliable and seam 2 results are meaningless. The checkpoint must propagate `not_checked` with `reason: "seam_N_failed"` to downstream seams when an upstream seam fails, not just when a prerequisite artifact is absent. The spec implies this but does not state it. This is a design gap.
- **Inline verdict hook depends on the tool.** `generate_projection` and `json_ui_generate` call the checkpoint after generating and embed the result. The tool must exist first; the inline hook is a thin post-generate call. Correct priority order in the implementation slice.
- **Ambient status freshness strategy is unresolved.** If `projection_coverage` stores a cached last-checkpoint-result per projection, the ambient status can go stale between edits. If it always runs a fresh lightweight check, it adds I/O to every `projection_coverage` call. The decision affects implementation complexity for item 3 of the implementation slice. This must be resolved before implementing that item.
- **Dedup algorithm for next_steps.** Exact-string-match on the `fix` string is the correct starting point. Semantic dedup (two validators describing the same repair in different words) is out of scope for the initial release.
- **Fix string normalization.** The checkpoint owns the `fix` string for seam 2 findings. For seams 1/3/4/5, it delegates to sub-validators. Sub-validators use different shapes: `diagnose_error` returns `fix_suggestions[].details`; `json_ui_verify_action` returns `message` and `candidate`; `validate_contracts` returns `mismatches[].details`. The checkpoint must define a normalization layer or explicitly pass through raw sub-validator output with a documented caveat.

---

## Gaps in the Current Spec

Issues absent from the design spec that must be resolved before or during implementation.

| Gap | Category | Impact | Resolution Needed |
|-----|----------|--------|-------------------|
| Seam failure → downstream not_checked propagation | Verdict aggregation | MEDIUM — without this, an agent sees `not_checked` on seam 3 and cannot tell if it is a cascade from seam 1 failure or an absent prerequisite | State explicitly: seam N failure propagates `not_checked` with `reason: "seam_N_failed"` to all dependent seams |
| Method threading in seam 3 (action→route) | action→route | LOW — produces a false negative only when two actions share a handler name with different methods | Specify whether `ActionDef` carries HTTP method; if so, thread it through `json_ui_verify_action` method filter |
| Intent + confidence in seam 4 finding | rendered_view | LOW — informational gap, not a correctness gap | Specify whether `RenderResult.intent` + `confidence` appear in the seam 4 finding or are omitted |
| Ambient status freshness strategy | projection_coverage / application_info | MEDIUM — cached stale result vs. fresh lightweight check are different implementation paths | Decide before implementing item 3 of the implementation slice |
| Dedup algorithm for next_steps | Aggregation | LOW — trivial for exact-string-match; non-trivial for semantic dedup | Define as exact-string-match on the `fix` string; semantic dedup out of scope |
| Fix string normalization for seams 1/3/4/5 | Verdict quality | MEDIUM — sub-validators return different shapes; agents expect uniform `fix` strings | Audit sub-validator output shapes; define normalization layer or explicit passthrough with caveat |
| Seam 2 model-resolver fallback behavior | field→column | LOW — spec says "not_checked when source model cannot be resolved" but does not specify resolution heuristic | Clarify: resolution uses the same `src/projections/` ↔ `src/models/` name-matching logic `projection_coverage` uses; on multi-match ambiguity, report `not_checked` with `reason: "ambiguous_model_match"` |

---

## MVP Definition

### Launch With (first release)

Seams that directly close the primary gap and make the loop functional.

- [ ] `checkpoint_projection` tool with seam 2 (field→column) — the primary new value; the only seam no existing tool checks
- [ ] Aggregation + ranked `next_steps` — without this, seam 2 alone is just another single-seam validator
- [ ] Coverage honesty — `not_checked` distinct from `pass`, with explicit `reason` — without this, the tool cannot be trusted
- [ ] `source` provenance per finding — without this, the agent cannot trace findings back to the producing validator
- [ ] `fix` string on seam 2 findings (checkpoint-owned) — without this, the agent must reason about the repair from first principles
- [ ] Inline return from `generate_projection` / `json_ui_generate` — closes the loop by default; standalone tool alone does not close the loop
- [ ] Dogfood acceptance: surface one real seam defect against a live consumer — gates launch per the spec

### Add After First Release

Wrapper seams that add value once the aggregation pattern is proven.

- [ ] Seam 1 (well-formed) wrapper — `validate_projection` already exists standalone; adds convenience and enables upstream-failure propagation to downstream seams
- [ ] Seam 3 (action→route) wrapper — `json_ui_verify_action` already exists standalone; adds convenience
- [ ] Seam 4 (rendered view) wrapper — higher complexity; render + validate in sequence; add after seams 2/3 prove out the pattern
- [ ] Seam 5 (props→contract) wrapper — most fragile sub-validator (regex-based); add after seam 4 is stable
- [ ] Ambient status in `application_info` / `projection_coverage` — useful project-level health signal; freshness strategy must be decided first

### Future Consideration

Refinements that improve honesty and context without changing core behavior.

- [ ] Seam failure → explicit cascade reason in downstream `not_checked` (refinement of the honesty rule)
- [ ] Intent + confidence in seam 4 finding (informational context)
- [ ] Semantic dedup for next_steps (requires semantic comparison; exact-string-match sufficient at launch)
- [ ] Method threading in seam 3 (low-impact false-negative edge case)

---

## Feature Prioritization Matrix

| Feature | Agent Value | Implementation Cost | Priority |
|---------|-------------|---------------------|----------|
| Field→column seam (seam 2) | HIGH — only check that catches the F11-class silent failure | MEDIUM — needs projection→model resolver + schema column lookup | P1 |
| Aggregation + ranked next_steps | HIGH — without it, seam 2 is not a checkpoint | LOW — pure aggregation logic | P1 |
| Coverage honesty (not_checked distinct from pass) | HIGH — trust foundation; one false pass destroys agent confidence permanently | LOW — enum variant + guard in aggregation | P1 |
| Provenance on findings | HIGH — agent cannot trace findings without it | LOW — string field on every finding | P1 |
| Fix string on seam 2 findings | HIGH — without fix hints agents hallucinate repairs | LOW — the fix string is known statically for seam 2 | P1 |
| Inline return on generate tools | HIGH — closes loop without agent discipline | LOW — post-generate call + embed | P1 |
| Dogfood acceptance gate | HIGH — prevents shipping a tool that finds nothing real | LOW (process) | P1 |
| Seam 1 (well-formed) wrapper | MEDIUM — `validate_projection` exists standalone | LOW — thin dispatch | P2 |
| Seam 3 (action→route) wrapper | MEDIUM — `json_ui_verify_action` exists standalone | LOW — thin dispatch | P2 |
| Seam 4 (rendered view) wrapper | MEDIUM — catches render-time spec failures | MEDIUM — render + validate in sequence | P2 |
| Seam 5 (props→contract) wrapper | MEDIUM — catches Rust↔TS drift | MEDIUM — `validate_contracts` regex fragility; threading carefully | P2 |
| Ambient status in coverage/application_info | MEDIUM — project-level health signal | MEDIUM — freshness strategy decision required first | P2 |
| Intent + confidence in seam 4 finding | LOW — informational context | LOW — already in RenderResult | P3 |
| Seam failure → cascade not_checked with reason | LOW — refinement of honesty rule | LOW — one guard condition | P3 |
| Dedup for semantically equivalent fix strings | LOW | MEDIUM — requires semantic comparison | P3 |

---

## Seam-by-Seam Feature Map

Quick reference: what each seam checks, what validator it delegates to, and what is new vs. existing.

| # | Seam | Check Performed | Produces | Delegated To | New Work |
|---|------|----------------|----------|--------------|----------|
| 1 | well_formed | ServiceDef round-trip validates | structural errors + warnings | `validate_projection` | NO — thin dispatch |
| 2 | field_to_column | each FieldDef name resolves to a real entity/migration column | field names with no backing column | checkpoint-owned | YES — primary new check |
| 3 | action_to_route | each ActionDef handler is a registered route | unregistered handlers + Levenshtein candidates | `json_ui_verify_action` | NO — thin dispatch |
| 4 | rendered_view | render the projection; validate the resulting JSON-UI spec | structural + catalog spec errors | `render_projection` + `json_ui_validate_spec` | NO — thin dispatch |
| 5 | props_to_contract | rendered props ↔ TypeScript interface match | field mismatches (missing/extra/type/nullability) | `validate_contracts` | NO — thin dispatch |

---

## Sources

- Design spec read directly: `docs/superpowers/specs/2026-06-09-projection-checkpoint-design.md` (HIGH confidence)
- Existing tool source read directly: `ferro-mcp/src/tools/validate_projection.rs`, `json_ui_verify_action.rs`, `validate_contracts.rs`, `projection_coverage.rs`, `diagnose_error.rs`, `json_ui_validate_spec.rs`, `render_projection.rs` (HIGH confidence)
- LSP Specification 3.17 — Diagnostic interface: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/ (HIGH confidence)
- Agent-aware MCP 10 patterns article: https://medium.com/@kumaran.isk/agent-aware-mcp-10-patterns-for-actionable-tool-responses-54029e337941 (MEDIUM confidence — single community source)
- Schema drift detection and actionable classification: https://apxml.com/courses/data-governance-quality-observability-production/chapter-3-data-observability-systems/schema-drift-detection (MEDIUM confidence)
- MCP tool trust and description–code mismatch research: https://arxiv.org/html/2602.03580v1 (MEDIUM confidence)
- Verification tool false positive/negative design: GitHub Taskflow Agent checkpoint pattern, https://github.blog/security/ai-supported-vulnerability-triage-with-the-github-security-lab-taskflow-agent/ (MEDIUM confidence)

---
*Feature research for: checkpoint_projection MCP tool — v12.5 Projection Checkpoint milestone*
*Researched: 2026-06-09*
