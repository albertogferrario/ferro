# Events & Listeners

Ferro provides a Laravel-inspired event system for decoupling your application components. Events represent something that happened, while listeners react to those events.

## Creating Events

### Using the CLI

Generate a new event:

```bash
ferro make:event OrderPlaced
```

This creates `src/events/order_placed.rs`:

```rust
use ferro::Event;

#[derive(Clone)]
pub struct OrderPlaced {
    pub order_id: i64,
    pub user_id: i64,
    pub total: f64,
}

impl Event for OrderPlaced {
    fn name(&self) -> &'static str {
        "OrderPlaced"
    }
}
```

### Event Requirements

Events must implement:
- `Clone` - Events may be sent to multiple listeners
- `Send + Sync + 'static` - For async safety
- `Event` trait - Provides the event name

## Creating Listeners

### Using the CLI

Generate a new listener:

```bash
ferro make:listener SendOrderConfirmation
```

This creates `src/listeners/send_order_confirmation.rs`:

```rust
use ferro::{Listener, Error, async_trait};
use crate::events::OrderPlaced;

pub struct SendOrderConfirmation;

#[async_trait]
impl Listener<OrderPlaced> for SendOrderConfirmation {
    async fn handle(&self, event: &OrderPlaced) -> Result<(), Error> {
        tracing::info!("Sending confirmation for order {}", event.order_id);
        // Send email logic...
        Ok(())
    }
}
```

### Listener Trait Methods

| Method | Description | Default |
|--------|-------------|---------|
| `handle(&self, event)` | Process the event | Required |
| `name(&self)` | Listener identifier | Type name |
| `should_stop_propagation(&self)` | Stop other listeners | false |

## Registering Listeners

Register listeners in `src/bootstrap.rs`:

```rust
use ferro::{App, EventDispatcher};
use crate::events::OrderPlaced;
use crate::listeners::{SendOrderConfirmation, UpdateInventory, NotifyWarehouse};

pub async fn register() {
    // ... other setup ...

    let dispatcher = App::event_dispatcher();

    // Register listeners for OrderPlaced event
    dispatcher.listen::<OrderPlaced, _>(SendOrderConfirmation);
    dispatcher.listen::<OrderPlaced, _>(UpdateInventory);
    dispatcher.listen::<OrderPlaced, _>(NotifyWarehouse);
}
```

### Closure Listeners

For simple cases, use closures:

```rust
dispatcher.on::<OrderPlaced, _, _>(|event| async move {
    tracing::info!("Order {} placed!", event.order_id);
    Ok(())
});
```

## Dispatching Events

### Ergonomic API (Recommended)

Call `.dispatch()` directly on events:

```rust
use crate::events::OrderPlaced;

// In a controller or service
OrderPlaced {
    order_id: 123,
    user_id: 456,
    total: 99.99,
}
.dispatch()
.await?;
```

### Fire and Forget

Dispatch without waiting for listeners:

```rust
OrderPlaced {
    order_id: 123,
    user_id: 456,
    total: 99.99,
}
.dispatch_sync();  // Returns immediately
```

### Using the Dispatcher Directly

```rust
use ferro::dispatch;

dispatch(OrderPlaced {
    order_id: 123,
    user_id: 456,
    total: 99.99,
}).await?;
```

## Queued Listeners

For long-running tasks, queue listeners for background processing:

```rust
use ferro::{Listener, ShouldQueue, Error, async_trait};
use crate::events::OrderPlaced;

pub struct GenerateInvoicePDF;

// Mark as queued
impl ShouldQueue for GenerateInvoicePDF {
    fn queue(&self) -> &'static str {
        "invoices"  // Send to specific queue
    }

    fn delay(&self) -> Option<u64> {
        Some(30)  // Wait 30 seconds before processing
    }

    fn max_retries(&self) -> u32 {
        5
    }
}

#[async_trait]
impl Listener<OrderPlaced> for GenerateInvoicePDF {
    async fn handle(&self, event: &OrderPlaced) -> Result<(), Error> {
        // This runs in a background worker
        tracing::info!("Generating PDF for order {}", event.order_id);
        Ok(())
    }
}
```

## Stopping Propagation

Stop subsequent listeners from running:

```rust
impl Listener<OrderPlaced> for FraudChecker {
    async fn handle(&self, event: &OrderPlaced) -> Result<(), Error> {
        if self.is_fraudulent(event) {
            return Err(Error::msg("Fraudulent order detected"));
        }
        Ok(())
    }

    fn should_stop_propagation(&self) -> bool {
        true  // Other listeners won't run if this fails
    }
}
```

## Example: Order Processing

```rust
// events/order_placed.rs
#[derive(Clone)]
pub struct OrderPlaced {
    pub order_id: i64,
    pub user_id: i64,
    pub items: Vec<OrderItem>,
    pub total: f64,
}

impl Event for OrderPlaced {
    fn name(&self) -> &'static str { "OrderPlaced" }
}

// listeners/send_order_confirmation.rs
pub struct SendOrderConfirmation;

#[async_trait]
impl Listener<OrderPlaced> for SendOrderConfirmation {
    async fn handle(&self, event: &OrderPlaced) -> Result<(), Error> {
        let user = User::find(event.user_id).await?;
        Mail::to(&user.email)
            .subject("Order Confirmation")
            .template("emails/order-confirmation", event)
            .send()
            .await?;
        Ok(())
    }
}

// listeners/update_inventory.rs
pub struct UpdateInventory;

#[async_trait]
impl Listener<OrderPlaced> for UpdateInventory {
    async fn handle(&self, event: &OrderPlaced) -> Result<(), Error> {
        for item in &event.items {
            Product::decrement_stock(item.product_id, item.quantity).await?;
        }
        Ok(())
    }
}

// bootstrap.rs
dispatcher.listen::<OrderPlaced, _>(SendOrderConfirmation);
dispatcher.listen::<OrderPlaced, _>(UpdateInventory);
```

## Best Practices

1. **Keep events immutable** - Events are data, not behavior
2. **Use descriptive names** - Past tense for things that happened (OrderPlaced, UserRegistered)
3. **Include all needed data** - Listeners shouldn't need to fetch additional data
4. **Queue heavy operations** - Use `ShouldQueue` for emails, PDFs, external APIs
5. **Handle failures gracefully** - Listeners should not break on individual failures
6. **Test listeners in isolation** - Unit test each listener independently

## MCP Tools

Use `list_events` to discover all registered events and listeners in the project.

### `list_events`

Returns all `Event` implementations found in `src/events/`, each with the event name, fields, and which listeners are registered to handle it. Use this to understand the event graph before adding new listeners or debugging dispatch issues.
