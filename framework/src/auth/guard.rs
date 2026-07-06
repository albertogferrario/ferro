//! Authentication guard (facade)

use std::sync::Arc;

use crate::container::App;
use crate::session::{
    auth_user_id, clear_auth_user, generate_csrf_token, regenerate_session_id, session,
    session_mut, set_auth_user, DatabaseSessionDriver, SessionStore,
};

use super::authenticatable::Authenticatable;
use super::provider::UserProvider;

/// Authentication facade
///
/// Provides Laravel-like static methods for authentication operations.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::Auth;
///
/// // Check if authenticated
/// if Auth::check() {
///     let user_id = Auth::id().unwrap();
/// }
///
/// // Log in
/// Auth::login(user_id);
///
/// // Log out
/// Auth::logout();
/// ```
pub struct Auth;

impl Auth {
    /// Get the authenticated user's ID
    ///
    /// Returns None if not authenticated.
    pub fn id() -> Option<i64> {
        auth_user_id()
    }

    /// Get the authenticated user's ID as a specific type
    ///
    /// Useful when your database uses i32 primary keys but Auth stores i64.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // SeaORM entities typically use i32 for primary keys
    /// let user_id: i32 = Auth::id_as().expect("User must be authenticated");
    /// ```
    pub fn id_as<T>() -> Option<T>
    where
        T: TryFrom<i64>,
    {
        Self::id().and_then(|id| T::try_from(id).ok())
    }

    /// Check if a user is currently authenticated
    pub fn check() -> bool {
        Self::id().is_some()
    }

    /// Check if the current user is a guest (not authenticated)
    pub fn guest() -> bool {
        !Self::check()
    }

    /// Log in a user by their ID
    ///
    /// This sets the user ID in the session, making them authenticated.
    ///
    /// # Security
    ///
    /// This method regenerates the session ID to prevent session fixation attacks.
    pub fn login(user_id: i64) {
        // Regenerate session ID to prevent session fixation
        regenerate_session_id();

        // Set the authenticated user
        set_auth_user(user_id);

        // Regenerate CSRF token for extra security
        session_mut(|session| {
            session.csrf_token = generate_csrf_token();
        });
    }

    /// Log in a user with "remember me" functionality
    ///
    /// This extends the session lifetime for persistent login.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's ID
    /// * `remember_token` - A secure token for remember me cookie
    pub fn login_remember(user_id: i64, _remember_token: &str) {
        // For now, just do a regular login
        // Remember me cookie handling is done in the controller
        Self::login(user_id);
    }

    /// Log out the current user
    ///
    /// Clears the authenticated user from the session.
    ///
    /// # Security
    ///
    /// This regenerates the CSRF token to prevent any cached tokens from being reused.
    pub fn logout() {
        // Clear the authenticated user
        clear_auth_user();

        // Regenerate CSRF token for security
        session_mut(|session| {
            session.csrf_token = generate_csrf_token();
        });
    }

    /// Log out and invalidate the entire session
    ///
    /// Use this for complete session destruction (e.g., "logout everywhere").
    pub fn logout_and_invalidate() {
        session_mut(|session| {
            session.flush();
            session.csrf_token = generate_csrf_token();
        });
    }

    /// Log out all other sessions for the current user.
    ///
    /// Destroys all sessions for the authenticated user except the current one.
    /// Use after password changes or security-sensitive operations.
    ///
    /// Returns the number of destroyed sessions, or None if not authenticated.
    pub async fn logout_other_devices() -> Option<Result<u64, crate::error::FrameworkError>> {
        let user_id = Self::id()?;
        let current_session_id = session().map(|s| s.id);
        // Lifetime values are irrelevant here — destroy_for_user only deletes by user_id.
        let store = DatabaseSessionDriver::new(
            std::time::Duration::from_secs(0),
            std::time::Duration::from_secs(0),
        );
        Some(
            store
                .destroy_for_user(user_id, current_session_id.as_deref())
                .await,
        )
    }

    /// Attempt to authenticate with a validator function
    ///
    /// The validator function should return the user ID if credentials are valid.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let user_id = Auth::attempt(async {
    ///     // Validate credentials
    ///     let user = User::find_by_email(&email).await?;
    ///     if user.verify_password(&password)? {
    ///         Ok(Some(user.id))
    ///     } else {
    ///         Ok(None)
    ///     }
    /// }).await?;
    ///
    /// if let Some(id) = user_id {
    ///     // Authentication successful
    /// }
    /// ```
    pub async fn attempt<F, Fut>(validator: F) -> Result<Option<i64>, crate::error::FrameworkError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<Option<i64>, crate::error::FrameworkError>>,
    {
        let result = validator().await?;
        if let Some(user_id) = result {
            Self::login(user_id);
        }
        Ok(result)
    }

    /// Validate credentials without logging in
    ///
    /// Useful for password confirmation dialogs.
    pub async fn validate<F, Fut>(validator: F) -> Result<bool, crate::error::FrameworkError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<bool, crate::error::FrameworkError>>,
    {
        validator().await
    }

    /// Get the currently authenticated user
    ///
    /// Returns `None` if not authenticated or if no `UserProvider` is registered.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_rs::Auth;
    ///
    /// if let Some(user) = Auth::user().await? {
    ///     println!("Logged in as user {}", user.auth_identifier());
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if no `UserProvider` is registered in the container.
    /// Make sure to register a `UserProvider` in your `bootstrap.rs`:
    ///
    /// ```rust,ignore
    /// bind!(dyn UserProvider, DatabaseUserProvider);
    /// ```
    pub async fn user() -> Result<Option<Arc<dyn Authenticatable>>, crate::error::FrameworkError> {
        let user_id = match Self::id() {
            Some(id) => id,
            None => return Ok(None),
        };

        let provider = App::make::<dyn UserProvider>().ok_or_else(|| {
            crate::error::FrameworkError::internal(
                "No UserProvider registered. Register one in bootstrap.rs with: \
                 bind!(dyn UserProvider, YourUserProvider)"
                    .to_string(),
            )
        })?;

        provider.retrieve_by_id(user_id).await
    }

    /// Get the authenticated user, cast to a concrete type
    ///
    /// This is a convenience method that retrieves the user and downcasts
    /// it to your concrete User type.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_rs::Auth;
    /// use ferro_rs::models::users::User;
    ///
    /// if let Some(user) = Auth::user_as::<User>().await? {
    ///     println!("Welcome, user #{}!", user.id);
    /// }
    /// ```
    ///
    /// # Type Parameters
    ///
    /// * `T` - The concrete user type that implements `Authenticatable` and `Clone`
    pub async fn user_as<T: Authenticatable + Clone>(
    ) -> Result<Option<T>, crate::error::FrameworkError> {
        let user = Self::user().await?;
        Ok(user.and_then(|u| u.as_any().downcast_ref::<T>().cloned()))
    }
}
