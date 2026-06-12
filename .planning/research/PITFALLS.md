# Pitfalls Research — v13.0 Compressive Validation

**Domain:** Empirical validation harnesses for a projection/intent framework (synthetic catalogs, agent-success measurement, benchmarks, cross-repo migration)
**Researched:** 2026-06-12
**Confidence:** HIGH — grounded in the existing codebase, v13.0 COMP-01..05 scope, known friction patterns (friction-loop release cadence, dogfood acceptance lessons from v12.5), and established evaluation research on LLM agent harnesses and snapshot test brittleness.

---

## Critical Pitfalls

### Pitfall 1: SNAPSHOT OSSIFICATION — golden corpus asserts current renderer output and breaks on every legitimate change

**What goes wrong:**
The synthetic catalog (COMP-02) is built by running the current `JsonUiRenderer` over a set of `ServiceDef` fixtures and saving the HTML/JSON output as golden files. Tests compare future output byte-for-byte against the saved snapshots. Within weeks, a routine renderer improvement (better Tailwind class, margin adjustment, new aria attribute) causes every golden file to fail. The maintainer runs `--update-snapshots` in bulk to restore green, learning nothing about whether the change was correct. The corpus now asserts the new output — which might itself contain a regression. The snapshot update ritual trains everyone to approve bulk changes without reviewing them.

**Why it happens:**
Golden file tests are the path of least resistance for output-heavy systems. The initial corpus is built cheaply by running the renderer once and committing the output. The tests "protect" the output, but what they really protect is the last bulk update. The more snapshots exist, the more expensive they are to review carefully, so the review threshold falls over time.

**How to avoid:**
- Structure the catalog around structural invariants, not byte-identical output. Tests assert: the rendered output contains a navigation element when the intent is Browse; the form has an `<input>` for every `FieldDef` with `Collect` meaning; the correct number of table columns are emitted for a Summarize intent. These tests survive renderer polish.
- Reserve exact-match golden files only for a small set of "canonical shapes" — one reference output per intent — that are treated as intentional contracts, not incidental snapshots. Updating a canonical shape requires a deliberate decision and a note in the commit explaining why the intent's canonical rendering changed.
- Run a diff-review gate: any snapshot update must be accompanied by a changelog entry naming the intent and the nature of the change. A bulk update with no description is a failing PR.

**Warning signs:**
- The test suite has more snapshot files than test files asserting structural properties.
- `--update-snapshots` has been run more than twice in the milestone without accompanying changelog entries.
- No test in the catalog would fail if the renderer emitted an empty `<div>` for a Browse intent.

**Phase to address:** COMP-02 (synthetic catalog). The invariant-vs-snapshot distinction must be established in the first deliverable. Adding structural invariants later, after a large snapshot corpus exists, is expensive.

---

### Pitfall 2: CATALOG OVERFITTING — the corpus only covers what already works, so it never catches regressions

**What goes wrong:**
The synthetic catalog is assembled by choosing `ServiceDef` fixtures that produce clean, representative output under the current renderer. Classes that produce known rendering gaps (e.g., a Track intent with a complex state machine, a multi-level Analyze intent with nested aggregations) are excluded because they expose a bug. The catalog passes on every run. Future intent vocabulary changes that break those excluded classes are not caught. The catalog rubber-stamps, it does not probe.

**Why it happens:**
The catalog author knows what works. Including broken cases requires explaining them; excluding them keeps the test suite green. The natural incentive is to ship a catalog that passes rather than one that surfaces weaknesses. This mirrors benchmark overfitting: the system is tuned to the test distribution, not to the real distribution of app classes.

**How to avoid:**
- The catalog must include at least one fixture per intent that exercises a non-trivial rendering path: a Browse with more than 8 columns (pagination/overflow), a Process intent with a state machine, a Track intent with multiple timeline event types. If any of these currently fail, they are added as `#[ignore]` cases with a linked issue — not excluded. The issue is either fixed before COMP-02 ships or explicitly deferred with a written decision.
- Before finalizing the catalog, explicitly ask: "Is there a class of application this catalog would not catch a regression on?" If yes, that class gets a fixture.
- The catalog serves the v1.0 criterion "projection / intent validated through a synthetic catalog of canonical app classes." A catalog that only tests already-working cases does not meet that criterion.

**Warning signs:**
- All 7 intents produce clean output on the first run, with no fixtures flagged for known limitations.
- No fixture exercises more than a minimal `ServiceDef` (2 fields, 1 action).
- The catalog was assembled in less than a day (insufficient time to encounter edge cases).

**Phase to address:** COMP-02. The catalog scope review — specifically the "what would this miss?" question — must happen before implementation begins, not after the fixtures are written.

---

### Pitfall 3: AGENT EVAL GAMING — pass criteria reward superficial output and are gameable by the agent

**What goes wrong:**
The COMP-03 agent-success-rate harness defines success as "the agent produced a `ServiceDef` that compiles and renders without a panic." An agent (or a future agent) learns to emit a minimal `ServiceDef` — correct field types, no actions, no state machine — that satisfies the compilation check. The harness reports high pass rates. The framework ships as "validated." Real applications built with it have incomplete projections that require hand-correction at every action and guard boundary. The harness measured syntactic correctness, not semantic usefulness.

**Why it happens:**
Compilation is an objective, cheap-to-check proxy for correctness. It is tempting to use it as the primary success criterion because it is unambiguous and deterministic. But compiling is a floor, not a ceiling. An agent that produces a structurally empty `ServiceDef` for any input will pass a compilation-only harness at 100%.

**How to avoid:**
- Define a multi-tier success criterion before building the harness. Minimum levels:
  1. **Structural validity**: the `ServiceDef` compiles and passes `validate_projection`.
  2. **Intent coverage**: the primary intent derived from the NL description matches the agent-authored projection's primary intent (e.g., a description of an order management board produces a `Process` or `Track` intent, not `Collect`).
  3. **Functional completeness**: actions named in the NL description are present as `ActionDef` entries with matching route shapes; guards referenced are present as `GuardDef` entries.
  4. **Checkpoint pass**: `checkpoint_projection` returns `pass` or `warn` (not `fail`) on the agent-authored projection.
- Each tier is reported separately. A harness result that shows tier-1 pass rate alongside tier-3 pass rate is honest about what was measured.
- Never aggregate all tiers into a single pass/fail score. The aggregate hides which tier is the bottleneck.

**Warning signs:**
- The harness reports >90% pass rate on the first run without any iteration on pass criteria.
- The success criterion can be satisfied by an empty `ServiceDef` with two filler fields.
- The harness has a single boolean `passed` output rather than a per-tier breakdown.

**Phase to address:** COMP-03. The multi-tier criterion must be designed before any agent runs are collected. A harness that starts with tier-1 only and defers tier-2 and tier-3 to "a future improvement" will never add them — the baseline number becomes a commitment.

---

### Pitfall 4: NON-DETERMINISM DRIFT — flaky LLM runs make the harness appear to detect regressions that aren't there

**What goes wrong:**
The COMP-03 harness runs each NL description against the agent once and records pass/fail. On the next run (different temperature, model update, minor prompt change), 15% of previously-passing cases fail. The team interprets this as a framework regression and begins investigating. The real cause is LLM non-determinism: the same prompt produces structurally different output on different runs. If the harness has no statistical baseline, every run is noise. The team loses trust in the harness and stops acting on it.

**Why it happens:**
LLMs are stochastic. A single-trial pass rate is a point estimate with high variance. Building a harness that runs each case once is natural because it is cheap — but it conflates LLM variance with framework regression.

**How to avoid:**
- Each test case runs at minimum 3 trials. Report the pass rate per case (e.g., "2/3 trials passed tier-2"). A case is marked as stable when it achieves 3/3 on structural validity (tier-1); 3/3 is not required for tier-3.
- Establish a baseline pass rate for the catalog on a known-stable model snapshot. A subsequent run is a regression only if the pass rate drops by more than a defined threshold (e.g., >10 percentage points) from the baseline.
- Use temperature=0 (or the lowest determinism setting available) for structural validity tier to reduce variance. Accept that higher tiers will be noisier.
- Record the model version and prompt version alongside each harness run. A harness result without provenance is uninterpretable.

**Warning signs:**
- The harness has no concept of "baseline." Every run is compared against a conceptual ideal of "100% pass."
- Two consecutive runs on the same day produce materially different pass rates with no framework changes.
- The harness uses temperature defaults (typically 1.0) for structural validity checks.

**Phase to address:** COMP-03. Multi-trial design must be specified upfront. Retrofitting it after data collection has started requires discarding the single-trial data.

---

### Pitfall 5: VANITY BENCHMARK — time-to-working-app measures only the happy path and a clean environment

**What goes wrong:**
The COMP-04 benchmark scripts `cargo new`, wires auth, three entity types, and a background job, and records wall-clock time. The benchmark runs on a developer laptop with a warm Cargo cache, a pre-installed Rust toolchain, and a local Postgres instance already running. The time recorded is 4 minutes. This number is quoted in documentation as "ferro gets you to a working app in under 5 minutes." A first-time user on a CI runner or a fresh machine spends 25 minutes waiting for `cargo build` and another 10 debugging a missing `DATABASE_URL`. The benchmark measures the experience of someone who already knows ferro.

**Why it happens:**
Benchmarks are run by the people who built the framework in the environment they use daily. The happy path is well-known, the environment is pre-configured, and caches are warm. Time-to-working measurements are environment-sensitive but are typically reported as if they were environment-independent.

**How to avoid:**
- The benchmark must specify its environment explicitly: cold/warm cache, toolchain version, database availability. A result without an environment spec is not a benchmark, it is an anecdote.
- Run the benchmark in at least two environments: warm (developer machine, warm Cargo cache) and cold (fresh Docker container, no pre-installed toolchain, no Cargo cache). Report both times. The cold time is the honest "first-time experience" number.
- Instrument the unhappy paths explicitly: what happens when `DATABASE_URL` is absent, when `cargo build` fails because a dependency update broke a compile, when a migration fails. These paths must have documented recovery times alongside the happy path.
- The benchmark is a structural diagnostic, not a marketing number. Its value is in identifying where the most time is spent (compilation? database setup? CLI scaffolding?) so that investment reduces that bottleneck.

**Warning signs:**
- The benchmark result is reported without specifying Cargo cache state.
- No cold-cache run exists.
- The benchmark is used as a headline number in a README before the unhappy paths have been measured.

**Phase to address:** COMP-04. Environment specification and cold-cache measurement must be part of the benchmark design. A warm-cache-only result is acceptable as an internal diagnostic but must never be reported externally as "time to working app."

---

### Pitfall 6: GESTISCILO BIG-BANG MIGRATION — treating COMP-01 as an atomic swap rather than a sliced roll-forward

**What goes wrong:**
COMP-01 is scoped as "migrate gestiscilo to projection-driven rendering." The implementer interprets this as: remove all hand-authored views, replace them with `ServiceDef` + `JsonUiRenderer` in a single branch, verify everything works, merge. The branch accumulates changes across 40+ views over 3 weeks. Meanwhile, ferro-projections changes are made to support the migration. The branch diverges from master. Merging requires resolving conflicts against 30+ ferro commits. During reconciliation, projection API changes that were reverted in main (because another phase found them wrong) are re-introduced. The merge is a 2-day fire drill.

**Why it happens:**
Big-bang is easier to reason about: one state (before) transitions to another (after). Slicing requires maintaining two rendering paths in parallel, which feels like technical debt. But a multi-week cross-repo branch in an active-development framework is not a stable migration strategy.

**How to avoid:**
- COMP-01 must be sliced: migrate one view at a time, merging each slice to master before starting the next. The `ServiceDef` renders the new view; the old view code is deleted; the merge happens. The next slice begins from a clean main.
- The migration order should be: simplest intent first (typically a Browse projection over a flat model), most complex last (a Process or Track projection with a state machine).
- Do not publish a new ferro version mid-migration. Per the friction-loop release cadence lesson, publish once at the end of the migration series — not after each slice if slices change the ferro API. If a slice needs a ferro API change, batch all necessary ferro changes and publish together at the end of the COMP-01 series.
- Track the migration state explicitly: a table in the phase notes listing each view, its migration status, and the ferro version it was migrated against. This prevents "we migrated this against 0.2.54 but it needs to be retested against 0.2.57" surprises.

**Warning signs:**
- The COMP-01 branch has been open for more than 2 weeks.
- Ferro API changes are being made on master while the COMP-01 branch is open.
- The branch is not merging to master slice-by-slice; each merge waits for the entire migration to complete.

**Phase to address:** COMP-01 planning. The slice-by-slice strategy must be specified before work starts. A big-bang plan should be rejected at planning review, not discovered after weeks of divergence.

---

### Pitfall 7: PREMATURE INTENT VOCABULARY REVISION — COMP-05 sketch triggers immediate intent redesign

**What goes wrong:**
COMP-05 produces a cross-modality sketch: one intent (e.g., Browse) expressed as mobile, voice, and CLI. The sketch reveals that "Browse" means different things in different modalities — mobile Browse involves scrollable cards with swipe actions; voice Browse involves a prompted enumeration; CLI Browse involves a paged list with filter flags. The team concludes that the Browse intent is too coarsely defined and begins revising the seven-intent vocabulary mid-milestone. The revision cascades: `derive_intents.rs`, all renderers, all catalog fixtures, gestiscilo's projections, all documentation. COMP-02 and COMP-03 data collected before the revision is invalidated. The milestone loses coherence.

**Why it happens:**
A sketch is generative: it surfaces genuine vocabulary gaps. The natural response to finding a gap is to fix it. But the fix requires touching every downstream system, and a v13.0 milestone is not the right scope for a fundamental vocabulary revision. The intent is to "inform any intent vocabulary revision," not to perform one.

**How to avoid:**
- COMP-05 is explicitly a sketch and an input to future work, not a deliverable that authorizes vocabulary changes. The output is a document: "cross-modality expression of [intent X], observed vocabulary tensions, and proposed directions for v14.0 Channel Projection." No code changes to `ferro-projections` or the renderer are authorized by COMP-05 alone.
- Any intent vocabulary change triggered by COMP-05 evidence is deferred to a named future milestone (v14.0 Channel Projection or a dedicated v13.x vocabulary revision). The deferred change is filed as a planning proposal, not implemented in v13.0.
- The COMP-05 sketch should cover one intent only. A single intent across three modalities is sufficient to surface vocabulary tensions without the scope risk of sketching all seven.

**Warning signs:**
- COMP-05 work includes changes to `ferro-projections/src/intent.rs` or `derive.rs`.
- The COMP-05 phase note includes implementation tasks rather than just observation and documentation.
- The team discusses which of the seven intents should be merged or split before COMP-02 and COMP-03 complete.

**Phase to address:** COMP-05 scoping. The "sketch only, no code changes" constraint must be in the phase spec. Any vocabulary revision is a separate planning proposal.

---

### Pitfall 8: VALIDATION DESIGNED TO PASS — honesty failure that makes v1.0 unachievable

**What goes wrong:**
The v1.0 criterion states "projection / intent validated through real applications and a synthetic catalog." The validation milestone is built with an implicit goal of confirming that the abstraction works. Fixtures are chosen that produce clean output; the agent harness is run on descriptions closely resembling the training content of the MCP tools; the benchmark runs on the framework's own `app/` sample application. Everything passes. The "validated" label is attached. Six months later, a real-world user builds an application that exercises a combination the validation missed, hits a silent rendering failure, and files a bug. The validation did not find a weakness because it was not trying to.

**Why it happens:**
The team that built the abstraction designs the validation for it. Confirmation bias is structural, not personal: the same intuitions that guided the design guide the choice of validation inputs. Inputs that would reveal weaknesses feel "unfair" or "out of scope." A v1.0 criterion labeled "validated" reads as a pass/fail gate, so the pressure is to pass it.

**How to avoid:**
- The validation explicitly targets weaknesses, not strengths. The design criterion is: "A weakness in any dimension is a v1.0 blocker." The validation's job is to find those weaknesses before users do.
- Each COMP item must include an adversarial fixture: a case designed to break the system (COMP-02: a fixture with 10+ fields and 5 actions; COMP-03: an NL description of a domain the agent has no prior exposure to; COMP-04: a cold-cache run on a machine without Postgres pre-installed; COMP-01: the most complex view in gestiscilo, not the simplest).
- A validation that produces zero failures is not evidence of correctness — it is evidence that the validation was not trying hard enough. If all COMP items pass on the first run without any discoveries, the milestone retrospective must explicitly address "what would we have caught if we had tried harder?"
- Frame the COMP deliverables explicitly as discovery work. The output is "what we learned about the projection/intent system's limits," not "proof that it works."

**Warning signs:**
- All COMP items pass on the first run.
- The synthetic catalog fixtures were authored by the same person who authored the renderer.
- The agent harness descriptions were drawn from the existing MCP tool documentation examples.
- The COMP retrospective contains no discovered weaknesses or deferred issues.

**Phase to address:** All COMP phases. The adversarial fixture requirement should be in every phase spec. The final milestone retrospective must include an explicit "what we found that was wrong" section; an empty section is a red flag, not a celebration.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Byte-identical snapshot tests for renderer output | Cheap to create; catches any output change | Bulk-update ritual; trains reviewers to approve changes without reviewing | Never as the primary catalog test strategy — structural invariants first |
| Agent harness with single trial per case | Cheaper runs | Cannot distinguish LLM variance from framework regression | Never for a baseline harness; acceptable for quick smoke checks |
| COMP-01 big-bang branch | Simpler to reason about | Branch divergence, conflict avalanche, invalidated test data | Never — slice-by-slice is required |
| Cold-cache benchmark skipped | Faster to run and iterate | Published numbers misrepresent first-time experience | Acceptable as internal diagnostic only; never as a published claim |
| COMP-05 sketch triggers immediate vocabulary revision | Fixes a real gap | Cascading invalidation of COMP-02 and COMP-03 data | Never in v13.0 — revision belongs to a named future milestone |
| Passing COMP with only auto-derived projections (no hand-authored or adversarial cases) | Guaranteed green | Validation cannot detect generation-vs-use gap; v1.0 criterion is not met | Never — adversarial fixtures are required |
| Publish ferro mid-COMP-01 slices when API changes occur | Unblocks gestiscilo sooner | API frozen before later slices can improve it; friction-loop cadence violated | Publish once at end of COMP-01 series |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| ferro + gestiscilo cross-repo during COMP-01 | Using path dependencies mid-migration for all slices | Path deps are acceptable for development; switch back to crates.io version at each merge-to-master; single publish at migration end |
| Agent harness + MCP tools | Calling `ferro-mcp` with a local debug binary that has unreproduced behavior | Pin the harness to a published version or a specific commit hash; document the binary version in every harness run |
| COMP-02 catalog + `checkpoint_projection` (v12.5) | Assuming `checkpoint_projection` covers the same ground as the catalog | `checkpoint_projection` verifies seams; the catalog verifies rendering intent coverage — they are complementary, not redundant |
| COMP-04 benchmark + Docker | Assuming the Docker cold-cache run matches a real user environment | A fresh Docker container still has a fast network for crates.io; a real first-time user may be on a slow connection. Document network assumptions. |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Catalog fixture count grows unbounded | CI time increases; maintainers add `#[ignore]` to manage runtime | Cap the catalog at one representative fixture per intent per complexity tier (simple / medium / complex) | When catalog exceeds ~50 fixtures |
| Agent harness runs on every CI push | CI is slow; developers disable the harness locally | Gate agent harness runs on a `[harness]` label or run nightly only — not on every PR | From the first day the harness exists |
| Benchmark added to main CI | CI time and flakiness from environment-sensitive timing | Benchmark is a manual artifact, not a CI gate. Run on a documented environment, commit the result document | From the first day the benchmark exists |

## "Looks Done But Isn't" Checklist

- [ ] **COMP-02 structural invariants exist:** At least one test per intent asserts a structural property of the rendered output (e.g., Browse emits a table element with the correct column count), not byte-identity with a snapshot.
- [ ] **COMP-02 adversarial fixture exists:** At least one fixture per intent exercises a non-trivial case (many fields, multiple actions, state machine, nested relationships).
- [ ] **COMP-03 multi-tier criteria defined:** The harness reports structural validity, intent coverage, and functional completeness separately before any agent runs are recorded.
- [ ] **COMP-03 baseline established:** A known-stable run (model version + prompt version + pass rates per tier) is committed alongside the harness code.
- [ ] **COMP-04 cold-cache run exists:** At least one benchmark result was collected in a clean Docker container with no warm Cargo cache.
- [ ] **COMP-01 slice-by-slice plan in phase spec:** Each gestiscilo view migration is listed as a separate slice with its own merge checkpoint.
- [ ] **COMP-05 "no code changes" constraint in phase spec:** The COMP-05 deliverable is explicitly a document, not a pull request against `ferro-projections`.
- [ ] **Adversarial fixture in each COMP item:** Every COMP phase spec names the adversarial input it uses to probe for weaknesses.
- [ ] **COMP retrospective has a "what we found wrong" section:** An empty section triggers a follow-up question, not a milestone close.

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Snapshot ossification | COMP-02 phase spec | Count structural-invariant tests vs. snapshot tests; invariants must outnumber snapshots |
| Catalog overfitting | COMP-02 scope review (before implementation) | Each intent has at least one non-trivial adversarial fixture |
| Agent eval gaming (compilation-only criteria) | COMP-03 harness design | Harness reports 4 separate tier pass rates; tier-1-only result is rejected |
| Non-determinism drift | COMP-03 harness design | Each case runs minimum 3 trials; baseline committed on first run |
| Vanity benchmark (warm-cache only) | COMP-04 benchmark spec | At least one cold-Docker result exists before any number is published |
| Big-bang COMP-01 migration | COMP-01 planning | Slice-by-slice plan committed before first code change; no branch open >2 weeks |
| Premature intent vocabulary revision | COMP-05 phase spec | COMP-05 deliverable contains zero changes to `ferro-projections` source |
| Validation designed to pass | All COMP phase specs + retrospective | Each phase spec names its adversarial input; retrospective has non-empty "discovered weaknesses" |

## Sources

- PROJECT.md v13.0 section: COMP-01..05 scope, v1.0 criteria, four beauty dimensions
- VISION.md: "A weakness in any dimension is a v1.0 blocker"; validation through real applications and synthetic catalog
- MEMORY.md: `feedback_friction_loop_release_cadence.md` — publish once at end; `feedback_audit_report_fix_discrepancies.md` — never silently work around
- v12.5 Phase 196 dogfood acceptance pattern: poisoned fixture requirement; "at least one real finding" criterion
- Snapshot testing research (2024-2025): brittleness of byte-identical golden files; hybrid structural+snapshot approach
- LLM agent evaluation research (2025): non-determinism requires multi-trial measurement; compilation ≠ correctness; separate tier reporting
- AI benchmark overfitting research (2025): "Are we training the model to pass the benchmark?" — direct analogy to catalog overfitting
- Agent evaluation frameworks (Braintrust, DeepEval, 2025): deterministic checks for structural validity; multi-dimensional scoring prevents gaming

---
*Pitfalls research for: v13.0 Compressive Validation — empirical validation harnesses for projection/intent*
*Researched: 2026-06-12*
