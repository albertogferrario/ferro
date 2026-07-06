//! Event dispatcher implementation.

use crate::{Error, Event, Listener};
use parking_lot::RwLock;
use std::any::TypeId;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Type alias for async listener functions.
type ListenerFn<E> =
    Arc<dyn Fn(&E) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> + Send + Sync>;

/// Type-erased listener storage.
struct ListenerEntry {
    /// The listener function.
    handler: Box<dyn std::any::Any + Send + Sync>,
    /// Priority (higher = runs first).
    priority: i32,
}

/// Global event dispatcher.
///
/// The dispatcher maintains a registry of listeners for each event type.
/// When an event is dispatched, all registered listeners are called in
/// priority order.
pub struct EventDispatcher {
    /// Listeners indexed by event TypeId.
    listeners: RwLock<HashMap<TypeId, Vec<ListenerEntry>>>,
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl EventDispatcher {
    /// Create a new event dispatcher.
    pub fn new() -> Self {
        Self {
            listeners: RwLock::new(HashMap::new()),
        }
    }

    /// Register a listener for an event type.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ferro_events::{EventDispatcher, Event, Listener, Error, async_trait};
    ///
    /// #[derive(Clone)]
    /// struct MyEvent;
    /// impl Event for MyEvent {
    ///     fn name(&self) -> &'static str { "MyEvent" }
    /// }
    ///
    /// struct MyListener;
    ///
    /// #[async_trait]
    /// impl Listener<MyEvent> for MyListener {
    ///     async fn handle(&self, _event: &MyEvent) -> Result<(), Error> {
    ///         Ok(())
    ///     }
    /// }
    ///
    /// let dispatcher = EventDispatcher::new();
    /// dispatcher.listen(MyListener);
    /// ```
    pub fn listen<E, L>(&self, listener: L)
    where
        E: Event,
        L: Listener<E>,
    {
        self.listen_with_priority(listener, 0);
    }

    /// Register a listener with a specific priority.
    ///
    /// Higher priority listeners are called first.
    pub fn listen_with_priority<E, L>(&self, listener: L, priority: i32)
    where
        E: Event,
        L: Listener<E>,
    {
        let listener = Arc::new(listener);
        let handler: ListenerFn<E> = Arc::new(move |event: &E| {
            let listener = Arc::clone(&listener);
            let event = event.clone();
            Box::pin(async move { listener.handle(&event).await })
        });

        let entry = ListenerEntry {
            handler: Box::new(handler),
            priority,
        };

        let type_id = TypeId::of::<E>();
        let mut listeners = self.listeners.write();
        let list = listeners.entry(type_id).or_default();
        list.push(entry);
        // Sort by priority (higher first)
        list.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Register a closure as a listener.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ferro_events::{EventDispatcher, Event, Error};
    ///
    /// #[derive(Clone)]
    /// struct UserCreated { id: i64 }
    /// impl Event for UserCreated {
    ///     fn name(&self) -> &'static str { "UserCreated" }
    /// }
    ///
    /// let dispatcher = EventDispatcher::new();
    /// dispatcher.on(|event: UserCreated| async move {
    ///     println!("User {} created", event.id);
    ///     Ok::<(), Error>(())
    /// });
    /// ```
    pub fn on<E, F, Fut>(&self, handler: F)
    where
        E: Event,
        F: Fn(E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), Error>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        let listener_fn: ListenerFn<E> = Arc::new(move |event: &E| {
            let handler = Arc::clone(&handler);
            let event = event.clone();
            Box::pin(async move { handler(event).await })
        });

        let entry = ListenerEntry {
            handler: Box::new(listener_fn),
            priority: 0,
        };

        let type_id = TypeId::of::<E>();
        let mut listeners = self.listeners.write();
        listeners.entry(type_id).or_default().push(entry);
    }

    /// Dispatch an event to all registered listeners.
    ///
    /// Returns `Ok(())` if all listeners succeeded, or the first error encountered.
    pub async fn dispatch<E: Event>(&self, event: E) -> Result<(), Error> {
        let type_id = TypeId::of::<E>();
        let event_name = event.name();

        debug!(event = event_name, "Dispatching event");

        let handlers: Vec<ListenerFn<E>> = {
            let listeners = self.listeners.read();
            match listeners.get(&type_id) {
                Some(entries) => entries
                    .iter()
                    .filter_map(|entry| entry.handler.downcast_ref::<ListenerFn<E>>().cloned())
                    .collect(),
                None => {
                    debug!(event = event_name, "No listeners registered");
                    return Ok(());
                }
            }
        };

        info!(
            event = event_name,
            listener_count = handlers.len(),
            "Calling listeners"
        );

        for handler in handlers {
            if let Err(e) = handler(&event).await {
                error!(event = event_name, error = %e, "Listener failed");
                return Err(e);
            }
        }

        debug!(event = event_name, "Event dispatched successfully");
        Ok(())
    }

    /// Dispatch an event without waiting for listeners to complete.
    ///
    /// This spawns the event handling as a background task.
    pub fn dispatch_async<E: Event + 'static>(&self, event: E) {
        let type_id = TypeId::of::<E>();
        let event_name = event.name();

        let handlers: Vec<ListenerFn<E>> = {
            let listeners = self.listeners.read();
            match listeners.get(&type_id) {
                Some(entries) => entries
                    .iter()
                    .filter_map(|entry| entry.handler.downcast_ref::<ListenerFn<E>>().cloned())
                    .collect(),
                None => return,
            }
        };

        tokio::spawn(async move {
            for handler in handlers {
                if let Err(e) = handler(&event).await {
                    error!(event = event_name, error = %e, "Async listener failed");
                }
            }
        });
    }

    /// Check if any listeners are registered for an event type.
    pub fn has_listeners<E: Event>(&self) -> bool {
        let type_id = TypeId::of::<E>();
        let listeners = self.listeners.read();
        listeners.get(&type_id).is_some_and(|v| !v.is_empty())
    }

    /// Remove all listeners for an event type.
    pub fn forget<E: Event>(&self) {
        let type_id = TypeId::of::<E>();
        let mut listeners = self.listeners.write();
        listeners.remove(&type_id);
    }

    /// Remove all listeners.
    pub fn flush(&self) {
        let mut listeners = self.listeners.write();
        listeners.clear();
    }
}

// Global dispatcher instance
static GLOBAL_DISPATCHER: std::sync::OnceLock<EventDispatcher> = std::sync::OnceLock::new();

/// Get the global event dispatcher.
pub fn global_dispatcher() -> &'static EventDispatcher {
    GLOBAL_DISPATCHER.get_or_init(EventDispatcher::new)
}

/// Dispatch an event using the global dispatcher.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_events::{dispatch, Event};
///
/// #[derive(Clone)]
/// struct UserLoggedIn { user_id: i64 }
/// impl Event for UserLoggedIn {
///     fn name(&self) -> &'static str { "UserLoggedIn" }
/// }
///
/// async fn login() {
///     // ... login logic ...
///     dispatch(UserLoggedIn { user_id: 123 }).await.unwrap();
/// }
/// ```
pub async fn dispatch<E: Event>(event: E) -> Result<(), Error> {
    global_dispatcher().dispatch(event).await
}

/// Dispatch an event synchronously (fire and forget).
///
/// This spawns the event handling as a background task and returns immediately.
pub fn dispatch_sync<E: Event + 'static>(event: E) {
    global_dispatcher().dispatch_async(event);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[derive(Clone)]
    struct TestEvent {
        value: u32,
    }

    impl Event for TestEvent {
        fn name(&self) -> &'static str {
            "TestEvent"
        }
    }

    #[tokio::test]
    async fn test_dispatch_to_closure() {
        let dispatcher = EventDispatcher::new();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        dispatcher.on::<TestEvent, _, _>(move |event| {
            let counter = Arc::clone(&counter_clone);
            async move {
                counter.fetch_add(event.value, Ordering::SeqCst);
                Ok(())
            }
        });

        dispatcher.dispatch(TestEvent { value: 5 }).await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    #[tokio::test]
    async fn test_multiple_listeners() {
        let dispatcher = EventDispatcher::new();
        let counter = Arc::new(AtomicU32::new(0));

        for _ in 0..3 {
            let counter_clone = Arc::clone(&counter);
            dispatcher.on::<TestEvent, _, _>(move |_| {
                let counter = Arc::clone(&counter_clone);
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            });
        }

        dispatcher.dispatch(TestEvent { value: 1 }).await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_priority_order() {
        let dispatcher = EventDispatcher::new();
        let order = Arc::new(RwLock::new(Vec::new()));

        // Register in reverse priority order
        for priority in [1, 3, 2] {
            let order_clone = Arc::clone(&order);
            let handler: ListenerFn<TestEvent> = Arc::new(move |_| {
                let order = Arc::clone(&order_clone);
                let p = priority;
                Box::pin(async move {
                    order.write().push(p);
                    Ok(())
                })
            });

            let entry = ListenerEntry {
                handler: Box::new(handler),
                priority,
            };

            let type_id = TypeId::of::<TestEvent>();
            let mut listeners = dispatcher.listeners.write();
            let list = listeners.entry(type_id).or_default();
            list.push(entry);
            list.sort_by(|a, b| b.priority.cmp(&a.priority));
        }

        dispatcher.dispatch(TestEvent { value: 0 }).await.unwrap();

        let result = order.read().clone();
        assert_eq!(result, vec![3, 2, 1]);
    }

    #[tokio::test]
    async fn test_has_listeners() {
        let dispatcher = EventDispatcher::new();
        assert!(!dispatcher.has_listeners::<TestEvent>());

        dispatcher.on::<TestEvent, _, _>(|_| async { Ok(()) });
        assert!(dispatcher.has_listeners::<TestEvent>());

        dispatcher.forget::<TestEvent>();
        assert!(!dispatcher.has_listeners::<TestEvent>());
    }

    #[tokio::test]
    async fn test_no_listeners() {
        let dispatcher = EventDispatcher::new();
        // Should not error when no listeners registered
        let result = dispatcher.dispatch(TestEvent { value: 1 }).await;
        assert!(result.is_ok());
    }
}
