//! `ferro-mcp` binary — standalone entry point so `ferro-cli` can spawn the
//! MCP server as a subprocess without taking a direct library dependency on
//! ferro-mcp (which would create a cycle, since ferro-mcp depends on ferro-cli
//! for shared deploy helpers per D-12).

use std::path::PathBuf;

fn main() {
    let project_root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    rt.block_on(async {
        let server = ferro_mcp::McpServer::with_project_root(project_root);
        if let Err(e) = server.run().await {
            eprintln!("ferro-mcp: failed to run server: {e}");
            std::process::exit(1);
        }
    });
}
