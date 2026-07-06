---
phase: 155
plan: "02"
subsystem: workspace-registration
tags: [workspace, publish-ci, documentation, version-bump]
dependency_graph:
  requires: [ferro-projection crate scaffold (plan 01)]
  provides: [workspace registration, Wave 1b CI slot, CLAUDE.md row, README.md bullet]
  affects: [Cargo.toml, .github/workflows/publish.yml, CLAUDE.md, README.md]
tech_stack:
  added: []
  patterns: [workspace version bump, Wave 1b publish registration, disambiguation surface extension]
key_files:
  created: []
  modified:
    - Cargo.toml
    - .github/workflows/publish.yml
    - CLAUDE.md
    - README.md
decisions:
  - Task 1 workspace member registration was idempotent (Plan 01 deviation pre-registered ferro-projection)
  - Version bumped 0.2.32 → 0.2.33 per D-54
  - ferro-projection placed in Wave 1b (not Wave 1a) per D-04/D-05: has ferro-events + ferro-broadcast deps
metrics:
  duration_seconds: 480
  completed_date: "2026-05-14"
  tasks_completed: 5
  files_created: 0
  files_modified: 4
---

# Phase 155 Plan 02: Workspace Registration and Documentation Summary

Registered `ferro-projection` as a recognized workspace member (workspace member entry was already present from Plan 01 deviation — idempotent), bumped workspace version 0.2.32 → 0.2.33, added the crate to `WAVE1B_CRATES` in CI publish workflow, and extended the D-02 disambiguation surface to CLAUDE.md and the workspace root README.md.

## Files Modified

| File | Change |
|------|--------|
| `Cargo.toml` | Version bumped 0.2.32 → 0.2.33; `ferro-projection` workspace member already present (Plan 01 deviation) |
| `.github/workflows/publish.yml` | `ferro-projection` appended at end of `WAVE1B_CRATES` (Wave 1b per D-04/D-05) |
| `CLAUDE.md` | New `ferro-projection` row in Workspace Structure table (after `ferro-reservation`, before `app`) |
| `README.md` | New `**Live read-models**` bullet in "What's included" list (after `Resource reservations` bullet) |

## Version Bump Applied

`0.2.32` → `0.2.33` (per D-54). Execution-time grep confirmed `version = "0.2.32"` was the current value before edit.

## Build Gate Results

- `cargo build -p ferro-projection` exits 0 (workspace package shorthand now works — replaces Plan 01's `--manifest-path` workaround)
- `cargo build --workspace` exits 0
- `cargo test -p ferro-projection --lib error::tests`: **5 passed, 0 failed**

## Disambiguation Phrase Coverage (D-02)

The literal phrase appears in these locations after Plan 02:

| Location | Phrase |
|----------|--------|
| `ferro-projection/Cargo.toml` description | `(not the same as ferro-projections plural)` |
| `ferro-projection/README.md` | `Not the same as \`ferro-projections\` (plural).` |
| `ferro-projection/src/lib.rs` rustdoc | `Not to be confused with [\`ferro-projections\`] (plural).` |
| `CLAUDE.md` workspace table row | `**Not the same as \`ferro-projections\` (plural)**` |
| workspace `README.md` bullet | `not the same as \`ferro-projections\` plural` |

Plans 07 will extend to `docs/src/features/live-read-models.md` and `CHANGELOG.md`.

## Deviations from Plan

### Task 1 Idempotency (Plan 01 Deviation)

**1. [No-op] Workspace member registration already done by Plan 01**

- **Found during:** Task 1 (grep check)
- **Issue:** Plan 01 deviated (Rule 3 — blocking) to add `"ferro-projection"` to `[workspace.members]` because `version.workspace = true` required it for the build gate. The workspace member entry was already present.
- **Fix:** Skipped the member-add edit; only applied the version bump (`0.2.32 → 0.2.33`).
- **Impact:** Task 1 partial no-op — version bump still needed and applied. All acceptance criteria met.

## Commits

| Hash | Type | Message |
|------|------|---------|
| `9754b3e7` | chore | bump workspace version 0.2.32 → 0.2.33 (D-54) |
| `0fd6adab` | chore | add ferro-projection to WAVE1B_CRATES in publish.yml (D-04, D-05, D-55) |
| `785f9e5e` | docs | add ferro-projection row to CLAUDE.md workspace structure table (D-02) |
| `6c6eeb5a` | docs | add ferro-projection bullet to README.md What's included list (D-02) |

## Requirements Addressed

| Decision | Status |
|----------|--------|
| D-04 — Wave 1b placement (ferro-events + ferro-broadcast deps) | Satisfied |
| D-05 — ferro-projection in WAVE1B_CRATES in publish.yml | Satisfied |
| D-54 — workspace version bump 0.2.32 → 0.2.33 | Satisfied |
| D-55 — Wave 1b auto-publish slot reserved; manual bootstrap deferred to plan 07 | Satisfied |

## Self-Check: PASSED

- [x] `Cargo.toml` contains `"ferro-projection"` in `[workspace.members]`
- [x] `Cargo.toml` `[workspace.package] version` = `"0.2.33"`
- [x] `WAVE1B_CRATES` ends with `ferro-projection` in publish.yml
- [x] `WAVE1A_CRATES` does NOT contain `ferro-projection`
- [x] `CLAUDE.md` row at line 61: after ferro-reservation (60), before app (62)
- [x] Disambiguation phrase in CLAUDE.md (grep-verified)
- [x] README.md bullet at line 74: after ferro-reservation (73)
- [x] Disambiguation phrase in README.md (grep-verified)
- [x] `cargo build -p ferro-projection` exits 0
- [x] `cargo build --workspace` exits 0
- [x] `cargo test -p ferro-projection --lib error::tests` reports 5 passed
- [x] All 4 task commits present in git log
