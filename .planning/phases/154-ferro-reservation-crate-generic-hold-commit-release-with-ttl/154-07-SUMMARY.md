---
phase: 154-ferro-reservation-crate-generic-hold-commit-release-with-ttl
plan: 07
subsystem: database
tags: [rust, ferro-reservation, docs, changelog, release, crates-io]

requires:
  - phase: 154-plan-06
    provides: 33 passing tests, run_sweep_once, full public API, green clippy

provides:
  - docs/src/database/reservations.md (D-54 user doc page, 14 sections)
  - docs/src/SUMMARY.md nav entry for reservations.md
  - CHANGELOG.md ## ferro-reservation section (D-58)
  - Workspace pre-release gate: fmt + clippy + test + doc all green

affects: [crates-io, docs-rs, STATE.md, ROADMAP.md]

tech-stack:
  added: []
  patterns:
    - "docs/src/database/*.md structure: anti-pattern → replacement → state diagram → API → lifecycle → context → TTL → events → audit → patterns → schema → errors → consistency → footguns"
    - "CHANGELOG newest-on-top: ## ferro-reservation above ## ferro-audit (Phase 152 D-25 convention)"

key-files:
  created:
    - docs/src/database/reservations.md
  modified:
    - docs/src/SUMMARY.md
    - CHANGELOG.md

decisions:
  - "Workspace version 0.2.32 used in CHANGELOG [0.2.32] — 2026-05-13 heading (no bump needed; plan 02 already bumped 0.2.31 → 0.2.32)"
  - "hold() SQLite concurrency limitation documented explicitly in Consistency Model section (per D-51 and 154-06 Bug #2 — tokio::Mutex pattern confirmed necessary)"
  - "Lifecycle Methods section notes the three-step hold() sequence is not a single atomic statement; callers on SQLite should serialize with a per-resource-key tokio::sync::Mutex"

metrics:
  duration: 10 min
  completed: 2026-05-13T21:43:00Z
  tasks_completed: 4
  tasks_pending: 1
  files_created: 1
  files_modified: 2
---

# Phase 154 Plan 07: Docs, CHANGELOG, and Release Bootstrap Summary

**Tasks 1-4 complete and committed; Task 5 (first-publish manual bootstrap) awaits operator action**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-05-13T21:33:00Z
- **Completed (Tasks 1-4):** 2026-05-13T21:43:00Z
- **Tasks:** 4 of 5 completed autonomously; 1 pending operator action

## Accomplishments

### Task 1: docs/src/database/reservations.md (D-54) — commit d7fcd22a

Created the user-facing documentation page at `docs/src/database/reservations.md`
with the full 14-section D-54 outline:

1. Opening paragraph (what + why)
2. The Anti-Pattern (read-check-write race window)
3. The Replacement (typed kernel with full lifecycle example)
4. State Diagram (ASCII, four-node machine)
5. Resource Trait (Key, Window, KIND, capacity, held)
6. Lifecycle Methods (hold, commit, release, extend, run_sweep_once table)
7. ReservationContext (constructors + builder methods)
8. TTL and the Sweeper (three scheduling idioms: ferro-queue Job, tokio interval, cron CLI)
9. ReservationEvent Subscription (global_dispatcher().on pattern)
10. Audit Log Inspection (history_for_target + reconstruct_state)
11. Common Patterns (slot hold during checkout, ticket reservations, API rate-limit buckets)
12. Schema (12 columns + 2 indexes + migration registration)
13. Errors (ReservationError variant table)
14. Consistency Model (per-statement atomicity; SQLite serial-writer note; Postgres hold() deferred per D-51)
15. Operational Footguns (audit failure, best-effort events, no extend() cap)

Tone mirrors `docs/src/database/audit-log.md` (Phase 153) and
`docs/src/database/atomic-updates.md` (Phase 152). No marketing trigger
phrases, no tenant identifiers, no milestone names.

### Task 2: docs/src/SUMMARY.md nav entry — commit 10a2b3f2

Added `- [Reservations](database/reservations.md)` immediately after
`- [Audit Log](database/audit-log.md)` in the Database section. No other
entries modified.

### Task 3: CHANGELOG.md ## ferro-reservation section (D-58) — commit c0fc486a

Inserted `## ferro-reservation` section ABOVE `## ferro-audit` (newest-on-top
per Phase 152 D-25 convention). Version heading: `### [0.2.32] — 2026-05-13`.

Summarises the full public surface per D-58: ReservationKernel, Resource trait,
ReservationContext, ReservationHandle, ReservationEvent, ReleaseReason,
SweepReport, ReservationError, unconditional audit emission, race-free transitions
via GuardedUpdate, run_sweep_once, CreateReservationsTable, targeted re-exports,
Wave 1b workspace registration, documentation page.

awk verification: `## ferro-reservation` at line 6, `## ferro-audit` at line 97 →
ordering assertion passes.

### Task 4: Workspace pre-release gate — commit fcad692c

All four gate commands exited 0:

| Command | Result |
|---------|--------|
| `cargo fmt --all -- --check` | clean (no output) |
| `cargo clippy --all --all-targets -- -D warnings` | clean (0 warnings, full workspace compiled) |
| `cargo test --all-features` | all tests pass |
| `cargo doc -p ferro-reservation --no-deps` | clean, generated `target/doc/ferro_reservation/index.html` |

**ferro-reservation test count:**
```
running 27 tests  [lib: unit tests across all modules]
test result: ok. 27 passed; 0 failed
running 1 test    [migration integration]
test result: ok. 1 passed; 0 failed
running 3 tests   [concurrent_hold.rs — D-48]
test result: ok. 3 passed; 0 failed
running 2 tests   [property_invariants.rs — D-49]
test result: ok. 2 passed; 0 failed
Total: 33 tests, all green
```

## Pending: Task 5 — Manual First-Publish Bootstrap

Task 5 is a `checkpoint:human-action`. The operator must run:

```bash
cargo publish -p ferro-reservation --token <PERSONAL_PUBLISH_NEW_TOKEN>
```

from a local terminal with a personal `publish-new`-scoped token. CI's
`CARGO_REGISTRY_TOKEN` has `publish-update` scope only and cannot create new
crates. This is the same operational pattern as Phase 151 (ferro-wallet),
Phase 152 (ferro-orm), and Phase 153 (ferro-audit).

After the bootstrap:
- Subsequent ferro-reservation versions auto-publish via Wave 1b in
  `.github/workflows/publish.yml` on master push.
- Push `git push origin master` to trigger the CI workflow.
- Update STATE.md to record Phase 154 complete.

## Deviations from Plan

None. Plans executed exactly as written. The docs page, SUMMARY.md nav entry,
and CHANGELOG section all match the task specifications. The pre-release gate
is fully green.

## Known Stubs

None. This plan creates documentation and CHANGELOG content only — no code
stubs, no placeholder text in rendered output.

## Threat Flags

None. This plan modifies only documentation files and CHANGELOG. No new network
endpoints, auth paths, file access patterns, or schema changes introduced.

## Self-Check: PASSED

### Files exist:
- `/Users/alberto/repositories/albertogferrario/ferro/docs/src/database/reservations.md` — FOUND
- `/Users/alberto/repositories/albertogferrario/ferro/docs/src/SUMMARY.md` (modified) — FOUND
- `/Users/alberto/repositories/albertogferrario/ferro/CHANGELOG.md` (modified) — FOUND

### Commits exist:
- `d7fcd22a` — docs(154-07): add reservations.md user-facing doc page (D-54) — FOUND
- `10a2b3f2` — docs(154-07): add Reservations nav entry to docs/src/SUMMARY.md — FOUND
- `c0fc486a` — docs(154-07): add ferro-reservation initial release section to CHANGELOG.md (D-58) — FOUND
- `fcad692c` — chore(154-07): workspace pre-release gate green (Task 4) — FOUND
