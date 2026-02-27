//! API key authentication for machine-to-machine access.
//!
//! Provides key generation (prefixed, hashed), a provider trait for
//! storage-agnostic verification, and middleware for route protection.
//!
//! # Key Format
//!
//! Keys follow the `fe_{env}_{random}` pattern (similar to Stripe):
//! - `fe_live_` prefix for production keys
//! - `fe_test_` prefix for test/development keys
//! - 43 random base62 characters for the secret portion
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::{ApiKeyMiddleware, generate_api_key};
//!
//! // Generate a new key (show raw_key once, store prefix + hash)
//! let key = generate_api_key("live");
//! println!("API Key: {}", key.raw_key); // show once
//! // Store key.prefix and key.hashed_key in database
//!
//! // Protect routes with middleware
//! group!("/api/v1")
//!     .middleware(ApiKeyMiddleware::new())
//!     .routes([...]);
//!
//! // Require specific scopes
//! group!("/api/v1/admin")
//!     .middleware(ApiKeyMiddleware::scopes(&["admin"]))
//!     .routes([...]);
//! ```

use crate::container::App;
use crate::http::{HttpResponse, Request, Response};
use crate::middleware::{Middleware, Next};
use async_trait::async_trait;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use subtle::ConstantTimeEq;

const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Result of generating a new API key.
///
/// `raw_key` should be shown to the user exactly once.
/// `prefix` and `hashed_key` are stored in the database.
#[derive(Debug, Clone)]
pub struct GeneratedApiKey {
    /// Full key (display once, never store)
    pub raw_key: String,
    /// First 16 characters for database lookup
    pub prefix: String,
    /// SHA-256 hex digest for verification
    pub hashed_key: String,
}

/// Information about an authenticated API key, available in request extensions.
#[derive(Debug, Clone)]
pub struct ApiKeyInfo {
    /// Database identifier for the key
    pub id: i64,
    /// Human-readable name (e.g., "Production Bot")
    pub name: String,
    /// Granted scopes (e.g., ["read", "write"])
    pub scopes: Vec<String>,
}

/// Storage-agnostic API key verification.
///
/// Implement this trait to connect the middleware to your key store
/// (database, cache, etc.).
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::{async_trait, ApiKeyProvider, ApiKeyInfo};
///
/// pub struct DbApiKeyProvider;
///
/// #[async_trait]
/// impl ApiKeyProvider for DbApiKeyProvider {
///     async fn verify_key(&self, raw_key: &str) -> Result<ApiKeyInfo, ()> {
///         let prefix = &raw_key[..16.min(raw_key.len())];
///         let record = api_key::Entity::find()
///             .filter(api_key::Column::Prefix.eq(prefix))
///             .one(&db())
///             .await
///             .map_err(|_| ())?
///             .ok_or(())?;
///
///         if verify_api_key_hash(raw_key, &record.hashed_key) {
///             Ok(ApiKeyInfo {
///                 id: record.id,
///                 name: record.name,
///                 scopes: serde_json::from_str(&record.scopes).unwrap_or_default(),
///             })
///         } else {
///             Err(())
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait ApiKeyProvider: Send + Sync + 'static {
    /// Look up and verify the raw key, returning key metadata on success.
    async fn verify_key(&self, raw_key: &str) -> Result<ApiKeyInfo, ()>;
}

/// Generate a new API key for the given environment.
///
/// Returns a [`GeneratedApiKey`] with the raw key (show once), a prefix for
/// database lookup, and a SHA-256 hash for storage.
pub fn generate_api_key(environment: &str) -> GeneratedApiKey {
    let prefix_str = format!("fe_{environment}_");
    let mut rng = rand::thread_rng();
    let random: String = (0..43)
        .map(|_| {
            let idx = rand::Rng::gen_range(&mut rng, 0..62);
            BASE62[idx] as char
        })
        .collect();

    let raw_key = format!("{prefix_str}{random}");
    let prefix = raw_key[..16].to_string();
    let hashed_key = hash_api_key(&raw_key);

    GeneratedApiKey {
        raw_key,
        prefix,
        hashed_key,
    }
}

/// Compute the SHA-256 hex digest of a raw API key.
pub fn hash_api_key(raw_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_key.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Constant-time comparison of a raw key against a stored hash.
///
/// Prevents timing attacks by using `subtle::ConstantTimeEq`.
pub fn verify_api_key_hash(raw_key: &str, stored_hash: &str) -> bool {
    let incoming_hash = hash_api_key(raw_key);
    incoming_hash
        .as_bytes()
        .ct_eq(stored_hash.as_bytes())
        .into()
}

/// Middleware that authenticates requests via API key in the Authorization header.
///
/// Extracts `Bearer {key}` from the `Authorization` header, resolves an
/// [`ApiKeyProvider`] from the service container, and verifies the key.
/// On success, stores [`ApiKeyInfo`] in request extensions.
///
/// # Example
///
/// ```rust,ignore
/// // Require any valid API key
/// group!("/api/v1")
///     .middleware(ApiKeyMiddleware::new())
///     .routes([...]);
///
/// // Require specific scopes
/// group!("/api/v1/admin")
///     .middleware(ApiKeyMiddleware::scopes(&["admin"]))
///     .routes([...]);
/// ```
pub struct ApiKeyMiddleware {
    required_scopes: Vec<String>,
}

impl ApiKeyMiddleware {
    /// Create middleware that accepts any valid API key.
    pub fn new() -> Self {
        Self {
            required_scopes: Vec::new(),
        }
    }

    /// Create middleware that requires specific scopes.
    pub fn scopes(scopes: &[&str]) -> Self {
        Self {
            required_scopes: scopes.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Default for ApiKeyMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract the bearer token from the Authorization header.
fn extract_bearer_token(request: &Request) -> Result<&str, HttpResponse> {
    let header = request.header("Authorization").ok_or_else(|| {
        HttpResponse::json(serde_json::json!({
            "error": "API key required",
            "hint": "Include Authorization: Bearer <key> header"
        }))
        .status(401)
    })?;

    let token = header.strip_prefix("Bearer ").ok_or_else(|| {
        HttpResponse::json(serde_json::json!({
            "error": "Invalid API key format"
        }))
        .status(401)
    })?;

    if token.is_empty() {
        return Err(HttpResponse::json(serde_json::json!({
            "error": "Invalid API key format"
        }))
        .status(401));
    }

    Ok(token)
}

#[async_trait]
impl Middleware for ApiKeyMiddleware {
    async fn handle(&self, mut request: Request, next: Next) -> Response {
        let raw_key = extract_bearer_token(&request)?;

        let provider: Arc<dyn ApiKeyProvider> =
            App::make::<dyn ApiKeyProvider>().ok_or_else(|| {
                HttpResponse::json(serde_json::json!({
                    "error": "API key authentication not configured"
                }))
                .status(500)
            })?;

        let key_info = provider.verify_key(raw_key).await.map_err(|()| {
            HttpResponse::json(serde_json::json!({
                "error": "Invalid API key"
            }))
            .status(401)
        })?;

        // Check scopes if required
        if !self.required_scopes.is_empty() {
            let has_wildcard = key_info.scopes.iter().any(|s| s == "*");
            if !has_wildcard {
                let missing: Vec<&String> = self
                    .required_scopes
                    .iter()
                    .filter(|required| !key_info.scopes.contains(required))
                    .collect();

                if !missing.is_empty() {
                    return Err(HttpResponse::json(serde_json::json!({
                        "error": "Insufficient permissions",
                        "required": self.required_scopes,
                        "provided": key_info.scopes
                    }))
                    .status(403));
                }
            }
        }

        request.insert(key_info);
        next(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_api_key_format() {
        let key = generate_api_key("live");
        assert!(key.raw_key.starts_with("fe_live_"));
        // fe_live_ = 8 chars + 43 random = 51 total
        assert_eq!(key.raw_key.len(), 51);
    }

    #[test]
    fn generate_api_key_test_env() {
        let key = generate_api_key("test");
        assert!(key.raw_key.starts_with("fe_test_"));
        assert_eq!(key.raw_key.len(), 51);
    }

    #[test]
    fn generate_api_key_prefix_length() {
        let key = generate_api_key("live");
        assert_eq!(key.prefix.len(), 16);
        assert!(key.prefix.starts_with("fe_live_"));
    }

    #[test]
    fn generate_api_key_hash_is_sha256_hex() {
        let key = generate_api_key("live");
        // SHA-256 hex = 64 characters
        assert_eq!(key.hashed_key.len(), 64);
        assert!(key.hashed_key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn generate_api_key_uniqueness() {
        let key1 = generate_api_key("live");
        let key2 = generate_api_key("live");
        assert_ne!(key1.raw_key, key2.raw_key);
        assert_ne!(key1.hashed_key, key2.hashed_key);
    }

    #[test]
    fn generate_api_key_base62_chars_only() {
        let key = generate_api_key("live");
        let random_part = &key.raw_key[8..]; // skip "fe_live_"
        assert!(random_part.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn hash_api_key_deterministic() {
        let hash1 = hash_api_key("fe_live_abc123");
        let hash2 = hash_api_key("fe_live_abc123");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn hash_api_key_different_inputs() {
        let hash1 = hash_api_key("fe_live_abc123");
        let hash2 = hash_api_key("fe_live_xyz789");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn verify_api_key_hash_correct() {
        let key = generate_api_key("live");
        assert!(verify_api_key_hash(&key.raw_key, &key.hashed_key));
    }

    #[test]
    fn verify_api_key_hash_wrong_key() {
        let key = generate_api_key("live");
        assert!(!verify_api_key_hash("wrong_key", &key.hashed_key));
    }

    #[test]
    fn verify_api_key_hash_wrong_hash() {
        let key = generate_api_key("live");
        assert!(!verify_api_key_hash(
            &key.raw_key,
            "0000000000000000000000000000000000000000000000000000000000000000"
        ));
    }

    #[test]
    fn verify_api_key_hash_manual() {
        let raw = "fe_test_SomeRandomBase62String";
        let hash = hash_api_key(raw);
        assert!(verify_api_key_hash(raw, &hash));
    }
}
