---
id: SEED-001
status: dormant
planted: 2026-08-05
planted_during: v18.0 (Phase 263 — Projection-Native Frontend Substrate)
trigger_when: Post-1.0 / next major version — ecosystem/connector expansion, not a core-stability milestone
scope: Large
---

# SEED-001: `ferro-shopify` — publishable, embeddable Shopify app integration

A `ferro-shopify` integration crate — a sibling to `ferro-stripe`, `ferro-whatsapp`,
and `ferro-payments` — that reduces "ferro app → publishable Shopify app" to a scaffold
plus a few configuration values, rather than a from-scratch integration each time.

Target surface:
- **OAuth install flow** — the merchant-facing install/authorize/callback handshake, token exchange, per-shop access-token storage.
- **Session-token auth** — verifying Shopify App Bridge session tokens (JWT) on embedded-app requests.
- **Webhook HMAC verification** — constant-time signature check on inbound webhooks (reuses the `ferro-stripe` webhook-verification shape).
- **Mandatory compliance webhooks** — the GDPR/data-request, data-erasure, and shop-redact webhooks Shopify requires for App Store listing.
- **App Bridge embedding** — serving the embedded-admin shell so the ferro app renders inside the Shopify admin iframe.
- **Billing API** — recurring/usage/one-time application charges through Shopify's billing surface.

## Why This Matters

Recent ferro milestones lean commercial: the v16.6 POS Component Suite, and Shopify POS is
already the register-design anchor (see `user_shopify_pos_design_anchor` in memory). A
first-class path to *publish* a ferro app into the Shopify App Store — not merely call
Shopify's API — turns that commerce affinity into distribution. The Shopify App Store is a
demand channel a ferro app can plug into; making that path a scaffold rather than a research
project lowers the barrier from weeks to hours.

It also extends the established integration-crate pattern (`ferro-stripe`, `ferro-whatsapp`,
`ferro-payments`) into a new axis: prior crates let a ferro app *consume* a third-party
service; `ferro-shopify` additionally lets a ferro app *be consumed as* a Shopify app —
publishing surface, not just client surface.

## When to Surface

**Trigger:** Post-1.0 / next major version — ecosystem/connector expansion work.

This seed should be presented during `/gsd-new-milestone` when the milestone scope matches
any of these conditions:
- Planning a major (2.x) version or an explicit ecosystem/connector milestone.
- Adding a new third-party integration crate in the `ferro-stripe` / `ferro-whatsapp` / `ferro-payments` family.
- A commerce/POS/marketplace milestone where App Store distribution is in scope.
- A consumer app (e.g. gestiscilo) surfaces a concrete need to publish to the Shopify App Store.

Deliberately parked post-1.0: the remaining gate to 1.0 is operational polish + validation
completeness, not new capability (see ROADMAP core-stability framing). A publishing-surface
connector is additive ecosystem work and should not compete with the 1.0 cut.

## Scope Estimate

**Large — a full milestone.** Shopify's app-publishing surface is broad: OAuth install,
session-token verification, mandatory compliance webhooks, App Bridge embedding, the Billing
API, and conformance to App Store review criteria. Realistically a milestone shaped like
`ferro-payments` (phases 233–236) — crate scaffold + auth, then webhooks + compliance, then
embedding, then billing + publish. A thinner first cut (OAuth + webhooks + scaffold, deferring
App Bridge/Billing) could be a phase or two, but the full "publishable" promise is milestone-sized.

## Breadcrumbs

Structural template — clone the integration-crate shape:
- `ferro-stripe/src/` — closest analog: `client.rs`, `config.rs`, `error.rs`, `webhook/`, `idempotency.rs`, `testing.rs`. The `webhook/` HMAC-verification and `config.rs` `from_env()` patterns transfer almost directly.
- `ferro-payments/` — polymorphic-entity + `Billable` trait pattern; a reference for the Billing side.
- `ferro-whatsapp/` — another Business-Cloud-API integration, for the OAuth/token-exchange shape.

Design anchor and prior mentions:
- `user_shopify_pos_design_anchor` (memory) — Shopify POS as the register design reference.
- `ferro-json-ui/src/projection/builder.rs` — existing Shopify string reference (POS/register context).
- `.planning/research/FEATURES.md`, `.planning/research/SUMMARY.md` — earlier Shopify references.

Framework conventions the crate must honour:
- **Project-agnostic (CLAUDE.md rule):** `ferro-shopify` must not hardcode app identity. App name / callback base URL / "powered by" strings come from `APP_NAME` / `APP_URL` via a crate-local config struct's `from_env()`, mirroring `ferro-inertia::InertiaConfig::app_name`. Reviewers reject hardcoded tenant strings.
- **Publish wave:** new workspace crate → add to `.github/workflows/publish.yml` (leaf = Wave 1; add a wave up if it depends on other `ferro-*` crates).
- **No native build tooling:** any codec/crypto dep must build on the Rust toolchain alone (no nasm/asm features) — see the rav1e/ravif gotcha.

## Notes

- Planted as a "little drift" capture during the v18.0 milestone-close session (2026-08-05); the milestone close itself is unaffected.
- Open design questions for whoever plans this: (a) is embedding served by `ferro-inertia` or a dedicated shell? (b) does Billing reuse `ferro-payments`' `Billable` abstraction or stay Shopify-native? (c) does session-token JWT verification live here or generalize into a shared auth primitive? None need answering now.
- Strategic/competitive framing intentionally omitted from this repo file (potentially public); keep any positioning notes in local memory per the neutral-repo-voice rule.
