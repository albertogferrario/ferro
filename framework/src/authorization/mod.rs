//! Authorization system for Ferro framework.
//!
//! Provides Laravel-like authorization with Gates and Policies.
//!
//! # Overview
//!
//! Ferro authorization works through two main concepts:
//!
//! - **Gates**: Simple closures for checking abilities not tied to specific models
//! - **Policies**: Classes that organize authorization logic around a model
//!
//! # Gates
//!
//! Gates are simple closures that determine if a user can perform an action:
//!
//! ```rust,ignore
//! use ferro_rs::authorization::Gate;
//!
//! // In bootstrap.rs
//! Gate::define("view-dashboard", |user, _| {
//!     user.as_any().downcast_ref::<User>()
//!         .map(|u| (u.is_admin || u.has_role("manager")).into())
//!         .unwrap_or_else(AuthResponse::deny_silent)
//! });
//!
//! // Admin bypass
//! Gate::before(|user, _ability| {
//!     if let Some(u) = user.as_any().downcast_ref::<User>() {
//!         if u.is_super_admin {
//!             return Some(true);
//!         }
//!     }
//!     None
//! });
//!
//! // In controller
//! if Gate::allows("view-dashboard", None) {
//!     // Show dashboard
//! }
//!
//! // Or authorize (returns Result)
//! Gate::authorize("view-dashboard", None)?;
//! ```
//!
//! # Policies
//!
//! Policies organize authorization logic around a specific model:
//!
//! ```rust,ignore
//! use ferro_rs::authorization::{Policy, AuthResponse};
//!
//! pub struct PostPolicy;
//!
//! impl Policy<Post> for PostPolicy {
//!     type User = User;
//!
//!     fn before(&self, user: &Self::User, _ability: &str) -> Option<bool> {
//!         if user.is_admin {
//!             return Some(true);
//!         }
//!         None
//!     }
//!
//!     fn view(&self, _user: &Self::User, _post: &Post) -> AuthResponse {
//!         AuthResponse::allow()
//!     }
//!
//!     fn update(&self, user: &Self::User, post: &Post) -> AuthResponse {
//!         (user.id == post.user_id).into()
//!     }
//! }
//! ```
//!
//! # Authorizable Trait
//!
//! User models can use the `Authorizable` trait for convenient methods:
//!
//! ```rust,ignore
//! // Automatically available on all Authenticatable types
//! if user.can("update", Some(&post)) {
//!     // Show edit button
//! }
//!
//! user.authorize("delete", Some(&post))?;
//! ```
//!
//! # Middleware
//!
//! Protect routes with authorization middleware:
//!
//! ```rust,ignore
//! use ferro_rs::authorization::Authorize;
//! use ferro_rs::can;
//!
//! Route::get("/admin", admin_dashboard)
//!     .middleware(Authorize::ability("view-admin"));
//!
//! // Or using the macro
//! Route::get("/admin", admin_dashboard)
//!     .middleware(can!("view-admin"));
//! ```

mod authorizable;
mod error;
mod gate;
mod middleware;
mod policy;
mod response;

pub use authorizable::Authorizable;
pub use error::AuthorizationError;
pub use gate::Gate;
pub use middleware::Authorize;
pub use policy::Policy;
pub use response::AuthResponse;
