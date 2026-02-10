//! WebSocket upgrade handler and connection message loop.
//!
//! Handles WebSocket connections at `/_ferro/ws`, bridging clients
//! to the Broadcaster for channel subscriptions and message dispatch.

use crate::container::App;
use bytes::Bytes;
use ferro_broadcast::{Broadcaster, ClientMessage, PresenceMember, ServerMessage};
use futures_util::{SinkExt, StreamExt};
use http_body_util::Full;
use hyper_tungstenite::tungstenite::Message as WsMsg;
use tokio::sync::mpsc;
use tokio::time::Instant;
use uuid::Uuid;

/// Perform WebSocket upgrade and spawn the connection handler.
///
/// Returns the HTTP 101 upgrade response directly. The actual WebSocket
/// message loop runs in a spawned task after the upgrade completes.
pub(crate) fn handle_ws_upgrade(
    mut req: hyper::Request<hyper::body::Incoming>,
) -> hyper::Response<Full<Bytes>> {
    let broadcaster = match App::get::<Broadcaster>() {
        Some(b) => b,
        None => {
            return hyper::Response::builder()
                .status(503)
                .body(Full::new(Bytes::from("Broadcasting not configured")))
                .unwrap();
        }
    };

    let (response, ws_future) = match hyper_tungstenite::upgrade(&mut req, None) {
        Ok(pair) => pair,
        Err(e) => {
            eprintln!("WebSocket upgrade failed: {}", e);
            return hyper::Response::builder()
                .status(400)
                .body(Full::new(Bytes::from("WebSocket upgrade failed")))
                .unwrap();
        }
    };

    tokio::spawn(async move {
        match ws_future.await {
            Ok(ws_stream) => {
                handle_connection(ws_stream, broadcaster).await;
            }
            Err(e) => {
                eprintln!("WebSocket connection failed: {}", e);
            }
        }
    });

    response
}

/// Run the WebSocket message loop with heartbeat and timeout.
async fn handle_connection(
    ws_stream: hyper_tungstenite::HyperWebsocketStream,
    broadcaster: Broadcaster,
) {
    let socket_id = Uuid::new_v4().to_string();
    let (mut ws_write, mut ws_read) = ws_stream.split();
    let (tx, mut rx) = mpsc::channel::<ServerMessage>(32);

    // Register client with broadcaster
    broadcaster.add_client(socket_id.clone(), tx);

    // Send connection_established message
    let connected = ServerMessage::Connected {
        socket_id: socket_id.clone(),
    };
    if let Ok(msg) = connected.to_ws_message() {
        let _ = ws_write.send(msg).await;
    }

    let config = broadcaster.config().clone();
    let mut heartbeat_interval = tokio::time::interval(config.heartbeat_interval);
    let mut last_activity = Instant::now();

    loop {
        tokio::select! {
            // Incoming WebSocket frame
            frame = ws_read.next() => {
                match frame {
                    Some(Ok(msg)) => {
                        last_activity = Instant::now();
                        match msg {
                            WsMsg::Text(text) => {
                                handle_client_message(
                                    &text,
                                    &socket_id,
                                    &broadcaster,
                                    &mut ws_write,
                                ).await;
                            }
                            WsMsg::Close(_) => break,
                            WsMsg::Pong(_) => {
                                // Activity already updated above
                            }
                            _ => {} // tungstenite handles Ping with auto-Pong
                        }
                    }
                    Some(Err(_)) => break,
                    None => break,
                }
            }
            // Server message to forward to client
            server_msg = rx.recv() => {
                match server_msg {
                    Some(msg) => {
                        if let Ok(ws_msg) = msg.to_ws_message() {
                            if ws_write.send(ws_msg).await.is_err() {
                                break;
                            }
                        }
                    }
                    None => break,
                }
            }
            // Heartbeat tick
            _ = heartbeat_interval.tick() => {
                if last_activity.elapsed() > config.client_timeout {
                    break;
                }
                if ws_write.send(WsMsg::Ping(vec![].into())).await.is_err() {
                    break;
                }
            }
        }
    }

    // Clean shutdown: send Close frame and remove from broadcaster
    let _ = ws_write.send(WsMsg::Close(None)).await;
    broadcaster.remove_client(&socket_id);
}

/// Parse and dispatch a client message.
async fn handle_client_message<S>(
    text: &str,
    socket_id: &str,
    broadcaster: &Broadcaster,
    ws_write: &mut S,
) where
    S: SinkExt<WsMsg, Error = hyper_tungstenite::tungstenite::Error> + Unpin,
{
    let client_msg = match ClientMessage::from_ws_text(text) {
        Ok(msg) => msg,
        Err(_) => {
            let err = ServerMessage::Error {
                message: "Invalid message format".into(),
            };
            if let Ok(ws_msg) = err.to_ws_message() {
                let _ = ws_write.send(ws_msg).await;
            }
            return;
        }
    };

    match client_msg {
        ClientMessage::Subscribe {
            channel,
            auth,
            channel_data,
        } => {
            let member_info = if channel.starts_with("presence-") {
                channel_data.and_then(|data| {
                    let user_id = data.get("user_id")?.as_str()?.to_string();
                    let user_info = data
                        .get("user_info")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);
                    Some(PresenceMember::new(socket_id, user_id).with_info(user_info))
                })
            } else {
                None
            };

            match broadcaster
                .subscribe(socket_id, &channel, auth.as_deref(), member_info)
                .await
            {
                Ok(()) => {
                    let msg = ServerMessage::Subscribed { channel };
                    if let Ok(ws_msg) = msg.to_ws_message() {
                        let _ = ws_write.send(ws_msg).await;
                    }
                }
                Err(e) => {
                    let msg = ServerMessage::SubscriptionError {
                        channel,
                        error: e.to_string(),
                    };
                    if let Ok(ws_msg) = msg.to_ws_message() {
                        let _ = ws_write.send(ws_msg).await;
                    }
                }
            }
        }
        ClientMessage::Unsubscribe { channel } => {
            broadcaster.unsubscribe(socket_id, &channel).await;
            let msg = ServerMessage::Unsubscribed { channel };
            if let Ok(ws_msg) = msg.to_ws_message() {
                let _ = ws_write.send(ws_msg).await;
            }
        }
        ClientMessage::Whisper {
            channel,
            event,
            data,
        } => {
            if let Err(e) = broadcaster.whisper(socket_id, &channel, &event, data).await {
                let msg = ServerMessage::Error {
                    message: e.to_string(),
                };
                if let Ok(ws_msg) = msg.to_ws_message() {
                    let _ = ws_write.send(ws_msg).await;
                }
            }
        }
        ClientMessage::Ping => {
            let msg = ServerMessage::Pong;
            if let Ok(ws_msg) = msg.to_ws_message() {
                let _ = ws_write.send(ws_msg).await;
            }
        }
    }
}
