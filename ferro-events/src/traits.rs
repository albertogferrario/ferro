//! Core traits for the event system.

use crate::Error;
use async_trait::async_trait;
use std::any::Any;

/// Marker trait for events that can be dispatched.
///
/// Events are simple data structures that represent something that happened
/// in your application. They should be cheap to clone and contain all the
/// data needed by listeners.
///
/// # Example
///
/// ```rust
/// use ferro_events::Event;
///
/// #[derive(Clone)]
/// struct OrderPlaced {
///     order_id: i64,
///     user_id: i64,
///     total: f64,
/// }
///
/// impl Event for OrderPlaced {
///     fn name(&self) -> &'static str {
///         "OrderPlaced"
///     }
/// }
///
/// // Dispatch the event (Laravel-style API):
/// // OrderPlaced { order_id: 1, user_id: 2, total: 99.99 }.dispatch().await?;
/// ```
pub trait Event: Clone + Send + Sync + 'static {
    /// Returns the name of the event for logging and debugging.
    fn name(&self) -> &'static str;

    /// Returns the event as Any for type erasure.
    fn as_any(&self) -> &dyn Any
    where
        Self: Sized,
    {
        self
    }

    /// Dispatch this event using the global dispatcher.
    ///
    /// This is the ergonomic Laravel-style API for dispatching events.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_events::Event;
    ///
    /// #[derive(Clone)]
    /// struct UserRegistered { user_id: i64 }
    /// impl Event for UserRegistered {
    ///     fn name(&self) -> &'static str { "UserRegistered" }
    /// }
    ///
    /// async fn register_user() -> Result<(), ferro_events::Error> {
    ///     // ... registration logic ...
    ///     UserRegistered { user_id: 123 }.dispatch().await?;
    ///     Ok(())
    /// }
    /// ```
    fn dispatch(
        self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Error>> + Send>>
    where
        Self: Sized,
    {
        Box::pin(crate::dispatch(self))
    }

    /// Dispatch this event without waiting (fire and forget).
    ///
    /// This spawns the event handling as a background task and returns immediately.
    /// Useful when you don't need to wait for listeners to complete.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_events::Event;
    ///
    /// #[derive(Clone)]
    /// struct PageViewed { page: String }
    /// impl Event for PageViewed {
    ///     fn name(&self) -> &'static str { "PageViewed" }
    /// }
    ///
    /// fn track_page_view(page: &str) {
    ///     PageViewed { page: page.to_string() }.dispatch_sync();
    /// }
    /// ```
    fn dispatch_sync(self)
    where
        Self: Sized,
    {
        crate::dispatch_sync(self)
    }
}

/// A listener that handles events of type `E`.
///
/// Listeners contain the logic that should run when an event is dispatched.
/// They can be synchronous or asynchronous.
///
/// # Example
///
/// ```rust
/// use ferro_events::{Event, Listener, Error, async_trait};
///
/// #[derive(Clone)]
/// struct UserRegistered { email: String }
///
/// impl Event for UserRegistered {
///     fn name(&self) -> &'static str { "UserRegistered" }
/// }
///
/// struct SendWelcomeEmail;
///
/// #[async_trait]
/// impl Listener<UserRegistered> for SendWelcomeEmail {
///     async fn handle(&self, event: &UserRegistered) -> Result<(), Error> {
///         println!("Welcome, {}!", event.email);
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait Listener<E: Event>: Send + Sync + 'static {
    /// Handle the event.
    ///
    /// This method is called when the event is dispatched. It receives
    /// an immutable reference to the event data.
    async fn handle(&self, event: &E) -> Result<(), Error>;

    /// Returns the name of the listener for logging and debugging.
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Whether this listener should stop propagation to other listeners.
    ///
    /// If this returns `true` after handling, no further listeners will
    /// be called for this event.
    fn should_stop_propagation(&self) -> bool {
        false
    }
}

/// Marker trait for listeners that should be queued for background processing.
///
/// Listeners implementing this trait will not be executed immediately.
/// Instead, they will be pushed to a job queue and processed asynchronously
/// by a worker.
///
/// # Example
///
/// ```rust
/// use ferro_events::{Event, Listener, ShouldQueue, Error, async_trait};
///
/// #[derive(Clone)]
/// struct LargeFileUploaded { path: String }
///
/// impl Event for LargeFileUploaded {
///     fn name(&self) -> &'static str { "LargeFileUploaded" }
/// }
///
/// struct ProcessUploadedFile;
///
/// impl ShouldQueue for ProcessUploadedFile {
///     fn queue(&self) -> &'static str {
///         "file-processing"
///     }
/// }
///
/// #[async_trait]
/// impl Listener<LargeFileUploaded> for ProcessUploadedFile {
///     async fn handle(&self, event: &LargeFileUploaded) -> Result<(), Error> {
///         // This will run in a background worker
///         println!("Processing file: {}", event.path);
///         Ok(())
///     }
/// }
/// ```
pub trait ShouldQueue {
    /// The queue name to dispatch this listener to.
    fn queue(&self) -> &'static str {
        "default"
    }

    /// The number of seconds to delay before processing.
    fn delay(&self) -> Option<u64> {
        None
    }

    /// The number of times to retry on failure.
    fn max_retries(&self) -> u32 {
        3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestEvent {
        message: String,
    }

    impl Event for TestEvent {
        fn name(&self) -> &'static str {
            "TestEvent"
        }
    }

    struct TestListener;

    #[async_trait]
    impl Listener<TestEvent> for TestListener {
        async fn handle(&self, event: &TestEvent) -> Result<(), Error> {
            assert_eq!(event.message, "hello");
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_listener_handle() {
        let listener = TestListener;
        let event = TestEvent {
            message: "hello".into(),
        };
        let result = listener.handle(&event).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_event_name() {
        let event = TestEvent {
            message: "test".into(),
        };
        assert_eq!(event.name(), "TestEvent");
    }

    // Event type for dispatch method test (unique to avoid test interference)
    #[derive(Clone)]
    struct DispatchTestEvent {
        value: u32,
    }

    impl Event for DispatchTestEvent {
        fn name(&self) -> &'static str {
            "DispatchTestEvent"
        }
    }

    #[tokio::test]
    async fn test_event_dispatch_method() {
        use crate::global_dispatcher;
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        // Register a listener using the global dispatcher
        global_dispatcher().on::<DispatchTestEvent, _, _>(move |event| {
            let counter = Arc::clone(&counter_clone);
            async move {
                counter.fetch_add(event.value, Ordering::SeqCst);
                Ok(())
            }
        });

        // Use the new ergonomic dispatch method
        let event = DispatchTestEvent { value: 42 };
        let result = event.dispatch().await;
        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 42);
    }

    // Event type for dispatch_sync test (unique to avoid test interference)
    #[derive(Clone)]
    struct SyncDispatchTestEvent {
        value: u32,
    }

    impl Event for SyncDispatchTestEvent {
        fn name(&self) -> &'static str {
            "SyncDispatchTestEvent"
        }
    }

    #[tokio::test]
    async fn test_event_dispatch_sync_method() {
        use crate::global_dispatcher;
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;
        use tokio::time::{sleep, Duration};

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        // Register a listener
        global_dispatcher().on::<SyncDispatchTestEvent, _, _>(move |event| {
            let counter = Arc::clone(&counter_clone);
            async move {
                counter.fetch_add(event.value, Ordering::SeqCst);
                Ok(())
            }
        });

        // Use dispatch_sync (fire and forget)
        let event = SyncDispatchTestEvent { value: 99 };
        event.dispatch_sync();

        // Give the background task time to complete
        sleep(Duration::from_millis(50)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 99);
    }
}
