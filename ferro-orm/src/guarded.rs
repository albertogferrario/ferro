//! `GuardedUpdate<E>` — chainable builder for atomic conditional `UPDATE`
//! statements. Compiles to exactly one `UPDATE … WHERE …` SQL statement.
//!
//! The database engine's per-statement atomicity (SQLite serial writer,
//! Postgres `READ COMMITTED`) is the entire correctness mechanism — this
//! builder adds the chainable surface and the rows-affected → `GuardedError`
//! mapping on top.

use sea_orm::sea_query::{Condition, IntoCondition, SimpleExpr};
use sea_orm::{ConnectionTrait, EntityTrait, QueryFilter, Update, Value};

use crate::GuardedError;

pub struct GuardedUpdate<E: EntityTrait> {
    entity: E,
    filters: Condition,
    sets: Vec<(E::Column, SimpleExpr)>,
}

impl<E: EntityTrait> GuardedUpdate<E> {
    /// Start a new builder targeting `entity`.
    pub fn new(entity: E) -> Self {
        Self {
            entity,
            filters: Condition::all(), // AND-combiner per D-06
            sets: Vec::new(),
        }
    }

    /// Add a filter expression. Multiple `.filter(...)` calls AND-combine.
    pub fn filter<F: IntoCondition>(mut self, f: F) -> Self {
        self.filters = self.filters.add(f.into_condition());
        self
    }

    /// Set a column to a value-derived expression.
    pub fn set_expr(mut self, col: E::Column, expr: SimpleExpr) -> Self {
        self.sets.push((col, expr));
        self
    }

    /// Set a column to a literal value.
    pub fn set_value(mut self, col: E::Column, value: Value) -> Self {
        self.sets.push((col, SimpleExpr::Value(value)));
        self
    }

    /// Execute the conditional UPDATE; succeed iff exactly one row matched.
    ///
    /// Returns `Err(GuardedError::NoRowsAffected)` on 0 rows (predicate
    /// failure — the race-free "capacity exhausted" signal). Returns
    /// `Err(GuardedError::TooManyRows { affected })` on `>1` rows
    /// (filter is not unique-key-equivalent — index/uniqueness bug).
    ///
    /// Note on `TooManyRows`: this variant is preserved for documentation
    /// and future-proofing. sea-orm's `UpdateMany::exec` returns
    /// `rows_affected` unconditionally on success, so a filter matching
    /// `>1` rows will mutate every matched row before this post-processor
    /// surfaces the error. The variant is the right way to shout when a
    /// supposed unique-key-equivalent filter turns out not to be — see
    /// Pitfall 4 in 152-RESEARCH.md.
    pub async fn exec_one<C: ConnectionTrait>(self, conn: &C) -> Result<(), GuardedError> {
        match self.exec_raw(conn).await? {
            0 => Err(GuardedError::NoRowsAffected),
            1 => Ok(()),
            n => Err(GuardedError::TooManyRows { affected: n }),
        }
    }

    /// Execute the conditional UPDATE; tolerate 0 rows as a normal outcome.
    ///
    /// Returns `Ok(true)` on 1 row, `Ok(false)` on 0 rows. `>1` rows still
    /// returns `Err(GuardedError::TooManyRows)` — the uniqueness contract
    /// is the same.
    pub async fn exec_at_most_one<C: ConnectionTrait>(
        self,
        conn: &C,
    ) -> Result<bool, GuardedError> {
        match self.exec_raw(conn).await? {
            0 => Ok(false),
            1 => Ok(true),
            n => Err(GuardedError::TooManyRows { affected: n }),
        }
    }

    async fn exec_raw<C: ConnectionTrait>(self, conn: &C) -> Result<u64, GuardedError> {
        // Load-bearing — sea-orm's `Updater::is_noop()` short-circuits with
        // `rows_affected: 0` when SET is empty, which would otherwise look
        // like a predicate miss (Pitfall 1 in 152-RESEARCH.md).
        if self.sets.is_empty() {
            return Err(GuardedError::EmptyUpdate);
        }

        let mut stmt = Update::many(self.entity).filter(self.filters);
        for (col, expr) in self.sets {
            stmt = stmt.col_expr(col, expr);
        }
        let result = stmt.exec(conn).await?; // From<DbErr> via #[from] on Db variant.
        Ok(result.rows_affected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::sea_query::Expr;
    use sea_orm::{
        ColumnTrait, ConnectionTrait, Database, DatabaseBackend, EntityTrait, Schema, Set,
        TransactionTrait,
    };

    mod counters {
        use sea_orm::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "counters")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            pub quantity: i32,
            pub status: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    async fn fresh_db() -> sea_orm::DatabaseConnection {
        let conn = Database::connect("sqlite::memory:")
            .await
            .expect("connect to in-memory sqlite");
        let schema = Schema::new(DatabaseBackend::Sqlite);
        let stmt = schema.create_table_from_entity(counters::Entity);
        conn.execute(conn.get_database_backend().build(&stmt))
            .await
            .expect("create counters table");
        conn
    }

    async fn insert_row(conn: &sea_orm::DatabaseConnection, id: i32, quantity: i32, status: &str) {
        counters::Entity::insert(counters::ActiveModel {
            id: Set(id),
            quantity: Set(quantity),
            status: Set(status.to_string()),
        })
        .exec(conn)
        .await
        .expect("insert counters row");
    }

    #[tokio::test]
    async fn predicate_matches_one_row_succeeds() {
        // T-16-1
        let conn = fresh_db().await;
        insert_row(&conn, 1, 5, "pending").await;

        GuardedUpdate::new(counters::Entity)
            .filter(counters::Column::Id.eq(1))
            .filter(counters::Column::Quantity.gte(3))
            .set_expr(
                counters::Column::Quantity,
                Expr::col(counters::Column::Quantity).sub(3),
            )
            .exec_one(&conn)
            .await
            .expect("guarded update should succeed");

        let row = counters::Entity::find_by_id(1)
            .one(&conn)
            .await
            .unwrap()
            .expect("row exists");
        assert_eq!(row.quantity, 2);
    }

    #[tokio::test]
    async fn predicate_fails_zero_rows() {
        // T-16-2
        let conn = fresh_db().await;
        insert_row(&conn, 1, 2, "pending").await;

        // exec_one path
        let err = GuardedUpdate::new(counters::Entity)
            .filter(counters::Column::Id.eq(1))
            .filter(counters::Column::Quantity.gte(5))
            .set_expr(
                counters::Column::Quantity,
                Expr::col(counters::Column::Quantity).sub(5),
            )
            .exec_one(&conn)
            .await
            .expect_err("should fail predicate");
        assert!(matches!(err, GuardedError::NoRowsAffected));

        // exec_at_most_one path
        let updated = GuardedUpdate::new(counters::Entity)
            .filter(counters::Column::Id.eq(1))
            .filter(counters::Column::Quantity.gte(5))
            .set_expr(
                counters::Column::Quantity,
                Expr::col(counters::Column::Quantity).sub(5),
            )
            .exec_at_most_one(&conn)
            .await
            .expect("exec_at_most_one tolerates 0 rows");
        assert!(!updated);

        let row = counters::Entity::find_by_id(1)
            .one(&conn)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row.quantity, 2);
    }

    #[tokio::test]
    async fn predicate_matches_multiple_rows() {
        // T-16-3
        // Note: SeaORM's `Update::many` is multi-row by SQL semantics. The
        // failing `exec_one` call below DOES mutate both rows before our
        // post-processor surfaces TooManyRows. The test re-seeds for the
        // exec_at_most_one check to verify the error variant alone.
        // This is the correct behavior per D-13 and Pitfall 4 — surface
        // the bug loudly rather than silently succeeding.
        let conn = fresh_db().await;
        insert_row(&conn, 1, 10, "pending").await;
        insert_row(&conn, 2, 10, "pending").await;

        let err = GuardedUpdate::new(counters::Entity)
            .filter(counters::Column::Status.eq("pending"))
            .set_expr(
                counters::Column::Quantity,
                Expr::col(counters::Column::Quantity).sub(1),
            )
            .exec_one(&conn)
            .await
            .expect_err("should fail with TooManyRows");
        assert!(matches!(err, GuardedError::TooManyRows { affected: 2 }));

        insert_row(&conn, 3, 10, "shipped").await;
        insert_row(&conn, 4, 10, "shipped").await;
        let err = GuardedUpdate::new(counters::Entity)
            .filter(counters::Column::Status.eq("shipped"))
            .set_expr(
                counters::Column::Quantity,
                Expr::col(counters::Column::Quantity).sub(1),
            )
            .exec_at_most_one(&conn)
            .await
            .expect_err("exec_at_most_one should also fail with TooManyRows");
        assert!(matches!(err, GuardedError::TooManyRows { affected: 2 }));
    }

    #[tokio::test]
    async fn empty_update_no_sets() {
        // T-16-4 — critical: this must error BEFORE any SQL fires (Pitfall 1).
        let conn = fresh_db().await;
        insert_row(&conn, 1, 5, "pending").await;

        let err = GuardedUpdate::new(counters::Entity)
            .filter(counters::Column::Id.eq(1))
            .exec_one(&conn)
            .await
            .expect_err("empty builder must error");
        assert!(matches!(err, GuardedError::EmptyUpdate));

        let err = GuardedUpdate::new(counters::Entity)
            .filter(counters::Column::Id.eq(1))
            .exec_at_most_one(&conn)
            .await
            .expect_err("empty builder must error in exec_at_most_one too");
        assert!(matches!(err, GuardedError::EmptyUpdate));

        let row = counters::Entity::find_by_id(1)
            .one(&conn)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row.quantity, 5);
    }

    #[tokio::test]
    async fn multi_column_set_atomic() {
        // T-16-5
        let conn = fresh_db().await;
        insert_row(&conn, 1, 5, "pending").await;

        GuardedUpdate::new(counters::Entity)
            .filter(counters::Column::Id.eq(1))
            .filter(counters::Column::Status.eq("pending"))
            .set_expr(
                counters::Column::Quantity,
                Expr::col(counters::Column::Quantity).sub(2),
            )
            .set_value(
                counters::Column::Status,
                Value::String(Some(Box::new("committed".to_string()))),
            )
            .exec_one(&conn)
            .await
            .expect("multi-column guarded update");

        let row = counters::Entity::find_by_id(1)
            .one(&conn)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row.quantity, 3);
        assert_eq!(row.status, "committed");
    }

    #[tokio::test]
    async fn transaction_rollback() {
        // T-16-6 — exec inside &DatabaseTransaction; rollback rolls back.
        let conn = fresh_db().await;
        insert_row(&conn, 1, 5, "pending").await;

        let txn = conn.begin().await.expect("begin transaction");

        GuardedUpdate::new(counters::Entity)
            .filter(counters::Column::Id.eq(1))
            .set_expr(
                counters::Column::Quantity,
                Expr::col(counters::Column::Quantity).sub(2),
            )
            .exec_one(&txn)
            .await
            .expect("guarded update inside transaction");

        let row_in_txn = counters::Entity::find_by_id(1)
            .one(&txn)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row_in_txn.quantity, 3);

        txn.rollback().await.expect("rollback");

        let row_after = counters::Entity::find_by_id(1)
            .one(&conn)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row_after.quantity, 5);
    }

    #[tokio::test]
    async fn filter_and_combine() {
        // T-16-7
        let conn = fresh_db().await;
        insert_row(&conn, 1, 5, "pending").await;
        insert_row(&conn, 2, 5, "shipped").await;
        insert_row(&conn, 3, 10, "pending").await;

        GuardedUpdate::new(counters::Entity)
            .filter(counters::Column::Status.eq("pending"))
            .filter(counters::Column::Quantity.eq(5))
            .set_value(
                counters::Column::Status,
                Value::String(Some(Box::new("matched".to_string()))),
            )
            .exec_one(&conn)
            .await
            .expect("AND-filter should match exactly row 1");

        assert_eq!(
            counters::Entity::find_by_id(1)
                .one(&conn)
                .await
                .unwrap()
                .unwrap()
                .status,
            "matched"
        );
        assert_eq!(
            counters::Entity::find_by_id(2)
                .one(&conn)
                .await
                .unwrap()
                .unwrap()
                .status,
            "shipped"
        );
        assert_eq!(
            counters::Entity::find_by_id(3)
                .one(&conn)
                .await
                .unwrap()
                .unwrap()
                .status,
            "pending"
        );
    }
}
