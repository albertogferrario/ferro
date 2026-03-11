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
    /// Optional tenant scope provider for tenant-aware job execution.
    tenant_scope: Option<Arc<dyn TenantScopeProvider>>,
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
            tenant_scope: None,
        }
    }

    /// Inject a tenant scope provider for tenant-aware job execution.
    ///
    /// When set, jobs with a `tenant_id` in their payload are executed inside
    /// a tenant context scope. Jobs without a tenant_id or workers without a
    /// provider run normally without any scope.
    pub fn with_tenant_scope(mut self, provider: Arc<dyn TenantScopeProvider>) -> Self {
        self.tenant_scope = Some(provider);
        self
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
        let tenant_scope = self.tenant_scope.clone();
        let tenant_id = payload.tenant_id;

        tokio::spawn(async move {
            let _permit = permit; // Hold permit until job completes

            debug!(
                job_id = %job_id,
                job_type = &job_type,
                tenant_id = ?tenant_id,
                "Processing job"
            );

            let handler = match handlers.get(&job_type) {
                Some(h) => h,
                None => {
                    warn!(job_type = &job_type, "No handler registered for job type");
                    return;
                }
            };

            // Wrap execution in tenant scope when tenant_id is present and provider is configured.
            // IMPORTANT: with_scope() is called inside the spawn — task-locals do not cross spawn boundaries.
            let job_result = match (&tenant_scope, tenant_id) {
                (Some(scope), Some(id)) => {
                    let job_fut = Box::pin(handler(payload.data.clone()));
                    scope.with_scope(id, job_fut).await
                }
                _ => handler(payload.data.clone()).await,
            };

            match job_result {
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
            tenant_scope: self.tenant_scope.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

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

    /// Mock TenantScopeProvider that tracks calls and optionally fails.
    struct MockScopeProvider {
        called_with: Arc<Mutex<Vec<i64>>>,
        should_fail: bool,
    }

    impl MockScopeProvider {
        fn new() -> Self {
            Self {
                called_with: Arc::new(Mutex::new(Vec::new())),
                should_fail: false,
            }
        }

        fn failing() -> Self {
            Self {
                called_with: Arc::new(Mutex::new(Vec::new())),
                should_fail: true,
            }
        }
    }

    #[async_trait]
    impl TenantScopeProvider for MockScopeProvider {
        async fn with_scope(
            &self,
            tenant_id: i64,
            f: Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>,
        ) -> Result<(), Error> {
            self.called_with.lock().unwrap().push(tenant_id);
            if self.should_fail {
                return Err(Error::tenant_not_found(tenant_id));
            }
            f.await
        }
    }

    /// Spawn a minimal fake Redis server and return a connected Worker.
    ///
    /// The fake server accepts TCP connections and echoes "+OK\r\n" for every
    /// incoming line. This satisfies ConnectionManager's initial CLIENT SETINFO
    /// handshake without needing a real Redis instance.
    async fn make_worker() -> Worker {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        // Fake Redis: accept connections, respond "+OK\r\n" to every RESP line.
        tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = listener.accept().await else {
                    break;
                };
                tokio::spawn(async move {
                    let (reader, mut writer) = stream.split();
                    let mut lines = BufReader::new(reader).lines();
                    while let Ok(Some(_line)) = lines.next_line().await {
                        // Respond OK to every command line. Real Redis sends back
                        // per-command replies; CLIENT SETINFO is silently ignored
                        // by the client (`pipe!().cmd(...).ignore()`), so any reply works.
                        let _ = writer.write_all(b"+OK\r\n").await;
                    }
                });
            }
        });

        let config = crate::QueueConfig::new(format!("redis://127.0.0.1:{port}"));
        let conn = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            crate::QueueConnection::new(config),
        )
        .await
        .expect("fake Redis connection timed out")
        .expect("fake Redis connection failed");

        Worker::new(conn, WorkerConfig::default())
    }

    /// Worker::with_tenant_scope() stores the provider (field is Some after call).
    #[tokio::test]
    async fn test_with_tenant_scope_stores_provider() {
        let worker = make_worker().await;
        let provider = Arc::new(MockScopeProvider::new());
        let worker = worker.with_tenant_scope(provider);
        assert!(
            worker.tenant_scope.is_some(),
            "tenant_scope must be Some after with_tenant_scope()"
        );
    }

    /// Worker without tenant_scope has None by default.
    #[tokio::test]
    async fn test_worker_without_scope_has_none_by_default() {
        let worker = make_worker().await;
        assert!(
            worker.tenant_scope.is_none(),
            "tenant_scope must be None by default"
        );
    }

    /// Worker::clone() preserves the tenant_scope field.
    #[tokio::test]
    async fn test_clone_preserves_tenant_scope() {
        let worker = make_worker().await;
        let provider: Arc<dyn TenantScopeProvider> = Arc::new(MockScopeProvider::new());
        let worker = worker.with_tenant_scope(provider);
        let cloned = worker.clone();
        assert!(
            cloned.tenant_scope.is_some(),
            "Clone must preserve tenant_scope"
        );
    }

    /// Worker clone without scope also has None.
    #[tokio::test]
    async fn test_clone_without_scope_preserves_none() {
        let worker = make_worker().await;
        let cloned = worker.clone();
        assert!(
            cloned.tenant_scope.is_none(),
            "Clone must preserve None tenant_scope"
        );
    }

    /// MockScopeProvider: with_scope calls the job future and records the tenant_id.
    #[tokio::test]
    async fn test_mock_scope_provider_calls_future() {
        let provider = MockScopeProvider::new();
        let calls = provider.called_with.clone();

        let result = provider.with_scope(42, Box::pin(async { Ok(()) })).await;

        assert!(result.is_ok());
        assert_eq!(calls.lock().unwrap().as_slice(), &[42]);
    }

    /// MockScopeProvider: failing variant returns TenantNotFound.
    #[tokio::test]
    async fn test_mock_scope_provider_failure_returns_tenant_not_found() {
        let provider = MockScopeProvider::failing();

        let result = provider.with_scope(99, Box::pin(async { Ok(()) })).await;

        assert!(matches!(
            result,
            Err(Error::TenantNotFound { tenant_id: 99 })
        ));
    }

    /// scope_dispatch_for_tenant: Some(id) + provider -> with_scope called.
    #[tokio::test]
    async fn test_scope_dispatch_tenant_id_some_calls_with_scope() {
        let mock = MockScopeProvider::new();
        let calls = mock.called_with.clone();
        let provider: Arc<dyn TenantScopeProvider> = Arc::new(mock);

        // Simulate the match logic from process_job
        let tenant_id: Option<i64> = Some(1);
        let tenant_scope: Option<Arc<dyn TenantScopeProvider>> = Some(provider);

        let job_ran = Arc::new(Mutex::new(false));
        let job_ran_clone = job_ran.clone();
        let job_fut = Box::pin(async move {
            *job_ran_clone.lock().unwrap() = true;
            Ok(())
        });

        let result = match (&tenant_scope, tenant_id) {
            (Some(scope), Some(id)) => scope.with_scope(id, job_fut).await,
            _ => job_fut.await,
        };

        assert!(result.is_ok());
        assert_eq!(calls.lock().unwrap().as_slice(), &[1i64]);
        assert!(*job_ran.lock().unwrap(), "job future must have been called");
    }

    /// scope_dispatch_no_tenant_id: None + provider -> with_scope NOT called.
    #[tokio::test]
    async fn test_scope_dispatch_tenant_id_none_skips_with_scope() {
        let mock = MockScopeProvider::new();
        let calls = mock.called_with.clone();
        let provider: Arc<dyn TenantScopeProvider> = Arc::new(mock);

        let tenant_id: Option<i64> = None;
        let tenant_scope: Option<Arc<dyn TenantScopeProvider>> = Some(provider);

        let job_ran = Arc::new(Mutex::new(false));
        let job_ran_clone = job_ran.clone();
        let job_fut = Box::pin(async move {
            *job_ran_clone.lock().unwrap() = true;
            Ok(())
        });

        let result = match (&tenant_scope, tenant_id) {
            (Some(scope), Some(id)) => scope.with_scope(id, job_fut).await,
            _ => job_fut.await,
        };

        assert!(result.is_ok());
        assert!(
            calls.lock().unwrap().is_empty(),
            "with_scope must not be called when tenant_id is None"
        );
        assert!(
            *job_ran.lock().unwrap(),
            "job future must still run directly"
        );
    }

    /// scope_dispatch_no_provider: Some(id) + no provider -> job runs directly.
    #[tokio::test]
    async fn test_scope_dispatch_no_provider_runs_job_directly() {
        let tenant_id: Option<i64> = Some(1);
        let tenant_scope: Option<Arc<dyn TenantScopeProvider>> = None;

        let job_ran = Arc::new(Mutex::new(false));
        let job_ran_clone = job_ran.clone();
        let job_fut = Box::pin(async move {
            *job_ran_clone.lock().unwrap() = true;
            Ok(())
        });

        let result = match (&tenant_scope, tenant_id) {
            (Some(scope), Some(id)) => scope.with_scope(id, job_fut).await,
            _ => job_fut.await,
        };

        assert!(result.is_ok());
        assert!(
            *job_ran.lock().unwrap(),
            "job must run directly without a provider"
        );
    }
}
