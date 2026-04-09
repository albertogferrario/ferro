---
phase: 129-publish-workflow-refinement
verified: 2026-04-09T00:00:00Z
status: passed
score: 6/6 must-haves verified
---

# Phase 129: Publish Workflow Refinement — Verification Report

**Phase Goal:** Stop releasing every workspace member on docs-only or CI-only commits. Gate the auto-patch-bump on whether any library crate actually changed (REPORT §8). Document `ferro_version` single-global-field behavior and add a per-crate override hook (`ferro_versions`) that is parsed and round-tripped but not wired into rewrite logic (REPORT §14).

**Verified:** 2026-04-09
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Docs-only / CI-only push will NOT trigger bump (`should_publish=none`, downstream jobs skip) | ✓ VERIFIED | `publish.yml` lines 63-72: `skip` step emits `should_publish=none` when `lib_changed == '0'`; downstream `test` and `publish` jobs condition on `== 'bump' \|\| == 'yes'` (lines 135, 165) |
| 2 | Library-crate change still triggers bump (unchanged `bump` / `yes` path) | ✓ VERIFIED | `publish.yml` lines 74-98: `check` step runs only when `lib_changed == '1'`, emits `should_publish=bump` or `should_publish=yes` per existing tag logic |
| 3 | First-run (no tag) still publishes | ✓ VERIFIED | `publish.yml` lines 39-42: `[ -z "$LAST_TAG" ]` branch sets `LIB_CHANGED=1`, bypassing exclusion list |
| 4 | `ferro_version` single-global-field behavior documented in PUBLISHING.md | ✓ VERIFIED | `PUBLISHING.md` lines 151-168: `## Version Model` section explains lockstep release, single `ferro_version` field, and rewrite behavior |
| 5 | `ferro_versions` override hook: exists in schema, is parsed, is round-tripped, NOT wired into rewrite logic | ✓ VERIFIED | `project.rs` lines 36, 117-131: field declared `Option<BTreeMap<String, String>>`, parser block reads it. `rewrite_ferro_version.rs` test `preserves_ferro_versions_override_roundtrip` (line 397) verifies byte-identity survival. Rewrite functions `rewrite_cargo_docker_toml` / `compute_cargo_docker_toml` contain zero references to `ferro_versions`. |
| 6 | No new MCP changes, no new CLI flags, no new doctor checks, no rewrite logic changes | ✓ VERIFIED | `ferro_versions` absent from `ferro-mcp/`, `ferro-cli/src/commands/`, and `ferro-cli/src/doctor/`. Rewrite function signatures unchanged. |

**Score:** 6/6 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `.github/workflows/publish.yml` | Gate step with `should_publish=none`, exclusion list, `git diff --name-only`, downstream `if:` using `'bump' \|\| 'yes'` | ✓ VERIFIED | All four elements present at lines 36-98 (gate), 63-72 (none path), 74-98 (bump/yes path), 103/135/165 (downstream conditions) |
| `ferro-cli/src/project.rs` | `FerroDeployMetadata.ferro_versions: Option<BTreeMap<String, String>>`, parser block, TODO comment referencing Phase 129 / REPORT §14, two new tests | ✓ VERIFIED | Field at line 36, parser at lines 117-131, TODO comment at lines 28-35, tests `parses_ferro_versions_override` (line 525) and `rejects_ferro_versions_wrong_type` (line 555) |
| `ferro-cli/src/deploy/rewrite_ferro_version.rs` | `preserves_ferro_versions_override_roundtrip` test | ✓ VERIFIED | Test at line 397 with Phase 129 / REPORT §14 comment at lines 390-395 |
| `PUBLISHING.md` | `## Version Model` and `## Publish Gating` sections, exclusion list verbatim, `should_publish=none`/`should_publish=yes` documented, `ferro_versions` example | ✓ VERIFIED | `## Publish Gating` at line 27 with exclusion list lines 36-48 and scenarios table lines 71-80; `## Version Model` at line 151 with per-crate override reservation at lines 170-190 |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `publish.yml` gate step | downstream `test` job | `should_publish == 'bump' \|\| == 'yes'` | ✓ WIRED | Line 135: `if: always() && (...should_publish == 'bump' || ...should_publish == 'yes')` |
| `publish.yml` gate step | downstream `publish` job | same condition | ✓ WIRED | Line 165: same pattern |
| `publish.yml` skip step | `should_publish=none` output | `echo "should_publish=none" >> $GITHUB_OUTPUT` | ✓ WIRED | Line 72 |
| `project.rs` parser | `ferro_versions` field | `table.get("ferro_versions")` → `BTreeMap` | ✓ WIRED | Lines 117-131; roundtrip test in `rewrite_ferro_version.rs` confirms survival |
| `ferro_versions` field | rewrite logic | intentionally NOT wired | ✓ VERIFIED ABSENT | Neither `rewrite_cargo_docker_toml` nor `compute_cargo_docker_toml` reference `ferro_versions` |

### Data-Flow Trace (Level 4)

Not applicable — this phase delivers CI workflow logic and a schema reservation, not user-facing rendering components.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `ferro_versions` parser round-trips correctly | `cargo test -p ferro-cli --lib project::tests::parses_ferro_versions_override` | Reported: 19 tests passed (per regression gate in prompt) | ✓ PASS |
| Rewriter preserves `ferro_versions` table | `cargo test -p ferro-cli --lib deploy::` | Reported: 37 tests passed (per regression gate in prompt) | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| REPORT §8 | 129-01 | Gate publish on library-crate changes | ✓ SATISFIED | `publish.yml` gate step with exclusion list; `should_publish=none` for non-library paths |
| REPORT §14 | 129-02, 129-03 | Document `ferro_version` as global; add `ferro_versions` override hook | ✓ SATISFIED | `project.rs` schema + parser; `PUBLISHING.md` Version Model + per-crate override reservation section |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| — | — | None detected | — | — |

No TODOs flagged as blockers. The `TODO(Phase 129 / REPORT §14)` comment in `project.rs` lines 28-35 is a deliberate deferral note, not a stub — the field is fully implemented (parsed, tested, round-tripped); the TODO documents that rewrite-logic integration is intentionally deferred until a real desync occurs.

### Human Verification Required

None. All phase deliverables are static artifacts (YAML workflow, Rust source, Markdown documentation) that are fully verifiable through code inspection.

### Gaps Summary

No gaps. All six observable truths are verified. The gate is implemented correctly in `publish.yml` with a complete exclusion list matching the documented list in `PUBLISHING.md`. The `ferro_versions` schema reservation is present in `project.rs`, parsed, covered by two tests in `project.rs` and one roundtrip test in `rewrite_ferro_version.rs`, and explicitly not wired into the rewrite pipeline. Documentation in `PUBLISHING.md` covers both the `## Publish Gating` and `## Version Model` sections with the required content.

---

_Verified: 2026-04-09_
_Verifier: Claude (gsd-verifier)_
