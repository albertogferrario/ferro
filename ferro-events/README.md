# ferro-events

Event dispatcher and listener system for the Ferro framework.

## Features

- Synchronous event listeners
- Asynchronous listeners
- Queued listeners via `ShouldQueue` marker
- Automatic listener discovery via inventory

## Usage

```rust
use ferro_events::{Event, Listener, dispatch};

// Define an event
#[derive(Clone)]
struct UserRegistered {
    user_id: i64,
    email: String,
}

impl Event for UserRegistered {
    fn name(&self) -> &'static str {
        "UserRegistered"
    }
}

// Define a listener
struct SendWelcomeEmail;

#[async_trait::async_trait]
impl Listener<UserRegistered> for SendWelcomeEmail {
    async fn handle(&self, event: &UserRegistered) -> Result<(), ferro_events::Error> {
        println!("Sending welcome email to {}", event.email);
        Ok(())
    }
}

// Dispatch an event
dispatch(UserRegistered {
    user_id: 1,
    email: "test@example.com".into(),
}).await;
```

## Queued Listeners

Mark a listener to be processed in the background:

```rust
impl ShouldQueue for SendWelcomeEmail {}
```

## License

MIT
