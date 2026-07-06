//! Atomic pointer-flip for deployment promotion (dual-backend raw SQL).
//!
//! All dynamic values are bound via `Statement::from_sql_and_values` with
//! `Value::*` parameters — no string interpolation of caller-supplied data
//! (T-186-04).
//!
//! The `conn.begin()` call (CR-01) pins every statement to one pooled
//! connection, guaranteeing atomicity on SQLite even with a connection pool.

use chrono::Utc;
use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement, TransactionTrait, Value,
};

use crate::error::Error;

/// Atomically flip the `deployment_pointers` row for `owner_key` to point at
/// `deployment_id`.
///
/// - If no row exists for `owner_key`, INSERTs one (first promotion).
/// - Otherwise UPDATEs in place: the previous `deployment_id` is preserved in
///   `previous_deployment_id` before being overwritten.
///
/// Returns the previous `deployment_id`, or `None` on first promotion.
pub(crate) async fn promote(
    conn: &DatabaseConnection,
    owner_key: &str,
    deployment_id: i64,
) -> Result<Option<i64>, Error> {
    match conn.get_database_backend() {
        DatabaseBackend::Postgres => promote_postgres(conn, owner_key, deployment_id).await,
        DatabaseBackend::Sqlite => promote_sqlite(conn, owner_key, deployment_id).await,
        _ => Err(Error::UnsupportedBackend),
    }
}

// ---------------------------------------------------------------------------
// SQLite path
// ---------------------------------------------------------------------------

async fn promote_sqlite(
    conn: &DatabaseConnection,
    owner_key: &str,
    deployment_id: i64,
) -> Result<Option<i64>, Error> {
    let now_iso = Utc::now().to_rfc3339();

    // CR-01: conn.begin() checks out ONE physical connection from the pool and
    // pins every statement on this handle to it. Direct conn.execute() calls
    // can land on different pooled connections, breaking atomicity.
    let txn = conn.begin().await.map_err(Error::Db)?;

    // Upsert: INSERT new pointer row, or UPDATE existing one atomically.
    // SET expressions read pre-update values (SQL standard — on both backends
    // the right-hand side of SET reads the OLD column value, not the NEW one).
    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "INSERT INTO deployment_pointers \
         (owner_key, deployment_id, previous_deployment_id, updated_at) \
         VALUES (?1, ?2, NULL, ?3) \
         ON CONFLICT (owner_key) DO UPDATE SET \
           previous_deployment_id = deployment_id, \
           deployment_id = ?2, \
           updated_at = ?3 \
         RETURNING previous_deployment_id",
        [
            Value::String(Some(Box::new(owner_key.to_string()))),
            Value::BigInt(Some(deployment_id)),
            Value::String(Some(Box::new(now_iso))),
        ],
    );

    let row = match txn.query_one(stmt).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            let _ = txn.rollback().await;
            return Err(Error::custom(
                "promote: RETURNING yielded no row; pointer state unknown",
            ));
        }
        Err(e) => {
            let _ = txn.rollback().await;
            return Err(Error::Db(e));
        }
    };
    txn.commit().await.map_err(Error::Db)?;

    let previous_id = row
        .try_get_by::<Option<i64>, _>("previous_deployment_id")
        .map_err(|e| Error::custom(format!("promote: parse previous_deployment_id: {e}")))?;
    Ok(previous_id)
}

// ---------------------------------------------------------------------------
// Postgres path
// ---------------------------------------------------------------------------

async fn promote_postgres(
    conn: &DatabaseConnection,
    owner_key: &str,
    deployment_id: i64,
) -> Result<Option<i64>, Error> {
    // CR-01: same rationale as promote_sqlite.
    let txn = conn.begin().await.map_err(Error::Db)?;

    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "INSERT INTO deployment_pointers \
         (owner_key, deployment_id, previous_deployment_id, updated_at) \
         VALUES ($1, $2, NULL, NOW()) \
         ON CONFLICT (owner_key) DO UPDATE SET \
           previous_deployment_id = deployment_pointers.deployment_id, \
           deployment_id = $2, \
           updated_at = NOW() \
         RETURNING previous_deployment_id",
        [
            Value::String(Some(Box::new(owner_key.to_string()))),
            Value::BigInt(Some(deployment_id)),
        ],
    );

    let row = match txn.query_one(stmt).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            let _ = txn.rollback().await;
            return Err(Error::custom(
                "promote: RETURNING yielded no row; pointer state unknown",
            ));
        }
        Err(e) => {
            let _ = txn.rollback().await;
            return Err(Error::Db(e));
        }
    };
    txn.commit().await.map_err(Error::Db)?;

    let previous_id = row
        .try_get_by::<Option<i64>, _>("previous_deployment_id")
        .map_err(|e| Error::custom(format!("promote: parse previous_deployment_id: {e}")))?;
    Ok(previous_id)
}
