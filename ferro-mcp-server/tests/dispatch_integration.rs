//! Integration tests for `dispatch()` — SQLite in-memory fixture.
//!
//! Proves the projection read path end-to-end:
//! - returns rows from a fixture table
//! - paginates correctly (limit/offset)
//! - filters by field value
//! - rejects unknown filter keys (allowlist — no SQL injection)

mod common;

use common::{item_service, setup_db};
use ferro_mcp_server::dispatch;
use ferro_projections::{DataType, FieldMeaning, ServiceDef};

#[tokio::test]
async fn dispatch_empty_filter_returns_all_rows() {
    let db = setup_db().await;
    let result = dispatch(&item_service(), serde_json::json!({}), 25, 0, &db, None)
        .await
        .expect("dispatch should succeed");
    assert_eq!(result.total, 3, "total count should be 3");
    assert_eq!(result.rows.len(), 3, "should return all 3 rows");
    assert_eq!(result.limit, 25);
    assert_eq!(result.offset, 0);
}

#[tokio::test]
async fn dispatch_filter_by_status_returns_matching_rows() {
    let db = setup_db().await;
    let result = dispatch(
        &item_service(),
        serde_json::json!({"status": "open"}),
        25,
        0,
        &db,
        None,
    )
    .await
    .expect("filtered dispatch should succeed");
    assert_eq!(result.total, 2, "two rows have status='open'");
    assert_eq!(result.rows.len(), 2);
    for row in &result.rows {
        assert_eq!(row["status"], serde_json::json!("open"));
    }
}

#[tokio::test]
async fn dispatch_limit_pagination_returns_subset_with_full_total() {
    let db = setup_db().await;
    let result = dispatch(&item_service(), serde_json::json!({}), 2, 0, &db, None)
        .await
        .expect("paginated dispatch should succeed");
    assert_eq!(
        result.total, 3,
        "total should reflect full count (3), not page size"
    );
    assert_eq!(result.rows.len(), 2, "only limit=2 rows returned");
    assert_eq!(result.limit, 2);
    assert_eq!(result.offset, 0);
}

#[tokio::test]
async fn dispatch_non_filterable_field_rejected() {
    // WR-01 regression: a known-but-non-filter-eligible field (Sensitive) must be
    // rejected as a filter key, identically to an unknown key — so it can never
    // reach the WHERE clause and leak via SELECT *.
    let db = setup_db().await;
    let service = ServiceDef::new("item")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("secret_token", DataType::String, FieldMeaning::Sensitive);
    let res = dispatch(
        &service,
        serde_json::json!({"secret_token": "x"}),
        25,
        0,
        &db,
        None,
    )
    .await;
    assert!(
        res.is_err(),
        "a Sensitive field must be rejected as a filter key, not interpolated into SQL"
    );
}

#[tokio::test]
async fn dispatch_unknown_filter_key_returns_err() {
    let db = setup_db().await;
    let res = dispatch(&item_service(), serde_json::json!({"bogus": 1}), 25, 0, &db, None).await;
    assert!(
        res.is_err(),
        "unknown filter key must be rejected, not interpolated into SQL"
    );
    let msg = res.unwrap_err().to_string();
    assert!(
        msg.contains("bogus"),
        "error message should name the rejected key, got: {msg}"
    );
}
