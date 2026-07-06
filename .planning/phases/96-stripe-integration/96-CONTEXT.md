# Phase 96: Stripe Integration - Context

**Gathered:** 2026-03-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Add Stripe payment capabilities to the Ferro framework as a `ferro-stripe` crate. Two billing dimensions: (1) platform SaaS subscriptions where the platform charges tenants for plan tiers, and (2) Stripe Connect where tenants connect their own Stripe accounts to process end-user one-time payments. Includes webhook handling, CLI scaffolding, MCP tools, test helpers, and documentation. Enriches TenantContext with subscription state and adds plan gate middleware.

</domain>

<decisions>
## Implementation Decisions

### Billing Scope
- Two-tier billing model: platform subscriptions (gestiscilo.it charges tenants) + Stripe Connect (tenants charge their end users)
- Platform subscriptions use fixed plan tiers (Free/Pro/Enterprise) — maps to TenantContext.plan
- End-user payments via tenant's connected Stripe account are one-time charges only (no end-user subscriptions)
- Stripe Checkout Sessions for payment collection (hosted page, zero PCI scope)
- Stripe Billing Portal redirect for tenant self-service subscription management
- Full subscription lifecycle: trial periods, grace periods on cancel, pause/resume
- Optional platform application fee on Connect transactions

### Webhook Handling
- Stripe events dispatched through ferro-events (dispatch_event pattern)
- Framework auto-handles core events: subscription.updated/deleted syncs TenantContext.plan automatically
- Two separate webhook endpoints: /stripe/webhook (platform) and /stripe/connect/webhook (connected accounts), each with its own signing secret
- All webhook processing queued via ferro-queue — verify signature inline, ack 200 immediately, process asynchronously

### Tenant-Billing Link
- TenantContext enriched with subscription details: subscription_status, trial_ends_at, on_grace_period, plus helper methods (tenant.on_trial(), tenant.subscribed())
- RequiresPlan("pro") middleware for plan-based route access control (like Auth middleware but for billing)
- Immediate restriction on subscription lapse — no grace period after cancellation/past-due, access downgrades instantly
- Phase 95's TenantContext.plan (currently Option<String>) gets replaced with rich subscription struct

### Developer Surface
- New `ferro-stripe` crate following ferro-cache/ferro-queue pattern, feature-gated re-export from framework
- `ferro make:stripe` CLI command scaffolds full integration: webhook routes, event listeners, migrations, env config, Connect setup
- Uses stripe-rust (async-stripe) SDK for type-safe Stripe API bindings
- MCP introspection tools: stripe config status, webhook event listing, subscription info
- Test helpers: mock webhook events, verify subscription state, fake Stripe responses
- Full documentation in docs/src/features/stripe.md

### Claude's Discretion
- API facade design: Stripe:: facade vs trait on TenantContext (evaluate which matches existing Ferro patterns better)
- Connect onboarding flow depth: full end-to-end helpers vs API wrappers only
- Storage approach: columns on tenant table vs separate billing table (evaluate trade-offs)
- Connect account ID placement: in TenantContext vs on-demand query (evaluate per-request frequency)
- Cache/TTL strategy for subscription data in TenantContext lookups

</decisions>

<specifics>
## Specific Ideas

- Primary consumer is gestiscilo.it — a multi-tenant platform where tenants use their own Stripe account for end-user payments
- Platform (gestiscilo.it) takes a subscription fee from tenants and possibly an application fee from tenant transactions
- Pattern reference: Laravel Cashier for subscription lifecycle, Stripe Connect docs for platform model

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `framework/src/tenant/mod.rs`: TenantContext with `plan: Option<String>` — designed to be enriched by this phase
- `ferro-events/src/dispatcher.rs`: Event dispatch pattern — Stripe webhook events map to ferro-events
- `ferro-queue/src/`: Background job queue — webhook processing dispatched here
- `ferro-notifications/src/`: Resend integration via reqwest — reference for external service integration pattern
- `framework/src/middleware/mod.rs`: Middleware trait — RequiresPlan middleware follows this pattern
- `framework/src/auth/`: Auth middleware — RequiresPlan is structurally similar (check state, block or pass)

### Established Patterns
- External service crates: `ferro-cache`, `ferro-queue`, `ferro-storage` — `ferro-stripe` follows same workspace pattern
- Feature-gated re-exports: `framework/src/lib.rs` with `#[cfg(feature = "stripe")]`
- Task-local context: TenantContext already stored in task-local — subscription data enriches it
- Builder pattern: consuming `mut self -> Self` for configuration
- Error types: `thiserror` derive, one Error enum per crate
- reqwest already used in ferro-notifications and ferro-mcp — async HTTP client available

### Integration Points
- `framework/src/tenant/mod.rs`: TenantContext struct needs subscription fields added
- `framework/src/tenant/lookup.rs`: DbTenantLookup needs to load Stripe subscription data
- `framework/src/lib.rs`: Re-export ferro-stripe types behind feature flag
- `ferro-cli/src/commands/`: New make_stripe.rs command
- `ferro-mcp/src/tools/`: New Stripe introspection tools
- `docs/src/features/`: New stripe.md documentation
- `.github/workflows/publish.yml`: Add ferro-stripe to publish workflow

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 96-stripe-integration*
*Context gathered: 2026-03-11*
