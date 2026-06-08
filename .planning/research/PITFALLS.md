# Pitfalls Research — v12.4 Form Validation DX

**Domain:** Async DB-backed uniqueness validation + DB-constraint-violation → field-level error mapping in a SeaORM dual-backend (SQLite + Postgres) Rust web framework
**Researched:** 2026-06-09
**Confidence:** HIGH (codebase inspection of `framework/src/validation/`, `framework/src/http/action.rs`, SeaORM 1.x docs, SQLite and Postgres error-code specifications)

---

## Critical Pitfalls

### Pitfall 1: Treating the async `unique` rule as the authoritative guarantee

**What goes wrong:**
The async pre-write SELECT that checks uniqueness creates a TOCTOU (time-of-check / time-of-use) window. Two concurrent requests can both pass the SELECT check, both proceed to INSERT, and one will hit the DB UNIQUE index. If the constraint→field mapping is not implemented (or is incomplete), that second INSERT surfaces as a raw `DbErr` with no field attribution — the exact problem the milestone exists to fix.

**Why it happens:**
The natural mental model is: "I checked before writing, so it must be unique." This leads consumers to implement the `unique` rule carefully but to handle the actual DB INSERT path with `?` on `DbErr` → `ActionError::from(err)`, which passes the raw message through.

**How to avoid:**
Communicate the architecture explicitly in the rule's documentation and in any generated code template:

1. The async `unique` rule is a UX layer — it catches the common case before the round-trip and produces a clean field-level error immediately.
2. The UNIQUE index is the invariant. It is always enforced by the DB.
3. The constraint→field mapping (step 2) is the safety net that must be present regardless of whether the async rule is registered.

Design principle: the async rule and the constraint mapping are not alternatives; they compose. The async rule without mapping is incomplete. The mapping without the async rule is correct but produces a worse UX (race is rare; pre-check prevents the error in the common case).

In code, the pattern is:
```rust
// Step 1: async pre-check (UX)
Validator::new(&data)
    .rules("slug", rules![unique("articles", "slug")])
    .validate_async(&db).await?;

// Step 2: write + constraint mapping (invariant)
Article::create(&data)
    .save(&db).await
    .map_constraint("articles_slug_key", "slug", "This slug is already taken.")?;
```

**Warning signs:**
- Generated `#[action]` handlers that use `?` directly on `Entity::insert().exec(&db).await?` after an async validation step — the constraint path is unhandled.
- MCP code templates that show the async rule but omit constraint mapping.
- Test suites that exercise only the serial (single-request) path, never concurrent inserts.

**Phase to address:**
The constraint-mapping primitive (Phase 1 of v12.4). The documentation and MCP template must make this composition explicit before any consumer can scaffold a uniqueness-validated form.

---

### Pitfall 2: Backend-portable constraint detection — SQLite message parsing vs Postgres structured fields

**What goes wrong:**
The constraint→field mapping must identify which constraint was violated to route the error to the right field. Postgres and SQLite provide this information differently:

- **Postgres**: `PgDatabaseError` exposes `constraint() -> Option<&str>` (the index name, e.g. `"articles_slug_key"`) and `table() -> Option<&str>` via sqlx downcast. SeaORM's `DbErr::sql_err()` returns `SqlErr::UniqueConstraintViolation(message)` where `message` is the human-readable error string (`"duplicate key value violates unique constraint \"articles_slug_key\""`). The constraint name is available without string parsing by downcasting through `DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(e)))` and calling `e.constraint()`.

- **SQLite**: `SqliteError` exposes only an extended error code (`2067` for UNIQUE constraint, `1555` for PRIMARY KEY constraint) and a message string. The message format is `"UNIQUE constraint failed: articles.slug"` (table.column format). There is no separate `constraint()` field — the constraint NAME is not included in SQLite's error output because SQLite does not name its UNIQUE indexes in the error text. The only structured information is the table and column name embedded in the message.

SeaORM's `sql_err()` wraps both into `SqlErr::UniqueConstraintViolation(message)` and discards the Postgres constraint name. If the mapping logic uses only what `sql_err()` returns, it can never access the Postgres constraint name without re-doing the downcast.

**Why it happens:**
Developers reach for `db_err.sql_err()` as the portable API (correct) but then try to extract the constraint name from the message string (wrong for Postgres — unnecessary; wrong for SQLite — the name is not there at all).

**How to avoid:**
Design the constraint→field mapping around two separate identification strategies, applied in order:

1. **Constraint name match (Postgres)**: Downcast through `DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(e)))` and call `e.constraint()`. Match against the registered constraint name (e.g. `"articles_slug_key"`). This is reliable and does not require string parsing.

2. **Table+column match (SQLite fallback)**: Parse the `message` string from `SqlErr::UniqueConstraintViolation(message)`. For SQLite the format is consistently `"UNIQUE constraint failed: <table>.<column>"`. Match on `table_name` + `column_name` extracted from this string.

The public API for consumers should register mappings at both levels:

```rust
db_err
    .map_constraint("articles_slug_key", "slug", "Slug already taken")   // Postgres: by index name
    .map_column("articles", "slug", "slug", "Slug already taken")         // SQLite: by table+column
```

Or unify into a single registration that stores both keys and matches whichever applies.

The Postgres extraction path requires re-downcasting after `sql_err()` has already returned. Structure the utility so the downcast happens once, and both the constraint name and the `SqlErr` variant are extracted in the same match arm.

**Warning signs:**
- Any code that calls `.contains("duplicate key")` or regex-matches against error message strings on the Postgres path.
- Any code that tries to extract a constraint name from a SQLite error message (the constraint name is not present in SQLite output).
- CI that only tests SQLite (the default `cargo test` environment) — Postgres-specific constraint-name matching is never exercised.

**Phase to address:**
The constraint-detection primitive phase. Must be tested against both backends. If Postgres CI is not available in the gate, add a compile-time check and a documented manual test step.

---

### Pitfall 3: Exclude-self bug on edit forms

**What goes wrong:**
The `unique` rule checks whether a value already exists in a column. For edit forms, the record being edited already exists, so the check must exclude the current record's ID. If the exclusion is omitted or uses the wrong column, the rule fires even when the user has not changed the field — the form refuses to save a record that was already saved with that value.

Common mistakes:
- Excluding by a different column than the actual primary key (e.g., excluding by `uuid` when the index uses `id`).
- Hardcoding `"id"` when the entity uses a non-standard primary key column name.
- Forgetting to exclude self entirely on edit handlers (copy-pasting the create handler's validator).

**Why it happens:**
The create and edit handlers share the same validator setup code in many templates. The create path works correctly; the edit path is a copy where the `exclude_id` parameter is not added.

**How to avoid:**
The `unique` rule must have an explicit `exclude_id` parameter. The API should make the exclusion positionally unavoidable on the edit path:

```rust
// Create form — no exclusion
rules![unique("articles", "slug")]

// Edit form — exclusion required; omitting it is a compile-time or runtime error
rules![unique("articles", "slug").exclude_self(article.id)]
```

The MCP code template for edit handlers must always include `exclude_self`. If the entity's PK column name is not `"id"`, the rule must accept a column name parameter: `exclude_self_on("uid", article.uid)`.

Test: assert that a record CAN be saved with its own existing slug — this is the self-exclusion regression test. This test is distinct from the "create duplicate" test and must exist independently.

**Warning signs:**
- Edit handler that copies the create handler's validator without adding `exclude_self`.
- Users reporting that editing a record without changing the slug produces a "slug already taken" error.
- Integration tests that only test create, not create-then-edit-with-same-value.

**Phase to address:**
The async rule implementation phase. The `exclude_self` parameter must be part of the first version of the rule; retrofitting it later risks silently incorrect edit forms in existing consumer code.

---

### Pitfall 4: Leaking raw SQL error messages to end users via `From<DbErr> for ActionError`

**What goes wrong:**
`framework/src/http/action.rs` line 196–199 implements:
```rust
impl From<sea_orm::DbErr> for ActionError {
    fn from(err: sea_orm::DbErr) -> Self {
        Self::msg(err.to_string())
    }
}
```

`DbErr::to_string()` for a Postgres unique constraint violation produces: `"error returned from database: duplicate key value violates unique constraint \"articles_slug_key\""`. For SQLite: `"error returned from database: UNIQUE constraint failed: articles.slug"`. Both strings contain table and column internals. They are the user-facing `ActionError::message` field, which lands in the flash toast.

The `?` operator inside `#[action]` handlers triggers this `From` impl automatically. Every `Entity::insert().exec(&db).await?` is a potential info leak.

**Why it happens:**
The `From<DbErr>` impl was added for ergonomics — `?` on DB operations just works. The fact that the converted message is user-facing is not obvious from the call site.

**How to avoid:**
The milestone's constraint→field mapping must intercept constraint violations before they reach the `From<DbErr>` conversion. The idiomatic pattern:

```rust
Article::insert(model)
    .exec(&db).await
    .map_constraint_err("articles_slug_key", "slug", "Slug already taken")?
    // ^ returns Err(ValidationError) routed through into_action_error, not Err(ActionError)
```

The `From<DbErr> for ActionError` impl should NOT be removed — it is correct for non-constraint DB errors (connection failures, syntax errors, etc.). Instead, constraint violations must be caught BEFORE the `?` operator converts them to `ActionError`. The safety net documentation in the `From<DbErr>` impl should note that constraint violations should be handled via `map_constraint_err` before `?`.

**Warning signs:**
- User sees a toast containing "UNIQUE constraint failed" or "duplicate key value violates unique constraint".
- `ActionError::message` containing table or column names from the schema.
- Handlers that use `entity_insert().exec(&db).await?` without a preceding `map_constraint_err` call when the entity has UNIQUE indexes.

**Phase to address:**
The constraint→field mapping phase. The `From<DbErr>` impl is a pre-existing leak; the milestone closes it by providing a correct interception point.

---

### Pitfall 5: Blocking the async executor with DB queries inside the validation loop

**What goes wrong:**
The existing `Rule` trait is synchronous:
```rust
pub trait Rule: Send + Sync {
    fn validate(&self, field: &str, value: &Value, data: &Value) -> Result<(), String>;
}
```

A `unique` rule that queries the database cannot implement this trait directly. The temptation is to call `tokio::runtime::Handle::current().block_on(db_query)` inside the synchronous `validate()` method, which blocks the async executor thread and can deadlock or severely degrade throughput.

**Why it happens:**
The easiest path to "make the test green" when the trait is sync is to block inside the rule. The issue is invisible in development with a single-threaded runtime and no concurrent requests.

**How to avoid:**
Introduce a separate `AsyncRule` trait (or an `AsyncValidator`) rather than hacking sync-to-async conversion inside a sync rule:

```rust
pub trait AsyncRule: Send + Sync {
    async fn validate(&self, field: &str, value: &Value, data: &Value, db: &DatabaseConnection) -> Result<(), String>;
}
```

The `Validator` type needs an `async fn validate_async(&self, db: &DatabaseConnection) -> Result<(), ValidationError>` method. Sync rules run synchronously; async rules run in the async path. They do not mix: a validator with async rules cannot call the sync `validate()` method.

The Tokio runtime constraint is hard: never call `block_on` from within an async context. The test that catches this: run the async validator under `#[tokio::test]` with a concurrent request simulation — if it deadlocks, `block_on` is present.

**Warning signs:**
- `Rule::validate()` implementation that calls `Handle::current().block_on(...)`.
- Handler under load shows executor thread starvation (all threads blocked, no progress).
- Any use of `std::sync::Mutex` or `std::thread::sleep` inside a `Rule` implementation.

**Phase to address:**
The async rule trait design phase — this is the foundational decision. The trait split must be the first thing designed; everything else builds on it.

---

### Pitfall 6: N queries for N unique fields in a single validation pass

**What goes wrong:**
A form with three unique fields (`slug`, `email`, `sku`) runs three separate DB queries if each field has its own `unique` rule. Under the async-rule design, `validate_async` iterates rules in sequence, and each `unique` rule issues one SELECT. At N=3 this is minor; in a form with many indexed fields or under high concurrency, the sequential query pattern amplifies DB load.

**Why it happens:**
The rule-per-field model maps naturally to one query per rule. Batching requires cross-field coordination that is absent from the per-field rule design.

**How to avoid:**
For the v12.4 scope (typically 1–3 unique fields per form), sequential queries are acceptable. Do not over-engineer batching now. However:

1. Document the N-query behavior explicitly so consumers know to minimize `unique` rule count.
2. Design the async-rule interface so that batching could be added later without breaking the consumer API (e.g., a `batch_validate_async` hook on a trait the rule can optionally implement).
3. Add a connection-acquisition cost note: each query in the pool may acquire a connection. With SeaORM's connection pool, this is cheap but not free. Do not use `unique` rules for fields that could instead be validated by DB triggers or deferred to the constraint-mapping path.

**Warning signs:**
- Forms with more than five `unique` rules.
- Slow form submission under load that is not explained by the write path.

**Phase to address:**
Async rule implementation phase. Add the documentation note; defer batching to a later phase if the need is confirmed.

---

### Pitfall 7: Case-sensitivity and collation mismatch between the app-level check and the DB index

**What goes wrong:**
The async `unique` SELECT uses whatever collation the query applies by default. If the DB index uses a case-insensitive collation (`COLLATE NOCASE` in SQLite, `citext` or `COLLATE "ci_..."` in Postgres), the proactive SELECT may pass ("HELLO" not found) while the DB insert fails the constraint ("hello" already exists — the index treats them as equal).

Alternatively, the proactive SELECT may use a case-insensitive comparison (`LIKE` or `LOWER(value) = LOWER(column)`) while the DB index is case-sensitive, producing false positives (the rule rejects the value, but the DB would have accepted it).

**Why it happens:**
The `unique` rule constructs a `SELECT COUNT(*) WHERE column = ?` query using SeaORM's query builder. The comparison uses the column's default collation. The developer writes the rule without inspecting the index's collation, and the mismatch is invisible until an edge case hits.

**How to avoid:**
1. The `unique` rule's SELECT must match the index's collation. For case-insensitive indexes, use `LOWER(column) = LOWER(?)` or a collation-aware SeaORM condition.
2. Document the collation parameter on the `unique` rule: `unique("articles", "slug").case_insensitive()` generates `WHERE LOWER(slug) = LOWER(?)`.
3. Prefer matching the DB index exactly: if the index is case-sensitive, the rule is case-sensitive. Do not add case-insensitivity at the app layer that the index does not share.
4. The constraint mapping path is immune to this problem — the DB enforces its own collation. This is another argument for treating constraint mapping as the ground truth and the async rule as a best-effort pre-check.

**Warning signs:**
- Users can create "hello" and "HELLO" as distinct slugs when they should be the same.
- Users are blocked from using a value that differs only in case from an existing value, when the index is case-sensitive.
- The rule passes, the INSERT fires, and the constraint violation fires anyway (the async rule's collation does not match the index).

**Phase to address:**
Async rule implementation phase. The rule must expose a collation parameter from the start. Retrofitting collation handling breaks existing rule invocations.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Implement `unique` as a sync rule using `block_on` | No new trait needed | Executor starvation, potential deadlock under concurrent requests | Never |
| Route constraint violations through `From<DbErr> for ActionError` with a generic message | No constraint-mapping API needed | Raw SQL internals visible to users; no field attribution | Never for user-facing forms |
| Use `sql_err()` message string parsing for both Postgres and SQLite constraint name extraction | Single code path | Fragile: Postgres message format is an implementation detail that can change; constraint name is not in SQLite messages at all | Never; use structured fields for Postgres, table.column parsing for SQLite |
| Skip `exclude_self` on the first version and add later | Faster initial implementation | Every consumer edit handler is silently broken until the upgrade | Never; must be in v1 of the rule |
| Only test SQLite in CI for constraint mapping | Fast CI | Postgres-specific constraint name matching is never tested | Acceptable only if a documented manual Postgres test step exists |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| SeaORM `sql_err()` | Using only `SqlErr::UniqueConstraintViolation(message)` and parsing the message for the constraint name | For Postgres: downcast the underlying `sqlx::Error` to `PgDatabaseError` and call `.constraint()`. For SQLite: parse `"UNIQUE constraint failed: <table>.<column>"` from the message. |
| SeaORM `sql_err()` scope | Calling `sql_err()` on `DbErr::Connect` or `DbErr::Migration` variants | `sql_err()` only returns `Some` for `DbErr::Exec` and `DbErr::Query` variants wrapping `RuntimeErr::SqlxError(sqlx::Error::Database(...))`. Other variants return `None`. |
| SeaORM dual-backend | SQLite error code `2067` (UNIQUE) vs `1555` (PRIMARY KEY) | Both map to `SqlErr::UniqueConstraintViolation`. Distinguish PK violations from column violations by the message content if needed, but for field mapping this distinction rarely matters. |
| Postgres `constraint()` | Assuming the constraint name matches the column name | Constraint names are index names, set at migration time (e.g. `"articles_slug_key"`). They do not match column names. Consumers must register the index name, not the column name. |
| SQLite UNIQUE constraint messages | Assuming `"UNIQUE constraint failed: table.column"` format is guaranteed | This is SQLite's current format and has been stable, but it is not part of SQLite's API contract. A version update could change it. Treat it as best-effort; the constraint-mapping path is the canonical guarantee. |
| `ValidationError::into_action_error` | Using `redirect_to` + `return Err(ActionError::msg(...))` instead | Use `ValidationError::into_action_error(url)` which flashes per-field errors AND suppresses the redundant generic toast. The old two-step pattern was deprecated in Phase 180. |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Sequential async uniqueness queries for N fields | Form submission latency increases linearly with N unique fields | Document the N-query cost; recommend limiting `unique` rules; design for optional batching later | Noticeable at N ≥ 5 unique fields under concurrent load |
| Connection pool exhaustion from validator acquiring connections inside a request | Request latency spikes; pool timeout errors | Reuse the request's existing DB connection in the validator, do not acquire a fresh pool connection per rule | Under load with a small pool (< 10 connections) |
| Duplicate uniqueness checks (async rule + redundant SELECT in the handler before insert) | 2x DB queries for no gain | Trust the async rule result; do not re-check in the handler body | Any load |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| `DbErr::to_string()` as `ActionError::message` on constraint violations | Leaks table names, column names, and index names to the user via the flash toast and URL query string | Intercept constraint violations with `map_constraint_err` before `?` converts them to `ActionError`. The `From<DbErr>` impl is not removed, but constraint violations must not reach it. |
| Constraint-field mapping accepting user-supplied field names at runtime | An attacker could supply an arbitrary field name and have arbitrary error messages attributed to arbitrary form fields | The constraint→field mapping must be static (compile-time or configuration-time). The field name in the error is always the developer-registered field key, never derived from the error message. |
| Exposing the constraint name in a user-visible error message | Leaks internal schema naming conventions | Never include the raw constraint name (e.g. `"articles_slug_key"`) in a user-visible message. The constraint name is an internal registration key only. |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Constraint violation with no field attribution (current state) | User sees a generic error toast with raw SQL text; no indication which field is invalid; form input is lost | Constraint→field mapping routes the error to the field inline with `with_old_input` preserving form state |
| Async `unique` rule without `with_old_input` | User sees field-level error but the form is blank — all input is lost on the redirect | Always chain `with_old_input(&data)` before `into_action_error(url)` |
| `unique` check passes, but constraint fails on concurrent insert, and no mapping is registered | User sees raw SQL error from the racing insert | The constraint mapping is the safety net; it must always be registered even when an async rule is present |
| Showing a uniqueness error before the user finishes typing | User is interrupted mid-input | The async rule runs on form submit only, never on keypress. No client-side polling. |

---

## "Looks Done But Isn't" Checklist

- [ ] **Async unique rule:** Verify `exclude_self` is in the API and tested with an edit-form scenario (record flagging itself).
- [ ] **Constraint mapping:** Verify it is tested against both SQLite (error code 2067, message parsing) AND Postgres (SQLSTATE 23505, constraint name via downcast).
- [ ] **Concurrent insert:** Verify there is a test that simulates two simultaneous inserts of the same unique value, confirming the constraint-mapping path fires and produces a field-level error (not a raw SQL message).
- [ ] **Old input preservation:** Verify form input survives the constraint-violation redirect — the flash round-trip must use `with_old_input` + `into_action_error`, not the query-param `?error=...` path.
- [ ] **No raw SQL in user output:** Verify `ActionError::message` never contains "UNIQUE constraint failed" or "duplicate key value" in any test path.
- [ ] **`From<DbErr> for ActionError` gap:** Verify the existing blanket `From` impl is documented with a warning that constraint violations must be handled upstream via `map_constraint_err`.
- [ ] **Collation coverage:** Verify the `unique` rule's SELECT uses the same case sensitivity as the target index (document the `case_insensitive()` parameter if it exists).
- [ ] **MCP code template:** Verify the `action_handler` template includes both the async rule AND the constraint-mapping call — not one without the other.

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| `unique` rule without `exclude_self` shipped to consumers | MEDIUM | Add `exclude_self` to the rule, update MCP template, issue a BREAKING CHANGE note since existing edit handlers need update |
| Constraint detection broken on Postgres but not SQLite | LOW | Fix the downcast path; no API change; add Postgres CI step |
| Raw SQL in user-visible messages found post-ship | LOW | Add `map_constraint_err` call to affected handlers; no framework change needed |
| Async rule using `block_on` discovered under load | HIGH | Requires trait redesign; all consumers must update to `validate_async`; cannot be hot-patched |
| Collation mismatch causing duplicate inserts to slip through async check | MEDIUM | Add `case_insensitive()` parameter to rule; existing call sites without the parameter retain old behavior |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| TOCTOU — async rule treated as the guarantee | Constraint-mapping primitive phase | Code review: every handler with a `unique` rule also has `map_constraint_err`; MCP template includes both |
| SQLite vs Postgres constraint detection divergence | Constraint-detection primitive phase | CI gate: integration tests run against both backends; Postgres constraint name test must be present |
| Exclude-self bug on edit forms | Async rule implementation phase | Integration test: create record, edit with same value, assert save succeeds |
| Raw SQL leak via `From<DbErr>` | Constraint-mapping primitive phase | Snapshot test: no `ActionError::message` ever matches `"UNIQUE constraint failed"` or `"duplicate key"` regex |
| `block_on` inside sync Rule | Async rule trait design — first | Compile-time: `AsyncRule` trait is separate; no `block_on` in any `Rule` impl; async validator path requires `validate_async` |
| N queries for N unique fields | Async rule implementation phase | Document; add note in rule docs; no structural fix needed for v12.4 |
| Case-collation mismatch | Async rule implementation phase | API review: `unique` rule exposes `case_insensitive()` from the start |
| Old input loss on constraint redirect | Constraint-mapping phase | Integration test: constraint redirect preserves all form fields in `req.old()` |

---

## Sources

- SeaORM 1.x `DbErr::sql_err()` and `SqlErr` enum: https://docs.rs/sea-orm/1.1.14/src/sea_orm/error.rs.html
- sqlx `PgDatabaseError::constraint()`: https://docs.rs/sea-orm/1.1.14/sea_orm/error/struct.SqlxPostgresError.html
- SQLite extended error codes 1555 (SQLITE_CONSTRAINT_PRIMARYKEY) and 2067 (SQLITE_CONSTRAINT_UNIQUE): https://www.sqlite.org/rescode.html
- Postgres SQLSTATE 23505 (unique_violation): https://www.postgresql.org/docs/current/errcodes-appendix.html
- Codebase inspection: `framework/src/http/action.rs` lines 196–199 (`From<sea_orm::DbErr> for ActionError` passthrough)
- Codebase inspection: `framework/src/validation/rule.rs` (synchronous `Rule` trait — no `db` parameter, no `async fn`)
- Codebase inspection: `framework/src/validation/error.rs` — `with_old_input`, `into_action_error`, `flash_into_session` chain
- PROJECT.md v12.4 milestone description — gestiscilo-it slug-uniqueness raw-SQL field-test source

---
*Pitfalls research for: async DB-backed uniqueness validation + constraint→field error mapping in SeaORM dual-backend Rust framework*
*Researched: 2026-06-09*
