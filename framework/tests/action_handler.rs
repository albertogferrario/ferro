//! Integration tests for the `#[action]` runtime helper.
//!
//! Exercises `ferro::http::action::handle_action_result` (the `pub #[doc(hidden)]`
//! runtime helper that the `#[action]` macro dispatches to) against simulated
//! `Ok(())` and `Err(ActionError::...)` inputs, asserting on:
//!
//! - 303 Location header (happy path)
//! - Success-side overrides via `req.flash(...)` / `req.redirect_to(...)` (D-02)
//! - Error-side `redirect_override` via `ActionError::*::redirect_to(...)` (D-01)
//! - T-180-02 open-redirect mitigation (both success and error paths)
//! - T-180-03 log-injection mitigation (sanitizer strips control chars)
//! - Back-compat query string (D-06)
//!
//! `handle_action_result` is `pub #[doc(hidden)]` from Plan 03 (raised from
//! `pub(crate)` so proc-macro-generated user code can call it) — integration
//! tests reach it via the fully qualified path `ferro::http::action::handle_action_result`.
//! No `__test_handle_action_result` shim is needed; the visibility is already
//! reachable.

extern crate ferro_rs as ferro;

use ferro::http::action::handle_action_result;
use ferro::{action, ActionError, ActionResult, FlashVariant, HttpResponse, Request, Response};

use hyper_util::rt::TokioIo;
use tokio::sync::oneshot;

/// Read the Location header value from the HttpResponse via the verified
/// getter `HttpResponse::headers() -> &[(String, String)]` (response.rs:142).
/// Case-insensitive on the header name to mirror RFC 7230 § 3.2.
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

/// Construct a real `ferro::Request` via TCP loopback — the canonical pattern
/// from `framework/src/tenant/mod.rs:166-208`. `Request::new` requires a
/// `hyper::Request<hyper::body::Incoming>` and `Incoming` has no default
/// constructor, so we use a real TCP connection.
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

/// Smoke test: the public API surface compiles in a downstream crate.
#[test]
fn public_surface_compiles() {
    let _r: ActionResult = Ok(());
    let _e = ActionError::msg("smoke");
    let _e2 = ActionError::not_found("missing");
    let _e3 = ActionError::forbidden("nope");
    let _e4 = ActionError::unauthorized("login")
        .with_flash(FlashVariant::Warning)
        .redirect_to("/login");
}

/// Macro smoke test: `#[action(redirect_to = "/x")]` compiles in a downstream
/// crate and produces a `Response`-returning async fn.
#[action(redirect_to = "/x")]
pub async fn macro_smoke_handler(_req: Request) -> ActionResult {
    Ok(())
}

#[test]
fn macro_generated_handler_has_correct_type() {
    let _f: fn(Request) -> _ = macro_smoke_handler;
}

#[tokio::test]
async fn happy_path_ok_unit_redirects_303() {
    let mut req = make_request().await;
    let resp = handle_action_result(Ok(()), "/dashboard", "test::happy_path", &mut req);
    let r = unwrap_response(&resp);
    assert_eq!(r.status_code(), 303);
    let loc = location_header(r).expect("Location header present");
    assert!(loc.starts_with("/dashboard?success="), "got: {loc}");
}

#[tokio::test]
async fn success_override_redirect_and_flash() {
    let mut req = make_request().await;
    req.redirect_to("/dashboard/pagine/42");
    req.flash("created");
    let resp = handle_action_result(Ok(()), "/dashboard", "test::success_override", &mut req);
    let r = unwrap_response(&resp);
    let loc = location_header(r).expect("Location header present");
    assert!(loc.starts_with("/dashboard/pagine/42"), "got: {loc}");
    assert!(loc.contains("success=created"), "got: {loc}");
}

#[tokio::test]
async fn error_path_default_redirect_with_msg() {
    let mut req = make_request().await;
    let err = ActionError::msg("boom");
    let resp = handle_action_result(Err(err), "/dashboard", "test::error_path", &mut req);
    let r = unwrap_response(&resp);
    assert_eq!(r.status_code(), 303);
    let loc = location_header(r).expect("Location header present");
    assert!(
        loc.starts_with("/dashboard?error=generic&msg="),
        "got: {loc}"
    );
    assert!(loc.contains("boom"), "got: {loc}");
}

#[tokio::test]
async fn error_path_with_redirect_override() {
    let mut req = make_request().await;
    let err = ActionError::unauthorized("login").redirect_to("/your-login-path");
    let resp = handle_action_result(Err(err), "/dashboard", "test::error_override", &mut req);
    let r = unwrap_response(&resp);
    let loc = location_header(r).expect("Location header present");
    assert!(loc.starts_with("/your-login-path"), "got: {loc}");
}

#[tokio::test]
async fn t_180_02_open_redirect_error_side_falls_back() {
    let mut req = make_request().await;
    let err = ActionError::msg("x").redirect_to("https://evil.example/");
    let resp = handle_action_result(Err(err), "/dashboard", "test::t_180_02_error", &mut req);
    let r = unwrap_response(&resp);
    let loc = location_header(r).expect("Location header present");
    assert!(loc.starts_with("/dashboard"), "got: {loc}");
    assert!(
        !loc.contains("evil.example"),
        "open redirect leaked attacker URL: {loc}"
    );
}

#[tokio::test]
async fn t_180_02_open_redirect_success_side_falls_back() {
    let mut req = make_request().await;
    req.redirect_to("https://evil.example/");
    let resp = handle_action_result(Ok(()), "/dashboard", "test::t_180_02_success", &mut req);
    let r = unwrap_response(&resp);
    let loc = location_header(r).expect("Location header present");
    assert!(loc.starts_with("/dashboard"), "got: {loc}");
    assert!(!loc.contains("evil.example"), "got: {loc}");
}

#[tokio::test]
async fn t_180_03_log_injection_message_percent_encoded() {
    // The sanitizer's tracing-side correctness is covered by the Plan 01
    // in-module unit test `sanitize_strips_control_chars`. Here we confirm
    // the message round-trips into the URL with the newline percent-encoded.
    let mut req = make_request().await;
    let err = ActionError::msg("a\nfake-log-line");
    let resp = handle_action_result(Err(err), "/dashboard", "test::t_180_03", &mut req);
    let r = unwrap_response(&resp);
    let loc = location_header(r).expect("Location header present");
    assert!(loc.contains("%0A") || loc.contains("%0a"), "got: {loc}");
}

/// Regression for WR-01 (180-REVIEW.md): redirect target that already
/// contains a query string must use `&` instead of `?` for the back-compat
/// success/error suffix. Without this fix, `/list?page=2` would become
/// `/list?page=2?success=created` (malformed URL).
#[tokio::test]
async fn redirect_target_with_query_string_uses_ampersand_separator_success_path() {
    let mut req = make_request().await;
    req.redirect_to("/list?page=2");
    req.flash("created");
    let resp = handle_action_result(Ok(()), "/dashboard", "test::ampersand_success", &mut req);
    let r = unwrap_response(&resp);
    let loc = location_header(r).expect("Location header present");
    assert!(
        loc.starts_with("/list?page=2&success=created"),
        "got: {loc} — expected '&' separator when target already has '?'"
    );
    assert!(
        !loc.contains("?page=2?"),
        "double '?' produced — got: {loc}"
    );
}

/// Regression for WR-01 (180-REVIEW.md): error path with a redirect-override
/// that already contains a query string.
#[tokio::test]
async fn redirect_target_with_query_string_uses_ampersand_separator_error_path() {
    let mut req = make_request().await;
    let err = ActionError::msg("boom").redirect_to("/list?page=2");
    let resp = handle_action_result(Err(err), "/dashboard", "test::ampersand_error", &mut req);
    let r = unwrap_response(&resp);
    let loc = location_header(r).expect("Location header present");
    assert!(
        loc.starts_with("/list?page=2&error=generic&msg="),
        "got: {loc} — expected '&' separator when target already has '?'"
    );
    assert!(
        !loc.contains("?page=2?"),
        "double '?' produced — got: {loc}"
    );
}

/// Regression for WR-02 (180-REVIEW.md): flash key is percent-encoded so
/// `&` / `=` / space in user-supplied keys do not break the URL.
#[tokio::test]
async fn flash_key_is_percent_encoded() {
    let mut req = make_request().await;
    req.flash("foo & bar");
    let resp = handle_action_result(Ok(()), "/dashboard", "test::flash_pct_encode", &mut req);
    let r = unwrap_response(&resp);
    let loc = location_header(r).expect("Location header present");
    assert!(
        loc.contains("success=foo+%26+bar") || loc.contains("success=foo%20%26%20bar"),
        "flash key not percent-encoded: {loc}"
    );
}

#[tokio::test]
async fn warning_flash_variant_records_303_on_error_path() {
    let mut req = make_request().await;
    let err = ActionError::msg("careful").with_flash(FlashVariant::Warning);
    let resp = handle_action_result(Err(err), "/dashboard", "test::warning", &mut req);
    let r = unwrap_response(&resp);
    assert_eq!(r.status_code(), 303);
}

// ── `*_mut` body reader tests — Phase 180 follow-up for consumer migration ──
//
// `#[action]` binds `req` as `&mut Request`, so the legacy `self`-consuming
// body readers (`form`, `body_bytes`, `multipart`, `file`, `json`, `input`)
// cannot be called inside an action body. The `*_mut` variants below close
// that gap. Tests prove they:
//   1. Read the body successfully when called inside an `#[action]` body
//      (representable here as a fresh `&mut Request`).
//   2. Cache the body so calling twice returns the same payload (instead of
//      panicking or erroring on the second read).
//   3. Coexist with `req.flash()` / `req.redirect_to()` (the success-side
//      overrides).

/// Build a request with a `Content-Type: application/x-www-form-urlencoded`
/// body containing the given bytes. Mirrors `make_request()` but writes a real
/// body so the body-reader methods have something to read.
async fn make_request_with_body(content_type: &str, body: &'static [u8]) -> Request {
    use http_body_util::Full;

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
        .method("POST")
        .uri("/test")
        .header("Content-Type", content_type)
        .body(Full::<bytes::Bytes>::from(body))
        .unwrap();
    let _ = sender.send_request(req).await;
    rx.await.unwrap()
}

#[tokio::test]
async fn body_bytes_mut_reads_body() {
    let mut req =
        make_request_with_body("application/x-www-form-urlencoded", b"name=alice&age=30").await;
    let bytes = req.body_bytes_mut().await.expect("body_bytes_mut succeeds");
    assert_eq!(&bytes[..], b"name=alice&age=30");
}

#[tokio::test]
async fn body_bytes_mut_is_idempotent_via_cache() {
    let mut req = make_request_with_body("application/x-www-form-urlencoded", b"x=1").await;
    let a = req.body_bytes_mut().await.expect("first read succeeds");
    let b = req
        .body_bytes_mut()
        .await
        .expect("second read succeeds (cached)");
    assert_eq!(&a[..], b"x=1");
    assert_eq!(&b[..], b"x=1");
}

#[tokio::test]
async fn form_mut_parses_urlencoded_body() {
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct LoginForm {
        username: String,
        password: String,
    }

    let mut req = make_request_with_body(
        "application/x-www-form-urlencoded",
        b"username=alice&password=secret",
    )
    .await;
    let form: LoginForm = req.form_mut().await.expect("form_mut parses");
    assert_eq!(
        form,
        LoginForm {
            username: "alice".into(),
            password: "secret".into(),
        }
    );
}

#[tokio::test]
async fn form_mut_can_be_called_after_body_bytes_mut() {
    // Demonstrates that the cache lets multiple `*_mut` readers compose.
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct Echo {
        v: String,
    }

    let mut req = make_request_with_body("application/x-www-form-urlencoded", b"v=hello").await;
    let raw = req.body_bytes_mut().await.expect("body_bytes_mut succeeds");
    assert_eq!(&raw[..], b"v=hello");
    let parsed: Echo = req
        .form_mut()
        .await
        .expect("form_mut succeeds against cached bytes");
    assert_eq!(parsed, Echo { v: "hello".into() });
}

#[tokio::test]
async fn json_mut_parses_json_body() {
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct Greeting {
        hello: String,
    }

    let mut req = make_request_with_body("application/json", b"{\"hello\":\"world\"}").await;
    let parsed: Greeting = req.json_mut().await.expect("json_mut parses");
    assert_eq!(
        parsed,
        Greeting {
            hello: "world".into()
        }
    );
}

#[tokio::test]
async fn input_mut_routes_by_content_type() {
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct One {
        k: String,
    }

    // form-urlencoded → form parser
    let mut req_form =
        make_request_with_body("application/x-www-form-urlencoded", b"k=urlencoded").await;
    let v: One = req_form.input_mut().await.expect("form path");
    assert_eq!(
        v,
        One {
            k: "urlencoded".into()
        }
    );

    // anything else (json) → json parser
    let mut req_json = make_request_with_body("application/json", b"{\"k\":\"json\"}").await;
    let v: One = req_json.input_mut().await.expect("json path");
    assert_eq!(v, One { k: "json".into() });
}

#[tokio::test]
async fn body_mut_coexists_with_flash_and_redirect_to() {
    // Proves that an `#[action]`-shaped handler can read the body AND set
    // success-side overrides in the same body. The macro runtime then sees
    // both: the response carries the override redirect, and (separately) the
    // body has been consumed without breaking the override pickup.
    let mut req = make_request_with_body("application/x-www-form-urlencoded", b"id=42").await;
    let bytes = req.body_bytes_mut().await.expect("body read");
    assert_eq!(&bytes[..], b"id=42");

    req.redirect_to("/dashboard/things/42");
    req.flash("created");
    let resp = handle_action_result(
        Ok(()),
        "/dashboard/things",
        "test::body_mut_with_overrides",
        &mut req,
    );
    let r = unwrap_response(&resp);
    let loc = location_header(r).expect("Location header present");
    assert!(
        loc.starts_with("/dashboard/things/42"),
        "override should win — got: {loc}"
    );
    assert!(
        loc.contains("success=created"),
        "flash should be present — got: {loc}"
    );
}
