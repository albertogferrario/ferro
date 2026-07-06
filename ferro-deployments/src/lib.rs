//! # ferro-deployments
//!
//! Immutable deployment rows and atomic pointer promotion for the Ferro framework.
//!
//! ## Overview
//!
//! Each deployment is recorded as an append-only row in the `deployments` table.
//! A per-owner `deployment_pointers` row tracks which deployment is currently active
//! and which was the previous one, enabling atomic promotion and rollback.
//!
//! Artifact storage is abstracted through [`ferro_storage`] so the same crate works
//! with local filesystem, S3, or any other configured driver. The artifact shape is
//! opaque — static site bundles, JSON-UI spec bundles, SSR manifests, and any other
//! byte sequence all fit without crate-level assumptions.
//!
//! ## Quick Start
//!
//! Register both migration helpers in your `Migrator`, then use
//! [`DeploymentConfig::from_env`] to read operator configuration.
//!
//! ```rust,ignore
//! use ferro_deployments::{CreateDeploymentsTable, CreateDeploymentPointersTable};
//!
//! impl MigratorTrait for Migrator {
//!     fn migrations() -> Vec<Box<dyn MigrationTrait>> {
//!         vec![
//!             Box::new(CreateDeploymentsTable),
//!             Box::new(CreateDeploymentPointersTable),
//!         ]
//!     }
//! }
//! ```
//!
//! ## Full lifecycle example (criterion 5 — JSON spec artifact)
//!
//! This example stores a JSON spec bundle through the full lifecycle API:
//! `create` → `store` → `mark_ready` → `promote` → `retrieve`. The artifact
//! is opaque bytes — the crate makes no assumption about content type.
//!
//! ```rust
//! # use ferro_deployments::{
//! #     CreateDeploymentPointersTable, CreateDeploymentsTable, DeploymentStorage,
//! #     Deployments, DeploymentStatus, StorageDeploymentStorage,
//! # };
//! # use ferro_storage::{DiskConfig, Storage, StorageConfig};
//! # use bytes::Bytes;
//! # use sea_orm::Database;
//! # use sea_orm_migration::MigratorTrait;
//! #
//! # struct DocMigrator;
//! # #[async_trait::async_trait]
//! # impl MigratorTrait for DocMigrator {
//! #     fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
//! #         vec![
//! #             Box::new(CreateDeploymentsTable),
//! #             Box::new(CreateDeploymentPointersTable),
//! #         ]
//! #     }
//! # }
//! #
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! // Connect to an in-memory SQLite database and run migrations.
//! let conn = Database::connect("sqlite::memory:").await.unwrap();
//! DocMigrator::up(&conn, None).await.unwrap();
//!
//! // Create a deployment handle and register a new build.
//! let deps = Deployments::new(conn);
//! let d = deps.create("project:demo", Some("sha-abcdef")).await.unwrap();
//! assert_eq!(d.status, DeploymentStatus::Building);
//!
//! // Build a memory-backed storage adapter and store a JSON spec bundle.
//! let storage_config = StorageConfig::new("mem").disk("mem", DiskConfig::memory());
//! let disk = Storage::with_storage_config(storage_config).disk("mem").unwrap();
//! let storage = StorageDeploymentStorage::new(disk);
//!
//! let spec_json = Bytes::from_static(br#"{"intent":"browse","fields":[]}"#);
//! storage.store(d.id, "spec.json", spec_json.clone()).await.unwrap();
//!
//! // Transition the deployment to ready, recording artifact location and size.
//! let artifact_location = format!("deployments/{}/", d.id);
//! deps.mark_ready(d.id, &artifact_location, spec_json.len() as i64)
//!     .await
//!     .unwrap();
//!
//! // Atomically promote the deployment.
//! let previous = deps.promote("project:demo", d.id).await.unwrap();
//! assert!(previous.is_none(), "first promotion has no previous deployment");
//!
//! // Retrieve the stored artifact and verify round-trip fidelity.
//! let retrieved = storage.retrieve(d.id, "spec.json").await.unwrap();
//! assert_eq!(retrieved, spec_json);
//! # });
//! ```

mod config;
pub(crate) mod deployment;
mod error;
mod migration;
pub(crate) mod promote;
mod storage;

pub use config::DeploymentConfig;
pub use deployment::{Deployment, DeploymentStatus, Deployments};
pub use error::Error;
pub use migration::{CreateDeploymentPointersTable, CreateDeploymentsTable};
pub use storage::{preview_url, DeploymentStorage, StorageDeploymentStorage};
