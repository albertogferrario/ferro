//! Integration tests for Phase 144 — group route trailing-slash semantics.
//!
//! Covers:
//! - D-07 / D-10: get_registered_routes() returns one entry per logical
//!   handler even when the handler is registered under two matchit leaves.
//! - T-144-12 mitigation: middleware attached to a group is reachable via
//!   get_route_middleware for both /prefix and /prefix/ request variants
//!   (Strategy A structural proof).
//! - Gestiscilo reproducer: group!("/s/{slug}", { get!("/", root),
//!   get!("/index.html", idx), get!("/{*path}", asset) }) routes all four
//!   URL shapes correctly.
//! - Regression (Pitfall 6): top-level get!("/", h) outside a group still
//!   produces exactly one RouteInfo.

extern crate ferro_rs as ferro;

use ferro_rs::{
    async_trait, get, get_registered_routes, group, GroupBuilder, Middleware, Next, Request,
    Response, Router,
};
use serial_test::serial;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// --------------------------------------------------------------------
// Fixtures
// --------------------------------------------------------------------

async fn serve_root(_req: Request) -> Response {
    ferro_rs::text("root")
}

async fn serve_index(_req: Request) -> Response {
    ferro_rs::text("index")
}

async fn serve_asset(_req: Request) -> Response {
    ferro_rs::text("asset")
}

async fn hello(_req: Request) -> Response {
    ferro_rs::text("hello")
}

/// Maps a `(params, route_pattern)` out of `router.match_route` for
/// assertion clarity. `router.match_route` takes `&hyper::Method`.
fn dispatch(
    router: &Router,
    method: &hyper::Method,
    path: &str,
) -> Option<(HashMap<String, String>, String)> {
    router
        .match_route(method, path)
        .map(|(_, params, pattern)| (params, pattern))
}

// --------------------------------------------------------------------
// D-07 / D-10: one RouteInfo per logical handler
// --------------------------------------------------------------------

#[test]
#[serial]
fn no_duplicate_route_info() {
    let before = get_registered_routes().len();

    // Single root-in-group handler — registers 2 matchit leaves but 1 RouteInfo.
    // group!() returns GroupDef; .register(Router::new()) builds and returns the Router.
    let _router: Router = group!("/api-i01", {
        get!("/", hello)
    })
    .register(Router::new());

    let after = get_registered_routes().len();

    // Path-specific check: exactly one RouteInfo for /api-i01.
    let count = get_registered_routes()
        .iter()
        .filter(|r| r.path == "/api-i01")
        .count();
    assert_eq!(
        count,
        1,
        "group!('/api-i01', {{ get!('/', h) }}) must produce exactly 1 RouteInfo (got {count})",
    );

    // Delta must also be 1 (nothing else registered in this serial test).
    let delta = after - before;
    assert_eq!(
        delta,
        1,
        "expected delta of 1, got {delta}",
    );
}

#[test]
#[serial]
fn no_duplicate_route_info_multi_handler_group() {
    let before = get_registered_routes().len();

    // Group with two handlers: one root (alternate emitted), one non-root (no alternate).
    // Expected RouteInfo delta: 2 (one per logical handler).
    let _router: Router = group!("/api-i02", {
        get!("/", hello),
        get!("/health", hello)
    })
    .register(Router::new());

    let after = get_registered_routes().len();
    let delta = after - before;
    assert_eq!(
        delta,
        2,
        "group with 2 handlers must produce exactly 2 RouteInfo entries (got delta {delta})",
    );

    // Verify individual paths.
    let root_count = get_registered_routes()
        .iter()
        .filter(|r| r.path == "/api-i02")
        .count();
    assert_eq!(root_count, 1, "/api-i02 must appear exactly once");

    let health_count = get_registered_routes()
        .iter()
        .filter(|r| r.path == "/api-i02/health")
        .count();
    assert_eq!(health_count, 1, "/api-i02/health must appear exactly once");
}

// --------------------------------------------------------------------
// T-144-12 mitigation: middleware lookup resolves under canonical key
// for both /prefix and /prefix/ variants (Strategy A structural proof).
//
// Uses the structural-assertion fallback (Plan 04 §Action, Note 2):
// no full server-dispatch helper is available in framework/tests/.
// The test proves via the public Router API that:
//   (a) middleware attached via GroupBuilder::middleware() is stored
//       under the canonical pattern "/api-i03" (not "/api-i03/"), and
//   (b) both match_route calls ("/api-i03" and "/api-i03/") return a
//       route_pattern equal to "/api-i03", so get_route_middleware
//       retrieves the same middleware slice for both URL variants.
// Plan 03's builder_middleware_registered_under_canonical_only unit
// test covers the registry layer; this integration test confirms the
// GroupBuilder pipeline end-to-end.
// --------------------------------------------------------------------

struct CounterMw {
    counter: Arc<AtomicUsize>,
}

#[async_trait]
impl Middleware for CounterMw {
    async fn handle(&self, request: Request, next: Next) -> Response {
        self.counter.fetch_add(1, Ordering::SeqCst);
        next(request).await
    }
}

#[test]
#[serial]
fn middleware_runs_for_both_variants() {
    use hyper::Method;

    let counter = Arc::new(AtomicUsize::new(0));

    // Use GroupBuilder API (Router::group) so .middleware() is available
    // without calling pub(crate) add_middleware directly.
    // GroupBuilder implements Into<Router> via From<GroupBuilder> for Router.
    let router: Router = Router::new()
        .group("/api-i03", |r| r.get("/", hello))
        .middleware(CounterMw {
            counter: counter.clone(),
        })
        .into();

    // Both URL variants must resolve to the canonical pattern "/api-i03".
    let (_, _, pattern_canonical) = router
        .match_route(&Method::GET, "/api-i03")
        .expect("GET /api-i03 must match");
    assert_eq!(
        pattern_canonical, "/api-i03",
        "/api-i03 must resolve to canonical pattern /api-i03"
    );

    let (_, _, pattern_alt) = router
        .match_route(&Method::GET, "/api-i03/")
        .expect("GET /api-i03/ must match");
    assert_eq!(
        pattern_alt, "/api-i03",
        "/api-i03/ must carry canonical pattern /api-i03 (Strategy A)"
    );

    // Middleware is registered under the canonical key and reachable for both variants.
    let mw_canonical = router.get_route_middleware(&pattern_canonical);
    assert_eq!(
        mw_canonical.len(),
        1,
        "canonical route must have exactly 1 middleware registered"
    );

    // Since pattern_alt == pattern_canonical, this is the same lookup —
    // proving that the alias leaf's stored canonical pattern routes middleware
    // correctly for both URL variants (Strategy A).
    let mw_alt = router.get_route_middleware(&pattern_alt);
    assert_eq!(
        mw_alt.len(),
        1,
        "alias route (via canonical pattern) must have 1 middleware — Strategy A proof"
    );

    // Counter unused in structural proof but kept for completeness.
    let _ = counter;
}

// Ensure GroupBuilder is considered used (it's the return type of Router::group).
const _: fn() -> Option<GroupBuilder> = || None;

// --------------------------------------------------------------------
// Gestiscilo-it field-test reproducer
// --------------------------------------------------------------------

#[test]
#[serial]
fn gestiscilo_reproducer() {
    use hyper::Method;

    // group!() returns GroupDef; .register(Router::new()) builds the Router.
    let router: Router = group!("/s/{slug}", {
        get!("/", serve_root),
        get!("/index.html", serve_index),
        get!("/{*path}", serve_asset)
    })
    .register(Router::new());

    // /s/foo -> serve_root, slug=foo, pattern=/s/{slug}
    let (params, pattern) =
        dispatch(&router, &Method::GET, "/s/foo").expect("GET /s/foo must match");
    assert_eq!(params.get("slug").map(String::as_str), Some("foo"));
    assert_eq!(pattern, "/s/{slug}");

    // /s/foo/ -> serve_root, slug=foo, pattern=/s/{slug} (canonical preserved)
    let (params, pattern) =
        dispatch(&router, &Method::GET, "/s/foo/").expect("GET /s/foo/ must match");
    assert_eq!(params.get("slug").map(String::as_str), Some("foo"));
    assert_eq!(pattern, "/s/{slug}");

    // /s/foo/index.html -> serve_index
    let (params, pattern) = dispatch(&router, &Method::GET, "/s/foo/index.html")
        .expect("GET /s/foo/index.html must match");
    assert_eq!(params.get("slug").map(String::as_str), Some("foo"));
    assert_eq!(pattern, "/s/{slug}/index.html");

    // /s/foo/bar.css -> serve_asset
    let (params, pattern) = dispatch(&router, &Method::GET, "/s/foo/bar.css")
        .expect("GET /s/foo/bar.css must match");
    assert_eq!(params.get("slug").map(String::as_str), Some("foo"));
    assert_eq!(params.get("path").map(String::as_str), Some("bar.css"));
    assert_eq!(pattern, "/s/{slug}/{*path}");
}

// --------------------------------------------------------------------
// Pitfall 6 regression: top-level root route is single-slot
// --------------------------------------------------------------------

#[test]
#[serial]
fn top_level_root_route_is_single_slash() {
    use hyper::Method;

    let before = get_registered_routes().len();

    // Top-level get!() returns RouteDefBuilder; .register(Router::new()) builds the Router.
    let router: Router = get!("/", hello).register(Router::new());

    let after = get_registered_routes().len();
    assert_eq!(
        after - before,
        1,
        "top-level get!('/', h) must produce exactly 1 RouteInfo"
    );

    assert!(router.match_route(&Method::GET, "/").is_some());
    assert!(
        router.match_route(&Method::GET, "//").is_none(),
        "top-level root must not emit an alternate for //"
    );
}
