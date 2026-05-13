//! D-31 integration test: replay round trip.
//!
//! Proves the design promise of `ferro-audit`: a sequence of audit entries
//! recording an entity's lifecycle (`created → updated × 3 → status changed`)
//! can be replayed via `history_for_target` + `reconstruct_state` to obtain
//! the entity's final state.
//!
//! This is the load-bearing integration test for Phase 153. If this test
//! fails, the design promise of the crate is broken.

use std::time::Duration;

use ferro_audit::{
    history_for_target, reconstruct_state, AuditActor, AuditEntry, AuditTarget, CreateAuditLogTable,
};
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::prelude::*;
use serde_json::json;

struct TestMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(CreateAuditLogTable)]
    }
}

async fn fresh_db() -> DatabaseConnection {
    let conn = Database::connect("sqlite::memory:")
        .await
        .expect("connect sqlite::memory:");
    TestMigrator::up(&conn, None).await.expect("run migration");
    conn
}

#[tokio::test]
async fn replay_round_trip_inventory_unit_lifecycle() {
    let conn = fresh_db().await;
    let actor = AuditActor::User("u_42".into());
    let target = AuditTarget::new("inventory.unit", "abc");

    // 1. Creation
    AuditEntry::record("inventory.unit.created")
        .actor(actor.clone())
        .target(target.clone())
        .after(json!({ "id": "abc", "quantity": 100, "status": "available" }))
        .write(&conn)
        .await
        .expect("write creation");
    tokio::time::sleep(Duration::from_millis(1100)).await;

    // 2. Adjustment 100 -> 80
    AuditEntry::record("inventory.unit.adjusted")
        .actor(actor.clone())
        .target(target.clone())
        .before(json!({ "quantity": 100 }))
        .after(json!({ "quantity": 80 }))
        .write(&conn)
        .await
        .expect("write adj1");
    tokio::time::sleep(Duration::from_millis(1100)).await;

    // 3. Adjustment 80 -> 50
    AuditEntry::record("inventory.unit.adjusted")
        .actor(actor.clone())
        .target(target.clone())
        .before(json!({ "quantity": 80 }))
        .after(json!({ "quantity": 50 }))
        .write(&conn)
        .await
        .expect("write adj2");
    tokio::time::sleep(Duration::from_millis(1100)).await;

    // 4. Adjustment 50 -> 30
    AuditEntry::record("inventory.unit.adjusted")
        .actor(actor.clone())
        .target(target.clone())
        .before(json!({ "quantity": 50 }))
        .after(json!({ "quantity": 30 }))
        .write(&conn)
        .await
        .expect("write adj3");
    tokio::time::sleep(Duration::from_millis(1100)).await;

    // 5. Status transition: available -> low_stock
    AuditEntry::record("inventory.unit.status_changed")
        .actor(actor.clone())
        .target(target.clone())
        .before(json!({ "status": "available" }))
        .after(json!({ "status": "low_stock" }))
        .reason("quantity_below_threshold")
        .write(&conn)
        .await
        .expect("write status_changed");

    // Fetch the full history (ASC by created_at)
    let entries = history_for_target(&target, &conn)
        .await
        .expect("history fetch");
    assert_eq!(entries.len(), 5, "should have 5 lifecycle entries");

    // Verify ASC ordering (allow same-second collisions with <=)
    for w in entries.windows(2) {
        assert!(
            w[0].created_at <= w[1].created_at,
            "entries should be ASC by created_at"
        );
    }

    // Verify the action sequence (independent of timestamp precision)
    assert_eq!(entries[0].action, "inventory.unit.created");
    assert_eq!(entries[1].action, "inventory.unit.adjusted");
    assert_eq!(entries[2].action, "inventory.unit.adjusted");
    assert_eq!(entries[3].action, "inventory.unit.adjusted");
    assert_eq!(entries[4].action, "inventory.unit.status_changed");

    // The design promise: reconstructed state equals the entity's final form
    let reconstructed =
        reconstruct_state(&entries).expect("reconstruct_state should yield Some(...)");
    let expected = json!({
        "id": "abc",
        "quantity": 30,
        "status": "low_stock"
    });
    assert_eq!(
        reconstructed, expected,
        "reconstruct_state should yield the final entity state"
    );
}
