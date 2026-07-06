//! Policy trait for model-based authorization.
//!
//! Policies organize authorization logic around a particular model or resource.

use super::response::AuthResponse;
use crate::auth::Authenticatable;

/// Trait for authorization policies.
///
/// Policies organize authorization logic around a particular model.
/// Each method corresponds to a specific ability (view, create, update, delete, etc.).
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::authorization::{Policy, AuthResponse};
/// use crate::models::{User, Post};
///
/// pub struct PostPolicy;
///
/// impl Policy<Post> for PostPolicy {
///     type User = User;
///
///     fn before(&self, user: &Self::User, _ability: &str) -> Option<bool> {
///         // Admins can do everything
///         if user.is_admin {
///             return Some(true);
///         }
///         None // Continue to specific check
///     }
///
///     fn view(&self, _user: &Self::User, _model: &Post) -> AuthResponse {
///         AuthResponse::allow() // Anyone can view posts
///     }
///
///     fn update(&self, user: &Self::User, post: &Post) -> AuthResponse {
///         if user.id == post.user_id {
///             AuthResponse::allow()
///         } else {
///             AuthResponse::deny("You do not own this post.")
///         }
///     }
///
///     fn delete(&self, user: &Self::User, post: &Post) -> AuthResponse {
///         self.update(user, post) // Same as update
///     }
/// }
/// ```
pub trait Policy<M>: Send + Sync {
    /// The user type for this policy.
    type User: Authenticatable;

    /// Run before any other authorization checks.
    ///
    /// Return `Some(true)` to allow, `Some(false)` to deny,
    /// or `None` to continue to the specific ability check.
    ///
    /// This is useful for implementing admin bypass.
    fn before(&self, _user: &Self::User, _ability: &str) -> Option<bool> {
        None
    }

    /// Determine whether the user can view any models.
    fn view_any(&self, _user: &Self::User) -> AuthResponse {
        AuthResponse::deny_silent()
    }

    /// Determine whether the user can view the model.
    fn view(&self, _user: &Self::User, _model: &M) -> AuthResponse {
        AuthResponse::deny_silent()
    }

    /// Determine whether the user can create models.
    fn create(&self, _user: &Self::User) -> AuthResponse {
        AuthResponse::deny_silent()
    }

    /// Determine whether the user can update the model.
    fn update(&self, _user: &Self::User, _model: &M) -> AuthResponse {
        AuthResponse::deny_silent()
    }

    /// Determine whether the user can delete the model.
    fn delete(&self, _user: &Self::User, _model: &M) -> AuthResponse {
        AuthResponse::deny_silent()
    }

    /// Determine whether the user can restore the model.
    fn restore(&self, _user: &Self::User, _model: &M) -> AuthResponse {
        AuthResponse::deny_silent()
    }

    /// Determine whether the user can permanently delete the model.
    fn force_delete(&self, _user: &Self::User, _model: &M) -> AuthResponse {
        AuthResponse::deny_silent()
    }

    /// Check an ability with the before hook applied.
    ///
    /// This method handles the `before` hook automatically.
    fn check(&self, user: &Self::User, ability: &str, model: Option<&M>) -> AuthResponse {
        // Check before hook first
        if let Some(result) = self.before(user, ability) {
            return result.into();
        }

        // Run the specific ability check
        match ability {
            "viewAny" | "view_any" => self.view_any(user),
            "view" => model
                .map(|m| self.view(user, m))
                .unwrap_or_else(AuthResponse::deny_silent),
            "create" => self.create(user),
            "update" => model
                .map(|m| self.update(user, m))
                .unwrap_or_else(AuthResponse::deny_silent),
            "delete" => model
                .map(|m| self.delete(user, m))
                .unwrap_or_else(AuthResponse::deny_silent),
            "restore" => model
                .map(|m| self.restore(user, m))
                .unwrap_or_else(AuthResponse::deny_silent),
            "forceDelete" | "force_delete" => model
                .map(|m| self.force_delete(user, m))
                .unwrap_or_else(AuthResponse::deny_silent),
            _ => AuthResponse::deny_silent(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

    // Test user
    #[derive(Debug, Clone)]
    struct TestUser {
        id: i64,
        is_admin: bool,
    }

    impl Authenticatable for TestUser {
        fn auth_identifier(&self) -> i64 {
            self.id
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    // Test model
    #[derive(Debug)]
    struct TestPost {
        #[allow(dead_code)]
        id: i64,
        user_id: i64,
    }

    // Test policy
    struct TestPostPolicy;

    impl Policy<TestPost> for TestPostPolicy {
        type User = TestUser;

        fn before(&self, user: &Self::User, _ability: &str) -> Option<bool> {
            if user.is_admin {
                return Some(true);
            }
            None
        }

        fn view(&self, _user: &Self::User, _model: &TestPost) -> AuthResponse {
            AuthResponse::allow()
        }

        fn update(&self, user: &Self::User, post: &TestPost) -> AuthResponse {
            (user.id == post.user_id).into()
        }

        fn delete(&self, user: &Self::User, post: &TestPost) -> AuthResponse {
            self.update(user, post)
        }
    }

    #[test]
    fn test_policy_allows_view() {
        let policy = TestPostPolicy;
        let user = TestUser {
            id: 1,
            is_admin: false,
        };
        let post = TestPost { id: 1, user_id: 2 };

        let response = policy.view(&user, &post);
        assert!(response.allowed());
    }

    #[test]
    fn test_policy_owner_can_update() {
        let policy = TestPostPolicy;
        let user = TestUser {
            id: 1,
            is_admin: false,
        };
        let post = TestPost { id: 1, user_id: 1 };

        let response = policy.update(&user, &post);
        assert!(response.allowed());
    }

    #[test]
    fn test_policy_non_owner_cannot_update() {
        let policy = TestPostPolicy;
        let user = TestUser {
            id: 1,
            is_admin: false,
        };
        let post = TestPost { id: 1, user_id: 2 };

        let response = policy.update(&user, &post);
        assert!(response.denied());
    }

    #[test]
    fn test_policy_admin_bypass() {
        let policy = TestPostPolicy;
        let admin = TestUser {
            id: 1,
            is_admin: true,
        };
        let post = TestPost {
            id: 1,
            user_id: 999,
        };

        // Admin can update any post due to before() hook
        let response = policy.check(&admin, "update", Some(&post));
        assert!(response.allowed());
    }

    #[test]
    fn test_policy_check_method() {
        let policy = TestPostPolicy;
        let user = TestUser {
            id: 1,
            is_admin: false,
        };
        let post = TestPost { id: 1, user_id: 1 };

        let response = policy.check(&user, "update", Some(&post));
        assert!(response.allowed());

        let other_post = TestPost { id: 2, user_id: 2 };
        let response = policy.check(&user, "update", Some(&other_post));
        assert!(response.denied());
    }
}
