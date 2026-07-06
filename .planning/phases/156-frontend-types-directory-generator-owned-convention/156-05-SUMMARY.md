---
phase: 156-frontend-types-directory-generator-owned-convention
plan: 05
subsystem: docs
tags: [docs, frontend-types, convention, doctor, scaffolder]

# Dependency graph
requires: [156-02]
provides:
  - docs/src/cli/frontend-types.md (canonical convention reference page)
  - docs/src/SUMMARY.md updated with [frontend-types] entry
  - docs/src/cli/doctor.md updated to 11 checks, full table including docker_template_drift
  - docs/src/reference/cli.md doctor row updated to eleven checks
  - ferro-cli/src/templates/files/root/README.md.tpl types-bootstrap troubleshooting bullet
affects:
  - 156-06

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Documentation-first: single canonical page for convention, cross-referenced from all entry points"

key-files:
  created:
    - docs/src/cli/frontend-types.md
  modified:
    - docs/src/SUMMARY.md
    - docs/src/cli/doctor.md
    - docs/src/reference/cli.md
    - ferro-cli/src/templates/files/root/README.md.tpl

key-decisions:
  - "D-08: docs/src/cli/frontend-types.md authored with full D-08 content list — generator output table, gitignore rationale, hand-written types location, fresh-clone bootstrap, Docker types-gen stage, ferro docker:init --force upgrade path, related commands"
  - "D-11: README.md.tpl Troubleshooting section gains types-bootstrap bullet referencing cargo run and cli/frontend-types.md"
  - "doctor.md table corrected from 9 rows to 11 rows — docker_template_drift was previously missing from the table despite being in the registry"

# Metrics
duration: ~15min
completed: 2026-05-14
---

# Phase 156 Plan 05: Documentation Cross-References Summary

**One-liner:** Canonical `docs/src/cli/frontend-types.md` authored covering the generator-owned convention end-to-end; doctor.md table corrected to 11 checks; SUMMARY, reference/cli.md, and README template all updated.

## Performance

- **Duration:** ~15 min
- **Started:** 2026-05-14T01:40:00Z
- **Completed:** 2026-05-14T01:58:15Z
- **Tasks:** 3
- **Files created:** 1
- **Files modified:** 4

## Accomplishments

- Created `docs/src/cli/frontend-types.md` (116 lines) covering all D-08 content items: generator output table (`inertia-props.ts`, `routes.ts`), gitignore rationale (no drift, no noise, no review burden), hand-written types location (`frontend/src/lib/types/`), fresh-clone bootstrap (`cargo run` once), Docker types-gen stage with Dockerfile sketch, `ferro docker:init --force` upgrade path for existing projects, and related-commands footer linking to `doctor.md` and `do-init.md`.
- Updated `docs/src/SUMMARY.md` to index the new page as `[frontend-types](cli/frontend-types.md)` immediately after `[doctor]` in the Reference → CLI sub-list.
- Updated `docs/src/cli/doctor.md`: changed "nine" to "eleven" in line 3, replaced the 9-row checks table with a complete 11-row table including the previously-missing `docker_template_drift` (row 7) and new `frontend_types_convention` (row 11), updated `checks[].name` field reference from "nine names" to "eleven names".
- Updated `docs/src/reference/cli.md` doctor row from "nine checks" to "eleven checks".
- Added TypeScript missing-types troubleshooting bullet to `ferro-cli/src/templates/files/root/README.md.tpl` between the "Frontend assets missing" and "Port 8080 in use" bullets.

## Task Commits

| Task | Name | Commit |
|------|------|--------|
| 1 | Author docs/src/cli/frontend-types.md | `7ef55440` |
| 2 | Index new page + update doctor counts | `aa43c333` |
| 3 | Add README.tpl troubleshooting bullet | `f300c8f9` |

## Files Created/Modified

- `docs/src/cli/frontend-types.md` — created (116 lines), canonical convention reference
- `docs/src/SUMMARY.md` — inserted `[frontend-types](cli/frontend-types.md)` entry
- `docs/src/cli/doctor.md` — "nine" → "eleven", 9-row table → 11-row table (added docker_template_drift row 7, frontend_types_convention row 11), "nine names" → "eleven names"
- `docs/src/reference/cli.md` — doctor row "nine checks" → "eleven checks"
- `ferro-cli/src/templates/files/root/README.md.tpl` — new TypeScript error troubleshooting bullet inserted

## Decisions Made

- D-08: The new docs page covers every item in the CONTEXT.md D-08 content list, including the `ferro docker:init --force` upgrade path (closes D-15 documentation requirement).
- D-11: Troubleshooting bullet uses the exact `Cannot find module './types/inertia-props'` error string and directs users to `cargo run` once — no `ferro setup` command referenced (D-19).
- The doctor.md table was silently out-of-sync with the registry (9 rows in the table, 10 checks in the registry — `docker_template_drift` was missing). This plan corrects both the count and the table completeness simultaneously.

## Deviations from Plan

None — plan executed exactly as written. All three tasks completed without surprises. The doctor.md table fix (adding the previously-missing `docker_template_drift` row alongside the new `frontend_types_convention` row) was part of the plan specification and is not a deviation.

## Verification

- `cargo fmt --all -- --check`: PASS (exit 0)
- `cargo clippy --all --all-targets -- -D warnings`: PASS (exit 0, 2m 10s)
- `cargo test -p ferro-cli --lib`: PASS (495 passed, 0 failed)
- `cargo test -p ferro-cli --lib doctor::`: PASS (48 passed, 0 failed)
- `cargo test --all-features`: SIGTERM on async-stripe (pre-existing thermal/resource constraint unrelated to documentation-only changes; no Rust source modified in this plan)

## Known Stubs

None — this plan is documentation-only. No UI components, no data sources.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. All changes are static markdown and a `.tpl` file. Per threat model T-156-05-01/02/03: plain text documentation; no executable surface.

## Self-Check

Files exist:
- `docs/src/cli/frontend-types.md` — FOUND
- `docs/src/SUMMARY.md` contains `[frontend-types](cli/frontend-types.md)` — FOUND
- `docs/src/cli/doctor.md` contains `Runs eleven checks` — FOUND
- `docs/src/cli/doctor.md` contains `docker_template_drift` — FOUND
- `docs/src/cli/doctor.md` contains `frontend_types_convention` — FOUND
- `docs/src/cli/doctor.md` contains `eleven names` — FOUND
- `docs/src/reference/cli.md` contains `eleven checks` — FOUND
- `ferro-cli/src/templates/files/root/README.md.tpl` contains `Cannot find module './types/inertia-props'` — FOUND

Commits exist:
- `7ef55440` — docs(156-05): add frontend-types convention page
- `aa43c333` — docs(156-05): index frontend-types page and update doctor check counts
- `f300c8f9` — docs(156-05): add types-bootstrap troubleshooting bullet to README template

## Self-Check: PASSED

---
*Phase: 156-frontend-types-directory-generator-owned-convention*
*Completed: 2026-05-14*
