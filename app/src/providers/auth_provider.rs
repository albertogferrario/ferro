//! Authentication provider for user retrieval and credential validation
//!
//! This provider implements the `UserProvider` trait to fetch users from the database
//! and validate credentials using bcrypt password hashing.

use async_trait::async_trait;
use ferro::auth::{Authenticatable, UserProvider};
use ferro::serde_json;
use ferro::FrameworkError;
use sea_orm::ColumnTrait;
use std::sync::Arc;

use crate::models::users::{Column, Model as User};

/// Database-backed user provider for authentication
///
/// Implements all three `UserProvider` methods:
/// - `retrieve_by_id` — lookup by primary key (for session hydration)
/// - `retrieve_by_credentials` — lookup by email (for login)
/// - `validate_credentials` — bcrypt password verification
///
/// Register in `bootstrap.rs`:
/// ```rust,ignore
/// bind!(dyn UserProvider, DatabaseUserProvider);
/// ```
#[derive(Default)]
pub struct DatabaseUserProvider;

#[async_trait]
impl UserProvider for DatabaseUserProvider {
    async fn retrieve_by_id(
        &self,
        id: i64,
    ) -> Result<Option<Arc<dyn Authenticatable>>, FrameworkError> {
        let user = User::query()
            .filter(Column::Id.eq(id as i32))
            .first()
            .await?;

        Ok(user.map(|u| Arc::new(u) as Arc<dyn Authenticatable>))
    }

    async fn retrieve_by_credentials(
        &self,
        credentials: &serde_json::Value,
    ) -> Result<Option<Arc<dyn Authenticatable>>, FrameworkError> {
        let email = match credentials.get("email").and_then(|v| v.as_str()) {
            Some(e) => e,
            None => return Ok(None),
        };

        let user = User::find_by_email(email).await?;
        Ok(user.map(|u| Arc::new(u) as Arc<dyn Authenticatable>))
    }

    async fn validate_credentials(
        &self,
        user: &dyn Authenticatable,
        credentials: &serde_json::Value,
    ) -> Result<bool, FrameworkError> {
        let password = match credentials.get("password").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return Ok(false),
        };

        let user = match user.as_any().downcast_ref::<User>() {
            Some(u) => u,
            None => return Ok(false),
        };

        ferro::verify(password, &user.password)
    }
}
