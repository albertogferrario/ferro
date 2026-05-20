---
phase: 177
plan: "03"
subsystem: docs
tags: [documentation, concurrency, reservations, atomicity]
dependency_graph:
  requires: [177-01, 177-02]
  provides: [accurate-reservation-docs]
  affects: [docs/src/database/reservations.md]
tech_stack:
  added: []
  patterns: []
key_files:
  created: []
  modified:
    - docs/src/database/reservations.md
decisions:
  - "Phrase 'atomically arbitrates' added to replacement 1 to satisfy success criteria grep; the plan's prescribed text conveyed the same meaning but used 'serialized at the database level' which does not match the required grep pattern"
  - "No feature name substitution needed: plan's action text did not reference a feature by name in the doc prose"
metrics:
  duration: "~5 minutes"
  completed: "2026-05-20"
  tasks_completed: 1
  files_modified: 1
---

# Phase 177 Plan 03: Documentation Correction Sweep — Summary

`docs/src/database/reservations.md` no longer recommends a `tokio::Mutex` per resource key for SQLite concurrency, no longer marks Postgres correctness as a roadmap item, and now accurately describes the Phase 177 transaction-based atomicity fix: a `SERIALIZABLE` transaction wrapping the capacity check, INSERT, and audit write, with SQLSTATE `40001` translated to `ReservationError::Insufficient` at the kernel boundary.

## What Was Built

### Task 1: Two surgical replacements in `docs/src/database/reservations.md`

**Commit:** `bf3c4914`

**Changed line ranges (from `git diff`):**

**Replacement 1 — `hold` sequence note (old lines 145-148, new lines 145-149):**

```diff
-Note: the capacity check and the INSERT are two separate statements, not a
-single atomic SQL operation. Under SQLite's serial-writer semantics concurrent
-tasks should serialize `hold` calls at the application layer (e.g., a
-`tokio::Mutex` per resource key). See the Consistency Model section.
+The capacity check, the INSERT, and the audit write all execute inside a
+single `SERIALIZABLE` transaction. The kernel atomically arbitrates concurrent
+holds at the database level — no application-layer mutex is required. The
+conflict-losing task receives `ReservationError::Insufficient`. See the
+Consistency Model section.
```

**Replacement 2 — Consistency Model section body (old lines 370-382, new lines 371-384):**

```diff
-**`hold` on SQLite:** SQLite WAL mode serializes writers at the file level, but
-the three-step hold sequence (capacity SELECT + held SELECT + INSERT) is not a
-single statement. Under concurrent tokio tasks connecting to the same SQLite
-database, the capacity check and the INSERT can interleave. Consumers running
-concurrent holds against SQLite should serialize `hold` calls at the application
-layer — a `tokio::sync::Mutex` per resource key is the idiomatic pattern.
-
-**`hold` on Postgres:** Under Postgres `READ COMMITTED`, the capacity check has
-a theoretical race window between the SELECT and the INSERT. The current crate is
-SQLite-validated; Postgres correctness for the capacity check is on the roadmap
-as a follow-up addition (`SELECT FOR UPDATE` or a counter column approach).
+**`hold`:** The capacity check, INSERT, and audit write execute inside a
+`SERIALIZABLE` transaction (`sea_orm::IsolationLevel::Serializable`). On SQLite
+the transaction aligns with the WAL single-writer model; on Postgres it prevents
+phantom reads between the SELECT and INSERT. If two concurrent tasks race on the
+same `(key, window)`, the database serializes them — exactly one succeeds and the
+other receives `ReservationError::Insufficient`. No application-layer mutex is
+needed.
+
+A conflict-losing task on Postgres may receive SQLSTATE `40001` (serialization
+failure); the kernel translates this to `ReservationError::Insufficient` before
+returning to the caller. The error contract is uniform across backends.
+
 `commit`, `release`, and `extend` via `GuardedUpdate` are race-free on both
-dialects.
+dialects (single `UPDATE … WHERE` statement).
```

Net delta: 1 file, +18 lines, -16 lines (small, targeted).

## Verification Results

All ten plan-specified checks satisfied:

| Check | Expected | Actual | Pass? |
|-------|----------|--------|-------|
| `grep -c -E 'tokio::(sync::)?Mutex'` | 0 | 0 | ✓ |
| `grep -c 'on the roadmap as a follow-up'` | 0 | 0 | ✓ |
| `grep -c 'SQLite-validated'` | 0 | 0 | ✓ |
| `grep -c 'SERIALIZABLE'` | ≥1 | 2 | ✓ |
| `grep -c '40001'` | ≥1 | 1 | ✓ |
| `grep -c 'sea_orm::IsolationLevel::Serializable'` | ≥1 | 1 | ✓ |
| `grep -c 'ReservationError::Insufficient'` | ≥2 | 3 | ✓ |
| `grep -c '^## Consistency Model'` | 1 | 1 | ✓ |
| `grep -c '^## The Anti-Pattern'` | 1 | 1 | ✓ |
| `grep -c '^## Operational Footguns'` | 1 | 1 | ✓ |

Prompt success criteria:
| Check | Expected | Actual | Pass? |
|-------|----------|--------|-------|
| `grep -c 'tokio::Mutex'` | 0 | 0 | ✓ |
| `grep -ciE 'serializable\|begin_with_config\|IsolationLevel'` | ≥1 | 3 | ✓ |
| `grep -ciE '40001\|SQLSTATE'` | ≥1 | 2 | ✓ |
| `grep -c 'atomically arbitrates'` | ≥1 | 1 | ✓ |
| `grep -ci 'no application-layer mutex'` | ≥2 | 2 | ✓ |

Regression checks:
- `cargo fmt --all -- --check` → exit 0 (no code touched)
- `cargo test -p ferro-reservation` → exit 0, 36 tests pass

## Line Offset Drift

RESEARCH.md Q8 cited the first passage at lines 145-148 and the second at lines 363-382. The actual current positions were identical for the first passage (145-148) and slightly shifted for the second (370-382 vs 363-382). The string-anchor replacement strategy (matching on exact text content, not line numbers) handled the offset correctly without any manual adjustment.

## Deviations from Plan

### Auto-adjusted Wording

**1. [Rule 2 - Missing] Added 'atomically arbitrates' phrase to replacement 1**
- **Found during:** Post-edit verification against prompt success criteria
- **Issue:** The plan's prescribed replacement 1 text ("Concurrent callers on the same `(key, window)` are serialized at the database level") conveyed the correct meaning but did not satisfy the prompt's success criterion grep for `atomically arbitrates` / `kernel arbitrates concurrent holds`. The plan's acceptance criteria and the prompt's success criteria were slightly inconsistent on this point.
- **Fix:** Rewrote the second sentence of replacement 1 to "The kernel atomically arbitrates concurrent holds at the database level — no application-layer mutex is required." This satisfies both the plan's semantic intent and the prompt's grep gate.
- **Files modified:** `docs/src/database/reservations.md`
- **Commit:** `bf3c4914`

### Naming Substitution (adapted_naming_alert)

No feature names appear in the doc prose for these two passages. The adapted_naming_alert in the plan prompt (correcting `postgres` → `sqlx-postgres`) did not apply to either replacement — neither passage referenced feature flags. No substitution needed.

## Known Stubs

None. The documentation now accurately describes shipped behavior.

## Threat Flags

No new network endpoints, auth paths, or security-relevant surfaces introduced. Documentation-only change.

## Self-Check: PASSED

- `docs/src/database/reservations.md` — modified, confirmed present
- Commit `bf3c4914` — exists in git log
- `cargo test -p ferro-reservation` — 36 tests pass, 0 failed
- `cargo fmt --all -- --check` — exit 0
- All ten plan verification greps and all five prompt success-criteria greps satisfied
