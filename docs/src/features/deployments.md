# Deployments

`ferro-deployments` provides immutable deployment rows, atomic pointer promotion, and an artifact storage abstraction. Each deployment is an append-only database record. A per-owner pointer row tracks the active and previous deployment IDs, enabling atomic promotion and single-step rollback without custom SQL in the consumer.

Artifact shape is opaque — the crate stores and retrieves raw bytes. Static site bundles, JSON-UI spec bundles, SSR manifests, and any other byte sequence all fit without crate-level assumptions about content type.

## Setup

### Migration

Register both migration helpers in your application's `Migrator` before your own migrations:

```rust
use ferro_deployments::{CreateDeploymentsTable, CreateDeploymentPointersTable};
use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(CreateDeploymentsTable),
            Box::new(CreateDeploymentPointersTable),
            // ... your own migrations
        ]
    }
}
```

`CreateDeploymentsTable` creates the `deployments` table. `CreateDeploymentPointersTable` creates the `deployment_pointers` table. Both use the SeaORM SchemaManager DDL builder and are portable across SQLite and Postgres.

### Environment Variables

```env
# Optional: wildcard subdomain domain for preview URLs.
# When set, preview_url() returns "https://{identifier}.{domain}/".
DEPLOYMENT_PREVIEW_DOMAIN=preview.example.com
```

## Lifecycle API

The `Deployments` handle wraps a `DatabaseConnection` and exposes the full lifecycle:

```rust
use ferro_deployments::Deployments;

let deps = Deployments::new(db);
```

### Creating a deployment

```rust
// owner_key scopes the deployment (e.g. tenant slug, project name).
// source_ref is optional (branch name, commit SHA, tag).
let d = deps.create("project:my-app", Some("sha-abc123")).await?;
// d.status == DeploymentStatus::Building
// d.identifier is a UUID v4 string, stable across retries.
```

### Transitioning to ready

After the artifact is built and stored, record the artifact location and byte size:

```rust
deps.mark_ready(d.id, "deployments/42/", 98304).await?;
```

### Marking a build as failed

```rust
deps.mark_failed(d.id, "compiler error: ...").await?;
```

The error string is emitted via `tracing::warn` — there is no error column in the schema.

### Fetching deployments

```rust
// By primary key.
let deployment = deps.get(id).await?;

// All deployments for an owner, newest first.
let all = deps.list("project:my-app").await?;

// Currently active deployment (None when no pointer row exists yet).
let active = deps.active("project:my-app").await?;
```

## Atomic Promote Model

`promote` flips the active pointer in a single `INSERT … ON CONFLICT DO UPDATE … RETURNING` statement. No separate SELECT-then-UPDATE — the atomic upsert preserves the previous pointer in the same operation.

```rust
// Returns Some(previous_id) or None on first promotion.
let previous_id = deps.promote("project:my-app", d.id).await?;
```

Promote enforces two guards:

- `Error::NotReady` — the deployment is not in `ready` status.
- `Error::ArtifactDeleted` — `artifact_deleted_at` is set; the artifact was garbage-collected.

### Rollback

Rollback is promote-of-previous: the pointer row's `previous_deployment_id` is read and promoted. All guards apply — the previous deployment must still be `ready` with its artifact intact.

```rust
// Promotes the previous deployment back to active.
deps.rollback("project:my-app").await?;
```

Returns `Error::NoPreviousDeployment` when the pointer row has no `previous_deployment_id` (first deployment, or previous was cleared).

## Artifact Storage

`DeploymentStorage` is an async trait abstracting five prefix-scoped operations. All object keys are scoped to `deployments/{deployment_id}/` — each deployment gets an isolated key namespace.

```rust
pub trait DeploymentStorage: Send + Sync {
    async fn store(&self, deployment_id: i64, path: &str, bytes: Bytes) -> Result<(), Error>;
    async fn retrieve(&self, deployment_id: i64, path: &str) -> Result<Bytes, Error>;
    async fn remove(&self, deployment_id: i64, path: &str) -> Result<(), Error>;
    async fn list(&self, deployment_id: i64) -> Result<Vec<String>, Error>;
    async fn remove_all(&self, deployment_id: i64) -> Result<(), Error>;
}
```

### Default implementation

`StorageDeploymentStorage` delegates to a `ferro_storage::Disk`:

```rust
use ferro_deployments::StorageDeploymentStorage;
use ferro_storage::{DiskConfig, Storage, StorageConfig};

let config = StorageConfig::from_env();
let disk = Storage::with_storage_config(config).disk("default")?;
let storage = StorageDeploymentStorage::new(disk);
```

For testing, use the memory driver:

```rust
let config = StorageConfig::new("mem").disk("mem", DiskConfig::memory());
let disk = Storage::with_storage_config(config).disk("mem").unwrap();
let storage = StorageDeploymentStorage::new(disk);
```

### Storing and retrieving an artifact

```rust
use ferro_deployments::DeploymentStorage;
use bytes::Bytes;

let spec = Bytes::from(serde_json::to_vec(&my_spec)?);
storage.store(d.id, "spec.json", spec.clone()).await?;

// Later, or in the serving path:
let retrieved = storage.retrieve(d.id, "spec.json").await?;
```

## Preview URLs

`preview_url` formats a wildcard-subdomain URL for a deployment. It reads the domain exclusively from `DeploymentConfig.preview_domain` (`DEPLOYMENT_PREVIEW_DOMAIN` env var) — no domain is hardcoded.

```rust
use ferro_deployments::{DeploymentConfig, preview_url};

let config = DeploymentConfig::from_env();
// Pass the deployment identifier string directly.
if let Some(url) = preview_url(&config, &deployment.identifier) {
    println!("Preview: {url}");
    // "https://{identifier}.preview.example.com/"
}
```

**Preview URLs are publicly addressable by design.** The subdomain identifier is not an access-control token and does not restrict who can fetch the URL. The consumer application owns authorization for preview routes.

## Error Reference

| Variant | Meaning |
|---------|---------|
| `Error::Db(DbErr)` | Database operation failed |
| `Error::NotFound { id }` | No deployment row with the given id |
| `Error::NotReady { id }` | Promote rejected — deployment is not in `ready` status |
| `Error::ArtifactDeleted { id }` | Promote rejected — artifact was garbage-collected |
| `Error::NoPreviousDeployment { owner_key }` | Rollback rejected — no previous pointer |
| `Error::Storage(StorageError)` | Underlying storage operation failed |
