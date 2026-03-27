# Broadcasting

Ferro provides a WebSocket broadcasting system for real-time communication. Push events to connected clients through public, private, and presence channels. The server handles WebSocket connections at `/_ferro/ws` with automatic heartbeat, timeout, and subscription management.

## Setup

### Registering the Broadcaster

In `bootstrap.rs`, create a `Broadcaster` and register it as a singleton. The broadcaster manages all connected clients and channel subscriptions.

```rust
use ferro::{Broadcaster, BroadcastConfig, App};

pub async fn register() {
    let broadcaster = Broadcaster::with_config(BroadcastConfig::from_env());
    App::singleton(broadcaster);
}
```

The framework automatically intercepts WebSocket upgrade requests to `/_ferro/ws` when a `Broadcaster` is registered. No additional route configuration is needed for the WebSocket endpoint itself.

### Manual Configuration

Instead of reading from environment variables, configure the broadcaster directly:

```rust
use ferro::{Broadcaster, BroadcastConfig};
use std::time::Duration;

let config = BroadcastConfig::new()
    .max_subscribers_per_channel(100)
    .max_channels(50)
    .heartbeat_interval(Duration::from_secs(30))
    .client_timeout(Duration::from_secs(60))
    .allow_client_events(true);

let broadcaster = Broadcaster::with_config(config);
```

## Channel Types

Channel type is determined by the channel name prefix:

| Type | Prefix | Authorization | Use Case |
|------|--------|---------------|----------|
| Public | none | No | News feeds, global notifications |
| Private | `private-` | Yes | User-specific data, order updates |
| Presence | `presence-` | Yes | Online status, who's typing |

```rust
// Public - anyone can subscribe
"orders"
"notifications"

// Private - requires authorization
"private-orders.123"
"private-user.456"

// Presence - tracks online members
"presence-chat.1"
"presence-room.gaming"
```

## Channel Authorization

Private and presence channels require an authorizer. Implement the `ChannelAuthorizer` trait and attach it to the broadcaster.

### Implementing a Channel Authorizer

```rust
use ferro::{AuthData, ChannelAuthorizer};

pub struct AppChannelAuth;

#[async_trait::async_trait]
impl ChannelAuthorizer for AppChannelAuth {
    async fn authorize(&self, data: &AuthData) -> bool {
        // data.socket_id   - client's WebSocket connection ID
        // data.channel     - channel name being requested
        // data.auth_token  - user ID from session auth (set by broadcasting_auth)

        match data.channel.as_str() {
            c if c.starts_with("private-orders.") => {
                let order_id: i64 = c
                    .strip_prefix("private-orders.")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                // Check if user owns this order
                check_order_ownership(data.auth_token.as_deref(), order_id).await
            }
            c if c.starts_with("presence-chat.") => {
                // Allow all authenticated users to join chat
                data.auth_token.is_some()
            }
            _ => false,
        }
    }
}
```

### Registering the Authorizer

Chain `.with_authorizer()` when creating the broadcaster:

```rust
let broadcaster = Broadcaster::with_config(BroadcastConfig::from_env())
    .with_authorizer(AppChannelAuth);
App::singleton(broadcaster);
```

## Auth Endpoint

Clients connecting to private or presence channels must authenticate through an HTTP endpoint. Ferro provides `broadcasting_auth`, a handler that bridges session authentication with channel authorization.

### Registering the Auth Route

```rust
use ferro::broadcasting_auth;

Route::post("/broadcasting/auth", broadcasting_auth)
    .middleware(SessionAuthMiddleware);
```

The handler:

1. Verifies the user is authenticated via session (`Auth::id()`)
2. Receives `channel_name` and `socket_id` from the client
3. Calls `Broadcaster::check_auth()` with the user's ID as the auth token
4. Returns 200 with auth confirmation if authorized, 401 if unauthenticated, 403 if unauthorized
5. For presence channels, includes `channel_data` with `user_id`

### Private Channel Auth Flow

The full authorization flow for private and presence channels:

1. Client connects to `ws://host/_ferro/ws` and receives a `socket_id`
2. Client sends HTTP POST to `/broadcasting/auth` with `channel_name` and `socket_id`
3. Server validates session auth and calls the registered `ChannelAuthorizer`
4. If authorized, client receives auth confirmation
5. Client sends a `subscribe` message over WebSocket with the `auth` token
6. Server subscribes the client to the channel

## Broadcasting from Handlers

### Fluent Builder API

The `Broadcast` builder provides a chainable interface for sending events:

```rust
use ferro::{Broadcast, Broadcaster, App};
use std::sync::Arc;

#[handler]
pub async fn update_order(req: Request, id: Path<i32>) -> Response {
    let db = req.db();
    let order = update_order_in_db(db, *id).await?;

    let broadcaster = App::get::<Broadcaster>().ok_or_else(|| HttpResponse::internal_server_error())?;
    let broadcast = Broadcast::new(Arc::new(broadcaster));

    broadcast
        .channel(&format!("orders.{}", id))
        .event("OrderUpdated")
        .data(&order)
        .send()
        .await
        .ok();

    Ok(json!(order))
}
```

### Excluding the Sender

When a client triggers an action, exclude them from the broadcast to avoid echo:

```rust
broadcast
    .channel("chat.1")
    .event("NewMessage")
    .data(&message)
    .except(&socket_id)
    .send()
    .await?;
```

### Direct Broadcaster API

For simpler cases, call `broadcast()` or `broadcast_except()` directly:

```rust
let broadcaster = App::get::<Broadcaster>().expect("Broadcaster not registered");

// Broadcast to all subscribers
broadcaster.broadcast("orders", "OrderCreated", &order).await?;

// Broadcast excluding a specific client
broadcaster
    .broadcast_except("chat.1", "MessageSent", &msg, &sender_socket_id)
    .await?;
```

## Client Connection

Clients connect via standard WebSocket to the `/_ferro/ws` endpoint. All messages use JSON.

### JavaScript Client

```javascript
const ws = new WebSocket('ws://localhost:8080/_ferro/ws');

ws.onopen = () => {
    console.log('Connected');
};

ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);

    switch (msg.type) {
        case 'connected':
            console.log('Socket ID:', msg.socket_id);
            // Subscribe to public channel
            ws.send(JSON.stringify({
                type: 'subscribe',
                channel: 'orders'
            }));
            break;

        case 'subscribed':
            console.log('Subscribed to:', msg.channel);
            break;

        case 'subscription_error':
            console.error('Failed:', msg.channel, msg.error);
            break;

        case 'event':
            console.log('Event:', msg.event, msg.data);
            break;

        case 'member_added':
            console.log('Joined:', msg.user_id, msg.user_info);
            break;

        case 'member_removed':
            console.log('Left:', msg.user_id);
            break;

        case 'pong':
            // Keepalive response
            break;

        case 'error':
            console.error('Error:', msg.message);
            break;
    }
};
```

### Subscribing to Private Channels

Private channels require an auth token. The client first authenticates via the HTTP endpoint, then includes the token in the subscribe message:

```javascript
async function subscribePrivate(ws, socketId, channel) {
    // Step 1: Get auth token from server
    const res = await fetch('/broadcasting/auth', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include', // Send session cookie
        body: JSON.stringify({
            channel_name: channel,
            socket_id: socketId
        })
    });

    if (!res.ok) {
        throw new Error('Authorization failed');
    }

    const data = await res.json();

    // Step 2: Subscribe with auth token
    ws.send(JSON.stringify({
        type: 'subscribe',
        channel: channel,
        auth: data.auth
    }));
}
```

## Whisper (Client Events)

Whisper messages are client-to-client events forwarded through the server. They are useful for ephemeral state like typing indicators and cursor positions. The sender is excluded from receiving the message.

```javascript
// Send a whisper
ws.send(JSON.stringify({
    type: 'whisper',
    channel: 'private-chat.1',
    event: 'typing',
    data: { name: 'Alice' }
}));
```

Whisper requires `allow_client_events: true` in configuration (enabled by default) and the sender must be subscribed to the channel.

## Message Protocol

### Client to Server

All client messages are JSON with a `type` field:

```json
// Subscribe to a channel
{"type": "subscribe", "channel": "orders"}

// Subscribe to a private channel with auth
{"type": "subscribe", "channel": "private-orders.1", "auth": "token"}

// Unsubscribe
{"type": "unsubscribe", "channel": "orders"}

// Whisper (client event)
{"type": "whisper", "channel": "chat", "event": "typing", "data": {"name": "Alice"}}

// Keepalive ping
{"type": "ping"}
```

### Server to Client

```json
// Connection established
{"type": "connected", "socket_id": "uuid-v4-here"}

// Subscription confirmed
{"type": "subscribed", "channel": "orders"}

// Subscription failed
{"type": "subscription_error", "channel": "private-secret", "error": "Authorization required"}

// Unsubscribed
{"type": "unsubscribed", "channel": "orders"}

// Broadcast event
{"type": "event", "event": "OrderUpdated", "channel": "orders", "data": {"id": 1}}

// Presence member joined
{"type": "member_added", "channel": "presence-chat.1", "user_id": "42", "user_info": {"name": "Alice"}}

// Presence member left
{"type": "member_removed", "channel": "presence-chat.1", "user_id": "42"}

// Keepalive response
{"type": "pong"}

// Error
{"type": "error", "message": "Invalid message format"}
```

## Presence Channels

Presence channels extend private channels with member tracking. When users join or leave, events are automatically broadcast to all channel members.

### Subscribing with Member Info

On the server side, presence subscriptions include member metadata:

```rust
use ferro::PresenceMember;

let member = PresenceMember::new(socket_id, user_id)
    .with_info(serde_json::json!({
        "name": user.name,
        "avatar": user.avatar_url,
    }));

broadcaster
    .subscribe(&socket_id, "presence-chat.1", Some(&auth_token), Some(member))
    .await?;
```

### Member Events

The server automatically broadcasts when members join or leave:

- `member_added` -- includes `user_id` and `user_info`
- `member_removed` -- includes `user_id`

### Querying Channel Members

```rust
if let Some(channel) = broadcaster.get_channel("presence-chat.1") {
    for member in channel.get_members() {
        println!("User {} is online", member.user_id);
    }
}
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `BROADCAST_MAX_SUBSCRIBERS` | Max subscribers per channel (0 = unlimited) | 0 |
| `BROADCAST_MAX_CHANNELS` | Max total channels (0 = unlimited) | 0 |
| `BROADCAST_HEARTBEAT_INTERVAL` | Heartbeat interval in seconds | 30 |
| `BROADCAST_CLIENT_TIMEOUT` | Client timeout in seconds (disconnect if no activity) | 60 |
| `BROADCAST_ALLOW_CLIENT_EVENTS` | Allow whisper messages (`true`/`false`) | true |

### Connection Management

The WebSocket connection handler runs a `tokio::select!` loop that manages:

- **Incoming frames** -- client messages dispatched to the broadcaster
- **Server messages** -- broadcast events forwarded to the client
- **Heartbeat** -- periodic ping/pong to detect stale connections

Clients that exceed `BROADCAST_CLIENT_TIMEOUT` without activity are disconnected. The server sends Close frames on clean shutdown and removes the client from all subscribed channels.

## Monitoring

```rust
let broadcaster = App::get::<Broadcaster>().expect("Broadcaster not registered");

// Connection stats
let clients = broadcaster.client_count();
let channels = broadcaster.channel_count();

// Channel details
if let Some(channel) = broadcaster.get_channel("orders") {
    println!("{} subscribers", channel.subscriber_count());
}

## MCP Tools

Use `list_broadcast_channels` to inspect active WebSocket channels at runtime.

### `list_broadcast_channels`

Returns all currently active broadcast channels with subscriber count, channel type (public, private, presence), and connected client IDs. Use this to verify that clients have subscribed to the expected channels or to diagnose subscription failures.
```
