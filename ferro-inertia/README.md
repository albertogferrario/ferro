# Ferro Inertia

Server-side [Inertia.js](https://inertiajs.com) adapter for the Ferro framework.

[![Crates.io](https://img.shields.io/crates/v/ferro-inertia.svg)](https://crates.io/crates/ferro-inertia)
[![Documentation](https://docs.rs/ferro-inertia/badge.svg)](https://docs.rs/ferro-inertia)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **Ferro integration** - Seamlessly works with the Ferro framework
- **Async-safe** - No thread-local storage, safe for async runtimes like Tokio
- **Partial reloads** - Efficient updates via `X-Inertia-Partial-Data` header
- **Shared props** - Easily share auth, flash messages, CSRF tokens across all responses
- **Version detection** - Automatic 409 Conflict responses for asset version mismatches
- **Vite integration** - Development mode with HMR support

## Installation

```toml
[dependencies]
ferro-inertia = "0.2"
```

## Quick Start

### 1. Implement the `InertiaRequest` trait

```rust
use ferro_inertia::InertiaRequest;

impl InertiaRequest for MyRequest {
    fn inertia_header(&self, name: &str) -> Option<&str> {
        self.headers().get(name).and_then(|v| v.to_str().ok())
    }

    fn path(&self) -> &str {
        self.uri().path()
    }
}
```

### 2. Render Inertia responses

```rust
use ferro_inertia::Inertia;
use serde_json::json;

async fn index(req: MyRequest) -> MyResponse {
    let response = Inertia::render(&req, "Home", json!({
        "title": "Welcome",
        "user": {
            "name": "John Doe",
            "email": "john@example.com"
        }
    }));

    // Convert InertiaHttpResponse to your framework's response type
    response.into()
}
```

### 3. Add shared props via middleware

```rust
use ferro_inertia::{Inertia, InertiaShared};

async fn handler(req: MyRequest) -> MyResponse {
    let shared = InertiaShared::new()
        .auth(get_current_user())
        .csrf(get_csrf_token())
        .flash(get_flash_messages());

    let response = Inertia::render_with_shared(&req, "Dashboard", props, &shared);
    response.into()
}
```

## Configuration

```rust
use ferro_inertia::InertiaConfig;

// Development (default)
let config = InertiaConfig::new()
    .vite_dev_server("http://localhost:5173")
    .entry_point("src/main.tsx");

// Production
let config = InertiaConfig::new()
    .version("1.0.0")
    .production();

// Custom HTML template
let config = InertiaConfig::new()
    .html_template(r#"
        <!DOCTYPE html>
        <html>
        <head><title>My App</title></head>
        <body>
            <div id="app" data-page="{page}"></div>
            <script src="/app.js"></script>
        </body>
        </html>
    "#);
```

## Version Conflict Detection

```rust
// In middleware, check for version mismatch
if let Some(conflict_response) = Inertia::check_version(&req, "1.0.0", "/") {
    return conflict_response.into();
}
```

## Partial Reloads

Partial reloads are handled automatically. When the client sends:

```
X-Inertia-Partial-Data: user,notifications
X-Inertia-Partial-Component: Dashboard
```

Only the requested props (`user`, `notifications`) will be included in the response.

## Usage in Ferro

`ferro-inertia` is ferro-coupled: the `Request` and `HttpResponse` types come from the framework, and `Inertia::render` accepts a `&Request` directly. The `ferro` crate re-exports `Inertia`, `InertiaProps`, and `SavedInertiaContext`.

### Basic render

```rust
use ferro::{handler, Inertia, InertiaProps, Request, Response};

#[derive(InertiaProps)]
pub struct HomeProps {
    pub title: String,
}

#[handler]
pub async fn index(req: Request) -> Response {
    Inertia::render(&req, "Home", HomeProps {
        title: "Welcome".to_string(),
    })
}
```

Component paths (`"Home"`) are validated at compile time against the frontend component tree.

### Form handlers — `SavedInertiaContext`

`req.input().await` consumes the request, which means you lose access to the Inertia headers needed by `render`. Save the context first, then use `render_ctx`:

```rust
use ferro::{handler, Inertia, Request, Response, SavedInertiaContext};

#[handler]
pub async fn store(req: Request) -> Response {
    let ctx = SavedInertiaContext::from(&req);
    let form = req.input().await?;  // Consumes req

    // ... validate and persist ...

    Inertia::render_ctx(&ctx, "Users/Show", ShowProps { /* ... */ })
}
```

### Shared props

```rust
use ferro::{Inertia, InertiaShared};

let shared = InertiaShared::new()
    .auth(current_user)
    .csrf(csrf_token)
    .flash(flash_messages);

Inertia::render_with_shared(&req, "Dashboard", props, &shared)
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
