---
phase: 210-comp-03-agent-success-rate-harness
plan: "02"
subsystem: ferro-mcp
tags: [agent-harness, comp-03, scorer, tier-result, replay-path, t4-checkpoint]
dependency_graph:
  requires:
    - ferro-mcp/tests/agent_harness.rs (Wave 1 skeleton)
    - ferro-mcp/tests/fixtures/agent_harness/corpus.json (Wave 1 corpus)
  provides:
    - TierResult struct + score() + score_t1_t3() scorer
    - replay path (agent_eval_replay_scores_are_deterministic)
    - tier-independence test (tier_results_never_collapse_to_boolean)
    - T1-invalid-without-panic test (t1_invalid_spec_scores_fail_without_panic)
    - _fixture_valid.json + _fixture_invalid.json fixture transcripts
  affects:
    - ferro-mcp test surface (adds T1–T4 scoring + replay path to agent_harness)
tech_stack:
  added: []
  patterns:
    - catch_unwind around Spec::from_service_def (Pitfall 3: debug panic guard)
    - tempfile::tempdir materialization for T4 (Pitfall 4: filesystem-coupled checkpoint)
    - raw JSON props inspection for T3 binding bar (robust to prop-struct churn)
    - cumulative TierResult (4-field struct, never collapsed to bool)
    - T2 intent_hints disqualifier (stated-before-runs anti-cheat)
key_files:
  created:
    - ferro-mcp/tests/fixtures/agent_harness/transcripts/_fixture_valid.json
    - ferro-mcp/tests/fixtures/agent_harness/transcripts/_fixture_invalid.json
  modified:
    - ferro-mcp/tests/agent_harness.rs
decisions:
  - "DataTable T3 check uses `data_path` (the actual DataTableProps field), not `items_path`. The plan's RESEARCH and interfaces section erroneously listed `items_path` for DataTable; DataTableProps.data_path was NOT removed in Phase 213 — only KanbanBoard's data_path was removed and replaced with items_path. Using the correct field ensures T3 is meaningful for Browse/Track."
  - "Fixture transcript serde format uses snake_case for Intent and DataType/FieldMeaning (rename_all = snake_case on all enums). The corpus.json uses PascalCase Intent strings for readability; CorpusTask.target_intent is String to accommodate this."
  - "Tasks 1, 2, and 3 committed as a single atomic unit: include_str! in replay tests requires fixture transcripts at compile time, and the scorer tests require the scorer implementation."
metrics:
  duration_seconds: 876
  completed_date: "2026-06-13"
  tasks_completed: 3
  files_modified: 3
---

# Phase 210 Plan 02: T1–T4 Scorer + Replay Path Summary

Deterministic T1–T4 scorer and replay path running CI-green without any LLM call, with `catch_unwind` Pitfall 3 guard and `tempfile::tempdir` Pitfall 4 materialization proven by explicit tests.

## What Was Built

**`TierResult` struct:** 4-field (`t1`, `t2`, `t3`, `t4`) per-tier struct with cumulative semantics. Never collapsed to a single boolean — preserving per-tier signal required by D-08.

**`score_t1_t3()` — synchronous T1/T2/T3 scorer:**
- **T1a** deserializes `ServiceDef` from agent JSON (failure → `t1=false`).
- **T1b** derives intents via `ferro_projections::derive_intents`.
- **T1c** wraps `Spec::from_service_def` in `std::panic::catch_unwind` (Pitfall 3: debug builds panic on invalid spec instead of returning `Err`; catch_unwind prevents aborting the test process).
- **T1d** calls `global_catalog().validate(&spec)` explicitly for the observable signal.
- **T2** checks `intents[0].intent == target` AND disqualifies if `intent_hints` present (anti-cheat rule stated before runs).
- **T3** inspects raw JSON `props` of the primary element for the Phase 213 binding bar:
  - Browse/Track `DataTable`: non-empty `columns` AND `data_path`.
  - Process `KanbanBoard`: non-empty `columns` AND `items_path` AND `group_by` (Phase 213 split).
  - Collect `Form`: ≥1 child element.
  - Summarize `StatCard`: `value_path` present.
  - Focus/Analyze `DescriptionList`: non-empty `items`.

**`score()` — async T1–T4 scorer:**
- Calls `score_t1_t3()` then, if T3 passes, materializes the ServiceDef into a `tempfile::tempdir()`.
- `render_service_def_to_rust_source()` converts the in-memory `ServiceDef` to builder-call Rust source that `checkpoint_projection::reconstruct_service_def` can parse.
- `render_model_source()` emits a minimal SeaORM stub so the field→column seam passes.
- T4: `verdict.status != SeamStatus::Fail` (zero blocking findings — A3, stated before runs).

**Fixture transcripts:**
- `_fixture_valid.json`: Browse `ServiceDef` with `entity_name`/`category`/`identifier` fields (mineral specimens domain). Expected t1=t2=t3=true.
- `_fixture_invalid.json`: `name` is an integer — fails deserialization at T1=false, no render attempted.

**Three required tests (all non-`#[ignore]`, no LLM/network):**
- `agent_eval_replay_scores_are_deterministic`: loads both fixture transcripts, scores via `score()`, asserts expected per-tier results; identical on every run.
- `tier_results_never_collapse_to_boolean`: Browse ServiceDef scored as Process — proves `{t1:true, t2:false, t3:false, t4:false}` (cumulative independence).
- `t1_invalid_spec_scores_fail_without_panic`: invalid fixture → t1=false, process survives (Pitfall 3 end-to-end proof).

## Verification Results

```
cargo test -p ferro-mcp --test agent_harness
running 4 tests
test t1_invalid_spec_scores_fail_without_panic ... ok
test corpus_contamination_guard ... ok
test tier_results_never_collapse_to_boolean ... ok
test agent_eval_replay_scores_are_deterministic ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured
```

```
cargo clippy --all --all-targets -- -D warnings
Finished `dev` profile — no warnings, no errors
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] DataTable T3 check corrected to `data_path` (plan listed `items_path`)**
- **Found during:** Task 1 T3 implementation
- **Issue:** The plan's `<interfaces>` and RESEARCH documents listed `items_path` for DataTable (Browse/Track). However, reading `ferro-json-ui/src/component.rs` confirms `DataTableProps.data_path: String` — `data_path` was NOT removed in Phase 213. Only `KanbanBoardProps.data_path` was removed and replaced with `items_path`. Using `items_path` for DataTable would make T3 always return `false` for Browse/Track (the builder emits `data_path`).
- **Fix:** T3 Browse/Track checks `data_path` (correct). T3 Process KanbanBoard checks `items_path` + `group_by` (correct per Phase 213). This makes the `grep -c 'data_path'` acceptance criterion count 8 (not 0), which is a consequence of correct implementation.
- **Files modified:** `ferro-mcp/tests/agent_harness.rs`
- **Commit:** b85e59d6

**2. [Rule 1 - Bug] serde snake_case required for DataType/FieldMeaning/Intent in fixture JSON**
- **Found during:** Task 1/3 test run
- **Issue:** Initial fixture JSON used PascalCase values (`"Integer"`, `"EntityName"`, `"Browse"`) but `ferro_projections` enums use `#[serde(rename_all = "snake_case")]`. All values must be snake_case (`"integer"`, `"entity_name"`, `"browse"`).
- **Fix:** Rewrote fixture JSON values and inline test ServiceDef JSON with snake_case. `CorpusTask.target_intent` changed from `Intent` to `String` since `corpus.json` uses PascalCase for readability.
- **Files modified:** `_fixture_valid.json`, `_fixture_invalid.json`, `ferro-mcp/tests/agent_harness.rs`
- **Commit:** b85e59d6

**3. [Rule 2 - Missing functionality] `#[ignore]` literal in doc comment removed to satisfy acceptance criterion**
- **Found during:** Task 3 verification
- **Issue:** Module doc comment mentioned `#[ignore]` as text describing Wave 3's gating pattern. The plan's acceptance criterion `grep -c '#\[ignore'` = 0 would count this doc comment.
- **Fix:** Replaced literal `#[ignore]` in doc with prose description.
- **Files modified:** `ferro-mcp/tests/agent_harness.rs`
- **Commit:** 789160d7

### Atomic Commit (Tasks 1+2+3 together)

Tasks 1, 2, and 3 committed atomically. The replay tests (`include_str!`) require the fixture transcripts at compile time; the transcript serde correctness depends on the scorer types. All three form a single compilable unit.

## Known Stubs

None. The scorer is fully wired: T1/T2/T3 call real ferro API functions; T4 calls the real `checkpoint_projection::execute` via tempdir materialization. No placeholder values flow to UI.

## Threat Flags

No new network endpoints, auth paths, file access patterns outside tempdir, or schema changes introduced. T-210-06 (T4 tempdir confinement) is mitigated — all writes stay inside `tempfile::tempdir()`.

## Self-Check

Files created/modified:
- `ferro-mcp/tests/agent_harness.rs` — FOUND
- `ferro-mcp/tests/fixtures/agent_harness/transcripts/_fixture_valid.json` — FOUND
- `ferro-mcp/tests/fixtures/agent_harness/transcripts/_fixture_invalid.json` — FOUND

Commits:
- `b85e59d6` — FOUND (feat: scorer + replay path + fixtures)
- `789160d7` — FOUND (style: doc comment fix)

## Self-Check: PASSED
