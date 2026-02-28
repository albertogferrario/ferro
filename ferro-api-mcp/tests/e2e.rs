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
        let status = std::process::Command::new("cargo")
            .args(["build", "--bin", "app", "--bin", "ferro-api-mcp"])
            .status()
            .expect("cargo build failed to execute");
        assert!(
            status.success(),
            "cargo build --bin app --bin ferro-api-mcp failed"
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

    let db_url = format!("sqlite://{db_path}?mode=rwc");

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
