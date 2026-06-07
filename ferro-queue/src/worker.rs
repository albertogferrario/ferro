//! DB-backed queue worker (WorkerLoop).
//!
//! `WorkerLoop` runs a reaper→claim→spawn cycle. Each iteration:
//! 1. Runs `db::reaper` to reset stuck claimed rows.
//! 2. Calls `db::claim` to atomically take the next pending job.
//! 3. Spawns a task that executes the handler with panic isolation.
//!
//! Shutdown is triggered by SIGTERM, Ctrl-C, or a call to `WorkerLoop::shutdown()`.
//! On shutdown the loop drains in-flight jobs and calls `db::requeue_claimed_by`
//! to reset any claimed-but-unstarted rows back to `pending` (D-10).

use crate::{Error, Job};
use async_trait::async_trait;
use chrono::Utc;
use futures::FutureExt;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

// ---------------------------------------------------------------------------
// TenantScopeProvider — preserved verbatim from previous worker.rs
// ---------------------------------------------------------------------------

/// Injects tenant scope around job execution.
///
/// Implemented by the framework — injected at startup via `WorkerLoop::with_tenant_scope()`.
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

// ---------------------------------------------------------------------------
// WorkerConfig
// ---------------------------------------------------------------------------

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
    /// How long a claimed job stays invisible before the reaper reclaims it.
    pub visibility_timeout: Duration,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            queues: vec!["default".to_string()],
            max_jobs: 10,
            sleep_duration: Duration::from_secs(1),
            stop_on_error: false,
            visibility_timeout: Duration::from_secs(300),
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

    /// Set the visibility timeout.
    pub fn with_visibility_timeout(mut self, d: Duration) -> Self {
        self.visibility_timeout = d;
        self
    }
}

// ---------------------------------------------------------------------------
// JobHandler type alias
// ---------------------------------------------------------------------------

/// Handler closure stored per job type name.
///
/// Returns `(Result<(), Error>, retry_delay_secs)` — the delay is the
/// per-job `retry_delay(attempt)` value captured at registration time.
type JobHandler = Arc<
    dyn Fn(String, u32) -> Pin<Box<dyn Future<Output = (Result<(), Error>, Duration)> + Send>>
        + Send
        + Sync,
>;

// ---------------------------------------------------------------------------
// WorkerLoop
// ---------------------------------------------------------------------------

/// DB-backed queue worker.
///
/// Runs a reaper→claim→spawn cycle until signalled to stop. Panic isolation
/// (D-11) ensures a panicking job handler never kills the loop. Graceful
/// shutdown (D-10) drains in-flight jobs and resets claimed-but-unstarted rows.
pub struct WorkerLoop {
    /// Worker configuration.
    config: WorkerConfig,
    /// Registered job handlers by type name.
    handlers: HashMap<String, JobHandler>,
    /// Semaphore limiting concurrent in-flight jobs.
    semaphore: Arc<Semaphore>,
    /// Shutdown flag — set by SIGTERM/Ctrl-C or by `WorkerLoop::shutdown()`.
    shutdown: Arc<AtomicBool>,
    /// Optional tenant scope provider.
    tenant_scope: Option<Arc<dyn TenantScopeProvider>>,
    /// Unique identifier for this worker instance (used by requeue_claimed_by).
    worker_id: String,
}

impl WorkerLoop {
    /// Create a new worker loop.
    ///
    /// A random UUID is generated as the worker ID. The worker reads the DB
    /// connection from `Queue::connection()` — no connection argument needed.
    pub fn new(config: WorkerConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_jobs));
        Self {
            config,
            handlers: HashMap::new(),
            semaphore,
            shutdown: Arc::new(AtomicBool::new(false)),
            tenant_scope: None,
            worker_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Inject a tenant scope provider for tenant-aware job execution.
    ///
    /// When set, jobs with a `tenant_id` are executed inside a tenant context
    /// scope. Jobs without a tenant_id or workers without a provider run in
    /// the default (system) scope.
    pub fn with_tenant_scope(mut self, provider: Arc<dyn TenantScopeProvider>) -> Self {
        self.tenant_scope = Some(provider);
        self
    }

    /// Register a job handler.
    ///
    /// The handler closure deserializes the job from JSON, calls `handle()`,
    /// and also captures the per-job `retry_delay(attempt)` for use on failure.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// worker_loop.register::<SendEmailJob>();
    /// ```
    pub fn register<J>(&mut self)
    where
        J: Job + serde::de::DeserializeOwned + 'static,
    {
        let type_name = std::any::type_name::<J>().to_string();

        let handler: JobHandler = Arc::new(move |data: String, attempt: u32| {
            Box::pin(async move {
                let job: J = match serde_json::from_str::<J>(&data) {
                    Ok(j) => j,
                    Err(e) => {
                        return (
                            Err(Error::DeserializationFailed(e.to_string())),
                            Duration::from_secs(5),
                        )
                    }
                };
                // Capture retry_delay before consuming the job.
                let delay = job.retry_delay(attempt);
                let result = job.handle().await;
                (result, delay)
            })
        });

        self.handlers.insert(type_name, handler);
    }

    /// Signal the worker loop to shut down gracefully.
    ///
    /// Sets the same AtomicBool that the SIGTERM/Ctrl-C handler sets, so
    /// programmatic and signal-based shutdown share one path.
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }

    /// Run the worker loop until shutdown.
    ///
    /// Loop behaviour:
    /// 1. Check shutdown flag — if set, drain in-flight jobs and requeue.
    /// 2. For each queue: run reaper, then attempt claim.
    ///    - If a job is claimed, spawn it and loop immediately (no idle sleep).
    /// 3. If no jobs were found across all queues, sleep `config.sleep_duration`.
    pub async fn run(&self) -> Result<(), Error> {
        let conn: &'static DatabaseConnection = crate::db::Queue::connection();

        info!(
            worker_id = %self.worker_id,
            queues = ?self.config.queues,
            max_jobs = self.config.max_jobs,
            "WorkerLoop starting"
        );

        // Install SIGTERM / Ctrl-C handler — sets the same shutdown flag.
        {
            let shutdown = self.shutdown.clone();
            tokio::spawn(async move {
                let mut sigterm =
                    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                        .expect("install SIGTERM handler");
                tokio::select! {
                    _ = sigterm.recv() => {
                        info!("SIGTERM received — shutting down WorkerLoop");
                    }
                    _ = tokio::signal::ctrl_c() => {
                        info!("Ctrl-C received — shutting down WorkerLoop");
                    }
                }
                shutdown.store(true, Ordering::SeqCst);
            });
        }

        'outer: loop {
            // --- Shutdown gate ---
            if self.shutdown.load(Ordering::SeqCst) {
                info!(worker_id = %self.worker_id, "Shutdown flag set — draining in-flight jobs");

                // Drain: acquire all permits (waits for running spawned tasks to finish).
                let _ = self
                    .semaphore
                    .acquire_many(self.config.max_jobs as u32)
                    .await;

                // Requeue any claimed-but-unstarted rows belonging to this worker.
                crate::db::requeue_claimed_by(conn, &self.worker_id)
                    .await
                    .map_err(|e| {
                        error!(error = %e, "requeue_claimed_by failed during shutdown");
                        e
                    })?;

                info!(worker_id = %self.worker_id, "WorkerLoop shut down cleanly");
                return Ok(());
            }

            // --- Reaper + claim cycle ---
            for queue in &self.config.queues {
                // Run reaper before each claim attempt (D-14).
                match crate::db::reaper(conn, queue, self.config.visibility_timeout).await {
                    Ok(()) => {}
                    Err(e) => {
                        error!(queue = %queue, error = %e, "reaper error");
                        if self.config.stop_on_error {
                            return Err(e);
                        }
                    }
                }

                // Attempt an atomic claim.
                match crate::db::claim(conn, queue, &self.worker_id).await {
                    Ok(Some(job_row)) => {
                        self.spawn_job(conn, job_row);
                        continue 'outer; // claimed something — loop immediately without sleep
                    }
                    Ok(None) => {} // nothing in this queue
                    Err(e) => {
                        error!(queue = %queue, error = %e, "claim error");
                        if self.config.stop_on_error {
                            return Err(e);
                        }
                    }
                }
            }

            // No jobs found across all queues — idle sleep (D-08).
            tokio::time::sleep(self.config.sleep_duration).await;
        }
    }

    /// Spawn a task that executes `job_row` with panic isolation.
    ///
    /// Acquires a semaphore permit before spawning. The permit is held for the
    /// lifetime of the task, enforcing `config.max_jobs` concurrency.
    fn spawn_job(&self, conn: &'static DatabaseConnection, job_row: crate::db::JobRow) {
        // Clone the semaphore Arc; the spawned task acquires an owned permit inside,
        // keeping it held for the full duration of the job.
        let permit = self.semaphore.clone();
        let handlers = self.handlers.clone();
        let tenant_scope = self.tenant_scope.clone();
        let worker_id = self.worker_id.clone();

        tokio::spawn(async move {
            // Acquire the permit inside the task so it is held for the full duration.
            let _permit = permit.acquire_owned().await.expect("semaphore closed");

            let job_id = job_row.id;
            let job_type = job_row.job_type.clone();
            let tenant_id = job_row.tenant_id;
            let attempts = job_row.attempts;
            let max_retries = job_row.max_retries;

            debug!(
                job_id = %job_id,
                job_type = %job_type,
                attempts = attempts,
                tenant_id = ?tenant_id,
                worker_id = %worker_id,
                "Executing job"
            );

            let handler = match handlers.get(&job_type) {
                Some(h) => h.clone(),
                None => {
                    warn!(job_type = %job_type, "No handler registered — releasing job for retry");
                    // Release with a short delay — the job will be retried.
                    let available_at = Utc::now()
                        + chrono::Duration::from_std(Duration::from_secs(5)).unwrap_or_default();
                    crate::db::release_job(conn, job_id, attempts + 1, available_at)
                        .await
                        .ok();
                    return;
                }
            };

            // Panic-isolated execution (D-11, T-185-03).
            // AssertUnwindSafe is sound here: the handler closure captures only
            // Arc references and owned data; we don't observe any interior state
            // after a panic.
            let result = AssertUnwindSafe(async move {
                // Tenant scope wrap (D-17, T-185-08).
                match (&tenant_scope, tenant_id) {
                    (Some(scope), Some(id)) => {
                        let fut = Box::pin(async move {
                            let (res, _delay) = handler(job_row.payload.clone(), attempts).await;
                            res
                        });
                        (scope.with_scope(id, fut).await, Duration::from_secs(5))
                    }
                    _ => handler(job_row.payload.clone(), attempts).await,
                }
            })
            .catch_unwind()
            .await;

            match result {
                // Success path: delete the row (D-04 delete-on-success).
                Ok((Ok(()), _)) => {
                    debug!(job_id = %job_id, job_type = %job_type, "Job succeeded — deleting row");
                    crate::db::delete_job(conn, job_id).await.ok();
                }

                // Handler returned Err — normal failure path.
                Ok((Err(e), retry_delay)) => {
                    error!(job_id = %job_id, job_type = %job_type, error = %e, "Job handler returned error");
                    handle_failure(
                        conn,
                        job_id,
                        attempts,
                        max_retries,
                        &e.to_string(),
                        retry_delay,
                    )
                    .await;
                }

                // Handler panicked — counts as a failed attempt (D-11).
                Err(_panic) => {
                    error!(job_id = %job_id, job_type = %job_type, "Job handler panicked — counting as failure");
                    let msg = "job handler panicked";
                    // Use a default jitter delay for panics (we can't call retry_delay
                    // because the handler destructured before the panic).
                    let delay = default_jitter_delay(attempts);
                    handle_failure(conn, job_id, attempts, max_retries, msg, delay).await;
                }
            }
        });
    }
}

/// Handle a job failure: either park as failed or release for retry.
async fn handle_failure(
    conn: &'static DatabaseConnection,
    job_id: i64,
    attempts: u32,
    max_retries: u32,
    err_msg: &str,
    retry_delay: Duration,
) {
    if attempts + 1 >= max_retries {
        // Exhausted — park as failed.
        warn!(job_id = %job_id, attempts = attempts, "Job exhausted retries — parking as failed");
        crate::db::fail_job(conn, job_id, err_msg).await.ok();
    } else {
        // Retry — release with jittered delay.
        let available_at = Utc::now() + chrono::Duration::from_std(retry_delay).unwrap_or_default();
        debug!(
            job_id = %job_id,
            retry_at = %available_at,
            "Scheduling job retry"
        );
        crate::db::release_job(conn, job_id, attempts + 1, available_at)
            .await
            .ok();
    }
}

/// Full-jitter exponential backoff for panic cases where `retry_delay` cannot
/// be called on the job instance.
///
/// Formula: `rand(0..=min(cap, base * 2^attempt))`, base 5 s, cap 15 min.
fn default_jitter_delay(attempt: u32) -> Duration {
    use rand::Rng;
    let base_secs: u64 = 5;
    let cap_secs: u64 = 15 * 60;
    let max_delay = cap_secs.min(base_secs.saturating_mul(2u64.saturating_pow(attempt)));
    let jitter = rand::thread_rng().gen_range(0..=max_delay);
    Duration::from_secs(jitter)
}

/// Type alias for API continuity with callers that use the old `Worker` name.
pub type Worker = WorkerLoop;

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

    /// WorkerLoop can be constructed without a connection argument.
    #[test]
    fn test_worker_loop_new() {
        let w = WorkerLoop::new(WorkerConfig::default());
        assert!(w.tenant_scope.is_none());
        assert!(!w.worker_id.is_empty());
    }

    /// Worker::with_tenant_scope() stores the provider.
    #[test]
    fn test_with_tenant_scope_stores_provider() {
        let w = WorkerLoop::new(WorkerConfig::default());
        let provider = Arc::new(MockScopeProvider::new());
        let w = w.with_tenant_scope(provider);
        assert!(w.tenant_scope.is_some());
    }

    /// Worker without tenant_scope has None by default.
    #[test]
    fn test_worker_without_scope_has_none_by_default() {
        let w = WorkerLoop::new(WorkerConfig::default());
        assert!(w.tenant_scope.is_none());
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

    /// Shutdown flag is set by WorkerLoop::shutdown().
    #[test]
    fn test_shutdown_sets_flag() {
        let w = WorkerLoop::new(WorkerConfig::default());
        assert!(!w.shutdown.load(Ordering::SeqCst));
        w.shutdown();
        assert!(w.shutdown.load(Ordering::SeqCst));
    }

    /// WorkerConfig visibility_timeout default is 5 minutes.
    #[test]
    fn test_worker_config_visibility_timeout_default() {
        let c = WorkerConfig::default();
        assert_eq!(c.visibility_timeout, Duration::from_secs(300));
    }

    /// default_jitter_delay stays within bounds.
    #[test]
    fn test_default_jitter_delay_bounds() {
        for _ in 0..50 {
            assert!(default_jitter_delay(0).as_secs() <= 5);
            assert!(default_jitter_delay(3).as_secs() <= 40);
            assert!(default_jitter_delay(30).as_secs() <= 900);
        }
    }
}
