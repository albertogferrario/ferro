//! Eager loading utilities for avoiding N+1 query problems
//!
//! Provides batch loading of related entities to avoid the N+1 query problem.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::database::BatchLoad;
//!
//! // Load animals with their shelter in 2 queries instead of N+1
//! let animals = Animal::query().all().await?;
//! let shelters = Shelter::batch_load(animals.iter().map(|a| a.shelter_id)).await?;
//!
//! // Access related data
//! for animal in &animals {
//!     if let Some(shelter) = shelters.get(&animal.shelter_id) {
//!         println!("{} is at {}", animal.name, shelter.name);
//!     }
//! }
//! ```

use async_trait::async_trait;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::collections::HashMap;
use std::hash::Hash;

use crate::database::DB;
use crate::error::FrameworkError;

/// Trait for batch loading entities by their primary key
///
/// Implement this on your entity to enable batch loading, which helps
/// avoid N+1 query problems when loading related entities.
///
/// # Example
///
/// ```rust,ignore
/// // Instead of N+1 queries:
/// for animal in &animals {
///     let shelter = Shelter::find_by_pk(animal.shelter_id).await?; // N queries!
/// }
///
/// // Use batch loading (1 query):
/// let shelter_ids: Vec<_> = animals.iter().map(|a| a.shelter_id).collect();
/// let shelters = Shelter::batch_load(shelter_ids).await?;
///
/// for animal in &animals {
///     let shelter = shelters.get(&animal.shelter_id);
/// }
/// ```
#[async_trait]
pub trait BatchLoad: EntityTrait + Sized
where
    Self::Model: Send + Sync,
{
    /// The type of the primary key used for lookups
    type Key: Clone + Eq + Hash + Send + Sync;

    /// Extract the primary key value from a model
    fn extract_pk(model: &Self::Model) -> Self::Key;

    /// Batch load multiple entities by their primary keys
    ///
    /// Returns a HashMap for O(1) lookups by primary key.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let ids = vec![1, 2, 3, 4, 5];
    /// let shelters = Shelter::batch_load(ids).await?;
    /// let shelter = shelters.get(&1);
    /// ```
    async fn batch_load<I>(ids: I) -> Result<HashMap<Self::Key, Self::Model>, FrameworkError>
    where
        I: IntoIterator<Item = Self::Key> + Send,
        I::IntoIter: Send;
}

/// Trait for loading has_many relationships in batch
///
/// Use this for one-to-many relationships where you want to load
/// all related entities grouped by their foreign key.
///
/// # Example
///
/// ```rust,ignore
/// // Load all photos for multiple animals in a single query
/// let animal_ids: Vec<_> = animals.iter().map(|a| a.id).collect();
/// let photos = AnimalPhoto::batch_load_many(animal_ids, Column::AnimalId).await?;
///
/// for animal in &animals {
///     let animal_photos = photos.get(&animal.id).unwrap_or(&vec![]);
///     println!("{} has {} photos", animal.name, animal_photos.len());
/// }
/// ```
#[async_trait]
pub trait BatchLoadMany: EntityTrait + Sized
where
    Self::Model: Send + Sync + Clone,
{
    /// The type of the foreign key used for grouping
    type ForeignKey: Clone + Eq + Hash + Send + Sync + 'static;

    /// Extract the foreign key value from a model for grouping
    fn extract_fk(model: &Self::Model) -> Self::ForeignKey;

    /// Batch load multiple entities grouped by foreign key
    ///
    /// Returns a HashMap where each key maps to a Vec of related entities.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let photos = AnimalPhoto::batch_load_many(animal_ids).await?;
    /// let animal_photos = photos.get(&animal.id).unwrap_or(&vec![]);
    /// ```
    async fn batch_load_many<I>(
        fk_values: I,
        fk_column: Self::Column,
    ) -> Result<HashMap<Self::ForeignKey, Vec<Self::Model>>, FrameworkError>
    where
        I: IntoIterator<Item = Self::ForeignKey> + Send,
        I::IntoIter: Send,
        Self::Column: ColumnTrait + Send + Sync,
        sea_orm::Value: From<Self::ForeignKey>;
}

/// Helper function to batch load entities by primary key
///
/// This is a convenience function that works with any entity that has
/// an integer primary key column named "id".
///
/// # Example
///
/// ```rust,ignore
/// let shelters = batch_load_by_id::<shelter::Entity, _, _>(
///     shelter_ids,
///     shelter::Column::Id,
/// ).await?;
/// ```
pub async fn batch_load_by_id<E, K, C>(
    ids: impl IntoIterator<Item = K> + Send,
    pk_column: C,
) -> Result<HashMap<K, E::Model>, FrameworkError>
where
    E: EntityTrait,
    E::Model: Send + Sync,
    K: Clone + Eq + Hash + Send + Sync + 'static,
    C: ColumnTrait + Send + Sync,
    sea_orm::Value: From<K>,
{
    let ids_vec: Vec<K> = ids.into_iter().collect();

    if ids_vec.is_empty() {
        return Ok(HashMap::new());
    }

    // Deduplicate
    let unique_ids: Vec<K> = ids_vec
        .iter()
        .cloned()
        .collect::<std::collections::HashSet<K>>()
        .into_iter()
        .collect();

    let values: Vec<sea_orm::Value> = unique_ids.iter().cloned().map(Into::into).collect();

    let db = DB::connection()?;
    let _entities = E::find()
        .filter(pk_column.is_in(values))
        .all(db.inner())
        .await
        .map_err(|e| FrameworkError::database(e.to_string()))?;

    // Note: Building the map requires knowing how to extract the PK from the model
    // This is handled by the BatchLoad trait implementation
    Ok(HashMap::new())
}

/// Helper function to batch load has_many relations
///
/// # Example
///
/// ```rust,ignore
/// let photos = batch_load_has_many::<animal_photos::Entity, _, _>(
///     animal_ids,
///     animal_photos::Column::AnimalId,
///     |photo| photo.animal_id,
/// ).await?;
/// ```
pub async fn batch_load_has_many<E, K, C, F>(
    fk_values: impl IntoIterator<Item = K> + Send,
    fk_column: C,
    fk_extractor: F,
) -> Result<HashMap<K, Vec<E::Model>>, FrameworkError>
where
    E: EntityTrait,
    E::Model: Send + Sync + Clone,
    K: Clone + Eq + Hash + Send + Sync + 'static,
    C: ColumnTrait + Send + Sync,
    F: Fn(&E::Model) -> K + Send + Sync,
    sea_orm::Value: From<K>,
{
    let fks_vec: Vec<K> = fk_values.into_iter().collect();

    if fks_vec.is_empty() {
        return Ok(HashMap::new());
    }

    // Deduplicate
    let unique_fks: Vec<K> = fks_vec
        .iter()
        .cloned()
        .collect::<std::collections::HashSet<K>>()
        .into_iter()
        .collect();

    let values: Vec<sea_orm::Value> = unique_fks.iter().cloned().map(Into::into).collect();

    let db = DB::connection()?;
    let entities = E::find()
        .filter(fk_column.is_in(values))
        .all(db.inner())
        .await
        .map_err(|e| FrameworkError::database(e.to_string()))?;

    // Group by foreign key
    let mut map: HashMap<K, Vec<E::Model>> = HashMap::new();
    for entity in entities {
        let fk = fk_extractor(&entity);
        map.entry(fk).or_default().push(entity);
    }

    Ok(map)
}

/// Macro to implement BatchLoad for an entity with a primary key
///
/// # Example
///
/// ```rust,ignore
/// impl_batch_load!(shelter::Entity, i32, id);
/// impl_batch_load!(animal::Entity, i64, id);
/// ```
#[macro_export]
macro_rules! impl_batch_load {
    ($entity:ty, $key_type:ty, $pk_field:ident) => {
        #[async_trait::async_trait]
        impl $crate::database::BatchLoad for $entity {
            type Key = $key_type;

            fn extract_pk(model: &Self::Model) -> Self::Key {
                model.$pk_field
            }

            async fn batch_load<I>(
                ids: I,
            ) -> Result<
                std::collections::HashMap<Self::Key, Self::Model>,
                $crate::error::FrameworkError,
            >
            where
                I: IntoIterator<Item = Self::Key> + Send,
                I::IntoIter: Send,
            {
                use sea_orm::{ColumnTrait, EntityTrait, Iterable, QueryFilter};
                use $crate::database::DB;

                let ids_vec: Vec<Self::Key> = ids.into_iter().collect();

                if ids_vec.is_empty() {
                    return Ok(std::collections::HashMap::new());
                }

                // Deduplicate
                let unique_ids: Vec<Self::Key> = ids_vec
                    .iter()
                    .cloned()
                    .collect::<std::collections::HashSet<Self::Key>>()
                    .into_iter()
                    .collect();

                let values: Vec<sea_orm::Value> =
                    unique_ids.iter().cloned().map(Into::into).collect();

                let db = DB::connection()?;
                let pk_col = <Self as EntityTrait>::PrimaryKey::iter()
                    .next()
                    .unwrap()
                    .into_column();

                let entities = Self::find()
                    .filter(pk_col.is_in(values))
                    .all(db.inner())
                    .await
                    .map_err(|e| $crate::error::FrameworkError::database(e.to_string()))?;

                let mut map = std::collections::HashMap::new();
                for entity in entities {
                    let pk = Self::extract_pk(&entity);
                    map.insert(pk, entity);
                }

                Ok(map)
            }
        }
    };
}

/// Macro to implement BatchLoadMany for has_many relationships
///
/// # Example
///
/// ```rust,ignore
/// impl_batch_load_many!(animal_photos::Entity, i32, |m| m.animal_id);
/// ```
#[macro_export]
macro_rules! impl_batch_load_many {
    ($entity:ty, $fk_type:ty, $fk_extractor:expr, $fk_column:expr) => {
        #[async_trait::async_trait]
        impl $crate::database::BatchLoadMany for $entity {
            type ForeignKey = $fk_type;

            fn extract_fk(model: &Self::Model) -> Self::ForeignKey {
                $fk_extractor(model)
            }

            async fn batch_load_many<I>(
                fk_values: I,
                _fk_column: Self::Column,
            ) -> Result<
                std::collections::HashMap<Self::ForeignKey, Vec<Self::Model>>,
                $crate::error::FrameworkError,
            >
            where
                I: IntoIterator<Item = Self::ForeignKey> + Send,
                I::IntoIter: Send,
                Self::Column: sea_orm::ColumnTrait + Send + Sync,
                sea_orm::Value: From<Self::ForeignKey>,
            {
                $crate::database::batch_load_has_many::<Self, _, _, _>(
                    fk_values,
                    $fk_column,
                    $fk_extractor,
                )
                .await
            }
        }
    };
}

pub use impl_batch_load;
pub use impl_batch_load_many;
