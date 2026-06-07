//! Job trait and payload structures.

use crate::Error;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A job that can be executed by a queue worker.
///
/// Jobs contain the logic that should run in the background.
/// They must be serializable so they can be stored in the queue.
///
/// # Example
///
/// ```rust
/// use ferro_queue::{Job, Error, async_trait};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// struct ProcessImage {
///     image_id: i64,
///     operations: Vec<String>,
/// }
///
/// #[async_trait]
/// impl Job for ProcessImage {
///     async fn handle(&self) -> Result<(), Error> {
///         println!("Processing image {} with {:?}", self.image_id, self.operations);
///         Ok(())
///     }
///
///     fn max_retries(&self) -> u32 {
///         5
///     }
///
///     fn retry_delay(&self, attempt: u32) -> std::time::Duration {
///         // Exponential backoff
///         std::time::Duration::from_secs(2u64.pow(attempt))
///     }
/// }
/// ```
#[async_trait]
pub trait Job: Send + Sync + 'static {
    /// Execute the job logic.
    async fn handle(&self) -> Result<(), Error>;

    /// The name of the job for logging and identification.
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Maximum number of times to retry the job on failure.
    fn max_retries(&self) -> u32 {
        3
    }

    /// Delay before retrying after a failure.
    fn retry_delay(&self, _attempt: u32) -> std::time::Duration {
        std::time::Duration::from_secs(5)
    }

    /// Called when the job fails after all retries are exhausted.
    async fn failed(&self, error: &Error) {
        tracing::error!(job = self.name(), error = %error, "Job failed permanently");
    }

    /// Timeout for job execution.
    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(60)
    }
}

/// Serialized job payload stored in the queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPayload {
    /// Unique job ID.
    pub id: Uuid,
    /// Job type name for deserialization.
    pub job_type: String,
    /// Serialized job data.
    pub data: String,
    /// Queue name.
    pub queue: String,
    /// Number of attempts made.
    pub attempts: u32,
    /// Maximum retry attempts.
    pub max_retries: u32,
    /// When the job was created.
    pub created_at: DateTime<Utc>,
    /// When the job should be available for processing.
    pub available_at: DateTime<Utc>,
    /// When the job was reserved by a worker (if any).
    pub reserved_at: Option<DateTime<Utc>>,
    /// Tenant ID for tenant-scoped job execution.
    /// None means the job runs in system scope (no tenant context).
    /// Old payloads without this field deserialize to None.
    #[serde(default)]
    pub tenant_id: Option<i64>,
}

impl JobPayload {
    /// Create a new job payload.
    pub fn new<J: Job + Serialize>(job: &J, queue: &str) -> Result<Self, Error> {
        let data =
            serde_json::to_string(job).map_err(|e| Error::SerializationFailed(e.to_string()))?;

        Ok(Self {
            id: Uuid::new_v4(),
            job_type: job.name().to_string(),
            data,
            queue: queue.to_string(),
            attempts: 0,
            max_retries: job.max_retries(),
            created_at: Utc::now(),
            available_at: Utc::now(),
            reserved_at: None,
            tenant_id: None,
        })
    }

    /// Create a job payload with a delay.
    pub fn with_delay<J: Job + Serialize>(
        job: &J,
        queue: &str,
        delay: std::time::Duration,
    ) -> Result<Self, Error> {
        let mut payload = Self::new(job, queue)?;
        payload.available_at = Utc::now() + chrono::Duration::from_std(delay).unwrap_or_default();
        Ok(payload)
    }

    /// Set the tenant ID for this payload.
    pub fn with_tenant_id(mut self, id: Option<i64>) -> Self {
        self.tenant_id = id;
        self
    }

    /// Check if the job is available for processing.
    pub fn is_available(&self) -> bool {
        Utc::now() >= self.available_at
    }

    /// Check if the job has exceeded max retries.
    pub fn has_exceeded_retries(&self) -> bool {
        self.attempts >= self.max_retries
    }

    /// Increment the attempt counter.
    pub fn increment_attempts(&mut self) {
        self.attempts += 1;
    }

    /// Mark the job as reserved.
    pub fn reserve(&mut self) {
        self.reserved_at = Some(Utc::now());
    }

    /// Serialize the payload to JSON.
    pub fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string(self).map_err(|e| Error::SerializationFailed(e.to_string()))
    }

    /// Deserialize a payload from JSON.
    pub fn from_json(json: &str) -> Result<Self, Error> {
        serde_json::from_str(json).map_err(|e| Error::DeserializationFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestJob {
        value: i32,
    }

    #[async_trait]
    impl Job for TestJob {
        async fn handle(&self) -> Result<(), Error> {
            Ok(())
        }
    }

    #[test]
    fn test_job_payload_creation() {
        let job = TestJob { value: 42 };
        let payload = JobPayload::new(&job, "default").unwrap();

        assert_eq!(payload.queue, "default");
        assert_eq!(payload.attempts, 0);
        assert!(payload.is_available());
    }

    #[test]
    fn test_job_payload_with_delay() {
        let job = TestJob { value: 42 };
        let payload =
            JobPayload::with_delay(&job, "default", std::time::Duration::from_secs(60)).unwrap();

        assert!(!payload.is_available());
    }

    #[test]
    fn test_job_payload_serialization() {
        let job = TestJob { value: 42 };
        let payload = JobPayload::new(&job, "default").unwrap();

        let json = payload.to_json().unwrap();
        let restored = JobPayload::from_json(&json).unwrap();

        assert_eq!(payload.id, restored.id);
        assert_eq!(payload.queue, restored.queue);
    }

    #[test]
    fn test_tenant_id_none_by_default() {
        let job = TestJob { value: 42 };
        let payload = JobPayload::new(&job, "default").unwrap();
        assert_eq!(payload.tenant_id, None);
    }

    #[test]
    fn test_tenant_id_none_serializes_as_null() {
        let job = TestJob { value: 42 };
        let payload = JobPayload::new(&job, "default").unwrap();
        let json = payload.to_json().unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(val["tenant_id"], serde_json::Value::Null);
    }

    #[test]
    fn test_tenant_id_some_round_trips() {
        let job = TestJob { value: 42 };
        let payload = JobPayload::new(&job, "default")
            .unwrap()
            .with_tenant_id(Some(42));
        let json = payload.to_json().unwrap();
        let restored = JobPayload::from_json(&json).unwrap();
        assert_eq!(restored.tenant_id, Some(42));
    }

    #[test]
    fn test_old_payload_without_tenant_id_deserializes_to_none() {
        // Old JSON payload without "tenant_id" key — must deserialize to None via serde(default)
        let old_json = r#"{"id":"550e8400-e29b-41d4-a716-446655440000","job_type":"test","data":"{}","queue":"default","attempts":0,"max_retries":3,"created_at":"2024-01-01T00:00:00Z","available_at":"2024-01-01T00:00:00Z","reserved_at":null}"#;
        let payload = JobPayload::from_json(old_json).unwrap();
        assert_eq!(payload.tenant_id, None);
    }

    #[test]
    fn test_with_tenant_id_builder() {
        let job = TestJob { value: 42 };
        let payload = JobPayload::new(&job, "default")
            .unwrap()
            .with_tenant_id(Some(99));
        assert_eq!(payload.tenant_id, Some(99));

        let payload_none = JobPayload::new(&job, "default")
            .unwrap()
            .with_tenant_id(None);
        assert_eq!(payload_none.tenant_id, None);
    }

    #[test]
    fn backoff_delay_range() {
        let job = TestJob { value: 0 };
        for _ in 0..100 {
            assert!(job.retry_delay(0).as_secs() <= 5);
            assert!(job.retry_delay(3).as_secs() <= 40);
            assert!(job.retry_delay(30).as_secs() <= 900);
        }
    }

    #[test]
    fn idempotency_key_defaults_to_none() {
        let job = TestJob { value: 0 };
        assert_eq!(job.idempotency_key(), None);
    }
}
