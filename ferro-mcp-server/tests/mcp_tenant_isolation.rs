//! Cross-tenant isolation, scope enforcement, and auth-parity tests for Phase 217.
//!
//! Uses an in-memory SQLite fixture — no consumer app models or Migrator.
//! Tables are created via raw SQL CREATE TABLE + INSERT (same pattern as
//! dispatch_integration.rs and dispatch.rs unit tests).

mod common;

use ferro_mcp_oauth::validate::validate_api_key;
use ferro_mcp_oauth::{validate_bearer, BearerCheck};
use ferro_mcp_server::{handle_tools_call, McpContext};
use ferro_projections::{DataType, FieldMeaning, ServiceDef};
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
use serde_json::json;

// ── Fixture helpers ──────────────────────────────────────────────────────────

async fn setup_isolation_db() -> sea_orm::DatabaseConnection {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("sqlite connect");

    // mcp_api_keys table (canonical Phase 217 schema)
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "CREATE TABLE mcp_api_keys (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            tenant_id  INTEGER NOT NULL,
            key_hash   TEXT NOT NULL UNIQUE,
            scope      TEXT NOT NULL DEFAULT 'read',
            revoked_at TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )"
        .to_string(),
    ))
    .await
    .expect("create mcp_api_keys");

    // orders table (same schema as dispatch.rs fixture)
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "CREATE TABLE orders (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            customer_name TEXT NOT NULL,
            total         REAL NOT NULL,
            status        TEXT NOT NULL,
            tenant_id     INTEGER NOT NULL
        )"
        .to_string(),
    ))
    .await
    .expect("create orders");

    // Seed two tenants' worth of orders
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "INSERT INTO orders (customer_name, total, status, tenant_id) VALUES
            ('Alice', 100.0, 'pending', 1),
            ('Bob',   200.0, 'shipped', 1),
            ('Carol', 150.0, 'pending', 2),
            ('Dave',  250.0, 'shipped', 2)"
            .to_string(),
    ))
    .await
    .expect("seed orders");

    db
}

fn order_service() -> ServiceDef {
    ServiceDef::new("order")
        .mcp_exposed(true)
        .tenant_column("tenant_id")
        .mcp_ability("view-orders")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("customer_name", DataType::String, FieldMeaning::EntityName)
        .field("total", DataType::Float, FieldMeaning::Money)
        .field("status", DataType::String, FieldMeaning::Status)
        .field("tenant_id", DataType::Integer, FieldMeaning::ForeignKey)
}

/// Seed an API key into `mcp_api_keys` and return the raw key.
///
/// When `revoked` is true the `revoked_at` value is a QUOTED date literal
/// (`'2020-01-01T00:00:00Z'`). The NULL branch uses the bare SQL keyword so
/// SQLite stores a proper NULL rather than the string "NULL".
async fn seed_api_key(
    db: &sea_orm::DatabaseConnection,
    tenant_id: i64,
    scope: &str,
    revoked: bool,
) -> String {
    use ferro_mcp_oauth::validate::{generate_mcp_api_key, hash_mcp_api_key};
    let (raw_key, key_hash) = generate_mcp_api_key();
    let revoked_at = if revoked { "'2020-01-01T00:00:00Z'" } else { "NULL" };
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        format!(
            "INSERT INTO mcp_api_keys (tenant_id, key_hash, scope, revoked_at) \
             VALUES ({tenant_id}, '{key_hash}', '{scope}', {revoked_at})"
        ),
    ))
    .await
    .expect("seed api key");
    // hash_mcp_api_key imported for the seed helper — key_hash already computed above
    let _ = hash_mcp_api_key; // suppress unused-import-like warning
    raw_key
}

// ── SC#2: Auth parity — API key and JWT produce same tenant_id ───────────────

/// RED: validate_api_key skeleton returns Invalid; the API-key half will fail
/// the Authenticated assertion until Plan 01 wires real lookup.
///
/// JWT parity assertion kept in ferro-mcp-oauth unit tests (validate.rs) to avoid
/// cross-crate JWT minting complexity here. This test asserts the API-key half only.
#[tokio::test]
async fn api_key_and_jwt_produce_same_tenant_id() {
    use ferro_mcp_oauth::config::OAuthConfig;
    use ferro_mcp_oauth::jwt::{build_claims, mint_token};

    let db = setup_isolation_db().await;
    let raw_key = seed_api_key(&db, 1, "read", false).await;

    // API-key half (RED — skeleton returns Invalid)
    let api_header = format!("Bearer {raw_key}");
    let api_result = validate_api_key(Some(&api_header), &db, None).await;
    let api_tenant_id = match api_result {
        BearerCheck::Authenticated(ref p) => p["tenant_id"].as_i64().expect("tenant_id"),
        other => panic!("expected Authenticated from api key, got {other:?}"),
    };

    // JWT half (already works — validate_bearer is real)
    let secret = b"validate-test-secret-that-is-at-least-32-bytes!!";
    let app_url = "https://app.example.com";
    let oauth_config = OAuthConfig {
        app_name: "Test".to_string(),
        app_url: app_url.to_string(),
        token_secret: secret.to_vec(),
    };
    let claims = build_claims(42, Some(1), app_url, 3600);
    let jwt = mint_token(&claims, secret).expect("mint");
    let jwt_header = format!("Bearer {jwt}");
    let jwt_result = validate_bearer(Some(&jwt_header), &oauth_config, None);
    let jwt_tenant_id = match jwt_result {
        BearerCheck::Authenticated(ref p) => p["tenant_id"].as_i64().expect("tenant_id"),
        other => panic!("expected Authenticated from jwt, got {other:?}"),
    };

    assert_eq!(api_tenant_id, jwt_tenant_id, "API key and JWT must resolve same tenant_id");
}

// ── SC#3: Scope enforcement ───────────────────────────────────────────────────

/// Scope gate wired and real in Wave 0 (D-06). A read-scoped key calling a
/// write tool (non-list_ prefix) must be rejected with -32603.
///
/// This test PASSES immediately — the scope gate in handle_tools_call is real code.
#[tokio::test]
async fn read_scope_key_rejected_on_write_tool_name() {
    let db = setup_isolation_db().await;
    let services = vec![order_service()];
    let ctx = McpContext {
        scope: Some("read".to_string()),
        ..Default::default()
    };

    // "create_order" is a synthetic write tool name (non-list_ prefix)
    let resp = handle_tools_call(
        json!({"name": "create_order", "arguments": {}}),
        &services,
        &db,
        None,
        &ctx,
    )
    .await;

    assert_eq!(
        resp["error"]["code"],
        -32603,
        "read-scoped key on write tool must return -32603, got: {resp}"
    );
    let msg = resp["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("scope insufficient"),
        "error message must mention scope insufficient, got: {msg}"
    );
}

/// A read-scoped key calling a read tool (list_ prefix) must be allowed.
///
/// This test PASSES immediately — list_ tools pass the scope gate.
/// The dispatch may fail (method-not-found if tenant filter logic kicks in), but
/// the scope gate itself must not reject it.
#[tokio::test]
async fn read_scope_key_allowed_on_read_tool() {
    let db = setup_isolation_db().await;
    let services = vec![order_service()];
    let ctx = McpContext {
        scope: Some("read".to_string()),
        tenant_id: Some(1),
        ..Default::default()
    };

    let resp = handle_tools_call(
        json!({"name": "list_order", "arguments": {"limit": 5}}),
        &services,
        &db,
        Some(1),
        &ctx,
    )
    .await;

    // Scope gate must NOT fire — response must have "result", not a scope-rejection error.
    if let Some(err) = resp.get("error") {
        let msg = err["message"].as_str().unwrap_or("");
        assert!(
            !msg.contains("scope insufficient"),
            "read tool must not be rejected by scope gate, got: {err}"
        );
    }
    // If result present, verify it has the expected shape.
    if resp.get("result").is_some() {
        assert!(resp["result"].is_object(), "result must be an object");
    }
}

// ── SC#5: Cross-tenant isolation ─────────────────────────────────────────────

/// RED: validate_api_key skeleton returns Invalid; tenant_id extraction will fail
/// until Plan 01 wires real key lookup. After Plan 01 this becomes GREEN.
#[tokio::test]
async fn api_key_cross_tenant_isolation() {
    let db = setup_isolation_db().await;
    let raw_key = seed_api_key(&db, 1, "read", false).await;
    let services = vec![order_service()];

    // Step 1: resolve tenant_id from API key (RED — skeleton returns Invalid)
    let header = format!("Bearer {raw_key}");
    let check = validate_api_key(Some(&header), &db, None).await;
    let tenant_id = match check {
        BearerCheck::Authenticated(ref p) => p["tenant_id"].as_i64().expect("tenant_id"),
        other => panic!("expected Authenticated from api key, got {other:?}"),
    };
    assert_eq!(tenant_id, 1, "tenant_id must be 1");

    // Step 2: dispatch with resolved tenant_id → only tenant 1 rows returned
    let ctx = McpContext {
        tenant_id: Some(tenant_id),
        ..Default::default()
    };
    let resp = handle_tools_call(
        json!({"name": "list_order", "arguments": {"limit": 10}}),
        &services,
        &db,
        Some(tenant_id),
        &ctx,
    )
    .await;

    let rows = resp["result"]["structuredContent"]["rows"]
        .as_array()
        .expect("structuredContent.rows");

    assert_eq!(rows.len(), 2, "tenant 1 has exactly 2 orders");
    for row in rows {
        assert_eq!(
            row["tenant_id"].as_i64(),
            Some(1),
            "all rows must belong to tenant 1, got: {row}"
        );
    }
}
