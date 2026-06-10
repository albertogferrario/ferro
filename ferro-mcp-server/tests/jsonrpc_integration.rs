mod common;

use common::{item_service, setup_db};
use ferro_mcp_server::config::McpServerConfig;
use ferro_mcp_server::jsonrpc::{handle_initialize, handle_tools_call, handle_tools_list};
use ferro_projections::{DataType, FieldMeaning, ServiceDef};
use serde_json::json;

fn test_config() -> McpServerConfig {
    McpServerConfig {
        app_name: "TestApp".to_string(),
        app_url: "https://test.example".to_string(),
        version: "0.0.0".to_string(),
    }
}

#[tokio::test]
async fn initialize_returns_correct_protocol_version() {
    let config = test_config();
    let resp = handle_initialize(json!({}), &config).await;
    assert_eq!(resp["result"]["protocolVersion"], "2025-03-26");
    assert!(resp["result"]["capabilities"]["tools"].is_object());
    assert_eq!(resp["result"]["serverInfo"]["name"], "TestApp");
}

#[tokio::test]
async fn tools_list_returns_only_exposed() {
    let config = test_config();
    let services = vec![
        ServiceDef::new("order").mcp_exposed(true).field(
            "id",
            DataType::Integer,
            FieldMeaning::Identifier,
        ),
        ServiceDef::new("internal").field("id", DataType::Integer, FieldMeaning::Identifier),
    ];
    let resp = handle_tools_list(&services, &config).await;
    let tools = resp["result"]["tools"].as_array().expect("tools array");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "list_order");
}

#[tokio::test]
async fn tools_call_returns_rows() {
    let db = setup_db().await;
    let services = vec![item_service()];
    let resp = handle_tools_call(
        json!({"name": "list_item", "arguments": {"limit": 10, "offset": 0}}),
        &services,
        &db,
        None,
    )
    .await;
    let content = resp["result"]["content"].as_array().expect("content array");
    assert_eq!(content.len(), 3);
}

#[tokio::test]
async fn tools_call_unknown_tool_is_method_not_found() {
    let db = setup_db().await;
    let services = vec![item_service()];
    let resp = handle_tools_call(
        json!({"name": "list_nonexistent", "arguments": {}}),
        &services,
        &db,
        None,
    )
    .await;
    assert_eq!(resp["error"]["code"], -32601);
}

#[tokio::test]
async fn tools_call_unknown_filter_is_invalid_params() {
    // WR-02: a non-filterable / unknown filter key is a client parameter
    // problem (-32602 Invalid params), distinct from an internal error (-32603).
    let db = setup_db().await;
    let services = vec![item_service()];
    let resp = handle_tools_call(
        json!({"name": "list_item", "arguments": {"not_a_field": "x"}}),
        &services,
        &db,
        None,
    )
    .await;
    assert_eq!(resp["error"]["code"], -32602);
}
