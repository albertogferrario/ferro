//! Fluent query builder for Eloquent-like API
//!
//! Provides a chainable query interface that uses the global DB connection.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::models::todos::{Todo, Column};
//!
//! // Simple query
//! let todos = Todo::query().all().await?;
//!
//! // With filters
//! let todo = Todo::query()
//!     .filter(Column::Title.eq("test"))
//!     .filter(Column::Id.gt(5))
//!     .first()
//!     .await?;
//!
//! // With ordering and pagination
//! let todos = Todo::query()
//!     .order_by_desc(Column::CreatedAt)
//!     .limit(10)
//!     .offset(20)
//!     .all()
//!     .await?;
//!
//! // With eager loading (avoids N+1)
//! let (animals, shelters) = Animal::query()
//!     .all_with(|animals| async {
//!         Shelter::batch_load(animals.iter().map(|a| a.shelter_id)).await
//!     })
//!     .await?;
//! ```

use sea_orm::{
    ColumnTrait, EntityTrait, Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Select,
};
use std::future::Future;

use crate::database::DB;
use crate::error::FrameworkError;

/// Fluent query builder wrapper
///
/// Wraps SeaORM's `Select` with methods that use the global DB connection.
/// This provides an Eloquent-like query API.
///
/// # Example
///
/// ```rust,ignore
/// let todos = Todo::query()
///     .filter(Column::Active.eq(true))
///     .order_by_asc(Column::Title)
///     .all()
///     .await?;
/// ```
pub struct QueryBuilder<E>
where
    E: EntityTrait,
{
    select: Select<E>,
}

impl<E> QueryBuilder<E>
where
    E: EntityTrait,
    E::Model: Send + Sync,
{
    /// Create a new query builder for the entity
    pub fn new() -> Self {
        Self { select: E::find() }
    }

    /// Add a filter condition
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let todos = Todo::query()
    ///     .filter(Column::Title.eq("test"))
    ///     .filter(Column::Active.eq(true))
    ///     .all()
    ///     .await?;
    /// ```
    pub fn filter<F>(mut self, filter: F) -> Self
    where
        F: sea_orm::sea_query::IntoCondition,
    {
        self.select = self.select.filter(filter);
        self
    }

    /// Add an order by clause (ascending)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let todos = Todo::query()
    ///     .order_by_asc(Column::Title)
    ///     .all()
    ///     .await?;
    /// ```
    pub fn order_by_asc<C>(mut self, col: C) -> Self
    where
        C: ColumnTrait,
    {
        self.select = self.select.order_by(col, Order::Asc);
        self
    }

    /// Add an order by clause (descending)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let todos = Todo::query()
    ///     .order_by_desc(Column::CreatedAt)
    ///     .all()
    ///     .await?;
    /// ```
    pub fn order_by_desc<C>(mut self, col: C) -> Self
    where
        C: ColumnTrait,
    {
        self.select = self.select.order_by(col, Order::Desc);
        self
    }

    /// Add an order by clause with custom order
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use sea_orm::Order;
    /// let todos = Todo::query()
    ///     .order_by(Column::Title, Order::Asc)
    ///     .all()
    ///     .await?;
    /// ```
    pub fn order_by<C>(mut self, col: C, order: Order) -> Self
    where
        C: ColumnTrait,
    {
        self.select = self.select.order_by(col, order);
        self
    }

    /// Limit the number of results
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let todos = Todo::query().limit(10).all().await?;
    /// ```
    pub fn limit(mut self, limit: u64) -> Self {
        self.select = self.select.limit(limit);
        self
    }

    /// Skip a number of results (offset)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Skip first 10, get next 10
    /// let todos = Todo::query().offset(10).limit(10).all().await?;
    /// ```
    pub fn offset(mut self, offset: u64) -> Self {
        self.select = self.select.offset(offset);
        self
    }

    /// Execute query and return all results
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let todos = Todo::query().all().await?;
    /// ```
    pub async fn all(self) -> Result<Vec<E::Model>, FrameworkError> {
        let db = DB::connection()?;
        self.select
            .all(db.inner())
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))
    }

    /// Execute query and return first result
    ///
    /// Returns `None` if no record matches.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let todo = Todo::query()
    ///     .filter(Column::Id.eq(1))
    ///     .first()
    ///     .await?;
    /// ```
    pub async fn first(self) -> Result<Option<E::Model>, FrameworkError> {
        let db = DB::connection()?;
        self.select
            .one(db.inner())
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))
    }

    /// Execute query and return first result or error
    ///
    /// Returns an error if no record matches.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let todo = Todo::query()
    ///     .filter(Column::Id.eq(1))
    ///     .first_or_fail()
    ///     .await?;
    /// ```
    pub async fn first_or_fail(self) -> Result<E::Model, FrameworkError> {
        self.first().await?.ok_or_else(|| {
            FrameworkError::database(format!("{} not found", std::any::type_name::<E::Model>()))
        })
    }

    /// Count matching records
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let count = Todo::query()
    ///     .filter(Column::Active.eq(true))
    ///     .count()
    ///     .await?;
    /// ```
    pub async fn count(self) -> Result<u64, FrameworkError> {
        let db = DB::connection()?;
        self.select
            .count(db.inner())
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))
    }

    /// Check if any records exist matching the query
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let has_active = Todo::query()
    ///     .filter(Column::Active.eq(true))
    ///     .exists()
    ///     .await?;
    /// ```
    pub async fn exists(self) -> Result<bool, FrameworkError> {
        Ok(self.count().await? > 0)
    }

    /// Get access to the underlying SeaORM Select for advanced queries
    ///
    /// Use this when you need SeaORM features not exposed by QueryBuilder.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let select = Todo::query()
    ///     .filter(Column::Active.eq(true))
    ///     .into_select();
    ///
    /// // Use with SeaORM directly
    /// let todos = select.all(db.inner()).await?;
    /// ```
    pub fn into_select(self) -> Select<E> {
        self.select
    }

    /// Execute query and load related entities in a single operation
    ///
    /// This method helps avoid N+1 queries by allowing you to batch load
    /// related entities after fetching the main results.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Load animals with their shelters (2 queries instead of N+1)
    /// let (animals, shelters) = Animal::query()
    ///     .filter(Column::Status.eq("available"))
    ///     .all_with(|animals| async {
    ///         let ids: Vec<_> = animals.iter().map(|a| a.shelter_id).collect();
    ///         Shelter::batch_load(ids).await
    ///     })
    ///     .await?;
    ///
    /// // Access related data
    /// for animal in &animals {
    ///     if let Some(shelter) = shelters.get(&animal.shelter_id) {
    ///         println!("{} is at {}", animal.name, shelter.name);
    ///     }
    /// }
    /// ```
    pub async fn all_with<R, F, Fut>(self, loader: F) -> Result<(Vec<E::Model>, R), FrameworkError>
    where
        F: FnOnce(&[E::Model]) -> Fut,
        Fut: Future<Output = Result<R, FrameworkError>>,
    {
        let models = self.all().await?;
        let related = loader(&models).await?;
        Ok((models, related))
    }

    /// Execute query and load multiple related entity types
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Load animals with shelters and photos
    /// let (animals, (shelters, photos)) = Animal::query()
    ///     .all_with2(
    ///         |animals| Shelter::batch_load(animals.iter().map(|a| a.shelter_id)),
    ///         |animals| AnimalPhoto::load_for_animals(animals),
    ///     )
    ///     .await?;
    /// ```
    pub async fn all_with2<R1, R2, F1, F2, Fut1, Fut2>(
        self,
        loader1: F1,
        loader2: F2,
    ) -> Result<(Vec<E::Model>, (R1, R2)), FrameworkError>
    where
        F1: FnOnce(&[E::Model]) -> Fut1,
        F2: FnOnce(&[E::Model]) -> Fut2,
        Fut1: Future<Output = Result<R1, FrameworkError>>,
        Fut2: Future<Output = Result<R2, FrameworkError>>,
    {
        let models = self.all().await?;
        let (r1, r2) = tokio::try_join!(loader1(&models), loader2(&models))?;
        Ok((models, (r1, r2)))
    }

    /// Execute query and load three related entity types
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let (animals, (shelters, photos, favorites)) = Animal::query()
    ///     .all_with3(
    ///         |a| Shelter::batch_load(a.iter().map(|x| x.shelter_id)),
    ///         |a| AnimalPhoto::load_for_animals(a),
    ///         |a| Favorite::load_for_animals(a),
    ///     )
    ///     .await?;
    /// ```
    pub async fn all_with3<R1, R2, R3, F1, F2, F3, Fut1, Fut2, Fut3>(
        self,
        loader1: F1,
        loader2: F2,
        loader3: F3,
    ) -> Result<(Vec<E::Model>, (R1, R2, R3)), FrameworkError>
    where
        F1: FnOnce(&[E::Model]) -> Fut1,
        F2: FnOnce(&[E::Model]) -> Fut2,
        F3: FnOnce(&[E::Model]) -> Fut3,
        Fut1: Future<Output = Result<R1, FrameworkError>>,
        Fut2: Future<Output = Result<R2, FrameworkError>>,
        Fut3: Future<Output = Result<R3, FrameworkError>>,
    {
        let models = self.all().await?;
        let (r1, r2, r3) = tokio::try_join!(loader1(&models), loader2(&models), loader3(&models))?;
        Ok((models, (r1, r2, r3)))
    }
}

impl<E> Default for QueryBuilder<E>
where
    E: EntityTrait,
    E::Model: Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}
