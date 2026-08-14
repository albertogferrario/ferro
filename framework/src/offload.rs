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
use std::sync::{Arc, OnceLock};

// ---------------------------------------------------------------------------
// ResolveError
// ---------------------------------------------------------------------------

/// Errors from the race-safe [`resolve`] helper.
#[derive(Debug, thiserror::Error)]
pub enum ResolveError {
    /// The projection snapshot read failed.
    #[error("projection read failed: {0}")]
    Projection(#[from] ProjectionError),
    /// The broadcaster subscribe call failed.
    #[error("broadcast subscribe failed: {0}")]
    Broadcast(String),
    /// The receive channel closed before a terminal result arrived.
    #[error("resolve channel closed before a result arrived")]
    ChannelClosed,
    /// The optional timeout elapsed before a result arrived.
    #[error("resolve timed out")]
    Timeout,
}

/// Worker-side broadcaster for offload result deltas (D-03 Option A).
///
/// The result hook is a `fn` pointer (`ferro_queue::OffloadResultHook`) and cannot
/// capture an `Arc<Broadcaster>`; it reads this static at invocation time instead,
/// mirroring `ferro_queue`'s `TENANT_ID_HOOK`. Set by
/// `register_offload_hooks_with_broadcaster` at bootstrap.
static OFFLOAD_BROADCASTER: OnceLock<Arc<ferro_broadcast::Broadcaster>> = OnceLock::new();

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

/// Persist a `{"status":"pending"}` marker under the handle key at enqueue (D-07).
///
/// Lets a read-back distinguish an unknown handle (no snapshot →
/// [`read_result`] returns `None`) from work that has not finished yet
/// (pending row → `Some(OffloadResult::Pending)`).
///
/// Written by the framework enqueue wrapper (Plan 02), never by `ferro-queue` (D-11).
///
/// # Non-fatal contract
///
/// Returns [`ProjectionError`] rather than panicking; callers log and continue.
pub async fn persist_pending(
    handle_key: &str,
    db: &DatabaseConnection,
) -> Result<(), ProjectionError> {
    let envelope = serde_json::json!({ "status": "pending" });
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
/// Returns `Ok(None)` when no snapshot exists for the handle — the handle is
/// either unknown or was never registered. Returns `Some(OffloadResult::Pending)`
/// when the work is enqueued but not yet finished (D-07). See [`persist_pending`].
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

/// Read back a result by handle, redacting the raw error for client-facing use (D-05/D-10).
///
/// Mirrors [`read_result`] but replaces a failed envelope's raw `Display` error with the
/// fixed non-sensitive marker `"terminal error"`. Completed values and the pending marker
/// pass through unchanged. The raw error remains only in the snapshot (via [`persist_error`])
/// and the worker logs, for authorized/server-side retrieval through [`read_result`].
///
/// This is the browser read-back leg of the subscribe → read-back → await pattern (D-09):
/// a client that receives the redacted delta reconciles by reading this back.
///
/// # Errors
///
/// [`ProjectionError::Db`] on a SeaORM error.
/// [`ProjectionError::Json`] if the stored envelope cannot be deserialized.
pub async fn read_result_redacted<T: OffloadSerializable>(
    handle_key: &str,
    db: &DatabaseConnection,
) -> Result<Option<OffloadResult<T>>, ProjectionError> {
    match read_result::<T>(handle_key, db).await? {
        None => Ok(None),
        Some(OffloadResult::Completed { value }) => Ok(Some(OffloadResult::Completed { value })),
        Some(OffloadResult::Failed { .. }) => Ok(Some(OffloadResult::Failed {
            error: "terminal error".to_string(),
        })),
        Some(OffloadResult::Pending) => Ok(Some(OffloadResult::Pending)),
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
///
/// When broadcasting is configured, use [`register_offload_hooks_with_broadcaster`]
/// instead — it also emits a delta on `projection.offload.result.{handle}` after
/// the snapshot persists (D-01/D-02).
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

/// Broadcast a result delta on `projection.offload.result.{handle}` (D-01/D-02/D-04).
///
/// Best-effort: a send failure is logged at `warn!` and swallowed — the snapshot is the
/// authoritative record and the job must never fail on a broadcast error (D-02, Pitfall 5).
async fn broadcast_delta(
    broadcaster: &Arc<ferro_broadcast::Broadcaster>,
    handle_key: &str,
    payload: serde_json::Value,
) {
    let channel = format!("projection.{}.{}", OFFLOAD_PROJECTION_NAME, handle_key);
    let send_result = ferro_broadcast::Broadcast::new(broadcaster.clone())
        .channel(channel.clone())
        .event("offload.result")
        .data(payload)
        .send()
        .await;
    if let Err(e) = send_result {
        tracing::warn!(
            handle_key = %handle_key,
            error = %e,
            channel = %channel,
            "offload delta broadcast failed; snapshot persisted"
        );
    }
}

/// Register the offload-result persistence hook with an attached broadcaster (D-01..D-05).
///
/// Sets `OFFLOAD_BROADCASTER` (D-03 Option A) so the result hook — which is a `fn` pointer
/// and cannot close over an `Arc<Broadcaster>` — can read it at invocation time. The hook
/// persists the snapshot first (D-02), then best-effort broadcasts a redacted delta on
/// `projection.offload.result.{handle}`.
///
/// Delta payload:
/// - Completed: `{"status":"completed","value":<v>}` (client receives the answer directly).
/// - Failed: `{"status":"failed"}` with NO `error` field (D-05 redaction; raw error stays
///   in the snapshot via [`persist_error`] for authorized server-side retrieval).
///
/// A broadcast failure is `tracing::warn!`-logged and swallowed — it never fails the job
/// or rolls back the snapshot (D-02 / Pitfall 5). When broadcasting is not configured,
/// use [`register_offload_hooks`] instead.
pub fn register_offload_hooks_with_broadcaster(broadcaster: Arc<ferro_broadcast::Broadcaster>) {
    let _ = OFFLOAD_BROADCASTER.set(broadcaster);
    ferro_queue::register_offload_result_hook(|key, outcome, db| {
        Box::pin(async move {
            // Build the client-facing (redacted) delta payload BEFORE consuming `outcome`.
            let delta = match &outcome {
                Ok(value) => serde_json::json!({ "status": "completed", "value": value }),
                Err(_) => serde_json::json!({ "status": "failed" }), // D-05: no raw error
            };
            // 1. Persist first (D-02).
            let res = match outcome {
                Ok(value) => persist_result_raw(&key, value, db).await,
                Err(msg) => persist_error(&key, &msg, db).await, // raw error stays in snapshot
            };
            if let Err(e) = res {
                tracing::warn!(
                    handle_key = %key,
                    error = %e,
                    "offload result persist failed — result not stored"
                );
                return; // do not broadcast a delta the snapshot cannot back
            }
            // 2. Broadcast best-effort (D-02).
            if let Some(b) = OFFLOAD_BROADCASTER.get() {
                broadcast_delta(b, &key, delta).await;
            }
        })
    });
}

/// Enqueue an offloaded job and write its pending marker (D-07/D-08).
///
/// The request-side entrypoint: dispatches the job via [`ferro_queue::Offloadable::offload`],
/// then writes `{"status":"pending"}` under the returned handle key so a read-back can
/// distinguish an unknown handle (no snapshot) from work in flight (pending). The pending
/// write is on the framework side (D-11: `ferro-queue` must not depend on
/// `ferro-projection`). Returns the handle the caller subscribes on. Does NOT await the
/// result — the request stays non-blocking (SC#2).
///
/// A pending-write failure is logged at `warn!` and swallowed: the job is already enqueued,
/// so a missing pending row degrades a read-back to `None` (unknown) rather than corrupting
/// state; the result snapshot still lands when the worker finishes.
///
/// Resolvable as `::ferro::offload::enqueue_and_mark_pending` (the `offload` module is
/// `pub mod offload` in `framework/src/lib.rs`).
pub async fn enqueue_and_mark_pending<J>(
    job: J,
    db: &DatabaseConnection,
) -> Result<ferro_queue::OffloadHandle<J::Output>, ferro_queue::Error>
where
    J: ferro_queue::Offloadable,
{
    let handle = job.offload().await?;
    if let Err(e) = persist_pending(handle.key(), db).await {
        tracing::warn!(
            handle_key = %handle.key(),
            error = %e,
            "offload pending marker write failed; result path unaffected"
        );
    }
    Ok(handle)
}

/// Resolve an offloaded result race-safely (D-09).
///
/// Subscribes to the handle's broadcast channel FIRST (preventing the TOCTOU
/// race where a delta fires between the read-back and the await), reads the
/// snapshot back once to short-circuit an already-terminal handle, then awaits
/// the delta and reads the authoritative snapshot on wake.
///
/// # Subscribe → read-back → await order
///
/// 1. **Subscribe first** — any delta fired after this point is buffered in the
///    `mpsc` receiver and will not be missed.
/// 2. **Read back once** — if the handle already completed or failed, return
///    immediately without waiting for a delta.
/// 3. **Await delta, read authoritative snapshot** — the delta is a redacted
///    wakeup signal; the snapshot is the authoritative record (D-06). On wake
///    `read_result` returns the full envelope (raw error included) — this helper
///    is server-side / in-process. Browser clients use `read_result_redacted`.
///
/// # Timeout
///
/// `timeout: None` waits until the channel closes or a terminal result arrives.
/// A terminally failed job records a `failed` snapshot + delta (Plan 02), so the
/// only unbounded wait is a job that never runs. Pass `Some(dur)` to bound it.
///
/// # Errors
///
/// - [`ResolveError::Broadcast`] — the `subscribe` call failed.
/// - [`ResolveError::Projection`] — a snapshot read failed.
/// - [`ResolveError::ChannelClosed`] — the mpsc receiver closed before a result
///   was observed. The channel closes when the client is removed (successful path)
///   or the broadcaster drops.
/// - [`ResolveError::Timeout`] — the optional timeout elapsed.
pub async fn resolve<T: OffloadSerializable>(
    handle: &ferro_queue::OffloadHandle<T>,
    broadcaster: &Arc<ferro_broadcast::Broadcaster>,
    db: &DatabaseConnection,
    timeout: Option<std::time::Duration>,
) -> Result<OffloadResult<T>, ResolveError> {
    use ferro_broadcast::ServerMessage;
    let key = handle.key();
    let channel = format!("projection.{}.{}", OFFLOAD_PROJECTION_NAME, key);
    // Unique client id per resolve call (Open Question #2 resolved).
    let client_id = format!("{}-resolve-{}", key, uuid::Uuid::new_v4());

    // 1. Subscribe FIRST (Pitfall 3 — prevents missing a delta that fires
    //    between the read-back and the await).
    let (tx, mut rx) = tokio::sync::mpsc::channel::<ServerMessage>(8);
    broadcaster.add_client(client_id.clone(), tx);
    // subscribe can fail after add_client succeeded — clean up the client
    // before returning so a failed subscribe does not leak an entry (WR-02).
    if let Err(e) = broadcaster
        .subscribe(&client_id, &channel, None, None)
        .await
    {
        broadcaster.remove_client(&client_id);
        return Err(ResolveError::Broadcast(e.to_string()));
    }

    // 2. Read back ONCE — short-circuit an already-terminal handle.
    match read_result::<T>(key, db).await {
        Ok(Some(result)) if !matches!(result, OffloadResult::Pending) => {
            broadcaster.remove_client(&client_id);
            return Ok(result);
        }
        Err(e) => {
            broadcaster.remove_client(&client_id);
            return Err(ResolveError::Projection(e));
        }
        _ => {} // None or Pending — fall through to await the delta
    }

    // 3. Await the delta, then read the authoritative snapshot on wake.
    let wait = async {
        while let Some(msg) = rx.recv().await {
            if let ServerMessage::Event(b) = msg {
                if b.event == "offload.result" {
                    // Delta received — read the authoritative snapshot.
                    return read_result::<T>(key, db)
                        .await
                        .map(|opt| opt.unwrap_or(OffloadResult::Pending))
                        .map_err(ResolveError::from);
                }
            }
        }
        // Channel closed — the client was removed (e.g. broadcaster shut down).
        // Try a final read before reporting ChannelClosed, in case the result
        // landed and remove_client fired.
        read_result::<T>(key, db)
            .await
            .map_err(ResolveError::from)
            .and_then(|opt| opt.ok_or(ResolveError::ChannelClosed))
    };

    let out = match timeout {
        Some(d) => match tokio::time::timeout(d, wait).await {
            Ok(inner) => inner,
            // On timeout, clean up the client before returning so a timed-out
            // resolve does not leak a broadcaster entry (WR-01).
            Err(_) => {
                broadcaster.remove_client(&client_id);
                return Err(ResolveError::Timeout);
            }
        },
        None => wait.await,
    };
    broadcaster.remove_client(&client_id);
    out
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

    /// `persist_pending` writes a retrievable pending snapshot; `read_result` returns
    /// `Some(Pending)` for a pending handle and `None` for an unknown handle (D-07).
    #[tokio::test]
    async fn offload_pending_round_trip() {
        let db = fresh_db().await;

        // Write the pending marker.
        persist_pending("k1", &db).await.expect("persist_pending");

        // Read back: must be Some(Pending), not None.
        let result = read_result::<()>("k1", &db)
            .await
            .expect("read_result after persist_pending");
        assert!(
            matches!(result, Some(OffloadResult::Pending)),
            "expected Some(Pending), got {result:?}"
        );

        // A never-written handle must still return None — distinct from not-done.
        let absent = read_result::<()>("nope", &db)
            .await
            .expect("read_result for unknown handle");
        assert!(
            absent.is_none(),
            "expected None for unknown handle, got {absent:?}"
        );
    }

    /// `read_result_redacted` replaces the raw failed error with `"terminal error"`;
    /// completed values and the pending marker pass through unchanged (D-05/D-10).
    #[tokio::test]
    async fn read_result_redacted_hides_error() {
        let db = fresh_db().await;

        // Failed path: raw error must NOT appear; fixed marker must appear.
        persist_error("kf", "sensitive-secret-value", &db)
            .await
            .expect("persist_error");
        let redacted = read_result_redacted::<String>("kf", &db)
            .await
            .expect("read_result_redacted failed");
        match redacted {
            Some(OffloadResult::Failed { error }) => {
                assert_eq!(
                    error, "terminal error",
                    "redacted error must be the fixed marker"
                );
                assert_ne!(
                    error, "sensitive-secret-value",
                    "raw error must not appear in the redacted output"
                );
            }
            other => panic!("expected Some(Failed), got {other:?}"),
        }

        // Completed path: value passes through unchanged.
        persist_result("kc", &"hello".to_string(), &db)
            .await
            .expect("persist_result");
        let completed = read_result_redacted::<String>("kc", &db)
            .await
            .expect("read_result_redacted completed");
        assert!(
            matches!(completed, Some(OffloadResult::Completed { ref value }) if value == "hello"),
            "expected Some(Completed {{ value: \"hello\" }}), got {completed:?}"
        );

        // Pending path: passes through as Some(Pending).
        persist_pending("kp", &db).await.expect("persist_pending");
        let pending = read_result_redacted::<String>("kp", &db)
            .await
            .expect("read_result_redacted pending");
        assert!(
            matches!(pending, Some(OffloadResult::Pending)),
            "expected Some(Pending), got {pending:?}"
        );

        // Unknown handle: returns None.
        let absent = read_result_redacted::<String>("absent", &db)
            .await
            .expect("read_result_redacted absent");
        assert!(
            absent.is_none(),
            "expected None for unknown handle, got {absent:?}"
        );
    }
}
