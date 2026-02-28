use clap::Parser;
use tracing_subscriber::EnvFilter;

/// Standalone MCP server that bridges OpenAPI specs to MCP tools.
#[derive(Parser, Debug)]
#[command(name = "ferro-api-mcp")]
struct Cli {
    /// URL to fetch the OpenAPI spec from (e.g., http://localhost:8080/api/docs/openapi.json).
    #[arg(long)]
    spec_url: String,

    /// API key for the Authorization header (optional, some APIs are public).
    #[arg(long)]
    api_key: Option<String>,

    /// Override the base URL for API calls (defaults to the spec's server URL or spec_url origin).
    #[arg(long)]
    base_url: Option<String>,

    /// Log level (debug, info, warn, error).
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&cli.log_level)),
        )
        .init();

    tracing::info!("ferro-api-mcp: connecting to {}...", cli.spec_url);

    // TODO: Fetch spec, parse, build service, start server
    tracing::debug!(?cli, "parsed CLI arguments");
}
