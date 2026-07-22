//! User provider trait for retrieving authenticated users from storage
//!
//! The application must implement this trait and register it with the container
//! to enable `Auth::user()`.

use async_trait::async_trait;
use sea_orm::{EntityTrait, PrimaryKeyTrait};
use std::marker::PhantomData;
use std::sync::Arc;

use super::authenticatable::Authenticatable;
use crate::database::DB;
use crate::error::FrameworkError;

/// Trait for retrieving authenticated users from storage
///
/// The application must implement this trait and register it with the container
/// to enable `Auth::user()`.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::auth::{UserProvider, Authenticatable};
/// use ferro_rs::FrameworkError;
/// use async_trait::async_trait;
/// use std::sync::Arc;
///
/// pub struct DatabaseUserProvider;
///
/// #[async_trait]
/// impl UserProvider for DatabaseUserProvider {
///     async fn retrieve_by_id(&self, id: i64) -> Result<Option<Arc<dyn Authenticatable>>, FrameworkError> {
///         let user = User::query()
///             .filter(Column::Id.eq(id as i32))
///             .first()
///             .await?;
///         Ok(user.map(|u| Arc::new(u) as Arc<dyn Authenticatable>))
///     }
/// }
/// ```
#[async_trait]
pub trait UserProvider: Send + Sync + 'static {
    /// Retrieve a user by their unique identifier
    async fn retrieve_by_id(
        &self,
        id: i64,
    ) -> Result<Option<Arc<dyn Authenticatable>>, FrameworkError>;

    /// Retrieve a user by credentials (for custom authentication flows)
    ///
    /// Default implementation returns None (not supported).
    /// Override this if you need to authenticate by credentials other than ID.
    async fn retrieve_by_credentials(
        &self,
        _credentials: &serde_json::Value,
    ) -> Result<Option<Arc<dyn Authenticatable>>, FrameworkError> {
        Ok(None)
    }

    /// Validate credentials against a user
    ///
    /// Default implementation returns false (not supported).
    /// Override this if you need password validation.
    async fn validate_credentials(
        &self,
        _user: &dyn Authenticatable,
        _credentials: &serde_json::Value,
    ) -> Result<bool, FrameworkError> {
        Ok(false)
    }
}

/// A generic [`UserProvider`] that loads a model by primary key.
///
/// Works for any entity whose `Model` is [`Authenticatable`] and whose primary
/// key is a single integer column (`i32`/`i64`). This removes the hand-written
/// provider apps otherwise need just to hydrate `Auth::user()` from the session.
///
/// ```rust,ignore
/// use ferro_rs::{bind, ModelUserProvider, UserProvider};
///
/// // in bootstrap:
/// bind!(dyn UserProvider, ModelUserProvider::<crate::models::user::Entity>::default());
/// ```
///
/// Only [`retrieve_by_id`](UserProvider::retrieve_by_id) is implemented;
/// credential lookup/validation use the trait defaults. For composite or
/// non-integer keys, or password login, implement `UserProvider` by hand.
pub struct ModelUserProvider<E: EntityTrait> {
    // `fn() -> E` keeps the struct `Send + Sync` regardless of `E`.
    _marker: PhantomData<fn() -> E>,
}

impl<E: EntityTrait> ModelUserProvider<E> {
    /// Create a new provider for entity `E`.
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<E: EntityTrait> Default for ModelUserProvider<E> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<E> UserProvider for ModelUserProvider<E>
where
    E: EntityTrait + 'static,
    E::Model: Authenticatable + Clone,
    <E::PrimaryKey as PrimaryKeyTrait>::ValueType: TryFrom<i64> + Send,
{
    async fn retrieve_by_id(
        &self,
        id: i64,
    ) -> Result<Option<Arc<dyn Authenticatable>>, FrameworkError> {
        // Narrow the session's i64 id to the entity's primary-key value type.
        // Fails only when the id is out of range for a narrower pk (e.g. i32).
        let pk = <<E as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType::try_from(id)
            .map_err(|_| {
                FrameworkError::internal(format!(
                    "authenticated id {id} is out of range for {}'s primary key",
                    std::any::type_name::<E>()
                ))
            })?;

        let db = DB::connection()?;
        let model = E::find_by_id(pk)
            .one(db.inner())
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))?;

        Ok(model.map(|m| Arc::new(m) as Arc<dyn Authenticatable>))
    }
}
