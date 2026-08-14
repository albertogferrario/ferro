//! Framework facade for offload result persistence and retrieval.
//!
//! Composes [`ferro_projection::snapshot_write`] / [`ferro_projection::snapshot_read`]
//! into typed envelope helpers so macro-generated code and worker hooks can
//! persist and retrieve offloaded method results via `::ferro::offload::*` paths
//! (D-11, per the 244/245 convention that generated code emits only `::ferro::*`).
//!
//! # Envelope shape (D-07)
//!
//! ```json
//! { "status": "completed", "value": <T-as-JSON> }
//! { "status": "failed",    "error": "<message>"  }
//! { "status": "pending" }
//! ```
//!
//! A `()` output type serializes to JSON `null`, so a completed-unit result is
//! `{"status":"completed","value":null}`. `serde_json::from_value::<()>(null)`
//! succeeds — no special case is needed (Pitfall 3 verified by the unit test).
//!
//! # Error handling
//!
//! `persist_result` and `persist_error` return their error rather than panicking
//! so the worker hook can log via `tracing::warn!` and continue (Pitfall 5 / T-246-02).
//! The error type is [`ferro_projection::ProjectionError`] — callers must NOT fail
//! the job on a persistence error.
//!
//! # Security notes (T-246-02 / T-247-info-disclosure)
//!
//! The failed envelope stores the raw error string (`Display` form). In Phase 246
//! the envelope is retrieved in-process only. Phase 247 provides
//! [`read_result_redacted`] as the client-facing sanitized read-back: it replaces
//! the raw error with a fixed `"terminal error"` marker. The raw error remains only
//! in the snapshot and the worker logs for authorized/server-side retrieval via
//! [`read_result`].

use ferro_projection::{snapshot_read, snapshot_write, ProjectionError, ProjectionKey};
use ferro_queue::OffloadSerializable;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

/// Reserved projection name for all offload results (D-13).
///
/// The 247 broadcast channel derives as `projection.offload.result.{handle}`
/// from `(OFFLOAD_PROJECTION_NAME, handle_key)`, so this choice also fixes the
/// subscription key Phase 247 will use.
pub const OFFLOAD_PROJECTION_NAME: &str = "offload.result";

/// Typed result of a completed or terminally failed offloaded call (D-07).
///
/// Internally tagged via `{"status":"completed","value":<T>}` /
/// `{"status":"failed","error":"<msg>"}`. `T: OffloadSerializable` guarantees
/// the `value` field round-trips through JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum OffloadResult<T> {
    /// The method returned `Ok(value)` (or a non-`Result` return value).
    Completed {
        /// The deserialized output value.
        value: T,
    },
    /// Retries exhausted or a terminal panic; the method never produced a result.
    Failed {
        /// Display-stringified error message from the worker.
        error: String,
    },
    /// The work is enqueued but not yet finished (D-07). Written at enqueue by
    /// [`persist_pending`]; distinguishes an unknown handle (no snapshot →
    /// [`read_result`] returns `None`) from work in flight (this variant).
    /// Carries no value.
    Pending,
}

/// Persist a completed offload result under the handle key.
///
/// Writes `{"status":"completed","value":<T-as-JSON>}` into the
/// `projection_snapshots` table at `(OFFLOAD_PROJECTION_NAME, handle_key)`.
///
/// # Non-fatal contract
///
/// Returns the [`ProjectionError`] rather than panicking. The caller (Plan 04
/// worker hook) must `tracing::warn!` on error and continue — do NOT fail the job.
///
/// # Errors
///
/// [`ProjectionError::Json`] if `value` cannot be serialized (should not occur
/// for well-behaved `OffloadSerializable` types).
/// [`ProjectionError::Db`] on a SeaORM error.
pub async fn persist_result<T: OffloadSerializable>(
    handle_key: &str,
    value: &T,
    db: &DatabaseConnection,
) -> Result<(), ProjectionError> {
    let state = serde_json::to_value(value).map_err(ProjectionError::from)?;
    let envelope = serde_json::json!({ "status": "completed", "value": state });
    snapshot_write(
        db,
        OFFLOAD_PROJECTION_NAME,
        &ProjectionKey::new(handle_key),
        envelope,
    )
    .await
}

/// Persist a terminal-error envelope under the handle key.
///
/// Writes `{"status":"failed","error":"<msg>"}` into the
/// `projection_snapshots` table at `(OFFLOAD_PROJECTION_NAME, handle_key)`.
///
/// # Non-fatal contract
///
/// Returns the [`ProjectionError`] rather than panicking; caller must log and continue.
///
/// # Errors
///
/// [`ProjectionError::Db`] on a SeaORM error.
pub async fn persist_error(
    handle_key: &str,
    error: &str,
    db: &DatabaseConnection,
) -> Result<(), ProjectionError> {
    let envelope = serde_json::json!({ "status": "failed", "error": error });
    snapshot_write(
        db,
        OFFLOAD_PROJECTION_NAME,
        &ProjectionKey::new(handle_key),
        envelope,
    )
    .await
}

/// Read back a result by handle key, deserialized to [`OffloadResult<T>`].
///
/// Returns `Ok(None)` when no snapshot exists yet — the handle is either
/// unknown or the work has not finished (D-08: "not done" is indistinguishable
/// from "unknown handle" in Phase 246; Phase 247 adds a pending marker).
///
/// # Errors
///
/// [`ProjectionError::Db`] on a SeaORM error.
/// [`ProjectionError::Json`] if the stored envelope cannot be deserialized into
/// `OffloadResult<T>` (indicates schema mismatch or data corruption).
pub async fn read_result<T: OffloadSerializable>(
    handle_key: &str,
    db: &DatabaseConnection,
) -> Result<Option<OffloadResult<T>>, ProjectionError> {
    match snapshot_read(db, OFFLOAD_PROJECTION_NAME, &ProjectionKey::new(handle_key)).await? {
        None => Ok(None),
        Some(v) => {
            let envelope: OffloadResult<T> =
                serde_json::from_value(v).map_err(ProjectionError::from)?;
            Ok(Some(envelope))
        }
    }
}

/// Persist a pre-serialized success value under the handle key.
///
/// Used by the worker hook, which already has the `serde_json::Value` from
/// `handle_with_value()` — avoids re-serializing a value that was already
/// serialized inside the handler closure. Writes the same
/// `{"status":"completed","value":<v>}` envelope as [`persist_result`].
///
/// # Non-fatal contract
///
/// Returns [`ProjectionError`] on failure; callers (the hook closure) must
/// log via `tracing::warn!` and continue — do NOT fail the job.
pub async fn persist_result_raw(
    handle_key: &str,
    value: serde_json::Value,
    db: &DatabaseConnection,
) -> Result<(), ProjectionError> {
    let envelope = serde_json::json!({ "status": "completed", "value": value });
    snapshot_write(
        db,
        OFFLOAD_PROJECTION_NAME,
        &ProjectionKey::new(handle_key),
        envelope,
    )
    .await
}

/// Register the offload-result persistence hook with `ferro-queue`.
///
/// Called once at framework bootstrap. The hook closure calls
/// [`persist_result_raw`] or [`persist_error`] based on the outcome, and
/// logs via `tracing::warn!` on error without changing the job's outcome
/// (T-246-05 non-fatal contract).
///
/// `ferro-queue` must not depend on `ferro-projection` (D-11); this
/// registration is the injection point that resolves that constraint.
pub fn register_offload_hooks() {
    ferro_queue::register_offload_result_hook(|key, outcome, db| {
        Box::pin(async move {
            let res = match outcome {
                Ok(value) => persist_result_raw(&key, value, db).await,
                Err(msg) => persist_error(&key, &msg, db).await,
            };
            if let Err(e) = res {
                tracing::warn!(
                    handle_key = %key,
                    error = %e,
                    "offload result persist failed — result not stored"
                );
            }
        })
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;
    use sea_orm_migration::MigratorTrait;

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(ferro_projection::CreateProjectionSnapshotsTable)]
        }
    }

    async fn fresh_db() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:").await.expect("connect");
        TestMigrator::up(&conn, None).await.expect("migrate");
        conn
    }

    /// A completed String result round-trips through persist_result → read_result.
    #[tokio::test]
    async fn offload_result_completed_round_trip() {
        let db = fresh_db().await;
        persist_result("k1", &"hello".to_string(), &db)
            .await
            .expect("persist_result");

        let result = read_result::<String>("k1", &db).await.expect("read_result");

        match result {
            Some(OffloadResult::Completed { value }) => {
                assert_eq!(value, "hello");
            }
            other => panic!("expected Completed, got {other:?}"),
        }
    }

    /// A failed envelope round-trips through persist_error → read_result.
    #[tokio::test]
    async fn offload_result_failed_round_trip() {
        let db = fresh_db().await;
        persist_error("k2", "boom", &db)
            .await
            .expect("persist_error");

        let result = read_result::<String>("k2", &db).await.expect("read_result");

        match result {
            Some(OffloadResult::Failed { error }) => {
                assert_eq!(error, "boom");
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    /// Reading a never-written handle returns Ok(None).
    #[tokio::test]
    async fn offload_result_absent_is_none() {
        let db = fresh_db().await;
        let result = read_result::<String>("nope", &db)
            .await
            .expect("read_result");
        assert!(result.is_none());
    }

    /// `{"status":"pending"}` deserializes to `OffloadResult::Pending`; existing
    /// completed/failed envelopes are unaffected (backward-compat, A3 verification).
    #[tokio::test]
    async fn offload_result_pending_round_trip() {
        // New variant round-trips.
        let pending = serde_json::from_str::<OffloadResult<()>>(r#"{"status":"pending"}"#)
            .expect("deserialize pending");
        assert!(
            matches!(pending, OffloadResult::Pending),
            "expected Pending, got {pending:?}"
        );

        // Backward-compat: completed envelope still deserializes correctly.
        let completed =
            serde_json::from_str::<OffloadResult<String>>(r#"{"status":"completed","value":"x"}"#)
                .expect("deserialize completed");
        assert!(
            matches!(completed, OffloadResult::Completed { ref value } if value == "x"),
            "expected Completed {{ value: \"x\" }}, got {completed:?}"
        );

        // Backward-compat: failed envelope still deserializes correctly.
        let failed =
            serde_json::from_str::<OffloadResult<String>>(r#"{"status":"failed","error":"boom"}"#)
                .expect("deserialize failed");
        assert!(
            matches!(failed, OffloadResult::Failed { ref error } if error == "boom"),
            "expected Failed {{ error: \"boom\" }}, got {failed:?}"
        );
    }

    /// Unit output () round-trips via JSON null (Pitfall 3 / A3 verification).
    ///
    /// `serde_json::to_value(&())` yields `Value::Null`.
    /// The stored envelope is `{"status":"completed","value":null}`.
    /// `serde_json::from_value::<()>(Value::Null)` succeeds — no special case needed.
    #[tokio::test]
    async fn offload_result_unit_output() {
        let db = fresh_db().await;
        persist_result("k3", &(), &db)
            .await
            .expect("persist_result unit");

        let result = read_result::<()>("k3", &db)
            .await
            .expect("read_result unit");

        match result {
            Some(OffloadResult::Completed { value: () }) => {}
            other => panic!("expected Completed {{ value: () }}, got {other:?}"),
        }
    }
}
