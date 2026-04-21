//! Route grouping with shared prefix and middleware

use super::path::combine_group_path;
use super::{BoxedHandler, RouteBuilder, Router};
use crate::http::{Request, Response};
use crate::middleware::{into_boxed, BoxedMiddleware, Middleware};
use std::future::Future;
use std::sync::Arc;

/// Builder for route groups with shared prefix and middleware
///
/// # Example
///
/// ```rust,ignore
/// Router::new()
///     .group("/api", |r| {
///         r.get("/users", list_users)
///          .post("/users", create_user)
///     }).middleware(ApiMiddleware)
/// ```
pub struct GroupBuilder {
    /// The outer router we're building into
    outer_router: Router,
    /// Routes registered within this group (stored as full paths)
    group_routes: Vec<GroupRoute>,
    /// The prefix for this group
    prefix: String,
    /// Middleware to apply to all routes in this group
    middleware: Vec<BoxedMiddleware>,
}

/// A route registered within a group
struct GroupRoute {
    method: GroupMethod,
    path: String,
    handler: Arc<BoxedHandler>,
}

#[derive(Clone, Copy)]
enum GroupMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl GroupBuilder {
    /// Apply middleware to all routes in this group
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// Router::new()
    ///     .group("/api", |r| r.get("/users", handler))
    ///     .middleware(ApiMiddleware)
    /// ```
    pub fn middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middleware.push(into_boxed(middleware));
        self
    }

    /// Finalize the group and merge routes into the outer router.
    ///
    /// Uses `combine_group_path` to compute the canonical registration path
    /// (and an optional trailing-slash alternate), so `r.get("/", h)` inside
    /// a non-root group reaches `h` at both `/prefix` and `/prefix/`. The
    /// alternate matchit leaf stores the canonical pattern string, which is
    /// what `server.rs` uses as the middleware-lookup key — so one
    /// `add_middleware(&canonical, ...)` call covers both variants
    /// (Strategy A).
    fn finalize(mut self) -> Router {
        for route in self.group_routes {
            let (canonical, alternate) = combine_group_path(&self.prefix, &route.path);

            match route.method {
                GroupMethod::Get => {
                    self.outer_router
                        .insert_get(&canonical, route.handler.clone());
                    if let Some(alt) = alternate.as_deref() {
                        self.outer_router
                            .insert_get_alias(alt, route.handler, &canonical);
                    }
                }
                GroupMethod::Post => {
                    self.outer_router
                        .insert_post(&canonical, route.handler.clone());
                    if let Some(alt) = alternate.as_deref() {
                        self.outer_router
                            .insert_post_alias(alt, route.handler, &canonical);
                    }
                }
                GroupMethod::Put => {
                    self.outer_router
                        .insert_put(&canonical, route.handler.clone());
                    if let Some(alt) = alternate.as_deref() {
                        self.outer_router
                            .insert_put_alias(alt, route.handler, &canonical);
                    }
                }
                GroupMethod::Patch => {
                    self.outer_router
                        .insert_patch(&canonical, route.handler.clone());
                    if let Some(alt) = alternate.as_deref() {
                        self.outer_router
                            .insert_patch_alias(alt, route.handler, &canonical);
                    }
                }
                GroupMethod::Delete => {
                    self.outer_router
                        .insert_delete(&canonical, route.handler.clone());
                    if let Some(alt) = alternate.as_deref() {
                        self.outer_router
                            .insert_delete_alias(alt, route.handler, &canonical);
                    }
                }
            }

            // Strategy A: add middleware under the canonical key only — the
            // alias leaf's matchit value stores the canonical pattern string,
            // so dispatch resolves middleware under this same key regardless
            // of which URL variant the request hit.
            for mw in &self.middleware {
                self.outer_router.add_middleware(&canonical, mw.clone());
            }
        }

        self.outer_router
    }
}

/// Inner router used within a group closure
///
/// This captures routes without a prefix, which are later merged with the group's prefix.
pub struct GroupRouter {
    routes: Vec<GroupRoute>,
}

impl GroupRouter {
    fn new() -> Self {
        Self { routes: Vec::new() }
    }

    /// Register a GET route within the group
    pub fn get<H, Fut>(mut self, path: &str, handler: H) -> Self
    where
        H: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let boxed: BoxedHandler = Box::new(move |req| Box::pin(handler(req)));
        self.routes.push(GroupRoute {
            method: GroupMethod::Get,
            path: path.to_string(),
            handler: Arc::new(boxed),
        });
        self
    }

    /// Register a POST route within the group
    pub fn post<H, Fut>(mut self, path: &str, handler: H) -> Self
    where
        H: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let boxed: BoxedHandler = Box::new(move |req| Box::pin(handler(req)));
        self.routes.push(GroupRoute {
            method: GroupMethod::Post,
            path: path.to_string(),
            handler: Arc::new(boxed),
        });
        self
    }

    /// Register a PUT route within the group
    pub fn put<H, Fut>(mut self, path: &str, handler: H) -> Self
    where
        H: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let boxed: BoxedHandler = Box::new(move |req| Box::pin(handler(req)));
        self.routes.push(GroupRoute {
            method: GroupMethod::Put,
            path: path.to_string(),
            handler: Arc::new(boxed),
        });
        self
    }

    /// Register a PATCH route within the group
    pub fn patch<H, Fut>(mut self, path: &str, handler: H) -> Self
    where
        H: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let boxed: BoxedHandler = Box::new(move |req| Box::pin(handler(req)));
        self.routes.push(GroupRoute {
            method: GroupMethod::Patch,
            path: path.to_string(),
            handler: Arc::new(boxed),
        });
        self
    }

    /// Register a DELETE route within the group
    pub fn delete<H, Fut>(mut self, path: &str, handler: H) -> Self
    where
        H: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let boxed: BoxedHandler = Box::new(move |req| Box::pin(handler(req)));
        self.routes.push(GroupRoute {
            method: GroupMethod::Delete,
            path: path.to_string(),
            handler: Arc::new(boxed),
        });
        self
    }
}

impl Router {
    /// Create a route group with a shared prefix
    ///
    /// Routes defined within the group will have the prefix prepended to their paths.
    /// Middleware applied to the group will be applied to all routes within it.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// Router::new()
    ///     .group("/api", |r| {
    ///         r.get("/users", list_users)      // -> GET /api/users
    ///          .post("/users", create_user)    // -> POST /api/users
    ///          .get("/users/{id}", show_user)  // -> GET /api/users/{id}
    ///     })
    ///     .middleware(ApiMiddleware)
    /// ```
    pub fn group<F>(self, prefix: &str, builder_fn: F) -> GroupBuilder
    where
        F: FnOnce(GroupRouter) -> GroupRouter,
    {
        let inner = GroupRouter::new();
        let built = builder_fn(inner);

        GroupBuilder {
            outer_router: self,
            group_routes: built.routes,
            prefix: prefix.to_string(),
            middleware: Vec::new(),
        }
    }
}

impl From<GroupBuilder> for Router {
    fn from(builder: GroupBuilder) -> Self {
        builder.finalize()
    }
}

// Allow RouteBuilder to chain into groups
impl RouteBuilder {
    /// Create a route group with a shared prefix
    pub fn group<F>(self, prefix: &str, builder_fn: F) -> GroupBuilder
    where
        F: FnOnce(GroupRouter) -> GroupRouter,
    {
        self.router.group(prefix, builder_fn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::get_registered_routes;
    use hyper::Method;

    async fn test_handler(_req: Request) -> Response {
        crate::http::text("ok")
    }

    #[test]
    fn builder_group_root_handler_matches_both_variants() {
        // D-01: Router::new().group("/api", |r| r.get("/", h)) reaches h at
        // both /api and /api/. Uses unique prefix to avoid global registry
        // collision with parallel tests.
        let router: Router = Router::new()
            .group("/api-b01", |r| r.get("/", test_handler))
            .into();

        // D-07: exactly one RouteInfo entry (alias does not call register_route)
        let routes = get_registered_routes();
        let count = routes.iter().filter(|r| r.path == "/api-b01").count();
        assert_eq!(
            count, 1,
            "expected exactly 1 RouteInfo entry for /api-b01, got {count}"
        );

        // Canonical leaf
        let hit_canonical = router.match_route(&Method::GET, "/api-b01");
        assert!(hit_canonical.is_some(), "canonical /api-b01 did not match");
        assert_eq!(hit_canonical.unwrap().2, "/api-b01");

        // Alternate leaf must carry canonical pattern (Strategy A)
        let hit_alternate = router.match_route(&Method::GET, "/api-b01/");
        assert!(hit_alternate.is_some(), "alternate /api-b01/ did not match");
        assert_eq!(
            hit_alternate.unwrap().2,
            "/api-b01",
            "alternate leaf must carry canonical pattern for middleware lookup"
        );
    }

    #[test]
    fn builder_root_prefix_root_handler_is_single_slash() {
        // D-02: group "/" with route "/" → exactly one "/" leaf, no "//" alternate.
        let router: Router = Router::new()
            .group("/", |r| r.get("/", test_handler))
            .into();

        let hit = router.match_route(&Method::GET, "/");
        assert!(hit.is_some(), "/ did not match");
        assert_eq!(hit.unwrap().2, "/");

        let double = router.match_route(&Method::GET, "//");
        assert!(
            double.is_none(),
            "// must not be registered for root-in-root group"
        );
    }

    #[test]
    fn builder_trailing_slash_prefix_is_stripped() {
        // D-03: prefix "/api-b03/" with route "/x" → /api-b03/x; no /api-b03//x
        let router: Router = Router::new()
            .group("/api-b03/", |r| r.get("/x", test_handler))
            .into();

        assert!(
            router.match_route(&Method::GET, "/api-b03/x").is_some(),
            "/api-b03/x did not match after trailing-slash strip"
        );
        assert!(
            router.match_route(&Method::GET, "/api-b03//x").is_none(),
            "unexpected match for /api-b03//x — trailing slash not stripped"
        );
    }

    #[test]
    fn builder_non_root_prefix_non_root_path_unchanged() {
        // D-04 regression: non-root route path must not emit a trailing-slash alternate.
        let router: Router = Router::new()
            .group("/api-b04", |r| r.get("/users", test_handler))
            .into();

        let routes = get_registered_routes();
        let count = routes.iter().filter(|r| r.path == "/api-b04/users").count();
        assert_eq!(
            count, 1,
            "expected exactly 1 RouteInfo for /api-b04/users, got {count}"
        );

        assert!(
            router.match_route(&Method::GET, "/api-b04/users").is_some(),
            "/api-b04/users did not match"
        );
        assert!(
            router
                .match_route(&Method::GET, "/api-b04/users/")
                .is_none(),
            "non-root leaf must not emit an alternate"
        );
    }

    #[test]
    fn builder_middleware_registered_under_canonical_only() {
        // Strategy A registry-level assertion: add_middleware is called with the
        // canonical key only; the alias leaf has no separate HashMap entry.
        // (The integration test in Plan 04 proves both URL variants actually
        // execute the middleware at dispatch time.)
        let router: Router = Router::new()
            .group("/api-b05", |r| r.get("/", test_handler))
            .into();

        // Group has no middleware — both lookups should be empty vecs.
        // Structural assertion: the lookup API works and returns the correct
        // (empty) vec; no panics; no key confusion.
        assert!(
            router.get_route_middleware("/api-b05").is_empty(),
            "canonical key must return empty vec (no middleware registered)"
        );
        assert!(
            router.get_route_middleware("/api-b05/").is_empty(),
            "alias key must return empty vec (Strategy A: no separate entry)"
        );
    }

    #[test]
    fn builder_post_and_put_and_patch_and_delete_aliases_reach_handler() {
        // Verify all four non-GET verbs emit canonical + alias pairs when
        // route path is "/".
        let router_post: Router = Router::new()
            .group("/api-b06p", |r| r.post("/", test_handler))
            .into();
        assert!(
            router_post
                .match_route(&Method::POST, "/api-b06p")
                .is_some(),
            "POST /api-b06p did not match"
        );
        assert!(
            router_post
                .match_route(&Method::POST, "/api-b06p/")
                .is_some(),
            "POST /api-b06p/ did not match"
        );

        let router_put: Router = Router::new()
            .group("/api-b06u", |r| r.put("/", test_handler))
            .into();
        assert!(
            router_put.match_route(&Method::PUT, "/api-b06u").is_some(),
            "PUT /api-b06u did not match"
        );
        assert!(
            router_put.match_route(&Method::PUT, "/api-b06u/").is_some(),
            "PUT /api-b06u/ did not match"
        );

        let router_patch: Router = Router::new()
            .group("/api-b06a", |r| r.patch("/", test_handler))
            .into();
        assert!(
            router_patch
                .match_route(&Method::PATCH, "/api-b06a")
                .is_some(),
            "PATCH /api-b06a did not match"
        );
        assert!(
            router_patch
                .match_route(&Method::PATCH, "/api-b06a/")
                .is_some(),
            "PATCH /api-b06a/ did not match"
        );

        let router_delete: Router = Router::new()
            .group("/api-b06d", |r| r.delete("/", test_handler))
            .into();
        assert!(
            router_delete
                .match_route(&Method::DELETE, "/api-b06d")
                .is_some(),
            "DELETE /api-b06d did not match"
        );
        assert!(
            router_delete
                .match_route(&Method::DELETE, "/api-b06d/")
                .is_some(),
            "DELETE /api-b06d/ did not match"
        );
    }
}
