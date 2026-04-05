# Phase 78: Memory Leak Audit — Research

## Problem Statement

Four unbounded in-memory data structures grow indefinitely in long-running Ferro servers, with no eviction, no size caps, and no background cleanup.

## Identified Leak Vectors

### 1. Framework InMemoryCache — Lazy TTL, No Eviction

**File:** `framework/src/cache/memory.rs:40`
**Structure:** `RwLock<HashMap<String, CacheEntry>>`

- Expired entries are only filtered on read (`is_expired()` check at line 83).
- Entries that are written and never read again persist forever.
- No maximum capacity — HashMap grows without bound.
- `RwLock` means readers block writers and vice versa under contention.

### 2. ferro-cache MemoryStore Tags — Unbounded DashMap

**File:** `ferro-cache/src/stores/memory.rs:14`
**Structure:** `DashMap<String, Vec<String>>`

- Tag-to-key mappings accumulate forever.
- When Moka evicts a cache entry, the tag DashMap retains the stale key reference.
- `tag_flush` cleans a single tag, but there's no periodic cleanup of orphaned entries.
- The inner `Vec<String>` can also grow unbounded per tag (duplicate `tag_add` calls push duplicates).

### 3. ferro-cache MemoryStore Counters — Unbounded DashMap

**File:** `ferro-cache/src/stores/memory.rs:15`
**Structure:** `DashMap<String, i64>`

- Counter keys are never evicted.
- `forget()` removes from both cache and counters, but counters created via `increment/decrement` without a corresponding cache entry are never cleaned.
- No TTL, no capacity limit.

### 4. Metrics Store — 404 Path Explosion

**File:** `framework/src/metrics/mod.rs:83`
**Structure:** `HashMap<String, RouteMetrics>`

- The critical issue is at `middleware/metrics.rs:58`:
  ```rust
  let route_pattern = request.route_pattern().unwrap_or_else(|| path.to_string());
  ```
- Matched routes use the normalized pattern (`/users/{id}`) — bounded by route count.
- **Unmatched routes (404s) fall back to the raw path** — every unique 404 URL creates a new HashMap entry.
- Bot scanners probing `/wp-admin/x`, `/.env`, `/api/v99/random` etc. create unlimited entries.
- No capacity limit, no periodic cleanup, no way to shrink.

## Existing Dependencies

| Crate | Version | Already Used In |
|-------|---------|-----------------|
| `moka` | 0.12 (future feature) | `ferro-cache` MemoryStore main cache |
| `dashmap` | 6 | `ferro-cache` tags + counters |
| `tokio` | 1 (sync, time) | `ferro-cache` |

## Solutions

### Fix 1: Framework InMemoryCache → Replace with Moka sync Cache

**Current:** Hand-rolled `RwLock<HashMap<String, CacheEntry>>` with manual TTL checks.
**Proposed:** Replace with `moka::sync::Cache` which provides:
- Bounded capacity with LRU eviction
- Per-entry TTL via the `Expiry` trait (`expire_after_create`)
- Proactive background eviction (no lazy-only cleanup)
- Lock-free concurrent reads

Add `moka` as a dependency to the `framework` crate (already a workspace dependency via ferro-cache).

**Impact:** Drop-in replacement. Public API (`CacheStore` trait) unchanged.

### Fix 2: ferro-cache Tags → Moka Eviction Listener + Dedup

**Current:** Tags DashMap never cleaned when Moka evicts cache entries.
**Proposed:**
- Register a Moka eviction listener that removes the evicted key from all tag sets.
- Deduplicate `tag_add` — use `HashSet<String>` instead of `Vec<String>` inside tags DashMap.
- Alternative: store tags inside the Moka cache value as metadata, so eviction handles cleanup automatically.

### Fix 3: ferro-cache Counters → Bounded with TTL

**Current:** `DashMap<String, i64>` grows forever.
**Proposed:** Replace with a second `moka::future::Cache<String, i64>` with:
- `max_capacity` matching the main cache (default 10,000)
- Optional TTL (counters rarely need infinite lifetime)
- Eviction follows same policy as main cache

### Fix 4: Metrics → Normalize 404s + Cap Entries

**Current:** Raw paths stored for unmatched routes.
**Proposed:** Two-part fix:
1. **Normalize unmatched routes:** Use a fixed bucket `"UNMATCHED"` for all 404 responses instead of the raw path. This bounds metrics entries to `registered_routes + 1`.
2. **Cap total entries:** Add `max_routes: usize` to MetricsStore (default 1000). Once reached, stop inserting new keys (existing ones still updated). This is a safety net even with normalization.

```rust
// In middleware/metrics.rs line 58:
let route_pattern = request.route_pattern().unwrap_or("UNMATCHED".to_string());
```

## Risk Assessment

| Fix | Risk | Complexity | Impact |
|-----|------|-----------|--------|
| 1. InMemoryCache → Moka | Low (internal impl swap) | Medium | High — eliminates largest leak |
| 2. Tags cleanup | Medium (eviction listener coordination) | Medium | Medium — prevents tag accumulation |
| 3. Counters → Moka | Low (simple replacement) | Low | Low-Medium — counters grow slower |
| 4. Metrics normalization | Very Low (one-line change + cap) | Low | High — eliminates attack vector |

## Recommended Execution Order

1. **Fix 4 first** — one-line change, highest security impact (DoS vector via 404 flooding)
2. **Fix 1** — framework InMemoryCache replacement, biggest memory impact
3. **Fix 3** — counter replacement, straightforward
4. **Fix 2** — tag cleanup, most complex coordination

## Sources

- [Moka per-entry TTL via Expiry trait](https://docs.rs/moka/latest/moka/policy/trait.Expiry.html)
- [Moka future::Cache docs](https://docs.rs/moka/latest/moka/future/struct.Cache.html)
- [Tokio spawn + interval for background tasks](https://docs.rs/tokio/latest/tokio/task/fn.spawn.html)
- [Rust memory leaks in long-running servers](https://fly.io/blog/rust-memory-leak/)
- [Tokio memory management patterns](https://medium.com/@adamszpilewicz/%EF%B8%8F-how-to-avoid-memory-leaks-in-tokio-0aeb9ae2387d)
