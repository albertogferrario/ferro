---
phase: 43-rate-limiting
plan: 01
subsystem: middleware
tags: [rate-limiting, cache, middleware, throttle, fixed-window]

requires:
  - phase: 38
    provides: Cache facade with InMemoryCache and RedisCache

provides:
  - CacheStore::expire() method for TTL on existing keys
  - Cache::expire() facade method
  - Limit struct with declarative rate limit configuration
  - RateLimiter::define() for named limiter registration
  - Throttle middleware implementing Middleware directly
  - Cache-backed fixed-window counter rate limiting

affects: [43-rate-limiting, 46-mcp-cli-updates]

tech-stack:
  added: []
  patterns:
    - "OnceLock + DashMap for global named limiter registry"
    - "Fixed-window counter with Cache INCR + EXPIRE"
    - "Fail-open rate limiting when cache unavailable"
    - "LimiterResponse enum for single/multiple limit returns"

key-files:
  created: []
  modified:
    - framework/src/cache/store.rs
    - framework/src/cache/memory.rs
    - framework/src/cache/redis.rs
    - framework/src/cache/mod.rs
    - framework/src/middleware/rate_limit.rs
    - framework/src/middleware/mod.rs
    - framework/src/lib.rs

key-decisions:
  - "eprintln! for warnings (consistent with framework pattern, no tracing dependency)"
  - "OnceLock<DashMap> for limiter registry (static, thread-safe, no initialization order dependency)"
  - "Fail-open on cache errors and missing named limiters (availability over strictness)"

patterns-established:
  - "Limit::per_minute(60).by(key) for declarative rate limits"
  - "RateLimiter::define(name, |req| ...) for named limiters"
  - "Throttle::named(name) / Throttle::per_minute(n) for middleware"

duration: 6min
completed: 2026-02-10
---

# Phase 43 Plan 01: Cache-Backed Rate Limiting Summary

**Refactored rate limiting from in-memory DashMap to cache-backed fixed-window counters with Laravel-style RateLimiter::define()/Throttle::named() API**

## Performance

- **Duration:** 6 min
- **Started:** 2026-02-10T05:45:27Z
- **Completed:** 2026-02-10T05:51:33Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Added `CacheStore::expire()` method to set TTL on existing cache keys, enabling the INCR+EXPIRE rate limiting pattern
- Replaced entire in-memory DashMap rate limiting with cache-backed storage using `Cache::increment()` and `Cache::expire()`
- Implemented Laravel-style `RateLimiter::define()` with closure-based dynamic limits and `Throttle::named()` for route middleware
- `Throttle` now implements `Middleware` directly (no more `into_middleware()`)
- Multiple limits per named limiter supported (first exceeded triggers 429)
- Rate limit headers added to both Ok and Err responses
- Fail-open behavior when cache unavailable or named limiter not registered

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix InMemoryCache increment TTL and add CacheStore::expire()** - `4b35e82` (feat)
2. **Task 2: Refactor rate_limit.rs with Limit, define/named, cache-backed storage** - `a82a979` (feat)

## Files Created/Modified

- `framework/src/cache/store.rs` - Added `expire()` to CacheStore trait
- `framework/src/cache/memory.rs` - InMemoryCache expire() implementation
- `framework/src/cache/redis.rs` - RedisCache expire() implementation using EXPIRE command
- `framework/src/cache/mod.rs` - Cache::expire() facade method
- `framework/src/middleware/rate_limit.rs` - Complete rewrite with Limit, RateLimiter, Throttle
- `framework/src/middleware/mod.rs` - Updated exports (Limit, LimiterResponse, RateLimiter, Throttle)
- `framework/src/lib.rs` - Updated re-exports for new types

## Decisions Made

- Used `eprintln!` for rate limiter warnings instead of `tracing::warn!` to match existing framework logging pattern
- Used `OnceLock<DashMap>` for the global limiter registry, consistent with the framework's global registry pattern
- Fail-open on all error paths: cache unavailable, named limiter not found, expire failure

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Rate limiting core implementation complete, ready for tests in Plan 02
- All old types removed; exports updated
- `Throttle::named("api")` compiles as middleware argument
- `RateLimiter::define("api", |req| Limit::per_minute(60))` compiles

---
*Phase: 43-rate-limiting*
*Completed: 2026-02-10*
