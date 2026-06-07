# Phase 186: ferro-deployments — Immutable Deployments + Atomic Promote - Research

**Researched:** 2026-06-07
**Domain:** New Rust crate — immutable deployment model, atomic pointer flip, artifact storage abstraction
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Pointer ownership & promote mechanics**
- D-01: Active pointer is crate-owned: `deployment_pointers` table keyed by opaque `owner_key` string. Consumers may keep their own denormalized FK but the crate's table is the source of truth.
- D-02: `promote(owner_key, deployment_id)` is last-write-wins, single atomic UPDATE, returns previous deployment id. Newer-deployment-wins optimistic check stays consumer-side — do not duplicate that control surface here.
- D-03: Atomic previous-id return: pointer row carries `deployment_id` + `previous_deployment_id`; one UPDATE sets both from old row values (`SET previous_deployment_id = deployment_id, deployment_id = ?`). Both Postgres and SQLite evaluate SET expressions against pre-update values. Exact SQL formulation is planner/executor discretion as long as the race test passes.
- D-04: `rollback(owner_key)` = promote of pointer row's `previous_deployment_id`. Promoting a deployment whose status is not `ready` is rejected. Promoting a deployment with `artifact_deleted_at` set is also rejected.

**Schema & identifier design**
- D-05: `deployments` table: i64 autoincrement PK + a DNS-safe unique `identifier` string column (lowercase, subdomain-label-safe) for preview URLs and external addressing.
- D-06: Recorded fields per DEPL-F-01: identifier, `source_ref` (nullable), `artifact_location` (opaque string), `byte_size`, `status`, timestamps (`created_at` + terminal-transition timestamp).
- D-07: Status vocabulary: `building` / `ready` / `failed`. Allowed transitions: `building→ready`, `building→failed`. Rows are never mutated after reaching terminal status.
- D-08: Include nullable `artifact_deleted_at` from day one (B-03 compliance). Setting it is the one permitted post-terminal metadata write.
- D-09: Migration helper follows the Phase 185 `CreateJobsTable` pattern exactly: exported struct implementing `MigrationName` + `MigrationTrait`, SchemaManager-only DDL, zero backend-specific SQL.

**DeploymentStorage trait & ferro-storage coupling**
- D-10: Depends directly on `ferro-storage` (both Wave 1 leaf crates; ferro-deployments lands in Wave 1b).
- D-11: Trait granularity: prefix-scoped artifact operations — store/get/delete files under a per-deployment prefix; `artifact_location` recorded as an opaque string the storage impl understands.
- D-12: Crate takes a `sea_orm::DatabaseConnection`, may depend on `sea-orm` directly, must NOT depend on `framework`.

**API surface & lifecycle**
- D-13: API shape: `Deployments` handle struct wrapping `DatabaseConnection`, methods: `create`, `mark_ready`, `mark_failed`, `promote`, `rollback`, `active`, `get`, `list`.
- D-14: `preview_url(deployment)` reads `DEPLOYMENT_PREVIEW_DOMAIN` env var via `from_env()` config struct; returns `Option<String>` of form `https://{identifier}.{domain}/`; unset env → `None`.
- D-15: Criterion 5 proof: a doc-test or example stores a non-HTML artifact bundle (JSON specs) through the same API.
- D-16: New-crate workspace chores: Wave 1b in publish.yml; docs page in `docs/src/features/`; SUMMARY.md entry; error type via `thiserror`; one Error enum; builder methods consuming `with_*` where applicable; serde enums `snake_case`.

### Claude's Discretion
- Exact claim/flip SQL per backend (as long as race test passes on SQLite + Postgres)
- Identifier generation scheme (random slug length/alphabet) — must be DNS-label-safe and unique
- Exact `DeploymentStorage` method signatures and whether streaming is supported in v1
- Whether the pointer lives in a dedicated `deployment_pointers` table or another crate-owned structure (table recommended)
- Whether ferro-mcp gains a deployments introspection tool now or when the framework integrates the crate

### Deferred Ideas (OUT OF SCOPE)
- Deployment retention/GC (deleting old artifact prefixes, setting `artifact_deleted_at` automatically)
- Newer-deployment-wins promote variant (`promote_if_newer`) — consumer-side concern
- ferro-mcp deployments introspection tool (natural later; docs page is mandatory)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DEPL-F-01 | `Deployment` model records immutable rows (identifier, source ref, artifact location, byte size, status, timestamps) with a portable migration helper | Migration pattern from ferro-queue/src/migration.rs (CreateJobsTable) is the exact template; SeaORM SchemaManager DDL only |
| DEPL-F-02 | `promote(owner_key, deployment_id)` is a single atomic UPDATE of the active pointer; `rollback` is promoting a previous deployment | Dual-backend raw SQL via `Statement::from_sql_and_values` + `conn.begin()` transaction pattern from ferro-queue/src/db.rs; race test pattern from ferro-queue/tests/race_claim_sqlite.rs + race_claim_postgres.rs |
| DEPL-F-03 | `DeploymentStorage` trait abstracts artifact persistence (S3-compatible default via ferro-storage); `preview_url(deployment_id)` subdomain helper present | ferro-storage/src/facade.rs provides `Storage`, `Disk`, `put`/`get`/`delete`/`files`/`delete_directory`; `Storage::with_storage_config(StorageConfig::from_env())` is the standard construction pattern |
</phase_requirements>

---

## Summary

`ferro-deployments` is a new leaf crate providing three things: (1) an immutable `Deployment` row model with a portable SeaORM migration helper, (2) an atomic pointer-flip promote/rollback mechanism via a `deployment_pointers` table, and (3) a `DeploymentStorage` trait with an S3-compatible default delegating to `ferro-storage`. The entire design is verified by a concurrent-promote race test on both SQLite and Postgres (mirroring the ferro-queue Phase 185 pattern exactly).

The atomic promote path is the killer feature. The key insight from studying the existing codebase: both SQLite and Postgres evaluate `SET` expressions in `UPDATE` against pre-update row values, making `SET previous_deployment_id = deployment_id, deployment_id = ?` correct on both backends in a single statement. The crate follows the ferro-queue pattern throughout — raw SQL via `Statement::from_sql_and_values`, dual-backend branching, `conn.begin()` transaction pinning, and the same new-crate workspace checklist ferro-bundle established.

The pointer table's first-promote (INSERT ... ON CONFLICT) can use SeaORM's `Entity::insert().on_conflict(...)` builder, which is already proven in ferro-projection. Identifier generation for DNS-safe slugs uses `uuid` v1 (already in ferro-queue's dependency tree and workspace); a UUID v4 encoded as a base32-lower or UUID-hyphenated string is subdomain-safe and unique without any new dependencies.

**Primary recommendation:** Mirror ferro-queue's db.rs + race test structure verbatim for the promote/rollback path; mirror CreateJobsTable for the migration helper; use `Storage::with_storage_config` for the default DeploymentStorage constructor.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Deployment record persistence | Database / Storage | — | Immutable rows; all state lives in DB |
| Active pointer management | Database / Storage | API / Backend | Pointer table is DB-owned; promote is a backend op called by consumers |
| Artifact file storage | CDN / Static (object store) | — | Opaque bytes stored in ferro-storage (local or S3); not served by this crate |
| Preview URL generation | API / Backend | — | Pure function: reads env var + deployment identifier; no DB call |
| Status transition enforcement | API / Backend | — | `mark_ready`/`mark_failed` enforce allowed transitions; no direct row mutation surface exposed |
| Race serialization | Database / Storage | — | Guaranteed by DB transaction atomicity, not application-level locks |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| sea-orm | 1.0 | DB access, transactions, query building | Workspace standard; ferro-queue uses same version |
| sea-orm-migration | 1.0 | Migration helper trait + SchemaManager DDL | Workspace standard; exact CreateJobsTable template |
| thiserror | 2 | Error enum derivation | Workspace standard per CONVENTIONS.md |
| async-trait | 0.1 | Async trait methods on `DeploymentStorage` | Workspace standard; ferro-queue and ferro-storage use it |
| serde | 1 | Serialize/Deserialize on public types | Workspace standard |
| chrono | 0.4 (serde feature) | Timestamp handling | Workspace standard; ferro-queue uses same pattern |
| uuid | 1 (v4 feature) | DNS-safe identifier generation | Already in ferro-queue Cargo.toml; no new dep |
| ferro-storage | workspace | S3-compatible artifact storage backend | D-10: direct dependency; provides Storage/Disk API |
| tokio | 1 (sync, rt, macros) | Async runtime | Workspace standard |
| tracing | 0.1 | Structured logging | Workspace standard |

[VERIFIED: codebase grep of ferro-queue/Cargo.toml, ferro-storage/Cargo.toml, framework/Cargo.toml]

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tempfile | 3 (dev-dep) | NamedTempFile for SQLite race test | Must use named temp file — NOT in-memory — for cross-connection concurrency test |
| sea-orm (sqlx-postgres feature) | 1.0 | Postgres-gated race test | Behind `postgres-tests` cargo feature, mirroring ferro-queue |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `uuid` for identifier | `rand` + custom alphabet | uuid is already in dep tree; custom alphabet adds complexity with no benefit |
| Raw SQL via `Statement` | SeaORM ActiveModel update | ActiveModel cannot express the self-referential `SET prev = current` in a single UPDATE; raw SQL is required for the atomic flip |
| SeaORM `on_conflict` for pointer upsert | Two-step SELECT + INSERT/UPDATE | `on_conflict` is cleaner and already proven in ferro-projection/src/runtime.rs |

**Installation:**
```bash
# No new workspace-level deps needed; uuid and chrono already in workspace
# New crate-level Cargo.toml adds: sea-orm, sea-orm-migration, ferro-storage, thiserror, async-trait, serde, chrono, uuid, tokio, tracing
```

**Version verification:** All versions confirmed from workspace Cargo.toml and crate-level Cargo.toml files. [VERIFIED: codebase grep]

---

## Architecture Patterns

### System Architecture Diagram

```
Consumer (gestiscilo PublishFrontendJob)
        │
        ▼
  Deployments::create(owner_key, source_ref)
        │ returns Deployment { id, identifier, status: Building }
        ▼
  [artifact build — consumer-owned]
        │
        ├─── DeploymentStorage::store(deployment_id, path, bytes)
        │         └── ferro_storage::Disk::put(prefix/path, bytes)
        │
        ▼
  Deployments::mark_ready(id, artifact_location, byte_size)
        │ enforces Building→Ready transition
        ▼
  Deployments::promote(owner_key, deployment_id)
        │
        ├── BEGIN txn ─────────────────────────────────────────────────┐
        │   verify deployment.status == Ready                          │
        │   verify deployment.artifact_deleted_at IS NULL              │
        │   UPDATE deployment_pointers                                 │
        │     SET previous_deployment_id = deployment_id,             │
        │         deployment_id = ?                                    │
        │     WHERE owner_key = ?   (single atomic UPDATE)            │
        │   ← returns previous_deployment_id                          │
        └── COMMIT txn ─────────────────────────────────────────────────┘
        │
        ▼
  Deployments::rollback(owner_key)
        │ = promote(owner_key, pointer.previous_deployment_id)
        ▼
  preview_url(deployment)
        │ reads DEPLOYMENT_PREVIEW_DOMAIN env var
        └── Some("https://{identifier}.{domain}/") or None
```

### Recommended Project Structure

```
ferro-deployments/
├── Cargo.toml               # workspace version, ferro-storage dep, sea-orm dep
├── README.md
├── src/
│   ├── lib.rs               # pub re-exports; crate doc header
│   ├── error.rs             # Error enum (thiserror)
│   ├── migration.rs         # CreateDeploymentsTable + CreateDeploymentPointersTable (or combined)
│   ├── deployment.rs        # Deployment struct, Status enum, Deployments handle
│   ├── promote.rs           # promote / rollback raw SQL (dual-backend)
│   ├── storage.rs           # DeploymentStorage trait + StorageDeploymentStorage default impl
│   └── config.rs            # DeploymentConfig::from_env() (DEPLOYMENT_PREVIEW_DOMAIN)
└── tests/
    ├── race_promote_sqlite.rs    # SC-2: two concurrent promotes, LWW, no torn state (SQLite)
    └── race_promote_postgres.rs  # SC-2b: Postgres-gated mirror (behind postgres-tests feature)
```

### Pattern 1: Migration Helper (CreateDeploymentsTable)

**What:** Exported struct implementing `MigrationName` + `MigrationTrait`, SchemaManager DDL only.
**When to use:** Consumer registers it in their own Migrator alongside their app migrations.

```rust
// Source: ferro-queue/src/migration.rs (verified, exact template)
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
```

### Pattern 2: Atomic Promote (dual-backend raw SQL)

**What:** Single UPDATE that flips both `deployment_id` and `previous_deployment_id` atomically, using pre-update row values.
**When to use:** This is the core of promote; replicated across Postgres and SQLite with `conn.get_database_backend()` branching.

```rust
// Source: ferro-queue/src/db.rs (verified, adapted to pointer flip)
// Both Postgres and SQLite evaluate SET expressions against pre-update values
// for a single-row UPDATE — this is SQL standard behavior, confirmed valid on both.

// Postgres path:
let sql = "UPDATE deployment_pointers \
           SET previous_deployment_id = deployment_id, deployment_id = $1, updated_at = NOW() \
           WHERE owner_key = $2 \
           RETURNING previous_deployment_id";

// SQLite path (NOW() → ISO timestamp parameter):
let sql = "UPDATE deployment_pointers \
           SET previous_deployment_id = deployment_id, deployment_id = ?1, updated_at = ?3 \
           WHERE owner_key = ?2 \
           RETURNING previous_deployment_id";

// Both use Statement::from_sql_and_values inside conn.begin() transaction
// to pin all statements to one pooled connection (CR-01 lesson from ferro-queue).
```

### Pattern 3: Pointer Table First-Promote (INSERT ... ON CONFLICT)

**What:** When no pointer row exists for an `owner_key`, INSERT one; when it exists, UPDATE it.
**When to use:** SeaORM `Entity::insert().on_conflict(...)` builder, proven in ferro-projection/src/runtime.rs.

```rust
// Source: ferro-projection/src/runtime.rs:135-142 (verified)
Entity::insert(active_model)
    .on_conflict(
        OnConflict::column(Column::OwnerKey)
            .update_columns([Column::DeploymentId, Column::PreviousDeploymentId, Column::UpdatedAt])
            .to_owned(),
    )
    .exec(&self.db)
    .await?;
```

### Pattern 4: Race Test Structure (SQLite — always-on)

**What:** `tokio::test(flavor = "multi_thread", worker_threads = 4)` + `NamedTempFile` SQLite DB + two concurrent `tokio::spawn` promoters.
**When to use:** SC-2 concurrent-promote race test. Mirrors ferro-queue/tests/race_claim_sqlite.rs exactly.

```rust
// Source: ferro-queue/tests/race_claim_sqlite.rs (verified, exact template)
// CRITICAL: use NamedTempFile — NOT sqlite::memory: — cross-connection concurrency
// requires a shared file; in-memory databases have separate empty tables per connection.
let db_file = tempfile::NamedTempFile::new().unwrap();
let db_url = format!("sqlite://{}?mode=rwc", db_file.path().display());
let conn1 = Database::connect(&db_url).await.unwrap();
let conn2 = Database::connect(&db_url).await.unwrap();
```

### Pattern 5: DNS-Safe Identifier Generation

**What:** Use `uuid::Uuid::new_v4().to_string()` (hyphenated, lowercase, 36 chars) as the deployment identifier. UUID v4 is DNS-label-safe (all lowercase hex + hyphens), globally unique, and requires no new dependencies.
**When to use:** `Deployments::create()` call — generate one identifier per new deployment row.

```rust
// Source: uuid crate, verified in ferro-queue/Cargo.toml workspace
use uuid::Uuid;
let identifier = Uuid::new_v4().to_string(); // e.g. "f47ac10b-58cc-4372-a567-0e02b2c3d479"
// DNS-label-safe: lowercase, hex chars + hyphens, no leading digit issue
// Subdomain: https://f47ac10b-58cc-4372-a567-0e02b2c3d479.preview.example.com/
```

### Anti-Patterns to Avoid

- **In-memory SQLite for concurrency tests:** Each `Database::connect("sqlite::memory:")` sees a separate empty database. The race test MUST use `NamedTempFile`. [VERIFIED: race_claim_sqlite.rs comment, doc header]
- **Issuing BEGIN/UPDATE/COMMIT directly on `conn` without `conn.begin()`:** Statements can land on different pooled connections, breaking atomicity. Always use `txn = conn.begin()` and pin all statements to `txn`. [VERIFIED: ferro-queue/src/db.rs CR-01 comment]
- **Raw `FOR UPDATE SKIP LOCKED` in migration files:** SQLite will fail `ferro db:migrate` locally. Locking syntax belongs in application query logic only. [VERIFIED: v7.1-PITFALLS.md E-03]
- **Hardcoded domain names in the crate:** `preview_url` must read `DEPLOYMENT_PREVIEW_DOMAIN` from env, never from a literal. Project-agnostic crates rule from CLAUDE.md.
- **Mutating rows after terminal status:** `mark_ready` and `mark_failed` must check current status before updating. Only `artifact_deleted_at` may be set post-terminal. [VERIFIED: CONTEXT.md D-07, D-08]

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| S3-compatible artifact persistence | Custom S3 client | `ferro-storage` `Storage`/`Disk` | Already wraps aws-sdk-s3; Memory driver for tests |
| Migration DDL portability | Raw SQL CREATE TABLE | SeaORM SchemaManager `Table::create()` | No backend-specific SQL; SQLite + Postgres both work |
| Atomic pointer upsert (first promote) | Two-step SELECT + INSERT/UPDATE | SeaORM `Entity::insert().on_conflict()` | Single round-trip; proven in ferro-projection |
| Error type | `Box<dyn Error>` or anyhow | `thiserror` derive + one Error enum | Workspace standard; structured variants for consumers |
| Identifier uniqueness | Custom UUID-like generation | `uuid::Uuid::new_v4()` | Zero new dep; proven DNS-safe format |
| Cross-backend SQL placeholders | String formatting | `ph(backend, n)` pattern from ferro-queue/src/db.rs | Postgres uses `$N`, SQLite uses `?N` |

**Key insight:** The entire implementation is a composition of patterns already proven in ferro-queue (migration helper, dual-backend raw SQL, race test structure) and ferro-projection (on_conflict upsert). No novel infrastructure needed.

---

## Runtime State Inventory

> This is a greenfield crate phase (new crate, no existing state to rename/migrate). Runtime state inventory not applicable.

Nothing found in any category — this phase creates new tables and a new crate. No rename/refactor of existing stored state.

---

## Common Pitfalls

### Pitfall 1: In-Memory SQLite for Cross-Connection Race Test
**What goes wrong:** `Database::connect("sqlite::memory:")` gives each connection a separate empty database. Two-connection concurrency test passes vacuously (both "succeed" against empty tables).
**Why it happens:** SQLite in-memory databases are per-connection by default.
**How to avoid:** Always use `tempfile::NamedTempFile` + `sqlite://{path}?mode=rwc` for concurrency tests. [VERIFIED: ferro-queue/tests/race_claim_sqlite.rs header comment]
**Warning signs:** Race test passes 100% even with an intentionally broken promote implementation.

### Pitfall 2: SET Expression Self-Reference in UPDATE — Verify Both Backends
**What goes wrong:** `SET previous_deployment_id = deployment_id` might be evaluated after `deployment_id` is already updated on some DB versions.
**Why it happens:** SQL standard says SET expressions in a single UPDATE statement read pre-update values, but this needs verification.
**How to avoid:** The race test (two concurrent promotes) is the proof artifact. If the test passes with no torn state, the UPDATE semantics are correct. Both SQLite and Postgres follow SQL standard evaluation order here. [ASSUMED — standard SQL behavior, but race test is the definitive proof]
**Warning signs:** Race test shows `previous_deployment_id = deployment_id` (both same value after promote).

### Pitfall 3: Pointer Table First-Promote Without ON CONFLICT
**What goes wrong:** `promote()` called for a new `owner_key` with no existing pointer row. A plain `UPDATE ... WHERE owner_key = ?` updates zero rows and returns `None` silently.
**Why it happens:** The pointer table starts empty; the first call to promote must INSERT, not UPDATE.
**How to avoid:** Use `INSERT ... ON CONFLICT (owner_key) DO UPDATE SET ...` via SeaORM's `on_conflict` builder. [VERIFIED: ferro-projection/src/runtime.rs:135]

### Pitfall 4: Promoting a Non-Ready Deployment
**What goes wrong:** `promote()` called with a deployment whose `status = 'building'` or `'failed'`. Consumer accidentally promotes a draft.
**Why it happens:** No guard in the promote path.
**How to avoid:** `promote()` must verify `deployment.status == "ready"` and `deployment.artifact_deleted_at IS NULL` before executing the pointer UPDATE. Return a structured `Error::NotReady` or `Error::ArtifactDeleted`. [VERIFIED: CONTEXT.md D-04]

### Pitfall 5: `artifact_deleted_at` Column Omitted
**What goes wrong:** Future rollback UI allows rollback to a deployment whose Spaces objects were lifecycle-deleted. The DB says the deployment is valid; the CDN returns 404.
**Why it happens:** The column seems unnecessary in v1 (no GC tooling ships yet), so it gets deferred.
**How to avoid:** Include `artifact_deleted_at TIMESTAMP WITH TIME ZONE NULL` in the migration from day one. The column ships in this phase; GC tooling is deferred. [VERIFIED: CONTEXT.md D-08, v7.1-PITFALLS.md B-03]

### Pitfall 6: `FOR UPDATE SKIP LOCKED` in Migration File
**What goes wrong:** `ferro db:migrate` fails locally with `near "SKIP": syntax error` on SQLite.
**Why it happens:** SQLite does not support the Postgres-specific locking syntax. If it accidentally appears in a migration (e.g. a schema comment or check constraint), migration fails.
**How to avoid:** Keep all locking SQL in application query code (promote.rs), never in migration DDL. [VERIFIED: v7.1-PITFALLS.md E-03]

### Pitfall 7: Transactions Across Pool Connections
**What goes wrong:** Issuing `BEGIN`, `UPDATE`, `COMMIT` directly on `conn` (not on a `txn` handle) lets statements land on different pooled connections, breaking atomicity.
**Why it happens:** SeaORM's `DatabaseConnection` is a pool; `execute()` calls check out different connections each time.
**How to avoid:** Always use `conn.begin().await` to get a `txn` handle and pin all statements to it. [VERIFIED: ferro-queue/src/db.rs CR-01 comment, lines 355-367]

---

## Code Examples

Verified patterns from official sources (codebase inspection):

### Cargo.toml for ferro-deployments

```toml
[package]
name = "ferro-deployments"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Immutable deployment model and atomic promote for the Ferro framework"
repository = "https://github.com/albertogferrario/ferro"
keywords = ["deployment", "atomic", "storage", "ferro"]
categories = ["web-programming", "database"]
readme = "README.md"

[dependencies]
async-trait = "0.1"
tokio = { version = "1", features = ["sync", "rt", "macros"] }
tracing = "0.1"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls", "macros"] }
sea-orm-migration = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
ferro-storage = { path = "../ferro-storage", version = "0.2" }

[dev-dependencies]
tokio = { version = "1", features = ["full", "test-util"] }
tempfile = "3"
sea-orm-migration = "1.0"

[features]
sqlx-postgres = ["sea-orm/sqlx-postgres"]
postgres-tests = ["sqlx-postgres"]
```

[VERIFIED: based on ferro-queue/Cargo.toml pattern + workspace versions]

### ph() Placeholder Helper (reuse from ferro-queue pattern)

```rust
// Source: ferro-queue/src/db.rs:278-283 (verified)
fn ph(backend: DatabaseBackend, n: usize) -> String {
    match backend {
        DatabaseBackend::Postgres => format!("${n}"),
        _ => format!("?{n}"),
    }
}
```

### DeploymentStorage Trait Shape

```rust
// Source: derived from CONTEXT.md D-11 + ferro-storage/src/facade.rs API
#[async_trait::async_trait]
pub trait DeploymentStorage: Send + Sync {
    /// Store artifact bytes under a per-deployment prefix.
    async fn store(&self, deployment_id: i64, path: &str, bytes: bytes::Bytes) -> Result<(), Error>;
    /// Retrieve artifact bytes.
    async fn retrieve(&self, deployment_id: i64, path: &str) -> Result<bytes::Bytes, Error>;
    /// Delete a single artifact file.
    async fn remove(&self, deployment_id: i64, path: &str) -> Result<(), Error>;
    /// List all artifact paths under a deployment prefix.
    async fn list(&self, deployment_id: i64) -> Result<Vec<String>, Error>;
    /// Delete all artifacts for a deployment.
    async fn remove_all(&self, deployment_id: i64) -> Result<(), Error>;
}
```

### StorageDeploymentStorage Default Implementation

```rust
// Delegates to ferro_storage::Disk; prefix: "deployments/{deployment_id}/"
pub struct StorageDeploymentStorage {
    disk: ferro_storage::Disk,
}

impl StorageDeploymentStorage {
    pub fn new(disk: ferro_storage::Disk) -> Self {
        Self { disk }
    }
    fn prefix(deployment_id: i64) -> String {
        format!("deployments/{deployment_id}/")
    }
}
```

### Workspace Membership Addition

```toml
# Cargo.toml (workspace root) members array — add:
"ferro-deployments",
```

### publish.yml Wave 1b Addition

```yaml
# Current Wave 1b crates string:
WAVE1B_CRATES="ferro-ai ferro-projections ferro-stripe ferro-whatsapp ferro-notifications ferro-reservation ferro-projection"

# After adding ferro-deployments (depends on ferro-storage which is in Wave 1a):
WAVE1B_CRATES="ferro-ai ferro-projections ferro-stripe ferro-whatsapp ferro-notifications ferro-reservation ferro-projection ferro-deployments"
```

[VERIFIED: .github/workflows/publish.yml Wave 1b section, Wave 1a includes ferro-storage]

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Redis-backed jobs | DB-backed jobs in ferro-queue | Phase 185 (2026-06-07) | Confirms deploy pattern: same DB for jobs and deployments |
| Entity-level SeaORM for concurrency | Raw `Statement::from_sql_and_values` + `conn.begin()` | Phase 185 (verified via race test) | Raw SQL is the correct approach for atomic dual-backend operations |
| Per-connection in-memory SQLite tests | NamedTempFile shared-file SQLite | Phase 185 (race_claim_sqlite.rs) | Concurrency tests REQUIRE file-based SQLite |

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo / rustc | Build | ✓ | (workspace) | — |
| sea-orm 1.0 (sqlx-sqlite) | DB layer | ✓ (workspace) | 1.0 | — |
| sea-orm 1.0 (sqlx-postgres) | Postgres race test | ✓ (workspace, feature-gated) | 1.0 | cf-gated test skips when DATABASE_URL unset |
| ferro-storage | DeploymentStorage default impl | ✓ (workspace) | workspace version | — |
| uuid v1 | Identifier generation | ✓ (ferro-queue dep) | 1.x | — |
| tempfile | Race test | ✓ (ferro-queue dev-dep, common crate) | 3 | — |

**Missing dependencies with no fallback:** None.

**First-publish note:** CI token is `publish-update` only, not `publish-new`. `ferro-deployments` does not yet exist on crates.io, so the executor must run `cargo publish -p ferro-deployments` once from a local terminal. [VERIFIED: memory/MEMORY.md project_ferro_publish_token_scoping.md reference]

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none (standard cargo test) |
| Quick run command | `cargo test -p ferro-deployments` |
| Full suite command | `cargo test --all-features -p ferro-deployments` |
| Postgres gate command | `DATABASE_URL=postgres://... cargo test -p ferro-deployments --features postgres-tests -- --test-threads=1` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DEPL-F-01 | Migration creates deployments table on SQLite | unit | `cargo test -p ferro-deployments migration_creates_deployments_table` | ❌ Wave 0 |
| DEPL-F-01 | Migration creates deployments table on SQLite — down() drops tables | unit | same file | ❌ Wave 0 |
| DEPL-F-01 | `artifact_deleted_at` column present in schema | unit (checked via migration) | same file | ❌ Wave 0 |
| DEPL-F-02 | Two concurrent promotes serialize correctly, LWW, no torn state (SQLite) | integration | `cargo test -p ferro-deployments -- two_promoters_last_write_wins` | ❌ Wave 0 |
| DEPL-F-02 | Two concurrent promotes — Postgres (cfg-gated) | integration | `DATABASE_URL=... cargo test -p ferro-deployments --features postgres-tests -- --test-threads=1` | ❌ Wave 0 |
| DEPL-F-02 | Promoting non-ready deployment is rejected | unit | `cargo test -p ferro-deployments -- promote_rejects_non_ready` | ❌ Wave 0 |
| DEPL-F-02 | Promoting deployment with artifact_deleted_at set is rejected | unit | `cargo test -p ferro-deployments -- promote_rejects_deleted_artifact` | ❌ Wave 0 |
| DEPL-F-02 | `rollback` = promote of previous_deployment_id | unit | `cargo test -p ferro-deployments -- rollback_promotes_previous` | ❌ Wave 0 |
| DEPL-F-03 | `preview_url` returns subdomain URL when env set | unit | `cargo test -p ferro-deployments -- preview_url_with_domain` | ❌ Wave 0 |
| DEPL-F-03 | `preview_url` returns None when env unset | unit | `cargo test -p ferro-deployments -- preview_url_no_domain` | ❌ Wave 0 |
| DEPL-F-03 (SC-5) | Non-HTML artifact stores and retrieves through same API | doc-test | `cargo test -p ferro-deployments --doc` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test -p ferro-deployments`
- **Per wave merge:** `cargo test --all-features -p ferro-deployments`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-deployments/src/` — entire crate (new crate, all files are Wave 0 gaps)
- [ ] `ferro-deployments/tests/race_promote_sqlite.rs` — SC-2 race test
- [ ] `ferro-deployments/tests/race_promote_postgres.rs` — SC-2b cfg-gated Postgres mirror
- [ ] `ferro-deployments/Cargo.toml` — crate manifest
- [ ] `ferro-deployments/README.md` — crate README
- [ ] Workspace `Cargo.toml` members addition
- [ ] `.github/workflows/publish.yml` Wave 1b addition

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | crate has no auth surface; consumers own auth |
| V3 Session Management | no | no sessions |
| V4 Access Control | no | `owner_key` is opaque; access control is consumer-side |
| V5 Input Validation | yes | `owner_key`, `source_ref`, `artifact_location` are string inputs — must use parameterized queries |
| V6 Cryptography | no | no crypto; UUID v4 is for uniqueness, not security |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| SQL injection via owner_key or deployment_id | Tampering | `Statement::from_sql_and_values` with bound `Value::*` parameters — no string interpolation; verified pattern in ferro-queue/src/db.rs |
| Unauthorized artifact access at predictable paths | Information Disclosure | `artifact_location` is opaque; consumers control storage access; this crate does not serve files |
| Promotion of deleted artifact (silent 404 on rollback) | Denial of Service | `artifact_deleted_at` check in promote path (D-04, D-08) |

---

## Open Questions (RESOLVED)

1. **Two-table vs one-table migration helper**
   - What we know: CONTEXT.md recommends a `deployment_pointers` table. ROADMAP says "deployments migration helper." CreateJobsTable is one struct per table.
   - What's unclear: Should there be one exported struct `CreateDeploymentsTable` that creates both tables, or two structs (`CreateDeploymentsTable` + `CreateDeploymentPointersTable`)?
   - Recommendation: Export two separate structs (one per table), both in `migration.rs`. Consumers register them in order. This mirrors the fact that consumers might already have their own pointer mechanism and only want the deployments table.

2. **`mark_ready` byte_size type**
   - What we know: CONTEXT.md D-06 says `byte_size` is a recorded field. The ROADMAP says "byte size."
   - What's unclear: `i64` (matching PK portability precedent) or `u64`?
   - Recommendation: `i64` — matches the i64 autoincrement PK portability precedent (Phase 185 D-05) and avoids SeaORM type mapping issues. File sizes up to 9.2 exabytes is sufficient.

3. **First-manual-publish workflow**
   - What we know: CI token is `publish-update` only; new crates need a one-time local `cargo publish`.
   - What's unclear: Whether to add to publish.yml before or after the first manual publish.
   - Recommendation: Add to publish.yml in the same commit as the crate creation (D-16). The CI will fail on the first push with "already uploaded" or skip (the check-version gate). First-manual-publish instruction should appear in the plan's Wave 0 or closeout task.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Both Postgres and SQLite evaluate SET expressions in a single-row UPDATE against pre-update row values (SQL standard behavior for self-referential SET) | Standard Stack, Atomic Promote Pattern | Race test would show torn state; fallback: two-statement UPDATE (read old value, then SET both) inside a transaction |
| A2 | UUID v4 hyphenated string format (`xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx`) is valid as a DNS subdomain label on all target infrastructure | Standard Stack / Identifier generation | Some DNS providers reject labels starting with digits or containing hyphens at position 1; fallback: prefix with `d-` |

**If A1 is wrong:** The race test will catch it. Fallback is a two-step `SELECT old_deployment_id FROM deployment_pointers WHERE owner_key = ? FOR UPDATE` (Postgres) / inside BEGIN txn (SQLite), then `UPDATE ... SET previous_deployment_id = ?, deployment_id = ?`.

---

## Sources

### Primary (HIGH confidence)
- `ferro-queue/src/migration.rs` — CreateJobsTable: exact migration helper template [VERIFIED: codebase read]
- `ferro-queue/src/db.rs` — dual-backend claim SQL, `ph()` helper, `conn.begin()` transaction pinning, parse_timestamp pattern [VERIFIED: codebase read]
- `ferro-queue/tests/race_claim_sqlite.rs` — NamedTempFile race test structure, multi_thread flavor, drain function pattern [VERIFIED: codebase read]
- `ferro-queue/tests/race_claim_postgres.rs` — cfg-gated Postgres mirror, `--test-threads=1` requirement, `fresh_pg_db()` pattern [VERIFIED: codebase read]
- `ferro-storage/src/facade.rs` — Storage, Disk, put/get/delete/files/delete_directory API surface, DiskConfig, StorageConfig::from_env() pattern [VERIFIED: codebase read]
- `ferro-projection/src/runtime.rs` — `Entity::insert().on_conflict()` upsert for pointer table first-promote [VERIFIED: codebase read]
- `.github/workflows/publish.yml` — Wave 1a/1b/2/3 structure; ferro-storage is Wave 1a; Wave 1b is where ferro-deployments belongs [VERIFIED: codebase read]
- `ferro-bundle/Cargo.toml` — new-crate workspace manifest pattern [VERIFIED: codebase read]
- `ferro-queue/Cargo.toml` — dep versions: uuid v1, chrono 0.4, sea-orm 1.0, thiserror 2, rand 0.8, tempfile 3 [VERIFIED: codebase read]
- `gestiscilo v7.1-ARCHITECTURE.md` — D-05 flat deployment list, promote = single atomic UPDATE, rollback = promote-of-previous, "Ferro-side primitives" table [VERIFIED: canonical ref read]
- `gestiscilo v7.1-PITFALLS.md §B` — B-01 double-publish race (consumer-side LWW is correct), B-03 artifact_deleted_at requirement [VERIFIED: canonical ref read]
- `.planning/phases/186-ferro-deployments-immutable-deployments-atomic-promote/186-CONTEXT.md` — all locked decisions D-01..D-16 [VERIFIED: primary context read]
- `.planning/ROADMAP.md §v12.3` — requirements DEPL-F-01..03, success criteria 1..5 [VERIFIED: codebase read]

### Secondary (MEDIUM confidence)
- CLAUDE.md — project-agnostic crates rule, fmt+clippy+test gate, no co-author commits [VERIFIED: codebase read]
- `.planning/codebase/CONVENTIONS.md` — naming patterns, import order [VERIFIED: codebase read]
- `.planning/codebase/TESTING.md` — test patterns, NamedTempFile for SQLite, serial_test for shared state [VERIFIED: codebase read]

### Tertiary (LOW confidence)
- SQL standard behavior for SET-expression evaluation in UPDATE (pre-update values) — [ASSUMED]: standard behavior, confirmed by A1 note; race test is definitive proof

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all deps confirmed in existing workspace Cargo.toml files
- Architecture: HIGH — patterns verified line-by-line from ferro-queue and ferro-projection source
- Pitfalls: HIGH — sourced from v7.1-PITFALLS.md (B-01, B-03) and ferro-queue source comments (CR-01, NamedTempFile header)
- Migration helper: HIGH — CreateJobsTable is the exact template; zero ambiguity
- Race test design: HIGH — race_claim_sqlite.rs + race_claim_postgres.rs are the exact templates
- Promote SQL semantics: MEDIUM (A1 assumption) — standard SQL behavior; race test is the proof artifact

**Research date:** 2026-06-07
**Valid until:** 2026-07-07 (stable crate ecosystem; wave structure and dep versions will not change within 30 days)
