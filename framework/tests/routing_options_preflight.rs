//! Router OPTIONS preflight: any path registered under any verb resolves to a
//! synthetic 204 handler with the matched route's pattern, so route-level
//! middleware (notably CORS) still runs and can short-circuit with proper
//! preflight headers.
//!
//! Without this fix, OPTIONS requests to a path with no explicit OPTIONS handler
//! returned 404 before middleware ever executed — making Cors::permissive()
//! useless for browser preflights even though the middleware itself
//! short-circuits OPTIONS correctly when invoked in isolation.

extern crate ferro_rs as ferro;

use ferro_rs::{Request, Response, Router, HttpResponse};
use serial_test::serial;

async fn ok_handler(_req: Request) -> Response {
    Ok(HttpResponse::new().status(200).set_body("ok"))
}

#[tokio::test]
#[serial]
async fn options_preflight_resolves_when_get_route_exists() {
    let router = Router::new().get("/api/v1/products", ok_handler).name("opt_get");

    let m = router.match_route(&hyper::Method::OPTIONS, "/api/v1/products");
    assert!(m.is_some(), "OPTIONS must resolve when a GET route exists");
    let (_handler, params, pattern) = m.unwrap();
    assert!(params.is_empty());
    assert_eq!(
        pattern, "/api/v1/products",
        "preflight must return the canonical pattern so middleware lookup matches"
    );
}

#[tokio::test]
#[serial]
async fn options_preflight_resolves_when_post_route_exists() {
    let router = Router::new()
        .post("/api/v1/bookings", ok_handler)
        .name("opt_post");

    let m = router.match_route(&hyper::Method::OPTIONS, "/api/v1/bookings");
    assert!(m.is_some(), "OPTIONS must resolve when a POST route exists");
    let (_handler, _params, pattern) = m.unwrap();
    assert_eq!(pattern, "/api/v1/bookings");
}

#[tokio::test]
#[serial]
async fn options_preflight_extracts_path_params() {
    let router = Router::new()
        .post("/api/v1/businesses/{slug}/bookings", ok_handler)
        .name("opt_params");

    let m = router.match_route(
        &hyper::Method::OPTIONS,
        "/api/v1/businesses/amaris-experience/bookings",
    );
    assert!(m.is_some());
    let (_handler, params, pattern) = m.unwrap();
    assert_eq!(
        params.get("slug").map(|s| s.as_str()),
        Some("amaris-experience")
    );
    assert_eq!(pattern, "/api/v1/businesses/{slug}/bookings");
}

#[tokio::test]
#[serial]
async fn options_on_unregistered_path_still_404s() {
    let router = Router::new().get("/api/v1/products", ok_handler).name("opt_unregistered");

    let m = router.match_route(&hyper::Method::OPTIONS, "/totally/unregistered");
    assert!(
        m.is_none(),
        "OPTIONS to a non-routed path must NOT synthesize a handler"
    );
}
