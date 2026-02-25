//! Job history tool - view background job execution history

use crate::error::{McpError, Result};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct JobHistoryInfo {
    pub queue_driver: String,
    pub pending_jobs: Vec<JobInfo>,
    pub failed_jobs: Vec<FailedJobInfo>,
    pub stats: JobStats,
}

#[derive(Debug, Serialize)]
pub struct JobInfo {
    pub id: String,
    pub queue: String,
    pub job_type: String,
    pub payload_preview: String,
    pub attempts: i32,
    pub available_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct FailedJobInfo {
    pub id: String,
    pub queue: String,
    pub job_type: String,
    pub payload_preview: String,
    pub exception: String,
    pub failed_at: String,
}

#[derive(Debug, Serialize)]
pub struct JobStats {
    pub pending_count: usize,
    pub failed_count: usize,
    pub jobs_by_queue: std::collections::HashMap<String, usize>,
    pub jobs_by_type: std::collections::HashMap<String, usize>,
}

pub async fn execute(
    project_root: &Path,
    queue_filter: Option<&str>,
    limit: Option<usize>,
) -> Result<JobHistoryInfo> {
    dotenvy::from_path(project_root.join(".env")).ok();

    let queue_driver = std::env::var("QUEUE_CONNECTION").unwrap_or_else(|_| "database".to_string());
    let limit = limit.unwrap_or(50);

    match queue_driver.as_str() {
        "database" => get_database_job_history(project_root, queue_filter, limit).await,
        "redis" => get_redis_job_history(queue_filter, limit),
        "sync" => Ok(JobHistoryInfo {
            queue_driver: "sync".to_string(),
            pending_jobs: vec![],
            failed_jobs: vec![],
            stats: JobStats {
                pending_count: 0,
                failed_count: 0,
                jobs_by_queue: std::collections::HashMap::new(),
                jobs_by_type: std::collections::HashMap::new(),
            },
        }),
        _ => Err(McpError::ConfigError(format!(
            "Unknown queue driver: {queue_driver}"
        ))),
    }
}

async fn get_database_job_history(
    project_root: &Path,
    queue_filter: Option<&str>,
    limit: usize,
) -> Result<JobHistoryInfo> {
    let database_url = get_database_url(project_root)?;

    let db: DatabaseConnection = Database::connect(&database_url)
        .await
        .map_err(|e| McpError::DatabaseError(format!("Failed to connect: {e}")))?;

    // Get pending jobs
    let pending_query = if let Some(queue) = queue_filter {
        format!("SELECT * FROM jobs WHERE queue = '{queue}' ORDER BY created_at DESC LIMIT {limit}")
    } else {
        format!("SELECT * FROM jobs ORDER BY created_at DESC LIMIT {limit}")
    };

    let pending_jobs = match db
        .query_all(Statement::from_string(
            db.get_database_backend(),
            pending_query,
        ))
        .await
    {
        Ok(rows) => rows
            .iter()
            .filter_map(|row| {
                let id: i64 = row.try_get_by("id").ok()?;
                let queue: String = row
                    .try_get_by("queue")
                    .unwrap_or_else(|_| "default".to_string());
                let payload: String = row.try_get_by("payload").unwrap_or_default();
                let attempts: i32 = row.try_get_by("attempts").unwrap_or(0);
                let available_at: Option<String> = row.try_get_by("available_at").ok();
                let created_at: String = row.try_get_by("created_at").unwrap_or_default();

                let job_type = extract_job_type(&payload);
                let payload_preview = truncate_payload(&payload, 200);

                Some(JobInfo {
                    id: id.to_string(),
                    queue,
                    job_type,
                    payload_preview,
                    attempts,
                    available_at,
                    created_at,
                })
            })
            .collect(),
        Err(_) => vec![],
    };

    // Get failed jobs
    let failed_query = if let Some(queue) = queue_filter {
        format!(
            "SELECT * FROM failed_jobs WHERE queue = '{queue}' ORDER BY failed_at DESC LIMIT {limit}"
        )
    } else {
        format!("SELECT * FROM failed_jobs ORDER BY failed_at DESC LIMIT {limit}")
    };

    let failed_jobs = match db
        .query_all(Statement::from_string(
            db.get_database_backend(),
            failed_query,
        ))
        .await
    {
        Ok(rows) => rows
            .iter()
            .filter_map(|row| {
                let id: i64 = row.try_get_by("id").ok()?;
                let queue: String = row
                    .try_get_by("queue")
                    .unwrap_or_else(|_| "default".to_string());
                let payload: String = row.try_get_by("payload").unwrap_or_default();
                let exception: String = row.try_get_by("exception").unwrap_or_default();
                let failed_at: String = row.try_get_by("failed_at").unwrap_or_default();

                let job_type = extract_job_type(&payload);
                let payload_preview = truncate_payload(&payload, 200);

                Some(FailedJobInfo {
                    id: id.to_string(),
                    queue,
                    job_type,
                    payload_preview,
                    exception: truncate_payload(&exception, 500),
                    failed_at,
                })
            })
            .collect(),
        Err(_) => vec![],
    };

    // Build stats
    let mut jobs_by_queue: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut jobs_by_type: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for job in &pending_jobs {
        *jobs_by_queue.entry(job.queue.clone()).or_insert(0) += 1;
        *jobs_by_type.entry(job.job_type.clone()).or_insert(0) += 1;
    }

    Ok(JobHistoryInfo {
        queue_driver: "database".to_string(),
        stats: JobStats {
            pending_count: pending_jobs.len(),
            failed_count: failed_jobs.len(),
            jobs_by_queue,
            jobs_by_type,
        },
        pending_jobs,
        failed_jobs,
    })
}

fn get_redis_job_history(queue_filter: Option<&str>, _limit: usize) -> Result<JobHistoryInfo> {
    let redis_url = std::env::var("REDIS_URL")
        .or_else(|_| std::env::var("QUEUE_REDIS_URL"))
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    match redis::Client::open(redis_url.as_str()) {
        Ok(client) => {
            match client.get_connection() {
                Ok(mut conn) => {
                    let queues = if let Some(queue) = queue_filter {
                        vec![queue.to_string()]
                    } else {
                        // Get all queue keys
                        let keys: Vec<String> = redis::cmd("KEYS")
                            .arg("queues:*")
                            .query(&mut conn)
                            .unwrap_or_default();
                        keys.into_iter()
                            .map(|k| k.trim_start_matches("queues:").to_string())
                            .collect()
                    };

                    let mut pending_jobs = Vec::new();
                    let mut jobs_by_queue: std::collections::HashMap<String, usize> =
                        std::collections::HashMap::new();

                    for queue in &queues {
                        let queue_key = format!("queues:{queue}");
                        let jobs: Vec<String> = redis::cmd("LRANGE")
                            .arg(&queue_key)
                            .arg(0)
                            .arg(50)
                            .query(&mut conn)
                            .unwrap_or_default();

                        *jobs_by_queue.entry(queue.clone()).or_insert(0) += jobs.len();

                        for (idx, payload) in jobs.iter().enumerate() {
                            let job_type = extract_job_type(payload);
                            pending_jobs.push(JobInfo {
                                id: format!("{queue}:{idx}"),
                                queue: queue.clone(),
                                job_type,
                                payload_preview: truncate_payload(payload, 200),
                                attempts: 0,
                                available_at: None,
                                created_at: "unknown".to_string(),
                            });
                        }
                    }

                    Ok(JobHistoryInfo {
                        queue_driver: "redis".to_string(),
                        stats: JobStats {
                            pending_count: pending_jobs.len(),
                            failed_count: 0,
                            jobs_by_queue,
                            jobs_by_type: std::collections::HashMap::new(),
                        },
                        pending_jobs,
                        failed_jobs: vec![],
                    })
                }
                Err(e) => Err(McpError::DatabaseError(format!(
                    "Redis connection failed: {e}"
                ))),
            }
        }
        Err(e) => Err(McpError::DatabaseError(format!(
            "Redis client creation failed: {e}"
        ))),
    }
}

fn extract_job_type(payload: &str) -> String {
    // Try to parse JSON and extract job type
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(payload) {
        parsed
            .get("job_type")
            .or_else(|| parsed.get("type"))
            .or_else(|| parsed.get("job"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        "unknown".to_string()
    }
}

fn truncate_payload(payload: &str, max_len: usize) -> String {
    if payload.len() > max_len {
        format!("{}...", &payload[..max_len])
    } else {
        payload.to_string()
    }
}

fn get_database_url(project_root: &Path) -> Result<String> {
    dotenvy::from_path(project_root.join(".env")).ok();

    std::env::var("DATABASE_URL")
        .map_err(|_| McpError::ConfigError("DATABASE_URL not set in .env".to_string()))
}
