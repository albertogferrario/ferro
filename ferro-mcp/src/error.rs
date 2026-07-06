//! Error types for the MCP server

use thiserror::Error;

pub type Result<T> = std::result::Result<T, McpError>;

#[derive(Error, Debug)]
pub enum McpError {
    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("File read error: {0}")]
    FileReadError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Tool error: {0}")]
    ToolError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),
}
