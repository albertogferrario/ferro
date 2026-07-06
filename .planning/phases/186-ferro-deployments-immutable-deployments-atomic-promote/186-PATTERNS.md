# Phase 186: ferro-deployments — Pattern Map

**Mapped:** 2026-06-07
**Files analyzed:** 10 (all new — greenfield crate)
**Analogs found:** 10 / 10

---

## File Classification

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `ferro-deployments/Cargo.toml` | config | — | `ferro-queue/Cargo.toml` | exact |
| `ferro-deployments/src/lib.rs` | utility | — | `ferro-queue/src/lib.rs` | exact |
| `ferro-deployments/src/error.rs` | utility | — | `ferro-queue/src/error.rs` | exact |
| `ferro-deployments/src/config.rs` | config | request-response | `ferro-queue/src/config.rs` | exact |
| `ferro-deployments/src/migration.rs` | migration | CRUD | `ferro-queue/src/migration.rs` | exact |
| `ferro-deployments/src/deployment.rs` | model | CRUD | `ferro-queue/src/db.rs` (JobRow / Queue handle) | role-match |
| `ferro-deployments/src/promote.rs` | service | CRUD | `ferro-queue/src/db.rs` (claim_sqlite / claim_postgres) | exact |
| `ferro-deployments/src/storage.rs` | service | file-I/O | `ferro-storage/src/facade.rs` (Disk) | role-match |
| `ferro-deployments/tests/race_promote_sqlite.rs` | test | CRUD | `ferro-queue/tests/race_claim_sqlite.rs` | exact |
| `ferro-deployments/tests/race_promote_postgres.rs` | test | CRUD | `ferro-queue/tests/race_claim_postgres.rs` | exact |

---

## Pattern Assignments

### `ferro-deployments/Cargo.toml` (config)

**Analog:** `ferro-queue/Cargo.toml`

**Package header pattern** (lines 1–10):
```toml
[package]
name = "ferro-queue"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Background job queue system for Ferro framework"
repository = "https://github.com/albertogferrario/ferro"
keywords = ["queue", "jobs", "background", "async", "ferro"]
categories = ["database", "asynchronous"]
readme = "README.md"
```

**Dependencies pattern** (lines 12–34) — copy structure, replace deps:
```toml
[dependencies]
async-trait = "0.1"
tokio = { version = "1", features = ["sync", "rt", "time", "macros", "signal"] }
tracing = "0.1"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls", "macros"] }
sea-orm-migration = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }

[dev-dependencies]
tokio = { version = "1", features = ["full", "test-util"] }
tempfile = "3"

[features]
sqlx-postgres = ["sea-orm/sqlx-postgres"]
postgres-tests = ["sqlx-postgres"]
```

**Difference from analog:** add `ferro-storage = { path = "../ferro-storage", version = "0.2" }` to `[dependencies]`; add `bytes = "1"` for the storage trait; remove `parking_lot`, `rand`, `futures` (not needed); remove `serial_test` from dev-deps.

---

### `ferro-deployments/src/lib.rs` (utility — crate root and public API)

**Analog:** `ferro-queue/src/lib.rs`

**Module declaration + re-export pattern** (lines 1–71):
```rust
mod config;
mod db;          // → mod deployment; mod promote;
mod dispatcher;
mod error;
mod migration;
// ...

pub use config::QueueConfig;         // → pub use config::DeploymentConfig;
pub use error::Error;
pub use migration::CreateJobsTable;  // → pub use migration::{CreateDeploymentsTable, CreateDeploymentPointersTable};
```

**Crate doc header style** (lines 1–47, the //! block): write a matching `//! # ferro-deployments` block summarizing the three exports (Deployments handle, migration helpers, DeploymentStorage trait).

**Key pattern:** every public symbol re-exported from `lib.rs`; no public modules, only re-exports.

---

### `ferro-deployments/src/error.rs` (utility — error enum)

**Analog:** `ferro-queue/src/error.rs`

**Full pattern** (lines 1–136 — copy structure exactly):
```rust
use thiserror::Error;

/// Errors that can occur in the deployments system.
#[derive(Debug, Error)]
pub enum Error {
    /// Database error.
    #[error("Database error: {0}")]
    Db(#[from] sea_orm::DbErr),

    /// Unsupported database backend.
    #[error("Unsupported database backend")]
    UnsupportedBackend,

    /// JSON error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Custom error.
    #[error("{0}")]
    Custom(String),
}
```

**Domain-specific variants to add** (mirroring the structured variants in lines 38–77 of the analog):
```rust
    /// Deployment is not in the ready state and cannot be promoted.
    #[error("Deployment {id} cannot be promoted: status is not ready")]
    NotReady { id: i64 },

    /// Deployment artifacts have been deleted; rollback refused.
    #[error("Deployment {id} cannot be promoted: artifact has been deleted")]
    ArtifactDeleted { id: i64 },

    /// No previous deployment exists for rollback.
    #[error("No previous deployment to roll back to for owner_key '{owner_key}'")]
    NoPreviousDeployment { owner_key: String },

    /// Deployment not found.
    #[error("Deployment {id} not found")]
    NotFound { id: i64 },

    /// Storage error (wraps ferro_storage::Error).
    #[error("Storage error: {0}")]
    Storage(#[from] ferro_storage::Error),
```

**Constructor helpers pattern** (lines 81–106 of analog):
```rust
impl Error {
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }
    // add domain-specific constructors following the same shape
}

impl From<String> for Error { ... }
impl From<&str> for Error { ... }
```

---

### `ferro-deployments/src/config.rs` (config — env-based configuration)

**Analog:** `ferro-queue/src/config.rs`

**Struct + from_env() pattern** (lines 1–97 — copy structure, change fields):
```rust
/// Deployment configuration.
#[derive(Debug, Clone)]
pub struct DeploymentConfig {
    /// Domain for preview subdomain URLs (DEPLOYMENT_PREVIEW_DOMAIN env var).
    pub preview_domain: Option<String>,
}

impl DeploymentConfig {
    /// Create configuration from environment variables.
    ///
    /// Reads:
    /// - `DEPLOYMENT_PREVIEW_DOMAIN`: domain for preview URLs (optional)
    pub fn from_env() -> Self {
        Self {
            preview_domain: std::env::var("DEPLOYMENT_PREVIEW_DOMAIN").ok(),
        }
    }
}
```

**Builder method pattern** (lines 75–97 of analog — `with_*` consuming methods):
```rust
    /// Set the preview domain.
    pub fn with_preview_domain(mut self, domain: impl Into<String>) -> Self {
        self.preview_domain = Some(domain.into());
        self
    }
```

**Test pattern** (lines 99–200 of analog — `EnvGuard` + `#[serial]`):
```rust
#[cfg(test)]
mod tests {
    use serial_test::serial;

    struct EnvGuard { vars: Vec<String> }
    impl EnvGuard { fn new() -> Self { ... } fn also_set(...) { ... } fn also_remove(...) { ... } }
    impl Drop for EnvGuard { fn drop(&mut self) { ... } }

    #[test]
    #[serial]
    fn preview_domain_from_env() { ... }

    #[test]
    #[serial]
    fn preview_domain_none_when_unset() { ... }
}
```

---

### `ferro-deployments/src/migration.rs` (migration — portable SchemaManager DDL)

**Analog:** `ferro-queue/src/migration.rs`

**Full file structure to copy verbatim, adapting column names** (lines 1–165):

```rust
//! `CreateDeploymentsTable` + `CreateDeploymentPointersTable` —
//! SeaORM migrations creating the `deployments` and `deployment_pointers`
//! tables. Portable across SQLite + Postgres: no backend-specific SQL,
//! only the SchemaManager DDL builder.
//!
//! Consumers register both in their own `Migrator` in order:
//! ```rust,ignore
//! impl MigratorTrait for Migrator {
//!     fn migrations() -> Vec<Box<dyn MigrationTrait>> {
//!         vec![
//!             Box::new(ferro_deployments::CreateDeploymentsTable),
//!             Box::new(ferro_deployments::CreateDeploymentPointersTable),
//!             // ... your app migrations
//!         ]
//!     }
//! }
//! ```

use sea_orm_migration::prelude::*;

pub struct CreateDeploymentsTable;

impl sea_orm_migration::MigrationName for CreateDeploymentsTable {
    fn name(&self) -> &str { "m_create_deployments_table" }
}

#[async_trait::async_trait]
impl MigrationTrait for CreateDeploymentsTable {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Deployments::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Deployments::Id).big_integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Deployments::Identifier).string().not_null().unique_key())
                    .col(ColumnDef::new(Deployments::OwnerKey).string().not_null())
                    .col(ColumnDef::new(Deployments::SourceRef).string().null())
                    .col(ColumnDef::new(Deployments::ArtifactLocation).string().null())
                    .col(ColumnDef::new(Deployments::ByteSize).big_integer().null())
                    .col(ColumnDef::new(Deployments::Status).string().not_null().default("building"))
                    .col(ColumnDef::new(Deployments::ArtifactDeletedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Deployments::TerminatedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Deployments::CreatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Deployments::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Deployments { Table, Id, Identifier, OwnerKey, SourceRef, ArtifactLocation, ByteSize, Status, ArtifactDeletedAt, TerminatedAt, CreatedAt }
```

**Second migration struct** for `deployment_pointers` table (same file, same pattern):
```rust
pub struct CreateDeploymentPointersTable;

impl sea_orm_migration::MigrationName for CreateDeploymentPointersTable {
    fn name(&self) -> &str { "m_create_deployment_pointers_table" }
}

// columns: owner_key (TEXT PK), deployment_id (BIGINT NOT NULL FK-like),
//          previous_deployment_id (BIGINT NULL), updated_at (TIMESTAMPTZ NOT NULL)
```

**Inline test pattern** (lines 166–241 of analog — `TestMigrator` + `tokio::test` + `sqlite_master` verification):
```rust
#[cfg(test)]
mod tests {
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
    use sea_orm_migration::MigratorTrait;

    struct TestMigrator;
    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(super::CreateDeploymentsTable), Box::new(super::CreateDeploymentPointersTable)]
        }
    }

    #[tokio::test]
    async fn migration_creates_deployments_table() {
        let conn = Database::connect("sqlite::memory:").await.expect("connect");
        TestMigrator::up(&conn, None).await.expect("migrate");
        // check sqlite_master for 'deployments' and 'deployment_pointers'
        // check artifact_deleted_at column present
        // verify down() drops both tables
    }
}
```

---

### `ferro-deployments/src/deployment.rs` (model + handle — Deployment struct and Deployments handle)

**Analog:** `ferro-queue/src/db.rs` (JobRow struct lines 95–117; Queue handle lines 20–88; parse helpers lines 239–283)

**Deployment struct pattern** (mirrors JobRow lines 95–117):
```rust
/// An immutable deployment row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub id: i64,
    pub identifier: String,
    pub owner_key: String,
    pub source_ref: Option<String>,
    pub artifact_location: Option<String>,
    pub byte_size: Option<i64>,
    pub status: DeploymentStatus,
    pub artifact_deleted_at: Option<DateTime<Utc>>,
    pub terminated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
```

**Status enum pattern** (mirrors JobState lines 124–133):
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentStatus {
    Building,
    Ready,
    Failed,
}
```

**Handle struct pattern** (mirrors Queue lines 31–88 but instance-based not global):
```rust
/// Handle to the deployments system, wrapping a `DatabaseConnection`.
#[derive(Clone)]
pub struct Deployments {
    db: DatabaseConnection,
}

impl Deployments {
    pub fn new(db: DatabaseConnection) -> Self { Self { db } }
    pub async fn create(&self, owner_key: &str, source_ref: Option<&str>) -> Result<Deployment, Error> { ... }
    pub async fn mark_ready(&self, id: i64, artifact_location: &str, byte_size: i64) -> Result<(), Error> { ... }
    pub async fn mark_failed(&self, id: i64, error: &str) -> Result<(), Error> { ... }
    pub async fn get(&self, id: i64) -> Result<Deployment, Error> { ... }
    pub async fn list(&self, owner_key: &str) -> Result<Vec<Deployment>, Error> { ... }
    pub async fn active(&self, owner_key: &str) -> Result<Option<Deployment>, Error> { ... }
    // promote and rollback are in promote.rs; re-exported through the handle
}
```

**parse_deployment_row helper pattern** (mirrors parse_job_row lines 193–237 + parse_timestamp lines 241–253 + parse_optional_timestamp lines 257–275):
```rust
fn parse_deployment_row(row: &sea_orm::QueryResult) -> Result<Deployment, Error> {
    let id: i64 = row.try_get_by::<i64, _>("id")
        .map_err(|e| Error::custom(format!("parse id: {e}")))?;
    // ... field by field; use parse_timestamp() for created_at
    // use parse_optional_timestamp() for artifact_deleted_at, terminated_at
}

fn parse_timestamp(row: &sea_orm::QueryResult, col: &str) -> Result<DateTime<Utc>, Error> {
    // copy verbatim from ferro-queue/src/db.rs lines 241-253
    if let Ok(dt) = row.try_get_by::<DateTime<Utc>, _>(col) { return Ok(dt); }
    let s: String = row.try_get_by::<String, _>(col)
        .map_err(|e| Error::custom(format!("parse {col}: {e}")))?;
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| Error::custom(format!("parse {col} as rfc3339 ('{s}'): {e}")))
}
```

**ph() helper** (copy verbatim from ferro-queue/src/db.rs lines 278–283):
```rust
fn ph(backend: DatabaseBackend, n: usize) -> String {
    match backend {
        DatabaseBackend::Postgres => format!("${n}"),
        _ => format!("?{n}"),
    }
}
```

**identifier generation** (place inside `create()`):
```rust
use uuid::Uuid;
let identifier = Uuid::new_v4().to_string();
```

---

### `ferro-deployments/src/promote.rs` (service — atomic pointer flip, dual-backend)

**Analog:** `ferro-queue/src/db.rs` — `claim_sqlite` (lines 347–394) and `claim_postgres` (lines 310–344)

**Dual-backend dispatch pattern** (lines 298–308):
```rust
pub async fn promote(
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
```

**SQLite path pattern** (mirrors claim_sqlite lines 347–394 — `conn.begin()` is mandatory):
```rust
async fn promote_sqlite(
    conn: &DatabaseConnection,
    owner_key: &str,
    deployment_id: i64,
) -> Result<Option<i64>, Error> {
    let now_iso = Utc::now().to_rfc3339();
    // CR-01: conn.begin() pins all statements to one pooled connection
    let txn = conn.begin().await.map_err(Error::Db)?;

    // Upsert: INSERT new pointer row, or UPDATE existing one atomically.
    // SET expressions read pre-update values on both backends (SQL standard).
    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "INSERT INTO deployment_pointers (owner_key, deployment_id, previous_deployment_id, updated_at) \
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
        Ok(r) => r,
        Err(e) => { let _ = txn.rollback().await; return Err(Error::Db(e)); }
    };
    txn.commit().await.map_err(Error::Db)?;

    // Extract previous_deployment_id from returned row
    Ok(row.and_then(|r| r.try_get_by::<Option<i64>, _>("previous_deployment_id").ok().flatten()))
}
```

**Postgres path pattern** (mirrors claim_postgres lines 310–344):
```rust
async fn promote_postgres(
    conn: &DatabaseConnection,
    owner_key: &str,
    deployment_id: i64,
) -> Result<Option<i64>, Error> {
    let txn = conn.begin().await.map_err(Error::Db)?;
    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "INSERT INTO deployment_pointers (owner_key, deployment_id, previous_deployment_id, updated_at) \
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
        Ok(r) => r,
        Err(e) => { let _ = txn.rollback().await; return Err(Error::Db(e)); }
    };
    txn.commit().await.map_err(Error::Db)?;
    Ok(row.and_then(|r| r.try_get_by::<Option<i64>, _>("previous_deployment_id").ok().flatten()))
}
```

**Pre-promote guard pattern** (placement: inside `Deployments::promote()` in deployment.rs before calling `promote::promote()`):
```rust
// Verify deployment is ready and artifact not deleted before flipping pointer
let dep = self.get(deployment_id).await?;
if dep.status != DeploymentStatus::Ready {
    return Err(Error::NotReady { id: deployment_id });
}
if dep.artifact_deleted_at.is_some() {
    return Err(Error::ArtifactDeleted { id: deployment_id });
}
```

---

### `ferro-deployments/src/storage.rs` (service — DeploymentStorage trait + default impl)

**Analog:** `ferro-storage/src/facade.rs` (Disk struct lines 297–399)

**Disk method surface to delegate to** (lines 308–399):
```rust
// Methods available on ferro_storage::Disk (copy from facade.rs):
disk.put(path, contents).await        // line 324
disk.get(path).await                  // line 313
disk.delete(path).await               // line 341
disk.files(directory).await           // line 376
disk.delete_directory(path).await     // line 393
```

**Trait pattern** (derive from D-11 + facade.rs surface):
```rust
/// Artifact storage abstraction for a deployment prefix.
#[async_trait::async_trait]
pub trait DeploymentStorage: Send + Sync {
    async fn store(&self, deployment_id: i64, path: &str, bytes: bytes::Bytes) -> Result<(), Error>;
    async fn retrieve(&self, deployment_id: i64, path: &str) -> Result<bytes::Bytes, Error>;
    async fn remove(&self, deployment_id: i64, path: &str) -> Result<(), Error>;
    async fn list(&self, deployment_id: i64) -> Result<Vec<String>, Error>;
    async fn remove_all(&self, deployment_id: i64) -> Result<(), Error>;
}
```

**Default impl pattern** (wraps `ferro_storage::Disk`):
```rust
/// Default `DeploymentStorage` backed by a ferro-storage `Disk`.
pub struct StorageDeploymentStorage {
    disk: ferro_storage::Disk,
}

impl StorageDeploymentStorage {
    pub fn new(disk: ferro_storage::Disk) -> Self { Self { disk } }

    fn prefix(deployment_id: i64) -> String {
        format!("deployments/{deployment_id}/")
    }
}

#[async_trait::async_trait]
impl DeploymentStorage for StorageDeploymentStorage {
    async fn store(&self, deployment_id: i64, path: &str, bytes: bytes::Bytes) -> Result<(), Error> {
        let full_path = format!("{}{}", Self::prefix(deployment_id), path);
        self.disk.put(&full_path, bytes).await.map_err(Error::Storage)
    }
    // ... retrieve → disk.get(); remove → disk.delete(); list → disk.files(); remove_all → disk.delete_directory()
}
```

**Storage::with_storage_config construction** (facade.rs lines 164–180 — use for test setup):
```rust
// For tests: use Memory driver
let config = ferro_storage::StorageConfig::new("mem")
    .disk("mem", ferro_storage::DiskConfig::memory());
let storage = ferro_storage::Storage::with_storage_config(config);
let disk = storage.disk("mem").unwrap();
let dep_storage = StorageDeploymentStorage::new(disk);
```

---

### `ferro-deployments/tests/race_promote_sqlite.rs` (test — concurrent promote race)

**Analog:** `ferro-queue/tests/race_claim_sqlite.rs` (exact template, lines 1–99)

**Header comment + NamedTempFile setup** (lines 1–43 — copy and adapt):
```rust
//! SC-2: concurrent last-write-wins promote race test on a shared temp-file SQLite DB.
//!
//! Two concurrent promoters both call promote() for the same owner_key.
//! The pointer must end up in a consistent state — exactly one deployment_id set,
//! no torn state, no DB error.
//!
//! CRITICAL: uses `tempfile::NamedTempFile` + `sqlite://{path}?mode=rwc`.
//! Per-connection in-memory SQLite sees different empty tables (Pitfall 1).

use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use ferro_deployments::{CreateDeploymentsTable, CreateDeploymentPointersTable, Deployments};

struct TestMigrator;
// impl MigratorTrait: both migration structs in order

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_promoters_last_write_wins() {
    let db_file = tempfile::NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}?mode=rwc", db_file.path().display());

    let conn1 = Database::connect(&db_url).await.unwrap();
    let conn2 = Database::connect(&db_url).await.unwrap();
    TestMigrator::up(&conn1, None).await.unwrap();

    // Create two ready deployments
    let deployments1 = Deployments::new(conn1);
    let deployments2 = Deployments::new(conn2);
    // ... create dep_a (ready), dep_b (ready)
    // ... tokio::spawn both promote() calls concurrently
    // ... verify: pointer.deployment_id ∈ {dep_a.id, dep_b.id}, no panic, no DbErr
}
```

**Concurrent drain pattern** (lines 55–80 of analog — adapt for promote):
```rust
// Replace drain() with a parallel pair of tokio::spawn promote() calls.
// Collect results into Arc<Mutex<Vec<Result<...>>>>.
// Assert: both futures completed without error; pointer in valid state.
let (h1, h2) = (
    tokio::spawn(async move { deployments1.promote("owner:1", dep_a_id).await }),
    tokio::spawn(async move { deployments2.promote("owner:1", dep_b_id).await }),
);
let (r1, r2) = tokio::join!(h1, h2);
// assert no torn state: query pointer row directly, verify deployment_id is one of the two
```

---

### `ferro-deployments/tests/race_promote_postgres.rs` (test — Postgres-gated mirror)

**Analog:** `ferro-queue/tests/race_claim_postgres.rs` (exact template, lines 1–137)

**Feature gate + module skip** (lines 18–19 of analog — copy verbatim):
```rust
#![cfg(feature = "postgres-tests")]
```

**`fresh_pg_db()` helper** (lines 46–53 of analog — adapt for deployments tables):
```rust
async fn fresh_pg_db() -> Option<sea_orm::DatabaseConnection> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let conn = sea_orm::Database::connect(&url).await.expect("connect to postgres");
    let _ = TestMigrator::down(&conn, None).await;
    TestMigrator::up(&conn, None).await.expect("migrate");
    Some(conn)
}
```

**Test body pattern** (lines 59–137 of analog):
```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_promoters_last_write_wins_postgres() {
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("DATABASE_URL not set — skipping postgres race test");
        return;
    }
    // same structure as SQLite version; conn setup uses DATABASE_URL
    // --test-threads=1 required (shared database, schema reset on each run)
}
```

---

## Shared Patterns

### ph() — dual-backend SQL placeholder helper
**Source:** `ferro-queue/src/db.rs` lines 278–283
**Apply to:** `promote.rs`, `deployment.rs` (any raw SQL in lifecycle ops)
```rust
fn ph(backend: DatabaseBackend, n: usize) -> String {
    match backend {
        DatabaseBackend::Postgres => format!("${n}"),
        _ => format!("?{n}"),
    }
}
```

### parse_timestamp / parse_optional_timestamp
**Source:** `ferro-queue/src/db.rs` lines 241–275
**Apply to:** `deployment.rs` (parse_deployment_row)
```rust
fn parse_timestamp(row: &sea_orm::QueryResult, col: &str) -> Result<DateTime<Utc>, Error> {
    if let Ok(dt) = row.try_get_by::<DateTime<Utc>, _>(col) { return Ok(dt); }
    let s: String = row.try_get_by::<String, _>(col)
        .map_err(|e| Error::custom(format!("parse {col}: {e}")))?;
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| Error::custom(format!("parse {col} as rfc3339 ('{s}'): {e}")))
}
```

### conn.begin() transaction pinning (CR-01)
**Source:** `ferro-queue/src/db.rs` lines 353–367 (comment block)
**Apply to:** `promote.rs` — all raw SQL paths
```
// SeaORM's begin() checks out ONE physical connection and pins every
// statement on this handle to it. Direct conn.execute() calls can land
// on different pooled connections, breaking atomicity.
let txn = conn.begin().await.map_err(Error::Db)?;
```

### Statement::from_sql_and_values with bound Value::* parameters
**Source:** `ferro-queue/src/db.rs` lines 317–325, 369–381
**Apply to:** `promote.rs` — every raw SQL statement (no string interpolation of caller data)
```rust
Statement::from_sql_and_values(
    backend,
    "...",
    [Value::String(Some(Box::new(s.to_string()))), Value::BigInt(Some(id))],
)
```

### thiserror + one Error enum + From<String> / From<&str>
**Source:** `ferro-queue/src/error.rs` lines 1–119
**Apply to:** `error.rs`

### from_env() config struct
**Source:** `ferro-queue/src/config.rs` lines 47–72
**Apply to:** `config.rs` (DeploymentConfig)

### Serde enum snake_case
**Source:** `ferro-queue/src/db.rs` lines 124–133 (JobState)
**Apply to:** `deployment.rs` (DeploymentStatus)
```rust
#[serde(rename_all = "snake_case")]
pub enum DeploymentStatus { Building, Ready, Failed }
```

### NamedTempFile for cross-connection SQLite tests
**Source:** `ferro-queue/tests/race_claim_sqlite.rs` lines 1–10 (header) + lines 36–37
**Apply to:** `tests/race_promote_sqlite.rs`
```rust
let db_file = tempfile::NamedTempFile::new().unwrap();
let db_url = format!("sqlite://{}?mode=rwc", db_file.path().display());
```

### #![cfg(feature = "postgres-tests")] module gate
**Source:** `ferro-queue/tests/race_claim_postgres.rs` line 18
**Apply to:** `tests/race_promote_postgres.rs` (first line of file)

### ferro-bundle Cargo.toml package header
**Source:** `ferro-bundle/Cargo.toml` lines 1–10
**Apply to:** `ferro-deployments/Cargo.toml` — exact same header structure, adjust name/description/keywords/categories

---

## No Analog Found

No files are without an analog. All 10 new files map to existing patterns in the codebase.

---

## Metadata

**Analog search scope:** `ferro-queue/`, `ferro-storage/src/facade.rs`, `ferro-projection/src/runtime.rs`, `ferro-bundle/Cargo.toml`
**Files scanned:** 10 analog files read in full
**Pattern extraction date:** 2026-06-07
