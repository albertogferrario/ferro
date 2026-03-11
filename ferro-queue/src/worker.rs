//! Queue worker for processing jobs.

use crate::{Error, Job, JobPayload, QueueConnection};
use async_trait::async_trait;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

/// Injects tenant scope around job execution.
///
/// Implemented by the framework — injected at startup via `Worker::with_tenant_scope()`.
/// The provider receives a tenant_id and a boxed future representing the job execution.
/// It must resolve the tenant, establish a task-local scope, and run the future within it.
/// Returns `TenantNotFound` error if the tenant ID does not resolve to a valid tenant.
#[async_trait]
pub trait TenantScopeProvider: Send + Sync {
    /// Run the given future within a tenant scope for the specified tenant.
    async fn with_scope(
        &self,
        tenant_id: i64,
        f: Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>,
    ) -> Result<(), Error>;
}

/// Worker configuration.
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Queues to process (in priority order).
    pub queues: Vec<String>,
    /// Maximum concurrent jobs.
    pub max_jobs: usize,
    /// Sleep duration when no jobs are available.
    pub sleep_duration: Duration,
    /// Whether to stop on error.
    pub stop_on_error: bool,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            queues: vec!["default".to_string()],
            max_jobs: 10,
            sleep_duration: Duration::from_secs(1),
            stop_on_error: false,
        }
    }
}

impl WorkerConfig {
    /// Create a new worker config for specific queues.
    pub fn new(queues: Vec<String>) -> Self {
        Self {
            queues,
            ..Default::default()
        }
    }

    /// Set max concurrent jobs.
    pub fn max_jobs(mut self, max: usize) -> Self {
        self.max_jobs = max;
        self
    }
}

/// Type alias for job handler functions.
type JobHandler =
    Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> + Send + Sync>;

/// Queue worker that processes jobs.
pub struct Worker {
    /// Queue connection.
    connection: QueueConnection,
    /// Worker configuration.
    config: WorkerConfig,
    /// Job handlers by type name.
    handlers: HashMap<String, JobHandler>,
    /// Semaphore for limiting concurrent jobs.
    semaphore: Arc<Semaphore>,
    /// Shutdown flag.
    shutdown: Arc<tokio::sync::Notify>,
}

impl Worker {
    /// Create a new worker.
    pub fn new(connection: QueueConnection, config: WorkerConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_jobs));
        Self {
            connection,
            config,
            handlers: HashMap::new(),
            semaphore,
            shutdown: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Register a job handler.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// worker.register::<SendEmailJob>();
    /// ```
    pub fn register<J>(&mut self)
    where
        J: Job + serde::de::DeserializeOwned + 'static,
    {
        let type_name = std::any::type_name::<J>().to_string();

        let handler: JobHandler = Arc::new(move |data: String| {
            Box::pin(async move {
                let job: J = serde_json::from_str(&data)
                    .map_err(|e| Error::DeserializationFailed(e.to_string()))?;
                job.handle().await
            })
        });

        self.handlers.insert(type_name, handler);
    }

    /// Run the worker until shutdown.
    pub async fn run(&self) -> Result<(), Error> {
        info!(
            queues = ?self.config.queues,
            max_jobs = self.config.max_jobs,
            "Starting queue worker"
        );

        // Spawn delayed job migrator
        let conn = self.connection.clone();
        let queues = self.config.queues.clone();
        let shutdown = self.shutdown.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown.notified() => break,
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {
                        for queue in &queues {
                            if let Err(e) = conn.migrate_delayed(queue).await {
                                error!(queue = queue, error = %e, "Failed to migrate delayed jobs");
                            }
                        }
                    }
                }
            }
        });

        // Main processing loop
        loop {
            tokio::select! {
                _ = self.shutdown.notified() => {
                    info!("Worker shutting down");
                    // Wait for all in-flight jobs to complete
                    info!("Waiting for in-flight jobs to complete");
                    let _ = self.semaphore.acquire_many(self.config.max_jobs as u32).await;
                    return Ok(());
                }
                result = self.process_next() => {
                    if let Err(e) = result {
                        error!(error = %e, "Error processing job");
                        if self.config.stop_on_error {
                            return Err(e);
                        }
                    }
                }
            }
        }
    }

    /// Process the next available job.
    async fn process_next(&self) -> Result<(), Error> {
        // Try each queue in order
        for queue in &self.config.queues {
            if let Some(payload) = self.connection.pop_nowait(queue).await? {
                self.process_job(payload).await?;
                return Ok(());
            }
        }

        // No jobs available, sleep briefly
        tokio::time::sleep(self.config.sleep_duration).await;
        Ok(())
    }

    /// Process a single job.
    async fn process_job(&self, payload: JobPayload) -> Result<(), Error> {
        let permit = self.semaphore.clone().acquire_owned().await.unwrap();
        let connection = self.connection.clone();
        let handlers = self.handlers.clone();
        let job_type = payload.job_type.clone();
        let job_id = payload.id;

        tokio::spawn(async move {
            let _permit = permit; // Hold permit until job completes

            debug!(job_id = %job_id, job_type = &job_type, "Processing job");

            let handler = match handlers.get(&job_type) {
                Some(h) => h,
                None => {
                    warn!(job_type = &job_type, "No handler registered for job type");
                    return;
                }
            };

            match handler(payload.data.clone()).await {
                Ok(()) => {
                    info!(job_id = %job_id, job_type = &job_type, "Job completed successfully");
                }
                Err(e) => {
                    error!(job_id = %job_id, job_type = &job_type, error = %e, "Job failed");

                    if payload.has_exceeded_retries() {
                        warn!(job_id = %job_id, "Job exceeded max retries, moving to failed queue");
                        if let Err(e) = connection.fail(payload, &e).await {
                            error!(error = %e, "Failed to move job to failed queue");
                        }
                    } else {
                        // Calculate retry delay (exponential backoff)
                        let delay = Duration::from_secs(2u64.pow(payload.attempts));
                        if let Err(e) = connection.release(payload, delay).await {
                            error!(error = %e, "Failed to release job for retry");
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Signal the worker to shut down gracefully.
    pub fn shutdown(&self) {
        self.shutdown.notify_waiters();
    }
}

// Allow handlers to be cloned for spawning
impl Clone for Worker {
    fn clone(&self) -> Self {
        Self {
            connection: self.connection.clone(),
            config: self.config.clone(),
            handlers: HashMap::new(), // Handlers can't be cloned, new instance starts empty
            semaphore: self.semaphore.clone(),
            shutdown: self.shutdown.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies TenantScopeProvider is object-safe (can be wrapped in Arc<dyn TenantScopeProvider>).
    #[test]
    fn test_tenant_scope_provider_is_object_safe() {
        struct NoopProvider;

        #[async_trait]
        impl TenantScopeProvider for NoopProvider {
            async fn with_scope(
                &self,
                _tenant_id: i64,
                f: Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>,
            ) -> Result<(), Error> {
                f.await
            }
        }

        // If this compiles, the trait is object-safe.
        let _provider: Arc<dyn TenantScopeProvider> = Arc::new(NoopProvider);
    }
}
