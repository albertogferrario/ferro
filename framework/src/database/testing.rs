//! Testing utilities for database operations
//!
//! Provides `TestDatabase` for setting up isolated test environments with
//! in-memory SQLite databases and automatic migration support.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::test_database;
//!
//! #[tokio::test]
//! async fn test_create_user() {
//!     let db = test_database!();
//!
//!     // Your test code here - actions using DB::connection()
//!     // will automatically use this test database
//! }
//! ```

use sea_orm::DatabaseConnection;
use sea_orm_migration::MigratorTrait;

use super::config::DatabaseConfig;
use super::connection::DbConnection;
use crate::container::testing::{TestContainer, TestContainerGuard};
use crate::error::FrameworkError;

/// Monotonic counter giving each `TestDatabase::fresh()` a uniquely-named
/// shared-cache in-memory database (cross-test isolation).
static TEST_DB_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Test database wrapper that provides isolated database environments
///
/// Each `TestDatabase` creates a fresh in-memory SQLite database with
/// migrations applied. The database is automatically registered in the
/// test container, so any code using `DB::connection()` or `#[inject] db: Database`
/// will receive this test database.
///
/// When the `TestDatabase` is dropped, the test container is cleared,
/// ensuring complete isolation between tests.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::testing::TestDatabase;
/// use ferro_rs::migrations::Migrator;
///
/// #[tokio::test]
/// async fn test_user_creation() {
///     let db = TestDatabase::fresh::<Migrator>().await.unwrap();
///
///     // Actions using DB::connection() automatically get this test database
///     let action = CreateUserAction::new();
///     let user = action.execute("test@example.com").await.unwrap();
///
///     // Query directly using db.conn()
///     let found = users::Entity::find_by_id(user.id)
///         .one(db.conn())
///         .await
///         .unwrap();
///     assert!(found.is_some());
/// }
/// ```
pub struct TestDatabase {
    conn: DbConnection,
    _guard: TestContainerGuard,
}

impl TestDatabase {
    /// Create a fresh test database with migrations applied
    ///
    /// This creates an in-memory SQLite database, runs all migrations,
    /// and registers the connection in the test container.
    ///
    /// # Type Parameters
    ///
    /// * `M` - The migrator type implementing `MigratorTrait`. Typically
    ///   this is `crate::migrations::Migrator` from your application.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Database connection fails
    /// - Migration execution fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_rs::testing::TestDatabase;
    /// use ferro_rs::migrations::Migrator;
    ///
    /// #[tokio::test]
    /// async fn test_example() {
    ///     let db = TestDatabase::fresh::<Migrator>().await.unwrap();
    ///     // ...
    /// }
    /// ```
    pub async fn fresh<M: MigratorTrait>() -> Result<Self, FrameworkError> {
        // 1. Create test container guard for isolation
        let guard = TestContainer::fake();

        // 2. Create in-memory SQLite database.
        //
        // Shared-cache, uniquely-named :memory: DB with a multi-connection pool
        // (mirrors a production pool). A plain single-connection `sqlite::memory:`
        // pool deadlocks any code that holds an open transaction on the one
        // connection and then acquires a fresh `DB::connection()` for a nested
        // query (e.g. a webhook handler whose loader reads on a separate
        // connection) — the second acquire blocks until the sqlx timeout.
        // `cache=shared` makes every pooled connection see the same database;
        // the unique name per `fresh()` keeps cross-test isolation; the unique
        // name + min_connections(1) keeps the DB alive (a shared-cache :memory:
        // DB is dropped when its last connection closes).
        let seq = TEST_DB_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let url = format!("sqlite:file:ferro_testdb_{seq}?mode=memory&cache=shared");
        let config = DatabaseConfig::builder()
            .url(url)
            .max_connections(8)
            .min_connections(1)
            .logging(false)
            .build();

        let conn = DbConnection::connect(&config).await?;

        // 3. Run migrations
        M::up(conn.inner(), None)
            .await
            .map_err(|e| FrameworkError::database(format!("Migration failed: {e}")))?;

        // 4. Register in TestContainer - this is the key integration!
        // Any code calling DB::connection() or App::resolve::<DbConnection>()
        // will now get this test database
        TestContainer::singleton(conn.clone());

        Ok(Self {
            conn,
            _guard: guard,
        })
    }

    /// Get a reference to the underlying database connection
    ///
    /// Use this when you need to execute queries directly in your tests.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let db = test_database!();
    /// let users = users::Entity::find().all(db.conn()).await?;
    /// ```
    pub fn conn(&self) -> &DatabaseConnection {
        self.conn.inner()
    }

    /// Get the `DbConnection` wrapper
    ///
    /// Use this when you need the full `DbConnection` type.
    pub fn db(&self) -> &DbConnection {
        &self.conn
    }
}

/// Create a test database with default migrator
///
/// This macro creates a `TestDatabase` using `crate::migrations::Migrator` as the
/// default migrator. This follows the Ferro convention where migrations are defined
/// in `src/migrations/mod.rs`.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::test_database;
///
/// #[tokio::test]
/// async fn test_user_creation() {
///     let db = test_database!();
///
///     let action = CreateUserAction::new();
///     let user = action.execute("test@example.com").await.unwrap();
///     assert!(user.id > 0);
/// }
/// ```
///
/// # With Custom Migrator
///
/// ```rust,ignore
/// let db = test_database!(my_crate::CustomMigrator);
/// ```
#[macro_export]
#[allow(clippy::crate_in_macro_def)]
macro_rules! test_database {
    () => {
        $crate::testing::TestDatabase::fresh::<crate::migrations::Migrator>()
            .await
            .expect("Failed to set up test database")
    };
    ($migrator:ty) => {
        $crate::testing::TestDatabase::fresh::<$migrator>()
            .await
            .expect("Failed to set up test database")
    };
}

#[cfg(test)]
mod fresh_pool_tests {
    use super::TestDatabase;
    use crate::database::DB;
    use sea_orm::{ConnectionTrait, DatabaseBackend, Statement, TransactionTrait};
    use sea_orm_migration::{MigrationTrait, MigratorTrait};

    /// Empty migrator — these tests build their own tables via raw SQL.
    struct NoopMigrator;
    impl MigratorTrait for NoopMigrator {
        fn migrations() -> Vec<Box<dyn MigrationTrait>> {
            vec![]
        }
    }

    /// Regression: a nested `DB::connection()` query MUST succeed while an outer
    /// transaction is open. Production runs a multi-connection pool, so a handler
    /// that holds a txn on one connection (writing table A) and reads a *different*
    /// table on another connection works fine; the old single-connection
    /// `sqlite::memory:` test pool deadlocked the second acquire until the sqlx
    /// timeout. Shared-cache makes the second connection see data committed before
    /// the txn. (Reads target a different table than the txn writes — exactly the
    /// real webhook flow: the txn writes `payment_intents`, the loader reads
    /// `orders` — so there is no SQLite table-lock contention.)
    #[tokio::test]
    async fn nested_connection_during_open_txn_does_not_deadlock() {
        let _db = TestDatabase::fresh::<NoopMigrator>().await.unwrap();
        let backend = DatabaseBackend::Sqlite;

        let c = DB::connection().unwrap();
        c.inner()
            .execute(Statement::from_string(
                backend,
                "CREATE TABLE outer_t (id INTEGER PRIMARY KEY)",
            ))
            .await
            .unwrap();
        c.inner()
            .execute(Statement::from_string(
                backend,
                "CREATE TABLE other_t (id INTEGER PRIMARY KEY, v TEXT)",
            ))
            .await
            .unwrap();
        c.inner()
            .execute(Statement::from_string(
                backend,
                "INSERT INTO other_t (id, v) VALUES (1, 'committed')",
            ))
            .await
            .unwrap();

        // Open a write transaction on `outer_t` (mirrors the handler writing
        // payment_intents).
        let txn = DB::connection().unwrap().inner().clone().begin().await.unwrap();
        txn.execute(Statement::from_string(
            backend,
            "INSERT INTO outer_t (id) VALUES (99)",
        ))
        .await
        .unwrap();

        // While the txn is open, a nested connection reads a DIFFERENT table
        // (mirrors the loader reading `orders`). Must NOT deadlock on acquire and
        // MUST see the committed row. A single-connection pool blocks here until
        // the acquire timeout.
        let nested = DB::connection().unwrap();
        let rows = nested
            .inner()
            .query_all(Statement::from_string(
                backend,
                "SELECT id FROM other_t WHERE id = 1",
            ))
            .await
            .unwrap();
        assert_eq!(
            rows.len(),
            1,
            "nested connection must read committed data without deadlocking"
        );

        txn.commit().await.unwrap();
    }

    /// Each `fresh()` must yield an isolated database (unique shared-cache name).
    #[tokio::test]
    async fn fresh_databases_are_isolated() {
        let backend = DatabaseBackend::Sqlite;
        {
            let _db1 = TestDatabase::fresh::<NoopMigrator>().await.unwrap();
            DB::connection()
                .unwrap()
                .inner()
                .execute(Statement::from_string(
                    backend,
                    "CREATE TABLE iso_t (id INTEGER PRIMARY KEY)",
                ))
                .await
                .unwrap();
        }
        // A new fresh DB must not see the previous DB's tables.
        let _db2 = TestDatabase::fresh::<NoopMigrator>().await.unwrap();
        let res = DB::connection()
            .unwrap()
            .inner()
            .query_all(Statement::from_string(
                backend,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='iso_t'",
            ))
            .await
            .unwrap();
        assert!(
            res.is_empty(),
            "a fresh test DB must not see a prior DB's tables (isolation)"
        );
    }
}
