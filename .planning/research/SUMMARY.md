# Project Research Summary

**Project:** ferro / v13.0 Compressive Validation (COMP-01..05)
**Domain:** Empirical validation harnesses for a projection/intent framework
**Researched:** 2026-06-12
**Confidence:** HIGH

## Executive Summary

v13.0 is a validation milestone, not a feature milestone. Its target is v1.0 criterion #2 — "projection / intent validated through real applications and a synthetic catalog of canonical app classes" — and it addresses the compressive dimension first (substance-first priority order). Five discrete COMP items span: a real-app migration (COMP-01, gestiscilo), a synthetic regression catalog (COMP-02), an agent-success-rate harness (COMP-03), a time-to-working-app benchmark (COMP-04), and a cross-modality vocabulary sketch (COMP-05). The research files converge strongly on build order, tooling, placement, and integrity guardrails.

The recommended approach is incremental and additive: no new published crates, no structural changes to the projection/intent vocabulary, no cross-crate refactors. COMP-02 (regression catalog) builds first because it establishes the ground-truth intent-per-class that COMP-03 consumes, uses existing test infrastructure (`ferro-projections/tests/`), and produces durable CI coverage immediately. COMP-05 (cross-modality sketch) is unblocked now — it is a document, not code — and should be done early to surface vocabulary gaps before v14.0 planning. COMP-01 (gestiscilo migration) is the highest-effort item and must be sliced one entity at a time; Slice A covers three entities (Browse, Process, Summarize) and is sufficient for meaningful real-world validation. COMP-03 and COMP-04 follow once the catalog ground-truth is in place.

The main risk is vanity validation: harnesses designed to pass rather than to probe limits. Four structural guardrails from the research must carry into every phase spec: (1) structural-invariant assertions over byte-identical snapshots; (2) multi-tier pass criteria for COMP-03 stated before any agent run, with minimum 3 trials per case; (3) a cold-cache benchmark run for COMP-04; (4) at least one adversarial fixture per COMP item and a mandatory "discovered weaknesses" retrospective section. A validation that produces zero failures is evidence that it was not trying hard enough, not that the abstraction is correct.

## Key Findings

### Recommended Stack

All tooling needs are minimal and additive. The existing workspace infrastructure handles almost everything; three dev-dependency additions cover the gaps. `insta 1.48` (with `json` and `redactions` features) provides snapshot testing for COMP-02 with an interactive `cargo insta review` workflow. `criterion 0.8.2` (`default-features = false, features = ["cargo_bench_support"]`) provides statistical wall-clock benchmarking for COMP-04 via `iter_custom` subprocess timing; `hyperfine` is rejected (external binary, violates no-external-tooling constraint). COMP-03 needs no new crate: `rmcp 0.12` already compiles `transport-async-rw` as a default feature; in-process `tokio::io::duplex` transport is the correct pattern, proven in `ferro-api-mcp/tests/e2e.rs`. `proptest 1` is already in the workspace and adds property-based invariant tests for COMP-02.

**Core technologies:**
- `insta 1.48` (dev-dep, `ferro-projections` + `ferro-json-ui`): snapshot/golden testing for `Vec<IntentScore>` and rendered `Spec` — `assert_json_snapshot!` over hand-rolled `assert_eq!` literals at catalog scale
- `criterion 0.8.2` (dev-dep, `ferro-cli`, `default-features = false`): wall-clock benchmark via `iter_custom`; statistical noise thresholds; baseline comparison (`--save-baseline` / `--load-baseline`)
- `rmcp 0.12` `transport-async-rw` + `client` features (add to `ferro-mcp` dev-deps only): in-process MCP client for COMP-03 via `tokio::io::duplex` — no subprocess, no port, no version bump
- `proptest 1` (already in workspace; add to `ferro-projections` dev-deps): property invariants for `derive_intents` across generated `ServiceDef` inputs

**Critical constraints:** criterion 0.8.2 requires Rust >= 1.88 (matches workspace MSRV). No rmcp upgrade — 0.12 is pinned across all three consuming crates.

### Expected Features

The five COMP items map to distinct validation audiences. COMP-02 and COMP-01 Slice A are required to meet the v1.0 criterion at all; COMP-03 and COMP-04 add measurement depth for a credible public claim; COMP-05 is required before v14.0 Channel Projection scope can be finalized.

**Must have (table stakes):**
- COMP-02 regression harness: one test per intent (7), each asserting primary intent + at least one key signal, with competing signals present — permanent machine-checkable baseline in `ferro-projections/tests/`
- COMP-05 cross-modality sketch: all 7 intents mapped to mobile/voice/CLI, at least one vocabulary gap identified, v14.0 implications stated — document only, zero changes to `ferro-projections` source
- COMP-01 Slice A: 3 gestiscilo entities across Browse + Process + Summarize, before/after render equivalence, at least one documented abstraction finding

**Should have (required for credible v1.0 claim):**
- COMP-03 agent success rate: 14+ tasks (2 per intent), 4-tier pass criteria defined before any run, minimum 3 trials per case, committed baseline artifact
- COMP-04 time-to-working-app benchmark: agent-assisted, cold-cache Docker run, fully specified start/end conditions, committed result document

**Defer (post-v13.0):**
- COMP-01 Slice B (Collect + Analyze, 2 more entities) — after Slice A confirms the pattern
- COMP-03 re-run after significant ferro-mcp tool description changes
- Extending COMP-02 corpus with Slice B migration fixtures

### Architecture Approach

No new crates. All five COMP items land as new files in existing crates. `ferro-projections` remains renderer-free; no `ferro-*` crate embeds app identity; CI disk constraints are respected by gating COMP-04's `cargo build` behind `FERRO_BENCH=1`. COMP-03 drives `ferro-mcp` (developer introspection), not `ferro-mcp-server` (consumer application MCP endpoint) — this distinction is architectural and must be preserved.

**Major components:**
1. `ferro-projections/tests/catalog.rs` — COMP-02: 7 canonical `ServiceDef` builders, structural invariant assertions + `insta` snapshots, `proptest` invariants; runs in `cargo test --all-features`
2. `ferro-mcp/tests/agent_harness.rs` — COMP-03: in-process MCP server via `tokio::io::duplex`, 14-task corpus, 4-tier per-task verdict, 3 trials per case, `#[ignore]` in default CI
3. `ferro-cli/benches/time_to_working_app.rs` — COMP-04: criterion `iter_custom` scaffold timing, gated `FERRO_BENCH=1` to protect CI disk
4. `ferro-projections/src/render/{cli,voice,mobile}.rs` — COMP-05: `pub(crate)` research renderers implementing `Renderer` trait; no public API, no production callers
5. `gestiscilo-it/app/src/projections/` (external) — COMP-01: `ServiceDef` builders replacing `JsonUi::render_file`, sliced one entity per merge to gestiscilo master

### Critical Pitfalls

1. **Snapshot ossification** — byte-identical golden files break on every legitimate renderer change. Use structural-invariant assertions (`scores[0].intent == Browse`, rendered HTML contains a table element) as the primary catalog tests; reserve `insta` snapshots only for a small set of named canonical intent shapes that require a deliberate decision to update.

2. **Agent eval gaming** — a compilation-only success criterion will be satisfied by an empty `ServiceDef` and report inflated pass rates. Define 4 tiers (structural validity / intent coverage / functional completeness / checkpoint verdict) before any agent run; report each tier separately; never aggregate into a single boolean passed/failed.

3. **Non-determinism drift** — a single-trial harness cannot distinguish LLM variance from framework regression. Run each COMP-03 task minimum 3 trials, use temperature=0 for tier-1 checks, commit model version and prompt version alongside every run, set a regression threshold (>10 percentage points drop from baseline).

4. **COMP-01 big-bang migration** — a multi-week cross-repo branch diverges catastrophically against an active-development framework. Slice one entity at a time, merge each slice to gestiscilo master before starting the next, publish a single ferro version at the end of the series.

5. **Validation designed to pass** — confirmation bias is structural when the builders design the validation. Every phase spec must name an adversarial fixture. The milestone retrospective must have a non-empty "discovered weaknesses" section; an empty section is a red flag, not a celebration.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: COMP-02 — Synthetic Regression Catalog
**Rationale:** The catalog is the dependency foundation for COMP-03 and the regression baseline for all future `derive_intents()` changes. Requires only existing infrastructure, no external dependencies, produces permanent CI value immediately.
**Delivers:** `ferro-projections/tests/catalog.rs` (7 canonical builders, 7+ intent tests, structural invariant assertions, `proptest` invariants); `ferro-json-ui/tests/catalog_render.rs` (rendered `Spec` structure assertions); all tests in `cargo test --all-features`.
**Addresses:** COMP-02 "good" criteria — all 7 intents covered, primary intent + key signal asserted per fixture, competing signals present, adversarial fixture per intent.
**Avoids:** Snapshot ossification (structural invariants primary, `insta` snapshots only for canonical shapes); catalog overfitting (adversarial fixture per intent mandatory before implementation begins).
**Research flag:** Standard patterns — `insta` well-documented, `proptest` already in workspace, `derive_intents()` internals directly readable.

### Phase 2: COMP-05 — Cross-Modality Vocabulary Sketch
**Rationale:** Unblocked immediately — no code dependencies, no implementation prerequisites. Produces vocabulary gap analysis v14.0 needs. No changes to `ferro-projections` source are authorized.
**Delivers:** Three `pub(crate)` sketch renderers in `ferro-projections/src/render/`; written analysis covering all 7 intents across 3 non-visual modalities; at least one identified vocabulary gap with v14.0 implications; zero changes to `intent.rs` or `derive.rs`.
**Addresses:** COMP-05 "good" criteria — all 7 intents appear, at least one gap identified.
**Avoids:** Premature intent vocabulary revision (phase spec must explicitly state the "no code changes to ferro-projections source" constraint; any vocabulary change is filed as a v14.0 proposal).
**Research flag:** Standard patterns — `Renderer` trait is modality-agnostic by construction; `pub(crate)` sketch modules require no new API decisions.

### Phase 3: COMP-01 Slice A — Gestiscilo Migration (Browse + Process + Summarize)
**Rationale:** The only source of real-world validation. Slice A (3 entities, 3 structurally distinct intents) is sufficient to surface abstraction gaps the synthetic corpus cannot. Cross-repo effort requires careful slicing — one entity per merge, no open branch longer than 2 weeks.
**Delivers:** 3 `ServiceDef` builders in `gestiscilo-it/app/src/projections/`, corresponding controller changes replacing `JsonUi::render_file`, before/after render equivalence documentation, at least one documented abstraction finding.
**Addresses:** COMP-01 Slice A "good" criteria — 3 entities × 3 intents, render equivalence verified, one finding documented, gestiscilo test suite green.
**Avoids:** Big-bang migration (slice-by-slice plan committed before first code change); mid-series ferro publish (single publish at COMP-01 series end).
**Research flag:** Needs phase-time planning to select the 3 entities from gestiscilo's 69 models, verify current render output for before/after comparison, confirm ferro version pinned in gestiscilo.

### Phase 4: COMP-03 — Agent-Success-Rate Harness
**Rationale:** Depends on COMP-02 catalog for domain descriptions and ground-truth intent per class. Harness design (4-tier criteria, 3 trials, baseline commit) is the substantive deliverable; agent runs are fast once the harness exists.
**Delivers:** `ferro-mcp/tests/agent_harness.rs` with 14+ tasks (2 per intent), 4-tier per-task reporting, 3 trials per case, `rmcp 0.12` in-process transport, committed baseline artifact (model version, prompt version, pass rates per tier). All tests `#[ignore]` in default CI.
**Addresses:** COMP-03 "good" criteria — 4-tier criteria defined before any run, diverse task corpus, ferro-mcp active as context source.
**Avoids:** Agent eval gaming (4-tier criterion mandatory before harness implementation); non-determinism drift (3-trial minimum in harness design, not retrofit).
**Research flag:** Calibration open question — success-rate floor (e.g., `assert!(rate >= 0.7)`) must be set at phase time from a first baseline run, not now.

### Phase 5: COMP-04 — Time-to-Working-App Benchmark
**Rationale:** Independent of catalog; benefits from COMP-03 having exercised the agent-assisted path. The benchmark is a manual artifact first, CI gate second (if at all). Primary deliverable is a committed result document with full apparatus specification.
**Delivers:** `ferro-cli/benches/time_to_working_app.rs` with criterion `iter_custom` scaffold timing, `FERRO_BENCH=1` gate; at least one cold-cache Docker run with fully specified environment; committed Markdown result document with start/end conditions, agent-assisted wall-clock time, per-step breakdown.
**Addresses:** COMP-04 "good" criteria — cold-cache run exists, start/end conditions precisely documented, agent-assisted, result committed.
**Avoids:** Vanity benchmark (cold-cache run is mandatory; warm-cache result is internal diagnostic only).
**Research flag:** Open question — wall-clock CI assertion threshold must be set after a first cold-cache run to measure variance. Decide at phase time whether to assert in CI or keep as manual-only artifact.

### Phase Ordering Rationale

- COMP-02 first: dependency foundation for COMP-03, immediate CI value, no external dependencies, no production risk
- COMP-05 second: unblocked immediately, produces design artifacts v14.0 needs, single-session scope
- COMP-01 third: highest-value signal but highest-effort; COMP-02 establishes the intent baseline gestiscilo migrations compare against
- COMP-03 fourth: domain descriptions and ground-truth sourced from COMP-02; harness design is the work, agent runs are fast
- COMP-04 last: independent, environment-sensitive, agent-assisted workflow already exercised in COMP-03

### Research Flags

Phases needing deeper research during planning:
- **Phase 3 (COMP-01 Slice A):** Entity selection from gestiscilo's 69 models requires reading `src/models/` and `src/controllers/` at phase time. Confirm current ferro version pinned in gestiscilo. Verify render equivalence testing method.
- **Phase 4 (COMP-03):** Success-rate floor threshold and the 14 specific NL task descriptions must be specified at phase time after reviewing COMP-02 fixtures. Threshold is empirical.
- **Phase 5 (COMP-04):** Wall-clock CI assertion threshold (if any) must be set after a first cold-cache run.

Phases with standard patterns (skip research-phase):
- **Phase 1 (COMP-02):** `insta` and `proptest` well-documented and already understood; `derive_intents()` internals directly readable.
- **Phase 2 (COMP-05):** `Renderer` trait is stable; `pub(crate)` sketch modules require no new API decisions.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All versions verified via docs.rs; rmcp transport pattern proven in existing `ferro-api-mcp/tests/e2e.rs`; workspace MSRV compatibility checked |
| Features | HIGH | All COMP-01..05 requirements read directly from PROJECT.md and existing source; 9 projection fixtures confirmed; gestiscilo model/view counts verified |
| Architecture | HIGH | All placement decisions grounded in direct source inspection of `ferro-projections/Cargo.toml`, `ferro-mcp/Cargo.toml`, `ferro-cli/Cargo.toml`, and existing test patterns |
| Pitfalls | HIGH | Grounded in codebase patterns (friction-loop cadence, v12.5 dogfood acceptance), LLM evaluation research, snapshot testing brittleness research |

**Overall confidence:** HIGH

### Gaps to Address

- **COMP-03 success-rate floor:** Empirical value; can only be set after a first baseline run. Handle at phase time: run harness once, observe distribution, set floor to flag genuine regression without being fragile to LLM variance.
- **COMP-04 wall-clock threshold:** Whether to assert a timing threshold in CI depends on CI disk and time budget. Handle at phase time: run cold-cache benchmark once, measure variance across 3 runs, decide whether a CI assertion is feasible.
- **COMP-01 entity selection:** The specific 3 gestiscilo entities for Slice A are not pre-determined. Handle at phase time: read gestiscilo's `src/models/` and `src/controllers/` to identify the most representative Browse, Process, and Summarize candidates with direct `JsonUi::render_file` calls.

## Sources

### Primary (HIGH confidence)
- `ferro-projections/src/{intent,service,derive}.rs` — 7-intent vocabulary, `derive_intents()`, `ServiceDef` builder API
- `app/src/projections/` — 9 existing projection fixtures (direct read)
- `ferro-api-mcp/tests/e2e.rs` — proven `rmcp 0.12` in-process client pattern
- `ferro-mcp/src/tools/checkpoint_projection.rs` — established test fixture patterns
- `ferro-projections/tests/generate_schemas.rs` — existing integration test placement pattern
- `docs.rs/insta/latest/insta/` — version 1.48.0, feature list verified
- `docs.rs/criterion/latest/criterion/` — version 0.8.2 verified
- `docs.rs/rmcp/0.12.0/features` — `transport-async-rw` confirmed as default feature in 0.12
- `.planning/PROJECT.md` — v13.0 COMP-01..05 requirements, v1.0 criteria, four beauty dimensions
- `./CLAUDE.md` — rendering architecture invariants, project-agnostic crate rule

### Secondary (MEDIUM confidence)
- Snapshot testing research (2024-2025) — brittleness of byte-identical golden files; hybrid structural+snapshot approach
- LLM agent evaluation research (2025) — non-determinism requires multi-trial measurement; compilation != correctness; separate tier reporting
- AI benchmark overfitting research (2025) — catalog overfitting analogy
- Agent evaluation frameworks (Braintrust, DeepEval, 2025) — deterministic checks for structural validity; multi-dimensional scoring

---
*Research completed: 2026-06-12*
*Ready for roadmap: yes*
