---
phase: 152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-
plan: 05
subsystem: database
tags: [ferro-orm, guarded-update, documentation, mdbook, atomic-updates, concurrency]

# Dependency graph
requires:
  - phase: 152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race- (plan 01)
    provides: ferro-orm crate scaffold with GuardedUpdate public API
  - phase: 152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race- (plan 03)
    provides: GuardedError variants and final exec_one / exec_at_most_one shape that the docs describe
provides:
  - docs/src/database/atomic-updates.md (user-facing conceptual narrative for GuardedUpdate)
  - mdBook navigation entry under # Features pointing at the new page
affects: [154-ferro-reservation, 152-06, future phases citing the docs URL]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Concept docs page under docs/src/database/ (new sibling to features/, dedicated to ORM-level primitives)"
    - "Flat-sibling SUMMARY.md placement under # Features (Pattern 1 per 152-PATTERNS.md)"

key-files:
  created:
    - docs/src/database/atomic-updates.md
  modified:
    - docs/src/SUMMARY.md

key-decisions:
  - "Place the page at docs/src/database/atomic-updates.md (NOT under docs/src/features/database/) — claims the database/ namespace at the docs root, matching the eventual home for further ORM-primitive docs"
  - "Flat-sibling SUMMARY.md entry, not a nested sub-page under Database — no other # Features entry uses nested children, so introducing nesting just for this entry would be inconsistent"
  - "Replaced internal-voice token 'load-bearing signal' with 'operative signal' in two places — 'load-bearing' is on the CLAUDE.md forbidden-trigger-phrase list (and the plan's own verify grep). Substance unchanged."
  - "Kept `rust,ignore` on every code block — examples reference entities not in scope; mdbook doctest would otherwise fail to compile"

patterns-established:
  - "docs/src/database/ houses ORM-primitive concept pages (atomic-updates today; reservation/audit/projection candidates as later phases ship them)"
  - "Anti-pattern → replacement → API → patterns → contract → errors H2 sequence for ORM-primitive doc pages — readers leave understanding *why* the type exists, not just *how*"

requirements-completed: []

# Metrics
duration: 3min
completed: 2026-05-13
---

# Phase 152 Plan 05: docs/src/database/atomic-updates.md Summary

**User-facing mdBook page for GuardedUpdate — walks the reader from the read → check → write anti-pattern through the builder API to the atomicity-per-statement contract, registered in the # Features sidebar.**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-05-13T15:42:13Z
- **Completed:** 2026-05-13T15:45:07Z
- **Tasks:** 2
- **Files modified:** 2 (1 created, 1 modified)

## Accomplishments

- New docs page `docs/src/database/atomic-updates.md` (176 lines) covering the conceptual narrative for `GuardedUpdate`
- The page documents the full API surface that matches `ferro-orm/src/guarded.rs`: `GuardedUpdate::new`, `.filter`, `.set_expr`, `.set_value`, `.exec_one`, `.exec_at_most_one`
- The page documents the four `GuardedError` variants: `NoRowsAffected`, `TooManyRows { affected }`, `EmptyUpdate`, `Db(DbErr)`
- The page contains the `exec_one` vs `exec_at_most_one` decision tree as a markdown table, and the atomicity-per-statement (D-15) and `UPDATE … RETURNING` (D-10) limitations are framed explicitly
- mdBook navigation gains a flat sibling entry under `# Features` pointing at `database/atomic-updates.md`
- `mdbook build docs/` succeeds with zero warnings; HTML for the new page is generated at `docs/book/database/atomic-updates.html`

## H2 Section Titles (page outline)

1. `The Anti-Pattern: read → check → write`
2. `The Replacement: GuardedUpdate`
3. `API` — with H3 subsections for `new`, `.filter`, `.set_expr / .set_value`, `.exec_one vs .exec_at_most_one`
4. `Common Patterns` — with H3 subsections for counter decrement, status transition, optimistic update
5. `Atomicity Guarantee (and Its Limit)`
6. `Errors`
7. `Postgres vs SQLite`

## Task Commits

1. **Task 1: Author docs/src/database/atomic-updates.md** — `b15a5511` (docs)
2. **Task 2: Add atomic-updates page to docs/src/SUMMARY.md** — `bd1909e6` (docs)

## Files Created/Modified

- `docs/src/database/atomic-updates.md` (created, 176 lines) — full conceptual narrative
- `docs/src/SUMMARY.md` (modified, +1 line at line 34) — registers the new page in the mdBook nav

## SUMMARY.md Pattern Confirmation

Pattern 1 (flat-sibling) was used per `152-PATTERNS.md`. The new entry is a peer of `[Database](features/database.md)`, not a nested child. Rationale: no other entry in `# Features` currently uses nested children, so introducing nesting just for this entry would create an inconsistent navigation tree.

## mdBook Build

`mdbook build docs/` was run from inside the worktree. Result:
- Exit status: 0
- Output: "INFO Book building has started" / "INFO HTML book written to …/docs/book"
- Warning/error count: 0
- Generated HTML for the new page confirmed at `docs/book/database/atomic-updates.html`

## Decisions Made

- **Page placement** — `docs/src/database/atomic-updates.md`, a new sibling directory at the docs root, NOT `docs/src/features/database/atomic-updates.md`. This claims the `database/` namespace for ORM-primitive concept pages and avoids confusing the existing `features/database.md` (which is the framework's higher-level ORM layer documentation).
- **SUMMARY.md style** — flat sibling under `# Features` immediately after `[Database](features/database.md)`. Pattern 1 per `152-PATTERNS.md`.
- **Tone** — neutral scientific voice throughout; no forbidden trigger phrases (`killer feature`, `the bet`, `load-bearing`, `forcing function`, `we accept that`, `the risk we`, `no stop-loss`). No tenant identifiers (`gestiscilo`, `ferro application`, `example.com`).
- **Examples** — all use generic entity names (`inventory_units`, `counters`, `reservations`, `sessions`) drawn from the design doc. None reference any specific consumer app.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Voice Bug] Replaced "load-bearing signal" with "operative signal" (two occurrences)**
- **Found during:** Task 1 (authoring the page)
- **Issue:** The plan's authoritative draft contained the phrase "the load-bearing signal" twice (once in the `exec_one` vs `exec_at_most_one` paragraph and once in the `NoRowsAffected` row of the Errors table). `load-bearing` is on CLAUDE.md's "Repository documents must read as neutral" forbidden-trigger-phrase list, and the plan's own `<verify>` grep flags it (`! grep -qiE 'killer feature|the bet|load-bearing|forcing function|we accept that|the risk we|no stop-loss'`). Shipping the draft verbatim would fail the verify rule.
- **Fix:** Substituted "operative" for "load-bearing" in both places. Substance unchanged — the phrase still names predicate failure as the meaningful error signal for `exec_one`.
- **Files modified:** `docs/src/database/atomic-updates.md`
- **Verification:** `grep -iE 'killer feature|the bet|load-bearing|forcing function|we accept that|the risk we|no stop-loss' docs/src/database/atomic-updates.md` returns zero matches.
- **Committed in:** `b15a5511` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 voice bug)
**Impact on plan:** Substantive content identical to the plan's draft; only the forbidden trigger phrase was substituted. No scope change, no API change.

## Issues Encountered

- The worktree was checked out at `5f7e0849` (a `ferro-wallet` commit) which predates the `ferro-orm` crate from plans 01–04. The docs page references `ferro-orm` by name but does not import Rust code (`rust,ignore` blocks throughout), so the missing crate has no effect on the documentation work. mdBook builds the page without touching Rust.

## Self-Check

- [x] `docs/src/database/atomic-updates.md` exists (176 lines, > 100 lines floor)
- [x] Page begins with `# Atomic Updates`
- [x] Page contains H2 sections for anti-pattern, replacement, API, common patterns, atomicity guarantee, errors, postgres-vs-sqlite
- [x] Page documents `GuardedUpdate::new`, `.filter`, `.set_expr`, `.set_value`, `.exec_one`, `.exec_at_most_one`
- [x] Page documents `GuardedError::{NoRowsAffected, TooManyRows, EmptyUpdate, Db}`
- [x] Page contains zero forbidden trigger phrases (verified by grep)
- [x] Page contains zero tenant identifiers (verified by grep)
- [x] `docs/src/SUMMARY.md` line 34 = `- [Atomic Updates](database/atomic-updates.md)`
- [x] Link target is `database/atomic-updates.md` (flat path), not `features/database/atomic-updates.md`
- [x] Task 1 commit `b15a5511` exists in `git log --oneline`
- [x] Task 2 commit `bd1909e6` exists in `git log --oneline`
- [x] `mdbook build docs/` exits 0 with zero warnings/errors

## User Setup Required

None — documentation only.

## Next Phase Readiness

- The user-facing documentation surface required by D-21 is shipped. D-22 (rustdoc on `ferro-orm/src/lib.rs`) is already present in the main repo and was confirmed during context-gathering.
- Plan 152-06 (release / publish) can proceed; the docs site no longer has a missing-coverage gap for GuardedUpdate.
- The new `docs/src/database/` directory establishes the conventional home for upcoming ORM-primitive concept pages (Phase 154 reservation, Phase 153 audit, Phase 155 projection) — they should mirror this page's H2 structure (anti-pattern → replacement → API → patterns → contract → errors).

---
*Phase: 152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-*
*Plan: 05*
*Completed: 2026-05-13*
