# Feature Research

**Domain:** Validation / measurement infrastructure for a projection/intent framework (v13.0 Compressive Validation)
**Researched:** 2026-06-12
**Confidence:** HIGH for COMP-01..03 (existing code read directly, pattern well-understood); MEDIUM for COMP-04 (benchmark methodology has no prior art in ferro); MEDIUM for COMP-05 (cross-modality is exploratory)

---

## Context

This file covers the five COMP requirements for v13.0. These are validation artifacts, not new product features. The projection/intent system they validate is already shipped: ferro-projections (7 intents, derive_intents, ServiceDef), ferro-mcp (35+ tools), ferro-json-ui (JsonUiRenderer, Spec::from_service_def), ferro-mcp-server (tools/call, tools/list). The question for each COMP item is: what does the validation artifact look like when done well, what is the genuine signal it provides, and what must be avoided to prevent vanity metrics masquerading as validation.

Gestiscilo (`gestiscilo-it/app`) is a live ferro application with 69 models, 130 JSON-UI views, and no projections yet. It uses `JsonUi::render_file` directly — the handler supplies data, the spec is a static JSON file, and there is no ServiceDef in the call chain. COMP-01 is migrating at least a meaningful subset of those views to projection-driven rendering.

The ferro `app/` sample already has 9 projection fixtures covering 7 intents (product→Browse, order→Process, revenue_dashboard→Summarize, sales_analytics→Analyze, feedback_form→Collect, todo/user→Focus, api_key→Track-adjacent). These are the existing synthetic corpus. COMP-02 formalizes this into a regression suite.

---

## Feature Landscape

### Table Stakes (Users Expect These)

For v13.0, "users" are two audiences: (a) the framework author validating whether the abstraction holds, and (b) agents and developers who will read the validation evidence as a signal about framework maturity. Missing any of these makes the validation unconvincing.

| Feature | Why Expected | COMP | Complexity | Notes |
|---------|--------------|------|------------|-------|
| At least one real-app projection migration covering a meaningful entity class | Real-world validation is a stated v1.0 criterion (#2). A framework that has only synthetic test fixtures has never been proven against messy real-world models. Gestiscilo has 69 models — the migration does not need to cover all of them, but must cover a representative subset across at least 3 intents | COMP-01 | HIGH | Cross-repo effort; see slicing section below |
| Synthetic catalog with one fixture per intent covering all 7 intents | A catalog that misses any intent has untested derivation paths. The 7-intent vocabulary was designed as a closed set; coverage must be demonstrated. The existing `app/` fixtures cover 7 intents across 9 projections but are not organized as a regression suite with assertions on primary intent | COMP-02 | LOW | 9 fixtures already exist in `app/src/projections/`; gap is regression harness, not fixtures |
| Regression tests that pin primary intent per fixture | A test that asserts "this ServiceDef derives Browse as primary" is the regression gate. Without it, a change to derive.rs that shifts the ranking is invisible until a rendered view changes unexpectedly | COMP-02 | LOW | `derive_intents()` returns a ranked `Vec<IntentScore>`; `scores[0].intent == expected` is the assertion pattern |
| Agent task + outcome record for COMP-03 | An agent-success-rate claim without a record of what the agent was asked to do and whether it succeeded is not a rate — it is a story. The record must include: the NL description given, the ServiceDef produced, whether it compiled, whether it rendered, and whether the rendered output matched the description. Without the record, the number is not reproducible | COMP-03 | MEDIUM | Harness design is the primary work; the actual agent runs are lightweight once the harness exists |
| Pass/fail criteria stated before the agent run | Criteria defined after seeing results are not criteria — they are post-hoc framing. COMP-03 requires: field names match the description, intent matches the description, rendered spec validates against the catalog, and the projection compiles. These four checks define "working output" | COMP-03 | LOW | Binary per-check; composite pass if all four pass |
| Time-to-working-app measurement that is reproducible by a second person | A benchmark no one else can reproduce is an anecdote. COMP-04 requires: a documented start condition (`cargo new` in a fresh directory, no prior ferro exposure in the session), a documented end condition (service runs, auth works, three entity types exist, one background job processes), and a clock that any developer can reproduce. The measurement apparatus must be committed alongside the result | COMP-04 | MEDIUM | Hardest to formalize; see differentiators for what "good" looks like |
| Cross-modality rendering of at least one intent as mobile, voice, and CLI | COMP-05 is a probe: does the 7-intent vocabulary make sense when the rendering target changes? The minimum is one intent rendered coherently in three non-visual forms. If the vocabulary breaks down (an intent that makes no sense as a voice interaction, or a CLI that needs a new intent), that finding is itself the validation signal | COMP-05 | MEDIUM | Exploratory; "finding" is valid output even if the finding is "the vocabulary needs revision" |
| COMP results committed as durable artifacts | Validation evidence that lives only in a session transcript cannot be cited, reviewed, or built upon. COMP results must be committed: migration diff, regression test file, agent run log, benchmark transcript, cross-modality sketch document | All | LOW | Process discipline, not implementation work |

### Differentiators (What Makes the Validation Credible)

| Feature | Value Proposition | COMP | Complexity | Notes |
|---------|-------------------|------|------------|-------|
| Gestiscilo migration covers at least 3 intent classes | Migrating only Browse (the easiest intent — products, customers, items) is not validation; it is cherry-picking. Credible migration includes at least one workflow entity (Process), one data-entry entity (Collect), and one overview entity (Summarize). Gestiscilo has bookings (Process), forms/onboarding (Collect), and statistiche/dashboard (Summarize) | COMP-01 | HIGH | The migration effort is not proportional to entity count; it is proportional to intent diversity. Three well-chosen entities beat ten Browse entities |
| Migration measures render output equivalence, not just "it compiled" | A projection migration that produces a different rendered output than the original JSON-UI view is a regression. The validation signal requires confirming that the projection-driven path produces equivalent visual output to the direct `render_file` path for the same data. Even if not pixel-identical, the primary fields and actions must be present | COMP-01 | MEDIUM | Requires a before/after comparison fixture or a snapshot test |
| Regression suite is in `ferro-projections/tests/` not `app/src/` | Tests in the sample app are app tests, not framework regression tests. A change to `derive_intents()` in `ferro-projections/src/derive.rs` must trigger the regression suite automatically. The suite belongs in the crate whose behavior it guards | COMP-02 | LOW | Move or duplicate the 9 fixtures into `ferro-projections/tests/intent_regression.rs`; keep `app/` projections as usage examples |
| Regression suite pins signal names, not just top intent | `scores[0].intent == Browse` tells you the top intent is correct. `scores[0].matching_signals.contains("entity_name")` tells you why. Signal pinning catches subtle changes to scoring weight that produce the same top intent but degrade confidence for reasons not visible from the top-level assertion. COMP-02 is stronger with both checks | COMP-02 | LOW | `has_signal(score, "entity_name")` pattern already exists in `derive.rs` test helpers |
| Agent-success-rate dataset is diverse across intents, not just easy cases | A dataset of 10 Browse projections inflates the success rate. COMP-03 requires at least 2 tasks per intent (7 intents × 2 = 14 minimum). Mix of simple and complex: simple = single-model entity with 5 fields; complex = entity with state machine, guards, and relationships | COMP-03 | MEDIUM | 14 tasks is the minimum corpus that is not obviously cherry-picked |
| Agent run uses ferro-mcp as the introspection context, not just the API docs | COMP-03 is measuring whether an agent reading `ferro-mcp` output can produce a working ServiceDef, not whether an agent that has memorized the ferro API can do it. The agent must be run with ferro-mcp as its context source. Run it without ferro-mcp loaded and the measurement is contaminated by training data | COMP-03 | LOW | Session setup: MCP server running, agent instructed to use introspection tools before generating |
| Time-to-working-app benchmark measures agent-assisted time, not manual time | COMP-04 is positioned as validation of ferro's AI-assisted authoring value. A manual benchmark (human types all code) measures typing speed, not the projection abstraction. The benchmark must be run with an agent (Claude Code, Cursor, or similar) using ferro-mcp. Record wall-clock time from `cargo new` to first successful HTTP request with all criteria met | COMP-04 | MEDIUM | Requires defining the agent's starting prompt carefully — the prompt is part of the benchmark apparatus |
| COMP-05 produces a written intent → rendering map, not just code | The cross-modality sketch is architectural thinking, not implementation. The artifact is a document: for each of the 7 intents, describe what a mobile, voice, and CLI rendering would look like, identify which intents survive the translation cleanly, which require adaptation, and which reveal vocabulary gaps. This document directly informs whether v14.0 Channel Projection needs intent vocabulary changes | COMP-05 | LOW | A 3-page document is the right scope; prototype code is optional and not the primary artifact |
| COMP-05 identifies at least one vocabulary gap | A cross-modality analysis that finds "all intents translate cleanly" is either correct (rare) or insufficiently critical. The exercise is designed to probe limits. If Process maps cleanly to mobile (it likely does — status transitions are modality-independent), but Analyze does not map cleanly to voice (time-series charts do not have a natural voice form), that finding constrains the v14.0 channel projection design. A gap is a valuable finding, not a failure | COMP-05 | LOW | The analysis is honest if it records "Analyze does not have a natural voice form; the closest is a summary report read aloud, which is closer to Summarize" |

### Anti-Features (Avoid These)

| Anti-Feature | Why Requested | Why Problematic | Alternative |
|--------------|---------------|-----------------|-------------|
| Migrating all 130 gestiscilo JSON-UI views to projections | Thoroughness signal; demonstrates commitment | At 130 views with 69 models, this is months of work. It also conflates "projection coverage" with "projection validation" — a complete migration that produces no regression evidence is not COMP-01. COMP-01 needs a slice that produces clear validation signal, not maximum coverage | Migrate 5–8 entities covering 3+ intent classes; capture before/after render equivalence; document what the migration revealed about the abstraction |
| Success rate measured as "the ServiceDef compiled" | Easy to measure; quantitative; sounds like a success rate | Compilation is not the bar. A ServiceDef that compiles, renders to a blank page, and has no relation to the NL description it was generated from is not a working projection. The bar is: field names match, intent matches, rendered spec validates, projection compiles | Define pass/fail criteria before running the agent, as stated above |
| Time-to-working-app benchmark run on a machine with a warm Rust cache | Reproducible and fast to execute | A warm cache makes the benchmark 3–4× faster than a cold build. The benchmark measures developer experience, which is experienced cold. Cold cache is the honest measurement condition | `cargo clean` before each run; document Rust toolchain version and machine specs alongside the result |
| Cross-modality sketch that only covers Browse and Collect | Those two intents are the easiest to sketch (list → scrollable list on mobile, form → form on mobile) | Browse and Collect map cleanly to every modality because they are structurally simple. The interesting tension is in Process (state machines on voice?), Analyze (time-series on CLI?), and Track (audit timeline on mobile?). Sketching only easy cases understates the design tension | All 7 intents must appear in the sketch; the interesting findings will cluster around Process, Analyze, and Track |
| Synthetic catalog fixtures that are trivially derivable | Each fixture confirms coverage | A fixture where the primary intent is obvious from a single field (e.g., a ServiceDef with one `FieldMeaning::Money` field deriving Summarize) does not stress-test the derivation engine. Interesting fixtures have mixed signals that require signal weighting to resolve correctly | Each fixture should have at least two competing intents (e.g., a Collect fixture also has EntityName fields that produce Browse signal); the test confirms the intended intent wins despite competition |
| Agent-success-rate run on the latest agent that was trained on ferro code | Inflated pass rate; that agent has prior knowledge | If the agent was trained on the ferro codebase, the success rate reflects memorization, not whether `ferro-mcp` is sufficient for zero-prior-knowledge generation. COMP-03 is measuring the MCP surface, not the agent's training data | Use an agent with explicit instruction to rely on tool output; alternate: run with ferro-mcp disabled as a control condition and compare |
| COMP results stored as unstructured prose in a phase completion note | Lower overhead; still records the finding | Unstructured prose cannot be diffed, cited by a later phase, or used as a regression baseline. A Collect intent benchmark run that lives only in a phase completion note is invisible to future work | Commit structured artifacts: JSON for agent run outcomes, Markdown with a defined schema for benchmark results, a Rust test file for regression assertions |

---

## Feature Dependencies

```
COMP-02 (regression harness)
    └──prerequisite for──> COMP-03 (agent tasks use regression fixtures as test cases)
    └──prerequisite for──> meaningful COMP-01 (migration fixtures validated by harness)

COMP-01 (gestiscilo migration)
    └──depends on──> gestiscilo running on current ferro (already true: gestiscilo uses ferro 0.2.x)
    └──depends on──> ServiceDef author understanding gestiscilo's domain model (read models/ dir)
    └──produces──> real-world projection fixtures that can join COMP-02 corpus

COMP-03 (agent success rate)
    └──depends on──> ferro-mcp server running against a project with known projections
    └──depends on──> COMP-02 fixtures to define the task corpus
    └──depends on──> pass/fail criteria defined before agent run (not after)

COMP-04 (time-to-working-app)
    └──depends on──> ferro-cli `ferro new` template that includes projection-driven starter option
    └──depends on──> ferro-mcp server accessible during the benchmark session
    └──independent of──> COMP-01..03 (can run in parallel)

COMP-05 (cross-modality sketch)
    └──depends on──> all 7 intents understood (COMP-02 fixtures provide concrete examples)
    └──produces──> vocabulary gap findings that constrain v14.0 Channel Projection scope
    └──independent of──> COMP-01, COMP-03, COMP-04

COMP-01 is the largest effort and is sliceable:
    Slice A (minimal): 3 entities × 1 intent each (Browse: product/client; Process: booking; Summarize: dashboard)
    Slice B (full): +2 entities × 2 more intents (Collect: onboarding form; Analyze: statistiche)
    Slice A validates the core three structurally distinct intents; Slice B adds depth but can be deferred
```

### Dependency Notes

- **COMP-02 should run first or in parallel with COMP-01.** The regression harness gives you a framework for capturing COMP-01 migration results as permanent tests, not just a one-time diff.
- **COMP-05 is unblocked immediately.** It requires no code changes — only reading the existing intent definitions, the ferro-projections fixtures, and thinking about non-visual rendering. It can be done as a standalone doc-writing exercise in a single session.
- **COMP-03 and COMP-04 both require a running ferro-mcp server.** They can be batched into the same session after COMP-02 is in place.
- **COMP-01 is cross-repo.** It requires changes in `gestiscilo-it/app` (adding `src/projections/`), changes to `gestiscilo-it/app/src/controllers/` (switching from `JsonUi::render_file` to `JsonUi::render` with a ServiceDef-sourced Spec), and validation that the rendered output is equivalent. Plan for it as a multi-session effort.

---

## MVP Definition

### Launch With (v13.0 first slice)

The minimum set that produces durable, non-vanity validation evidence.

- [ ] COMP-02: regression harness in `ferro-projections/tests/intent_regression.rs` with one fixture per intent, each asserting primary intent and at least one key signal — establishes the regression baseline and is permanently machine-checkable
- [ ] COMP-05: cross-modality sketch document committed under `docs/` or `.planning/` — unblocked immediately, produces the vocabulary gap analysis v14.0 needs, no implementation work
- [ ] COMP-01 Slice A: 3 gestiscilo entities migrated to projection-driven rendering (Browse + Process + Summarize), before/after render equivalence documented — validates the abstraction against a messy real codebase without requiring full migration

### Add After First Slice

- [ ] COMP-03: agent-success-rate measurement with 14-task corpus, structured result artifact — requires COMP-02 as task corpus
- [ ] COMP-04: time-to-working-app benchmark, agent-assisted, cold cache, committed result with apparatus documentation — requires `ferro new` projection-starter option

### Future Consideration

- [ ] COMP-01 Slice B: 2 additional gestiscilo entities (Collect, Analyze) — extend migration after Slice A confirms the pattern works
- [ ] COMP-02 extended: add COMP-01 migration fixtures to the regression corpus so gestiscilo projections are continuously validated against derive.rs changes
- [ ] COMP-03 re-run: repeat agent-success-rate measurement after any significant change to ferro-mcp tool descriptions — the rate should improve or hold; regression is a signal to investigate

---

## Feature Prioritization Matrix

| Feature | Validation Value | Implementation Cost | Priority |
|---------|-----------------|---------------------|----------|
| COMP-02 regression harness (all 7 intents) | HIGH — permanent machine-checkable baseline; catches derive.rs regressions | LOW — fixtures exist; write test assertions | P1 |
| COMP-05 cross-modality sketch | HIGH — unblocks v14.0 scope decision; no code | LOW — document writing | P1 |
| COMP-01 Slice A (3 entities, 3 intents) | HIGH — only real-world validation; surfaces abstraction gaps that synthetic corpus cannot | HIGH — cross-repo, requires render equivalence verification | P1 |
| COMP-03 agent success rate (14 tasks) | MEDIUM — measures the MCP surface quality directly | MEDIUM — harness design + 14 agent runs | P2 |
| COMP-04 time-to-working-app benchmark | MEDIUM — concrete developer-experience evidence | MEDIUM — requires clean apparatus documentation | P2 |
| COMP-01 Slice B (2 more entities) | LOW (incremental) — adds coverage but not qualitatively new signal | HIGH — more cross-repo work | P3 |

**Priority key:** P1 = validates the compressive dimension directly and is required to call COMP done; P2 = adds measurement depth and is required for a credible public v1.0 claim; P3 = incremental coverage, defer unless COMP-01 Slice A reveals gaps that Slice B would address.

---

## Per-COMP "Good" Criteria

These define what passes validation for each requirement. They are the pass/fail criteria that must be stated before the work begins, not after.

### COMP-01: Gestiscilo Migration

**Good looks like:**
- At least 3 entities across 3 intent classes migrated, each with a ServiceDef in `gestiscilo-it/app/src/projections/`
- For each migrated entity, the projection-driven render path (`Spec::from_service_def` → `JsonUi::render`) produces HTML that contains the same primary fields as the original `JsonUi::render_file` path for representative sample data
- At least one finding documented: something the migration revealed about the projection abstraction that would not have been visible from the synthetic corpus (gap in FieldMeaning coverage, intent derivation edge case, etc.)
- The migration diff compiles and the gestiscilo test suite (if any) stays green

**Not good (vanity):**
- "All 130 views migrated" with no render equivalence check
- "10 entities migrated" with only Browse intent (cherry-picked)
- ServiceDef that produces a compiled projection but renders a view missing half the fields from the original

### COMP-02: Synthetic Catalog Regression Suite

**Good looks like:**
- One test per intent (7 tests minimum); each test asserts `scores[0].intent == expected_intent`
- Each test also asserts at least one key signal is present in `scores[0].matching_signals`
- Each fixture has at least one competing signal from a different intent to confirm the intended intent wins under competition
- Tests live in `ferro-projections/tests/` and run on every `cargo test` in the workspace
- CI catches a hypothetical regression: if `analyze_field_meanings` weight for `Summarize` were doubled, at least 2 tests would fail

**Not good (vanity):**
- Fixtures with only one field that trivially derives one intent (no competition)
- Tests that only assert `!scores.is_empty()` without pinning the primary intent
- Tests in `app/src/` that don't run as part of the framework's own test suite

### COMP-03: Agent Success Rate

**Good looks like:**
- 14+ tasks (2 per intent), each with an explicit NL description and a binary pass/fail per four criteria (compiles, renders, intent matches, field names match)
- Agent run with ferro-mcp active; the introspection tool call log is committed alongside the result
- Aggregate pass rate calculated and committed as a structured artifact
- At least one task per intent fails (a 100% pass rate with no analysis of failure modes is suspicious)
- The artifact includes: which tasks failed, what the agent produced, what the expected output was

**Not good (vanity):**
- "The agent successfully generated a projection" with no stated criteria for success
- Only Browse and Collect tasks (easiest intents)
- Agent run without ferro-mcp context (measures training data, not the MCP surface)

### COMP-04: Time-to-Working-App Benchmark

**Good looks like:**
- Start condition precisely documented: `cargo new` in a fresh directory, `cargo clean`, Rust toolchain version stated, machine specs stated
- End condition precisely documented: HTTP 200 on the root route, auth endpoint returns 200, all 3 entity types have a working list route, one background job processes successfully from the queue
- Wall-clock time recorded at start and end with a method that a second person could reproduce (terminal recording, screen capture timestamp, or timestamped commit log)
- Agent-assisted run with ferro-mcp active (not manual)
- Result is a single committed Markdown file with the apparatus, the transcript reference, and the time

**Not good (vanity):**
- "It took about 2 hours" with no start/end condition documentation
- Warm Rust cache
- Manual (non-agent-assisted) run
- Run on a machine with unusual hardware that makes the number non-representative

### COMP-05: Cross-Modality Sketch

**Good looks like:**
- All 7 intents appear in the document
- For each intent, mobile, voice, and CLI rendering described concretely (not just "it would work")
- At least one intent identified where the vocabulary requires adaptation for non-visual modalities
- At least one vocabulary gap or tension identified (e.g., "Analyze has no natural voice form; the voice modality needs a new intent or a Summarize variant")
- Direct implication for v14.0 Channel Projection scope stated: what the analysis changes about the design assumptions

**Not good (vanity):**
- Only Browse and Collect sketched (easiest)
- "All intents translate cleanly to all modalities" with no gaps found
- Sketch that describes what components would look like instead of whether the intent vocabulary survives the translation

---

## Sources

- ferro-projections/src/intent.rs, service.rs, derive.rs — read directly (HIGH confidence)
- app/src/projections/ — 9 existing fixtures read directly (HIGH confidence)
- gestiscilo-it/app/src/ — model count, controller list, view count, rendering approach confirmed directly (HIGH confidence)
- ferro-ai/tests/projection_roundtrip.rs — projection roundtrip test pattern confirmed directly (HIGH confidence)
- .planning/PROJECT.md — v13.0 COMP-01..05 requirements, v1.0 criteria, four beauty dimensions (HIGH confidence)

---
*Feature research for: v13.0 Compressive Validation — COMP-01..05 validation artifacts*
*Researched: 2026-06-12*
