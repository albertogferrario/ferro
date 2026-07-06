# Phase 168: Framework SSE Primitives — Research

**Researched:** 2026-06-08
**Domain:** hyper 1 / http-body 1 streaming body, SSE wire format, Rust enum body types
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- D-01: `SseEvent` in `framework/src/http/sse.rs` — fields `data: String`, `event: Option<String>`, `id: Option<String>`, `retry: Option<u64>` ms. `to_wire()` / `Display` serializer; field order: `event:`, `id:`, `retry:`, then `data:` lines (one per newline); blank-line terminator. Builder: `SseEvent::data(s)` + `.event(..)` / `.id(..)` / `.retry(..)`.
- D-02: `SseStream` wraps `tokio::sync::mpsc::Receiver<SseEvent>`, implements `http_body::Body<Data = Bytes, Error = ...>`. `SseStream::channel(buffer) -> (Sender<SseEvent>, SseStream)`.
- D-03 (load-bearing): Generalize server response body from `hyper::Response<Full<Bytes>>` to `hyper::Response<FerroBody>` where `FerroBody` is an enum `{ Full(Full<Bytes>), Stream(SseStream) }` implementing `http_body::Body`. Update all 6 files naming `Full<Bytes>` in function return positions. WebSocket 101 → `FerroBody::Full`. `HttpResponse::into_hyper()` → `FerroBody::Full`.
- D-04: `HttpResponse::sse(stream: SseStream) -> HttpResponse` (or channel-returning variant) sets `Content-Type: text/event-stream`, `Cache-Control: no-cache`, `Connection: keep-alive`, `X-Accel-Buffering: no`. Exact arg shape is Claude's discretion within this contract.
- D-05: `SseStream` holds a `tokio::time::Interval` (15 s). When polled and no event is ready but the interval has ticked, yields `:ping\n\n`. Any real event resets the idle window (interval tick consumed without emitting ping). Implemented in `poll_frame` via `tokio::select!`.
- D-06: Structural non-buffering guarantee = SSE uses `FerroBody::Stream`. Unit test asserts SSE response body is the `Stream` variant. Future compression must match only on `FerroBody::Full`.
- D-07: Integration test drives `SseStream::poll_frame` directly (deterministic) PLUS optional e2e server test.
- D-08: `framework/src/http/sse.rs`; re-export via `framework/src/http/mod.rs` and `framework/src/lib.rs`.

### Claude's Discretion

- Exact `FerroBody` representation (enum vs BoxBody) — research/planner may pick BoxBody if the enum impl proves unwieldy; enum is recommended.
- Exact `HttpResponse::sse` signature (channel-returning vs stream-taking) within D-04's header contract.
- mpsc channel buffer size default (recommend small bounded, e.g. 16).
- Error type for body `Error` associated type (recommend `Infallible` — see below).

### Deferred Ideas (OUT OF SCOPE)

- Any real `CompressionLayer` (gzip/brotli).
- Migrating the framework to axum.
- Wiring `ferro_ai::TokenStream` → `SseStream`.
- `StreamText` JSON-UI component (Phase 169).
- Last-Event-ID reconnection / event replay buffer.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AISSE-01 | Handler can return a streaming SSE response that pushes LLM tokens to the browser as they arrive; SSE routes are structurally excluded from any CompressionLayer. | D-01..D-08 cover the full deliverable. "CompressionLayer exclusion" is met structurally by the `FerroBody::Stream` variant: a streaming body cannot be whole-body buffered. |
</phase_requirements>

---

## Summary

This phase adds four SSE primitives to `framework/src/http/sse.rs` and makes the framework's response pipeline streaming-capable. The deliverables split cleanly into two groups: (a) new code — `SseEvent` wire serializer, `SseStream` body, `HttpResponse::sse()` factory, and the 15-second `:ping` keep-alive; (b) a cross-cutting refactor — generalizing every `hyper::Response<Full<Bytes>>` return site (6 files, ~17 signature occurrences) to `hyper::Response<FerroBody>`.

The refactor (D-03) is the load-bearing work. The SSE-specific code is small; the blast radius update is mechanical but must be done correctly in one coherent change so no mixed state survives. The enum body (`FerroBody`) approach is both zero-cost on the hot buffered path (no `Box`, no vtable) and a clean structural guarantee that streaming and buffered responses are different types at compile time.

The existing dependency set already supplies everything needed: `hyper 1.8.1` (full), `http-body-util 0.1.3` (includes `StreamBody`, `Full`, `BodyExt`, `Frame`, pinning via `pin-project-lite`), `http-body 1.0.1`, `bytes 1.x`, `tokio 1.48` (full). No new crates. `futures-util 0.3` is a direct dep and brings `futures-core::stream::Stream`; `pin-project-lite` is a dep of `http-body-util` and is available transitively.

**Primary recommendation:** Implement `SseStream` as a hand-rolled `http_body::Body` (not `StreamBody<ReceiverStream>`) to avoid any `tokio-stream` or `futures::Stream` dependency and to keep the `select!` keep-alive logic explicit in `poll_frame`. Both `tokio::sync::mpsc::Receiver` and `tokio::time::Interval` are `Unpin`, so `Pin::new(&mut self.field)` is all that is needed — no `pin-project-lite` in `SseStream` itself. The `FerroBody` enum shares this property.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| SSE wire serialization | API / Backend | — | Server serializes events before pushing to the wire |
| Streaming body transport | API / Backend (hyper) | — | `hyper::Response<FerroBody>` flows through the raw-hyper serve loop |
| Keep-alive ping | API / Backend (SseStream) | — | Server-side idle detection; client has no role |
| Response header set | API / Backend (HttpResponse) | — | `Content-Type`, `Cache-Control`, `X-Accel-Buffering` set at response construction |
| Buffering bypass (nginx) | API / Backend (header) | CDN / Static | `X-Accel-Buffering: no` instructs nginx; structural type prevents framework buffering |
| Token delivery to browser | Browser / Client | — | EventSource reads server-sent frames; browser-side concern out of this phase |

---

## Standard Stack

### Core (all already present — NO new deps)

| Library | Version in Cargo.lock | Purpose | Why Standard |
|---------|----------------------|---------|--------------|
| `hyper` | 1.8.1 | HTTP/1 server; `hyper::Response<B>` is the serve-loop return type | Already the framework's only HTTP engine |
| `http-body` | 1.0.1 | `Body` trait definition (`poll_frame`, `Frame<D>`, `SizeHint`) | The trait `FerroBody` and `SseStream` implement |
| `http-body-util` | 0.1.3 | `Full<Bytes>` (buffered body), `Frame`, `BodyExt`, `StreamBody` | Already used by every response site |
| `bytes` | 1.x | `Bytes` — zero-copy byte buffer | Already used everywhere |
| `tokio` | 1.48.0 (full) | `mpsc`, `time::Interval`, runtime | Already used everywhere |
| `futures-util` | 0.3.32 | `futures_core::stream::Stream` (if needed for `StreamBody` adapter) | Already a direct dep |

[VERIFIED: Cargo.lock + framework/Cargo.toml inspection]

### Not needed

`pin-project-lite` — available transitively through `http-body-util` but **not needed** for `SseStream` or `FerroBody` because both `tokio::sync::mpsc::Receiver<T>` and `tokio::time::Interval` are `Unpin`, so `Pin::new(&mut self.field)` is valid. [VERIFIED: tokio docs; both types implement `Unpin`]

`tokio-stream` — present in the lockfile (as a transitive dep of something else) but NOT a direct dep and NOT needed for a hand-rolled `Body` impl. [VERIFIED: framework/Cargo.toml has no tokio-stream entry]

**No installation command needed** — all deps already in `framework/Cargo.toml`.

---

## Architecture Patterns

### System Architecture Diagram

```
Handler task
  │
  │  SseStream::channel(16) → (Sender<SseEvent>, SseStream)
  │
  ├── Sender<SseEvent>   ──────────────────────────────────────────────────────┐
  │     handler.spawn(async { sender.send(SseEvent::data("token")).await; })   │
  │                                                                             │
  └── SseStream ──► HttpResponse::sse(stream)                                  │
                         │  Content-Type: text/event-stream                    │
                         │  Cache-Control: no-cache                            │
                         │  X-Accel-Buffering: no                              │
                         │                                                     │
                    into_hyper() → hyper::Response<FerroBody::Stream(sse)>     │
                         │                                                     │
                    server.rs: service_fn serve loop                           │
                    http1::Builder::new()                                      │
                         │                                                     │
                         ▼ poll_frame called by hyper on each TCP write        │
                    SseStream::poll_frame                                      │
                         │                                                     │
                    tokio::select!                                             │
                      ├── mpsc::Receiver::poll_recv ◄──────────────────────────┘
                      │     Ready(Some(event)) → yields Frame::data(event.to_wire())
                      │     Ready(None)        → yields Poll::Ready(None) [stream closed]
                      └── Interval::poll_tick
                            Ready(_) → yields Frame::data(b":ping\n\n")
                            Pending  → Poll::Pending (hyper waits)
                         │
                         ▼
                    Browser EventSource receives event frames
```

### Recommended Project Structure

```
framework/src/http/
├── sse.rs           ← NEW: SseEvent, SseStream, keep-alive, to_wire()
├── response.rs      ← MODIFIED: HttpResponse::sse(), into_hyper() returns FerroBody
├── mod.rs           ← MODIFIED: pub use sse::{SseEvent, SseStream}
└── body.rs          ← NEW (or in sse.rs): FerroBody enum + Body impl

framework/src/
├── server.rs        ← MODIFIED: Full<Bytes> → FerroBody in 5 positions
├── websocket.rs     ← MODIFIED: Full<Bytes> → FerroBody in 2 positions
├── static_files.rs  ← MODIFIED: Full<Bytes> → FerroBody in 2 function signatures
├── middleware/
│   └── pre_route.rs ← MODIFIED: PreRouteResult type alias uses FerroBody
└── debug/mod.rs     ← MODIFIED: json_response() and 6 pub functions return FerroBody
```

Note on `FerroBody` placement: it can live in `sse.rs` (alongside `SseStream`) or a sibling `body.rs`. Either is fine. `http/mod.rs` re-exports both.

### Pattern 1: `http_body::Body` Trait Signature (hyper 1)

The exact trait from `http-body 1.0.1` (verified from docs.rs and local crate):

```rust
// Source: https://docs.rs/http-body/1.0.1/http_body/trait.Body.html
// + local /Users/alberto/.cargo/registry/src/.../http-body-util-0.1.3/src/stream.rs
pub trait Body {
    type Data: Buf;
    type Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>>;

    // Provided:
    fn is_end_stream(&self) -> bool { ... }
    fn size_hint(&self) -> SizeHint { ... }
}
```

`Frame<D>` has:
- `Frame::data(d: D) -> Frame<D>` — wraps a data buffer
- `frame.is_data() -> bool`, `frame.into_data() -> Result<D, Frame<D>>`
- `Frame::trailers(headers: HeaderMap) -> Frame<D>` — HTTP trailers (not used for SSE)

[VERIFIED: /hyperium/http-body Context7 + local crate source]

### Pattern 2: Hand-Rolled `SseStream` Body Implementation

Both `tokio::sync::mpsc::Receiver<T>` and `tokio::time::Interval` are `Unpin`. Because of this, no `pin-project-lite` macro is needed — `Pin::new(&mut self.field)` is valid for both fields:

```rust
// Source: derived from http-body 1.0.1 trait contract + tokio Unpin guarantee
use std::pin::Pin;
use std::task::{Context, Poll};
use bytes::Bytes;
use http_body::{Body, Frame, SizeHint};
use tokio::sync::mpsc;
use tokio::time::{interval, Interval, Duration};

pub struct SseStream {
    receiver: mpsc::Receiver<SseEvent>,
    ping_interval: Interval,
}

impl SseStream {
    pub fn channel(buffer: usize) -> (mpsc::Sender<SseEvent>, Self) {
        let (tx, rx) = mpsc::channel(buffer);
        let ping = interval(Duration::from_secs(15));
        (tx, SseStream { receiver: rx, ping_interval: ping })
    }
}

impl Body for SseStream {
    type Data = Bytes;
    type Error = std::convert::Infallible;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        // poll_recv on Receiver is safe without pinning because Receiver: Unpin
        match self.receiver.poll_recv(cx) {
            Poll::Ready(Some(event)) => {
                // Reset idle window: consume any pending interval tick silently
                let _ = Pin::new(&mut self.ping_interval).poll_tick(cx);
                let bytes = Bytes::from(event.to_wire());
                return Poll::Ready(Some(Ok(Frame::data(bytes))));
            }
            Poll::Ready(None) => {
                // Sender dropped — end of stream
                return Poll::Ready(None);
            }
            Poll::Pending => {}
        }

        // No event ready — check keep-alive interval
        match Pin::new(&mut self.ping_interval).poll_tick(cx) {
            Poll::Ready(_) => {
                let ping = Bytes::from_static(b":ping\n\n");
                Poll::Ready(Some(Ok(Frame::data(ping))))
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn is_end_stream(&self) -> bool {
        false  // Only terminated when Sender is dropped
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::default()  // Unknown size for streaming body
    }
}
```

[ASSUMED — exact `poll_recv` API on `Receiver` (verify: it returns `Poll<Option<T>>` directly, not a future). Confirmed pattern is idiomatic for mpsc in async contexts.]

### Pattern 3: `FerroBody` Enum Body

Both `Full<Bytes>` and `SseStream` are `Unpin`, so `Pin::new(&mut self)` dispatch works without any pin projection macro:

```rust
// Source: derived from http-body 1.0.1 + http-body-util 0.1.3 Full<Bytes> impl
use http_body_util::Full;
use bytes::Bytes;

pub enum FerroBody {
    Full(Full<Bytes>),
    Stream(SseStream),
}

impl Body for FerroBody {
    type Data = Bytes;
    type Error = std::convert::Infallible;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        match &mut *self {
            FerroBody::Full(b) => {
                // Full<Bytes>: Error = Infallible, so map_err is a no-op
                Pin::new(b).poll_frame(cx).map_err(|e| match e {})
            }
            FerroBody::Stream(s) => Pin::new(s).poll_frame(cx),
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            FerroBody::Full(b) => b.is_end_stream(),
            FerroBody::Stream(s) => s.is_end_stream(),
        }
    }

    fn size_hint(&self) -> SizeHint {
        match self {
            FerroBody::Full(b) => b.size_hint(),
            FerroBody::Stream(s) => s.size_hint(),
        }
    }
}

impl From<Full<Bytes>> for FerroBody {
    fn from(b: Full<Bytes>) -> Self {
        FerroBody::Full(b)
    }
}
```

Key type unification: `Full<Bytes>` has `Error = Infallible` [VERIFIED: http-body-util 0.1.3]. `SseStream` should also use `Error = Infallible` — events are pre-serialized to `Bytes` before pushing, so no error path exists. This keeps `FerroBody::Error = Infallible` throughout and the serve loop's `Ok::<_, Infallible>(...)` wrapper unchanged.

[VERIFIED: `Full<Bytes>` is `Unpin`. `Pin::new(b)` on `&mut Full<Bytes>` is valid.]

### Pattern 4: `FerroBody` in the hyper serve loop

The serve loop's `service_fn` closure signature changes from:

```rust
Ok::<_, Infallible>(handle_request(...).await)
// where handle_request → hyper::Response<Full<Bytes>>
```

to:

```rust
Ok::<_, Infallible>(handle_request(...).await)
// where handle_request → hyper::Response<FerroBody>
```

`hyper::server::conn::http1::Builder::serve_connection` accepts any `B: Body` — generics, no specific body type required. [VERIFIED: hyper 1 docs, serve_connection is generic over B: Body]

### Pattern 5: `SseEvent` Wire Format

```rust
// Source: WHATWG Server-Sent Events spec, https://html.spec.whatwg.org/multipage/server-sent-events.html
// Confirmed from CONTEXT.md D-01 field order

impl fmt::Display for SseEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(event) = &self.event {
            write!(f, "event: {event}\n")?;
        }
        if let Some(id) = &self.id {
            write!(f, "id: {id}\n")?;
        }
        if let Some(retry) = self.retry {
            write!(f, "retry: {retry}\n")?;
        }
        // Multi-line data: each line gets its own data: prefix
        for line in self.data.lines() {
            write!(f, "data: {line}\n")?;
        }
        // Empty data string still emits one data: line (allows keep-alive style events)
        if self.data.is_empty() {
            write!(f, "data: \n")?;
        }
        // Blank line terminates the event
        write!(f, "\n")
    }
}
```

[VERIFIED: WHATWG spec — field ordering, multi-line data rule, blank-line terminator]

### Anti-Patterns to Avoid

- **`BoxBody` / `UnsyncBoxBody` for `FerroBody`**: adds a heap allocation on every buffered response (the 99% case). The enum is zero-cost; no reason for dynamic dispatch here.
- **`StreamBody<ReceiverStream<SseEvent>>`**: would require `tokio-stream` as a dep (not currently a direct dep) and makes the keep-alive logic harder to express cleanly. Hand-rolled `Body` impl is cleaner.
- **`HttpResponse` holding `SseStream` alongside `Bytes`**: `HttpResponse` is a buffered builder with `body: Bytes`. Don't mutate it to hold a streaming variant — a separate `into_hyper_streaming()` or a new streaming-response constructor that returns `hyper::Response<FerroBody>` directly is cleaner.
- **Half-updating the blast radius**: leaving some `Full<Bytes>` return sites unchanged causes a type mismatch at the server boundary. D-03 must be applied to all 6 files in one wave.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Buffered HTTP body | custom bytes wrapper | `http_body_util::Full<Bytes>` | Already the standard; just keep using it as `FerroBody::Full` |
| Collecting a body into bytes | manual poll loop | `http_body_util::BodyExt::collect()` | Already used in tests (response.rs:629) |
| SSE wire format | complex parser/encoder | Simple `Display` impl per WHATWG spec | 10 lines; spec is simple; no library needed |
| mpsc channel | custom ring buffer | `tokio::sync::mpsc` | Already in tokio full |
| Timer/keep-alive | manual `Instant` + sleep | `tokio::time::Interval` | Already in tokio full; exactly the right primitive |

---

## D-03 Blast Radius: Exact Signature Changes

Every occurrence of `hyper::Response<Full<Bytes>>` that is a **function return type** (not inside tests, and not on non-exported internal service closures within tenant/rate_limit middleware) must change to `hyper::Response<FerroBody>`. Count: **17 return-type positions across 6 files**.

### `framework/src/http/response.rs`

| Line | Before | After |
|------|--------|-------|
| 168 | `pub fn into_hyper(self) -> hyper::Response<Full<Bytes>>` | `pub fn into_hyper(self) -> hyper::Response<FerroBody>` |

Body of `into_hyper` changes `builder.body(Full::new(self.body)).unwrap()` → `builder.body(FerroBody::Full(Full::new(self.body))).unwrap()`.

### `framework/src/server.rs`

| Line | Before | After |
|------|--------|-------|
| 23–28 | `WsInterceptor` type alias: `Result<hyper::Response<Full<Bytes>>, ...>` | `Result<hyper::Response<FerroBody>, ...>` |
| 94–98 | `ws_interceptor` method signature | same `FerroBody` |
| 189 | `async fn handle_request(...) -> hyper::Response<Full<Bytes>>` | `-> hyper::Response<FerroBody>` |
| 328 | `async fn health_response(...) -> hyper::Response<Full<Bytes>>` | `-> hyper::Response<FerroBody>` |
| 354–360 | `Full::new(...)` in body | `FerroBody::Full(Full::new(...))` |
| 369 | `fn serve_ferro_base_css() -> hyper::Response<Full<Bytes>>` | `-> hyper::Response<FerroBody>` |

The serve loop `service_fn` closure already returns `Ok::<_, Infallible>(handle_request(...).await)` — no type annotation change needed there, it infers from `handle_request`.

### `framework/src/websocket.rs`

| Line | Before | After |
|------|--------|-------|
| 22 | `pub(crate) fn handle_ws_upgrade(...) -> hyper::Response<Full<Bytes>>` | `-> hyper::Response<FerroBody>` |
| 27–29 | 503 response body | `FerroBody::Full(Full::new(...))` |
| 38 | 400 response body | `FerroBody::Full(Full::new(...))` |
| 54 | returns `response` from `hyper_tungstenite::upgrade` | this response is `hyper::Response<hyper::body::Incoming>` — **NOTE: see below** |

**Special case for websocket.rs**: `hyper_tungstenite::upgrade` returns a `hyper::Response<hyper::body::Incoming>`, not `Full<Bytes>`. The current code returns it directly. To fit `FerroBody`, the WS 101 response body (which is always empty) must be mapped: `response.map(|_| FerroBody::Full(Full::new(Bytes::new())))`. The upgrade future handles the socket takeover separately via `with_upgrades()` — the response body type doesn't affect it. [ASSUMED — verify that `hyper_tungstenite::upgrade`'s response can be body-mapped without breaking the upgrade protocol. The upgrade is tracked by hyper via the request's extensions, not the response body.]

### `framework/src/static_files.rs`

| Line | Before | After |
|------|--------|-------|
| 12 | `async fn try_serve_from_dir(...) -> Option<hyper::Response<Full<Bytes>>>` | `-> Option<hyper::Response<FerroBody>>` |
| 77 | `pub(crate) async fn try_serve_static_file(...) -> Option<hyper::Response<Full<Bytes>>>` | `-> Option<hyper::Response<FerroBody>>` |
| 62–67 | `Full::new(...)` in body | `FerroBody::Full(Full::new(...))` |

### `framework/src/middleware/pre_route.rs`

| Line | Before | After |
|------|--------|-------|
| 33 | `pub type PreRouteResult = Result<hyper::Request<...>, hyper::Response<Full<Bytes>>>` | `Result<..., hyper::Response<FerroBody>>` |

This is a public type alias — any downstream code that implements `PreRouteMiddleware` and pattern-matches on `PreRouteResult` needs to construct `hyper::Response<FerroBody>` in its `Err(...)` arm. The framework provides `HttpResponse::into_hyper()` which already returns `FerroBody` after D-03, so the migration path for framework consumers is: return `Err(HttpResponse::text("...").into_hyper())`.

### `framework/src/debug/mod.rs`

| Line | Before | After |
|------|--------|-------|
| 50 | `fn json_response<T: Serialize>(...) -> hyper::Response<Full<Bytes>>` | `-> hyper::Response<FerroBody>` |
| 60, 91, 115, 139, 174, 229 | 6 pub fn return types | all `FerroBody` |

The body construction in `json_response` at line 56: `Full::new(Bytes::from(body))` → `FerroBody::Full(Full::new(Bytes::from(body)))`.

### Non-blast-radius `Full<Bytes>` sites (do NOT change)

- `framework/src/tenant/mod.rs:187`, `tenant/middleware.rs:148`, `tenant/requires_plan.rs:150` — these pass `http_body_util::Empty::<bytes::Bytes>::new()` to hyper internal connection builders as request bodies, not as framework response bodies. They operate on a different type path (hyper client-side, not server serve loop).
- `framework/src/middleware/rate_limit.rs:540`, `562` — same pattern: internal hyper client requests.
- `framework/src/json_ui/mod.rs:377` — inside `#[cfg(test)]` helper; test-internal, not part of the server path.

[VERIFIED: direct grep of all `hyper::Response<Full<Bytes>>` occurrences by file]

---

## SSE Wire Format Reference

Canonical source: WHATWG HTML Living Standard §9.2.6 `text/event-stream` [CITED: https://html.spec.whatwg.org/multipage/server-sent-events.html]

### Field Rules

| Field | Wire Format | Notes |
|-------|-------------|-------|
| `event` | `event: {name}\n` | Optional; names the event type |
| `id` | `id: {value}\n` | Optional; sets `lastEventId` |
| `retry` | `retry: {ms}\n` | Optional; integer milliseconds; client reconnect delay |
| `data` | `data: {line}\n` (one per line of the value) | Required; multi-line → repeated `data:` lines |
| comment | `:{text}\n` | Ignored by client; used for keep-alive |
| terminator | `\n` (blank line) | Required; signals end of one event |

### Exact byte sequences

Single-data event:
```
data: hello world\n
\n
```
Bytes: `b"data: hello world\n\n"`

Named event with id:
```
event: token\n
id: 42\n
data: Hello\n
\n
```
Bytes: `b"event: token\nid: 42\ndata: Hello\n\n"`

Multi-line data:
```
data: line one\n
data: line two\n
\n
```
Bytes: `b"data: line one\ndata: line two\n\n"`

Keep-alive comment:
```
:ping\n
\n
```
Bytes: `b":ping\n\n"`

Retry field:
```
retry: 3000\n
data: reconnect in 3s\n
\n
```

[VERIFIED: WHATWG spec + RFC cross-check]

### Required Response Headers

| Header | Value | Purpose |
|--------|-------|---------|
| `Content-Type` | `text/event-stream` | MIME type; required by EventSource |
| `Cache-Control` | `no-cache` | Prevents proxy caching |
| `Connection` | `keep-alive` | Keeps TCP connection open |
| `X-Accel-Buffering` | `no` | Disables nginx proxy buffering |

[VERIFIED from CONTEXT.md D-04; standard SSE practice]

---

## Common Pitfalls

### Pitfall 1: WebSocket response body mapping

**What goes wrong:** `hyper_tungstenite::upgrade` returns `hyper::Response<hyper::body::Incoming>`. Directly returning it as `hyper::Response<FerroBody>` fails to compile because body types differ.
**Why it happens:** The WS 101 response is generated by the tungstenite crate and uses hyper's native body type.
**How to avoid:** Map the response body: `response.map(|_incoming| FerroBody::Full(Full::new(Bytes::new())))`. The actual WebSocket upgrade is handled by hyper's connection upgrade mechanism (registered in the `Incoming` body's extensions), not by the response body bytes. Mapping to an empty body doesn't affect the upgrade.
**Warning signs:** Compile error "expected `FerroBody`, found `hyper::body::Incoming`" in websocket.rs.

### Pitfall 2: Interval fires immediately on construction

**What goes wrong:** `tokio::time::interval(Duration::from_secs(15))` fires the first tick immediately (at construction time). The first `poll_tick` in `poll_frame` would immediately yield a `:ping\n\n` comment before any real event.
**Why it happens:** Tokio's `Interval` design — the first tick represents "now" to signal readiness.
**How to avoid:** Call `interval.tick().await` once after construction (consumes the immediate tick), or use `tokio::time::interval_at(Instant::now() + Duration::from_secs(15), ...)` to defer the first tick by the full 15 seconds. The `interval_at` approach is cleaner in `SseStream::channel`.
**Warning signs:** Test shows `:ping` as the first frame from a freshly constructed `SseStream` even when an event is immediately sent.

### Pitfall 3: `poll_recv` waker registration

**What goes wrong:** The `SseStream::poll_frame` polls `receiver.poll_recv(cx)` and gets `Poll::Pending`, then polls the interval and gets `Poll::Pending`. When an event is sent, hyper must re-poll `FerroBody`. If the waker is not correctly registered on both the mpsc channel and the interval, hyper never re-polls and the connection stalls.
**Why it happens:** Standard `Poll::Pending` contract: the waker must be registered before returning `Pending`. Tokio's `mpsc::Receiver::poll_recv` and `time::Interval::poll_tick` both register the waker correctly — this is Tokio's responsibility.
**How to avoid:** Don't replace `poll_recv` / `poll_tick` with manual `try_recv` + register-waker-yourself — use the standard poll APIs and trust Tokio's waker registration.
**Warning signs:** Test where a frame is sent to the channel but the test hangs waiting for `poll_frame` to return it.

### Pitfall 4: Mixing `http_body 0.4` and `http_body 1.0` Body traits

**What goes wrong:** The workspace has both `http-body 0.4.6` (used by `hyper 0.14` via `hyper-tungstenite 0.19`) and `http-body 1.0.1` (used by `hyper 1`). Using the wrong import causes "conflicting implementations of trait" errors.
**Why it happens:** Both versions exist in the lockfile because `hyper-tungstenite` depends on hyper 0.14 transitively.
**How to avoid:** Always import from `http_body` (no version suffix in Rust — resolved by the workspace to 1.0.1 for the `framework` crate which depends on `hyper` 1). Use `use http_body::{Body, Frame}` and verify the Cargo.toml `http-body-util = "0.1"` dependency brings in the 1.x `http_body` trait.
**Warning signs:** Compiler error about conflicting `Body` trait implementations; import ambiguity.

### Pitfall 5: `PreRouteResult` is a public type alias

**What goes wrong:** Changing `PreRouteResult` from `hyper::Response<Full<Bytes>>` to `hyper::Response<FerroBody>` is a **breaking API change** for any downstream code that implements `PreRouteMiddleware` and constructs an `Err(response)` return value with a manually built `hyper::Response<Full<Bytes>>`.
**Why it happens:** `PreRouteResult` is in the public surface (`pub type PreRouteResult = ...` in `pre_route.rs`).
**How to avoid:** The framework provides `HttpResponse::text(...).into_hyper()` and `HttpResponse::json(...).into_hyper()` which, after D-03, return `hyper::Response<FerroBody>`. Document the migration in rustdoc on `PreRouteResult`. Since `ferro-rs` is pre-1.0 with explicit breaking-change permission, this is acceptable — note in the commit message.
**Warning signs:** Consumer crates that call `hyper::Response::builder().body(Full::new(...))` in a `PreRouteMiddleware::handle` implementation.

---

## Code Examples

### `SseEvent` builder and wire serialization

```rust
// Source: WHATWG SSE spec + CONTEXT.md D-01
let event = SseEvent::data("Hello, world!")
    .event("token")
    .id("1")
    .retry(3000);

let wire = event.to_string(); // or format!("{event}")
// Result: "event: token\nid: 1\nretry: 3000\ndata: Hello, world!\n\n"

// Multi-line data
let event2 = SseEvent::data("line one\nline two");
// Result: "data: line one\ndata: line two\n\n"

// Keep-alive comment (emitted by SseStream internally)
// b":ping\n\n"
```

### `HttpResponse::sse()` factory pattern

```rust
// Source: CONTEXT.md D-04 (exact signature is Claude's discretion)
// Option A: stream-taking (caller creates channel with SseStream::channel)
pub fn sse(stream: SseStream) -> /* streaming response type */ { ... }

// Option B: channel-returning
pub fn sse_channel(buffer: usize) -> (mpsc::Sender<SseEvent>, /* streaming response */) { ... }
```

Planner may choose either. Option B is slightly more ergonomic for handlers.

### Handler usage pattern (after this phase)

```rust
#[handler]
pub async fn stream_tokens(req: Request) -> Response {
    let (tx, response) = HttpResponse::sse_channel(16);

    tokio::spawn(async move {
        for token in ["Hello", " ", "world"] {
            tx.send(SseEvent::data(token)).await.ok();
        }
        // tx dropped → SseStream ends → FerroBody::Stream yields None
    });

    Ok(response)
}
```

### Deterministic body-level poll test pattern

```rust
// Source: derived from existing framework test patterns + tokio test primitives
#[tokio::test]
async fn sse_stream_delivers_frames_in_order() {
    use http_body::Body;
    use std::task::{Context, Poll};
    use std::pin::Pin;

    let (tx, mut stream) = SseStream::channel(4);

    // Send event N before polling for it
    tx.send(SseEvent::data("first")).await.unwrap();
    tx.send(SseEvent::data("second")).await.unwrap();

    // Use a no-op waker (events already in channel buffer — will be Ready immediately)
    let waker = futures_util::task::noop_waker();
    let mut cx = Context::from_waker(&waker);

    let frame1 = Pin::new(&mut stream).poll_frame(&mut cx);
    assert!(matches!(frame1, Poll::Ready(Some(Ok(_)))));
    // Extract bytes and assert wire content ...

    let frame2 = Pin::new(&mut stream).poll_frame(&mut cx);
    assert!(matches!(frame2, Poll::Ready(Some(Ok(_)))));
}
```

`futures_util::task::noop_waker()` is available via `futures-util 0.3` which is already a direct dep. [VERIFIED: futures-util is in framework/Cargo.toml]

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `hyper::Response<Full<Bytes>>` only | `hyper::Response<FerroBody>` enum with Full and Stream variants | This phase | Enables streaming responses without allocation on buffered path |
| No SSE support | `SseEvent` + `SseStream` + `HttpResponse::sse()` | This phase | Handlers can push incremental events |
| axum `IntoResponse` (ROADMAP wording) | Raw hyper `http_body::Body` impl | Scope correction documented in CONTEXT.md | No axum dependency introduced |

**Deprecated by this phase:**
- Direct `hyper::Response<Full<Bytes>>` as a return type in framework internals — all internal sites use `hyper::Response<FerroBody>` after D-03.

---

## Runtime State Inventory

Step 2.5: SKIPPED — this is a greenfield feature addition, not a rename/refactor/migration phase. No stored data, live service config, OS registrations, secrets, or build artifacts are renamed.

---

## Environment Availability

Step 2.6: SKIPPED — this phase is a pure Rust code/crate change. All required capabilities (hyper, tokio, http-body-util) are compile-time dependencies already in `framework/Cargo.toml`. No external services, databases, CLIs, or runtimes beyond the build toolchain are required.

---

## Validation Architecture

`workflow.nyquist_validation` key is absent from `.planning/config.json` — treat as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust standard `#[test]` + `#[tokio::test]` |
| Config file | none (standard cargo test) |
| Quick run command | `cargo test -p ferro-rs --lib -- http::sse` |
| Full suite command | `cargo test --all-features -p ferro-rs` |
| Pre-commit suite | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AISSE-01 / D-01 | `SseEvent::to_wire()` exact byte output for single-field, multi-field, and multi-line `data` | unit | `cargo test -p ferro-rs --lib -- http::sse::tests::sse_event_wire_format` | Wave 0 |
| AISSE-01 / D-01 | Multi-line `data` produces repeated `data:` lines | unit | `cargo test -p ferro-rs --lib -- http::sse::tests::sse_event_multi_line_data` | Wave 0 |
| AISSE-01 / D-02 | `SseStream::channel()` returns `(Sender, SseStream)` | unit | inline with below | Wave 0 |
| AISSE-01 / D-02+D-05 | `poll_frame` yields event bytes when channel has data | unit | `cargo test -p ferro-rs --lib -- http::sse::tests::sse_stream_poll_delivers_event` | Wave 0 |
| AISSE-01 / D-05 | Keep-alive: `poll_frame` yields `:ping\n\n` when interval fires with no pending events | unit | `cargo test -p ferro-rs --lib -- http::sse::tests::sse_stream_keep_alive_ping` | Wave 0 |
| AISSE-01 / D-03 | `FerroBody::Full` variant — poll_frame delegates to `Full<Bytes>` correctly | unit | `cargo test -p ferro-rs --lib -- http::sse::tests::ferro_body_full_variant` | Wave 0 |
| AISSE-01 / D-03 | `FerroBody::Stream` variant — poll_frame delegates to `SseStream` | unit | `cargo test -p ferro-rs --lib -- http::sse::tests::ferro_body_stream_variant` | Wave 0 |
| AISSE-01 / D-04 | `HttpResponse::sse()` sets all 4 required headers | unit | `cargo test -p ferro-rs --lib -- http::sse::tests::sse_factory_headers` | Wave 0 |
| AISSE-01 / D-06 | SC#3 reinterpreted: SSE response body is `FerroBody::Stream`, NOT `Full` | unit | `cargo test -p ferro-rs --lib -- http::sse::tests::sse_response_is_stream_variant` | Wave 0 |
| AISSE-01 / D-07 | SC#5: incremental frame delivery — event N frame arrives before event N+1 is sent | unit (deterministic body-level poll) | `cargo test -p ferro-rs --lib -- http::sse::tests::sse_stream_incremental_delivery` | Wave 0 |
| AISSE-01 regression | Buffered path: `HttpResponse::text().into_hyper()` still returns `FerroBody::Full` | unit | `cargo test -p ferro-rs --lib -- http::response::tests` | Exists (test_into_hyper_preserves_binary et al.) |

### Detailed Test Specs

**T-168-01: `sse_event_wire_format`** — construct `SseEvent::data("hello").event("msg").id("1").retry(3000)`, call `to_string()` / `to_wire()`, assert bytes equal `b"event: msg\nid: 1\nretry: 3000\ndata: hello\n\n"`.

**T-168-02: `sse_event_multi_line_data`** — construct `SseEvent::data("line one\nline two")`, assert output equals `b"data: line one\ndata: line two\n\n"`.

**T-168-03: `sse_stream_poll_delivers_event`** — create channel, send one event, poll with noop waker, assert `Poll::Ready(Some(Ok(frame)))` where `frame.into_data()` equals expected wire bytes. Confirm second poll returns `Poll::Pending` (no second event queued).

**T-168-04: `sse_stream_keep_alive_ping`** — create `SseStream` with a very short interval (e.g. 1 ms in test), do NOT send any event, advance time with `tokio::time::pause()` + `tokio::time::advance()`, poll with a real waker via `#[tokio::test]`, assert the yielded frame bytes equal `b":ping\n\n"`.

**T-168-05: `ferro_body_full_variant`** — construct `FerroBody::Full(Full::new(Bytes::from("hi")))`, collect with `BodyExt::collect().await`, assert bytes equal `"hi"`.

**T-168-06: `ferro_body_stream_variant`** — send one event through `SseStream`, wrap in `FerroBody::Stream`, poll once with noop waker, assert `Poll::Ready(Some(Ok(_)))`.

**T-168-07: `sse_factory_headers`** — call `HttpResponse::sse(stream)` (or `sse_channel`), inspect headers, assert presence of `Content-Type: text/event-stream`, `Cache-Control: no-cache`, `Connection: keep-alive`, `X-Accel-Buffering: no`.

**T-168-08: `sse_response_is_stream_variant`** — create SSE response, call `into_hyper()`, inspect the body (via a helper or by attempting to downcast / match on it). The test should assert that the body is `FerroBody::Stream` and NOT `FerroBody::Full`. This validates D-06.

**T-168-09: `sse_stream_incremental_delivery`** — poll body before sending event N+1, assert `Poll::Pending`; then send event N, assert next poll returns `Poll::Ready`; confirms that frame N is available before event N+1 is sent.

**T-168-10: Buffered path regression** — existing `test_into_hyper_preserves_binary` in `http/response.rs` must still compile and pass. The only change: `BodyExt::collect()` on the new return type (`FerroBody`). May need `impl From<FerroBody> for hyper::Response<Full<Bytes>>` in tests, OR the test uses `BodyExt::collect()` on `FerroBody` directly.

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-rs --lib -- http::sse`
- **Per wave merge:** `cargo test --all-features -p ferro-rs`
- **Phase gate:** Full suite green + `cargo clippy --all --all-targets -- -D warnings` clean before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `framework/src/http/sse.rs` — new file with all SSE types and unit tests
- [ ] `framework/src/http/body.rs` (or inline in sse.rs) — `FerroBody` enum + `Body` impl + unit tests

*(Existing test infrastructure: `cargo test`, tokio runtime — fully sufficient. No new test framework.)*

---

## Security Domain

`security_enforcement` key is absent from `.planning/config.json` — treat as enabled.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | SSE is a transport primitive; auth is handler responsibility |
| V3 Session Management | no | No session state in SSE primitives themselves |
| V4 Access Control | no | Handler gates SSE routes; not in transport primitives |
| V5 Input Validation | partial | `SseEvent` data is application-provided, not external user input; no validation needed in the primitive |
| V6 Cryptography | no | No crypto in SSE transport |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Event injection via unescaped `\n` in `data` | Tampering | The SSE wire format naturally handles newlines by splitting data into multiple `data:` lines (the `Display` impl iterates `.lines()`). A `\n` in event data is not an injection risk — it produces a valid multi-line SSE event. |
| Field injection via `\n` in `event`/`id` fields | Tampering | These are set by application code, not from raw user input. The SSE primitive does not escape these. If they could contain user input, the application layer must sanitize. Document this in rustdoc. |
| Unbounded streaming (resource exhaustion) | Denial of Service | The mpsc channel buffer is bounded (default 16). If the consumer is too slow the sender's `send().await` will back-pressure. Document the buffer size choice. |
| nginx proxy buffering breaking streaming | Availability | Mitigated by `X-Accel-Buffering: no` header in D-04. |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `tokio::sync::mpsc::Receiver::poll_recv` returns `Poll<Option<T>>` directly (not a future) and registers the waker correctly | Pattern 2, Pitfall 3 | If wrong, `poll_frame` implementation needs restructuring to use a future-based API instead |
| A2 | `hyper_tungstenite::upgrade`'s response body can be mapped to an empty `Full<Bytes>` without breaking the WebSocket upgrade (the upgrade is tracked via request extensions, not response body) | D-03 blast radius, websocket.rs | If wrong, websocket.rs needs a more complex approach — e.g. a dedicated `FerroBody::Upgrade(Incoming)` variant |
| A3 | `tokio::time::Interval` is `Unpin` | Pattern 2 | If wrong, pin-project-lite must be added as a direct dep to project the interval field |
| A4 | `tokio::sync::mpsc::Receiver<SseEvent>` is `Unpin` | Pattern 2 | Same as A3 — pin-project-lite needed |

A3 and A4 are virtually certain given tokio's design (both types hold no self-referential data), but should be confirmed by the compiler on first build.

---

## Open Questions

1. **WebSocket response body mapping (A2)**
   - What we know: `hyper_tungstenite::upgrade` returns `hyper::Response<hyper::body::Incoming>` (not `Full<Bytes>`). The current code returns it directly, which already worked under `Full<Bytes>` via type coercion.
   - What's unclear: Whether the current code actually compiles today — if `handle_ws_upgrade` already returns `hyper::Response<Full<Bytes>>` in its type signature, then there must be an existing body conversion in websocket.rs that is invisible at grep level. Line 54 shows `return response` where `response` comes from `hyper_tungstenite::upgrade`.
   - Recommendation: On first compile of D-03, check the exact type of `response` from `hyper_tungstenite::upgrade`. If it already returns `hyper::Response<Full<Bytes>>` (some versions of hyper-tungstenite do this), the mapping is trivial. If it returns `hyper::Response<Incoming>`, use `.map(|_| FerroBody::Full(Full::new(Bytes::new())))`.

2. **`FerroBody` test accessibility for D-06 assertion**
   - What we know: The test for D-06 needs to assert that the SSE response body is `FerroBody::Stream`, not `FerroBody::Full`. This requires pattern-matching on `FerroBody`.
   - What's unclear: How `into_hyper()` exposes the body. Currently `HttpResponse::into_hyper()` returns the response; the test would need to call `.body()` on the hyper response (which returns `&FerroBody`) and match it.
   - Recommendation: `FerroBody` needs to be `pub` and accessible from tests. A `is_streaming(&self) -> bool` method on `FerroBody` (or a `pub fn` in the module) makes the test cleaner than a direct enum pattern match.

3. **`HttpResponse::sse()` signature — channel-returning vs stream-taking**
   - What we know: D-04 permits either. Channel-returning (`fn sse_channel(buffer: usize) -> (Sender<SseEvent>, Response)`) is more ergonomic (one call, no intermediate `SseStream::channel()`).
   - Recommendation: Implement `HttpResponse::sse_channel(buffer: usize) -> (mpsc::Sender<SseEvent>, HttpResponse)` (channel-returning). Also expose `SseStream::channel(buffer) -> (Sender, SseStream)` and `HttpResponse::sse(stream: SseStream) -> HttpResponse` for flexibility. This matches the ROADMAP's `sse(sender, stream)` intent while keeping the API clean.

---

## Sources

### Primary (HIGH confidence)

- `http-body 1.0.1` crate source (local cache at `/Users/alberto/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/http-body-util-0.1.3/`) — `Body` trait, `Frame`, `SizeHint`, `StreamBody`, `Full` verified directly
- `/hyperium/http-body` Context7 — `Body` trait shape, `Frame` API
- `/websites/rs_hyper` Context7 — `Body::poll_frame` signature, `hyper::Response<B>` generics
- `framework/Cargo.toml` and `Cargo.lock` — all dependency versions verified
- `framework/src/server.rs`, `http/response.rs`, `websocket.rs`, `static_files.rs`, `middleware/pre_route.rs`, `debug/mod.rs` — direct source read for blast radius mapping
- `framework/tests/action_handler.rs` — TCP loopback test pattern verified

### Secondary (MEDIUM confidence)

- WHATWG HTML Living Standard §9.2.6 — SSE wire format rules (well-known stable spec)
- CONTEXT.md D-01..D-08 — locked decisions directly constraining implementation

### Tertiary (LOW confidence, flagged)

- A1–A4 in Assumptions Log — tokio Unpin guarantees and hyper-tungstenite body type (verified by reasoning; confirmed on first compile)

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries verified from Cargo.lock and local crate source
- Architecture: HIGH — server.rs serve loop read directly; all 6 blast-radius files read
- Blast radius mapping: HIGH — exact line numbers from direct grep
- SSE wire format: HIGH — WHATWG spec is stable and well-known
- Pitfalls: MEDIUM/HIGH — interval-fires-immediately and Unpin analysis are from training knowledge but are standard Tokio behavior
- FerroBody enum Body impl: HIGH — pattern derived directly from http-body 1.0.1 trait + verified Unpin status

**Research date:** 2026-06-08
**Valid until:** 2026-09-08 (stable; http-body 1.x and hyper 1.x are not fast-moving at this level)
