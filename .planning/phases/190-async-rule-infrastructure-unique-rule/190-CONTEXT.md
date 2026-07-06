# Phase 190: Async Rule Infrastructure + `unique` Rule - Context

**Gathered:** 2026-06-09
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults selected; review decisions below)

<domain>
## Phase Boundary

Add a DB-backed asynchronous validation path to `framework/src/validation/` so a
handler can assert a field's value is unique in a table **before** the
insert/update, with exclude-self support for edit forms. The new path is
parallel to the existing synchronous `Validator` — the sync API and its rules
are untouched. Failures surface through the existing `ValidationError` →
`with_old_input()` → 303 redirect-back flow.

In scope (VALID-01, VALID-02, VALID-03):
- `AsyncRule` trait
- `Unique` rule struct with `.ignore()` exclude-self
- `AsyncValidator` + `validate_async()`
- `validation.unique` ferro-lang translation key

Out of scope (later phases): DB constraint-violation mapping at the write site
(`ConstraintMap`, Phase 191); ferro-mcp template + docs (Phase 192).
</domain>

<decisions>
## Implementation Decisions

### AsyncRule trait mechanism
- **D-01:** Use the `async-trait` crate for `AsyncRule` (already a `framework`
  dependency at `framework/Cargo.toml:30`). This keeps the trait
  dyn-compatible so async rules are stored as `Box<dyn AsyncRule>`, mirroring
  the existing `Box<dyn Rule>` ergonomics. Trait shape mirrors `Rule`:
  `async fn validate(&self, field: &str, value: &Value, data: &Value) -> Result<(), String>`
  plus `fn name(&self) -> &'static str`.

### AsyncValidator composition
- **D-02:** `AsyncValidator` holds **both** sync rules (`Box<dyn Rule>`) and
  async rules (`Box<dyn AsyncRule>`). A handler uses one validator, not two.
  `.async_rule(field, rule)` registers async rules; the existing sync
  `.rule()` / `.rules()` builder ergonomics are preserved for sync rules.
- **D-03:** `validate_async().await` runs **all sync rules first**, then runs
  async rules **only on fields that have no sync error** (fail-fast — no DB
  query for a field that already failed a sync check). This is locked by
  ROADMAP success criterion 3.
- **D-04:** `validate_async` returns the existing `ValidationError` on
  validation failure (reuses `with_old_input()` / `redirect_back()` /
  `into_action_error()` — no new error-surfacing path).
- **D-05 (Claude's discretion — see note):** Exact constructor/`validate_async`
  signature. Recommended: mirror the sync `Validator::new(&data)` borrow so the
  validator has the submitted values, and obtain the DB inside the rule (D-07),
  not via a threaded argument. The ROADMAP criterion-1 snippet
  (`AsyncValidator::new().…validate_async(&req)`) is illustrative; planner
  picks the precise signature provided the **behavioral** contract holds: sync
  data available, DB via singleton, `ValidationError` returned. Either
  `validate_async(&data)` or `validate_async(&req)` is acceptable as long as it
  does not double-consume `req.input()`.

### `unique` rule — query and identifier safety
- **D-06:** `unique(table, column)` takes string identifiers (per ROADMAP /
  VALID-01: `unique("pages", "slug")`). The uniqueness check is a parameterized
  `SELECT COUNT(*) FROM <table> WHERE <column> = ?` (value bound as a SQL
  parameter). Backend is detected via the connection's
  `get_database_backend()` so quoting is correct for SQLite and Postgres.
- **D-07:** DB access inside `Unique` is via the `DB::connection()` singleton
  (`framework/src/database/mod.rs:171`), which Derefs to
  `sea_orm::DatabaseConnection`. No connection is threaded through the
  `AsyncRule` signature or `validate_async()` (locked by ROADMAP criterion 5 /
  VALID-03).
- **D-08:** Table and column are **developer-controlled** identifiers (they come
  from handler code, never end-user input). They cannot be SQL-bound, so they
  are interpolated; guard them by per-backend quoting and rejecting any
  identifier outside `[A-Za-z0-9_]`. Document this trust boundary in the rule's
  rustdoc.

### Exclude-self (edit forms)
- **D-09:** `.ignore(id)` accepts `impl Into<sea_orm::Value>` so `i64`, `Uuid`,
  `String`, and `&str` ids all work. When set, the query gains
  `AND <pk> <> ?` with the id bound as a parameter.
- **D-10:** The excluded primary-key column defaults to `"id"`. A non-`id` PK is
  supported via an explicit form (e.g. `.ignore_on(pk_col, id)` or a `pk`
  argument) — planner chooses the exact spelling; the default-`"id"` happy path
  is mandatory (VALID-02).

### Default message + localization
- **D-11:** Default message uses the `validation.unique` translation key with an
  `("attribute", field)` param, English fallback
  `"The {field} has already been taken."` — exactly mirroring the
  `translate_validation(...).unwrap_or_else(...)` pattern every existing rule
  uses (`framework/src/validation/rules.rs`). A per-rule custom message
  overrides the default, consistent with the sync `Validator` message map.

### DB / infrastructure failure semantics
- **D-12:** A DB or infrastructure failure while running an async rule (e.g.
  connection error) is **not** a validation result. It must propagate as a
  framework error (handler returns 500) — never silently pass and never be
  reported as a field-level validation failure. Planner picks the concrete
  `Result` shape that carries both a `ValidationError` (validation failure) and
  a framework error (infra failure) without conflating them.

### Claude's Discretion
- Precise `AsyncValidator` constructor / `validate_async` signature (D-05).
- Exact spelling of the non-default-PK exclude-self API (D-10).
- Concrete `Result` type encoding the validation-vs-infra distinction (D-12).
- File split within `framework/src/validation/` (e.g. `async_rule.rs`,
  `async_validator.rs`, `rules_async.rs`) — all components are new files in
  that module per the ROADMAP key-constraints note.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase contract
- `.planning/ROADMAP.md` § "v12.4 Form Validation DX (Phases 190-192)" → "Phase 190: Async Rule Infrastructure + `unique` Rule" — milestone goal, key constraints, and the 5 locked success criteria.
- `.planning/REQUIREMENTS.md` — VALID-01, VALID-02, VALID-03 (this phase); VALID-04..06 are Phases 191/192 (context only).

### Existing validation surface (extend, do not duplicate)
- `framework/src/validation/mod.rs` — module exports + `rules!` macro; where new async exports are added.
- `framework/src/validation/rule.rs` — the sync `Rule` trait the async trait mirrors.
- `framework/src/validation/validator.rs` — `Validator` builder/`validate()` the async validator parallels (nullable handling, per-field rule loop, custom messages/attributes).
- `framework/src/validation/error.rs` — `ValidationError` with `with_old_input()` / `redirect_back()` / `redirect_to()` / `into_action_error()` — reuse, do not re-create.
- `framework/src/validation/rules.rs` — the `translate_validation(key, params).unwrap_or_else(English)` pattern each rule follows (template for `Unique`).
- `framework/src/validation/bridge.rs` — `translate_validation` / `TranslatorFn` (`validation.<rule>` keys, `attribute` param).

### DB access
- `framework/src/database/mod.rs:171` — `DB::connection() -> Result<DbConnection, FrameworkError>`; `DbConnection` Derefs to `sea_orm::DatabaseConnection` (use `.get_database_backend()`, `query_one(Statement)`).

### Dependency already present
- `framework/Cargo.toml:30` — `async-trait = "0.1"` (use for `AsyncRule`).

No external ADRs/specs — the contract is fully captured by ROADMAP + REQUIREMENTS plus the existing validation module.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ValidationError` (error.rs): full flash round-trip (`with_old_input` →
  `redirect_back`/`into_action_error`). The async path returns this type — zero
  new error-surfacing code.
- `translate_validation` (bridge.rs): localization hook. `Unique` registers a
  `validation.unique` key the same way `required`/`email`/etc. do.
- `DB::connection()` (database/mod.rs): the singleton DB accessor the rule uses,
  satisfying "no connection threaded" directly.
- `rules!` macro (mod.rs): boxes heterogeneous rules; a parallel `async_rules!`
  macro may be added for `Box<dyn AsyncRule>` (planner's call).

### Established Patterns
- Builder validators consume `self` and return `Self`; `validate()` consumes
  `self` and returns `Result<(), ValidationError>` — async validator follows the
  same shape.
- Every rule: `translate_validation(key, &[("attribute", field)]).unwrap_or_else(|| English)`.
- Sync rules are `Send + Sync`; async rules must be too (boxed trait objects
  shared across the request).

### Integration Points
- New exports added in `framework/src/validation/mod.rs` and re-exported from
  `framework/src/lib.rs` if user-facing (e.g. `AsyncValidator`, `unique`).
- The `validation.unique` key needs a fallback/registration story consistent
  with how existing keys are wired (consumer ferro-lang files; framework ships
  English fallback).
</code_context>

<specifics>
## Specific Ideas

- Field-test motivation: gestiscilo-it slug-uniqueness violations surfaced as
  raw SQL errors. The proactive `unique` rule is the UX fix; the constraint
  mapping (Phase 191) is the concurrency safety net. Phase 190 delivers the
  proactive layer only.
- Exclude-self is the regression guard: a copy-pasted create handler reused for
  edit, without `.ignore(id)`, would falsely reject an unchanged unique value.
  This is why `.ignore()` ships in v1 rather than being retrofitted.
</specifics>

<deferred>
## Deferred Ideas

- **Scoped / conditional uniqueness (e.g. per-tenant slug uniqueness)** — v1
  ships `unique(table, column)` + `.ignore(id)` only, matching the locked
  success criteria. A `.where_eq(col, val)` scope (unique *within* a tenant) is
  the most likely fast-follow. **Open question for the researcher:** confirm
  gestiscilo-it's tenancy model — if tenants are separate databases, global
  `unique` is already correct and no scoping is needed; if tenancy is a
  `tenant_id` column in a shared table, a scope predicate is required and this
  becomes a v12.4 follow-up phase rather than a Phase-190 addition. Do not
  expand Phase 190 scope without that confirmation.
- **Additional async rules** (`exists`, `custom_async`) — the stale v12.4 draft
  mentioned `.custom_async(...)`. Not in the Phase 190 success criteria; add
  only if it falls out of the `AsyncRule` trait for free, otherwise defer.

### Reviewed Todos (not folded)
None — `todo match-phase` surfaced no matches for Phase 190.

</deferred>

---

*Phase: 190-async-rule-infrastructure-unique-rule*
*Context gathered: 2026-06-09*
