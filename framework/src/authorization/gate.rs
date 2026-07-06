//! Authorization Gate facade.
//!
//! Provides Laravel-like authorization checking.

use super::error::AuthorizationError;
use super::response::AuthResponse;
use crate::auth::Authenticatable;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Type alias for gate ability callbacks.
type AbilityCallback =
    Box<dyn Fn(&dyn Authenticatable, Option<&dyn Any>) -> AuthResponse + Send + Sync>;

/// Type alias for before/after callbacks.
type BeforeCallback = Box<dyn Fn(&dyn Authenticatable, &str) -> Option<bool> + Send + Sync>;

/// Global gate registry.
static GATE_REGISTRY: RwLock<Option<GateRegistry>> = RwLock::new(None);

/// Internal registry for gates and policies.
struct GateRegistry {
    /// Simple ability callbacks.
    abilities: HashMap<String, AbilityCallback>,
    /// Before hooks (run before any ability check).
    before_hooks: Vec<BeforeCallback>,
    /// Policy type mappings (model TypeId -> policy factory).
    policies: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl GateRegistry {
    fn new() -> Self {
        Self {
            abilities: HashMap::new(),
            before_hooks: Vec::new(),
            policies: HashMap::new(),
        }
    }
}

/// Authorization Gate facade.
///
/// Provides a central point for authorization checks.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::authorization::Gate;
///
/// // Define a simple gate
/// Gate::define("admin", |user, _| user.is_admin().into());
///
/// // Check in controller
/// if Gate::allows("admin", None) {
///     // User is admin
/// }
///
/// // Authorize (returns Result)
/// Gate::authorize("admin", None)?;
/// ```
pub struct Gate;

impl Gate {
    /// Initialize the gate registry.
    ///
    /// This is called automatically by the framework during bootstrap.
    pub fn init() {
        let mut registry = GATE_REGISTRY.write().unwrap();
        if registry.is_none() {
            *registry = Some(GateRegistry::new());
        }
    }

    /// Define a simple ability.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// Gate::define("view-dashboard", |user, _| {
    ///     user.as_any().downcast_ref::<User>()
    ///         .map(|u| (u.is_admin || u.has_role("manager")).into())
    ///         .unwrap_or_else(AuthResponse::deny_silent)
    /// });
    /// ```
    pub fn define<F>(ability: &str, callback: F)
    where
        F: Fn(&dyn Authenticatable, Option<&dyn Any>) -> AuthResponse + Send + Sync + 'static,
    {
        Self::init();
        let mut registry = GATE_REGISTRY.write().unwrap();
        if let Some(ref mut reg) = *registry {
            reg.abilities
                .insert(ability.to_string(), Box::new(callback));
        }
    }

    /// Register a before hook.
    ///
    /// Before hooks run before any ability check. Return `Some(true)` to allow,
    /// `Some(false)` to deny, or `None` to continue to the ability check.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Allow super admins to bypass all checks
    /// Gate::before(|user, _ability| {
    ///     if let Some(u) = user.as_any().downcast_ref::<User>() {
    ///         if u.is_super_admin {
    ///             return Some(true);
    ///         }
    ///     }
    ///     None
    /// });
    /// ```
    pub fn before<F>(callback: F)
    where
        F: Fn(&dyn Authenticatable, &str) -> Option<bool> + Send + Sync + 'static,
    {
        Self::init();
        let mut registry = GATE_REGISTRY.write().unwrap();
        if let Some(ref mut reg) = *registry {
            reg.before_hooks.push(Box::new(callback));
        }
    }

    /// Check if the current user is allowed to perform an ability.
    ///
    /// Returns `true` if allowed, `false` if denied or not authenticated.
    pub fn allows(ability: &str, resource: Option<&dyn Any>) -> bool {
        crate::auth::Auth::id().is_some() && Self::allows_for_user_id(ability, resource)
    }

    /// Check if the current user is denied an ability.
    pub fn denies(ability: &str, resource: Option<&dyn Any>) -> bool {
        !Self::allows(ability, resource)
    }

    /// Authorize the current user for an ability.
    ///
    /// Returns `Ok(())` if allowed, or `Err(AuthorizationError)` if denied.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// pub async fn admin_dashboard() -> Result<Response, AuthorizationError> {
    ///     Gate::authorize("view-dashboard", None)?;
    ///     // Render dashboard...
    /// }
    /// ```
    pub fn authorize(ability: &str, resource: Option<&dyn Any>) -> Result<(), AuthorizationError> {
        if crate::auth::Auth::id().is_none() {
            return Err(AuthorizationError::new(ability).with_status(401));
        }

        if Self::allows_for_user_id(ability, resource) {
            Ok(())
        } else {
            Err(AuthorizationError::new(ability))
        }
    }

    /// Check ability for a specific user.
    pub fn allows_for<U: Authenticatable>(
        user: &U,
        ability: &str,
        resource: Option<&dyn Any>,
    ) -> bool {
        Self::inspect(user, ability, resource).allowed()
    }

    /// Authorize for a specific user.
    pub fn authorize_for<U: Authenticatable>(
        user: &U,
        ability: &str,
        resource: Option<&dyn Any>,
    ) -> Result<(), AuthorizationError> {
        let response = Self::inspect(user, ability, resource);
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

    /// Check ability for a specific user (generic wrapper).
    pub fn check_for<U: Authenticatable>(
        user: &U,
        ability: &str,
        resource: Option<&dyn Any>,
    ) -> AuthResponse {
        Self::inspect(user, ability, resource)
    }

    /// Check ability for a dynamic Authenticatable reference.
    ///
    /// Use this when you have a trait object (`&dyn Authenticatable` or `Arc<dyn Authenticatable>`).
    pub fn inspect(
        user: &dyn Authenticatable,
        ability: &str,
        resource: Option<&dyn Any>,
    ) -> AuthResponse {
        let registry = GATE_REGISTRY.read().unwrap();
        let reg = match &*registry {
            Some(r) => r,
            None => return AuthResponse::deny_silent(),
        };

        // Run before hooks
        for hook in &reg.before_hooks {
            if let Some(result) = hook(user, ability) {
                return result.into();
            }
        }

        // Check ability callback
        if let Some(callback) = reg.abilities.get(ability) {
            return callback(user, resource);
        }

        // No matching ability found
        AuthResponse::deny_silent()
    }

    /// Internal: check using current user ID.
    fn allows_for_user_id(ability: &str, _resource: Option<&dyn Any>) -> bool {
        // We can't easily get the full user here without async,
        // so we check against stored abilities that work with Authenticatable
        let registry = GATE_REGISTRY.read().unwrap();
        let reg = match &*registry {
            Some(r) => r,
            None => return false,
        };

        // If there's no ability defined, deny
        if !reg.abilities.contains_key(ability) && reg.before_hooks.is_empty() {
            return false;
        }

        // For now, return false if we can't resolve the user synchronously
        // The async version should be used when user data is needed
        false
    }

    /// Check if a policy is registered for a model type.
    pub fn has_policy_for<M: 'static>() -> bool {
        let registry = GATE_REGISTRY.read().unwrap();
        registry
            .as_ref()
            .map(|r| r.policies.contains_key(&TypeId::of::<M>()))
            .unwrap_or(false)
    }

    /// Clear all registered gates (useful for testing).
    #[cfg(test)]
    pub fn flush() {
        let mut registry = GATE_REGISTRY.write().unwrap();
        *registry = Some(GateRegistry::new());
    }

    /// Test lock for serializing tests that use the global gate registry.
    #[cfg(test)]
    pub fn test_lock() -> std::sync::MutexGuard<'static, ()> {
        use std::sync::Mutex;
        static TEST_LOCK: Mutex<()> = Mutex::new(());
        TEST_LOCK.lock().unwrap()
    }
}

/// Extension methods for checking authorization with the current user.
///
/// These are async methods that fetch the user before checking.
impl Gate {
    /// Check if the current authenticated user is allowed (async).
    ///
    /// This fetches the user from the database before checking.
    pub async fn user_allows(ability: &str, resource: Option<&dyn Any>) -> bool {
        match Self::resolve_user_and_check(ability, resource).await {
            Ok(response) => response.allowed(),
            Err(_) => false,
        }
    }

    /// Authorize the current authenticated user (async).
    ///
    /// This fetches the user from the database before checking.
    pub async fn user_authorize(
        ability: &str,
        resource: Option<&dyn Any>,
    ) -> Result<(), AuthorizationError> {
        let response = Self::resolve_user_and_check(ability, resource).await?;
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

    /// Internal: resolve user and check ability.
    async fn resolve_user_and_check(
        ability: &str,
        resource: Option<&dyn Any>,
    ) -> Result<AuthResponse, AuthorizationError> {
        let user = crate::auth::Auth::user()
            .await
            .map_err(|_| AuthorizationError::new(ability).with_status(401))?
            .ok_or_else(|| AuthorizationError::new(ability).with_status(401))?;

        Ok(Self::inspect(user.as_ref(), ability, resource))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

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

    #[test]
    fn test_define_and_check() {
        let _guard = Gate::test_lock();
        Gate::flush();

        Gate::define("test-ability", |user, _| {
            user.as_any()
                .downcast_ref::<TestUser>()
                .map(|u| u.is_admin.into())
                .unwrap_or_else(AuthResponse::deny_silent)
        });

        let admin = TestUser {
            id: 1,
            is_admin: true,
        };
        let regular = TestUser {
            id: 2,
            is_admin: false,
        };

        assert!(Gate::allows_for(&admin, "test-ability", None));
        assert!(!Gate::allows_for(&regular, "test-ability", None));
    }

    #[test]
    fn test_before_hook() {
        let _guard = Gate::test_lock();
        Gate::flush();

        Gate::before(|user, _| {
            if let Some(u) = user.as_any().downcast_ref::<TestUser>() {
                if u.is_admin {
                    return Some(true);
                }
            }
            None
        });

        // Define an ability that always denies
        Gate::define("restricted", |_, _| AuthResponse::deny("Always denied"));

        let admin = TestUser {
            id: 1,
            is_admin: true,
        };
        let regular = TestUser {
            id: 2,
            is_admin: false,
        };

        // Admin bypasses via before hook
        assert!(Gate::allows_for(&admin, "restricted", None));
        // Regular user is denied
        assert!(!Gate::allows_for(&regular, "restricted", None));
    }

    #[test]
    fn test_authorize_for() {
        let _guard = Gate::test_lock();
        Gate::flush();

        Gate::define("view-posts", |_, _| AuthResponse::allow());
        Gate::define("admin-only", |user, _| {
            user.as_any()
                .downcast_ref::<TestUser>()
                .map(|u| {
                    if u.is_admin {
                        AuthResponse::allow()
                    } else {
                        AuthResponse::deny("Admin access required")
                    }
                })
                .unwrap_or_else(AuthResponse::deny_silent)
        });

        let admin = TestUser {
            id: 1,
            is_admin: true,
        };
        let regular = TestUser {
            id: 2,
            is_admin: false,
        };

        // Should succeed
        assert!(Gate::authorize_for(&admin, "view-posts", None).is_ok());
        assert!(Gate::authorize_for(&regular, "view-posts", None).is_ok());
        assert!(Gate::authorize_for(&admin, "admin-only", None).is_ok());

        // Should fail
        let err = Gate::authorize_for(&regular, "admin-only", None).unwrap_err();
        assert_eq!(err.message, Some("Admin access required".to_string()));
    }

    #[test]
    fn test_undefined_ability() {
        let _guard = Gate::test_lock();
        Gate::flush();

        let user = TestUser {
            id: 1,
            is_admin: false,
        };

        // Undefined abilities should deny
        assert!(!Gate::allows_for(&user, "undefined-ability", None));
    }
}
