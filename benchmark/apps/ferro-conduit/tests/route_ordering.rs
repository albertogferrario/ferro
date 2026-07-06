//! Route-ordering guard (RESEARCH Pitfall 2).
//!
//! `GET /api/articles/feed` (literal) MUST resolve to the feed handler, not to
//! `GET /api/articles/{slug}` with `slug = "feed"`. Ferro's matchit router gives
//! literal segments priority over wildcards; this test locks that behaviour in
//! before the article routes land in Plan 04/05. It mirrors the production
//! ordering by registering both routes against a local `Router`.

use ferro::{get, http::HttpResponse, routes, Response};
use hyper::Method;

#[ferro::handler]
async fn feed() -> Response {
    Ok(HttpResponse::json(ferro::serde_json::json!({ "sentinel": "feed" })))
}

#[ferro::handler]
async fn show() -> Response {
    Ok(HttpResponse::json(ferro::serde_json::json!({ "sentinel": "slug" })))
}

// Mirrors production ordering: literal `/api/articles/feed` declared before the
// parameterized `/api/articles/{slug}`. `routes!` expands to `register()`.
routes! {
    get!("/api/articles/feed", feed),
    get!("/api/articles/{slug}", show),
}

#[test]
fn feed_resolves_before_slug() {
    let router = register();

    // The literal route wins: pattern is the feed route, not the {slug} route.
    let (_, params, pattern) = router
        .match_route(&Method::GET, "/api/articles/feed")
        .expect("/api/articles/feed must resolve");
    assert_eq!(
        pattern, "/api/articles/feed",
        "feed must match the literal route, not /api/articles/{{slug}}"
    );
    assert!(
        !params.contains_key("slug"),
        "feed must not be captured as slug={:?}",
        params.get("slug")
    );

    // A real slug still resolves to the parameterized route.
    let (_, params, pattern) = router
        .match_route(&Method::GET, "/api/articles/how-to-train")
        .expect("/api/articles/{slug} must resolve");
    assert_eq!(pattern, "/api/articles/{slug}");
    assert_eq!(params.get("slug").map(String::as_str), Some("how-to-train"));
}
