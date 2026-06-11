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
pub mod resume;
pub mod store;
pub mod token;
pub mod validate;

pub use config::{OAuthConfig, OAuthConfigError};
pub use error::OAuthError;
pub use jwt::McpTokenClaims;
pub use migration::Migration as CreateOauthClientsTable;
pub use resume::{oauth_resume_redirect, store_oauth_return_to, take_oauth_return_to};
pub use validate::{validate_bearer, BearerCheck};

/// Route handler re-exports for mounting in `app/src/routes.rs`.
pub mod handlers {
    pub use crate::authorize::authorize_get;
    pub use crate::consent::authorize_post;
    pub use crate::discovery::{authorization_server_handler, protected_resource_handler};
    pub use crate::register::register_client;
    pub use crate::token::token_exchange;
}

/// Test helpers for bootstrapping the in-memory cache in unit/integration tests.
///
/// Not gated by `#[cfg(test)]` so they are usable in integration tests under `tests/`.
/// These functions are intentionally public — they are harmless no-ops on a live server
/// because `Cache::bootstrap()` via `Server::run()` replaces any earlier binding.
pub mod cache_test_helpers {
    use ferro::cache::{CacheStore, InMemoryCache};
    use ferro::container::App;
    use std::sync::Arc;

    /// Bind a fresh `InMemoryCache` into the App container.
    ///
    /// Call once per test (or test module) before any `Cache::put`/`Cache::get`/`Cache::forget`.
    pub fn bootstrap_test_cache() {
        App::bind::<dyn CacheStore>(Arc::new(InMemoryCache::new()));
    }
}
