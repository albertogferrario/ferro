//! Ferro MCP - Model Context Protocol server for AI-assisted Ferro Framework development

pub mod error;
pub mod introspection;
pub mod resources;
pub mod server;
pub mod service;
pub mod tools;

pub use server::McpServer;

/// Library entrypoint used by `ferro-cli` to launch the MCP server in-process.
///
/// Resolves the project root from the first CLI argument if present, falling
/// back to the current working directory. Mirrors the behaviour of the former
/// standalone `ferro-mcp` binary so callers can swap a subprocess spawn for a
/// direct function call.
pub async fn run() -> anyhow::Result<()> {
    let project_root = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        });

    let server = McpServer::with_project_root(project_root);
    server
        .run()
        .await
        .map_err(|e| anyhow::anyhow!("ferro-mcp server failed: {e}"))
}
