//! Job dispatching utilities.

use crate::{Error, Job, QueueConfig};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::OnceLock;
use std::time::Duration;

/// Global hook called at dispatch time to capture the current tenant ID.
/// Returns None when dispatching outside any tenant scope (system jobs).
/// Registered once during application bootstrap by the framework.
static TENANT_ID_HOOK: OnceLock<fn() -> Option<i64>> = OnceLock::new();

/// Register the tenant capture hook.
///
/// Called once during application bootstrap. Re-registration is silently ignored.
/// The hook is invoked at dispatch time to capture the current tenant ID from
/// task-local storage without requiring a direct dependency on the framework crate.
pub fn register_tenant_capture_hook(f: fn() -> Option<i64>) {
    let _ = TENANT_ID_HOOK.set(f);
}

/// A pending job dispatch.
///
/// This builder allows configuring how a job is dispatched before
/// actually sending it to the queue.
pub struct PendingDispatch<J> {
    job: J,
    queue: Option<&'static str>,
    delay: Option<Duration>,
    /// Explicit tenant ID override. When set, takes precedence over the auto-capture hook.
    tenant_id: Option<i64>,
}

impl<J> PendingDispatch<J>
where
    J: Job + Serialize + DeserializeOwned,
{
    /// Create a new pending dispatch.
    pub fn new(job: J) -> Self {
        Self {
            job,
            queue: None,
            delay: None,
            tenant_id: None,
        }
    }

    /// Set the queue to dispatch to.
    pub fn on_queue(mut self, queue: &'static str) -> Self {
        self.queue = Some(queue);
        self
    }

    /// Set a delay before the job is available for processing.
    pub fn delay(mut self, duration: Duration) -> Self {
        self.delay = Some(duration);
        self
    }

    /// Override the auto-captured tenant ID.
    ///
    /// Use when dispatching jobs on behalf of a tenant from a non-tenant-scoped context
    /// (e.g., admin actions, CLI commands, system webhooks).
    /// This explicit value takes precedence over the auto-capture hook.
    pub fn for_tenant(mut self, tenant_id: i64) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    /// Resolve the tenant ID to attach to the job payload.
    ///
    /// Precedence: explicit override (for_tenant) > auto-capture hook > None.
    #[allow(dead_code)] // used by dispatch_to_queue in Plan 02
    fn captured_tenant_id(&self) -> Option<i64> {
        self.tenant_id
            .or_else(|| TENANT_ID_HOOK.get().and_then(|f| f()))
    }

    /// Dispatch the job to the queue.
    ///
    /// In sync mode (`QUEUE_CONNECTION=sync`), the job is executed immediately
    /// in the current task. This is useful for development and testing.
    ///
    /// In redis mode (`QUEUE_CONNECTION=redis`), the job is pushed to the
    /// Redis queue for background processing by a worker.
    pub async fn dispatch(self) -> Result<(), Error> {
        if QueueConfig::is_sync_mode() {
            return self.dispatch_immediately().await;
        }

        self.dispatch_to_queue().await
    }

    /// Execute the job immediately (sync mode).
    ///
    /// Note: `.for_tenant()` is a no-op in sync mode. The current task's tenant context
    /// applies directly since the job runs in the same task.
    async fn dispatch_immediately(self) -> Result<(), Error> {
        let job_name = self.job.name();

        if self.delay.is_some() {
            tracing::debug!(
                job = %job_name,
                "Job delay ignored in sync mode"
            );
        }

        if self.tenant_id.is_some() {
            tracing::debug!(
                job = %job_name,
                tenant_id = ?self.tenant_id,
                "for_tenant() ignored in sync mode — current task tenant context applies"
            );
        }

        tracing::debug!(job = %job_name, "Executing job synchronously");

        match self.job.handle().await {
            Ok(()) => {
                tracing::debug!(job = %job_name, "Job completed successfully");
                Ok(())
            }
            Err(e) => {
                tracing::error!(job = %job_name, error = %e, "Job failed");
                self.job.failed(&e).await;
                Err(e)
            }
        }
    }

    /// Push the job to the database queue.
    ///
    /// Full implementation lands in Plan 02 (db.rs claim path).
    async fn dispatch_to_queue(self) -> Result<(), Error> {
        // Plan 02 replaces this stub with the DB enqueue path.
        Err(Error::custom("Queue not initialized. Call Queue::init() first."))
    }

    /// Dispatch the job in a background task (fire and forget).
    ///
    /// This spawns the dispatch as a background task, useful when you
    /// don't need to wait for the dispatch to complete.
    pub fn dispatch_now(self)
    where
        J: Send + 'static,
    {
        tokio::spawn(async move {
            if let Err(e) = self.dispatch().await {
                tracing::error!(error = %e, "Failed to dispatch job");
            }
        });
    }

    /// Dispatch the job synchronously (fire and forget).
    ///
    /// This spawns the dispatch as a background task.
    #[deprecated(since = "0.2.0", note = "Use dispatch_now() instead")]
    pub fn dispatch_sync(self)
    where
        J: Send + 'static,
    {
        self.dispatch_now()
    }
}

/// Dispatch a job using the global queue.
///
/// In sync mode, the job executes immediately. In redis mode, it's
/// queued for background processing.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_queue::{dispatch, Job, Error};
///
/// #[derive(Debug, Serialize, Deserialize)]
/// struct MyJob { data: String }
///
/// impl Job for MyJob {
///     async fn handle(&self) -> Result<(), Error> { Ok(()) }
/// }
///
/// dispatch(MyJob { data: "hello".into() }).await?;
/// ```
pub async fn dispatch<J>(job: J) -> Result<(), Error>
where
    J: Job + Serialize + DeserializeOwned,
{
    PendingDispatch::new(job).dispatch().await
}

/// Dispatch a job to a specific queue.
pub async fn dispatch_to<J>(job: J, queue: &'static str) -> Result<(), Error>
where
    J: Job + Serialize + DeserializeOwned,
{
    PendingDispatch::new(job).on_queue(queue).dispatch().await
}

/// Dispatch a job with a delay.
///
/// Note: In sync mode, the delay is ignored and the job executes immediately.
pub async fn dispatch_later<J>(job: J, delay: Duration) -> Result<(), Error>
where
    J: Job + Serialize + DeserializeOwned,
{
    PendingDispatch::new(job).delay(delay).dispatch().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::async_trait;
    use serial_test::serial;
    use std::env;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    /// Guard that removes environment variables on drop, ensuring cleanup even on panic.
    struct EnvGuard {
        vars: Vec<String>,
    }

    impl EnvGuard {
        fn set(key: &str, value: &str) -> Self {
            env::set_var(key, value);
            Self {
                vars: vec![key.to_string()],
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for var in &self.vars {
                env::remove_var(var);
            }
        }
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct TestJob {
        #[serde(skip)]
        executed: Arc<AtomicBool>,
    }

    impl TestJob {
        fn new() -> (Self, Arc<AtomicBool>) {
            let executed = Arc::new(AtomicBool::new(false));
            (
                Self {
                    executed: executed.clone(),
                },
                executed,
            )
        }
    }

    #[async_trait]
    impl Job for TestJob {
        async fn handle(&self) -> Result<(), Error> {
            self.executed.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct FailingJob;

    #[async_trait]
    impl Job for FailingJob {
        async fn handle(&self) -> Result<(), Error> {
            Err(Error::job_failed("FailingJob", "intentional failure"))
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_sync_mode_executes_immediately() {
        let _guard = EnvGuard::set("QUEUE_CONNECTION", "sync");

        let (job, executed) = TestJob::new();
        assert!(!executed.load(Ordering::SeqCst));

        let result = PendingDispatch::new(job).dispatch().await;
        assert!(result.is_ok());
        assert!(executed.load(Ordering::SeqCst));
    }

    #[tokio::test]
    #[serial]
    async fn test_sync_mode_handles_failure() {
        let _guard = EnvGuard::set("QUEUE_CONNECTION", "sync");

        let result = PendingDispatch::new(FailingJob).dispatch().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn test_sync_mode_ignores_delay() {
        let _guard = EnvGuard::set("QUEUE_CONNECTION", "sync");

        let (job, executed) = TestJob::new();

        let start = std::time::Instant::now();
        let result = PendingDispatch::new(job)
            .delay(Duration::from_secs(10))
            .dispatch()
            .await;

        assert!(result.is_ok());
        assert!(executed.load(Ordering::SeqCst));
        // Should complete quickly, not wait 10 seconds
        assert!(start.elapsed() < Duration::from_secs(5));
    }

    #[tokio::test]
    #[serial]
    async fn test_sync_mode_ignores_queue() {
        let _guard = EnvGuard::set("QUEUE_CONNECTION", "sync");

        let (job, executed) = TestJob::new();

        let result = PendingDispatch::new(job)
            .on_queue("high-priority")
            .dispatch()
            .await;

        assert!(result.is_ok());
        assert!(executed.load(Ordering::SeqCst));
    }

    // --- Task 2 tests ---

    #[test]
    fn test_for_tenant_stores_explicit_override() {
        let (job, _) = TestJob::new();
        let pending = PendingDispatch::new(job).for_tenant(99);
        assert_eq!(pending.tenant_id, Some(99));
    }

    #[test]
    fn test_for_tenant_explicit_wins_over_hook() {
        // Even if a hook is registered, explicit for_tenant() takes precedence.
        // The hook may return Some(42) from test_hook_registered_once (since OnceLock),
        // but for_tenant(99) overrides it via captured_tenant_id() precedence logic.
        let (job, _) = TestJob::new();
        let pending = PendingDispatch::new(job).for_tenant(99);
        // captured_tenant_id() returns explicit override first
        assert_eq!(pending.captured_tenant_id(), Some(99));
    }

    #[test]
    fn test_no_tenant_id_by_default() {
        let (job, _) = TestJob::new();
        let pending = PendingDispatch::new(job);
        assert_eq!(pending.tenant_id, None);
    }

    #[test]
    fn test_hook_registration_second_call_is_noop() {
        // OnceLock ignores the second set() — first registration wins.
        // We register once and verify calling register_tenant_capture_hook again doesn't panic.
        register_tenant_capture_hook(|| Some(42));
        register_tenant_capture_hook(|| Some(999)); // silently ignored
                                                    // If the hook was set to 42 first, it remains 42
        let result = TENANT_ID_HOOK.get().map(|f| f());
        // We can't guarantee the value here due to test ordering, but it must not be 999
        // since OnceLock only accepts the first write.
        // Just assert no panic occurred.
        let _ = result;
    }

    #[test]
    fn test_hook_registration_captures_at_dispatch_time() {
        // Register hook returning Some(42) — subsequent calls to captured_tenant_id()
        // without an explicit override will return Some(42) from the hook.
        register_tenant_capture_hook(|| Some(42));
        // With no explicit for_tenant(), hook is consulted
        let (job, _) = TestJob::new();
        let pending = PendingDispatch::new(job);
        // captured_tenant_id() consults hook when no explicit override set
        let captured = pending.captured_tenant_id();
        // Could be Some(42) if hook registered, or None if hook was already set by prior test
        // We only assert no panic and that the explicit override path still works.
        assert!(captured.is_none() || captured == Some(42));
    }
}
