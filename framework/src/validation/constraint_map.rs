//! Defensive mapping of DB UNIQUE-constraint violations to field-level
//! [`ValidationError`]s.
//!
//! A [`ConstraintMap`] is registered at the call site with the constraint names
//! and corresponding field/message pairs. When an [`sea_orm::DbErr`] is a UNIQUE
//! violation that matches a registered entry, [`ConstraintMap::try_map`] returns
//! `Ok(ValidationError)` carrying the entry's field and message. Any
//! non-matching `DbErr` — including non-UNIQUE errors and unregistered UNIQUE
//! violations — is returned as `Err(original_dberr)` unchanged so the caller's
//! `?` reaches the existing [`From<sea_orm::DbErr> for ActionError`] passthrough.
//!
//! All constraint/field/message strings are consumer-owned. The `framework`
//! crate holds no application-specific literals.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::{ConstraintMap, MapConstraintExt};
//!
//! let map = ConstraintMap::new()
//!     .on("pages_slug_unique", "slug", "has already been taken")
//!     .sqlite("pages.slug");
//!
//! // In a handler:
//! let page = new_page.insert(db).await
//!     .map_constraint(&map, &data, "/pages/new")?;
//! ```

use sea_orm::{DbErr, RuntimeErr, SqlxError};
use sea_orm::error::SqlErr;

use crate::validation::error::ValidationError;

/// A single registered mapping entry.
#[derive(Clone)]
struct ConstraintEntry {
    /// Postgres structured constraint name — primary `.on()` key.
    pg_name: String,
    /// SQLite `table.column` discriminator (set via `.sqlite()`).
    sqlite_key: Option<String>,
    /// Logical field the error attaches to.
    field: String,
    /// User-visible message.
    message: String,
}

/// Consumer-registered map from DB UNIQUE-constraint violations to field-level
/// validation errors. Construct per call site (cheap; no global state).
///
/// # First-match semantics
///
/// Entries are matched in registration order. The first entry whose Postgres
/// constraint name or SQLite `table.column` key matches the violation wins.
/// Register the most specific entries first when multiple UNIQUE constraints
/// exist on one table.
///
/// # Postgres vs SQLite identity
///
/// - **Postgres:** matched by constraint name via the `DatabaseError::constraint()`
///   trait method (protocol field `'n'`). No message-string parsing.
/// - **SQLite:** matched by `table.column` extracted from the error message
///   (`"UNIQUE constraint failed: table.column"`). SQLite does not expose
///   structured constraint names so the message token is the reliable identifier.
///
/// A single registration can cover both backends by chaining `.sqlite("t.c")`
/// after `.on(...)`.
#[derive(Clone, Default)]
pub struct ConstraintMap {
    entries: Vec<ConstraintEntry>,
}

impl ConstraintMap {
    /// Create an empty `ConstraintMap`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a UNIQUE constraint name → field/message mapping.
    ///
    /// `pg_constraint` is the Postgres constraint name (the structured value
    /// in the DB error protocol, e.g. `"pages_slug_unique"`). It is also used
    /// as the logical entry key when chaining `.sqlite(...)`.
    ///
    /// Optionally chain `.sqlite("table.column")` immediately after to add a
    /// SQLite discriminator to the same entry so one registration covers both
    /// backends.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// ConstraintMap::new()
    ///     .on("pages_slug_unique", "slug", "has already been taken")
    ///     .sqlite("pages.slug");
    /// ```
    pub fn on(
        mut self,
        pg_constraint: impl Into<String>,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        self.entries.push(ConstraintEntry {
            pg_name: pg_constraint.into(),
            sqlite_key: None,
            field: field.into(),
            message: message.into(),
        });
        self
    }

    /// Add a SQLite `table.column` discriminator to the LAST registered entry.
    ///
    /// Must be chained immediately after `.on(...)`. No-op when called without
    /// a prior `.on()` call on this map.
    ///
    /// SQLite does not expose structured constraint names; the `table.column`
    /// token parsed from the error message is the reliable identifier for SQLite
    /// backends (dev/CI environments).
    pub fn sqlite(mut self, table_col: impl Into<String>) -> Self {
        if let Some(last) = self.entries.last_mut() {
            last.sqlite_key = Some(table_col.into());
        }
        self
    }

    /// Map a DB UNIQUE-constraint violation to a field-level [`ValidationError`].
    ///
    /// Returns `Ok(ValidationError)` when `err` is a UNIQUE violation that
    /// matches a registered entry. Returns `Err(err)` **unchanged** (by move)
    /// when:
    /// - `err` is not a UNIQUE violation, or
    /// - `err` is a UNIQUE violation but no registered entry matches.
    ///
    /// The returned `ValidationError` carries the entry's `field` and `message`
    /// and composes with `.with_old_input(&data).into_action_error(url)` exactly
    /// like a Phase 190 async-rule failure.
    ///
    /// # Safety contract (SC2)
    ///
    /// This method NEVER swallows a `DbErr`. A non-matching error is returned
    /// unchanged so the caller's `?` reaches `From<DbErr> for ActionError`.
    pub fn try_map(&self, err: DbErr) -> Result<ValidationError, DbErr> {
        // Step 1: portable violation-type gate (borrows err, does NOT consume it).
        // All non-UNIQUE DbErr variants fall through immediately unchanged.
        if !matches!(err.sql_err(), Some(SqlErr::UniqueConstraintViolation(_))) {
            return Err(err);
        }

        // Step 2a: Postgres constraint name via the DatabaseError trait method.
        // `e.constraint()` dispatches to PgDatabaseError::constraint() at runtime
        // (returns protocol field 'n'). No downcast, no #[cfg] guard needed.
        let pg_name: Option<String> = match &err {
            DbErr::Exec(RuntimeErr::SqlxError(SqlxError::Database(e)))
            | DbErr::Query(RuntimeErr::SqlxError(SqlxError::Database(e))) => {
                e.constraint().map(ToOwned::to_owned)
            }
            _ => None,
        };

        // Step 2b: SQLite table.column from message string.
        // sql_err() borrows err again — legal because err has not been moved.
        // Defensive parse: split on ": " and take token after it; None on any
        // unexpected format (never panics — T-191-01 mitigation).
        let sqlite_key: Option<String> = match err.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(msg)) => {
                // "UNIQUE constraint failed: table.column" → "table.column"
                msg.split(": ").nth(1).map(|s| s.trim().to_owned())
            }
            _ => None,
        };

        // Step 3: first-match entry lookup.
        for entry in &self.entries {
            let pg_hit = pg_name
                .as_deref()
                .map(|c| c == entry.pg_name)
                .unwrap_or(false);
            let sqlite_hit = sqlite_key
                .as_deref()
                .zip(entry.sqlite_key.as_deref())
                .map(|(k, r)| k == r)
                .unwrap_or(false);
            if pg_hit || sqlite_hit {
                let mut ve = ValidationError::new();
                ve.add(&entry.field, &entry.message);
                return Ok(ve);
            }
        }

        // Step 4: no entry matched — fall through unchanged (SC2).
        Err(err)
    }
}

/// Extension trait on `Result<T, DbErr>` for ergonomic call-site mapping.
///
/// Eliminates closure ladders at the write site: instead of chaining
/// `.map_err(|e| map.try_map(e).map(...).unwrap_or_else(...))`, use:
///
/// ```rust,ignore
/// let page = new_page.insert(db).await
///     .map_constraint(&map, &data, "/pages/new")?;
/// ```
///
/// On a matching UNIQUE violation the `ValidationError` is flashed into the
/// session and an [`crate::http::action::ActionError`] configured to redirect
/// to `url` is returned (identical to the Phase 190 surfacing chain).
///
/// On any non-matching error, `From<DbErr> for ActionError` is applied —
/// the existing passthrough — so no error is ever swallowed.
pub trait MapConstraintExt<T> {
    /// Map a DB UNIQUE-constraint violation to a field-level error and redirect.
    ///
    /// See the trait documentation for full semantics.
    fn map_constraint(
        self,
        map: &ConstraintMap,
        data: &serde_json::Value,
        url: impl Into<String>,
    ) -> Result<T, crate::http::action::ActionError>;
}

impl<T> MapConstraintExt<T> for Result<T, DbErr> {
    fn map_constraint(
        self,
        map: &ConstraintMap,
        data: &serde_json::Value,
        url: impl Into<String>,
    ) -> Result<T, crate::http::action::ActionError> {
        self.map_err(|err| match map.try_map(err) {
            Ok(ve) => ve.with_old_input(data).into_action_error(url),
            Err(original) => crate::http::action::ActionError::from(original),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Non-DB unit tests — no DB singleton needed, no #[serial] required.
    // These cover SC2 (passthrough contract) and builder correctness.
    // DB-backed SQLite identity and TOCTOU simulation tests live in
    // framework/tests/constraint_map_integration.rs (Plan 02).

    #[test]
    fn non_unique_dberr_passes_through_unchanged() {
        // SC2: a non-UNIQUE DbErr must be returned UNCHANGED.
        let map = ConstraintMap::new()
            .on("some_constraint", "field", "message")
            .sqlite("t.col");
        let err = DbErr::Custom("boom".to_string());
        match map.try_map(err) {
            Err(DbErr::Custom(msg)) => assert_eq!(msg, "boom"),
            other => panic!("expected Err(DbErr::Custom(\"boom\")), got {other:?}"),
        }
    }

    #[test]
    fn empty_map_passes_through_any_dberr_unchanged() {
        // An empty ConstraintMap must never swallow any error.
        let map = ConstraintMap::new();
        let err = DbErr::Custom("any error".to_string());
        match map.try_map(err) {
            Err(DbErr::Custom(msg)) => assert_eq!(msg, "any error"),
            other => panic!("expected Err(DbErr::Custom), got {other:?}"),
        }
    }

    #[test]
    fn builder_chains_on_and_sqlite_without_panic() {
        // Verify .on(...).sqlite(...) builds without panic and the map is reusable.
        let map = ConstraintMap::new()
            .on("a_constraint", "field_a", "msg a")
            .sqlite("table_a.col_a")
            .on("b_constraint", "field_b", "msg b");

        // Two entries registered; passing a non-UNIQUE error exercises the code
        // path without needing a DB connection.
        let err = DbErr::Custom("test".to_string());
        assert!(map.try_map(err).is_err(), "non-UNIQUE must fall through");

        // Clone the map to verify Clone is derived correctly.
        let map2 = map.clone();
        let err2 = DbErr::Custom("test2".to_string());
        assert!(map2.try_map(err2).is_err());
    }

    #[test]
    fn sqlite_no_op_without_prior_on() {
        // .sqlite() without a prior .on() must be a no-op (not panic).
        let map = ConstraintMap::new().sqlite("table.col");
        assert!(map.entries.is_empty());
    }
}
