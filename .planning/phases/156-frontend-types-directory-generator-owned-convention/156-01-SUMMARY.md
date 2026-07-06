---
phase: 156-frontend-types-directory-generator-owned-convention
plan: 01
subsystem: cli
tags: [typescript, gitignore, generate-types, scaffolder, convention]

# Dependency graph
requires: []
provides:
  - app/frontend/src/types/ untracked from git index (files retained on disk)
  - gitignore.tpl load-bearing comment naming generator-owned convention and doc cross-reference
  - generate_types.rs header comment corrected to point to frontend/src/lib/types/
affects:
  - 156-02
  - 156-03
  - 156-04
  - 156-05
  - 156-06

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "gitignore.tpl load-bearing comment pattern: explicit annotation that a rule is convention-enforcing with doc cross-reference"

key-files:
  created: []
  modified:
    - app/frontend/src/types/inertia-props.ts (removed from git index)
    - app/frontend/src/types/routes.ts (removed from git index)
    - ferro-cli/src/templates/files/root/gitignore.tpl
    - ferro-cli/src/commands/generate_types.rs

key-decisions:
  - "D-05: git rm --cached only — files remain on disk; dev server continues working"
  - "D-18 fix also required updating test_generated_output_includes_header to assert frontend/src/lib/types/ (Rule 1 — test was asserting old/wrong path)"

patterns-established:
  - "Load-bearing gitignore comments: annotate convention-enforcing rules with a note naming the convention and linking to documentation"

requirements-completed: [D-05, D-06, D-18]

# Metrics
duration: 12min
completed: 2026-05-14
---

# Phase 156 Plan 01: frontend/src/types Generator-Owned Convention — Initial Surface Fixes Summary

**Reference app generated TS files untracked from git, gitignore.tpl annotated as load-bearing, and generate_types.rs header corrected to point to frontend/src/lib/types/**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-05-14T01:24:00Z
- **Completed:** 2026-05-14T01:36:27Z
- **Tasks:** 3
- **Files modified:** 4 (2 removed from index, 2 modified)

## Accomplishments

- Untracked `app/frontend/src/types/inertia-props.ts` and `routes.ts` from git index while preserving files on disk — `git ls-files app/frontend/src/types/` now returns empty
- Strengthened `gitignore.tpl` comment from bare `# generated_types` to a load-bearing annotation naming the convention and cross-referencing `docs/src/cli/frontend-types.md`
- Fixed `generate_types.rs` header comment at lines 710-711: `frontend/src/types/` → `frontend/src/lib/types/` so the generator stops misdirecting users to the generator-owned directory

## Task Commits

All tasks committed atomically in a single commit (all three changes are small, independent, and directly serve the same goal):

1. **Task 1: Untrack generated TS files** - `63f6e8bc` (chore)
2. **Task 2: Strengthen gitignore.tpl comment** - `63f6e8bc` (chore)
3. **Task 3: Fix generate_types.rs header comment** - `63f6e8bc` (chore)

## Files Created/Modified

- `app/frontend/src/types/inertia-props.ts` — removed from git index (file retained on disk)
- `app/frontend/src/types/routes.ts` — removed from git index (file retained on disk)
- `ferro-cli/src/templates/files/root/gitignore.tpl` — comment on line 14 expanded to load-bearing annotation with doc cross-reference
- `ferro-cli/src/commands/generate_types.rs` — line 711 path fixed from `frontend/src/types/` to `frontend/src/lib/types/`; line 1995 test assertion updated to match

## Decisions Made

- Followed plan as specified for all three tasks.
- The D-18 fix required a companion test update (see Deviations below).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated test_generated_output_includes_header to assert corrected path**
- **Found during:** Task 3 (fix generate_types.rs header comment path)
- **Issue:** `test_generated_output_includes_header` at line 1995 asserted `typescript.contains("frontend/src/types/")` — this was verifying the old (wrong) path that D-18 explicitly corrects. After the fix, the test failed with the corrected behavior.
- **Fix:** Changed test assertion to `typescript.contains("frontend/src/lib/types/")` to match the now-correct emitted header.
- **Files modified:** `ferro-cli/src/commands/generate_types.rs` (line 1995)
- **Verification:** `cargo test -p ferro-cli` passes: 480 tests, 0 failed
- **Committed in:** `63f6e8bc` (same task commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 — bug: test asserting old path)
**Impact on plan:** Auto-fix necessary for test correctness. The test was a regression guard for the wrong behavior; updating it is required for the fix to hold.

## Issues Encountered

None — all three tasks completed without surprises.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 01 establishes the first observable truth: `git ls-files app/frontend/src/types/` returns empty, convention is annotated in the template, generator header is corrected.
- Plans 02–06 can proceed: doctor check (02), docs page (03), Dockerfile types-gen stage (04), README troubleshooting entry (05), version bump (06).
- No blockers.

## Self-Check

- [x] `app/frontend/src/types/inertia-props.ts` removed from git index: `git ls-files app/frontend/src/types/` returns 0 lines
- [x] `app/frontend/src/types/routes.ts` removed from git index: same check
- [x] Both files still exist on disk: verified
- [x] `gitignore.tpl` contains `load-bearing: frontend/src/types/ is owned by`: verified
- [x] `gitignore.tpl` contains `see docs/src/cli/frontend-types.md`: verified
- [x] `generate_types.rs` contains `output.push_str("// frontend/src/lib/types/\n")`: verified
- [x] Commit `63f6e8bc` exists: verified
- [x] `cargo fmt --all -- --check`: PASS
- [x] `cargo clippy --all --all-targets -- -D warnings`: PASS
- [x] `cargo test --all-features`: PASS (0 FAILED)

## Self-Check: PASSED

---
*Phase: 156-frontend-types-directory-generator-owned-convention*
*Completed: 2026-05-14*
