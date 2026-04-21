//! Ferro MCP - Model Context Protocol server for AI-assisted Ferro Framework development

pub mod error;
pub mod introspection;
pub mod resources;
pub mod server;
pub mod service;
pub mod tools;

pub use server::McpServer;

/// Library entrypoint used by `ferro-cli` to launch the MCP server in-process.
/// Uses the current working directory as the project root (the CLI sets it via
/// `set_current_dir` before calling this).
pub async fn run() -> anyhow::Result<()> {
    let project_root =
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    let server = McpServer::with_project_root(project_root);
    server
        .run()
        .await
        .map_err(|e| anyhow::anyhow!("ferro-mcp server failed: {e}"))
}
