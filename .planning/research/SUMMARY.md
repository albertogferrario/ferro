# Project Research Summary

**Project:** ferro v12.4 Form Validation DX
**Domain:** Async DB-backed uniqueness validation + DB constraint-to-field-error mapping in a SeaORM dual-backend (SQLite + Postgres) Rust web framework
**Researched:** 2026-06-09
**Confidence:** HIGH

## Executive Summary

The v12.4 milestone addresses a concrete, field-confirmed deficiency: uniqueness violations in ferro forms surface raw SQL error strings to end users instead of inline field-level errors with preserved input. Every mature web framework solves this with two complementary mechanisms — a proactive pre-write uniqueness rule and a defensive post-write constraint-violation mapper. The two are not alternatives; they compose. The proactive rule handles the common case before the write with a clean user experience; the constraint mapper is the safety net that closes the TOCTOU race window the proactive rule cannot eliminate. Shipping only one of the two is incomplete, and the MCP code template must make this composition explicit to prevent the pattern from being applied incorrectly by agents scaffolding handlers.

The technical path is fully resolved. Rust's async trait limitations on stable 2021 edition rule out making the existing sync `Rule` trait async — instead, a separate `AsyncRule` trait using `Pin<Box<dyn Future>>` is the correct object-safe design, requiring no new crate dependencies (`async-trait` is already in the workspace). The `Unique` struct implements `AsyncRule` by running a raw `SELECT COUNT(*)` via `sea_orm::Statement::from_sql_and_values`, which is backend-agnostic. For constraint detection, `DbErr::sql_err()` is the portable entry point; constraint-to-field mapping lives at the handler call site as an explicit `ConstraintMap` builder, which respects the project-agnostic-crates rule (no consumer strings embed in `framework/`). Both features are entirely contained within `framework/src/validation/` with no changes to `framework/src/http/action.rs`.

The primary implementation risk is backend divergence in constraint identification: SQLite embeds `"table.column"` in the error message string while Postgres embeds the index name. The `ConstraintMap::try_map()` implementation must handle both formats. The Postgres path has a structured `constraint()` field available via sqlx downcast — message-string parsing on the Postgres path is fragile and should not be used. CI exercises SQLite by default; Postgres constraint-name matching requires either a Postgres CI step or a documented manual gate. This is the only open question that cannot be fully resolved before implementation.

## Key Findings

### Recommended Stack

No version changes or new crates required. The entire milestone is implementable against the pinned stack: `sea-orm 1.1.19` (with `sqlx-postgres`, `sqlx-sqlite` features), `sqlx 0.8.6` (transitive). `async-trait` is added to `framework/Cargo.toml` but is already present in the workspace via `ferro-events` and `ferro-queue`. All SeaORM APIs used (`DbErr::sql_err()`, `SqlErr::UniqueConstraintViolation`, `Statement::from_sql_and_values`) are verified against sea-orm 1.1.14/1.1.19 docs.

**Core technologies:**
- `sea-orm 1.1.19`: DB layer and constraint detection via `DbErr::sql_err()` — portable across SQLite and Postgres without feature flags
- `async-trait` (workspace, no new dep): enables `Box<dyn AsyncRule>` object safety on stable Rust — required by `AsyncRule` trait
- `sqlx 0.8.6` (transitive only): `PgDatabaseError::constraint()` available for structured constraint name extraction on Postgres; not added as a direct dep to preserve backend portability

### Expected Features

**Must have (table stakes for v12.4):**
- `AsyncRule` trait — object-safe async rule trait; foundation for all async validation
- `unique(table, column)` rule — pre-write SELECT COUNT(*) check; every framework with a validator ships this
- `.ignore(id)` on `Unique` — exclude-self for edit forms; without this, every unchanged-slug edit fails (the #1 foot-gun across all frameworks)
- `AsyncValidator` — unified place to declare both sync and async rules; sync rules run first (fail-fast before DB hit)
- `ConstraintMap::new().on(constraint, field, message).try_map(err)` — explicit opt-in constraint-to-field mapping at the handler write site; closes the TOCTOU window

**Should have (differentiators):**
- Driver-aware constraint hint matching (SQLite `"table.column"` format + Postgres constraint-name-in-message format) in `ConstraintMap::try_map()`
- `suppress_url_envelope` on constraint-mapped errors — reuses existing Phase 180 flag; constraint violation surfaces identically to a proactive rule failure
- MCP code template including both async rule AND constraint mapping in the `action_handler` template — makes the two-layer composition impossible to miss when scaffolding

**Defer (post-v12.4):**
- `map_unique_auto(field, message)` — auto-parses constraint name without a hint; needs field testing on both drivers
- `exists(table, column)` async rule — complement to `unique`
- Async rule support in `ValidateRules` derive macro
- Multi-column scoped uniqueness: `unique(...).scoped_to("account_id", id)`

### Architecture Approach

Both features are additive within `framework/src/validation/` and share no code with `framework/src/http/action.rs` (unchanged). The `AsyncRule` trait and `AsyncValidator` are parallel to the existing sync `Rule`/`Validator` pair — existing callers see no change. `ConstraintMap` is a standalone struct at the handler call site, keeping consumer constraint names out of the framework crate. The global `DB::connection()` singleton is the DB access pattern for async rules, consistent with `framework/src/database/model.rs`, `query_builder.rs`, and `transaction.rs`.

**Major components:**
1. `async_rule.rs` (new) — `AsyncRule` trait: `Pin<Box<dyn Future>>` return, `Send + Sync`, `async-trait` derived
2. `rules/unique.rs` (new) — `Unique` struct + `unique()` constructor + `.ignore()` + `.ignore_where()` builders, implements `AsyncRule`
3. `validator.rs` (modified) — `async_rule()` builder + `validate_async()` method added; sync `validate()` unchanged
4. `constraint.rs` (new) — `ConstraintMap` builder + `try_map(DbErr) -> Result<ValidationError, DbErr>` with SQLite/Postgres bifurcated detection
5. `mod.rs` (modified) — re-exports `AsyncRule`, `Unique`, `unique`, `AsyncValidator`, `ConstraintMap`

### Critical Pitfalls

1. **TOCTOU — async `unique` rule treated as the authoritative guarantee** — two concurrent requests can both pass the pre-write SELECT and both attempt the INSERT; one hits the DB constraint. If `ConstraintMap` is absent, the constraint violation leaks as raw SQL. Mitigation: always pair `unique` rule with `ConstraintMap` at the write site; MCP code template must show both; code review must flag handlers with `unique` but no downstream `ConstraintMap`.

2. **`block_on` inside sync `Rule::validate()`** — blocking the async executor inside a sync rule causes starvation under concurrent load and can deadlock. Mitigation: separate `AsyncRule` trait is the architectural invariant; the trait split must be the first design decision.

3. **SQLite vs Postgres constraint detection divergence** — SQLite embeds `"table.column"` in the error message string; Postgres exposes the index name via `PgDatabaseError::constraint()` (sqlx downcast). Parsing the Postgres message string for the constraint name is fragile. Mitigation: `ConstraintMap::try_map()` uses structured `constraint()` for Postgres and string-match for SQLite, applied in that order.

4. **Exclude-self bug on edit forms** — copy-pasting the create handler's validator to the edit handler without adding `.ignore(id)` causes the rule to reject the record's own existing value. Mitigation: `.ignore(id)` must be in the first API version; MCP edit-handler template must always include it; integration tests must cover create-then-edit-with-same-value.

5. **Raw SQL leak via `From<DbErr> for ActionError`** — `?` on an unguarded `Entity::insert().exec(&db).await?` passes `"UNIQUE constraint failed: pages.slug"` to the user as a flash message. Mitigation: constraint violations must be intercepted by `ConstraintMap::try_map()` before `?` converts them to `ActionError`; the `From<DbErr>` impl is not removed (correct for non-constraint errors) but must not be reached for constraint violations.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Async Rule Infrastructure + `unique` Rule

**Rationale:** `AsyncRule` trait is the strict foundation — nothing else in the milestone compiles without it. The `Unique` struct is the only concrete `AsyncRule` in v12.4 scope and validates the trait design immediately. `AsyncValidator` is the integration point that proves both work end-to-end. This unit has no dependency on constraint detection and can ship atomically.

**Delivers:**
- `AsyncRule` trait (object-safe, stable Rust, `async-trait`, no new crate dep beyond workspace)
- `Unique` struct with `unique()`, `.ignore()`, `.ignore_where()` builders
- `AsyncValidator` with sync-then-async execution and field-skipping on prior sync failures
- `validation.unique` translation key in the ferro-lang bridge
- Integration tests: create-form uniqueness, edit-form self-exclusion (the regression Pitfall 3 identifies as most likely to be missed)

**Addresses:** AsyncRule, unique rule, exclude-self, AsyncValidator (all P1 from FEATURES.md)

**Avoids:** `block_on` inside sync Rule (Pitfall 5), exclude-self bug on edit forms (Pitfall 3)

**Research flag:** Standard patterns. `Pin<Box<dyn Future>>` object-safe async trait is well-documented stable Rust. `async-trait` usage in `ferro-events` and `ferro-queue` provides workspace precedent. No further research needed.

---

### Phase 2: `ConstraintMap` + Constraint Detection

**Rationale:** Depends on Phase 1 confirming the `AsyncRule`/`AsyncValidator` API is stable before the handler-level API is finalized. `ConstraintMap` is independently testable (no async dependency) but its call-site API is more clearly designed with the full picture visible. This phase closes the TOCTOU window and eliminates the raw SQL leak confirmed in `action.rs:196`.

**Delivers:**
- `ConstraintMap::new().on(constraint, field, message).try_map(err)` struct in `constraint.rs`
- Bifurcated detection: SQLite message-string parse + Postgres `PgDatabaseError::constraint()` downcast
- `Err(DbErr)` return on no-match (never silently swallows; falls through to existing `From<DbErr>` passthrough)
- Integration tests for both backends; concurrent-insert simulation for the TOCTOU path
- Old-input preservation verified: constraint redirect uses `with_old_input` + `into_action_error`, not `?error=` query param

**Addresses:** DB-constraint mapping, driver-aware hint matching, suppress_url_envelope (all P1 from FEATURES.md)

**Avoids:** Raw SQL leak via `From<DbErr>` (Pitfall 4), constraint detection divergence (Pitfall 2), silent error swallowing

**Research flag:** Postgres constraint-name extraction via `PgDatabaseError::constraint()` requires a real Postgres instance to test. If Postgres CI is not available, require a documented manual test step as part of the phase closure criteria.

---

### Phase 3: MCP Template + Docs

**Rationale:** Both runtime primitives must be stable before templates and documentation can accurately represent the composition pattern. The MCP `action_handler` template is the highest-leverage surface — every agent-scaffolded handler with a unique constraint will follow it. A template that shows only the async rule without the `ConstraintMap` call leaks TOCTOU into every generated handler.

**Delivers:**
- `ferro-mcp/src/tools/code_templates.rs`: `unique_validation` and `constraint_map` templates; updated `action_handler` template showing both layers together
- `docs/src/the-basics/validation.md`: async rules section + constraint mapping section with explicit proactive-vs-defensive framing
- Phase gate: confirm no generated handler template shows `unique` without a downstream `ConstraintMap`

**Addresses:** ferro-mcp code template (P1 from FEATURES.md), proactive-vs-defensive documentation

**Avoids:** TOCTOU in agent-generated code (Pitfall 1 — most likely failure mode in scaffolded handlers)

**Research flag:** Standard patterns. MCP template registration follows the established `code_templates.rs` pattern from Phase 180. No further research needed.

---

### Phase Ordering Rationale

- `AsyncRule` trait must precede `Unique` rule (which implements it) must precede `AsyncValidator` (which holds `Vec<Box<dyn AsyncRule>>`). These are a strict compile-time dependency chain and belong in one phase.
- `ConstraintMap` is independently testable but its call-site API is more clearly designed once `AsyncValidator` is finalized. Phase 2 gets one phase of design stability before finalizing the complementary API.
- MCP templates and docs must trail stable runtime code. Documenting a pattern before its API is locked risks shipping docs that describe the wrong surface.
- All three phases are entirely within `framework/src/validation/` and `ferro-mcp/src/tools/` — no cross-crate coordination needed.

### Research Flags

Phases needing deeper research or explicit verification during planning:
- **Phase 2 (ConstraintMap):** Postgres constraint-name extraction must be tested against a real Postgres instance. Plan must include either a Postgres CI step or a documented manual verification gate as part of closure criteria.
- **Phase 1 (AsyncValidator — design decision):** Mixed sync+async rules in one `AsyncValidator` call vs. two separate calls must be locked before writing code. Research notes both are valid; the phase plan must make this call explicitly. Two-call pattern is more explicit; unified pattern is more ergonomic. Either is acceptable.

Phases with standard patterns (no research phase needed):
- **Phase 1 (AsyncRule trait):** `Pin<Box<dyn Future>>` object-safe async trait is well-documented; workspace precedent in `ferro-events` and `ferro-queue`.
- **Phase 3 (MCP + Docs):** Template registration follows Phase 180 pattern exactly.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | sea-orm 1.1.19 + sqlx 0.8.6 verified from Cargo.toml + cargo tree. All APIs verified via Context7 + docs.rs. No version changes needed. |
| Features | HIGH | Table-stakes features cross-validated against Laravel, Rails, Django. Anti-features justified with concrete failure modes from the codebase. |
| Architecture | HIGH | All integration points verified against actual framework source files. `ferro-reservation/src/kernel.rs` provides constraint-detection downcast precedent. |
| Pitfalls | HIGH | Critical pitfalls derived from codebase inspection (confirmed `From<DbErr>` passthrough at action.rs:196) + SeaORM/SQLite/Postgres error-code specifications. |

**Overall confidence:** HIGH

### Gaps to Address

- **Mixed sync+async rules in one `AsyncValidator` call:** Design choice left open for Phase 1 planning. Recommendation: start with two-call (explicit); defer unified API to post-v12.4 if gestiscilo usage makes the ergonomic argument concrete.
- **Postgres-only CI coverage for constraint-name path:** `PgDatabaseError::constraint()` downcast cannot be exercised by `cargo test` defaults. Phase 2 plan must address this explicitly — either add a Postgres CI step or define a manual verification gate in the closure criteria.
- **`ConstraintMap` file location:** Research notes it could be inlined in `mod.rs` or split to `constraint.rs`. Recommendation: split to `constraint.rs` (consistent with `async_rule.rs` and `rules/unique.rs` being new files). Lock in Phase 2.

## Sources

### Primary (HIGH confidence)
- Context7 `/websites/rs_sea-orm_1_1_14` — `DbErr::sql_err()` full source, `SqlErr` enum, per-backend error-code dispatch (Postgres SQLSTATE 23505, SQLite extended codes 1555/2067)
- `https://docs.rs/sqlx/0.8.6/sqlx/postgres/struct.PgDatabaseError.html` — `constraint() -> Option<&str>` confirmed Postgres-only
- `https://docs.rs/sqlx/0.8.6/sqlx/error/trait.DatabaseError.html` — `DatabaseError` trait; no `column()` method confirmed
- `framework/Cargo.toml` + `cargo tree` — sea-orm 1.1.19, sqlx 0.8.6 pinning verified
- `framework/src/validation/rule.rs`, `validator.rs`, `error.rs`, `bridge.rs` — existing sync validation surface
- `framework/src/http/action.rs` lines 196-199 — `From<DbErr> for ActionError` passthrough (confirmed pre-existing leak)
- `framework/src/database/mod.rs` — `DB::connection()` singleton facade
- `ferro-reservation/src/kernel.rs` — `DbErr::Exec(RuntimeErr::SqlxError(...))` destructuring precedent for constraint detection
- `https://www.sqlite.org/rescode.html` — SQLite extended result codes 1555, 2067
- `https://www.postgresql.org/docs/current/errcodes-appendix.html` — Postgres SQLSTATE 23505

### Secondary (MEDIUM confidence)
- Laravel validation docs (`Rule::unique`, `.ignore()`) — cross-validation of proactive/exclude-self API ergonomics
- Rails Active Record Validations guide (`uniqueness:`, rescue pattern) — cross-validation
- Django docs (`UniqueConstraint`, `ModelForm.validate_unique()`) — cross-validation
- SQLite `"UNIQUE constraint failed: table.column"` message format — stable in practice, not part of SQLite's formal API contract

---
*Research completed: 2026-06-09*
*Ready for roadmap: yes*
