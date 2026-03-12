//! Theme resolver trait and concrete implementations for Ferro framework.
//!
//! Defines the contract for pluggable theme resolution strategies and provides
//! three built-in implementations: tenant-based, header-based, and default.

use crate::Request;
use async_trait::async_trait;
use ferro_theme::Theme;
use std::sync::Arc;

/// Resolves the active theme from an incoming request.
///
/// Implement this trait to provide custom resolution strategies such as
/// reading from tenant context, request headers, or any other source.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::theme::{Theme, ThemeResolver};
/// use ferro_rs::Request;
/// use async_trait::async_trait;
/// use std::sync::Arc;
///
/// struct MyResolver;
///
/// #[async_trait]
/// impl ThemeResolver for MyResolver {
///     async fn resolve(&self, req: &Request) -> Option<Arc<Theme>> {
///         None
///     }
/// }
/// ```
#[async_trait]
pub trait ThemeResolver: Send + Sync {
    /// Resolve the active theme from the given request.
    ///
    /// Returns `None` if this resolver cannot determine a theme for the request.
    async fn resolve(&self, req: &Request) -> Option<Arc<Theme>>;
}

/// Resolves theme from the current tenant's plan field.
///
/// Reads the plan name from `current_tenant().plan` and loads the matching
/// theme directory. Uses a moka TTL cache to avoid redundant disk reads.
///
/// Requires `TenantMiddleware` to run before `ThemeMiddleware` so that
/// `current_tenant()` is populated.
///
/// Note: Uses `tenant.plan` as theme name until a dedicated `theme_name`
/// field is added to `TenantContext`. For v1, the plan name doubles as
/// theme selector.
pub struct TenantThemeResolver {
    /// moka TTL cache: theme_name -> Arc<Theme>
    cache: moka::sync::Cache<String, Arc<Theme>>,
    /// Base directory containing theme subdirectories
    themes_dir: String,
}

impl TenantThemeResolver {
    /// Create a new `TenantThemeResolver`.
    ///
    /// - `themes_dir` — path to the directory containing theme subdirectories.
    ///   Each subdirectory name maps to a plan name.
    pub fn new(themes_dir: impl Into<String>) -> Self {
        Self {
            cache: moka::sync::Cache::builder()
                .time_to_live(std::time::Duration::from_secs(300))
                .max_capacity(100)
                .build(),
            themes_dir: themes_dir.into(),
        }
    }
}

#[async_trait]
impl ThemeResolver for TenantThemeResolver {
    async fn resolve(&self, _req: &Request) -> Option<Arc<Theme>> {
        let tenant = crate::tenant::current_tenant()?;
        let theme_name = tenant.plan.as_deref()?;

        if let Some(cached) = self.cache.get(theme_name) {
            return Some(cached);
        }

        let path = format!("{}/{}", self.themes_dir, theme_name);
        let theme = Arc::new(Theme::from_path(&path).ok()?);
        self.cache
            .insert(theme_name.to_string(), Arc::clone(&theme));
        Some(theme)
    }
}

/// Resolves theme from the `X-Theme` request header.
///
/// Reads the header value and loads the matching theme directory. Uses a
/// moka TTL cache to avoid redundant disk reads.
pub struct HeaderThemeResolver {
    themes_dir: String,
    cache: moka::sync::Cache<String, Arc<Theme>>,
}

impl HeaderThemeResolver {
    /// Create a new `HeaderThemeResolver`.
    ///
    /// - `themes_dir` — path to the directory containing theme subdirectories.
    ///   Each subdirectory name maps to a theme name in the `X-Theme` header.
    pub fn new(themes_dir: impl Into<String>) -> Self {
        Self {
            themes_dir: themes_dir.into(),
            cache: moka::sync::Cache::builder()
                .time_to_live(std::time::Duration::from_secs(300))
                .max_capacity(100)
                .build(),
        }
    }
}

#[async_trait]
impl ThemeResolver for HeaderThemeResolver {
    async fn resolve(&self, req: &Request) -> Option<Arc<Theme>> {
        let theme_name = req.header("x-theme")?;

        if let Some(cached) = self.cache.get(theme_name) {
            return Some(cached);
        }

        let path = format!("{}/{}", self.themes_dir, theme_name);
        let theme = Arc::new(Theme::from_path(&path).ok()?);
        self.cache
            .insert(theme_name.to_string(), Arc::clone(&theme));
        Some(theme)
    }
}

/// Resolver that always returns the configured default theme.
///
/// Use as the last resolver in the chain to ensure every request has a theme.
pub struct DefaultResolver {
    default: Arc<Theme>,
}

impl DefaultResolver {
    /// Create a new `DefaultResolver` with the given theme.
    pub fn new(theme: Theme) -> Self {
        Self {
            default: Arc::new(theme),
        }
    }
}

#[async_trait]
impl ThemeResolver for DefaultResolver {
    async fn resolve(&self, _req: &Request) -> Option<Arc<Theme>> {
        Some(Arc::clone(&self.default))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant::context::{tenant_scope, with_tenant_scope};
    use crate::tenant::TenantContext;
    use bytes::Bytes;
    use http_body_util::Empty;
    use hyper_util::rt::TokioIo;
    use std::sync::Mutex;
    use tokio::sync::oneshot;

    fn make_tenant_with_plan(plan: &str) -> TenantContext {
        TenantContext {
            id: 1,
            slug: "acme".to_string(),
            name: "ACME Corp".to_string(),
            plan: Some(plan.to_string()),
            #[cfg(feature = "stripe")]
            subscription: None,
        }
    }

    fn make_tenant_no_plan() -> TenantContext {
        TenantContext {
            id: 1,
            slug: "acme".to_string(),
            name: "ACME Corp".to_string(),
            plan: None,
            #[cfg(feature = "stripe")]
            subscription: None,
        }
    }

    /// Create a test Request via TCP loopback with optional headers.
    async fn make_request_with_header(header_name: &str, header_value: &str) -> Request {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = oneshot::channel();
        let tx_holder = Arc::new(Mutex::new(Some(tx)));

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let io = TokioIo::new(stream);
            let tx_holder = tx_holder.clone();
            let service =
                hyper::service::service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
                    let tx_holder = tx_holder.clone();
                    async move {
                        if let Some(tx) = tx_holder.lock().unwrap().take() {
                            let _ = tx.send(Request::new(req));
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

        let req = hyper::Request::builder()
            .uri("/test")
            .header(header_name, header_value)
            .body(Empty::<Bytes>::new())
            .unwrap();

        let _ = sender.send_request(req).await;
        rx.await.unwrap()
    }

    async fn make_request() -> Request {
        make_request_with_header("x-test", "1").await
    }

    fn make_theme_dir(name: &str) -> (tempfile::TempDir, String) {
        let dir = tempfile::tempdir().unwrap();
        let theme_dir = dir.path().join(name);
        std::fs::create_dir_all(&theme_dir).unwrap();
        std::fs::write(
            theme_dir.join("tokens.css"),
            "@theme { --color-primary: oklch(55% 0.2 250); }",
        )
        .unwrap();
        let themes_dir = dir.path().to_str().unwrap().to_string();
        (dir, themes_dir)
    }

    // Test: ThemeResolver trait is object-safe
    #[test]
    fn theme_resolver_is_object_safe() {
        let _: Box<dyn ThemeResolver>;
    }

    // Test: DefaultResolver always returns the configured default theme
    #[tokio::test]
    async fn default_resolver_always_returns_default() {
        let theme = Theme::default_theme();
        let resolver = DefaultResolver::new(theme);
        let req = make_request().await;
        let result = resolver.resolve(&req).await;
        assert!(result.is_some());
        assert!(result.unwrap().css.contains("--color-primary"));
    }

    // Test: DefaultResolver returns theme regardless of request content
    #[tokio::test]
    async fn default_resolver_returns_theme_for_any_request() {
        let theme = Theme::default_theme();
        let resolver = DefaultResolver::new(theme);
        // Request with x-theme header — should still return default
        let req = make_request_with_header("x-theme", "some-theme").await;
        let result = resolver.resolve(&req).await;
        assert!(result.is_some());
    }

    // Test: HeaderThemeResolver returns Some when X-Theme header present and theme dir exists
    #[tokio::test]
    async fn header_theme_resolver_returns_some_when_header_present_and_dir_exists() {
        let (_dir, themes_dir) = make_theme_dir("pro");
        let resolver = HeaderThemeResolver::new(&themes_dir);
        let req = make_request_with_header("x-theme", "pro").await;
        let result = resolver.resolve(&req).await;
        assert!(
            result.is_some(),
            "expected Some(theme) for valid x-theme header"
        );
    }

    // Test: HeaderThemeResolver returns None when X-Theme header absent
    #[tokio::test]
    async fn header_theme_resolver_returns_none_when_header_absent() {
        let (_dir, themes_dir) = make_theme_dir("pro");
        let resolver = HeaderThemeResolver::new(&themes_dir);
        let req = make_request().await; // no x-theme header
        let result = resolver.resolve(&req).await;
        assert!(result.is_none());
    }

    // Test: HeaderThemeResolver returns None when theme dir does not exist
    #[tokio::test]
    async fn header_theme_resolver_returns_none_when_dir_does_not_exist() {
        let resolver = HeaderThemeResolver::new("/nonexistent/themes");
        let req = make_request_with_header("x-theme", "pro").await;
        let result = resolver.resolve(&req).await;
        assert!(result.is_none());
    }

    // Test: HeaderThemeResolver moka cache returns cached theme on second resolve (no disk read)
    #[tokio::test]
    async fn header_theme_resolver_cache_returns_on_second_resolve() {
        let (_dir, themes_dir) = make_theme_dir("enterprise");
        let resolver = HeaderThemeResolver::new(&themes_dir);

        // First resolve: loads from disk and caches
        let req1 = make_request_with_header("x-theme", "enterprise").await;
        let result1 = resolver.resolve(&req1).await;
        assert!(result1.is_some());

        // Delete the theme dir to prove cache is used on second resolve
        std::fs::remove_dir_all(format!("{themes_dir}/enterprise")).unwrap();

        // Second resolve: must return cached theme (dir no longer exists on disk)
        let req2 = make_request_with_header("x-theme", "enterprise").await;
        let result2 = resolver.resolve(&req2).await;
        assert!(
            result2.is_some(),
            "second resolve should return cached theme even after disk deletion"
        );
    }

    // Test: TenantThemeResolver returns Some when current_tenant().plan matches a theme dir
    #[tokio::test]
    async fn tenant_theme_resolver_returns_some_when_plan_matches_dir() {
        let (_dir, themes_dir) = make_theme_dir("pro");
        let resolver = TenantThemeResolver::new(&themes_dir);

        let scope = tenant_scope();
        {
            let mut guard = scope.write().await;
            *guard = Some(make_tenant_with_plan("pro"));
        }

        let req = make_request().await;
        let result = with_tenant_scope(scope, resolver.resolve(&req)).await;
        assert!(
            result.is_some(),
            "expected Some(theme) when plan matches dir"
        );
    }

    // Test: TenantThemeResolver returns None when no tenant in context
    #[tokio::test]
    async fn tenant_theme_resolver_returns_none_when_no_tenant() {
        let resolver = TenantThemeResolver::new("/some/themes");
        let req = make_request().await;
        // No tenant scope — current_tenant() returns None
        let result = resolver.resolve(&req).await;
        assert!(result.is_none());
    }

    // Test: TenantThemeResolver returns None when tenant has no plan
    #[tokio::test]
    async fn tenant_theme_resolver_returns_none_when_no_plan() {
        let resolver = TenantThemeResolver::new("/some/themes");

        let scope = tenant_scope();
        {
            let mut guard = scope.write().await;
            *guard = Some(make_tenant_no_plan());
        }

        let req = make_request().await;
        let result = with_tenant_scope(scope, resolver.resolve(&req)).await;
        assert!(result.is_none());
    }

    // Test: TenantThemeResolver moka cache returns cached theme on second resolve (no disk read)
    #[tokio::test]
    async fn tenant_theme_resolver_cache_returns_on_second_resolve() {
        let (_dir, themes_dir) = make_theme_dir("starter");
        let resolver = TenantThemeResolver::new(&themes_dir);

        let scope1 = tenant_scope();
        {
            let mut guard = scope1.write().await;
            *guard = Some(make_tenant_with_plan("starter"));
        }

        // First resolve: loads from disk
        let req1 = make_request().await;
        let result1 = with_tenant_scope(scope1, resolver.resolve(&req1)).await;
        assert!(result1.is_some());

        // Delete the theme dir
        std::fs::remove_dir_all(format!("{themes_dir}/starter")).unwrap();

        // Second resolve: must return cached theme (dir deleted)
        let scope2 = tenant_scope();
        {
            let mut guard = scope2.write().await;
            *guard = Some(make_tenant_with_plan("starter"));
        }
        let req2 = make_request().await;
        let result2 = with_tenant_scope(scope2, resolver.resolve(&req2)).await;
        assert!(
            result2.is_some(),
            "second resolve should return cached theme even after disk deletion"
        );
    }
}
