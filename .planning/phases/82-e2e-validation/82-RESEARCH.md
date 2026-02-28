# Phase 82: End-to-End Validation - Research

**Researched:** 2026-02-28
**Domain:** Integration testing a multi-binary pipeline (make:api → compile → serve → MCP bridge → tool calls)
**Confidence:** HIGH

<research_summary>
## Summary

Researched the ecosystem for end-to-end testing of the ferro-api-mcp pipeline: CLI code generation, Ferro server startup, OpenAPI spec serving, and MCP tool invocation against live data. The standard approach uses rmcp's `TokioChildProcess` transport to spawn ferro-api-mcp as a child process and interact with it programmatically via `list_tools()` and `call_tool()`.

The key insight is that this phase has **two distinct validation layers**: (1) generated code compiles and serves correctly, and (2) ferro-api-mcp connects and tools work against real data. The first is a standard Rust compilation check; the second uses rmcp's client SDK to drive the MCP protocol programmatically.

**Primary recommendation:** Use rmcp's `transport-child-process` feature to spawn ferro-api-mcp as a child process in integration tests. For the Ferro server, spawn it as a separate `tokio::process::Command` with a random port and poll for TCP readiness. Use SQLite for zero-config database setup. Test against the sample `app/` which already has models, routes, and migrations.
</research_summary>

<standard_stack>
## Standard Stack

### Core (already in workspace)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rmcp | 0.12 | MCP client for test assertions | Same version as ferro-api-mcp; `TokioChildProcess` spawns MCP server |
| tokio | 1 | Async runtime + process spawning | `tokio::process::Command` for spawning Ferro server |
| reqwest | 0.12 | HTTP client for direct API validation | Verify OpenAPI endpoint independently of MCP |
| serde_json | 1 | JSON assertion on tool results | Already in workspace |

### Supporting (for test infrastructure)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio::net::TcpStream | (in tokio) | Port readiness polling | Wait for Ferro server to accept connections |
| tempfile | 3 | Temp directories for SQLite DBs | Isolated test databases |
| assert_cmd | 2 | CLI binary testing | If testing `ferro make:api` output validation |

### Not Needed
| Library | Why Not |
|---------|---------|
| assert_cmd | Integration tests can call `cargo build` + spawn directly; no need for extra dep |
| testcontainers | SQLite eliminates need for containerized Postgres |
| wiremock | Testing against real server, not mocks |

**Key:** rmcp needs `transport-child-process` feature for `TokioChildProcess`:
```toml
[dev-dependencies]
rmcp = { version = "0.12", features = ["client", "transport-child-process"] }
```
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Test Pipeline Architecture
```
Test Runner
  │
  ├─ 1. Setup: Create temp SQLite DB
  │
  ├─ 2. Start Ferro server (tokio::process::Command)
  │     └─ ./app serve --no-migrate  (on random port)
  │     └─ Poll TCP until ready
  │
  ├─ 3. Verify OpenAPI spec served (reqwest GET /api/openapi.json)
  │
  ├─ 4. Start ferro-api-mcp (TokioChildProcess)
  │     └─ --spec-url http://127.0.0.1:{port}/api/openapi.json
  │     └─ --api-key {test_key}
  │
  ├─ 5. MCP assertions (list_tools, call_tool)
  │
  └─ 6. Teardown: Kill server, cleanup temp DB
```

### Pattern 1: Spawn Ferro Server with Random Port
**What:** Start the sample app on a random available port
**When to use:** Every E2E test needs a running Ferro server
**Example:**
```rust
use tokio::process::Command;
use std::process::Stdio;

async fn start_ferro_server(port: u16, db_path: &str) -> tokio::process::Child {
    Command::new("cargo")
        .args(["run", "--bin", "app", "--", "serve", "--no-migrate"])
        .env("SERVER_PORT", port.to_string())
        .env("SERVER_HOST", "127.0.0.1")
        .env("DATABASE_URL", format!("sqlite://{db_path}"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("failed to start ferro server")
}
```

### Pattern 2: Wait for TCP Port Ready
**What:** Poll until the server is accepting connections
**When to use:** After spawning a server process, before making requests
**Example:**
```rust
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration, timeout};

async fn wait_for_port(port: u16, max_wait: Duration) -> bool {
    let addr = format!("127.0.0.1:{port}");
    let deadline = timeout(max_wait, async {
        loop {
            if TcpStream::connect(&addr).await.is_ok() {
                return true;
            }
            sleep(Duration::from_millis(100)).await;
        }
    });
    deadline.await.unwrap_or(false)
}
```

### Pattern 3: MCP Client via TokioChildProcess
**What:** Spawn ferro-api-mcp as a child process and interact via MCP protocol
**When to use:** Testing MCP tool discovery and execution
**Example:**
```rust
use rmcp::{
    model::CallToolRequestParam,
    service::ServiceExt,
    transport::{TokioChildProcess, ConfigureCommandExt},
};

async fn connect_mcp(spec_url: &str, api_key: &str) -> rmcp::service::RunningService<...> {
    let service = ().serve(TokioChildProcess::new(
        tokio::process::Command::new("cargo")
            .configure(|cmd| {
                cmd.args([
                    "run", "--bin", "ferro-api-mcp", "--",
                    "--spec-url", spec_url,
                    "--api-key", api_key,
                ]);
            })
    ).unwrap()).await.unwrap();

    service
}

// List tools
let tools = service.list_tools(Default::default()).await.unwrap();

// Call a tool
let result = service.call_tool(CallToolRequestParam {
    name: "list_users".into(),
    arguments: serde_json::json!({"page": "1"}).as_object().cloned(),
}).await.unwrap();
```

### Pattern 4: Find Available Port
**What:** Get an OS-assigned available port for test isolation
**When to use:** Each test needs its own port to avoid conflicts
**Example:**
```rust
fn get_available_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}
```

### Anti-Patterns to Avoid
- **Hardcoded ports:** Tests will conflict when run in parallel
- **Sleep-based waits:** Use TCP polling, not `sleep(Duration::from_secs(5))`
- **Shared database:** Each test should have its own SQLite file
- **Skipping --no-migrate:** Migrations need DB setup first; run them explicitly
- **Testing generated code in-place:** Generate in temp dir to avoid polluting app/
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| MCP protocol client | Custom JSON-RPC over stdin/stdout | rmcp `TokioChildProcess` + `ServiceExt` | Protocol compliance, handshake, message framing all handled |
| Port readiness check | `sleep(5)` | TCP connect poll loop with timeout | Deterministic, fast, no flaky timing |
| Process lifecycle | Manual pid tracking + kill | `kill_on_drop(true)` on `tokio::process::Child` | Automatic cleanup even on test panic |
| Test database | Postgres setup/teardown | SQLite temp file | Zero config, fast, disposable |
| OpenAPI validation | Parse JSON manually | serde_json::from_str + field assertions | Type-safe, catches schema drift |

**Key insight:** rmcp already provides the complete MCP client implementation. `TokioChildProcess` handles spawning the binary, piping stdin/stdout, and implementing the Transport trait. Don't build any MCP protocol handling — just use the client SDK.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Ferro Server Needs DB Before Startup
**What goes wrong:** Server crashes immediately because `DB::init()` fails
**Why it happens:** The sample app calls `bootstrap::register()` which does `DB::init()` — needs a valid DATABASE_URL
**How to avoid:** Create SQLite file before spawning server; run migrations before serve
**Warning signs:** Server process exits with code 1 immediately

### Pitfall 2: API Key Must Exist in Database
**What goes wrong:** All MCP tool calls return 401 Unauthorized
**Why it happens:** `ApiKeyProvider` queries `api_keys` table — if no key exists, all requests fail
**How to avoid:** After running migrations, insert a test API key directly into SQLite; use its unhashed value for ferro-api-mcp `--api-key`
**Warning signs:** All tool calls return error content with "401"

### Pitfall 3: Port Race Condition
**What goes wrong:** Port is available when checked but taken when server binds
**Why it happens:** `get_available_port()` releases the port, another process grabs it
**How to avoid:** Bind port 0 in test, parse actual port from server output; or accept rare failures
**Warning signs:** "address already in use" error on server startup

### Pitfall 4: Cargo Build Latency in Tests
**What goes wrong:** Tests take minutes because each spawns `cargo run`
**Why it happens:** Cargo recompiles on each `cargo run` invocation
**How to avoid:** Build binaries once with `cargo build` before tests; spawn the built binary directly from `target/debug/`
**Warning signs:** Each test takes 30+ seconds even for trivial assertions

### Pitfall 5: MCP Server Startup Needs Spec Fetch
**What goes wrong:** ferro-api-mcp fails because spec URL isn't ready yet
**Why it happens:** ferro-api-mcp fetches spec on startup — if Ferro server isn't ready, it gets connection refused
**How to avoid:** Wait for Ferro server's TCP port AND verify `/api/openapi.json` returns 200 before starting ferro-api-mcp
**Warning signs:** "failed to fetch OpenAPI spec: connection refused"

### Pitfall 6: make:api Output Not in Sample App
**What goes wrong:** Sample app doesn't have API routes to test against
**Why it happens:** `make:api` generates files, but sample app may not have them integrated
**How to avoid:** Either (a) run `make:api` and integrate into app, or (b) pre-configure the sample app with API routes during test setup
**Warning signs:** `/api/openapi.json` returns 404 or empty spec

### Pitfall 7: OpenAPI Spec Requires API Routes Registration
**What goes wrong:** OpenAPI spec is empty or missing operations
**Why it happens:** `build_openapi_spec()` reads route metadata — routes must be registered with OpenAPI attributes
**How to avoid:** Verify app's `routes.rs` includes the generated API routes module; verify spec has operations before MCP connection
**Warning signs:** ferro-api-mcp starts but reports "0 tools registered"
</common_pitfalls>

<code_examples>
## Code Examples

### Full E2E Test Skeleton
```rust
// Source: rmcp docs + Ferro patterns
use rmcp::{
    model::CallToolRequestParam,
    service::ServiceExt,
    transport::{TokioChildProcess, ConfigureCommandExt},
};
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout, Duration};

#[tokio::test]
async fn test_full_pipeline() {
    // 1. Setup
    let port = get_available_port();
    let db_path = format!("/tmp/ferro_e2e_test_{port}.db");

    // 2. Run migrations
    let migrate = tokio::process::Command::new("target/debug/app")
        .args(["db:migrate"])
        .env("DATABASE_URL", format!("sqlite://{db_path}?mode=rwc"))
        .output()
        .await
        .expect("migration failed");
    assert!(migrate.status.success());

    // 3. Insert test API key
    // (direct SQLite insert or seed command)

    // 4. Start Ferro server
    let mut server = tokio::process::Command::new("target/debug/app")
        .args(["serve", "--no-migrate"])
        .env("SERVER_PORT", port.to_string())
        .env("SERVER_HOST", "127.0.0.1")
        .env("DATABASE_URL", format!("sqlite://{db_path}"))
        .kill_on_drop(true)
        .spawn()
        .expect("server spawn failed");

    // 5. Wait for server
    assert!(wait_for_port(port, Duration::from_secs(30)).await);

    // 6. Verify OpenAPI spec
    let spec_url = format!("http://127.0.0.1:{port}/api/openapi.json");
    let spec = reqwest::get(&spec_url).await.unwrap();
    assert_eq!(spec.status(), 200);

    // 7. Connect ferro-api-mcp
    let mcp = ().serve(TokioChildProcess::new(
        tokio::process::Command::new("target/debug/ferro-api-mcp")
            .configure(|cmd| {
                cmd.args([
                    "--spec-url", &spec_url,
                    "--api-key", "test_api_key_value",
                ]);
            })
    ).unwrap()).await.unwrap();

    // 8. List tools
    let tools = mcp.list_tools(Default::default()).await.unwrap();
    assert!(!tools.tools.is_empty(), "Expected at least one tool");

    // 9. Call a tool
    let result = mcp.call_tool(CallToolRequestParam {
        name: "list_users".into(),
        arguments: serde_json::json!({"page": "1"}).as_object().cloned(),
    }).await.unwrap();
    // Verify result contains expected data structure

    // 10. Cleanup
    mcp.cancel().await.ok();
    server.kill().await.ok();
    std::fs::remove_file(&db_path).ok();
}
```

### Pre-Building Binaries for Speed
```rust
// Source: Cargo test patterns
use std::sync::Once;

static BUILD: Once = Once::new();

fn ensure_binaries_built() {
    BUILD.call_once(|| {
        let status = std::process::Command::new("cargo")
            .args(["build", "--bin", "app", "--bin", "ferro-api-mcp"])
            .status()
            .expect("cargo build failed");
        assert!(status.success(), "cargo build failed");
    });
}
```

### Insert Test API Key into SQLite
```rust
// Source: Ferro API key auth pattern (SHA-256 hashing)
use std::process::Command;

fn insert_test_api_key(db_path: &str, key_value: &str) {
    // Hash the key with SHA-256 (matching ApiKeyProvider)
    use std::collections::hash_map::DefaultHasher; // placeholder
    // In practice: sha2::Sha256 digest of key_value

    let sql = format!(
        "INSERT INTO api_keys (name, prefix, hashed_key, scopes, created_at) \
         VALUES ('test', '{}', '{}', '[]', datetime('now'))",
        &key_value[..16],  // prefix
        hex_sha256(key_value),
    );

    Command::new("sqlite3")
        .args([db_path, &sql])
        .status()
        .expect("sqlite3 insert failed");
}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Mock MCP client | rmcp `TokioChildProcess` client | rmcp 0.12 (2025) | Real protocol testing, not mocks |
| Manual JSON-RPC | `service.call_tool()` | rmcp 0.12 | Type-safe tool invocation |
| Postgres for tests | SQLite with temp files | Always available | Zero-config, no Docker needed |
| Sleep-based waits | TCP poll loops | Best practice | Deterministic, no flaky tests |

**New tools/patterns to consider:**
- **rmcp `transport-child-process` feature:** Purpose-built for spawning MCP servers as child processes — handles stdin/stdout piping, process lifecycle
- **`kill_on_drop(true)`:** Tokio process feature that ensures child process cleanup even on panic — critical for test reliability

**Deprecated/outdated:**
- **Manual JSON-RPC messaging over pipes:** rmcp handles all protocol framing
- **Compile-time MCP tool testing:** For dynamic tools, must test at runtime with real spec
</sota_updates>

<open_questions>
## Open Questions

1. **Should the sample app be pre-configured with `make:api` output?**
   - What we know: The sample app has User and Todo models but no API routes from `make:api`
   - What's unclear: Whether to run `make:api` as part of test setup or pre-commit the generated files
   - Recommendation: Pre-commit API routes into the sample app — tests validate the running pipeline, not code generation. Code generation is already tested in ferro-cli's 42 unit tests.

2. **How to seed test data for CRUD tool validation?**
   - What we know: Need at least one record to test list/show/update/delete tools
   - What's unclear: Best approach — direct SQLite INSERT, Ferro factories, or store tool
   - Recommendation: Use direct SQLite INSERT for speed and independence from framework; or use the MCP `store` tool itself (tests store first, then list/show)

3. **Integration test location: ferro-api-mcp/tests/ or workspace root tests/?**
   - What we know: ferro-api-mcp currently has only inline tests; no `tests/` directory
   - What's unclear: Whether E2E tests belong in the crate or at workspace level
   - Recommendation: `ferro-api-mcp/tests/e2e.rs` — it's testing ferro-api-mcp's behavior, even though it depends on the sample app binary

4. **rmcp client features needed**
   - What we know: rmcp 0.12 `TokioChildProcess` requires `transport-child-process` feature + `client` feature
   - What's unclear: Whether current rmcp 0.12 has `CallToolRequestParam.task` field (seen in newer docs) or not
   - Recommendation: Check rmcp 0.12 API surface during implementation; may need version pin or feature gate
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- rmcp docs (Context7 /websites/rs_rmcp) — TokioChildProcess transport, client SDK, `ServiceExt::serve()`, `call_tool()` API
- Ferro codebase (direct read) — ferro-api-mcp architecture, sample app structure, server startup flow
- Phase 79 research (79-RESEARCH.md) — rmcp 0.12 ToolRoute::new_dyn, openapiv3, project architecture decisions

### Secondary (MEDIUM confidence)
- [rmcp GitHub](https://github.com/modelcontextprotocol/rust-sdk) — Official Rust SDK, confirmed TokioChildProcess pattern
- [assert_cmd crate](https://crates.io/crates/assert_cmd) — CLI binary testing patterns (evaluated, not needed)
- [Tokio testing docs](https://tokio.rs/tokio/topics/testing) — `#[tokio::test]` patterns, async test best practices
- [axum server testing discussion](https://github.com/tokio-rs/axum/discussions/1701) — Wait-for-ready patterns in integration tests

### Tertiary (LOW confidence - needs validation)
- rmcp `CallToolRequestParam` field names — docs show `.task` field in some versions but not others; verify against 0.12
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: rmcp client SDK for MCP protocol testing
- Ecosystem: tokio process spawning, TCP readiness polling, SQLite for tests
- Patterns: Multi-process integration test architecture, port isolation
- Pitfalls: Database setup, API key seeding, build latency, startup ordering

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already in workspace, verified with Context7
- Architecture: HIGH — patterns derived from rmcp official docs + Ferro codebase analysis
- Pitfalls: HIGH — derived from direct codebase reading (server startup flow, API key auth)
- Code examples: MEDIUM — rmcp client code verified with Context7; full pipeline untested

**Research date:** 2026-02-28
**Valid until:** 2026-03-30 (30 days — rmcp 0.12 stable, Ferro internals known)
</metadata>

---

*Phase: 82-e2e-validation*
*Research completed: 2026-02-28*
*Ready for planning: yes*
