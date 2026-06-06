//! Integration test for Phase 184 primitives.
//!
//! Exercises `Request::inline_budget`, `Request::telemetry_record`,
//! `Request::telemetry_record_scoped`, and `RequestTelemetry::snapshot` against
//! a real `Request` constructed via the TCP-loopback pattern from
//! `framework/tests/action_handler.rs:47-94`.

extern crate ferro_rs as ferro;

use ferro::{Decision, Request, RequestTelemetry, Sample};
use serde_json::json;

use hyper_util::rt::TokioIo;
use tokio::sync::oneshot;

/// Construct a real `ferro::Request` by spinning up a TCP loopback hyper service
/// and capturing the inbound `Request` on a oneshot channel.
///
/// Copied verbatim from `framework/tests/action_handler.rs:47-94`. Kept
/// self-contained because Rust integration test files don't share modules.
///
/// `Request::new(req)` is synchronous (no `.await`) — verified at
/// `framework/src/http/request.rs:55-65`. The only constructor on `Request`.
async fn make_request() -> Request {
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

    let req = hyper::Request::builder()
        .uri("/test")
        .body(http_body_util::Empty::<bytes::Bytes>::new())
        .unwrap();
    let _ = sender.send_request(req).await;
    rx.await.unwrap()
}

#[tokio::test]
async fn inline_budget_and_telemetry_round_trip() {
    // Test isolation: clear the global telemetry store at the top.
    // `clear()` is public; `reset()` is pub(crate) and not reachable here.
    RequestTelemetry::clear();

    let mut req = make_request().await;

    // SC-1 (Inline path): 50 bytes is well below the 102_400 threshold.
    let d1 = req.inline_budget("k", 50, "/fallback");
    assert_eq!(d1, Decision::Inline);

    // SC-1 (Preload path): cumulative becomes 50 + 102_400 = 102_450, which is
    // > 102_400 — first cross triggers Preload with the caller-supplied URL.
    let d2 = req.inline_budget("k", 102_400, "/fallback");
    match d2 {
        Decision::Preload(url) => assert_eq!(url, "/fallback"),
        Decision::Inline => panic!("expected Preload after threshold cross, got Inline"),
    }

    // SC-3a: record + snapshot round-trip (unscoped).
    req.telemetry_record("latency", Sample::now(json!({"ms": 42})));
    let snap_unscoped = RequestTelemetry::snapshot("latency", None);
    assert_eq!(snap_unscoped.len(), 1);
    assert_eq!(snap_unscoped[0].value, json!({"ms": 42}));

    // SC-3a: record + snapshot round-trip (scoped to a tenant).
    req.telemetry_record_scoped("latency", Some("tenant:42"), Sample::now(json!({"ms": 50})));
    let snap_scoped = RequestTelemetry::snapshot("latency", Some("tenant:42"));
    assert_eq!(snap_scoped.len(), 1);
    assert_eq!(snap_scoped[0].value, json!({"ms": 50}));

    // Scope isolation: the unscoped bucket is untouched by the scoped write.
    let snap_unscoped_after = RequestTelemetry::snapshot("latency", None);
    assert_eq!(snap_unscoped_after.len(), 1);
    assert_eq!(snap_unscoped_after[0].value, json!({"ms": 42}));
}
