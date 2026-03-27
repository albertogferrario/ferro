# Rate Limiting

Ferro provides cache-backed rate limiting through the `Throttle` middleware. Define named limiters with dynamic, per-request rules or apply inline limits directly on routes. Rate counters use the framework's Cache facade -- in-memory by default, Redis for multi-server deployments.

## Defining Rate Limiters

Register named limiters in `bootstrap.rs` (or a service provider). Each limiter receives the incoming `Request` and returns one or more `Limit` values.

### Basic Limiter

```rust
use ferro::{RateLimiter, Limit};

pub fn register_rate_limiters() {
    RateLimiter::define("api", |_req| {
        Limit::per_minute(60)
    });
}
```

### Auth-Based Segmentation

Use the request to vary limits by authentication state:

```rust
use ferro::{RateLimiter, Limit, Auth};

RateLimiter::define("api", |req| {
    match Auth::id() {
        Some(id) => Limit::per_minute(120).by(format!("user:{}", id)),
        None => Limit::per_minute(60),
    }
});
```

Unauthenticated requests default to the client IP as the rate limit key. Authenticated users get a higher limit keyed by their user ID.

### Multiple Limits

Return a `Vec<Limit>` to enforce several windows simultaneously. The first limit exceeded triggers a `429` response.

```rust
use ferro::{RateLimiter, Limit};

RateLimiter::define("login", |req| {
    let ip = req.header("X-Forwarded-For")
        .and_then(|s| s.split(',').next())
        .unwrap_or("unknown")
        .trim()
        .to_string();

    vec![
        Limit::per_minute(500),           // Global burst cap
        Limit::per_minute(5).by(ip),      // Per-IP cap
    ]
});
```

## Applying to Routes

### Named Throttle

Reference a registered limiter by name with `Throttle::named()`:

```rust
use ferro::Throttle;

routes! {
    group!("/api", {
        get!("/users", controllers::users::index),
        get!("/users/{id}", controllers::users::show),
    }).middleware(Throttle::named("api")),

    group!("/auth", {
        post!("/login", controllers::auth::login),
    }).middleware(Throttle::named("login")),
}
```

### Inline Throttle

For simple cases that do not need a named registration:

```rust
use ferro::Throttle;

get!("/health", controllers::health::check)
    .middleware(Throttle::per_minute(10))
```

Inline limits support the same time windows: `per_second`, `per_minute`, `per_hour`, `per_day`.

## The Limit Struct

`Limit` describes how many requests are allowed in a time window.

### Constructors

| Method | Window |
|--------|--------|
| `Limit::per_second(n)` | 1 second |
| `Limit::per_minute(n)` | 60 seconds |
| `Limit::per_hour(n)` | 3600 seconds |
| `Limit::per_day(n)` | 86400 seconds |

### Key Segmentation

By default, rate limits are keyed by client IP (from `X-Forwarded-For` or `X-Real-IP` headers). Override with `.by()`:

```rust
// Per-user limit
Limit::per_minute(120).by(format!("user:{}", user_id))

// Per-API-key limit
Limit::per_minute(1000).by(api_key)
```

### Custom 429 Response

Override the default JSON error with `.response()`:

```rust
use ferro::HttpResponse;

Limit::per_minute(60).response(|| {
    HttpResponse::json(serde_json::json!({
        "error": "Quota exceeded",
        "upgrade_url": "https://example.com/pricing"
    })).status(429)
})
```

## Response Headers

Every response from a throttled route includes rate limit headers:

| Header | Description |
|--------|-------------|
| `X-RateLimit-Limit` | Maximum requests allowed in the window |
| `X-RateLimit-Remaining` | Requests remaining in the current window |
| `X-RateLimit-Reset` | Seconds until the current window resets |

When a request is rejected (429), an additional header is included:

| Header | Description |
|--------|-------------|
| `Retry-After` | Seconds until the client should retry |

### Example Headers

Successful request:

```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 42
X-RateLimit-Reset: 38
```

Rate limited request (429):

```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 38
Retry-After: 38
```

## Cache Backend

Rate limiting uses the framework's Cache facade for counter storage. The algorithm is fixed-window counters with atomic `INCR` + `EXPIRE` operations.

| Setup | Configuration |
|-------|---------------|
| Single server (default) | No configuration needed. Uses in-memory cache. |
| Multi-server | Set `CACHE_DRIVER=redis` and `REDIS_URL` in `.env` |

```env
# .env for multi-server deployments
CACHE_DRIVER=redis
REDIS_URL=redis://127.0.0.1:6379
```

Cache keys follow the pattern `rate_limit:{name}:{identifier}:{window}` and expire automatically after each window.

## Fail-Open Behavior

Rate limiting is designed to never cause application errors:

- **Cache unavailable:** Requests are allowed with a warning logged to stderr.
- **Named limiter not registered:** Requests are allowed with a warning logged to stderr.
- **Expire call fails:** The counter still works; the key may persist longer than intended.

Rate limiting failures never produce `500` errors. The system prioritizes availability over strict enforcement.

## MCP Tools

Use `list_rate_limiters` to inspect all configured rate limiters in the project.

### `list_rate_limiters`

Returns all named rate limiters registered via `RateLimiter::define()`, including the limiter name, limit values, time windows, and which routes apply the limiter via `Throttle::named()`. Use this to audit rate limiting coverage before adding new API endpoints.
