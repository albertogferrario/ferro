//! HTTP body types and parsing utilities.
//!
//! This module provides:
//! - [`FerroBody`] — the unified response body type for the framework's hyper serve loop.
//! - Request-body collection and parsing helpers.

use crate::error::FrameworkError;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::{Body, Frame, SizeHint};
use hyper::body::Incoming;
use serde::de::DeserializeOwned;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Collect the full body from an Incoming stream
pub async fn collect_body(body: Incoming) -> Result<Bytes, FrameworkError> {
    body.collect()
        .await
        .map(|collected| collected.to_bytes())
        .map_err(|e| FrameworkError::internal(format!("Failed to read request body: {e}")))
}

/// Parse bytes as JSON into the target type
pub fn parse_json<T: DeserializeOwned>(bytes: &Bytes) -> Result<T, FrameworkError> {
    serde_json::from_slice(bytes)
        .map_err(|e| FrameworkError::internal(format!("Failed to parse JSON body: {e}")))
}

/// Parse bytes as form-urlencoded into the target type
pub fn parse_form<T: DeserializeOwned>(bytes: &Bytes) -> Result<T, FrameworkError> {
    serde_urlencoded::from_bytes(bytes)
        .map_err(|e| FrameworkError::internal(format!("Failed to parse form body: {e}")))
}

// ──────────────────────────────────────────────────────────────────────────────
// FerroBody — unified response body for the hyper serve loop
// ──────────────────────────────────────────────────────────────────────────────

/// Unified HTTP response body for the framework's hyper serve loop.
///
/// Carries either a fully-buffered body ([`Full<Bytes>`]) or a streaming SSE body
/// ([`SseStream`](crate::http::sse::SseStream)), allowing the same `hyper::Response<FerroBody>`
/// type to flow through all response-path code without dynamic dispatch or heap allocation on
/// the buffered hot path.
///
/// # Structural compression rule (D-06)
///
/// If a compression layer is added in a future phase it MUST match on `FerroBody::Full` only
/// and pass `FerroBody::Stream` through untouched. A streaming body cannot be whole-body
/// buffered without breaking the SSE protocol.
///
/// # Error type
///
/// `FerroBody::Error = Infallible` — both variants are infallible so the serve loop's
/// `Ok::<_, Infallible>(...)` wrapper is unchanged.
pub enum FerroBody {
    /// Fully-buffered response body (the common, allocation-free path).
    Full(Full<Bytes>),
    /// Streaming SSE body (held-open connection; yields frames as events arrive).
    Stream(crate::http::sse::SseStream),
}

impl std::fmt::Debug for FerroBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FerroBody::Full(b) => write!(f, "FerroBody::Full({b:?})"),
            FerroBody::Stream(_) => write!(f, "FerroBody::Stream(..)"),
        }
    }
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
                // Full<Bytes>::Error = Infallible — map_err is a no-op but satisfies the type.
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

#[cfg(test)]
mod ferro_body_tests {
    use super::*;
    use crate::http::sse::{SseEvent, SseStream};
    use futures_util::task::noop_waker;
    use http_body_util::BodyExt;

    /// T-168-05: FerroBody::Full variant collects to expected bytes
    #[tokio::test]
    async fn ferro_body_full_variant() {
        let body = FerroBody::Full(Full::new(Bytes::from("hi")));
        let collected = body.collect().await.unwrap().to_bytes();
        assert_eq!(collected, Bytes::from("hi"));
    }

    /// T-168-06: FerroBody::Stream variant delegates poll_frame to SseStream
    #[tokio::test]
    async fn ferro_body_stream_variant() {
        let (tx, stream) = SseStream::channel(4);
        tx.send(SseEvent::data("test")).await.unwrap();

        let mut body = FerroBody::Stream(stream);
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);

        let frame = Pin::new(&mut body).poll_frame(&mut cx);
        assert!(
            matches!(frame, Poll::Ready(Some(Ok(_)))),
            "expected Poll::Ready(Some(Ok(_))), got {frame:?}"
        );
    }
}
