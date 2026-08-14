//! Redis-gated integration tests for the shared broadcast transport.
//!
//! Run with:
//!   REDIS_URL=redis://127.0.0.1:6379 \
//!     cargo test -p ferro-broadcast --features redis-transport -- redis_integration
//!
//! Without REDIS_URL set, each test exits early with a diagnostic.
//! Without `--features redis-transport`, this file compiles to an empty module.
#![cfg(feature = "redis-transport")]

use ferro_broadcast::transport::redis::RedisTransport;
use ferro_broadcast::{Broadcaster, ServerMessage};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

fn redis_url() -> Option<String> {
    std::env::var("REDIS_URL")
        .or_else(|_| std::env::var("BROADCAST_REDIS_URL"))
        .ok()
        .filter(|s| !s.is_empty())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn redis_integration_cross_process_delivery() {
    let Some(url) = redis_url() else {
        eprintln!("REDIS_URL not set — skipping live-Redis integration test");
        return;
    };

    // Unique channel per run to isolate from concurrent test runs sharing one Redis.
    let channel = format!("ferro:broadcast:test:{}", uuid::Uuid::new_v4());

    let bus_a = Arc::new(
        RedisTransport::new(&url, channel.clone())
            .await
            .expect("connect A"),
    );
    let bus_b = Arc::new(
        RedisTransport::new(&url, channel.clone())
            .await
            .expect("connect B"),
    );

    let a = Broadcaster::with_config(Default::default()).with_transport(bus_a);
    let b = Broadcaster::with_config(Default::default()).with_transport(bus_b);

    let (tx_b, mut rx_b) = mpsc::channel(16);
    b.add_client("socket_b".into(), tx_b);
    b.subscribe("socket_b", "orders.9", None, None)
        .await
        .unwrap();

    // Let both SUBSCRIBE connections attach to Redis.
    tokio::time::sleep(Duration::from_millis(150)).await;

    a.broadcast("orders.9", "OrderUpdated", serde_json::json!({"id": 9}))
        .await
        .unwrap();

    let msg = tokio::time::timeout(Duration::from_secs(2), rx_b.recv())
        .await
        .expect("timed out waiting for live-Redis cross-process delivery")
        .expect("client channel closed");
    match msg {
        ServerMessage::Event(m) => assert_eq!(m.channel, "orders.9"),
        other => panic!("expected Event, got {other:?}"),
    }
}
