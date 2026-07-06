# Ferro Extension Brief — gestiscilo.it

**From:** gestiscilo.it project planning
**For:** Ferro framework development
**Date:** 2026-03-11

## Context

gestiscilo.it is a multi-tenant business management platform built on Ferro. Before any service work begins, Ferro needs five capabilities that don't exist yet. This brief describes what's needed — how to implement is up to the Ferro side.

## Requirements

### 1. Multi-Tenant Middleware (FERRO-01)

Ferro needs middleware that resolves a tenant from the request and injects a `TenantContext` into the handler — without any tenant-resolution code in the handler itself.

- Tenant routes live under `/s/{slug}/...`
- Invalid or missing slug returns 404
- Handlers receive `TenantContext` via the existing extraction pattern
- Non-tenant routes (landing page, auth, health checks) must not be affected

**Verification:** A request carrying a slug resolves to a `TenantContext` injected into the handler without any application code in the handler.

### 2. Stripe Integration (FERRO-02)

Ferro needs a Stripe integration using `async-stripe` that supports:

- Stripe Connect Standard account onboarding (linking a business to a Stripe account)
- Creating PaymentIntents with 3DS2/SCA support
- Processing charges through a connected account

Out of scope for now: webhooks, refunds, subscriptions, platform application fees.

**Verification:** An integration test creates an order and processes a charge via `async-stripe` against the Stripe test environment without panicking.

### 3. QR Code Generation (FERRO-03)

Ferro needs QR code generation that:

- Takes a URL and produces a QR code image
- Outputs both PNG and SVG formats
- Plain black-and-white (no colors, no logo embedding)
- Stores generated images via ferro-storage

**Verification:** A QR code PNG is generated for a test URL and written to disk via Ferro's file storage.

### 4. Tenant-Aware Background Jobs (FERRO-04)

Background jobs dispatched via ferro-queue must carry tenant context across async boundaries. A job enqueued in the context of a tenant must be able to access that tenant's identity when it executes.

**Verification:** A background job enqueued with a `tenant_id` logs that `tenant_id` from within the job handler, proving tenant context survives async dispatch.

### 5. ferro-json-ui Stable Release (FERRO-05)

ferro-json-ui needs to be published as a crate (like all other Ferro crates) so gestiscilo can pin to a specific version. gestiscilo will add integration tests on its side to catch breaking schema changes.

**Verification:** ferro-json-ui is published and gestiscilo's Cargo.toml references a pinned version with passing integration tests for dashboard rendering.

## Priority

All five are blockers for gestiscilo Phase 2+. No service work can begin until these are verified.

## Notes

- gestiscilo uses ferro-json-ui as the sole rendering engine for all UIs (dashboard and customer-facing)
- The platform is multi-tenant: every table will have `tenant_id`, every route resolves a business via slug
- Stripe Connect Standard is the payment model — the platform connects businesses, not the other way around
- Background jobs are used for email sending, reminders, and async processing — all tenant-scoped

---

*Generated from gestiscilo.it Phase 1 planning*
