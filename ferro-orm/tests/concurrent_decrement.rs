//! Concurrent-decrement integration test — 10 tokio tasks attempt
//! `GuardedUpdate` on a counter starting at K=3. The SQLite serial writer
//! enforces atomicity at the SQL layer; the test asserts that exactly 3
//! tasks see `Ok(())` and the remaining 7 see `Err(NoRowsAffected)`.
//!
//! Uses `sqlite:file::memory:?cache=shared` + `max_connections = 4` so
//! multiple connections see the same in-memory DB (RESEARCH Pitfall 2).
//! Without the shared-cache variant and a multi-thread tokio runtime, the
//! test would pass for the wrong reason (pool serialization, not SQL-level
//! atomicity), and a buggy read-then-write implementation would also pass.

use std::sync::Arc;

use ferro_orm::{GuardedError, GuardedUpdate};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait, ConnectOptions, ConnectionTrait, Database, DatabaseBackend, EntityTrait, Schema,
    Set,
};

mod counters {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "counters")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub quantity: i32,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ten_tasks_against_capacity_three_exactly_three_succeed() {
    // Shared-cache memory variant so multiple connections see the same DB.
    // `max_connections = 4` lets the pool actually parallelize.
    let mut opts = ConnectOptions::new("sqlite:file::memory:?cache=shared");
    opts.max_connections(4).min_connections(1);
    let conn = Arc::new(
        Database::connect(opts)
            .await
            .expect("connect to shared-cache in-memory sqlite"),
    );

    // Schema
    let schema = Schema::new(DatabaseBackend::Sqlite);
    let stmt = schema.create_table_from_entity(counters::Entity);
    conn.execute(conn.get_database_backend().build(&stmt))
        .await
        .expect("create counters table");

    // Seed: id=1, quantity=3
    counters::Entity::insert(counters::ActiveModel {
        id: Set(1),
        quantity: Set(3),
    })
    .exec(&*conn)
    .await
    .expect("seed initial row");

    // Spawn 10 tasks each trying to decrement by 1 with guard `quantity >= 1`.
    let mut tasks = Vec::with_capacity(10);
    for _ in 0..10 {
        let conn = Arc::clone(&conn);
        tasks.push(tokio::spawn(async move {
            GuardedUpdate::new(counters::Entity)
                .filter(counters::Column::Id.eq(1))
                .filter(counters::Column::Quantity.gte(1))
                .set_expr(
                    counters::Column::Quantity,
                    Expr::col(counters::Column::Quantity).sub(1),
                )
                .exec_one(&*conn)
                .await
        }));
    }

    // Hand-rolled await loop (no `futures` dev-dep).
    let mut successes = 0usize;
    let mut no_rows = 0usize;
    for handle in tasks {
        match handle.await.expect("join task") {
            Ok(()) => successes += 1,
            Err(GuardedError::NoRowsAffected) => no_rows += 1,
            Err(other) => panic!("unexpected error: {other}"),
        }
    }

    assert_eq!(successes, 3, "exactly 3 of 10 tasks should succeed");
    assert_eq!(no_rows, 7, "the other 7 should fail with NoRowsAffected");

    // Final quantity must be 0
    let final_row = counters::Entity::find_by_id(1)
        .one(&*conn)
        .await
        .expect("query final row")
        .expect("row exists");
    assert_eq!(final_row.quantity, 0);
}
