//! MCP Server implementation

use crate::service::FerroMcpService;
use rmcp::ServiceExt;

pub struct McpServer {
    project_root: std::path::PathBuf,
}

impl McpServer {
    pub fn new() -> Self {
        let project_root =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        Self { project_root }
    }

    pub fn with_project_root(project_root: std::path::PathBuf) -> Self {
        Self { project_root }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let service = FerroMcpService::new(self.project_root.clone());

        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let server = service.serve((stdin, stdout)).await?;

        server.waiting().await?;

        Ok(())
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}
