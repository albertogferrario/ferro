# ferro-cache

Caching with tags for the Ferro framework.

## Features

- Multiple backends (Redis, in-memory)
- Cache tags for bulk invalidation
- Remember pattern for lazy caching
- TTL (time-to-live) support

## Usage

```rust
use ferro_cache::{Cache, CacheConfig};
use std::time::Duration;

// Create in-memory cache
let cache = Cache::memory();

// Store a value
cache.put("user:1", &user, Duration::from_secs(3600)).await?;

// Get a value
let user: Option<User> = cache.get("user:1").await?;

// Delete a value
cache.forget("user:1").await?;
```

## Remember Pattern

Get from cache or compute and store:

```rust
let users = cache.remember("users:active", Duration::from_secs(3600), || async {
    User::where_active().all().await
}).await?;
```

## Cache Tags

Tags allow bulk invalidation of related entries:

```rust
// Store with tags
cache.tags(&["users", "admins"])
    .put("user:1", &admin, Duration::from_secs(3600))
    .await?;

cache.tags(&["users"])
    .put("user:2", &regular_user, Duration::from_secs(3600))
    .await?;

// Flush all entries tagged with "users"
cache.tags(&["users"]).flush().await?;
// Both user:1 and user:2 are now invalidated
```

## Redis Backend

Enable the `redis-backend` feature:

```toml
[dependencies]
ferro-cache = { version = "0.2", features = ["redis-backend"] }
```

```rust
let cache = Cache::redis("redis://localhost:6379").await?;
```

## License

MIT
