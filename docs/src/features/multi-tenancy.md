# Multi-Tenancy

Ferro provides built-in support for multi-tenant applications using a shared-schema approach where every tenant shares the same database schema. Each tenant's data is isolated by a `tenant_id` foreign key column, enforced at the query layer via `TenantScope`.

The middleware resolves the current tenant once per request, stores it in task-local context, and makes it available to handlers and query scopes without requiring explicit parameter passing.

## Overview

The multi-tenancy system has three components:

1. **`TenantMiddleware`** — resolves the tenant from the incoming request using a pluggable resolver strategy.
2. **`TenantContext`** — the resolved tenant data (id, slug, name, plan), available as a handler parameter via `FromRequest`.
3. **`TenantScope`** — a query scope that filters database queries by the current tenant's ID.

## Quick Start

Add `TenantMiddleware` to your application in `bootstrap.rs`:

```rust
use ferro::{
    global_middleware, TenantMiddleware, SubdomainResolver, DbTenantLookup,
    TenantFailureMode,
};
use std::sync::Arc;

pub fn register() {
    let lookup = Arc::new(DbTenantLookup::new(
        // Find tenant ID by slug
        |slug: String| async move {
            tenant::Entity::find()
                .filter(tenant::Column::Slug.eq(&slug))
                .one(DB::get())
                .await
                .ok()
                .flatten()
                .map(|t| t.id)
        },
        // Find full tenant by ID
        |id: i64| async move {
            tenant::Entity::find_by_id(id)
                .one(DB::get())
                .await
                .ok()
                .flatten()
                .map(|t| TenantContext {
                    id: t.id,
                    slug: t.slug,
                    name: t.name,
                    plan: t.plan,
                })
        },
    ));

    global_middleware!(
        TenantMiddleware::new()
            .resolver(SubdomainResolver {
                base_domain_parts: 2,
                tenant_lookup: lookup,
            })
            .on_failure(TenantFailureMode::NotFound)
    );
}
```

This configuration resolves the tenant from the request subdomain (e.g., `acme.yourapp.com` resolves to tenant slug `acme`) and returns 404 if no tenant is found.

## Resolver Strategies

Four built-in resolvers cover the most common patterns. Multiple resolvers can be chained — the first one that returns `Some` wins.

### Subdomain Resolver

Extracts the tenant slug from the request subdomain.

```rust
SubdomainResolver {
    base_domain_parts: 2, // strips last 2 parts: yourapp.com
    tenant_lookup: lookup,
}
```

- **When to use:** SaaS applications with per-tenant subdomains (`acme.yourapp.com`).
- `base_domain_parts: 2` means `yourapp.com` is the base — the first segment before it is the slug.
- For `app.yourapp.com` (3 parts), use `base_domain_parts: 2` and `app.` is the slug.

### Header Resolver

Extracts the tenant slug from a custom HTTP header.

```rust
HeaderResolver {
    header_name: "X-Tenant-ID".to_string(),
    tenant_lookup: lookup,
}
```

- **When to use:** API clients, internal services, or mobile apps where the client explicitly sends the tenant identifier.

### Path Resolver

Extracts the tenant slug from a route parameter.

```rust
PathResolver {
    param_name: "tenant".to_string(),
    tenant_lookup: lookup,
}
```

Route: `/t/:tenant/dashboard`

- **When to use:** Applications where the tenant is embedded in the URL path (e.g., `/t/acme/dashboard`).

### JWT Claim Resolver

Extracts the tenant slug from a JWT claim stored in request extensions.

```rust
JwtClaimResolver {
    claim_field: "tenant_id".to_string(),
    tenant_lookup: lookup,
}
```

- **When to use:** Stateless APIs where an upstream JWT middleware has already parsed the token and stored claims in the request extensions as `serde_json::Value`.
- Requires upstream middleware to insert parsed JWT claims into `req.extensions`.

### Chaining Resolvers

Multiple resolvers are tried in order — the first to return `Some` is used:

```rust
TenantMiddleware::new()
    .resolver(HeaderResolver { header_name: "X-Tenant".to_string(), tenant_lookup: lookup.clone() })
    .resolver(SubdomainResolver { base_domain_parts: 2, tenant_lookup: lookup })
    .on_failure(TenantFailureMode::Allow)
```

## Handler Extraction

`TenantContext` implements `FromRequest` and can be used as a handler parameter directly:

```rust
use ferro::{handler, TenantContext, Response, json};

#[handler]
pub async fn dashboard(tenant: TenantContext) -> Response {
    Ok(json!({
        "tenant": tenant.name,
        "plan": tenant.plan,
    }))
}
```

The handler will receive a 400 error if no tenant context is available (i.e., the route is not behind `TenantMiddleware`). Use this to enforce that handlers always have a tenant.

You can also read the tenant anywhere in a call chain using `current_tenant()`:

```rust
use ferro::current_tenant;

pub async fn some_service() -> Option<String> {
    current_tenant().map(|t| t.slug)
}
```

`current_tenant()` returns `None` outside a `TenantMiddleware` scope.

## Query Scoping

Use `TenantScope` to filter queries by the current tenant's ID. This prevents data from leaking between tenants.

```rust
use ferro::{TenantScope, ScopedQuery};

// Fetch all posts belonging to the current tenant
let posts = post::Entity::scoped(TenantScope(post::Column::TenantId))
    .all()
    .await?;
```

`TenantScope(column)` implements `Scope<E>` — it reads the current tenant's ID from task-local context and applies `.filter(column.eq(tenant_id))` to the query.

### Panic Behavior

`TenantScope` panics with a clear message if called outside a `TenantMiddleware` scope:

```
TenantScope used outside TenantMiddleware scope — ensure this route is behind TenantMiddleware
```

This is intentional. Using `TenantScope` without middleware is a programming error, not a runtime condition. The panic surfaces the mistake immediately in development rather than silently returning unscoped data.

## Failure Modes

`TenantFailureMode` controls what happens when the resolver cannot find a tenant:

| Variant     | HTTP Status | When to use                                               |
|-------------|-------------|-----------------------------------------------------------|
| `NotFound`  | 404         | Standard SaaS apps — unknown tenants don't exist          |
| `Forbidden` | 403         | Apps where tenants exist but some are blocked             |
| `Allow`     | — (passes through) | Public routes that work with or without a tenant   |

```rust
TenantMiddleware::new()
    .resolver(SubdomainResolver { ... })
    .on_failure(TenantFailureMode::Allow) // allow public pages without tenant
```

With `Allow`, `current_tenant()` returns `None` for unresolved requests. Handlers that require a tenant should use `TenantContext` as a parameter — which returns a 400 error — rather than calling `current_tenant()` directly.

## Custom TenantLookup

`DbTenantLookup` covers the most common case: a `tenants` table with `slug` and `id` columns. For custom schemas, implement `TenantLookup` directly:

```rust
use ferro::{TenantLookup, TenantContext};
use async_trait::async_trait;

pub struct MyCustomLookup;

#[async_trait]
impl TenantLookup for MyCustomLookup {
    async fn find_by_slug(&self, slug: &str) -> Option<TenantContext> {
        // Custom lookup logic — read from DB, cache, config, etc.
        todo!()
    }
}
```

The `TenantLookup` trait is object-safe (`Arc<dyn TenantLookup>`), so resolvers can share a single lookup instance across threads.

## Background Jobs

Jobs dispatched from tenant-scoped handlers automatically carry the current tenant's ID in the payload. When a worker processes the job, it restores the full `TenantContext` before execution — so `current_tenant()` works inside job handlers the same way it does in HTTP handlers.

### Setup

Register the capture hook and configure the worker with a tenant scope provider during bootstrap:

```rust
use ferro::{
    Worker, WorkerConfig, Queue, register_tenant_capture_hook,
    current_tenant,
    tenant::{DbTenantLookup, FrameworkTenantScopeProvider},
};
use std::sync::Arc;

// Register once at startup. Called at dispatch time to capture the current tenant ID.
register_tenant_capture_hook(|| current_tenant().map(|t| t.id));

// Build the lookup (same instance used by TenantMiddleware).
let lookup = Arc::new(DbTenantLookup::new(
    |slug| Box::pin(async move { /* find by slug */ None }),
    |id| Box::pin(async move { /* find by id */ None }),
));

// Attach the scope provider to the worker.
let worker = Worker::new(Queue::connection(), WorkerConfig::default())
    .with_tenant_scope(Arc::new(FrameworkTenantScopeProvider::new(lookup)));
```

`register_tenant_capture_hook` is a no-op if called more than once — only the first registration takes effect.

### Using current_tenant() in Jobs

Job handlers can call `current_tenant()` directly, without any extra setup:

```rust
use ferro::{Job, Error, current_tenant};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SendWelcomeEmail;

#[async_trait::async_trait]
impl Job for SendWelcomeEmail {
    async fn handle(&self) -> Result<(), Error> {
        let tenant = current_tenant().expect("job must run inside tenant scope");
        // send email for tenant.name ...
        Ok(())
    }
}
```

### Dispatching on Behalf of a Tenant

In admin or system contexts where no ambient tenant scope exists (CLI commands, webhooks, scheduled tasks), use `.for_tenant(id)` to explicitly attach a tenant ID to the job:

```rust
use ferro::queue_dispatch;

// Runs the job in the scope of tenant 42, regardless of current context.
queue_dispatch(SendWelcomeEmail)
    .for_tenant(42)
    .dispatch()
    .await?;
```

### Behavior When Tenant Not Found

If the worker cannot resolve the tenant ID stored in the payload (tenant deleted, DB unavailable), the job fails with `QueueError::TenantNotFound`. The worker follows the normal retry/failed-queue flow based on the job's retry configuration.

## Safety Notes

**Every query on tenant-owned tables must use `TenantScope`.** Unscoped queries silently return data from all tenants.

```rust
// WRONG: returns posts from all tenants
let all_posts = post::Entity::query().all().await?;

// CORRECT: returns posts for the current tenant only
let tenant_posts = post::Entity::scoped(TenantScope(post::Column::TenantId))
    .all()
    .await?;
```

**Cache keys must be tenant-prefixed.** When caching tenant-specific data, prefix keys with the tenant ID to prevent cross-tenant cache pollution:

```rust
let key = format!("tenant:{}:feature_flags", tenant.id);
```

**Routes that access tenant data must be behind `TenantMiddleware`.** Use route groups or apply the middleware globally. Calling `TenantScope` on a route that is not behind the middleware panics in development, surfacing the misconfiguration early.
