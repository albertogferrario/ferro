---
name: ferro:middleware
description: Generate middleware
allowed-tools:
  - Bash
  - Read
  - Write
  - Glob
---

<objective>
Generate a new Ferro middleware.

Creates:
1. Middleware file in `src/middleware/{name}.rs`
2. Updates `src/middleware/mod.rs`
</objective>

<arguments>
Required:
- `name` - Middleware name (e.g., `RateLimit`, `Cors`, `Logging`)

Optional:
- `--before` - Only before-request logic
- `--after` - Only after-response logic

Examples:
- `/ferro:middleware RateLimit`
- `/ferro:middleware RequestLogger --before`
- `/ferro:middleware ResponseCache --after`
</arguments>

<process>

<step name="parse_args">

Parse middleware name and convert to appropriate case:
- PascalCase for struct name
- snake_case for file name

</step>

<step name="generate_middleware">

Generate middleware file at `src/middleware/{snake_name}.rs`:

```rust
//! {MiddlewareName} Middleware

use ferro_rs::prelude::*;
use ferro_rs::middleware::{Middleware, Next};

/// {MiddlewareName} middleware
///
/// Add description of what this middleware does.
#[derive(Debug, Clone, Copy, Default)]
pub struct {MiddlewareName};

impl {MiddlewareName} {
    /// Create a new instance
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Middleware for {MiddlewareName} {
    async fn handle(&self, req: Request, next: Next) -> Result<HttpResponse, HttpResponse> {
        // ============================================================
        // BEFORE REQUEST
        // Code here runs before the request reaches the handler
        // ============================================================

        // Example: Check something before proceeding
        // if some_condition {
        //     return Err(HttpResponse::forbidden().body("Access denied"));
        // }

        // Example: Add data to request extensions
        // req.extensions_mut().insert(SomeData::new());

        // Call next middleware/handler
        let response = next.run(req).await?;

        // ============================================================
        // AFTER RESPONSE
        // Code here runs after the handler returns
        // ============================================================

        // Example: Add headers to response
        // let response = response.header("X-Custom-Header", "value");

        // Example: Log response
        // tracing::info!("Response status: {}", response.status());

        Ok(response)
    }
}
```

**Before-only middleware (--before):**

```rust
#[async_trait]
impl Middleware for {MiddlewareName} {
    async fn handle(&self, req: Request, next: Next) -> Result<HttpResponse, HttpResponse> {
        // Your before-request logic here

        next.run(req).await
    }
}
```

**After-only middleware (--after):**

```rust
#[async_trait]
impl Middleware for {MiddlewareName} {
    async fn handle(&self, req: Request, next: Next) -> Result<HttpResponse, HttpResponse> {
        let response = next.run(req).await?;

        // Your after-response logic here

        Ok(response)
    }
}
```

</step>

<step name="update_mod">

Update `src/middleware/mod.rs`:

```rust
pub mod {snake_name};
pub use {snake_name}::{MiddlewareName};
```

</step>

<step name="usage_example">

Output usage example:

```
Created middleware: {MiddlewareName}

File: src/middleware/{snake_name}.rs

Usage in routes:

// Apply to specific route
Route::get("/protected", handler::index)
    .middleware({MiddlewareName}::new());

// Apply to route group
Route::group()
    .prefix("/api")
    .middleware({MiddlewareName}::new())
    .routes(|r| {
        r.get("/users", users::index);
        r.get("/posts", posts::index);
    });

// Global middleware (in main.rs or app setup)
app.middleware({MiddlewareName}::new());
```

</step>

</process>

<common_middleware>

## Rate Limiting
```rust
pub struct RateLimit {
    requests_per_minute: u32,
}
```

## CORS
```rust
pub struct Cors {
    allowed_origins: Vec<String>,
    allowed_methods: Vec<String>,
}
```

## Request Logging
```rust
pub struct RequestLogger;
// Logs: method, path, status, duration
```

## Authentication Check
```rust
pub struct RequireAuth;
// Returns 401 if not authenticated
```

## Response Compression
```rust
pub struct Compress {
    threshold: usize,
}
```

</common_middleware>
