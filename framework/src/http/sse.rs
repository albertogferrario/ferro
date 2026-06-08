//! Server-Sent Events (SSE) types: wire serialization, streaming body, and response factory.
//!
//! # Overview
//!
//! ```text
//! SseStream::channel(16) → (Sender<SseEvent>, SseStream)
//!     │
//!     ├── Sender<SseEvent>   ── handler spawns a task that calls tx.send(event).await
//!     └── SseStream ──► HttpResponse::sse(stream) ──► into_hyper() ──► FerroBody::Stream
//! ```
//!
//! # Security note
//!
//! The `event` and `id` fields of [`SseEvent`] are set by application code, not from raw
//! user input. If these fields could contain user-controlled data, the application layer must
//! sanitize them — the SSE primitive does not escape field values. The `data` field is safe:
//! newlines in `data` produce repeated `data:` lines per the WHATWG spec, which is not an
//! injection risk.

use bytes::Bytes;
use hyper::body::{Body, Frame, SizeHint};
use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use tokio::time::{interval_at, Duration, Instant, Interval};

// ──────────────────────────────────────────────────────────────────────────────
// SseEvent
// ──────────────────────────────────────────────────────────────────────────────

/// A single Server-Sent Event, serializable to the WHATWG `text/event-stream` wire format.
///
/// Field ordering in the wire output follows the WHATWG spec recommendation:
/// `event:`, `id:`, `retry:`, then one or more `data:` lines, terminated by a blank line.
///
/// # Example
///
/// ```rust,ignore
/// let event = SseEvent::data("hello")
///     .event("token")
///     .id("42")
///     .retry(3000);
/// // Wire: "event: token\nid: 42\nretry: 3000\ndata: hello\n\n"
/// ```
#[derive(Debug, Clone)]
pub struct SseEvent {
    /// The event payload. Multi-line strings produce repeated `data:` lines.
    pub data: String,
    /// Optional named event type (`event:` field).
    pub event: Option<String>,
    /// Optional last-event ID (`id:` field).
    pub id: Option<String>,
    /// Optional client reconnection delay in milliseconds (`retry:` field).
    pub retry: Option<u64>,
}

impl SseEvent {
    /// Create an event with the given data string.
    ///
    /// This is the primary constructor; chain `.event()`, `.id()`, `.retry()` for additional fields.
    pub fn data(data: impl Into<String>) -> Self {
        Self {
            data: data.into(),
            event: None,
            id: None,
            retry: None,
        }
    }

    /// Set the named event type.
    pub fn event(mut self, event: impl Into<String>) -> Self {
        self.event = Some(event.into());
        self
    }

    /// Set the last-event ID.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the client reconnection delay in milliseconds.
    pub fn retry(mut self, ms: u64) -> Self {
        self.retry = Some(ms);
        self
    }

    /// Serialize to the SSE wire format.
    ///
    /// Equivalent to `format!("{event}")` via the [`Display`](fmt::Display) impl.
    pub fn to_wire(&self) -> String {
        self.to_string()
    }
}

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
        // Multi-line data: each line gets its own `data:` prefix per WHATWG spec.
        for line in self.data.lines() {
            write!(f, "data: {line}\n")?;
        }
        // Empty data still emits one data: line.
        if self.data.is_empty() {
            write!(f, "data: \n")?;
        }
        // Blank line terminates the event.
        write!(f, "\n")
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// SseStream
// ──────────────────────────────────────────────────────────────────────────────

/// Streaming HTTP body that serializes [`SseEvent`]s from an mpsc channel.
///
/// Implements [`http_body::Body`] so it can be carried as `FerroBody::Stream` through the
/// framework's hyper serve loop. The stream ends when the [`mpsc::Sender`] is dropped.
///
/// A `:ping\n\n` keep-alive comment is emitted every 15 seconds while the channel is idle.
/// Any real event resets the idle window.
///
/// # Bounded back-pressure
///
/// The internal channel is bounded (default 16 slots via [`SseStream::channel`]). If the
/// client is too slow, `Sender::send().await` will apply back-pressure to the producer.
///
/// # Connection count limits
///
/// Each active `SseStream` holds a TCP connection open. Connection-count limits are the
/// application's responsibility, not this primitive's.
pub struct SseStream {
    receiver: mpsc::Receiver<SseEvent>,
    ping_interval: Interval,
}

impl SseStream {
    /// Create a bounded channel for pushing events and the streaming body.
    ///
    /// Returns `(sender, stream)`. The handler holds the sender and pushes events from
    /// a spawned task; the stream is wrapped in an [`HttpResponse::sse`] response.
    ///
    /// Uses [`interval_at`] with an initial delay equal to `interval_period` so that
    /// the first ping is deferred — avoiding an immediate `:ping` frame on connection.
    pub fn channel(buffer: usize) -> (mpsc::Sender<SseEvent>, Self) {
        let (tx, rx) = mpsc::channel(buffer);
        // interval_at defers the first tick, avoiding the immediate-first-tick pitfall.
        let period = Duration::from_secs(15);
        let ping = interval_at(Instant::now() + period, period);
        (tx, SseStream { receiver: rx, ping_interval: ping })
    }

    /// Returns `true` if the internal channel has been closed (sender dropped).
    pub fn is_closed(&self) -> bool {
        self.receiver.is_closed()
    }

    /// Create a channel with a custom ping interval period.
    ///
    /// Intended for tests that need a short interval without waiting 15 seconds.
    #[cfg(test)]
    pub(crate) fn channel_with_interval(
        buffer: usize,
        interval_period: Duration,
    ) -> (mpsc::Sender<SseEvent>, Self) {
        let (tx, rx) = mpsc::channel(buffer);
        let ping = interval_at(Instant::now() + interval_period, interval_period);
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
        // Both Receiver and Interval are Unpin — Pin::new is valid without pin-project.
        match self.receiver.poll_recv(cx) {
            Poll::Ready(Some(event)) => {
                // Reset idle window: consume any pending interval tick silently.
                let _ = Pin::new(&mut self.ping_interval).poll_tick(cx);
                let bytes = Bytes::from(event.to_wire());
                return Poll::Ready(Some(Ok(Frame::data(bytes))));
            }
            Poll::Ready(None) => {
                // Sender dropped — signal end of stream.
                return Poll::Ready(None);
            }
            Poll::Pending => {}
        }

        // No event ready — check keep-alive interval.
        match Pin::new(&mut self.ping_interval).poll_tick(cx) {
            Poll::Ready(_) => {
                let ping = Bytes::from_static(b":ping\n\n");
                Poll::Ready(Some(Ok(Frame::data(ping))))
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn is_end_stream(&self) -> bool {
        // Only terminated when the Sender is dropped; we cannot know ahead of time.
        false
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::default()
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::task::noop_waker;

    // ── SseEvent wire format ──────────────────────────────────────────────────

    /// T-168-01: full event wire format
    #[test]
    fn sse_event_wire_format() {
        let event = SseEvent::data("hello").event("msg").id("1").retry(3000);
        let wire = event.to_wire();
        assert_eq!(wire, "event: msg\nid: 1\nretry: 3000\ndata: hello\n\n");
    }

    /// T-168-02: multi-line data → repeated `data:` lines
    #[test]
    fn sse_event_multi_line_data() {
        let event = SseEvent::data("line one\nline two");
        let wire = event.to_wire();
        assert_eq!(wire, "data: line one\ndata: line two\n\n");
    }

    /// Empty data emits exactly one `data: \n` line
    #[test]
    fn sse_event_empty_data() {
        let event = SseEvent::data("");
        let wire = event.to_wire();
        assert_eq!(wire, "data: \n\n");
    }

    /// data-only event (no optional fields)
    #[test]
    fn sse_event_data_only() {
        let wire = SseEvent::data("hello world").to_wire();
        assert_eq!(wire, "data: hello world\n\n");
    }

    // ── SseStream poll_frame ──────────────────────────────────────────────────

    /// T-168-03: poll_frame delivers event bytes from channel
    #[tokio::test]
    async fn sse_stream_poll_delivers_event() {
        let (tx, mut stream) = SseStream::channel(4);
        tx.send(SseEvent::data("first")).await.unwrap();

        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);

        let frame = Pin::new(&mut stream).poll_frame(&mut cx);
        match frame {
            Poll::Ready(Some(Ok(f))) => {
                let data = f.into_data().expect("expected data frame");
                assert_eq!(data, Bytes::from("data: first\n\n"));
            }
            other => panic!("expected Poll::Ready(Some(Ok(frame))), got {other:?}"),
        }

        // Second poll — no more events queued.
        let frame2 = Pin::new(&mut stream).poll_frame(&mut cx);
        assert!(
            matches!(frame2, Poll::Pending),
            "expected Poll::Pending with no queued events, got {frame2:?}"
        );
    }

    /// T-168-04: keep-alive ping is emitted when the interval fires with no pending events.
    ///
    /// Uses a 10 ms interval (via the test-only `channel_with_interval` constructor) and
    /// a real sleep so we don't need the `test-util` tokio feature for `pause/advance`.
    #[tokio::test]
    async fn sse_stream_keep_alive_ping() {
        let period = Duration::from_millis(10);
        let (_tx, mut stream) = SseStream::channel_with_interval(4, period);

        // Wait for the interval to fire.
        tokio::time::sleep(period * 3).await;

        // Drive poll_frame with a real waker via a one-shot future.
        use http_body_util::BodyExt;
        let frame = tokio::time::timeout(Duration::from_millis(200), stream.frame())
            .await
            .expect("timed out waiting for :ping frame")
            .expect("stream ended unexpectedly")
            .expect("poll_frame returned error");

        let data = frame.into_data().expect("expected data frame");
        assert_eq!(data, Bytes::from_static(b":ping\n\n"));
    }

    /// T-168-09: incremental delivery — event N frame before event N+1 is sent
    #[tokio::test]
    async fn sse_stream_incremental_delivery() {
        let (tx, mut stream) = SseStream::channel(4);

        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);

        // Before sending: Pending
        let before = Pin::new(&mut stream).poll_frame(&mut cx);
        assert!(
            matches!(before, Poll::Pending),
            "expected Poll::Pending before send"
        );

        // Send event N
        tx.send(SseEvent::data("N")).await.unwrap();

        // Now Ready
        let after = Pin::new(&mut stream).poll_frame(&mut cx);
        assert!(
            matches!(after, Poll::Ready(Some(Ok(_)))),
            "expected Poll::Ready after send"
        );

        // Still Pending — event N+1 not yet sent
        let still_pending = Pin::new(&mut stream).poll_frame(&mut cx);
        assert!(
            matches!(still_pending, Poll::Pending),
            "expected Poll::Pending before N+1 send"
        );
    }
}
