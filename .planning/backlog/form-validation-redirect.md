# Form Validation & Redirect-Back

**Source:** gestiscilo-it field test, 2026-04-18
**Severity:** Medium (UX impact — raw SQL errors shown to users on constraint violations)
**ferro version:** 0.2.x

## Problem

Every form-based controller in a Ferro app manually:
1. Validates fields with inline `if` checks
2. Builds redirect URLs with `?error=field_name` query params
3. Reads query params in GET handlers and maps error codes to Italian strings
4. Shows errors via `InputProps { error: Some(...) }`

This is ~50 lines of boilerplate per form. Worse, DB constraint violations (e.g. UNIQUE) bypass validation entirely and surface as raw SQL error pages via `error_response(500, ...)`.

## Desired Behavior

### 1. Validator struct

```rust
// In POST handler:
let v = Validator::new()
    .required("label", "Il nome è obbligatorio")
    .max_len("label", 200)
    .custom("slug_path", |v| is_valid_slug(v), "Indirizzo non valido")
    .unique::<Page>("slug_path", Column::TenantId.eq(tenant_id), "Indirizzo già in uso");

if let Err(errors) = v.validate(&form_data) {
    return errors.redirect_back(); // flashes errors + old input into session
}
```

### 2. Old input preservation

On validation failure, flash all submitted values into the session. The next GET request can populate `default_value` via `req.old("field_name")`.

```rust
// In GET handler / form builder:
InputProps {
    default_value: req.old("label"),
    error: req.validation_error("label"),
    ..
}
```

### 3. DB constraint → friendly error

A middleware or error hook that catches `UNIQUE constraint failed` (SQLite) / `duplicate key value violates unique constraint` (Postgres) from SeaORM and converts them to validation-style redirects instead of raw 500 pages.

## Implementation Notes

- Ferro already has `session.flash()` / `session.get_flash()` in `framework/src/session/store.rs` — use this for old input and error storage
- `Request` struct needs `old()` and `validation_error()` convenience methods that read from flash
- `Validator` lives in the framework crate, not a separate crate
- The `unique` rule needs async (DB query) — consider `v.validate_async(&form_data).await`
- For JSON-UI apps, `redirect_back()` should use the `Referer` header or an explicit redirect target
- The DB constraint hook should be opt-in (middleware), not implicit magic

## Scope

Three phases:

1. **Validator + old input** — `Validator` struct, `req.old()`, `req.validation_error()`, flash-based round-trip
2. **Async rules** — `unique` and other DB-backed validation rules
3. **DB constraint error mapping** — middleware that catches constraint violations and converts to redirect-back with errors

## Current Workaround

gestiscilo-it manually checks uniqueness before insert and redirects with query params. Works but verbose and doesn't preserve old input.
