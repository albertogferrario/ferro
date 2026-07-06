---
phase: 190-async-rule-infrastructure-unique-rule
reviewed: 2026-06-09T00:00:00Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - framework/src/validation/async_rule.rs
  - framework/src/validation/rules_async.rs
  - framework/src/validation/async_validator.rs
  - framework/src/validation/mod.rs
  - framework/src/lib.rs
  - framework/tests/async_rule_fixture.rs
  - framework/tests/async_validation_integration.rs
findings:
  critical: 0
  warning: 3
  info: 3
  total: 6
status: issues_found
---

# Phase 190: Code Review Report

**Reviewed:** 2026-06-09
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

The async validation infrastructure is well-structured. The three security-critical requirements from the prompt are correctly implemented: identifier guards run before any DB call (T-190-01), DB errors surface as `AsyncValidationError::Infra` via the sentinel and never as field messages (D-12), and no consumer-specific strings appear in non-test source. `Send + Sync` bounds are correct for boxed async trait objects.

Three warnings are raised: a silent type-conversion swallow that could mask infra errors, a semantic error in the `null` → `sea_orm::Value` conversion, and a footgun in the `From<AsyncValidationError> for ActionError` impl that discards all field errors. Three info items cover a `rules()` / `rule()` overwrite inconsistency, code duplication, and a missing bulk `async_rules()` builder.

---

## Critical Issues

None.

---

## Warnings

### WR-01: `null` JSON value converts to string `"null"` instead of SQL NULL

**File:** `framework/src/validation/rules_async.rs:183`
**Issue:** In `json_value_to_sea_value`, the catch-all arm serialises `Value::Null` (and objects/arrays) to `sea_orm::Value::String(Some(Box::new(v.to_string())))`. For `null` this produces the string `"null"`. Any `WHERE col = ?` query then binds the literal string `"null"` rather than SQL `NULL`, making the comparison `col = 'null'` instead of `col IS NULL`. In practice the `Unique` rule never receives a `null` value (the async phase skips null-nullable fields), but a direct call to `json_value_to_sea_value` on `null` silently produces a wrong binding that would always return 0 rows and pass uniqueness.

**Fix:**
```rust
serde_json::Value::Null => sea_orm::Value::String(None),
// Note: a NULL SQL value will never match any existing row in a `col = ?`
// comparison (SQL NULL semantics). If a caller genuinely needs IS NULL
// support they should use a separate code path; this is safer than "null".
```

---

### WR-02: Type-conversion failure on COUNT column silently returns 0 (passes validation)

**File:** `framework/src/validation/rules_async.rs:149-151`
**Issue:** The count is read with:
```rust
let count: i64 = row
    .and_then(|r| r.try_get::<i64>("", "count").ok())
    .unwrap_or(0);
```
`r.try_get(...).ok()` converts any `DbErr` from the type conversion into `None`, and `unwrap_or(0)` then treats a failed read as "0 rows found" — meaning uniqueness passes silently. If the column alias, type, or backend behaviour changes, the rule will stop detecting duplicates with no error surfaced. This should be an infra error.

**Fix:**
```rust
let count: i64 = match row {
    None => 0,
    Some(r) => r
        .try_get::<i64>("", "count")
        .map_err(|e| format!("__infra_error__: count column read failed: {e}"))?,
};
```

---

### WR-03: `From<AsyncValidationError> for ActionError` discards all field errors on the `Validation` variant

**File:** `framework/src/validation/async_validator.rs:52-63`
**Issue:** The `From` impl for the `Validation` variant returns `ActionError::validation_failed("/")`, hardcoding the redirect URL and discarding all field errors and old input. Any caller that uses the `?` operator on an `AsyncValidationError` (or calls `ActionError::from(e)` directly) will get a bare redirect to `"/"` with no error flash. The documented usage pattern in the module doc correctly calls `e.with_old_input(...).into_action_error(...)` on the inner `ValidationError`, but the `From` impl creates a pitfall for callers who don't read that carefully.

The impl should either be removed (forcing callers to match explicitly) or panic/document that it is intentionally lossy. A silent wrong-redirect is worse than a compile error.

**Fix (option A — remove the impl to enforce explicit handling):**
```rust
// Remove the `From<AsyncValidationError> for ActionError` impl entirely.
// Callers must match on the enum variant:
//   Err(AsyncValidationError::Validation(e)) => e.with_old_input(&data).into_action_error(url),
//   Err(AsyncValidationError::Infra(fe)) => return Err(fe.into()),
```

**Fix (option B — keep but document the footgun prominently):**
```rust
impl From<AsyncValidationError> for crate::http::action::ActionError {
    /// LOSSY: discards field errors and old input. Only use when the caller
    /// has already extracted and flashed `ValidationError` separately.
    /// Prefer matching on the enum variants directly.
    fn from(e: AsyncValidationError) -> Self {
        match e {
            AsyncValidationError::Validation(_) => {
                crate::http::action::ActionError::validation_failed("/")
            }
            AsyncValidationError::Infra(fe) => crate::http::action::ActionError::from(fe),
        }
    }
}
```

---

## Info

### IN-01: `rules()` silently overwrites previously added sync rules for a field; `rule()` does not

**File:** `framework/src/validation/async_validator.rs:136-139`
**Issue:** `rules()` calls `self.sync_rules.insert(field.into(), rules)`, which replaces any rules already registered for that field via prior `rule()` or `rules()` calls. `rule()` (line 115-122) correctly appends. Calling `.rule("x", r1).rules("x", vec![r2])` silently drops `r1`. The same inconsistency exists in `Validator` (the sync counterpart), but it is more likely to be triggered in `AsyncValidator` where callers often mix `.rule()` and `.rules()` in the same chain.

**Fix:** Make `rules()` append rather than overwrite:
```rust
pub fn rules(mut self, field: impl Into<String>, rules: Vec<Box<dyn Rule>>) -> Self {
    self.sync_rules
        .entry(field.into())
        .or_default()
        .extend(rules);
    self
}
```

---

### IN-02: DB fixture `seed_widget` uses string interpolation for SQL in test helpers

**File:** `framework/tests/async_rule_fixture.rs:50-51` and `framework/src/validation/rules_async.rs:222-224` (inline copy), `framework/src/validation/async_validator.rs:453-460` (inline copy)
**Issue:** `seed_widget` interpolates `id` and `slug` directly into a SQL string using `format!()`. Although these are test-only paths and `slug` is a developer-controlled fixture string, the pattern is a bad habit that contradicts the `Statement::from_sql_and_values` pattern used in the production code just above it. If a test fixture slug ever contains a single quote (e.g., `"it's"`) the statement will fail unexpectedly.

**Fix:** Use parameterised statements in the fixture:
```rust
db.execute(Statement::from_sql_and_values(
    db.get_database_backend(),
    "INSERT INTO widgets (id, slug) VALUES (?, ?)",
    [sea_orm::Value::BigInt(Some(id)), sea_orm::Value::String(Some(Box::new(slug.to_string())))],
))
.await
.expect("seed widget row");
```
(Use `$1`/`$2` for Postgres backends — or branch on `db.get_database_backend()` as the production code does.)

---

### IN-03: `async_rule()` has no bulk `async_rules()` counterpart; asymmetry with sync side

**File:** `framework/src/validation/async_validator.rs:144-150`
**Issue:** The sync side exposes both `rule()` (single) and `rules()` (bulk). The async side exposes only `async_rule()` (single). This asymmetry means callers cannot pass a `Vec<Box<dyn AsyncRule>>` from a helper function. Adding `async_rules()` would complete the API surface and allow consistent macro-style ergonomics.

**Fix:** Add:
```rust
pub fn async_rules(mut self, field: impl Into<String>, rules: Vec<Box<dyn AsyncRule>>) -> Self {
    self.async_rules
        .entry(field.into())
        .or_default()
        .extend(rules);
    self
}
```

---

_Reviewed: 2026-06-09_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
