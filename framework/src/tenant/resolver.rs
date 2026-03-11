//! Tenant resolver trait and concrete implementations for Ferro framework.
//!
//! Defines the contract for pluggable tenant resolution strategies and provides
//! four built-in implementations: subdomain, header, path parameter, and JWT claim.

use crate::tenant::TenantContext;
use crate::Request;
use async_trait::async_trait;
use std::sync::Arc;

use super::lookup::TenantLookup;

/// Resolves the current tenant from an incoming request.
///
/// Implement this trait to provide custom resolution strategies such as
/// subdomain parsing, header inspection, or JWT claim extraction.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::tenant::{TenantContext, TenantResolver};
/// use ferro_rs::Request;
/// use async_trait::async_trait;
///
/// struct SubdomainResolver;
///
/// #[async_trait]
/// impl TenantResolver for SubdomainResolver {
///     async fn resolve(&self, req: &Request) -> Option<TenantContext> {
///         // Extract subdomain from Host header and resolve tenant
///         None
///     }
/// }
/// ```
#[async_trait]
pub trait TenantResolver: Send + Sync {
    /// Resolve the tenant from the given request.
    ///
    /// Returns `None` if no tenant could be determined.
    async fn resolve(&self, req: &Request) -> Option<TenantContext>;
}

/// Resolves tenant from the Host header subdomain.
///
/// Extracts the leftmost subdomain segment by stripping the rightmost
/// `base_domain_parts` segments. For example, with `base_domain_parts = 2`:
/// - `acme.yourapp.com` → slug `"acme"`
/// - `yourapp.com` → `None` (no subdomain)
/// - `acme.yourapp.com:8080` → slug `"acme"` (port stripped)
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::tenant::{SubdomainResolver, DbTenantLookup};
/// use std::sync::Arc;
///
/// let resolver = SubdomainResolver::new(2, Arc::new(lookup));
/// ```
pub struct SubdomainResolver {
    base_domain_parts: usize,
    tenant_lookup: Arc<dyn TenantLookup>,
}

impl SubdomainResolver {
    /// Create a new `SubdomainResolver`.
    ///
    /// - `base_domain_parts` — number of base domain segments to strip (e.g. 2 for `yourapp.com`)
    /// - `tenant_lookup` — lookup implementation for DB verification
    pub fn new(base_domain_parts: usize, tenant_lookup: Arc<dyn TenantLookup>) -> Self {
        Self {
            base_domain_parts,
            tenant_lookup,
        }
    }
}

#[async_trait]
impl TenantResolver for SubdomainResolver {
    async fn resolve(&self, req: &Request) -> Option<TenantContext> {
        let host = req.header("host")?;
        // Strip port: "acme.yourapp.com:8080" -> "acme.yourapp.com"
        let host_no_port = host.split(':').next()?;
        let parts: Vec<&str> = host_no_port.split('.').collect();
        if parts.len() <= self.base_domain_parts {
            return None;
        }
        let slug = parts[0];
        self.tenant_lookup.find_by_slug(slug).await
    }
}

/// Resolves tenant from a configurable HTTP header.
///
/// Reads the header named `header_name` and passes its value to the lookup
/// as a slug.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::tenant::{HeaderResolver, DbTenantLookup};
/// use std::sync::Arc;
///
/// let resolver = HeaderResolver::new("X-Tenant-ID", Arc::new(lookup));
/// ```
pub struct HeaderResolver {
    header_name: String,
    tenant_lookup: Arc<dyn TenantLookup>,
}

impl HeaderResolver {
    /// Create a new `HeaderResolver`.
    ///
    /// - `header_name` — the HTTP header to read the tenant slug from
    /// - `tenant_lookup` — lookup implementation for DB verification
    pub fn new(header_name: impl Into<String>, tenant_lookup: Arc<dyn TenantLookup>) -> Self {
        Self {
            header_name: header_name.into(),
            tenant_lookup,
        }
    }
}

#[async_trait]
impl TenantResolver for HeaderResolver {
    async fn resolve(&self, req: &Request) -> Option<TenantContext> {
        let value = req.header(&self.header_name)?;
        self.tenant_lookup.find_by_slug(value).await
    }
}

/// Resolves tenant from a route path parameter.
///
/// Reads the route parameter named `param_name` and passes it to the lookup
/// as a slug.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::tenant::{PathResolver, DbTenantLookup};
/// use std::sync::Arc;
///
/// // For routes like /{tenant_slug}/dashboard
/// let resolver = PathResolver::new("tenant_slug", Arc::new(lookup));
/// ```
pub struct PathResolver {
    param_name: String,
    tenant_lookup: Arc<dyn TenantLookup>,
}

impl PathResolver {
    /// Create a new `PathResolver`.
    ///
    /// - `param_name` — the route parameter name containing the tenant slug
    /// - `tenant_lookup` — lookup implementation for DB verification
    pub fn new(param_name: impl Into<String>, tenant_lookup: Arc<dyn TenantLookup>) -> Self {
        Self {
            param_name: param_name.into(),
            tenant_lookup,
        }
    }
}

#[async_trait]
impl TenantResolver for PathResolver {
    async fn resolve(&self, req: &Request) -> Option<TenantContext> {
        let value = req.param(&self.param_name).ok()?;
        self.tenant_lookup.find_by_slug(value).await
    }
}

/// Resolves tenant from a JWT claim stored in request extensions.
///
/// Reads a `serde_json::Value` inserted by an upstream JWT middleware, extracts
/// the claim field named `claim_field` as an `i64`, and passes it to the lookup
/// as a tenant ID.
///
/// The upstream JWT/auth middleware must insert the parsed claims as
/// `serde_json::Value` via `req.insert::<serde_json::Value>(claims)`.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::tenant::{JwtClaimResolver, DbTenantLookup};
/// use std::sync::Arc;
///
/// // Expects upstream JWT middleware to insert serde_json::Value with a "tenant_id" field
/// let resolver = JwtClaimResolver::new("tenant_id", Arc::new(lookup));
/// ```
pub struct JwtClaimResolver {
    claim_field: String,
    tenant_lookup: Arc<dyn TenantLookup>,
}

impl JwtClaimResolver {
    /// Create a new `JwtClaimResolver`.
    ///
    /// - `claim_field` — the claim key whose value is the tenant ID (i64)
    /// - `tenant_lookup` — lookup implementation for DB verification
    pub fn new(claim_field: impl Into<String>, tenant_lookup: Arc<dyn TenantLookup>) -> Self {
        Self {
            claim_field: claim_field.into(),
            tenant_lookup,
        }
    }
}

#[async_trait]
impl TenantResolver for JwtClaimResolver {
    async fn resolve(&self, req: &Request) -> Option<TenantContext> {
        let claims = req.get::<serde_json::Value>()?;
        let id = claims[&self.claim_field].as_i64()?;
        self.tenant_lookup.find_by_id(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant::{TenantContext, TenantLookup};
    use async_trait::async_trait;
    use bytes::Bytes;
    use http_body_util::Empty;
    use hyper_util::rt::TokioIo;
    use std::sync::Arc;
    use std::sync::Mutex;
    use tokio::sync::oneshot;

    fn make_tenant(slug: &str) -> TenantContext {
        TenantContext {
            id: 42,
            slug: slug.to_string(),
            name: "Test Corp".to_string(),
            plan: None,
        }
    }

    /// Mock lookup that returns a tenant for known slugs/ids and None otherwise.
    struct MockLookup;

    #[async_trait]
    impl TenantLookup for MockLookup {
        async fn find_by_slug(&self, slug: &str) -> Option<TenantContext> {
            match slug {
                "acme" | "beta" => Some(make_tenant(slug)),
                _ => None,
            }
        }

        async fn find_by_id(&self, id: i64) -> Option<TenantContext> {
            if id == 42 {
                Some(make_tenant("acme"))
            } else {
                None
            }
        }
    }

    /// Create a test Request via TCP loopback with optional host, header, and path param.
    async fn make_request_opts(
        host: Option<&str>,
        extra_header: Option<(&str, &str)>,
        params: std::collections::HashMap<String, String>,
    ) -> Request {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = oneshot::channel();
        let tx_holder = Arc::new(Mutex::new(Some(tx)));
        let params_clone = params.clone();

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let io = TokioIo::new(stream);
            let tx_holder = tx_holder.clone();
            let params_inner = params_clone.clone();
            let service =
                hyper::service::service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
                    let tx_holder = tx_holder.clone();
                    let params_for_req = params_inner.clone();
                    async move {
                        if let Some(tx) = tx_holder.lock().unwrap().take() {
                            let _ = tx.send(Request::new(req).with_params(params_for_req));
                        }
                        Ok::<_, hyper::Error>(hyper::Response::new(Empty::<Bytes>::new()))
                    }
                });
            hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service)
                .await
                .ok();
        });

        let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        let io = TokioIo::new(stream);
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await.unwrap();
        tokio::spawn(async move {
            conn.await.ok();
        });

        let mut builder = hyper::Request::builder().uri("/test");
        if let Some(h) = host {
            builder = builder.header("host", h);
        }
        if let Some((name, value)) = extra_header {
            builder = builder.header(name, value);
        }
        let req = builder.body(Empty::<Bytes>::new()).unwrap();

        let _ = sender.send_request(req).await;
        rx.await.unwrap()
    }

    // Test 1: SubdomainResolver extracts "acme" from Host "acme.yourapp.com" with base_domain_parts=2
    #[tokio::test]
    async fn subdomain_resolver_extracts_slug_from_host() {
        let lookup = Arc::new(MockLookup);
        let resolver = SubdomainResolver::new(2, lookup);
        let req = make_request_opts(Some("acme.yourapp.com"), None, Default::default()).await;

        let result = resolver.resolve(&req).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().slug, "acme");
    }

    // Test 2: SubdomainResolver returns None for "yourapp.com" (no subdomain, only base parts)
    #[tokio::test]
    async fn subdomain_resolver_returns_none_for_no_subdomain() {
        let lookup = Arc::new(MockLookup);
        let resolver = SubdomainResolver::new(2, lookup);
        let req = make_request_opts(Some("yourapp.com"), None, Default::default()).await;

        let result = resolver.resolve(&req).await;
        assert!(result.is_none());
    }

    // Test 3: SubdomainResolver strips port from Host header
    #[tokio::test]
    async fn subdomain_resolver_strips_port() {
        let lookup = Arc::new(MockLookup);
        let resolver = SubdomainResolver::new(2, lookup);
        let req = make_request_opts(Some("acme.yourapp.com:8080"), None, Default::default()).await;

        let result = resolver.resolve(&req).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().slug, "acme");
    }

    // Test 4: SubdomainResolver calls tenant_lookup.find_by_slug() with extracted slug
    #[tokio::test]
    async fn subdomain_resolver_calls_find_by_slug() {
        let lookup = Arc::new(MockLookup);
        let resolver = SubdomainResolver::new(2, lookup);
        // "unknown" is not in MockLookup so returns None — proving find_by_slug was called
        let req = make_request_opts(Some("unknown.yourapp.com"), None, Default::default()).await;

        let result = resolver.resolve(&req).await;
        assert!(result.is_none()); // lookup returned None for unknown slug
    }

    // Test 5: HeaderResolver extracts from X-Tenant-ID header and calls find_by_slug
    #[tokio::test]
    async fn header_resolver_extracts_from_header() {
        let lookup = Arc::new(MockLookup);
        let resolver = HeaderResolver::new("x-tenant-id", lookup);
        let req = make_request_opts(None, Some(("x-tenant-id", "acme")), Default::default()).await;

        let result = resolver.resolve(&req).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().slug, "acme");
    }

    // Test 6: HeaderResolver returns None when header is absent
    #[tokio::test]
    async fn header_resolver_returns_none_when_absent() {
        let lookup = Arc::new(MockLookup);
        let resolver = HeaderResolver::new("x-tenant-id", lookup);
        // No x-tenant-id header in request
        let req = make_request_opts(None, None, Default::default()).await;

        let result = resolver.resolve(&req).await;
        assert!(result.is_none());
    }

    // Test 7: PathResolver extracts from request.param("tenant_slug") and calls find_by_slug
    #[tokio::test]
    async fn path_resolver_extracts_from_param() {
        let lookup = Arc::new(MockLookup);
        let resolver = PathResolver::new("tenant_slug", lookup);
        let mut params = std::collections::HashMap::new();
        params.insert("tenant_slug".to_string(), "beta".to_string());
        let req = make_request_opts(None, None, params).await;

        let result = resolver.resolve(&req).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().slug, "beta");
    }

    // Test 8: PathResolver returns None when path parameter is absent
    #[tokio::test]
    async fn path_resolver_returns_none_when_param_absent() {
        let lookup = Arc::new(MockLookup);
        let resolver = PathResolver::new("tenant_slug", lookup);
        // No tenant_slug param
        let req = make_request_opts(None, None, Default::default()).await;

        let result = resolver.resolve(&req).await;
        assert!(result.is_none());
    }

    // Test 9: JwtClaimResolver extracts tenant_id from request extension (pre-parsed JWT claims)
    #[tokio::test]
    async fn jwt_claim_resolver_extracts_from_extensions() {
        let lookup = Arc::new(MockLookup);
        let resolver = JwtClaimResolver::new("tenant_id", lookup);
        let mut req = make_request_opts(None, None, Default::default()).await;

        // Upstream JWT middleware inserts claims as serde_json::Value
        req.insert::<serde_json::Value>(serde_json::json!({"tenant_id": 42, "sub": "user1"}));

        let result = resolver.resolve(&req).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().slug, "acme"); // MockLookup returns "acme" for id=42
    }

    // Test 10: JwtClaimResolver returns None when no JWT claims in request
    #[tokio::test]
    async fn jwt_claim_resolver_returns_none_without_claims() {
        let lookup = Arc::new(MockLookup);
        let resolver = JwtClaimResolver::new("tenant_id", lookup);
        let req = make_request_opts(None, None, Default::default()).await;
        // No claims inserted into extensions

        let result = resolver.resolve(&req).await;
        assert!(result.is_none());
    }

    // Object safety test from original resolver.rs
    #[test]
    fn tenant_resolver_is_object_safe() {
        // If TenantResolver were not object-safe, this would not compile.
        let _: Box<dyn TenantResolver>;
    }
}
