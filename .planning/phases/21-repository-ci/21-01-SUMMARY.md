---
phase: 21-repository-ci
plan: 01
subsystem: infra
tags: [github-actions, ci-cd, installer, cargo-toml, mdbook]

# Dependency graph
requires:
  - phase: 20-templates-scaffolding
    provides: CLI templates rebranded to ferro
provides:
  - release.yml workflow builds and packages ferro-cli binary
  - install.sh and create-app.sh download from ferroframework/ferro
  - All Cargo.toml files point to ferroframework/ferro repository
  - Documentation config uses ferro-rs.dev URLs
affects: [22-publishing-announcement]

# Tech tracking
tech-stack:
  added: []
  patterns: []

key-files:
  created: []
  modified:
    - .github/workflows/release.yml
    - scripts/install.sh
    - scripts/create-app.sh
    - framework/Cargo.toml
    - ferro-cli/Cargo.toml
    - ferro-events/Cargo.toml
    - ferro-queue/Cargo.toml
    - ferro-notifications/Cargo.toml
    - ferro-broadcast/Cargo.toml
    - ferro-macros/Cargo.toml
    - docs/book.toml

key-decisions:
  - "inertia-rs repository URL unchanged (separate project)"

# Metrics
duration: 8min
completed: 2026-01-16
---

# Phase 21 Plan 01: Repository & CI Rebrand Summary

**Updated GitHub Actions release workflow, installer scripts, Cargo.toml repository URLs, and documentation config from cancer to ferro naming**

## Performance

- **Duration:** 8 min
- **Started:** 2026-01-16T14:35:00Z
- **Completed:** 2026-01-16T14:43:00Z
- **Tasks:** 4
- **Files modified:** 11

## Accomplishments

- Updated release.yml to build ferro-cli and package ferro binaries
- Rebranded both installer scripts with ferroframework/ferro URLs and ferro binary names
- Updated all relevant Cargo.toml files with ferroframework/ferro repository URLs
- Updated docs/book.toml with ferro-rs.dev URLs and Ferro Framework title

## Task Commits

Each task was committed atomically:

1. **Task 1: Update GitHub Actions release workflow** - `18177b4` (chore)
2. **Task 2: Update installer scripts** - `fd0002e` (chore)
3. **Task 3: Update Cargo.toml repository URLs** - `98b8e7c` (chore)
4. **Task 4: Update documentation config** - `66d2bf1` (chore)

## Files Created/Modified

- `.github/workflows/release.yml` - Build ferro-cli, package ferro binaries, ferro archive names
- `scripts/install.sh` - ferroframework/ferro repo, ferro binary, FERRO_INSTALL_DIR
- `scripts/create-app.sh` - ferroframework/ferro repo, ferro binary, ferro archive names
- `framework/Cargo.toml` - repository and homepage URLs
- `ferro-cli/Cargo.toml` - repository and homepage URLs
- `ferro-events/Cargo.toml` - repository URL
- `ferro-queue/Cargo.toml` - repository URL
- `ferro-notifications/Cargo.toml` - repository URL
- `ferro-broadcast/Cargo.toml` - repository URL
- `ferro-macros/Cargo.toml` - repository URL
- `docs/book.toml` - git-repository-url, edit-url-template, site-url, cname, title

## Decisions Made

- **inertia-rs repository URL unchanged** - inertia-rs has its own repository (inertiajs/inertia-rs) which is correct and should not be changed to ferroframework/ferro
- **ferro-storage, ferro-cache, ferro-mcp skipped** - These crates don't have repository fields in their Cargo.toml files

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All infrastructure references updated to ferroframework/ferro
- Ready for Phase 22: Publishing & Announcement
- Note: Actual GitHub repository rename is a manual step outside this plan

---
*Phase: 21-repository-ci*
*Completed: 2026-01-16*
