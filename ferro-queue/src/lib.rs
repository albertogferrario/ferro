//! # Ferro Queue
//!
//! Background job queue system for the Ferro framework.
//!
//! Provides a Laravel-inspired queue system backed by the application database:
//! - SQLite (`BEGIN IMMEDIATE`) and Postgres (`FOR UPDATE SKIP LOCKED`) atomic claim
//! - Job delays, retries with full-jitter exponential backoff, and idempotency keys
//! - Multiple named queues processed in priority order
//! - Panic-isolated worker loop with SIGTERM graceful shutdown
//! - Tenant-scoped job execution
//!
//! ## Example
//!
//! ```rust,ignore
//! use ferro_queue::{Job, Queue, QueueConfig, WorkerLoop, WorkerConfig, Queueable};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! struct SendEmail {
//!     to: String,
//!     subject: String,
//! }
//!
//! #[async_trait::async_trait]
//! impl Job for SendEmail {
//!     async fn handle(&self) -> Result<(), ferro_queue::Error> {
//!         println!("Sending email to {}: {}", self.to, self.subject);
//!         Ok(())
//!     }
//! }
//!
//! // Initialise the queue at application start (once):
//! // Queue::init(db_connection).await?;
//!
//! // Dispatch a job (sync mode by default; set QUEUE_CONNECTION=db for background):
//! SendEmail { to: "user@example.com".into(), subject: "Hello".into() }
//!     .dispatch()
//!     .await?;
//!
//! // Dispatch with delay
//! SendEmail { to: "user@example.com".into(), subject: "Reminder".into() }
//!     .delay(std::time::Duration::from_secs(60))
//!     .on_queue("emails")
//!     .dispatch()
//!     .await?;
//! ```

mod config;
mod db;
mod dispatcher;
mod error;
mod job;
mod migration;
mod worker;

pub use config::QueueConfig;
pub use db::{
    claim, delete_job, enqueue, fail_job, get_delayed_jobs, get_failed_jobs, get_pending_jobs,
    get_stats, reap_startup_claims, reaper, release_job, requeue_claimed_by, FailedJobInfo,
    JobInfo, JobRow, JobState, Queue, QueueStats, SingleQueueStats,
};
pub use dispatcher::{
    dispatch, dispatch_later, dispatch_to, register_tenant_capture_hook, PendingDispatch,
};
pub use error::Error;
pub use job::{Job, JobPayload};
pub use migration::CreateJobsTable;
pub use worker::{TenantScopeProvider, Worker, WorkerConfig, WorkerLoop};

/// Re-export async_trait for convenience
pub use async_trait::async_trait;

/// Trait for types that can be dispatched to a queue.
pub trait Queueable: Job + serde::Serialize + serde::de::DeserializeOwned {
    /// Create a pending dispatch for this job.
    fn dispatch(self) -> PendingDispatch<Self>
    where
        Self: Sized,
    {
        PendingDispatch::new(self)
    }

    /// Dispatch this job with a delay.
    fn delay(self, duration: std::time::Duration) -> PendingDispatch<Self>
    where
        Self: Sized,
    {
        PendingDispatch::new(self).delay(duration)
    }

    /// Dispatch this job to a specific queue.
    fn on_queue(self, queue: &'static str) -> PendingDispatch<Self>
    where
        Self: Sized,
    {
        PendingDispatch::new(self).on_queue(queue)
    }
}

/// Blanket implementation for all types that implement Job + Serialize + DeserializeOwned.
impl<T> Queueable for T where T: Job + serde::Serialize + serde::de::DeserializeOwned {}
