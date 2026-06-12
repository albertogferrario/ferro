# Phase 210: COMP-03 — Agent-Success-Rate Harness - Context

**Gathered:** 2026-06-13 (--auto; recommended defaults selected and logged)
**Status:** Ready for planning

<domain>
## Phase Boundary

A harness in `ferro-mcp/tests/agent_harness.rs` that measures whether an LLM
agent — reading `ferro-mcp` introspection tools — can turn a natural-language
description into a **working projection** (a `ServiceDef` that renders and passes
the projection checkpoint). Scored against 4 cumulative tiers, ≥3 trials per
task, corpus spanning all 7 intents, with a committed baseline artifact.

**Locked by COMP-03 (not open for discussion):** file path; 14+ tasks (2 per
intent); 4 tiers stated before any run; ≥3 trials; `rmcp 0.12` in-process
transport driving the dev tools (NOT `ferro-mcp-server`); committed baseline
(model + prompt version, per-tier rates); contamination guard required. The
**success-rate floor is set AFTER the first baseline run** — not in this phase.

This phase builds the harness and produces the first committed baseline. It does
NOT set a CI pass threshold and does NOT modify `intent.rs`/`derive.rs`/renderers.
</domain>

<decisions>
## Implementation Decisions

### Execution model
- **D-01:** Hybrid execution. Live agent runs are gated behind `FERRO_AGENT_EVAL=1`
  (normal `cargo test` / CI skip them — no API key, no network, no cost, no LLM
  flakiness). A gated run drives the agent live and writes/refreshes the committed
  **baseline** + per-task **transcripts**. A non-gated path replays the committed
  transcripts through the **same scorer**, so CI guards the scorer + tier logic
  deterministically without LLM calls. Mirrors the `FERRO_BENCH=1` gate pattern
  used for COMP-04 (Phase 211).
- **D-02:** Baseline model pinned to `claude-opus-4-8` (latest/most capable per
  project default), recorded verbatim in the baseline artifact alongside the
  prompt version and per-tier rates. Cost is bounded and on-demand: ~14×3 = 42
  calls per full gated run.
- **D-03:** Reuse the **ferro-ai completion client** (`ferro-ai/src/complete.rs`,
  `ferro-ai/src/client/`) for the live call if it exposes a usable text/tool
  completion API; otherwise a minimal Anthropic client confined to the test
  target (`#[cfg(test)]` / gated), adding NO always-on dependency to ferro-mcp's
  default build. Researcher confirms the ferro-ai surface first.
- **D-04:** The agent runs as an **in-process rmcp client** over the dev tools
  (rmcp 0.12 in-process / async-rw transport), not `ferro-mcp-server`. ≥3 trials
  per task.

### Agent output + toolset
- **D-05:** The agent's deliverable is a **`ServiceDef`** (structured JSON the
  harness deserializes into `ferro_projections::ServiceDef`). Success is measured
  on the chain `ServiceDef` → `derive_intents` → `Spec::from_service_def` →
  `checkpoint_projection`. Note: `generate_projection` derives a ServiceDef *from
  a SeaORM model*, not from NL — so the agent **hand-authors** the ServiceDef
  guided by the introspection tools; it is not a single tool call.
- **D-06:** Agent toolset = `generation_context` (authoring guidance),
  `json_ui_catalog` (component + intent vocabulary), `checkpoint_projection`
  (self-verify before submitting). `generate_projection` is **excluded** (it is
  model-derivation, not NL authoring). The agent reads these, then emits the
  ServiceDef.

### Tier pass definitions (cumulative; stated before any run)
- **D-07:**
  - **T1 Structural validity** — the ServiceDef deserializes AND
    `Spec::from_service_def` renders AND `Catalog::validate(&spec)` returns 0 errors.
  - **T2 Intent coverage** — `derive_intents(&service)[0].intent` equals the task's
    declared target intent.
  - **T3 Functional completeness** — the rendered spec's primary content element is
    data-bound, not a placeholder, per the Phase 213 content-binding bar:
    Browse/Track `DataTable` has non-empty `columns` + `data_path`; Process
    `KanbanBoard` has `columns` + `items_path` + `group_by`; Collect `Form` has ≥1
    field; Summarize `StatCard` has `value_path`; Focus/Analyze primary fields
    bound. No empty/placeholder values.
  - **T4 Checkpoint pass** — `checkpoint_projection` returns a verdict with zero
    blocking findings.
- **D-08:** A task-trial passes tier N iff it passes tiers 1..N. The baseline
  reports per-tier pass rate across all 14 tasks × ≥3 trials.

### Corpus + contamination guard
- **D-09:** Corpus = 14 hand-authored NL tasks (2 per intent, all 7 intents), each
  declaring its target intent (for T2). Committed as a fixtures file alongside the
  harness.
- **D-10:** Contamination guard = **invented synthetic domains** — entity / field /
  domain nouns NOT drawn from ferro docs, the sample app, gestiscilo, or the Phase
  207 catalog; phrasing paraphrased so it does not quote ferro's own intent
  vocabulary. The agent must *derive* intent + field meanings, not pattern-match
  memorized examples.
- **D-11:** Tasks are realistic-but-novel business descriptions spanning the 7
  intents (e.g., Process: "a registry of telescope observation-time slots that
  staff move through requested → scheduled → observed → archived").

### Claude's Discretion
- Exact prompt template + prompt-version string; transcript file format; trial
  aggregation (per-trial vs mean); the specific 14 invented domains (within the
  D-10 guard); baseline-artifact file layout.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirement + roadmap
- `.planning/REQUIREMENTS.md` §COMP-03 — the binding requirement (multi-tier,
  ≥3 trials, baseline contents, in-process transport, contamination guard).
- `.planning/ROADMAP.md` §"Phase 210" + the "open decisions" note (success-rate
  floor deferred to after first baseline).

### Agent toolset (the introspection surface under test)
- `ferro-mcp/src/tools/generation_context.rs` — authoring guidance tool.
- `ferro-mcp/src/tools/mod.rs` + `json_ui_catalog` tool — component/intent vocabulary.
- `ferro-mcp/src/tools/checkpoint_projection.rs` — T4 verdict source.
- `ferro-mcp/src/tools/generate_projection.rs` — EXCLUDED from the toolset (model
  derivation, not NL); read to confirm the exclusion rationale.

### Scoring pipeline
- `ferro-projections/src/derive.rs` (`derive_intents`) — T2.
- `ferro-json-ui/src/projection/builder.rs` (`Spec::from_service_def`) + catalog
  validate — T1/T3.
- `.planning/phases/213-projection-render-completeness/213-06-SUMMARY-gap-a-root-fix.md`
  — the content-binding completeness bar T3 enforces.

### Reuse candidates
- `ferro-ai/src/complete.rs`, `ferro-ai/src/client/mod.rs` — live LLM client (D-03).
- `ferro-projections/tests/catalog.rs` — the 7 canonical intent shapes; reference
  ONLY (its domains must NOT seed the task corpus — contamination, D-10).
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **ferro-ai completion client** (`ferro-ai/src/complete.rs`, `client/`) — live LLM call.
- **checkpoint_projection** MCP tool — T4 scorer, already validates the field→column seam.
- **derive_intents + Spec::from_service_def + Catalog::validate** — T1/T2/T3 scorers.
- **Phase 207 catalog builders** — intent-shape reference (not corpus).

### Established Patterns
- **Gated env-var test** — mirror COMP-04's `FERRO_BENCH=1` with `FERRO_AGENT_EVAL=1`.
- **rmcp 0.12 in-process transport** — already a dependency; may need the
  in-process/async-rw transport feature enabled.

### Integration Points
- New `ferro-mcp/tests/agent_harness.rs` (greenfield — `ferro-mcp/tests/` is empty).
- Committed baseline + transcripts + task corpus under the phase/test dir.
</code_context>

<specifics>
## Specific Ideas

- Hybrid gate so the project's "always-green, no-network `cargo test`" invariant
  holds: live eval is opt-in; CI replays transcripts deterministically.
- T3 is the tier that Phase 213 just made meaningful — before 213 every Process /
  Summarize / actions task would have failed T3 on placeholders. This harness is
  the standing regression guard that the render stays content-complete.
</specifics>

<deferred>
## Deferred Ideas

- **Success-rate floor / CI pass threshold** — set AFTER the first committed
  baseline run (ROADMAP open decision); a follow-up, not this phase. Must flag
  genuine regression without being fragile to LLM variance.
- **Expanding the corpus beyond 2/intent or adding multi-step agent tasks** —
  future hardening; this phase establishes the 14-task baseline.

### Reviewed Todos (not folded)
None — no pending todos matched Phase 210.
</deferred>

---

*Phase: 210-comp-03-agent-success-rate-harness*
*Context gathered: 2026-06-13 (--auto)*
