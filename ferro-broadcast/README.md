# ferro-broadcast

WebSocket broadcasting and real-time channels for the Ferro framework.

## Features

- Public channels (anyone can subscribe)
- Private channels (require authorization)
- Presence channels (track online users)
- Laravel Echo compatible

## Usage

```rust
use ferro_broadcast::{Broadcast, Broadcaster};
use std::sync::Arc;

// Create a broadcaster
let broadcaster = Arc::new(Broadcaster::new());

// Broadcast to a public channel
Broadcast::new(broadcaster.clone())
    .channel("orders")
    .event("OrderCreated")
    .data(&order)
    .send()
    .await?;

// Broadcast to a private channel
Broadcast::new(broadcaster.clone())
    .channel("private-orders.1")
    .event("OrderUpdated")
    .data(&order)
    .send()
    .await?;
```

## Channel Types

Channels are determined by their name prefix:
- `orders` - Public channel
- `private-orders.1` - Private channel (requires auth)
- `presence-chat.1` - Presence channel (tracks members)

## Authorization

```rust
use ferro_broadcast::{AuthData, ChannelAuthorizer};

struct MyAuthorizer;

#[async_trait::async_trait]
impl ChannelAuthorizer for MyAuthorizer {
    async fn authorize(&self, data: &AuthData) -> bool {
        // Verify user can access channel
        verify_user_access(&data.channel, &data.auth_token)
    }
}
```

## License

MIT
