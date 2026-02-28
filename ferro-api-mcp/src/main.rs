use std::sync::Arc;

use clap::Parser;
use tracing_subscriber::EnvFilter;
use url::Url;

use ferro_api_mcp::http::HttpClient;
use ferro_api_mcp::schema::build_input_schema;
use ferro_api_mcp::server::McpServer;
use ferro_api_mcp::service::ApiMcpService;
use ferro_api_mcp::spec;

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

    // Tracing goes to stderr so stdout is clean for the MCP JSON-RPC transport
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&cli.log_level)),
        )
        .init();

    tracing::debug!(?cli, "parsed CLI arguments");

    if let Err(err) = run(cli).await {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Fetch spec
    let spec_json = spec::fetch_spec(&cli.spec_url).await.map_err(|e| {
        format!(
            "Failed to fetch OpenAPI spec from {}: {e}",
            cli.spec_url
        )
    })?;

    // 2. Parse spec into operations
    let mut operations =
        spec::parse_spec(&spec_json).map_err(|e| format!("Failed to parse OpenAPI spec: {e}"))?;

    eprintln!("Parsed {} API operations from spec", operations.len());

    // 3. Extract metadata (title, server URL)
    let metadata = spec::extract_metadata(&spec_json)
        .map_err(|e| format!("Failed to extract spec metadata: {e}"))?;

    // 4. Determine base URL
    let base_url = resolve_base_url(&cli.base_url, &metadata.server_url, &cli.spec_url)?;
    eprintln!("Base URL: {base_url}");

    // 5. Build input schemas for each operation
    for op in &mut operations {
        op.input_schema = build_input_schema(&op.parameters, op.request_body_schema.as_ref());
    }

    // 6. Create HTTP client
    let http_client = Arc::new(HttpClient::new(base_url, cli.api_key));

    // 7. Create service
    let service = ApiMcpService::new(metadata.title, operations, http_client);

    // 8. Start server
    let server = McpServer::new(service);
    server.run().await?;

    Ok(())
}

/// Resolve the base URL from CLI override, spec servers, or spec URL origin.
fn resolve_base_url(
    cli_base_url: &Option<String>,
    spec_server_url: &Option<String>,
    spec_url: &str,
) -> Result<Url, Box<dyn std::error::Error>> {
    if let Some(url_str) = cli_base_url {
        return Url::parse(url_str)
            .map_err(|e| format!("Invalid --base-url '{url_str}': {e}").into());
    }

    if let Some(url_str) = spec_server_url {
        if let Ok(url) = Url::parse(url_str) {
            return Ok(url);
        }
        // Server URL might be a relative path; fall through to spec URL origin
    }

    // Extract origin from spec_url
    let parsed = Url::parse(spec_url)
        .map_err(|e| format!("Cannot determine base URL from spec URL '{spec_url}': {e}"))?;
    Ok(parsed.origin().unicode_serialization().parse()?)
}
