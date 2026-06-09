# Requirements: v12.4 Form Validation DX

## Milestone Goal

Make uniqueness validation a first-class, ergonomic part of ferro forms — both proactively (an async DB-backed `unique` rule that runs before the write) and defensively (DB constraint violations mapped to field-level errors instead of leaking raw SQL to end users). The killer feature: a uniqueness violation that today surfaces as a raw SQL error instead lands inline under the right field with the user's input preserved — uniqueness "just works" before the write (async rule, UX) and as a safety net at the write (constraint mapping, concurrency invariant).

Source: gestiscilo-it field test — slug-uniqueness violations surfaced as raw SQL errors through the `From<sea_orm::DbErr> for ActionError` passthrough.

## Conceptual Coherence Anchor

v12.4 introduces no new abstraction. It extends the existing validation layer (`framework/src/validation/`) with an async sibling of the established sync `Rule`/`Validator` surface, and composes with Phase 180's `#[action]` / `ActionError` and the existing `ValidationError` → redirect-back-with-old-input path. Both new surfaces produce the same `ValidationError` shape and flow through the same 303 redirect mechanism as every existing rule — the user-visible behavior is identical whether a failure is caught proactively or defensively.

**Two-layer model (both required, neither sufficient alone):**
- **Proactive** — the async `unique` rule is UX: it catches the common case with a clean field error before the write.
- **Defensive** — the DB UNIQUE index remains the source of truth; constraint→field mapping closes the check-then-insert (TOCTOU) race that the proactive rule cannot.

## v1 Requirements

### Async DB-Backed Validation

- [x] **VALID-01** — A developer can validate that a field's value is unique in a DB table via an async rule (`unique(table, column)`), failing validation **before** the insert/update with a field-level error message.
- [x] **VALID-02** — A developer can exclude the current record from the uniqueness check on edit forms (`.ignore(id)` / exclude-self), so saving an unchanged unique value does not falsely fail. Exclude-self ships in v1 (retrofitting it later is a breaking change for edit handlers).
- [x] **VALID-03** — Async rules run through an `AsyncValidator` / `validate_async` path that leaves the existing synchronous `Validator` API and its existing rules unchanged, obtains its DB connection via the existing `DB::connection()` singleton (no connection threaded through the rule signature), and surfaces failures through the existing `ValidationError` → `with_old_input` → 303 redirect-back flow.

### DB Constraint → Field-Level Error Mapping

- [x] **VALID-04** — A developer can opt in to mapping a DB UNIQUE-constraint violation to a specific field's validation error at the handler call site (e.g. a `ConstraintMap` / `map_unique` builder), so a concurrent-insert violation surfaces inline under the field with input preserved — identical to a proactive rule failure — instead of a raw SQL error.
- [x] **VALID-05** — Constraint-violation detection is backend-portable across SQLite and Postgres (via `DbErr::sql_err()` and bifurcated identification — Postgres constraint name, SQLite table.column from the message). A `DbErr` that does not match a registered mapping falls through unchanged to the existing `From<sea_orm::DbErr> for ActionError` passthrough — never swallowed, never panics. The framework holds no consumer-specific constraint/field strings (project-agnostic-crates rule): mapping is registered at the consumer call site.

### Introspection & Docs

- [ ] **VALID-06** — The `ferro-mcp` `action_handler` code template and the validation docs demonstrate the async `unique` rule **and** constraint mapping together (proactive + defensive), so the two-layer pattern is discoverable and no surface shows one layer without the other.

## Anti-Requirements (explicit non-goals to prevent scope drift)

- The synchronous `Validator` / `Rule` API is not changed or deprecated — async is a parallel path, not a replacement.
- No general-purpose async rule library beyond `unique` for this milestone (other async rules can follow the same `AsyncRule` trait later).
- No automatic, framework-level `DbErr` → field inference without explicit consumer registration (would require embedding consumer strings in `framework`).
- The `From<sea_orm::DbErr> for ActionError` passthrough at `action.rs:196` is retained as the non-constraint fallback — not removed.

## Future Requirements (deferred)

- Additional async rules (e.g. `exists`, async cross-field checks) on the `AsyncRule` trait.
- CHECK / FK / NOT NULL constraint mapping beyond UNIQUE.
- Per-rule async timeout guard.

## Out of Scope

- Original v12.1-era Phase 137–139 scope (Validator struct, sync rules, old-input flash, `req.old()`) — already shipped organically via the validation module.
- Client-side / JS validation — ferro forms are server-validated; out of scope.
- ORM-entity-generic uniqueness (typed entity column references) — raw `SELECT COUNT(*)` via `Statement` is backend-agnostic and sufficient.

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| VALID-01 | Phase 190 | Complete |
| VALID-02 | Phase 190 | Complete |
| VALID-03 | Phase 190 | Complete |
| VALID-04 | Phase 191 | Complete |
| VALID-05 | Phase 191 | Complete |
| VALID-06 | Phase 192 | Pending |
