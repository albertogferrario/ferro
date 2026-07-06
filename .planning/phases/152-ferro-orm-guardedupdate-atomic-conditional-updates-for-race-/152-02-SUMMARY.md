---
phase: 152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-
plan: 02
subsystem: workspace-registration
tags: [workspace, publish, ci, wave-1a, leaf-crate, docs]

# Dependency graph
requires: [01]
provides:
  - ferro-orm registered in three workspace discovery surfaces (root Cargo.toml, publish.yml Wave 1a, CLAUDE.md Workspace Structure table)
  - Compile boundary unchanged from plan 01 (cargo build --workspace continues to compile ferro-orm cleanly)
  - Wave 1a publish slot reserved for ferro-orm (first run will fail without manual bootstrap; plan 06 owns the bootstrap)
affects: [152-03, 152-04, 152-05, 152-06]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Wave 1a append-only convention preserved (new crate at end of WAVE1A_CRATES, never reordered)"
    - "CLAUDE.md Workspace Structure table follows phase-introduction order (new row at end of ferro-* block, before app)"

key-files:
  created: []
  modified:
    - .github/workflows/publish.yml
    - CLAUDE.md

key-decisions:
  - "Task 1 (Cargo.toml workspace.members append) was an idempotent no-op: plan 01 already pulled forward the registration (Rule 3 blocking deviation in 152-01-SUMMARY.md). Verified the entry exists exactly once at line 25, directly after ferro-wallet. No additional commit produced for Task 1."
  - "ferro-wallet table row remains absent from CLAUDE.md (deferred per plan scope-discipline flag from the planning PATTERNS.md; documented in 152-02-PLAN.md Task 3 acceptance criteria)"

patterns-established:
  - "Idempotent registration handling: when a follow-up plan finds its first edit already applied by a prior plan's Rule 3 deviation, document the no-op in the SUMMARY rather than duplicating the edit or skipping verification"

requirements-completed: []

# Metrics
duration: 4m 20s
completed: 2026-05-13
---

# Phase 152 Plan 02: ferro-orm workspace registration Summary

**ferro-orm registered in publish.yml Wave 1a and CLAUDE.md Workspace Structure table; root Cargo.toml entry was already in place from plan 01's Rule 3 deviation (idempotent no-op, verified).**

## Performance

- **Duration:** 4m 20s
- **Started:** 2026-05-13T15:31:02Z
- **Completed:** 2026-05-13T15:35:22Z
- **Tasks:** 3 (1 idempotent no-op + 2 single-line edits)
- **Files created:** 0
- **Files modified:** 2 (.github/workflows/publish.yml, CLAUDE.md)
- **Commits:** 2 (Task 1 produced no commit; see Decisions)

## Accomplishments

- `cargo build --workspace` exits 0 with ferro-orm compiled as a workspace member (1m 51s clean build from a cold worktree).
- `cargo clippy --workspace --all-targets -- -D warnings` exits 0 (1m 14s).
- `cargo fmt --all -- --check` exits 0.
- `cargo metadata --no-deps` lists `ferro-orm` exactly once in the workspace member set.
- publish.yml WAVE1A_CRATES contains `ferro-wallet ferro-orm` (ferro-orm appended after ferro-wallet, never before).
- CLAUDE.md Workspace Structure table contains a new `ferro-orm` row positioned between `ferro-whatsapp` and `app` (phase-introduction order preserved).
- ferro-wallet table row remains absent from CLAUDE.md, as required by the plan's scope-discipline directive (cleanup deferred to a future phase).

## Task Commits

1. **Task 1: root Cargo.toml workspace.members append** — **idempotent no-op, no commit produced**
   - The line `"ferro-orm",` was already present at Cargo.toml line 25 (directly after `"ferro-wallet",` at line 24), having been added by plan 01 as a Rule 3 blocking deviation (see 152-01-SUMMARY.md §Deviations from Plan §1).
   - Verified via `grep -c '"ferro-orm"' Cargo.toml` → 1, plus ordering awk check.
   - No file was modified; no commit produced.
2. **Task 2: .github/workflows/publish.yml — append ferro-orm to WAVE1A_CRATES** — `c05a1cc9` (ci)
   - Single-line edit on line 201; diff is 1 insertion / 1 deletion.
3. **Task 3: CLAUDE.md — insert ferro-orm row in Workspace Structure table** — `f1733e47` (docs)
   - Single-line insertion at line 58 (between `ferro-whatsapp` and `app`).

## Files Modified

- `.github/workflows/publish.yml` (line 201) — appended ` ferro-orm` to the `WAVE1A_CRATES` env-var string. The publish loop now attempts to publish ferro-orm last in Wave 1a. First CI run after merge will fail with "no upload permission" / "not found" because the crate does not yet exist on crates.io; that is expected exactly once (token scope is `publish-update`, not `publish-new`). Plan 06 owns the manual `cargo publish -p ferro-orm` bootstrap from a local terminal with a personal `publish-new`-scoped token.
- `CLAUDE.md` (line 58) — inserted row `` | `ferro-orm` | Atomic conditional updates and ORM primitives (`GuardedUpdate`) | `src/lib.rs` | `` between the `ferro-whatsapp` row and the `app` row. Downstream agents reading CLAUDE.md will see the crate immediately.

## Files NOT Modified (per plan scope)

- `Cargo.toml` — already-correct from plan 01's Rule 3 deviation; documented above as an idempotent no-op.
- No other files in publish.yml or CLAUDE.md were touched; both diffs are minimal single-line changes.

## Decisions Made

- **Task 1 idempotent no-op, no commit.** Plan 01's SUMMARY (file lines 105-110) explicitly notes the workspace registration was pulled forward "to keep plan 01's verification self-contained — plan 02's existing edit becomes idempotent." That prediction held: the line is already present at the exact insertion point the plan describes, with the correct phase-introduction ordering. The orchestrator instruction prompt anticipated this case ("If it is, mark Task 1 as idempotent-no-op...DO NOT duplicate the entry"). No commit was produced because no file changed; a no-op marker commit would have been semantically meaningless and would pollute git log.
- **`ferro-wallet` table row deferred.** CLAUDE.md's Workspace Structure table is currently missing a row for `ferro-wallet` (oversight from Phase 151). The plan explicitly directs NOT to fix this here: "decide based on scope discipline (probably defer to a separate cleanup phase)." This plan honors the boundary. Tracked as a known cleanup item.
- **Plan-level verification chain.** Beyond per-task verifies, ran full pre-commit discipline (`cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`) once after the last task. All green. This mirrors the CI command exactly per project memory (`feedback_ci_clippy_command_match.md`).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule N/A — Plan Idempotency] Task 1 detected as already-applied; no commit produced**
- **Found during:** Task 1 (pre-edit verification of Cargo.toml state)
- **Issue:** Plan 02 Task 1 specifies appending `"ferro-orm",` to `[workspace.members]`. The line was already present from plan 01's Rule 3 deviation (152-01-SUMMARY.md §Deviations §1). Re-applying the edit would either duplicate the entry (incorrect) or be a no-op edit that wouldn't pass `Edit` tool uniqueness checks.
- **Fix:** Recognized this is the documented idempotent path. Verified the entry is present exactly once with correct ordering (after `ferro-wallet`, before the closing bracket). No file modified, no commit produced.
- **Verification:**
  - `grep -n '"ferro-orm",' Cargo.toml` → `25:    "ferro-orm",`
  - `grep -c '"ferro-orm"' Cargo.toml` → `1` (exactly one occurrence)
  - `awk` ordering check → "ORDER-OK: ferro-orm follows ferro-wallet"
  - `cargo metadata --no-deps | jq '.workspace_members[]' | grep -c ferro-orm` → `1`
- **Committed in:** No commit. The orchestrator's prompt explicitly authorized this path: "DO NOT duplicate the entry, and proceed to Task 2 (publish.yml) and Task 3 (CLAUDE.md)."

---

**Total deviations:** 1 (idempotent no-op on Task 1, anticipated by both plan 01's SUMMARY and the orchestrator prompt).
**Impact on plan:** No scope change. The 3-task plan resolves to 2 commits + 1 documented no-op. All success criteria are met because plan 01 already paid the cost.

## Issues Encountered

None — execution was a single clean pass. No build failures, no clippy regressions, no fmt drift.

## User Setup Required

None — this plan modifies only repository-internal registration files. The first CI publish attempt for ferro-orm WILL fail (CI token scope), and that is the expected pre-bootstrap state; plan 06 includes the manual bootstrap step.

## Next Phase Readiness

- **Plan 03 (GuardedUpdate body):** Can proceed in parallel with this plan (same Wave 2 per phase plan-wave layout). ferro-orm is now visible to `cargo build --workspace`, `cargo metadata`, and downstream agent introspection.
- **Plan 04 (integration test):** Unblocked — the integration test will live at `ferro-orm/tests/concurrent_decrement.rs` and SeaORM dev-deps are already in `ferro-orm/Cargo.toml` from plan 01.
- **Plan 05 (docs):** Unblocked — docs/src/database/atomic-updates.md will reference the published API surface.
- **Plan 06 (release):** Unblocked — version bump + first-publish bootstrap path is documented and CI scaffold is in place.
- **No blockers, no concerns.**

## Self-Check: PASSED

- Cargo.toml line 25 `"ferro-orm",` present — FOUND
- publish.yml line 201 contains `ferro-wallet ferro-orm` — FOUND
- publish.yml inverse order `ferro-orm.*ferro-wallet` — ABSENT (correct)
- CLAUDE.md `ferro-orm` row present at line 58 — FOUND
- CLAUDE.md ferro-orm row positioned after ferro-whatsapp and before app — VERIFIED via awk
- CLAUDE.md `ferro-wallet` row still ABSENT (deferred per plan) — VERIFIED
- Commit `c05a1cc9` (Task 2 ci) — FOUND in `git log`
- Commit `f1733e47` (Task 3 docs) — FOUND in `git log`
- `cargo build --workspace` exits 0 with ferro-orm compiled — VERIFIED (1m 51s)
- `cargo clippy --workspace --all-targets -- -D warnings` exits 0 — VERIFIED (1m 14s)
- `cargo fmt --all -- --check` exits 0 — VERIFIED
- `cargo metadata --no-deps` lists ferro-orm exactly once — VERIFIED (`grep -c` = 1)

---
*Phase: 152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-*
*Completed: 2026-05-13*
