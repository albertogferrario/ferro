---
phase: 168-framework-sse-primitives
reviewed: 2026-06-08T00:00:00Z
depth: standard
files_reviewed: 11
files_reviewed_list:
  - framework/src/http/sse.rs
  - framework/src/http/body.rs
  - framework/src/http/response.rs
  - framework/src/http/mod.rs
  - framework/src/server.rs
  - framework/src/websocket.rs
  - framework/src/static_files.rs
  - framework/src/middleware/pre_route.rs
  - framework/src/debug/mod.rs
  - framework/src/json_ui/mod.rs
  - framework/src/lib.rs
findings:
  critical: 0
  warning: 1
  info: 2
  total: 3
status: issues_found
---

# Phase 168: Code Review Report

**Reviewed:** 2026-06-08
**Depth:** standard
**Files Reviewed:** 11
**Status:** issues_found

## Summary

Phase 168 introduces `FerroBody { Full, Stream }` as a unified response body type and
`SseStream` / `SseEvent` as first-class SSE primitives. The 11-file refactor replaces every
`hyper::Response<Full<Bytes>>` call site with `hyper::Response<FerroBody>`. The implementation
is broadly sound: pin-projection is correct for `Unpin` fields, the `Error = Infallible`
contract is met, the four required SSE headers are present, field-injection sanitization
covers both `\n` and `\r`, and the WS upgrade body-mapping is correct.

One warning and two info items are raised.

---

## Warnings

### WR-01: Idle-window reset in `poll_frame` does not reliably defer keep-alive pings after event delivery

**File:** `framework/src/http/sse.rs:215-218`

**Issue:**

The code intends to "reset the idle window" whenever an event is delivered, so that
a `:ping` is not emitted soon after an event. The mechanism is:

```rust
Poll::Ready(Some(event)) => {
    // Reset idle window: consume any pending interval tick silently.
    let _ = Pin::new(&mut self.ping_interval).poll_tick(cx);
    ...
    return Poll::Ready(Some(Ok(Frame::data(bytes))));
}
```

`poll_tick` only *consumes* a tick if one has already elapsed. If the tick has not yet
elapsed (the common case: the event arrived within the 15-second window), `poll_tick`
returns `Poll::Pending` and does **not** reschedule the interval — the interval fires
at its original absolute deadline.

The practical effect: pings can arrive shortly after events. For example, if the interval
was created 14.9s ago and an event arrives, the ping fires 0.1s later regardless of the
event. The comment says "Reset idle window" but the reset is only effective when a tick
happens to be ready simultaneously with an event — the least-needed case.

A secondary effect: when `poll_tick(cx)` returns `Pending`, it registers the interval's
waker in the current `cx`. This causes an extra poll of the body when the timer fires,
even though the caller already received an event frame. This is harmless for `Body`
implementors (spurious polls are permitted) but is wasted work.

**Fix:**

To reliably reset the interval after each event, replace `Interval` with
a re-armed one-shot deadline. The simplest approach is to call `reset` on the interval:

```rust
Poll::Ready(Some(event)) => {
    // Reset idle window: reschedule the next ping deadline.
    let period = Duration::from_secs(15);
    self.ping_interval.reset_at(Instant::now() + period);
    let bytes = Bytes::from(event.to_wire());
    return Poll::Ready(Some(Ok(Frame::data(bytes))));
}
```

`tokio::time::Interval::reset_at` (stable since tokio 1.21) reschedules the next tick
from the given instant, genuinely deferring the next ping by a full period after each
event. Drop the `poll_tick(cx)` call entirely — it is not needed once `reset_at` is used.

If `reset_at` is not available in the pinned tokio version, `reset()` (which resets to
`Instant::now() + period`) is equivalent for this purpose.

---

## Info

### IN-01: `\0` (null byte) in `id` field is not sanitized

**File:** `framework/src/http/sse.rs:87-91`

**Issue:**

The WHATWG SSE spec (§9.2.6 step 4) treats a null byte (`\0`) in the `id` field as a
signal to reset the browser's last-event-id to the empty string. A caller-supplied `id`
value containing `\0` may silently nullify reconnection state on the client without any
visible indicator.

The current sanitization strips `\n` and `\r` (injection prevention) but not `\0`. This
is not an injection vector, but it is a subtle behavioral edge case that can be triggered
via untrusted input reaching the `id` setter.

**Fix:**

Extend the strip pattern in `SseEvent::id`:

```rust
pub fn id(mut self, id: impl Into<String>) -> Self {
    let s: String = id.into();
    self.id = Some(s.replace(['\n', '\r', '\0'], ""));
    self
}
```

For defense-in-depth, the same strip pattern could be applied in `SseEvent::event`,
though the spec behavior for `\0` in event type is simply to pass it through (no special
treatment) — so it is lower priority there.

---

### IN-02: `SseStream` not publicly re-exported from `sse_channel` / pattern asymmetry

**File:** `framework/src/http/response.rs:204-219`

**Issue:**

`HttpResponse::sse_channel` returns `(mpsc::Sender<SseEvent>, hyper::Response<FerroBody>)`
rather than `(mpsc::Sender<SseEvent>, HttpResponse)`. This is intentional (the SSE response
can only be `FerroBody::Stream`, not the `HttpResponse` builder struct), but it creates an
asymmetry in the public API: all other `HttpResponse` constructors return `HttpResponse`
(then call `.into_hyper()` at the serve boundary), while `sse_channel` returns a
`hyper::Response<FerroBody>` directly.

This is not a bug, but it breaks the builder-chaining pattern — the caller cannot call
`.header()` on the returned SSE response if they need to add a custom header. The companion
`HttpResponse::sse(stream)` has the same shape. Both skip the cookie/header builder methods.

**Fix:**

Consider returning `HttpResponse`-shaped types and converting at the last moment, or
document the asymmetry explicitly in the function-level rustdoc so callers know to use
`hyper::Response::builder()` directly when custom headers are needed. At minimum, add a
note to the existing rustdoc:

```rust
/// Note: returns `hyper::Response<FerroBody>` directly (not `HttpResponse`) because the
/// body is a live stream. Custom headers beyond the four defaults must be added with
/// `hyper::Response::builder()` manually.
```

---

_Reviewed: 2026-06-08_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
