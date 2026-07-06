//! Typed extractors for authenticated users
//!
//! Provides `AuthUser<T>` and `OptionalUser<T>` for handler parameter injection,
//! eliminating boilerplate `Auth::user_as::<T>()` calls.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::{handler, AuthUser, OptionalUser, Response};
//!
//! #[handler]
//! pub async fn dashboard(user: AuthUser<User>) -> Response {
//!     // `user` is guaranteed to be authenticated
//!     // Returns 401 automatically if not
//!     json_response!({ "name": user.name })
//! }
//!
//! #[handler]
//! pub async fn home(user: OptionalUser<User>) -> Response {
//!     match user.as_ref() {
//!         Some(u) => json_response!({ "greeting": format!("Hello, {}!", u.name) }),
//!         None => json_response!({ "greeting": "Hello, guest!" }),
//!     }
//! }
//! ```
//!
//! # Limitations
//!
//! These extractors use the `FromRequest` trait which takes ownership of the
//! request. This means they cannot be combined with `FormRequest` types or
//! `Request` in the same handler signature. If you need both auth and request
//! body, use `Auth::user_as::<T>()` manually in the handler body.

use std::ops::Deref;

use async_trait::async_trait;

use super::authenticatable::Authenticatable;
use super::guard::Auth;
use crate::error::FrameworkError;
use crate::http::FromRequest;
use crate::Request;

/// Extracts the authenticated user, returning 401 if not authenticated.
///
/// Use this in handler signatures where authentication is required.
/// The handler macro will automatically extract the user from the session.
///
/// # Type Parameters
///
/// * `T` - The concrete user type implementing `Authenticatable + Clone`
///
/// # Example
///
/// ```rust,ignore
/// #[handler]
/// pub async fn profile(user: AuthUser<User>) -> Response {
///     json_response!({ "id": user.id, "email": user.email })
/// }
/// ```
pub struct AuthUser<T>(pub T);

#[async_trait]
impl<T> FromRequest for AuthUser<T>
where
    T: Authenticatable + Clone + 'static,
{
    async fn from_request(_req: Request) -> Result<Self, FrameworkError> {
        let user = Auth::user().await?;

        match user {
            None => Err(FrameworkError::domain("Unauthenticated.", 401)),
            Some(arc_user) => {
                let concrete = arc_user
                    .as_any()
                    .downcast_ref::<T>()
                    .cloned()
                    .ok_or_else(|| {
                        FrameworkError::internal(format!(
                            "AuthUser downcast failed: user is not of type {}",
                            std::any::type_name::<T>()
                        ))
                    })?;
                Ok(AuthUser(concrete))
            }
        }
    }
}

impl<T> Deref for AuthUser<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

/// Extracts the authenticated user as `Option<T>`, never failing on auth.
///
/// Use this in handler signatures where authentication is optional.
/// Returns `None` for guests instead of a 401 error.
///
/// # Type Parameters
///
/// * `T` - The concrete user type implementing `Authenticatable + Clone`
///
/// # Example
///
/// ```rust,ignore
/// #[handler]
/// pub async fn home(user: OptionalUser<User>) -> Response {
///     if let Some(u) = user.as_ref() {
///         json_response!({ "message": format!("Welcome back, {}!", u.name) })
///     } else {
///         json_response!({ "message": "Welcome, guest!" })
///     }
/// }
/// ```
pub struct OptionalUser<T>(pub Option<T>);

#[async_trait]
impl<T> FromRequest for OptionalUser<T>
where
    T: Authenticatable + Clone + 'static,
{
    async fn from_request(_req: Request) -> Result<Self, FrameworkError> {
        let user = Auth::user().await?;

        match user {
            None => Ok(OptionalUser(None)),
            Some(arc_user) => {
                let concrete = arc_user.as_any().downcast_ref::<T>().cloned();
                Ok(OptionalUser(concrete))
            }
        }
    }
}

impl<T> Deref for OptionalUser<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Option<T> {
        &self.0
    }
}
