//! Application Bootstrap
//!
//! This is where you register global middleware and services that need runtime configuration.
//! Services that don't need runtime config can use `#[service(ConcreteType)]` instead.
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
//!     // Initialize database (errors provide actionable guidance)
//!     DB::init().await.unwrap_or_else(|e| {
//!         eprintln!("Error: Failed to connect to database");
//!         eprintln!("  Cause: {}", e);
//!         std::process::exit(1);
//!     });
//!
//!     // Global middleware
//!     global_middleware!(middleware::LoggingMiddleware);
//!
//!     // Services
//!     bind!(dyn Database, PostgresDB::new());
//! }
//! ```

#[allow(unused_imports)]
use ferro::{
    bind, global_middleware, singleton, ApiKeyProvider, App, LangMiddleware, Limit, RateLimiter,
    UserProvider, DB,
};

use crate::middleware;
use crate::providers::{ApiKeyProviderImpl, DatabaseUserProvider};

/// Register global middleware and services
///
/// Called from main.rs before `Server::from_config()`.
/// Middleware and services registered here can use environment variables, config files, etc.
pub async fn register() {
    // Initialize database connection
    DB::init().await.unwrap_or_else(|e| {
        eprintln!("Error: Failed to connect to database");
        eprintln!("  Cause: {e}");
        eprintln!();
        eprintln!("How to fix:");
        eprintln!("  1. Check DATABASE_URL is set in .env");
        eprintln!("  2. Ensure the database server is running");
        eprintln!("  3. Verify connection credentials are correct");
        eprintln!();
        eprintln!("Example .env:");
        eprintln!("  DATABASE_URL=sqlite://./database.db");
        eprintln!("  DATABASE_URL=postgres://user:pass@localhost/mydb");
        std::process::exit(1);
    });

    // Global middleware (runs on every request in registration order)
    global_middleware!(middleware::LoggingMiddleware);
    global_middleware!(middleware::ShareInertiaData);
    global_middleware!(LangMiddleware);

    // Register the user provider for Auth::user()
    bind!(dyn UserProvider, DatabaseUserProvider);

    // Register the API key provider for ApiKeyMiddleware
    bind!(dyn ApiKeyProvider, ApiKeyProviderImpl);

    // Define named rate limiters for API routes
    RateLimiter::define("api", |_req| Limit::per_minute(60));

    // Example: Register a trait binding with runtime config
    // bind!(dyn Database, PostgresDB::new());

    // Example: Register a concrete singleton
    // singleton!(CacheService::new());

    // Add your middleware and service registrations here
}
