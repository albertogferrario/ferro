---
name: ferro:route:explain
description: Explain a specific route in detail
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
---

<objective>
Show detailed information about a specific route including handler code, middleware, and request/response types.

Uses ferro-mcp `explain_route` and `get_handler` tools.
</objective>

<arguments>
Required:
- `<path>` - Route path to explain (e.g., `/api/users`, `/api/posts/{id}`)

Optional:
- `--method=METHOD` - HTTP method if multiple handlers for same path

Examples:
- `/ferro:route:explain /api/users`
- `/ferro:route:explain /api/users/{id} --method=PUT`
</arguments>

<process>

<step name="find_route">

Use ferro-mcp `explain_route` tool to get route details.

If MCP unavailable, search route definitions:

```bash
grep -rn "\"$PATH\"" src/routes.rs src/**/routes.rs 2>/dev/null
```

</step>

<step name="get_handler">

Use ferro-mcp `get_handler` tool to retrieve handler code.

If MCP unavailable, locate and read handler file:

```bash
# Extract handler path from route definition
# e.g., controllers::users::show -> src/controllers/users.rs
```

</step>

<step name="analyze_middleware">

Identify middleware applied to this route:
1. Route-specific middleware
2. Group middleware
3. Global middleware

</step>

<step name="present_info">

Display comprehensive route information:

```
# Route: GET /api/users/{id}

## Overview
- Method: GET
- Path: /api/users/{id}
- Handler: controllers::users::show
- File: src/controllers/users.rs:45

## Path Parameters
- id: i32 (required)

## Query Parameters
- include: Option<String> - Relations to include

## Middleware Stack (in order)
1. CorsMiddleware (global)
2. RequestLogger (global)
3. ApiMiddleware (group: /api)
4. AuthMiddleware (route)
5. RateLimiter (route) - 100 req/min

## Handler Code

```rust
/// GET /api/users/{id}
///
/// Retrieve a single user by ID.
#[handler]
pub async fn show(
    req: Request,
    id: Path<i32>,
    include: Query<Option<String>>,
    _user: AuthUser,  // Requires authentication
) -> Response {
    let user = User::find(id.0)
        .await?
        .ok_or_else(|| HttpResponse::not_found())?;

    // Handle includes
    let response = if let Some(includes) = include.0 {
        // Load relations
        json!(user.with_relations(&includes).await?)
    } else {
        json!(user)
    };

    Ok(response)
}
```

## Request Example

```bash
curl -X GET "http://localhost:8080/api/users/1?include=posts" \
  -H "Authorization: Bearer <token>" \
  -H "Accept: application/json"
```

## Response

**200 OK**
```json
{
  "id": 1,
  "email": "user@example.com",
  "name": "John Doe",
  "posts": [
    { "id": 1, "title": "Hello World" }
  ]
}
```

**404 Not Found**
```json
{
  "error": "User not found"
}
```

**401 Unauthorized**
```json
{
  "error": "Unauthorized"
}
```

## Related Routes
- POST /api/users - Create user
- PUT /api/users/{id} - Update user
- DELETE /api/users/{id} - Delete user
```

</step>

</process>
