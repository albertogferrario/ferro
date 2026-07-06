---
phase: 154-ferro-reservation-crate-generic-hold-commit-release-with-ttl
plan: 02
subsystem: infra
tags: [workspace-registration, cargo-toml, publish-ci, version-bump, wave-1b]
dependency_graph:
  requires:
    - phase: 154-01
      provides: ferro-reservation crate scaffold (already added ferro-reservation to workspace members as Rule 3 deviation)
  provides:
    - workspace version bumped to 0.2.32 (D-56)
    - ferro-reservation in WAVE1B_CRATES in publish.yml (D-04, D-57)
    - CLAUDE.md Workspace Structure table row for ferro-reservation
    - README.md What's included bullet for ferro-reservation
  affects: [154-03, 154-04, 154-05, 154-06, 154-07, cargo build --workspace]
tech-stack:
  added: []
  patterns:
    - "workspace-members add already done in plan 01 Rule 3 deviation — plan 02 task 1 is a no-op for members, only version bump needed"
    - "Wave 1b crate registration pattern: append to WAVE1B_CRATES after ferro-notifications"
key-files:
  created: []
  modified:
    - Cargo.toml
    - .github/workflows/publish.yml
    - CLAUDE.md
    - README.md
key-decisions:
  - "Workspace members add was already done in plan 01 (Rule 3 deviation, commit a62e93d5) — plan 02 task 1 was a no-op for the members list, only the version bump was applied"
  - "Version bumped 0.2.31 → 0.2.32 per D-56 (execution-time value was still 0.2.31 as expected)"
  - "ferro-reservation placed in WAVE1B_CRATES (not WAVE1A_CRATES) per D-04 — has ferro-orm + ferro-events + ferro-audit runtime deps; must serialize after Wave 1a"
  - "First publish of ferro-reservation is a manual bootstrap (plan 154-07) — CI token has publish-update only, cannot create new crate"
patterns-established:
  - "Plan 01 workspace-members Rule 3 deviation pattern: when path deps require workspace resolution, add to members in plan 01 even if plan 02 owns that task — document clearly in both summaries"
requirements-completed: [D-04, D-56, D-57]
duration: ~5min
completed: 2026-05-13
---

# Phase 154 Plan 02: ferro-reservation Workspace Registration Summary

**Workspace version bumped 0.2.31 → 0.2.32; ferro-reservation registered in WAVE1B_CRATES, CLAUDE.md table, and README.md crates list — cargo build -p ferro-reservation now works without --manifest-path workaround**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-05-13T00:00:00Z
- **Completed:** 2026-05-13T00:05:00Z
- **Tasks:** 4 (Task 1 workspace-members add was a no-op — already done in plan 01)
- **Files modified:** 4

## Accomplishments

- Version bump 0.2.31 → 0.2.32 applied to `[workspace.package]` (D-56)
- `ferro-reservation` registered in `WAVE1B_CRATES` in `.github/workflows/publish.yml` (D-04, D-57) — NOT in WAVE1A_CRATES
- `CLAUDE.md` Workspace Structure table row added between `ferro-audit` and `app`
- `README.md` "What's included" bullet added immediately after `ferro-audit` entry
- `cargo build -p ferro-reservation` now resolves via workspace (no `--manifest-path` workaround needed)
- `cargo metadata --no-deps` exits 0; publish.yml YAML valid

## Task Commits

1. **Task 1: Register ferro-reservation in workspace Cargo.toml and bump version** - `12891ea4` (chore)
   - Note: workspace-members add was a no-op (plan 01 Rule 3 deviation already inserted the member)
   - Version bump 0.2.31 → 0.2.32 was the only change
2. **Task 2: Add ferro-reservation to WAVE1B_CRATES in publish.yml** - `5f0104e9` (chore)
3. **Task 3: Add ferro-reservation row to CLAUDE.md Workspace Structure table** - `5f2ef163` (docs)
4. **Task 4: Add ferro-reservation bullet to README.md** - `6296c52e` (docs)

## Files Created/Modified

- `Cargo.toml` — `[workspace.package] version` bumped from `0.2.31` to `0.2.32`; `ferro-reservation` already present in `[workspace.members]` (plan 01 deviation)
- `.github/workflows/publish.yml` — `WAVE1B_CRATES` line 236 now ends with `ferro-reservation`
- `CLAUDE.md` — new row `| \`ferro-reservation\` | Generic hold/commit/release reservation kernel | \`src/lib.rs\` |` inserted between `ferro-audit` and `app` rows
- `README.md` — new bullet `- **Resource reservations** — race-free hold/commit/release with TTL, audit, and event broadcast (\`ferro-reservation\`)` inserted after `ferro-audit` bullet

## Decisions Made

- **Workspace-members add no-op:** Plan 01 added `ferro-reservation` to `[workspace.members]` as a Rule 3 deviation (blocking — path deps to ferro-orm/ferro-events/ferro-audit require workspace resolution before any build can succeed). Task 1 of this plan did NOT re-add it. The version bump was the only change in Task 1.
- **Version target confirmed at execution time:** `grep -E '^version = ' Cargo.toml` returned `0.2.31` (not already auto-bumped by CI), confirming the D-56 target of `0.2.32` was correct.
- **WAVE1B placement confirmed correct:** ferro-reservation has ferro-orm + ferro-events + ferro-audit as runtime deps (D-03). All three are Wave 1a crates. Adding ferro-reservation to WAVE1A_CRATES would race-publish it against its own deps. The verify check `! grep -q 'WAVE1A_CRATES=".*ferro-reservation'` passes.

## Deviations from Plan

### Task 1 — Workspace Members Add Was Already Done (No-Op)

Plan 02 Task 1 specified adding `ferro-reservation` to `[workspace.members]`. This was already done by plan 01 as a Rule 3 deviation (commit `a62e93d5`). The members list required no change; only the version bump was applied.

This matches the documented behavior in the `<important_context>` block and in `154-01-SUMMARY.md` §Deviations. The same pattern occurred in Phase 152 plan 01 and Phase 153 plan 01.

**No other deviations** — plan executed as written for Tasks 2, 3, and 4.

## Gate Results

| Gate | Result |
|------|--------|
| `grep '^version = "0.2.32"' Cargo.toml` | PASS |
| `grep '"ferro-reservation"' Cargo.toml` (members) | PASS |
| `grep 'WAVE1B_CRATES=".*ferro-reservation"' publish.yml` | PASS |
| `! grep 'WAVE1A_CRATES=".*ferro-reservation' publish.yml` | PASS |
| `! grep 'WAVE2_CRATES=".*ferro-reservation' publish.yml` | PASS |
| `grep '| \`ferro-reservation\` |' CLAUDE.md` | PASS |
| `grep 'ferro-reservation' README.md` | PASS |
| `cargo metadata --no-deps --format-version 1` | PASS |
| `cargo build -p ferro-reservation` (v0.2.32, no --manifest-path) | PASS |
| python3 YAML validation of publish.yml | PASS |

## Issues Encountered

None. All tasks executed cleanly.

## Next Phase Readiness

- `cargo build -p ferro-reservation` works via workspace (no workaround needed)
- Plans 03–06 can proceed against the stable workspace member
- Plan 154-07 (manual crates.io bootstrap) is the next gate — requires personal publish-new token; WAVE1B_CRATES slot is reserved for subsequent auto-publishes

---
*Phase: 154-ferro-reservation-crate-generic-hold-commit-release-with-ttl*
*Completed: 2026-05-13*
