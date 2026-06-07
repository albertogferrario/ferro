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
//! with local filesystem, S3, or any other configured driver.
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
