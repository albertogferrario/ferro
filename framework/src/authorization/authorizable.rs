//! Authorizable trait for user models.
//!
//! Provides convenience methods for authorization checks on user instances.

use super::error::AuthorizationError;
use super::gate::Gate;
use super::response::AuthResponse;
use crate::auth::Authenticatable;
use std::any::Any;

/// Trait for entities that can be authorized.
///
/// Implement this trait on your User model to enable `user.can()` and `user.authorize()`.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::authorization::Authorizable;
/// use ferro_rs::auth::Authenticatable;
///
/// impl Authorizable for User {}
///
/// // Usage
/// let user = Auth::user_as::<User>().await?;
/// if user.can("update", Some(&post)) {
///     // User can update the post
/// }
///
/// // Or authorize with Result
/// user.authorize("delete", Some(&post))?;
/// ```
pub trait Authorizable: Authenticatable {
    /// Check if the user can perform an ability.
    ///
    /// # Arguments
    ///
    /// * `ability` - The ability to check (e.g., "view", "update", "delete")
    /// * `resource` - Optional resource to check against
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if user.can("update", Some(&post)) {
    ///     // Show edit button
    /// }
    /// ```
    fn can(&self, ability: &str, resource: Option<&dyn Any>) -> bool
    where
        Self: Sized,
    {
        Gate::inspect(self, ability, resource).allowed()
    }

    /// Check if the user cannot perform an ability.
    ///
    /// Opposite of `can()`.
    fn cannot(&self, ability: &str, resource: Option<&dyn Any>) -> bool
    where
        Self: Sized,
    {
        !self.can(ability, resource)
    }

    /// Authorize an ability or return an error.
    ///
    /// # Arguments
    ///
    /// * `ability` - The ability to authorize
    /// * `resource` - Optional resource to check against
    ///
    /// # Returns
    ///
    /// `Ok(())` if authorized, `Err(AuthorizationError)` if not.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// pub async fn update(req: Request, Path(id): Path<i64>) -> Result<Response, AuthorizationError> {
    ///     let user = Auth::user_as::<User>().await?.unwrap();
    ///     let post = Post::find(id).await?;
    ///
    ///     user.authorize("update", Some(&post))?;
    ///
    ///     // Proceed with update...
    /// }
    /// ```
    fn authorize(&self, ability: &str, resource: Option<&dyn Any>) -> Result<(), AuthorizationError>
    where
        Self: Sized,
    {
        let response = Gate::inspect(self, ability, resource);
        if response.allowed() {
            Ok(())
        } else {
            let mut error = AuthorizationError::new(ability);
            if let Some(msg) = response.message() {
                error.message = Some(msg.to_string());
            }
            error.status = response.status();
            Err(error)
        }
    }

    /// Check ability and get the full response.
    ///
    /// Returns the `AuthResponse` with message and status details.
    fn check_ability(&self, ability: &str, resource: Option<&dyn Any>) -> AuthResponse
    where
        Self: Sized,
    {
        Gate::inspect(self, ability, resource)
    }
}

/// Blanket implementation for all Authenticatable types.
///
/// This allows any User model that implements Authenticatable
/// to automatically get authorization methods.
impl<T: Authenticatable> Authorizable for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authorization::gate::Gate;
    use std::any::Any;

    #[derive(Debug, Clone)]
    struct TestUser {
        id: i64,
        role: String,
    }

    impl Authenticatable for TestUser {
        fn auth_identifier(&self) -> i64 {
            self.id
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[derive(Debug)]
    struct TestResource {
        owner_id: i64,
    }

    #[test]
    fn test_can_method() {
        let _guard = Gate::test_lock();
        Gate::flush();

        Gate::define("edit-resource", |user, resource| {
            let user = user.as_any().downcast_ref::<TestUser>();
            let resource = resource.and_then(|r| r.downcast_ref::<TestResource>());

            match (user, resource) {
                (Some(u), Some(r)) => (u.id == r.owner_id).into(),
                _ => AuthResponse::deny_silent(),
            }
        });

        let owner = TestUser {
            id: 1,
            role: "user".to_string(),
        };
        let other = TestUser {
            id: 2,
            role: "user".to_string(),
        };
        let resource = TestResource { owner_id: 1 };

        assert!(owner.can("edit-resource", Some(&resource)));
        assert!(!other.can("edit-resource", Some(&resource)));
    }

    #[test]
    fn test_cannot_method() {
        let _guard = Gate::test_lock();
        Gate::flush();

        Gate::define("admin-action", |user, _| {
            user.as_any()
                .downcast_ref::<TestUser>()
                .map(|u| (u.role == "admin").into())
                .unwrap_or_else(AuthResponse::deny_silent)
        });

        let admin = TestUser {
            id: 1,
            role: "admin".to_string(),
        };
        let regular = TestUser {
            id: 2,
            role: "user".to_string(),
        };

        assert!(!admin.cannot("admin-action", None));
        assert!(regular.cannot("admin-action", None));
    }

    #[test]
    fn test_authorize_method() {
        let _guard = Gate::test_lock();
        Gate::flush();

        Gate::define("manage", |user, _| {
            user.as_any()
                .downcast_ref::<TestUser>()
                .map(|u| {
                    if u.role == "manager" {
                        AuthResponse::allow()
                    } else {
                        AuthResponse::deny("Manager access required")
                    }
                })
                .unwrap_or_else(AuthResponse::deny_silent)
        });

        let manager = TestUser {
            id: 1,
            role: "manager".to_string(),
        };
        let regular = TestUser {
            id: 2,
            role: "user".to_string(),
        };

        assert!(manager.authorize("manage", None).is_ok());

        let err = regular.authorize("manage", None).unwrap_err();
        assert_eq!(err.message, Some("Manager access required".to_string()));
    }

    #[test]
    fn test_check_ability() {
        let _guard = Gate::test_lock();
        Gate::flush();

        Gate::define("view", |_, _| AuthResponse::allow());

        let user = TestUser {
            id: 1,
            role: "user".to_string(),
        };

        let response = user.check_ability("view", None);
        assert!(response.allowed());
    }
}
