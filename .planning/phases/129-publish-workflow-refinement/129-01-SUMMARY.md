---
phase: 129-publish-workflow-refinement
plan: 01
subsystem: infra
tags: [github-actions, ci-cd, publish, crates-io, bash, yaml]

requires:
  - phase: 126-deploy-experience-feedback
    provides: REPORT.md §8 identifying publish workflow auto-bumping on every push

provides:
  - Library-change gate inside check-version job emitting should_publish=none for non-library pushes
  - Explicit downstream if: guards on bump/yes equality preventing YAML boolean coercion
  - Single-source exclusion case statement for non-library paths

affects:
  - 129-02
  - 129-03

tech-stack:
  added: []
  patterns:
    - "should_publish three-state enum: bump/yes/none via $GITHUB_OUTPUT"
    - "Exclusion case statement as single source of truth for non-library paths"

key-files:
  created: []
  modified:
    - .github/workflows/publish.yml

key-decisions:
  - "Use should_publish=none (not no) to avoid YAML 1.1 boolean coercion of unquoted 'no'"
  - "Downstream jobs gate on explicit 'bump'|'yes' equality — eliminates the != '' sentinel pattern"
  - "Separate skip step (id: skip) and check step (id: check) both populate outputs; job outputs use || to merge"
  - "Exclusion list inlined as shell case statement inside gate step — handles globs natively"

patterns-established:
  - "Multi-state workflow output: gate step emits lib_changed=0|1; downstream steps conditioned on it"
  - "Job outputs merged across conditional steps via ${{ steps.a.outputs.x || steps.b.outputs.x }}"

requirements-completed: []

duration: 1min
completed: 2026-04-09
---

# Phase 129 Plan 01: Publish Workflow Refinement Summary

**Library-change gate in check-version job: non-library pushes (ferro-cli, docs, CI config, planning) set should_publish=none and skip all downstream jobs**

## Performance

- **Duration:** ~2 min
- **Started:** 2026-04-09T14:56:48Z
- **Completed:** 2026-04-09T14:58:36Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- Added `gate` step to `check-version` using `git diff --name-only <last-tag>..HEAD` filtered through an exclusion case statement
- Introduced `should_publish=none` as a third output value; downstream `test` and `publish` jobs now gate on explicit `== 'bump' || == 'yes'` equality
- Eliminated all `!= ''` guard patterns that would have passed `none` through
- First-run edge case (no v* tag) treated as library change — publish proceeds
- YAML validates cleanly via `python3 yaml.safe_load`; `check-version` outputs surface unchanged

## Task Commits

1. **Task 1: Add library-change gate to check-version job** - `922b5070` (feat)
2. **Task 2: Verify workflow YAML is syntactically valid** - verification only, no files changed

## Files Created/Modified

- `.github/workflows/publish.yml` - Added gate step, skip step, updated outputs and downstream if: guards

## Decisions Made

- Used a separate `skip` step (id: skip) for the no-library-change branch rather than wrapping the existing `check` step with complex conditionals. Both steps write the same three outputs; job outputs use `||` to merge whichever ran.
- `should_publish=none` chosen over `no` to avoid YAML 1.1 boolean coercion (`no` → `false` in some contexts).
- Downstream guards rewritten from `!= ''` to explicit `== 'bump' || == 'yes'` — more self-documenting and immune to future enum extension.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Gate is live in the workflow. Plans 129-02 and 129-03 (PUBLISHING.md docs + ferro-cli schema extension) can proceed independently.

---
*Phase: 129-publish-workflow-refinement*
*Completed: 2026-04-09*
