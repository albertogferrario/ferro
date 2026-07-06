/// Errors that can occur in the ferro-api-mcp server.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to fetch the OpenAPI spec from the given URL.
    #[error("failed to fetch OpenAPI spec: {0}")]
    SpecFetch(String),

    /// Failed to parse the OpenAPI spec JSON.
    #[error("failed to parse OpenAPI spec: {0}")]
    SpecParse(String),

    /// The OpenAPI spec version is not supported (only 3.0.x).
    #[error("unsupported OpenAPI version: {0}")]
    UnsupportedVersion(String),

    /// A `$ref` in the spec could not be resolved.
    #[error("unresolved $ref: {0}")]
    UnresolvedRef(String),

    /// HTTP client error during an API call.
    #[error("HTTP client error: {0}")]
    HttpClient(String),

    /// The API returned a non-success status code.
    #[error("API error (status {status}): {body}")]
    ApiError { status: u16, body: String },

    /// Error during MCP tool execution.
    #[error("tool execution error: {0}")]
    ToolExecution(String),

    /// MCP server error.
    #[error("server error: {0}")]
    Server(String),
}
