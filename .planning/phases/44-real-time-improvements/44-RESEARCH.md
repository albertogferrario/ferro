# Phase 44: Real-time Improvements - Research

**Researched:** 2026-02-10
**Domain:** WebSocket broadcasting with connection management and channel authorization for Ferro (Rust)
**Confidence:** HIGH

<research_summary>
## Summary

Researched the current state of ferro-broadcast, the Pusher WebSocket protocol (which Laravel Broadcasting implements), the tokio-tungstenite/hyper-tungstenite ecosystem, and the integration gaps between ferro-broadcast and the rest of the framework.

ferro-broadcast already has solid data structures: channel types (public/private/presence), message protocol (ClientMessage/ServerMessage), presence member tracking, a ChannelAuthorizer trait, and a fluent Broadcast API. What's missing is the **runtime glue**: no WebSocket upgrade handler, no heartbeat/timeout implementation, no whisper forwarding, no broadcasting auth endpoint, and no integration with ferro's session auth system (added in Phase 39-40).

**Primary recommendation:** Build a WebSocket handler in the framework crate that upgrades HTTP connections to WebSocket, implements the message loop with heartbeat/timeout, and add a `/broadcasting/auth` HTTP endpoint that integrates ChannelAuthorizer with session auth. Use hyper-tungstenite for the upgrade mechanism. The protocol should be Pusher-compatible (as Laravel Reverb is) so that Laravel Echo clients work out of the box.
</research_summary>

<standard_stack>
## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio-tungstenite | 0.28.0 | Async WebSocket protocol | De facto standard for async Rust WebSocket; ferro currently uses 0.26, needs bump |
| hyper-tungstenite | 0.19.0 | HTTP→WS upgrade for hyper | Only maintained bridge between hyper 1.x and tungstenite |
| hyper | 1.8.1 | HTTP server | Already in use by ferro |
| tokio | 1.x | Async runtime | Already in use by ferro |
| dashmap | 6.x | Concurrent client/channel maps | Already in use by ferro-broadcast |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| futures-util | 0.3 | Stream/Sink traits for WS | Already in ferro-broadcast; needed for split() on WS stream |
| uuid | 1.x | Socket ID generation | Already in ferro-broadcast |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| hyper-tungstenite | axum's ws extractor | Would require switching HTTP framework; not viable |
| tokio-tungstenite | fastwebsockets | Faster but less maintained; tungstenite is standard |
| Pusher protocol | Custom protocol | Pusher compat means Laravel Echo works; custom loses ecosystem |

### Version Bump Required

ferro-broadcast currently depends on `tokio-tungstenite = "0.26"`. hyper-tungstenite 0.19.0 requires `tokio-tungstenite = "0.28.0"` and `tungstenite = "0.28.0"`.

**Breaking change in 0.26→0.28:** Message payload changed from `Vec<u8>` to `Bytes` (from the bytes crate), and Text messages use `Utf8Bytes` instead of `String`. This affects `message.rs` serialization but the change is mechanical.

**Installation (framework Cargo.toml):**
```toml
hyper-tungstenite = "0.19"
```

**Update (ferro-broadcast Cargo.toml):**
```toml
tokio-tungstenite = "0.28"
```
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Current ferro-broadcast Architecture
```
ferro-broadcast/src/
├── lib.rs          # Public API re-exports
├── broadcast.rs    # Fluent Broadcast builder (server→channel)
├── broadcaster.rs  # Broadcaster: client registry, channel management, message dispatch
├── channel.rs      # ChannelInfo, ChannelType, PresenceMember
├── config.rs       # BroadcastConfig (heartbeat, timeout, limits)
├── error.rs        # Error types
└── message.rs      # BroadcastMessage, ClientMessage, ServerMessage
```

### What Needs to Be Added
```
framework/src/
├── server.rs        # MODIFY: add .with_upgrades() to serve_connection
└── websocket.rs     # NEW: WebSocket upgrade handler + message loop

ferro-broadcast/src/
├── broadcaster.rs   # MODIFY: add whisper forwarding
└── (Cargo.toml)     # MODIFY: bump tokio-tungstenite to 0.28
```

### Pattern 1: WebSocket Upgrade via hyper-tungstenite
**What:** Intercept upgrade requests in the server loop and hand off to WebSocket handler
**When to use:** Every WebSocket connection
**Critical detail:** `serve_connection` MUST chain `.with_upgrades()` or upgrades silently fail

Current server.rs (line 108):
```rust
// CURRENT - does NOT support upgrades
http1::Builder::new().serve_connection(io, service).await

// REQUIRED - enables HTTP upgrade to WebSocket
http1::Builder::new().serve_connection(io, service).with_upgrades().await
```

The WebSocket upgrade handler:
```rust
// In handle_request, check for WS upgrade at a designated path
if path == "/_ferro/ws" && hyper_tungstenite::is_upgrade_request(&req) {
    let (response, ws_future) = hyper_tungstenite::upgrade(&mut req, None)?;

    // Spawn the WebSocket connection handler
    let broadcaster = App::get::<Arc<Broadcaster>>().unwrap();
    tokio::spawn(async move {
        let ws_stream = ws_future.await.unwrap();
        handle_websocket(ws_stream, broadcaster).await;
    });

    // Return the upgrade response immediately
    return response;
}
```

### Pattern 2: WebSocket Message Loop with Heartbeat
**What:** Split WS stream into read/write halves, run select loop with heartbeat timer
**When to use:** Every active WebSocket connection

```rust
async fn handle_websocket(ws_stream: WebSocketStream, broadcaster: Arc<Broadcaster>) {
    let socket_id = Uuid::new_v4().to_string();
    let (mut ws_write, mut ws_read) = ws_stream.split();
    let (tx, mut rx) = mpsc::channel::<ServerMessage>(32);

    broadcaster.add_client(socket_id.clone(), tx);

    // Send connection_established
    let connected = ServerMessage::Connected { socket_id: socket_id.clone() };
    ws_write.send(Message::Text(connected.to_json().unwrap())).await;

    let config = broadcaster.config().clone();
    let mut heartbeat_interval = tokio::time::interval(config.heartbeat_interval);
    let mut last_pong = Instant::now();

    loop {
        tokio::select! {
            // Client message
            msg = ws_read.next() => { /* parse ClientMessage, dispatch */ }
            // Server message to forward
            msg = rx.recv() => { /* serialize and send via ws_write */ }
            // Heartbeat tick
            _ = heartbeat_interval.tick() => {
                if last_pong.elapsed() > config.client_timeout {
                    break; // Client timed out
                }
                ws_write.send(Message::Ping(vec![].into())).await;
            }
        }
    }

    broadcaster.remove_client(&socket_id);
}
```

### Pattern 3: Broadcasting Auth Endpoint
**What:** HTTP POST endpoint that authorizes private/presence channel access using session auth
**When to use:** Before subscribing to private/presence channels

Laravel pattern: `POST /broadcasting/auth` with `channel_name` and `socket_id`.

Ferro implementation: Use the existing `Auth::id()` facade (session-based) to identify the user, then call the registered `ChannelAuthorizer`.

```rust
// POST /broadcasting/auth
#[handler]
pub async fn authorize_channel(req: Request) -> Response {
    let user_id = Auth::id().ok_or_else(|| FrameworkError::domain("Unauthenticated", 401))?;
    let form = req.input().await?;
    let channel_name: String = form.get("channel_name");
    let socket_id: String = form.get("socket_id");

    let broadcaster = App::get::<Arc<Broadcaster>>().unwrap();
    let auth_data = AuthData {
        socket_id,
        channel: channel_name,
        auth_token: Some(user_id.to_string()),
    };

    // Delegate to registered authorizer
    // Return success or 403
}
```

### Pattern 4: Pusher-Compatible Protocol (Simplified)
**What:** Use Pusher protocol message format so Laravel Echo clients work
**When to use:** If frontend uses Laravel Echo

Pusher protocol wraps everything in `{"event": "...", "data": {...}}`:
```json
// Connection established
{"event": "pusher:connection_established", "data": "{\"socket_id\":\"123.456\",\"activity_timeout\":120}"}

// Subscribe
{"event": "pusher:subscribe", "data": {"channel": "private-orders.1", "auth": "..."}}

// Subscription succeeded
{"event": "pusher_internal:subscription_succeeded", "channel": "private-orders.1", "data": "{}"}

// Broadcast event
{"event": "OrderUpdated", "channel": "orders.1", "data": "{\"id\":1}"}

// Client event (whisper) - must start with "client-"
{"event": "client-typing", "channel": "private-chat.1", "data": "{\"name\":\"Alice\"}"}
```

**Decision needed:** Use Pusher protocol format (Laravel Echo compatible) or keep ferro's current simpler `{"type":"..."}` format? The current format is already defined and used. Pusher compat could be added as an optional layer.

### Anti-Patterns to Avoid
- **Blocking in the WS message loop:** Never do DB queries or HTTP calls in the select loop; use channels
- **Not calling `.with_upgrades()`:** Upgrades silently fail without this on `serve_connection`
- **Holding DashMap guards across awaits:** This will deadlock; always drop guards before .await
- **Manual ping/pong at application level when tungstenite handles it:** Tungstenite auto-responds to Ping frames with Pong. Server heartbeat should use WS-level Ping frames, not application-level JSON messages
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTTP→WS upgrade | Manual header parsing + upgrade | hyper-tungstenite `upgrade()` | Handles Sec-WebSocket-Key, Sec-WebSocket-Version, response headers correctly |
| WebSocket framing | Custom frame parser | tungstenite (via tokio-tungstenite) | RFC 6455 compliant, handles masking, fragmentation, control frames |
| Ping/Pong response | Application-level pong logic | tungstenite auto-pong | Library automatically queues Pong when Ping received |
| Socket ID generation | Custom ID scheme | UUID v4 | Already in use; collision-free, no coordination needed |
| Concurrent client map | RwLock<HashMap> | DashMap | Already in use; better performance under contention |
| Pusher protocol parsing | Custom parser | Serde with tagged enums | Already partially done in message.rs |

**Key insight:** The WebSocket protocol (RFC 6455) has many edge cases in framing, masking, close handshake, and UTF-8 validation. tungstenite handles all of these. The only custom code needed is the **message loop** (connecting WS frames to the broadcaster) and the **auth endpoint** (connecting session auth to channel authorization).
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Missing `.with_upgrades()` on serve_connection
**What goes wrong:** WebSocket upgrade requests get a normal HTTP response instead of upgrading; client sees connection refused or protocol error
**Why it happens:** hyper 1.x requires explicit opt-in for HTTP upgrades; without it, the upgrade future never resolves
**How to avoid:** Change `http1::Builder::new().serve_connection(io, service).await` to `http1::Builder::new().serve_connection(io, service).with_upgrades().await`
**Warning signs:** WebSocket connections fail with "unexpected response" but regular HTTP works fine

### Pitfall 2: DashMap Guard Held Across .await
**What goes wrong:** Deadlock when two tasks try to access the same DashMap entry
**Why it happens:** DashMap guards are not Send; holding them across await points causes issues. The current code already has `drop(channel)` before broadcasting calls in `broadcaster.rs`, showing awareness of this
**How to avoid:** Always `drop()` DashMap guards before any `.await` call; clone data out if needed
**Warning signs:** Server hangs when multiple clients subscribe to the same channel simultaneously

### Pitfall 3: Confusing WS-level Ping with Application-level Ping
**What goes wrong:** Double ping/pong traffic; or heartbeat doesn't detect dead connections
**Why it happens:** tungstenite handles RFC 6455 Ping/Pong frames automatically. The current `ClientMessage::Ping` / `ServerMessage::Pong` are application-level JSON messages, separate from WS-level ping frames
**How to avoid:** Use WS-level Ping frames (`Message::Ping`) for heartbeat detection. The application-level Ping/Pong in ferro's protocol can remain for client-initiated keepalive but server heartbeat should use WS frames
**Warning signs:** Dead connections not detected; or clients send both WS ping and JSON ping

### Pitfall 4: Not Handling WebSocket Close Handshake
**What goes wrong:** Resource leak; broadcaster keeps "connected" clients that are gone
**Why it happens:** WebSocket close is a two-way handshake; if you just drop the stream, the close may not complete cleanly
**How to avoid:** When receiving `Message::Close`, send close frame back, then break the loop and call `remove_client`. When the read stream returns `None`, also clean up
**Warning signs:** `broadcaster.client_count()` keeps growing; memory leak over time

### Pitfall 5: Auth Endpoint Without Session Context
**What goes wrong:** `Auth::id()` returns None even for authenticated users
**Why it happens:** The broadcasting auth endpoint needs session middleware to run; if it's registered as a bare handler without the session middleware chain, the session won't be loaded
**How to avoid:** Ensure the `/broadcasting/auth` route goes through the normal middleware stack (session middleware must run first)
**Warning signs:** All private channel subscriptions fail with "Unauthenticated"
</common_pitfalls>

<code_examples>
## Code Examples

### hyper-tungstenite Upgrade (from official docs)
```rust
// Source: hyper-tungstenite docs
use hyper_tungstenite::{is_upgrade_request, upgrade};
use hyper::Response;

async fn handle(mut req: hyper::Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    if is_upgrade_request(&req) {
        let (response, websocket) = upgrade(&mut req, None).unwrap();

        tokio::spawn(async move {
            let ws = websocket.await.unwrap();
            // Use ws as a WebSocketStream
        });

        // response type is hyper::Response<Full<Bytes>>
        Ok(response)
    } else {
        Ok(Response::new(Full::new(Bytes::from("Hello"))))
    }
}
```

### serve_connection with upgrades (from hyper docs)
```rust
// Source: hyper docs
http1::Builder::new()
    .serve_connection(io, service)
    .with_upgrades()  // Required for WebSocket
    .await
```

### WebSocket Split + Select Loop (from tokio-tungstenite patterns)
```rust
// Source: tokio-tungstenite examples + community patterns
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

let (mut write, mut read) = ws_stream.split();

loop {
    tokio::select! {
        Some(Ok(msg)) = read.next() => {
            match msg {
                Message::Text(text) => {
                    // Parse client message
                }
                Message::Close(_) => break,
                _ => {} // tungstenite handles Ping automatically
            }
        }
        Some(server_msg) = rx.recv() => {
            let json = serde_json::to_string(&server_msg).unwrap();
            if write.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
        _ = heartbeat.tick() => {
            if write.send(Message::Ping(vec![].into())).await.is_err() {
                break;
            }
        }
        else => break,
    }
}
```

### Pusher Protocol Connection Established
```json
// Source: Pusher WebSocket Protocol spec
{
  "event": "pusher:connection_established",
  "data": "{\"socket_id\":\"131232.12312\",\"activity_timeout\":120}"
}
```

### Pusher Protocol Subscribe (Private Channel)
```json
// Source: Pusher WebSocket Protocol spec
{
  "event": "pusher:subscribe",
  "data": {
    "channel": "private-orders.1",
    "auth": "APP_KEY:signature"
  }
}
```

### Laravel Broadcasting Auth Response
```json
// Source: Laravel 12.x docs
// Private channel - return true/false from callback
// Presence channel - return user data array
{
  "auth": "key:signature",
  "channel_data": "{\"user_id\":\"123\",\"user_info\":{\"name\":\"Alice\"}}"
}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| tungstenite Message::Text(String) | Message::Text(Utf8Bytes) | tungstenite 0.26 (Dec 2024) | Must update message serialization in ferro-broadcast |
| tungstenite Message::Binary(Vec<u8>) | Message::Binary(Bytes) | tungstenite 0.26 (Dec 2024) | Zero-copy broadcasting possible (shared Bytes across clients) |
| Laravel Echo + Pusher SaaS | Laravel Reverb (self-hosted) | Laravel 11 (2024) | Validates self-hosted WS server approach; Ferro follows this pattern |
| hyper 0.14 upgrades | hyper 1.x with `.with_upgrades()` | hyper 1.0 (2023) | Must explicitly opt in to upgrade support |

**New patterns to consider:**
- **Bytes for zero-copy broadcast:** With tungstenite 0.28, broadcast messages can use shared `Bytes` instead of cloning `String` per client. Serialize once, send shared reference to all
- **Laravel Reverb's approach:** Self-hosted Pusher-compatible WS server validates ferro's architecture direction

**Deprecated/outdated:**
- **hyper 0.14 upgrade mechanism:** Completely different API; don't reference old examples
- **tungstenite < 0.26:** Uses String/Vec<u8> instead of Utf8Bytes/Bytes
</sota_updates>

<open_questions>
## Open Questions

1. **Pusher protocol compatibility vs ferro's current protocol**
   - What we know: ferro-broadcast already defines `ClientMessage`/`ServerMessage` with `serde(tag = "type")` format. Pusher uses `{"event": "...", "data": {...}}` format. They're incompatible
   - What's unclear: Should ferro adopt Pusher protocol (Laravel Echo compat) or keep its own? Could support both?
   - Recommendation: Keep ferro's simpler protocol as primary. If Pusher compat is desired later, it can be an adapter layer. Discuss with user before deciding

2. **Broadcasting auth endpoint location**
   - What we know: Laravel uses `POST /broadcasting/auth`. ferro has `/_ferro/*` for framework endpoints
   - What's unclear: Should it be `/_ferro/broadcasting/auth` (framework-managed) or user-defined route?
   - Recommendation: Provide a built-in handler function that users register as a route, similar to how auth controllers work in Phase 39

3. **Broadcaster storage in App container**
   - What we know: Docs reference `App::set_broadcaster()` but it doesn't exist. App container uses `App::singleton()` and `App::get::<T>()`
   - What's unclear: Should there be a dedicated method or just use the generic container?
   - Recommendation: Use `App::singleton(broadcaster)` and `App::get::<Arc<Broadcaster>>()` — follows existing patterns, no special method needed

4. **WebSocket path**
   - What we know: ferro uses `/_ferro/*` for built-in endpoints
   - What's unclear: Should WS endpoint be `/_ferro/ws`, `/ws`, or configurable?
   - Recommendation: `/_ferro/ws` to match the framework prefix convention; users don't typically need to customize this
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- ferro-broadcast source code — all 7 files read and analyzed
- framework/src/server.rs — HTTP server implementation, hyper 1.x usage
- framework/src/auth/guard.rs — Auth facade, session integration
- [hyper-tungstenite 0.19.0 docs](https://docs.rs/hyper-tungstenite/0.19.0) — upgrade API, dependency versions (hyper ^1.0, tokio-tungstenite ^0.28)
- [Pusher Channels Protocol](https://pusher.com/docs/channels/library_auth_reference/pusher-websockets-protocol/) — complete message format specification
- [Laravel 12.x Broadcasting docs](https://laravel.com/docs/12.x/broadcasting) — auth endpoint, channel authorization, Echo integration
- [tokio-tungstenite CHANGELOG](https://github.com/snapview/tokio-tungstenite/blob/master/CHANGELOG.md) — version 0.26→0.28 changes

### Secondary (MEDIUM confidence)
- [hyper-tungstenite GitHub Cargo.toml](https://github.com/de-vri-es/hyper-tungstenite-rs/blob/main/Cargo.toml) — confirmed exact dependency versions
- [tungstenite docs](https://docs.rs/tungstenite/latest/tungstenite/protocol/struct.WebSocket.html) — auto ping/pong behavior, close handshake
- [hyper 1.0 serve_connection docs](https://docs.rs/hyper/latest/hyper/server/conn/http1/struct.Builder.html) — with_upgrades() requirement
- [Scalable WebSocket Server with Tokio](https://oneuptime.com/blog/post/2026-01-25-scalable-websocket-server-tokio-rust/view) — production patterns

### Tertiary (LOW confidence - needs validation)
- None — all findings verified against official docs or source code
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: tokio-tungstenite + hyper-tungstenite for WebSocket in hyper 1.x
- Ecosystem: Pusher WebSocket protocol, Laravel Broadcasting/Echo patterns
- Patterns: WS upgrade, message loop with heartbeat, channel auth, concurrent connection management
- Pitfalls: .with_upgrades(), DashMap guard deadlocks, ping confusion, close handshake, session context

**Confidence breakdown:**
- Standard stack: HIGH — verified versions from Cargo.toml, crates.io, docs.rs
- Architecture: HIGH — based on reading ferro source code and hyper-tungstenite docs
- Pitfalls: HIGH — derived from actual code reading + documented library behaviors
- Code examples: HIGH — from official docs and verified library APIs

**Research date:** 2026-02-10
**Valid until:** 2026-03-12 (30 days — Rust WebSocket ecosystem stable)
</metadata>

---

*Phase: 44-real-time-improvements*
*Research completed: 2026-02-10*
*Ready for planning: yes*
