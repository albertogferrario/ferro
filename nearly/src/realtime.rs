//! Real-time broadcasting helpers.
//!
//! The framework hosts the WebSocket endpoint (`/_ferro/ws`) and resolves the
//! `Broadcaster` we register in `bootstrap`. Controllers call [`emit`] to push a
//! server event to every client subscribed to a channel:
//!
//! - `nearby` (public) — presence updates; the map moves/adds pins live.
//! - `private-user.{id}` (private, signed) — a trillo ping to one recipient.

use std::sync::Arc;

use ferro::serde_json::Value;
use ferro::{App, Broadcast, Broadcaster};

/// Broadcast `event` with `data` to `channel`. A no-op (logged) if broadcasting
/// isn't configured, so a missing broadcaster never breaks a request.
pub async fn emit(channel: &str, event: &str, data: Value) {
    let Some(broadcaster) = App::get::<Broadcaster>() else {
        return;
    };
    // `Broadcaster` is Clone-over-Arc, so wrapping a resolved clone still reaches
    // the same connected sockets.
    if let Err(e) = Broadcast::new(Arc::new(broadcaster))
        .channel(channel)
        .event(event)
        .data(&data)
        .send()
        .await
    {
        eprintln!("broadcast to {channel} failed: {e}");
    }
}
