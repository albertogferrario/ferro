//! Transaction helpers for database operations
//!
//! Provides convenient ways to wrap database operations in transactions.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::database::transaction;
//!
//! // Simple transaction
//! let result = transaction(|txn| async move {
//!     User::insert_one_with(user_data, &txn).await?;
//!     Profile::insert_one_with(profile_data, &txn).await?;
//!     Ok(())
//! }).await?;
//!
//! // With return value
//! let user = transaction(|txn| async move {
//!     let user = User::insert_one_with(user_data, &txn).await?;
//!     Ok(user)
//! }).await?;
//! ```

use async_trait::async_trait;
use sea_orm::{
    AccessMode, DatabaseConnection, DatabaseTransaction, IsolationLevel, TransactionTrait,
};
use std::future::Future;

use crate::database::DB;
use crate::error::FrameworkError;

/// Execute a closure within a database transaction
///
/// The transaction is automatically committed if the closure returns `Ok`,
/// and rolled back if it returns `Err` or panics.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::database::transaction;
///
/// // Create multiple records atomically
/// transaction(|txn| async move {
///     let user = user::ActiveModel {
///         email: Set("test@example.com".to_string()),
///         ..Default::default()
///     };
///     user.insert(txn).await?;
///
///     let profile = profile::ActiveModel {
///         user_id: Set(user.id),
///         ..Default::default()
///     };
///     profile.insert(txn).await?;
///
///     Ok(())
/// }).await?;
/// ```
pub async fn transaction<F, T, Fut>(f: F) -> Result<T, FrameworkError>
where
    F: FnOnce(&DatabaseTransaction) -> Fut,
    Fut: Future<Output = Result<T, FrameworkError>>,
{
    let db = DB::connection()?;
    let txn = db
        .inner()
        .begin()
        .await
        .map_err(|e| FrameworkError::database(format!("Failed to begin transaction: {e}")))?;

    match f(&txn).await {
        Ok(result) => {
            txn.commit()
                .await
                .map_err(|e| FrameworkError::database(format!("Failed to commit: {e}")))?;
            Ok(result)
        }
        Err(e) => {
            // Rollback is automatic when txn is dropped without commit
            Err(e)
        }
    }
}

/// Execute a closure within a transaction with custom isolation level
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::database::transaction_with;
/// use sea_orm::IsolationLevel;
///
/// transaction_with(IsolationLevel::Serializable, |txn| async move {
///     // Operations that require serializable isolation
///     Ok(())
/// }).await?;
/// ```
pub async fn transaction_with<F, T, Fut>(
    isolation_level: IsolationLevel,
    f: F,
) -> Result<T, FrameworkError>
where
    F: FnOnce(&DatabaseTransaction) -> Fut,
    Fut: Future<Output = Result<T, FrameworkError>>,
{
    let db = DB::connection()?;
    let txn = db
        .inner()
        .begin_with_config(Some(isolation_level), Some(AccessMode::ReadWrite))
        .await
        .map_err(|e| FrameworkError::database(format!("Failed to begin transaction: {e}")))?;

    match f(&txn).await {
        Ok(result) => {
            txn.commit()
                .await
                .map_err(|e| FrameworkError::database(format!("Failed to commit: {e}")))?;
            Ok(result)
        }
        Err(e) => Err(e),
    }
}

/// Extension trait for running transactions on a database connection
#[async_trait]
pub trait TransactionExt {
    /// Execute a closure within a transaction
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_rs::database::{DB, TransactionExt};
    ///
    /// DB::connection()?.transaction(|txn| async move {
    ///     // Your transactional operations
    ///     Ok(())
    /// }).await?;
    /// ```
    async fn transaction<F, T, Fut>(&self, f: F) -> Result<T, FrameworkError>
    where
        F: FnOnce(&DatabaseTransaction) -> Fut + Send,
        Fut: Future<Output = Result<T, FrameworkError>> + Send,
        T: Send;

    /// Execute a closure within a transaction with custom isolation level
    async fn transaction_with<F, T, Fut>(
        &self,
        isolation_level: IsolationLevel,
        f: F,
    ) -> Result<T, FrameworkError>
    where
        F: FnOnce(&DatabaseTransaction) -> Fut + Send,
        Fut: Future<Output = Result<T, FrameworkError>> + Send,
        T: Send;
}

#[async_trait]
impl TransactionExt for DatabaseConnection {
    async fn transaction<F, T, Fut>(&self, f: F) -> Result<T, FrameworkError>
    where
        F: FnOnce(&DatabaseTransaction) -> Fut + Send,
        Fut: Future<Output = Result<T, FrameworkError>> + Send,
        T: Send,
    {
        let txn = self
            .begin()
            .await
            .map_err(|e| FrameworkError::database(format!("Failed to begin transaction: {e}")))?;

        match f(&txn).await {
            Ok(result) => {
                txn.commit()
                    .await
                    .map_err(|e| FrameworkError::database(format!("Failed to commit: {e}")))?;
                Ok(result)
            }
            Err(e) => Err(e),
        }
    }

    async fn transaction_with<F, T, Fut>(
        &self,
        isolation_level: IsolationLevel,
        f: F,
    ) -> Result<T, FrameworkError>
    where
        F: FnOnce(&DatabaseTransaction) -> Fut + Send,
        Fut: Future<Output = Result<T, FrameworkError>> + Send,
        T: Send,
    {
        let txn = self
            .begin_with_config(Some(isolation_level), Some(AccessMode::ReadWrite))
            .await
            .map_err(|e| FrameworkError::database(format!("Failed to begin transaction: {e}")))?;

        match f(&txn).await {
            Ok(result) => {
                txn.commit()
                    .await
                    .map_err(|e| FrameworkError::database(format!("Failed to commit: {e}")))?;
                Ok(result)
            }
            Err(e) => Err(e),
        }
    }
}

/// Macro for cleaner transaction syntax
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::txn;
///
/// txn! {
///     User::insert_one(user_data).await?;
///     Profile::insert_one(profile_data).await?;
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! txn {
    ($($body:tt)*) => {
        $crate::database::transaction(|_txn| async move {
            $($body)*
        }).await
    };
}

pub use txn;
