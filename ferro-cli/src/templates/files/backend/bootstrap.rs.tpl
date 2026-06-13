//! Application Bootstrap
//!
//! This is where you register global middleware, services, and initialize
//! framework features like events, queue workers, and broadcasting.
//!
//! # Example
//!
//! ```rust,ignore
//! // For services with no runtime config, use the macro:
//! #[service(RedisCache)]
//! pub trait CacheStore { ... }
//!
//! // For services needing runtime config, register here:
//! pub async fn register() {
//!     // Initialize database
//!     DB::init().await.expect("Failed to connect to database");
//!
//!     // Global middleware
//!     global_middleware!(middleware::LoggingMiddleware);
//!
//!     // Event listeners
//!     register_listeners(App::event_dispatcher());
//!
//!     // Services
//!     bind!(dyn Database, PostgresDB::new());
//! }
//! ```

#[allow(unused_imports)]
use ferro::{
    bind, global_middleware, singleton, App, CsrfMiddleware,
    SecurityHeaders, SessionConfig, SessionMiddleware, DB,
    // Events
    EventDispatcher,
    // Queue (accessed as ferro::queue::Queue and ferro::queue::QueueConfig)
    queue::{Queue, QueueConfig},
    // Storage
    Storage, DiskConfig,
    // Cache
    Cache, CacheConfig,
};

use crate::middleware;

// Import your events and listeners
// use crate::events;
// use crate::listeners;

/// Register global middleware and services
///
/// Called from main.rs before `Server::from_config()`.
/// Middleware and services registered here can use environment variables, config files, etc.
pub async fn register() {
    // Initialize tracing for logs
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("ferro=info".parse().unwrap())
        )
        .init();

    // Initialize database connection
    DB::init().await.expect("Failed to connect to database");

    // Global middleware (runs on every request in registration order)
    global_middleware!(middleware::LoggingMiddleware);

    // Session middleware (required for authentication)
    let session_config = SessionConfig::from_env();
    global_middleware!(SessionMiddleware::new(session_config));

    // CSRF protection (validates tokens on POST/PUT/PATCH/DELETE)
    global_middleware!(CsrfMiddleware::new());

    // Security headers (X-Content-Type-Options, X-Frame-Options, CSP, etc.)
    global_middleware!(SecurityHeaders::new());
    // For production with HTTPS, enable HSTS:
    // global_middleware!(SecurityHeaders::new().with_hsts());

    // =========================================================================
    // Event Listeners
    // =========================================================================
    // Register event listeners for your domain events
    // Example:
    // let dispatcher = App::event_dispatcher();
    // dispatcher.listen::<events::UserRegistered, _>(listeners::SendWelcomeEmail);
    // dispatcher.listen::<events::OrderPlaced, _>(listeners::NotifyAdmin);

    // =========================================================================
    // Queue Workers
    // =========================================================================
    // Configure queue connection (defaults to sync for development)
    // For production, set QUEUE_CONNECTION=redis in .env
    // Example:
    // let queue_config = QueueConfig::from_env();
    // App::bind_queue(Queue::new(queue_config));

    // =========================================================================
    // Storage
    // =========================================================================
    // Configure file storage disks
    // Example:
    // Storage::configure(DiskConfig::local("storage/app"));
    // Storage::configure(DiskConfig::s3("my-bucket"));

    // =========================================================================
    // Cache
    // =========================================================================
    // Configure cache store (defaults to memory for development)
    // For production, set CACHE_DRIVER=redis in .env
    // Example:
    // let cache_config = CacheConfig::from_env();
    // App::bind_cache(Cache::new(cache_config));

    // Add your middleware and service registrations here
}

/// Register event listeners
///
/// Called during bootstrap to wire up event listeners.
#[allow(dead_code)]
fn register_listeners(dispatcher: &EventDispatcher) {
    // Register your event listeners here
    // Example:
    // dispatcher.listen::<events::UserRegistered, _>(listeners::SendWelcomeEmail);
    // dispatcher.listen::<events::UserRegistered, _>(listeners::CreateUserProfile);
    //
    // You can also use closure listeners:
    // dispatcher.on::<events::UserRegistered, _, _>(|event| async move {
    //     println!("User {} registered!", event.user_id);
    //     Ok(())
    // });
    let _ = dispatcher; // Silence unused warning
}
