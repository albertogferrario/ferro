//! Async validation rules.

// rules_async is declared in a private module; pub use re-exports are wired in
// Plan 04. Suppress dead_code until the pub use chain is in place.
#![allow(dead_code)]

use async_trait::async_trait;
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use serde_json::Value;

use crate::database::DB;
use crate::validation::translate_validation;

use super::async_rule::AsyncRule;

/// Validates that a field value is unique in `table`.`col`.
///
/// # Trust boundary (T-190-01)
/// `table` and `col` are developer-controlled identifiers from handler source,
/// never end-user input. They cannot be SQL-bound, so they are interpolated;
/// each is validated against `[A-Za-z0-9_]` and double-quoted. The *value*
/// being checked (and any excluded id) is always passed as a bound SQL
/// parameter, never interpolated.
///
/// # Scope limitation
/// This rule checks system-wide uniqueness (one `col` across the whole table).
/// It does NOT support scoped/per-tenant uniqueness (e.g. unique within a
/// `tenant_id`). For tenant-scoped tables, a `.where_eq(col, val)` predicate
/// is a planned follow-up — do not use bare `unique` on a tenant-scoped column.
pub struct Unique {
    table: String,
    col: String,
    ignore: Option<(String, sea_orm::Value)>, // (pk_col, pk_value)
}

/// Creates a uniqueness validation rule for `table`.`col`.
///
/// Use `.ignore(id)` to exclude the current record when validating on edit
/// forms (prevents the record from failing its own existing value).
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::unique;
///
/// // Create form: reject any duplicate slug.
/// unique("articles", "slug")
///
/// // Edit form: allow the record with id=42 to keep its current slug.
/// unique("articles", "slug").ignore(42_i64)
/// ```
pub fn unique(table: impl Into<String>, col: impl Into<String>) -> Unique {
    Unique {
        table: table.into(),
        col: col.into(),
        ignore: None,
    }
}

impl Unique {
    /// Exclude the record with this PK value from the uniqueness check.
    ///
    /// Uses the default PK column `"id"`. Call this on edit forms to allow
    /// the current record to keep its unchanged value.
    pub fn ignore(mut self, id: impl Into<sea_orm::Value>) -> Self {
        self.ignore = Some(("id".to_string(), id.into()));
        self
    }

    /// Exclude the record using an explicit PK column name.
    ///
    /// Use when the primary key column is not named `"id"`.
    pub fn ignore_on(mut self, pk_col: impl Into<String>, id: impl Into<sea_orm::Value>) -> Self {
        self.ignore = Some((pk_col.into(), id.into()));
        self
    }

    /// Validate that an identifier is safe to interpolate into SQL.
    ///
    /// Accepts only `[A-Za-z0-9_]` characters (non-empty). Rejects anything
    /// that could be used for SQL injection via identifier manipulation.
    fn validate_identifier(ident: &str) -> Result<(), String> {
        if !ident.is_empty() && ident.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            Ok(())
        } else {
            Err(format!("Invalid SQL identifier: {ident:?}"))
        }
    }

    /// Wrap an identifier in ANSI double-quotes for safe interpolation.
    fn quote_ident(ident: &str) -> String {
        format!("\"{ident}\"")
    }

    /// Build the per-backend COUNT SQL string (table/col already double-quoted).
    ///
    /// SQLite and MySQL use `?` positional placeholders; Postgres uses `$1`/`$2`.
    /// `Statement::from_sql_and_values` does NOT translate placeholders — the SQL
    /// is passed verbatim to the sqlx driver.
    fn build_sql(&self, backend: DatabaseBackend, table: &str, col: &str) -> String {
        match (&self.ignore, backend) {
            (None, DatabaseBackend::Postgres) => {
                format!("SELECT COUNT(*) AS count FROM {table} WHERE {col} = $1")
            }
            (None, _) => {
                format!("SELECT COUNT(*) AS count FROM {table} WHERE {col} = ?")
            }
            (Some((pk_col, _)), DatabaseBackend::Postgres) => {
                let pk = Self::quote_ident(pk_col);
                format!("SELECT COUNT(*) AS count FROM {table} WHERE {col} = $1 AND {pk} <> $2")
            }
            (Some((pk_col, _)), _) => {
                let pk = Self::quote_ident(pk_col);
                format!("SELECT COUNT(*) AS count FROM {table} WHERE {col} = ? AND {pk} <> ?")
            }
        }
    }
}

#[async_trait]
impl AsyncRule for Unique {
    async fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        // 1. Identifier guards (T-190-01) — runs before any DB access.
        Self::validate_identifier(&self.table)
            .map_err(|e| format!("Unique rule misconfigured: {e}"))?;
        Self::validate_identifier(&self.col)
            .map_err(|e| format!("Unique rule misconfigured: {e}"))?;
        if let Some((ref pk_col, _)) = self.ignore {
            Self::validate_identifier(pk_col)
                .map_err(|e| format!("Unique rule misconfigured: {e}"))?;
        }

        let table = Self::quote_ident(&self.table);
        let col = Self::quote_ident(&self.col);

        // 2. DB singleton — infra failure → __infra_error__ sentinel (D-12).
        let db = DB::connection().map_err(|e| format!("__infra_error__: {e}"))?;
        let backend = db.get_database_backend();

        // 3. Build per-backend SQL (placeholders are NOT auto-translated).
        let sql = self.build_sql(backend, &table, &col);
        let values: Vec<sea_orm::Value> = match &self.ignore {
            None => vec![json_value_to_sea_value(value)],
            Some((_, pk_val)) => vec![json_value_to_sea_value(value), pk_val.clone()],
        };

        // 4. Execute — query error → __infra_error__ sentinel (never swallow DbErr).
        let stmt = Statement::from_sql_and_values(backend, sql, values);
        let row = db
            .query_one(stmt)
            .await
            .map_err(|e| format!("__infra_error__: {e}"))?;
        let count: i64 = row
            .and_then(|r| r.try_get::<i64>("", "count").ok())
            .unwrap_or(0);

        if count > 0 {
            Err(
                translate_validation("validation.unique", &[("attribute", field)])
                    .unwrap_or_else(|| format!("The {field} has already been taken.")),
            )
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "unique"
    }
}

/// Convert a `serde_json::Value` to a `sea_orm::Value` for use as a bound
/// SQL parameter. Used internally by `Unique::validate`.
pub(crate) fn json_value_to_sea_value(v: &serde_json::Value) -> sea_orm::Value {
    match v {
        serde_json::Value::String(s) => sea_orm::Value::String(Some(Box::new(s.clone()))),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                sea_orm::Value::BigInt(Some(i))
            } else if let Some(f) = n.as_f64() {
                sea_orm::Value::Double(Some(f))
            } else {
                sea_orm::Value::String(Some(Box::new(n.to_string())))
            }
        }
        serde_json::Value::Bool(b) => sea_orm::Value::Bool(Some(*b)),
        _ => sea_orm::Value::String(Some(Box::new(v.to_string()))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::DatabaseBackend;
    use serde_json::json;
    use serial_test::serial;

    // -----------------------------------------------------------------------
    // Inline DB fixture (mirrors framework/tests/async_rule_fixture.rs).
    // Uses crate:: paths — valid in lib unit tests.
    // Tests touching the DB singleton MUST be annotated #[serial].
    // -----------------------------------------------------------------------

    async fn init_test_db() {
        use crate::database::{DatabaseConfig, DB};
        use sea_orm::{ConnectionTrait, Statement};
        let config = DatabaseConfig::builder().url("sqlite::memory:").build();
        DB::init_with(config).await.expect("init in-memory sqlite");
        let db = DB::connection().expect("connection after init");
        db.execute(Statement::from_string(
            db.get_database_backend(),
            "CREATE TABLE IF NOT EXISTS widgets (id INTEGER PRIMARY KEY, slug TEXT)".to_owned(),
        ))
        .await
        .expect("create widgets scratch table");
    }

    async fn seed_widget(id: i64, slug: &str) {
        use crate::database::DB;
        use sea_orm::{ConnectionTrait, Statement};
        let db = DB::connection().expect("connection for seed_widget");
        db.execute(Statement::from_string(
            db.get_database_backend(),
            format!("INSERT INTO widgets (id, slug) VALUES ({id}, '{slug}')"),
        ))
        .await
        .expect("seed widget row");
    }

    // -------------------------------------------------------------------------
    // Pure unit tests (no DB) — Task 1
    // -------------------------------------------------------------------------

    #[test]
    fn validate_identifier_accepts_valid_names() {
        assert!(Unique::validate_identifier("slug").is_ok());
        assert!(Unique::validate_identifier("my_table").is_ok());
        assert!(Unique::validate_identifier("Table123").is_ok());
        assert!(Unique::validate_identifier("a").is_ok());
    }

    #[test]
    fn validate_identifier_rejects_invalid_names() {
        assert!(Unique::validate_identifier("").is_err());
        assert!(Unique::validate_identifier("a;b").is_err());
        assert!(Unique::validate_identifier("a b").is_err());
        assert!(Unique::validate_identifier("a.b").is_err());
        assert!(Unique::validate_identifier("a-b").is_err());
        assert!(Unique::validate_identifier("a'b").is_err());
    }

    #[test]
    fn quote_ident_wraps_in_double_quotes() {
        assert_eq!(Unique::quote_ident("slug"), "\"slug\"");
        assert_eq!(Unique::quote_ident("my_col"), "\"my_col\"");
    }

    #[test]
    fn ignore_sets_default_id_pk() {
        let u = unique("widgets", "slug").ignore(5_i64);
        let (pk_col, _) = u.ignore.expect("ignore should be set");
        assert_eq!(pk_col, "id");
    }

    #[test]
    fn ignore_on_sets_custom_pk() {
        let u = unique("widgets", "slug").ignore_on("uuid", "abc");
        let (pk_col, _) = u.ignore.expect("ignore should be set");
        assert_eq!(pk_col, "uuid");
    }

    #[test]
    fn json_value_to_sea_value_string() {
        let v = json!("hello");
        let sv = json_value_to_sea_value(&v);
        assert!(matches!(sv, sea_orm::Value::String(Some(s)) if s.as_str() == "hello"));
    }

    #[test]
    fn json_value_to_sea_value_integer() {
        let v = json!(7_i64);
        let sv = json_value_to_sea_value(&v);
        assert!(matches!(sv, sea_orm::Value::BigInt(Some(7))));
    }

    #[test]
    fn json_value_to_sea_value_bool() {
        let v = json!(true);
        let sv = json_value_to_sea_value(&v);
        assert!(matches!(sv, sea_orm::Value::Bool(Some(true))));
    }

    #[test]
    fn json_value_to_sea_value_null_uses_string_fallback() {
        let v = json!(null);
        let sv = json_value_to_sea_value(&v);
        assert!(matches!(sv, sea_orm::Value::String(_)));
    }

    // -------------------------------------------------------------------------
    // Per-backend SQL string tests (no DB) — Task 2
    // -------------------------------------------------------------------------

    #[test]
    fn unique_postgres_sql_uses_dollar_placeholders() {
        let u = unique("widgets", "slug");
        let sql = u.build_sql(DatabaseBackend::Postgres, "\"widgets\"", "\"slug\"");
        assert!(sql.contains("$1"), "expected $1 in: {sql}");
        assert!(
            !sql.contains('?'),
            "should not have ? in postgres sql: {sql}"
        );
    }

    #[test]
    fn unique_postgres_sql_with_ignore_uses_dollar_two() {
        let u = unique("widgets", "slug").ignore(1_i64);
        let sql = u.build_sql(DatabaseBackend::Postgres, "\"widgets\"", "\"slug\"");
        assert!(sql.contains("$1"), "expected $1 in: {sql}");
        assert!(sql.contains("$2"), "expected $2 in: {sql}");
    }

    #[test]
    fn unique_sqlite_sql_uses_question_placeholder() {
        let u = unique("widgets", "slug");
        let sql = u.build_sql(DatabaseBackend::Sqlite, "\"widgets\"", "\"slug\"");
        assert!(sql.contains('?'), "expected ? in sqlite sql: {sql}");
        assert!(
            !sql.contains("$1"),
            "should not have $1 in sqlite sql: {sql}"
        );
    }

    // -------------------------------------------------------------------------
    // Identifier-guard test (no DB access) — Task 2
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn unique_rejects_bad_identifier_before_db() {
        // This test must NOT call init_test_db — the guard must short-circuit
        // before any DB access is attempted.
        let data = json!({});
        let result = unique("bad;name", "slug")
            .validate("slug", &json!("value"), &data)
            .await;
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.starts_with("Unique rule misconfigured"),
            "expected 'Unique rule misconfigured' prefix, got: {msg}"
        );
    }

    #[tokio::test]
    async fn unique_rejects_bad_column_identifier_before_db() {
        let data = json!({});
        let result = unique("widgets", "bad col")
            .validate("slug", &json!("value"), &data)
            .await;
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.starts_with("Unique rule misconfigured"));
    }

    // -------------------------------------------------------------------------
    // DB-backed tests — Task 2
    // -------------------------------------------------------------------------

    #[tokio::test]
    #[serial]
    async fn unique_detects_existing_value() {
        init_test_db().await;
        seed_widget(1, "taken").await;
        let data = json!({});
        let result = unique("widgets", "slug")
            .validate("slug", &json!("taken"), &data)
            .await;
        assert!(result.is_err(), "expected Err for duplicate slug");
        let msg = result.unwrap_err();
        // Must not be an infra error
        assert!(
            !msg.starts_with("__infra_error__"),
            "must not be an infra error: {msg}"
        );
    }

    #[tokio::test]
    #[serial]
    async fn unique_passes_on_no_match() {
        init_test_db().await;
        // No widget seeded — "free" is available.
        let data = json!({});
        let result = unique("widgets", "slug")
            .validate("slug", &json!("free"), &data)
            .await;
        assert!(result.is_ok(), "expected Ok for non-duplicate slug");
    }

    #[tokio::test]
    #[serial]
    async fn unique_ignore_excludes_self() {
        init_test_db().await;
        seed_widget(1, "taken").await;
        let data = json!({});

        // Excluding own row (id=1): "taken" should pass
        let result = unique("widgets", "slug")
            .ignore(1_i64)
            .validate("slug", &json!("taken"), &data)
            .await;
        assert!(result.is_ok(), "expected Ok when ignoring own row");

        // Excluding a different row (id=2): "taken" still exists, should fail
        let result = unique("widgets", "slug")
            .ignore(2_i64)
            .validate("slug", &json!("taken"), &data)
            .await;
        assert!(
            result.is_err(),
            "expected Err when ignoring a different row"
        );
    }

    #[tokio::test]
    #[serial]
    async fn unique_ignore_on_custom_pk() {
        init_test_db().await;
        // Create a second scratch table with a non-default PK column.
        let db = crate::database::DB::connection().expect("connection");
        db.execute(sea_orm::Statement::from_string(
            db.get_database_backend(),
            "CREATE TABLE IF NOT EXISTS items (uid INTEGER PRIMARY KEY, code TEXT)".to_owned(),
        ))
        .await
        .expect("create items scratch table");
        db.execute(sea_orm::Statement::from_string(
            db.get_database_backend(),
            "DELETE FROM items".to_owned(),
        ))
        .await
        .expect("clear items");
        db.execute(sea_orm::Statement::from_string(
            db.get_database_backend(),
            "INSERT INTO items (uid, code) VALUES (10, 'ABC')".to_owned(),
        ))
        .await
        .expect("seed item");

        let data = json!({});

        // Excluding own row via custom PK col "uid" = 10: "ABC" should pass
        let result = unique("items", "code")
            .ignore_on("uid", 10_i64)
            .validate("code", &json!("ABC"), &data)
            .await;
        assert!(
            result.is_ok(),
            "expected Ok when ignoring own row via custom PK"
        );

        // Excluding a different row (uid=99): "ABC" still exists, should fail
        let result = unique("items", "code")
            .ignore_on("uid", 99_i64)
            .validate("code", &json!("ABC"), &data)
            .await;
        assert!(
            result.is_err(),
            "expected Err when different row owns the value"
        );
    }
}
