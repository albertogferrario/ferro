use crate::http::{Request, Response};
use crate::middleware::{into_boxed, BoxedMiddleware, Middleware};
use matchit::Router as MatchitRouter;
use serde::Serialize;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, OnceLock, RwLock};

/// Global registry mapping route names to path patterns
static ROUTE_REGISTRY: OnceLock<RwLock<HashMap<String, String>>> = OnceLock::new();

/// Global registry of all registered routes for introspection
static REGISTERED_ROUTES: OnceLock<RwLock<Vec<RouteInfo>>> = OnceLock::new();

/// Information about a registered route for introspection
#[derive(Debug, Clone, Default, Serialize)]
pub struct RouteInfo {
    /// HTTP method (GET, POST, PUT, DELETE)
    pub method: String,
    /// Route path pattern (e.g., "/users/{id}")
    pub path: String,
    /// Optional route name (e.g., "users.show")
    pub name: Option<String>,
    /// Middleware applied to this route
    pub middleware: Vec<String>,
    /// Override for auto-generated MCP tool name
    pub mcp_tool_name: Option<String>,
    /// Override for auto-generated MCP description
    pub mcp_description: Option<String>,
    /// Hint text appended to MCP description for AI agent guidance
    pub mcp_hint: Option<String>,
    /// When true, route is hidden from MCP tool discovery
    pub mcp_hidden: bool,
}

/// Register a route for introspection
fn register_route(method: &str, path: &str) {
    let registry = REGISTERED_ROUTES.get_or_init(|| RwLock::new(Vec::new()));
    if let Ok(mut routes) = registry.write() {
        routes.push(RouteInfo {
            method: method.to_string(),
            path: path.to_string(),
            name: None,
            middleware: Vec::new(),
            mcp_tool_name: None,
            mcp_description: None,
            mcp_hint: None,
            mcp_hidden: false,
        });
    }
}

/// Update the most recently registered route with its name
fn update_route_name(path: &str, name: &str) {
    let registry = REGISTERED_ROUTES.get_or_init(|| RwLock::new(Vec::new()));
    if let Ok(mut routes) = registry.write() {
        // Find the most recent route with this path and update its name
        if let Some(route) = routes.iter_mut().rev().find(|r| r.path == path) {
            route.name = Some(name.to_string());
        }
    }
}

/// Update a route with middleware name
fn update_route_middleware(path: &str, middleware_name: &str) {
    let registry = REGISTERED_ROUTES.get_or_init(|| RwLock::new(Vec::new()));
    if let Ok(mut routes) = registry.write() {
        // Find the most recent route with this path and add middleware
        if let Some(route) = routes.iter_mut().rev().find(|r| r.path == path) {
            route.middleware.push(middleware_name.to_string());
        }
    }
}

/// Update a route with MCP metadata overrides
pub(crate) fn update_route_mcp(
    path: &str,
    tool_name: Option<String>,
    description: Option<String>,
    hint: Option<String>,
    hidden: bool,
) {
    let registry = REGISTERED_ROUTES.get_or_init(|| RwLock::new(Vec::new()));
    if let Ok(mut routes) = registry.write() {
        if let Some(route) = routes.iter_mut().rev().find(|r| r.path == path) {
            route.mcp_tool_name = tool_name;
            route.mcp_description = description;
            route.mcp_hint = hint;
            route.mcp_hidden = hidden;
        }
    }
}

/// Get all registered routes for introspection
pub fn get_registered_routes() -> Vec<RouteInfo> {
    REGISTERED_ROUTES
        .get()
        .and_then(|r| r.read().ok())
        .map(|routes| routes.clone())
        .unwrap_or_default()
}

/// Register a route name -> path mapping
pub fn register_route_name(name: &str, path: &str) {
    let registry = ROUTE_REGISTRY.get_or_init(|| RwLock::new(HashMap::new()));
    if let Ok(mut map) = registry.write() {
        map.insert(name.to_string(), path.to_string());
    }
    // Also update the introspection registry
    update_route_name(path, name);
}

/// Generate a URL for a named route with parameters
///
/// # Arguments
/// * `name` - The route name (e.g., "users.show")
/// * `params` - Slice of (key, value) tuples for path parameters
///
/// # Returns
/// * `Some(String)` - The generated URL with parameters substituted
/// * `None` - If the route name is not found
///
/// # Example
/// ```ignore
/// let url = route("users.show", &[("id", "123")]);
/// assert_eq!(url, Some("/users/123".to_string()));
/// ```
pub fn route(name: &str, params: &[(&str, &str)]) -> Option<String> {
    let registry = ROUTE_REGISTRY.get()?.read().ok()?;
    let path_pattern = registry.get(name)?;

    let mut url = path_pattern.clone();
    for (key, value) in params {
        url = url.replace(&format!("{{{key}}}"), value);
    }
    Some(url)
}

/// Generate URL with HashMap parameters (used internally by Redirect)
pub fn route_with_params(name: &str, params: &HashMap<String, String>) -> Option<String> {
    let registry = ROUTE_REGISTRY.get()?.read().ok()?;
    let path_pattern = registry.get(name)?;

    let mut url = path_pattern.clone();
    for (key, value) in params {
        url = url.replace(&format!("{{{key}}}"), value);
    }
    Some(url)
}

/// HTTP method for tracking the last registered route
#[derive(Clone, Copy)]
enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

/// Type alias for route handlers
pub type BoxedHandler =
    Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;

/// Value stored in the router: handler + pattern for metrics
type RouteValue = (Arc<BoxedHandler>, String);

/// HTTP Router with Laravel-like route registration
pub struct Router {
    get_routes: MatchitRouter<RouteValue>,
    post_routes: MatchitRouter<RouteValue>,
    put_routes: MatchitRouter<RouteValue>,
    patch_routes: MatchitRouter<RouteValue>,
    delete_routes: MatchitRouter<RouteValue>,
    /// Middleware assignments: path -> boxed middleware instances
    route_middleware: HashMap<String, Vec<BoxedMiddleware>>,
    /// Fallback handler for when no routes match (overrides default 404)
    fallback_handler: Option<Arc<BoxedHandler>>,
    /// Middleware for the fallback route
    fallback_middleware: Vec<BoxedMiddleware>,
}

impl Router {
    /// Create an empty router with no routes registered.
    pub fn new() -> Self {
        Self {
            get_routes: MatchitRouter::new(),
            post_routes: MatchitRouter::new(),
            put_routes: MatchitRouter::new(),
            patch_routes: MatchitRouter::new(),
            delete_routes: MatchitRouter::new(),
            route_middleware: HashMap::new(),
            fallback_handler: None,
            fallback_middleware: Vec::new(),
        }
    }

    /// Get middleware for a specific route path
    pub fn get_route_middleware(&self, path: &str) -> Vec<BoxedMiddleware> {
        self.route_middleware.get(path).cloned().unwrap_or_default()
    }

    /// Register middleware for a path (internal use)
    pub(crate) fn add_middleware(&mut self, path: &str, middleware: BoxedMiddleware) {
        self.route_middleware
            .entry(path.to_string())
            .or_default()
            .push(middleware);
    }

    /// Set the fallback handler for when no routes match
    pub(crate) fn set_fallback(&mut self, handler: Arc<BoxedHandler>) {
        self.fallback_handler = Some(handler);
    }

    /// Add middleware to the fallback route
    pub(crate) fn add_fallback_middleware(&mut self, middleware: BoxedMiddleware) {
        self.fallback_middleware.push(middleware);
    }

    /// Get the fallback handler and its middleware
    pub fn get_fallback(&self) -> Option<(Arc<BoxedHandler>, Vec<BoxedMiddleware>)> {
        self.fallback_handler
            .as_ref()
            .map(|h| (h.clone(), self.fallback_middleware.clone()))
    }

    /// Insert a GET route with a pre-boxed handler (internal use for groups)
    pub(crate) fn insert_get(&mut self, path: &str, handler: Arc<BoxedHandler>) {
        self.get_routes
            .insert(path, (handler, path.to_string()))
            .ok();
        register_route("GET", path);
    }

    /// Insert a POST route with a pre-boxed handler (internal use for groups)
    pub(crate) fn insert_post(&mut self, path: &str, handler: Arc<BoxedHandler>) {
        self.post_routes
            .insert(path, (handler, path.to_string()))
            .ok();
        register_route("POST", path);
    }

    /// Insert a PUT route with a pre-boxed handler (internal use for groups)
    pub(crate) fn insert_put(&mut self, path: &str, handler: Arc<BoxedHandler>) {
        self.put_routes
            .insert(path, (handler, path.to_string()))
            .ok();
        register_route("PUT", path);
    }

    /// Insert a PATCH route with a pre-boxed handler (internal use for groups)
    pub(crate) fn insert_patch(&mut self, path: &str, handler: Arc<BoxedHandler>) {
        self.patch_routes
            .insert(path, (handler, path.to_string()))
            .ok();
        register_route("PATCH", path);
    }

    /// Insert a DELETE route with a pre-boxed handler (internal use for groups)
    pub(crate) fn insert_delete(&mut self, path: &str, handler: Arc<BoxedHandler>) {
        self.delete_routes
            .insert(path, (handler, path.to_string()))
            .ok();
        register_route("DELETE", path);
    }

    /// Insert a GET route alias pointing at the same handler as a previously
    /// registered canonical route. Skips `register_route` so `RouteInfo` and
    /// `get_registered_routes()` stay canonical (D-07). The stored matchit
    /// value carries the CANONICAL pattern string so middleware lookup in
    /// `server.rs` (keyed by `route_pattern`) resolves to the canonical
    /// `add_middleware` key regardless of which variant matched.
    pub(crate) fn insert_get_alias(
        &mut self,
        alias_path: &str,
        handler: Arc<BoxedHandler>,
        canonical_path: &str,
    ) {
        self.get_routes
            .insert(alias_path, (handler, canonical_path.to_string()))
            .ok();
    }

    /// Insert a POST route alias pointing at the same handler as a previously
    /// registered canonical route. See `insert_get_alias` for the invariants.
    pub(crate) fn insert_post_alias(
        &mut self,
        alias_path: &str,
        handler: Arc<BoxedHandler>,
        canonical_path: &str,
    ) {
        self.post_routes
            .insert(alias_path, (handler, canonical_path.to_string()))
            .ok();
    }

    /// Insert a PUT route alias pointing at the same handler as a previously
    /// registered canonical route. See `insert_get_alias` for the invariants.
    pub(crate) fn insert_put_alias(
        &mut self,
        alias_path: &str,
        handler: Arc<BoxedHandler>,
        canonical_path: &str,
    ) {
        self.put_routes
            .insert(alias_path, (handler, canonical_path.to_string()))
            .ok();
    }

    /// Insert a PATCH route alias pointing at the same handler as a previously
    /// registered canonical route. See `insert_get_alias` for the invariants.
    pub(crate) fn insert_patch_alias(
        &mut self,
        alias_path: &str,
        handler: Arc<BoxedHandler>,
        canonical_path: &str,
    ) {
        self.patch_routes
            .insert(alias_path, (handler, canonical_path.to_string()))
            .ok();
    }

    /// Insert a DELETE route alias pointing at the same handler as a previously
    /// registered canonical route. See `insert_get_alias` for the invariants.
    pub(crate) fn insert_delete_alias(
        &mut self,
        alias_path: &str,
        handler: Arc<BoxedHandler>,
        canonical_path: &str,
    ) {
        self.delete_routes
            .insert(alias_path, (handler, canonical_path.to_string()))
            .ok();
    }

    /// Register a GET route
    pub fn get<H, Fut>(mut self, path: &str, handler: H) -> RouteBuilder
    where
        H: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let handler: BoxedHandler = Box::new(move |req| Box::pin(handler(req)));
        self.get_routes
            .insert(path, (Arc::new(handler), path.to_string()))
            .ok();
        register_route("GET", path);
        RouteBuilder {
            router: self,
            last_path: path.to_string(),
            _last_method: Method::Get,
        }
    }

    /// Register a POST route
    pub fn post<H, Fut>(mut self, path: &str, handler: H) -> RouteBuilder
    where
        H: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let handler: BoxedHandler = Box::new(move |req| Box::pin(handler(req)));
        self.post_routes
            .insert(path, (Arc::new(handler), path.to_string()))
            .ok();
        register_route("POST", path);
        RouteBuilder {
            router: self,
            last_path: path.to_string(),
            _last_method: Method::Post,
        }
    }

    /// Register a PUT route
    pub fn put<H, Fut>(mut self, path: &str, handler: H) -> RouteBuilder
    where
        H: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let handler: BoxedHandler = Box::new(move |req| Box::pin(handler(req)));
        self.put_routes
            .insert(path, (Arc::new(handler), path.to_string()))
            .ok();
        register_route("PUT", path);
        RouteBuilder {
            router: self,
            last_path: path.to_string(),
            _last_method: Method::Put,
        }
    }

    /// Register a PATCH route
    pub fn patch<H, Fut>(mut self, path: &str, handler: H) -> RouteBuilder
    where
        H: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let handler: BoxedHandler = Box::new(move |req| Box::pin(handler(req)));
        self.patch_routes
            .insert(path, (Arc::new(handler), path.to_string()))
            .ok();
        register_route("PATCH", path);
        RouteBuilder {
            router: self,
            last_path: path.to_string(),
            _last_method: Method::Patch,
        }
    }

    /// Register a DELETE route
    pub fn delete<H, Fut>(mut self, path: &str, handler: H) -> RouteBuilder
    where
        H: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let handler: BoxedHandler = Box::new(move |req| Box::pin(handler(req)));
        self.delete_routes
            .insert(path, (Arc::new(handler), path.to_string()))
            .ok();
        register_route("DELETE", path);
        RouteBuilder {
            router: self,
            last_path: path.to_string(),
            _last_method: Method::Delete,
        }
    }

    /// Match a request and return the handler with extracted params and route pattern
    ///
    /// Returns (handler, params, route_pattern) where route_pattern is the original
    /// pattern like "/users/{id}" for metrics grouping.
    ///
    /// OPTIONS requests are dispatched through `match_preflight` (private helper):
    /// any path registered under any other verb returns a synthetic 204 handler so
    /// route-level middleware (CORS in particular) still runs. The CORS middleware
    /// then short-circuits the preflight with the configured ACAO / ACAH / ACAM
    /// headers. Without this, OPTIONS would 404 before the middleware chain ran.
    pub fn match_route(
        &self,
        method: &hyper::Method,
        path: &str,
    ) -> Option<(Arc<BoxedHandler>, HashMap<String, String>, String)> {
        let router = match *method {
            hyper::Method::GET => &self.get_routes,
            hyper::Method::POST => &self.post_routes,
            hyper::Method::PUT => &self.put_routes,
            hyper::Method::PATCH => &self.patch_routes,
            hyper::Method::DELETE => &self.delete_routes,
            hyper::Method::OPTIONS => return self.match_preflight(path),
            _ => return None,
        };

        router.at(path).ok().map(|matched| {
            let params: HashMap<String, String> = matched
                .params
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            let (handler, pattern) = matched.value.clone();
            (handler, params, pattern)
        })
    }

    /// Synthesize a 204 handler for an OPTIONS preflight when any verb matches the path.
    ///
    /// Scans every method table for the path; the first match wins. The returned
    /// pattern is the canonical pattern that match_route would have returned for the
    /// matching verb, so server.rs resolves the same route-level middleware (CORS,
    /// auth, etc.) as the live verb would. The synthetic handler returns 204 No
    /// Content with an empty body — when CORS middleware sits in the chain it
    /// short-circuits before the handler runs and applies its preflight headers.
    fn match_preflight(
        &self,
        path: &str,
    ) -> Option<(Arc<BoxedHandler>, HashMap<String, String>, String)> {
        let tables = [
            &self.get_routes,
            &self.post_routes,
            &self.put_routes,
            &self.patch_routes,
            &self.delete_routes,
        ];
        for table in tables {
            if let Ok(matched) = table.at(path) {
                let params: HashMap<String, String> = matched
                    .params
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();
                let (_, pattern) = matched.value.clone();
                let handler: BoxedHandler = Box::new(|_req| {
                    Box::pin(async move { Ok(crate::http::HttpResponse::new().status(204)) })
                });
                return Some((Arc::new(handler), params, pattern));
            }
        }
        None
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder returned after registering a route, enabling .name() chaining
pub struct RouteBuilder {
    pub(crate) router: Router,
    last_path: String,
    #[allow(dead_code)]
    _last_method: Method,
}

impl RouteBuilder {
    /// Name the most recently registered route
    pub fn name(self, name: &str) -> Router {
        register_route_name(name, &self.last_path);
        self.router
    }

    /// Apply middleware to the most recently registered route
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// Router::new()
    ///     .get("/admin", admin_handler).middleware(AuthMiddleware)
    ///     .get("/api/users", users_handler).middleware(CorsMiddleware)
    /// ```
    pub fn middleware<M: Middleware + 'static>(mut self, middleware: M) -> RouteBuilder {
        // Track middleware name for introspection
        let type_name = std::any::type_name::<M>();
        let short_name = type_name.rsplit("::").next().unwrap_or(type_name);
        update_route_middleware(&self.last_path, short_name);

        self.router
            .add_middleware(&self.last_path, into_boxed(middleware));
        self
    }

    /// Apply pre-boxed middleware to the most recently registered route
    /// (Used internally by route macros)
    pub fn middleware_boxed(mut self, middleware: BoxedMiddleware) -> RouteBuilder {
        // Track middleware (name not available for boxed middleware)
        update_route_middleware(&self.last_path, "BoxedMiddleware");

        self.router
            .route_middleware
            .entry(self.last_path.clone())
            .or_default()
            .push(middleware);
        self
    }

    /// Register a GET route (for chaining without .name())
    pub fn get<H, Fut>(self, path: &str, handler: H) -> RouteBuilder
    where
        H: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.router.get(path, handler)
    }

    /// Register a POST route (for chaining without .name())
    pub fn post<H, Fut>(self, path: &str, handler: H) -> RouteBuilder
    where
        H: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.router.post(path, handler)
    }

    /// Register a PUT route (for chaining without .name())
    pub fn put<H, Fut>(self, path: &str, handler: H) -> RouteBuilder
    where
        H: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.router.put(path, handler)
    }

    /// Register a PATCH route (for chaining without .name())
    pub fn patch<H, Fut>(self, path: &str, handler: H) -> RouteBuilder
    where
        H: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.router.patch(path, handler)
    }

    /// Register a DELETE route (for chaining without .name())
    pub fn delete<H, Fut>(self, path: &str, handler: H) -> RouteBuilder
    where
        H: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.router.delete(path, handler)
    }
}

impl From<RouteBuilder> for Router {
    fn from(builder: RouteBuilder) -> Self {
        builder.router
    }
}
