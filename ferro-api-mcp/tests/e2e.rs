//! End-to-end integration tests for ferro-api-mcp.
//!
//! These tests validate the full pipeline:
//! 1. Sample app serves OpenAPI spec
//! 2. ferro-api-mcp parses it and registers MCP tools
//! 3. MCP tool calls execute against the real API

use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Once;
use std::time::Duration;

use rmcp::model::CallToolRequestParam;
use rmcp::service::RunningService;
use rmcp::transport::TokioChildProcess;
use rmcp::ServiceExt;
use serde_json::json;
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio::time::{sleep, timeout};

/// Known test API key (raw value passed to ferro-api-mcp --api-key).
const TEST_API_KEY: &str = "ferro_test_0123456789abcdef0123456789abcdef";

/// Pre-computed SHA-256 hex digest of TEST_API_KEY.
const TEST_API_KEY_HASH: &str = "1f2625874f1a97f5c45077c40d0a4575168a79e6aca3e72c1e78ee51fc430eb3";

/// First 16 characters of TEST_API_KEY (used as prefix in api_keys table).
const TEST_API_KEY_PREFIX: &str = "ferro_test_01234";

// ── Helpers ──────────────────────────────────────────────────────────

/// Returns an available TCP port by binding to port 0 and releasing.
fn get_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind to port 0");
    listener.local_addr().unwrap().port()
}

/// Polls TCP connectivity until the port accepts connections or times out.
async fn wait_for_port(port: u16, max_wait: Duration) -> bool {
    let addr = format!("127.0.0.1:{port}");
    let result = timeout(max_wait, async {
        loop {
            if TcpStream::connect(&addr).await.is_ok() {
                return true;
            }
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await;
    result.unwrap_or(false)
}

static BUILD: Once = Once::new();

/// Builds the `app` and `ferro-api-mcp` binaries once across all tests.
fn ensure_binaries_built() {
    BUILD.call_once(|| {
        let root = workspace_root();
        let status = std::process::Command::new("cargo")
            .args([
                "build",
                "--workspace",
                "--bin",
                "app",
                "--bin",
                "ferro-api-mcp",
            ])
            .current_dir(&root)
            .status()
            .expect("cargo build failed to execute");
        assert!(
            status.success(),
            "cargo build --workspace --bin app --bin ferro-api-mcp failed"
        );
    });
}

/// Returns the workspace root (parent of ferro-api-mcp/).
fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("ferro-api-mcp should be inside workspace")
        .to_path_buf()
}

/// Holds a running test app instance with cleanup on drop.
struct TestContext {
    #[allow(dead_code)]
    port: u16,
    db_path: String,
    server: tokio::process::Child,
    spec_url: String,
}

impl TestContext {
    /// Kills the server process and removes the temp DB file.
    async fn cleanup(mut self) {
        self.server.kill().await.ok();
        std::fs::remove_file(&self.db_path).ok();
        // Also remove SQLite WAL/SHM files
        std::fs::remove_file(format!("{}-wal", &self.db_path)).ok();
        std::fs::remove_file(format!("{}-shm", &self.db_path)).ok();
    }
}

/// Sets up a complete test app: builds binaries, runs migrations,
/// seeds an API key, and starts the server on a random port.
async fn setup_test_app() -> TestContext {
    ensure_binaries_built();

    let port = get_available_port();
    let db_path = format!("/tmp/ferro_e2e_{port}.db");
    let root = workspace_root();
    let app_bin = root.join("target/debug/app");

    let db_url = format!("sqlite://{db_path}");

    // Run migrations
    let migrate_output = Command::new(&app_bin)
        .arg("db:migrate")
        .env("DATABASE_URL", &db_url)
        .env("APP_NAME", "FerroTest")
        .env("APP_KEY", "test-key-for-e2e-validation-only")
        .env("SESSION_DRIVER", "cookie")
        .env("CACHE_DRIVER", "memory")
        .env("SERVER_HOST", "127.0.0.1")
        .env("SERVER_PORT", port.to_string())
        .current_dir(&root)
        .output()
        .await
        .expect("failed to run migrations");

    assert!(
        migrate_output.status.success(),
        "db:migrate failed: {}",
        String::from_utf8_lossy(&migrate_output.stderr)
    );

    // Insert test API key directly via sqlite3
    let insert_sql = format!(
        "INSERT INTO api_keys (name, prefix, hashed_key, scopes, created_at) \
         VALUES ('e2e-test', '{TEST_API_KEY_PREFIX}', '{TEST_API_KEY_HASH}', '[]', datetime('now'));",
    );

    let sqlite_output = Command::new("sqlite3")
        .args([&db_path, &insert_sql])
        .output()
        .await
        .expect("failed to run sqlite3");

    assert!(
        sqlite_output.status.success(),
        "sqlite3 insert failed: {}",
        String::from_utf8_lossy(&sqlite_output.stderr)
    );

    // Start the server
    let server = Command::new(&app_bin)
        .args(["serve", "--no-migrate"])
        .env("DATABASE_URL", &db_url)
        .env("APP_NAME", "FerroTest")
        .env("APP_KEY", "test-key-for-e2e-validation-only")
        .env("SESSION_DRIVER", "cookie")
        .env("CACHE_DRIVER", "memory")
        .env("SERVER_HOST", "127.0.0.1")
        .env("SERVER_PORT", port.to_string())
        .current_dir(&root)
        .kill_on_drop(true)
        .spawn()
        .expect("failed to spawn server");

    // Wait for server to be ready
    assert!(
        wait_for_port(port, Duration::from_secs(30)).await,
        "server did not become ready on port {port} within 30s"
    );

    let spec_url = format!("http://127.0.0.1:{port}/api/openapi.json");

    TestContext {
        port,
        db_path,
        server,
        spec_url,
    }
}

/// Connects ferro-api-mcp to a running test app via MCP protocol.
async fn connect_mcp(spec_url: &str, api_key: &str) -> RunningService<rmcp::RoleClient, ()> {
    let root = workspace_root();
    let mcp_bin = root.join("target/debug/ferro-api-mcp");

    let mut cmd = tokio::process::Command::new(mcp_bin);
    cmd.args(["--spec-url", spec_url, "--api-key", api_key]);

    let transport = TokioChildProcess::new(cmd).expect("failed to spawn ferro-api-mcp");

    ().serve(transport).await.expect("MCP handshake failed")
}

/// Extracts the text content from a CallToolResult.
fn extract_tool_text(result: &rmcp::model::CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| c.raw.as_text())
        .map(|t| t.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

// ── Tests ───────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn test_openapi_spec_served() {
    let ctx = setup_test_app().await;

    let response = reqwest::get(&ctx.spec_url)
        .await
        .expect("GET openapi.json failed");

    assert_eq!(response.status(), 200, "OpenAPI spec should return 200");

    let body: serde_json::Value = response
        .json()
        .await
        .expect("response should be valid JSON");

    // OpenAPI version field exists
    assert!(
        body.get("openapi").is_some(),
        "spec should have 'openapi' version field"
    );

    // Paths object is non-empty
    let paths = body.get("paths").and_then(|p| p.as_object());
    assert!(
        paths.is_some() && !paths.unwrap().is_empty(),
        "spec should have non-empty 'paths' object"
    );

    // At least one path contains /users
    let has_users_path = paths.unwrap().keys().any(|k| k.contains("/users"));
    assert!(has_users_path, "spec should have a /users path");

    ctx.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_mcp_tool_discovery() {
    let ctx = setup_test_app().await;

    let mcp = connect_mcp(&ctx.spec_url, TEST_API_KEY).await;

    let tools_result = mcp
        .peer()
        .list_tools(None)
        .await
        .expect("list_tools failed");

    assert!(
        !tools_result.tools.is_empty(),
        "MCP should register at least one tool from the OpenAPI spec"
    );

    // Collect tool names
    let tool_names: Vec<String> = tools_result
        .tools
        .iter()
        .map(|t| t.name.to_string())
        .collect();

    // Verify user-related tools exist (names contain "user")
    let user_tools: Vec<&String> = tool_names
        .iter()
        .filter(|n| n.to_lowercase().contains("user"))
        .collect();

    assert!(
        user_tools.len() >= 4,
        "expected at least 4 user-related tools (CRUD), found {}: {:?}",
        user_tools.len(),
        user_tools
    );

    // Print discovered tools for debugging
    eprintln!("Discovered {} tools:", tool_names.len());
    for name in &tool_names {
        eprintln!("  - {name}");
    }

    mcp.cancel().await.ok();
    ctx.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_mcp_crud_operations() {
    let ctx = setup_test_app().await;

    let mcp = connect_mcp(&ctx.spec_url, TEST_API_KEY).await;

    // Discover tools first
    let tools_result = mcp
        .peer()
        .list_tools(None)
        .await
        .expect("list_tools failed");

    let tool_names: Vec<String> = tools_result
        .tools
        .iter()
        .map(|t| t.name.to_string())
        .collect();

    eprintln!("Available tools: {tool_names:?}");

    // Find user CRUD tools by pattern matching
    let find_tool = |patterns: &[&str]| -> Option<String> {
        tool_names
            .iter()
            .find(|name| {
                let lower = name.to_lowercase();
                patterns.iter().any(|p| lower.contains(p))
            })
            .cloned()
    };

    let store_tool = find_tool(&["store", "create_user"]);
    let list_tool = find_tool(&["index", "list_user"]);
    let show_tool = find_tool(&["show", "get_user"]);
    let destroy_tool = find_tool(&["destroy", "delete_user"]);

    // ── Create ──
    let store_name = store_tool.expect("should have a store/create user tool");
    eprintln!("Creating user via tool: {store_name}");

    let create_result = mcp
        .peer()
        .call_tool(CallToolRequestParam {
            name: store_name.into(),
            arguments: Some(
                json!({
                    "body": {
                        "name": "E2E Test User",
                        "email": "e2e@example.com",
                        "password": "secret123"
                    }
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        })
        .await
        .expect("store tool call failed");

    let create_text = extract_tool_text(&create_result);
    eprintln!("Create response: {create_text}");
    assert!(
        !create_result.is_error.unwrap_or(false),
        "create should succeed, got error: {create_text}"
    );

    // Try to extract user ID from response
    let create_json: serde_json::Value = serde_json::from_str(&create_text).unwrap_or(json!({}));
    let user_id = create_json
        .pointer("/data/id")
        .and_then(|v| v.as_i64())
        .unwrap_or(1);
    eprintln!("Created user ID: {user_id}");

    // ── List ──
    let list_name = list_tool.expect("should have a list/index user tool");
    eprintln!("Listing users via tool: {list_name}");

    let list_result = mcp
        .peer()
        .call_tool(CallToolRequestParam {
            name: list_name.into(),
            arguments: Some(json!({"page": "1"}).as_object().unwrap().clone()),
        })
        .await
        .expect("list tool call failed");

    let list_text = extract_tool_text(&list_result);
    eprintln!("List response: {list_text}");
    assert!(
        !list_result.is_error.unwrap_or(false),
        "list should succeed, got error: {list_text}"
    );

    // ── Show ──
    if let Some(show_name) = show_tool {
        eprintln!("Showing user {user_id} via tool: {show_name}");

        let show_result = mcp
            .peer()
            .call_tool(CallToolRequestParam {
                name: show_name.into(),
                arguments: Some(
                    json!({"user": user_id.to_string()})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            })
            .await
            .expect("show tool call failed");

        let show_text = extract_tool_text(&show_result);
        eprintln!("Show response: {show_text}");
        assert!(
            !show_result.is_error.unwrap_or(false),
            "show should succeed, got error: {show_text}"
        );
    }

    // ── Delete ──
    if let Some(destroy_name) = destroy_tool {
        eprintln!("Deleting user {user_id} via tool: {destroy_name}");

        let delete_result = mcp
            .peer()
            .call_tool(CallToolRequestParam {
                name: destroy_name.into(),
                arguments: Some(
                    json!({"user": user_id.to_string()})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            })
            .await
            .expect("destroy tool call failed");

        let delete_text = extract_tool_text(&delete_result);
        eprintln!("Delete response: {delete_text}");
        assert!(
            !delete_result.is_error.unwrap_or(false),
            "delete should succeed, got error: {delete_text}"
        );
    }

    mcp.cancel().await.ok();
    ctx.cleanup().await;
}
