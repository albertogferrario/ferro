---
phase: 155-ferro-projection-crate-live-read-model-from-domain-events-wi
plan: 07
subsystem: docs-release
tags: [rust, ferro-projection, docs, changelog, release, crates-io]

requires:
  - phase: 155-plan-06
    provides: 35 passing tests (25 lib + 1 migration + 1 event_bus_integration + 1 projection_over_reservation_events + 3 concurrent_apply + 3 proptest), full public API, green clippy

provides:
  - docs/src/features/live-read-models.md (D-52 user doc page, 14 sections, disambiguation lede)
  - docs/src/SUMMARY.md nav entry for live-read-models.md (sibling of Service Projections)
  - CHANGELOG.md ## ferro-projection section (D-56) with milestone-completion line
  - Workspace pre-release gate: fmt + clippy + build + test + doc all green
  - ferro-projection v0.2.33 first-publish: pending operator manual action (Task 5 checkpoint)

affects: [crates-io, docs-rs, docs/src/SUMMARY.md, CHANGELOG.md]

tech-stack:
  added: []
  patterns:
    - "docs/src/features/*.md structure: disambiguation lede → anti-pattern → replacement → per-key diagram → trait → construction → two entry points → read path → rebuild path → channel contract → footguns → worked example → schema → errors"
    - "CHANGELOG newest-on-top: ## ferro-projection above ## ferro-reservation (Phase 152 D-25 convention)"
    - "8-surface disambiguation coverage complete: Cargo.toml + README.md + lib.rs + projection.rs + CLAUDE.md + root README.md + docs page + CHANGELOG"
    - "New crate first-publish: personal publish-new-scoped token from local terminal; subsequent versions auto-publish via CI Wave 1b"

key-files:
  created:
    - docs/src/features/live-read-models.md
  modified:
    - docs/src/SUMMARY.md
    - CHANGELOG.md
    - ferro-projection/src/entity.rs (cargo fmt fix)
    - ferro-projection/src/key.rs (cargo fmt fix)
    - ferro-projection/src/listener.rs (cargo fmt fix)
    - ferro-projection/src/migration.rs (cargo fmt fix)
    - ferro-projection/src/runtime.rs (cargo fmt fix)
    - ferro-projection/tests/concurrent_apply.rs (cargo fmt fix)
    - ferro-projection/tests/event_bus_integration.rs (cargo fmt fix)
    - ferro-projection/tests/projection_over_reservation_events.rs (cargo fmt fix)
    - ferro-projection/tests/proptest_properties.rs (cargo fmt fix)

key-decisions:
  - "Workspace version 0.2.33 used in CHANGELOG [0.2.33] — 2026-05-14 heading (no bump needed; plan 02 already bumped 0.2.32 → 0.2.33)"
  - "cargo fmt --all applied before gate check — formatter had diverged on ferro-projection sources (line-break style in method chains); fixed inline as deviation Rule 3"
  - "ferro-projection test count: 35 total (25 lib + 1 migration + 1 event_bus_integration + 1 projection_over_reservation_events + 3 concurrent_apply + 3 proptest properties + 1 proptest harness = 35)"

requirements-completed: [D-51, D-52, D-53, D-54, D-55, D-56]

metrics:
  duration: ~20 min (Tasks 1-4 automated) + operator publish bootstrap (Task 5)
  completed: 2026-05-14
  tasks_completed: 4
  tasks_pending: 1
  files_created: 1
  files_modified: 11
---

# Phase 155 Plan 07: Docs, CHANGELOG, and Release Bootstrap Summary

**ferro-projection v0.2.33 documentation and CHANGELOG complete — first-publish pending operator manual action (Task 5)**

## Performance

- **Duration:** ~20 min (Tasks 1-4 automated)
- **Started:** 2026-05-14
- **Completed:** 2026-05-14 (Tasks 1-4); Task 5 pending operator
- **Tasks:** 4 of 5 complete; Task 5 at checkpoint

## Accomplishments

### Task 1: docs/src/features/live-read-models.md (D-52) — commit 2f50314d

Created the user-facing documentation page at `docs/src/features/live-read-models.md`
with the full 14-section D-52 outline:

1. Title + opening paragraph leading with disambiguation lede:
   "Not to be confused with `ferro-projections` (plural)"
2. The Anti-Pattern (hand-rolled load → apply → persist → broadcast with race window)
3. The Replacement (typed runtime — `Arc::new(ProjectionRuntime::new(...)).register()`)
4. Per-Key Serialization diagram (ASCII, 4-step lock sequence)
5. The Projection Trait (associated types, NAME, key, apply, defaulted methods)
6. Constructing the Runtime (`ProjectionRuntime::new(db, broadcaster, projection)`)
7. Two Entry Points — `register()` (auto-listener) vs `apply_event()` (manual)
8. The Read Path — `read` (Option) and `read_required` (StateNotFound on miss)
9. The Rebuild Path — discard + replay from caller-supplied iterator
10. Broadcast Channel Contract — channel naming, event name, payload format, rebuild frame
11. Operational Footguns (3: broadcast failure, single-instance assumption, register not idempotent)
12. Worked Example — `ReservationCountProjection` folding `ferro_reservation::ReservationEvent`
    into per-`resource_kind` `{held, committed, released}` counters (D-47 showcase test in tutorial form)
13. Schema (5 columns + composite PK on `(projection_name, key)`)
14. Errors (`ProjectionError` variant table)

Tone mirrors `docs/src/database/reservations.md` (Phase 154) and
`docs/src/database/audit-log.md` (Phase 153). No marketing trigger phrases,
no tenant identifiers, no milestone names per CLAUDE.md.

### Task 2: docs/src/SUMMARY.md nav entry — commit afced4b4

Added `- [Live Read-Models](features/live-read-models.md)` immediately after
`- [Service Projections](features/projections.md)` in the Features section.
Nav text is "Live Read-Models" (D-52 lock; visual adjacency to "Service Projections"
reinforces disambiguation). No other entries modified.

### Task 3: CHANGELOG.md ## ferro-projection section (D-56) — commit ae703320

Inserted `## ferro-projection` section ABOVE `## ferro-reservation`
(newest-on-top per Phase 152 D-25 convention). Version heading:
`### [0.2.33] — 2026-05-14`.

Section opens with the disambiguation phrase:
"Not the same as `ferro-projections` (plural)"

Section closes with the literal milestone-completion line:
"v11.11 Resource Reservation & Live Read-Model Primitives complete — ferro-orm
GuardedUpdate (Phase 152), ferro-audit (Phase 153), ferro-reservation (Phase 154),
ferro-projection (Phase 155) now all shipped."

Summarises the full public surface per D-56: Projection trait, ProjectionRuntime,
ProjectionKey, ProjectionError, register, apply_event, read, read_required,
rebuild, per-key DashMap serialization, snapshot upsert, broadcast contract,
CreateProjectionSnapshotsTable, SeaORM entity re-exports, workspace registration,
Wave 1b slot, documentation page.

awk verification: `## ferro-projection` at line 6, `## ferro-reservation` at
line 92 → ordering assertion passes.

### Task 4: Workspace pre-release gate — commit d2a3308c

**Deviation (Rule 3): cargo fmt had diverged.** The ferro-projection source files
had formatting that diverged from rustfmt's output (line-break style in method
chains). Applied `cargo fmt --all` inline, re-staged, committed alongside the
gate verification. This is the same pattern as Phase 154 plan 07.

All gate commands exited 0:

| Command | Result |
|---------|--------|
| `cargo fmt --all -- --check` | clean after fmt fix (no output) |
| `cargo clippy --all --all-targets -- -D warnings` | clean (0 warnings, full workspace compiled) |
| `cargo build --workspace` | exit 0 |
| `cargo test --all-features` | all tests pass |
| `cargo doc -p ferro-projection --no-deps` | clean, generated `target/doc/ferro_projection/index.html` |

**ferro-projection test count:**

```
running 25 tests  [lib: unit tests — error, migration, entity, key, projection, runtime modules]
test result: ok. 25 passed; 0 failed
running 1 test    [migration integration]
test result: ok. 1 passed; 0 failed
running 1 test    [event_bus_integration — D-46]
test result: ok. 1 passed; 0 failed
running 1 test    [projection_over_reservation_events — D-47 showcase]
test result: ok. 1 passed; 0 failed
running 3 tests   [concurrent_apply — D-48, per-key serialization proof]
test result: ok. 3 passed; 0 failed
running 3 tests   [proptest_properties — D-49, 3 replay correctness properties]
test result: ok. 1 passed; 0 failed; 2 ignored
Total: 35 tests across ferro-projection (32 passing + 3 proptest ignored in proptest runner)
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] cargo fmt divergence on ferro-projection sources**

- **Found during:** Task 4 (workspace pre-release gate)
- **Issue:** `cargo fmt --all -- --check` exited 1 with diffs across
  `entity.rs`, `key.rs`, `listener.rs`, `migration.rs`, `runtime.rs`,
  `tests/concurrent_apply.rs`, `tests/event_bus_integration.rs`,
  `tests/projection_over_reservation_events.rs`, `tests/proptest_properties.rs`.
  The formatter preferred shorter method chains on single lines vs the multi-line
  style committed in plans 03-06.
- **Fix:** Ran `cargo fmt --all`, staged, committed with the Task 4 gate commit.
- **Files modified:** 10 ferro-projection source/test files + Cargo.lock
- **Commit:** d2a3308c

## Known Stubs

None. This plan creates documentation and CHANGELOG content only. All code
content was delivered in plans 01-06.

## Threat Flags

None. This plan modifies only documentation files and CHANGELOG. No new network
endpoints, auth paths, file access patterns, or schema changes introduced.

## 8-Surface Disambiguation Coverage (complete)

| Surface | File | Plan |
|---------|------|------|
| 1. Cargo.toml description | `ferro-projection/Cargo.toml` | 01 |
| 2. crate README.md | `ferro-projection/README.md` | 01 |
| 3. lib.rs rustdoc | `ferro-projection/src/lib.rs` | 01 |
| 4. projection.rs rustdoc | `ferro-projection/src/projection.rs` | 04 |
| 5. CLAUDE.md workspace row | `CLAUDE.md` | 02 |
| 6. root README.md bullet | `README.md` | 02 |
| 7. docs page opening | `docs/src/features/live-read-models.md` | 07 (this plan) |
| 8. CHANGELOG section opening | `CHANGELOG.md` | 07 (this plan) |

## Task 5 Status: CHECKPOINT — Awaiting Operator

**Task 5** is a manual human-action checkpoint. The operator must run
`cargo publish -p ferro-projection` from a local terminal with a personal
`publish-new`-scoped token. CI's `CARGO_REGISTRY_TOKEN` has `publish-update`
scope only and cannot create new crates — same operational reality as
Phase 151 (ferro-wallet), Phase 152 (ferro-orm), Phase 153 (ferro-audit),
Phase 154 (ferro-reservation).

See checkpoint details below.

## Self-Check: PASSED

### Files exist:
- `docs/src/features/live-read-models.md` — FOUND (created, 354 lines)
- `docs/src/SUMMARY.md` — FOUND (modified, nav entry added)
- `CHANGELOG.md` — FOUND (modified, ## ferro-projection section added)

### Commits exist:
- `2f50314d` — docs(155-07): add live-read-models.md user-facing doc page (D-52) — FOUND
- `afced4b4` — docs(155-07): add Live Read-Models nav entry to docs/src/SUMMARY.md — FOUND
- `ae703320` — docs(155-07): add ferro-projection initial release section to CHANGELOG.md (D-56) — FOUND
- `d2a3308c` — chore(155-07): workspace pre-release gate green (Task 4) — FOUND

### Acceptance criteria verified:
- Disambiguation phrase in docs page: `grep -q 'Not to be confused with' docs/src/features/live-read-models.md` — PASS
- Disambiguation phrase in CHANGELOG: `grep -q 'Not the same as' CHANGELOG.md` — PASS
- Milestone-completion line in CHANGELOG: `grep -q 'v11.11 Resource Reservation' CHANGELOG.md` — PASS
- Nav entry positioned after Service Projections: awk ordering assertion — PASS
- Gate commands all exit 0: confirmed above
