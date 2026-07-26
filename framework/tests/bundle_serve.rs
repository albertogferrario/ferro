//! Integration tests for `ferro::bundle::serve` — the framework adapter that
//! wraps `ferro_bundle::serve_path` into an `HttpResponse`.
//!
//! Verifies that the adapter preserves status, headers, and body from the
//! pre-decouple `Bundle::serve(req)` behavior for the 200 (cold), 304
//! (conditional), 301 (alias redirect), and 404 (not found) paths.
//!
//! Bundle names use the prefix `frameworkserve_` to avoid duplicate-name
//! panics against other tests that run in the same process binary.

extern crate ferro_rs as ferro;

use ferro::bundle::{serve as bundle_serve, Bundle};
use ferro::Request;
use hyper_util::rt::TokioIo;
use tokio::sync::oneshot;

// ── Helper: construct a real Request via TCP loopback ─────────────────────────

/// Build a `Request` with the given path and optional `If-None-Match` header.
async fn make_request(path: &str, if_none_match: Option<&str>) -> Request {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (tx, rx) = oneshot::channel::<Request>();
    let tx_holder = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));

    tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            let io = TokioIo::new(stream);
            let tx_holder = tx_holder.clone();
            hyper::server::conn::http1::Builder::new()
                .serve_connection(
                    io,
                    hyper::service::service_fn(move |req| {
                        let tx_holder = tx_holder.clone();
                        async move {
                            if let Some(tx) = tx_holder.lock().unwrap().take() {
                                let _ = tx.send(Request::new(req));
                            }
                            Ok::<_, hyper::Error>(hyper::Response::new(http_body_util::Empty::<
                                bytes::Bytes,
                            >::new(
                            )))
                        }
                    }),
                )
                .await
                .ok();
        }
    });

    let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await.unwrap();
    tokio::spawn(async move { conn.await.ok() });

    let mut builder = hyper::Request::builder().uri(path);
    if let Some(etag) = if_none_match {
        builder = builder.header("if-none-match", etag);
    }
    let hyper_req = builder
        .body(http_body_util::Empty::<bytes::Bytes>::new())
        .unwrap();
    let _ = sender.send_request(hyper_req).await;
    rx.await.unwrap()
}

// ── Helper: extract a header value from an HttpResponse ──────────────────────

fn header_value<'a>(resp: &'a ferro::HttpResponse, name: &str) -> Option<&'a str> {
    resp.headers()
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// 200 cold path: adapter returns status 200, correct Content-Type,
/// Cache-Control, and body bytes.
#[tokio::test]
async fn serve_200_cold_path() {
    static BYTES: &[u8] = b"console.log(1);";
    let bundle = Bundle::new("frameworkserve_app_js", BYTES).content_type("application/javascript");
    let hashed = bundle.hashed_url();

    let req = make_request(&hashed, None).await;
    let resp = bundle_serve(&req);

    assert_eq!(
        resp.status_code(),
        200,
        "expected 200, got {}",
        resp.status_code()
    );
    assert_eq!(
        header_value(&resp, "Content-Type"),
        Some("application/javascript"),
        "Content-Type mismatch"
    );
    assert_eq!(
        header_value(&resp, "Cache-Control"),
        Some("public, max-age=31536000, immutable"),
        "Cache-Control mismatch"
    );
    assert_eq!(resp.body_bytes().as_ref(), BYTES, "body mismatch");
}

/// 304 conditional path: adapter forwards 304 when ETag matches If-None-Match.
#[tokio::test]
async fn serve_304_conditional_get() {
    static BYTES: &[u8] = b"body { color: red; }";
    let bundle = Bundle::new("frameworkserve_app_css", BYTES).content_type("text/css");
    let hashed = bundle.hashed_url();

    // First, get the ETag from a cold 200 hit.
    let req_cold = make_request(&hashed, None).await;
    let resp_cold = bundle_serve(&req_cold);
    assert_eq!(resp_cold.status_code(), 200);
    let etag = header_value(&resp_cold, "ETag")
        .expect("ETag header present on 200")
        .to_string();

    // Conditional GET with matching ETag → 304.
    let req_cond = make_request(&hashed, Some(&etag)).await;
    let resp_cond = bundle_serve(&req_cond);
    assert_eq!(
        resp_cond.status_code(),
        304,
        "expected 304 on matching ETag, got {}",
        resp_cond.status_code()
    );
    assert!(resp_cond.body_bytes().is_empty(), "304 body must be empty");
}

/// 301 alias redirect path: adapter returns 301 with Location header.
#[tokio::test]
async fn serve_301_alias_redirect() {
    static BYTES: &[u8] = b"<svg/>";
    let bundle = Bundle::new("frameworkserve_logo_svg", BYTES)
        .content_type("image/svg+xml")
        .with_alias("/logo.svg");
    let hashed = bundle.hashed_url();

    let req = make_request("/logo.svg", None).await;
    let resp = bundle_serve(&req);

    assert_eq!(
        resp.status_code(),
        301,
        "expected 301 for alias redirect, got {}",
        resp.status_code()
    );
    let location = header_value(&resp, "Location").expect("Location header present on 301");
    assert_eq!(location, hashed, "Location must point to hashed URL");
}

/// 404 path: adapter returns 404 for unknown paths.
#[tokio::test]
async fn serve_404_unknown_path() {
    let req = make_request("/bundles/nonexistent.abc123.js", None).await;
    let resp = bundle_serve(&req);

    assert_eq!(
        resp.status_code(),
        404,
        "expected 404 for unknown path, got {}",
        resp.status_code()
    );
}
