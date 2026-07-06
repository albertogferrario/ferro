# Phase 168: Framework SSE Primitives - Pattern Map

**Mapped:** 2026-06-08
**Files analyzed:** 8 (2 new, 6 modified)
**Analogs found:** 8 / 8

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `framework/src/http/sse.rs` (NEW) | utility / streaming body | streaming | `framework/src/http/response.rs` | role-match |
| `framework/src/http/body.rs` (NEW — `FerroBody` enum) | utility / body type | streaming + buffered | `framework/src/http/body.rs` (current request body) + `response.rs` `into_hyper` | partial |
| `framework/src/http/response.rs` (MODIFY) | response builder | request-response | self | exact |
| `framework/src/server.rs` (MODIFY) | server / serve loop | request-response | self | exact |
| `framework/src/websocket.rs` (MODIFY) | service / protocol upgrade | request-response | self | exact |
| `framework/src/static_files.rs` (MODIFY) | service / file I/O | request-response | self | exact |
| `framework/src/middleware/pre_route.rs` (MODIFY) | middleware | request-response | self | exact |
| `framework/src/debug/mod.rs` (MODIFY) | utility / introspection | request-response | self | exact |
| `framework/src/http/mod.rs` (MODIFY) | config / re-export | — | self | exact |
| `framework/src/lib.rs` (MODIFY) | config / re-export | — | self | exact |

---

## Pattern Assignments

### `framework/src/http/sse.rs` (NEW — `SseEvent`, `SseStream`, keep-alive)

**Analog:** `framework/src/http/response.rs`

**Structural note on WebSocket open question (RESEARCH A2, resolved):**
`hyper_tungstenite::upgrade` (v0.19.0) already returns `Response<Full<Bytes>>` — verified at
`/Users/alberto/.cargo/registry/src/.../hyper-tungstenite-0.19.0/src/lib.rs:154`:
```
pub fn upgrade<B>(...) -> Result<(Response<Full<Bytes>>, HyperWebsocket), ProtocolError>
```
`websocket.rs` line 55 `return response` already compiles today against `Full<Bytes>`. The
`response.map(|_| FerroBody::Full(...))` wrinkle in RESEARCH.md is NOT needed — just change
the return type annotation; the value itself is already `Full<Bytes>` and will convert via
`From<Full<Bytes>> for FerroBody`.

**Imports to copy from `response.rs` (lines 1–3) + add new deps:**
```rust
use bytes::Bytes;
use http_body_util::Full;
// ADD for sse.rs:
use http_body::{Body, Frame, SizeHint};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use tokio::time::{interval_at, Duration, Instant, Interval};
```

**Builder pattern to copy from `response.rs` (lines 16–96) — consuming `mut self -> Self`:**
```rust
// Every builder method follows this shape:
pub fn status(mut self, status: u16) -> Self {
    self.status = status;
    self
}
// Named constructor pattern:
pub fn text(body: impl Into<String>) -> Self { ... }
```
`SseEvent` builder follows the same named-constructor + chain pattern:
```rust
pub fn data(s: impl Into<String>) -> Self { ... }    // named constructor
pub fn event(mut self, e: impl Into<String>) -> Self { ... }
pub fn id(mut self, i: impl Into<String>) -> Self { ... }
pub fn retry(mut self, ms: u64) -> Self { ... }
```

**Inline test module convention — copy from `response.rs` lines 527–700:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_constructor() { ... }

    // tokio::test for async:
    #[tokio::test]
    async fn test_into_hyper_preserves_binary() {
        use http_body_util::BodyExt;
        let rt = tokio::runtime::Runtime::new().unwrap();
        let collected = rt.block_on(async { ... });
        // OR use #[tokio::test] directly (preferred for new code)
    }
}
```

**Keep-alive interval pattern — copy from `websocket.rs` lines 79–130 (`tokio::select!` shape):**
```rust
// websocket.rs uses interval + select! to multiplex events and timer:
let mut heartbeat_interval = tokio::time::interval(config.heartbeat_interval);
// ...
loop {
    tokio::select! {
        frame = ws_read.next() => { ... }
        _ = heartbeat_interval.tick() => { ... }
    }
}
```
`SseStream::poll_frame` uses the same `select!` idea but as a `Body` poll, not a loop.
Use `interval_at(Instant::now() + Duration::from_secs(15), Duration::from_secs(15))` to
avoid the immediate-first-tick pitfall.

---

### `framework/src/http/body.rs` (NEW — `FerroBody` enum + `Body` impl)

**Analog:** `framework/src/http/body.rs` (current file — request body utilities, NOT the response body)

The existing `body.rs` (lines 1–29) handles request body collection — `collect_body`, `parse_json`,
`parse_form`. It shows the `BodyExt` import pattern:
```rust
// framework/src/http/body.rs lines 1–9 (EXISTING — imports to reference)
use crate::error::FrameworkError;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use serde::de::DeserializeOwned;
```

**Note:** `FerroBody` can live in this same file (appended below the existing content) or in
`sse.rs`. Either placement works — `mod.rs` re-exports both. The existing `body.rs` is short
(29 lines) and the content is unrelated enough that a sibling `body.rs` section or a dedicated
placement in `sse.rs` are both clean.

**`FerroBody` itself has no prior analog in the codebase.** Use the RESEARCH.md Pattern 3
(`FerroBody` enum Body impl, lines 265–307) as the authoritative spec — it was derived
directly from the `http-body 1.0.1` trait source.

Key `From` impl pattern (follow `response.rs`'s `From<Redirect> for Response` at line 278):
```rust
// framework/src/http/response.rs lines 278–284 — From impl convention
impl From<Redirect> for Response {
    fn from(redirect: Redirect) -> Response {
        Ok(HttpResponse::new()
            .status(redirect.status)
            .header("Location", redirect.build_url()))
    }
}
// Mirror for FerroBody:
impl From<Full<Bytes>> for FerroBody {
    fn from(b: Full<Bytes>) -> Self { FerroBody::Full(b) }
}
```

---

### `framework/src/http/response.rs` (MODIFY — `into_hyper` return type + `sse` factory)

**Analog:** self

**Exact site to change — line 168:**
```rust
// BEFORE (line 168):
pub fn into_hyper(self) -> hyper::Response<Full<Bytes>> {
    let mut builder = hyper::Response::builder().status(self.status);
    for (name, value) in self.headers {
        builder = builder.header(name, value);
    }
    builder.body(Full::new(self.body)).unwrap()
}

// AFTER:
pub fn into_hyper(self) -> hyper::Response<FerroBody> {
    let mut builder = hyper::Response::builder().status(self.status);
    for (name, value) in self.headers {
        builder = builder.header(name, value);
    }
    builder.body(FerroBody::Full(Full::new(self.body))).unwrap()
}
```
One character change at the body construction site; return type annotation change.

**New `sse` / `sse_channel` factory to add (after line 176):**
Follow the `HttpResponse::text()` constructor shape (lines 27–34) — named constructor,
returns `Self` for the streaming path or `(Sender, Self)` for the channel-returning variant.
The factory sets 4 headers (copy `.header()` chain from lines 121–126):
```rust
// Pattern from lines 27–34 (text constructor) + lines 121–126 (header chain):
pub fn text(body: impl Into<String>) -> Self {
    let s: String = body.into();
    Self {
        status: 200,
        body: Bytes::from(s),
        headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
    }
}
// SSE factory will look similar but HttpResponse cannot hold SseStream in body:Bytes.
// Use a separate streaming-response type or return hyper::Response<FerroBody> directly.
// See RESEARCH.md anti-pattern note: "Don't mutate HttpResponse to hold a streaming variant."
```

**Test regression site — line 628–638 (must still compile after D-03):**
```rust
// framework/src/http/response.rs lines 628–638 — test_into_hyper_preserves_binary
#[test]
fn test_into_hyper_preserves_binary() {
    use http_body_util::BodyExt;
    let data = vec![0xFF, 0x00, 0xFE];
    let resp = HttpResponse::bytes(data.clone());
    let hyper_resp = resp.into_hyper();   // now returns hyper::Response<FerroBody>
    let rt = tokio::runtime::Runtime::new().unwrap();
    let collected =
        rt.block_on(async { hyper_resp.into_body().collect().await.unwrap().to_bytes() });
    // collect() works on FerroBody because FerroBody: Body + BodyExt (via blanket impl)
    assert_eq!(collected.as_ref(), &data);
}
```
`BodyExt::collect()` is a blanket impl on any `Body` — it will work on `FerroBody` unchanged.

---

### `framework/src/server.rs` (MODIFY — 5 sites)

**Analog:** self

**All `Full<Bytes>` import additions:** add `use crate::http::body::FerroBody;` (or wherever
`FerroBody` lives) alongside the existing `use http_body_util::Full;` at line 9.

**Site 1 — `WsInterceptor` type alias, lines 23–29:**
```rust
// BEFORE (lines 23–29):
type WsInterceptor = Box<
    dyn Fn(
            hyper::Request<hyper::body::Incoming>,
        ) -> Result<hyper::Response<Full<Bytes>>, hyper::Request<hyper::body::Incoming>>
        + Send
        + Sync,
>;

// AFTER:
type WsInterceptor = Box<
    dyn Fn(
            hyper::Request<hyper::body::Incoming>,
        ) -> Result<hyper::Response<FerroBody>, hyper::Request<hyper::body::Incoming>>
        + Send
        + Sync,
>;
```

**Site 2 — `ws_interceptor` method signature, lines 92–99:**
```rust
// BEFORE (lines 92–99):
pub fn ws_interceptor<F>(mut self, handler: F) -> Self
where
    F: Fn(
            hyper::Request<hyper::body::Incoming>,
        )
            -> Result<hyper::Response<Full<Bytes>>, hyper::Request<hyper::body::Incoming>>
        + Send
        + Sync
        + 'static,

// AFTER: same but Full<Bytes> → FerroBody in the return type
```

**Site 3 — `handle_request` return type, line 189:**
```rust
// BEFORE (line 189):
) -> hyper::Response<Full<Bytes>> {

// AFTER:
) -> hyper::Response<FerroBody> {
```
The body of `handle_request` calls `http_response.into_hyper()` (lines 284, 314, 318) which
already returns `hyper::Response<FerroBody>` after the `response.rs` change — no body changes
needed inside `handle_request`.

**Site 4 — `health_response` return type + body construction, lines 328–361:**
```rust
// BEFORE (line 328):
async fn health_response(query: &str) -> hyper::Response<Full<Bytes>> {
// ...line 359:
    .body(Full::new(Bytes::from(body)))

// AFTER (line 328):
async fn health_response(query: &str) -> hyper::Response<FerroBody> {
// ...line 359:
    .body(FerroBody::Full(Full::new(Bytes::from(body))))
```

**Site 5 — `serve_ferro_base_css` return type + body construction, lines 369–378:**
```rust
// BEFORE (line 369):
fn serve_ferro_base_css() -> hyper::Response<Full<Bytes>> {
// ...line 376:
    .body(Full::new(Bytes::from_static(css.as_bytes())))

// AFTER:
fn serve_ferro_base_css() -> hyper::Response<FerroBody> {
// ...
    .body(FerroBody::Full(Full::new(Bytes::from_static(css.as_bytes()))))
```

**`service_fn` closure — lines 161–169 (NO CHANGE NEEDED):**
```rust
// framework/src/server.rs lines 161–169 — service_fn infers from handle_request
let service = service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
    // ...
    async move {
        Ok::<_, Infallible>(
            handle_request(router, middleware, ws_interceptor, req).await,
        )
    }
});
// The Ok::<_, Infallible>(...) wrapper infers its type from handle_request's return.
// hyper::server::conn::http1::Builder::serve_connection is generic over B: Body.
// No type annotation change needed in the closure.
```

**Inline test in `server.rs` (lines 400–453) — pattern for `#[cfg(all(test, feature))]`:**
```rust
#[cfg(all(test, feature = "json-ui"))]
mod ferro_base_css_route_tests {
    use super::*;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn serve_ferro_base_css_returns_200_with_text_css_content_type() {
        let response = serve_ferro_base_css();
        assert_eq!(response.status(), 200, "expected 200 OK");
        // ...
        let body_bytes = response.into_body().collect().await ...
        // into_body().collect() works unchanged on FerroBody
    }
}
```
Tests use `response.into_body().collect().await` — works unchanged on `FerroBody` via `BodyExt`.

---

### `framework/src/websocket.rs` (MODIFY — 2 sites, SIMPLER than RESEARCH predicted)

**Analog:** self

**Key finding (resolves RESEARCH open question 1 / A2):**
`hyper_tungstenite::upgrade` v0.19.0 already returns `Response<Full<Bytes>>` (verified in
`hyper-tungstenite-0.19.0/src/lib.rs:154`). The value at line 33 (`response`) is already
`hyper::Response<Full<Bytes>>`. The return at line 55 (`return response`) already works.
After D-03 the only change needed is:
- Return type annotation: `Full<Bytes>` → `FerroBody`
- The `From<Full<Bytes>> for FerroBody` impl makes `response` auto-coerce via `.into()` or
  explicit `response.map(FerroBody::Full)`.

**Site 1 — function signature, line 22:**
```rust
// BEFORE (line 22):
pub(crate) fn handle_ws_upgrade(
    mut req: hyper::Request<hyper::body::Incoming>,
) -> hyper::Response<Full<Bytes>> {

// AFTER:
pub(crate) fn handle_ws_upgrade(
    mut req: hyper::Request<hyper::body::Incoming>,
) -> hyper::Response<FerroBody> {
```

**Site 2 — 503 error response, lines 26–30:**
```rust
// BEFORE (lines 26–30):
return hyper::Response::builder()
    .status(503)
    .body(Full::new(Bytes::from("Broadcasting not configured")))
    .unwrap();

// AFTER:
return hyper::Response::builder()
    .status(503)
    .body(FerroBody::Full(Full::new(Bytes::from("Broadcasting not configured"))))
    .unwrap();
```

**Site 3 — 400 error response, lines 37–41:**
```rust
// BEFORE (lines 37–41):
return hyper::Response::builder()
    .status(400)
    .body(Full::new(Bytes::from("WebSocket upgrade failed")))
    .unwrap();

// AFTER:
return hyper::Response::builder()
    .status(400)
    .body(FerroBody::Full(Full::new(Bytes::from("WebSocket upgrade failed"))))
    .unwrap();
```

**Site 4 — WS 101 return, line 55:**
```rust
// BEFORE (line 55):
response   // type: hyper::Response<Full<Bytes>>

// AFTER — two valid options:
response.map(FerroBody::Full)  // explicit; clearest intent
// OR: response.map(Into::into) if From<Full<Bytes>> for FerroBody is in scope
```

---

### `framework/src/static_files.rs` (MODIFY — 2 sites)

**Analog:** self

**Site 1 — `try_serve_from_dir` return type, line 12:**
```rust
// BEFORE (line 12):
) -> Option<hyper::Response<Full<Bytes>>> {

// AFTER:
) -> Option<hyper::Response<FerroBody>> {
```

**Body construction site, lines 61–67:**
```rust
// BEFORE (lines 61–67):
let response = hyper::Response::builder()
    .status(200)
    .header("Content-Type", &content_type)
    .header("Content-Length", bytes.len().to_string())
    .header("Cache-Control", cache_control)
    .body(Full::new(Bytes::from(bytes)))
    .unwrap();

// AFTER — only .body() line changes:
    .body(FerroBody::Full(Full::new(Bytes::from(bytes))))
```

**Site 2 — `try_serve_static_file` return type, line 77:**
```rust
// BEFORE (line 77):
pub(crate) async fn try_serve_static_file(
    request_path: &str,
) -> Option<hyper::Response<Full<Bytes>>> {

// AFTER:
pub(crate) async fn try_serve_static_file(
    request_path: &str,
) -> Option<hyper::Response<FerroBody>> {
```

**Tests (lines 82–210) — `into_body().collect()` pattern used in tests, e.g. line 143:**
```rust
let body = resp.into_body().collect().await.unwrap().to_bytes();
// Works unchanged on FerroBody via BodyExt blanket impl
```

---

### `framework/src/middleware/pre_route.rs` (MODIFY — 1 site)

**Analog:** self

**Site 1 — `PreRouteResult` type alias, lines 32–33:**
```rust
// BEFORE (lines 32–33):
pub type PreRouteResult =
    Result<hyper::Request<hyper::body::Incoming>, hyper::Response<Full<Bytes>>>;

// AFTER:
pub type PreRouteResult =
    Result<hyper::Request<hyper::body::Incoming>, hyper::Response<FerroBody>>;
```

**Breaking change note for rustdoc:** This is a public type alias. After D-03, consumers
building the `Err` arm must use `HttpResponse::text("...").into_hyper()` (returns `FerroBody`
after the change) rather than `hyper::Response::builder().body(Full::new(...))` directly.
Add a rustdoc note on `PreRouteResult` documenting this. Follow the existing rustdoc style
in `pre_route.rs` lines 7–23.

---

### `framework/src/debug/mod.rs` (MODIFY — 7 sites, all mechanical)

**Analog:** self

**`json_response` helper — line 50:**
```rust
// BEFORE (line 50):
fn json_response<T: Serialize>(data: T, status: u16) -> hyper::Response<Full<Bytes>> {
    let body = serde_json::to_string_pretty(&data).unwrap_or_else(|_| "{}".to_string());
    hyper::Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(body)))
        .unwrap()
}

// AFTER:
fn json_response<T: Serialize>(data: T, status: u16) -> hyper::Response<FerroBody> {
    let body = serde_json::to_string_pretty(&data).unwrap_or_else(|_| "{}".to_string());
    hyper::Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(FerroBody::Full(Full::new(Bytes::from(body))))
        .unwrap()
}
```

**6 public function signatures (lines 60, 91, 115, 139, 174, 229):**
All follow the same pattern — only the return type annotation changes:
```rust
// BEFORE:
pub fn handle_routes() -> hyper::Response<Full<Bytes>> { ... }
pub fn handle_middleware() -> hyper::Response<Full<Bytes>> { ... }
pub fn handle_services() -> hyper::Response<Full<Bytes>> { ... }
pub fn handle_metrics() -> hyper::Response<Full<Bytes>> { ... }
pub async fn handle_queue_jobs() -> hyper::Response<Full<Bytes>> { ... }
pub async fn handle_queue_stats() -> hyper::Response<Full<Bytes>> { ... }

// AFTER: same but return type → hyper::Response<FerroBody>
// Bodies of these functions all delegate to json_response() — no body changes needed there
```

---

### `framework/src/http/mod.rs` (MODIFY — re-export additions)

**Analog:** self

**Current re-export block (lines 13–26) — where `SseEvent`/`SseStream` are added:**
```rust
// framework/src/http/mod.rs lines 1–26 — current state
pub mod action;
mod body;
pub mod cookie;
// ... other mods
mod response;
// ADD after existing mod declarations:
mod sse;   // (or pub mod sse if SseEvent/SseStream need to be in the http:: namespace directly)

pub use action::{...};
pub use body::{collect_body, parse_form, parse_json};
// ADD to body re-export or as separate line:
pub use body::FerroBody;   // if FerroBody lives in body.rs
// ADD new sse re-export:
pub use sse::{SseEvent, SseStream};
pub use response::{HttpResponse, InertiaRedirect, Redirect, RedirectRouteBuilder, Response, ResponseExt};
```

---

### `framework/src/lib.rs` (MODIFY — public surface re-export)

**Analog:** self

**Current `http` re-export block (lines 111–116):**
```rust
// framework/src/lib.rs lines 111–116 — current pub use http::{...} block
pub use http::{
    bytes, json, request_host, text, validate_mime, validate_size, Cookie, CookieOptions,
    FormRequest, FromParam, FromRequest, HttpResponse, InertiaRedirect, MultipartForm,
    PaginationLinks, PaginationMeta, Redirect, Request, Resource, ResourceCollection, ResourceMap,
    Response, ResponseExt, SameSite, UploadedFile,
};

// AFTER — add SseEvent, SseStream (and FerroBody if public-facing):
pub use http::{
    bytes, json, request_host, text, validate_mime, validate_size, Cookie, CookieOptions,
    FormRequest, FromParam, FromRequest, FerroBody, HttpResponse, InertiaRedirect, MultipartForm,
    PaginationLinks, PaginationMeta, Redirect, Request, Resource, ResourceCollection, ResourceMap,
    Response, ResponseExt, SameSite, SseEvent, SseStream, UploadedFile,
};
```

---

## Shared Patterns

### `Full<Bytes>` → `FerroBody` Mechanical Rule
**Source:** every modified file
**Apply to:** all 6 modified source files in the blast radius

The migration at every `Full<Bytes>` return site is mechanical — two kinds of change only:
1. Return type annotation: `hyper::Response<Full<Bytes>>` → `hyper::Response<FerroBody>`
2. Body construction: `.body(Full::new(x))` → `.body(FerroBody::Full(Full::new(x)))`

The `From<Full<Bytes>> for FerroBody` impl (on `FerroBody`) enables `.map(FerroBody::Full)` at
the websocket.rs line 55 return site. Everywhere else, the construction is explicit.

### `BodyExt::collect()` in tests — unchanged
**Source:** `framework/src/http/response.rs` line 629, `framework/src/static_files.rs` line 143,
`framework/src/server.rs` line 443
**Apply to:** all existing tests that call `into_body().collect()`

`BodyExt` is a blanket impl — it works on any `T: Body`. Changing the body type from
`Full<Bytes>` to `FerroBody` requires no changes to existing test calls of `.collect()`.

### Inline test module convention
**Source:** `framework/src/http/response.rs` lines 527–700 (most complete example)
**Apply to:** `framework/src/http/sse.rs` inline tests

All tests live in `#[cfg(test)] mod tests { use super::*; ... }` at the bottom of the file.
Sync tests use `#[test]`; async tests use `#[tokio::test]`.

### `hyper::Response::builder()` chain pattern
**Source:** `framework/src/debug/mod.rs` lines 52–56 (simplest example), `server.rs` lines 356–360
**Apply to:** `HttpResponse::sse()` factory, `FerroBody` body construction sites

```rust
// The canonical builder chain used throughout:
hyper::Response::builder()
    .status(status_code)
    .header("Header-Name", "value")
    .body(/* body */)
    .unwrap()
// Use .unwrap() — all framework-internal response builds use this (never user-controlled headers)
```

---

## No Analog Found

None. All new and modified files have close analogs in the codebase. The `FerroBody` enum
`Body` impl is the only construct with no direct precedent in the project — use RESEARCH.md
Pattern 3 (lines 265–307) as the authoritative spec for that impl.

---

## Metadata

**Analog search scope:** `framework/src/`
**Files read:** `http/response.rs`, `http/mod.rs`, `http/body.rs`, `server.rs`, `websocket.rs`,
  `static_files.rs`, `middleware/pre_route.rs`, `debug/mod.rs`, `lib.rs`
**External source read:** `hyper-tungstenite-0.19.0/src/lib.rs` (resolves RESEARCH open question A2)
**Pattern extraction date:** 2026-06-08

**Open question resolution:**
- RESEARCH A2 (websocket body type): RESOLVED — `hyper_tungstenite::upgrade` already returns
  `Response<Full<Bytes>>`. Line 55 `return response` needs `.map(FerroBody::Full)` added, but
  NO special `FerroBody::Upgrade` variant is needed.
- RESEARCH A3/A4 (Unpin): Confirmed by the existing `websocket.rs` `tokio::time::interval`
  usage at line 79 without pin-project — the same pattern holds for `SseStream`.
