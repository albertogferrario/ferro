//! Test helper: `BroadcastCapture` for capturing broadcast frames during
//! integration tests.
//!
//! Uses the PRODUCTION `Broadcaster` code path — real `Broadcaster::new`,
//! real `add_client` with a real `mpsc::Sender<ServerMessage>`, real
//! `subscribe(...).await`, real drain via `try_recv` against the receiver.
//! No mocks, no trait forks; this is the canonical way to assert on
//! broadcast frames in ferro-projection tests.
//!
//! Shape locked by RESEARCH.md §Technical Concerns #5.

#![allow(dead_code)]

use ferro_broadcast::{BroadcastMessage, Broadcaster, ServerMessage};
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct BroadcastCapture {
    pub broadcaster: Arc<Broadcaster>,
    rx: mpsc::Receiver<ServerMessage>,
}

impl BroadcastCapture {
    /// Construct a fresh Broadcaster, add a mock client, and subscribe
    /// that client to `channel`. Returns the capture; callers pass
    /// `capture.broadcaster.clone()` into `ProjectionRuntime::new(...)`.
    pub async fn subscribe(channel: &str) -> Self {
        let broadcaster = Arc::new(Broadcaster::new());
        let (tx, rx) = mpsc::channel(64);
        let socket_id = "test-client".to_string();
        broadcaster.add_client(socket_id.clone(), tx);
        broadcaster
            .subscribe(&socket_id, channel, None, None)
            .await
            .expect("subscribe to test channel");
        Self { broadcaster, rx }
    }

    /// Drain all received broadcast events. Returns the
    /// `BroadcastMessage` payloads (channel, event, data tuples).
    pub fn drain(&mut self) -> Vec<BroadcastMessage> {
        let mut out = Vec::new();
        while let Ok(msg) = self.rx.try_recv() {
            if let ServerMessage::Event(bm) = msg {
                out.push(bm);
            }
        }
        out
    }
}
