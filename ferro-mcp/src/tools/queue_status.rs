//! Queue status tool - show pending, delayed, and failed jobs
//!
//! This tool fetches queue information from the running application via
//! the `/_ferro/queue/jobs` and `/_ferro/queue/stats` debug endpoints.

use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Timeout for HTTP requests to the running application
const HTTP_TIMEOUT: Duration = Duration::from_secs(2);

/// Queue status information
#[derive(Debug, Serialize)]
pub struct QueueStatusInfo {
    /// Jobs information (pending, delayed, failed)
    pub jobs: Option<QueueJobsSnapshot>,
    /// Queue statistics
    pub stats: Option<QueueStatsSnapshot>,
    /// Data source
    pub source: QueueSource,
}

/// Source of queue data
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QueueSource {
    /// Data fetched from running application via HTTP endpoint
    Runtime,
    /// App not running or endpoint unavailable
    Unavailable,
    /// Queue not initialized (sync mode)
    NotInitialized,
}

/// Response format from the `/_ferro/queue/jobs` endpoint
#[derive(Debug, Deserialize)]
struct DebugJobsResponse {
    success: bool,
    data: QueueJobsSnapshot,
}

/// Response format from the `/_ferro/queue/stats` endpoint
#[derive(Debug, Deserialize)]
struct DebugStatsResponse {
    success: bool,
    data: QueueStatsSnapshot,
}

/// Jobs snapshot from the runtime
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueueJobsSnapshot {
    /// Pending jobs (ready to process)
    pub pending: Vec<JobInfo>,
    /// Delayed jobs (waiting for available_at)
    pub delayed: Vec<JobInfo>,
    /// Failed jobs
    pub failed: Vec<FailedJobInfo>,
}

/// Queue statistics snapshot
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueueStatsSnapshot {
    /// Stats per queue
    pub queues: Vec<SingleQueueStats>,
    /// Total failed jobs count
    pub total_failed: usize,
}

/// Statistics for a single queue
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SingleQueueStats {
    /// Queue name
    pub name: String,
    /// Number of pending jobs
    pub pending: usize,
    /// Number of delayed jobs
    pub delayed: usize,
}

/// Job information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobInfo {
    /// Unique job ID
    pub id: String,
    /// Job type name
    pub job_type: String,
    /// Queue name
    pub queue: String,
    /// Number of attempts made
    pub attempts: u32,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// When the job was created
    pub created_at: DateTime<Utc>,
    /// When the job should be available for processing
    pub available_at: DateTime<Utc>,
    /// Job state
    pub state: String,
}

/// Failed job information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FailedJobInfo {
    /// Job info
    pub job: JobInfo,
    /// Error message
    pub error: String,
    /// When the job failed
    pub failed_at: DateTime<Utc>,
}

/// Try to fetch jobs from the running application
async fn fetch_runtime_jobs(base_url: &str) -> Option<QueueJobsSnapshot> {
    let url = format!("{base_url}/_ferro/queue/jobs");

    let client = reqwest::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .build()
        .ok()?;
    let response = client.get(&url).send().await.ok()?;

    if !response.status().is_success() {
        return None;
    }

    let debug_response: DebugJobsResponse = response.json().await.ok()?;

    if !debug_response.success {
        return None;
    }

    Some(debug_response.data)
}

/// Try to fetch stats from the running application
async fn fetch_runtime_stats(base_url: &str) -> Option<QueueStatsSnapshot> {
    let url = format!("{base_url}/_ferro/queue/stats");

    let client = reqwest::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .build()
        .ok()?;
    let response = client.get(&url).send().await.ok()?;

    if !response.status().is_success() {
        return None;
    }

    let debug_response: DebugStatsResponse = response.json().await.ok()?;

    if !debug_response.success {
        return None;
    }

    Some(debug_response.data)
}

pub async fn execute() -> Result<QueueStatusInfo> {
    // Try runtime endpoints
    for base_url in ["http://localhost:8080", "http://127.0.0.1:8080"] {
        let jobs = fetch_runtime_jobs(base_url).await;
        let stats = fetch_runtime_stats(base_url).await;

        // If we got either, we connected to the app
        if jobs.is_some() || stats.is_some() {
            return Ok(QueueStatusInfo {
                jobs,
                stats,
                source: QueueSource::Runtime,
            });
        }
    }

    // No data available - app not running or queue not initialized
    Ok(QueueStatusInfo {
        jobs: None,
        stats: None,
        source: QueueSource::Unavailable,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_source_serialization() {
        assert_eq!(
            serde_json::to_string(&QueueSource::Runtime).unwrap(),
            "\"runtime\""
        );
        assert_eq!(
            serde_json::to_string(&QueueSource::Unavailable).unwrap(),
            "\"unavailable\""
        );
        assert_eq!(
            serde_json::to_string(&QueueSource::NotInitialized).unwrap(),
            "\"not_initialized\""
        );
    }

    #[test]
    fn test_queue_source_debug() {
        assert!(format!("{:?}", QueueSource::Runtime).contains("Runtime"));
        assert!(format!("{:?}", QueueSource::Unavailable).contains("Unavailable"));
        assert!(format!("{:?}", QueueSource::NotInitialized).contains("NotInitialized"));
    }

    #[test]
    fn test_single_queue_stats_serialization() {
        let stats = SingleQueueStats {
            name: "default".to_string(),
            pending: 10,
            delayed: 5,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let restored: SingleQueueStats = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.name, "default");
        assert_eq!(restored.pending, 10);
        assert_eq!(restored.delayed, 5);
    }

    #[test]
    fn test_queue_stats_snapshot_serialization() {
        let stats = QueueStatsSnapshot {
            queues: vec![SingleQueueStats {
                name: "emails".to_string(),
                pending: 25,
                delayed: 10,
            }],
            total_failed: 3,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let restored: QueueStatsSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.queues.len(), 1);
        assert_eq!(restored.queues[0].name, "emails");
        assert_eq!(restored.total_failed, 3);
    }

    #[test]
    fn test_job_info_serialization() {
        let now = Utc::now();
        let job = JobInfo {
            id: "job-123".to_string(),
            job_type: "SendNotification".to_string(),
            queue: "notifications".to_string(),
            attempts: 1,
            max_retries: 3,
            created_at: now,
            available_at: now,
            state: "pending".to_string(),
        };

        let json = serde_json::to_string(&job).unwrap();
        let restored: JobInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.id, "job-123");
        assert_eq!(restored.job_type, "SendNotification");
        assert_eq!(restored.attempts, 1);
    }

    #[test]
    fn test_failed_job_info_serialization() {
        let now = Utc::now();
        let failed = FailedJobInfo {
            job: JobInfo {
                id: "job-456".to_string(),
                job_type: "ProcessPayment".to_string(),
                queue: "payments".to_string(),
                attempts: 3,
                max_retries: 3,
                created_at: now,
                available_at: now,
                state: "failed".to_string(),
            },
            error: "Payment gateway timeout".to_string(),
            failed_at: now,
        };

        let json = serde_json::to_string(&failed).unwrap();
        let restored: FailedJobInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.job.id, "job-456");
        assert_eq!(restored.error, "Payment gateway timeout");
    }

    #[test]
    fn test_queue_jobs_snapshot_serialization() {
        let now = Utc::now();
        let snapshot = QueueJobsSnapshot {
            pending: vec![JobInfo {
                id: "job-1".to_string(),
                job_type: "Job1".to_string(),
                queue: "default".to_string(),
                attempts: 0,
                max_retries: 3,
                created_at: now,
                available_at: now,
                state: "pending".to_string(),
            }],
            delayed: vec![],
            failed: vec![],
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let restored: QueueJobsSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.pending.len(), 1);
        assert!(restored.delayed.is_empty());
        assert!(restored.failed.is_empty());
    }

    #[test]
    fn test_queue_status_info_debug() {
        let info = QueueStatusInfo {
            jobs: None,
            stats: None,
            source: QueueSource::Unavailable,
        };
        assert!(format!("{info:?}").contains("QueueStatusInfo"));
    }

    #[test]
    fn test_debug_jobs_response_deserialization() {
        let json = r#"{
            "success": true,
            "data": {
                "pending": [],
                "delayed": [],
                "failed": []
            }
        }"#;

        let response: DebugJobsResponse = serde_json::from_str(json).unwrap();
        assert!(response.success);
        assert!(response.data.pending.is_empty());
    }

    #[test]
    fn test_debug_stats_response_deserialization() {
        let json = r#"{
            "success": true,
            "data": {
                "queues": [{"name": "default", "pending": 5, "delayed": 2}],
                "total_failed": 1
            }
        }"#;

        let response: DebugStatsResponse = serde_json::from_str(json).unwrap();
        assert!(response.success);
        assert_eq!(response.data.queues.len(), 1);
        assert_eq!(response.data.total_failed, 1);
    }

    // Note: Testing the execute() function and fetch_* functions
    // requires integration tests with a running application server.
}
