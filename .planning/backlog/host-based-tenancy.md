# Host-based tenancy — first-class custom-domain tenant resolution

Surfaced 2026-05-16 while migrating gestiscilo to v12.0/json-ui-v2. The downstream agent reached for low-level `PreRouteMiddleware` to rewrite paths based on the `Host` header — 50–100 lines of hyper-layer code — because ferro has no resolver for "custom domain → tenant identity via DB lookup". The framework forced the consumer into a workaround instead of supporting their actual intent.

## Planning Note

This document is a sketch from a downstream-app perspective, not an inside-ferro design. When promoted from backlog to a phase, the ferro planning agent must reconcile this proposal against `.planning/VISION.md` and the existing `framework/src/tenant/` primitives before drafting `PLAN.md`. Specifically:

- The exact API for host → tenant lookup (closure vs trait vs `TenantHostQuery` enum) is a ferro architectural decision and may diverge from the sketch below.
- The relationship between `HostResolver` and the existing `DbTenantLookup` should be examined — possibly fold this into `DbTenantLookup` as another lookup mode rather than a new resolver.
- The question of whether `PreRouteMiddleware` should remain a public surface at all (or be demoted to `pub(crate)` once all legitimate consumer use cases have first-class APIs) is part of this work.

## Context

ferro's tenant resolution today supports:

- `PathResolver` — tenant slug embedded in URL path (`/s/{slug}/...`)
- `SubdomainResolver` — tenant identified by subdomain of a fixed apex (`tenant.example.com`)
- `HeaderResolver` — tenant from an explicit `X-Tenant` header (or similar)
- `JwtClaimResolver` — tenant from a claim in a verified JWT
- `DbTenantLookup` — pluggable DB-backed lookup, currently used by some resolvers

Missing: a resolver for arbitrary custom domains backed by a DB table mapping `host → tenant_id`. This is the canonical SaaS pattern — every tenant maps their own domain (`shopA.com`, `shopB.com`) at the DNS layer, and the application identifies the tenant by inspecting the `Host` header.

Because no such resolver exists, downstream consumers compensate by writing pre-route middleware that:

1. Reads the `Host` header on every request
2. Queries a DB table for the tenant slug
3. Rewrites the URL path from `clienteA.com/foo` to `/s/clienteA-slug/foo`
4. Lets the existing path-based router and `PathResolver` take over

This is a workaround for a missing first-class feature. It also forces the consumer to write code at the `hyper::Request<hyper::body::Incoming>` layer — the framework's lowest level — for what should be a configuration concern.

## Desired consumer experience

Multi-tenant routing with custom domains should require zero middleware code. Bootstrap should read like this:

```rust
// bootstrap.rs
TenantMiddleware::with_resolver(
    HostResolver::with_db_lookup(db_pool, |host| {
        // How to look up the tenant for this host. The closure returns a
        // query descriptor the framework dispatches against the DB.
        TenantHostQuery::ExactDomain(host.to_owned())
    })
);
```

After registration:

- No `host.rs` middleware file
- No path rewriting
- No `pre_route_middleware!` call
- URL paths represent *what* the user wants, not *which* tenant they belong to
- `current_tenant()` returns the right tenant inside handlers, scoped queries work, etc.

## Scope (starting points, not commitments)

- `framework/src/tenant/host_resolver.rs` — new resolver, peer to `path_resolver.rs` and `subdomain_resolver.rs`. Implements the `TenantResolver` trait. Internally registers a `PreRouteMiddleware` so the resolution happens before route matching, but that's an implementation detail the consumer never sees.
- `TenantHostQuery` — small enum describing how to query (`ExactDomain`, `DomainPattern`, `Custom(Box<dyn Fn>)`) — keeps the common case ergonomic while leaving an escape hatch for unusual lookups.
- Cache layer — host → tenant resolution is per-request, so a small LRU or `ferro-cache`-backed lookup is necessary to avoid a DB hit on every request. Cache invalidation on tenant updates is part of the design.
- 404 path — if the host is unknown, the framework returns a structured "unknown host" response with a customisable handler (some apps want to redirect to a marketing site instead of 404).
- Docs — `docs/src/tenancy/custom-domains.md` with the three patterns (path, subdomain, custom domain) compared side by side, so the reader can pick the right resolver without writing any middleware.
- Audit `PreRouteMiddleware`'s public surface — once `HostResolver` lands, ask whether any legitimate consumer use case still needs the raw hyper-level API. If not, demote `PreRouteMiddleware` to `pub(crate)` and remove the `pre_route_middleware!` macro from the public prelude.

## Killer feature

The framework supports custom-domain tenancy as a one-liner. No middleware, no hyper exposure, no path acrobatics. The consumer states intent ("resolve tenant from host via DB") and ferro handles the plumbing.

## Source

gestiscilo migration to v12.0/json-ui-v2 (2026-05-16). The downstream agent's Phase 138 was blocked on writing `host.rs` middleware that exposed `hyper::Request<hyper::body::Incoming>` and `hyper::Response<Full<Bytes>>` to a consumer who only wanted "this domain belongs to this tenant". The conversation that surfaced this is the immediate input to this backlog item.
