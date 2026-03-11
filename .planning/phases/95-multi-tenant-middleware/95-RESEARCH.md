# Phase 95: Multi-Tenant Middleware - Research

**Researched:** 2026-03-11
**Domain:** Rust multi-tenancy, Actix-web/Hyper middleware, SeaORM query scoping, tokio task-local context
**Confidence:** HIGH

## Summary

Multi-tenant middleware in a Rust web framework requires solving three distinct problems: (1) resolving which tenant owns a request, (2) threading that context safely through async middleware and handlers, and (3) scoping database queries to prevent cross-tenant data leakage. Ferro already has well-established patterns for all three from the session, lang, and auth subsystems. The implementation should mirror those patterns exactly.

Ferro's Request type already has a type-map extensions system (`request.insert::<T>()` / `request.get::<T>()`), and the session and lang middlewares demonstrate two proven approaches to per-request context: request extensions (synchronous, one-shot) and `tokio::task_local!` scoped context (async-safe, accessible from anywhere without passing the request). For tenant context, `task_local` is the superior choice because database helper methods (`Model::all()`, `QueryBuilder`) call `DB::connection()` internally and cannot take a request argument. This mirrors exactly how `session()` and `locale()` work today.

SeaORM 1.x (which Ferro uses) does not have automatic global query filters. Tenant scoping must be explicit: the middleware resolves the tenant, stores it in task-local context, and each query must use `TenantScope` (a new scope type built on Ferro's existing `Scope<E>` / `ScopedQuery` pattern). SeaORM 2.0's `RestrictedConnection` is not relevant here — Ferro is on 1.x and that feature is RBAC-oriented, not tenant-filter-oriented.

**Primary recommendation:** Implement `TenantMiddleware` with `tokio::task_local!` tenant context, multiple pluggable resolver strategies, and a `TenantScope` query helper. Follow the session middleware pattern for context threading and the lang middleware pattern for the task-local facade.

## Standard Stack

### Core (no new external dependencies needed)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio (task_local) | 1.x (already in Cargo.toml) | Per-request tenant context storage | Used by session and lang middleware; async-safe across await points |
| sea_orm (already in Cargo.toml) | 1.0 | Tenant-scoped queries via filter() | Already the ORM; no global-filter feature exists in 1.x |
| serde + serde_json (already in Cargo.toml) | 1.x | Tenant struct serialization | Already present |
| jsonwebtoken | 9.x (already used via ApiKey) | JWT claim extraction for tenant resolution | If JWT strategy needed; verify existing dep first |

### Supporting (no new crates needed)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| dashmap (already in Cargo.toml) | 6.x | Tenant lookup cache (slug -> TenantRecord) | Cache tenant DB lookups to avoid per-request queries |
| moka (already in Cargo.toml) | 0.12 | Alternative tenant cache with TTL | If TTL-based eviction is preferred over dashmap |
| regex (already in Cargo.toml) | 1.x | Subdomain extraction from Host header | Already present |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| task_local tenant context | Request extensions only | Extensions require passing Request through every call site; DB helpers call `DB::connection()` directly — they cannot receive a tenant-scoped connection without API changes |
| Ferro's existing Scope/ScopedQuery | Custom macro | Ferro's `Scope<E>` / `ScopedQuery` traits are already the established pattern; extend them, don't replace them |
| Shared schema (tenant_id column) | Schema-per-tenant | Schema-per-tenant requires separate DB connections per tenant — massive complexity. Shared schema is the right v1 choice. |

**No new Cargo dependencies required.** All needed infrastructure (tokio, sea_orm, dashmap/moka, regex) is already in framework/Cargo.toml.

## Architecture Patterns

### Recommended Project Structure

```
framework/src/
├── tenant/
│   ├── mod.rs            # Public API: TenantMiddleware, current_tenant(), TenantContext, resolver traits
│   ├── context.rs        # task_local TENANT_CONTEXT, current_tenant(), tenant_scope()
│   ├── middleware.rs      # TenantMiddleware: Middleware impl, resolver chain, 404/403 on fail
│   ├── resolver.rs       # TenantResolver trait + SubdomainResolver, HeaderResolver, PathResolver, JwtClaimResolver
│   └── scope.rs          # TenantScope<E>: wraps Scope<E> to auto-inject tenant_id filter
```

### Pattern 1: Task-Local Tenant Context (mirrors session/lang)

**What:** `tokio::task_local!` stores the resolved `TenantContext` for the duration of each request. Any code in the call tree can call `current_tenant()` without receiving a request argument.

**When to use:** Always. This is the only approach that works with Ferro's existing `Model::all()`, `QueryBuilder`, and `DB::connection()` helpers, which do not accept a tenant argument.

**Example:**

```rust
// Source: mirrors framework/src/lang/mod.rs and framework/src/session/middleware.rs

// context.rs
tokio::task_local! {
    static TENANT_CONTEXT: Arc<RwLock<Option<TenantContext>>>;
}

/// Get the current request's tenant context.
///
/// Returns None if called outside TenantMiddleware scope, or if the
/// middleware ran but failed to resolve a tenant.
pub fn current_tenant() -> Option<TenantContext> {
    TENANT_CONTEXT
        .try_with(|ctx| ctx.try_read().ok().and_then(|g| g.clone()))
        .ok()
        .flatten()
}

pub(crate) fn tenant_scope() -> Arc<RwLock<Option<TenantContext>>> {
    Arc::new(RwLock::new(None))
}

// middleware.rs
#[async_trait]
impl Middleware for TenantMiddleware {
    async fn handle(&self, request: Request, next: Next) -> Response {
        let resolved = self.resolver.resolve(&request).await;
        let ctx = tenant_scope();
        {
            let mut guard = ctx.write().await;
            *guard = resolved;
        }
        TENANT_CONTEXT
            .scope(ctx, async { next(request).await })
            .await
    }
}
```

### Pattern 2: TenantResolver Trait (pluggable strategy)

**What:** A single trait with multiple implementations. The middleware holds a `Vec<Box<dyn TenantResolver>>` and tries each in order until one succeeds.

**When to use:** Always — resolution strategy is application-specific.

```rust
// resolver.rs
#[async_trait]
pub trait TenantResolver: Send + Sync {
    /// Attempt to resolve tenant from request. Returns None to try next resolver.
    async fn resolve(&self, req: &Request) -> Option<TenantContext>;
}

/// Extracts subdomain from Host header: "acme.yourapp.com" -> "acme"
pub struct SubdomainResolver {
    /// Number of parts in base domain. "yourapp.com" = 2, "sub.yourapp.com" = 3
    pub base_domain_parts: usize,
    pub tenant_lookup: Arc<dyn TenantLookup>,
}

/// Extracts from X-Tenant-ID or X-Tenant-Slug header
pub struct HeaderResolver {
    pub header_name: String, // e.g. "X-Tenant-ID"
    pub tenant_lookup: Arc<dyn TenantLookup>,
}

/// Extracts from route path parameter: /t/{tenant_slug}/...
pub struct PathResolver {
    pub param_name: String, // e.g. "tenant_slug"
    pub tenant_lookup: Arc<dyn TenantLookup>,
}

/// Extracts from a JWT claim (works with existing ApiKeyMiddleware)
pub struct JwtClaimResolver {
    pub claim_field: String, // e.g. "tenant_id"
}
```

### Pattern 3: TenantContext Struct

**What:** A value type stored in task-local context. Clone is required (tokio task_local stores by value on scope entry).

```rust
/// Resolved tenant context available throughout the request.
#[derive(Debug, Clone)]
pub struct TenantContext {
    /// Database row ID
    pub id: i64,
    /// URL-safe identifier (subdomain, slug)
    pub slug: String,
    /// Display name (for logging, UI)
    pub name: String,
}
```

### Pattern 4: TenantScope for Database Queries

**What:** A `Scope<E>` implementation that injects a `tenant_id` filter. Works with Ferro's existing `ScopedQuery` trait.

**When to use:** Every query on tenant-owned tables. This is the only enforcement mechanism in SeaORM 1.x.

```rust
// scope.rs

/// Scope that filters by tenant_id from the current task-local TenantContext.
///
/// Panics with a clear message if called outside TenantMiddleware scope.
pub struct TenantScope<TenantColumn: ColumnTrait>(pub TenantColumn);

impl<E: EntityTrait, TenantColumn: ColumnTrait> Scope<E> for TenantScope<TenantColumn>
where
    E::Model: Send + Sync,
{
    fn apply(self, query: QueryBuilder<E>) -> QueryBuilder<E> {
        let ctx = current_tenant()
            .expect("TenantScope used outside TenantMiddleware scope");
        query.filter(self.0.eq(ctx.id))
    }
}

// Usage in a handler:
// Post::scoped(TenantScope(post::Column::TenantId)).all().await?
```

### Pattern 5: CurrentTenant FromRequest Extractor

**What:** Allows handlers to receive `TenantContext` as a typed parameter via `#[handler]`.

```rust
// In extractor module or tenant/mod.rs
#[async_trait]
impl FromRequest for TenantContext {
    async fn from_request(_req: Request) -> Result<Self, FrameworkError> {
        current_tenant()
            .ok_or_else(|| FrameworkError::domain("No tenant context. Ensure TenantMiddleware is active.", 400))
    }
}

// Handler usage:
#[handler]
pub async fn show(tenant: TenantContext, id: i64) -> Response {
    let post = Post::scoped(TenantScope(post::Column::TenantId)).first_or_fail().await?;
    Ok(json!(post))
}
```

### Anti-Patterns to Avoid

- **Global static tenant**: Using `static CURRENT_TENANT: Mutex<Option<...>>` — it is process-global and will bleed across concurrent requests. Always use `tokio::task_local!`.
- **Trust client-supplied X-Tenant-ID without DB verification**: Always look up the tenant in the database to confirm it exists and is active. Never resolve directly from user input.
- **Forgetting TenantScope on all entity queries**: One unscoped query leaks cross-tenant data. The enforcement must be at query time, not just middleware time.
- **Using thread_local! instead of task_local!**: Thread-local is unsafe in async code because tasks move between threads across await points.
- **Caching without tenant prefix**: Cache keys must include tenant id: `format!("tenant:{}:users:{}", tenant_id, user_id)`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Subdomain extraction | Custom regex | `request.header("host").split('.').collect()` + existing `regex` crate | Host header parsing is 3 lines; no crate needed |
| Tenant cache | Custom HashMap | `dashmap::DashMap` (already in Cargo.toml) | Thread-safe concurrent map; already a dependency |
| JWT claim reading | Custom JWT library | Existing `jsonwebtoken` / API key infrastructure in Ferro | API key system already handles tokens |
| Tenant-scoped DB connection (SeaORM 2.0) | RestrictedConnection wrapper | Don't — Ferro is on SeaORM 1.x; use TenantScope instead | RestrictedConnection is SeaORM 2.0 RBAC, not 1.x |
| Global query filters | Proc macro or monkey-patching | TenantScope on ScopedQuery | SeaORM 1.x has no automatic global filters; explicit is safer anyway |

**Key insight:** Ferro's existing `Scope<E>` / `ScopedQuery` / `QueryBuilder` pattern is already the correct abstraction. `TenantScope` is a zero-boilerplate wrapper around it. The task-local pattern is proven by session + lang middleware.

## Common Pitfalls

### Pitfall 1: thread_local! vs task_local! Confusion
**What goes wrong:** Using `thread_local!` causes tenant context to persist across requests on the same thread, or to be `None` when a task migrates to a different thread at an await point.
**Why it happens:** Session and lang middleware already use `task_local!` for this exact reason, but the pitfall recurs when developers copy the wrong pattern.
**How to avoid:** Use `tokio::task_local!` exclusively. The pattern is already established in `framework/src/session/middleware.rs` and `framework/src/lang/mod.rs`.
**Warning signs:** Tests pass when sequential but fail under concurrent load.

### Pitfall 2: Unscoped Queries in Handler Code
**What goes wrong:** Developer calls `Post::all().await?` in a tenant-protected handler instead of `Post::scoped(TenantScope(...)).all().await?`. All tenants' posts are returned.
**Why it happens:** SeaORM 1.x has no global query filters — nothing prevents unscoped queries at compile time.
**How to avoid:** Lint rule or code review checklist: every query on a tenant-owned model must use `TenantScope`. The `Model::all()` method bypasses tenant scoping — do not use it on tenant-owned entities.
**Warning signs:** Tests that populate multiple tenants' data and verify isolation.

### Pitfall 3: Cross-Tenant Cache Leakage
**What goes wrong:** Cache key `user:{id}` is shared across tenants if the same user ID exists in both.
**Why it happens:** Cache keys forget the tenant prefix.
**How to avoid:** All cache operations on tenant-owned data use keys prefixed with `tenant:{tenant_id}:`. The cache module (`ferro-cache`) should offer a tenant-aware helper.
**Warning signs:** Tenant A can see Tenant B's cached data.

### Pitfall 4: Tenant Resolution Order Matters
**What goes wrong:** A subdomain resolver fires before a JWT resolver, and the subdomain matches a tenant the JWT does not authorize — granting wrong-tenant access.
**Why it happens:** Resolution strategy conflicts when multiple resolvers are chained.
**How to avoid:** Use a single resolver strategy per route group, or verify JWT claim matches resolved tenant. The resolver chain should short-circuit on first success.

### Pitfall 5: Missing Tenant on Unprotected Routes
**What goes wrong:** Health check or public routes accidentally hit `current_tenant()` code paths and panic or return confusing errors.
**Why it happens:** `TenantScope` panics if called outside middleware scope.
**How to avoid:** `current_tenant()` returns `Option<TenantContext>`, not a forced unwrap. `TenantScope` panics — use it only in routes behind `TenantMiddleware`. Public routes should never call `TenantScope`.

### Pitfall 6: Job Dispatch Loses Tenant Context
**What goes wrong:** A handler dispatches a background job from within `TenantMiddleware` scope, but the job runs later in a worker process with no tenant context.
**Why it happens:** `tokio::task_local!` scope ends when the request ends. Jobs are serialized and re-executed in a different async context.
**How to avoid:** Embed `tenant_id` as a field on the job struct (not in task-local). The job reads tenant context from its own fields, not from global context. This is Phase 98's concern, but the data model must anticipate it.

### Pitfall 7: Trusting Unverified Tenant Identifiers
**What goes wrong:** The subdomain or header value is used directly as tenant slug without a database lookup — a non-existent or deactivated tenant gets processed.
**Why it happens:** DB lookup skipped for performance.
**How to avoid:** Always look up the tenant in the DB (or cache with TTL). Return 404 for unknown tenants, 403 for inactive ones.

## Code Examples

### Minimal TenantMiddleware Implementation

```rust
// Source: mirrors framework/src/lang/middleware.rs pattern

use crate::middleware::{Middleware, Next};
use crate::http::Response;
use crate::tenant::context::{tenant_scope, TENANT_CONTEXT};
use crate::Request;
use async_trait::async_trait;

pub struct TenantMiddleware {
    resolvers: Vec<Box<dyn TenantResolver>>,
    on_failure: TenantFailureMode,
}

pub enum TenantFailureMode {
    /// Return 404 if no tenant resolved (for subdomain apps)
    NotFound,
    /// Allow request to continue with no tenant (for mixed public/tenant routes)
    Allow,
}

#[async_trait]
impl Middleware for TenantMiddleware {
    async fn handle(&self, request: Request, next: Next) -> Response {
        let mut resolved = None;
        for resolver in &self.resolvers {
            if let Some(ctx) = resolver.resolve(&request).await {
                resolved = Some(ctx);
                break;
            }
        }

        if resolved.is_none() {
            match self.on_failure {
                TenantFailureMode::NotFound => {
                    return Err(crate::http::HttpResponse::json(
                        serde_json::json!({"error": "Tenant not found"})
                    ).status(404));
                }
                TenantFailureMode::Allow => {}
            }
        }

        let ctx = tenant_scope();
        {
            let mut guard = ctx.write().await;
            *guard = resolved;
        }
        TENANT_CONTEXT
            .scope(ctx, async { next(request).await })
            .await
    }
}
```

### SubdomainResolver Implementation

```rust
// Source: framework/src/http/request.rs header() method

pub struct SubdomainResolver {
    pub base_domain_parts: usize,
    pub tenant_lookup: Arc<dyn TenantLookup>,
}

#[async_trait]
impl TenantResolver for SubdomainResolver {
    async fn resolve(&self, req: &Request) -> Option<TenantContext> {
        let host = req.header("host")?;
        // Strip port if present
        let host = host.split(':').next()?;
        let parts: Vec<&str> = host.split('.').collect();
        if parts.len() <= self.base_domain_parts {
            return None; // No subdomain
        }
        let slug = parts[0];
        self.tenant_lookup.find_by_slug(slug).await
    }
}
```

### HeaderResolver Implementation

```rust
pub struct HeaderResolver {
    pub header_name: String,
    pub tenant_lookup: Arc<dyn TenantLookup>,
}

#[async_trait]
impl TenantResolver for HeaderResolver {
    async fn resolve(&self, req: &Request) -> Option<TenantContext> {
        let value = req.header(&self.header_name)?;
        self.tenant_lookup.find_by_slug(value).await
    }
}
```

### TenantLookup Trait (database abstraction)

```rust
#[async_trait]
pub trait TenantLookup: Send + Sync {
    async fn find_by_slug(&self, slug: &str) -> Option<TenantContext>;
    async fn find_by_id(&self, id: i64) -> Option<TenantContext>;
}

/// Default implementation: queries tenants table + DashMap cache
pub struct DbTenantLookup {
    cache: Arc<dashmap::DashMap<String, TenantContext>>,
}

#[async_trait]
impl TenantLookup for DbTenantLookup {
    async fn find_by_slug(&self, slug: &str) -> Option<TenantContext> {
        if let Some(ctx) = self.cache.get(slug) {
            return Some(ctx.clone());
        }
        // Query tenants table via SeaORM
        let tenant = tenant::Entity::find()
            .filter(tenant::Column::Slug.eq(slug))
            .filter(tenant::Column::Active.eq(true))
            .one(DB::connection().ok()?.inner())
            .await
            .ok()??;
        let ctx = TenantContext { id: tenant.id, slug: tenant.slug.clone(), name: tenant.name };
        self.cache.insert(tenant.slug, ctx.clone());
        Some(ctx)
    }
}
```

### Using TenantScope in a Handler

```rust
// Source: mirrors framework/src/database/model.rs ScopedQuery pattern

// In a handler or service:
use ferro_rs::{tenant::TenantScope, Model, ScopedQuery};

// posts table has tenant_id column
let posts = post::Entity::scoped(TenantScope(post::Column::TenantId))
    .all()
    .await?;

// With additional filters:
let active_posts = post::Entity::scoped(TenantScope(post::Column::TenantId))
    .and(PostScope::Active)
    .all()
    .await?;
```

### Bootstrap Registration

```rust
// In bootstrap.rs — mirrors lang and session middleware registration

use ferro::{global_middleware, TenantMiddleware, SubdomainResolver, DbTenantLookup};
use std::sync::Arc;

pub async fn register() {
    // ... existing DB::init() etc ...

    let lookup = Arc::new(DbTenantLookup::new());
    global_middleware!(
        TenantMiddleware::new()
            .resolver(SubdomainResolver {
                base_domain_parts: 2, // "yourapp.com"
                tenant_lookup: lookup.clone(),
            })
            .on_failure(TenantFailureMode::NotFound)
    );
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| thread_local! for request context | tokio::task_local! | ~2019 (tokio 0.2) | Required for async-safe context; thread_local corrupts data under async |
| Global query filters (EF Core style) | Explicit per-query scope (Rust/SeaORM 1.x) | SeaORM design choice | More verbose but more explicit; no accidental bypass |
| Schema-per-tenant (separate schemas) | Shared schema with tenant_id | Default for new SaaS | Lower ops cost; adequate for most workloads |
| SeaORM RestrictedConnection | Not applicable (SeaORM 2.0 RBAC only) | SeaORM 2.0 (not yet released) | Ferro is on SeaORM 1.0 — use TenantScope instead |

**Deprecated/outdated:**
- SeaORM 2.0 `RestrictedConnection`: RBAC feature, not tenant-filter. Even if SeaORM is upgraded, RestrictedConnection solves a different problem.
- Connection-per-tenant: Opens a separate DB connection per tenant per request — O(tenants) memory, O(1) query safety. Not scalable. Use shared pool + tenant_id column.

## Open Questions

1. **Tenant table location: framework or user app?**
   - What we know: The `TenantLookup` trait abstracts the DB query. The framework can provide the trait and `DbTenantLookup` struct but the actual `tenant` SeaORM entity must live in user's app (since the framework has no migrations).
   - What's unclear: Should the framework provide a CLI command `ferro make:tenant` to scaffold the migration and model?
   - Recommendation: Provide the trait + `DbTenantLookup` struct that expects a generic `TenantModel` conforming to a `HasTenantColumns` marker. Scaffold via `ferro make:tenant` is a nice-to-have, deferred to later.

2. **Cache invalidation for tenant lookup cache**
   - What we know: `DashMap` cache never expires — a deactivated tenant continues to resolve until server restart.
   - What's unclear: Is TTL-based expiry (via `moka`) worth the complexity for v1?
   - Recommendation: Use `moka` with a 5-minute TTL for the default `DbTenantLookup`. Prevents stale-active-tenant attacks without complex invalidation.

3. **Integration with ferro-cache for tenant-keyed caching**
   - What we know: ferro-cache exists but doesn't know about tenants.
   - What's unclear: Should `ferro-cache` get a `tenant_key()` helper that prefixes with `tenant:{id}:`?
   - Recommendation: Document the naming convention in Phase 95; implement the helper in Phase 98 (tenant-aware background jobs).

4. **MCP introspection for tenant middleware**
   - What we know: ferro-mcp introspects routes, middleware, projections.
   - What's unclear: Does the MCP `application_info` tool need updating to surface tenant config?
   - Recommendation: Update `get_global_middleware_info()` display — no structural MCP changes needed for v1.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `tokio::test` |
| Config file | none (workspace-level) |
| Quick run command | `cargo test -p ferro-rs --lib tenant` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MT-01 | SubdomainResolver extracts slug from Host header | unit | `cargo test -p ferro-rs --lib tenant::resolver::tests::subdomain` | No - Wave 0 |
| MT-02 | HeaderResolver extracts from X-Tenant-ID | unit | `cargo test -p ferro-rs --lib tenant::resolver::tests::header` | No - Wave 0 |
| MT-03 | PathResolver extracts from route param | unit | `cargo test -p ferro-rs --lib tenant::resolver::tests::path` | No - Wave 0 |
| MT-04 | TenantMiddleware resolves and stores in task-local | unit | `cargo test -p ferro-rs --lib tenant::middleware::tests` | No - Wave 0 |
| MT-05 | current_tenant() returns None outside middleware scope | unit | `cargo test -p ferro-rs --lib tenant::context::tests::outside_scope` | No - Wave 0 |
| MT-06 | TenantScope applies tenant_id filter to queries | unit | `cargo test -p ferro-rs --lib tenant::scope::tests` | No - Wave 0 |
| MT-07 | TenantContext FromRequest extractor works in handler | unit | `cargo test -p ferro-rs --lib tenant::tests::from_request` | No - Wave 0 |
| MT-08 | Unknown slug returns 404 when on_failure = NotFound | unit | `cargo test -p ferro-rs --lib tenant::middleware::tests::not_found` | No - Wave 0 |
| MT-09 | DbTenantLookup caches resolved tenants | unit | `cargo test -p ferro-rs --lib tenant::lookup::tests::caching` | No - Wave 0 |
| MT-10 | Concurrent requests get isolated tenant contexts | integration | `cargo test -p ferro-rs --test tenant_isolation` | No - Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-rs --lib tenant`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `framework/src/tenant/mod.rs` — module stub
- [ ] `framework/src/tenant/context.rs` — task_local + current_tenant()
- [ ] `framework/src/tenant/resolver.rs` — TenantResolver trait
- [ ] `framework/src/tenant/middleware.rs` — TenantMiddleware
- [ ] `framework/src/tenant/scope.rs` — TenantScope<E>
- [ ] `framework/tests/tenant_isolation.rs` — integration test for concurrent isolation

## Sources

### Primary (HIGH confidence)
- `framework/src/session/middleware.rs` — tokio::task_local! pattern, scope() usage, `SESSION_CONTEXT.scope(ctx, ...)` idiom — directly copied for tenant context
- `framework/src/lang/mod.rs` and `framework/src/lang/middleware.rs` — simpler task_local facade pattern (locale_scope, with_locale_scope)
- `framework/src/database/model.rs` — `Scope<E>`, `ScopedQuery`, `ScopedQueryBuilder` types — TenantScope extends this directly
- `framework/src/http/request.rs` — `request.insert::<T>()` / `request.get::<T>()` type-map extensions
- `framework/src/http/extract.rs` — `FromRequest` trait — TenantContext extractor follows this
- `framework/src/middleware/mod.rs` — `Middleware` trait interface and `Next` type
- OWASP Multi-Tenant Security Cheat Sheet — https://cheatsheetseries.owasp.org/cheatsheets/Multi_Tenant_Security_Cheat_Sheet.html

### Secondary (MEDIUM confidence)
- WorkOS developer guide to SaaS multi-tenant architecture — https://workos.com/blog/developers-guide-saas-multi-tenant-architecture — tenant resolution strategies and pitfalls
- Oneuptime multi-tenant Rust guide (Jan 2026) — https://oneuptime.com/blog/post/2026-01-25-multi-tenant-apis-tenant-isolation-rust/view — Rust-specific patterns
- Adam Chalmers blog on HTTP extensions — https://blog.adamchalmers.com/what-are-extensions/ — type-map pattern explanation
- SeaQL blog: SeaORM 2.0 sneak peek — https://dev.to/seaql/a-sneak-peek-at-seaorm-20-3473 — confirms RestrictedConnection is 2.0-only, RBAC-focused

### Tertiary (LOW confidence)
- GitHub SeaORM discussion #2595 — https://github.com/SeaQL/sea-orm/discussions/2595 — opened May 2025, zero replies, confirms no established SeaORM 1.x multi-tenant pattern

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all needed deps already in Cargo.toml; verified against framework source
- Architecture: HIGH — task_local pattern is established in two existing middleware (session, lang); TenantScope extends existing Scope<E> API
- Pitfalls: HIGH — cross-verified with OWASP, WorkOS guide, and codebase analysis; async context pitfall confirmed by tokio docs
- SeaORM 1.x global filters: HIGH (negative claim) — documented absence of global query filters in 1.x; RestrictedConnection confirmed as 2.0 RBAC feature only

**Research date:** 2026-03-11
**Valid until:** 2026-04-11 (stable domain; SeaORM 2.0 release could shift recommendations if Ferro upgrades)
