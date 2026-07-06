//! # Ferro Events
//!
//! Event dispatcher and listener system for the Ferro framework.
//!
//! Provides a Laravel-inspired event system with support for:
//! - Synchronous listeners
//! - Asynchronous listeners
//! - Queued listeners (via `ShouldQueue` marker)
//! - Ergonomic dispatch API (`.dispatch()` on events)
//!
//! ## Example
//!
//! ```rust,ignore
//! use ferro_events::{Event, Listener, Error};
//!
//! #[derive(Clone)]
//! struct UserRegistered {
//!     user_id: i64,
//!     email: String,
//! }
//!
//! impl Event for UserRegistered {
//!     fn name(&self) -> &'static str {
//!         "UserRegistered"
//!     }
//! }
//!
//! struct SendWelcomeEmail;
//!
//! #[async_trait::async_trait]
//! impl Listener<UserRegistered> for SendWelcomeEmail {
//!     async fn handle(&self, event: &UserRegistered) -> Result<(), Error> {
//!         println!("Sending welcome email to {}", event.email);
//!         Ok(())
//!     }
//! }
//!
//! // Dispatch an event (ergonomic Laravel-style API)
//! UserRegistered { user_id: 1, email: "test@example.com".into() }.dispatch().await?;
//!
//! // Or fire and forget (spawns background task)
//! UserRegistered { user_id: 1, email: "test@example.com".into() }.dispatch_sync();
//! ```

mod dispatcher;
mod error;
mod traits;

pub use dispatcher::{dispatch, dispatch_sync, global_dispatcher, EventDispatcher};
pub use error::Error;
pub use traits::{Event, Listener, ShouldQueue};

/// Re-export async_trait for convenience
pub use async_trait::async_trait;
