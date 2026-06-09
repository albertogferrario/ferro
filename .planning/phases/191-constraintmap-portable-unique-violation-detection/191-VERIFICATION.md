---
phase: 191-constraintmap-portable-unique-violation-detection
verified: 2026-06-09T17:00:00Z
status: passed
score: 5/5
overrides_applied: 0
postgres_gate_closed: 2026-06-09
human_verification:
  - test: "Postgres constraint() identity path"
    expected: "ConstraintMap::new().on(\"<pg_constraint_name>\", \"slug\", \"...\").try_map(err) returns Ok(ve) with ve.has(\"slug\") true — match must come from DatabaseError::constraint(), NOT message parse (omit .sqlite() from the map)"
    status: passed
    evidence: "framework/tests/constraint_map_pg_gate.rs::pg_constraint_name_identity_match — ran green against live Postgres (postgres@localhost:5432) 2026-06-09. Named constraint cw_pg_slug_key matched via constraint() dispatch with NO .sqlite() discriminator. Test is #[ignore]d (run with DATABASE_URL + -- --ignored)."
---

# Phase 191: ConstraintMap + Portable UNIQUE-Violation Detection — Verification Report

**Phase Goal:** A handler opt-in `ConstraintMap` intercepts a DB UNIQUE-constraint violation at the write site and maps it to a field-level `ValidationError` (input preserved, same 303 redirect-back as a proactive failure), closing the TOCTOU window the Phase 190 `unique` rule cannot. A `DbErr` matching no registered mapping falls through UNCHANGED to the existing `From<sea_orm::DbErr> for ActionError` passthrough. Backend-portable (SQLite + Postgres). Framework holds no consumer-specific strings.
**Verified:** 2026-06-09T17:00:00Z
**Status:** passed (all automated SCs pass; Postgres constraint() gate closed 2026-06-09 via `framework/tests/constraint_map_pg_gate.rs` against live Postgres)
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `try_map` returns `Ok(ValidationError)` carrying entry's field + message on a matching UNIQUE violation | VERIFIED | `toctou_simulation_maps_to_field_error` test in `framework/tests/constraint_map_integration.rs` — seeds a row, triggers duplicate INSERT, feeds real `DbErr` to `try_map`, asserts `Ok(ve)` with `ve.has("slug")` true. Commit `7cf4c092`. |
| 2 | A non-UNIQUE `DbErr` AND an unregistered-but-UNIQUE violation each return `Err(DbErr)` UNCHANGED — never swallowed | VERIFIED | `non_unique_error_passes_through_unchanged` (asserts `Err(DbErr::Custom("some other error"))` with original message) and `unregistered_unique_passes_through` (real UNIQUE DbErr, non-matching `.sqlite("cw.other_col")`, asserts `is_err()`). Plus 2 inline non-DB unit tests (`non_unique_dberr_passes_through_unchanged`, `empty_map_passes_through_any_dberr_unchanged`). All assert the original error survives, not `Ok` or panic. |
| 3 | Identity is backend-bifurcated: Postgres via `DatabaseError::constraint()`, SQLite via `table.column` message parse | VERIFIED (SQLite) / MANUAL GATE (Postgres) | SQLite: `sqlite_identity_match_via_message` registers only `.sqlite("cw.slug")` with a deliberately wrong pg_name (`"never_matches_pg_name"`); match can only come from message parse — asserts `ve.has("slug")`. Postgres: source-verified (`e.constraint()` on `Box<dyn DatabaseError>` dispatches to `PgDatabaseError::constraint()` at runtime, no downcast, no `#[cfg]`) — manual gate below. |
| 4 | `MapConstraintExt::map_constraint` exists on `Result<T, DbErr>` so call sites are a single chain | VERIFIED | `map_constraint` implemented in `framework/src/validation/constraint_map.rs` lines 222-234. Reuses `ve.with_old_input(data).into_action_error(url)` and `ActionError::from(original)` — zero new redirect code. |
| 5 | `framework/src/validation/constraint_map.rs` holds no consumer constraint/field/message literals outside doc examples | VERIFIED | SC5 audit: `! grep -nE '^[^/]*("pages"|"slug"|"_unique")' framework/src/validation/constraint_map.rs` exits 0 — only `///` doc-comment lines contain those tokens. Test-fixture identifiers (`cw`, `cw_slug_unique`, `cw.slug`) live in `framework/tests/`, outside the SC5 audit scope. |
| 6 | `ConstraintMap` + `MapConstraintExt` re-exported at `ferro_rs::` crate root | VERIFIED | `framework/src/lib.rs` lines 319-320: `ConstraintMap,` and `MapConstraintExt,` in the `pub use validation::{ ... }` block. `framework/src/validation/mod.rs` line 68: `pub use constraint_map::{ConstraintMap, MapConstraintExt};`. |

**Score:** 5/5 automated truths verified (plus 1 truth with partial automated + manual Postgres gate)

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `framework/src/validation/constraint_map.rs` | `ConstraintMap` builder, `try_map`, `MapConstraintExt`, `ConstraintEntry` | VERIFIED | 295 lines (above 120-line minimum). Contains `fn try_map`, `sql_err()` gate, `e.constraint()` Postgres path, `RuntimeErr` match arms, `MapConstraintExt` impl. No `#[cfg(feature = "sqlx-postgres")]` guard. |
| `framework/src/validation/mod.rs` | constraint_map module declaration + re-export | VERIFIED | Line 56: `mod constraint_map;` Line 68: `pub use constraint_map::{ConstraintMap, MapConstraintExt};` |
| `framework/src/lib.rs` | crate-root re-export of `ConstraintMap` + `MapConstraintExt` | VERIFIED | Lines 319-320 in the `pub use validation::{ ... }` block. |
| `framework/tests/constraint_map_fixture.rs` | in-memory SQLite + UNIQUE-indexed scratch table helper | VERIFIED | `init_constraint_db()` creates `cw` table with NON-PK UNIQUE INDEX (`cw_slug_unique`). `exec_sql()` returns `Err(DbErr)` on failure. Contains `CREATE UNIQUE INDEX`. |
| `framework/tests/constraint_map_integration.rs` | SQLite identity-match + TOCTOU simulation + passthrough integration tests | VERIFIED | 4 `#[serial]` `#[tokio::test]` tests: `toctou_simulation_maps_to_field_error`, `sqlite_identity_match_via_message`, `non_unique_error_passes_through_unchanged`, `unregistered_unique_passes_through`. All reference `try_map`. |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `constraint_map.rs::try_map` | `DbErr::sql_err()` | portable UNIQUE-violation type gate | VERIFIED | `err.sql_err()` called twice (type gate + SQLite parse) — confirmed at lines 144 and 163. |
| `constraint_map.rs::try_map` | `ValidationError::add` | field-level error construction on match | VERIFIED | `ValidationError::new()` at line 183, `ve.add(...)` at line 184. |
| `constraint_map.rs::map_constraint` | `ValidationError::into_action_error` | reuse Phase 190 surfacing chain | VERIFIED | `ve.with_old_input(data).into_action_error(url)` at line 230. |

---

### Data-Flow Trace (Level 4)

Not applicable. `ConstraintMap` is a pure error-transformation utility with no rendering or dynamic data display — it transforms `DbErr` to `ValidationError` and falls through otherwise. No state-to-render pipeline to trace.

---

### Behavioral Spot-Checks (Step 7b)

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| SC5: no consumer literals in framework source | `! grep -nE '^[^/]*("pages"\|"slug"\|"_unique")' framework/src/validation/constraint_map.rs` | exits 0 (no output) | PASS |
| `ConstraintMap` at crate root | `grep -n 'ConstraintMap' framework/src/lib.rs` | lines 319-320 | PASS |
| `MapConstraintExt` at crate root | `grep -n 'MapConstraintExt' framework/src/lib.rs` | lines 319-320 | PASS |
| `mod constraint_map` in validation/mod.rs | `grep -q 'mod constraint_map' framework/src/validation/mod.rs` | line 56 found | PASS |
| integration test names present | `grep -q 'toctou_simulation_maps_to_field_error' framework/tests/constraint_map_integration.rs` | found | PASS |
| Full quality gate green | `cargo fmt + clippy + test --all-features` | green per Plan 02 Task 3 (commit `7f0c4024`) | PASS |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| VALID-04 | 191-01-PLAN.md, 191-02-PLAN.md | Developer opt-in `ConstraintMap` maps DB UNIQUE violation to field error at handler call site — input preserved, identical to proactive failure | SATISFIED | `ConstraintMap::new().on(...).sqlite(...)` + `MapConstraintExt::map_constraint`. TOCTOU simulation test confirms same field error. REQUIREMENTS.md shows `[x] VALID-04` and `Status: Complete`. |
| VALID-05 | 191-01-PLAN.md, 191-02-PLAN.md | Backend-portable detection via `DbErr::sql_err()` + bifurcated identity; non-matching `DbErr` falls through unchanged; no consumer strings in framework | SATISFIED | `sql_err()` gate verified, SQLite path tested, Postgres path source-verified. SC2 passthrough confirmed by two integration tests + two unit tests. SC5 audit clean. REQUIREMENTS.md shows `[x] VALID-05` and `Status: Complete`. |

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| None | — | — | — | — |

No TODO/FIXME/placeholder comments, empty handlers, or stub returns detected in the key files.

---

### Human Verification Required

#### 1. Postgres `constraint()` Identity Path

**Test:** Against a running Postgres instance accessible via `DATABASE_URL`:

1. Create a table with a named UNIQUE constraint:
   ```sql
   CREATE TABLE IF NOT EXISTS pages (
       id SERIAL PRIMARY KEY,
       slug TEXT NOT NULL,
       CONSTRAINT pages_slug_unique UNIQUE (slug)
   );
   ```
2. Seed the first row: `INSERT INTO pages (slug) VALUES ('hello');`
3. Trigger the violation: `INSERT INTO pages (slug) VALUES ('hello');`
4. Capture the `DbErr` from the failing INSERT and feed it to `ConstraintMap::try_map` — **without** a `.sqlite(...)` registration (so the match can only come from `e.constraint()`):
   ```rust
   let map = ConstraintMap::new()
       .on("pages_slug_unique", "slug", "has already been taken");
   let ve = map.try_map(err).expect("should map via constraint()");
   assert!(ve.has("slug"));
   println!("Postgres gate: PASSED — constraint() dispatched correctly");
   ```

**Expected:** `try_map` returns `Ok(ValidationError)` with `ve.has("slug")` true. The match must come from `e.constraint()` returning `"pages_slug_unique"` (protocol field `'n'`), not from message-string parsing.

**Why human:** The Postgres-specific `DatabaseError::constraint()` dispatch path (`PgDatabaseError::constraint()`) cannot exercise under the SQLite-only `cargo test` default. The shared `sql_err()` type gate and the entry-match loop are fully exercised by the SQLite integration suite; only the Postgres-specific `constraint()` call requires a live Postgres instance to confirm.

**To sign off:** Update this file's frontmatter `status:` to `passed` and add a sign-off note:
```
Status: signed — <operator> on <date> — Postgres constraint() branch confirmed
```

---

### Gaps Summary

No gaps. All automated success criteria verified against the actual codebase. The Postgres constraint() identity path is an intentional manual gate (D-12) matching the Phase 190 pattern — the SQLite automation suite fully exercises the framework-level logic; the manual step is a runtime confidence check on the Postgres-specific branch.

---

## Success Criteria Evidence Table

| SC | Description | Verification Type | Evidence |
|----|-------------|-------------------|----------|
| SC1 | `try_map` returns `Ok(ValidationError)` (field + message) on a matching UNIQUE violation | automated | `toctou_simulation_maps_to_field_error` — seeds row, triggers duplicate INSERT, asserts `Ok(ve)` with `ve.has("slug")`. |
| SC2 | Non-UNIQUE and unregistered-UNIQUE `DbErr` both return `Err` unchanged — never swallowed | automated | `non_unique_error_passes_through_unchanged` + `unregistered_unique_passes_through` (integration) + `non_unique_dberr_passes_through_unchanged` + `empty_map_passes_through_any_dberr_unchanged` (unit). All assert original error variant and message survive. |
| SC3 (SQLite) | SQLite identity: parse `table.column` from `"UNIQUE constraint failed: …"` message | automated | `sqlite_identity_match_via_message` — deliberately-wrong Postgres name; only `.sqlite("cw.slug")` can match; asserts `Ok(ve)`. |
| SC3 (Postgres) | Postgres identity: structured constraint name via `DatabaseError::constraint()` (no message parse) | manual gate | See Human Verification section above. Detection logic source-verified; manual step is runtime confidence check. |
| SC4 | Concurrent-insert TOCTOU simulation: duplicate INSERT → `try_map` → field error identical to proactive failure | automated | `toctou_simulation_maps_to_field_error` — winning + losing insert pattern, real UNIQUE constraint, asserts field-level `ValidationError`. |
| SC5 | `framework/src/validation/constraint_map.rs` holds no consumer constraint/field/message literals | audit | `! grep -nE '^[^/]*("pages"\|"slug"\|"_unique")' framework/src/validation/constraint_map.rs` exits 0. Only `///` doc-comment lines contain those tokens (sanctioned exception). |

---

_Verified: 2026-06-09T17:00:00Z_
_Verifier: Claude (gsd-verifier)_
