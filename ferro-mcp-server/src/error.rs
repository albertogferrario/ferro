use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("render error: {0}")]
    Render(String),
    /// A caller-supplied filter key is unknown or not filter-eligible. This is a
    /// client parameter problem (maps to JSON-RPC `-32602` Invalid params),
    /// distinct from an internal `Database` failure (`-32603` Internal error).
    #[error("invalid filter: {0}")]
    InvalidFilter(String),
    /// Caller is not authenticated or their credential scope is insufficient.
    /// Maps to JSON-RPC -32603 at the jsonrpc layer (same envelope as OAuth invalid-token).
    #[error("auth error: {0}")]
    Auth(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// The resolved action name is not found in any mcp_exposed ServiceDef.
    /// Maps to JSON-RPC -32601 (method not found) at the jsonrpc layer.
    #[error("action not found: {0}")]
    ActionNotFound(String),
    /// A precondition guard returned false or errored at execution time.
    /// Maps to a structured tool error result (isError:true), NOT a -32603.
    /// Never discloses which guard or what state it checked.
    #[error("guard failed: {0}")]
    GuardFailed(String),
    /// Input validation failed (required field missing, wrong type, etc.).
    /// Maps to a structured tool error result (isError:true).
    #[error("validation error: {0}")]
    Validation(String),
    /// A destructive action was called without a valid confirmation token.
    /// Maps to a structured tool error result (isError:true) pointing the agent
    /// to `request_confirm_<action>`.
    /// Feature-gated: only reachable when the `confirmation` feature is enabled.
    #[cfg(feature = "confirmation")]
    #[error("confirmation required for action: {0}")]
    ConfirmationRequired(String),
}

/// Map the channel-agnostic kernel error ([`ferro_rs::write::WriteError`]) into
/// the MCP crate error at the framing boundary, variant-for-variant. This
/// preserves the JSON-RPC error-code mapping and `isError:true` semantics: no
/// variant is silently downgraded to success (threat T-232-03).
impl From<ferro_rs::write::WriteError> for Error {
    fn from(e: ferro_rs::write::WriteError) -> Self {
        use ferro_rs::write::WriteError as W;
        match e {
            W::Database(m) => Error::Database(m),
            W::Serialization(e) => Error::Serialization(e),
            W::GuardFailed(m) => Error::GuardFailed(m),
            W::Validation(m) => Error::Validation(m),
            W::ActionNotFound(m) => Error::ActionNotFound(m),
            #[cfg(feature = "confirmation")]
            W::ConfirmationRequired(m) => Error::ConfirmationRequired(m),
        }
    }
}
