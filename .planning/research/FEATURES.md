# Feature Research

**Domain:** Rust web framework — async DB-backed uniqueness validation + DB-constraint-to-field-error mapping (ferro v12.4)
**Researched:** 2026-06-09
**Confidence:** HIGH (primary sources: Laravel docs, Rails source, Django source, SeaORM, existing ferro codebase)

---

## Existing ferro Validation Surface (Do Not Re-implement)

Before classifying what to build, what already exists and is in-scope as a dependency:

| Capability | Location | State |
|---|---|---|
| Sync `Rule` trait (`fn validate(&self, field, value, data) -> Result<(), String>`) | `framework/src/validation/rule.rs` | Shipped |
| `Validator` builder with `.rules(field, rules![...])`, `.validate()` | `framework/src/validation/validator.rs` | Shipped |
| `ValidationError` with `.with_old_input()`, `.into_action_error()`, `.redirect_to()` | `framework/src/validation/error.rs` | Shipped |
| `#[action]` macro + `ActionError` + `ActionResult` + `handle_action_result` | `framework/src/http/action.rs` | Shipped (Phase 180) |
| `From<sea_orm::DbErr> for ActionError` (raw passthrough — the target of this milestone) | `framework/src/http/action.rs:196` | Shipped, raw |
| `DB::get() -> Result<DbConnection, FrameworkError>` (sync, returns cloneable wrapper) | `framework/src/database/mod.rs` | Shipped |
| ferro-lang bridge (`OnceLock<TranslatorFn>` for validation messages) | `framework/src/validation/bridge.rs` | Shipped |

The gap: no async rule execution path; no DB-query rule; no way to map a `DbErr` to a specific field's validation error.

---

## Feature Landscape

### Table Stakes (Users Expect These)

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| `unique(table, column)` async rule | Every framework with a validator ships this (Laravel `Rule::unique`, Rails `validates :slug, uniqueness: true`, Django `UniqueConstraint` + `validate_unique()`). Absence forces hand-written pre-insert queries in every handler. | MEDIUM | Must be async — SeaORM DB ops are always async. Requires an `AsyncRule` trait or a parallel async validation path. Calls `DB::get()` internally (follows the service-container pattern Laravel uses). |
| Exclude-self semantics for edit forms (`.ignore(id)`) | Without ignore, every save of an unchanged slug on an edit form fails — the record matches its own current value. The #1 foot-gun in uniqueness validation across all frameworks. Every mature framework ships it (Laravel `.ignore($id)`, Rails implicit `AND id != record.id` on persisted records, Django `exclude={'pk': instance.pk}`). | LOW | Builder method on the rule struct: `unique("pages", "slug").ignore(id)`. Appends `AND id_col != $ignore` to the query. Ignore column defaults to `"id"`, overridable via `.ignore_column("uuid")`. |
| DB-constraint violation → field-level error mapping | Raw SQL unique constraint errors currently pass through as `ActionError::msg(err.to_string())`, surfacing "UNIQUE constraint failed: pages.slug" to the user. Frameworks catch these and map them to the right field with a human message. This is the safety net for the TOCTOU race the proactive rule cannot close. | MEDIUM | Extends `ActionError` with a builder chain: `ActionError::from_db_err(err).map_unique(constraint_hint, field, message)`. Matches the constraint hint against the `DbErr` message string. On match: produces a `ValidationError` flashed into session with `suppress_url_envelope: true`. On no-match: falls through to existing raw `DbErr` behavior. |
| Old-input preservation on async validation failure | The existing `with_old_input(&data).into_action_error(back_url)` path preserves form values on sync validation failure. Async uniqueness errors must go through the same path; otherwise the form resets on slug collision. | LOW | No new mechanism needed if async rules surface failures through `ValidationError`. The existing session-flash chain handles it unchanged. |
| `AsyncValidator` — async-capable counterpart to `Validator` | Handlers need one place to declare both sync and async rules, then call `.validate_async().await`. Having two separate calls (sync first, then manual async) is error-prone. | MEDIUM | `AsyncValidator::new(&data).rules("slug", rules![...]).async_rules("slug", async_rules![...]).validate_async().await`. Sync rules run in the same call before async rules (fail-fast on sync, skip DB hit if sync already fails). |

### Differentiators (Competitive Advantage)

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| `suppress_url_envelope` on constraint-mapped errors | Constraint violation caught at the DB layer surfaces like a proactive validation error — no redundant `?error=generic&msg=...` URL toast alongside the inline field error. This already exists on `ValidationError::into_action_error`; constraint mapping must use the same flag. | LOW | Reuse existing `ActionError::suppress_url_envelope: true` from Phase 180. No new mechanism. |
| Explicit proactive-vs-defensive layering in the API surface | The `unique` rule is proactive (prevents the error before the write); `map_unique` is defensive (handles the race condition after the write). Both are needed; the API names and docs make this distinction explicit. Most frameworks obscure it. | LOW | Method naming + docstring explicitly names the two layers and their relationship. Developers understand what they get from each. |
| Driver-aware constraint hint matching | SQLite format: `"UNIQUE constraint failed: pages.slug"`. Postgres format: `duplicate key value violates unique constraint "pages_slug_key"`. A small parser that recognizes both formats lets `map_unique("pages.slug", ...)` work for both drivers. | MEDIUM | Parse the `DbErr` message string. Extract table + column from SQLite format; extract constraint name from Postgres format and match against a `"table_column_key"` pattern. Confidence LOW — driver message formats are undocumented guarantees. Degrade gracefully: unrecognized format → fall through to raw `DbErr` message. Flagged as a phase-specific implementation risk. |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Sync `unique` rule (blocking DB call inside sync `Rule::validate`) | Looks simpler — no `AsyncValidator` needed; existing `Validator` unchanged. | Blocks the async executor. `tokio` will either deadlock or panic when a sync task tries to block on an async future. SeaORM is async-only. Structurally wrong. | `AsyncRule` trait + `AsyncValidator::validate_async().await`. The sync `Validator` stays unchanged. |
| Blanket auto-map ALL `DbErr::SqlError` variants to field errors | Seems convenient — catch everything, map to "something went wrong". | Different constraint types (unique, foreign key, check) need different field attribution. Blanket mapping produces wrong field attribution or exposes internal schema structure to users. | Explicit `.map_unique(constraint_hint, field, message)`. The caller knows which field the constraint protects. |
| Swallowing unmapped constraint violations silently | Prevents raw error strings from reaching users. | Hides real DB errors during development. An unmapped `DbErr` should surface as a generic `ActionError::msg(err.to_string())` so the developer sees it in logs, not as a silent 303 success. | Default: unmapped `DbErr` falls through to the existing `From<sea_orm::DbErr> for ActionError` passthrough. No silent swallowing. |
| Global constraint-name registry derived from migrations | Auto-register all unique constraints from migration metadata so `map_unique` always knows the names. | SeaORM migration constraint names vary by driver and are not stable across driver versions. Maintaining a registry introduces a second source of truth that can drift from the actual DB schema. | Two small string-pattern matchers (SQLite format, Postgres format). Call-site `.map_unique(hint, field, message)` is explicit and driver-portable. |
| Async rule support in the `ValidateRules` derive macro | Completeness — if the derive macro does sync rules, it should do async ones too. | The derive macro generates code that is statically typed to `Validator`, a sync type. Adding async rule support to the derive requires generating `AsyncValidator` code, which is a separate code-generation concern. Scope creep for v12.4. | Add `AsyncValidator` as a first-class type; `ValidateRules` macro extension deferred to a follow-on phase after the runtime is validated. |

---

## Feature Dependencies

```
AsyncRule trait (new)
    └──required by──> unique() rule builder
    └──required by──> AsyncValidator::async_rules()

unique() rule builder (new)
    └──calls──> DB::get() at validate_async() time (existing)
    └──calls──> SeaORM raw SELECT (existing SeaORM)
    └──uses──> validation.unique translation key (new ferro-lang entry)

AsyncValidator (new)
    └──composes──> existing Validator sync rules (reused, no change)
    └──runs──> AsyncRule list after sync rules pass
    └──produces──> ValidationError (existing, unchanged)

ValidationError (existing, unchanged)
    └──consumed by──> with_old_input().into_action_error() (existing)

ActionError (existing, extended)
    └──extended by──> from_db_err(err).map_unique(hint, field, msg) chain
    └──existing From<DbErr>──> raw passthrough for unmapped errors (unchanged)
    └──suppress_url_envelope = true when map_unique matches (existing flag, Phase 180)
```

### Dependency Notes

- **`AsyncRule` requires a new trait**: the existing `Rule` has a sync `fn validate()`. Async traits in Rust require either `async-trait` (boxes futures, stable) or RPITIT (nightly). Use `async_trait::async_trait` — matches SeaORM's own pattern; `async-trait` is already in the workspace transitively.
- **`unique` calls `DB::get()` internally**: the rule constructor is called inside `async_rules![...]` at build time (before the async context). `DB::get()` is called inside `validate_async()` at execution time. Same pattern Laravel uses for service-container resolution inside rule execution.
- **`AsyncValidator` is additive**: `Validator` stays sync, unchanged. Handlers with no async rules keep using `Validator::new().validate()` without modification. `AsyncValidator` is a new type, not a replacement.
- **Constraint mapping is independent of proactive rule**: a handler may use `map_unique` without the `unique` rule (defensive-only, accepting the TOCTOU window), or `unique` without `map_unique` (proactive-only, for low-concurrency forms). Both layers are independently opt-in.
- **The DB constraint index must exist independently**: the `unique` rule is UX (fail before the write with a clean message); the DB UNIQUE index is correctness (guarantee under concurrency). The rule does not create the index. Existing migration tooling handles index creation.

---

## Proactive-vs-Defensive: The Two-Layer Model

This distinction must be explicit in the API surface and documentation.

**Proactive layer (`unique` async rule):**
- SELECT COUNT(*) before the INSERT/UPDATE
- User sees a clean field-level error before any write occurs
- Handles the common case (single concurrent write, low contention)
- TOCTOU window: two concurrent requests can both pass the SELECT check and both attempt the INSERT; one will hit the DB constraint
- Appropriate for: admin forms, low-concurrency editing tools (gestiscilo, most ferro apps)

**Defensive layer (`map_unique` on `ActionError`):**
- Catches the `DbErr` emitted when the DB UNIQUE index fires on INSERT/UPDATE
- Maps it to a field-level error through the existing `ValidationError` flash path
- Old input preserved; user sees the same inline error as from the proactive rule
- Closes the TOCTOU window — even concurrent inserts produce a clean user experience
- Required for: high-concurrency public-facing forms (user registration, public slug creation)

Both layers together: the proactive rule handles 99%+ of cases cleanly; the defensive layer ensures the 1% race condition never leaks raw SQL to the user. For the gestiscilo use case (the motivating field test), the proactive rule alone is sufficient — the defensive layer is the correct professional standard, not overkill.

---

## Proposed Ferro-Idiomatic Developer Surface

These are design proposals, not final implementation — the phase plan will lock details.

### `AsyncRule` trait

```rust
// framework/src/validation/async_rule.rs
#[async_trait::async_trait]
pub trait AsyncRule: Send + Sync {
    async fn validate_async(
        &self,
        field: &str,
        value: &serde_json::Value,
        data: &serde_json::Value,
    ) -> Result<(), String>;

    fn name(&self) -> &'static str;
}
```

### `unique` rule builder

```rust
pub struct Unique {
    table: &'static str,
    column: &'static str,
    ignore_value: Option<serde_json::Value>,
    ignore_column: &'static str,  // default "id"
}

pub fn unique(table: &'static str, column: &'static str) -> Unique { ... }

impl Unique {
    /// Exclude the record with this pk from the uniqueness check (edit forms).
    pub fn ignore(mut self, id: impl Into<serde_json::Value>) -> Self { ... }

    /// Override the pk column name (default "id").
    pub fn ignore_column(mut self, col: &'static str) -> Self { ... }
}

#[async_trait::async_trait]
impl AsyncRule for Unique { ... }
```

### `AsyncValidator` — in-handler usage

```rust
// CREATE form
let errors = AsyncValidator::new(&data)
    .rules("slug", rules![required(), alpha_dash(), max(255)])
    .async_rules("slug", async_rules![unique("pages", "slug")])
    .validate_async()
    .await;

if let Err(e) = errors {
    return Err(e.with_old_input(&data).into_action_error(&back_url));
}

// EDIT form (exclude self — exclude-self semantics)
let errors = AsyncValidator::new(&data)
    .rules("slug", rules![required(), alpha_dash(), max(255)])
    .async_rules("slug", async_rules![unique("pages", "slug").ignore(page.id)])
    .validate_async()
    .await;
```

### DB-constraint mapping (defensive layer)

```rust
// In an #[action] handler body, after the insert/update call:
page.insert(&*DB::get()?)
    .await
    .map_err(|err| {
        ActionError::from_db_err(err)
            .map_unique("pages.slug", "slug", "has already been taken")
            // if DbErr message contains "pages.slug" (SQLite) or "pages_slug" (Postgres):
            //   → flashes ValidationError{"slug": ["has already been taken"]} to session
            //   → returns ActionError with redirect_to=back_url, suppress_url_envelope=true
            // otherwise:
            //   → returns ActionError::msg(err.to_string()) (existing raw passthrough)
            .redirect_to(&back_url)
    })?;
```

The `from_db_err().map_unique()` chain: `from_db_err` wraps the `DbErr`; `map_unique` inspects the message and either flashes + converts to a silent-envelope `ActionError`, or returns the raw message as a generic `ActionError`. `redirect_to` is the final builder method (same as existing `ActionError::redirect_to`).

---

## MVP Definition

### Launch With (v12.4 scope — both features)

- [x] `AsyncRule` trait with `async fn validate_async()` and `fn name()`
- [x] `unique(table, column)` constructor + `Unique` builder struct implementing `AsyncRule`
- [x] `.ignore(id)` on `Unique` (exclude-self semantics for edit forms)
- [x] `.ignore_column(col)` on `Unique` (non-default pk column name)
- [x] `async_rules![...]` macro mirroring `rules![...]`
- [x] `AsyncValidator` with `.rules()`, `.async_rules()`, `.validate_async().await`
- [x] Sync rules run first in `validate_async()`, short-circuit before DB hit on sync failure
- [x] `ActionError::from_db_err(err).map_unique(hint, field, message).redirect_to(url)` chain
- [x] Driver-aware constraint hint matching (SQLite + Postgres formats)
- [x] Translation key `validation.unique` in ferro-lang bridge
- [x] ferro-mcp `async_rule` code template registered (so agents know the pattern)

### Add After Validation (post-v12.4)

- [ ] `map_unique_auto(field, message)` — parses the constraint name without requiring the table-qualified hint; needs field testing on both drivers
- [ ] `exists(table, column)` async rule (field must reference an existing row — complement to `unique`)
- [ ] Async rule support in the `ValidateRules` derive macro

### Future Consideration (v13.0+)

- [ ] `AsyncValidator` integration with `#[action]` macro (auto-call on proc-macro level)
- [ ] Multi-column uniqueness: `unique("pages", "slug").scoped_to("account_id", account_id)`

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| `AsyncRule` trait | HIGH | LOW | P1 |
| `unique(table, column)` rule | HIGH | LOW | P1 |
| `.ignore(id)` on `Unique` | HIGH | LOW | P1 |
| `AsyncValidator` | HIGH | MEDIUM | P1 |
| `map_unique` on `ActionError` | HIGH | MEDIUM | P1 |
| Driver-aware hint matching | MEDIUM | MEDIUM | P1 (in scope — gestiscilo runs SQLite; Postgres must also work) |
| `validation.unique` translation key | MEDIUM | LOW | P1 |
| ferro-mcp code template | LOW | LOW | P1 (standard for all new patterns in ferro) |
| `map_unique_auto` | MEDIUM | MEDIUM | P2 |
| `exists` rule | MEDIUM | LOW | P2 |

---

## Competitor Ergonomics Summary

| Behavior | Laravel | Rails | Django | Ferro (proposed) |
|---|---|---|---|---|
| Proactive unique rule | `Rule::unique('table', 'col')` | `validates :col, uniqueness: true` | `UniqueConstraint` + `validate_unique()` | `unique("table", "col")` as `AsyncRule` |
| Exclude-self on edit | `.ignore($id)` or `.ignore($id, 'uuid')` | Implicit on persisted record (auto `AND id != ?`) | `exclude={'pk': instance.pk}` | `.ignore(id)` / `.ignore_column("uuid")` |
| Scope to parent | `.where(['account_id' => $accountId])` | `scope: :account_id` | `fields=['slug','account']` | deferred (post-v12.4) |
| Constraint mapping | N/A — proactive is sufficient in Laravel's sync world | `rescue ActiveRecord::RecordNotUnique` in controller | Caught by `ModelForm.save()`, re-raised as `ValidationError` | `ActionError::from_db_err(err).map_unique(hint, field, msg)` |
| Old-input preserved | `withInput()` in exception handler (automatic) | Manual `flash` in rescue block | Automatic in `ModelForm` round-trip | Existing `with_old_input(&data).into_action_error(url)` (unchanged) |

---

## Sources

- Laravel validation docs — `Rule::unique`, `->ignore()`: https://laravel.com/docs/validation#rule-unique — HIGH confidence
- Rails Guides — Active Record Validations — `uniqueness:`, `scope:`, rescue pattern: https://guides.rubyonrails.org/active_record_validations.html — HIGH confidence
- Django docs — `UniqueConstraint`, `ModelForm.validate_unique()`: https://docs.djangoproject.com/en/stable/ref/models/constraints/ — HIGH confidence
- ferro `framework/src/validation/rule.rs` — existing sync `Rule` trait — codebase, HIGH confidence
- ferro `framework/src/validation/validator.rs` — existing sync `Validator` — codebase, HIGH confidence
- ferro `framework/src/http/action.rs` — `ActionError`, `From<DbErr>`, `suppress_url_envelope`, Phase 180 — codebase, HIGH confidence
- ferro `framework/src/validation/error.rs` — `with_old_input`, `into_action_error`, `flash_into_session` — codebase, HIGH confidence
- ferro `framework/src/database/mod.rs` — `DB::get()`, `DbConnection` — codebase, HIGH confidence
- SQLite UNIQUE constraint error message format ("UNIQUE constraint failed: table.column") — MEDIUM confidence (de-facto stable, not formally documented)
- Postgres UNIQUE constraint error message format ("duplicate key value violates unique constraint ...") — MEDIUM confidence (stable across Postgres versions in practice)

---
*Feature research for: ferro v12.4 Form Validation DX — async unique rule + DB-constraint-to-field-error mapping*
*Researched: 2026-06-09*
