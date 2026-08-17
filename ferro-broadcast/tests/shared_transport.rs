//! In-memory shared-transport proofs (SC1 / SC2 subset / SC3 / D-03 / D-04 / D-06).
//! Deterministic, no live Redis — runs on the default `cargo test -p ferro-broadcast`.

use ferro_broadcast::transport::memory::InMemoryTransport;
use ferro_broadcast::{Broadcaster, ServerMessage};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// SC1 — Delta published on process A reaches subscriber on process B.
///
/// Validates: a `ServerMessage::Event` published via `broadcast()` on replica A
/// crosses the shared in-memory bus and is received exactly once by a client
/// wired to replica B within the timeout window.
///
/// No sleep: `with_transport` awaits the readiness signal from `subscribe_loop`
/// before returning, so the subscription is established by construction when
/// `broadcast` fires (WR-02 closed).
#[tokio::test]
async fn sc1_cross_process_delivery() {
    let bus = Arc::new(InMemoryTransport::new(64));
    let a = Broadcaster::with_config(Default::default())
        .with_transport(bus.clone())
        .await;
    let b = Broadcaster::with_config(Default::default())
        .with_transport(bus.clone())
        .await;

    // Wire a client to replica B and subscribe it to "orders.1".
    let (tx_b, mut rx_b) = mpsc::channel(16);
    b.add_client("socket_b".into(), tx_b);
    b.subscribe("socket_b", "orders.1", None, None)
        .await
        .unwrap();

    // No sleep needed — readiness is guaranteed by construction (WR-02 closed).

    // Replica A publishes; A has no local subscriber on "orders.1".
    a.broadcast("orders.1", "OrderUpdated", serde_json::json!({"id": 1}))
        .await
        .unwrap();

    let msg = tokio::time::timeout(Duration::from_millis(100), rx_b.recv())
        .await
        .expect("timed out waiting for cross-process delivery")
        .expect("client channel closed");
    match msg {
        ServerMessage::Event(m) => {
            assert_eq!(m.channel, "orders.1");
            assert_eq!(m.event, "OrderUpdated");
        }
        other => panic!("expected Event, got {other:?}"),
    }

    // Exactly once: no second delivery within the window.
    let second = tokio::time::timeout(Duration::from_millis(50), rx_b.recv()).await;
    assert!(second.is_err(), "duplicate cross-process delivery");
}

/// SC1 echo suppression — Own echo is dropped (D-03).
///
/// Validates: after `broadcast()` on replica A, the local subscriber on A receives
/// exactly ONE copy (the immediate local delivery). The bus echo that arrives back
/// via the SUBSCRIBE loop is suppressed by origin-id comparison and is not delivered.
#[tokio::test]
async fn sc1_own_echo_suppressed() {
    let bus = Arc::new(InMemoryTransport::new(64));
    let a = Broadcaster::with_config(Default::default())
        .with_transport(bus.clone())
        .await;

    let (tx, mut rx) = mpsc::channel(16);
    a.add_client("socket_a".into(), tx);
    a.subscribe("socket_a", "chat", None, None).await.unwrap();
    // No sleep needed — readiness guaranteed by construction (WR-02 closed).

    a.broadcast("chat", "Msg", serde_json::json!({}))
        .await
        .unwrap();

    // Exactly one delivery (the immediate local one); the bus echo is dropped.
    let first = tokio::time::timeout(Duration::from_millis(50), rx.recv())
        .await
        .expect("timed out waiting for local delivery")
        .expect("channel closed");
    assert!(matches!(first, ServerMessage::Event(_)));
    let second = tokio::time::timeout(Duration::from_millis(50), rx.recv()).await;
    assert!(
        second.is_err(),
        "echo suppression failed — received a second copy"
    );
}

/// SC3 / D-05 — Presence membership stays per-process.
///
/// Validates: `MemberAdded` emitted when a presence member joins on replica A is
/// NOT delivered to an observer on replica B. The `fan_out` filter allows only
/// `ServerMessage::Event(_)` onto the bus; presence variants are per-process.
#[tokio::test(flavor = "multi_thread")]
async fn presence_stays_per_process() {
    use ferro_broadcast::{BroadcastConfig, PresenceMember};

    // Both replicas share one bus AND the same signing secret so presence
    // subscriptions are accepted on the signature-verification path.
    let bus = Arc::new(InMemoryTransport::new(64));
    let cfg_a = BroadcastConfig::new().signing_secret("test-secret");
    let cfg_b = BroadcastConfig::new().signing_secret("test-secret");
    let a = Broadcaster::with_config(cfg_a)
        .with_transport(bus.clone())
        .await;
    let b = Broadcaster::with_config(cfg_b)
        .with_transport(bus.clone())
        .await;

    // Observer on replica B, subscribed to the SAME presence channel name.
    // If MemberAdded were fanned out, this observer's rx would receive it.
    let (tx_b, mut rx_b) = mpsc::channel(16);
    b.add_client("observer_b".into(), tx_b);
    let token_b = b
        .sign_subscription("observer_b", "presence-room", Some("observer"))
        .expect("signing enabled");
    b.subscribe(
        "observer_b",
        "presence-room",
        Some(&token_b),
        Some(PresenceMember::new("observer_b", "observer")),
    )
    .await
    .unwrap();

    // No sleep needed — readiness guaranteed by construction (WR-02 closed).

    // Drain the local MemberAdded that B's own presence subscribe delivered to
    // its existing members (a local, per-process event) so the assertion window
    // only observes anything that would have crossed the bus.
    while tokio::time::timeout(Duration::from_millis(20), rx_b.recv())
        .await
        .is_ok()
    {}

    // Replica A adds a presence member. This emits MemberAdded on A only;
    // it must never fan out to the bus (D-05), so B's observer must not see it.
    let (tx_a, _rx_a) = mpsc::channel(16);
    a.add_client("member_a".into(), tx_a);
    let token_a = a
        .sign_subscription("member_a", "presence-room", Some("alice"))
        .expect("signing enabled");
    a.subscribe(
        "member_a",
        "presence-room",
        Some(&token_a),
        Some(PresenceMember::new("member_a", "alice")),
    )
    .await
    .unwrap();

    // B's observer must receive NOTHING from the bus for A's membership change.
    let leaked = tokio::time::timeout(Duration::from_millis(50), rx_b.recv()).await;
    assert!(
        leaked.is_err(),
        "presence membership leaked across the bus: {leaked:?}"
    );
}

/// D-06 — Bus publish error does not fail the caller.
///
/// Validates: when the transport's `publish()` returns `Err`, `broadcast()` still
/// returns `Ok(())` and local delivery still completes successfully.
struct FailingTransport;

#[ferro_broadcast::async_trait]
impl ferro_broadcast::BroadcastTransport for FailingTransport {
    async fn publish(
        &self,
        _e: &ferro_broadcast::BusEnvelope,
    ) -> Result<(), ferro_broadcast::Error> {
        Err(ferro_broadcast::Error::transport("boom"))
    }

    async fn subscribe_loop(
        &self,
        sink: tokio::sync::mpsc::Sender<ferro_broadcast::BusEnvelope>,
        ready: tokio::sync::oneshot::Sender<()>,
    ) -> Result<(), ferro_broadcast::Error> {
        // Drop ready without firing — with_transport receives Err(RecvError) and
        // degrades to local-only per D-02/D-06. The subscription path is not under
        // test here; only the publish-error propagation is tested.
        drop(ready);
        sink.closed().await; // park until the receiver is dropped
        Ok(())
    }
}

#[tokio::test]
async fn publish_error_does_not_propagate() {
    let a = Broadcaster::with_config(Default::default())
        .with_transport(Arc::new(FailingTransport))
        .await;
    let (tx, mut rx) = mpsc::channel(16);
    a.add_client("socket_a".into(), tx);
    a.subscribe("socket_a", "chat", None, None).await.unwrap();

    // Caller must observe Ok(()) despite the transport publish error (D-06).
    a.broadcast("chat", "Msg", serde_json::json!({}))
        .await
        .unwrap();

    // Local delivery still happened.
    let first = tokio::time::timeout(Duration::from_millis(50), rx.recv())
        .await
        .expect("local delivery must still occur")
        .expect("channel closed");
    assert!(matches!(first, ServerMessage::Event(_)));
}
