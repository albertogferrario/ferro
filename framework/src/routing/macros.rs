//! Route definition macros and helpers for Laravel-like routing syntax
//!
//! This module provides a clean, declarative way to define routes:
//!
//! ```rust,ignore
//! use ferro_rs::{routes, get, post, put, delete, group};
//!
//! routes! {
//!     get!("/", controllers::home::index).name("home"),
//!     get!("/users", controllers::user::index).name("users.index"),
//!     post!("/users", controllers::user::store).name("users.store"),
//!     get!("/protected", controllers::home::index).middleware(AuthMiddleware),
//!
//!     // Route groups with prefix and middleware
//!     group!("/api", {
//!         get!("/users", controllers::api::user::index).name("api.users.index"),
//!         post!("/users", controllers::api::user::store).name("api.users.store"),
//!     }).middleware(AuthMiddleware),
//! }
//! ```

use crate::http::{Request, Response};

/// Const function to validate route paths start with '/'
///
/// This provides compile-time validation that all route paths begin with '/'.
/// If a path doesn't start with '/', compilation will fail with a clear error.
///
/// # Panics
///
/// Panics at compile time if the path is empty or doesn't start with '/'.
pub const fn validate_route_path(path: &'static str) -> &'static str {
    let bytes = path.as_bytes();
    if bytes.is_empty() || bytes[0] != b'/' {
        panic!("Route path must start with '/'")
    }
    path
}
use super::path::combine_group_path;
use super::router::update_route_mcp;
use crate::middleware::{into_boxed, BoxedMiddleware, Middleware};
use crate::routing::router::{register_route_name, BoxedHandler, Router};
use std::future::Future;
use std::sync::Arc;

/// Convert Express-style `:param` route parameters to matchit-style `{param}`
///
/// This allows developers to use either syntax:
/// - `/:id` (Express/Rails style)
/// - `/{id}` (matchit native style)
///
/// # Examples
///
/// - `/users/:id` → `/users/{id}`
/// - `/posts/:post_id/comments/:id` → `/posts/{post_id}/comments/{id}`
/// - `/users/{id}` → `/users/{id}` (already correct syntax, unchanged)
fn convert_route_params(path: &str) -> String {
    let mut result = String::with_capacity(path.len() + 4); // Extra space for braces
    let mut chars = path.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == ':' {
            // Start of parameter - collect until '/' or end
            result.push('{');
            while let Some(&next) = chars.peek() {
                if next == '/' {
                    break;
                }
                result.push(chars.next().unwrap());
            }
            result.push('}');
        } else {
            result.push(ch);
        }
    }
    result
}

/// HTTP method for route definitions
#[derive(Clone, Copy)]
pub enum HttpMethod {
    /// HTTP GET method.
    Get,
    /// HTTP POST method.
    Post,
    /// HTTP PUT method.
    Put,
    /// HTTP PATCH method.
    Patch,
    /// HTTP DELETE method.
    Delete,
}

/// Builder for route definitions that supports `.name()` and `.middleware()` chaining
pub struct RouteDefBuilder<H> {
    method: HttpMethod,
    path: &'static str,
    handler: H,
    name: Option<&'static str>,
    middlewares: Vec<BoxedMiddleware>,
    mcp_tool_name: Option<String>,
    mcp_description: Option<String>,
    mcp_hint: Option<String>,
    mcp_hidden: bool,
}

impl<H, Fut> RouteDefBuilder<H>
where
    H: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    /// Create a new route definition builder
    pub fn new(method: HttpMethod, path: &'static str, handler: H) -> Self {
        Self {
            method,
            path,
            handler,
            name: None,
            middlewares: Vec::new(),
            mcp_tool_name: None,
            mcp_description: None,
            mcp_hint: None,
            mcp_hidden: false,
        }
    }

    /// Name this route for URL generation
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = Some(name);
        self
    }

    /// Add middleware to this route
    pub fn middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middlewares.push(into_boxed(middleware));
        self
    }

    /// Override the auto-generated MCP tool name for this route
    pub fn mcp_tool_name(mut self, name: &str) -> Self {
        self.mcp_tool_name = Some(name.to_string());
        self
    }

    /// Override the auto-generated MCP description for this route
    pub fn mcp_description(mut self, desc: &str) -> Self {
        self.mcp_description = Some(desc.to_string());
        self
    }

    /// Add a hint that is appended to the MCP description for AI guidance
    pub fn mcp_hint(mut self, hint: &str) -> Self {
        self.mcp_hint = Some(hint.to_string());
        self
    }

    /// Hide this route from MCP tool discovery
    pub fn mcp_hidden(mut self) -> Self {
        self.mcp_hidden = true;
        self
    }

    /// Register this route definition with a router
    pub fn register(self, router: Router) -> Router {
        // Convert :param to {param} for matchit compatibility
        let converted_path = convert_route_params(self.path);

        // Capture MCP fields before moving self
        let has_mcp = self.mcp_tool_name.is_some()
            || self.mcp_description.is_some()
            || self.mcp_hint.is_some()
            || self.mcp_hidden;
        let mcp_tool_name = self.mcp_tool_name;
        let mcp_description = self.mcp_description;
        let mcp_hint = self.mcp_hint;
        let mcp_hidden = self.mcp_hidden;

        // First, register the route based on method
        let builder = match self.method {
            HttpMethod::Get => router.get(&converted_path, self.handler),
            HttpMethod::Post => router.post(&converted_path, self.handler),
            HttpMethod::Put => router.put(&converted_path, self.handler),
            HttpMethod::Patch => router.patch(&converted_path, self.handler),
            HttpMethod::Delete => router.delete(&converted_path, self.handler),
        };

        // Apply any middleware
        let builder = self
            .middlewares
            .into_iter()
            .fold(builder, |b, m| b.middleware_boxed(m));

        // Apply MCP metadata if any fields are set
        if has_mcp {
            update_route_mcp(
                &converted_path,
                mcp_tool_name,
                mcp_description,
                mcp_hint,
                mcp_hidden,
            );
        }

        // Apply name if present, otherwise convert to Router
        if let Some(name) = self.name {
            builder.name(name)
        } else {
            builder.into()
        }
    }
}

/// Create a GET route definition with compile-time path validation
///
/// # Example
/// ```rust,ignore
/// get!("/users", controllers::user::index).name("users.index")
/// ```
///
/// # Compile Error
///
/// Fails to compile if path doesn't start with '/'.
#[macro_export]
macro_rules! get {
    ($path:expr, $handler:expr) => {{
        const _: &str = $crate::validate_route_path($path);
        $crate::__get_impl($path, $handler)
    }};
}

/// Internal implementation for GET routes (used by the get! macro)
#[doc(hidden)]
pub fn __get_impl<H, Fut>(path: &'static str, handler: H) -> RouteDefBuilder<H>
where
    H: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    RouteDefBuilder::new(HttpMethod::Get, path, handler)
}

/// Create a POST route definition with compile-time path validation
///
/// # Example
/// ```rust,ignore
/// post!("/users", controllers::user::store).name("users.store")
/// ```
///
/// # Compile Error
///
/// Fails to compile if path doesn't start with '/'.
#[macro_export]
macro_rules! post {
    ($path:expr, $handler:expr) => {{
        const _: &str = $crate::validate_route_path($path);
        $crate::__post_impl($path, $handler)
    }};
}

/// Internal implementation for POST routes (used by the post! macro)
#[doc(hidden)]
pub fn __post_impl<H, Fut>(path: &'static str, handler: H) -> RouteDefBuilder<H>
where
    H: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    RouteDefBuilder::new(HttpMethod::Post, path, handler)
}

/// Create a PUT route definition with compile-time path validation
///
/// # Example
/// ```rust,ignore
/// put!("/users/{id}", controllers::user::update).name("users.update")
/// ```
///
/// # Compile Error
///
/// Fails to compile if path doesn't start with '/'.
#[macro_export]
macro_rules! put {
    ($path:expr, $handler:expr) => {{
        const _: &str = $crate::validate_route_path($path);
        $crate::__put_impl($path, $handler)
    }};
}

/// Internal implementation for PUT routes (used by the put! macro)
#[doc(hidden)]
pub fn __put_impl<H, Fut>(path: &'static str, handler: H) -> RouteDefBuilder<H>
where
    H: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    RouteDefBuilder::new(HttpMethod::Put, path, handler)
}

/// Create a PATCH route definition with compile-time path validation
///
/// # Example
/// ```rust,ignore
/// patch!("/users/{id}", controllers::user::patch).name("users.patch")
/// ```
///
/// # Compile Error
///
/// Fails to compile if path doesn't start with '/'.
#[macro_export]
macro_rules! patch {
    ($path:expr, $handler:expr) => {{
        const _: &str = $crate::validate_route_path($path);
        $crate::__patch_impl($path, $handler)
    }};
}

/// Internal implementation for PATCH routes (used by the patch! macro)
#[doc(hidden)]
pub fn __patch_impl<H, Fut>(path: &'static str, handler: H) -> RouteDefBuilder<H>
where
    H: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    RouteDefBuilder::new(HttpMethod::Patch, path, handler)
}

/// Create a DELETE route definition with compile-time path validation
///
/// # Example
/// ```rust,ignore
/// delete!("/users/{id}", controllers::user::destroy).name("users.destroy")
/// ```
///
/// # Compile Error
///
/// Fails to compile if path doesn't start with '/'.
#[macro_export]
macro_rules! delete {
    ($path:expr, $handler:expr) => {{
        const _: &str = $crate::validate_route_path($path);
        $crate::__delete_impl($path, $handler)
    }};
}

/// Internal implementation for DELETE routes (used by the delete! macro)
#[doc(hidden)]
pub fn __delete_impl<H, Fut>(path: &'static str, handler: H) -> RouteDefBuilder<H>
where
    H: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    RouteDefBuilder::new(HttpMethod::Delete, path, handler)
}

// ============================================================================
// Fallback Route Support
// ============================================================================

/// Builder for fallback route definitions that supports `.middleware()` chaining
///
/// The fallback route is invoked when no other routes match, allowing custom
/// handling of 404 scenarios.
pub struct FallbackDefBuilder<H> {
    handler: H,
    middlewares: Vec<BoxedMiddleware>,
}

impl<H, Fut> FallbackDefBuilder<H>
where
    H: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    /// Create a new fallback definition builder
    pub fn new(handler: H) -> Self {
        Self {
            handler,
            middlewares: Vec::new(),
        }
    }

    /// Add middleware to this fallback route
    pub fn middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middlewares.push(into_boxed(middleware));
        self
    }

    /// Register this fallback definition with a router
    pub fn register(self, mut router: Router) -> Router {
        let handler = self.handler;
        let boxed: BoxedHandler = Box::new(move |req| Box::pin(handler(req)));
        router.set_fallback(Arc::new(boxed));

        // Apply middleware
        for mw in self.middlewares {
            router.add_fallback_middleware(mw);
        }

        router
    }
}

/// Create a fallback route definition
///
/// The fallback handler is called when no other routes match the request,
/// allowing you to override the default 404 behavior.
///
/// # Example
/// ```rust,ignore
/// routes! {
///     get!("/", controllers::home::index),
///     get!("/users", controllers::user::index),
///
///     // Custom 404 handler
///     fallback!(controllers::fallback::invoke),
/// }
/// ```
///
/// With middleware:
/// ```rust,ignore
/// routes! {
///     get!("/", controllers::home::index),
///     fallback!(controllers::fallback::invoke).middleware(LoggingMiddleware),
/// }
/// ```
#[macro_export]
macro_rules! fallback {
    ($handler:expr) => {{
        $crate::__fallback_impl($handler)
    }};
}

/// Internal implementation for fallback routes (used by the fallback! macro)
#[doc(hidden)]
pub fn __fallback_impl<H, Fut>(handler: H) -> FallbackDefBuilder<H>
where
    H: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    FallbackDefBuilder::new(handler)
}

// ============================================================================
// Route Grouping Support
// ============================================================================

/// A route stored within a group (type-erased handler)
pub struct GroupRoute {
    method: HttpMethod,
    path: &'static str,
    handler: Arc<BoxedHandler>,
    name: Option<&'static str>,
    middlewares: Vec<BoxedMiddleware>,
    mcp_tool_name: Option<String>,
    mcp_description: Option<String>,
    mcp_hint: Option<String>,
    mcp_hidden: bool,
}

/// An item that can be added to a route group - either a route or a nested group
pub enum GroupItem {
    /// A single route
    Route(GroupRoute),
    /// A nested group with its own prefix and middleware
    NestedGroup(Box<GroupDef>),
}

/// Trait for types that can be converted into a GroupItem
pub trait IntoGroupItem {
    /// Convert this value into a group item for the route builder.
    fn into_group_item(self) -> GroupItem;
}

/// MCP defaults inherited from parent groups to child routes
#[derive(Default, Clone)]
struct McpDefaults {
    tool_name: Option<String>,
    description: Option<String>,
    hint: Option<String>,
    hidden: bool,
}

/// Group definition that collects routes and applies prefix/middleware
///
/// Supports nested groups for arbitrary route organization:
///
/// ```rust,ignore
/// routes! {
///     group!("/api", {
///         get!("/users", controllers::user::index).name("api.users"),
///         post!("/users", controllers::user::store),
///         // Nested groups are supported
///         group!("/admin", {
///             get!("/dashboard", controllers::admin::dashboard),
///         }),
///     }).middleware(AuthMiddleware),
/// }
/// ```
pub struct GroupDef {
    prefix: &'static str,
    items: Vec<GroupItem>,
    group_middlewares: Vec<BoxedMiddleware>,
    mcp_tool_name: Option<String>,
    mcp_description: Option<String>,
    mcp_hint: Option<String>,
    mcp_hidden: bool,
}

impl GroupDef {
    /// Create a new route group with the given prefix (internal use)
    ///
    /// Use the `group!` macro instead for compile-time validation.
    #[doc(hidden)]
    pub fn __new_unchecked(prefix: &'static str) -> Self {
        Self {
            prefix,
            items: Vec::new(),
            group_middlewares: Vec::new(),
            mcp_tool_name: None,
            mcp_description: None,
            mcp_hint: None,
            mcp_hidden: false,
        }
    }

    /// Add an item (route or nested group) to this group
    ///
    /// This is the primary method for adding items to a group. It accepts
    /// anything that implements `IntoGroupItem`, including routes created
    /// with `get!`, `post!`, etc., and nested groups created with `group!`.
    #[allow(clippy::should_implement_trait)]
    pub fn add<I: IntoGroupItem>(mut self, item: I) -> Self {
        self.items.push(item.into_group_item());
        self
    }

    /// Add a route to this group (backward compatibility)
    ///
    /// Prefer using `.add()` which accepts both routes and nested groups.
    pub fn route<H, Fut>(self, route: RouteDefBuilder<H>) -> Self
    where
        H: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.add(route)
    }

    /// Add middleware to all routes in this group
    ///
    /// Middleware is applied in the order it's added.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// group!("/api", {
    ///     get!("/users", handler),
    /// }).middleware(AuthMiddleware).middleware(RateLimitMiddleware)
    /// ```
    pub fn middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.group_middlewares.push(into_boxed(middleware));
        self
    }

    /// Set a default MCP tool name for all routes in this group
    ///
    /// Route-level overrides take precedence.
    pub fn mcp_tool_name(mut self, name: &str) -> Self {
        self.mcp_tool_name = Some(name.to_string());
        self
    }

    /// Set a default MCP description for all routes in this group
    ///
    /// Route-level overrides take precedence.
    pub fn mcp_description(mut self, desc: &str) -> Self {
        self.mcp_description = Some(desc.to_string());
        self
    }

    /// Set a default MCP hint for all routes in this group
    ///
    /// Route-level overrides take precedence.
    pub fn mcp_hint(mut self, hint: &str) -> Self {
        self.mcp_hint = Some(hint.to_string());
        self
    }

    /// Hide all routes in this group from MCP tool discovery
    ///
    /// Route-level overrides take precedence.
    pub fn mcp_hidden(mut self) -> Self {
        self.mcp_hidden = true;
        self
    }

    /// Register all routes in this group with the router
    ///
    /// This prepends the group prefix to each route path and applies
    /// group middleware to all routes. Nested groups are flattened recursively,
    /// with prefixes concatenated and middleware inherited from parent groups.
    ///
    /// # Path Combination
    ///
    /// - If route path is "/" (root), the full path is just the group prefix
    /// - Otherwise, prefix and route path are concatenated
    ///
    /// # Middleware Inheritance
    ///
    /// Parent group middleware is applied before child group middleware,
    /// which is applied before route-specific middleware.
    pub fn register(self, mut router: Router) -> Router {
        let mcp_defaults = McpDefaults::default();
        self.register_with_inherited(&mut router, "", &[], &mcp_defaults);
        router
    }

    /// Internal recursive registration with inherited prefix and middleware
    fn register_with_inherited(
        self,
        router: &mut Router,
        parent_prefix: &str,
        inherited_middleware: &[BoxedMiddleware],
        inherited_mcp: &McpDefaults,
    ) {
        // Build the full prefix for this group. Strip a trailing `/` from the
        // parent prefix before concatenation so nested groups like
        // `group!("/a/", { group!("/b", ...) })` accumulate to `/a/b` instead
        // of `/a//b` (D-06, RESEARCH §Pitfall 3).
        let full_prefix = if parent_prefix.is_empty() {
            self.prefix.to_string()
        } else {
            let stripped_parent = parent_prefix.strip_suffix('/').unwrap_or(parent_prefix);
            format!("{}{}", stripped_parent, self.prefix)
        };

        // Combine inherited middleware with this group's middleware
        // Parent middleware runs first (outer), then this group's middleware
        let combined_middleware: Vec<BoxedMiddleware> = inherited_middleware
            .iter()
            .cloned()
            .chain(self.group_middlewares.iter().cloned())
            .collect();

        // Merge MCP defaults: this group's settings override inherited
        let combined_mcp = McpDefaults {
            tool_name: self.mcp_tool_name.or(inherited_mcp.tool_name.clone()),
            description: self.mcp_description.or(inherited_mcp.description.clone()),
            hint: self.mcp_hint.or(inherited_mcp.hint.clone()),
            hidden: self.mcp_hidden || inherited_mcp.hidden,
        };

        for item in self.items {
            match item {
                GroupItem::Route(route) => {
                    // Convert :param to {param} for matchit compatibility
                    let converted_route_path = convert_route_params(route.path);

                    // Combine the full prefix with the route path using the
                    // shared helper. May emit a second matchit leaf so
                    // `get!("/", ...)` inside a non-root group reaches its
                    // handler at both `/prefix` and `/prefix/` (D-01 through
                    // D-04, D-06).
                    let (canonical, alternate) =
                        combine_group_path(&full_prefix, &converted_route_path);
                    let canonical_path: &'static str =
                        Box::leak(canonical.into_boxed_str());
                    let alternate_path: Option<&'static str> = alternate
                        .map(|s| Box::leak(s.into_boxed_str()) as &'static str);

                    // Register the canonical route first (touches
                    // `register_route`, which appends to `REGISTERED_ROUTES`
                    // so `update_route_name` / `update_route_mcp` /
                    // `update_route_middleware` later in this block resolve
                    // via `.rev().find(path == canonical)`). Then register
                    // the alternate as a matchit-only alias carrying the
                    // canonical pattern string so middleware lookup resolves
                    // under the canonical key (Strategy A).
                    match route.method {
                        HttpMethod::Get => {
                            router.insert_get(canonical_path, route.handler.clone());
                            if let Some(alt) = alternate_path {
                                router.insert_get_alias(alt, route.handler, canonical_path);
                            }
                        }
                        HttpMethod::Post => {
                            router.insert_post(canonical_path, route.handler.clone());
                            if let Some(alt) = alternate_path {
                                router.insert_post_alias(alt, route.handler, canonical_path);
                            }
                        }
                        HttpMethod::Put => {
                            router.insert_put(canonical_path, route.handler.clone());
                            if let Some(alt) = alternate_path {
                                router.insert_put_alias(alt, route.handler, canonical_path);
                            }
                        }
                        HttpMethod::Patch => {
                            router.insert_patch(canonical_path, route.handler.clone());
                            if let Some(alt) = alternate_path {
                                router.insert_patch_alias(alt, route.handler, canonical_path);
                            }
                        }
                        HttpMethod::Delete => {
                            router.insert_delete(canonical_path, route.handler.clone());
                            if let Some(alt) = alternate_path {
                                router.insert_delete_alias(alt, route.handler, canonical_path);
                            }
                        }
                    }

                    // Register route name if present
                    if let Some(name) = route.name {
                        register_route_name(name, canonical_path);
                    }

                    // Apply MCP metadata: route-level overrides group defaults
                    let mcp_tool_name = route.mcp_tool_name.or(combined_mcp.tool_name.clone());
                    let mcp_description =
                        route.mcp_description.or(combined_mcp.description.clone());
                    let mcp_hint = route.mcp_hint.or(combined_mcp.hint.clone());
                    let mcp_hidden = route.mcp_hidden || combined_mcp.hidden;

                    if mcp_tool_name.is_some()
                        || mcp_description.is_some()
                        || mcp_hint.is_some()
                        || mcp_hidden
                    {
                        update_route_mcp(
                            canonical_path,
                            mcp_tool_name,
                            mcp_description,
                            mcp_hint,
                            mcp_hidden,
                        );
                    }

                    // Apply combined middleware (inherited + group), then route-specific.
                    // The alias leaf stores the canonical pattern so middleware registered
                    // here under canonical_path is reached for both URL variants (Strategy A).
                    for mw in &combined_middleware {
                        router.add_middleware(canonical_path, mw.clone());
                    }
                    for mw in route.middlewares {
                        router.add_middleware(canonical_path, mw);
                    }
                }
                GroupItem::NestedGroup(nested) => {
                    // Recursively register the nested group with accumulated prefix and middleware
                    nested.register_with_inherited(
                        router,
                        &full_prefix,
                        &combined_middleware,
                        &combined_mcp,
                    );
                }
            }
        }
    }
}

impl<H, Fut> RouteDefBuilder<H>
where
    H: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    /// Convert this route definition to a type-erased GroupRoute
    ///
    /// This is used internally when adding routes to a group.
    pub fn into_group_route(self) -> GroupRoute {
        let handler = self.handler;
        let boxed: BoxedHandler = Box::new(move |req| Box::pin(handler(req)));
        GroupRoute {
            method: self.method,
            path: self.path,
            handler: Arc::new(boxed),
            name: self.name,
            middlewares: self.middlewares,
            mcp_tool_name: self.mcp_tool_name,
            mcp_description: self.mcp_description,
            mcp_hint: self.mcp_hint,
            mcp_hidden: self.mcp_hidden,
        }
    }
}

// ============================================================================
// IntoGroupItem implementations
// ============================================================================

impl<H, Fut> IntoGroupItem for RouteDefBuilder<H>
where
    H: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    fn into_group_item(self) -> GroupItem {
        GroupItem::Route(self.into_group_route())
    }
}

impl IntoGroupItem for GroupDef {
    fn into_group_item(self) -> GroupItem {
        GroupItem::NestedGroup(Box::new(self))
    }
}

/// Define a route group with a shared prefix
///
/// Routes within a group will have the prefix prepended to their paths.
/// Middleware can be applied to the entire group using `.middleware()`.
/// Groups can be nested arbitrarily deep.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::{routes, get, post, group};
///
/// routes! {
///     get!("/", controllers::home::index),
///
///     // All routes in this group start with /api
///     group!("/api", {
///         get!("/users", controllers::user::index),      // -> GET /api/users
///         post!("/users", controllers::user::store),     // -> POST /api/users
///
///         // Nested groups are supported
///         group!("/admin", {
///             get!("/dashboard", controllers::admin::dashboard), // -> GET /api/admin/dashboard
///         }),
///     }).middleware(AuthMiddleware),  // Applies to ALL routes including nested
/// }
/// ```
///
/// # Middleware Inheritance
///
/// Middleware applied to a parent group is automatically inherited by all nested groups.
/// The execution order is: parent middleware -> child middleware -> route middleware.
///
/// # Compile Error
///
/// Fails to compile if prefix doesn't start with '/'.
#[macro_export]
macro_rules! group {
    ($prefix:expr, { $( $item:expr ),* $(,)? }) => {{
        const _: &str = $crate::validate_route_path($prefix);
        let mut group = $crate::GroupDef::__new_unchecked($prefix);
        $(
            group = group.add($item);
        )*
        group
    }};
}

/// Define routes with a clean, Laravel-like syntax
///
/// This macro generates a `pub fn register() -> Router` function automatically.
/// Place it at the top level of your `routes.rs` file.
///
/// # Example
/// ```rust,ignore
/// use ferro_rs::{routes, get, post, put, delete};
/// use ferro_rs::controllers;
/// use ferro_rs::middleware::AuthMiddleware;
///
/// routes! {
///     get!("/", controllers::home::index).name("home"),
///     get!("/users", controllers::user::index).name("users.index"),
///     get!("/users/{id}", controllers::user::show).name("users.show"),
///     post!("/users", controllers::user::store).name("users.store"),
///     put!("/users/{id}", controllers::user::update).name("users.update"),
///     delete!("/users/{id}", controllers::user::destroy).name("users.destroy"),
///     get!("/protected", controllers::home::index).middleware(AuthMiddleware),
/// }
/// ```
#[macro_export]
macro_rules! routes {
    ( $( $route:expr ),* $(,)? ) => {
        pub fn register() -> $crate::Router {
            let mut router = $crate::Router::new();
            $(
                router = $route.register(router);
            )*
            router
        }
    };
}

// ============================================================================
// RESTful Resource Routing Support
// ============================================================================

/// Actions available for resource routing
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResourceAction {
    /// List all resources (GET /resources).
    Index,
    /// Show creation form (GET /resources/create).
    Create,
    /// Persist a new resource (POST /resources).
    Store,
    /// Display a single resource (GET /resources/{id}).
    Show,
    /// Show edit form (GET /resources/{id}/edit).
    Edit,
    /// Update an existing resource (PUT /resources/{id}).
    Update,
    /// Delete a resource (DELETE /resources/{id}).
    Destroy,
}

impl ResourceAction {
    /// Get all available resource actions
    pub const fn all() -> &'static [ResourceAction] {
        &[
            ResourceAction::Index,
            ResourceAction::Create,
            ResourceAction::Store,
            ResourceAction::Show,
            ResourceAction::Edit,
            ResourceAction::Update,
            ResourceAction::Destroy,
        ]
    }

    /// Get the HTTP method for this action
    pub const fn method(&self) -> HttpMethod {
        match self {
            ResourceAction::Index => HttpMethod::Get,
            ResourceAction::Create => HttpMethod::Get,
            ResourceAction::Store => HttpMethod::Post,
            ResourceAction::Show => HttpMethod::Get,
            ResourceAction::Edit => HttpMethod::Get,
            ResourceAction::Update => HttpMethod::Put,
            ResourceAction::Destroy => HttpMethod::Delete,
        }
    }

    /// Get the path suffix for this action (relative to resource path)
    pub const fn path_suffix(&self) -> &'static str {
        match self {
            ResourceAction::Index => "/",
            ResourceAction::Create => "/create",
            ResourceAction::Store => "/",
            ResourceAction::Show => "/{id}",
            ResourceAction::Edit => "/{id}/edit",
            ResourceAction::Update => "/{id}",
            ResourceAction::Destroy => "/{id}",
        }
    }

    /// Get the route name suffix for this action
    pub const fn name_suffix(&self) -> &'static str {
        match self {
            ResourceAction::Index => "index",
            ResourceAction::Create => "create",
            ResourceAction::Store => "store",
            ResourceAction::Show => "show",
            ResourceAction::Edit => "edit",
            ResourceAction::Update => "update",
            ResourceAction::Destroy => "destroy",
        }
    }
}

/// A resource route stored within a ResourceDef (type-erased handler)
pub struct ResourceRoute {
    action: ResourceAction,
    handler: Arc<BoxedHandler>,
}

/// Resource definition that generates RESTful routes from a controller module
///
/// Generates 7 standard routes following Rails/Laravel conventions:
///
/// - GET    /users          -> index   (list all)
/// - GET    /users/create   -> create  (show create form)
/// - POST   /users          -> store   (create new)
/// - GET    /users/{id}     -> show    (show one)
/// - GET    /users/{id}/edit -> edit   (show edit form)
/// - PUT    /users/{id}     -> update  (update one)
/// - DELETE /users/{id}     -> destroy (delete one)
///
/// Route names are auto-generated: users.index, users.create, etc.
///
/// # Example
///
/// ```rust,ignore
/// routes! {
///     resource!("/users", controllers::user),
///     resource!("/posts", controllers::post).middleware(AuthMiddleware),
///     resource!("/comments", controllers::comment, only: [index, show]),
/// }
/// ```
pub struct ResourceDef {
    prefix: &'static str,
    routes: Vec<ResourceRoute>,
    middlewares: Vec<BoxedMiddleware>,
}

impl ResourceDef {
    /// Create a new resource definition with no routes (internal use)
    #[doc(hidden)]
    pub fn __new_unchecked(prefix: &'static str) -> Self {
        Self {
            prefix,
            routes: Vec::new(),
            middlewares: Vec::new(),
        }
    }

    /// Add a route for a specific action
    #[doc(hidden)]
    pub fn __add_route(mut self, action: ResourceAction, handler: Arc<BoxedHandler>) -> Self {
        self.routes.push(ResourceRoute { action, handler });
        self
    }

    /// Add middleware to all routes in this resource
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// resource!("/admin", controllers::admin, only: [index, show])
    ///     .middleware(AuthMiddleware)
    ///     .middleware(AdminMiddleware)
    /// ```
    pub fn middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middlewares.push(into_boxed(middleware));
        self
    }

    /// Register all resource routes with the router
    pub fn register(self, mut router: Router) -> Router {
        // Derive route name prefix from path: "/users" -> "users", "/api/users" -> "api.users"
        let name_prefix = self.prefix.trim_start_matches('/').replace('/', ".");

        for route in self.routes {
            let action = route.action;
            let path_suffix = action.path_suffix();

            // Build full path
            let full_path = if path_suffix == "/" {
                self.prefix.to_string()
            } else {
                format!("{}{}", self.prefix, path_suffix)
            };
            let full_path: &'static str = Box::leak(full_path.into_boxed_str());

            // Build route name
            let route_name = format!("{}.{}", name_prefix, action.name_suffix());
            let route_name: &'static str = Box::leak(route_name.into_boxed_str());

            // Register the route
            match action.method() {
                HttpMethod::Get => {
                    router.insert_get(full_path, route.handler);
                }
                HttpMethod::Post => {
                    router.insert_post(full_path, route.handler);
                }
                HttpMethod::Put => {
                    router.insert_put(full_path, route.handler);
                }
                HttpMethod::Patch => {
                    router.insert_patch(full_path, route.handler);
                }
                HttpMethod::Delete => {
                    router.insert_delete(full_path, route.handler);
                }
            }

            // Register route name
            register_route_name(route_name, full_path);

            // Apply middleware
            for mw in &self.middlewares {
                router.add_middleware(full_path, mw.clone());
            }
        }

        router
    }
}

/// Helper function to create a boxed handler from a handler function
#[doc(hidden)]
pub fn __box_handler<H, Fut>(handler: H) -> Arc<BoxedHandler>
where
    H: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    let boxed: BoxedHandler = Box::new(move |req| Box::pin(handler(req)));
    Arc::new(boxed)
}

/// Define RESTful resource routes with convention-over-configuration
///
/// Generates 7 standard routes following Rails/Laravel conventions from a
/// controller module reference.
///
/// # Convention Mapping
///
/// | Method | Path            | Action  | Route Name    |
/// |--------|-----------------|---------|---------------|
/// | GET    | /users          | index   | users.index   |
/// | GET    | /users/create   | create  | users.create  |
/// | POST   | /users          | store   | users.store   |
/// | GET    | /users/{id}     | show    | users.show    |
/// | GET    | /users/{id}/edit| edit    | users.edit    |
/// | PUT    | /users/{id}     | update  | users.update  |
/// | DELETE | /users/{id}     | destroy | users.destroy |
///
/// # Basic Usage
///
/// ```rust,ignore
/// routes! {
///     resource!("/users", controllers::user),
/// }
/// ```
///
/// # With Middleware
///
/// ```rust,ignore
/// routes! {
///     resource!("/admin", controllers::admin).middleware(AuthMiddleware),
/// }
/// ```
///
/// # Subset of Actions
///
/// Use `only:` to generate only specific routes:
///
/// ```rust,ignore
/// routes! {
///     // Only index, show, and store - no create/edit forms, update, or destroy
///     resource!("/posts", controllers::post, only: [index, show, store]),
/// }
/// ```
///
/// # Path Naming
///
/// Route names are derived from the path:
/// - `/users` → `users.index`, `users.show`, etc.
/// - `/api/users` → `api.users.index`, `api.users.show`, etc.
///
/// # Compile Error
///
/// Fails to compile if path doesn't start with '/'.
#[macro_export]
macro_rules! resource {
    // Full resource (all 7 routes)
    // Note: The module path is followed by path segments to each handler
    ($path:expr, $($controller:ident)::+) => {{
        const _: &str = $crate::validate_route_path($path);
        $crate::ResourceDef::__new_unchecked($path)
            .__add_route($crate::ResourceAction::Index, $crate::__box_handler($($controller)::+::index))
            .__add_route($crate::ResourceAction::Create, $crate::__box_handler($($controller)::+::create))
            .__add_route($crate::ResourceAction::Store, $crate::__box_handler($($controller)::+::store))
            .__add_route($crate::ResourceAction::Show, $crate::__box_handler($($controller)::+::show))
            .__add_route($crate::ResourceAction::Edit, $crate::__box_handler($($controller)::+::edit))
            .__add_route($crate::ResourceAction::Update, $crate::__box_handler($($controller)::+::update))
            .__add_route($crate::ResourceAction::Destroy, $crate::__box_handler($($controller)::+::destroy))
    }};

    // Subset of routes with `only:` parameter
    ($path:expr, $($controller:ident)::+, only: [$($action:ident),* $(,)?]) => {{
        const _: &str = $crate::validate_route_path($path);
        let mut resource = $crate::ResourceDef::__new_unchecked($path);
        $(
            resource = resource.__add_route(
                $crate::__resource_action!($action),
                $crate::__box_handler($($controller)::+::$action)
            );
        )*
        resource
    }};
}

/// Internal macro to convert action identifier to ResourceAction enum
#[doc(hidden)]
#[macro_export]
macro_rules! __resource_action {
    (index) => {
        $crate::ResourceAction::Index
    };
    (create) => {
        $crate::ResourceAction::Create
    };
    (store) => {
        $crate::ResourceAction::Store
    };
    (show) => {
        $crate::ResourceAction::Show
    };
    (edit) => {
        $crate::ResourceAction::Edit
    };
    (update) => {
        $crate::ResourceAction::Update
    };
    (destroy) => {
        $crate::ResourceAction::Destroy
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_route_params() {
        // Basic parameter conversion
        assert_eq!(convert_route_params("/users/:id"), "/users/{id}");

        // Multiple parameters
        assert_eq!(
            convert_route_params("/posts/:post_id/comments/:id"),
            "/posts/{post_id}/comments/{id}"
        );

        // Already uses matchit syntax - should be unchanged
        assert_eq!(convert_route_params("/users/{id}"), "/users/{id}");

        // No parameters - should be unchanged
        assert_eq!(convert_route_params("/users"), "/users");
        assert_eq!(convert_route_params("/"), "/");

        // Mixed syntax (edge case)
        assert_eq!(
            convert_route_params("/users/:user_id/posts/{post_id}"),
            "/users/{user_id}/posts/{post_id}"
        );

        // Parameter at the end
        assert_eq!(
            convert_route_params("/api/v1/:version"),
            "/api/v1/{version}"
        );
    }

    // Helper for creating test handlers
    async fn test_handler(_req: Request) -> Response {
        crate::http::text("ok")
    }

    #[test]
    fn test_group_item_route() {
        // Test that RouteDefBuilder can be converted to GroupItem
        let route_builder = RouteDefBuilder::new(HttpMethod::Get, "/test", test_handler);
        let item = route_builder.into_group_item();
        matches!(item, GroupItem::Route(_));
    }

    #[test]
    fn test_group_item_nested_group() {
        // Test that GroupDef can be converted to GroupItem
        let group_def = GroupDef::__new_unchecked("/nested");
        let item = group_def.into_group_item();
        matches!(item, GroupItem::NestedGroup(_));
    }

    #[test]
    fn test_group_add_route() {
        // Test adding a route to a group
        let group = GroupDef::__new_unchecked("/api").add(RouteDefBuilder::new(
            HttpMethod::Get,
            "/users",
            test_handler,
        ));

        assert_eq!(group.items.len(), 1);
        matches!(&group.items[0], GroupItem::Route(_));
    }

    #[test]
    fn test_group_add_nested_group() {
        // Test adding a nested group to a group
        let nested = GroupDef::__new_unchecked("/users");
        let group = GroupDef::__new_unchecked("/api").add(nested);

        assert_eq!(group.items.len(), 1);
        matches!(&group.items[0], GroupItem::NestedGroup(_));
    }

    #[test]
    fn test_group_mixed_items() {
        // Test adding both routes and nested groups
        let nested = GroupDef::__new_unchecked("/admin");
        let group = GroupDef::__new_unchecked("/api")
            .add(RouteDefBuilder::new(
                HttpMethod::Get,
                "/users",
                test_handler,
            ))
            .add(nested)
            .add(RouteDefBuilder::new(
                HttpMethod::Post,
                "/users",
                test_handler,
            ));

        assert_eq!(group.items.len(), 3);
        matches!(&group.items[0], GroupItem::Route(_));
        matches!(&group.items[1], GroupItem::NestedGroup(_));
        matches!(&group.items[2], GroupItem::Route(_));
    }

    #[test]
    fn test_deep_nesting() {
        // Test deeply nested groups (3 levels)
        let level3 = GroupDef::__new_unchecked("/level3").add(RouteDefBuilder::new(
            HttpMethod::Get,
            "/",
            test_handler,
        ));

        let level2 = GroupDef::__new_unchecked("/level2").add(level3);

        let level1 = GroupDef::__new_unchecked("/level1").add(level2);

        assert_eq!(level1.items.len(), 1);
        if let GroupItem::NestedGroup(l2) = &level1.items[0] {
            assert_eq!(l2.items.len(), 1);
            if let GroupItem::NestedGroup(l3) = &l2.items[0] {
                assert_eq!(l3.items.len(), 1);
            } else {
                panic!("Expected nested group at level 2");
            }
        } else {
            panic!("Expected nested group at level 1");
        }
    }

    #[test]
    fn test_backward_compatibility_route_method() {
        // Test that the old .route() method still works
        let group = GroupDef::__new_unchecked("/api").route(RouteDefBuilder::new(
            HttpMethod::Get,
            "/users",
            test_handler,
        ));

        assert_eq!(group.items.len(), 1);
        matches!(&group.items[0], GroupItem::Route(_));
    }

    // -------------------------------------------------------------------------
    // D-01 through D-08 full-router dispatch tests
    // -------------------------------------------------------------------------

    #[test]
    fn group_root_handler_matches_both_variants() {
        // D-01: group!("/api", { get!("/", h) }) reaches h at both /api and /api/
        use hyper::Method;
        use crate::routing::get_registered_routes;

        let group = GroupDef::__new_unchecked("/api-d01")
            .add(RouteDefBuilder::new(HttpMethod::Get, "/", test_handler));
        let router = group.register(Router::new());

        // D-07: exactly one RouteInfo entry for /api-d01 (no duplicate for the alias)
        let routes = get_registered_routes();
        let count = routes.iter().filter(|r| r.path == "/api-d01").count();
        assert_eq!(count, 1, "expected exactly 1 RouteInfo entry for /api-d01, got {count}");

        // Both URL variants reach the handler with the canonical pattern
        let hit_canonical = router.match_route(&Method::GET, "/api-d01");
        assert!(hit_canonical.is_some(), "canonical /api-d01 did not match");
        assert_eq!(hit_canonical.unwrap().2, "/api-d01");

        let hit_alternate = router.match_route(&Method::GET, "/api-d01/");
        assert!(hit_alternate.is_some(), "alternate /api-d01/ did not match");
        assert_eq!(
            hit_alternate.unwrap().2,
            "/api-d01",
            "alternate leaf must carry canonical pattern for middleware lookup"
        );
    }

    #[test]
    fn root_prefix_root_handler_is_single_slash() {
        // D-02: group!("/", { get!("/", h) }) registers exactly one / route.
        // Uses a unique prefix to avoid collision with other tests that also register "/".
        use hyper::Method;
        use crate::routing::get_registered_routes;

        let group = GroupDef::__new_unchecked("/")
            .add(RouteDefBuilder::new(HttpMethod::Get, "/", test_handler));
        let router = group.register(Router::new());

        // D-07 / D-02: no duplicate RouteInfo for "/" — check via matchit (only one
        // canonical leaf exists; a duplicate would cause insert to silently overwrite,
        // which matchit handles idempotently, so the behaviour test below is the proof).
        let hit = router.match_route(&Method::GET, "/");
        assert!(hit.is_some(), "/ did not match");
        assert_eq!(hit.unwrap().2, "/");

        // No spurious "//" alternate registered
        let double = router.match_route(&Method::GET, "//");
        assert!(double.is_none(), "// must not be registered for root-in-root group");

        // Single RouteInfo for "/"  — filter by exact path in the global registry
        let routes = get_registered_routes();
        let slash_count = routes.iter().filter(|r| r.path == "/" && r.method == "GET").count();
        // At least 1 (this test registered one); must be exactly 1 more than before.
        // We cannot assert == 1 globally (other tests may have registered "/").
        // Assert no duplicate within this session for this specific invocation path by
        // verifying the router itself only has one matchit leaf — done via the match above.
        assert!(slash_count >= 1, "expected at least one GET / in registry");
    }

    #[test]
    fn trailing_slash_prefix_is_stripped() {
        // D-03: group!("/api/", { get!("/x", h) }) normalises to /api/x, not /api//x
        use hyper::Method;

        let group = GroupDef::__new_unchecked("/api/")
            .add(RouteDefBuilder::new(HttpMethod::Get, "/x", test_handler));
        let router = group.register(Router::new());

        let hit = router.match_route(&Method::GET, "/api/x");
        assert!(hit.is_some(), "/api/x did not match after trailing-slash strip");

        let double = router.match_route(&Method::GET, "/api//x");
        assert!(double.is_none(), "/api//x should not match (double slash must not be registered)");
    }

    #[test]
    fn non_root_prefix_non_root_path_unchanged() {
        // D-04 regression: group!("/api-d04", { get!("/users", h) }) produces /api-d04/users, no alternate
        use hyper::Method;
        use crate::routing::get_registered_routes;

        let group = GroupDef::__new_unchecked("/api-d04")
            .add(RouteDefBuilder::new(HttpMethod::Get, "/users", test_handler));
        let router = group.register(Router::new());

        // Canonical path must match
        let hit = router.match_route(&Method::GET, "/api-d04/users");
        assert!(hit.is_some(), "/api-d04/users did not match");

        // No spurious trailing-slash alternate for non-root route paths (D-04)
        let alt = router.match_route(&Method::GET, "/api-d04/users/");
        assert!(alt.is_none(), "/api-d04/users/ must not be registered (no alternate for non-root path)");

        // Exactly one RouteInfo entry for this canonical path (D-07)
        let routes = get_registered_routes();
        let count = routes.iter().filter(|r| r.path == "/api-d04/users").count();
        assert_eq!(count, 1, "expected exactly 1 RouteInfo for /api-d04/users, got {count}");
    }

    #[test]
    fn nested_group_root_matches_both_variants() {
        // D-06 + Pitfall 3: nested groups with root handler reach both variants
        use hyper::Method;

        // Case 1: clean prefixes
        let inner1 = GroupDef::__new_unchecked("/b")
            .add(RouteDefBuilder::new(HttpMethod::Get, "/", test_handler));
        let router1 = GroupDef::__new_unchecked("/a")
            .add(inner1)
            .register(Router::new());
        assert!(router1.match_route(&Method::GET, "/a/b").is_some(), "/a/b did not match");
        assert!(router1.match_route(&Method::GET, "/a/b/").is_some(), "/a/b/ did not match");

        // Case 2: outer prefix has trailing slash (Pitfall 3)
        let inner2 = GroupDef::__new_unchecked("/b")
            .add(RouteDefBuilder::new(HttpMethod::Get, "/", test_handler));
        let router2 = GroupDef::__new_unchecked("/a/")
            .add(inner2)
            .register(Router::new());
        assert!(router2.match_route(&Method::GET, "/a/b").is_some(), "/a/b did not match with trailing-slash outer");
        assert!(router2.match_route(&Method::GET, "/a/b/").is_some(), "/a/b/ did not match with trailing-slash outer");
    }

    #[test]
    fn named_route_resolves_to_canonical() {
        // D-08: named routes register only the canonical path
        use crate::routing::route;

        let group = GroupDef::__new_unchecked("/api")
            .add(RouteDefBuilder::new(HttpMethod::Get, "/", test_handler).name("home_canonical_test"));
        let _router = group.register(Router::new());

        let url = route("home_canonical_test", &[]);
        assert_eq!(url, Some("/api".to_string()), "named route must resolve to canonical /api, not /api/");
    }

    #[test]
    fn top_level_root_route_is_single_slash() {
        // Regression: top-level get!("/", h) (not inside a group) must still match /.
        // Confirms the fix lives only in GroupDef, not in RouteDefBuilder::register.
        use hyper::Method;

        let router: Router = Router::new().get("/", test_handler).into();

        let hit = router.match_route(&Method::GET, "/");
        assert!(hit.is_some(), "top-level / did not match");
        assert_eq!(hit.unwrap().2, "/", "top-level / pattern must be /");

        // No spurious alternate — top-level router.get() does not emit aliases
        let double = router.match_route(&Method::GET, "//");
        assert!(double.is_none(), "// must not match from a top-level get!(\"/\") registration");
    }
}
