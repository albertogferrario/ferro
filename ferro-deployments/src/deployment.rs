//! Deployment model, status enum, and the `Deployments` handle.
//!
//! All SQL uses `Statement::from_sql_and_values` with bound `Value::*` params —
//! no string interpolation of caller-supplied data (T-186-04).

use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement, Value};
use serde::{Deserialize, Serialize};

use crate::error::Error;

// ---------------------------------------------------------------------------
// Deployment — an immutable row from the `deployments` table
// ---------------------------------------------------------------------------

/// An immutable deployment row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    /// Primary key.
    pub id: i64,
    /// Unique identifier (UUID v4 string, stable across retries).
    pub identifier: String,
    /// Scoping key (e.g. tenant slug or app name).
    pub owner_key: String,
    /// Optional VCS ref (branch, tag, commit SHA).
    pub source_ref: Option<String>,
    /// Storage path prefix for the deployment artifact.
    pub artifact_location: Option<String>,
    /// Artifact size in bytes.
    pub byte_size: Option<i64>,
    /// Lifecycle status.
    pub status: DeploymentStatus,
    /// When the artifact was garbage-collected; non-null means the deployment
    /// can no longer be promoted.
    pub artifact_deleted_at: Option<DateTime<Utc>>,
    /// When the deployment entered a terminal state (ready or failed).
    pub terminated_at: Option<DateTime<Utc>>,
    /// Row creation timestamp.
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// DeploymentStatus enum
// ---------------------------------------------------------------------------

/// Lifecycle state of a deployment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentStatus {
    /// Build in progress; artifact not yet available.
    Building,
    /// Artifact uploaded and verified; eligible for promotion.
    Ready,
    /// Build failed; terminal state.
    Failed,
}

impl DeploymentStatus {
    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "building" => Ok(Self::Building),
            "ready" => Ok(Self::Ready),
            "failed" => Ok(Self::Failed),
            other => Err(Error::custom(format!("unknown deployment status: {other}"))),
        }
    }
}

// ---------------------------------------------------------------------------
// Deployments handle
// ---------------------------------------------------------------------------

/// Handle to the deployments system, wrapping a `DatabaseConnection`.
///
/// All methods bind caller-supplied strings as `Value::*` parameters — no
/// string interpolation (T-186-04).
#[derive(Clone)]
pub struct Deployments {
    db: DatabaseConnection,
}

impl Deployments {
    /// Create a new `Deployments` handle backed by `db`.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Insert a new deployment row with status `building`.
    ///
    /// Returns the complete `Deployment` row including its assigned `id`.
    pub async fn create(
        &self,
        owner_key: &str,
        source_ref: Option<&str>,
    ) -> Result<Deployment, Error> {
        let identifier = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let now_iso = now.to_rfc3339();
        let backend = self.db.get_database_backend();

        let (p1, p2, p3, p4) = (
            ph(backend, 1),
            ph(backend, 2),
            ph(backend, 3),
            ph(backend, 4),
        );
        let sql = format!(
            "INSERT INTO deployments \
             (identifier, owner_key, source_ref, artifact_location, byte_size, status, \
              artifact_deleted_at, terminated_at, created_at) \
             VALUES ({p1}, {p2}, {p3}, NULL, NULL, 'building', NULL, NULL, {p4}) \
             RETURNING id"
        );
        let stmt = Statement::from_sql_and_values(
            backend,
            &sql,
            [
                Value::String(Some(Box::new(identifier.clone()))),
                Value::String(Some(Box::new(owner_key.to_string()))),
                source_ref.map_or(Value::String(None), |s| {
                    Value::String(Some(Box::new(s.to_string())))
                }),
                Value::String(Some(Box::new(now_iso.clone()))),
            ],
        );

        let row = self
            .db
            .query_one(stmt)
            .await
            .map_err(Error::Db)?
            .ok_or_else(|| Error::custom("create: INSERT RETURNING returned no row"))?;

        let id: i64 = row
            .try_get_by::<i64, _>("id")
            .map_err(|e| Error::custom(format!("create: parse id: {e}")))?;

        Ok(Deployment {
            id,
            identifier,
            owner_key: owner_key.to_string(),
            source_ref: source_ref.map(str::to_string),
            artifact_location: None,
            byte_size: None,
            status: DeploymentStatus::Building,
            artifact_deleted_at: None,
            terminated_at: None,
            created_at: now,
        })
    }

    /// Transition a `building` deployment to `ready`, recording the artifact
    /// location and byte size.
    ///
    /// Returns `Err` when the row does not exist or is already in a terminal
    /// state (`ready` or `failed`).
    pub async fn mark_ready(
        &self,
        id: i64,
        artifact_location: &str,
        byte_size: i64,
    ) -> Result<(), Error> {
        let now_iso = Utc::now().to_rfc3339();
        let backend = self.db.get_database_backend();
        let (p1, p2, p3, p4) = (
            ph(backend, 1),
            ph(backend, 2),
            ph(backend, 3),
            ph(backend, 4),
        );
        let sql = format!(
            "UPDATE deployments \
             SET status = 'ready', artifact_location = {p1}, byte_size = {p2}, terminated_at = {p3} \
             WHERE id = {p4} AND status = 'building'"
        );
        let stmt = Statement::from_sql_and_values(
            backend,
            &sql,
            [
                Value::String(Some(Box::new(artifact_location.to_string()))),
                Value::BigInt(Some(byte_size)),
                Value::String(Some(Box::new(now_iso))),
                Value::BigInt(Some(id)),
            ],
        );
        let result = self.db.execute(stmt).await.map_err(Error::Db)?;
        if result.rows_affected() == 0 {
            // Either the row does not exist or is already terminal.
            // Distinguish: if get() returns NotFound propagate that; else it's an
            // illegal transition (already ready/failed).
            self.get(id).await?;
            return Err(Error::custom(format!(
                "deployment {id} is not in building state; transition to ready rejected"
            )));
        }
        Ok(())
    }

    /// Transition a `building` deployment to `failed`.
    ///
    /// The `error` message is emitted as a tracing warning (no error column in
    /// the schema). Returns `Err` when the row does not exist or is already
    /// terminal.
    pub async fn mark_failed(&self, id: i64, error: &str) -> Result<(), Error> {
        tracing::warn!(
            deployment_id = id,
            error = error,
            "deployment marked failed"
        );
        let now_iso = Utc::now().to_rfc3339();
        let backend = self.db.get_database_backend();
        let (p1, p2) = (ph(backend, 1), ph(backend, 2));
        let sql = format!(
            "UPDATE deployments \
             SET status = 'failed', terminated_at = {p1} \
             WHERE id = {p2} AND status = 'building'"
        );
        let stmt = Statement::from_sql_and_values(
            backend,
            &sql,
            [
                Value::String(Some(Box::new(now_iso))),
                Value::BigInt(Some(id)),
            ],
        );
        let result = self.db.execute(stmt).await.map_err(Error::Db)?;
        if result.rows_affected() == 0 {
            self.get(id).await?;
            return Err(Error::custom(format!(
                "deployment {id} is not in building state; transition to failed rejected"
            )));
        }
        Ok(())
    }

    /// Fetch a deployment by its primary key.
    pub async fn get(&self, id: i64) -> Result<Deployment, Error> {
        let backend = self.db.get_database_backend();
        let p1 = ph(backend, 1);
        let sql = format!(
            "SELECT id, identifier, owner_key, source_ref, artifact_location, byte_size, \
             status, artifact_deleted_at, terminated_at, created_at \
             FROM deployments WHERE id = {p1}"
        );
        let stmt = Statement::from_sql_and_values(backend, &sql, [Value::BigInt(Some(id))]);
        let row = self
            .db
            .query_one(stmt)
            .await
            .map_err(Error::Db)?
            .ok_or(Error::NotFound { id })?;
        parse_deployment_row(&row)
    }

    /// List all deployments for `owner_key`, newest first.
    pub async fn list(&self, owner_key: &str) -> Result<Vec<Deployment>, Error> {
        let backend = self.db.get_database_backend();
        let p1 = ph(backend, 1);
        let sql = format!(
            "SELECT id, identifier, owner_key, source_ref, artifact_location, byte_size, \
             status, artifact_deleted_at, terminated_at, created_at \
             FROM deployments WHERE owner_key = {p1} ORDER BY id DESC"
        );
        let stmt = Statement::from_sql_and_values(
            backend,
            &sql,
            [Value::String(Some(Box::new(owner_key.to_string())))],
        );
        let rows = self.db.query_all(stmt).await.map_err(Error::Db)?;
        rows.iter().map(parse_deployment_row).collect()
    }

    /// Return the currently-active deployment for `owner_key`, or `None` when
    /// no pointer row exists yet.
    pub async fn active(&self, owner_key: &str) -> Result<Option<Deployment>, Error> {
        let backend = self.db.get_database_backend();
        let p1 = ph(backend, 1);
        let sql = format!("SELECT deployment_id FROM deployment_pointers WHERE owner_key = {p1}");
        let stmt = Statement::from_sql_and_values(
            backend,
            &sql,
            [Value::String(Some(Box::new(owner_key.to_string())))],
        );
        let row = self.db.query_one(stmt).await.map_err(Error::Db)?;
        match row {
            None => Ok(None),
            Some(r) => {
                let deployment_id: i64 = r
                    .try_get_by::<i64, _>("deployment_id")
                    .map_err(|e| Error::custom(format!("active: parse deployment_id: {e}")))?;
                self.get(deployment_id).await.map(Some)
            }
        }
    }

    /// Atomically promote `deployment_id` to be the active deployment for
    /// `owner_key`.
    ///
    /// Guards:
    /// - `Error::NotReady` when the deployment status is not `ready`.
    /// - `Error::ArtifactDeleted` when `artifact_deleted_at` is set.
    ///
    /// Returns the previously-active deployment id, or `None` when this is the
    /// first promotion.
    pub async fn promote(&self, owner_key: &str, deployment_id: i64) -> Result<Option<i64>, Error> {
        let dep = self.get(deployment_id).await?;
        if dep.status != DeploymentStatus::Ready {
            return Err(Error::NotReady { id: deployment_id });
        }
        if dep.artifact_deleted_at.is_some() {
            return Err(Error::ArtifactDeleted { id: deployment_id });
        }
        crate::promote::promote(&self.db, owner_key, deployment_id).await
    }

    /// Roll back to the previous deployment.
    ///
    /// Returns `Err(Error::NoPreviousDeployment)` when the pointer row has no
    /// `previous_deployment_id`.
    pub async fn rollback(&self, owner_key: &str) -> Result<Option<i64>, Error> {
        let backend = self.db.get_database_backend();
        let p1 = ph(backend, 1);
        let sql = format!(
            "SELECT previous_deployment_id FROM deployment_pointers WHERE owner_key = {p1}"
        );
        let stmt = Statement::from_sql_and_values(
            backend,
            &sql,
            [Value::String(Some(Box::new(owner_key.to_string())))],
        );
        let row = self.db.query_one(stmt).await.map_err(Error::Db)?;
        let previous_id = match row {
            None => {
                return Err(Error::NoPreviousDeployment {
                    owner_key: owner_key.to_string(),
                })
            }
            Some(r) => {
                let opt: Option<i64> = r
                    .try_get_by::<Option<i64>, _>("previous_deployment_id")
                    .map_err(|e| {
                        Error::custom(format!("rollback: parse previous_deployment_id: {e}"))
                    })?;
                opt
            }
        };
        match previous_id {
            None => Err(Error::NoPreviousDeployment {
                owner_key: owner_key.to_string(),
            }),
            Some(prev_id) => self.promote(owner_key, prev_id).await,
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn parse_deployment_row(row: &sea_orm::QueryResult) -> Result<Deployment, Error> {
    let id: i64 = row
        .try_get_by::<i64, _>("id")
        .map_err(|e| Error::custom(format!("parse id: {e}")))?;
    let identifier: String = row
        .try_get_by::<String, _>("identifier")
        .map_err(|e| Error::custom(format!("parse identifier: {e}")))?;
    let owner_key: String = row
        .try_get_by::<String, _>("owner_key")
        .map_err(|e| Error::custom(format!("parse owner_key: {e}")))?;
    let source_ref: Option<String> = row
        .try_get_by::<Option<String>, _>("source_ref")
        .map_err(|e| Error::custom(format!("parse source_ref: {e}")))?;
    let artifact_location: Option<String> = row
        .try_get_by::<Option<String>, _>("artifact_location")
        .map_err(|e| Error::custom(format!("parse artifact_location: {e}")))?;
    let byte_size: Option<i64> = row
        .try_get_by::<Option<i64>, _>("byte_size")
        .map_err(|e| Error::custom(format!("parse byte_size: {e}")))?;
    let status_str: String = row
        .try_get_by::<String, _>("status")
        .map_err(|e| Error::custom(format!("parse status: {e}")))?;
    let status = DeploymentStatus::from_str(&status_str)?;
    let artifact_deleted_at = parse_optional_timestamp(row, "artifact_deleted_at")?;
    let terminated_at = parse_optional_timestamp(row, "terminated_at")?;
    let created_at = parse_timestamp(row, "created_at")?;

    Ok(Deployment {
        id,
        identifier,
        owner_key,
        source_ref,
        artifact_location,
        byte_size,
        status,
        artifact_deleted_at,
        terminated_at,
        created_at,
    })
}

/// Parse a required timestamp column — Postgres timestamptz or SQLite ISO-8601 text.
fn parse_timestamp(row: &sea_orm::QueryResult, col: &str) -> Result<DateTime<Utc>, Error> {
    if let Ok(dt) = row.try_get_by::<DateTime<Utc>, _>(col) {
        return Ok(dt);
    }
    let s: String = row
        .try_get_by::<String, _>(col)
        .map_err(|e| Error::custom(format!("parse {col}: {e}")))?;
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| Error::custom(format!("parse {col} as rfc3339 ('{s}'): {e}")))
}

/// Parse a nullable timestamp column, returning `Ok(None)` for SQL NULL.
fn parse_optional_timestamp(
    row: &sea_orm::QueryResult,
    col: &str,
) -> Result<Option<DateTime<Utc>>, Error> {
    if let Ok(opt) = row.try_get_by::<Option<DateTime<Utc>>, _>(col) {
        return Ok(opt);
    }
    let s: Option<String> = row
        .try_get_by::<Option<String>, _>(col)
        .map_err(|e| Error::custom(format!("parse {col}: {e}")))?;
    match s {
        None => Ok(None),
        Some(s) => DateTime::parse_from_rfc3339(&s)
            .map(|dt| Some(dt.with_timezone(&Utc)))
            .map_err(|e| Error::custom(format!("parse {col} as rfc3339 ('{s}'): {e}"))),
    }
}

/// SQL placeholder style: Postgres uses `$N`, SQLite uses `?N`.
fn ph(backend: DatabaseBackend, n: usize) -> String {
    match backend {
        DatabaseBackend::Postgres => format!("${n}"),
        _ => format!("?{n}"),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;
    use sea_orm_migration::MigratorTrait;

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![
                Box::new(crate::migration::CreateDeploymentsTable),
                Box::new(crate::migration::CreateDeploymentPointersTable),
            ]
        }
    }

    async fn setup() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:")
            .await
            .expect("connect sqlite::memory:");
        TestMigrator::up(&conn, None).await.expect("run migrations");
        conn
    }

    #[tokio::test]
    async fn create_sets_building() {
        let conn = setup().await;
        let d = Deployments::new(conn);
        let dep = d
            .create("owner:1", Some("abc123"))
            .await
            .expect("create failed");
        assert_eq!(dep.status, DeploymentStatus::Building);
        assert_eq!(dep.owner_key, "owner:1");
        assert_eq!(dep.source_ref.as_deref(), Some("abc123"));
        assert!(!dep.identifier.is_empty(), "identifier must be set");
        assert!(dep.artifact_location.is_none());
        assert!(dep.byte_size.is_none());
    }

    #[tokio::test]
    async fn get_round_trips() {
        let conn = setup().await;
        let d = Deployments::new(conn);
        let dep = d.create("owner:2", Some("ref-xyz")).await.expect("create");
        let fetched = d.get(dep.id).await.expect("get");
        assert_eq!(fetched.id, dep.id);
        assert_eq!(fetched.identifier, dep.identifier);
        assert_eq!(fetched.owner_key, "owner:2");
        assert_eq!(fetched.source_ref.as_deref(), Some("ref-xyz"));
        assert_eq!(fetched.status, DeploymentStatus::Building);
    }

    #[tokio::test]
    async fn mark_ready_transitions() {
        let conn = setup().await;
        let d = Deployments::new(conn);
        let dep = d.create("owner:3", None).await.expect("create");
        d.mark_ready(dep.id, "deployments/1/", 4096)
            .await
            .expect("mark_ready failed");
        let fetched = d.get(dep.id).await.expect("get");
        assert_eq!(fetched.status, DeploymentStatus::Ready);
        assert_eq!(fetched.artifact_location.as_deref(), Some("deployments/1/"));
        assert_eq!(fetched.byte_size, Some(4096));
        assert!(
            fetched.terminated_at.is_some(),
            "terminated_at must be set after mark_ready"
        );
    }

    #[tokio::test]
    async fn mark_ready_rejects_terminal() {
        let conn = setup().await;
        let d = Deployments::new(conn);
        let dep = d.create("owner:4", None).await.expect("create");
        d.mark_ready(dep.id, "path/", 100)
            .await
            .expect("first mark_ready");
        // Second call on an already-ready row must return Err.
        let result = d.mark_ready(dep.id, "path2/", 200).await;
        assert!(
            result.is_err(),
            "mark_ready on terminal row should return Err"
        );
    }

    #[tokio::test]
    async fn mark_failed_transitions() {
        let conn = setup().await;
        let d = Deployments::new(conn);
        let dep = d.create("owner:5", None).await.expect("create");
        d.mark_failed(dep.id, "build exploded")
            .await
            .expect("mark_failed");
        let fetched = d.get(dep.id).await.expect("get");
        assert_eq!(fetched.status, DeploymentStatus::Failed);
        assert!(
            fetched.terminated_at.is_some(),
            "terminated_at must be set after mark_failed"
        );
    }

    #[tokio::test]
    async fn list_returns_owner_rows() {
        let conn = setup().await;
        let d = Deployments::new(conn);
        d.create("owner:6", None).await.expect("create 1");
        d.create("owner:6", None).await.expect("create 2");
        d.create("other-owner", None).await.expect("create other");

        let rows = d.list("owner:6").await.expect("list");
        assert_eq!(rows.len(), 2, "list should return 2 rows for owner:6");

        let other_rows = d.list("other-owner").await.expect("list other");
        assert_eq!(other_rows.len(), 1);
    }

    #[tokio::test]
    async fn active_returns_none_without_pointer() {
        let conn = setup().await;
        let d = Deployments::new(conn);
        let result = d.active("owner:7").await.expect("active");
        assert!(
            result.is_none(),
            "active should be None when no pointer exists"
        );
    }

    #[tokio::test]
    async fn promote_rejects_non_ready() {
        let conn = setup().await;
        let d = Deployments::new(conn);
        let dep = d.create("owner:8", None).await.expect("create");
        // dep is still Building — promote must reject it
        let result = d.promote("owner:8", dep.id).await;
        assert!(
            matches!(result, Err(Error::NotReady { id }) if id == dep.id),
            "promote should return Error::NotReady for a Building deployment, got: {result:?}"
        );
    }

    #[tokio::test]
    async fn promote_rejects_deleted_artifact() {
        let conn = setup().await;
        let d = Deployments::new(conn.clone());
        let dep = d.create("owner:9", None).await.expect("create");
        d.mark_ready(dep.id, "artifacts/9/", 512)
            .await
            .expect("mark_ready");
        // Manually set artifact_deleted_at to simulate GC.
        let now_iso = chrono::Utc::now().to_rfc3339();
        conn.execute(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            format!(
                "UPDATE deployments SET artifact_deleted_at = '{}' WHERE id = {}",
                now_iso, dep.id
            ),
        ))
        .await
        .expect("set artifact_deleted_at");
        let result = d.promote("owner:9", dep.id).await;
        assert!(
            matches!(result, Err(Error::ArtifactDeleted { id }) if id == dep.id),
            "promote should return Error::ArtifactDeleted, got: {result:?}"
        );
    }

    #[tokio::test]
    async fn promote_returns_previous_id() {
        let conn = setup().await;
        let d = Deployments::new(conn);
        let dep_a = d.create("owner:10", None).await.expect("create a");
        let dep_b = d.create("owner:10", None).await.expect("create b");
        d.mark_ready(dep_a.id, "a/", 1).await.expect("ready a");
        d.mark_ready(dep_b.id, "b/", 2).await.expect("ready b");

        // First promote: no previous.
        let prev = d
            .promote("owner:10", dep_a.id)
            .await
            .expect("first promote");
        assert!(
            prev.is_none(),
            "first promote should return None as previous"
        );

        // Second promote: previous is dep_a.
        let prev2 = d
            .promote("owner:10", dep_b.id)
            .await
            .expect("second promote");
        assert_eq!(
            prev2,
            Some(dep_a.id),
            "second promote should return dep_a.id as previous"
        );
    }

    #[tokio::test]
    async fn rollback_promotes_previous() {
        let conn = setup().await;
        let d = Deployments::new(conn);
        let dep_a = d.create("owner:11", None).await.expect("create a");
        let dep_b = d.create("owner:11", None).await.expect("create b");
        d.mark_ready(dep_a.id, "a/", 1).await.expect("ready a");
        d.mark_ready(dep_b.id, "b/", 2).await.expect("ready b");
        d.promote("owner:11", dep_a.id).await.expect("promote a");
        d.promote("owner:11", dep_b.id).await.expect("promote b");

        // Rollback: should promote dep_a again.
        d.rollback("owner:11").await.expect("rollback");
        let active = d
            .active("owner:11")
            .await
            .expect("active after rollback")
            .expect("should have active deployment");
        assert_eq!(
            active.id, dep_a.id,
            "after rollback, active should be dep_a"
        );
    }
}
