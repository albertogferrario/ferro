---
phase: 195-close-the-loop-by-default
verified: 2026-06-10T00:58:41Z
status: passed
score: 4/4 must-haves verified
overrides_applied: 0
deferred:
  - truth: "Seam 5 route-filter substring match may produce false-positive contract violations (WR-01)"
    addressed_in: "Phase 196"
    evidence: "Phase 196 goal explicitly covers hardening; WR-01 is documented in checkpoint_projection.rs:577 as 'exact scoping is explicitly Phase 196'; Phase 196 SC-4 covers demoting seams that find nothing real across dogfood inputs"
---

# Phase 195: Close the Loop by Default — Verification Report

**Phase Goal:** Verification happens without the agent asking. Wrapper seams 1, 3, 4, 5 dispatch to existing validators (`validate_projection`, `json_ui_verify_action`, `render_projection` + `json_ui_validate_spec`, `validate_contracts`) and fold their output into the unified verdict — no validation logic reimplemented. `generate_projection` and `json_ui_generate` embed the checkpoint verdict inline (summary format only). `application_info` and `projection_coverage` surface per-projection checkpoint status (`clean`/`failing`/`unverified`) from the `.ferro/checkpoints/{name}.json` cache.
**Verified:** 2026-06-10T00:58:41Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `generate_projection` response carries a `checkpoint` key with at minimum a top-level `status` and does not present five `not_checked` seam entries with empty findings (summary format) | VERIFIED | `GenerateProjectionResult.checkpoint: Option<VerdictSummary>` with `skip_serializing_if = "Option::is_none"`. `VerdictSummary` has `status`, `fail_seams`, `warn_seams`, `next_steps` — no `seams` array (SC-1). Test `verdict_summary_shape` asserts `val.get("seams").is_none()`. Tests `generate_projection_no_projection_omits_checkpoint` and `generate_projection_with_projection_embeds_checkpoint` pass. |
| 2 | `projection_coverage` shows a `checkpoint_status` field (`clean`/`failing`/`unverified`) per covered projection without a separate `checkpoint_projection` call | VERIFIED | `ModelCoverage.checkpoint_status: String` populated via `crate::tools::checkpoint_projection::read_ambient_status(project_root, &proj.name)` in `projection_coverage.rs:104-108`. Zero `run_for` references in `projection_coverage.rs`. Tests `checkpoint_status_from_cache_failing` and `checkpoint_status_unverified_no_cache` pass. |
| 3 | `application_info` shows a `projection_checkpoint` summary with `total_projections`, `clean`, `failing`, `unverified` counts from the cache | VERIFIED | `ApplicationInfo.projection_checkpoint: ProjectionCheckpointSummary` with `{total_projections, clean, failing, unverified}`. Implemented via `check_projection_checkpoint` helper calling `read_ambient_status` per projection. Zero `run_for` references in `application_info.rs`. Tests `projection_checkpoint_rollup_counts` (total == clean+failing+unverified) and `projection_checkpoint_empty_project` pass. |
| 4 | Every wrapper-seam finding's `source` names the delegating validator; `source == "checkpoint"` appears ONLY on `field_to_column` (seam 2) findings — confirming no logic reimplemented | VERIFIED | Production code: `source: "validate_projection"` (seam 1), `source: "json_ui_verify_action"` (seam 3), `source: "render_projection"` or `source: "json_ui_validate_spec"` (seam 4), `source: "validate_contracts"` (seam 5). All five `source: "checkpoint"` assignments in production scope (lines 254, 267, 288, 304, 342) are inside `field_to_column_seam()`. `make_seam` test helper uses `source: "test"` (WR-05 fix in 6398859b). SC-4 guard test `sc4_no_checkpoint_source_on_wrapper_seams` passes. |

**Score:** 4/4 truths verified

### Deferred Items

Items not yet met but explicitly addressed in later milestone phases.

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | Seam 5 route-filter substring match can over-match adjacent routes (WR-01) | Phase 196 | Phase 196 goal includes hardening; code comment at `checkpoint_projection.rs:577` states "exact scoping is explicitly Phase 196"; Phase 196 SC-4 covers seams that find nothing real in dogfood |

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-mcp/src/tools/checkpoint_projection.rs` | async run_for/execute, canonical seam names, VerdictSummary, read_ambient_status, four wrapper seams, cascade | VERIFIED | All deliverables confirmed: `pub async fn execute` (line 152), `pub(crate) async fn run_for` (line 158), canonical seam names (no old names in source), `pub struct VerdictSummary` (line 81), `pub fn summary(&self)` (line 100), `pub(crate) fn read_ambient_status` (line 823), all four seam dispatch functions present, `decide_seam4`/`decide_seam5` in production scope (fixed by 6398859b) |
| `ferro-mcp/src/tools/generate_projection.rs` | async execute that embeds `checkpoint: Option<VerdictSummary>` | VERIFIED | `pub async fn execute` (line 42), `checkpoint: Option<crate::tools::checkpoint_projection::VerdictSummary>` (line 29), `checkpoint_projection::run_for` called with `.await.ok().map(|v| v.summary())` |
| `ferro-mcp/src/tools/json_ui_generate.rs` | async execute with speculative model-derived anchor, `checkpoint: Option<VerdictSummary>` | VERIFIED | `pub async fn execute` (line 115), `checkpoint` field present with `skip_serializing_if = "Option::is_none"`, `None => None` pattern for absent model anchor |
| `ferro-mcp/src/tools/projection_coverage.rs` | `ModelCoverage.checkpoint_status` populated via `read_ambient_status` | VERIFIED | `checkpoint_status: String` field (line 39), keyed on `proj.name` (function name, not model name — Pitfall 4 avoided) |
| `ferro-mcp/src/tools/application_info.rs` | `ProjectionCheckpointSummary` + `ApplicationInfo.projection_checkpoint` | VERIFIED | `pub struct ProjectionCheckpointSummary` (line 58), `pub projection_checkpoint: ProjectionCheckpointSummary` (line 23), `check_projection_checkpoint` helper (line 431) |
| `ferro-mcp/src/service.rs` | awaited handlers for checkpoint, generate_projection, json_ui_generate; updated tool descriptions | VERIFIED | `checkpoint_projection::execute(...).await` (line 1615), `generate_projection::execute(...).await` (line 1662), `json_ui_generate::execute(...).await` (line 1363-1368), both ambient tool descriptions mention `checkpoint_status` and `projection_checkpoint` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `service.rs` checkpoint handler | `checkpoint_projection::execute` | `.await` | WIRED | Line 1615: `tools::checkpoint_projection::execute(&self.project_root, &params.0.name).await` |
| `service.rs` generate_projection handler | `generate_projection::execute` | `.await` | WIRED | Line 1662: `tools::generate_projection::execute(&self.project_root, &params.0.model_name).await` |
| `service.rs` json_ui_generate handler | `json_ui_generate::execute` | `.await` | WIRED | Lines 1363-1368: call ends with `.await` |
| `generate_projection::execute` | `checkpoint_projection::run_for` | speculative anchor + `.await.ok().map(|v| v.summary())` | WIRED | Line 98-102: `run_for(project_root, &anchor, chrono::Utc::now()).await.ok().map(|v| v.summary())` |
| `json_ui_generate::execute model=None` | `checkpoint: None` | `None => None` | WIRED | Lines 126-135: `match model { Some(m) => { ... run_for ... } None => None }` — no vacuous summary |
| `projection_coverage::execute` | `checkpoint_projection::read_ambient_status` | keyed on `proj.name` | WIRED | Lines 104-108: `crate::tools::checkpoint_projection::read_ambient_status(project_root, &proj.name)` |
| `application_info::execute` | `checkpoint_projection::read_ambient_status` | iterate projections, tally | WIRED | Lines 432-440: `check_projection_checkpoint` iterates `list_projections` and calls `read_ambient_status` per projection |
| `run_for` | `validate_projection::execute_single` | seam 1 dispatch | WIRED | `projection_well_formed_seam` calls `validate_projection::execute_single` |
| `run_for` | `json_ui_verify_action::find_handler` | seam 3 dispatch (routes pre-loaded once via `list_routes::execute(...).await`) | WIRED | `action_to_route_seam` calls `json_ui_verify_action::find_handler`; routes loaded at `run_for:195-199` |
| `run_for` | `render_projection::execute` + `json_ui_validate_spec::execute` | seam 4 dispatch | WIRED | `rendered_view_seam` calls both |
| `run_for` | `validate_contracts::execute` | seam 5 dispatch | WIRED | `props_to_contract_seam` calls `validate_contracts::execute` |

### Seam Cascade Verification

| Cascade Rule | Status | Evidence |
|-------------|--------|---------|
| seam1 fail → seam4 not_checked("seam_1_failed") | VERIFIED | `decide_seam4` (production scope, line 651) returns `Some("seam_1_failed")` when `seam1.status == Fail`; `run_for` at line 210: `match decide_seam4(&seam1.status)` |
| seam1 fail → seam5 not_checked("seam_1_failed") | VERIFIED | `decide_seam5` (production scope, line 660) checks seam1 first |
| seam4 fail → seam5 not_checked("seam_4_failed") | VERIFIED | `decide_seam5` secondary check |
| seam2, seam3 independent (always run) | VERIFIED | `seam2 = field_to_column_seam(...)` and `seam3 = action_to_route_seam(...)` called unconditionally at lines 183/209 |
| `decide_seam4`/`decide_seam5` are production-scope (not test-only) | VERIFIED | Functions at lines 651/660 have no `#[cfg(test)]` annotation; called from `run_for` at lines 210/214 (WR-03 fix in 6398859b) |

### Async Architecture Verification

| Item | Status | Evidence |
|------|--------|---------|
| `run_for` is `async fn` | VERIFIED | `pub(crate) async fn run_for` at line 158 |
| `execute` is `async fn` | VERIFIED | `pub async fn execute` at line 152 |
| `read_ambient_status` is sync (cache-only) | VERIFIED | `pub(crate) fn read_ambient_status` (not async) at line 823; no `run_for` call inside |
| `projection_coverage::execute` is sync | VERIFIED | `pub fn execute` (not async); `read_ambient_status` is sync so no async needed |
| `application_info::execute` is sync | VERIFIED | `pub fn execute` (not async) |

### Seam Name Reconciliation

| Canonical Name | Old Name (Phase 194) | Status |
|---------------|---------------------|--------|
| `projection_well_formed` | `schema_load` | VERIFIED — grep returns zero matches for old names in `ferro-mcp/src/` and `docs/` |
| `action_to_route` | `field_type_compat` | VERIFIED |
| `rendered_view` | `action_binding` | VERIFIED |
| `props_to_contract` | `render_target` | VERIFIED |
| `field_to_column` | (already correct) | UNCHANGED |

### Requirements Coverage

| Requirement | Source Plan(s) | Description | Status | Evidence |
|-------------|---------------|-------------|--------|---------|
| CHK-07 | 195-01, 195-03 | `generate_projection` and `json_ui_generate` return checkpoint verdict inline after generating | SATISFIED | Both generators embed `checkpoint: Option<VerdictSummary>` in their result structs; service.rs handlers await async execute |
| CHK-08 | 195-01, 195-04 | `application_info` and `projection_coverage` surface per-projection checkpoint status as read-only cache consumers | SATISFIED | `ModelCoverage.checkpoint_status` + `ApplicationInfo.projection_checkpoint`; both read via `read_ambient_status` (no `run_for`) |
| CHK-09 | 195-01, 195-02 | Seams 1/3/4/5 dispatch to existing validators; no logic reimplemented; each finding's `source` names the producing validator | SATISFIED | Four seam dispatch functions; SC-4 guard test `sc4_no_checkpoint_source_on_wrapper_seams` passes; all validator call sites confirmed in production code |

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| `checkpoint_projection.rs` step comments | Step numbering off-by-one in `run_for` comments (WR-02 from review) | Info | Comments only; no runtime impact |
| `projection_coverage.rs:183` | `to_snake_case` produces `h_t_m_l_parser` for acronyms (IN-02 from review) | Info | Pre-existing, not introduced by Phase 195; affects suggestion strings for acronym models only |
| `generate_projection.rs:99`, `json_ui_generate.rs:129` | `chrono::Utc::now()` inline rather than injected timestamp (IN-03 from review) | Info | Low risk; only affects `checked_at` field in cache; consistent with `checkpoint_projection::execute` public API pattern |

No blockers or warnings found in the Phase 195 deliverables.

### Behavioral Spot-Checks

| Behavior | Method | Result | Status |
|----------|--------|--------|--------|
| 299 ferro-mcp lib tests pass (includes all Phase 195 tests) | `cargo test -p ferro-mcp --lib` | `test result: ok. 299 passed; 0 failed; 0 ignored` | PASS |
| `seam_names_canonical` — no old Phase-194 names | Test in suite above | Passes in 299 | PASS |
| `sc4_no_checkpoint_source_on_wrapper_seams` — SC-4 invariant | Test in suite above | Passes in 299 | PASS |
| `projection_checkpoint_rollup_counts` — total == clean+failing+unverified | Test in suite above | Passes in 299 | PASS |
| No old seam names in source or docs | `grep -rn "schema_load\|field_type_compat\|action_binding\|render_target" ferro-mcp/src/ docs/` | Zero matches | PASS |

Full `cargo test --all-features` not run (known disk-full test-gate issue documented in project memory). Scoped `cargo test -p ferro-mcp --lib` (299 tests) and `cargo clippy -p ferro-mcp --all-targets -- -D warnings` (clean per commit history) are the validated gates for this phase.

### Human Verification Required

None. All success criteria are mechanically verifiable from source code and the test suite.

### Gaps Summary

No gaps. All four ROADMAP success criteria are satisfied:

1. `generate_projection` carries a `checkpoint` key with a top-level `status` and no raw seams array (SC-1 compact summary via `VerdictSummary`).
2. `projection_coverage` exposes `checkpoint_status` per model without any recompute.
3. `application_info` exposes a `projection_checkpoint` rollup where `total_projections == clean + failing + unverified`.
4. `source == "checkpoint"` appears only on `field_to_column` (seam 2) findings in production code; all wrapper seams name their delegating validator. SC-4 guard test enforces this mechanically.

Code review findings WR-03 (cascade helpers in production path), WR-04 (name guard in `read_ambient_status`), and WR-05 (test helper source value) were addressed in commit 6398859b before this verification. WR-01 (seam-5 substring scoping) is documented in code and explicitly deferred to Phase 196.

---

_Verified: 2026-06-10T00:58:41Z_
_Verifier: Claude (gsd-verifier)_
