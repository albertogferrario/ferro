# Phase 95: Multi-Tenant Middleware - Context

**Gathered:** 2026-03-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Add a `TenantMiddleware` that resolves which tenant owns each request, stores the tenant in task-local context, and provides a `TenantScope` query helper to prevent cross-tenant data leakage. The middleware follows the same patterns as session and lang middleware. Background job tenant awareness is Phase 98. Stripe/billing integration is Phase 96.

</domain>

<decisions>
## Implementation Decisions

### Tenant Resolution Strategy
- Ship all four resolver strategies: Subdomain, Header, Path, JWT
- All resolvers implement a `TenantResolver` trait (pluggable)
- Middleware supports chaining resolvers: `Vec<Box<dyn TenantResolver>>`, tries each in order until one succeeds
- Every resolver validates tenant existence via DB lookup (through `TenantLookup` trait) — never trust raw identifiers
- Inactive/non-existent tenants are treated the same: 404

### Failure Behavior
- `TenantMiddleware` uses a configurable `TenantFailureMode` enum (NotFound 404, Forbidden 403, Allow pass-through)
- Applied per-route-group via `.middleware(TenantMiddleware::new()...)`, not globally
- Inactive tenants return 404 (don't reveal tenant existence)
- Error responses are fixed JSON format: `{"error": "Tenant not found"}` with appropriate status code

### Query Scoping
- `TenantScope` extends existing `Scope<E>` / `ScopedQuery` pattern
- Usage: `Post::scoped(TenantScope(post::Column::TenantId)).all().await?`
- Generic over the column — no assumed column name convention
- Panics with clear message if used outside `TenantMiddleware` scope (programming error)
- `TenantContext` available as handler parameter via `FromRequest` extractor

### TenantContext Shape
- Public fields: `id: i64`, `slug: String`, `name: String`, `plan: {type TBD}`
- Includes a plan/tier field to anticipate Phase 96 (Stripe integration)
- Derives `Clone`, `Debug`, `Serialize`
- Only represents active tenants — inactive tenants are filtered at resolve time and never reach TenantContext

### Claude's Discretion
- Whether to provide a default `DbTenantLookup` implementation or trait-only (leaning toward providing one with moka cache)
- Plan field type: `Option<String>` vs required `String` (architecture decision)
- Cache TTL duration for tenant lookups
- Exact module file structure within `framework/src/tenant/`

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `framework/src/session/middleware.rs`: `tokio::task_local!` + `Arc<RwLock<Option<T>>>` scope pattern — directly reusable for tenant context
- `framework/src/lang/mod.rs`: Simpler task_local facade (`locale_scope()`, `with_locale_scope()`) — reference for public API design
- `framework/src/database/model.rs`: `Scope<E>` trait, `ScopedQuery` trait, `ScopedQueryBuilder` — TenantScope implements `Scope<E>`
- `framework/src/http/extract.rs`: `FromRequest` trait — TenantContext implements this for handler injection
- `framework/src/middleware/mod.rs`: `Middleware` trait, `Next` type, `into_boxed()` — TenantMiddleware implements `Middleware`

### Established Patterns
- Task-local context: `tokio::task_local!` with `Arc<RwLock<Option<T>>>`, scoped via `CONTEXT.scope(ctx, async { ... })` — used by both session and lang
- Builder pattern: consuming `mut self -> Self` methods — used by `MiddlewareRegistry`, `QueryBuilder`, `ServiceDef`
- Scope pattern: `ScopedQuery::scoped(scope).and(other_scope).all()` — chainable, explicit
- Error types: `thiserror` derive, one Error enum per crate
- Global middleware registration: `global_middleware!` macro in bootstrap.rs

### Integration Points
- `framework/src/lib.rs`: Re-export TenantMiddleware, TenantContext, TenantScope, current_tenant, TenantResolver
- `framework/src/middleware/registry.rs`: TenantMiddleware can be registered globally or per-group
- `framework/src/routing/group.rs`: Per-route-group middleware application
- `ferro-mcp/src/tools/`: May need update to surface tenant middleware info in application_info

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches following the established session/lang patterns.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 95-multi-tenant-middleware*
*Context gathered: 2026-03-11*
