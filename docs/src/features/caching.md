# Caching

Ferro provides a unified caching API with support for multiple backends, cache tags for bulk invalidation, and the convenient "remember" pattern for lazy caching.

## Configuration

### Environment Variables

Configure caching in your `.env` file:

```env
# Cache driver (memory or redis)
CACHE_DRIVER=memory

# Key prefix for all cache entries
CACHE_PREFIX=myapp

# Default TTL in seconds
CACHE_TTL=3600

# Memory store capacity (max entries)
CACHE_MEMORY_CAPACITY=10000

# Redis URL (required if CACHE_DRIVER=redis)
REDIS_URL=redis://127.0.0.1:6379
```

### Bootstrap Setup

In `src/bootstrap.rs`, configure caching:

```rust
use ferro::{App, Cache};
use std::sync::Arc;

pub async fn register() {
    // ... other setup ...

    // Create cache from environment variables
    let cache = Arc::new(Cache::from_env().await?);

    // Store in app state for handlers to access
    App::set_cache(cache);
}
```

### Manual Configuration

```rust
use ferro::{Cache, CacheConfig};
use std::time::Duration;

// In-memory cache with custom config
let config = CacheConfig::new()
    .with_ttl(Duration::from_secs(1800))
    .with_prefix("myapp");

let cache = Cache::memory().with_config(config);

// Redis cache
let cache = Cache::redis("redis://127.0.0.1:6379").await?;
```

## Basic Usage

### Storing Values

```rust
use std::time::Duration;

// Store a value with specific TTL
cache.put("user:1", &user, Duration::from_secs(3600)).await?;

// Store with default TTL
cache.put_default("user:1", &user).await?;

// Store forever (10 years TTL)
cache.forever("config:settings", &settings).await?;
```

### Retrieving Values

```rust
// Get a value
let user: Option<User> = cache.get("user:1").await?;

if let Some(user) = user {
    println!("Found user: {}", user.name);
}

// Check if key exists
if cache.has("user:1").await? {
    println!("User is cached");
}
```

### Removing Values

```rust
// Remove a single key
cache.forget("user:1").await?;

// Remove all cached values
cache.flush().await?;
```

### Pull (Get and Remove)

```rust
// Get value and remove it from cache
let session: Option<Session> = cache.pull("session:abc123").await?;
```

## Remember Pattern

The `remember` pattern retrieves a cached value or computes and caches it if missing:

```rust
use std::time::Duration;

// Get from cache or compute if missing
let users = cache.remember("users:active", Duration::from_secs(3600), || async {
    // This only runs if "users:active" is not in cache
    User::where_active().all().await
}).await?;

// Remember forever
let config = cache.remember_forever("app:config", || async {
    load_config_from_database().await
}).await?;
```

This pattern is excellent for:
- Database query results
- API responses
- Expensive computations
- Configuration that rarely changes

## Cache Tags

Tags allow you to group related cache entries for bulk invalidation.

### Storing with Tags

```rust
use std::time::Duration;

// Store with a single tag
cache.tags(&["users"])
    .put("user:1", &user, Duration::from_secs(3600))
    .await?;

// Store with multiple tags
cache.tags(&["users", "admins"])
    .put("admin:1", &admin, Duration::from_secs(3600))
    .await?;

// Remember with tags
let user = cache.tags(&["users"])
    .remember("user:1", Duration::from_secs(3600), || async {
        User::find(1).await
    })
    .await?;
```

### Flushing Tags

```rust
// Flush all entries tagged with "users"
cache.tags(&["users"]).flush().await?;

// This removes:
// - "user:1" (tagged with ["users"])
// - "admin:1" (tagged with ["users", "admins"])
```

### Tag Use Cases

```rust
// Cache user data
cache.tags(&["users", &format!("user:{}", user.id)])
    .put(&format!("user:{}", user.id), &user, ttl)
    .await?;

// Cache user's posts
cache.tags(&["posts", &format!("user:{}:posts", user.id)])
    .put(&format!("user:{}:posts", user.id), &posts, ttl)
    .await?;

// When user is updated, flush their cache
cache.tags(&[&format!("user:{}", user.id)]).flush().await?;

// When any user data changes, flush all user cache
cache.tags(&["users"]).flush().await?;
```

## Atomic Operations

### Increment and Decrement

```rust
// Increment a counter
let views = cache.increment("page:views", 1).await?;
println!("Page has {} views", views);

// Increment by more than 1
let score = cache.increment("player:score", 100).await?;

// Decrement
let stock = cache.decrement("product:stock", 1).await?;
```

## Cache Backends

### Memory Store

Fast in-memory caching backed by [moka](https://github.com/moka-rs/moka). Best for:
- Single-server deployments
- Development/testing
- Non-critical cache data

```rust
// Default capacity (10,000 entries)
let cache = Cache::memory();

// Custom capacity
let store = MemoryStore::with_capacity(50_000);
let cache = Cache::new(Arc::new(store));
```

The memory store is bounded: when capacity is reached, least-recently-used entries are evicted automatically. Each entry respects its own TTL — expired entries are never returned and are cleaned up proactively by the cache engine. Counters (increment/decrement) share the same capacity bound.

### Redis Store

Distributed caching with Redis. Best for:
- Multi-server deployments
- Persistent cache (survives restarts)
- Shared cache across services

```rust
let cache = Cache::redis("redis://127.0.0.1:6379").await?;

// With authentication
let cache = Cache::redis("redis://:password@127.0.0.1:6379").await?;

// With database selection
let cache = Cache::redis("redis://127.0.0.1:6379/2").await?;
```

Enable the Redis backend in `Cargo.toml`:

```toml
[dependencies]
ferro = { version = "0.1", features = ["redis-backend"] }
```

## Example: API Response Caching

```rust
use ferro::{Request, Response, Cache};
use std::sync::Arc;
use std::time::Duration;

async fn get_products(
    request: Request,
    cache: Arc<Cache>,
) -> Response {
    let category = request.param("category")?;

    // Cache key based on category
    let cache_key = format!("products:category:{}", category);

    // Get from cache or fetch from database
    let products = cache.remember(&cache_key, Duration::from_secs(300), || async {
        Product::where_category(&category).all().await
    }).await?;

    Response::json(&products)
}
```

## Example: User Session Caching

```rust
use ferro::Cache;
use std::sync::Arc;
use std::time::Duration;

async fn cache_user_session(
    cache: Arc<Cache>,
    user_id: i64,
    session: &UserSession,
) -> Result<(), Error> {
    // Cache with user-specific tag for easy invalidation
    cache.tags(&["sessions", &format!("user:{}", user_id)])
        .put(
            &format!("session:{}", session.id),
            session,
            Duration::from_secs(86400), // 24 hours
        )
        .await
}

async fn invalidate_user_sessions(
    cache: Arc<Cache>,
    user_id: i64,
) -> Result<(), Error> {
    // Flush all sessions for this user
    cache.tags(&[&format!("user:{}", user_id)]).flush().await
}
```

## Example: Rate Limiting with Cache

```rust
use ferro::Cache;
use std::sync::Arc;

async fn check_rate_limit(
    cache: Arc<Cache>,
    user_id: i64,
    limit: i64,
) -> Result<bool, Error> {
    let key = format!("rate_limit:user:{}", user_id);

    // Increment the counter
    let count = cache.increment(&key, 1).await?;

    // Set TTL on first request (1 minute window)
    if count == 1 {
        // Note: For production, use Redis SETEX or similar
        // This is a simplified example
    }

    Ok(count <= limit)
}
```

## Environment Variables Reference

| Variable | Description | Default |
|----------|-------------|---------|
| `CACHE_DRIVER` | Cache backend ("memory" or "redis") | `memory` |
| `CACHE_PREFIX` | Key prefix for all entries | - |
| `CACHE_TTL` | Default TTL in seconds | `3600` |
| `CACHE_MEMORY_CAPACITY` | Max entries for memory store | `10000` |
| `REDIS_URL` | Redis connection URL | `redis://127.0.0.1:6379` |

## Best Practices

1. **Use meaningful cache keys** - `user:123:profile` not `key1`
2. **Set appropriate TTLs** - Balance freshness vs performance
3. **Use tags for related data** - Makes invalidation easier
4. **Cache at the right level** - Cache complete objects, not fragments
5. **Handle cache misses gracefully** - Always have a fallback
6. **Use remember pattern** - Cleaner code, less boilerplate
7. **Prefix keys in production** - Avoid collisions between environments
8. **Monitor cache hit rates** - Identify optimization opportunities
