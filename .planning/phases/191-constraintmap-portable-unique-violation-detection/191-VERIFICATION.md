---
phase: 191-constraintmap-portable-unique-violation-detection
status: pending manual run
created: 2026-06-09
---

# Phase 191: Verification — ConstraintMap + Portable UNIQUE-Violation Detection

## Success Criteria Evidence Table

| SC | Description | Verification Type | Evidence |
|----|-------------|-------------------|----------|
| SC1 | `try_map` returns `Ok(ValidationError)` (field + message) on a matching UNIQUE violation | automated | `toctou_simulation_maps_to_field_error` test in `framework/tests/constraint_map_integration.rs` — seeds a row, triggers a duplicate INSERT, feeds the real `DbErr` to `try_map`, asserts `Ok(ve)` with `ve.has("slug")`. |
| SC2 | Non-UNIQUE and unregistered-UNIQUE `DbErr` both return `Err` unchanged — never swallowed | automated | `non_unique_error_passes_through_unchanged` (custom DbErr), `unregistered_unique_passes_through` (real UNIQUE violation, non-matching entry), and four inline unit tests in `constraint_map.rs` (`non_unique_dberr_passes_through_unchanged`, `empty_map_passes_through_any_dberr_unchanged`). All assert the original error variant and message survive. |
| SC3 (SQLite) | SQLite identity: parse `table.column` from `"UNIQUE constraint failed: …"` message | automated | `sqlite_identity_match_via_message` test — registers a deliberately-wrong Postgres name so only the `.sqlite("cw.slug")` discriminator can match; asserts `Ok(ve)`. |
| SC3 (Postgres) | Postgres identity: structured constraint name via `DatabaseError::constraint()` (no message parse) | manual gate | See **Postgres Manual Verification Gate** section below. The detection logic is source-verified; the manual step is a confidence check on the runtime Postgres-specific `constraint()` dispatch. |
| SC4 | Concurrent-insert TOCTOU simulation: duplicate INSERT → `try_map` → same field error a proactive failure produces | automated | `toctou_simulation_maps_to_field_error` — two inserts, winning insert seeds the slug, losing insert hits the constraint, `try_map` converts to `ValidationError` with the same field/message pair. |
| SC5 | `framework/src/validation/constraint_map.rs` holds no consumer-specific constraint/field/message literals | audit | `! grep -nE '^[^/]*("pages"|"slug"|"_unique")' framework/src/validation/constraint_map.rs` — only `///` doc-comment lines contain those tokens (as samples); no production literals. Confirmed in Plan 01 verification. Test-fixture identifiers (`cw`, `cw_slug_unique`, `cw.slug`) live in `framework/tests/`, outside the SC5 audit scope. |

---

## Postgres Manual Verification Gate

**Background (D-12):** `ConstraintMap::try_map` detects Postgres UNIQUE violations via
`DatabaseError::constraint()` — a trait method on `dyn DatabaseError` that dispatches to
`PgDatabaseError::constraint()` at runtime and returns protocol field `'n'` (the structured
constraint name, not the human-readable message). This path cannot run under the SQLite-only
`cargo test` default (no Postgres instance in CI). The shared `sql_err()` type gate and the
entry-match loop are fully exercised by the SQLite integration suite; only the Postgres-specific
`constraint()` branch requires manual confirmation.

**Why the manual step is a confidence check, not sole evidence:**

- `DatabaseError::constraint()` is a stable trait method in sqlx-core-0.8.6, declared with a
  default returning `None` and overridden by `PgDatabaseError` to return protocol field `'n'`
  (verified from `sqlx-postgres-0.8.6/src/error.rs`).
- The `try_map` implementation calls `e.constraint()` on `Box<dyn DatabaseError>` — no downcast,
  no `#[cfg]` guard — so the dispatch happens at runtime with no compile-time Postgres dependency.
- The SQLite suite exercises the same `sql_err()` type gate and the full entry-match loop; only
  the identity-extraction branch differs between backends.
- Source-verification confirms the `constraint()` call returns the protocol name; the manual step
  confirms the end-to-end runtime path under real Postgres.

### Reproducible Steps

**Prerequisites:** a running Postgres instance accessible via `DATABASE_URL`.

**1. Create a test table with a named UNIQUE constraint:**

```sql
CREATE TABLE IF NOT EXISTS pages (
    id SERIAL PRIMARY KEY,
    slug TEXT NOT NULL,
    CONSTRAINT pages_slug_unique UNIQUE (slug)
);
```

**2. Seed the first row:**

```sql
INSERT INTO pages (slug) VALUES ('hello');
```

**3. Trigger the UNIQUE violation:**

```sql
INSERT INTO pages (slug) VALUES ('hello');
-- Expected: ERROR: duplicate key value violates unique constraint "pages_slug_unique"
```

**4. In a Rust test or REPL connected to the same Postgres `DATABASE_URL`, capture the `DbErr`
and feed it to `ConstraintMap::try_map`:**

```rust
use ferro_rs::ConstraintMap;

// err is the DbErr from the failing INSERT above
let map = ConstraintMap::new()
    .on("pages_slug_unique", "slug", "has already been taken");

let ve = map.try_map(err)
    .expect("should map to ValidationError via DatabaseError::constraint()");

assert!(ve.has("slug"));
println!("Postgres gate: PASSED — constraint() dispatched correctly");
```

**5. Confirm:** `try_map` returns `Ok(ValidationError)` with `ve.has("slug")` true.
The match must come from the `e.constraint()` call returning `"pages_slug_unique"` (the
protocol field `'n'`), NOT from message-string parsing.

**Distinguishing Postgres from SQLite identity:** if `.sqlite("pages.slug")` is NOT registered
(omit the `.sqlite(...)` chain as shown above), the match can only come from the Postgres
`constraint()` path. This confirms the structured-name branch is operative.

---

## Sign-Off

**Status:** pending manual run

To close the gate: perform the steps above against a live Postgres instance, confirm the test
assertion passes, and update this line:

```
Status: signed — <operator> on <date> — Postgres constraint() branch confirmed
```

Per D-12 and the ROADMAP closure criteria, Phase 191 may close on the documented gate with
`status: pending manual run` because the SQLite automation suite (SC1, SC2, SC3 SQLite, SC4, SC5)
fully exercises the framework-level logic; the manual step is a confidence check on the
Postgres-specific `constraint()` runtime dispatch only.

---

## Full Quality Gate (automated)

Run command (one at a time per feedback-one-cpu-op-at-a-time constraint):

```bash
cargo fmt --all -- --check
cargo clippy --all --all-targets -- -D warnings
cargo test --all-features
```

Status: green — verified at Task 3 execution (2026-06-09).
