# Phase 191: ConstraintMap + Portable UNIQUE-Violation Detection - Context

**Gathered:** 2026-06-09
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults selected; review decisions below)

<domain>
## Phase Boundary

Add the **defensive** layer of the v12.4 two-layer uniqueness story: a
handler-opt-in `ConstraintMap` that intercepts a DB UNIQUE-constraint violation
at the write site and maps it to a field-level `ValidationError` — input
preserved, same 303 redirect-back as a proactive rule failure — closing the
TOCTOU window the Phase 190 async `unique` rule cannot eliminate.

Detection is backend-portable (SQLite + Postgres) via `DbErr::sql_err()` plus
bifurcated identification. A `DbErr` that matches no registered mapping falls
through **unchanged** to the existing `From<sea_orm::DbErr> for ActionError`
passthrough — never swallowed, never panics. The framework holds no
consumer-specific constraint/field strings; all mapping is registered at the
consumer call site.

In scope (VALID-04, VALID-05):
- `ConstraintMap` builder (`.on(...)` registration, `.try_map(err)`)
- Portable UNIQUE-violation detection (`sql_err()` + backend bifurcation)
- Fall-through-unchanged contract to the existing `From<DbErr>` passthrough

Out of scope (later phase): ferro-mcp `action_handler` template + validation
docs showing the two layers together (VALID-06, Phase 192).
</domain>

<decisions>
## Implementation Decisions

### ConstraintMap API surface
- **D-01:** Canonical builder mirrors ROADMAP success-criterion 1 **literally**:
  `ConstraintMap::new().on("pages_slug_unique", "slug", "has already been taken")`.
  `.on(constraint, field, message)` is a consuming builder (`mut self -> Self`,
  the established ferro builder convention). The primary registration key is the
  **Postgres constraint name**.
- **D-02:** `try_map(err: DbErr) -> Result<ValidationError, DbErr>`. Returns
  `Ok(ValidationError)` when `err` is a UNIQUE violation matching a registered
  entry (carrying the entry's `field` + `message`); returns `Err(err)`
  **unchanged** when it is not a UNIQUE violation OR matches no entry. Never
  swallows, never panics (ROADMAP SC1, SC2; VALID-05).
- **D-03 (Claude's discretion — locked):** The returned `ValidationError` is
  built via the existing `framework/src/validation/error.rs` constructor so it
  composes with `.with_old_input(&data).into_action_error(redirect_url)` exactly
  like a Phase 190 async-rule failure. Zero new error-surfacing code (SC1).

### Module home
- **D-04:** New file `framework/src/validation/constraint_map.rs`, re-exported
  through `framework/src/validation/mod.rs` and `framework/src/lib.rs`
  (`ConstraintMap` at crate root). ROADMAP key constraint: all Phase 191
  implementation lives in `framework/src/validation/` — the only Phase-192 carve-out
  is the ferro-mcp template. (Considered `ferro-orm`, where `GuardedUpdate` lives,
  but the mapping target is `ValidationError`, a `framework` validation type, so
  validation/ is the coherent home.)

### Portable detection (the killer correctness property)
- **D-05:** Violation-**type** detection is portable via sea-orm 1.1
  `DbErr::sql_err() -> Option<SqlErr>`, matching `SqlErr::UniqueConstraintViolation(_)`.
  This is the single portable entry point (VALID-05). Confirmed available in
  `sea-orm 1.1.20` (`framework/Cargo.toml:51`).
- **D-06:** Violation-**identity** detection is backend-bifurcated (ROADMAP SC3):
  - **Postgres:** structured constraint name via downcast of the `DbErr` inner
    sqlx error to `sqlx::postgres::PgDatabaseError` and reading `.constraint()`.
    No Postgres message-string parsing.
  - **SQLite:** parse `table.column` from the error message string
    (`"UNIQUE constraint failed: pages.slug"` → `pages.slug`). SQLite does not
    expose constraint names in the error, so the message token is the only
    available identifier.

### Match-key portability (the genuine gray area)
- **D-07 (Claude's discretion — locked):** A SQLite deployment never sees the
  Postgres constraint name, and a Postgres deployment never sees `table.column`.
  To make a single registration portable, each `ConstraintMap` entry stores BOTH
  identifiers. The canonical `.on(constraint, field, message)` keys by the
  Postgres constraint name; an optional chained `.sqlite("table.column")` adds the
  SQLite discriminator to the **same** entry, e.g.
  `.on("pages_slug_unique", "slug", "has already been taken").sqlite("pages.slug")`.
  - Postgres-only deployments work with `.on(...)` alone.
  - CI / dev on SQLite require the `.sqlite(...)` hint to match.
  - No magic constraint-name → `table.column` derivation (rejected as fragile —
    constraint naming conventions vary). The mapping is explicit.
  - Planner may refine the exact spelling (`.sqlite(...)` vs a `.on_sqlite(...)`
    sibling vs a `ConstraintId` value object) provided the behavioral contract
    holds: one entry matches its violation on whichever backend is live.

### Error surfacing & project-agnostic rule
- **D-08:** Reuse the Phase 190 surfacing chain end to end —
  `ValidationError` → `with_old_input()` → `into_action_error()` → 303
  redirect-back. No new redirect path, no new flash mechanism (SC1; mirrors
  Phase 190 D-04).
- **D-09:** `ConstraintMap` and all `.on(...)` strings (constraint names, field
  names, messages) are **consumer-owned**, held in the consumer-constructed map
  value. The `framework` crate carries zero constraint/field literals
  (project-agnostic-crates rule; VALID-05, SC5). Reviewer check: no `"pages"`,
  `"slug"`, `"_unique"` literals in `framework/src/validation/constraint_map.rs`
  outside doc examples.

### Verification strategy
- **D-10:** SQLite path is fully `cargo test`-able (in-memory SQLite, reuse the
  Phase 190 `widgets` fixture pattern): seed a row, attempt a duplicate INSERT,
  feed the resulting `DbErr` to `try_map`, assert `Ok(ValidationError)` with the
  right field; assert a non-UNIQUE `DbErr` (and an unregistered constraint)
  returns `Err(_)` unchanged.
- **D-11:** Concurrent-insert simulation (SC4) is modeled as: both logical
  handlers pass the pre-write `unique` check, then one INSERT hits the
  constraint — exercised by inserting the duplicate directly and asserting
  `try_map` yields the same field-level error a proactive failure would. A true
  multi-connection race is unnecessary; the constraint violation is the
  observable contract.
- **D-12:** Postgres constraint-name extraction (`PgDatabaseError::constraint()`)
  cannot run under the SQLite-only `cargo test` default. Closure includes a
  **documented manual verification gate** signed off in 191-VERIFICATION.md
  (mirrors the Phase 190 Postgres manual gate). Where feasible, assert the
  detection logic with a constructed/fixture error so the manual step is a
  confidence check, not the sole evidence.

### Claude's Discretion
- Exact spelling of the SQLite discriminator API (D-07): `.sqlite("table.column")`
  chained modifier vs `.on_sqlite(...)` sibling vs a `ConstraintId` value object.
- Concrete `try_map` internals: order of type-check (`sql_err()`) then
  identity-match, and how the inner sqlx error is downcast for Postgres.
- Whether `ConstraintMap` is `Clone` / reusable across requests or constructed
  per-handler (recommended: cheap to construct per call site; no global state).
- File-internal helper split within `constraint_map.rs`.

### Folded Todos
None — `todo match-phase 191` surfaced no matches.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase contract
- `.planning/ROADMAP.md` § "v12.4 Form Validation DX (Phases 190-192)" → "Phase 191: ConstraintMap + Portable UNIQUE-Violation Detection" — milestone goal, 5 locked success criteria, and the documented Postgres verification gate.
- `.planning/REQUIREMENTS.md` — VALID-04, VALID-05 (this phase); VALID-06 is Phase 192 (context only).

### Sibling phase (the proactive layer this complements)
- `.planning/phases/190-async-rule-infrastructure-unique-rule/190-CONTEXT.md` — locked decisions for the async `unique` rule; the `__infra_error__` sentinel and `AsyncValidationError` distinction; the two layers must read as one coherent story.
- `framework/src/validation/rules_async.rs` — Phase 190 `Unique` rule (proactive layer); identifier-guard + bound-parameter patterns to stay consistent with.

### Existing error surface (reuse, do not duplicate)
- `framework/src/http/action.rs:196` — `impl From<sea_orm::DbErr> for ActionError` — the exact passthrough `try_map` falls through to on no-match (VALID-05).
- `framework/src/validation/error.rs` — `ValidationError` with `with_old_input()` / `redirect_back()` / `into_action_error()` — the surfacing chain `try_map` reuses.
- `framework/src/validation/mod.rs`, `framework/src/lib.rs` — where the `ConstraintMap` re-export is wired (mirror the Phase 190 `unique` re-export chain).

### Database / sea-orm
- `framework/Cargo.toml:51` — `sea-orm 1.0` (resolved 1.1.20), features `sqlx-postgres` + `sqlx-sqlite`.
- sea-orm `DbErr::sql_err() -> Option<SqlErr>` and `SqlErr::UniqueConstraintViolation(String)` (sea-orm `src/error.rs`) — portable violation-type detection.
- `sqlx::postgres::PgDatabaseError::constraint()` — structured Postgres constraint name (Postgres identity path).

No external ADRs/specs — the contract is fully captured by ROADMAP + REQUIREMENTS plus the existing validation/error surface.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ValidationError` (validation/error.rs): full flash round-trip. `try_map`
  returns this type — zero new error-surfacing code, identical to Phase 190.
- `From<sea_orm::DbErr> for ActionError` (action.rs:196): the no-match
  fall-through target. `try_map` returning `Err(DbErr)` lets the caller's `?`
  reach this impl unchanged.
- Phase 190 in-memory SQLite fixture pattern (`widgets` scratch table,
  `DB::init_with`, `#[serial]`) — directly reusable for the SQLite `try_map` tests.

### Established Patterns
- Builder validators consume `self` and return `Self`; terminal method consumes
  `self`. `ConstraintMap::on()` follows this; `try_map(&self, err)` borrows so the
  map can be reused across multiple `?` sites in one handler.
- sea-orm error handling already flows through `From<DbErr>` impls in both
  `FrameworkError` (error.rs:454) and `ActionError` (action.rs:196) — the
  defensive layer slots in *before* that fallback, not replacing it.

### Integration Points
- New `ConstraintMap` export at `framework/src/validation/mod.rs` → re-exported
  from `framework/src/lib.rs` (crate-root `ferro_rs::ConstraintMap`).
- Consumer handler shape: `record.insert(db).await.map_err(|e| map.try_map(e)
  .map(|ve| ve.with_old_input(&data).into_action_error(url)).unwrap_or_else(ActionError::from))`
  — planner picks the exact ergonomic helper so the call site is not a closure
  ladder (this is the DX the phase exists to deliver).
</code_context>

<specifics>
## Specific Ideas

- Field-test motivation: gestiscilo-it slug-uniqueness violations surfaced as
  raw SQL errors through the `From<DbErr> for ActionError` passthrough. Phase 190
  added the proactive rule (UX); Phase 191 is the concurrency safety net that
  closes the check-then-insert race the proactive rule cannot.
- The authorize/capture mirror is intentional: like Phase 190's `unique`, the
  defensive layer must produce a field-level error *identical* to the proactive
  one, so a user hitting the race sees the same inline message as a user caught
  by the pre-check — the two layers are indistinguishable to the end user.
</specifics>

<deferred>
## Deferred Ideas

- **Foreign-key / check / not-null constraint mapping** — `SqlErr` also exposes
  `ForeignKeyConstraintViolation`. v12.4 scope is UNIQUE only (VALID-04/05). A
  generalized `ConstraintMap` covering FK/check violations is a plausible
  fast-follow but is not in the locked success criteria — do not expand Phase 191.
- **ferro-mcp template + validation docs** — the two-layer proactive+defensive
  pattern shown together (VALID-06) is Phase 192, gated on this phase's runtime
  surface being stable.

### Reviewed Todos (not folded)
None — `todo match-phase` surfaced no matches for Phase 191.
</deferred>

---

*Phase: 191-constraintmap-portable-unique-violation-detection*
*Context gathered: 2026-06-09 via /gsd-discuss-phase --auto*
