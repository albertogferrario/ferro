# Phase 43: Rate Limiting - Research

**Researched:** 2026-02-10
**Domain:** Rate limiting middleware with cache-backed storage (Laravel-inspired)
**Confidence:** HIGH

<research_summary>
## Summary

Ferro already has a functional rate limiting middleware (`RateLimiter`, `Throttle`, `RateLimiters`) with in-memory storage using `DashMap`/`AtomicU64`. The existing implementation covers per-second/minute/hour/day windows, custom key resolvers, response headers (X-RateLimit-*), and named rate limiter collections.

Phase 43's focus should be on **upgrading the existing implementation** rather than building from scratch. The key gaps are: (1) no cache/Redis backend — limits reset on restart and don't work across multiple servers, (2) no Laravel-style `RateLimiter::for()` declarative configuration with closure-based dynamic limits, (3) no integration with the auth system for user-based segmentation, (4) the `Throttle` type doesn't implement `Middleware` directly, and (5) no sliding window support (current implementation uses fixed window with stale-window detection).

**Primary recommendation:** Add a `CacheStore`-backed rate limiting backend that uses ferro-cache's existing `increment()` + TTL primitives. Adopt Laravel's `RateLimiter::for("name", |req| Limit::per_minute(60).by(user_or_ip))` pattern for declarative named limiters. Keep the in-memory backend as the default (no Redis required), with automatic Redis upgrade when `CACHE_DRIVER=redis`.
</research_summary>

<standard_stack>
## Standard Stack

### What Already Exists in Ferro

| Component | Location | Status |
|-----------|----------|--------|
| `RateLimiter` middleware | `framework/src/middleware/rate_limit.rs` | Working, in-memory only |
| `RateLimitConfig` | Same file | Basic: max_requests, window_seconds, key_prefix |
| `RateLimitStore` | Same file | In-memory `DashMap<String, Arc<RateLimitEntry>>` |
| `RateLimiters` (named collection) | Same file | Static defaults: api/authenticated/sensitive/auth |
| `Throttle` builder | Same file | Builder pattern → `into_middleware()` conversion |
| `ferro-cache` increment/decrement | `ferro-cache/src/cache.rs` | Atomic counters on both Memory and Redis backends |
| `ferro-cache` TTL | Same | Per-entry TTL on both backends |
| `ferro-cache` tags | `ferro-cache/src/tagged.rs` | Could be used for bulk limit resets |

### What Needs to Be Added

| Component | Purpose | Laravel Equivalent |
|-----------|---------|-------------------|
| Cache-backed store | Distributed rate limiting | Laravel uses Cache facade internally |
| `Limit` struct | Declarative limit configuration | `Illuminate\Cache\RateLimiting\Limit` |
| `RateLimiter::for()` | Named limiter registration with closures | `RateLimiter::for('api', fn)` |
| `throttle:name` middleware | Apply named limiter to routes | `throttle:api` middleware |
| User-based segmentation | `.by(user_id)` on limits | `Limit::perMinute(60)->by($user->id)` |
| Custom 429 response | `.response()` on limits | `Limit::perMinute(60)->response(fn)` |
| Multiple limits per limiter | Array of limits on one name | `return [Limit::perMinute(500), Limit::perMinute(3)->by($email)]` |

### No External Crates Needed

The existing ferro-cache primitives (`increment`, TTL, Redis/Memory backends) provide everything needed for production rate limiting. No need for `governor`, `tower-governor`, `brakes`, or other external rate limiting crates.

**Rationale:**
- `governor` uses GCRA (token bucket variant) — more complex than needed, different algorithm than Laravel
- Ferro's pattern is Laravel-inspired, which uses simple fixed/sliding window counters with cache backends
- ferro-cache already provides atomic `increment()` on both Memory and Redis
- Adding a dependency for something that's ~200 lines of code with existing primitives is unnecessary
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Pattern 1: Cache-Backed Rate Limit Store

Replace in-memory `DashMap` with ferro-cache for distributed support.

**Algorithm: Fixed Window Counter (matches current behavior + Laravel)**

```
Key format: "rate_limit:{prefix}:{identifier}:{window_number}"
Window number: current_timestamp / window_seconds (floored)

Check request:
1. key = format!("rate_limit:{}:{}:{}", prefix, identifier, now / window_secs)
2. count = cache.increment(key, 1)  // Atomic, returns new count
3. If count == 1: set TTL on key to window_seconds (first request in window)
4. If count > max: return 429 with Retry-After header
5. Else: pass through with X-RateLimit-* headers
```

**Why fixed window (not sliding window):**
- Laravel uses fixed window counter by default
- Simpler, faster, lower memory
- Boundary bursts are acceptable for most use cases
- ferro-cache's `increment()` maps directly to this pattern
- Redis `INCR` + `EXPIRE` is the standard production pattern

### Pattern 2: Laravel-Style Declarative Named Limiters

```rust
// In bootstrap.rs or a service provider
RateLimiter::define("api", |req: &Request| {
    let user_id = Auth::id(req);
    match user_id {
        Some(id) => Limit::per_minute(120).by(format!("user:{}", id)),
        None => Limit::per_minute(60).by(req.ip()),
    }
});

// In routes.rs - apply by name
get!("/api/users", controllers::users::index).middleware(Throttle::named("api"))
```

Key design elements:
- `RateLimiter::define()` stores closures in a global registry (like middleware registry)
- `Limit` struct carries: max_requests, window_seconds, key (from `.by()`), custom response
- `Throttle::named("api")` looks up the named limiter at request time
- Closure evaluated per-request, enabling dynamic limits based on auth state

### Pattern 3: Multiple Limits Per Limiter

```rust
RateLimiter::define("login", |req: &Request| {
    vec![
        Limit::per_minute(500),                          // Global: 500/min total
        Limit::per_minute(5).by(req.input_or("email", req.ip())),  // Per-email: 5/min
    ]
});
```

All limits checked; first one exceeded triggers 429.

### Recommended Module Structure

```
framework/src/middleware/
├── mod.rs              # Existing: exports
├── rate_limit.rs       # Refactored: Limit, RateLimiter::define(), cache-backed store
├── chain.rs            # Existing: unchanged
├── metrics.rs          # Existing: unchanged
└── registry.rs         # Existing: unchanged
```

No new files needed — rate_limit.rs gets refactored to add the new API while keeping backward compatibility with existing `RateLimiter::per_minute(60)` usage.

### Anti-Patterns to Avoid

- **Don't use Lua scripts for Redis rate limiting:** ferro-cache's `increment()` + TTL is sufficient for fixed window. Lua scripts add complexity for sliding window/token bucket, which aren't needed.
- **Don't create a separate store trait:** Use ferro-cache's `CacheStore` directly. Adding another abstraction layer is unnecessary since ferro-cache already abstracts Memory/Redis.
- **Don't make Redis required:** In-memory must remain the default. Many Ferro apps are single-server and don't need Redis.
- **Don't break existing API:** `RateLimiter::per_minute(60)` must continue to work. The new `RateLimiter::define()` / `Throttle::named()` API is additive.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Atomic counter | Custom AtomicU64 + DashMap | `ferro_cache::Cache::increment()` | Already abstracts Memory + Redis, handles initialization |
| Key expiration | Manual `Instant::now()` window tracking | Cache TTL (auto-expire) | Redis EXPIRE is atomic; moka handles TTL |
| Distributed locking | Mutex/RwLock for concurrent access | Redis INCR atomicity | Redis INCR is atomic by design, no locks needed |
| Cleanup of expired entries | Background task / retain() | Cache TTL eviction | Both moka (LRU) and Redis (EXPIRE) handle this |
| IP extraction | Custom header parsing | Existing `get_key()` logic | Already handles X-Forwarded-For, X-Real-IP |

**Key insight:** The existing in-memory `RateLimitStore` with `DashMap<String, Arc<RateLimitEntry>>` reimplements what ferro-cache already provides. By switching to ferro-cache as the backend, we get Redis support, TTL eviction, and atomic counters for free, while deleting the custom `RateLimitEntry` and `RateLimitStore` types.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Fixed Window Boundary Burst
**What goes wrong:** Client sends max requests at second 59 of window 1 and max requests at second 0 of window 2, effectively getting 2x the limit in 2 seconds.
**Why it happens:** Fixed window counter resets at window boundaries.
**How to avoid:** Accept this tradeoff (Laravel does). For most APIs, 2x burst at boundaries is acceptable. Document it. If strict enforcement is needed in the future, sliding window counter (weighted average of current + previous window) can be added later.
**Warning signs:** Abuse reports showing double the expected traffic volume in short bursts.

### Pitfall 2: Race Condition on First Request in Window
**What goes wrong:** Two concurrent first-requests both get count=1 from INCR but only one sets EXPIRE, or the TTL gets set incorrectly.
**Why it happens:** INCR and EXPIRE are separate Redis commands; not atomic together.
**How to avoid:** Use Redis pipeline or check: if INCR returns 1, set EXPIRE. Since INCR is atomic and only one request will see count=1, only one EXPIRE is set. This is the standard Redis rate limiting pattern.
**Warning signs:** Keys without TTL accumulating in Redis (memory leak).

### Pitfall 3: Cache Backend Not Initialized
**What goes wrong:** Rate limiter tries to use Cache before bootstrap completes.
**How to avoid:** Rate limiter should gracefully fall back to allowing the request if cache is unavailable, with a warning log. Never block requests due to rate limiter infrastructure failure.
**Warning signs:** 500 errors on startup or after Redis disconnect.

### Pitfall 4: Key Cardinality Explosion
**What goes wrong:** Using fine-grained keys (e.g., per-endpoint-per-user) with long windows creates millions of cache entries.
**Why it happens:** Each unique key is a separate cache entry.
**How to avoid:** Use reasonable key granularity. Default to IP-based. User-based only for authenticated routes. Keep windows short (minutes, not days) unless specifically needed.
**Warning signs:** Redis memory growing unboundedly, moka cache at max capacity evicting non-rate-limit entries.

### Pitfall 5: Rate Limit Headers on Error Responses
**What goes wrong:** Rate limit headers not added to error responses (only added to Ok responses).
**How to avoid:** Add X-RateLimit-* headers to both Ok and Err responses. The current implementation only adds headers to Ok responses — this should be fixed.
**Warning signs:** Clients can't track their rate limit usage when receiving error responses.
</common_pitfalls>

<code_examples>
## Code Examples

### Cache-Backed Rate Limit Check (Core Algorithm)

```rust
// Core rate limiting logic using ferro-cache
async fn check_rate_limit(
    cache: &ferro_cache::Cache,
    key: &str,
    max_requests: u32,
    window_seconds: u64,
) -> RateLimitResult {
    // Window-based key: "rate_limit:api:192.168.1.1:28431"
    // where 28431 = unix_timestamp / window_seconds
    let window = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() / window_seconds;
    let window_key = format!("{}:{}", key, window);

    // Atomic increment — returns new count
    let count = match cache.increment(&window_key, 1).await {
        Ok(count) => count as u32,
        Err(_) => return RateLimitResult::Allowed { remaining: max_requests, limit: max_requests, retry_after: window_seconds },
    };

    // Set TTL on first request in window (count == 1)
    if count == 1 {
        let _ = cache.put(&window_key, &1i64, Duration::from_secs(window_seconds + 1)).await;
    }

    let remaining = max_requests.saturating_sub(count);
    let retry_after = window_seconds - (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() % window_seconds);

    if count > max_requests {
        RateLimitResult::Exceeded { retry_after, limit: max_requests }
    } else {
        RateLimitResult::Allowed { remaining, limit: max_requests, retry_after }
    }
}
```

### Laravel-Style Limit Struct

```rust
pub struct Limit {
    pub max_requests: u32,
    pub window_seconds: u64,
    pub key: Option<String>,           // from .by()
    pub response: Option<ResponseFn>,  // custom 429 response
}

impl Limit {
    pub fn per_second(max: u32) -> Self { Self { max_requests: max, window_seconds: 1, key: None, response: None } }
    pub fn per_minute(max: u32) -> Self { Self { max_requests: max, window_seconds: 60, key: None, response: None } }
    pub fn per_hour(max: u32) -> Self { Self { max_requests: max, window_seconds: 3600, key: None, response: None } }
    pub fn per_day(max: u32) -> Self { Self { max_requests: max, window_seconds: 86400, key: None, response: None } }

    pub fn by(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn response<F>(mut self, f: F) -> Self
    where
        F: Fn(&Request, RateLimitHeaders) -> HttpResponse + Send + Sync + 'static,
    {
        self.response = Some(Arc::new(f));
        self
    }
}
```

### Named Limiter Registration

```rust
// In bootstrap.rs
use ferro::middleware::{RateLimiter, Limit};

pub async fn register() {
    // Named rate limiter with dynamic limit based on auth
    RateLimiter::define("api", |req: &Request| {
        match Auth::id(req) {
            Some(id) => Limit::per_minute(120).by(format!("user:{}", id)),
            None => Limit::per_minute(60).by(req.ip()),
        }
    });

    // Strict limit for auth endpoints
    RateLimiter::define("auth", |req: &Request| {
        Limit::per_minute(5).by(req.ip())
    });
}

// In routes.rs
routes! {
    group!("/api", {
        get!("/users", controllers::users::index),
    }).middleware(Throttle::named("api")),

    group!("/auth", {
        post!("/login", controllers::auth::login),
        post!("/register", controllers::auth::register),
    }).middleware(Throttle::named("auth")),
}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| In-memory only rate limiting | Cache-backed (Redis/Memory) | Standard practice | Enables multi-server deployments |
| Per-instance DashMap stores | Shared cache backend | Standard practice | Consistent limits across instances |
| Static rate limit configs | Dynamic closure-based limits | Laravel 8+ (2020) | Per-user, per-tier adaptive limits |

**What hasn't changed:**
- Fixed window counter remains the standard for most web applications
- X-RateLimit-Limit/Remaining/Reset headers are the standard response headers
- 429 Too Many Requests with Retry-After is the standard error response
- IP-based default key with user-based segmentation for authenticated routes

**What Ferro should NOT adopt:**
- Token bucket / leaky bucket: Over-engineering for web API rate limiting. These are designed for network traffic shaping, not HTTP request throttling.
- Sliding window log (sorted sets): O(n) memory per client. Only needed for exact enforcement requirements.
- `governor` crate / GCRA: Different paradigm than Laravel's approach. Would create inconsistency with Ferro's Laravel-inspired patterns.
</sota_updates>

<open_questions>
## Open Questions

1. **Should the `with_decay` flag on `Throttle` be implemented?**
   - What we know: `Throttle.with_decay` field exists but `into_middleware()` ignores it
   - What's unclear: Whether decay (sliding window counter) is worth implementing now
   - Recommendation: Remove the dead `with_decay` field. Add sliding window as a future enhancement if requested.

2. **Should `RateLimitStore` be kept for backward compatibility?**
   - What we know: Current `RateLimitStore` uses DashMap + AtomicU64
   - What's unclear: Whether any external code depends on `RateLimitStore` directly
   - Recommendation: Remove it. `RateLimitStore` is not a commonly used public API — it's an implementation detail. Replace with cache-backed implementation.

3. **How to handle cache TTL for the `increment` + TTL pattern?**
   - What we know: ferro-cache's `increment()` on MemoryStore uses DashMap counters separately from the TTL-managed moka cache
   - What's unclear: Whether MemoryStore's `increment()` entries auto-expire or persist forever
   - Recommendation: Verify MemoryStore increment behavior during implementation. May need to add TTL support to the counter storage if it doesn't expire.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- Ferro codebase: `framework/src/middleware/rate_limit.rs` — existing implementation, 598 lines
- Ferro codebase: `ferro-cache/src/cache.rs` — Cache trait with increment/decrement + TTL
- Ferro codebase: `ferro-cache/src/stores/redis.rs` — Redis INCR implementation
- Ferro codebase: `ferro-cache/src/stores/memory.rs` — DashMap-based counters
- Laravel 12.x docs: Rate limiting — `RateLimiter::for()`, `Limit` class, `throttle:name` middleware
- Redis.io: Rate limiting patterns — Fixed window, sliding window, token bucket with Redis commands

### Secondary (MEDIUM confidence)
- Governor crate docs — Confirmed GCRA algorithm is not the right fit for Laravel-inspired patterns
- Tower-governor — Confirmed Axum/Tower integration exists but is unnecessary for Ferro's custom middleware system

### Tertiary (LOW confidence - needs validation)
- MemoryStore increment TTL behavior — Needs verification during implementation (Open Question #3)
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Ferro middleware system + ferro-cache
- Ecosystem: Rust rate limiting crates (governor, brakes, limitr) — evaluated and rejected
- Patterns: Laravel RateLimiter::for(), fixed window counter, cache-backed storage
- Pitfalls: Window boundary bursts, race conditions, cache initialization, key cardinality

**Confidence breakdown:**
- Standard stack: HIGH — existing codebase fully explored, no external deps needed
- Architecture: HIGH — Laravel patterns well-documented, ferro-cache primitives verified
- Pitfalls: HIGH — standard rate limiting pitfalls well-known in literature
- Code examples: MEDIUM — examples are design proposals, not tested code

**Research date:** 2026-02-10
**Valid until:** 2026-03-10 (30 days — rate limiting patterns are stable)
</metadata>

---

*Phase: 43-rate-limiting*
*Research completed: 2026-02-10*
*Ready for planning: yes*
