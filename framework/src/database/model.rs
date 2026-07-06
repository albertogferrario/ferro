//! Model traits for Ferro ORM
//!
//! Provides Laravel-like active record pattern over SeaORM entities.
//!
//! # Scoped Queries
//!
//! Use the `ScopedQuery` trait to define reusable query scopes:
//!
//! ```rust,ignore
//! impl ScopedQuery for animals::Entity {
//!     type Scope = AnimalScope;
//! }
//!
//! enum AnimalScope {
//!     ForShelter(i64),
//!     Available,
//!     Species(String),
//! }
//!
//! // Usage:
//! let animals = Animal::scoped(AnimalScope::ForShelter(shelter_id))
//!     .and(AnimalScope::Available)
//!     .all()
//!     .await?;
//! ```

use async_trait::async_trait;
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait,
    PaginatorTrait, PrimaryKeyTrait, TryIntoModel,
};

use crate::database::{QueryBuilder, DB};
use crate::error::FrameworkError;

/// Trait providing Laravel-like read operations on SeaORM entities
///
/// Implement this trait on your SeaORM Entity to get convenient static methods
/// for querying records.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::database::Model;
/// use sea_orm::entity::prelude::*;
///
/// #[derive(Clone, Debug, DeriveEntityModel)]
/// #[sea_orm(table_name = "users")]
/// pub struct Model {
///     #[sea_orm(primary_key)]
///     pub id: i32,
///     pub name: String,
///     pub email: String,
/// }
///
/// #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
/// pub enum Relation {}
///
/// impl ActiveModelBehavior for ActiveModel {}
///
/// // Add Ferro's Model trait
/// impl ferro_rs::database::Model for Entity {}
///
/// // Now you can use:
/// let users = Entity::all().await?;
/// let user = Entity::find_by_pk(1).await?;
/// ```
#[async_trait]
pub trait Model: EntityTrait + Sized
where
    Self::Model: ModelTrait<Entity = Self> + Send + Sync,
{
    /// Find all records
    ///
    /// # Example
    /// ```rust,ignore
    /// let users = user::Entity::all().await?;
    /// ```
    async fn all() -> Result<Vec<Self::Model>, FrameworkError> {
        let db = DB::connection()?;
        Self::find()
            .all(db.inner())
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))
    }

    /// Find a record by primary key (generic version)
    ///
    /// # Example
    /// ```rust,ignore
    /// let user = user::Entity::find_by_pk(1).await?;
    /// ```
    async fn find_by_pk<K>(id: K) -> Result<Option<Self::Model>, FrameworkError>
    where
        K: Into<<Self::PrimaryKey as PrimaryKeyTrait>::ValueType> + Send,
    {
        let db = DB::connection()?;
        Self::find_by_id(id)
            .one(db.inner())
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))
    }

    /// Find a record by primary key or return an error
    ///
    /// # Example
    /// ```rust,ignore
    /// let user = user::Entity::find_or_fail(1).await?;
    /// ```
    async fn find_or_fail<K>(id: K) -> Result<Self::Model, FrameworkError>
    where
        K: Into<<Self::PrimaryKey as PrimaryKeyTrait>::ValueType> + Send + std::fmt::Debug + Copy,
    {
        Self::find_by_pk(id).await?.ok_or_else(|| {
            FrameworkError::database(format!(
                "{} with id {:?} not found",
                std::any::type_name::<Self>(),
                id
            ))
        })
    }

    /// Count all records
    ///
    /// # Example
    /// ```rust,ignore
    /// let count = user::Entity::count_all().await?;
    /// ```
    async fn count_all() -> Result<u64, FrameworkError> {
        let db = DB::connection()?;
        Self::find()
            .count(db.inner())
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))
    }

    /// Check if any records exist
    ///
    /// # Example
    /// ```rust,ignore
    /// if user::Entity::exists_any().await? {
    ///     println!("Users exist!");
    /// }
    /// ```
    async fn exists_any() -> Result<bool, FrameworkError> {
        Ok(Self::count_all().await? > 0)
    }

    /// Get the first record
    ///
    /// # Example
    /// ```rust,ignore
    /// let first_user = user::Entity::first().await?;
    /// ```
    async fn first() -> Result<Option<Self::Model>, FrameworkError> {
        let db = DB::connection()?;
        Self::find()
            .one(db.inner())
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))
    }
}

/// Trait providing Laravel-like write operations on SeaORM entities
///
/// Implement this trait alongside `Model` to get insert/update/delete methods.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::database::{Model, ModelMut};
/// use sea_orm::Set;
///
/// // Implement both traits
/// impl ferro_rs::database::Model for Entity {}
/// impl ferro_rs::database::ModelMut for Entity {}
///
/// // Insert a new record
/// let new_user = user::ActiveModel {
///     name: Set("John".to_string()),
///     email: Set("john@example.com".to_string()),
///     ..Default::default()
/// };
/// let user = user::Entity::insert_one(new_user).await?;
///
/// // Delete by ID
/// user::Entity::delete_by_pk(user.id).await?;
/// ```
#[async_trait]
pub trait ModelMut: Model
where
    Self::Model: ModelTrait<Entity = Self> + IntoActiveModel<Self::ActiveModel> + Send + Sync,
    Self::ActiveModel: ActiveModelTrait<Entity = Self> + ActiveModelBehavior + Send,
{
    /// Insert a new record
    ///
    /// # Example
    /// ```rust,ignore
    /// let new_user = user::ActiveModel {
    ///     name: Set("John".to_string()),
    ///     email: Set("john@example.com".to_string()),
    ///     ..Default::default()
    /// };
    /// let user = user::Entity::insert_one(new_user).await?;
    /// ```
    async fn insert_one(model: Self::ActiveModel) -> Result<Self::Model, FrameworkError> {
        let db = DB::connection()?;
        model
            .insert(db.inner())
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))
    }

    /// Update an existing record
    ///
    /// # Example
    /// ```rust,ignore
    /// let mut user: user::ActiveModel = user.into();
    /// user.name = Set("Updated Name".to_string());
    /// let updated = user::Entity::update_one(user).await?;
    /// ```
    async fn update_one(model: Self::ActiveModel) -> Result<Self::Model, FrameworkError> {
        let db = DB::connection()?;
        model
            .update(db.inner())
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))
    }

    /// Delete a record by primary key
    ///
    /// # Example
    /// ```rust,ignore
    /// let rows_deleted = user::Entity::delete_by_pk(1).await?;
    /// ```
    async fn delete_by_pk<K>(id: K) -> Result<u64, FrameworkError>
    where
        K: Into<<Self::PrimaryKey as PrimaryKeyTrait>::ValueType> + Send,
    {
        let db = DB::connection()?;
        let result = Self::delete_by_id(id)
            .exec(db.inner())
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))?;
        Ok(result.rows_affected)
    }

    /// Save a model (insert or update based on whether primary key is set)
    ///
    /// # Example
    /// ```rust,ignore
    /// let user = user::ActiveModel {
    ///     name: Set("John".to_string()),
    ///     ..Default::default()
    /// };
    /// let saved = user::Entity::save_one(user).await?;
    /// ```
    async fn save_one(model: Self::ActiveModel) -> Result<Self::Model, FrameworkError>
    where
        Self::ActiveModel: TryIntoModel<Self::Model>,
    {
        let db = DB::connection()?;
        let saved = model
            .save(db.inner())
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))?;
        saved
            .try_into_model()
            .map_err(|e| FrameworkError::database(e.to_string()))
    }
}

// ============================================================================
// SCOPED QUERIES
// ============================================================================

/// Trait for defining reusable query scopes on entities
///
/// Implement this trait to define common filters that can be applied
/// to queries in a chainable, reusable way.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::database::{ScopedQuery, Scope};
///
/// // Define scopes for your entity
/// pub enum AnimalScope {
///     ForShelter(i64),
///     Available,
///     Species(String),
///     Size(String),
/// }
///
/// impl Scope<animals::Entity> for AnimalScope {
///     fn apply(self, query: QueryBuilder<animals::Entity>) -> QueryBuilder<animals::Entity> {
///         use animals::Column;
///         match self {
///             Self::ForShelter(id) => query.filter(Column::ShelterId.eq(id)),
///             Self::Available => query.filter(Column::Status.eq("available")),
///             Self::Species(s) => query.filter(Column::Species.eq(s)),
///             Self::Size(s) => query.filter(Column::Size.eq(s)),
///         }
///     }
/// }
///
/// // Implement ScopedQuery for your entity
/// impl ScopedQuery for animals::Entity {
///     type Scope = AnimalScope;
/// }
///
/// // Now use scopes in queries:
/// let dogs = Animal::scoped(AnimalScope::Species("dog".into()))
///     .and(AnimalScope::Available)
///     .all()
///     .await?;
/// ```
pub trait ScopedQuery: EntityTrait + Sized
where
    Self::Model: Send + Sync,
{
    /// The scope type for this entity
    type Scope: Scope<Self>;

    /// Start a query with the given scope applied
    fn scoped(scope: Self::Scope) -> ScopedQueryBuilder<Self> {
        let builder = QueryBuilder::new();
        ScopedQueryBuilder {
            inner: scope.apply(builder),
        }
    }

    /// Start a query for records belonging to a specific owner
    ///
    /// This is a convenience method for common "for_user" or "for_owner" patterns.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // If your entity has a user_id column
    /// let user_favorites = Favorite::for_owner(user.id, Column::UserId).all().await?;
    /// ```
    fn for_owner<C, V>(owner_id: V, column: C) -> QueryBuilder<Self>
    where
        C: ColumnTrait,
        V: Into<sea_orm::Value>,
    {
        QueryBuilder::new().filter(column.eq(owner_id))
    }
}

/// A scope that can be applied to a query
pub trait Scope<E: EntityTrait>
where
    E::Model: Send + Sync,
{
    /// Apply this scope to a query builder
    fn apply(self, query: QueryBuilder<E>) -> QueryBuilder<E>;
}

/// Query builder with scopes applied
pub struct ScopedQueryBuilder<E>
where
    E: EntityTrait,
    E::Model: Send + Sync,
{
    inner: QueryBuilder<E>,
}

impl<E> ScopedQueryBuilder<E>
where
    E: EntityTrait,
    E::Model: Send + Sync,
{
    /// Add another scope to the query
    pub fn and<S: Scope<E>>(self, scope: S) -> Self {
        Self {
            inner: scope.apply(self.inner),
        }
    }

    /// Add a filter condition
    pub fn filter<F>(self, filter: F) -> Self
    where
        F: sea_orm::sea_query::IntoCondition,
    {
        Self {
            inner: self.inner.filter(filter),
        }
    }

    /// Get the underlying query builder
    pub fn into_query(self) -> QueryBuilder<E> {
        self.inner
    }

    /// Execute query and return all results
    pub async fn all(self) -> Result<Vec<E::Model>, FrameworkError> {
        self.inner.all().await
    }

    /// Execute query and return first result
    pub async fn first(self) -> Result<Option<E::Model>, FrameworkError> {
        self.inner.first().await
    }

    /// Execute query and return first result or error
    pub async fn first_or_fail(self) -> Result<E::Model, FrameworkError> {
        self.inner.first_or_fail().await
    }

    /// Count matching records
    pub async fn count(self) -> Result<u64, FrameworkError> {
        self.inner.count().await
    }

    /// Check if any records exist
    pub async fn exists(self) -> Result<bool, FrameworkError> {
        self.inner.exists().await
    }
}

/// Macro to define scopes for an entity
///
/// # Example
///
/// ```rust,ignore
/// define_scopes!(animals::Entity {
///     ForShelter(shelter_id: i64) => Column::ShelterId.eq(shelter_id),
///     Available => Column::Status.eq("available"),
///     Species(species: String) => Column::Species.eq(species),
/// });
///
/// // Usage:
/// let dogs = Animal::scoped(AnimalScope::Species("dog".into())).all().await?;
/// ```
#[macro_export]
macro_rules! define_scopes {
    ($entity:ty { $($scope_name:ident $(($($arg:ident : $arg_ty:ty),*))? => $filter:expr),* $(,)? }) => {
        pub enum Scope {
            $($scope_name $(($($arg_ty),*))?,)*
        }

        impl $crate::database::Scope<$entity> for Scope {
            fn apply(self, query: $crate::database::QueryBuilder<$entity>) -> $crate::database::QueryBuilder<$entity> {
                match self {
                    $(Self::$scope_name $(($($arg),*))? => query.filter($filter),)*
                }
            }
        }

        impl $crate::database::ScopedQuery for $entity {
            type Scope = Scope;
        }
    };
}

pub use define_scopes;
