use crate::service::ApiMcpService;
use rmcp::ServiceExt;

/// MCP server that runs the API service over stdio transport.
pub struct McpServer {
    service: ApiMcpService,
}

impl McpServer {
    pub fn new(service: ApiMcpService) -> Self {
        Self { service }
    }

    /// Start the MCP server on stdin/stdout.
    ///
    /// Stdout is the MCP JSON-RPC transport; all diagnostic output
    /// must go to stderr (via tracing or eprintln).
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let server = self.service.serve((stdin, stdout)).await?;
        server.waiting().await?;

        Ok(())
    }
}
