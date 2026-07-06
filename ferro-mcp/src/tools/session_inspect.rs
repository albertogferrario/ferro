//! Session inspection tool - view active sessions for debugging auth issues

use crate::error::{McpError, Result};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub user_id: Option<i64>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub last_activity: String,
    pub payload_preview: String,
}

#[derive(Debug, Serialize)]
pub struct SessionsResult {
    pub total_sessions: usize,
    pub authenticated_sessions: usize,
    pub sessions: Vec<SessionInfo>,
}

/// Inspect active sessions in the database
///
/// Useful for debugging authentication issues like:
/// - Session not persisting after login
/// - User ID not being set in session
/// - Session cookie mismatches
pub async fn execute(project_root: &Path, session_id: Option<&str>) -> Result<SessionsResult> {
    let database_url = get_database_url(project_root)?;

    let db: DatabaseConnection = Database::connect(&database_url)
        .await
        .map_err(|e| McpError::DatabaseError(format!("Failed to connect: {e}")))?;

    let query = if let Some(id) = session_id {
        format!(
            "SELECT id, user_id, ip_address, user_agent, last_activity, payload FROM sessions WHERE id = '{}'",
            id.replace('\'', "''") // Basic SQL injection prevention
        )
    } else {
        "SELECT id, user_id, ip_address, user_agent, last_activity, payload FROM sessions ORDER BY last_activity DESC LIMIT 20".to_string()
    };

    let result = db
        .query_all(Statement::from_string(db.get_database_backend(), query))
        .await
        .map_err(|e| McpError::DatabaseError(format!("Query failed: {e}")))?;

    let mut sessions = Vec::new();
    let mut authenticated_count = 0;

    for row in &result {
        let id: String = row.try_get_by("id").unwrap_or_default();
        let user_id: Option<i64> = row.try_get_by("user_id").ok();
        let ip_address: Option<String> = row.try_get_by("ip_address").ok();
        let user_agent: Option<String> = row.try_get_by("user_agent").ok();
        let last_activity: String = row
            .try_get_by::<String, _>("last_activity")
            .unwrap_or_else(|_| "unknown".to_string());
        let payload: String = row.try_get_by("payload").unwrap_or_default();

        if user_id.is_some() {
            authenticated_count += 1;
        }

        // Truncate payload for preview
        let payload_preview = if payload.len() > 200 {
            format!("{}...", &payload[..200])
        } else {
            payload
        };

        sessions.push(SessionInfo {
            id,
            user_id,
            ip_address,
            user_agent,
            last_activity,
            payload_preview,
        });
    }

    Ok(SessionsResult {
        total_sessions: sessions.len(),
        authenticated_sessions: authenticated_count,
        sessions,
    })
}

fn get_database_url(project_root: &Path) -> Result<String> {
    dotenvy::from_path(project_root.join(".env")).ok();

    std::env::var("DATABASE_URL")
        .map_err(|_| McpError::ConfigError("DATABASE_URL not set in .env".to_string()))
}
