# Phase 79: ferro-api-mcp Core - Research

**Researched:** 2026-02-28
**Domain:** Standalone MCP server that bridges OpenAPI specs to MCP tools at runtime
**Confidence:** HIGH

<research_summary>
## Summary

Researched the ecosystem for building a standalone MCP server binary that dynamically registers tools from an OpenAPI spec. The standard approach uses `openapiv3` (v2.2.0) for parsing OpenAPI 3.0.x specs and rmcp's `ToolRoute::new_dyn()` + `ToolRouter::with_route()` for runtime tool registration without compile-time macros.

An existing crate `rmcp-openapi` (v0.24.8) solves this exact problem but uses rmcp 0.15, actix-web transport, and external dependencies that don't align with Ferro's architecture. Building our own is the right approach: leaner, stdio transport (consistent with ferro-mcp), full control for Phase 80's `x-mcp` extensions, and no rmcp version upgrade risk.

**Primary recommendation:** Build a new `ferro-api-mcp` crate using rmcp 0.12 (current), `openapiv3` 2.2.0 for spec parsing, `reqwest` for HTTP calls, and `ToolRoute::new_dyn()` for dynamic tool creation. The crate should be a standalone binary with stdio transport.
</research_summary>

<standard_stack>
## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rmcp | 0.12.0 | MCP protocol SDK | Already used by ferro-mcp; `ToolRoute::new_dyn()` supports dynamic tool registration |
| openapiv3 | 2.2.0 | Parse OpenAPI 3.0.x specs | 604K downloads/month, mature, clean serde deserialization, MIT/Apache-2.0 |
| reqwest | 0.12 | HTTP client for API calls | Already a workspace dependency; async, TLS, JSON support |
| serde_json | 1 | JSON handling | Already a workspace dependency |
| tokio | 1 | Async runtime | Already a workspace dependency |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| schemars | 1 | JSON Schema types | Already in ferro-mcp; may help with schema conversion |
| url | 2 | URL parsing/validation | For base URL handling and spec URL validation |
| thiserror | 2 | Error types | Already a workspace dependency |
| clap | 4 | CLI argument parsing | For the binary's command-line interface |
| tracing | 0.1 | Structured logging | Already a workspace dependency |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| openapiv3 | oas3 (0.4) | oas3 has validation utilities but fewer downloads (less battle-tested) |
| openapiv3 | openapi3-parser | Less mature, fewer features |
| Build our own | rmcp-openapi (0.24.8) | External dep, uses rmcp 0.15 + actix-web, no x-mcp extension support |
| rmcp 0.12 | rmcp 0.16+ | Newer features (trait-based tools, sorted listing) but requires ferro-mcp upgrade too |

**Installation:**
```toml
[dependencies]
rmcp = { version = "0.12", features = ["server", "transport-io"] }
openapiv3 = "2"
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
url = "2"
thiserror = "2"
tracing = "0.1"
clap = { version = "4", features = ["derive"] }
```
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Recommended Crate Structure
```
ferro-api-mcp/
├── Cargo.toml
└── src/
    ├── main.rs              # CLI entry point, argument parsing
    ├── server.rs            # MCP server setup (stdio transport)
    ├── service.rs           # ServerHandler impl with dynamic ToolRouter
    ├── spec/
    │   ├── mod.rs           # OpenAPI spec fetching and parsing
    │   ├── parser.rs        # OpenAPI → internal operation model
    │   └── resolver.rs      # $ref resolution for schemas
    ├── bridge/
    │   ├── mod.rs           # OpenAPI operation → MCP tool conversion
    │   ├── tool_builder.rs  # Build rmcp Tool structs from operations
    │   └── schema.rs        # OpenAPI schema → JSON Schema for input_schema
    ├── http/
    │   ├── mod.rs           # HTTP client for executing API calls
    │   └── auth.rs          # API key header injection
    └── error.rs             # Error types
```

### Pattern 1: Dynamic Tool Registration via ToolRoute::new_dyn
**What:** Register MCP tools at runtime from parsed OpenAPI operations instead of compile-time macros
**When to use:** When tools are discovered at startup (not known at compile time)
**Example:**
```rust
use rmcp::handler::server::router::tool::{ToolRoute, ToolRouter};
use rmcp::model::Tool;
use std::sync::Arc;

fn build_tool_router(operations: Vec<ApiOperation>, http_client: Arc<HttpClient>) -> ToolRouter<ApiMcpService> {
    let mut router = ToolRouter::new();

    for op in operations {
        let client = http_client.clone();
        let op_clone = op.clone();

        let tool = Tool::new(
            op.tool_name.clone(),           // e.g., "list_users"
            op.description.clone(),          // from OpenAPI summary/description
            op.input_schema.clone(),         // JSON Schema from OpenAPI params
        );

        let route = ToolRoute::new_dyn(tool, move |ctx| {
            let client = client.clone();
            let op = op_clone.clone();
            Box::pin(async move {
                // Extract arguments from ctx, execute HTTP request
                let args = ctx.request().arguments.clone();
                client.execute(&op, args).await
            })
        });

        router.add_route(route);
    }

    router
}
```

### Pattern 2: Custom ServerHandler with Dynamic Router
**What:** Implement ServerHandler manually instead of using #[tool_handler] macro, allowing dynamic router access
**When to use:** When the ToolRouter is built at runtime and stored in self
**Example:**
```rust
use rmcp::{ServerHandler, model::ServerInfo};

impl ServerHandler for ApiMcpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(format!(
                "API tools for {}. {} tools available.",
                self.api_name, self.tool_router.list_all().len()
            )),
            capabilities: rmcp::model::ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }

    async fn list_tools(&self, _request: Option<PaginatedRequestParam>, _context: RequestContext<RoleServer>)
        -> Result<ListToolsResult, ErrorData>
    {
        Ok(ListToolsResult::with_all_items(self.tool_router.list_all()))
    }

    async fn call_tool(&self, request: CallToolRequestParam, context: RequestContext<RoleServer>)
        -> Result<CallToolResult, ErrorData>
    {
        let tcc = ToolCallContext::new(self, request, context);
        self.tool_router.call(tcc).await
    }
}
```

### Pattern 3: OpenAPI Operation → Internal Model
**What:** Parse OpenAPI spec into a simplified internal representation before building MCP tools
**When to use:** Always — decouple OpenAPI parsing from MCP tool construction
**Example:**
```rust
/// Simplified representation of an API operation for MCP tool generation.
struct ApiOperation {
    tool_name: String,          // from operationId or generated
    method: String,             // GET, POST, PUT, DELETE, PATCH
    path: String,               // /api/v1/users/{id}
    description: String,        // from summary + description
    parameters: Vec<ApiParam>,  // path, query, header params
    request_body: Option<serde_json::Value>,  // JSON Schema for body
    input_schema: Arc<serde_json::Map<String, serde_json::Value>>,  // merged JSON Schema
}

struct ApiParam {
    name: String,
    location: ParamLocation,    // Path, Query, Header
    required: bool,
    schema: serde_json::Value,  // JSON Schema
    description: Option<String>,
}
```

### Pattern 4: Tool Naming from OpenAPI
**What:** Generate consistent MCP tool names from OpenAPI operations
**When to use:** For every operation in the spec
**Rules:**
1. Use `operationId` if present (already unique per spec)
2. Fall back to `{method}_{path_segments}` (e.g., `get_api_v1_users`)
3. Sanitize: lowercase, replace non-alphanumeric with `_`, collapse consecutive `_`
4. Deduplicate if needed

### Anti-Patterns to Avoid
- **Inline reference resolution:** Don't resolve `$ref` lazily during tool execution. Resolve all `$ref` entries once at startup when parsing the spec.
- **One tool per HTTP method per path:** Don't create separate "list" and "get" if the path is different. Map each unique (method, path) pair to exactly one tool.
- **Hardcoded JSON Schema:** Don't hand-construct JSON Schema objects. Convert from OpenAPI's parameter/requestBody schemas directly.
- **Blocking HTTP in tool handler:** All API calls must be async via reqwest. Never block the tokio runtime.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| OpenAPI parsing | Custom JSON walker | `openapiv3` crate | 3.0.x spec is complex with $ref, allOf, oneOf, discriminators |
| MCP protocol | Custom JSON-RPC | `rmcp` crate | Protocol compliance, transport handling, tool routing |
| HTTP client | Custom TCP/TLS | `reqwest` | TLS, redirects, timeouts, connection pooling |
| JSON Schema validation | Custom validator | Defer to MCP client | MCP clients validate tool inputs against schemas themselves |
| $ref resolution | Custom resolver | Manual traversal of `components.schemas` | openapiv3's `ReferenceOr::Item` vs `ReferenceOr::Reference` is sufficient; walk `components` map for refs |
| URL construction | String formatting | `url` crate | Path parameter interpolation, query string encoding |
| CLI args | Manual parsing | `clap` | Validation, help text, error messages |

**Key insight:** The hard part of this phase is the *mapping logic* between OpenAPI and MCP — converting operations to tools, parameters to input schemas, and executing the right HTTP call with the right arguments. The protocol handling (rmcp) and spec parsing (openapiv3) are solved problems. Focus implementation effort on the bridge layer.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Unresolved $ref in OpenAPI Schema
**What goes wrong:** Parameter or request body schemas reference `$ref: "#/components/schemas/User"` but the tool's input_schema contains the raw reference instead of the resolved schema
**Why it happens:** openapiv3 preserves `ReferenceOr::Reference` variants; they don't auto-resolve
**How to avoid:** Resolve all `$ref` entries during spec parsing phase, before building MCP tools. Walk `components.schemas` and inline referenced schemas into each operation's parameter definitions.
**Warning signs:** MCP client shows empty or invalid parameter descriptions; tool calls fail validation

### Pitfall 2: Missing operationId Causes Unnamed Tools
**What goes wrong:** OpenAPI operations without `operationId` produce tools with no name or duplicate names
**Why it happens:** Ferro's `build_openapi_spec()` uses route names (e.g., `api.users.index`) as operationId, but external specs may omit it
**How to avoid:** Generate deterministic fallback names from `{method}_{sanitized_path}`. Check for uniqueness and deduplicate.
**Warning signs:** Tools named `null` or `undefined`, or MCP client errors about duplicate tool names

### Pitfall 3: Request Body Not Mapped to Tool Input
**What goes wrong:** POST/PUT/PATCH operations that expect a JSON body have no input parameters in the MCP tool
**Why it happens:** Only mapping `parameters` (path/query) and forgetting `requestBody`
**How to avoid:** Merge both `parameters` (path, query) and `requestBody` schema into a single JSON Schema `input_schema`. Use a convention like wrapping body fields under a `"body"` property or flattening them.
**Warning signs:** Create/update tools accept no arguments, or the AI can't provide required data

### Pitfall 4: Path Parameter Interpolation Errors
**What goes wrong:** API calls fail with 404 because path parameters like `{id}` aren't substituted in the URL
**Why it happens:** Forgetting to replace `{param}` in the path template with actual values from tool arguments
**How to avoid:** Before making the HTTP request, iterate path parameters and replace `{name}` in the URL template with the corresponding argument value.
**Warning signs:** 404 responses for endpoints that should exist; URLs containing literal `{id}`

### Pitfall 5: OpenAPI 3.1 vs 3.0 Incompatibility
**What goes wrong:** Spec fails to parse or produces unexpected results
**Why it happens:** openapiv3 (v2.2.0) only supports OpenAPI 3.0.x. OpenAPI 3.1 changed JSON Schema dialect (from draft-04 to 2020-12) and other structural differences.
**How to avoid:** Validate the spec's `openapi` version field at startup. Reject 3.1+ specs with a clear error message pointing to this limitation. Ferro's own specs are 3.0.x (generated by utoipa 5).
**Warning signs:** Parse errors on valid-looking specs; unexpected `null` in schema types
</common_pitfalls>

<code_examples>
## Code Examples

### Parsing an OpenAPI Spec from URL
```rust
// Source: openapiv3 docs + reqwest patterns
use openapiv3::OpenAPI;

async fn fetch_and_parse_spec(url: &str) -> Result<OpenAPI, Error> {
    let response = reqwest::get(url).await?;
    let json_text = response.text().await?;
    let spec: OpenAPI = serde_json::from_str(&json_text)?;

    // Validate version
    if !spec.openapi.starts_with("3.0") {
        return Err(Error::UnsupportedVersion(spec.openapi.clone()));
    }

    Ok(spec)
}
```

### Extracting Operations from OpenAPI Spec
```rust
// Source: openapiv3 PathItem + Operation structs
use openapiv3::{OpenAPI, ReferenceOr, PathItem, Operation};

fn extract_operations(spec: &OpenAPI) -> Vec<ApiOperation> {
    let mut operations = Vec::new();

    for (path, path_item_ref) in &spec.paths.paths {
        let path_item = match path_item_ref {
            ReferenceOr::Item(item) => item,
            ReferenceOr::Reference { .. } => continue, // Skip path-level refs
        };

        let methods = [
            ("GET", &path_item.get),
            ("POST", &path_item.post),
            ("PUT", &path_item.put),
            ("PATCH", &path_item.patch),
            ("DELETE", &path_item.delete),
        ];

        for (method, op_opt) in methods {
            if let Some(operation) = op_opt {
                operations.push(build_api_operation(
                    spec, method, path, operation, &path_item.parameters,
                ));
            }
        }
    }

    operations
}
```

### Building MCP Tool Input Schema from OpenAPI Parameters
```rust
// Source: MCP spec + openapiv3 parameter handling
use serde_json::{json, Map, Value};

fn build_input_schema(
    params: &[ApiParam],
    request_body_schema: Option<&Value>,
) -> Arc<Map<String, Value>> {
    let mut properties = Map::new();
    let mut required = Vec::new();

    // Add path and query parameters
    for param in params {
        let mut prop = param.schema.clone();
        if let Some(desc) = &param.description {
            prop.as_object_mut().map(|o| o.insert(
                "description".to_string(), Value::String(desc.clone())
            ));
        }
        properties.insert(param.name.clone(), prop);
        if param.required {
            required.push(Value::String(param.name.clone()));
        }
    }

    // Add request body as "body" property for POST/PUT/PATCH
    if let Some(body_schema) = request_body_schema {
        properties.insert("body".to_string(), json!({
            "type": "object",
            "description": "Request body",
            "properties": body_schema.get("properties").cloned().unwrap_or(json!({})),
        }));
    }

    let schema = json!({
        "type": "object",
        "properties": properties,
        "required": required,
    });

    Arc::new(schema.as_object().unwrap().clone())
}
```

### Creating Dynamic MCP Tool Route
```rust
// Source: rmcp 0.12 ToolRoute::new_dyn docs
use rmcp::handler::server::router::tool::ToolRoute;
use rmcp::model::{Tool, CallToolResult, Content};

fn create_tool_route(
    op: ApiOperation,
    http_client: Arc<HttpClient>,
) -> ToolRoute<ApiMcpService> {
    let tool = Tool::new(
        op.tool_name.clone(),
        op.description.clone(),
        op.input_schema.clone(),
    );

    ToolRoute::new_dyn(tool, move |ctx| {
        let client = http_client.clone();
        let op = op.clone();
        Box::pin(async move {
            let args = ctx.request().arguments.clone().unwrap_or_default();
            match client.execute(&op, &args).await {
                Ok(response) => Ok(CallToolResult::success(vec![
                    Content::text(serde_json::to_string_pretty(&response)
                        .unwrap_or_else(|_| response.to_string()))
                ])),
                Err(e) => Ok(CallToolResult::error(vec![
                    Content::text(format!("API call failed: {e}"))
                ])),
            }
        })
    })
}
```

### HTTP Client with API Key Auth
```rust
// Source: reqwest patterns, Ferro API key convention
struct HttpClient {
    client: reqwest::Client,
    base_url: url::Url,
    api_key: Option<String>,
}

impl HttpClient {
    async fn execute(
        &self,
        op: &ApiOperation,
        args: &Map<String, Value>,
    ) -> Result<Value, Error> {
        // Interpolate path parameters
        let mut path = op.path.clone();
        for param in &op.parameters {
            if param.location == ParamLocation::Path {
                if let Some(val) = args.get(&param.name) {
                    let val_str = match val {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    path = path.replace(&format!("{{{}}}", param.name), &val_str);
                }
            }
        }

        let url = self.base_url.join(&path)?;
        let mut request = self.client.request(
            op.method.parse()?,
            url,
        );

        // Add API key header
        if let Some(key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {key}"));
        }

        // Add query parameters
        let query_params: Vec<(&str, String)> = op.parameters.iter()
            .filter(|p| p.location == ParamLocation::Query)
            .filter_map(|p| args.get(&p.name).map(|v| (p.name.as_str(), v.to_string())))
            .collect();
        if !query_params.is_empty() {
            request = request.query(&query_params);
        }

        // Add request body for POST/PUT/PATCH
        if let Some(body) = args.get("body") {
            request = request.json(body);
        }

        let response = request.send().await?;
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            serde_json::from_str(&body).or(Ok(Value::String(body)))
        } else {
            Err(Error::ApiError { status: status.as_u16(), body })
        }
    }
}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Custom MCP JSON-RPC | rmcp crate (official SDK) | 2025 | Standard protocol handling, maintained by MCP team |
| Static tool registration only | `ToolRoute::new_dyn()` in rmcp | rmcp 0.10+ | Enables runtime tool discovery from external specs |
| Manual OpenAPI parsing | openapiv3 v2.2.0 | Stable since 2024 | Full 3.0.x spec coverage with serde |
| rmcp-openapi (external) | Custom ferro-api-mcp | N/A | Better control for x-mcp extensions, consistent transport |

**New tools/patterns to consider:**
- **rmcp 0.17**: Adds trait-based tool declaration, but 0.12 `new_dyn` is sufficient for our dynamic case
- **rmcp-openapi crate**: Reference implementation for OpenAPI→MCP bridge. Useful for architecture reference but not as a dependency.
- **MCP tool annotations**: rmcp supports `ToolAnnotations` (readOnlyHint, destructiveHint, idempotentHint) — we should set these based on HTTP method semantics (GET=readonly+idempotent, DELETE=destructive, etc.)

**Deprecated/outdated:**
- **oas3 crate**: Lower adoption than openapiv3, no clear advantage
- **Manual JSON-RPC for MCP**: Always use rmcp
</sota_updates>

<open_questions>
## Open Questions

1. **Body parameter flattening vs nesting**
   - What we know: POST/PUT operations have both path/query params and a request body. Both need to be in the tool's input_schema.
   - What's unclear: Should body fields be nested under a `"body"` key (clean separation) or flattened alongside other params (simpler for AI)?
   - Recommendation: Use `"body"` nesting — prevents name collisions between path params and body fields, and aligns with rmcp-openapi's approach.

2. **rmcp version: stay on 0.12 or upgrade?**
   - What we know: 0.12 has `ToolRoute::new_dyn()` which is all we need. 0.13+ adds blanket impls, 0.17 adds trait-based tools.
   - What's unclear: Whether 0.13-0.17 changes break ferro-mcp compatibility.
   - Recommendation: Stay on 0.12 for Phase 79. Consider upgrade as separate future work if needed.

3. **Schema $ref resolution depth**
   - What we know: Ferro's generated specs (from utoipa) are relatively flat — route parameters inline, no deep $ref chains.
   - What's unclear: How deep external specs might nest $ref entries.
   - Recommendation: Implement single-level $ref resolution (lookup in `components.schemas`). Flag unresolved deep refs as warnings. Sufficient for Ferro specs and most real-world APIs.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- rmcp 0.12 docs — ToolRoute::new_dyn, ToolRouter, Tool struct, ServerHandler trait ([docs.rs/rmcp/0.12.0](https://docs.rs/rmcp/0.12.0/rmcp/))
- openapiv3 2.2.0 docs — OpenAPI, PathItem, Operation, ReferenceOr structs ([docs.rs/openapiv3](https://docs.rs/openapiv3/latest/openapiv3/))
- Ferro codebase — ferro-mcp/src/service.rs (existing ToolRouter pattern), framework/src/api/openapi.rs (OpenAPI spec generation)
- Context7 /websites/rs_rmcp — ToolRoute::new_dyn signature, Tool::new constructor, ToolRouter::with_route/add_route

### Secondary (MEDIUM confidence)
- rmcp-openapi v0.24.8 architecture — reference for OpenAPI→MCP conversion patterns ([lib.rs/crates/rmcp-openapi-server](https://lib.rs/crates/rmcp-openapi-server))
- rmcp GitHub releases — version changelog 0.12 through 0.17 ([github.com/modelcontextprotocol/rust-sdk/releases](https://github.com/modelcontextprotocol/rust-sdk/releases))
- mcp-openapi (Python) — tool naming strategy, auth forwarding patterns ([github.com/conorbranagan/mcp-openapi](https://github.com/conorbranagan/mcp-openapi))
- openapiv3 GitHub — ReferenceOr resolution patterns ([github.com/glademiller/openapiv3](https://github.com/glademiller/openapiv3))

### Tertiary (LOW confidence - needs validation)
- None — all findings verified against official docs or source code
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: rmcp 0.12 (MCP SDK) + openapiv3 2.2.0 (spec parsing)
- Ecosystem: reqwest (HTTP client), schemars (JSON Schema), clap (CLI)
- Patterns: Dynamic tool registration, OpenAPI→MCP mapping, $ref resolution
- Pitfalls: Unresolved refs, missing operationId, body mapping, path interpolation, 3.0 vs 3.1

**Confidence breakdown:**
- Standard stack: HIGH — rmcp and openapiv3 verified via Context7 and docs.rs
- Architecture: HIGH — ToolRoute::new_dyn confirmed in rmcp 0.12 docs; pattern validated against rmcp-openapi reference impl
- Pitfalls: HIGH — derived from OpenAPI spec complexity and verified against rmcp-openapi issues
- Code examples: HIGH — constructed from verified API signatures in rmcp 0.12 and openapiv3 2.2.0

**Research date:** 2026-02-28
**Valid until:** 2026-03-30 (30 days — rmcp ecosystem stable at 0.12, openapiv3 stable)
</metadata>

---

*Phase: 79-ferro-api-mcp-core*
*Research completed: 2026-02-28*
*Ready for planning: yes*
