//! Integration tests for the Phase 176 helper:
//! `ActionError::validation_failed(url)` + `ValidationError::into_action_error(url)`.
//!
//! Pins three invariants:
//!
//! 1. `validation_failed` returns an `ActionError` with `suppress_url_envelope`
//!    set and an empty message — the consumer carries the user-visible text via
//!    per-field flash.
//! 2. When `handle_action_result` sees a suppressed error, the redirect Location
//!    carries the target URL VERBATIM — no `?error=<kind>&msg=<pct>` appended,
//!    even though 303 + log line still fire.
//! 3. `ValidationError::into_action_error(url)` chains the per-field flash side
//!    effect (verifying the flash write itself requires a real session, which is
//!    out of scope for this unit-level test) with the constructor — the returned
//!    `ActionError` carries the same suppression flag + redirect override.

extern crate ferro_rs as ferro;

use ferro::http::action::handle_action_result;
use ferro::{ActionError, FlashVariant, HttpResponse, Request, Response, ValidationError};

use hyper_util::rt::TokioIo;
use tokio::sync::oneshot;

/// Read the Location header value from the HttpResponse — mirrors the helper in
/// `tests/action_handler.rs`.
fn location_header(resp: &HttpResponse) -> Option<&str> {
    resp.headers()
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("location"))
        .map(|(_, v)| v.as_str())
}

/// Unwrap a `ferro::Response` (which is `Result<HttpResponse, HttpResponse>`)
/// for inspection — both arms carry an HttpResponse.
fn unwrap_response(resp: &Response) -> &HttpResponse {
    match resp {
        Ok(r) => r,
        Err(r) => r,
    }
}

/// Construct a real `ferro::Request` via TCP loopback — same shape as
/// `tests/action_handler.rs::make_request`.
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

#[test]
fn validation_failed_constructor_sets_suppress_flag() {
    let e = ActionError::validation_failed("/dashboard/staff/nuovo");
    assert_eq!(
        e.message, "",
        "message must be empty — per-field flash carries the text"
    );
    assert!(
        matches!(e.flash_variant, FlashVariant::Error),
        "flash_variant defaults to Error"
    );
    assert_eq!(
        e.redirect_override.as_deref(),
        Some("/dashboard/staff/nuovo"),
        "redirect_override carries the caller-supplied URL"
    );
    // `suppress_url_envelope` is `pub(crate)` — observable only via the
    // handle_action_result behaviour test below. The constructor pins it
    // structurally; the runtime test pins the consequence.
}

#[tokio::test]
async fn handle_action_result_skips_url_envelope_when_suppress_flag_set() {
    let mut req = make_request().await;
    let err = ActionError::validation_failed("/dashboard/staff/nuovo");
    let resp = handle_action_result(
        Err(err),
        "/fallback",
        "test::validation_failed_no_envelope",
        &mut req,
    );
    let r = unwrap_response(&resp);

    // 303 status still emitted.
    assert_eq!(r.status_code(), 303);

    let loc = location_header(r).expect("Location header present");

    // Location is the caller-supplied URL VERBATIM — no `?error=`, no `msg=`,
    // no `&error=` suffix.
    assert_eq!(
        loc, "/dashboard/staff/nuovo",
        "location must carry the redirect override verbatim — got: {loc}"
    );
    assert!(
        !loc.contains("?error="),
        "URL envelope must not be written when suppress_url_envelope is set; got: {loc}"
    );
    assert!(
        !loc.contains("&error="),
        "URL envelope must not be written when suppress_url_envelope is set; got: {loc}"
    );
    assert!(
        !loc.contains("msg="),
        "msg= must not be written when suppress_url_envelope is set; got: {loc}"
    );
}

#[tokio::test]
async fn handle_action_result_still_writes_envelope_for_normal_errors() {
    // Regression guard: the suppress flag must NOT leak into the default
    // `ActionError::msg` path — existing behaviour must be unchanged.
    let mut req = make_request().await;
    let err = ActionError::msg("boom");
    let resp = handle_action_result(
        Err(err),
        "/dashboard",
        "test::default_msg_still_writes_envelope",
        &mut req,
    );
    let r = unwrap_response(&resp);
    let loc = location_header(r).expect("Location header present");
    assert!(
        loc.starts_with("/dashboard?error=generic&msg="),
        "default ActionError::msg path must still write URL envelope; got: {loc}"
    );
}

#[test]
fn into_action_error_chains_flash_and_constructor() {
    // Verify the returned ActionError carries the suppress flag + redirect
    // override. The session flash side effect of `flash_into_session` requires
    // a real session middleware and is exercised by the consumer-side
    // integration tests in gestiscilo Plan 02 Task 3.
    let mut errors = ValidationError::new();
    errors.add("slug", "Slug già usato");
    let data = ferro::serde_json::json!({"slug": "x"});
    let e = errors
        .with_old_input(&data)
        .into_action_error("/dashboard/staff/nuovo");

    assert_eq!(e.message, "");
    assert_eq!(
        e.redirect_override.as_deref(),
        Some("/dashboard/staff/nuovo"),
    );
}
