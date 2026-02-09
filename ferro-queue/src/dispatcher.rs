//! Job dispatching utilities.

use crate::{Error, Job, JobPayload, Queue, QueueConfig};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

/// A pending job dispatch.
///
/// This builder allows configuring how a job is dispatched before
/// actually sending it to the queue.
pub struct PendingDispatch<J> {
    job: J,
    queue: Option<&'static str>,
    delay: Option<Duration>,
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
    async fn dispatch_immediately(self) -> Result<(), Error> {
        let job_name = self.job.name();

        if self.delay.is_some() {
            tracing::debug!(
                job = %job_name,
                "Job delay ignored in sync mode"
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

    /// Push the job to the Redis queue.
    async fn dispatch_to_queue(self) -> Result<(), Error> {
        let conn = Queue::connection();
        let queue = self.queue.unwrap_or(&conn.config().default_queue);

        let payload = match self.delay {
            Some(delay) => JobPayload::with_delay(&self.job, queue, delay)?,
            None => JobPayload::new(&self.job, queue)?,
        };

        conn.push(payload).await
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
}
