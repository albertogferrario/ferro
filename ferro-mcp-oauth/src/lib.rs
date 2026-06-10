//! OAuth 2.1 authorization server for MCP endpoints.
//!
//! Provides mountable route handlers (discovery, DCR, authorize, consent, token)
//! and a `validate_bearer` function for validating bearer tokens at the `/mcp`
//! handler call site. Consumers mount the routes and call `validate_bearer`
//! directly — `ferro-mcp-server` gains no new dependency.

pub mod authorize;
pub mod config;
pub mod consent;
pub mod discovery;
pub mod error;
pub mod jwt;
pub mod migration;
pub mod pkce;
pub mod register;
pub mod store;
pub mod token;
pub mod validate;

pub use config::{OAuthConfig, OAuthConfigError};
pub use error::OAuthError;
pub use jwt::McpTokenClaims;
pub use migration::Migration as CreateOauthClientsTable;
pub use validate::{validate_bearer, BearerCheck};
