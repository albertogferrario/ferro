# Roadmap: Ferro Framework

## Milestones

- ✅ [**v1.0 DX Overhaul**](milestones/v1.0-ROADMAP.md) — Phases 1-12 (shipped 2026-01-16)
- ✅ [**v2.0 Rebrand**](milestones/v2.0-ROADMAP.md) — Phases 13-22 (shipped 2026-01-16)
- ✅ **v2.0.1 Macro Fix** — Phase 22.1-22.3 (shipped 2026-01-17)
- ✅ [**v2.0.2 Type Generator Fixes**](milestones/v2.0.2-ROADMAP.md) — Phase 22.4-22.9 (shipped 2026-01-17)
- ✅ [**v2.0.3 DO Apps Deploy**](milestones/v2.0.3-ROADMAP.md) — Phase 22.10 (shipped 2026-01-17)
- ✅ [**v2.1 Inertia DX & Fixes**](milestones/v2.1-ROADMAP.md) — Phases 33-34 (shipped 2026-01-17)
- ✅ [**v2.2 CLI Improvements**](milestones/v2.2-ROADMAP.md) — Phases 35-37 (shipped 2026-02-09)
- ✅ [**v3.0 JSON-UI**](milestones/v3.0-ROADMAP.md) — Phases 23-32 (shipped 2026-02-09)
- ✅ [**v4.0 Production Readiness**](milestones/v4.0-ROADMAP.md) — Phases 38-46 (shipped 2026-02-10)
- ✅ [**v5.0 Proximity — JSON-UI Field Test**](milestones/v5.0-ROADMAP.md) — Phases 47-53 (shipped 2026-02-10)
- ✅ [**v5.1 Housekeeping**](milestones/v5.1-ROADMAP.md) — Phases 54-57 (shipped 2026-02-13)
- ✅ [**v6.0 ferro-lang — Localization**](milestones/v6.0-ROADMAP.md) — Phases 58-66 (shipped 2026-02-13)
- ✅ **v6.1 Fix Known Issues** — Phase 67 (shipped 2026-02-24)
- ✅ **v7.0 Resend Integration** — Phase 68 (shipped 2026-02-25)
- ✅ **v7.1 Static File Serving** — Phase 69 (shipped)
- ✅ **v7.2 CI Stability** — Phase 70 (shipped)
- ✅ **v7.3 Vite Manifest** — Phase 71 (shipped)
- ✅ **v7.4 Security Hardening** — Phases 72-74 (shipped 2026-02-26)
- ✅ **v7.5 Type Generator Fix** — Phase 75 (shipped 2026-02-27)
- ✅ **v7.6 Default API Scaffold** — Phase 76 (shipped 2026-02-27)
- ✅ **v7.7 Validate & Fix API Scaffold** — Phase 77 (shipped 2026-02-28)
- ✅ **v7.8 Memory Leak Fixes** — Phase 78 (shipped 2026-02-28)
- ✅ **v8.0 Consumer MCP — OpenAPI Bridge** — Phases 79-82 (shipped 2026-02-28)
- ✅ **v8.1 API DX Polish** — Phase 83 (shipped 2026-02-28)
- ✅ [**v9.0 Service Projections**](milestones/v9.0-ROADMAP.md) — Phases 84-94 (shipped 2026-03-01)
- ✅ [**v10.0 JSON-UI Visual Overhaul**](milestones/v10.0-ROADMAP.md) — Phases 102-107 (shipped 2026-03-26)
- ✅ [**v11.0 Framework Consolidation Audit**](milestones/v11.0-ROADMAP.md) — Phases 108-114 (shipped 2026-04-05)
- ✅ [**v11.1 Template Renderer**](milestones/v11.1-ROADMAP.md) — Phase 114.1 (shipped 2026-04-05)
- ✅ **v11.2 Deploy & Scaffolder Hardening** — Phases 122-131 (shipped 2026-04-14)
- ✅ **v11.3 S3 Storage Driver** — Phase 132 (shipped 2026-04-14)
- ✅ **v11.5 Projection Architecture Prep** — Phases 133-135 (shipped 2026-04-17). Generalize Renderer trait, relocate renderers to output crates, ServiceDef derivation bridge.
- ✅ **v11.6 ferro-stripe Capability Refactor** — Phases 140-142. Reshape `ferro-stripe` from Stripe-product axis (`connect/`, `subscription/`) to capability axis (`checkout`, `refund`, `account`, `webhook`); land `CheckoutBuilder` / `CheckoutIntent`, `ProcessedEventLog` trait, fully-typed events (no `event_json` smuggling), `SyncDispatcher` as the sole handler registry for both sync and queue dispatch paths (Stripe events do not implement `ferro_events::Event`), queue path opt-in for eventual-consistency events. Source: gestiscilo-it v6.3 field test. [Design](research/v11.6-FERRO-STRIPE-REFACTOR.md)
- ✅ **v11.7 Tailwind Static CSS Pipeline** — Phase 143 (shipped 2026-04-21). Pre-built `ferro-base.css` embedded at compile time, served from `/_ferro/ferro-base.css`; `tailwind_cdn` default flipped to `false`; `stylesheet_urls` added; theme injection migrated to plain `<style>`. Full details archived in [milestones/v11.7-ROADMAP.md](milestones/v11.7-ROADMAP.md).
- ✅ **v11.8 HttpResponse Header Semantics Fix** — Phase 143.1 (shipped 2026-04-21). `HttpResponse::header()` currently pushes instead of replacing, producing comma-joined Content-Type headers like `text/plain,text/html; charset=utf-8` for every `JsonUi::render` response. Safari reads the first value and renders raw text — the actual cause of the gestiscilo.it field report that drove phase 143. Fix is replace-semantics (case-insensitive) plus an `append_header()` escape hatch for `Set-Cookie`. Phase 143 remains valuable (pre-built CSS > dev-only CDN) but did not and could not fix the reported Safari bug. [Context](phases/143.1-http-response-header-replace-semantics/143.1-CONTEXT.md)
- ✅ **v11.9 Notifications & Rich-Text Foundations** — Phases 149-150 (shipped 2026-05-01; verification passed, UAT 2/2 — roadmap status reconciled 2026-06-07). Source: gestiscilo-it v6.4 Documents & Notifications field test. Extends `ferro-notifications` with `Channel::WhatsApp` + `Channel::InApp` adapters and `MailMessage::attachment()` builder; ships `ferro-json-ui RichTextEditor` component (Quill 2.0.3 plugin pattern) so consumer apps can author rich-text bodies without bundling. Auto-publishes via GH Actions. Single load-bearing prerequisite for gestiscilo-it v6.4 Phase 120 (notification dispatcher) and Phase 125 (document template editor).
- ✅ **v11.10 ferro-wallet — Digital Wallet Passes** — Phase 151 (shipped 2026-05-11; retroactive verification 2026-06-07: 8/8 criteria, 41/41 tests, published on crates.io at 0.2.44). One out-of-band acceptance remains with the consumer: real-credential Apple/Google pass build on device, owned by the gestiscilo-it wallet integration. Source: gestiscilo-it digital wallet booking pass field test. New project-agnostic crate `ferro-wallet` providing the `WalletSubject` trait, `ApplePassBuilder` (PKCS#7-signed `.pkpass`), `GoogleWalletBuilder` (RS256 save-link JWT), and image / QR primitives. Follows architecture principle 6 (project-agnostic, reads `APP_NAME` / `APP_URL` via `WalletConfig::from_env`). Single load-bearing prerequisite for gestiscilo-it wallet booking passes integration. [Context](phases/151-ferro-wallet-crate/151-CONTEXT.md) · [Spec](../docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md)
- 📋 **v11.11 Resource Reservation & Live Read-Model Primitives** — Phases 152-155 (planned 2026-05-13). Source: gestiscilo-it inventory monitoring field test. Four reusable horizontal primitives: `ferro-orm::GuardedUpdate` (atomic conditional updates), `ferro-audit` (structured before/after log), `ferro-reservation` (generic hold/commit/release with TTL), `ferro-projection` (live read-model from domain events with broadcast deltas). Unblocks gestiscilo-it v6.3 online checkout reservation TTL and v6.7 inventory monitoring. [Design](research/INVENTORY-PRIMITIVES.md)
- ✅ **v11.12 Migration Deploy Safety** — Phase 157 (shipped 2026-05-14; verification passed — roadmap status reconciled 2026-06-07). Source: gestiscilo-it 2026-05-13 production breakage — SQLite-hardcoded backfill SQL failed on Postgres, runtime runner swallowed the error, no PRE_DEPLOY gate, server served a stale schema. Three framework gaps closed at once: backend-portable migration helpers (`ferro_migration::backfill_random_hex`, etc.), `ferro do:init` emits a `PRE_DEPLOY` migrate job by default, `ferro doctor --deploy` adds a `migrate_gate` check that fails when migrations exist but no PRE_DEPLOY job is configured. [Context](phases/157-migration-deploy-safety-backend-portable-backfill-helpers-fe/157-CONTEXT.md)
- ✅ **v12.0 JSON-UI v2 — Spec-Driven Rendering** — Phases 115-121, 159-164 (shipped 2026-05-19). Spec wire-shape public contract, expression engine, 42 built-in components incl. DataTable/KanbanBoard/DetailPage, JSON Schema validation, v1 API removed, production-validated via gestiscilo v7.0 friction loop. 491 commits merged.
- ✅ **v12.0.1 JSON-UI v2 Runtime Patches — Staff-Domain Field Test** — Phase 175 (shipped 2026-05-20; verification passed — roadmap status reconciled 2026-06-07). Source: gestiscilo-it v6.9 staff-domain UAT (consumer phase 151 α). Six runtime gaps surfaced by a per-staff weekly hours editor with copy-source-to-N-targets shortcut: F1 spec-depth limit too low (depth-8 trees stripped, diagnostic conflated with cycle detection), F2 `CheckboxGroup` not registered in v2 catalog, F3 tabbed pages render every panel concurrently, F4 `Switch` component does not render, F5 `Input[type=file]` not rendered + `Form.enctype` not propagated, F6 DataTable `{row.X}` placeholders not interpolated in per-row form actions. One plan per finding; default to bidirectional adaptation for F2/F4 (re-introduce + document substitution). [Context](phases/175-json-ui-v2-runtime-patches-staff-domain-field-test/175-CONTEXT.md)
  - **Plans:** 6 plans
    - [x] 175-01-PLAN.md — F1: bump `MAX_NESTING_DEPTH` 5→16 and split depth-limit/cycle diagnostics (Wave 1)
    - [x] 175-02-PLAN.md — F3: extend tabs runtime IIFE with `initTabFromUrl` (URL `?tab=` honored at boot) (Wave 2)
    - [x] 175-03-PLAN.md — F6: extend DataTable `template_actions`/`template_url` to interpolate `{row.X}` alias (Wave 2)
    - [x] 175-04-PLAN.md — F2: register `CheckboxGroup` as v2 alias for `CheckboxList` (catalog + dispatch + docs) (Wave 3)
    - [x] 175-06-PLAN.md — F5: add `InputType::File` + `InputProps.accept` + `FormProps.enctype` end-to-end (Wave 4)
    - [x] 175-05-PLAN.md — F4: pin Switch-at-depth-8 regression after F1; document Checkbox-styled-as-switch substitution (Wave 5 — runs after 175-04 to avoid `components.md` overlap and after 175-06 to avoid `render/form.rs` overlap)
- ✅ **v12.0.2 JSON-UI v2 Runtime Patches — Booking↔Staff Binding Field Test** — Phase 176 (shipped 2026-05-21; UAT closed 2026-06-07 via consumer field evidence: gestiscilo `calendar_day.json` exercises Card.badge, Card.subtitle and element `visible` in production through Chrome MCP UAT walkthroughs on ferro 0.2.42). Source: gestiscilo-it v6.9 booking↔staff binding UAT (consumer phase 152 β). Three runtime gaps surfaced by a kanban dashboard with countdown badges + per-staff filter chip strip + staff-member detail widget: F7 `Card.badge` prop silently dropped (server emits, renderer ignores), F8 `Card.subtitle` prop silently dropped (server emits, renderer ignores), F9 `Grid.visible` conditional drops the entire subtree even when the path evaluates to true. Each finding ships server spec correctly; renderer template has no slot for `badge`/`subtitle`, and Grid's visibility evaluator either doesn't parse `visible` or evaluates against the wrong scope. One plan per finding; F7+F8 both extend the Card component slot template and can share a plan if the planner judges them coupled. [Context](phases/176-json-ui-v2-runtime-patches-booking-staff-field-test/176-CONTEXT.md)
- ✅ **v11.11.1 ferro-reservation Kernel Atomicity Hardening** — Phase 177 (shipped 2026-05-21). Source: gestiscilo-it v6.9 β acceptance failure (consumer phase 152 STBOOK-15). Closed the `ReservationKernel::hold` check-then-act race: hold body now runs in a serializable transaction, with SQLSTATE 40001 translated to `ReservationError::Insufficient` at every write site (INSERT, audit write, commit — not just commit); migration columns switched to `json_binary` so btree indexes are valid on Postgres. Verified 6/6 success criteria; 50-iteration race tests pass flake-free on both SQLite and live Postgres SERIALIZABLE; gestiscilo-it consumer regression `concurrent_double_book_same_staff` passes deterministically. [Context](phases/177-reservation-kernel-hold-atomicity/177-CONTEXT.md) · [Verification](phases/177-reservation-kernel-hold-atomicity/177-VERIFICATION.md)
- 📋 **v11.6.1 ferro-stripe Manual Capture** — Phase 189 (planned 2026-06-07). Source: gestiscilo-it v6.3-extended booking fund-hold field test. Extends the v11.6 capability-axis crate with Stripe manual capture so consumer apps can authorize card funds without charging (booking deposits): `CheckoutBuilder::manual_capture()` sets `payment_intent_data.capture_method = manual`; new `payment_intent.rs` capability module with `capture(payment_intent_id, amount_cents: Option<i64>)` (partial capture supported) and `cancel(payment_intent_id)`; new typed events `StripePaymentIntentAmountCapturableUpdated` and `StripePaymentIntentCanceled` registered in the parser contract with golden-JSON fixtures; manual capture must compose with `destination()` Connect charges (authorize on platform, capture transfers to connected account). The authorize/capture/cancel triple deliberately mirrors `ferro-reservation` hold/commit/release semantics — document the correspondence in `docs/src/features/stripe.md`. Out of scope: SetupIntent save-card flow for authorizations beyond the ~7-day card window (consumer-side design decision at gestiscilo v6.3 plan time; promote to a ferro phase only if gestiscilo picks that path). Consumer: gestiscilo-it v6.3 Online Checkout & Payments (queued after its v7.1), consumes via published crates.io bump per the Phase 176 ↔ ferro Phase 181 pattern.
- ✅ **v11.6.2 ferro-stripe Refund Event Completeness + 0.7.0 Release** — Phase 193 (code complete 2026-06-09; **ferro-stripe 0.7.0 publish pending operator `git push`** — GH Actions auto-publishes on push to master, unblocking gestiscilo Phase 99). Source: gestiscilo-it v6.3 Phase 99 (Refund dashboard UX) field test — operator-locked Option B per gestiscilo CONTEXT.md D-27. Closes a payload-coverage gap in the `StripeChargeRefunded` typed event: the struct currently exposes `event_id / charge_id / payment_intent_id / amount_refunded_cents / metadata` but NOT the `refund_id` field that Stripe always sends. Consumer use case: gestiscilo Phase 99 `on_refunded` handler needs to look up its local `refunds` table row by `stripe_refund_id` (set when the operator clicked "Emetti rimborso") and mark `confirmed_at = now()` — without the `refund_id` field on the event struct, the consumer cannot perform that lookup without bypassing ferro-stripe via direct `stripe::` imports (violates the V-95-01 "no direct `stripe::` import" gate established in v11.6). Adds `refund_id: Option<String>` to `StripeChargeRefunded` (parsed from the charge's refunds list — `charge.refunds.data[].id`; a `charge.refunded` event carries a `Charge`, not a top-level `Refund`), with golden-JSON fixture updates + parser-contract test. Releases ferro-stripe 0.7.0 — the published version label that captures Phase 189 (Manual Capture, shipped 2026-06-07 but not yet released) + this new refund_id work as combined breaking changes per gestiscilo Phase 97 D-14 expectation. Consumer: gestiscilo-it v6.3 Phase 99 Plan 03 (webhook extension) hard-blocks on the field; Plan 04 (closeout) hard-blocks on the 0.7.0 publish per `feedback_ferro_publish.md` auto-publish via GitHub Actions on push to master. Out of scope: backporting refund_id to existing v0.5.x consumers (v0.7.0 is opt-in via the documented consumer bump).
- ✅ **v12.1 AI — ferro-ai SDK & AI as Projection Consumer** — Phases 165-173 (shipped 2026-06-09; planned 2026-05-15, reframed 2026-06-07, started 2026-06-08). AI as a first-class consumer of the projection/intent core. Capstone (Phase 173): `make:json-view` consumes a `ServiceDef` via the existing `Spec::from_service_def` renderer + the projection-roundtrip proof test (NL → ServiceDef → rendered JSON-UI). All 9 phases verified.
- ✅ **v11.6.3 ferro-stripe Connect Application Fee Helper** — Phase 201 (shipped 2026-06-11; implemented commit `705bac6b`, verified retroactively — 5/6 criteria green, criterion 6 = pending operator `git push` → auto-publish 0.9.0). Source: gestiscilo-it v7.1 photographer payment-gated-share field test (Marea Studio). The Connect destination-charge surface is otherwise complete in 0.8.0 — `account::{create_account,create_link,retrieve_account}` (Standard), `CheckoutBuilder::destination(account_id, fee_cents)` (sets `application_fee_amount` + `transfer_data.destination` + `on_behalf_of`, composes with `manual_capture`), `WebhookEvent.account: Option<String>` (Connect account routing), `StripeConnectAccountUpdated { account_id, charges_enabled, payouts_enabled, details_submitted }`, `verify_webhook(body, sig, secret)` (consumer passes the connect secret), and `StripeConfig.{connect_webhook_secret, application_fee_percent}` all ship. The one missing primitive is fee computation: `StripeConfig::application_fee_for(amount_cents) -> Option<i64>` turning the platform `application_fee_percent` into rounded cents (`None` when unset/0). Adds ferro-mcp `stripe_config_status` parity (connect-webhook-secret presence + application-fee-percent) and a `docs/src/features/stripe.md` end-to-end Connect application-fee example, mirroring the Phase 189 manual-capture correspondence doc. Publishes ferro-stripe 0.9.0. Consumer: gestiscilo-it v6.10 Phase 204 (platform fee + destination wiring) hard-blocks on the helper and consumes via crates.io bump per the Phase 193 ↔ gestiscilo Phase 99 pattern; gestiscilo Phase 203 (Connect webhook endpoint + secret split) needs no new ferro surface (consumes the already-shipped 0.8.0 webhook primitives).
- ✅ **v12.4 Form Validation DX** — Phases 190-192 (shipped 2026-06-09). Async DB-backed `unique` rule with exclude-self (edit-form safety) + `ConstraintMap` opt-in DB constraint→field-level error mapping + ferro-mcp template and docs showing the two-layer proactive+defensive pattern together. Source: gestiscilo-it field test (slug-uniqueness violations surfacing as raw SQL errors). Both live-Postgres paths verified via `#[ignore]`d gate tests. All 3 phases verified.
- ✅ **v12.5 Projection Checkpoint** — Phases 194-196 (shipped 2026-06-10). Close the agent write→verify loop: `checkpoint_projection` MCP tool walks the intent-slice spine, owns the field→column seam (the only silent gap no existing validator covers), delegates the remaining seams to existing validators, and returns a single structured verdict with ranked next steps. Closes by default after generation; ambient status in `application_info`/`projection_coverage`. Killer feature: a dangling projection field (no backing migration column) surfaces statically in one call rather than at runtime.
- ✅ **v12.6 Consumer App MCP (Browser Login)** — Phases 197-200 (shipped 2026-06-11; all four verified passed, dogfood acceptance GO — see `200-ACCEPTANCE.md`. Published to crates.io 2026-06-11 via run 27321715231 — `ferro-mcp-server` + `ferro-mcp-oauth` bootstrap-published at 0.2.51 then registered in publish.yml Wave 2 for auto-publish). A deployed ferro application serves its own OAuth-protected MCP endpoint so a consumer agent can authenticate through the browser and use the application's projections as per-tenant tools. New `ferro-mcp-server` output crate with `McpRenderer` (projection→tool, `ServiceDef`-derived schema, opt-in marker); Streamable HTTP MCP endpoint; OAuth 2.1 browser login (discovery metadata, DCR, PKCE, consent, audience-bound tokens); per-tenant scoping and policy enforcement reused structurally from existing middleware. Design spec: `docs/superpowers/specs/2026-06-10-consumer-app-mcp-browser-login-design.md`.
- ✅ **v12.7 Passwordless MCP Auth** — Phases 202-203 (shipped 2026-06-12). Source: field finding while validating v12.6 against the gestiscilo consumer — the OAuth browser-login flow assumed a synchronous password form, but passwordless (magic-link) apps break it two ways: (a) the post-login handler must resume the authorize request via `oauth_return_to`, which a magic-link `verify` handler does not do by default, and (b) email links open on any device, which the authorization-code-over-loopback flow cannot reconcile. (202) formalizes the login-resume contract in `ferro-mcp-oauth` (a helper any login handler calls to obtain the post-login redirect target) and converts the bundled sample app login to magic-link as the golden-path exemplar, with an async-flow acceptance test; consumer pairing: gestiscilo `verify_magic_link` adopts the contract (same-device path). (203) adds OAuth 2.0 Device Authorization Grant (RFC 8628) to `ferro-mcp-oauth` — `device_authorization` endpoint, a user-code verification page bound to the existing consent + tenant scoping, and device-code token polling — the auth path for passwordless, cross-device, and headless/CLI MCP clients. Reuses the v12.6 consent and tenant-scoping surfaces; no second token issuer. Detailed phase scope at the end of this file.
- 📋 **v12.2 Frontend Performance Hardening** — Phases 182-184 (planned 2026-06-06). Source: gestiscilo-it jetskiadriatic startup-lifecycle audit. Three runtime/framework primitives, each paired 1:1 with a gestiscilo v6.6.1 phase that consumes the published primitive via crates.io bump (mirrors the Phase 181 ↔ gestiscilo Phase 176 pattern). (182) `ferro-json-ui` `data-lazy-hero` runtime primitive — IntersectionObserver promoting `<video preload="none">` → `preload="auto"` on viewport approach; (183) `ferro-bundle` new crate — in-memory immutable byte blobs with content-hashed immutable-cache serving; (184) `ferro::InlineBudget` + `ferro::RequestTelemetry` — request-scoped accumulator with inline/preload decision + per-key ring buffer.
- 📋 **v12.3 Deployment Platform Primitives** — Phases 185-188 (planned 2026-06-07). Source: gestiscilo-it v7.1 Tenant Frontend Platform (locked design at gestiscilo `.planning/research/v7.1-ARCHITECTURE.md`, D-01..D-06). Four generic primitives, each paired 1:1 with a gestiscilo consumer phase that bumps the published crate (185 ↔ gestiscilo 188, 186 ↔ gestiscilo 188, 187 ↔ gestiscilo 189, 188 ↔ gestiscilo 190). (185) `ferro::queue` — DB-backed job queue replacing the Redis-only ferro-queue backend: `Job` trait, `WorkerLoop` in `ferro serve`, atomic claim (Postgres `FOR UPDATE SKIP LOCKED` / SQLite `BEGIN IMMEDIATE` + `UPDATE…RETURNING`), retry/backoff, stuck-job reaper; (186) `ferro-deployments` new crate — immutable `Deployment` model, `DeploymentStorage` trait, atomic `promote`/`rollback`, `preview_url` helper, artifact-shape agnostic; (187) `ferro-assets` new crate — `Pipeline` composer with content-type-aware transforms: `html_minify` (lol_html), `css_minify` (lightningcss), `js_minify` (swc_ecma_minifier), `image_transcode` (pure-Rust `image`+`ravif`, AVIF+JPEG responsive variants — libvips rejected for thread-safety), `inject_before_tag`; (188) `ferro-storage` extension — `cdn_url()`, `PurgeApi` trait, DO Spaces CDN adapter (feature-flagged Bunny/Cloudflare). Primitives stay consumer-agnostic: static HTML sites, JSON-UI spec bundles, and Inertia SSR manifests all fit the deployment abstraction. See "v12.3 Deployment Platform Primitives" phase details at the end of this file.
- ✅ **v13.0 Compressive Validation** — Phases 207-211 (complete 2026-06-13; all five COMP items done — 207 COMP-02, 208 COMP-05, 209 COMP-01 Slice A, 210 COMP-03, 211 COMP-04. Substance-first: the abstraction was fixed first, then measured. COMP-04 surfaced the scaffold↔library drift later fixed in v13.3/Phase 214). First slice of the Road to v1.0 program: empirical validation of the projection/intent abstraction across five COMP items. COMP-02 synthetic regression catalog (Phase 207), COMP-05 cross-modality vocabulary sketch (Phase 208), COMP-01 gestiscilo migration Slice A (Phase 209), COMP-03 agent-success-rate harness (Phase 210), COMP-04 time-to-working-app benchmark (Phase 211). Targets v1.0 criterion #2 (projection/intent validated through real applications and a synthetic catalog) and the compressive beauty dimension (priority #1).
- ✅ **v13.1 CRUD Handler Proc Macros** — Phase 212 (complete 2026-06-13, CRUD-01–06; 12/12 verified, code review 0 critical). With this the v13.x batch scoped so far — v13.0/v13.1/v13.2/v13.3 — is complete; 0.2.56 bumped locally, not yet published. Framework-ergonomics feature driven by the gestiscilo Phase 202 duplication survey: two route-attribute proc macros (`#[resource_get]`, `#[resource_post]`) that fold the recurring tenant-resolve + typed-param + tenant-scoped-lookup + 404-dispatch prelude (repeated 200+ times in a single consumer) into a macro, plus a `Validator::validate_or_redirect` helper. Scoped to ferro's framework-product axis, not one consumer's LoC. Originally mis-numbered 209 by the cross-repo gestiscilo evidence pass; relocated here to keep v13.0 purely Compressive Validation. Phase scope at `phases/212-crud-handler-proc-macros/212-CONTEXT.md`.
- ✅ **v13.2 Projection Render Completeness** — Phase 213 (SHIPPED in 0.2.55, 2026-06-13 — Gap A kanban structure/content split integration-verified live on gestiscilo feat/207; Gaps B–E unit+live where exercisable. Scoped 2026-06-12 from COMP-01 Slice A findings). Make `JsonUiRenderer`'s projection render *content-complete* so real-world views actually migrate. Phase 209 confirmed the render is layout-complete but content-incomplete: Browse data-binds, but Process emits a placeholder kanban (`emit_kanban_root`, "state-machine awareness is a deferred idea"), Summarize emits empty StatCard values (`emit_statcard_root`, `value: String::new()`), the `actions` slot is an empty stub for every intent (`emit_actions_placeholder`, "Deferred to Phase 118+"), `ImageUrl` fields don't render in tables, and projections emit a standalone spec with no app-shell layout. This is the unblock for all future projection migration. Scope at `phases/213-projection-render-completeness/213-CONTEXT.md` (to be created). Depends on Phase 209 (the validation that scoped it).
- ✅ **v13.3 Scaffold↔Library Parity & Published-Artifact Smoke Test** — Phase 214 (complete 2026-06-13 — parity fixed via `ferro` facade exports + corrected scaffold templates, plus a two-layer CI guard; 10/10 must-haves verified; the `ci.yml`/`publish.yml` jobs await a manual `workflow`-scope push). Source: COMP-04 (Phase 211) cold-cache benchmark, which found the **published 0.2.55 scaffold does not compile** — `cargo build` of a freshly scaffolded app fails with 52 errors (scaffold templates reference `ferro::error_response!`, `#[rule]`, `ferro::Queue`, and `ActiveValue` that the published `ferro` crate doesn't export, plus `make:job` emits `use ferro_queue::…` without adding `ferro-queue` to the generated `Cargo.toml`). Two parts: (1) align the `ferro-cli` scaffold templates with the published `ferro` surface so `ferro new → make:auth → make:scaffold ×3 → make:job → cargo build` compiles clean; (2) add a CI smoke test that scaffolds and builds against the *published* artifact so a non-compiling release can never ship silently again — the permanent guard COMP-04's apparatus enables. Framework-correctness, not a Compressive Validation item. Scope at `phases/214-scaffold-library-parity/214-CONTEXT.md`. Depends on Phase 211 (the validation that found it).
- ✅ [**v14.0 Channel Projection — Non-Visual Rendering**](milestones/v14.0-ROADMAP.md) — Phases 215-216 (shipped 2026-06-13). First production non-visual `Renderer` (`ferro-text::TextRenderer`) projecting the same `ServiceDef` as the visual/MCP renderers, plus the `BaseContext`/`FieldDef`/`Intent` extensions (CHAN-01–04). Voice, structured-API, mobile `device_class`/chart-card, and inbound `ferro-ai` classification deferred to a follow-up channel milestone.
- ✅ [**v15.0 Agent-Operable App (Consumer MCP)**](milestones/v15.0-ROADMAP.md) — Phases 217-221 (shipped 2026-06-14). Extends the projection/intent abstraction to a write-and-act MCP surface: per-tenant API-key auth, `ActionDef`-derived write tools (guard-filtered), server-side guard re-enforcement at execution, `ferro-ai` confirmation gating for destructive actions, and an inbound natural-language intent loop with a replay/smoke CI path (CI-testable without live-LLM spend). Validated against gestiscilo via synthetic fixtures; consumer adoption is a separate follow-up.
- ✅ [**v16.0 Write-Boundary AX**](milestones/v16.0-ROADMAP.md) — Phases 231-232 (shipped 2026-06-16). The projection write path now derives transitions from the `StateMachine` the framework owns — `TransitionPlan` + `derive_transition_plan` (no hand-written `match`), server-side guard re-eval, post-persist override hook, registration-time drift gate, and one `framework::write` kernel backing both the MCP and the new visual `POST /{service}/{action}` write surfaces (single-source, no per-channel executor).
- ✅ **v16.2 ferro-inertia First-Load HTML Shell** — Phase 238 (completed 2026-06-21). `ferro-inertia` emits a complete first-load HTML document (embedded `data-page` + resolved Vite asset tags) via content negotiation, with `App::set_inertia_config`/`InertiaConfig::from_env` plumbing, a configurable root template (title/head_extras/mount_id), and same-origin + Vite `server.proxy` docs. Verified live (dev hydration + prod manifest tags). Promoted from the downstream `u` app's deferred first-load shell.
- 🚧 **v16.3 MCP CRUD Data Surface (Track A)** — Phases 239-243 (in progress, started 2026-06-23). A projection that opts in (`.creatable`/`.updatable`/`.deletable` + `.mcp_write_ability`) derives a complete, safe, tenant-scoped CRUD interface (`create_`/`update_`/`delete_<svc>` + query-polished `list_<svc>`) as MCP tools with zero hand-written tool code. CRUD verbs dispatch through a new `derive_crud_plan` that **extends** the shipped `framework::write` kernel (231/232) — reusing the override-hook registry, idempotency, channel-parameterized audit, and confirmation; it does not rebuild the dispatcher. Soft-delete (`deleted_at`) + confirmation gating; `read_write` scope + `.mcp_write_ability` Gate + server-side tenant injection (non-disclosure). Declaration surface + `validate()` write-ability rule already shipped (`5cb17d60`). Anchor spec: `docs/superpowers/specs/2026-06-23-projection-crud-data-surface-design.md` (Track A of the four-track MCP capability program).

---

### ✅ v11.0 Framework Consolidation Audit (Shipped 2026-04-05)

Phases 108–114 — full details archived in [milestones/v11.0-ROADMAP.md](milestones/v11.0-ROADMAP.md).

---

### ✅ v11.1 Template Renderer (Shipped 2026-04-05)

Phase 114.1 — full details archived in [milestones/v11.1-ROADMAP.md](milestones/v11.1-ROADMAP.md).

---

### ✅ v11.5 Projection Architecture Prep (Shipped 2026-04-17)

Phases 133–135:
- **133**: Generalized `Renderer` trait with associated `Output` and `Context` types (modality-agnostic)
- **134**: Relocated `JsonUiRenderer` from ferro-projections to ferro-json-ui; broke ferro-projections → ferro-theme dependency
- **135**: `ServiceDef::from_model()` derivation bridge + `generate_projection` MCP tool

---

### ✅ v11.6 ferro-stripe Capability Refactor (Phases 140-142, shipped 2026-04-20)

**Milestone Goal:** Reshape `ferro-stripe` along the capability axis and elevate protocol-level concerns (idempotency, typed events, signature verification, sync vs eventual-consistency dispatch) into the framework. Today's ferro-stripe splits its modules by Stripe product (`connect/`, `subscription/`) rather than capability, defaults all webhook handling to queue-job dispatch (wrong for payment-correctness events), stubs idempotency with a TODO, and ships a single-line-item `create_connect_checkout` helper too thin to replace hand-rolled `CreateCheckoutSession` usage. Pre-1.0 is the one chance to fix this before consumer assumptions ossify.

**Source:** gestiscilo-it v6.3 Online Checkout & Payments field test. Ferro-side design lives in `.planning/research/v11.6-FERRO-STRIPE-REFACTOR.md` and is self-sufficient. Full cross-repo context (app-side state machine, reservation TTL, refund UX) lives in the gestiscilo repo at `.planning/research/v6.3-ONLINE-CHECKOUT.md` — not linked here because cross-repo relative paths assume a sibling checkout layout.

**What changes:**
- Module layout: `checkout.rs` / `refund.rs` / `account.rs` / `webhook/{verify,events,sync,queue}` / `idempotency.rs` / `client.rs`. `connect::*` and `subscription::*` removed.
- `CheckoutBuilder` → `CheckoutIntent` primitive (typed return carrying `session_id`, `url`, `expires_at`, `idempotency_key`). Replaces `create_connect_checkout` / `create_subscription_checkout`.
- `ProcessedEventLog` trait + `MemoryProcessedLog` impl; recommended SQL schema documented. Apps implement against their DB. Replaces the stubbed `is_processed` free fn.
- Typed events drop `event_json: String` smuggling; every event carries fully-parsed fields.
- **Stripe event structs do not implement `ferro_events::Event`.** `SyncDispatcher` is the sole handler registry. `ProcessStripeWebhook` (queue path) holds `Arc<SyncDispatcher>` and delegates to it — both dispatch paths share one handler registration point, eliminating double-fire risk.
- `SyncDispatcher::on::<E, _>(handler)` registers per-event-type handlers; `dispatch` returns `Result` so webhook endpoints return 500 on handler error and Stripe retries. Default path for payment-correctness events.
- Existing queue-based `ProcessStripeWebhook` relocates to `webhook::queue` — opt-in for eventual-consistency events (subscription drift, analytics). Accepts `Arc<SyncDispatcher>`.
- `refund::create` and `account::retrieve` added (missing today).
- `client.rs` adds `Stripe::with(key)` scoped override alongside the static default for per-tenant key scenarios.
- ferro-mcp `stripe_webhook_events` and `stripe_config_status` updated for capability-axis structure.

**What stays:**
- `Stripe::init` static default facade.
- `verify_webhook` signature function.
- `ferro-queue` dependency (queue path remains opt-in).
- Connect Standard destination-charge pattern.

**Breaking-change ledger** — see design doc §4.6. Versions: ferro-stripe 0.3.x → 0.4 → 0.5 across phases 140-141; Phase 142 is ferro-mcp only (no ferro-stripe release).

**Key risks:**
1. **Idempotency semantics** (MEDIUM): must guarantee exactly-once application of state effects even under concurrent dispatchers. Solved by DB-level unique constraint on the `event_id` column inside the app-implemented log, plus handler-side state-conflict errors.
2. **Event typing maintenance** (LOW): each new Stripe event type needs a typed struct + parser + dispatcher wiring. Documented in module comments; minor ongoing work.
3. **Consumer migration** (LOW): the current `ferro-stripe` consumer is gestiscilo; no external consumers pre-1.0. Breaking changes are absorbed in-workspace.

#### Phases

- [x] **Phase 140: Core reshape** — module tree + `CheckoutBuilder`/`CheckoutIntent` + `ProcessedEventLog`/`MemoryProcessedLog` + remove `connect::*`/`subscription::*` + `Stripe::with(key)`. `ferro-stripe 0.4.0`. (completed 2026-04-20)
- [x] **Phase 141: Protocol uplift** — typed events (drop `event_json`), `SyncDispatcher` as sole handler registry, queue path opt-in with `Arc<SyncDispatcher>`, all 5 new event types, golden-JSON fixtures. `ferro-stripe 0.5.0`. (completed 2026-04-20)
- [ ] **Phase 142: ferro-mcp parity** — update `stripe_webhook_events` and `stripe_config_status` for capability-axis module tree and `SyncDispatcher` handler discovery.

#### Phase Details

### Phase 140: Core reshape

**Goal:** Replace the product-axis module tree with the capability-axis tree and land three new API surfaces in one coherent release: `CheckoutBuilder`/`CheckoutIntent`, `ProcessedEventLog`/`MemoryProcessedLog`, and `Stripe::with(key)`. Remove all product-axis modules and the stubbed `is_processed` free fn. This is the meaningful unit — neither `CheckoutBuilder` nor `ProcessedEventLog` are useful without the module layout that contextualises them.

**Depends on:** nothing (first phase of milestone).

**Success Criteria:**
  1. Module tree matches design §3.1: `checkout.rs`, `refund.rs`, `account.rs`, `webhook/{verify,events,sync,queue}`, `idempotency.rs`, `client.rs`. `connect/` and `subscription/` directories deleted.
  2. `idempotency.rs` declares `#[async_trait] pub trait ProcessedEventLog { async fn try_mark_processed(&self, event_id: &str) -> Result<bool, Error>; }`
  3. `MemoryProcessedLog` impl (DashMap-backed) returns `Ok(true)` on first insert, `Ok(false)` on subsequent calls with the same `event_id`
  4. Module doc comment ships the recommended SQL schema: `CREATE TABLE stripe_processed_events (event_id TEXT PRIMARY KEY, event_type TEXT NOT NULL, received_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP)`
  5. `CheckoutBuilder::new(Mode::Payment|Subscription)` with combinators `.line_item`, `.success_url`, `.cancel_url`, `.metadata`, `.customer_email`, `.customer_email_opt`, `.destination(account_id, fee_cents)`, `.idempotency_key` (required before `create()`)
  6. `CheckoutBuilder::create()` returns `CheckoutIntent { session_id, url, expires_at, idempotency_key }`; returns `Err(Error::MissingIdempotencyKey)` when key not set
  7. `refund.rs` ships `create(charge_id, amount_cents, idempotency_key, reason) -> Refund` and `retrieve(refund_id) -> Refund`
  8. `account.rs` consolidates `create_account`, `create_link`, `retrieve_account` (new), `billing_portal_url` (moved from `subscription::checkout`)
  9. `client.rs::Stripe::with(&str) -> Client` scoped override alongside the static default
  10. `webhook::is_processed` free fn removed; no callers remain
  11. All pub re-exports in `lib.rs` updated; no dead imports
  12. Unit tests: `MemoryProcessedLog` true-then-false contract; concurrent `try_mark_processed` from 2 tokio tasks applies once
  13. `ferro-stripe 0.4.0` released; `cargo test --all-features` + `cargo clippy --all -- -D warnings` pass
  14. CHANGELOG entry documents every breaking change and migration path

**Plans:** 5/5 plans complete

- [x] 140-01-PLAN.md — Foundation: dashmap dep, Error::MissingIdempotencyKey, Stripe::with(key) (Wave 1)
- [x] 140-02-PLAN.md — idempotency.rs: ProcessedEventLog trait + MemoryProcessedLog + tests (Wave 1)
- [x] 140-03-PLAN.md — New capability files: checkout.rs (CheckoutBuilder/CheckoutIntent), refund.rs, account.rs (Wave 2)
- [x] 140-04-PLAN.md — Module restructure: delete connect/, subscription/, handler.rs; extract webhook/verify.rs; add sync/queue stubs; rewrite lib.rs (Wave 3)
- [x] 140-05-PLAN.md — Framework consumer migration + CHANGELOG + ferro-stripe 0.4.0 version bump (Wave 4)

### Phase 141: Protocol uplift

**Goal:** Drop `event_json: String` from all typed event structs and ship `SyncDispatcher` as the default webhook path. Stripe event structs do not implement `ferro_events::Event` — `SyncDispatcher` is the sole handler registry for both dispatch paths. `ProcessStripeWebhook` (queue path) accepts `Arc<SyncDispatcher>` and delegates to it; consumers register handlers once and both paths share that registry. Ship all five new event types in the same release — they follow the identical pattern, and shipping them alongside the framework that handles them is the natural unit.

**Depends on:** Phase 140 (module layout in place).

**Success Criteria:**
  1. All existing event structs (`StripeCheckoutCompleted`, `StripeSubscriptionUpdated`, `StripeSubscriptionDeleted`, `StripeInvoicePaid`, `StripeConnectPaymentSucceeded`) carry fully-parsed fields; `event_json` field removed; none implement `ferro_events::Event`
  2. `StripeEvent` marker trait: `pub trait StripeEvent: Send + Sync + 'static { fn from_raw(event: &stripe::Event) -> Option<Self> where Self: Sized; }`
  3. `SyncDispatcher` in `webhook/sync.rs` with `new() -> Self`, `on<E: StripeEvent, H, Fut>(handler) -> Self`, `async dispatch(event: stripe::Event) -> Result<(), Error>`
  4. `dispatch` returns `Err` when any handler returns `Err`; unknown event types are logged and return `Ok(())` (no-op)
  5. `ProcessStripeWebhook` moves to `webhook/queue.rs`; accepts `Arc<SyncDispatcher>`; calls `dispatcher.dispatch(event)` — no separate handler registration on the queue path
  6. Doc comments guide consumers: sync path for payment-correctness events, queue path for eventual-consistency events
  7. `StripeCheckoutExpired` (event `checkout.session.expired`) carries `event_id`, `session_id`, `metadata`
  8. `StripePaymentIntentFailed` (event `payment_intent.payment_failed`) carries `event_id`, `payment_intent_id`, `session_id` (Option), `failure_code`, `failure_message`, `metadata`
  9. `StripeChargeRefunded` (event `charge.refunded`) carries `event_id`, `charge_id`, `payment_intent_id`, `amount_refunded_cents`, `metadata`
  10. `StripeChargeDisputeCreated` (event `charge.dispute.created`) carries `event_id`, `charge_id`, `payment_intent_id`, `dispute_reason`, `amount_cents`
  11. `StripeConnectAccountUpdated` (event `account.updated`) carries `event_id`, `account_id`, `charges_enabled`, `payouts_enabled`, `details_submitted`
  12. Golden-JSON fixtures per event type in `tests/fixtures/stripe_events/`; parser-contract test asserts field-by-field match
  13. Unit tests: `Err` handler bubbles up; `Ok` path; unknown event no-op; dispatcher thread-safe across `Arc`
  14. `ferro-stripe 0.5.0` released; workspace CI green

**Plans**: TBD

### Phase 142: ferro-mcp parity

**Goal:** Update ferro-mcp introspection tools to reflect the capability-axis module tree and `SyncDispatcher` handler discovery. After Phase 141, `stripe_webhook_events` scans for the wrong patterns (`ferro_events::Event` listener impls that no longer exist on Stripe events) and `stripe_config_status` checks a scaffold layout that no longer matches the module structure. Since ferro-mcp is the surface agents read to author applications, a stale introspection layer contradicts the framework's core proposition.

**Depends on:** Phase 141 (final ferro-stripe shape in place).

**Success Criteria:**
  1. `stripe_webhook_events` discovers `SyncDispatcher::on::<E, _>(handler)` registrations in app source, not `ferro_events` listener impls
  2. `stripe_config_status` reports scaffold structure matching the capability-axis tree (`checkout.rs`, `refund.rs`, `account.rs`, `webhook/`)
  3. `stripe_subscription_info` tool updated or retired if the subscription module no longer warrants a distinct introspection surface
  4. MCP tool descriptions updated to match the `SyncDispatcher` dispatch model
  5. `ferro mcp` JSON schema regenerated for any changed tool signatures
  6. Workspace CI green; `ferro-mcp` version bumped

**Plans:** 2 plans

- [x] 142-01-PLAN.md — Update ferro-mcp/src/tools/stripe.rs: WebhookEventInfo + StripeConfigStatus structs, walkdir-based scan, dual-regex (closure + turbofish), capability-axis fields, tests
- [x] 142-02-PLAN.md — Update ferro-mcp/src/service.rs MCP tool descriptions for the three Stripe tools; bump workspace version 0.2.2 → 0.2.3

---

### ✅ v11.7 Tailwind Static CSS Pipeline (Phase 143 — Shipped 2026-04-21)

Phase 143 — full details archived in [milestones/v11.7-ROADMAP.md](milestones/v11.7-ROADMAP.md).

---

### ✅ v11.8 HttpResponse Header Semantics Fix (Phase 143.1 — Shipped 2026-04-21)

**Milestone Goal:** Fix the actual Safari "raw text" bug that phase 143 tried to solve. `HttpResponse::header(name, value)` pushes rather than replaces, so every `JsonUi::render` response emits a double `Content-Type` header (`text/plain` from `HttpResponse::text()` plus the intended `text/html; charset=utf-8` from the follow-up `.header()` call). Cloudflare comma-joins them; Safari reads the first value and renders the HTML source as plain text.

**Source:** Live Chrome DevTools MCP capture of gestiscilo.it/accedi, 2026-04-21, confirming `content-type: text/plain,text/html; charset=utf-8` on the wire after phase 143 shipped.

**Relationship to phase 143:** The gestiscilo Safari field report that drove phase 143 was misdiagnosed as a `@tailwindcss/browser@4` failure. The runtime never executed because Safari interpreted the whole response as plain text. Phase 143's static CSS pipeline is still a net architectural improvement (dev-only CDN out of prod, no WASM download, no third-party dependency), but it did not and could not fix the reported bug. Do not roll back 143; do capture the misdiagnosis in its retrospective.

**What changes:**
- `framework/src/http/response.rs::header()` — replace semantics, case-insensitive name match.
- `framework/src/http/response.rs::append_header()` — new method preserving current push behaviour, used only by `cookie()`.
- `framework/src/http/response.rs::cookie()` — routed through `append_header` to preserve multi-cookie support.
- No API removal. No behaviour change for any other method.

**What stays:**
- `Vec<(String, String)>` header storage — fine once replace-semantics lands.
- All existing constructors (`text`, `json`, `bytes`, `file_download`, etc.) — their prepopulated Content-Types now behave correctly when overridden.

**Success criteria:**
1. `HttpResponse::text("x").header("Content-Type", "text/html").headers()` returns exactly one Content-Type entry equal to `text/html`.
2. Multi-cookie responses still emit multiple `Set-Cookie` headers on the wire.
3. Case-insensitive replace: `.header("Content-Type", ...)` replaces a prior `"content-type"`.
4. gestiscilo.it `/accedi` wire response shows `content-type: text/html; charset=utf-8` — no comma, no `text/plain`.
5. Safari desktop + iOS render gestiscilo.it pages as styled HTML.

**Release:** ferro 0.2.5 patch (workspace version verified as 0.2.4 at plan-time — one-patch-step bump). Downstream: gestiscilo bumps `ferro_version` to 0.2.5 and `cargo update`.

#### Phases

**Plans:** 1 plan

- [x] 143.1-01-PLAN.md — Apply replace-semantics to `HttpResponse::header()`, add `append_header()` escape hatch, reroute `cookie()`, expose `headers()` accessor, update docstrings and stale json_ui comment, add 5 unit tests, bump workspace version 0.2.4 → 0.2.5.

---

### ✅ v12.0 JSON-UI v2 — Spec-Driven Rendering (Shipped 2026-05-19)

**Milestone Goal:** Pivot ferro-json-ui from nested component trees built in Rust to flat, JSON-first specs with JSON Schema as the validation contract. AI generates specs constrained by schema; developers write static JSON files validated by the same schema. Handlers become data-only providers.

**Context:** Three proven approaches inform this design:
- **Vercel json-render** (13k+ GitHub stars, Jan 2026): flat element maps, Zod-defined catalogs, AI-constrained generation. Validates the AI → JSON → UI thesis. Early issues: infinite re-render bugs, tight Zod coupling, expensive model dependency.
- **JSON Forms** (jsonforms.io): two-schema separation — JSON Schema for data, UI Schema for layout hints. Framework-agnostic core with pluggable renderers. Pain points: slow array rendering, limited layout types (4), low maintenance velocity.
- **react-jsonschema-form** (rjsf): JSON Schema → auto-generated forms with uiSchema overrides. Pain points: catastrophic performance with large oneOf (86 variants freezes UI), full re-render on every keystroke, schema version lag.
- **Production SDUI** (Airbnb, DoorDash, Lyft): GraphQL unions or protobuf for component typing, 3-tier hierarchy (Screen > Section > Component), version fragmentation is the hardest operational problem.

Ferro adopts the structural patterns (flat element map, props separation, formalized catalog with JSON Schema export) while keeping its server-authoritative model. The key enrichment over the original plan: **JSON Schema becomes the single source of truth** for validation, AI generation constraints, and tooling interop.

**Key risks identified by domain research:**
1. **Inner platform effect** (HIGH): Expression system must stay minimal (`$data` + `$template` only). Every SDUI system warns about schemas becoming programming languages.
2. **AI schema complexity** (HIGH): 36-component oneOf is too complex for LLM structured output. Two-tier strategy: concise prompt + per-component schemas.
3. **Manual JsonSchema impls** (HIGH): Component enum + recursive Props structs need ~200 lines of manual `JsonSchema` implementations.
4. **Schema size** (LOW): Estimated 40-80 KB for full catalog — acceptable for validation, too large for AI prompts.

**Rust ecosystem:** schemars 1.2.0 (generation, already in Cargo.lock) + jsonschema 0.45.0 (validation, to add). Both target JSON Schema 2020-12, no known incompatibilities. Compiled validators for zero per-request overhead.

**What changes:**
- Spec format: flat `elements` map + `root` key (replaces nested `Vec<ComponentNode>`)
- Props: separate `props` object per element (replaces flattened custom serialization)
- Catalog: machine-readable struct with `prompt()`, `validate()`, `json_schema()` (replaces `COMPONENT_CATALOG` const string)
- JSON Schema contract: per-component schemas via `schemars::JsonSchema` derives, full spec schema, standalone export
- Expressions: `$data` and `$template` resolved server-side at render time (enriches current `data_path`)
- Schema-driven projection: `Spec::from_service_def()` generates v2 specs from ServiceDef using JSON Schema type mapping
- Page loader: framework loads JSON spec files, merges handler data, renders HTML
- AI constraints: `catalog.prompt()` embeds JSON Schema for structured output; `catalog.validate()` uses `jsonschema` crate
- **Clean break**: v1 types (`JsonUiView`, nested `ComponentNode`) are removed entirely — no backward compatibility layer

**What stays:**
- Server-side HTML + Tailwind rendering (zero client JS runtime)
- Server-authoritative state (no client-side state management)
- Action → handler POST model (server round-trips)
- SSE for live updates
- Compile-time Rust type safety (bonus layer over runtime validation)
- Layout system (dashboard chrome is first-class)

## Phases

- [x] **Phase 115: Spec v2 Data Structures** — New `Spec` type with flat element map, props separation, clean break from v1 (shipped 2026-05-19 with v12.0)
- [x] **Phase 116: Flat Element Renderer** — Update render pipeline to walk flat element map via ID lookups
- [x] **Phase 117: Catalog & JSON Schema** — Machine-readable `Catalog` with per-component JSON Schema, full spec schema, validation, and `ferro json-ui:schema` CLI export (completed 2026-04-18)
- [x] **Phase 117.1: Schema-Driven Projections** — `Spec::from_service_def()` generates v2 specs from ServiceDef using JSON Schema type mapping, replacing hardcoded `field_to_input()` mappings (completed 2026-04-18)
- [x] **Phase 118: Server-Side Expressions** — `$data` path resolution and `$template` string interpolation at render time (completed 2026-04-19)
- [x] **Phase 119: Page Loader** — Framework loads JSON spec files, merges handler data, integrates with layouts (shipped 2026-05-19 with v12.0)
- [x] **Phase 120: CLI & MCP Updates** — Update `make:json-view` and MCP tools for v2 format with JSON Schema as structured output constraint (completed 2026-04-21)
- [x] **Phase 121: Documentation & Field Test** — Update all JSON-UI docs, convert one gestiscilo page as proof of concept (completed 2026-05-15)

#### Phase Details

### Phase 115: Spec v2 Data Structures
**Goal**: Replace v1 types with the v2 spec format — flat element map, props separation, manual `JsonSchema` impl for Component enum, clean break
**Depends on**: Nothing (first phase of milestone)
**Requirements**: SPEC-01, SPEC-02, SPEC-03, SPEC-04
**Caveats** (from domain research):
  - Component enum has custom ser/de (not `#[serde(tag = "type")]`), so `#[derive(JsonSchema)]` won't work. Need manual impl building `oneOf` with `"type"` discriminator const. ~200 lines.
  - ~10 Props structs containing recursive `Vec<ComponentNode>` currently skip `JsonSchema` derive. Must add manual impls using `$ref: "#"` for self-references (schemars 1.x handles this).
  - Max nesting depth: enforce 3 levels (Screen > Section > Component) — matches Airbnb/DoorDash/Lyft production patterns.
**Success Criteria** (what must be TRUE):
  1. `Spec` struct exists with `root: String`, `elements: HashMap<String, Element>`, `title`, `layout`, `data`
  2. `Element` struct has `type_name`, `props: serde_json::Value`, `children: Vec<String>`, `action`, `visible`
  3. `Spec::from_json()` parses flat JSON specs and round-trips cleanly (serialize → deserialize = identity)
  4. `JsonUiView`, nested `ComponentNode`, and `Vec<ComponentNode>` patterns are deleted — clean break, no v1 types remain
  5. Schema version is `ferro-json-ui/v2`
  6. All Component variants and Props structs implement `JsonSchema` (manual impls where derive is blocked by custom ser/de or recursion)
  7. Spec nesting depth is validated: reject specs deeper than 3 levels

**Plans:** 5 plans

Plans:
- [x] 115-01-PLAN.md — Create spec.rs (Spec, Element, builders, SpecError, from_json, validation) + tests/fixtures + round_trip.rs + reject.rs — additive re-exports
- [x] 115-02-PLAN.md — Delete v1 types (JsonUiView, Component, ComponentNode, PluginProps, view.rs) and rewrite render.rs / resolve.rs / projection/mod.rs / lib.rs for v2
- [x] 115-03-PLAN.md — Migrate framework/src/json_ui/mod.rs (JsonUi::render(&Spec, ...)) + framework/src/lib.rs re-exports + port ~30 inline tests
- [x] 115-04-PLAN.md — Migrate ferro-mcp (8 files) + ferro-cli templates (3 files) to v2 syntax — workspace-wide build green
- [x] 115-05-PLAN.md — Full workspace verification: fmt + clippy + test all green; 7 ROADMAP success criteria confirmed


### Phase 116: Flat Element Renderer
**Goal**: New render pipeline that walks the flat element map by ID lookups, replacing the recursive tree walker
**Depends on**: Phase 115
**Requirements**: RENDER-01, RENDER-02, RENDER-03
**Success Criteria** (what must be TRUE):
  1. `render_spec_to_html(spec, data)` renders all component types from flat element map
  2. Element ID lookup handles missing children gracefully (skip + warn, don't panic)
  3. Action resolution works on flat elements (handler → URL via callback)
  4. Visibility evaluation works on flat elements (conditional rendering)
  5. Plugin components render correctly in v2 specs
  6. Old `render_to_html(view, data)` function is deleted

### Phase 117: Catalog & JSON Schema
**Goal**: Replace `COMPONENT_CATALOG` const string with a machine-readable `Catalog` backed by JSON Schema. Each component's props schema is derived from `schemars::JsonSchema` impls (Phase 115). The catalog validates specs, generates LLM prompts, and exports standalone schema files.
**Progress**: Plan 02/07 complete — BUILTIN_SPECS 39 entries, Catalog::build() discovery impl, sanitize_schema(), 12 unit tests passing
**Depends on**: Phase 116
**Requirements**: CAT-01, CAT-02, CAT-03, CAT-04, SCHEMA-01, SCHEMA-02, SCHEMA-03
**Caveats** (from domain research):
  - Full catalog schema (36-component oneOf) estimated at 40-80 KB — too large for AI system prompts. `catalog.prompt()` must emit a concise text summary, NOT the raw JSON Schema.
  - `jsonschema` crate doesn't optimize oneOf with discriminators (checks sub-schemas sequentially). Add pre-dispatch by `"type"` string for O(1) per-element validation.
  - Compile the schema validator once at startup via `jsonschema::validator_for()`, reuse for all requests. No per-request compilation.
  - AI models work reliably with per-component schemas but may produce malformed output when given 30+ component oneOf. Two-tier strategy: concise prompt + per-component structured output.
**Success Criteria** (what must be TRUE):
  1. `Catalog::build()` auto-discovers all Component variants with descriptions and JSON Schema per props struct
  2. `catalog.prompt()` generates a concise text system prompt summarizing components, props, and constraints — NOT the raw JSON Schema (too large for AI context)
  3. `catalog.validate(&spec)` validates specs using the `jsonschema` crate with compiled validator — returns typed errors for unknown component types, invalid props, missing required fields. Pre-dispatches by `"type"` string before full schema validation.
  4. `catalog.json_schema()` exports the complete JSON Schema document for the full v2 spec format (root + elements + all component types via `oneOf`)
  5. `catalog.component_schema("Card")` returns the JSON Schema for a single component's props — for targeted AI structured output generation
  6. `ferro json-ui:schema` CLI command exports the spec schema to stdout or file — consumable by external tools and IDEs
  7. `COMPONENT_CATALOG` const string is replaced by `catalog.prompt()` output
  8. Schema validator is compiled once (e.g., in `Catalog::build()`) and reused — no per-validation compilation

### Phase 117.1: Schema-Driven Projections
**Goal**: Bridge ferro-projections and ferro-json-ui v2 — generate v2 specs directly from ServiceDef definitions using JSON Schema type mappings instead of hardcoded `field_to_input()` / `field_to_column()` functions
**Depends on**: Phase 117
**Requirements**: PROJ-01, PROJ-02, PROJ-03
**Success Criteria** (what must be TRUE):
  1. `Spec::from_service_def(service, intents, ctx)` produces a valid v2 spec from a ServiceDef
  2. `DataType` + `FieldMeaning` → component selection uses the catalog's JSON Schema (not hardcoded match arms)
  3. Intent-to-layout mapping produces flat element specs (Browse → table layout, Collect → form layout, etc.)
  4. Output validates against `catalog.json_schema()` — projections and catalog are consistent by construction (two-pass: generate then validate)
  5. `render/json_ui.rs` (v1 JsonUiRenderer) and `render/field_map.rs` are replaced by the new schema-driven pipeline

**Plans:** 3/3 plans complete

Plans:
- [x] 117.1-01-PLAN.md — Foundational types: ProjectionError enum + MEANING_COMPONENT_TABLE (lookup_meaning + typed Props helpers + drift guard) + intent_layout (default_template + pick_intent_template)
- [x] 117.1-02-PLAN.md — Spec::from_service_def orchestrator in builder.rs: slot-based display pipeline, Input-mode Form collapse (D-11), system-field filter (D-10), template override (D-05), Catalog::validate two-pass (D-06)
- [x] 117.1-03-PLAN.md — Clean break: delete field_map.rs + relationship_map.rs, slim projection/mod.rs, rewire JsonUiRenderer::render as one-line delegate, add ProjectionError to lib.rs re-exports, full workspace quality gate

### Phase 118: Server-Side Expressions
**Goal**: Add `$data` and `$template` expression types that resolve against handler data at render time. Hard cap: ONLY these two expression types. No `$if`, `$for`, `$state`, `$bind`.
**Depends on**: Phase 116
**Requirements**: EXPR-01, EXPR-02, EXPR-03
**Caveats** (from domain research):
  - Inner platform effect is the #1 strategic risk in SDUI. Every production SDUI system (Airbnb, DoorDash, Lyft) warns about schemas evolving into programming languages. `$data` and `$template` are the correct scope — resist pressure to add conditionals or loops.
  - Binding expressions (`{{query.data}}`) used by Appsmith/ToolJet/Retool are more flexible but harder to validate at compile time. Ferro's `$data`/`$template` approach is deliberately simpler.
**Success Criteria** (what must be TRUE):
  1. `{"$data": "path/to/value"}` in any props field resolves against `spec.data` before rendering
  2. `{"$template": "Hello, {user.name}!"}` interpolates data paths within strings
  3. Expressions work in all props positions (string, number, boolean values)
  4. Missing data paths resolve to `null`/empty — never panic
  5. Expressions are evaluated before component rendering, so renderers receive resolved concrete values
  6. No other expression types exist — only `$data` and `$template`. This is a hard architectural constraint, not a backlog item.

**Plans:** 2/2 plans complete

Plans:
- [x] 118-01-PLAN.md — Create expression.rs resolver module (resolve_expressions, $data / $template helpers, 28 unit tests) + register in ferro-json-ui/src/lib.rs
- [x] 118-02-PLAN.md — Wire resolve_expressions into framework JsonUi::resolve + JsonUi::resolve_with_errors + 4 end-to-end integration tests covering every public render path

### Phase 119: Page Loader
**Goal**: Framework-level support for loading JSON spec files and merging with handler-provided data
**Depends on**: Phase 118
**Requirements**: LOAD-01, LOAD-02, LOAD-03
**Success Criteria** (what must be TRUE):
  1. `Spec::from_file("path/to/page.json")` or `include_str!()` loads and parses specs
  2. Loaded specs are validated against `catalog.json_schema()` at load time using the compiled validator — invalid specs fail fast with clear errors
  3. Handler data merges into `spec.data` (handler data takes precedence over spec defaults)
  4. Layout data (sidebar, header, sse_url) injects automatically for dashboard-layout specs
  5. Loaded specs are cached (compiled once, reused across requests)
  6. Dev mode: file watcher reloads specs on change (hot reload without recompilation)

**Plans:** 0/3 plans

Plans:
- [x] 119-01-PLAN.md — Add Spec::merge_data consuming-builder method (shallow top-level merge, Null→Object init, non-Object ignored)
- [x] 119-02-PLAN.md — Create ferro-json-ui/src/loader.rs (LoadError enum, global spec cache, load_cached with dev-mode mtime invalidation) + lib.rs re-exports
- [ ] 119-03-PLAN.md — Add JsonUi::render_file to framework (load_cached + merge_data + delegate to render_with_config; dev/prod-gated 500 bodies)

### Phase 120: CLI & MCP Updates
**Goal**: Update all AI-facing tools to generate v2 specs using two-tier AI strategy (concise prompt + per-component structured output)
**Depends on**: Phase 117, Phase 119
**Requirements**: TOOL-01, TOOL-02, TOOL-03, TOOL-04
**Caveats** (from domain research):
  - Two-pass AI generation reduces hallucination: generate description first, then structured spec. v0.dev and Lovable both use this pattern.
  - LLMs hallucinate to fill arrays — may generate unnecessary components. Validate AI output against schema and flag suspiciously large specs.
  - Token overhead: JSON output costs ~2-3x tokens vs free text. Per-component schema keeps overhead manageable.
**Success Criteria** (what must be TRUE):
  1. `ferro make:json-view` generates v2 flat specs using two-pass generation (describe → structure)
  2. MCP `json_ui_generate` tool uses `catalog.prompt()` for concise context and `catalog.component_schema()` for per-component structured output
  3. MCP `json_ui_catalog` tool exposes JSON Schema per component (replaces text-only catalog inspection)
  4. MCP `json_ui_inspect` tool works with v2 format and reports validation errors against schema
  5. All code templates in ferro-mcp use v2 spec format
  6. No references to v1 types remain in CLI or MCP code
  7. Generated specs are validated against `catalog.json_schema()` before being returned to the user

**Plans:** 5 plans

Plans:
- [ ] 120-01-PLAN.md — MCP json_ui_catalog: add json_schema + component_schemas fields; generation_context.json_ui_view rewritten to v2 JSON (Wave 1)
- [ ] 120-02-PLAN.md — MCP json_ui_generate VIEW_EXAMPLE / ViewConventions / list_existing_views + code_templates json_view_templates rewritten to v2 (+ json_view_handler) (Wave 1)
- [ ] 120-03-PLAN.md — MCP json_ui_inspect rewritten to walk src/views/*.json; BUILTIN_TYPES removed; inspect_component uses global_catalog (Wave 1)
- [ ] 120-04-PLAN.md — ferro-cli ai.rs: call_anthropic_plain / call_anthropic_structured / build_json_view_pass1 / build_json_view_pass2; build_view_context deleted (Wave 1)
- [ ] 120-05-PLAN.md — ferro-cli make_json_view + templates/make.rs: .json output, two-pass orchestration, Spec::from_json + catalog.validate with static fallback (Wave 2, depends on 04)

### Phase 121: Documentation & Field Test
**Goal**: Complete docs rewrite for v2 and validate with a real gestiscilo page conversion
**Depends on**: Phase 120
**Requirements**: DOC-01, DOC-02, FIELD-01
**Success Criteria** (what must be TRUE):
  1. All JSON-UI documentation pages rewritten for v2 spec format with flat element examples — no v1 references remain
  2. JSON Schema export documented with usage examples (IDE validation, external tool integration, AI structured output)
  3. Expression system documented with explicit "hard cap" rationale — only `$data` and `$template`, with explanation of why no `$if`/`$for`
  4. One gestiscilo dashboard page (e.g., pagamenti) converted from Rust component tree to JSON spec file — handler reduced to data-only
  5. Converted page renders identically to the Rust-built version

#### Progress

**Execution Order:**
Phases execute in order: 115 → 116 → 117 → 117.1 → 118 (parallel with 117) → 119 → 120 → 121

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 115. Spec v2 Data Structures | 5/5 | Complete   | 2026-04-18 |
| 116. Flat Element Renderer | 6/6 | Complete   | 2026-04-18 |
| 117. Catalog & JSON Schema | 7/7 | Complete    | 2026-04-18 |
| 117.1. Schema-Driven Projections | 3/3 | Complete    | 2026-04-18 |
| 118. Server-Side Expressions | 2/2 | Complete    | 2026-04-19 |
| 119. Page Loader | 2/3 | In Progress|  |
| 120. CLI & MCP Updates | 5/5 | Complete    | 2026-04-21 |
| 121. Documentation & Field Test | 6/6 | Complete    | 2026-05-15 |

**Plans:**
6/6 plans complete
- [x] 121-01-PLAN.md — Add JsonUi::render_file to framework (Wave 1, FIELD-01 blocker)
- [x] 121-02-PLAN.md — Rewrite getting-started.md, actions.md, features/json-ui.md (Wave 2, DOC-01)
- [x] 121-03-PLAN.md — Rewrite components.md and data-binding.md (Wave 2, DOC-01)
- [x] 121-04-PLAN.md — Rewrite layouts.md and plugins.md (Wave 2, DOC-01)
- [x] 121-05-PLAN.md — Create expressions.md, json-schema.md, update SUMMARY.md (Wave 3, DOC-02)
- [x] 121-06-PLAN.md — Field test: pagamenti.json + handler + route (Wave 4, FIELD-01)

**v12.0 scope is held firm.** No expansion beyond the 8 phases above. The projection / intent abstraction already exists in v9.0 ferro-projections; v12.0 refines the rendering target.

---

## v1.0 Criteria

Ferro v1.0 is the first release where the framework is considered feature-complete for its target domain. No target date.

**Modality:**
- Visual modality complete (HTML + Tailwind, server-rendered).
- Additional rendering modalities (audio, physical) are out of scope for v1.0.

**Projection / intent validation:**
- Validated through real-world applications and a synthetic catalog of canonical app classes covering the seven intents.

**Quality bars:**
- Conceptual coherence pass complete across all 20 crates.
- Beauty across four dimensions: aesthetic, conceptual, operational, compressive.

---

## Pre-v1.0 Work Items

Concrete items that contribute to v1.0 readiness. Not assigned to specific phases yet.

| Item | Notes |
|------|-------|
| **MCP integration documentation for common AI development environments** | Document how to wire `ferro-mcp` into Claude Code, Cursor, and other agent runtimes that follow the MCP standard. |
| **Audit projection MCP tool descriptions for completeness** | Verify `list_projections`, `inspect_projection`, `render_projection`, `validate_projection`, and `projection_coverage` tool descriptions are complete and accurate enough to author projections without out-of-band guidance. |
| **Improve projection authoring guide via MCP introspection** | Identify gaps in tool descriptions, examples, and field-level documentation that an agent would need to compose a projection cleanly. |
| **Document the agent-assisted deploy workflow end-to-end** | A complete walkthrough of `ferro docker:init` → `ferro do:init` → `ferro doctor` → push, with the role MCP introspection plays at each step. |
| **Projection-driven starter template for `ferro new`** | Add an option to scaffold a project that exercises the projection / intent system end-to-end as the default example, alongside the current scaffold. |
| **Iteration loop ergonomics for projection-driven development** | Investigate the change → rebuild cycle for projection-driven apps. Identify whether incremental compilation, hot reload, or runtime spec swapping reduces friction. |
| **`ferro doctor` multi-bin support** | `db_connection` and `migrations_pending` checks should automatically pass `--bin <pkg>` for workspaces without `default-run`. Tracked in `.planning/phases/122.2-deploy-simplification/122.2-VERIFICATION.md`. |

---

## Experiments

Lightweight investigations queued without commitment to a phase. Each is intended to inform a future design decision rather than ship a feature.

| Experiment | Cost | Goal |
|------------|------|------|
| **Intent vocabulary cross-modality sketch** | hours | Take one intent (e.g. `Process`) and one real feature using it. Sketch on paper how the same feature would be expressed as a single-screen mobile flow, a voice interaction, and a CLI command. Identify which fields the projection needs that it does not currently have, and which existing fields stop being meaningful. Inform any future intent vocabulary revision. |

---

## Future Milestones (v2.0+)

Items intentionally out of v1.0 scope. No phase numbers, no dates.

| Item | Target | Notes |
|------|--------|-------|
| **Multimodal generation exploration** | exploratory | Evaluate whether the seven intents (Browse, Focus, Collect, Process, Summarize, Analyze, Track) generalize cleanly to non-visual rendering targets. Inform any required revision of the intent vocabulary. |
| **Audio modality renderer** | v2.0+ | Render projections as voice / conversational interfaces. May require intent vocabulary revision. |
| **Physical modality** | v3.0+ | Haptic, gesture, and tangible rendering targets. |

---

## Design Principles

Operating principles applied across every phase. See [`.planning/VISION.md`](VISION.md) for the full design philosophy.

- **Substance-first investment ordering:** compressive → operational → conceptual → aesthetic.
- **Continuous conceptual coherence:** every phase pays a coherence cost against the existing 20 crates at write-time; no deferred cleanup milestones.
- **Validation through real-world applications and synthetic catalogs:** the projection / intent system is iterated against both.

---

## Completed Milestones

<details>
<summary>✅ v10.0 JSON-UI Visual Overhaul (Phases 102-107) — SHIPPED 2026-03-26</summary>

**Milestone Goal:** Reach professional visual quality across all JSON-UI components.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 102. Foundation | 2/2 | Complete | 2026-03-24 |
| 103. Surface Elevation | 2/2 | Complete | 2026-03-25 |
| 104. Typography Scale | 1/1 | Complete | 2026-03-25 |
| 105. Form Polish | 1/1 | Complete | 2026-03-25 |
| 106. Interactive States | 1/1 | Complete | 2026-03-25 |
| 107. Component Details | 1/1 | Complete | 2026-03-25 |

**Total:** 6 phases, 8 plans

**What was built:**
- Inter Variable font via Bunny Fonts CDN with Tailwind v4 --font-sans token fix (Phase 102)
- Three-tier surface elevation (background → surface → card) with WCAG 4.5:1 dark mode contrast (Phase 103)
- Typography scale: H1/H2 tight, H3 snug, body relaxed line-height (Phase 104)
- Form polish: SVG select chevron, destructive error rings, transitions, disabled states (Phase 105)
- Focus-visible rings and hover states on all interactive elements (Phase 106)
- SVG icons for alerts/bell/breadcrumb/collapsible, shimmer animation, semibold tabs (Phase 107)

[Full details →](milestones/v10.0-ROADMAP.md)

</details>

<details>
<summary>✅ v8.1 API DX Polish (Phase 83) — SHIPPED 2026-02-28</summary>

**Milestone Goal:** Close the DX gaps between `ferro make:api` scaffold and a working MCP integration. Add API key CLI command, post-scaffold guidance, model/field selection, and x-mcp route-level customization.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 83. API DX Polish | 5/5 | Complete | 2026-02-28 |

**Total:** 1 phase, 5 plans

**What was built:**
- `ferro make:api-key` CLI command: generates API keys with SHA-256 hashing, SQL/Rust code snippets (Plan 01)
- Route-level x-MCP builder API: .mcp_tool_name(), .mcp_description(), .mcp_hint(), .mcp_hidden() on RouteDefBuilder and GroupDef with group-level defaults (Plan 02)
- Field exclusion in make:api: --exclude, --include-all flags, auto-excludes 8 sensitive field patterns (Plan 03)
- `ferro api:check` CLI command: validates server, OpenAPI spec, API key auth, prints ferro-api-mcp config (Plan 04)
- Enhanced post-scaffold guidance: generated files list, setup steps, MCP config snippets for Claude Desktop/Code (Plan 05)
- Documentation updates for all new features in docs/src/features/api.md and api-mcp.md (Plan 05)

</details>

<details>
<summary>✅ v8.0 Consumer MCP — OpenAPI Bridge (Phases 79-82) — SHIPPED 2026-02-28</summary>

**Milestone Goal:** Let consumers interact with any Ferro web service through natural language via a standalone MCP server that auto-discovers API operations from OpenAPI specs.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 79. ferro-api-mcp Core | 4/4 | Complete | 2026-02-28 |
| 80. x-mcp OpenAPI Extensions | 2/2 | Complete | 2026-02-28 |
| 81. Consumer DX & Polish | 3/3 | Complete | 2026-02-28 |
| 82. End-to-End Validation | 2/2 | Complete | 2026-02-28 |

**Total:** 4 phases, 11 plans

**What was built:**
- ferro-api-mcp standalone binary: fetches OpenAPI spec, parses operations, registers dynamic MCP tools (Phase 79)
- x-mcp OpenAPI extensions: framework emits x-mcp-tool-name/description/hint/hidden, ferro-api-mcp consumes them (Phase 80)
- Consumer DX: startup diagnostics, --dry-run, input validation, categorized errors, setup documentation (Phase 81)
- E2E validation: sample app API layer + 3 integration tests proving full pipeline works (Phase 82)

</details>

<details>
<summary>✅ v7.8 Memory Leak Fixes (Phase 78) — SHIPPED 2026-02-28</summary>

**Milestone Goal:** Fix four unbounded in-memory data structures that grow indefinitely in long-running Ferro servers.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 78. Memory Leak Fixes | 3/3 | Complete | 2026-02-28 |

**Total:** 1 phase, 3 plans

**What was built:**
- Unmatched routes normalized to "UNMATCHED" bucket + MAX_ROUTE_ENTRIES=1000 cap (Plan 01)
- Framework InMemoryCache replaced with moka::sync::Cache — bounded capacity, per-entry TTL, proactive eviction (Plan 02)
- ferro-cache MemoryStore: per-entry TTL fixed, tags deduplicated with HashSet, stale tag cleanup on eviction, counters bounded with moka (Plan 03)

</details>

<details>
<summary>✅ v7.7 Validate & Fix API Scaffold (Phase 77) — SHIPPED 2026-02-28</summary>

**Milestone Goal:** Fix bugs found during Phase 76 audit, add missing tests, and validate end-to-end make:api output compiles.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 77. Validate & Fix API Scaffold | 3/3 | Complete | 2026-02-28 |

**Total:** 1 phase, 3 plans

**What was built:**
- Fixed `.await` on sync `DB::connection()` and `Vec<serde_json::Value>` → typed Resource vec in all templates (Plan 01)
- 43 unit tests for MCP CRUD operations + fixed `per_page=0` producing `LIMIT 0` (Plan 02)
- Fixed 5 template bugs: singular model names, module import paths, From trait compatibility, mod.rs generation (Plan 03)
- 32 regression tests for make:api template validation (Plan 03)
- `ferro make:api` now generates compilable code for real models — 75 total tests

</details>

<details>
<summary>✅ v7.6 Default API Scaffold (Phase 76) — SHIPPED 2026-02-27</summary>

**Milestone Goal:** Scaffold a default API layer that MCP agents can use to manage dashboard data programmatically.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 76. Default API Scaffold | 4/4 | Complete | 2026-02-27 |

**Total:** 1 phase, 4 plans

**What was built:**
- API key auth with SHA-256 hashing and constant-time verification (Phase 76, Plan 01)
- OpenAPI spec builder from route metadata with ReDoc UI (Phase 76, Plan 01)
- MCP CRUD tools: crud_create, crud_list, crud_update, crud_delete (Phase 76, Plan 02)
- `ferro make:api` CLI command scaffolding complete REST API from models (Phase 76, Plan 03)
- Comprehensive documentation and MCP code templates (Phase 76, Plan 04)

</details>

<details>
<summary>✅ v7.5 Type Generator Fix (Phase 75) — SHIPPED 2026-02-27</summary>

**Milestone Goal:** Fix two bugs in Ferro's Inertia scaffolding discovered during mkmenu production deployment.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 75. Inertia Template Fixes | 1/1 | Complete | 2026-02-27 |

**Total:** 1 phase, 1 plan

**What was built:**
- Self-contained TypeScript type generation (no shared.ts circular imports)
- Test file exclusion from Inertia page glob patterns

</details>

<details>
<summary>✅ v7.4 Security Hardening (Phases 72-74) — SHIPPED 2026-02-26</summary>

**Milestone Goal:** Address framework-level security gaps found during mkmenu security audit. Provide safe primitives so apps don't need unsafe workarounds.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 72. Binary Response Type | 1/1 | Complete | 2026-02-26 |
| 73. Security Headers | 2/2 | Complete | 2026-02-26 |
| 74. Session Absolute Expiry | 2/2 | Complete | 2026-02-26 |

**Total:** 3 phases, 5 plans

**What was built:**
- Binary-safe HttpResponse with bytes()/download() constructors (Phase 72)
- SecurityHeaders middleware with OWASP defaults and builder API (Phase 73)
- Dual idle + absolute session timeouts with created_at tracking (Phase 74)
- Auth::logout_other_devices() and invalidate_all_for_user() APIs (Phase 74)
- CLI templates updated with created_at column and SESSION_ABSOLUTE_LIFETIME env var

</details>

<details>
<summary>✅ v7.0 Resend Integration (Phase 68) — SHIPPED 2026-02-25</summary>

**Milestone Goal:** Add Resend as a mail driver in ferro-notifications alongside SMTP, with env-based driver selection.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 68. Resend Mail Driver | 3/3 | Complete | 2026-02-25 |

**Total:** 1 phase, 3 plans

**What was built:**
- Multi-driver mail architecture (MailDriver enum, SmtpConfig, ResendConfig)
- Resend HTTP API transport via reqwest
- Driver-based dispatch (`MAIL_DRIVER=smtp|resend`)
- CLI scaffold templates updated with Resend config
- Documentation updated with driver setup guide
- 23 notification tests passing

</details>

<details>
<summary>✅ v6.1 Fix Known Issues (Phase 67) — SHIPPED 2026-02-24</summary>

**Milestone Goal:** Fix all known issues discovered during production readiness assessment.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 67. Fix Known Issues | 1/1 | Complete | 2026-02-24 |

**Total:** 1 phase, 1 plan

**Issues fixed:**
- COMPONENT_CATALOG drift between CLI and MCP (synced Text element options, added Input.step to CLI, updated Map props in both)
- Flaky validator test `test_validator_custom_attribute` (OnceLock race with translator)
- Flaky lang config test `from_env_reads_env_vars` (env var race between parallel tests)
- Scheduler `.unwrap()` calls replaced with `expect()` + input validation on factory methods
- Clippy `approx_constant` errors in validation test data (3.14 → 3.17)

</details>


<details>
<summary>✅ v6.0 ferro-lang — Localization (Phases 58-66) — SHIPPED 2026-02-13</summary>

**Milestone Goal:** Add localization infrastructure via ferro-lang crate: JSON translations, per-request locale detection, validation message localization, CLI scaffolding.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 58. Core Translator | 1/1 | Complete | 2026-02-13 |
| 59. Config & Error Types | 1/1 | Complete | 2026-02-13 |
| 60. Locale Context | 1/1 | Complete | 2026-02-13 |
| 61. Validation Bridge | 1/1 | Complete | 2026-02-13 |
| 62. Validation Rules Update | 1/1 | Complete | 2026-02-13 |
| 63. Framework Integration | 1/1 | Complete | 2026-02-13 |
| 64. CLI Scaffolding | 1/1 | Complete | 2026-02-13 |
| 65. MCP & Documentation | 2/2 | Complete | 2026-02-13 |
| 66. Tests & Polish | 3/3 | Complete | 2026-02-13 |

**Total:** 9 phases, 11 plans

[Full details →](milestones/v6.0-ROADMAP.md)

</details>

<details>
<summary>✅ v5.1 Housekeeping (Phases 54-57) — SHIPPED 2026-02-13</summary>

**Milestone Goal:** Resolve technical debt and improve project hygiene discovered during v5.0 field test.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 54. Env Example | 1/1 | Complete | 2026-02-13 |
| 55. Split Templates | 2/2 | Complete | 2026-02-13 |
| 56. Update Concerns | 1/1 | Complete | 2026-02-13 |
| 57. Deployment Template Fixes | 1/1 | Complete | 2026-02-13 |

**Total:** 4 phases, 5 plans

[Full details →](milestones/v5.1-ROADMAP.md)

</details>

<details>
<summary>✅ v5.0 Proximity — JSON-UI Field Test (Phases 47-53) — SHIPPED 2026-02-10</summary>

**Milestone Goal:** Build a map-based social network app as the first real-world validation of JSON-UI and v4.0 features.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 47. JSON-UI Map Component | 4/4 | Complete | 2026-02-10 |
| 48. App Scaffold + Auth & Profiles | 3/3 | Complete | 2026-02-10 |
| 49. Map View & Nearby Users | 2/2 | Complete | 2026-02-10 |
| 50. Location Posts & Check-ins | 3/3 | Complete | 2026-02-10 |
| 51. Real-time Presence | 3/3 | Complete | 2026-02-10 |
| 52. Polish & JSON-UI Fixes | 4/4 | Complete | 2026-02-10 |
| 53. Solve Known Issues | 1/1 | Complete | 2026-02-10 |

**Total:** 7 phases, 20 plans

[Full details →](milestones/v5.0-ROADMAP.md)

</details>

<details>
<summary>✅ v4.0 Production Readiness (Phases 38-46) — SHIPPED 2026-02-10</summary>

**Milestone Goal:** Make Ferro production-ready with authentication, API resources, rate limiting, real-time improvements, and stability fixes.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 38. Fix Pre-existing Blockers | 2/2 | Complete | 2026-02-09 |
| 39. Core Authentication | 4/4 | Complete | 2026-02-09 |
| 40. Auth Middleware | 2/2 | Complete | 2026-02-10 |
| 41. API Resources Basics | 3/3 | Complete | 2026-02-10 |
| 42. API Resources Advanced | 3/3 | Complete | 2026-02-10 |
| 43. Rate Limiting | 3/3 | Complete | 2026-02-10 |
| 44. Real-time Improvements | 4/4 | Complete | 2026-02-10 |
| 45. DX Polish | 3/3 | Complete | 2026-02-10 |
| 46. MCP + CLI Updates | 3/3 | Complete | 2026-02-10 |

**Total:** 9 phases, 24 plans

[Full details →](milestones/v4.0-ROADMAP.md)

</details>

<details>
<summary>✅ v3.0 JSON-UI (Phases 23-32) — SHIPPED 2026-02-09</summary>

**Milestone Goal:** Add JSON-based UI rendering as an alternative to Inertia for rapid, beautiful UI without frontend builds.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 23. JSON-UI Schema | 2/2 | Complete | 2026-02-09 |
| 24. Component Catalog | 3/3 | Complete | 2026-02-09 |
| 25. Data Binding | 2/2 | Complete | 2026-02-09 |
| 26. Action System | 2/2 | Complete | 2026-02-09 |
| 27. Validation Integration | 2/2 | Complete | 2026-02-09 |
| 28. HTML Renderer | 2/2 | Complete | 2026-02-09 |
| 29. Layout System | 2/2 | Complete | 2026-02-09 |
| 30. CLI Scaffolding | 2/2 | Complete | 2026-02-09 |
| 31. MCP UI Tools | 3/3 | Complete | 2026-02-09 |
| 32. Documentation | 4/4 | Complete | 2026-02-09 |

**Total:** 10 phases, 24 plans

</details>

<details>
<summary>✅ v2.2 CLI Improvements (Phases 35-37) — SHIPPED 2026-02-09</summary>

**Milestone Goal:** Add CLI commands for common development workflows.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 35. CLI Seed Command | 2/2 | Complete | 2026-02-09 |
| 36. Gitignore Generated Types | 1/1 | Complete | 2026-02-09 |
| 37. Model Update Builder | 2/2 | Complete | 2026-02-09 |

**Total:** 3 phases, 5 plans

[Full details →](milestones/v2.2-ROADMAP.md)

</details>

<details>
<summary>✅ v2.1 Inertia DX & Fixes (Phases 33-34) — SHIPPED 2026-01-17</summary>

**Milestone Goal:** Improve Inertia developer experience and fix documentation issues.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 33. Inertia DX Improvements | 3/3 | Complete | 2026-01-17 |
| 34. Docs URL References | 1/1 | Complete | 2026-01-17 |

**Total:** 2 phases, 4 plans

[Full details →](milestones/v2.1-ROADMAP.md)

</details>

<details>
<summary>✅ v2.0.3 DO Apps Deploy (Phase 22.10) — SHIPPED 2026-01-17</summary>

**Milestone Goal:** Enable one-click deployment to DigitalOcean App Platform with minimal infrastructure configuration.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 22.10 DigitalOcean Apps Deploy | 1/1 | Complete | 2026-01-17 |

**Total:** 1 phase, 1 plan

[Full details →](milestones/v2.0.3-ROADMAP.md)

</details>

<details>
<summary>✅ v2.0.2 Type Generator Fixes (Phases 22.4-22.9) — SHIPPED 2026-01-17</summary>

**Milestone Goal:** Fix type generation issues discovered during adotta-animali port to improve TypeScript integration reliability.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 22.4 Type Generator Fixes | 1/1 | Complete | 2026-01-17 |
| 22.5 Prop Naming Collisions | 1/1 | Complete | 2026-01-17 |
| 22.6 Contract Validation CLI | 1/1 | Complete | 2026-01-17 |
| 22.7 DateTime Handling | 1/1 | Complete | 2026-01-17 |
| 22.8 Nested Types Generation | 1/1 | Complete | 2026-01-17 |
| 22.9 ValidationErrors Type | 1/1 | Complete | 2026-01-17 |

**Total:** 6 phases, 6 plans

[Full details →](milestones/v2.0.2-ROADMAP.md)

</details>

### ✅ v2.0.1 Macro Fix (Complete)

**Milestone Goal:** Fix hardcoded `::ferro_rs::` paths in proc macros to use canonical `ferro::` name.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 22.1 Macro Crate Paths | 3/3 | ✅ Complete | 2026-01-17 |
| 22.2 Simplify Macro Crate Paths | 1/1 | ✅ Complete | 2026-01-17 |
| 22.3 Complete Rebrand | 2/2 | ✅ Complete | 2026-01-17 |

**Total:** 3 phases, 6 plans

<details>
<summary>✅ v2.0 Rebrand (Phases 13-22) — SHIPPED 2026-01-16</summary>

**Milestone Goal:** Rename the framework from "cancer" to "ferro" for crates.io publication and public release.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 13. Rebrand Audit | 1/1 | Complete | 2026-01-16 |
| 14. Core Framework Rename | 1/1 | Complete | 2026-01-16 |
| 15. Supporting Crates Rename | 1/1 | Complete | 2026-01-16 |
| 16. CLI Rebrand | 1/1 | Complete | 2026-01-16 |
| 17. MCP Server Rebrand | 1/1 | Complete | 2026-01-16 |
| 18. Documentation Update | 3/3 | Complete | 2026-01-16 |
| 19. Sample App Migration | 1/1 | Complete | 2026-01-16 |
| 20. Templates & Scaffolding | 1/1 | Complete | 2026-01-16 |
| 21. Repository & CI | 1/1 | Complete | 2026-01-16 |
| 22. Publishing & Announcement | 2/2 | Complete | 2026-01-16 |

**Total:** 10 phases, 13 plans

[Full details →](milestones/v2.0-ROADMAP.md)

</details>

<details>
<summary>✅ v1.0 DX Overhaul (Phases 1-12) — SHIPPED 2026-01-16</summary>

**Milestone Goal:** Transform the framework from developer-centric to agent-first.

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 1. Handler Simplification | 1/1 | Complete | 2026-01-15 |
| 2. Model Boilerplate Reduction | 1/1 | Complete | 2026-01-15 |
| 3. Validation Syntax Streamlining | 1/1 | Complete | 2026-01-15 |
| 4. Convention-over-Configuration | 1/1 | Complete | 2026-01-15 |
| 5. MCP Intent Understanding | 1/1 | Complete | 2026-01-15 |
| 6. MCP Error Context | 1/1 | Complete | 2026-01-15 |
| 7. MCP Relationship Visibility | 1/1 | Complete | 2026-01-15 |
| 8. MCP Generation Hints | 1/1 | Complete | 2026-01-15 |
| 9. CLI Feature Scaffolding | 1/1 | Complete | 2026-01-15 |
| 10. CLI Smart Defaults | 1/1 | Complete | 2026-01-15 |
| 11. CLI Component Integration | 3/3 | Complete | 2026-01-15 |
| 12. Agent-First Polish | 5/5 | Complete | 2026-01-16 |

**Total:** 12 phases, 18 plans

[Full details →](milestones/v1.0-ROADMAP.md)

</details>

### Phase 122: Deploy scaffold core rewrite

**Goal:** Rewrite ferro-cli `docker_init`/`do_init` and templates so generated `Dockerfile` + `.do/app.yaml` work for real Ferro apps with zero hand-patching. Conditional frontend stage, multi-binary support, runtime extras hook, themes/lang/public/migrations detection, GITHUB_TOKEN ARG, rust-toolchain.toml pickup, workspace-aware cargo-chef recipe. Path→git ferro dep rewrite via generated `scripts/rewrite-ferro-deps.sh` invoked from Dockerfile + CLI pre-flight verifying ferro git ref is pushed/reachable. `app.yaml` gains `--region`, envs block from `.env.example` with auto SECRET classification, optional `databases:` block, `workers:` for non-server bins. CLI commands gain `--force`, walk-up Cargo.toml lookup, owner/repo validation, shared `project::package_name()` helper. `.dockerignore` adds `database.db`, `*.sqlite*`, `.planning/`, `storage/`, `data/`. Validation: regenerating in gestiscilo and mkmenu produces working builds with zero hand edits. See `phases/122-deploy-scaffold-core-rewrite/SCOPE.md`.
**Requirements**: TBD
**Depends on:** Phase 121
**Plans:** 5/5 plans complete

Plans:
- [x] TBD (run /gsd:plan-phase 122 to break down) (completed 2026-04-07)

### Phase 123: Deploy MCP tools

**Goal:** Expose deploy lifecycle helpers via ferro-mcp: `deploy_check` (pre-flight against missing env, path deps, sqlite in DATABASE_URL, dirty git tree, missing Dockerfile/app.yaml), `deploy_diff_env` (local .env vs .do/app.yaml drift), `runtime_requirements` (scan source for chromium/ffmpeg/etc and report needed runtime apt packages). Read-only.
**Requirements**: TBD
**Depends on:** Phase 122
**Plans:** 5/5 plans complete

Plans:
- [x] 123-01-PLAN.md — runtime_deps registry + Cargo.toml scanner in ferro-cli
- [x] 123-02-PLAN.md — ferro-mcp depends on ferro-cli; deploy_common re-exports
- [x] 123-03-PLAN.md — deploy_check MCP tool (severity-tagged pre-flight report)
- [x] 123-04-PLAN.md — deploy_diff_env MCP tool (.env vs app.yaml drift)
- [x] 123-05-PLAN.md — runtime_requirements MCP tool + Dockerfile cross-check

### Phase 124: Doctor, introspection, CI scaffold

**Goal:** `ferro doctor` (toolchain + DB + migrations + env completeness in one command), `ferro routes --json` (machine-readable for MCP/agents), CI workflow scaffold dropped by `do:init` (`.github/workflows/ci.yml` running `cargo test`, `ferro api:check`, `ferro validate:contracts`), keep `.dockerignore` and `.gitignore` in sync via shared template.
**Requirements**: D-01..D-22 (decisions in 124-CONTEXT.md)
**Depends on:** Phase 122, Phase 123
**Plans:** 5/5 plans complete

Plans:
- [x] 124-01-PLAN.md — ignore_patterns.toml single source of truth + ignore:sync command
- [x] 124-02-PLAN.md — ferro generate-routes --json (stable schema for agents/MCP)
- [x] 124-03-PLAN.md — GitHub Actions CI workflow template + ferro ci:init command
- [x] 124-04-PLAN.md — ferro doctor (7 health checks, human + JSON output)
- [x] 124-05-PLAN.md — wire ci.yml generation into do:init

### Phase 125: Module scaffolder and json-ui runtime split

**Goal:** `ferro make:module <name>` creating `controllers/`, `models/`, `views/`, `routes.rs` skeleton enforcing feature-module convention. Split ferro-json-ui monolithic IIFE in `runtime.rs` into named functions (tabs, SSE, toasts, sidebar) with a small dispatcher, still emitted as one file but testable in isolation.
**Requirements**: TBD
**Depends on:** Phase 122
**Plans:** 1/2 plans executed

Plans:
- [x] 125-01-PLAN.md — ferro make:module command + stub templates + clap wiring (D-01..D-05)
- [ ] 125-02-PLAN.md — ferro-json-ui runtime split into per-concern submodules + ferroRuntime dispatcher (D-06..D-11)

### Phase 122.2: Deploy simplification

**Goal:** Replace the Phase 122/122.1/123/124 deploy machinery with a simpler, heuristic-light, provider-honest shape. Cut custom logic from ~1500 LOC to ~375 LOC. Reduce surviving heuristics from 6 to 1. Delete the 3 deploy MCP tools, revert the ferro-cli↔ferro-mcp circular-dep workaround to in-process launch, delete the golden fixture suite, and fold surviving deploy checks into `ferro doctor`. New Cargo.toml `[package.metadata.ferro.deploy]` schema (runtime_apt, copy_dirs, ferro_version) drives the new Dockerfile renderer. New `.env.production` key-only parser replaces the `.env.example` SECRET classifier. `.do/app.yaml` becomes a one-shot starter owned by the user after scaffold. See `phases/122.2-deploy-simplification/SCOPE.md`.
**Requirements**: SCOPE §1..§13 + Verification
**Depends on:** Phase 122, 122.1, 123, 124
**Plans:** 8/9 plans executed

Plans:
- [x] 122.2-01-PLAN.md — Delete ferro-mcp deploy tools (SCOPE §9)
- [x] 122.2-02-PLAN.md — Revert ferro-cli↔ferro-mcp circular dep, in-process mcp launch (SCOPE §10)
- [x] 122.2-03-PLAN.md — Delete obsolete ferro-cli deploy modules, commands, golden tests (SCOPE §6, §8, §11, §13)
- [x] 122.2-04-PLAN.md — Stub docker_init and do_init before Wave 2 rewrite (SCOPE §2-§5 prep)
- [x] 122.2-05-PLAN.md — Metadata reader + env_production parser + rewrite_ferro_version rewriter (SCOPE §1, §2, §6)
- [x] 122.2-06-PLAN.md — New Dockerfile renderer + static ignores + docker:init rewrite (SCOPE §2, §3, §8)
- [x] 122.2-07-PLAN.md — New app.yaml renderer + do:init rewrite + decouple ci.yml (SCOPE §4, §5, §7)
- [x] 122.2-08-PLAN.md — ferro doctor 9-check revision (SCOPE §12)
- [ ] 122.2-09-PLAN.md — Live UAT against gestiscilo-it/app + phase-end gate (SCOPE §Verification)

### Phase 126: Deploy experience feedback triage

**Goal:** Read `phases/126-deploy-experience-feedback/REPORT.md` (field notes from the first end-to-end gestiscilo deploy against the Phase 122.2 scaffold — 2 fixed bugs already shipped, 9 sharp edges still present, 6 DX improvements), cross-reference each item against existing phases 122–125, and produce a `PROPOSAL.md` classifying every item as: already-in-scope / new-phase / follow-up-plan / dropped. Analysis only — no implementation. Output is a concrete actionable proposal the user reviews before any new phases are added. See `phases/126-deploy-experience-feedback/SCOPE.md`.
**Requirements**: TBD
**Depends on:** Phase 122.2
**Plans:** 0 plans

Plans:
- [x] 126-01 — Triage REPORT items and write PROPOSAL.md ✅

---

### Phase 127: Generated artifact polish (deploy blocker fix + template hygiene)

**Goal:** Make the artifacts that `docker:init` and `do:init` emit actually runnable end-to-end. Today even a successful `docker build` produces an image that silently exits because the Dockerfile has no `ENTRYPOINT` or `CMD` (REPORT item 18) — the same gap will break DigitalOcean App Platform `web` services because the generated `app.yaml` has no `run_command`. Alongside this critical fix, sweep the small template-quality issues from the same gestiscilo session: stop running `cargo build --release` three times, stop reordering dep tables on re-serialization, generate real `envs:` entries instead of comment scaffolds, add a "Next steps" footer to both init commands, ship `--dry-run` for both init commands, and stop generating cargo warnings from `.dockerignore`-excluded README files. Absorbs REPORT items 5, 6, 7, 9, 10, 16, 18. Sequenced first because item 18 is a hard deploy blocker.
**Requirements**: TBD
**Depends on:** Phase 122.2
**Plans:** 4/4 plans complete

Plans:
- [x] 127-01-PLAN.md — Foundation helpers: toml_edit dep, bin_detect, secret_keys, structured .env.example parser, rewrite_ferro_version migration
- [x] 127-02-PLAN.md — Dockerfile template: ENTRYPOINT/CMD token wiring, single build invocation, .dockerignore README whitelist
- [x] 127-03-PLAN.md — DO template: real envs block from .env.example with secret typing, no run_command on web service
- [x] 127-04-PLAN.md — Command wiring: compute/persist split, --dry-run flag, cargo-style Next steps footer

---

### Phase 128: Deploy preflight (`ferro doctor` deploy checks + drift detection)

**Goal:** Catch deploy failures *before* a 1–10 minute Docker round-trip. Extend `ferro doctor` (Phase 124 surface) with deploy-specific checks the gestiscilo session discovered one painful build at a time: `copy_dirs` entries that collide with `.dockerignore` (REPORT item 3), version skew between local path deps and the rewritten `Cargo.docker.toml` (items 4, 13), and `Cargo.docker.toml` staleness vs `Cargo.toml` (item 17). Also ship the interactive `ferro deploy:init` scaffolder for the `[package.metadata.ferro.deploy]` block (item 15) so users do not have to hand-type the table from docs. The same check registry is exposed via the existing Phase 123 `deploy_check` MCP tool — one implementation, two surfaces (per Phase 126 PROPOSAL.md D-07 resolution). Absorbs REPORT items 3, 4, 13, 15, 17.
**Requirements**: REPORT-03, REPORT-04, REPORT-13, REPORT-15, REPORT-17
**Depends on:** Phase 123, Phase 124, Phase 122.2
**Plans:** 4/4 plans complete

Plans:
- [x] 128-01-PLAN.md — DoctorCheck CheckCategory + shared read_path_dep_version helper (foundation)
- [x] 128-02-PLAN.md — copy_dirs_dockerignore_collision + ferro_version_skew + registry + --deploy flag
- [x] 128-03-PLAN.md — ferro deploy:init scaffolder with --dry-run and --yes
- [x] 128-04-PLAN.md — MCP deploy_check tool + deploy docs update

---

### Phase 129: Publish workflow refinement (gated bumps, per-crate version notes)

**Goal:** Stop releasing every workspace member on docs-only or CI-only commits. Gate the auto-patch-bump on whether any *library* crate actually changed (REPORT item 8) — `ferro-cli/`-only or `docs/`-only pushes should not churn versions on every other crate. Document in `PUBLISHING.md` that `ferro_version` is currently a single global field (item 14) and add a per-crate override hook for the day a crate desyncs from the lockstep release; do not implement desync support until a real desync forces it. Absorbs REPORT items 8, 14. Maintainer ergonomics — lowest user-pain of the three follow-ups.
**Requirements**: TBD
**Depends on:** None
**Plans:** 3/3 plans complete

Plans:
- [x] TBD (run /gsd:plan-phase 129 to break down) (completed 2026-04-09)

### Phase 130: Invert dep convention (simple)

**Goal:** Retire `Cargo.docker.toml` and the `cargo_docker_toml_staleness` doctor check. Docker builds use `Cargo.toml` directly. Local ferro development happens via a hand-written, uncommitted `[patch.crates-io]` block. No new CLI verbs, no new doctor check, no hooks, no CLAUDE.md section.
**Requirements**: P130-R1 (delete cargo_docker_toml_staleness check), P130-R2 (delete ferro_version_skew check and rewrite_ferro_version module), P130-R3 (remove Cargo.docker.toml generation from docker:init and do:init), P130-R4 (Dockerfile template builds from Cargo.toml directly), P130-R5 (Cargo.toml scaffold hint comment for local [patch.crates-io] dev)
**Depends on:** Phase 129
**Plans:** 1 plan

Plans:
- [ ] 130-01-PLAN.md — Delete Cargo.docker.toml infrastructure, update templates and docs

### Phase 131: Scaffolder multi-bin, copy_dirs, runtime_apt, DO app.yaml robustness, drift detection

**Goal:** Make `ferro docker:init` and `ferro do:init` handle non-trivial projects without hand-maintenance. (1) Multi-bin detection — build and wire every `[[bin]]` (web + workers) in Dockerfile and `.do/app.yaml`. (2) Runtime `copy_dirs` from `[package.metadata.ferro.docker]` (e.g. `themes/`, `migrations/`). (3) Runtime apt packages from `runtime_apt` metadata. (4) `.do/app.yaml` robustness — preserve `region`/`name`/repo binding on `--force`; fix envs-from-`.env.example` path; drop unconditional `health_check`; remove dead Node.js frontend build stage for server-rendered projects. (5) `ferro doctor` check `docker_template_drift` that re-runs scaffolder in-memory and diffs against committed files. Test bed: gestiscilo-it commit `6f6d397` must become byte-identical to scaffolder output. Source: `.planning/backlog/gestiscilo-scaffolder-multibin-gap.md`.
**Requirements**: REQ-131-01, REQ-131-02, REQ-131-03, REQ-131-04, REQ-131-05, REQ-131-06, REQ-131-07, REQ-131-08, REQ-131-09, REQ-131-10, REQ-131-11
**Depends on:** Phase 130
**Plans:** 3/3 plans complete

Plans:
- [x] 131-01-PLAN.md — Freeze gestiscilo 6f6d397 fixtures and write byte-identical regeneration tests (Wave 0 gap audit)
- [x] 131-02-PLAN.md — .do/app.yaml identity preservation on --force and docker_template_drift doctor check
- [x] 131-03-PLAN.md — Collapse duplicate read_bins into single canonical reader

### ✅ v11.3 S3 Storage Driver

**Milestone Goal:** Replace the stub S3 driver in ferro-storage with a working implementation backed by `aws-sdk-s3`, enabling gestiscilo to drop its hand-rolled storage service.

## Phases

- [x] **Phase 132: Implement ferro-storage S3 Driver** (completed 2026-04-14)

### Phase 132: Implement ferro-storage S3 Driver

**Goal:** Replace the stub `S3Driver` in `ferro-storage/src/drivers/s3.rs` with a working implementation using the already-declared `aws-sdk-s3` dependency. The S3 feature gate and config wiring (`AWS_*` env vars, `DiskConfig`, `DiskDriver::S3`) are already in place — only the driver methods need real implementations.

Implement all 15 `StorageDriver` trait methods (`exists`, `get`, `put`, `delete`, `copy`, `size`, `metadata`, `url`, `temporary_url`, `files`, `all_files`, `directories`, `make_directory`, `delete_directory`). Use `aws-sdk-s3` client initialized from the `DiskConfig` fields (`bucket`, `region`) and `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` credentials. Support custom endpoints via `AWS_URL` for S3-compatible providers (DigitalOcean Spaces, MinIO, R2). Add integration tests gated behind an `s3-tests` feature that run against a real bucket.

**Plans:** 1/1 plans complete

Plans:
- [x] 132-01-PLAN.md — S3Driver implementation, facade wiring, unit tests, integration test scaffold
**Exit criteria:** `Storage::disk("s3").put("test.txt", bytes).await` works against DigitalOcean Spaces; all 15 trait methods return real results instead of `Error::not_implemented`; gestiscilo can replace its hand-rolled `src/services/storage.rs` with `ferro::Storage`.

**Depends on:** Phase 131

**Field test:** gestiscilo-it will drop `src/services/storage.rs` and its `SPACES_*` env vars in favor of `ferro::Storage::disk("s3")` with `AWS_*` vars once this phase ships.

### Phase 133: Remove envs block from do:init scaffolder

**Goal:** Stop generating the `envs:` block in `.do/app.yaml`. Users configure environment variables directly in the DigitalOcean dashboard or via `doctl` — the app spec should only declare infrastructure (services, workers, regions, instance sizing), not runtime configuration. The current scaffolder reads `.env.example` and emits an envs block that duplicates what the user already manages through the platform UI, creating a maintenance burden and a false-positive in `deploy_env_parity`.

Remove the envs-from-`.env.example` rendering path in `do:init`. Remove the `deploy_env_parity` doctor check (it exists only to validate the envs block that is being removed). Update the gestiscilo fixture to reflect the envs-free `.do/app.yaml`.

**Exit criteria:** `ferro do:init --force` produces an `.do/app.yaml` with no `envs:` block; `ferro doctor` no longer includes `deploy_env_parity`.

**Depends on:** Phase 132

---

### ✅ v11.5 Projection Architecture Prep

**Milestone Goal:** Refactor the projection rendering pipeline so the Renderer trait is modality-agnostic. This unblocks v12.0 (which rewrites the visual renderer) and v14.0 (which adds non-visual renderers). Without this, channel adapters would be bolted on rather than projected through.

## Phases

- [x] **Phase 133: Generalize Renderer trait** (completed 2026-04-14)
- [x] **Phase 134: Relocate renderers to output crates** (completed 2026-04-17)
- [x] **Phase 135: ServiceDef derivation bridge** (completed 2026-04-17)

### Phase 133: Generalize Renderer trait

**Goal:** Replace the visual-only `Renderer` trait with a modality-agnostic version. The current trait returns `serde_json::Value` and takes a `RenderContext` containing `ThemeTemplates` — both assumptions lock renderers to JSON-UI output. Introduce associated `Output` and `Context` types so each renderer declares its own output format and context needs.

Concrete changes: (1) `Renderer` trait gains `type Output` and `type Context: Default`, replacing hardcoded `serde_json::Value` return and `RenderContext` parameter. (2) Current `RenderContext` (with `ThemeTemplates`) becomes `VisualContext` — the context type for visual renderers only. (3) `TemplateRenderer` and `JsonUiRenderer` updated to implement the new trait. (4) `ferro-projections → ferro-theme` dependency removed — `ThemeTemplates` stays in ferro-theme but is only referenced by visual renderers, not by the core trait.

**Exit criteria:** `Renderer` trait has associated types. `cargo test --all-features` passes. ferro-projections no longer depends on ferro-theme. `ThemeTemplates` is consumed by `JsonUiRenderer`'s context, not by the base trait.


**Plans:** 2/2 plans complete

Plans:
- [x] 133-01-PLAN.md — Refactor Renderer trait, split context types, gate ferro-theme
- [x] 133-02-PLAN.md — Update ferro-mcp imports for compilation

**Depends on:** Phase 132

### Phase 134: Relocate renderers to output crates

**Goal:** Move `JsonUiRenderer` and its supporting modules (`field_map.rs`, `relationship_map.rs`) from `ferro-projections/src/render/` to `ferro-json-ui`. ferro-projections retains: the `Renderer` trait, `derive_intents()`, `ServiceDef`, `IntentScore`, and `TemplateRenderer` (generic JSON output). ferro-json-ui gains a dependency on ferro-projections for the trait and types.

This establishes the pattern for v14.0: each output crate provides its own `Renderer` implementation. A WhatsApp renderer would live in ferro-whatsapp with a `projections` feature flag, not in ferro-projections.

**Exit criteria:** `JsonUiRenderer` importable from `ferro_json_ui`, not from `ferro_projections`. `ferro-projections/src/render/` contains only `mod.rs` (trait), `template.rs`. All existing tests pass. MCP tools and CLI updated to import from new location.

**Depends on:** Phase 133

**Plans:** 2/2 plans complete

Plans:
- [x] 134-01-PLAN.md — Relocate JsonUiRenderer + field_map + relationship_map to ferro-json-ui behind projections feature
- [x] 134-02-PLAN.md — Clean ferro-projections, update ferro-mcp imports

### Phase 135: ServiceDef derivation bridge

**Goal:** Reduce the gap between a SeaORM model and a working projection. Currently ServiceDef is hand-authored via the builder API. Add a `ServiceDef::from_model()` derivation that infers fields, data types, and field meanings from SeaORM model metadata. Also expose this through ferro-mcp as a `generate_projection` tool that produces a ServiceDef from `db_schema` + `list_routes` output.

This is the time-to-working-projection bottleneck. An agent should be able to go from `cargo new` to a rendered projection without hand-writing ServiceDef builders.

**Exit criteria:** `ServiceDef::from_model(model_metadata)` produces a reasonable ServiceDef from SeaORM column types. `ferro-mcp` has a `generate_projection` tool. A round-trip test demonstrates: create model → derive ServiceDef → derive intents → render.

**Depends on:** Phase 134

**Plans:** 2/2 plans complete

Plans:
- [x] 135-01-PLAN.md — ModelMetadata, DataType::from_column_type(), ServiceDef::from_model() in ferro-projections
- [x] 135-02-PLAN.md — generate_projection MCP tool in ferro-mcp

### Phase 136: implement workflow for executing a full roadmap in auto with gsd

**Goal:** GitHub Actions workflow that drives an entire milestone through the GSD pipeline — one fresh claude CLI invocation per phase, with failure handling via GitHub issues.
**Status:** Workflow committed, awaiting live test before marking complete.
**Requirements**: TBD
**Depends on:** Phase 135
**Plans:** 1 plan

Plans:
- [x] 136-01-PLAN.md — Create gsd-roadmap.yml workflow: phase loop, claude CLI per phase, failure issues

### Phase 141: protocol-uplift

**Goal:** Drop `event_json: String` from the five existing typed event structs and remove their `ferro_events::Event` impls. Ship `SyncDispatcher` in `webhook/sync.rs` as the sole handler registry. Relocate `ProcessStripeWebhook` to `webhook/queue.rs` wired to `Arc<SyncDispatcher>`. Add five new event types (`StripeCheckoutExpired`, `StripePaymentIntentFailed`, `StripeChargeRefunded`, `StripeChargeDisputeCreated`, `StripeConnectAccountUpdated`) with fully-parsed fields via the `StripeEvent::from_raw` trait method. Provide golden-JSON fixtures with parser-contract tests. Release `ferro-stripe 0.5.0`.
**Requirements**: SC-1..SC-14 (Phase 141 success criteria in milestone §"Phase 141: Protocol uplift")
**Depends on:** Phase 140
**Plans:** 4/4 plans complete

Plans:
- [x] 141-01-PLAN.md — Foundation: Cargo.toml deps + StripeEvent trait + 10 reshaped/new event structs + relocate signed_webhook_payload (Wave 1)
- [x] 141-02-PLAN.md — SyncDispatcher in webhook/sync.rs + integration tests (tests/dispatcher.rs) (Wave 2)
- [x] 141-03-PLAN.md — 10 golden-JSON fixtures + parser-contract integration tests (Wave 2)
- [x] 141-04-PLAN.md — Relocate ProcessStripeWebhook to webhook/queue.rs + framework re-exports + full workspace gate (Wave 3)

### Phase 142: protocol-uplift

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 141
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 142 to break down)

### Phase 144: Fix root path routing in group routes

**Goal:** `get!("/", handler)` registered inside a non-root `group!("/prefix", { ... })` is reachable at both `/prefix` and `/prefix/`. Trailing-slash prefix normalization applied in both the macro-based (`GroupDef::register_with_inherited`) and builder-based (`GroupBuilder::finalize`) group implementations. Route introspection (`get_registered_routes()`, `ferro-mcp list_routes`) continues to report one canonical entry per logical handler. Ships as patch release 0.2.13.
**Requirements**: D-01, D-02, D-03, D-04, D-05, D-06, D-07, D-08, D-09, D-10, D-11, D-12, D-13 (from 144-CONTEXT.md)
**Depends on:** Phase 143
**Plans:** 5/5 plans complete

Plans:
- [x] 144-01-PLAN.md — combine_group_path helper + 8-row matrix test (Wave 0, new framework/src/routing/path.rs)
- [x] 144-02-PLAN.md — apply helper in GroupDef::register_with_inherited + add insert_*_alias methods on Router; inline tests for D-01..D-04, D-06, D-08 (Wave 1)
- [x] 144-03-PLAN.md — apply helper in GroupBuilder::finalize; mirrored test module for D-05, D-11 lockstep (Wave 1, parallel with Plan 02)
- [x] 144-04-PLAN.md — integration tests for D-07, D-10, middleware-on-both-variants, gestiscilo reproducer (Wave 2)
- [x] 144-05-PLAN.md — docs (routing.md, middleware.md, rustdoc), CHANGELOG 0.2.13 entry (neutral voice), workspace version bump, final full gate (Wave 3)
### Phase 145: ferro serve manual reload key and watch supervisor

**Goal:** Replace the external `cargo-watch` dependency in `ferro serve` with an in-process supervisor. Make auto-watch opt-in via `--watch` (off by default). Add a runtime `r` key that triggers a backend rebuild and types regeneration, cancelling any in-flight build. Use `notify-debouncer-mini` for trailing-edge debounce (500 ms fixed) so a burst of file-saves produces one rebuild rather than many.
**Requirements**: Design spec at `docs/superpowers/specs/2026-04-22-ferro-serve-reload-key-design.md`. Targets `ferro-cli/src/commands/serve.rs`. Deletes `ensure_cargo_watch()` and `start_type_watcher()`. Adds `notify-debouncer-mini` and `crossterm` deps; drops external `cargo-watch` install step.
**Depends on:** Phase 144
**Plans:** 5/5 plans complete

Plans:
- [x] 145-01-PLAN.md — Wave 0 test infrastructure: minimal-serve fixture + integration-test scaffold + pure-function contracts (render_banner, classify_key, KbAction, ReloadTrigger, format_trigger_source, should_spawn_keyboard) + 5 inline #[ignore]-gated unit-test stubs (Wave 0)
- [x] 145-02a-PLAN.md — Deps, clap surface, deletions, pure helpers: add --watch clap flag, delete ensure_cargo_watch() + start_type_watcher() + cargo-watch spawn path, extract spawn_child_with_prefix, fill render_banner/classify_key/format_trigger_source/should_spawn_keyboard bodies, un-ignore 4 pure-helper tests (Wave 1, depends on 01)
- [x] 145-02b-PLAN.md — BackendSupervisor + producers + run() rewire: BackendSupervisor struct + RawModeGuard + spawn_keyboard_thread + spawn_file_watcher[_at] + drain_triggers, rewire run() with shutdown ordering per D-29, un-ignore remaining 3 supervisor-dependent tests (Wave 1, depends on 02a)
- [x] 145-03-PLAN.md — Integration tests: 4 un-ignored tests in serve_supervisor.rs against minimal-serve fixture covering SIGINT shutdown budget, `r`-key trigger, `--watch` burst coalescing, non-TTY banner; adds Unix libc dev-dep + env-var test hook (Wave 2, depends on 01+02b)
- [x] 145-04-PLAN.md — Docs refresh: update docs/src/reference/cli.md serve section (options table, key legend, What-it-does), rewrite ferro-cli/src/commands/skills/serve.md, phase-wide cargo-watch grep gate (Wave 2, depends on 02b, parallel with 03)

### Phase 146: Add KeyValueEditor component to ferro-json-ui — dynamic key-value pair editor with suggested keys, custom rows, JSON serialization to hidden field, and runtime behavior in ferro-json-ui IIFE

**Goal:** Ship a `KeyValueEditor` JSON-UI component that renders a dynamic list of key/value rows backed by a hidden JSON field, supports seeded rows from `data_path`, suggested keys via `<datalist>` or restricted `<select>`, error-state propagation, and event-delegated add/delete/input serialization via a new `setupKeyValueEditor()` runtime module.
**Requirements**: R1 (html_escape on all dynamic HTML), R2 (data_path pre-fill), R3 (error state classes), R4 (select variant), R5 (datalist variant), R6 (empty hidden field defaults to `{}`), R7 (bundle contains setupKeyValueEditor), R8 (dispatcher invokes setupKeyValueEditor), R9 (serde round-trip)
**Depends on:** Phase 145
**Plans:** 3/3 plans complete

Plans:
- [x] 146-01-PLAN.md — Wave 0 RED tests: 7 render_key_value_editor unit tests in render.rs, 2 serde round-trip tests in component.rs, update both runtime/mod.rs test arrays to require setupKeyValueEditor (Wave 1, no deps)
- [x] 146-02-PLAN.md — Rust implementation: KeyValueEditorProps struct + Component::KeyValueEditor variant + serde match arms in component.rs, render_key_value_editor() + dispatch arm in render.rs, public re-export + COMPONENT_CATALOG entry in lib.rs (Wave 2, depends on 01)
- [x] 146-03-PLAN.md — Runtime JS: new ferro-json-ui/src/runtime/key_value_editor.rs with ES5 setupKeyValueEditor/initKeyValueEditor/syncHiddenField, wire module decl + SOURCE push + dispatcher call into runtime/mod.rs (Wave 3, depends on 01)

### Phase 147: DetailForm component for inline edit — ferro-json-ui

**Goal:** Ship a `DetailForm` JSON-UI component that renders the same structural container in View and Edit modes, driven by a server-side URL query param (`?mode=edit`); View renders a `<dl>` + "Modifica" link, Edit wraps the same `<dl>` in a `<form>` with "Salva"/"Annulla" actions and method spoofing for PUT/PATCH/DELETE. Adds `EditMode` enum with `from_query()`, `DetailField`, `DetailFormProps`, `Component::DetailForm` variant with serde + resolver arms, `ComponentNode::detail_form` factory, and ferro-mcp catalog entry (also backfills KeyValueEditor catalog gap from Phase 146). No runtime JS.
**Requirements**: D-01..D-20 (per 147-CONTEXT.md) — EditMode enum + from_query; DetailField/DetailFormProps structs; structural coherence contract (§5 of 147-UI-SPEC); method-spoofing integrity (T-147-01); html_escape XSS mitigation (T-147-02); resolver participation in all three passes; Option-A label authoring rule documented in catalog + docs
**Depends on:** Phase 146
**Plans:** 5/5 plans complete

Plans:
- [x] 147-01-PLAN.md — Wave 0 RED tests: EditMode + DetailFormProps serde tests in component.rs (13 tests); render_detail_form_* tests in render.rs (12+ tests covering View/Edit/spoofing/buttons/escapes/invariance); resolver tests in resolve.rs (3 tests); ferro-mcp exhaustive-list assertion bumped to 41 + DetailForm/KeyValueEditor added to expected names (Wave 0, no deps)
- [x] 147-02-PLAN.md — Rust types: EditMode enum + DetailField struct + DetailFormProps struct + Component::DetailForm variant + serde match arms + ComponentNode::detail_form factory in component.rs; public re-exports + ### DetailForm COMPONENT_CATALOG entry in lib.rs (Wave 1, depends on 01)
- [x] 147-03-PLAN.md — Renderer: fn render_detail_form in render.rs + dispatch arm in render_component + container arm in collect_plugin_types_node; emits identical <dl> scaffold across modes with html_escape discipline and method-spoofing copied verbatim from render_form (Wave 1, depends on 01)
- [x] 147-04-PLAN.md — Resolver: three Component::DetailForm arms in resolve.rs (resolve_component_node, collect_unresolved_node, resolve_errors_node) — mirrors Component::Form, preserves D-16 (edit_url/cancel_url never resolved) (Wave 1, depends on 01)
- [x] 147-05-PLAN.md — MCP catalog + docs + CI gate: DetailForm CatalogComponent entry + KeyValueEditor backfill in ferro-mcp/src/tools/json_ui_catalog.rs; ### DetailForm section in docs/src/json-ui/components.md with Option-A rule; full CI gate (cargo fmt + clippy --all --all-targets -- -D warnings + test --all-features) (Wave 1, depends on 01)

### Phase 148: ImageProps inline-SVG source — extend Image, don't add HtmlEmbed

**Goal:** Extend `ferro-json-ui`'s `ImageProps` with an `ImageSource` serde-untagged enum so `Component::Image` can carry either an external URL (current `src`) or a server-constructed inline SVG string. Renderer gains one branch; `alt: String` stays required (compile-enforced a11y); URL wire format stays fully backward-compatible. No new component variant, no new resolver arm, no MCP exhaustive-list bump — the existing `Image` slot absorbs the capability. Unblocks gestiscilo.it v6.1 Statistiche revenue-trend bar chart without introducing a generic HTML escape hatch.
**Requirements**: IMG-SRC-01..IMG-SRC-05 — ImageSource enum (`Url {src}` / `InlineSvg {svg}`) with untagged serde; ImageProps flattens source + alt stays required; render_image branches on source (URL path unchanged, InlineSvg emits `<div role="img" aria-label="{escaped alt}">{svg verbatim}</div>`); COMPONENT_CATALOG + MCP catalog + docs all reflect both variants with SVG-branch safety callout; CI gate green
**Depends on:** Phase 147
**Plans:** 3/3 plans complete

Plans:
- [x] 148-01-PLAN.md — Wave 0 RED tests: extend image_round_trips + all_known_types_round_trip fixture with InlineSvg cases; add image_source_tests module (5 tests) in component.rs; rewrite existing ImageProps struct literals at component.rs:2173 and render.rs:3758/3780/3798 to target API shape; add three RED InlineSvg render tests in render.rs (div-role-img, load-bearing script-passthrough bypass documentation, alt-xss-escaped) (Wave 0, no deps)
- [x] 148-02-PLAN.md — Wave 1 implementation: introduce ImageSource untagged enum + refactor ImageProps with #[serde(flatten)] source field + add ImageProps::url / ImageProps::inline_svg constructors (with D-12 safety rustdoc on both); branch render_image on props.source with inline // SAFETY comment on InlineSvg arm (Wave 1, depends on 01)
- [x] 148-03-PLAN.md — Wave 2 surface updates + final CI gate: ### Image section added to COMPONENT_CATALOG in ferro-json-ui/src/lib.rs (pre-existing gap closed); ferro-mcp CatalogComponent for Image widened (dual-source description, src + svg props, count stays 41); ### Image section added to docs/src/json-ui/components.md with props table + safety callout + Rust + JSON examples + "no generic HTML escape hatch" pointer; full CI gate (cargo fmt + clippy --all --all-targets -- -D warnings + test --all-features) (Wave 2, depends on 02)

Prior planning artifacts for the rejected `Component::HtmlEmbed` scope are archived at `.planning/phases/148-image-inline-svg-source/archive-htmlembed/` for decision-trail traceability (see `148-DISCUSSION-LOG.md`).

---

### 📋 v11.9 Notifications & Rich-Text Foundations (Phases 149-150, planned 2026-04-28)

Source: gestiscilo-it v6.4 Documents & Notifications field test. Two upstream additions consumed by gestiscilo Phases 120 and 125 respectively. Auto-publishes via GH Actions on push to master; consumer apps (gestiscilo) bump `Cargo.toml` after publish.

**Phase number reconciliation:** v11.9 inserts itself at the next available ferro phase number (149-150). v12.0 JSON-UI v2 still owns Phases 115-121 in its own scope; v11.9 does not collide.

### Phase 149: ferro-notifications WhatsApp + InApp channels + MailMessage attachment

**Goal:** Extend `ferro-notifications` with two new channel adapters and a Mail attachment builder so consumer apps can dispatch transactional notifications across WhatsApp + in-app SSE banners and attach binary files (PDFs) to email. Additive, non-breaking to existing `Notification` impls. `Channel::Push` remains an enum-only stub (no APNs/FCM adapter) — consumer matrix UIs render the column as "coming soon".

**Source:** gestiscilo-it v6.4 milestone — see `gestiscilo-it/app/.planning/REQUIREMENTS.md` FERRO-01, FERRO-02, FERRO-03, and `.planning/research/v6.4-DOCUMENTS-NOTIFICATIONS-STACK.md` for the full integration design.

**Depends on:** ferro-whatsapp (existing crate); lettre 0.11 (already a transitive dep via ferro-notifications Mail driver).

**Success Criteria** (what must be TRUE):
  1. `ferro_notifications::Channel::WhatsApp` and `Channel::InApp` enum variants exist; existing `Channel::Mail`/`Database`/`Slack`/`Sms`/`Push` variants unchanged; the `Push` variant carries no adapter and the dispatcher emits a structured "channel not configured" no-op for it
  2. `Notification::to_whatsapp(&self) -> Option<WhatsAppMessage>` and `Notification::to_in_app(&self) -> Option<InAppMessage>` are added as default-`None` trait methods so all existing `Notification` impls compile unchanged
  3. `WhatsAppChannel` adapter dispatches via the static `ferro_whatsapp::WhatsApp::send` facade (no client injection — `ferro-whatsapp` owns global state via `WhatsApp::init` at app startup); gated by `NotificationConfig::whatsapp_enabled` (default `false`, opt-in via `WHATSAPP_ENABLED` env or builder)
  4. `InAppChannel` adapter accepts an SSE broker handle plus a `DatabaseNotificationStore` trait object and writes both legs on dispatch
  5. `MailMessage::attachment(filename, content_type, bytes)` builder exists; lettre wiring delivers a multi-part email with the attached file; max-size guard returns a typed error at 25 MB; round-trip integration test verifies attachment arrives intact at a Mailpit fixture
  6. `cargo clippy --all --all-targets -- -D warnings` and `cargo test --all-features` green across the workspace; GH Actions publishes the new ferro-notifications version to crates.io
  7. Consumer-side smoke test in gestiscilo-it: `use ferro_notifications::{Channel, WhatsAppChannel, InAppChannel};` resolves; `MailMessage::new().attachment(...)` compiles and sends

**Plans:** 7/7 plans complete

Plans:
- [x] 149-01-PLAN.md — Wave 0: skeleton message types and channels module wiring
- [x] 149-02-PLAN.md — Wave 1: Channel + Notification + Error surface (D-01, D-02, D-05, ARCH-FINDING-03)
- [x] 149-03-PLAN.md — Wave 1: MailAttachment + 25MB-capped attachment() builder (D-09, D-10, D-11)
- [x] 149-04-PLAN.md — Wave 2: SMTP multipart + Resend base64 attachment payload (D-12)
- [x] 149-05-PLAN.md — Wave 3: WhatsApp adapter (D-04, D-14, ARCH-FINDING-01)
- [x] 149-06-PLAN.md — Wave 4: InApp adapter + DB-store wire (D-06, D-07, D-08, D-13, ARCH-FINDING-02)
- [x] 149-07-PLAN.md — Wave 5: Re-exports, publish.yml move, docs, Mailpit integration test, final CI (D-15, D-16, ARCH-FINDING-05)

### Phase 150: ferro-json-ui RichTextEditor component

**Goal:** Ship a `RichTextEditor` component in `ferro-json-ui` that wraps Quill 2.0.3 (Snow theme, jsDelivr CDN, SRI-pinned, vanilla — no bundler) so consumer apps can author rich-text bodies in dashboard forms without a JS build step. Pattern mirrors the v6.1 `Chart` plugin and the existing `KeyValueEditor` component (Phase 146). Output is dual-format: Delta JSON (canonical, lossless) + sanitized HTML cache (rendering input). Toolbar `formats` whitelist constrained at the component-prop level so consumer apps cannot accidentally enable image/video/HTML-paste paths.

**Source:** gestiscilo-it v6.4 milestone — used by Phase 125 (document template editor). See `gestiscilo-it/app/.planning/REQUIREMENTS.md` DOC-02, DOC-04 for consumer requirements.

**Depends on:** Phase 149 (not strictly — independent — but bundling them in v11.9 keeps the upstream-publish cadence aligned).

**Success Criteria** (what must be TRUE):
  1. `Component::RichTextEditor(RichTextEditorProps)` exists in the ferro-json-ui component catalog with `name: String` (form field name), `value: Option<String>` (initial Delta JSON or HTML), `formats: Vec<String>` (toolbar whitelist; defaults to bold/italic/underline/lists/headings/links), `placeholder: Option<String>`, `theme: String` (defaults to "snow"); compile-enforced via the existing component derive
  2. Renderer emits a `<div data-rich-text-editor>` host element plus the Quill IIFE bootstrap in the page footer; Quill is loaded from `cdn.jsdelivr.net/npm/quill@2.0.3/dist/quill.js` with SRI hash; CSS from `dist/quill.snow.css` with SRI hash
  3. On form submit, the runtime IIFE serializes the editor state to two hidden inputs: `{name}_delta` (Delta JSON) and `{name}_html` (sanitized HTML); consumer controllers read both
  4. The `formats` whitelist is enforced both at editor initialization (passed as Quill toolbar config) and at HTML serialization (post-process strips disallowed tags); consumer cannot bypass by mutating the DOM
  5. Component round-trips via the standard ferro-json-ui JSON-UI serde fixtures; documented under `### RichTextEditor` in `docs/src/json-ui/components.md` with props table + Rust + JSON example
  6. `cargo clippy --all --all-targets -- -D warnings` and `cargo test --all-features` green; ferro-json-ui MCP catalog component count incremented and the new component documented in MCP catalog
  7. ferro-mcp `CatalogComponent` for `RichTextEditor` exposes the schema so AI tooling can generate forms with rich-text fields

**Plans:** 5/5 plans complete

Plans:
- [x] 150-01-PLAN.md — Wave 1 RED tests: render_rich_text_editor_* unit tests in render.rs, serde round-trip + theme-default tests in component.rs, runtime/mod.rs test arrays extended to require setupRichTextEditor (Wave 1, no deps)
- [x] 150-02-PLAN.md — Quill 2.0.3 SHA-384 SRI bootstrap: compute hashes from live jsDelivr bytes via curl + openssl, create ferro-json-ui/src/assets/quill.rs with four pinned pub(crate) consts (URLs + SRI), promote assets.rs to assets/ directory and wire submodule (Wave 2, depends on 01)
- [x] 150-03-PLAN.md — Component variant + render fn + asset injection: RichTextEditorProps + Component::RichTextEditor + serde arms + ComponentNode::rich_text_editor factory in component.rs; render_rich_text_editor + dispatch arm + collect_plugin_types_node enrollment in render.rs; new plugins/rich_text_editor.rs (RichTextEditorPlugin asset-only adapter) registered in global_plugin_registry — first-class component reuses the plugin asset pipeline (D-02) (Wave 3, depends on 01, 02)
- [x] 150-04-PLAN.md — Runtime IIFE: new ferro-json-ui/src/runtime/rich_text_editor.rs with ES5 setupRichTextEditor / initRichTextEditor / formatsToToolbarConfig / sanitizeHtmlByFormats; submit interception writes {name}_delta + {name}_html; formats whitelist enforced at init (Quill option) and submit (DOM-walker post-process); wire module + SOURCE push + dispatcher call into runtime/mod.rs (Wave 4, depends on 01, 03)
- [x] 150-05-PLAN.md — Public surface + docs + final CI: re-export RichTextEditorProps and RichTextEditorPlugin from lib.rs; ### RichTextEditor in COMPONENT_CATALOG; ferro-mcp CatalogComponent entry + count assertion 41→42; docs/src/json-ui/components.md ### RichTextEditor section; final fmt + clippy -D warnings + test --all-features gate (Wave 5, depends on 03, 04)

### 📋 v11.11 Resource Reservation & Live Read-Model Primitives (Phases 152-155, planned 2026-05-13)

**Source:** gestiscilo-it inventory monitoring field test (2026-05-13 audit). Two consumer milestones already need this stack — v6.3 Online Checkout (slot hold with TTL during Stripe payment) and v6.7 Inventory Monitoring (booking reservations + live Magazzino dashboard).

**What ships:** four reusable horizontal primitives that any future capacity-constrained app can adopt. Domain-neutral — ferro stays out of inventory semantics. Full design in [research/INVENTORY-PRIMITIVES.md](research/INVENTORY-PRIMITIVES.md).

**Build order:** 152 (guarded) and 153 (audit) are foundational and parallelizable. 154 (reservation) depends on both. 155 (projection) is independent of the others but typically deployed alongside.

### Phase 152: ferro-orm GuardedUpdate — atomic conditional updates for race-free counter mutations

**Goal:** Ship `ferro-orm` as a new top-level workspace crate exposing `GuardedUpdate<E>` — a chainable builder that compiles to a single `UPDATE … WHERE …` SQL statement, replacing the hand-rolled `read → check → write` pattern wherever a column's value is conditionally mutated. Race-free by construction at the database layer. Foundational kernel for v11.11 (reservation kernel + live read-models depend on this).
**Requirements**: none — feature-driven phase, `phase_req_ids` is null; locked decisions D-01..D-25 in 152-CONTEXT.md are the must-haves
**Depends on:** none
**Plans:** 6/6 plans complete

Plans:
- [x] 152-01-PLAN.md — scaffold ferro-orm crate (Cargo.toml, lib.rs, error.rs, README.md; guarded.rs stub)
- [x] 152-02-PLAN.md — register ferro-orm in workspace (root Cargo.toml + publish.yml Wave 1a + CLAUDE.md table row)
- [x] 152-03-PLAN.md — implement GuardedUpdate builder body + 7 unit tests (T-16-1..T-16-7)
- [x] 152-04-PLAN.md — concurrent_decrement integration test (T-17-1: 10 tokio tasks vs K=3, exactly 3 succeed)
- [x] 152-05-PLAN.md — docs/src/database/atomic-updates.md + SUMMARY.md nav entry
- [x] 152-06-PLAN.md — release: pre-release gate + CHANGELOG entry + first-publish bootstrap (manual checkpoint)

### Phase 153: ferro-audit crate — structured before/after audit log with replay

**Goal:** Ship the `ferro-audit` Wave 1a leaf crate — append-only structured before/after audit log with replay-ready query helpers, a SeaORM migration consumers register in their own `Migrator`, and the `AuditEntry::record(action).…write(&conn)` builder API. Includes `AuditActor` typed enum, `AuditTarget` struct, three query helpers (`history_for_target`, `recent_by_actor`, `recent`), pure `reconstruct_state` shallow-merge fold, and `prune_older_than` retention helper. Bumps workspace version 0.2.30 → 0.2.31 and bootstraps first publish to crates.io.
**Requirements**: D-01..D-40 (feature-driven phase; decision IDs from 153-CONTEXT.md)
**Depends on:** none (parallel with Phase 152)
**Plans:** 6/6 plans complete

Plans:
- [x] 153-01-PLAN.md — scaffold ferro-audit crate (Cargo.toml, lib.rs, error.rs, actor.rs, target.rs, README.md, stub modules)
- [x] 153-02-PLAN.md — register ferro-audit in workspace (Cargo.toml members + version bump 0.2.30 → 0.2.31, publish.yml Wave 1a, CLAUDE.md, README.md)
- [x] 153-03-PLAN.md — SeaORM entity + migration (audit_log table + 2 indexes) + migration unit test
- [x] 153-04-PLAN.md — AuditEntry builder + write() with post-INSERT re-fetch + 5 happy-path unit tests
- [x] 153-05-PLAN.md — query helpers (history_for_target/recent_by_actor/recent) + reconstruct_state + prune_older_than + 4 unit tests
- [x] 153-06-PLAN.md — integration test + docs/src/database/audit-log.md + SUMMARY.md nav + CHANGELOG + pre-release gate + first-publish bootstrap (manual checkpoint)

### Phase 154: ferro-reservation crate — generic hold/commit/release with TTL and event broadcast

**Goal:** Ship a domain-neutral resource reservation kernel as a new top-level Wave 1b workspace crate (`ferro-reservation`). The crate exposes `ReservationKernel<R: Resource>` with `hold` / `commit` / `release` / `extend` / `run_sweep_once` — a typed, race-free state-transition pipeline composing `ferro-orm::GuardedUpdate` (atomic state transitions), `ferro-audit::AuditEntry` (unconditional audit emission), and `ferro-events::dispatch` (best-effort domain events). Consumers implement the `Resource` trait against their own domain model; the kernel knows nothing about inventory, products, slots, or seats. Anchored by D-48 (concurrent_hold integration test: N=20 vs capacity=5 → exactly 5 succeed), D-49 (proptest properties: capacity invariant + state-machine validity via audit replay), and D-50 (cross-crate showcase: 2 events + 2 audit entries + reconstruct_state).
**Requirements**: D-01..D-58 (feature-driven phase — every locked decision from 154-CONTEXT.md is a must-have)
**Depends on:** Phase 152 (ferro-orm GuardedUpdate, shipped 0.2.30), Phase 153 (ferro-audit, shipped 0.2.31)
**Plans:** 7/7 plans complete

Plans:
- [x] 154-01-PLAN.md — Scaffold ferro-reservation crate (Cargo.toml, lib.rs with rustdoc + state diagram, full ReservationError body, 8 stub modules)
- [x] 154-02-PLAN.md — Register crate in workspace (Cargo.toml members + version bump 0.2.31→0.2.32, publish.yml WAVE1B_CRATES, CLAUDE.md, README.md)
- [x] 154-03-PLAN.md — SeaORM migration CreateReservationsTable + entity Model (12 columns + 2 composite indexes) + lib.rs entity re-exports
- [x] 154-04-PLAN.md — Leaf-type bodies: Resource trait, ReservationContext builder, ReservationEvent + ReleaseReason (serde + Event impl), ReservationHandle serde tests
- [x] 154-05-PLAN.md — ReservationKernel<R> with hold/commit/release/extend (GuardedUpdate + AuditEntry + dispatch ordering) + 7 unit tests covering D-47-1..7
- [x] 154-06-PLAN.md — run_sweep_once + concurrent_hold integration (D-48) + proptest property tests (D-49) + cross-crate integration (D-50)
- [x] 154-07-PLAN.md — Release: user doc page reservations.md, CHANGELOG entry, pre-release gate (fmt+clippy+test+doc), manual first-publish bootstrap to crates.io

### Phase 155: ferro-projection crate — live read-model from domain events with delta broadcast

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** none (uses existing ferro-events + ferro-broadcast)
**Plans:** 7/7 plans complete

Plans:
- [x] TBD (run /gsd-plan-phase 155 to break down) (completed 2026-05-14)

### Phase 156: frontend/src/types/ — Generator-Owned Convention Cleanup

**Goal:** Reconcile the contradiction between the scaffold gitignore template (which marks `frontend/src/types/` as generator-owned) and Ferro's reference app (which tracks generated files). Untrack generated files in the reference app, add a `ferro doctor` check for hand-written files in `frontend/src/types/`, update the Dockerfile renderer to add a `types-gen` stage so Docker builds work without committed generated files, fix the generator header comment, and document the convention.
**Requirements**: D-01..D-21 (decision IDs from 156-CONTEXT.md — no formal REQ-IDs assigned for this phase)
**Depends on:** none
**Plans:** 6/6 plans complete

Plans:
- [x] 156-01-PLAN.md — trivial fixes: untrack reference app types, gitignore comment, generate_types.rs header path
- [x] 156-02-PLAN.md — new doctor check `frontend_types_convention` + registry + tests
- [x] 156-03-PLAN.md — Dockerfile renderer: DockerContext.ferro_version, types-gen stage, resolve_ferro_version helper
- [x] 156-04-PLAN.md — wire docker_init.rs and docker_template_drift.rs to the real resolve_ferro_version (replaces Plan 03 placeholders)
- [x] 156-05-PLAN.md — docs: frontend-types.md page, SUMMARY index, doctor.md count + table, reference/cli.md count, README.md.tpl troubleshooting bullet
- [x] 156-06-PLAN.md — workspace version bump + CHANGELOG entry + pre-release gate + human-authorized push

---

### 📋 v11.12 Migration Deploy Safety (Phase 157, planned 2026-05-13, URGENT)

**Source:** gestiscilo-it 2026-05-13 production breakage. Field-test detail in [phases/157-.../157-CONTEXT.md](phases/157-migration-deploy-safety-backend-portable-backfill-helpers-fe/157-CONTEXT.md).

**What ships:** three framework gaps closed in one phase — backend-portable backfill helpers, scaffolder-emitted PRE_DEPLOY migrate job, doctor `migrate_gate` check.

### Phase 157: Migration deploy safety — backend-portable backfill helpers, ferro do:init PRE_DEPLOY migrate job, ferro doctor check for migrate gate

**Goal:** Close three migration-deploy-safety gaps surfaced by the 2026-05-13 gestiscilo-it production breakage so the next consumer cannot rediscover them — a new `ferro-migration` crate exporting backend-portable backfill helpers (`backfill_random_hex`, `backfill_random_uuid`, `backfill_current_timestamp`, `backfill`), `ferro do:init` scaffolding a `PRE_DEPLOY` migrate job in `.do/app.yaml` by default, a `ferro doctor --deploy` `migrate_gate` check that errors when migrations exist without a PRE_DEPLOY gate, and fixing `run_migrations_silent` to `process::exit(1)` on failure across framework + sample app + new-project template.
**Requirements**: D-01, D-02, D-03, D-04, D-05, D-06
**Depends on:** none (independent of v11.11 primitives; can land in parallel)
**Plans:** 4/4 plans complete

Plans:
- [x] 157-01-PLAN.md — ferro-migration crate: backend-portable backfill helpers + workspace + CI Wave 1a
- [x] 157-02-PLAN.md — ferro do:init {{JOBS_BLOCK}} PRE_DEPLOY migrate job
- [x] 157-03-PLAN.md — ferro doctor migrate_gate check (CheckCategory::Deploy)
- [x] 157-04-PLAN.md — Fix run_migrations_silent silent-failure anti-pattern (framework + app + template)

### Phase 158: Request::file() multipart upload primitive

**Goal:** Add multipart/form-data parsing to the framework so handlers can receive uploaded files via `req.multipart()` and `req.file("field")`. Include an `UploadedFile` type with a `store()` helper that bridges directly to `ferro-storage`. Killer feature: a handler can receive an uploaded file and persist it to local disk or S3 in three lines, using the same `ferro-storage` API already wired into the app.
**Requirements**: MULTIPART-01..09
**Depends on:** Phase 157
**Plans:** 2/2 plans complete

Plans:
- [x] 158-01-PLAN.md — Add multer dep, create http/multipart.rs (UploadedFile, MultipartForm, parser, validators, env helpers), wire into http/mod.rs + lib.rs
- [x] 158-02-PLAN.md — Add Request::multipart() / Request::file() methods + #[cfg(test)] mod tests covering D-03/04/07/08/12/13/14/18
### Phase 159: v12.0 end-to-end browser verification and docs build check

**Goal:** Confirm the v12.0/json-ui-v2 branch delivers what it promises before touching the v1 API. Start the ferro sample app, hit `/pagamenti` via Chrome MCP and verify `JsonUi::render_file` produces a correctly rendered HTML page end-to-end. Then run `mdbook build docs/` and confirm the rewritten JSON-UI docs build with no broken links. Both checks must pass before v1 removal begins.
**Requirements**: Chrome MCP browser test of /pagamenti passes; `mdbook build docs/` exits cleanly with no broken links.
**Depends on:** Phase 121
**Plans:** 3/3 plans complete

Plans:
- [x] 159-01-PLAN.md — Run mdbook build docs/ and produce DOCS-CHECK.md verdict (docs half of the phase gate)
- [x] 159-02-PLAN.md — Chrome MCP test of /pagamenti at http://localhost:8080, capture screenshot, produce BROWSER-CHECK.md verdict (browser half of the phase gate)
- [x] 159-03-PLAN.md — Gap closure: fix render_file path bug in app/src/controllers/pagamenti.rs (line 34), re-run Chrome MCP test, overwrite BROWSER-CHECK.md with PASS verdict (closes Phase 159 gate D-11) (completed 2026-05-15)

### Phase 160: Remove v1 JSON-UI API from ferro-json-ui — delete view.rs, Component enum, ComponentNode and all v1 builder surface

**Goal:** Permanently delete all v1 API surface from ferro-json-ui: `view.rs` (`JsonUiView`, `SCHEMA_VERSION = "ferro-json-ui/v1"`), `Component` enum and all typed `*Props` structs that are not reused by v2 (`ComponentNode`, builder convenience methods on `JsonUiView`). No `#[deprecated]` attributes, no feature flags, no compat shims. The crate public surface after this phase exposes only `Spec`, `Element`, `SpecBuilder`, `ElementBuilder` and the expression/render pipeline. Gate: all three repos (`ferro`, `ferro-code`, `gestiscilo`) compile and their test suites pass after deletion. **Depends on gestiscilo Phase 143 being complete** — do not start until gestiscilo no longer imports any v1 type.
**Requirements**: `cargo build --all-features` green; `cargo test --all-features` green; `cargo clippy --all --all-targets -- -D warnings` clean; no reference to `JsonUiView`, `ComponentNode`, `Component::` remains in any crate.
**Depends on:** Phase 159, Phase 164
**Plans:** 10/10 plans complete

Plans:
- [x] 160-01-PLAN.md — Rewrite v1-framing doc comments in ferro-json-ui/src/render/* + projection/builder.rs + layout.rs (D-01, D-02, D-03, Pattern-1, Pattern-8)
- [x] 160-02-PLAN.md — Delete migration_v1_to_v2_templates fn, registration, and integration test in ferro-mcp/src/tools/code_templates.rs (D-04, Pattern-3)
- [x] 160-03-PLAN.md — Rewrite scan_json_ui_specs to count v2 JSON spec files + add unit tests (D-05, Pattern-2)
- [x] 160-04-PLAN.md — Rename json_ui_inspect test fixture to neutral names (D-06, Pattern-4)
- [x] 160-05-PLAN.md — Rewrite ferro-json-ui/README.md Usage block to current v2 API — Phase 161 publish blocker (D-08, Pattern-6)
- [x] 160-06-PLAN.md — Rewrite docs/protocol/src/{terminology,architecture,rendering}.md JsonUiRenderer paragraphs to v2 Spec shape (D-07, Pattern-5)
- [x] 160-07-PLAN.md — Sync docs/src/features/projections.md minimal example to ferro-json-ui/src/projection/mod.rs:79-97 rustdoc (D-07, Pattern-5)
- [x] 160-08-PLAN.md — Rewrite docs/src/reference/cli.md make:json-view example to current CLI output (JSON spec + handler) (D-08, Pattern-7)
- [x] 160-09-PLAN.md — D-08 broad narrative-framing sweep + AUDIT-D08.md classification report (D-08)
- [x] 160-10-PLAN.md — Final verification gate: D-10 grep gates + ferro fmt/clippy/test + gestiscilo cross-repo + ferro-code descope (D-09, D-10, D-11)

### Phase 161: Merge v12.0/json-ui-v2 to master — full test pass, clippy clean, merge PR

**Goal:** Final integration step closing the v12.0 milestone. Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` on the v12.0/json-ui-v2 branch. Fix any remaining issues. Create the merge PR from v12.0/json-ui-v2 → master, confirm CI passes, merge.
**Requirements**: All CI checks green; master HEAD contains Phase 115–121 and 159–160 commits; v12.0/json-ui-v2 branch archived.
**Depends on:** Phase 160
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 161 to break down)

### Phase 162: JSON-UI improvements batch 1 — components, expressions, and spec ergonomics discovered during gestiscilo auth/dashboard migration

**Goal:** Read the FRICTION.md files produced by gestiscilo Phases 138 and 139 (auth/account/onboarding/pages and dashboard/statistiche/settings). Triage every friction point: missing component, awkward prop shape, expression gap, spec authoring pain, or render bug. Implement the highest-value fixes — new or improved components, expression enhancements, catalog accuracy fixes, or `render_file` ergonomics. Publish the results so gestiscilo Phases 140+ can benefit from them. Each shipped fix must have a test. Dropped items are documented with rationale in a DEFERRED.md.
**Requirements**: All friction items triaged; shipped fixes have tests; ferro-json-ui builds clean; catalog and MCP tool descriptions updated to reflect new surface; gestiscilo can pick up the new ferro version for Phase 140.
**Depends on:** gestiscilo Phase 139
**Plans:** 10/11 plans complete

Plans:
- [x] 162-01-PLAN.md — Add CheckboxList first-class component (D-01/D-02)
- [x] 162-02-PLAN.md — Generalize DataTable row_actions URL placeholder interpolation (D-03/D-04)
- [x] 162-03-PLAN.md — Re-add SwitchProps.compact and ImageProps.inline_svg (D-16/D-17)
- [x] 162-04-PLAN.md — Re-implement RichTextEditor as v2 plugin (D-18)
- [x] 162-05-PLAN.md — Triple-lockstep catalog count reconciliation (D-21)
- [x] 162-06-PLAN.md — Remove AuthLayout card wrapper (D-05/D-06)
- [x] 162-07-PLAN.md — Spec footer-ID validation: error + duplicate warning (D-07/D-08)
- [x] 162-08-PLAN.md — strum::AsRefStr derive on six variant enums (D-11/D-12)
- [x] 162-09-PLAN.md — json_ui_verify_action MCP tool (D-09/D-10)
- [x] 162-10-PLAN.md — migration-v1-to-v2 docs + plugins guide + code_templates (D-13-D-15, D-19, D-20, D-22)
- [ ] 162-11-PLAN.md — Phase verification gate: full suite + CHANGELOG + human audit (D-23-D-25)


### Phase 163: JSON-UI improvements batch 2 — iteration directives and spec construction ergonomics

**Goal:** Ship the iteration-and-ergonomics slice of gestiscilo Phase 138 FRICTION.md. Adds two element-level directives (`$each` for homogeneous list iteration, `$if` for conditional emission), a validator gate for malformed directives, an ergonomic nested-tree `SpecBuilder` layer for truly heterogeneous Rust-side construction, an AST-based `ferro json-ui:migrate-v1` codemod, and MCP catalog reflection. Closes 3 of 4 cassa heterogeneous-iteration friction sites; the 4th (orders detail conditional actions) is covered by `$if`.
**Requirements**: 13 locked CONTEXT decisions (D-01 through D-13) implemented with tests; ferro-json-ui builds clean; ferro-cli codemod has fixture-driven integration tests; docs/src/json-ui/spec-construction.md ships the four-quadrant decision rubric.
**Depends on:** Phase 162, gestiscilo Phase 138
**Plans:** 10/10 plans complete

Plans:
- [x] 163-01-PLAN.md — Add `$each` (EachDirective struct + Element.each field + serde tests)
- [x] 163-02-PLAN.md — Add `$if` (Element.if_ field reusing Visibility enum + serde tests)
- [x] 163-03-PLAN.md — `expand_directives` resolve pass + JsonUi::resolve wiring + 12 unit tests
- [x] 163-04-PLAN.md — Validator gates (5 SpecError variants + validate_directives + 11 unit tests)
- [x] 163-05-PLAN.md — SpecBuilder ergonomic layer (NestedElement + element_nested + 7 tests)
- [x] 163-06-PLAN.md — MCP `json_ui_catalog` reflects directives (DirectiveInfo + 3 tests)
- [x] 163-07-PLAN.md — `ferro json-ui:migrate-v1` AST codemod (subcommand + fixtures + 5 integration tests)
- [x] 163-08-PLAN.md — End-to-end directive integration tests (4 tests against full pipeline)
- [x] 163-09-PLAN.md — Decision rubric docs (spec-construction.md + expressions.md $each/$if sections)
- [x] 163-10-PLAN.md — CHANGELOG entry under Unreleased (no version bump per Phase 161 release cadence)


### Phase 163.1: Codemod multi-root handler fix (G-163-01) — reject as Unsupported with TODO marker (INSERTED)

**Goal:** Close the WR-01 finding from 163-REVIEW.md: the `ferro json-ui:migrate-v1` codemod silently orphans elements when a v1 handler has multiple top-level nodes (root set to first node only, remaining elements unreachable from root). Apply Option B from the code review — reject multi-root handlers as Unsupported, emit the existing `// TODO: codemod could not auto-translate` marker on the controller, do not produce a JSON spec file. Aligns with D-11 from Phase 163 CONTEXT ("codemod is best-effort; cases it cannot translate get a TODO marker, not a silent skip").
**Requirements**: Multi-root handler detection runs before `Spec::builder()` construction; affected handler gets the TODO marker; existing `out_auth_login_form.json` fixture deleted (or replaced with a single-root variant); integration test `codemod_one_handler_emits_spec_and_rewrites_controller` rewritten to assert TODO marker for multi-root input AND clean JSON for a single-root input; ferro-cli tests pass clean; clippy + fmt clean.
**Depends on:** Phase 163
**Plans:** 1/1 plans complete

Plans:
- [x] 163.1-01-PLAN.md — Add multi-root guard in try_migrate_handler, delete invalid out_auth fixtures, create single-root fixture trio, rewrite integration test to cover both branches, fmt+clippy+ferro-cli tests clean


### Phase 164: JSON-UI improvements batch 3 — V7-RUNTIME frictions (F1–F10), v1-deletion-readiness audit, COMPLETED.md

**Goal:** Absorb two friction sources into the closing batch of the v12.0 loop. (a) **V7-RUNTIME-FRICTION.md** (gestiscilo, 2026-05-17) — ten runtime frictions discovered after the patched ferro at 162/163.1 went active; F1/F2 already fixed gestiscilo-side, F3/F4/F7/F8/F9/F10 require ferro changes (decisions D-12..D-18 in 164-CONTEXT), F5/F6 are gestiscilo-side fixes with optional ferro pre-empt (D-19). (b) **Residual Phase 138 FRICTION.md items** not absorbed by Phase 162 or 163, plus the v1-deletion-readiness audit gating Phase 160. Produce COMPLETED.md summarising all improvements shipped across Phases 162-164 and any intentional gaps retained for future milestones.
**Requirements**: V7-RUNTIME F3/F4/F7/F8/F9/F10 land as ferro fixes with tests; F5 error message improved; F2 codemod uppercase-methods fix shipped; v1 deletion audit produces zero `BLOCKER` rows; all friction items triaged; ferro-json-ui builds clean; COMPLETED.md written; ferro Phase 160 (v1 deletion) is unblocked.
**Depends on:** Phase 163.1, gestiscilo V7-RUNTIME-FRICTION.md (consumed)
**Plans:** 12/12 plans complete

Plans:
- [x] 164-01-PLAN.md — D-14: Raise MAX_NESTING_DEPTH 3→5 + tests + doc (spec.rs)
- [x] 164-02-PLAN.md — D-19/F2: Codemod uppercase HTTP methods regression test (ferro-cli)
- [x] 164-03-PLAN.md — D-15 + D-17a: Image/DescList data_path + RawHtml component + catalog count bumps (component.rs, render/atoms, catalog, ferro-mcp)
- [x] 164-04-PLAN.md — D-12: Spec.title binding (TitleBinding/DataRef enums + framework title resolution)
- [x] 164-05-PLAN.md — D-18: CardVariant enum (Bordered/Elevated) + render_card branch
- [x] 164-06-PLAN.md — D-13a: KanbanBoard.data_path + render_kanban_board branch
- [x] 164-07-PLAN.md — D-16: Two-stage validation pipeline (structural at load, catalog at render after expand_directives)
- [x] 164-08-PLAN.md — D-19/F5 + D-19/F6: Visibility custom Deserialize (shape-naming error) + PageHeader.actions lax deserialize_with
- [x] 164-09-PLAN.md — D-04 + D-05: MCP json_ui_validate_spec tool + directive validator audit
- [x] 164-10-PLAN.md — D-08 + D-09 + D-13b: Documentation pass + v1→v2 cheat sheet + $each-for-kanban example
- [x] 164-11-PLAN.md — D-01..D-03 + D-06..D-07: V1-DELETION-AUDIT.md + Plugin paper audit (CHECKPOINT)
- [x] 164-12-PLAN.md — D-10..D-11: COMPLETED.md (5 required sections; unblocks Phase 160)

**Wave structure (7 waves):**
- Wave 1: 01, 02, 03 (independent — no file conflicts)
- Wave 2: 04 (spec.rs + framework/json_ui/mod.rs + lib.rs; sequential after 01+03), 05 (component.rs + containers.rs + lib.rs; sequential after 03)
- Wave 3: 06 (containers.rs sequential after 05), 07 (framework/json_ui/mod.rs sequential after 04)
- Wave 4: 08 (component.rs sequential after 05+06), 09 (depends on 07 for two-stage output)
- Wave 5: 10 (docs — depends on all code waves 01-09)
- Wave 6: 11 (audit — depends on all prior including docs; ends with user checkpoint)
- Wave 7: 12 (COMPLETED.md — depends on 11's audit output)

---

## Progress Summary

| Milestone | Phases | Plans | Status | Shipped |
|-----------|--------|-------|--------|---------|
| v1.0 DX Overhaul | 1-12 | 18 | ✅ Complete | 2026-01-16 |
| v2.0 Rebrand | 13-22 | 13 | ✅ Complete | 2026-01-16 |
| v2.0.1 Macro Fix | 22.1-22.3 | 6 | ✅ Complete | 2026-01-17 |
| v2.0.2 Type Generator Fixes | 22.4-22.9 | 6 | ✅ Complete | 2026-01-17 |
| v2.0.3 DO Apps Deploy | 22.10 | 1 | ✅ Complete | 2026-01-17 |
| v2.1 Inertia DX & Fixes | 33-34 | 4 | ✅ Complete | 2026-01-17 |
| v2.2 CLI Improvements | 35-37 | 5 | ✅ Complete | 2026-02-09 |
| v3.0 JSON-UI | 23-32 | 24 | ✅ Complete | 2026-02-09 |
| v4.0 Production Readiness | 38-46 | 24 | ✅ Complete | 2026-02-10 |
| v5.0 Proximity — JSON-UI Field Test | 47-53 | 20 | ✅ Complete | 2026-02-10 |
| v5.1 Housekeeping | 54-57 | 5 | ✅ Complete | 2026-02-13 |
| v6.0 ferro-lang — Localization | 58-66 | 11 | ✅ Complete | 2026-02-13 |
| v6.1 Fix Known Issues | 67 | 1 | ✅ Complete | 2026-02-24 |
| v7.0 Resend Integration | 68 | 3 | ✅ Complete | 2026-02-25 |
| v7.4 Security Hardening | 72-74 | 5 | ✅ Complete | 2026-02-26 |
| v7.5 Type Generator Fix | 75 | 1 | ✅ Complete | 2026-02-27 |
| v7.6 Default API Scaffold | 76 | 4 | ✅ Complete | 2026-02-27 |
| v7.7 Validate & Fix API Scaffold | 77 | 3 | ✅ Complete | 2026-02-28 |
| v7.8 Memory Leak Fixes | 78 | 3 | ✅ Complete | 2026-02-28 |
| v8.0 Consumer MCP — OpenAPI Bridge | 79-82 | 11 | ✅ Complete | 2026-02-28 |
| v8.1 API DX Polish | 83 | 5 | ✅ Complete | 2026-02-28 |
| v9.0 Service Projections | 84-94 | 30 | ✅ Complete | 2026-03-01 |
| v10.0 JSON-UI Visual Overhaul | 102-107 | 8 | ✅ Complete | 2026-03-26 |
| v11.0 Framework Consolidation Audit | 108-114 | 13 | ✅ Shipped | 2026-04-05 |
| v11.1 Template Renderer | 114.1 | 1 | ✅ Shipped | 2026-04-05 |
| v11.2 Deploy & Scaffolder Hardening | 122-131 | 49 | ✅ Shipped | 2026-04-14 |
| v11.3 S3 Storage Driver | 132 | 1 | ✅ Shipped | 2026-04-14 |
| v11.5 Projection Architecture Prep | 133-135 | 4 | ✅ Shipped | 2026-04-17 |
| v11.6 ferro-stripe Capability Refactor | 140-142 | 11 | ✅ Shipped | 2026-04-20 |
| v11.7 Tailwind Static CSS Pipeline | 143 | 4 | ✅ Shipped | 2026-04-21 |
| v12.0 JSON-UI v2 — Spec-Driven Rendering | 115-121, 159-164 | 491 commits | ✅ Shipped | 2026-05-19 |
| v12.2 Frontend Performance Hardening | 182-184 | 10 | ✅ Shipped | 2026-06-06 |
| v12.1 AI — ferro-ai SDK & AI as Projection Consumer | 165-173 | — | 📋 Planned (not started) | - |

**Total: 30 milestones shipped, plan totals approximate (v12.0 logged 491 commits in the top-line milestone log).**

> **Reconciliation notes (2026-06-07 audit):** v12.0 milestone header and Phase 115/119 checkboxes were stale — flipped to `✅` based on the top-line milestone log entry at the start of this file. Standalone shipped phases not yet attached to a milestone: **179** (DataTable RawHtml-free heterogeneous rows — shipped 2026-05-25 in v0.2.38; closed with SUMMARY.md, no separate VERIFICATION.md), **180** (Declarative action handler primitive), **181** (JSON-UI Input error prop inline render). Removed during this audit: **178** (json-ui plugin registry refactor — never planned, only a `.gitkeep` placeholder; architectural intent preserved in memory `project_ferro_json_ui_plugin_registry_debt.md`). v12.1 Form Validation DX (phases 137-139) has no phase directories in `.planning/phases/` — either deferred or relocated to the gestiscilo repo.


---

### 📋 v12.1 AI — ferro-ai SDK & AI as Projection Consumer (Phases 165-173, planned 2026-05-15, reframed 2026-06-07)

**Milestone Goal:** Expand `ferro-ai` into a production-grade, provider-agnostic AI SDK and make AI a first-class consumer of the projection / intent core. The killer feature: `ferro ai:make <description>` produces a typed `ferro_projections::ServiceDef` — the universal projection contract. The existing rendering pipeline (`ferro-json-ui` renderer, `ferro-mcp` introspection renderer, future modality renderers) covers everything downstream. AI does NOT recreate the pre-projections multi-file scaffolding workflow; it generates the input the projection layer already knows how to render. Live `ferro-mcp` introspection (called in-process, not via subprocess) supplies the project-specific context so generated `ServiceDef`s reference existing models, intents, and conventions rather than generic templates.

**Conceptual coherence anchor:** Every AI surface either produces or consumes a `ServiceDef`. The structured-outputs schema normalizer is `ServiceDef`-aware so the LLM cannot drift from the intent system. See `.planning/REQUIREMENTS.md` for the full anti-requirements set.

**Requirements:** AISDK-01..06, AISSE-01..02, AICLI-01..06 (14 requirements; AICLI-04 was deferred pending v12.0 — now unblocked since v12.0 shipped 2026-05-19)

**Relationship to v12.0:** v12.0 shipped 2026-05-19. AICLI-04 (`make:json-view` v2 — the first concrete `Renderer` over an AI-produced `ServiceDef`) is now unblocked and joins Phase 171 in closing the produce-then-render loop end-to-end via AICLI-06.

**New dependencies:**
- `reqwest-eventsource 0.6` — parse incoming SSE from Anthropic/OpenAI/Groq/Ollama (new to workspace)
- `pgvector 0.4` — optional feature-gate on ferro-ai for vector storage (new to workspace)

**Build order:**
- Wave 1 (ferro-ai foundation): Phases 165, 166, 167 — ferro-ai leaf crate first; everything builds on `LlmClient` trait. Phase 166's schema normalizer must ship the `ServiceDef`-aware path here (required by the killer feature in Wave 4).
- Wave 1b (parallel): Phase 168 — SSE primitives in framework have no ferro-ai dependency
- Wave 2: Phase 169 — StreamText depends on SSE URL convention from Phase 168
- Wave 3: Phase 170 — ferro-cli migration, validates SDK against existing `make:json-view` command
- Wave 4: Phase 171 — `ai:make` produces `ServiceDef`, `ai:explain` returns projection-framed explanation; uses ferro-mcp in-process. Killer feature.
- Wave 5: Phase 172 — MCP tool wrappers (thin layer on top of CLI logic); Phase 173 — `make:json-view` v2 as the first concrete `Renderer` over a ServiceDef produced by `ai:make`, plus AICLI-06 projection-roundtrip test closing the loop end-to-end.

## Phases

- [x] **Phase 165: LlmClient Trait & Provider Implementations** — `LlmClient` trait + Anthropic/OpenAI/Ollama providers + `AiConfig::from_env()` + `ClassifierConfig` default-model fix (completed 2026-06-08)
- [x] **Phase 166: Structured Outputs, Tool Calling & ServiceDef-aware Schema Normalizer** — `ferro_ai::complete::<T>()` + generic schema normalizer (resolves `$ref`/`$defs`, adds `additionalProperties: false`) + `ServiceDef`-aware specialization that locks the LLM to valid projection shapes when `T` is `ferro_projections::ServiceDef` + `ToolRegistry` with `max_iterations` hard cap (completed 2026-06-08)
- [x] **Phase 167: Embeddings & pgvector** — `embed()` + `cosine_similarity()` pure Rust helpers + optional `pgvector` feature-gated module (completed 2026-06-08)
- [x] **Phase 168: Framework SSE Primitives** — `SseEvent` + `SseStream` + `HttpResponse::sse()` in framework crate; SSE routes structurally excluded from CompressionLayer (completed 2026-06-08)
- [x] **Phase 169: StreamText Component** — `StreamText` ferro-json-ui component rendering a token stream from an SSE endpoint URL (completed 2026-06-08)
- [x] **Phase 170: ferro-cli Migration** — delete `ferro-cli/src/ai.rs` blocking client; wire all LLM calls through `ferro_ai::complete::<T>()` (completed 2026-06-08)
- [x] **Phase 171: ferro ai:make & ferro ai:explain CLI Commands** — killer-feature commands. `ai:make <description>` produces a typed `ferro_projections::ServiceDef` (NOT a multi-file scaffold bundle and NO `ScaffoldPlan` intermediary — structured outputs complete directly into the projection contract). `ai:explain <route|model|service>` returns a projection-framed explanation (`Intent`, `FieldMeaning`, `ActionDef`/`GuardDef`, `StateMachine`). Live ferro-mcp introspection in-process; selective context loading. (completed 2026-06-08)
- [x] **Phase 172: MCP Tool Wrappers** — `ai_scaffold` + `ai_explain` tools in ferro-mcp wrapping CLI command logic for in-process agent consumption. `ai_scaffold` returns the same `ServiceDef` shape the CLI produces — no parallel surface. (completed 2026-06-08)
- [x] **Phase 173: make:json-view v2 + projection-roundtrip test** — `ferro make:json-view` upgraded to structured outputs + `ServiceDef` introspection. The first concrete `Renderer` over a ServiceDef produced by `ai:make`. Includes AICLI-06 projection-roundtrip test (NL description → `ServiceDef` → rendered JSON-UI spec) as the structural proof that AI is a first-class projection consumer. (completed 2026-06-09)

#### Phase Details

### Phase 165: LlmClient Trait & Provider Implementations
**Goal**: Establish the provider-agnostic `LlmClient` trait and ship four provider implementations (Anthropic, OpenAI, Ollama, plus Groq as an OpenAI config variant). Fix the `ClassifierConfig` hardcoded default model that breaks non-Anthropic providers.
**Depends on**: Nothing (first phase of milestone; ferro-ai is a leaf crate)
**Requirements**: AISDK-01
**Success Criteria** (what must be TRUE):
  1. `LlmClient` trait exists in `ferro-ai/src/client/mod.rs` with `async fn complete(...)`, `async fn complete_stream(...)`, `async fn embed(...)` methods; missing capabilities return `Err(Error::Unsupported)` rather than panic
  2. `AnthropicClient`, `OpenAiClient` (doubles as Groq via `base_url` override), and `OllamaClient` implement `LlmClient`; `Box<dyn LlmClient>` is instantiable for each
  3. `AiConfig::from_env()` reads `FERRO_AI_PROVIDER`, `FERRO_AI_MODEL`, `FERRO_AI_API_KEY`, `FERRO_AI_BASE_URL` and returns the correct provider; unknown provider names return a clear error at startup, not at the first LLM call
  4. `ClassifierConfig` default model is resolved through `LlmClient::default_model()` per provider; the hardcoded `"claude-sonnet-4-6"` string is removed from `ClassifierConfig::default()`
  5. `Classifier<T>` compiles and passes its existing tests with the new client plumbing underneath; `ClassificationProvider` and existing public API are preserved
  6. `reqwest-eventsource 0.6` is declared as a `pub(crate)` dependency in provider modules only — not re-exported as a public ferro-ai surface
**Plans**: 4 plans
- [x] 165-01-PLAN.md — Foundation: streaming deps + Error restructure (Unsupported/Provider{status,message}/is_retryable) + LlmClient trait + CompletionRequest/TokenStream + client module skeleton
- [x] 165-02-PLAN.md — AnthropicClient + OpenAiClient (Groq via base_url): complete/complete_stream (SSE)/embed/default_model
- [x] 165-03-PLAN.md — OllamaClient: NDJSON streaming, no-auth local default, /api/chat + /api/embed
- [x] 165-04-PLAN.md — Convergence: AiConfig::from_env() + AnthropicProvider→AnthropicClient bridge (delete dup HTTP) + classifier retry/default-model fix + lib.rs re-exports + phase gate

### Phase 166: Structured Outputs, Tool Calling & ServiceDef-aware Schema Normalizer
**Goal**: Ship `ferro_ai::complete::<T>()` for typed structured outputs, the schema normalizer that makes `schemars` output compatible with provider structured-output APIs, the `ServiceDef`-aware specialization that locks the LLM to valid projection shapes, and `ToolRegistry` with a hard `max_iterations` guard.
**Depends on**: Phase 165
**Requirements**: AISDK-02, AISDK-03
**Success Criteria** (what must be TRUE):
  1. `ferro_ai::complete::<T>(client, prompt)` where `T: schemars::JsonSchema + serde::DeserializeOwned` returns `Result<T, Error>` — caller never calls schemars or JSON parsing directly
  2. `ferro_ai::schema::for_structured_output(root_schema)` resolves all `$ref`/`$defs` inline, adds `additionalProperties: false` to every object schema, and strips constraints Anthropic structured-output rejects; a unit test verifies the output against Anthropic's documented constraints
  3. **`ServiceDef`-aware path:** when `T` is `ferro_projections::ServiceDef` (or contains one), the normalizer constrains the schema to valid projection shapes: `FieldMeaning` enum values, `Intent` enum (Browse / Focus / Collect / Process / Summarize / Analyze / Track), `Cardinality` enum, `ActionDef` / `GuardDef` / `StateDef` shapes derived from `ferro-projections`. A unit test asserts the LLM cannot produce a schema-passing `ServiceDef` that contains an invalid `FieldMeaning` or `Intent` value. This is the structural guarantee referenced by AISDK-02's projection-coherence clause.
  4. `ToolDef` struct carries `name: String`, `description: String`, `parameters_schema: serde_json::Value` (normalized via `for_structured_output`), and a handler closure
  5. `ToolRegistry::dispatch(messages, client)` runs the tool-calling loop; `max_iterations: u32` (default 10) is required at construction time and enforced with no override path to an unbounded loop; a warning is logged at 5 iterations and an error at the hard cap
  6. Tool errors carry model-legible `ToolError { message: String }` descriptions — not raw Rust stack traces or DB constraint strings
  7. `cargo test --all-features` passes; existing `Classifier<T>` tests are green
**Plans**: 5 plans
- [x] 166-01-PLAN.md — Wave 0 foundation: deps (schemars, ferro-projections, jsonschema dev) + Error variants + schemars anyOf shape probe (resolves A1/A2)
- [x] 166-02-PLAN.md — generic `for_structured_output` normalizer (resolve $ref/$defs, strip Anthropic-rejected keywords, additionalProperties:false) + SC#2
- [x] 166-03-PLAN.md — ServiceDef-aware enum closing (D-06/07/08) + SC#3 jsonschema test + typed `complete::<T>()` (D-01) + SC#1
- [x] 166-04-PLAN.md — tool calling: client `complete_with_tools` (D-14) + `ToolDef`/`ToolRegistry`/`ToolError` + dispatch max_iterations loop (SC#4/5/6)
- [x] 166-05-PLAN.md — publish.yml WAVE1B reorder (ferro-projections before ferro-ai) + full fmt/clippy/test gate (SC#7)

### Phase 167: Embeddings & pgvector
**Goal**: Ship pure-Rust embedding helpers and cosine similarity, plus an optional pgvector integration for semantic search.
**Depends on**: Phase 165
**Requirements**: AISDK-04, AISDK-05
**Success Criteria** (what must be TRUE):
  1. `ferro_ai::embed(client, text)` calls the provider's embedding endpoint and returns `Vec<f32>`; Anthropic, OpenAI, and Ollama providers implement `LlmClient::embed()`
  2. `ferro_ai::cosine_similarity(a: &[f32], b: &[f32]) -> f32` is a pure Rust function with no extra crates; returns a value in [-1.0, 1.0]; panics with a clear message on empty or dimension-mismatched inputs
  3. `ferro_ai::pgvector` module exists behind the `pgvector` cargo feature; `PgVectorStore::store` and `PgVectorStore::nearest` accept raw sqlx connections and return typed results
  4. Feature flag `pgvector` adds only `pgvector 0.4` to the dependency graph; non-flagged builds do not pull pgvector
  5. Unit tests for `cosine_similarity`: orthogonal vectors return 0.0, identical vectors return 1.0, opposite vectors return -1.0
**Plans**: 2 plans
- [x] 167-01-PLAN.md — Wave 1 (AISDK-04): cosine_similarity + embed() free fn + D-13 embed-model fix (Ollama/OpenAI) + Error::Sqlx variant + lib.rs core re-exports
- [x] 167-02-PLAN.md — Wave 2 (AISDK-05): pgvector feature + optional pgvector/sqlx deps + PgVectorStore (store/nearest) + gated integration test + SC#4 dep-graph assertion

### Phase 168: Framework SSE Primitives
**Goal**: Add SSE streaming support to the framework so handlers can push events to the browser. SSE routes are structurally excluded from CompressionLayer — this is a guarantee, not documentation.
**Depends on**: Nothing (parallel-capable with Phases 165-167; framework crate has no ferro-ai dependency)
**Requirements**: AISSE-01
**Success Criteria** (what must be TRUE):
  1. `SseEvent` exists in `framework/src/http/sse.rs` with `data`, `event`, `id`, and `retry` fields; serializes to the SSE wire format (`data: ...\n\n`) correctly
  2. `SseStream` wraps a tokio mpsc channel and implements `IntoResponse` for axum; `HttpResponse::sse(sender, stream)` factory constructs an SSE response
  3. SSE responses are excluded from `CompressionLayer` at the router level via a structural mechanism (not per-route annotation); the exclusion is tested, not only documented
  4. A keep-alive `:ping\n\n` comment is emitted every 15 seconds on idle SSE connections to prevent reverse-proxy idle-timeout disconnects
  5. An integration test verifies token-by-token delivery: a test SSE endpoint sends three events with delays; the test client receives each event before the next is sent
**Plans**: 2 plans
- [x] 168-01-PLAN.md — FerroBody enum + http_body::Body impl + 17-site Full<Bytes>→FerroBody refactor (load-bearing structural change; buffered-path regression green)
- [x] 168-02-PLAN.md — SseEvent wire serializer + SseStream keep-alive + HttpResponse::sse factory + full SSE unit suite

### Phase 169: StreamText Component
**Goal**: Ship the `StreamText` ferro-json-ui component that connects to an SSE endpoint URL and renders token-by-token output in place. No external JS framework required.
**Depends on**: Phase 168 (SSE URL convention established in framework)
**Requirements**: AISSE-02
**Success Criteria** (what must be TRUE):
  1. `Component::StreamText(StreamTextProps)` exists with `sse_url: String`, `placeholder: Option<String>`, and `loading_text: Option<String>` props; round-trips via ferro-json-ui serde fixtures
  2. Renderer emits `<div data-ferro-stream-url="{escaped_url}">` with a loading state and inline `EventSource` JS that appends tokens as they arrive
  3. `COMPONENT_CATALOG` and ferro-mcp `CatalogComponent` include `StreamText` with accurate prop descriptions for AI generation
  4. Documented under `### StreamText` in `docs/src/json-ui/components.md`
  5. `cargo clippy --all --all-targets -- -D warnings` and `cargo test --all-features` green
**Plans**: 3 plans
- [x] 169-01-PLAN.md — StreamTextProps struct + render_streamtext leaf renderer + escaping (Wave 1)
- [x] 169-02-PLAN.md — Registry sync (BUILTIN_TYPES/dispatch/count) + built-in init-script mechanism + EventSource JS + catalog registration (Wave 2)
- [x] 169-03-PLAN.md — ### StreamText docs section + event:done server contract (Wave 2)

### Phase 170: ferro-cli Migration
**Goal**: Delete the blocking Anthropic-only `ferro-cli/src/ai.rs` client and route all LLM calls through the `ferro_ai` SDK. Validates the SDK against the existing `make:json-view` command before new AI commands are built on top.
**Depends on**: Phase 166 (structured outputs and schema normalizer in place)
**Requirements**: AISDK-06
**Success Criteria** (what must be TRUE):
  1. `ferro-cli/src/ai.rs` is deleted; no `reqwest::blocking::Client` or direct Anthropic API calls remain in ferro-cli
  2. `ferro-cli` depends on `ferro-ai`; all LLM calls go through the ferro-ai SDK (`LlmClient::complete()`) using `AiConfig::from_env()` — see Phase 170 plan `<scope_note>` for why `complete::<T>()` is not literally applicable here (Pass 1 is schema-less plain text; Pass 2 must carry the catalog runtime schema, not a schemars-derived one)
  3. `ferro make:json-view` works end-to-end after the migration; existing behavior is preserved
  4. `FERRO_AI_PROVIDER`, `FERRO_AI_MODEL`, `FERRO_AI_API_KEY` env vars control the provider for `make:json-view` (previously only Anthropic was supported)
  5. `cargo test --all-features` passes; no new compilation warnings in ferro-cli
**Plans**: 1 plan (1 wave)
  - [x] 170-01-PLAN.md — Delete ai.rs, add ferro-ai dep, relocate transport-agnostic helpers, rewire make:json-view two-pass generation through AiConfig::from_env() + client.complete() with a tokio runtime bridge

### Phase 171: ferro ai:make & ferro ai:explain CLI Commands
**Goal**: Ship the killer-feature CLI commands. `ferro ai:make <description>` produces a typed `ferro_projections::ServiceDef` — the universal projection contract — using live ferro-mcp introspection loaded in-process (not subprocess). `ferro ai:explain <route|model|service>` returns a projection-framed explanation of an existing service using actual source loaded through ferro-mcp. **No `ScaffoldPlan` intermediary type; no multi-file scaffold output.** The existing rendering pipeline (Phase 173 `make:json-view` v2, ferro-mcp introspection renderer, future modality renderers) consumes the `ServiceDef` to produce downstream artifacts.
**Depends on**: Phase 170 (SDK migration complete), Phase 166 (structured outputs + `ServiceDef`-aware schema normalizer)
**Requirements**: AICLI-01, AICLI-02, AICLI-03
**Success Criteria** (what must be TRUE):
  1. `ferro ai:make <description>` calls ferro-mcp library functions in-process to load `list_routes`, `list_models`, `db_schema`, `generation_context`, and existing `ServiceDef`s in the project; context is filtered to items semantically relevant to the description before prompt construction (prevents context window overflow on large projects)
  2. `ferro ai:make` produces a typed `ferro_projections::ServiceDef` via `ferro_ai::complete::<ServiceDef>()` using the `ServiceDef`-aware schema normalizer path from Phase 166. The output is a single commit-ready `ServiceDef` definition — fields with `FieldMeaning`, `Intent` hints, `ActionDef`s with `GuardDef`s, `StateMachine` if stateful, `RelationshipDef`s with `Cardinality`. `--dry-run` prints the `ServiceDef` without registering it.
  3. `ferro ai:make` does NOT write a multi-file scaffold bundle. Downstream artifacts (rendered JSON-UI spec, route registration glue, migration scaffolding) are produced by existing `make:*` helpers consuming the `ServiceDef` — `make:json-view` v2 (Phase 173) is the primary downstream `Renderer`. There is no parallel file-writing path inside `ai:make`.
  4. `ferro ai:explain <route|model|service>` calls ferro-mcp introspection in-process; when a `ServiceDef` is found for the target, the explanation is projection-framed: the `Intent`s the service projects, which fields' `FieldMeaning`s drive the rendering, which `ActionDef`s are exposed under which `GuardDef`s, what state transitions exist via `StateMachine`. Plain code prose is the fallback only when no `ServiceDef` is found for the target.
  5. Both commands respect `FERRO_AI_MAX_TOKENS_PER_COMMAND` env var as a cost guard; both support `--dry-run`
  6. Neither command generates non-ferro code; the produced `ServiceDef` references existing models, intents, and conventions as reported by ferro-mcp introspection — not generic templates.
**Plans**: 4 plans
- [x] 171-01-PLAN.md — ferro-ai `complete_with::<T>()` + `CompleteOptions` (configurable max_tokens/system/model; cost-guard enabler)
- [x] 171-02-PLAN.md — `ferro ai:make`: in-process introspection + lexical relevance filter + ServiceDef→builder-source emitter + single-file output + sanitization + dry-run
- [x] 171-03-PLAN.md — `ferro ai:explain`: service→route→model resolution, projection-framed prompt, raw prose completion, dry-run
- [x] 171-04-PLAN.md — full CI gate (fmt+clippy -D warnings+test --all-features) + human-verify live ai:make/ai:explain quality (SC#4/SC#6)

### Phase 172: MCP Tool Wrappers
**Goal**: Expose `ai_scaffold` and `ai_explain` as ferro-mcp tools so agents can invoke `ServiceDef` production and projection-framed explanation logic in-process without shelling out to the CLI.
**Depends on**: Phase 171 (CLI command logic validated end-to-end)
**Requirements**: AICLI-05
**Success Criteria** (what must be TRUE):
  1. `ai_scaffold` MCP tool accepts `description: String` and returns a `ferro_projections::ServiceDef` JSON object (the same shape `ai:make` produces — no parallel surface, no `ScaffoldPlan` intermediary)
  2. `ai_explain` MCP tool accepts `target: String` (route path, model name, or service name) and returns the projection-framed explanation (`Intent`, `FieldMeaning`, `ActionDef` / `GuardDef`, `StateMachine`) as structured JSON; plain prose fallback when no `ServiceDef` is found
  3. Both tools share the same logic path as the CLI commands — no duplicate implementation
  4. MCP tool descriptions are accurate and sufficient for an agent to use them without out-of-band guidance
  5. `ferro-mcp` version bumped; `cargo test --all-features` passes
**Plans**: 4 plans (Wave 1: 172-01 relevance relocation + ENV_LOCK foundation; Wave 2: 172-02 scaffold_core + ai_explain_core cores; Wave 3: 172-03 register ai_scaffold/ai_explain MCP tools; Wave 4: 172-04 CLI rewire + version bump + docs + full gate)
Plans:
- [x] 172-01-PLAN.md — Relocate relevance filter into ferro-mcp (pub) + add test ENV_LOCK
- [x] 172-02-PLAN.md — scaffold_core + ai_explain_core async cores (structured + prose branches)
- [x] 172-03-PLAN.md — Register ai_scaffold + ai_explain #[tool] methods in service.rs
- [x] 172-04-PLAN.md — Delete CLI relevance dup, thin CLI wrappers, version 0.2.47, docs, full gate

### Phase 173: make:json-view v2 + projection-roundtrip test
**Goal**: Upgrade `ferro make:json-view` to use structured outputs with `ServiceDef` introspection and schema-driven component selection. This is the **first concrete `Renderer` over a `ServiceDef` produced by `ai:make`** (Phase 171). Ship AICLI-06 alongside: a single end-to-end test that runs NL description → `ServiceDef` (via `ai:make`) → rendered JSON-UI spec (via `make:json-view` v2) → renderable view. This roundtrip is the structural proof that AI is a first-class projection consumer rather than a parallel scaffolding system.
**Depends on**: Phase 171 (`ai:make` produces `ServiceDef`); Phase 170 (SDK migration). v12.0 Phase 117 / Phase 120 already shipped.
**Requirements**: AICLI-04, AICLI-06
**Status**: UNBLOCKED — v12.0 shipped 2026-05-19; this phase is now in the active build order.
**Success Criteria** (what must be TRUE):
  1. `ferro make:json-view` uses `catalog.prompt()` for concise AI context and `catalog.component_schema()` for per-component structured output (not the flat string prompt from v1)
  2. Generated views are v2 flat specs validated against `catalog.json_schema()` before being written to disk
  3. `make:json-view` consumes a `ServiceDef` (either freshly produced by `ai:make` or loaded from an existing project file); selection of JSON-UI components is driven by `FieldMeaning` and `Intent` from the `ServiceDef`, not by re-prompting the LLM about field types
  4. No v1 `JsonUiView` types appear in the generated output or the generation pipeline
  5. **Projection-roundtrip test** at `ferro-ai/tests/projection_roundtrip.rs`: a fixed NL description completes against `ai:make` → produces a deterministic-shape `ServiceDef` (asserted on `Intent` derivation outputs, `FieldMeaning` set, `ActionDef` set) → that `ServiceDef` runs through `make:json-view` v2 → produces a JSON-UI spec validated against `catalog.json_schema()`. The test passes via the `ServiceDef`-aware path; it cannot pass via the generic schema-normalization fallback.
**Plans**: 2 plans

Plans:
- [x] 173-01-PLAN.md — Rewire make:json-view to the ServiceDef-driven projection path (NL via scaffold_core or --from-service-json) feeding Spec::from_service_def; delete the direct NL→spec two-pass (AICLI-04)
- [x] 173-02-PLAN.md — Offline projection-roundtrip proof test (ferro-ai/tests/projection_roundtrip.rs) + ferro-json-ui dev-dep + 173-VERIFICATION.md; pins the ServiceDef-aware path via the Money→currency assertion (AICLI-06)

#### Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 165. LlmClient Trait & Providers | 4/4 | Complete    | 2026-06-08 |
| 166. Structured Outputs & Tool Calling | 5/5 | Complete    | 2026-06-08 |
| 167. Embeddings & pgvector | 2/2 | Complete    | 2026-06-08 |
| 168. Framework SSE Primitives | 2/2 | Complete    | 2026-06-08 |
| 169. StreamText Component | 3/3 | Complete    | 2026-06-08 |
| 170. ferro-cli Migration | 1/1 | Complete    | 2026-06-08 |
| 171. ai:make & ai:explain CLI Commands | 4/4 | Complete    | 2026-06-08 |
| 172. MCP Tool Wrappers | 4/4 | Complete    | 2026-06-08 |
| 173. make:json-view v2 + roundtrip | 2/2 | Complete    | 2026-06-09 |

### Phase 180: Declarative action handler primitive — typed Result return so POST handlers redirect-on-error without manual try-catch ladders

**Goal:** Ship `#[action(redirect_to = "/path")]` and the `ActionError` / `ActionOk` / `ActionResult` / `IntoActionError` runtime types so POST handlers can return `ActionResult` and use bare `?` end-to-end — failures redirect 303 with a structured flash payload instead of stranding the browser at the POST URL. Wraps Plan 01 runtime types in `framework/src/http/action.rs`, Plan 02 shared param-extraction refactor in `ferro-macros/src/utils.rs`, Plan 03 `#[action]` proc-macro, Plan 04 trybuild + integration test corpus, Plan 05 docs page. Consumer-side sweep across ~40-60 handlers is the friction-loop deliverable in the gestiscilo-it repo; this phase ships the primitive only.
**Requirements**: D-01 .. D-10 (locked decisions in 180-CONTEXT.md)
**Depends on:** Phase 179
**Plans:** 2/2 plans complete

Plans:
- [x] 180-01-PLAN.md — Runtime types (`ActionError`, `ActionOk`, `ActionResult`, `IntoActionError`, `handle_action_result`) in `framework/src/http/action.rs` + re-exports
- [x] 180-02-PLAN.md — Extract param-extraction helpers from `ferro-macros/src/handler.rs` into `ferro-macros/src/utils.rs` as `pub(crate)`
- [x] 180-03-PLAN.md — `#[action]` proc-macro in `ferro-macros/src/action.rs` + registration + re-export
- [x] 180-04-PLAN.md — trybuild UI tests + integration smoke tests + back-compat query-string verification
- [x] 180-05-PLAN.md — `docs/src/the-basics/action-handlers.md` user guide + SUMMARY.md wiring
- [x] 180-06-PLAN.md — `action_handler` MCP code template in `ferro-mcp/src/tools/code_templates.rs`

### Phase 181: JSON-UI Input — render `error` prop inline below the field

**Goal:** Fix the JSON-UI resolution pipeline so form-control error messages bound via `{"$data": "/<field>_error"}` or via `JsonUi::render_validation_error` actually reach `props.error` and render as the locked DOM shape `<p id="err-{field}" class="text-sm text-destructive">{error}</p>` below the offending field. Two pipeline fixes (D-02 root causes 1 & 2): (Fix A) `JsonUi::render` merges runtime `data` into a `spec.data` clone before `resolve_expressions` so `$data` bindings resolve against handler-supplied data; (Fix B) `attach_errors` writes singular `error: String` matching the form-control prop shape (was writing plural `errors: Vec<String>` which serde silently dropped). Bring `Checkbox`, `CheckboxList`, `Switch`, and `Input (file)` to error-state class+ARIA parity with `Input (text)` / `Select` (D-06). Cross-repo audit confirms no gestiscilo consumer was reading the pre-fix plural shape (D-08 clean break). Docs page covers the blessed `render_validation_error` path, the manual `$data` escape hatch, the flash round-trip, and the cross-field summary (D-09).

**Requirements**: D-01 .. D-09 (locked decisions in 181-CONTEXT.md)
**Depends on:** None (touches resolve.rs pipeline + form renderers — independent of v12.0 closure phases)
**Plans:** 8/8 plans complete

Plans:
- [x] 181-01-PLAN.md — Wave 0 RED-state tests: 2 new pipeline integration tests + upgrade 2 existing tests to `html_body` + `<p id="err-` assertion (D-07)
- [x] 181-02-PLAN.md — Fix A (merge runtime data before resolve in `JsonUi::render` + `JsonUi::render_with_errors_config`) AND Fix B (`attach_errors` writes singular `error: String`) + update 2 resolve.rs tests (D-02, D-03, D-04, D-08)
- [x] 181-03-PLAN.md — D-06 Checkbox parity: `border-destructive` + `focus-visible:ring-destructive` + ARIA on `<input>`, `id="err-{field}"` on error `<p>` + new unit test
- [x] 181-04-PLAN.md — D-06 CheckboxList parity: fieldset ARIA + per-option `border-destructive` + `id` on error `<p>` + new unit test
- [x] 181-05-PLAN.md — D-06 Switch parity: `peer-focus:ring-destructive/30` on pill + ARIA on hidden `<input>` + `id` on error `<p>` + new unit test
- [x] 181-06-PLAN.md — D-06 Input (file) parity: `ring-1 ring-destructive` + ARIA + new unit test
- [x] 181-07-PLAN.md — D-08 cross-repo gestiscilo audit (`rg` for plural `errors` reads) + manual UAT on 5 representative forms + full pre-commit gate
- [x] 181-08-PLAN.md — D-09 docs page `docs/src/json-ui/forms.md` covering the four authoring patterns (blessed / `$data` escape hatch / flash round-trip / cross-field summary) + SUMMARY.md navigation entry

Discovery: surfaced during gestiscilo Phase 175 UAT (2026-05-31) on the operator product-edit form. Backend validation logic ships correctly via `ValidationError::new().add(field, msg).with_old_input(&data).redirect_to(...)`, but the error string never reaches a visible DOM element — operator only sees the generic ferro URL-fallback flash `?error=generic&msg=…`. CONTEXT and RESEARCH (2026-05-31) revised the original framing: the renderer is already correct; the bug lives in the resolution pipeline. Full repro and root-cause analysis in `.planning/phases/181-json-ui-input-error-prop-inline-render/181-CONTEXT.md` and `181-RESEARCH.md`. Cross-tracked as gestiscilo Phase 176 [FERRO REPO].

---

### ✅ v12.2 Frontend Performance Hardening (Phases 182-184, shipped 2026-06-06)

Three runtime/framework primitives surfaced by the gestiscilo-it jetskiadriatic startup-lifecycle audit on 2026-06-06. Each phase pairs 1:1 with a gestiscilo v6.6.1 phase that consumes the published primitive via crates.io bump (mirrors the Phase 181 ↔ gestiscilo Phase 176 pattern). Build order recommendation: 182 → 183 → 184 (smallest to largest ferro-side scope; 182 acts as the pattern-rodage phase).

#### Phases

- [x] **Phase 182: `ferro-json-ui` `data-lazy-hero` runtime primitive** — Add an IntersectionObserver block to `ferro-json-ui/src/runtime.rs` (sibling of SSE/tabs/toasts/sidebar) promoting `<video preload="none">` → `preload="auto"` on viewport approach. Single observer per page fans out to all `[data-lazy-hero]` elements. Per-element `rootMargin` via `data-lazy-hero-margin="…"` attribute. Default `200px 0px`. Idempotent via `data-lazy-hero-promoted="1"` marker. Pure attribute-driven, zero per-page JS for consumers. **Paired with:** gestiscilo Phase 186 (SDK auto-wiring + adopt `data-lazy-hero`). (completed 2026-06-06)
- [x] **Phase 183: `ferro-bundle` capability (new crate)** — Top-level crate `ferro-bundle` for in-memory immutable byte blobs registered at boot. `Bundle::new(name, bytes).content_type(…).hashed_url()` → returns `/bundles/{name}.{sha8}.{ext}`-style URL; `Bundle::serve(req)` serves with `Cache-Control: public, max-age=31536000, immutable` + SHA-256 ETag + `If-None-Match` 304 fast path. `.with_alias("/embed/v1.js")` registers a plain-URL alias that 301-redirects to the current hash for backward compat. Targets symbolic byte blobs; does NOT replace the filesystem static-file handler (two parallel paths intentional — filesystem path is mutable, freshness via `bust_asset_urls`; bundle path is immutable, freshness via content hash). **Paired with:** gestiscilo Phase 185 (consume `ferro-bundle` + tenant asset minification + font subsetting). (completed 2026-06-06)
- [x] **Phase 184: `ferro::InlineBudget` + `ferro::RequestTelemetry`** — Two request-scoped primitives. (a) `InlineBudget`: request extension `req.inline_budget(key, bytes) -> Decision::{Inline, Preload(url)}`. Tracks cumulative inlined bytes per request, fires a structured warning + flips to `Preload` once a configurable threshold is crossed. Targets: HTML inline scripts/styles, JSON-LD blobs, critical-CSS. (b) `RequestTelemetry`: per-key ring buffer (last N samples, in-process). `req.telemetry_record(key, sample)` for writers; `RequestTelemetry::snapshot(key, scope) -> Vec<Sample>` for operator surfaces. Thread-safe, lost-on-restart documented. Crate location decision (extension trait in `ferro-core` vs new `ferro-telemetry` crate) locked during discuss. **Paired with:** gestiscilo Phase 187 (consume `InlineBudget` + `RequestTelemetry` + bootstrap-endpoint fallback). (completed 2026-06-06)

### Phase 182: `ferro-json-ui` `data-lazy-hero` runtime primitive

**Goal:** Extend `ferro-json-ui/src/runtime.rs` with an IntersectionObserver primitive that promotes `<video preload="none">` to `preload="auto"` (and calls `.load()` defensively) when the video crosses a configurable `rootMargin`. A single observer per page fans out to all `[data-lazy-hero]` elements, reading per-element `rootMargin` via `data-lazy-hero-margin="400px 0px"` (string parsed at observer setup). Default `200px 0px` — gives the network ~half a second before viewport entry. Idempotent via `data-lazy-hero-promoted="1"` marker. The `data-lazy-hero` attribute name and the override attribute name are part of the public ferro contract.

**Depends on:** None (single-file runtime extension; sibling primitives already exist for SSE/tabs/toasts/sidebar).

**Requirements:** TBD (locked during /gsd-discuss-phase 182)

**Success Criteria** (what must be TRUE):
  1. Loading any page with `<video preload="none" data-lazy-hero>` below the fold and scrolling causes the `preload` attribute to flip to `"auto"` exactly when the element crosses the configured `rootMargin` boundary (verified via Chrome DevTools Network panel showing video bytes only after scroll).
  2. Per-element override via `data-lazy-hero-margin="400px 0px"` is honored at observer setup.
  3. The promoted-marker (`data-lazy-hero-promoted="1"`) prevents double-promotion; re-running the observer on the same element is a no-op.
  4. The runtime IIFE size grows by at most ~400 bytes (single-observer fan-out, no per-element observer cost).
  5. `ferro-json-ui` publishes the new version to crates.io via the existing GH Actions workflow; gestiscilo Phase 186 consumes it via Cargo.toml bump.

**Plans:** 3/3 plans complete

Plans:
- [x] 182-01-PLAN.md — Create ferro-json-ui/src/runtime/hero_lazy.rs (setupLazyHeroes SOURCE) + wire into runtime/mod.rs (mod list, push_str chain, dispatcher, three test extensions/additions)
- [x] 182-02-PLAN.md — Create docs/src/json-ui/runtime-primitives.md (public DOM-attribute contract page) + register in docs/src/SUMMARY.md
- [x] 182-03-PLAN.md — Bump workspace.package.version 0.2.41 → 0.2.42 in Cargo.toml + sync Cargo.lock (triggers existing Wave1A publish workflow on master merge)

Discovery: surfaced during the 2026-06-06 jetskiadriatic startup-lifecycle audit. Tenant `index.html` has 4 below-the-fold heroes at `preload="none"`; the only way to lazily promote them today is per-page IntersectionObserver boilerplate. Pure generic web primitive — any ferro app with above-the-fold + below-the-fold hero videos benefits. Cross-tracked as gestiscilo Phase 186 [FERRO REPO]. Same elevation rule as Phase 165 F11/F13/F14 (runtime gaps belong in ferro, not in consumer-side scripts).

### Phase 183: `ferro-bundle` capability (new crate)

**Goal:** Ship a new top-level crate `ferro-bundle` for in-memory immutable byte blobs registered at boot. Public API: `Bundle::new(name: &str, bytes: &'static [u8]).content_type(ct).hashed_url() -> String` returns `/bundles/{name}.{sha8}.{ext}`-style URL; `Bundle::serve(req) -> HttpResponse` handles the request, returning the bytes with `Cache-Control: public, max-age=31536000, immutable`, `ETag: "{sha256}"`, and a 304 fast-path on `If-None-Match` match. `.with_alias("/embed/v1.js")` registers a plain-URL alias that 301-redirects to the current hashed URL for backward compat. Two parallel asset-serving paths intentional: filesystem static-file handler stays for mutable tenant assets (freshness via `bust_asset_urls` timestamp); `ferro-bundle` targets symbolic immutable blobs (freshness via content hash). The split is documented in the crate README so future contributors do not fold them.

**Depends on:** None (new crate, additive to the workspace).

**Requirements:** BUNDLE-01..BUNDLE-06 (informal IDs aligned 1:1 with the six Success Criteria below; `phase_req_ids: null` in REQUIREMENTS.md — Phase 183 is not enumerated in the v12.1 AI requirements doc)

**Success Criteria** (what must be TRUE):
  1. `Bundle::new("embed-v1", BYTES).content_type("application/javascript").hashed_url()` returns a string like `/bundles/embed-v1.{8hex}.js` deterministically derived from SHA-256 of `BYTES`.
  2. `Bundle::serve(req)` returns 200 with `Cache-Control: public, max-age=31536000, immutable` + `ETag` header on a cold request; returns 304 on `If-None-Match` exact match.
  3. `.with_alias("/embed/v1.js")` registers a plain-URL alias that returns 301 redirect to the current hashed URL.
  4. Content-type is caller-provided at registration (no filename sniffing); default `application/octet-stream` if unspecified.
  5. The crate README documents the bundle-vs-filesystem split (immutable byte blobs vs mutable filesystem assets) so future contributors do not collapse them.
  6. `ferro-bundle` publishes to crates.io via the existing GH Actions workflow; gestiscilo Phase 185 consumes it via Cargo.toml bump.

**Plans:** 4/4 plans complete

Plans:
- [x] 183-01-PLAN.md — Scaffold ferro-bundle crate (Cargo.toml + lib.rs stub + README) + workspace member + version bump 0.2.42 -> 0.2.43 + publish.yml Wave 3 entry (Shape B: appended alongside ferro-cli)
- [x] 183-02-PLAN.md — Core implementation: Bundle struct + 5 builder methods + Error enum + OnceLock<DashMap> registries + serve_inner dispatcher + unit tests (BUNDLE-01, BUNDLE-04)
- [x] 183-03-PLAN.md — Integration tests: serve_cold, serve_304, alias_redirect via __test_internals::serve_inner shim (BUNDLE-02 cold, BUNDLE-02 304, BUNDLE-03)
- [x] 183-04-PLAN.md — Publish bootstrap: cargo publish --dry-run gate + manual cargo publish from local terminal (D-12) + SUMMARY with runbook (BUNDLE-06)

Discovery: gestiscilo `/embed/v1.js` SDK bundle is forever-stable per the SDK-10 contract but served today with `max-age=300, stale-while-revalidate=86400` (adequate but not optimal). A content-hashed URL unlocks truly immutable caching with one-year `max-age`. Generic enough to live in ferro: any ferro app shipping versioned static asset bundles can reuse the same primitive. Cross-tracked as gestiscilo Phase 185 [FERRO REPO].

### Phase 184: `ferro::InlineBudget` + `ferro::RequestTelemetry`

**Goal:** Ship two request-scoped framework primitives. (a) **`InlineBudget`** — request extension `req.inline_budget(key, bytes) -> Decision::{Inline, Preload(url)}`. Tracks cumulative inlined bytes per request keyed by `key` (e.g. `"products_payload"`, `"jsonld_blob"`, `"critical_css"`); compares against a configurable threshold; returns `Decision::Inline` when below, `Decision::Preload(url)` once crossed (caller-provided fallback URL for the `<link rel=preload>`). Fires a structured warning the first time threshold is crossed per request. (b) **`RequestTelemetry`** — per-key in-process ring buffer. `req.telemetry_record(key, sample)` for writers; `RequestTelemetry::snapshot(key, scope) -> Vec<Sample>` for operator surfaces. Thread-safe, lost-on-restart semantics explicitly documented. Crate location decision (extension trait in `ferro-core` vs new `ferro-telemetry` crate) is owned by this phase's discuss — NOT pre-committed here. Both primitives are generic — gestiscilo Phase 187 is the first consumer; JSON-LD inlining, critical-CSS inlining, render-latency telemetry, cache-hit-rate telemetry are all future consumers.

**Depends on:** None (new request extensions, additive to the request lifecycle).

**Requirements:** TBD (locked during /gsd-discuss-phase 184)

**Success Criteria** (what must be TRUE):
  1. `req.inline_budget(key, bytes)` returns `Decision::Inline` when cumulative bytes for `key` remain below the configured threshold within the same request; returns `Decision::Preload(url)` once crossed.
  2. The structured warning fires exactly once per `key` per request when the threshold is crossed (no warning spam on subsequent inline_budget calls past the threshold).
  3. `req.telemetry_record(key, sample)` and `RequestTelemetry::snapshot(key, scope)` round-trip Sample data correctly; concurrent reads + writes are thread-safe; the buffer keeps at most N samples per (key, scope) and drops oldest on overflow.
  4. The crate location decision (ferro-core extension vs new ferro-telemetry crate) is recorded in the phase's CONTEXT.md with rationale.
  5. ferro publishes the new version to crates.io via GH Actions; gestiscilo Phase 187 consumes both primitives via Cargo.toml bump.

**Plans:** 3/3 plans complete

Plans:
- [x] 184-01-PLAN.md — Foundation: telemetry module (Sample + RequestTelemetry + global store + Decision enum + InlineBudgetState) + AppConfig.inline_budget_threshold_bytes field + crate-root re-exports (Decision, RequestTelemetry, Sample — NOT InlineBudget per OQ2)
- [x] 184-02-PLAN.md — Request integration: decide() body with state machine + fire-once tracing::warn! + Request methods (inline_budget, telemetry_record, telemetry_record_scoped) added to the second impl block
- [x] 184-03-PLAN.md — Integration test (tests/telemetry_smoke.rs) + docs page (docs/src/the-basics/inline-budget-and-telemetry.md) + SUMMARY.md entry + workspace.package.version bump 0.2.43 → 0.2.44 + cargo publish --dry-run gate

Discovery: surfaced during the 2026-06-06 jetskiadriatic startup-lifecycle audit. gestiscilo `inject_config_and_products` unconditionally inlines up to 100 products into every HTML response — fat tenants can blow past 200 KB, paid as HTML-parse cost on every page load. The right primitive (decide inline vs preload based on measured bytes) is request-scoped + framework-level, not gestiscilo-specific. Same elevation rule as `feedback_ferro_first_primitives.md`: cross-cutting capabilities go in ferro by default rather than waiting for N consumers. Cross-tracked as gestiscilo Phase 187 [FERRO REPO].

---

### 🔭 Future UI Spec Evaluation (Phase 174, planned 2026-05-17)

Forward-looking exploration of alternative server-driven UI protocols. May seed a downstream prototype milestone, or terminate as a documented decision to stay on the JSON spec.

#### Phases

- [ ] **Phase 174: Explore Hyperview / HXML as a candidate next-generation UI spec format** — Research-only. Evaluate the [Hyperview](https://hyperview.org/) HXML protocol against the current JSON spec across protocol shape, component model, expression/visibility primitives, plugin and extension story, projection/intent composition, JSON-Schema introspection equivalents for agent authoring, and server-side rendering pipeline reuse. Includes an explicit Appo angle: today Appo wraps the Ferro web frontend in a native shell; an HXML-style protocol would let the same Ferro server drive a fully native iOS/Android UI without the WebView layer. Output is a decision-quality `HXML-RESEARCH.md` covering protocol comparison, what HXML does better, what's worse or unclear, the Appo angle, and a recommendation (stay on the JSON spec / migrate / build HXML as a parallel renderer). Ends with a go/no-go gate for any downstream prototype phase. No code changes.

### Phase 174: Explore Hyperview / HXML as a candidate next-generation UI spec format

**Goal**: Produce a research-only evaluation of the Hyperview HXML protocol as a candidate next-generation UI spec format for ferro, including the strategic angle for Appo (native mobile UI without a WebView). Decision-quality output document; no code changes.

**Depends on**: Phase 161 (v12.0 merge to master — finalizes the current JSON spec surface to compare against)

**Requirements**: TBD (research phase — requirements may be derived from the output document)

**Success Criteria** (what must be TRUE):
  1. `HXML-RESEARCH.md` exists in the phase directory with the following sections populated: Protocol comparison (HXML vs current JSON spec), What HXML does better, What's worse or unclear, Appo angle (capabilities gained, capabilities lost, integration shape), Recommendation (stay / migrate / parallel renderer)
  2. The recommendation is justified against ferro's design principles (projection/intent as core abstraction, beauty dimensions, agent-readable surface)
  3. The Appo angle explicitly inventories what changes for the WebView → native transition: which `usePush` / `useCamera` / `useBiometrics` / etc. hooks still apply, which become redundant, and which new primitives ferro would need to emit
  4. The go/no-go gate either schedules a downstream prototype phase (with explicit scope) or records a decision to stay on the JSON spec (with rationale)
  5. Plugin model parity is addressed — can HXML host arbitrary native widgets the way ferro-json-ui hosts Plugin components? If not, what's the equivalent?

**Plans**: TBD (run /gsd-plan-phase 174 to break down)

### Phase 176: v12.0.2 JSON-UI v2 Runtime Patches — Booking↔Staff Binding Field Test (F7–F9)

**Goal**: Close three runtime gaps in ferro-json-ui v2 surfaced by the gestiscilo-it β booking↔staff binding UAT (consumer phase 152). All three findings have server specs that emit correctly today — the renderer silently drops props/conditionals it should respect.

- **F7 — `Card.badge` prop silently dropped.** Server emits `Card { props: { title, description, badge: "Scade tra Nm" } }`; the rendered DOM has `<h3>title</h3><p>description</p>` only — no `badge` slot. Consumer use case: countdown badges on kanban cards.
- **F8 — `Card.subtitle` prop silently dropped.** Server emits `Card { props: { title, description, subtitle: "Marco Rossi" } }`; the rendered DOM has no `subtitle` slot. Consumer use case: secondary identifier (staff name snapshot) beneath the customer name on booking cards.
- **F9 — `Grid.visible` conditional drops entire subtree.** Server emits `Grid { children: [...], visible: { path: "/has_staff", operator: "eq", value: true } }` with `data.has_staff: true`. Rendered DOM has no Grid element at all. Either Grid's renderer does not parse `visible`, or it evaluates the predicate against the wrong scope. Consumer use case: per-staff filter chip strip hidden when the tenant has no staff configured.

**Source:** Consumer chrome-mcp field test 2026-05-20, documented at `.planning/phases/152-booking-staff-binding/152-UI-FINDINGS.md` in the gestiscilo-it repo (Bugs R2/R3/R4).

**Decision boundary (matches Phase 175):** F7+F8 both extend the `Card` component template — planner judges whether to ship as one combined plan or split. F9 is a Grid-renderer change and ships independently.

**Depends on**: Phase 175 shipped (v12.0.1 batch 4) — F7/F8/F9 are layered on the same Card/Grid templates Phase 175 touched.

**Requirements**: TBD (derive from per-finding plans)

**Success Criteria** (what must be TRUE):
  1. A v2 spec declaring `Card { props: { title: "T", badge: "B" } }` renders DOM containing both the title text "T" and a badge element with text "B"; the badge is visually distinguished (Badge component-styled, right-aligned or per Card layout convention).
  2. A v2 spec declaring `Card { props: { title: "T", subtitle: "S" } }` renders DOM containing both the title text "T" and a subtitle element with text "S" beneath the title (muted-text class).
  3. A v2 spec declaring `Grid { children: [...], visible: { path: "/has_staff", operator: "eq", value: true } }` with `data.has_staff = true` renders the Grid + all children. The same spec with `data.has_staff = false` renders no Grid element. Same predicate semantics as other components' `visible` clause (audit which v2 components currently support `visible` and document the union — Grid joining if absent, or fixing the evaluator scope if Grid is supposed to support it).
  4. Catalog JSON schema is updated for F7+F8: `Card.props` accepts optional `badge: String` and `subtitle: String`. Doctests + component tests added.
  5. v2 component docs updated to reflect the new Card slots and to clarify `Grid.visible` behavior.
  6. `cargo test --all-features` passes; gestiscilo-it consumer re-runs its β UAT against the patched runtime and confirms F7/F8/F9 closed (chrome-mcp snapshot showing `badge`/`subtitle` rendered and chip strip visible).

**Plans**: TBD (run /gsd-plan-phase 176 to break down)

### Phase 177: ferro-reservation Kernel Atomicity Hardening — `hold` race fix

**Goal**: Close the `ReservationKernel::hold` check-then-act race condition that allows two concurrent `tokio::spawn` tasks racing identical `(resource_kind, resource_key, window)` to both succeed when at most `capacity` should. Fix the kernel so the `held ≤ capacity` invariant holds under concurrent INSERTs.

**Source:** Consumer field test 2026-05-20 — gestiscilo-it v6.9 β killer-feature acceptance test `concurrent_double_book_same_staff` fails 5/5 deterministically (~0.07s each, not a timing artifact). Two tokio tasks racing `StaffBookingService::reserve_for_booking` on the same `(tenant_id, staff_id, window)` both produce Ok handles when exactly one should produce `Err(Insufficient)`. Documented at `.planning/phases/152-booking-staff-binding/152-UI-FINDINGS.md` (Bug R5) in the gestiscilo-it repo.

**Root cause (verified by reading ferro-reservation/src/kernel.rs:54-122):** the `hold` method does a check-then-act sequence with no transaction and no unique constraint:
```rust
// Steps 2–3: capacity check (consumer-defined)
let capacity = self.resource.capacity(conn, &key, &window).await?;
let held = self.resource.held(conn, &key, &window).await?;
let available = capacity.saturating_sub(held);
// Step 4: enforce invariant
if quantity > available { return Err(Insufficient {...}); }
// Step 5: INSERT reservations row  <-- nothing prevents two concurrent INSERTs
```
Both tokio tasks read `held = 0`, both pass `available = capacity - 0 ≥ quantity`, both INSERT a `status='held'` row. `GuardedUpdate` is used in `commit/release/sweeper` (UPDATEs) but never in `hold` (the INSERT path).

**Depends on**: None (independent kernel-internal fix). `ferro-orm::GuardedUpdate` already in workspace; `sea_orm::TransactionTrait` already a dependency. No new external deps.

**Requirements**: TBD (derive from plan-time fix-path selection)

**Success Criteria** (what must be TRUE):
  1. A new integration test in ferro-reservation/tests/ races two `tokio::spawn` tasks calling `kernel.hold(...)` on identical `(key, window)` with `quantity = capacity`; exactly one returns `Ok(ReservationHandle)` and exactly one returns `Err(ReservationError::Insufficient)`. Test passes 50/50 runs in CI (zero flakiness).
  2. Boundary-touch behavior preserved: two `hold(...)` calls on the same `(key, ...)` with non-overlapping windows BOTH succeed. The atomicity fix MUST NOT introduce false positives that reject legitimate non-overlapping holds.
  3. The existing inventory test suite at gestiscilo-it (Phase 130/131/132) passes unchanged against the patched ferro-reservation. Behaviour byte-identical for the single-writer case.
  4. The fix path is one of: (a) wrap `hold` body in `conn.begin()` + commit with serializable isolation (SQLite native, Postgres `SERIALIZABLE`); (b) add a unique partial index on `reservations (resource_kind, resource_key, window_hash) WHERE status='held'` with deterministic JSON-canonical window hashing; (c) `INSERT … SELECT … WHERE NOT EXISTS` portable atomic check-and-insert. Planner picks at plan time; default is (a) for minimum blast radius.
  5. Audit log entry semantics unchanged — `reservation.held` audit row still written exactly once per successful hold, never written for the conflict-losing task.
  6. PITFALLS T-69-1.2 documentation in consumer field tests is now factually correct (kernel arbitrates concurrent holds).

**Plans:** 3/3 plans complete
- [x] 177-01-PLAN.md — Kernel atomicity fix + SQLite primary tests (SC-1, SC-2, SC-3, SC-4, SC-5)
- [x] 177-02-PLAN.md — Postgres feature scaffolding + cfg-gated mirror test (SC-1 Postgres facet)
- [x] 177-03-PLAN.md — Documentation correction sweep in docs/src/database/reservations.md (SC-6)

### Phase 179: DataTable RawHtml-free heterogeneous rows — Badge column format + per-row visible_if on row_actions

**Goal:** Two additive ferro-json-ui primitives that let a DataTable express per-row varying actions and typed status pills without forcing callers to emit raw HTML strings into cell values (which would then be escaped, currently a silent UX bug). After this phase: (a) `ColumnFormat::Badge` reads `{variant, label}` per cell and emits the same `<span>` shape as the existing Badge component; (b) `DropdownMenuAction.visible_if: Option<String>` gates an action item per row based on a boolean row field, so a single table-level `row_actions` declaration can serve heterogeneous row states.

**Killer feature:** A DataTable can render a status pill column AND a kebab dropdown whose item set varies per row, with the controller emitting structured data only — no HTML strings, no per-row HTML builders, no XSS surface.

**Motivation:** Gestiscilo Phase 172 (unified Documenti tab) tried to render heterogeneous per-row kebab actions by emitting HTML strings into a cell and assuming a RawHtml cell variant existed. It doesn't — `render_data_table` html-escapes every cell unconditionally (tested at `ferro-json-ui/src/render/data.rs:687-704`). Result: the table renders escaped literal HTML instead of badges/buttons. Adding `RawHtml` would solve it but introduces an XSS surface that callers can misuse. The typed `Badge` column format + per-row `visible_if` solves the same problem without the escape hatch.

**Requirements:** [internal — design decisions in PLAN]
**Plans:** 1 plan

Plans:
- [x] 179-01-PLAN.md — Badge column format + DropdownMenuAction.visible_if + tests + version bump (shipped 2026-05-25 in workspace v0.2.38)

---

## v12.3 Deployment Platform Primitives (Phases 185–188)

**Source:** gestiscilo-it v7.1 Tenant Frontend Platform — locked design at gestiscilo `.planning/research/v7.1-ARCHITECTURE.md` (D-01..D-06) + research at `.planning/research/v7.1-{STACK,INTEGRATION,PITFALLS}.md`. Consumer pairing: ferro 185+186 ↔ gestiscilo Phase 188, ferro 187 ↔ gestiscilo Phase 189, ferro 188 ↔ gestiscilo Phase 190. Each crate auto-publishes via GH Actions on push to master; the gestiscilo consumer phase closes with an atomic `[patch.crates-io]` revert + version bump.

**Design constraint (architecture principle):** every primitive stays consumer-agnostic. A deployment is "a versioned, addressable bundle of artifacts with an atomic active pointer" — static HTML sites, compiled JSON-UI spec bundles, and Inertia-style SSR manifests all fit. No gestiscilo-specific assumptions in any crate.

**Requirements:**

- **QUEUE-F-01**: A consumer app can define a background job by implementing `ferro::queue::Job` and have it claimed, executed, retried with exponential backoff, and parked after max retries
- **QUEUE-F-02**: Job claim is atomic on Postgres (`FOR UPDATE SKIP LOCKED`) and SQLite (`BEGIN IMMEDIATE` + `UPDATE … RETURNING`) — two concurrent workers never execute the same job
- **QUEUE-F-03**: `WorkerLoop` runs inside `ferro serve` (work-stealing single binary, D-01); a crashed/killed worker's claimed jobs are reaped and retried
- **QUEUE-F-04**: `jobs` table migration helper provided; migration portable across SQLite + Postgres
- **DEPL-F-01**: `Deployment` model records immutable rows (identifier, source ref, artifact location, byte size, status, timestamps) with a portable migration helper
- **DEPL-F-02**: `promote(owner_key, deployment_id)` is a single atomic UPDATE of the active pointer; `rollback` is promoting a previous deployment
- **DEPL-F-03**: `DeploymentStorage` trait abstracts artifact persistence (S3-compatible default via ferro-storage); `preview_url(deployment_id)` subdomain helper present (consumers may defer wiring it)
- **ASSET-F-01**: `Pipeline` composes ordered transforms; each transform declares accepted content types and passes everything else through unchanged
- **ASSET-F-02**: `html_minify` (lol_html), `css_minify` (lightningcss `=1.0.0-alpha.71`), `js_minify` (swc_ecma_minifier) ship as built-in transforms; inline `<script>`/`<style>` content survives byte-correct (no template corruption)
- **ASSET-F-03**: `image_transcode` emits AVIF + JPEG responsive variants via pure-Rust `image`+`ravif` (no C system deps), with bounded concurrency (semaphore) so encoding cannot OOM a small instance
- **ASSET-F-04**: `inject_before_tag` generic HTML injection transform (SDK script tags, token substitution) ships as a built-in
- **STOR-F-01**: `Storage::cdn_url(path)` returns the full CDN URL for a stored object
- **STOR-F-02**: `PurgeApi` trait with default DO Spaces CDN adapter (`DELETE /v2/cdn/endpoints/{id}/cache`, batches ≤50 files/request, respects 5 req/10s rate limit, wildcard purge supported); Bunny + Cloudflare adapters feature-flagged

**Build order:** 185 → 186 → 187 → 188 is the natural publish order (gestiscilo consumes them in that sequence), but 186/187/188 have no inter-dependency and can run in parallel once 185's `jobs` infrastructure exists. Estimated effort per the design doc: 185 ≈ 2w, 186 ≈ 1-2w, 187 ≈ 2-3w, 188 ≈ 1w.

## Phases

- [x] **Phase 185: ferro::queue — DB-Backed Job Queue** — `Job` trait + `WorkerLoop` in `ferro serve` + atomic claim (Postgres/SQLite) + retry/backoff + reaper; replaces the Redis-only ferro-queue backend (completed 2026-06-07)
- [x] **Phase 186: ferro-deployments — Immutable Deployments + Atomic Promote** — new crate: `Deployment` model, `DeploymentStorage` trait, `promote`/`rollback`, `preview_url` helper (completed 2026-06-07)
- [x] **Phase 187: ferro-assets — Asset Pipeline Composer** — new crate: content-type-aware `Pipeline` with HTML/CSS/JS minify, pure-Rust image transcode, generic injection — 4 plans planned (completed 2026-06-07)
- [x] **Phase 188: ferro-storage CDN Extension** — `cdn_url()` + `PurgeApi` trait + DO Spaces CDN adapter, feature-flagged Bunny/Cloudflare — 3 plans planned (completed 2026-06-07)

#### Phase Details

### Phase 185: ferro::queue — DB-Backed Job Queue
**Goal**: Replace the Redis-only ferro-queue backend with a DB-backed queue living in the framework crate: consumers implement `Job`, `ferro serve` runs the `WorkerLoop` in-process (work-stealing across identical instances per gestiscilo D-01), and the claim path is atomic on both production Postgres and dev SQLite.
**Depends on**: Nothing (first phase of milestone)
**Requirements**: QUEUE-F-01, QUEUE-F-02, QUEUE-F-03, QUEUE-F-04
**Success Criteria** (what must be TRUE):
  1. A race test with two concurrent `WorkerLoop`s against the same `jobs` table claims each job exactly once — verified on SQLite (`BEGIN IMMEDIATE` + `UPDATE … RETURNING`) and, behind a cfg-gated test, on Postgres (`FOR UPDATE SKIP LOCKED`); no raw `FOR UPDATE SKIP LOCKED` SQL in any migration file (claim SQL branches on live backend at runtime)
  2. A job whose worker dies mid-execution is reaped after a visibility timeout and retried; a job failing `max_retries` times is parked as `failed` with its error recorded and never blocks subsequent claims (poison-job isolation)
  3. Retry delay follows exponential backoff with jitter; `Job` trait exposes `max_retries()` and an idempotency-key hook
  4. `WorkerLoop` starts inside `ferro serve` with no separate process; CPU-heavy job bodies documented to use `tokio::task::spawn_blocking`; graceful shutdown re-queues claimed-but-incomplete jobs
  5. The existing `Job`/`Queueable` public API surface is preserved where possible; any breaking change is documented with a migration table (consumer: gestiscilo Phase 188 migrates 4 job types against it); Redis dependency droppable by consumers after migration
**Plans**: 5 plans (5 sequential waves — one-CPU-op-at-a-time constraint)
- [x] 185-01-PLAN.md — Foundation: drop redis, add deps, Job idempotency_key + jittered retry_delay, QueueConfig refactor, CreateJobsTable migration
- [x] 185-02-PLAN.md — DB engine: dual-backend atomic claim, reaper, idempotent enqueue, lifecycle ops, stat queries, Queue global
- [x] 185-03-PLAN.md — WorkerLoop (panic isolation, SIGTERM drain+requeue) + DB-backed dispatcher; delete queue.rs; lib.rs re-exports
- [x] 185-04-PLAN.md — Framework: ferro::queue namespaced module, WorkerLoop auto-start, debug endpoints over DB, ferro-mcp job_history fix
- [x] 185-05-PLAN.md — Proof artifacts: SQLite + Postgres race tests, shutdown test, docs rewrite, full-suite gate

### Phase 186: ferro-deployments — Immutable Deployments + Atomic Promote
**Goal**: New crate providing the deployment abstraction: every publish is an immutable, addressable row; going live is one atomic pointer flip; rollback is promoting an older row. Artifact shape is opaque — static HTML, JSON-UI bundles, and SSR manifests all fit.
**Depends on**: Phase 185 (jobs table conventions; not a hard compile dependency)
**Requirements**: DEPL-F-01, DEPL-F-02, DEPL-F-03
**Success Criteria** (what must be TRUE):
  1. `deployments` migration helper creates a portable schema (SQLite + Postgres) recording identifier, source ref (e.g. git SHA), artifact location, byte size, status (`building`/`ready`/`failed`), timestamps; rows are never mutated after reaching a terminal status
  2. `promote(owner_key, deployment_id)` executes a single atomic UPDATE of the active pointer and returns the previously-active deployment id; a race test shows two concurrent promotes serialize correctly (last-write-wins, no torn state)
  3. `rollback` is implemented as promote-of-previous; promoting a deployment whose status is not `ready` is rejected
  4. `DeploymentStorage` trait abstracts artifact persistence with an S3-compatible default implementation delegating to ferro-storage; `preview_url(deployment_id)` returns the wildcard-subdomain URL form (consumers may leave it unwired)
  5. Crate contains zero HTML/gestiscilo-specific assumptions — a doc-test or example stores a non-HTML artifact bundle (e.g. JSON specs) through the same API
**Plans**: 4 plans
- [x] 186-01-PLAN.md — Crate skeleton + workspace/publish.yml registration + error/config + portable migration helpers (Wave 1)
- [x] 186-02-PLAN.md — Deployments handle lifecycle + atomic promote/rollback + concurrent-promote race tests (Wave 2)
- [x] 186-03-PLAN.md — DeploymentStorage trait (S3-compatible default) + preview_url subdomain helper (Wave 2)
- [x] 186-04-PLAN.md — Criterion-5 JSON-artifact doc-test + docs page + version bump 0.2.45 + publish dry-run (Wave 3)

### Phase 187: ferro-assets — Asset Pipeline Composer
**Goal**: New crate providing a composable, content-type-aware asset pipeline for publish-time optimization: HTML/CSS/JS minification, pure-Rust image transcoding with responsive variants, and generic tag injection — the Tier 1 pipeline gestiscilo's `PublishFrontendJob` composes.
**Depends on**: Nothing (leaf crate; parallel-capable with 186)
**Requirements**: ASSET-F-01, ASSET-F-02, ASSET-F-03, ASSET-F-04
**Success Criteria** (what must be TRUE):
  1. `Pipeline::new().add(transform)...run(files)` applies transforms in order; each transform declares accepted content types and files outside its types pass through byte-identical (a JSON file run through the full HTML/CSS/JS/image pipeline is untouched)
  2. `html_minify` (lol_html), `css_minify` (lightningcss pinned `=1.0.0-alpha.71`), `js_minify` (swc_ecma_minifier) ship as built-ins; a fixture with inline `<script>` templating and inline `<style>` survives minification with semantics intact (regression fixtures from a real tenant site)
  3. `image_transcode` emits AVIF (`ravif`) + JPEG fallback at configurable responsive widths using pure-Rust codecs — `cargo build` introduces no new C system dependencies; concurrent encodes bounded by a semaphore (default ≤2) so peak memory stays bounded on a 512MB instance
  4. `responsive_images` lol_html rewriter transforms `<img>` → `<picture><source srcset=…>` referencing the emitted variants; `inject_before_tag(tag, html)` covers SDK script injection and is string-substitution safe (used for `%%TOKEN%%`-style replacement)
  5. Pipeline failure at any stage returns a structured per-file error and produces NO partial output set — the caller can implement all-or-nothing upload (gestiscilo PUB-05 two-phase invariant builds on this)
**Plans**: 4 plans (4 waves, serialized for the one-CPU-op-at-a-time constraint)

- [x] 187-01-PLAN.md — Crate scaffold + Wave 0 swc-version verification + Asset/ContentType/Error model + Transform/Pipeline (all-or-nothing) + passthrough (SC-1) & atomicity (SC-5) tests (Wave 1)
- [x] 187-02-PLAN.md — Text transforms: html_minify (opaque script/style, SC-2 fixture), css_minify, js_minify, inject_before_tag, replace_tokens (Wave 2)
- [x] 187-03-PLAN.md — Image transforms: image_transcode (image+ravif+rayon, AVIF+JPEG, no-upscale, bounded ≤2) + responsive_images (img→picture), SC-3 test (Wave 3)
- [x] 187-04-PLAN.md — README + docs/src feature page + SUMMARY link + full real-transform passthrough proof + CI-parity gate + manual first-publish checkpoint (Wave 4)

### Phase 188: ferro-storage CDN Extension
**Goal**: Extend ferro-storage with CDN awareness: full CDN URLs for stored objects and a cache-purge abstraction with a DigitalOcean Spaces CDN default adapter, so promote-then-purge becomes a two-call sequence for any consumer.
**Depends on**: Nothing (extension to existing crate; parallel-capable)
**Requirements**: STOR-F-01, STOR-F-02
**Success Criteria** (what must be TRUE):
  1. `Storage::cdn_url(path)` returns the CDN edge URL for a stored object, configured via env (CDN base URL); falls back to the origin URL when no CDN is configured
  2. `PurgeApi` trait exposes `purge(paths: &[String])`; the DO Spaces adapter calls `DELETE /v2/cdn/endpoints/{id}/cache`, batches requests at ≤50 files each, honors the 5 req/10s rate limit (internal throttle, not caller burden), and supports wildcard paths (1 wildcard = 1 file slot)
  3. DO adapter config reads `DO_SPACES_CDN_ID` + API token from env; a missing CDN id makes `purge` a logged no-op (consumers without CDN keep working)
  4. Bunny and Cloudflare adapters compile behind cargo features without entering the default dependency graph
**Plans**: 3 plans

- [x] 188-01-PLAN.md — cdn_url() field/builder/method + AWS_CDN_URL env + Error::Cdn + reqwest/tokio-time/cdn-bunny/cdn-cloudflare Cargo scaffolding (Wave 1)
- [x] 188-02-PLAN.md — PurgeApi trait + DoSpacesCdn adapter (DELETE/204, <=50 batch, 5 req/10s throttle, wildcard, missing-id no-op, token-redacted Debug) + wiremock tests (Wave 2)
- [x] 188-03-PLAN.md — Bunny + Cloudflare feature-gated adapters + default-graph absence proof + docs CDN section + version bump 0.2.45->0.2.46 + full --all-features CI gate (Wave 3)

## v11.6.1 ferro-stripe Manual Capture (Phase 189)

**Source:** gestiscilo-it v6.3-extended booking fund-hold field test. Extends the v11.6 capability-axis crate with Stripe manual capture so consumer apps can authorize card funds without charging (booking deposits). Consumer: gestiscilo-it v6.3 Online Checkout & Payments (queued after its v7.1), consumes via published crates.io bump per the Phase 176 ↔ ferro Phase 181 pattern.

**Design note:** the authorize/capture/cancel triple deliberately mirrors `ferro-reservation` hold/commit/release semantics — the correspondence is documented, not coupled (no compile dependency between the crates).

**Out of scope:** SetupIntent save-card flow for authorizations beyond the ~7-day card window (consumer-side design decision at gestiscilo v6.3 plan time; promote to a ferro phase only if gestiscilo picks that path).

**Requirements:**

- **STRIPE-MC-01**: `CheckoutBuilder::manual_capture()` sets `payment_intent_data.capture_method = manual` on the created Checkout Session, in `mode=payment` only
- **STRIPE-MC-02**: New `payment_intent.rs` capability module exposes `capture(payment_intent_id, amount_cents: Option<i64>)` (partial capture supported via `Some(n)`) and `cancel(payment_intent_id)`
- **STRIPE-MC-03**: New typed events `StripePaymentIntentAmountCapturableUpdated` and `StripePaymentIntentCanceled` registered in the webhook parser contract with golden-JSON fixtures
- **STRIPE-MC-04**: Manual capture composes with `destination()` Connect charges — authorize on platform, capture transfers to connected account
- **STRIPE-MC-05**: `docs/src/features/stripe.md` documents the authorize/capture/cancel ↔ `ferro-reservation` hold/commit/release correspondence

## Phases

- [x] **Phase 189: ferro-stripe Manual Capture** — `CheckoutBuilder::manual_capture()` + `payment_intent.rs` capture/cancel module + typed PaymentIntent webhook events with golden-JSON fixtures + Connect `destination()` composition (completed 2026-06-07)

#### Phase Details

### Phase 189: ferro-stripe Manual Capture
**Goal**: A consumer app can authorize card funds at checkout without charging (booking deposit hold), then later capture some-or-all of the authorized amount or release the hold — with typed webhook events covering the authorization lifecycle and full composition with Connect destination charges.
**Depends on**: Nothing (extends existing ferro-stripe capability axis; independent of v12.3 phases)
**Requirements**: STRIPE-MC-01, STRIPE-MC-02, STRIPE-MC-03, STRIPE-MC-04, STRIPE-MC-05
**Success Criteria** (what must be TRUE):
  1. `CheckoutBuilder::new(Mode::Payment).manual_capture()` produces a Checkout Session whose PaymentIntent has `capture_method = manual`; calling it in a non-payment mode is rejected at build time or returns a structured error
  2. `payment_intent::capture(id, None)` captures the full authorized amount; `capture(id, Some(n))` performs a partial capture of `n` cents; `payment_intent::cancel(id)` releases the hold — all three return structured `Error` values on invalid ids and Stripe API failures (same error contract as `refund.rs`)
  3. `StripePaymentIntentAmountCapturableUpdated` and `StripePaymentIntentCanceled` implement the `StripeEvent` trait, are parsed from golden-JSON webhook fixtures in tests, and unknown/other event types continue to pass through unmatched
  4. A checkout built with both `manual_capture()` and `destination(account_id, fee)` authorizes on the platform account and, on capture, transfers to the connected account per the destination-charge pattern (covered by a builder-level test of the generated params; live-mode verification owned by the consumer field test)
  5. `docs/src/features/stripe.md` documents manual capture end-to-end and the hold/commit/release ↔ authorize/capture/cancel correspondence with `ferro-reservation`
**Plans**: 4 plans

- [x] 189-01-PLAN.md — CheckoutBuilder::manual_capture() flag + mode guard + merged payment_intent_data (Connect composition) (Wave 1)
- [x] 189-02-PLAN.md — payment_intent.rs capability module: capture/cancel/retrieve + lib.rs registration (Wave 2)
- [x] 189-03-PLAN.md — Two typed PaymentIntent webhook events + golden-JSON fixtures + parser-contract tests (Wave 3)
- [x] 189-04-PLAN.md — docs/src/features/stripe.md Manual Capture section + ferro-reservation correspondence (Wave 4)

---

## v12.4 Form Validation DX (Phases 190-192)

**Source:** gestiscilo-it field test — slug-uniqueness violations surfacing as raw SQL errors through the `From<sea_orm::DbErr> for ActionError` passthrough.

**Milestone Goal:** Make uniqueness validation a first-class, ergonomic part of ferro forms — both proactively (async DB-backed `unique` rule that runs before the write) and defensively (DB constraint violations mapped to field-level errors instead of leaking raw SQL to end users). The killer feature: a uniqueness violation that today surfaces as a raw SQL error lands inline under the right field with user input preserved — uniqueness "just works" before the write (async rule, UX) and as a safety net at the write (constraint mapping, concurrency invariant).

**Key constraints encoded in this roadmap:**
- The sync `Validator` / `Rule` API is unchanged — async is a parallel path, not a replacement.
- Exclude-self (`.ignore(id)`) ships in Phase 190 (not retrofitted); retrofitting is a breaking change for edit handlers.
- All implementation contained in `framework/src/validation/` except the Phase 192 ferro-mcp template.
- Project-agnostic-crates rule: no consumer constraint/field strings in `framework` or `ferro-*` crates. All mapping registered at consumer call sites.
- Phase 191 Postgres constraint-name path cannot be exercised by `cargo test` defaults (SQLite-only CI). Closure criteria include a documented manual verification gate.

**Requirements:** VALID-01, VALID-02, VALID-03, VALID-04, VALID-05, VALID-06

## Phases

- [x] **Phase 190: Async Rule Infrastructure + `unique` Rule** — `AsyncRule` trait, `Unique` struct with `.ignore()` exclude-self, `AsyncValidator` / `validate_async`, ferro-lang translation key (completed 2026-06-09)
- [x] **Phase 191: ConstraintMap + Portable UNIQUE-Violation Detection** — `ConstraintMap` builder, SQLite/Postgres bifurcated detection, `try_map` falls through unchanged to `From<DbErr>` (completed 2026-06-09)
- [x] **Phase 192: ferro-mcp Template + Validation Docs** — `action_handler` code template updated with both layers, validation docs page extended (completed 2026-06-09)

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 190. Async Rule Infrastructure | 4/4 | Complete    | 2026-06-09 |
| 191. ConstraintMap + Detection | 2/2 | Complete    | 2026-06-09 |
| 192. MCP Template + Docs | 2/2 | Complete    | 2026-06-09 |

## Phase Details

### Phase 190: Async Rule Infrastructure + `unique` Rule
**Goal**: Developers can validate field uniqueness against the DB before insert/update, with exclude-self for edit forms, through a new async validation path that leaves the existing sync API untouched.
**Depends on**: Nothing (first phase of milestone; all components are new files in `framework/src/validation/`)
**Requirements**: VALID-01, VALID-02, VALID-03
**Success Criteria** (what must be TRUE):
  1. A handler can call `AsyncValidator::new().async_rule("slug", unique("pages", "slug"))` and `validate_async(&req).await` surfaces a field-level error under `"slug"` when the value already exists in the table — not a raw SQL error, not a server 500
  2. An edit handler calling `.ignore(record_id)` (exclude-self) does not reject a record whose slug is unchanged from its own current value in the DB — the regression that every copy-pasted create handler without exclude-self would trigger
  3. `validate_async()` runs sync rules first and skips async rules on fields that already have sync errors (fail-fast before hitting the DB)
  4. The existing `Validator` / `validate()` sync API compiles and behaves identically with no changes to its call sites
  5. `DB::connection()` is the access pattern inside `Unique` — no DB connection threaded through the rule signature or `validate_async()`
**Plans:** 4/4 plans complete

Plans:
- [x] 190-01-PLAN.md — AsyncRule trait (#[async_trait], dyn-compatible) + Wave 0 in-memory SQLite fixture (Wave 1)
- [x] 190-02-PLAN.md — Unique rule: per-backend parameterized COUNT, .ignore()/.ignore_on() exclude-self, identifier guard, validation.unique message (Wave 2)
- [x] 190-03-PLAN.md — AsyncValidator + AsyncValidationError: sync-first/fail-fast run loop, infra-vs-validation distinction (Wave 3)
- [x] 190-04-PLAN.md — Public re-exports (mod.rs + lib.rs) + end-to-end integration test + full quality gate (Wave 4)

### Phase 191: ConstraintMap + Portable UNIQUE-Violation Detection
**Goal**: A handler can intercept a DB UNIQUE constraint violation at the write site and surface it as a field-level validation error — with user input preserved and the same 303 redirect behavior as a proactive rule failure — closing the TOCTOU window the async `unique` rule cannot eliminate.
**Depends on**: Phase 190 (async validation surface stable before finalizing the complementary handler-level API)
**Requirements**: VALID-04, VALID-05
**Success Criteria** (what must be TRUE):
  1. `ConstraintMap::new().on("pages_slug_unique", "slug", "has already been taken").try_map(err)` returns `Ok(ValidationError)` when `err` is a UNIQUE constraint violation matching the registered constraint, and returns `Err(DbErr)` unchanged when it does not match — never silently swallowing any error
  2. A `DbErr` that is not a UNIQUE violation (e.g. a connection error) passes through `try_map` unchanged and reaches the existing `From<sea_orm::DbErr> for ActionError` fallback without any interception
  3. Constraint-name detection is backend-bifurcated: SQLite matches on `"table.column"` from the error message string; Postgres matches on the structured constraint name via `PgDatabaseError::constraint()` (no Postgres message-string parsing)
  4. A concurrent-insert scenario simulation (two handlers both pass the pre-write `unique` check, one INSERT hits the constraint) results in the losing request rendering the same field-level error with old input as a proactive rule failure — not a raw SQL message
  5. The `ConstraintMap` type and its API carry no consumer-specific strings inside the framework crate; all constraint names and field mappings are registered at the application call site (project-agnostic-crates rule)
**Plans**: 2 plans

Plans:
- [x] 191-01-PLAN.md — ConstraintMap + try_map + MapConstraintExt (struct, bifurcated detection, re-exports) [Wave 1]
- [x] 191-02-PLAN.md — SQLite TOCTOU/identity integration tests + Postgres manual gate + full quality gate [Wave 2]

**Gate**: Postgres constraint-name extraction (`PgDatabaseError::constraint()`) requires a real Postgres instance to verify. Phase closure criteria include either a Postgres CI step or a documented manual test step signed off in the phase VERIFICATION.md.

### Phase 192: ferro-mcp Template + Validation Docs
**Goal**: An agent scaffolding a handler with a unique field sees the two-layer proactive + defensive pattern together in the `action_handler` code template, and the validation documentation page covers both layers explicitly — so neither layer is used in isolation.
**Depends on**: Phase 191 (both runtime surfaces stable before templates and docs can accurately represent the composition)
**Requirements**: VALID-06
**Success Criteria** (what must be TRUE):
  1. The ferro-mcp `action_handler` code template includes both `AsyncValidator` with `unique` (proactive layer) and `ConstraintMap::try_map` at the write site (defensive layer) — no generated handler template shows `unique` without a downstream `ConstraintMap`
  2. `docs/src/features/validation.md` has a dedicated section for async rules showing `unique` with and without exclude-self, and a dedicated section for constraint mapping showing the `ConstraintMap` builder with the two-layer rationale (proactive catches UX case; defensive closes TOCTOU race)
  3. The two sections are cross-referenced so a developer reading either section discovers the other
**Plans:** 2/2 plans complete

Plans:
- [x] 192-01-PLAN.md — Enrich ferro-mcp `action_handler` template with both validation layers (proactive AsyncValidator+unique, defensive ConstraintMap/map_constraint) + SC1 catalog audit (Wave 1)
- [x] 192-02-PLAN.md — Add Async Rules (DB-backed) + Constraint Mapping sections to validation.md, cross-referenced, + MCP Tools note (Wave 1)

---

## v11.6.2 ferro-stripe Refund Event Completeness + 0.7.0 Release (Phase 193)

**Source:** gestiscilo-it v6.3 Phase 99 (Refund dashboard UX) field test — operator-locked Option B per gestiscilo `99-CONTEXT.md` D-27. Closes a payload-coverage gap in the `StripeChargeRefunded` typed event: the consumer's webhook handler (`on_refunded` in `src/controllers/cassa/stripe_handlers.rs`) needs `refund_id` from the event payload to look up its local `refunds` table row and mark `confirmed_at = now()`. Without the field, the consumer must bypass ferro-stripe via direct `stripe::` imports (violates V-95-01 "no direct `stripe::` import" gate established in v11.6).

**Design note:** the addition is purely additive on the consumer surface (`Option<String>`); existing consumers do not need to provide the field. ferro-stripe 0.7.0 is the published version label that captures Phase 189 (Manual Capture, shipped 2026-06-07 but not yet released) + this new refund_id work as combined breaking changes per gestiscilo Phase 97 D-14 expectation. Bumping past 0.6.x directly to 0.7.0 matches the consumer's planning aspiration and avoids stranding an intermediate 0.6.x release.

**Out of scope:** backporting `refund_id` to v0.5.x consumers (v0.7.0 is opt-in via the documented consumer bump per `feedback_ferro_publish.md` auto-publish flow).

**Consumer pairing:** gestiscilo-it v6.3 Phase 99 Plan 03 (webhook extension) hard-blocks on `StripeChargeRefunded::refund_id`; gestiscilo Phase 99 Plan 04 (closeout) hard-blocks on the ferro-stripe 0.7.0 publish per `feedback_ferro_publish.md`. Both gestiscilo plans have `<precondition>` first-task gates with grep / `cargo search` verification commands that block cleanly with operator-actionable messages until ferro Phase 193 ships.

### Phase 193: ferro-stripe Refund Event Completeness + 0.7.0 Release

**Goal:** Add `refund_id: Option<String>` to `StripeChargeRefunded` (parsed from the underlying `stripe::Refund::id`), update golden-JSON fixtures + parser-contract tests, and release ferro-stripe 0.7.0 to crates.io so the gestiscilo Phase 99 dashboard refund flow can round-trip operator-initiated refunds end-to-end without bypassing the framework.

**Depends on:** Phase 189 (manual capture work, already shipped in working tree — its additive changes are bundled into the 0.7.0 release label per gestiscilo Phase 97 D-14).

**Requirements:** STRIPE-REFUND-01, STRIPE-REFUND-02 (to be added to `.planning/REQUIREMENTS.md` when the phase opens — placeholder names; the ferro planner picks final IDs).

**Success Criteria** (what must be TRUE):
  1. `StripeChargeRefunded` struct in `ferro-stripe/src/webhook/events.rs` carries a new `pub refund_id: Option<String>` field positioned between `payment_intent_id` and `amount_refunded_cents`
  2. The `from_raw` parser populates `refund_id` from the charge's refunds list — `charge.refunds.data[].id` (a `charge.refunded` event carries an `EventObject::Charge`, not a top-level `Refund`; the refund id lives on `charge.refunds`, consistent with SC3). Recommended: `charge.refunds.as_ref().and_then(|l| l.data.first()).map(|r| r.id.to_string())`. Returns `None` when no refund is present (defensive — guards against a malformed `charge.refunded` with an empty refunds list). *(Corrected 2026-06-09: original SC2 named `EventObject::Refund`, which is not the object shape of a `charge.refunded` event.)*
  3. Golden-JSON fixtures at `ferro-stripe/tests/fixtures/charge_refunded_*.json` are updated to include the `refunds.data[].id` field that real Stripe webhooks ship; parser-contract test verifies the parsed event carries `refund_id = Some("re_...")` matching the fixture
  4. ferro-stripe Cargo.toml version label bumped from `0.5.0` to `0.7.0`; CHANGELOG.md entry under `## [0.7.0]` documents (a) the new `refund_id` field, (b) the Phase 189 manual-capture additions that were ready but unpublished, (c) the version-skip rationale (no 0.6.x release — combined breaking change per gestiscilo Phase 97 D-14)
  5. `cargo test --all-features` + `cargo clippy --all -- -D warnings` pass on the ferro-stripe workspace
  6. Push to ferro/master triggers GitHub Actions auto-publish per `feedback_ferro_publish.md`; `cargo search ferro-stripe --limit 1` returns `ferro-stripe = "0.7.0"` after publish completes
  7. Gestiscilo Phase 99 Plan 03 Task 1 precondition gate (grep `refund_id` in consumed `events.rs`) passes against the published version; Phase 99 Plan 04 Task 1 precondition gate (cargo search for 0.7.0) passes

**Plans:** 1/1 plans complete
- [x] 193-01-PLAN.md — refund_id field + parser from charge.refunds, fixture + parser-contract round-trip, 0.7.0 version bump + CHANGELOG (no push/publish)


---

## v11.6.3 ferro-stripe Connect Application Fee Helper (Phase 201)

**Source:** gestiscilo-it v7.1 photographer payment-gated-share field test (Marea Studio). The killer feature — "select files → one paid link → client pays → downloads" — routes the client payment to the tenant's Standard Connect account minus a platform fee. Auditing ferro-stripe 0.8.0 for that flow showed the Connect destination-charge surface is complete *except* the fee-computation step: the platform percent lives in `StripeConfig.application_fee_percent` but nothing turns it into a cents amount for `CheckoutBuilder::destination`.

**Already shipped in 0.8.0 (no work):** `account::{create_account, create_link, retrieve_account}` (Standard), `CheckoutBuilder::destination(account_id, fee_cents)` (sets `application_fee_amount` + `transfer_data.destination` + `on_behalf_of`, composes with `manual_capture`), `WebhookEvent.account: Option<String>` (Connect account routing), `StripeConnectAccountUpdated { account_id, charges_enabled, payouts_enabled, details_submitted }`, `verify_webhook(body, sig, secret)` (consumer passes the connect secret), and `StripeConfig.{connect_webhook_secret, application_fee_percent}`.

**Consumer pairing:** gestiscilo-it v6.10 Phase 204 consumes `application_fee_for` via the published 0.9.0 bump; gestiscilo Phase 203 (Connect webhook endpoint + secret split) needs no new ferro surface (it consumes the already-shipped 0.8.0 webhook primitives). Auto-publishes via GH Actions on push to master per `feedback_ferro_publish.md`.

### Phase 201: ferro-stripe Connect application-fee helper + config-status parity + docs

**Goal:** A consumer holding a charge amount and a configured platform fee percent can compute the application fee in one call, introspect Connect-fee readiness via ferro-mcp, and follow a documented end-to-end Connect application-fee example.

**Depends on:** nothing (additive on 0.8.0).

**Success Criteria:**
  1. `StripeConfig::application_fee_for(amount_cents: i64) -> Option<i64>` returns `Some(round(amount_cents × application_fee_percent / 100))` when the percent is set and > 0; `None` when unset or ≤ 0; result is non-negative and never exceeds `amount_cents`; unit tests cover unset, 0%, normal, rounding, and clamp cases
  2. ferro-mcp `stripe_config_status` reports `connect_webhook_secret` presence (bool, never the value) and `application_fee_percent` (the number or null) alongside existing fields
  3. `docs/src/features/stripe.md` gains a "Connect destination charges with a platform fee" section walking account create→link→`account.updated` capability persistence→`CheckoutBuilder::destination(account_id, StripeConfig::application_fee_for(amount))`, and notes the correspondence with the manual-capture flow from Phase 189
  4. ferro-stripe Cargo.toml bumped `0.8.0 → 0.9.0`; CHANGELOG `## [0.9.0]` documents the helper + mcp parity + docs (additive, non-breaking)
  5. `cargo test --all-features` + `cargo clippy --all -- -D warnings` pass on the ferro-stripe workspace
  6. Push to ferro/master triggers GH Actions auto-publish; `cargo search ferro-stripe --limit 1` returns `ferro-stripe = "0.9.0"` after publish completes

**Status:** ✅ Shipped 2026-06-11. Implemented directly on master in commit `705bac6b` (outside the GSD flow), verified retroactively — see `.planning/phases/201-ferro-stripe-connect-application-fee-helper-config-status-parity-docs/201-VERIFICATION.md`. Criteria 1-5 green (clippy + tests re-run clean); criterion 6 (publish) is a pending operator `git push`.

**Plans:** None — implemented outside the discuss→plan→execute flow; reconciled via retroactive verification.


---

## v12.5 Projection Checkpoint (Phases 194–196)

**Source:** Design spec `docs/superpowers/specs/2026-06-09-projection-checkpoint-design.md`. Closes the one gap in ferro's generate→verify loop that no existing tool covers: cross-artifact seam coherence anchored on a projection. Killer feature: a projection field referencing a model attribute the migration never created surfaces statically in one MCP call instead of at runtime.

**Design decisions locked at roadmap time:**
- **Seam cascade:** when seam 1 (well-formed) fails, seams 4 and 5 report `not_checked` with `reason: "seam_1_failed"` (they depend on a valid ServiceDef parse). Seam 2 (field→column) runs independently if `reconstruct_service_def` succeeds. Seam 3 (action→route) runs independently. If seam 4 (render) fails, seam 5 (props→contract) reports `not_checked` with `reason: "seam_4_failed"`.
- **Fix-string normalization:** the checkpoint normalizes all seam findings into the uniform `Finding { subject, detail, fix }` shape at the module boundary. Sub-validator heterogeneous output shapes (`fix_suggestions[].details`, `message/candidate`, `mismatches[].details`) are translated by per-seam normalization functions inside `checkpoint_projection.rs`. This output contract is established in Phase 194 (P1) and reused verbatim by Phase 195 wrapper seams.
- **Ambient status freshness:** `application_info` and `projection_coverage` read the `.ferro/checkpoints/{name}.json` status cache written by the last `run_for` call (stale-ok read). No live recompute on ambient status queries. Mitigated by the inline hook on generators (Phase 195) ensuring the cache is refreshed on every generation. The alternative (always-fresh recompute) has I/O cost proportional to projection count and is not acceptable for `application_info` (called frequently by agents surveying the project).

**Conceptual coherence note:** no new abstraction. The unit of verification is the intent slice, anchored on the projection/ServiceDef. The checkpoint is a pure orchestrator — it owns exactly one new check (field→column) plus aggregation. Every other seam delegates to an existing validator with no logic duplication.

## Phases

- [x] **Phase 194: Core Checkpoint Tool** — `checkpoint_projection` MCP tool + field→column seam (the new check) + aggregation + ranked `next_steps` + `not_checked` coverage-honesty invariant + reconstruction completeness assertion + false-positive exemptions + status cache write (completed 2026-06-09)
- [x] **Phase 195: Close the Loop by Default** — wrapper seams 1/3/4/5 dispatching to existing validators + inline verdict hook in `generate_projection`/`json_ui_generate` + per-projection checkpoint status in `application_info`/`projection_coverage` (completed 2026-06-10)
- [x] **Phase 196: Dogfood Acceptance + Hardening** — acceptance run across synthetic catalog (including a deliberately poisoned fixture) + one live consumer; `next_steps` capped to 5; go/no-go gate (completed 2026-06-10)

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 194. Core Checkpoint Tool | 3/3 | Complete    | 2026-06-09 |
| 195. Close the Loop by Default | 4/4 | Complete    | 2026-06-10 |
| 196. Dogfood Acceptance + Hardening | 4/4 | Complete    | 2026-06-10 |

#### Phase Details

### Phase 194: Core Checkpoint Tool

**Goal:** An agent calling `checkpoint_projection { name }` receives a single structured verdict (`pass`/`warn`/`fail`) with per-seam results and a ranked, actionable `next_steps` list. The field→column seam is the load-bearing new check: it resolves the projection to its source model via the same predicate `projection_coverage` uses, compares every `FieldDef` name against the entity's column set, and reports findings with `source: "checkpoint"` and a concrete `fix` string. Coverage-honesty holds by construction: `not_checked` is a distinct `SeamStatus` variant, never coerced to `pass`.

**Depends on:** Phase 193 (continues numbering; no runtime dependency)

**Requirements:** CHK-01, CHK-02, CHK-03, CHK-04, CHK-05, CHK-06

**Success Criteria** (what must be TRUE):
  1. An agent calling `checkpoint_projection { name: "Booking" }` on a projection with a field referencing no backing entity column receives `status: "fail"`, a `seams` entry with `seam: "field_to_column"` and a finding that names the dangling field in `subject` and a concrete migration step in `fix`.
  2. An agent calling `checkpoint_projection` on a projection whose source model cannot be resolved receives `seams[field_to_column].status: "not_checked"` (never `"pass"`), and the overall verdict is not elevated to `"fail"` solely because of this.
  3. A projection with a `has_many` or `belongs_to` relationship field and a computed display field produces zero findings on the field→column seam — no false positives on legitimate non-column fields.
  4. A projection source file where the field-builder invocation count exceeds `ServiceDef.fields.len()` (reconstruction is incomplete) reports a `warn` on the field→column seam stating reconstruction may be incomplete — not a silent clean result.
  5. A mixed-seam fixture with a seam 2 `fail` and a seam 1 `warn` produces a `next_steps` list where the seam 2 failure appears before the seam 1 warning.

**Plans:** 3/3 plans complete

- [x] 194-01-PLAN.md — Foundation: module + public output types (Finding/SeamStatus/SeamResult/Verdict) + path-traversal name guard + test scaffold (Wave 1)
- [x] 194-02-PLAN.md — Field→column seam: completeness counter + dangling-field detection + not_checked paths + relationship/computed exemption (Wave 2)
- [x] 194-03-PLAN.md — Aggregation + ranked/deduped next_steps + cache write + MCP tool registration + docs (Wave 3)

### Phase 195: Close the Loop by Default

**Goal:** Verification happens without the agent asking for it. Wrapper seams 1, 3, 4, and 5 dispatch to existing validators (`validate_projection`, `json_ui_verify_action`, `render_projection` + `json_ui_validate_spec`, `validate_contracts`) and fold their output into the unified verdict — no validation logic reimplemented in the checkpoint. `generate_projection` and `json_ui_generate` embed the checkpoint verdict inline (summary format only — not a full five-seam breakdown immediately after generation). `application_info` and `projection_coverage` surface per-projection checkpoint status (`clean`/`failing`/`unverified`) from the `.ferro/checkpoints/{name}.json` cache.

**Depends on:** Phase 194 (checkpoint tool and output types stable)

**Requirements:** CHK-07, CHK-08, CHK-09

**Success Criteria** (what must be TRUE):
  1. An agent calling `generate_projection` receives a `checkpoint` key in the response; it contains at minimum a top-level `status` field and does not present five `not_checked` seam entries with empty findings arrays (summary format, not full seam breakdown).
  2. An agent calling `projection_coverage` sees a `checkpoint_status` field (`"clean"`, `"failing"`, or `"unverified"`) on each covered projection without issuing a separate `checkpoint_projection` call.
  3. An agent calling `application_info` sees a `projection_checkpoint` summary with `total_projections`, `clean`, `failing`, and `unverified` counts reflecting the last-run cache state.
  4. The `source` field on every seam finding produced by a wrapper seam names the delegating validator (`"validate_projection"`, `"json_ui_verify_action"`, `"render_projection"`, `"json_ui_validate_spec"`, `"validate_contracts"`) — `"checkpoint"` appears only on field→column (seam 2) findings, confirming no logic was reimplemented.

**Plans:** 4/4 plans complete

Plans:
- [x] 195-01-PLAN.md — Foundation: async run_for/execute + seam-name reconciliation (D-01) + VerdictSummary + read_ambient_status (Wave 1)
- [x] 195-02-PLAN.md — Wrapper seams 1/3/4/5 dispatch+normalization + seam cascade (D-06) + SC-4 guard (Wave 2)
- [x] 195-03-PLAN.md — Inline checkpoint hook in generate_projection + json_ui_generate (CHK-07) (Wave 3)
- [x] 195-04-PLAN.md — Ambient status in projection_coverage + application_info (CHK-08) (Wave 4)

### Phase 196: Dogfood Acceptance + Hardening

**Goal:** The checkpoint earns its place by finding a real seam defect in a real project. The synthetic app catalog must include at least one deliberately poisoned projection (a field with no backing migration column, since model-derived projections auto-pass seam 2 and would make the gate vacuous). The live consumer must produce at least one finding (fail or warn on any seam). Any wrapper seam that produces zero findings across all dogfood inputs is demoted to reporting `not_checked` by default rather than shipped active. `next_steps` is capped to 5 entries.

**Depends on:** Phase 195 (all seams active and inline hook in place)

**Requirements:** CHK-10

**Success Criteria** (what must be TRUE):
  1. The poisoned synthetic-catalog projection produces `status: "fail"` with the field→column seam finding naming exactly the planted dangling field in `subject` — and no other field in the same projection.
  2. Running `checkpoint_projection` against at least one live consumer projection produces at least one finding (fail or warn on any seam); a run that finds nothing fails acceptance and the design is revisited, not shipped.
  3. `next_steps` in any verdict contains at most 5 entries; a fixture with more than 5 findings confirms the cap is enforced.
  4. Any wrapper seam (1, 3, 4, 5) that produced zero findings across all dogfood inputs is documented as `not_checked`-by-default in the tool description, not silently omitted.

**Plans:** 4/4 plans complete

- [x] 196-01-PLAN.md — D-05: next_steps cap 10→5 (MAX_NEXT_STEPS const, 4 doc locations, over-cap test) (Wave 1)
- [x] 196-02-PLAN.md — D-01/SC-1: poisoned-fixture acceptance test (one dangling field, exact-subject + no-other-field assertions) (Wave 2)
- [x] 196-03-PLAN.md — D-02/D-03/SC-2: dogfood run against app/ (direct per-file seam calls) + 196-ACCEPTANCE.md GO/NO-GO gate (Wave 3)
- [x] 196-04-PLAN.md — D-04/SC-4: evidence-driven demotion of zero-finding wrapper seams to not_checked + service.rs/docs (Wave 4)


---

## v12.6 Consumer App MCP (Browser Login) (Phases 197–200)

**Milestone goal:** A deployed ferro application serves its own OAuth-protected MCP endpoint. A consumer agent authenticates through the browser, receives a token bound to `(user, tenant)`, and calls a tool rendered from an opt-in projection. The tool returns that tenant's data, gated by the application's existing authorization policies.

**Design center:** The MCP surface is a rendering target for the projection / intent system — the same `ServiceDef` that renders to JSON-UI (visual) also renders to MCP tool schema and tool output (agent-consumable) via `McpRenderer`. One source of truth; no parallel hand-maintained tool contract.

**Design spec:** `docs/superpowers/specs/2026-06-10-consumer-app-mcp-browser-login-design.md`

### Phases

- [x] **Phase 197: McpRenderer & ferro-mcp-server** — New output crate `ferro-mcp-server` with `McpRenderer` implementing the `Renderer` trait; projection→tool schema derivation from `ServiceDef`; opt-in `mcp_exposed` marker; unit-tested in-process without a live HTTP server. (completed 2026-06-10)
- [x] **Phase 198: Streamable HTTP Endpoint + Unauthenticated Challenge** — App-served `POST /mcp` supporting `initialize` / `tools/list` / `tools/call`; `401` + `WWW-Authenticate` on unauthenticated requests. (completed 2026-06-10)
- [x] **Phase 199: OAuth Browser Login** — `.well-known` discovery metadata, dynamic client registration, `GET /authorize` (reuses existing login + consent step, issues PKCE auth code), `POST /token` (exchanges code for audience-bound `(user, tenant)` access token); bearer-token validation middleware on `/mcp`. (completed 2026-06-10)
- [x] **Phase 200: Per-Tenant Scoping, Policy Authorization & Dogfood Acceptance** — Tool calls execute within the token's tenant context via existing multi-tenant middleware; policy layer gates each call; dogfood GO/NO-GO acceptance: a real MCP client completes browser login against a live consumer application and lists one projection's data scoped to the authenticated tenant. (completed 2026-06-10)

### Phase Details

### Phase 197: McpRenderer & ferro-mcp-server

**Goal:** A `ServiceDef`-marked projection appears in an in-process `tools/list` call as exactly one MCP tool, with input JSON schema derived from the projection's filter and pagination fields and output derived from its read path. `ferro-projections` gains no renderer dependency.

**Depends on:** Nothing (first phase of milestone; `ferro-projections` `Renderer` trait and `ServiceDef` from v11.5 are prerequisites already shipped).

**Requirements:** AMCP-01, AMCP-02, AMCP-03, AMCP-04

**Success Criteria** (what must be TRUE):
  1. A projection with `mcp_exposed: true` appears in `tools/list`; a projection without it does not.
  2. The tool's `inputSchema` is derived solely from the projection's `ServiceDef` filter and pagination fields — no separately declared schema exists.
  3. Calling the tool's dispatch function executes the projection's existing read path and returns its rows as MCP structured content, with the output shape derived from the projection.
  4. `ferro-projections` has no new dependency on `ferro-mcp-server`; the dependency direction is `ferro-mcp-server` → `ferro-projections`.
  5. The new crate is registered in `.github/workflows/publish.yml` at the correct publish wave.

**Plans:** 3/3 plans complete

Plans:
- [x] 197-01-PLAN.md — Scaffold ferro-mcp-server crate + add mcp_exposed marker to ServiceDef
- [x] 197-02-PLAN.md — McpRenderer::render + inputSchema derivation from ServiceDef fields
- [x] 197-03-PLAN.md — Dispatch read path (parameterized SQL) + SQLite test + publish.yml Wave 2 registration
**UI hint**: no

### Phase 198: Streamable HTTP Endpoint + Unauthenticated Challenge

**Goal:** The application server mounts a Streamable HTTP MCP endpoint. An unauthenticated request to it returns `401` with a `WWW-Authenticate` header that a standard MCP client can follow to discover the protected-resource metadata. Authenticated calls are not yet wired (that is Phase 199's responsibility).

**Depends on:** Phase 197 (`McpRenderer` and tool dispatch available).

**Requirements:** AMCP-05, AMCP-06

**Success Criteria** (what must be TRUE):
  1. `POST /mcp` handles `initialize`, `tools/list`, and `tools/call` JSON-RPC methods over Streamable HTTP.
  2. An unauthenticated `POST /mcp` returns HTTP `401` with a `WWW-Authenticate` header referencing the protected-resource metadata URL.
  3. The endpoint integrates into the application server via the same middleware stack as other framework routes.
  4. Integration tests exercise the three JSON-RPC methods and the `401` path without requiring a live OAuth server.

**Plans:** 2/2 plans complete
- [x] 198-01-PLAN.md — ferro-mcp-server JSON-RPC dispatch (initialize/tools-list/tools-call) + config + bearer seam + integration tests (Wave 1)
- [x] 198-02-PLAN.md — app POST /mcp handler: 401+WWW-Authenticate challenge, route mount, GET 405, expose order projection (Wave 2)
**UI hint**: no

### Phase 199: OAuth Browser Login

**Goal:** A standard MCP client can discover the authorization server, dynamically register, complete a browser authorization-code + PKCE flow that reuses the application's existing login, approve a consent screen, and exchange the code for an access token bound to `(user, tenant)` with this endpoint as audience. The bearer-token validation middleware on `/mcp` accepts valid tokens and rejects invalid or expired ones.

**Depends on:** Phase 198 (MCP endpoint and `401` challenge in place).

**Requirements:** AMCP-07, AMCP-08, AMCP-09

**Success Criteria** (what must be TRUE):
  1. `GET /.well-known/oauth-protected-resource` and `GET /.well-known/oauth-authorization-server` return spec-compliant discovery documents advertising authorization-code + PKCE (S256).
  2. `POST /register` (dynamic client registration, RFC 7591) accepts a registration request and returns a `client_id`.
  3. `GET /authorize` redirects to the application's existing login when no session exists; after login, presents a consent screen; after consent approval, redirects back with a PKCE authorization code.
  4. `POST /token` exchanges a valid code + PKCE verifier for an access token bound to `(user, tenant)` with the MCP endpoint as audience and a short expiry.
  5. An invalid or expired bearer token on `POST /mcp` returns `401`; an audience or tenant mismatch returns `403`.

**Plans:** 5/5 plans complete
- [x] 199-01-PLAN.md — Crate scaffold (ferro-mcp-oauth Wave-2), fail-closed OAuthConfig + sanitized_app_url, oauth_clients migration, flow-test harness, publish.yml/workspace registration (Wave 1; AMCP-07/08/09)
- [x] 199-02-PLAN.md — Discovery metadata (RFC 8414/9728) + Dynamic Client Registration (RFC 7591) (Wave 2; AMCP-07; SC-1, SC-2)
- [x] 199-03-PLAN.md — Crypto core: PKCE S256 (constant-time) + HS256 JWT mint/decode (alg-pinned, aud-bound) + validate_bearer 401/403 mapping (Wave 2; AMCP-08/09; SC-4, SC-5)
- [x] 199-04-PLAN.md — Browser flow: GET/POST /authorize (login reuse + consent + CSRF), /token single-use code + PKCE verify + JWT mint, end-to-end PKCE integration test (Wave 3; AMCP-08; SC-3, SC-4)
- [x] 199-05-PLAN.md — Seam wiring: mount 6 OAuth routes, /mcp bearer validation + Origin check, post-login return-to, delete extract_bearer (Wave 4; AMCP-09; SC-5)
**UI hint**: no

### Phase 200: Per-Tenant Scoping, Policy Authorization & Dogfood Acceptance

**Goal:** A tool call executes inside the token's tenant context via the existing multi-tenant middleware and is gated by the same policy layer as the web surface. An agent's reach equals the authenticated user's reach — no parallel permission system, no per-tool ownership filter. The phase closes with a dogfood GO/NO-GO gate: a real MCP client completes browser login against a live consumer application and lists one projection's tenant-scoped data. A run that does not work end to end is cause to revise the design rather than ship.

**Depends on:** Phase 199 (valid tokens issued and validated).

**Requirements:** AMCP-10, AMCP-11

**Success Criteria** (what must be TRUE):
  1. A tool call with a token scoped to tenant A returns only tenant A's rows; a token scoped to tenant B returns only tenant B's rows.
  2. A tool call denied by the application's existing policy layer returns an MCP tool error with a clear message and no data disclosure.
  3. The tenant context established by the MCP middleware is structurally identical to the context established by the web-surface multi-tenant middleware — no second permission system exists.
  4. Dogfood GO/NO-GO: a real MCP client (e.g. Claude Desktop or a script using the MCP SDK) completes a browser login against a live consumer application and successfully calls `tools/list` followed by `tools/call` for one exposed projection, receiving that tenant's rows. A run that fails end to end is GO/NO-GO = NO-GO and the design is revised before marking this phase complete.

**Plans:** 7/7 plans complete
- [x] 200-01-PLAN.md — ServiceDef tenant_column/mcp_ability plain-metadata fields (Wave 1; AMCP-10/11)
- [x] 200-02-PLAN.md — dispatch tenant predicate (bound param, fail-closed) + handle_tools_call forwarding (Wave 1; AMCP-10)
- [x] 200-03-PLAN.md — two-tenant fixture: tenants/orders/users-tenant_id migrations + models (Wave 1; AMCP-10)
- [x] 200-04-PLAN.md — BearerAuthMiddleware + /mcp & /authorize wiring + Gate ability + DbTenantLookup + seed + order projection metadata (Wave 2; AMCP-10/11)
- [x] 200-05-PLAN.md — controller Gate::authorize_for check + fail-closed + D-09 tool error + tenant_id forwarding (Wave 3; AMCP-11)
- [x] 200-06-PLAN.md — two-tenant isolation + middleware-parity integration tests (Wave 4; AMCP-10/11; SC-1/SC-3)
- [x] 200-07-PLAN.md — dogfood scripted MCP client + GO/NO-GO acceptance (Wave 5; AMCP-10/11; SC-4)
**UI hint**: no

### Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 197. McpRenderer & ferro-mcp-server | 3/3 | Complete    | 2026-06-10 |
| 198. Streamable HTTP Endpoint + Unauthenticated Challenge | 2/2 | Complete    | 2026-06-10 |
| 199. OAuth Browser Login | 5/5 | Complete    | 2026-06-10 |
| 200. Per-Tenant Scoping, Policy Authorization & Dogfood Acceptance | 7/7 | Complete    | 2026-06-10 |

### Phase 205: Fix ferro-mcp-server tools/call result content blocks — wrap projection rows as valid MCP content blocks (type:text / structuredContent) so real MCP clients parse the result; add a client-schema interop regression test; re-run the live :8090 browser-OAuth dogfood (alice@acme.test list_order). Defect isolated to result formatting; OAuth/login-resume/consent/token/tenant-scoping already verified working.

**Goal:** The ferro-mcp-server `tools/call` success result is a valid MCP `CallToolResult` (one `type:text` content block + `structuredContent` carrying `{rows,total,limit,offset}`), so a strict MCP client parses it without Zod errors; a client-schema interop regression test deserializes the emitted result with the client's own rmcp type; the live :8090 browser-OAuth dogfood (alice@acme.test → list_order) re-runs to GO with tenant scoping intact.
**Requirements**: AMCP-03 (content fix), AMCP-10 (tenant scoping preserved)
**Depends on:** Phase 204
**Plans:** 3/3 plans complete

Plans:
- [x] 205-01-PLAN.md — Fix the Ok arm (CallToolResult::structured) + inline D-04 interop regression test in jsonrpc.rs
- [x] 205-02-PLAN.md — Re-point tenant_a/tenant_b isolation tests to structuredContent.rows + assert content[0].type==text
- [x] 205-03-PLAN.md — Live :8090 browser-OAuth dogfood (D-06 acceptance gate, autonomous:false) + 205-ACCEPTANCE.md

---

## v12.7 Passwordless MCP Auth (Phases 202–203)

**Source:** Field finding while validating v12.6 (Consumer App MCP) against the gestiscilo consumer. The v12.6 OAuth browser-login flow was verified against the bundled sample app, which uses a synchronous password login. The real consumer is passwordless (magic-link), which exposes two gaps the sample app did not:

1. **Login-resume continuation.** `ferro-mcp-oauth`'s `/authorize` stores the in-flight authorize request as `oauth_return_to` in the session and redirects unauthenticated users to the app login (D-06 login reuse). A synchronous password handler redirects back to `oauth_return_to` on success within the same request. A magic-link `verify` handler runs in a separate request and, by default, redirects to a dashboard — never resuming the OAuth flow, so no authorization code is issued.
2. **Cross-device delivery.** Magic-link emails open on any device. The authorization-code-over-loopback flow requires the code to redirect to a callback listener on the client's own machine, which a different device cannot reach.

**Conceptual coherence:** both phases reuse the v12.6 consent and tenant-scoping surfaces and the existing token issuer — no parallel permission system, no second token path. Phase 202 makes the login-resume contract explicit and feature-agnostic; Phase 203 adds an alternate front door (device grant) to the same token issuance.

### Phase 202: Login-resume contract + magic-link sample app ✅ shipped 2026-06-11

**Verification:** 5/5 success criteria passed (`202-VERIFICATION.md`). Code review: 0 critical, 4 non-blocking warnings (2 pre-existing Phase 199, 2 out of scope). Full `--all-features` clippy + test + `cargo doc -Dwarnings` gate green; CWD-independent boot confirmed.

**Goal:** A passwordless (magic-link) ferro app completes the OAuth/MCP browser-login flow because its login handler resumes the authorize request via `oauth_return_to`, and the bundled sample app demonstrates this as the golden-path exemplar.

**Depends on:** v12.6 (OAuth browser login).

**Success Criteria:**
  1. `ferro-mcp-oauth` exposes a documented login-resume helper (e.g. `oauth_resume_redirect()` / `take_oauth_return_to()`) that a login handler calls to obtain the post-login redirect target (the stored `oauth_return_to`, or a caller-provided default), clearing it from the session; docs state that any login method must honor it to participate in the OAuth flow.
  2. The bundled sample app login is converted from password to magic-link: a request-link handler issues a single-use, TTL-bounded token; the `verify` handler authenticates and redirects via the resume helper. In dev (`APP_ENV=local`) the link is surfaced without a real email send.
  3. An acceptance test drives the full async sequence: unauthenticated `GET /authorize` → 302 `/auth/login` → request link → `verify` (with `oauth_return_to` in session) → 302 resume `/authorize` → consent page rendered.
  4. The login + magic-link views render through JSON-UI (consistent with the rest of the sample app) and are themed via `ThemeMiddleware`.
  5. `cargo clippy --all-targets --all-features -- -D warnings` + `cargo test --all-features` pass; the app boots from any working directory (no CWD-relative startup panics).

**Consumer pairing:** gestiscilo `verify_magic_link` adopts the resume helper so gestiscilo users complete the MCP browser login on the same device. Cross-device is addressed by Phase 203.

**Plans:** 5/5 plans complete

Plans:
- [x] 202-01-PLAN.md — Resume contract in ferro-mcp-oauth: resume.rs helpers + key constant, refactor authorize.rs/consent.rs (single owner), lib.rs exports, rustdoc + authentication.md doc (Wave 1; SC-1)
- [x] 202-02-PLAN.md — Magic-link token + handlers in app: rand/base64 deps, request-link handler, GET /auth/verify, delete password path, dev/mail branch, single-use/expiry/dev-surface tests (Wave 2; SC-2)
- [x] 202-03-PLAN.md — JSON-UI auth views: email-only login.json + login_confirm.json, update login_view test (Wave 3; SC-4)
- [x] 202-04-PLAN.md — Async-flow acceptance test: store-return-to → token-issue → consume → resume-redirect, offline staged (Wave 3; SC-3)
- [x] 202-05-PLAN.md — Gate + CWD-independent boot: fmt + clippy --all-features + test --all-features green, boot from repo root (Wave 4; SC-5)

### Phase 203: OAuth Device Authorization Grant (RFC 8628)

**Goal:** `ferro-mcp-oauth` supports the OAuth 2.0 Device Authorization Grant so passwordless, cross-device, and headless/CLI MCP clients can authenticate without a same-device browser callback.

**Depends on:** Phase 202 (shares the login + consent + tenant-scoping surfaces).

**Success Criteria:**
  1. `POST /device_authorization` returns `device_code`, `user_code`, `verification_uri`, `verification_uri_complete`, `expires_in`, and `interval` per RFC 8628 §3.2.
  2. A verification page (`GET` `verification_uri`) prompts for / confirms the `user_code`, authenticates the user (reusing the app login + consent), and binds the `device_code` to the authenticated user and tenant.
  3. `POST /token` with `grant_type=urn:ietf:params:oauth:grant-type:device_code` returns `authorization_pending`, `slow_down`, `expired_token`, or `access_token` per RFC 8628 §3.5; issued tokens are audience-bound and tenant-scoped identically to the authorization-code flow.
  4. Authorization-server discovery metadata advertises `device_authorization_endpoint`; `device_code` / `user_code` are single-use with a TTL.
  5. Tests cover the pending→approved polling transition, expiry, `slow_down` backoff, denied consent, and tenant binding; `--all-features` clippy + tests pass.

**Consumer pairing:** gestiscilo (magic-link, cross-device users) adopts device grant as its primary MCP authentication path.

**Plans:** 5/5 plans complete

- [x] 203-01-PLAN.md — device.rs foundation: DeviceGrant record + user_code/device_code primitives (SC-1/SC-4 substrate)
- [x] 203-02-PLAN.md — discovery.rs: advertise device_authorization_endpoint + device-code grant type (SC-4)
- [x] 203-03-PLAN.md — device.rs handlers: device_authorization + verification page (login-resume, CSRF, user/tenant binding) (SC-1/SC-2)
- [x] 203-04-PLAN.md — token.rs device-code arm: §3.5 polling state machine + identical-mint Approved arm (SC-3)
- [x] 203-05-PLAN.md — wiring (lib.rs exports + routes.rs mounts) + full SC-5 test matrix + blocking workspace gate (SC-2/SC-3/SC-4/SC-5)

### Phase 204: ferro-storage provider-agnostic CDN configuration

**Goal:** Collapse the AWS / DO / Bunny / Cloudflare CDN env-var clusters in ferro-storage into a single provider-agnostic quartet — `CDN_URL` (public base), `CDN_PROVIDER` (`none` | `digitalocean` | `bunny` | `cloudflare`), `CDN_PURGE_TOKEN` (provider API token), `CDN_PURGE_ZONE` (provider-specific zone/endpoint id). Old variable names (`AWS_CDN_URL`, `DO_SPACES_CDN_ID`, `DIGITALOCEAN_ACCESS_TOKEN`, `BUNNY_CDN_URL`, `BUNNY_ACCESS_KEY`, `CF_CDN_URL`, `CF_API_TOKEN`, `CF_ZONE_ID`) read as deprecated fallbacks for one release with a `tracing::warn!` log.

**Depends on:** nothing (additive on current ferro-storage).

**Success Criteria:**
  1. `ferro_storage::cdn::Config::from_env` reads `CDN_URL` / `CDN_PROVIDER` / `CDN_PURGE_TOKEN` / `CDN_PURGE_ZONE` as primary
  2. When `CDN_URL` is unset, fall back to `AWS_CDN_URL` (logged warn); same fallback chain for the other three vars (`CDN_PURGE_ZONE` ← `DO_SPACES_CDN_ID`/`CF_ZONE_ID`; `CDN_PURGE_TOKEN` ← `DIGITALOCEAN_ACCESS_TOKEN`/`CF_API_TOKEN`/`BUNNY_ACCESS_KEY`)
  3. `Disk::cdn_url()` returns the same URL it does today for unchanged callers (parity test against `AWS_CDN_URL`-only env)
  4. `purge()` authenticates against the same DO Spaces CDN API today when using the legacy vars (parity test)
  5. `CDN_PROVIDER=none` → `purge()` is an explicit logged no-op; `CDN_PROVIDER` invalid → boot error with a clear message listing valid values
  6. ferro-storage Cargo.toml bumps minor version; CHANGELOG `## [X.Y.0]` documents the new vars + deprecation policy
  7. `cargo test --all-features` + `cargo clippy --all -- -D warnings` pass on the ferro-storage workspace

**Consumer pairing:** gestiscilo Phase 205 (consumer-side rename + atomic Cargo.toml bump per the Phase 176 ↔ ferro Phase 181 closeout pattern).

**Origin:** Surfaced 2026-06-11 during a cross-repo env-files audit. Today's setup fragments the same DO Spaces CDN across 3 different env-var prefix conventions (`AWS_*`, `SPACES_*`, `DO_SPACES_*`) plus the parallel `BUNNY_*` and `CF_*` clusters — provider-agnostic naming makes the abstraction match what the code actually does (one CDN, one provider at a time).

**Plans:** 3/3 plans complete

Plans:
- [x] 204-01-PLAN.md — Unified cdn::Config + CdnProvider + env_with_fallback + build_purge_api + error variants + exports (SC-1/2/5a/b/c)
- [x] 204-02-PLAN.md — Wire CDN_URL through cdn::Config at config.rs:119; SC-3 display-URL parity + SC-4 DO purge-auth parity tests
- [x] 204-03-PLAN.md — Version bump 0.2.53 + CHANGELOG + .env.example/docs quartet migration + BLOCKING full-workspace gate (SC-6/7)

### Phase 206: ferro-storage provider-agnostic STORAGE_* env vars

**Goal:** Apply the Phase 204 provider-agnostic naming pattern to the six S3-style env vars (`AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` / `AWS_DEFAULT_REGION` / `AWS_BUCKET` / `AWS_URL` / `AWS_PUBLIC_URL`). All ferro-storage's S3 surface — `StorageConfig::from_env`, `S3Driver::new`, the facade's S3 endpoint read — now read provider-agnostic `STORAGE_*` names through the hoisted `env_with_fallback` helper, with the legacy `AWS_*` names accepted as deprecated aliases for one release. Honest naming: the `s3` driver targets *any* S3-compatible backend (DO Spaces, Wasabi, R2, B2, MinIO), so `STORAGE_*` reflects what's actually abstracted while `FILESYSTEM_DISK` selects the driver.

**Depends on:** Phase 204 (env_with_fallback helper + deprecation-warning convention).

**Success Criteria:**
  1. `StorageConfig::from_env` reads `STORAGE_BUCKET` / `STORAGE_REGION` / `STORAGE_ENDPOINT` / `STORAGE_PUBLIC_URL` primary with `AWS_*` legacy fallback (per-var `tracing::warn!`)
  2. `S3Driver::new` reads `STORAGE_ACCESS_KEY_ID` / `STORAGE_SECRET_KEY` primary with `AWS_*` legacy fallback
  3. `Storage::create_driver` (facade) reads `STORAGE_ENDPOINT` primary with `AWS_URL` legacy fallback
  4. `env_with_fallback` hoisted from `cdn::mod` private fn to crate-level `env_helpers` module (reused by all four surfaces)
  5. Legacy fallback parity: existing `from_env_cdn_url` + `cdn_url_parity_aws_fallback` tests (which set `AWS_BUCKET` + `AWS_CDN_URL`) continue to pass byte-identical
  6. New primary-path test `from_env_storage_primary` sets `STORAGE_BUCKET` / `STORAGE_REGION` / `STORAGE_PUBLIC_URL` and asserts the s3 disk registers with the expected fields
  7. `ferro/app/.env.example` declares the `STORAGE_*` set as primary with a deprecated-alias note for `AWS_*`
  8. Workspace `Cargo.toml` bumps to 0.2.54; `ferro-storage/CHANGELOG.md` documents the rename + deprecation table

**Consumer pairing:** gestiscilo follow-up phase mirrors Phase 205's shape — bump ferro to 0.2.54, rename `.env.example` + `app-env/production/.env` to `STORAGE_*` (preserve values verbatim), update ROADMAP Pending Operator Tasks.

**Origin:** Operator-locked principle on 2026-06-12 after Phase 204/205 shipped — ferro env vars must name the *role* (CDN, STORAGE, MAIL), not the *vendor* (AWS, DO, RESEND), where the role is genuinely provider-agnostic. The AWS_* family was the next candidate identified during that conversation. Provider-specific names stay (Stripe / Resend / WhatsApp Cloud / Anthropic) because their values are stamped with one vendor's API contract.

**Plans:** 1/1 complete (single-wave, mechanical rename — see `phases/206-ferro-storage-provider-agnostic-storage-env-vars/206-01-PLAN.md`)

---

## ✅ v13.0 Compressive Validation (Phases 207–211, complete 2026-06-13)

**Milestone Goal:** Validate the projection / intent abstraction empirically — the first slice of the Road to v1.0 program. Targets v1.0 criterion #2 ("projection / intent validated through real applications and a synthetic catalog of canonical app classes") and the compressive beauty dimension (substance-first priority #1).

**Scope:** Five COMP items — synthetic regression catalog, gestiscilo migration Slice A, agent-success-rate harness, time-to-working-app benchmark, cross-modality vocabulary sketch. Validation and measurement work against ferro's own projection/intent system; no new published crates; no changes to the seven-intent vocabulary (`ferro-projections/src/intent.rs`).

**Honesty requirement (applies to every phase):** Validation must be able to fail and surface real weaknesses. Every phase names an adversarial fixture and includes a "discovered weaknesses" section in its verification. A phase that finds nothing wrong is a red flag, not a success.

**Phase calibration notes (resolve at phase-time, not roadmap-time):**
- Phase 209 entity selection: read gestiscilo `src/models/` and `src/controllers/` to identify the three most representative Browse, Process, and Summarize candidates with direct `JsonUi::render_file` calls.
- Phase 210 success-rate floor: set after a first baseline run — threshold must flag genuine regression without being fragile to LLM variance.
- Phase 211 wall-clock threshold: decide after a first cold-cache run whether to assert in CI or keep as a manual-only artifact.

#### Phases

- [x] **Phase 207: COMP-02 — Synthetic Regression Catalog** — `ferro-projections/tests/catalog.rs`; 7 canonical `ServiceDef` builders; structural-invariant assertions (not byte snapshots); `proptest` invariants; adversarial fixture per intent; `insta` snapshots only for named canonical shapes. (completed 2026-06-12)
- [x] **Phase 208: COMP-05 — Cross-Modality Vocabulary Sketch** — Three `pub(crate)` sketch renderers in `ferro-projections/src/render/`; written analysis covering all 7 intents across 3 non-visual modalities; at least one vocabulary gap identified; zero changes to `intent.rs` or `derive.rs`. (completed 2026-06-12)
- [x] **Phase 209: COMP-01 Slice A — Gestiscilo Migration (Browse + Process + Summarize)** — VALIDATED 2026-06-12. Goal (first real-world validation signal) achieved decisively; SC#2/#4/#5 met. SC#1 (migrate+merge 3 entities) deliberately NOT met — the validation found the projection render is content-incomplete (Process placeholder kanban, Summarize empty values, actions deferred), so no entity reached merge-worthy parity; both probe branches left unmerged. Browse (Staff) is data-bound and works. Finding → v13.2 Phase 213. Depends on Phase 207.
- [x] **Phase 210: COMP-03 — Agent-Success-Rate Harness** — `ferro-mcp/tests/agent_harness.rs`; 14+ tasks (2 per intent); 4-tier pass criteria defined before any agent run; ≥3 trials per case; `rmcp 0.12` in-process transport; committed baseline artifact (model version, prompt version, per-tier pass rates). Depends on Phase 207. (completed 2026-06-13)
- [x] **Phase 211: COMP-04 — Time-to-Working-App Benchmark** — `ferro-cli/tests/benchmark_new_project.rs`; criterion 0.8.2 `iter_custom` scaffold timing; `FERRO_BENCH=1` gate; at least one cold-cache Docker run; committed Markdown result document with start/end conditions and per-step breakdown. (completed 2026-06-13)

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 207. COMP-02 Synthetic Regression Catalog | 1/1 | Complete    | 2026-06-12 |
| 208. COMP-05 Cross-Modality Vocabulary Sketch | 2/2 | Complete    | 2026-06-12 |
| 209. COMP-01 Slice A Gestiscilo Migration | 1/2 | In Progress|  |
| 210. COMP-03 Agent-Success-Rate Harness | 4/4 | Complete    | 2026-06-13 |
| 211. COMP-04 Time-to-Working-App Benchmark | 2/2 | Complete    | 2026-06-13 |

#### Phase Details

### Phase 207: COMP-02 — Synthetic Regression Catalog

**Goal:** A permanent, machine-checkable baseline asserts that `derive_intents()` produces the correct primary intent for each canonical app class. The catalog is the regression foundation for every future change to `ferro-projections/src/derive.rs` and `intent.rs`, and it is the ground-truth source that Phase 210 (agent harness) consumes.

**Depends on:** Nothing (first phase, no prerequisites).

**Requirements:** COMP-02

**Success Criteria** (what must be TRUE):
  1. Seven canonical `ServiceDef` builder functions exist in `ferro-projections/tests/catalog.rs`, one per structural intent (Browse / Focus / Collect / Process / Summarize / Analyze / Track); each has a test asserting `derive_intents(&service)[0].intent == ExpectedIntent` plus a confidence threshold.
  2. Structural-invariant assertions outnumber `insta` snapshot assertions; at minimum one test per intent asserts a structural property of the rendered output (e.g. Browse produces a spec whose root element resolves to a table shape) — no test is satisfied by an empty or minimal `ServiceDef` producing empty output.
  3. At least one fixture per intent exercises a non-trivial case (many fields, multiple actions, state machine, or competing signals), and at least one fixture is explicitly adversarial — designed to probe a known edge of the derivation logic (e.g. a fixture with competing Browse/Summarize signals that should resolve to Summarize) and documented as such in a comment.
  4. All seven catalog tests pass under `cargo test --all-features` and are integrated into the standard CI gate (no `#[ignore]`); a future change to `derive.rs` that breaks intent derivation for any canonical class causes a named, legible CI failure.
  5. A "discovered weaknesses" note in the phase verification names at least one real limitation surfaced by writing the catalog (e.g. a canonical class where derivation confidence is lower than expected, or a signal gap). An empty weaknesses section fails the phase close.

**Plans:** 1/1 plans complete
- [x] 207-01-PLAN.md — Synthetic regression catalog: 7 canonical fixtures + per-intent identity/floor/margin tests, 4 adversarial confusable-pair fixtures, proptest engine invariants, insta snapshots (signals only), calibration, discovered-weaknesses note

### Phase 208: COMP-05 — Cross-Modality Vocabulary Sketch

**Goal:** Determine whether the seven-intent vocabulary is sufficient for non-visual rendering modalities before v14.0 Channel Projection begins. The deliverable is a document and three `pub(crate)` sketch renderers — not a shipped feature, not a vocabulary change, not a production API.

**Depends on:** Nothing (independent, unblocked immediately).

**Requirements:** COMP-05

**Success Criteria** (what must be TRUE):
  1. Three sketch renderers (`CliSummaryRenderer`, `VoiceRenderer`, `MobileCardRenderer`) exist as `pub(crate)` modules in `ferro-projections/src/render/`; each implements the `Renderer` trait with a non-trivial output (not empty strings); all are marked with a `// Research sketch — not stable API` comment.
  2. `ferro-projections/src/intent.rs` and `ferro-projections/src/derive.rs` are unchanged: `grep -n` of both files before and after the phase produces identical line counts for all intent-vocabulary symbols (`Browse`, `Focus`, `Collect`, `Process`, `Summarize`, `Analyze`, `Track`). Any vocabulary change triggered by the sketch is filed as a named v14.0 proposal, not implemented here.
  3. The analysis document (a module-level doc block or a file in `docs/`) covers all seven intents across the three non-visual modalities, and names at least one vocabulary tension — a case where the intent boundary is unclear or insufficient for non-visual rendering.
  4. The document includes a "v14.0 implications" section listing specific open questions for Channel Projection scope (e.g. whether `BaseContext` needs a `device_class` field, whether `Track` maps cleanly to voice).
  5. A "discovered weaknesses" note names at least one tension found: a place where the current intent vocabulary requires a workaround or an awkward output to satisfy the sketch. An empty section fails the phase close.

**Plans:** 2/2 plans complete
- [x] 208-01-PLAN.md — Three pub(crate) sketch renderers (CliSummary/Voice/MobileCard) against the shared Process anchor fixture + smoke tests
- [x] 208-02-PLAN.md — Cross-modality analysis document (7x3 matrix, vocabulary tension, v14.0 implications, discovered weaknesses) + intent.rs/derive.rs byte-freeze verification

### Phase 209: COMP-01 Slice A — Gestiscilo Migration (Browse + Process + Summarize)

**Goal:** Deliver the first real-world validation signal for the projection/intent abstraction by migrating three gestiscilo views (one Browse, one Process, one Summarize) to `ServiceDef` + `JsonUiRenderer` and recording render-equivalence evidence. The migration is one-per-merge to gestiscilo master with no long-lived branch; a single ferro publish at slice end only if a discovered gap forces a fix.

**Repo split (validation-only ferro phase):** This ferro phase is **validation-only** — it owns the ferro-side intent-assertion tests (`ferro-projections/tests/catalog.rs`), the render-equivalence records, the weakness note, and the publish decision. The actual view migration (controller swaps, the `projections` feature flag, branches, merges, server, screenshots) is a **gestiscilo-repo phase**, executed in a gestiscilo GSD session from `GESTISCILO-MIGRATION-BRIEF.md`. Ferro 209 does not modify any gestiscilo file (CONTEXT D-09). The success criteria below remain the COMP-01 contract; SC#1 and the branch-discipline half of SC#3 are *executed* by the gestiscilo phase and *evidenced* by ferro 209.

**Depends on:** Phase 207 (catalog baseline establishes the verified intent vocabulary these migrations compare against). External: the gestiscilo migration phase (`GESTISCILO-MIGRATION-BRIEF.md`) must merge the three entities before ferro 209 Plan 02 records its evidence.

**Requirements:** COMP-01

**Success Criteria** (what must be TRUE):
  1. Three gestiscilo entities (one Browse, one Process, one Summarize) are migrated to `ServiceDef` + `JsonUiRenderer`; the old `JsonUi::render_file` call for each is deleted; each migration is committed and merged to gestiscilo master as a separate slice before the next begins.
  2. A render-equivalence document per migrated entity shows side-by-side screenshots or HTML diffs of the before and after views, confirming the projection-driven view is functionally equivalent for the primary use case.
  3. No open branch against gestiscilo has been alive for more than two weeks at any point in the slice series; no ferro API changes are made on master while a gestiscilo migration branch is open.
  4. A single ferro version is published at the end of the slice series (not mid-series); the ferro version published is the same version all three slices were migrated against.
  5. A "what the migration revealed" section in the phase verification names at least one real abstraction gap or friction point surfaced by working against a production codebase (e.g. a `ServiceDef` field that had no clean mapping, a renderer output that required a workaround). An empty section fails the phase close.

**Phase-time calibration:** Entity selection is RESOLVED (RESEARCH §1): Staff list → Browse, Orders kanban → Process, Statistics dashboard → Summarize. Criteria and backups in RESEARCH §1; full migration spec in `GESTISCILO-MIGRATION-BRIEF.md`.

**Plans:** 2/2 executed
- [x] 209-01-PLAN.md — ferro abstraction proof: three `derive_intents()` intent-assertion fixtures (staff/order/stats) in `ferro-projections/tests/catalog.rs`; three EQUIV stubs. DONE (tests green).
- [x] 209-02-PLAN.md — evidence + sign-off: EQUIV records filled from the Orders + Staff probe migrations + Stats source-assessment (SC#2); WEAKNESS-NOTE.md (SC#5, Gaps A–E); PUBLISH-DECISION.md (Path A, no publish, SC#4/D-06). DONE.

**Outcome (VALIDATED):** The projection abstraction derives intent and selects layout correctly for all three intents, but the render is **content-incomplete** at 0.2.54: Browse data-binds (Staff table works — `screenshots/after-staff-208-rows.png`), Process emits a placeholder kanban (Orders — `screenshots/after-orders-207.png`), Summarize emits empty StatCard values (Stats, source-confirmed), and the actions slot is deferred for every intent. Root cause in `ferro-json-ui/src/projection/builder.rs`. No entity reached merge-worthy parity → migrations blocked, both probe branches left unmerged. Follow-up: **v13.2 Phase 213 (Projection Render Completeness)**.

**Gestiscilo migration (separate, in the gestiscilo repo):** roadmap phases 207–209 added there; spec is `GESTISCILO-MIGRATION-BRIEF.md`. Orders (207) + Staff (208) probed on branches `feat/207`/`feat/208`; both blocked on the ferro render gap above and preserved unmerged for re-verification after Phase 213.
**UI hint**: yes

### Phase 210: COMP-03 — Agent-Success-Rate Harness

**Goal:** Measure whether an agent reading `ferro-mcp` introspection can produce a working projection from a natural-language description. The harness design — 4-tier criteria, ≥3 trials per case, committed baseline — is the substantive deliverable; the agent runs follow.

**Depends on:** Phase 207 (catalog provides ground-truth intent per domain class, which the harness uses for tier-2 intent-coverage checks and to construct the 14 task descriptions).

**Requirements:** COMP-03

**Success Criteria** (what must be TRUE):
  1. The harness exists at `ferro-mcp/tests/agent_harness.rs`; it drives `ferro-mcp` developer introspection tools (`list_projections`, `generate_projection`, `checkpoint_projection`) via an in-process `rmcp 0.12` `tokio::io::duplex` transport (the proven pattern from `ferro-api-mcp/tests/e2e.rs`) — not `ferro-mcp-server`, not a subprocess, not a new rmcp version.
  2. Four-tier pass criteria are defined in the harness source code **before any agent run is collected**: (1) structural validity — `ServiceDef` compiles and passes `validate_projection`; (2) intent coverage — primary derived intent matches the NL description's expected intent from the COMP-02 catalog; (3) functional completeness — named actions and guards in the description are present as `ActionDef`/`GuardDef` entries; (4) checkpoint pass — `checkpoint_projection` returns `pass` or `warn`. Each tier is reported separately; no tier is collapsed into a boolean aggregate.
  3. The corpus spans all seven intents with at least 14 task descriptions (2 per intent); descriptions use generic domain language matching COMP-02 catalog classes (e.g. "a product catalog with name, price, and category") — no gestiscilo-specific descriptions, no descriptions copied verbatim from MCP tool documentation examples.
  4. Each task runs ≥3 trials; a committed baseline artifact (model version, prompt version, per-tier pass rates per task) is checked into the repository alongside the harness code; all harness tests are marked `#[ignore]` in default CI with a comment explaining that they require a live LLM API key.
  5. A "discovered weaknesses" section in the phase verification names at least one real finding: a tier or task where pass rates are lower than expected, or a structural pattern the agent consistently gets wrong. An empty section fails the phase close.

**Phase-time calibration:** Success-rate floor threshold (e.g. `assert!(rate >= 0.7)`) is set after a first baseline run, not now. The tier-2 and tier-3 floor thresholds may differ.

**Plans:** 4/4 plans complete
- [x] 210-01-PLAN.md — Foundation: dev-dep delta + 14-task contamination-guarded corpus + contamination test
- [x] 210-02-PLAN.md — Deterministic T1–T4 scorer + replay path (CI-green, no LLM; pitfall mitigations)
- [x] 210-03-PLAN.md — In-process rmcp duplex transport + gated complete_with_tools agent loop
- [x] 210-04-PLAN.md — First committed baseline (gated live run, manual) + replay-equals-baseline + SC#5 weakness finding

### Phase 211: COMP-04 — Time-to-Working-App Benchmark

**Goal:** Measure `cargo new` → a running service with auth, three entity types, and one background job — producing a committed result document with full environment specification. The cold-cache run is the honest "first-time experience" number; the benchmark apparatus is the permanent artifact.

**Depends on:** Nothing (independent of catalog; benefits from Phase 210 having exercised the agent-assisted path but does not depend on it).

**Requirements:** COMP-04

**Success Criteria** (what must be TRUE):
  1. A benchmark scaffold exists at `ferro-cli/tests/benchmark_new_project.rs` using criterion 0.8.2 `iter_custom` (`default-features = false, features = ["cargo_bench_support"]`); the benchmark is gated behind a `FERRO_BENCH=1` env var check so it is skipped in default CI and does not create a second target directory on the standard CI disk budget.
  2. The benchmark measures five steps: `ferro new <tmpdir>` → `ferro make:auth` → `ferro make:model <X>` × 3 → `ferro make:job <Y>` → `cargo build` in tmpdir; each step's wall-clock time is recorded individually; the scaffold compile step asserts exit code 0.
  3. At least one cold-cache Docker run is executed and its result is committed: a clean container with no pre-installed Rust toolchain and no Cargo cache; the cold-cache time is the number reported in any external documentation.
  4. A committed Markdown result document specifies: Rust toolchain version, `cargo` cache state (cold/warm), host machine class, agent-assistance level (manual commands vs agent-driven), per-step wall-clock breakdown, and total time. A result without an environment specification is not accepted.
  5. A "discovered weaknesses" section in the phase verification names at least one real finding: a step that was slower than expected, an unhappy path that was not measured, or a CI-gate decision with a rationale. An empty section fails the phase close.

**Phase-time calibration:** Whether to assert a wall-clock threshold in CI (and at what value) is decided after a first cold-cache run. If CI disk constraints make it infeasible, the benchmark remains a committed manual artifact.

**Plans:** 2/2 plans complete
- [x] 211-01-PLAN.md — Benchmark apparatus (autonomous): gated criterion `iter_custom` benchmark (5 steps, per-step Instants, build asserts exit 0), criterion dev-dep, cold-cache Dockerfile + RESULTS.md template with seeded weakness
- [x] 211-02-PLAN.md — Cold-cache run (human-action): developer runs the Docker benchmark, fills RESULTS.md with the real `cache: cold` row + full env spec, finalizes the SC#5 discovered-weaknesses finding

---

## ✅ v13.1 CRUD Handler Proc Macros (Phase 212, complete 2026-06-13)

**Milestone Goal:** Eliminate the recurring "GET form" + "POST handler" CRUD boilerplate that ferro consumers write today as 5–15 lines per handler, by shipping two route-attribute proc macros plus a validator helper. This is a framework-ergonomics feature on ferro's framework-product axis — not a Compressive Validation item — which is why it sits in its own milestone rather than inside v13.0.

**Source:** The gestiscilo Phase 202 duplication survey (`gestiscilo-it/app/.planning/phases/202-adopt-ferro-crud-macros/202-EVIDENCE.md`) — 244 `resolve_tenant()` calls, 55+ tenant-scoped lookups, 129 form-error redirects across one consumer's controllers. The existing `param_as` / `into_action_error` APIs close ~75% of the boilerplate; the macros close the remaining tenant-resolve + lookup + 404-dispatch prelude.

**Provenance note:** This phase was originally mis-numbered 209 by the cross-repo gestiscilo evidence pass, colliding with ferro ROADMAP's 209 (COMP-01 Gestiscilo Migration). Resolved 2026-06-12 by relocating it to a dedicated milestone (v13.1, Phase 212); 209 remains COMP-01.

#### Phases

- [x] **Phase 212: CRUD Handler Proc Macros** — `#[resource_get]` + `#[resource_post]` proc macros (tenant + typed-param + tenant-scoped-lookup + 404 prelude), `Validator::validate_or_redirect` helper, `TenantResolver` / `TenantScoped` traits, reference fixture, `cargo expand` rustdoc. Seven open design questions to lock in discuss-phase. Paired (optional, post-publish) with gestiscilo Phase 202b adoption. (completed 2026-06-13)

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 212. CRUD Handler Proc Macros | 3/3 | Complete    | 2026-06-13 |

#### Phase Details

### Phase 212: CRUD Handler Proc Macros

**Goal:** Ship `#[resource_get]` and `#[resource_post]` route-attribute proc macros that fold the recurring tenant-scoped CRUD prelude into a macro, plus `Validator::validate_or_redirect(&data, &url)`. Full scope, motivation, consumer evidence, deliverables, and the seven open design questions live in `phases/212-crud-handler-proc-macros/212-CONTEXT.md`.

**Depends on:** Nothing (independent framework feature; consumes existing `param_as` / `into_action_error` / `#[action]` surfaces).

**Requirements:** CRUD-01, CRUD-02, CRUD-03, CRUD-04, CRUD-05, CRUD-06 (defined in 212-CONTEXT.md D-10; established by this phase's plans).

**Status:** Planned — 3 plans across 3 waves. Execute with `/gsd-execute-phase 212`.

**Plans:** 3/3 plans complete
- [x] 212-01-PLAN.md — Foundations: `Validator::validate_or_redirect` (CRUD-03) + `TenantScoped` trait (CRUD-04) + unit tests (Wave 1, no deps)
- [x] 212-02-PLAN.md — Proc macros: `#[resource_get]` + `#[resource_post]` + trybuild harness/fixtures + facade re-exports (CRUD-01, CRUD-02, CRUD-05) (Wave 2, depends_on 01)
- [x] 212-03-PLAN.md — Reference + release: dual-macro reference fixture + cargo-expand rustdoc + CHANGELOG + version bump 0.2.56 (CRUD-06) (Wave 3, depends_on 02)

## 📋 v13.2 Projection Render Completeness (Phase 213, scoped 2026-06-12)

**Milestone Goal:** Make the projection render content-complete. COMP-01 Slice A (Phase 209) was the first real-world migration of gestiscilo views to `ServiceDef` + `JsonUiRenderer` and it returned a precise, source-confirmed verdict: the projection pipeline is **layout-complete but content-incomplete**. Intent derivation and layout selection work across all seven intents; the gap is one level down, in `ferro-json-ui/src/projection/builder.rs`, where several content emitters are deliberate placeholders. This milestone closes those gaps so that migrating a real view to a projection produces a usable page, not a skeleton. It is the direct unblock for every future projection migration and the prerequisite for resuming gestiscilo Slice A.

**Source:** Phase 209 `phases/209-comp-01-slice-a-gestiscilo-migration/WEAKNESS-NOTE.md` (Gaps A–E), with live evidence (`screenshots/after-orders-207.png` placeholder kanban; `screenshots/after-staff-208-rows.png` working Browse table) and the two unmerged probe branches in the gestiscilo repo (`feat/207-orders-projection-migration`, `feat/208-staff-projection-migration`) preserved for re-verification.

#### Phases

- [x] **Phase 213: Projection Render Completeness** — content binding for the projection builder: state-machine→kanban column derivation + card binding (`emit_kanban_root`), StatCard value binding (`emit_statcard_root`), action-slot wiring from `ServiceDef` actions (`emit_actions_placeholder`), `ImageUrl` column rendering, and an app-shell/layout context. May split into per-gap phases at planning time. Depends on Phase 209. (completed 2026-06-12; **Gap A kanban root fix + integration-verification 2026-06-13** — see 213-06-SUMMARY)

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 213. Projection Render Completeness | 5/5 + Gap A root fix | Complete (integration-verified) | 2026-06-13 |

**Integration re-verify (2026-06-13):** The 213-01..05 work was unit-green but the live gestiscilo re-verify exposed a blank Orders kanban — `KanbanBoardProps` conflated lane structure and card content (`data_path` wholesale-replaced `columns`). Gap A root fix (213-06) split structure (`columns`, always rendered) from content (`items_path` + `group_by` + `card_*`/`row_*`, mirroring `MediaCardGrid`); the renderer buckets a flat array by status. Live feat/207 Orders kanban now renders all 5 lanes with correct counts + cards bucketed by status. feat/208 Staff Browse + Gap D avatar `<img>` confirmed live; Gap B row-actions accepted on unit coverage (staff probe declares no actions — consumer-wiring gap). Both probe branches remain pristine/unmerged.

#### Phase Details

### Phase 213: Projection Render Completeness

**Goal:** Close the content-binding gaps that Phase 209 surfaced so that Process, Summarize, and action-bearing views render usably from a `ServiceDef`, not as placeholders. Today (ferro 0.2.54) only Browse/DataTable is data-bound; the rest of the projection content emitters in `ferro-json-ui/src/projection/builder.rs` are intentional stubs.

**Depends on:** Phase 209 (COMP-01 Slice A — the validation that scoped this; its WEAKNESS-NOTE.md is the requirements source).

**Requirements:** GAP-A, GAP-B, GAP-C, GAP-D, GAP-E (derived from Phase 209 WEAKNESS-NOTE Gaps A–E; one label per gap)

**Success Criteria** (what must be TRUE) — draft, to refine in discuss-phase:
  1. Process render derives kanban columns from the `ServiceDef` state machine and binds card data — `emit_kanban_root` is no longer a single placeholder column. The gestiscilo `feat/207` branch's Orders kanban renders its columns + cards.
  2. Summarize render binds StatCard values to runtime data — `emit_statcard_root` no longer emits `value: String::new()`.
  3. The `actions` slot emits action elements (Button/DropdownMenu) from `ServiceDef` actions — `emit_actions_placeholder` is replaced. The gestiscilo `feat/208` Staff table regains row actions.
  4. `ImageUrl` fields render in a Browse `DataTable` (as an image column), or the exclusion is a documented, intentional contract.
  5. A layout/app-shell context lets a projection render inside surrounding chrome, or a documented composition pattern covers it.
  6. Re-verification: the two preserved gestiscilo probe branches reach functional parity for their primary use case under the matured renderer.

**Provenance:** Scoped from Phase 209 findings. The migration code already exists on the two gestiscilo probe branches; this phase makes the renderer worthy of merging them.

**Plans:** 5/5 plans complete
- [x] 213-01-PLAN.md — Gap B (actions) + Wave 0 fixtures: `emit_actions_placeholder` DropdownMenu + DataTable `row_actions` from `service.actions`
- [x] 213-02-PLAN.md — Gap A (kanban): `emit_kanban_root` derives columns from the state machine + `data_path` binding
- [x] 213-03-PLAN.md — Gap C (statcard): `StatCardProps.value_path` extension + `render_stat_card` resolution + primary-stat emit
- [x] 213-04-PLAN.md — Gap D (imageurl): `ColumnFormat::Image` + ImageUrl column inclusion + `<img>` cell render
- [x] 213-05-PLAN.md — Gap E doc (composition pattern) + full gate + gestiscilo probe-branch re-verification (checkpoints)

## ✅ v13.3 Scaffold↔Library Parity & Published-Artifact Smoke Test (Phase 214, complete 2026-06-13)

**Milestone Goal:** Make a freshly scaffolded ferro app compile against the *published* `ferro` crate, and add a CI guard that keeps it that way. COMP-04 (Phase 211) ran the first cold-cache time-to-working-app benchmark and found the honest first-time experience is broken: the CLI scaffolding steps are sub-second, but `cargo build` of the generated app fails with **52 compile errors** — the published 0.2.55 `ferro-cli` scaffold templates reference APIs the published `ferro` crate does not expose. This is framework-correctness on ferro's framework-product axis, not a Compressive Validation item, which is why it sits in its own milestone.

**Source:** Phase 211 `phases/211-comp-04-time-to-working-app-benchmark/211-WEAKNESSES.md` (Finding W1), with the committed cold-cache apparatus (`ferro-cli/tests/benchmark_new_project.rs`, `ferro-cli/tests/fixtures/benchmark/{Dockerfile,RESULTS.md}`) as both the evidence and the basis for the permanent CI guard. Because the generated `Cargo.toml` pins `ferro = { package = "ferro-rs", version = "0.2" }` from crates.io (not a path dep), scaffolding with the local workspace binary reproduces the failure — the published library, not the scaffolding binary, is the constraint.

#### Phases

- [x] **Phase 214: Scaffold↔Library Parity & Smoke Test** — align the `ferro-cli` scaffold templates with the published `ferro` surface (export/`use` the symbols the templates emit, or change the templates to match what's exported): `error_response!` macro, `#[rule]` validation attribute, `ferro::Queue`/`QueueConfig`, the `make:job` `ferro_queue` dependency (add to generated `Cargo.toml` or re-export under `ferro`), `ActiveValue` import in scaffold controllers, `crate::models::users` resolution, `ferro::database::connection` usage. Then add a CI smoke test that scaffolds + `cargo build`s against the published artifact. May split into a parity-fix plan + a CI-guard plan at planning time. Depends on Phase 211. (completed 2026-06-13)

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 214. Scaffold↔Library Parity & Smoke Test | 2/2 | Complete    | 2026-06-13 |

#### Phase Details

### Phase 214: Scaffold↔Library Parity & Published-Artifact Smoke Test

**Goal:** A freshly scaffolded app (`ferro new → make:auth → make:scaffold ×3 → make:job → cargo build`) compiles clean against the published `ferro` crate, and a CI smoke test enforces this on every release so a non-compiling scaffold can never ship silently again.

**Depends on:** Phase 211 (COMP-04 — the validation that found the defect; its 211-WEAKNESSES.md W1 is the requirements source, and its benchmark apparatus is the smoke-test basis).

**Requirements:** SCAF-01 (templates reference only the published `ferro` surface), SCAF-02 (`make:job` Cargo.toml declares every imported crate — queue via `ferro::queue`, no missing `ferro-queue`), SCAF-03 (clean scaffold `cargo build`s exit 0 against published `ferro-rs`), SCAF-04 (release-time CI gate against the published artifact), SCAF-05 (per-PR scaffold-build guard against the workspace path dep). Derived from 211-WEAKNESSES W1 (CONTEXT D-10).

**Success Criteria** (what must be TRUE) — draft, to refine in discuss-phase:
  1. The scaffold templates and the published `ferro` surface agree: every symbol a generated project references (`error_response!`, `#[rule]`, `ferro::Queue`/`QueueConfig`, `ActiveValue`, `crate::models::users`, `ferro::database::connection`) either resolves from the published crate or the template no longer emits it.
  2. `make:job` produces a project whose `Cargo.toml` declares every crate its generated code imports (`ferro-queue` present, or the job code routes through a `ferro` re-export).
  3. A clean scaffold (`ferro new → make:auth → make:scaffold ×3 → make:job`) `cargo build`s with exit 0 against the published `ferro-rs` — the assertion COMP-04's benchmark makes now passes.
  4. A CI job scaffolds and builds against the published artifact (reusing the Phase 211 apparatus / a published-crate variant) and fails the pipeline on any scaffold↔library regression.
  5. The decision on whether the smoke test runs per-PR vs per-release (cost/time) is recorded with a rationale.

**Provenance:** Scoped from Phase 211 findings. COMP-04's value was catching this; this phase fixes it and makes the guard permanent.

**Status:** Planned — 2 plans across 2 waves. `/gsd-execute-phase 214`.

**Plans:** 2/2 plans complete

Plans:
- [x] 214-01-PLAN.md — Parity fix: `error_response!` macro + `ActiveValue` re-export (framework) + 5 template fixes (queue import, ValidateRules derive, model path, DB connection) + docs (Wave 1). Covers SCAF-01, SCAF-02.
- [x] 214-02-PLAN.md — CI guard: `scaffold_builds_against_workspace_ferro` path-dep test + `ci.yml` scaffold-smoke job + Dockerfile `ARG FERRO_VERSION` + `publish.yml` post-publish release gate (Wave 2, depends_on 01). Covers SCAF-03, SCAF-04, SCAF-05.

---

### ✅ v14.0 Channel Projection — Non-Visual Rendering (Shipped 2026-06-13)

Phases 215–216 (CHAN-01–04) — full details archived in [milestones/v14.0-ROADMAP.md](milestones/v14.0-ROADMAP.md).

Shipped the first production non-visual `Renderer`: `ferro-text::TextRenderer` projects the same `ServiceDef` the visual/MCP renderers consume into conversational text, guard-filtered and verbosity-aware, with a defined Focus/Analyze fallback. Phase 215 extended the renderer-free surface (`BaseContext.evaluated_guards`/`verbosity`, `Intent::label()`, `Error::NoIntents`); Phase 216 added `FieldDef.render_hint` and the `ferro-text` output crate, re-exported via the `ferro` facade behind the `projections` feature.

---

### ✅ v15.0 Agent-Operable App (Consumer MCP) — Shipped 2026-06-14

Phases 217–221 (AMCP-01–06) — full details archived in [milestones/v15.0-ROADMAP.md](milestones/v15.0-ROADMAP.md).

A tenant can operate a live ferro application through a per-tenant MCP endpoint whose tools are projection-derived: per-tenant API-key auth + tenant/guard context (217), write tools rendered from `ActionDef` (218), tenant-scoped write dispatch with server-side guard re-evaluation, idempotency, and audit (219), confirmation gating for destructive actions (220), and the inbound natural-language intent loop — classify → guard → confirm → dispatch — CI-testable without live-LLM spend (221). Extends projection/intent to a fourth `Renderer` target (`ServiceDef → MCP tools`). All work landed in `ferro-mcp-server` with `ferro-ai` behind feature flags; consumer (gestiscilo) adoption is a separate follow-up.

---

## v13.4 Cache-Events Bridge (Phase 222, scoped 2026-06-13)

**Milestone Goal:** Bridge `ferro-cache` and `ferro-events` so consumers can declare "when event `E` fires, flush these cache tags" once at boot — instead of writing per-app `impl Listener<E>` glue that knows about the cache. This is the missing primitive for short-TTL read-through caches on hot endpoints (e.g. availability windows in gestiscilo): the cache machinery already exists in `ferro-cache::TaggedCache`, and the bus already exists in `ferro-events`, but consumers today have to hand-author the bridge listener and remember every write site that mutates cached state.

**Source:** Surfaced 2026-06-13 during gestiscilo availability-perf investigation. The per-request structural fix (gestiscilo Phase 210 prerequisite work — hoisting prefetches out of the per-slot loop) cuts a single request from ~1k DB queries to ~5, but cross-request redundancy remains: every browser open rebuilds the same window. The operator's read — that the deeper issue is caching/architecture — is correct, and the principle that cross-cutting primitives belong in ferro (`feedback_ferro_first_primitives.md` on the gestiscilo side) points at this bridge as the natural ferro surface to add.

### Phases

- [x] **Phase 222: Cache-Events Bridge** — Added `register_invalidator::<E, F>(cache, key_fn)` (+ `register_invalidator_on` dispatcher overload) to `ferro-cache`, subscribing an event-bus listener that flushes the tags returned by `key_fn(&event)`. Tag scheme stays consumer-defined (string keys). Listener failure is best-effort and logged — it does not propagate back to the original event dispatcher. Shipped v0.2.59. (Closure-only surface — the originally-scoped `CacheInvalidator` trait was dropped as unnecessary indirection; see SC#1 note.)

### Phase Details

### Phase 222: Cache-Events Bridge

**Goal:** A consumer can register one line at boot — `register_invalidator::<BookingCreated, _>(cache.clone(), |evt| vec![format!("business:{}:product:{}", evt.business_id, evt.product_id)]).await` — and the bus + cache do the rest. No hand-authored `impl Listener<E>` per-app glue. The cache mechanics + event integration both stay framework-side; consumers stay declarative.

**Depends on:** `ferro-cache` (already shipped: `Cache`, `TaggedCache`, `MemoryStore`, `RedisStore`) and `ferro-events` (already shipped: `Event`, `Listener`, `EventDispatcher`, `dispatch`). No new crates; bridge lands in `ferro-cache` with `ferro-events` added as a non-optional workspace dep (small surface, used by every ferro app that already takes both).

**Success Criteria** (what must be TRUE) — to refine in discuss-phase:
  1. ⚠️ DEVIATION (as-shipped): no `CacheInvalidator` trait. The trait existed in the scope only to be blanket-impl'd by a closure, so the shipped design exposes the closure helper directly — `register_invalidator::<E, F>(cache, key_fn)` and `register_invalidator_on::<E, F>(&dispatcher, cache, key_fn)`, both `F: Fn(&E) -> Vec<String> + Send + Sync + 'static`. Same user-facing capability (map event → tags at boot), less indirection. The named-trait form is not part of the public surface.
  2. Dispatching an event for which an invalidator is registered flushes the returned tags via the existing `TaggedCache::flush` path — verified by a unit test: insert tagged entry → dispatch event → assert `get` returns `None`.
  3. Multiple invalidators can be registered for the same event type — all run; order is unspecified but documented.
  4. Listener failure (e.g. cache store unavailable) is logged and swallowed — it does not propagate back to `EventDispatcher::dispatch`, so a degraded cache cannot brick the original write path.
  5. The bridge respects the existing `ferro-events` queued-listener marker: if `CacheInvalidator` is intentionally synchronous-only (chosen during discuss-phase), this is documented and enforced.
  6. A doc example in `ferro-cache/src/lib.rs` shows the end-to-end pattern (cache construction → invalidator registration → event dispatch → cache miss on next read).
  7. `CHANGELOG.md` entry + workspace version bump (0.2.58 → 0.2.59); the published crate exposes the new surface under a stable path.

**Provenance:** Scoped from gestiscilo 2026-06-13 availability-perf session. Pairs with gestiscilo Phase 210 (consumer-side adoption — mounts a `TaggedCache` on the availability endpoint and uses `register_invalidator` for `BookingCreated`, `BookingCancelled`, and the new `ClosedDayChanged` / `InventoryUnitStatusChanged` gestiscilo-side events).

**Status:** ✅ Shipped (v0.2.59, commits `4d81a596` + `2172c8e0`). Locked decisions (resolved to lean defaults): D-01 `Vec<String>`, D-02 synchronous in-dispatch, D-03 multi-invalidator, D-04 `Fn` closure, D-05 closure-captured `Arc<Cache>`. SC#2–7 met; SC#1 shipped as a closure-only surface (no `CacheInvalidator` trait — see deviation note above). Verified 2026-06-14: `cargo test -p ferro-cache --all-features` → 23 passed (6 invalidator tests). Note: the consumer companion (gestiscilo Phase 210) blocks on the 0.2.59 crates.io publish.

**Plans:** 3/3 plans complete

### Phase 225: Release Workflow rustls Migration and E2E CLI-from-Release Test

**Goal:** The `ferro` release binary and `cargo install ferro-cli` build with no system OpenSSL (no libssl-dev/pkg-config/C-cross), via a workspace-wide native-tls→rustls/ring migration; aarch64-linux builds natively without `cross`; and a from-release e2e gate exercises the real released binary scaffolding a real app against the published `ferro-rs` library (catching the COMP-04 "ships silently broken" class).
**Requirements**: TBD (driven by CONTEXT.md decisions D-01..D-10)
**Depends on:** Phase 224
**Plans:** 3/3 plans complete

Plans:
- [x] 225-01-PLAN.md — Workspace-wide native-tls→rustls/ring migration (18 sea-orm/lettre occurrences + reqwest coherence) + structural verification (D-01, D-02, D-03, D-05)
- [x] 225-02-PLAN.md — release.yml: drop `cross` for aarch64, native cross-linker + ring CC env (D-04)
- [x] 225-03-PLAN.md — release.yml: e2e-tag + e2e-drift jobs (continue-on-error, COMP-04 sequence vs published ferro-rs) (D-06, D-07, D-08, D-09, D-10)

### Phase 226: Homebrew Tap Distribution for ferro-cli

**Goal:** Make `brew install` a first-class way for a new user to get the `ferro` CLI and run `ferro new` — no Rust toolchain, no `curl | sh`, no manual PATH. Stand up an own Homebrew tap (`homebrew-ferro` repo → `brew install albertogferrario/ferro/ferro`) rather than homebrew-core (avoids the pre-1.0 notability/review gate and a likely `ferro` formula-name collision; can graduate to core post-1.0). Ship a binary formula pointing at the GitHub release tarballs already produced by `release.yml` (`ferro-<tag>-<target>.tar.gz`, macOS arm64/x86_64 + Linux) with per-arch sha256 — no user-side compile — and a tag-triggered auto-bump job in `release.yml` that recomputes the SHA256s and updates `Formula/ferro.rb` in the tap so it is never manual toil. Surface `brew install` in the install docs/README.

**Requirements**: D-01..D-06 (CONTEXT decisions; no REQUIREMENTS.md IDs mapped) + operator actions
**Depends on:** Phase 225 (release.yml already builds the per-arch tarballs; the rustls migration removed the openssl/pkg-config build dependency, so even a source fallback is clean)
**Plans:** 4/4 plans complete

**Resolved decisions** (see 226-CONTEXT.md / 226-RESEARCH.md):
- Binary-only formula (4 arches: macOS arm64/x86_64 + Linux x86_64/aarch64); no source fallback (D-02).
- Auto-bump via an IN-REPO SHELL SCRIPT (`scripts/bump-homebrew-formula.sh`), NOT the mislav action — it cannot update multi-arch conditional formulae (D-03).
- Push to the tap via a fine-grained PAT secret `HOMEBREW_TAP_TOKEN` (Contents:write on homebrew-ferro only), direct commit to main (D-04).
- Formula `test do` + tap CI (`brew audit --strict` + `test-bot`), staged in-repo for the operator (D-05).

Plans:
- [x] 226-01-PLAN.md — Seed binary formula template + in-repo bump script (D-02, D-03) [wave 1]
- [x] 226-02-PLAN.md — Wire bump-homebrew-formula job into release.yml + stage tap CI (D-03, D-04, D-05) [wave 2]
- [x] 226-03-PLAN.md — Surface `brew install` in installation docs + README (D-06) [wave 1]
- [x] 226-04-PLAN.md — Operator runbook: create tap repo + PAT secret + live brew install verification (D-01, D-04) [wave 3, non-autonomous]

### Phase 227: Documentation Audit and Update for v0.2.61

**Goal:** Comprehensive sweep of `docs/src/` for accuracy after the v0.2.59→0.2.61 changes. Only the install pages were updated inline (brew install added in Phase 226; MSRV 1.88 + rustls/toolchain-free clarified on 2026-06-14). Audit every other page for stale content: any `runtime-tokio-native-tls`/OpenSSL in config or example snippets (now rustls), references to the old install/getting-started flow, version pins, the scaffold structure (now ships `runtime-tokio-rustls`), and the generators (`make:auth`/`make:scaffold`/`make:job`) + `ferro serve` flow. Verify code/command examples against the live CLI. Surface and fix discrepancies, don't silently work around.

**Requirements**: TBD (capture in discuss-phase)
**Depends on:** Phase 226 (brew/rustls shipped — the facts the docs must now reflect)
**Plans:** 3/3 plans complete

**Scope notes:** focus on factual accuracy, not a rewrite. Known-good already: `docs/src/getting-started/installation.md` (install section). Likely stale candidates: any TLS/sea-orm config examples, version numbers, getting-started walkthrough. CHANGELOG entry deferred (no CHANGELOG system exists; out of scope per CONTEXT.md). READMEs/scripts are Phase 228.

Plans:
- [x] 227-01-PLAN.md — reference/cli.md: brew-first install + db:sync --skip-migrations fix (D-05, DISC-02) [wave 1]
- [x] 227-02-PLAN.md — phantom make:model, stale 0.2.33 pin + broken link, stale MCP config, stale milestone/tool-count (DISC-03..07) [wave 1]
- [x] 227-03-PLAN.md — whole-tree clean-confirmation sweep: TLS/version-pin/phantom-command grep + mdbook build (D-01, D-04) [wave 2]

### Phase 228: README and Scaffold Doc Sweep

**Goal:** Make every README + the generated-app docs consistent and current. Covers: root `README.md` (verify the brew-first install + quickstart match the docs and the real flow); the `albertogferrario/homebrew-ferro` tap repo (add a README describing `brew install albertogferrario/ferro/ferro`, the token-free self-bump, and how it tracks ferro releases); the scaffold's generated `README.md` template (`ferro-cli/src/templates/files/backend/` — ensure it reflects the rustls/SQLite-default app and the `ferro serve` flow); and `scripts/install.sh`/`create-app.sh` user-facing messaging. Ensure the toolchain-free-CLI vs Rust-needed-to-build-app distinction is stated consistently everywhere.

**Requirements**: TBD (capture in discuss-phase)
**Depends on:** Phase 227 (align READMEs to the audited docs)
**Plans:** 1/1 plans complete

Plans:
- [x] 228-01-PLAN.md — README + scaffold-template + installer-script consistency sweep (brew-first, ferro db:migrate/ferro serve, MSRV 1.88+/Node 18+, neutral Status line) + tap-repo README draft

### Phase 229: Framework Benchmark — Harness Foundation (1A): build the reproducible benchmark/ harness (contracts + static-counter + perf-runner + reporting toolbox) and prove it end-to-end on four micro-endpoints in Ferro and a minimal Laravel app, producing the first committed perf + static results. Source spec: docs/superpowers/specs/2026-06-15-ferro-framework-benchmark-design.md; task plan: docs/superpowers/plans/2026-06-15-benchmark-1a-harness-foundation.md.

**Goal:** Build the reproducible `benchmark/` harness (contracts + static-counter + perf-runner + reporting toolbox) and prove it end-to-end on four micro-endpoints (`/json`, `/db`, `/queries`, `/updates`) implemented in Ferro and a minimal Laravel 11 app, producing the first committed perf + static results with recorded hardware.
**Requirements**: none mapped (validation/tooling phase; gated by conformance + pytest units)
**Depends on:** Phase 228
**Plans:** 5/5 plans complete

Plans:
- [x] 229-01-PLAN.md — Scaffold benchmark/ tree + shared micro-endpoints contract (Wave 1, light)
- [x] 229-02-PLAN.md — Harness TDD units: perf parser, static counter, report builder (Wave 1, light)
- [x] 229-03-PLAN.md — Pinned toolbox image (oha+tokei) + perf runner [HEAVY/THERMAL] (Wave 2)
- [x] 229-04-PLAN.md — Ferro + Laravel micro-endpoint apps [HEAVY/THERMAL] (Wave 2)
- [x] 229-05-PLAN.md — Conformance + compose + first results run + README [HEAVY/THERMAL] (Wave 2)

### Phase 230: Framework Benchmark 1B — Ferro Conduit (RealWorld backend): implement the RealWorld/Conduit API spec (JWT auth, users/profiles, articles CRUD, comments, favorites, follows, tags, feeds, pagination) as a Ferro app under benchmark/apps/ferro-conduit, conforming to the published Conduit API contract; vendor a pinned community Laravel RealWorld backend (gothinkster/laravel-realworld-example-app) as the competitor; run the existing harness (static compression + perf via php-fpm/octane) on the real-app workload. Extends Phase 229 harness. Source design: docs/superpowers/specs/2026-06-15-ferro-framework-benchmark-design.md.

**Goal:** A Ferro implementation of the RealWorld/Conduit backend (benchmark/apps/ferro-conduit, OUTSIDE the root workspace) passes the full official RealWorld Newman conformance collection; a vendored, commit-pinned community Laravel Conduit backend passes the same collection (fair like-for-like baseline); and the Phase 229 harness reports the real-app static-compression (with the hand-rolled JWT counted separately and labeled "not framework-provided") and perf (Ferro vs php-fpm vs octane on shared Postgres) with honest caveats.
**Requirements**: none mapped (benchmark phase — requirements: [] intentional)
**Depends on:** Phase 229
**Plans:** 7/7 plans complete

Plans:
- [x] 230-01-PLAN.md — Scaffold the isolated ferro-conduit app + hand-rolled JWT module + auth middleware (unit-tested)
- [x] 230-02-PLAN.md — Models + migrations + relations (users/articles/comments/tags + follows/favorites/article_tags junctions)
- [x] 230-03-PLAN.md — Vendor Newman collection + DTOs + auth endpoints + route-ordering test → Newman Auth green
- [x] 230-04-PLAN.md — Articles CRUD + slugs + list/filter/pagination (feed-first ordering) → Newman Articles green
- [x] 230-05-PLAN.md — Profiles + follow/unfollow → Newman Profiles green
- [x] 230-06-PLAN.md — Comments + favorites + tags + real feed → remaining Newman folders + full single-app green
- [x] 230-07-PLAN.md — Vendor + pin Laravel; full Newman against BOTH; harness static (JWT separate) + perf; honest RESULTS

---

## v13.5 Cache Invalidation Completeness (Phases 223–224, scoped 2026-06-13)

**Milestone Goal:** Round out the cache-invalidation primitive shipped in Phase 222 with the two pieces deliberately deferred from v1: cross-replica fanout (Phase 223) and operator-grade observability (Phase 224). Both are real gaps for production consumers running multi-replica deploys or wanting SLO dashboards on cache hit-rate / invalidation-rate — neither blocks single-process consumers like gestiscilo today, which is why they live in their own milestone rather than re-opening Phase 222.

**Source:** Scoped 2026-06-13 during the Phase 222 / gestiscilo Phase 210 discussion. The honest framing of Phase 222 ("v1 framework primitive — works for the stated use case, has bounded gaps") names exactly these two as the deferred work. Capturing them as named, plannable phases so they cannot be silently forgotten.

### Phases

- [ ] **Phase 223: Redis Pub/Sub Cross-Replica Invalidation Channel** — Make `register_invalidator` (and `register_invalidator_on`) work across replicas of a multi-instance deploy. Today the invalidation fires only on the dispatching instance; with a Redis-backed `CacheStore`, a write on replica A flushes A's local view of the tagged keys but not B's. Add a pub/sub channel + receiver loop so every instance reacts to every published invalidation.

- [ ] **Phase 224: Cache Invalidator Metrics + Introspection** — Operators today see `tracing::warn!` on failure and nothing else. Add a metrics surface (counters: invalidations_fired, invalidations_failed, tags_flushed; histograms: time_to_flush) and an introspection API (list registered invalidators per event type, last-fire timestamp). Cross-references the same `tracing` subscriber chain so it composes with existing observability without forcing a metrics-crate dependency on every consumer.

### Phase Details

### Phase 223: Redis Pub/Sub Cross-Replica Invalidation Channel

**Goal:** A `BookingCreated` dispatched on replica A flushes the matching cache tag on replicas A, B, C, … so a stale read on replica B is impossible. Today the bridge is single-process: the listener registered via `register_invalidator` runs on the dispatching replica only; other replicas' `MemoryStore` (or per-replica `RedisStore` view) never gets the flush signal.

**Depends on:** Phase 222 (the registration surface) + `ferro-cache` already supporting `RedisStore`.

**Success Criteria** (what must be TRUE) — draft, to refine in discuss-phase:
  1. A new `register_invalidator` variant — or an opt-in flag on the existing one — publishes the tag set to a Redis pub/sub channel (e.g. `ferro-cache:invalidations`) instead of (or in addition to) the local `cache.tags(...).flush()` call.
  2. A background receiver loop on every replica subscribes to the channel and runs the local flush when a payload arrives. Loop survives Redis disconnects (reconnect with exponential backoff; no crash on transient outage).
  3. Pub/sub payload schema is documented (JSON: `{ tags: [..], origin: "replica-id" }`); origin field lets a replica skip flushing its own publish (it already flushed locally).
  4. The pub/sub path is opt-in. Single-process consumers (gestiscilo today) keep the Phase 222 local-flush behaviour with zero config. Multi-replica consumers wire one extra line at boot.
  5. Integration test: two `Cache` instances backed by the same Redis instance + the pub/sub channel; an invalidation on instance 1 evicts entries on instance 2 within (configurable) bounded latency.
  6. Failure isolation: receiver loop failures (deserialization error, channel disconnect) are logged and do not propagate to the cache's data-plane reads/writes.

**Provenance:** Named gap in Phase 222 honest-framing review. Operator-aware deferral; consumer phases that need it (e.g. gestiscilo when multi-replica) call this out as a dependency.

**Status:** Not started — pending consumer demand. Re-open when a multi-replica deploy is on the table.

**Plans:** TBD.

### Phase 224: Cache Invalidator Metrics + Introspection

**Goal:** An operator can answer "how many invalidations fired in the last hour for `BookingCreated`?" and "what invalidators are registered for `OrderCreated`?" without reading source. Today Phase 222 emits `tracing::warn!` on per-tag flush failure and nothing else — no counts, no timings, no registry visibility.

**Depends on:** Phase 222.

**Success Criteria** (what must be TRUE) — draft, to refine in discuss-phase:
  1. `register_invalidator` (and the `_on` overload) emit `tracing` events at `info!` level on every fire with structured fields: `event_name`, `tags_flushed`, `duration_us`. Failures emit `warn!` with `error` field (preserves the Phase 222 behaviour as a special case).
  2. An optional `metrics` feature flag wires the same counts/timings into the `metrics` crate (counters: `ferro_cache.invalidations.fired`, `ferro_cache.invalidations.failed`; histogram: `ferro_cache.invalidations.duration`) so operators using a Prometheus/OTLP exporter get them for free.
  3. An introspection API (`ferro_cache::list_invalidators_for::<E>()` or similar) returns the count + last-fire timestamp of registered invalidators per event type. Counts only — not closures (no way to introspect a `Fn` closure body in Rust without runtime reflection).
  4. Consumers that do not enable the `metrics` feature flag still get full `tracing` visibility — i.e. the metrics dependency is fully opt-in, not transitively required.
  5. The introspection API is read-only and lock-cheap (one read on the dispatcher's internal `RwLock<HashMap<TypeId, …>>`).

**Provenance:** Named gap in Phase 222 honest-framing review. Operator-aware deferral.

**Status:** Not started — pending consumer demand. Re-open when an operator asks for SLO dashboards on cache behaviour.

**Plans:** TBD.

## v16.1 ferro-payments — Polymorphic Billable Layer (Phases 233–236) [CONSUMER-PAIRED with gestiscilo Phases 218–223]

**Source spec:** `docs/superpowers/specs/2026-06-17-ferro-payments-crate-design.md`.
**Consumer companion:** `gestiscilo-it/app:docs/superpowers/specs/2026-06-17-tenant-booking-upfront-payment-design.md`.

Four ferro phases that together ship a new workspace crate `ferro-payments`,
providing a polymorphic `PaymentIntent` entity and `Billable` trait so consumer
apps can take Stripe payments for any first-class entity without re-implementing
the wiring. Reuses the existing `ferro-stripe::SyncDispatcher`,
`ProcessedEventLog`, `CheckoutBuilder`, and Connect destination-charge support —
no new ferro-stripe surface required.

First consumer: gestiscilo Phases 218–223 (tenant booking upfront payment),
blocked on Phase 236 publishing `ferro-payments 0.1.0` alongside a ferro version
bump.

### Phase 233: crate scaffold + PaymentIntent entity + migration

**Goal:** Create new workspace member `ferro-payments` parallel to `ferro-stripe`.
Implement `BillableKind`, `PaymentIntentStatus` enums, the SeaORM `Entity` for
`payment_intents`, and the lifecycle methods on the model (`create_reserved`,
`mark_paid`, `mark_released`, `mark_refunded`, `find_active_for`,
`find_by_stripe_session`). Ship migration
`m20260617_create_payment_intents` portable across Postgres + SQLite + MySQL with
partial unique index `(billable_kind, billable_id) WHERE status IN ('reserved',
'paid')` and the supporting indexes. Unit tests cover state transitions and
partial-unique enforcement against in-memory SQLite. No service layer yet.

**Requirements**: PAY-POLY-DM-01..04.

**Depends on:** Phase 232.

**Plans:** 3/3 plans complete

Plans:
- [x] 233-01-PLAN.md — Crate scaffold: Cargo.toml + workspace members + publish.yml Wave 1b + lib.rs/error.rs + PaymentIntentStatus enum (PAY-POLY-DM-02)
- [x] 233-02-PLAN.md — PaymentIntent entity + cross-backend migration with partial unique index + supporting indexes (PAY-POLY-DM-01, PAY-POLY-DM-04)
- [x] 233-03-PLAN.md — Lifecycle methods (create_reserved / mark_* / find_*) via GuardedUpdate no-op semantics + tests (PAY-POLY-DM-03)

### Phase 234: Billable trait + Loader + PaymentService core

**Goal:** Implement the `Billable` trait and `BillableLoader` trait per the spec.
Implement `PaymentService<L: BillableLoader>` with `start_checkout` (mints a
Stripe Checkout session via `ferro_stripe::CheckoutBuilder`, snapshots
`application_fee_cents`, attaches the session id) and `request_refund` (calls
Stripe refund API, snapshots `refund_amount_cents`). Implement `PaymentError`
enum (extended with `Stripe`/`Loader`/`AutoRefundTriggered`). Stripe is abstracted
behind a local `StripeGateway` trait seam (ferro-stripe exposes no injectable
client — it is a static facade), so unit tests inject a `MockStripeGateway` and
a mocked `BillableLoader` with no `Stripe::init`. No webhook integration yet —
that's Phase 235.

**Requirements**: PAY-POLY-SVC-01..05.

**Depends on:** Phase 233.

**Plans:** 3/3 plans complete

Plans:
- [x] 234-01-PLAN.md — ferro-stripe dep + extended PaymentError/AutoRefundReason + publish.yml Wave 1c
- [x] 234-02-PLAN.md — Billable + BillableLoader traits + lifecycle::attach_session
- [x] 234-03-PLAN.md — StripeGateway seam + PaymentService (start_checkout/request_refund) + unit tests + lib re-exports

### Phase 235: webhook SyncDispatcher integration + auto-refund fallback

**Goal:** Implement `wire_dispatcher` helper that registers three typed-event
handlers (`OnCheckoutCompleted`, `OnCheckoutExpired`, `OnChargeRefunded`) on the
caller's `SyncDispatcher`. Implement `PaymentService::handle_session_completed`
/ `handle_session_expired` / `handle_charge_refunded` with idempotency via
`ProcessedEventLog`, transactional dispatch to `Billable::on_paid` /
`on_released` / `on_refunded`, and auto-refund fallback for the "loader returns
None" and "billable already in side state" cases. Race-condition tests: webhook
+ reaper interleaved, webhook replay, loader-not-found.

**Requirements**: PAY-POLY-WH-01..06.

**Depends on:** Phase 234.

**Plans:** 5/5 plans complete

Plans:
- [x] 235-01-PLAN.md — BillableKind → Cow<'static, str> + from_string (WR-04 fix-first)
- [x] 235-02-PLAN.md — ferro-stripe refund::create_for_payment_intent primitive (D-08)
- [x] 235-03-PLAN.md — lifecycle find_by_payment_intent / find_by_charge_id / attach_payment_intent
- [x] 235-04-PLAN.md — StripeGateway pi-refund method + PaymentService processed_log + WR-03 guard
- [x] 235-05-PLAN.md — webhook.rs wire_dispatcher + 3 handlers + auto-refund + 12 race/replay tests

### Phase 236: reapers + workspace test bin + publish 0.1.0

**Goal:** Implement `ReleaseExpiredPaymentIntents` (single SQL pass over
`payment_intents WHERE status = 'reserved' AND expires_at < now()`, dispatches
`on_released` per row in a transaction) and `ReconcileRefundsInFlight` (polls
Stripe for intents in `refund_requested` state > 1 hour). Both as
ferro-queue–compatible job structs that consumers schedule via cron expression.
Add a tiny example `Billable` in a workspace test bin to drive end-to-end against
ferro-stripe test mode. Version-bump ferro workspace + publish `ferro-payments
0.1.0` to crates.io. After publication: gestiscilo Phase 218 plan can be
written.

**Requirements**: PAY-POLY-REAP-01..04.

**Depends on:** Phase 235.

**Plans:** 7/7 plans complete

Plans:
- [x] 236-01-PLAN.md — find_expired + find_refunds_in_flight lifecycle finders (wave 1)
- [x] 236-02-PLAN.md — ferro-stripe refund poll primitive + RefundStatus + gateway/mock (wave 1)
- [x] 236-03-PLAN.md — release_expired + reconcile_refunds_in_flight PaymentService methods (wave 2)
- [x] 236-04-PLAN.md — ferro-queue Job structs (ReleaseExpired/ReconcileRefunds) + wiring (wave 3)
- [x] 236-05-PLAN.md — #[ignore]-gated end-to-end integration test + example Billable (wave 4)
- [x] 236-06-PLAN.md — docs/src/features/payments.md consumer + recovery docs (wave 4)
- [x] 236-07-PLAN.md — git rebase + version bump + ferro-payments 0.1.0 publish (wave 5, operator-gated)

---

## ✅ v16.0 Write-Boundary AX — StateMachine-Derived Executor — Shipped 2026-06-16

Phases 231–232 (EXEC-01..05) — full details archived in [milestones/v16.0-ROADMAP.md](milestones/v16.0-ROADMAP.md).

Delivered: the projection/intent write path now derives transitions from the `StateMachine` declaration across both the MCP and visual write surfaces through one `framework::write` kernel — "declare twice" eliminated, no per-channel executor.

---

## v16.1 ferro-json-ui ActionGroup Action Primitive (Phase 237)

The dashboard kebab/action pattern becomes a first-class component. `ActionGroup`
replaces `DropdownMenu` as the sole public action primitive, enforcing the
"primary first / destructive in the kebab / kebab last / ≤N inline buttons"
conventions structurally rather than by author discipline. Research seed:
[research/actiongroup-component.md](research/actiongroup-component.md).

### Phase 237: ActionGroup component + DropdownMenu replacement + 0.2.72 release

**Goal:** ferro-json-ui exposes an `ActionGroup` component that takes one ordered
action list and renders inline buttons + a trailing overflow kebab — forcing
destructive items into the kebab (rendered last), capping inline buttons at
`max_inline`, auto-wrapping non-GET inline buttons in `<form>`, and accepting
`items` as a literal array or `{"$data":"/path"}` binding with `{row_key}` /
`visible_if` row semantics. `DropdownMenu` is removed from the public surface (its
kebab rendering may survive as an internal helper ActionGroup calls).

**Depends on:** none — json-ui primitive, independent of the payments / write-boundary lines.

**Replaces:** the public `DropdownMenu` component. Per "delete old code completely":
once internal usages migrate, no consumer-authored `DropdownMenu` spec remains.

**Success Criteria** (what must be TRUE):
  1. A spec authoring a single `ActionGroup` with N items renders inline buttons up to `max_inline` (default 2) plus a trailing kebab holding the overflow; any `destructive: true` item appears in the kebab and last, regardless of input order.
  2. `items` bound via `{"$data":"/x/actions"}` renders identically to a literal list; `{row_key}` substitution and `visible_if` row gates work in DataTable/Kanban contexts (parity with the former DropdownMenu).
  3. A non-GET inline action renders inside a `<form>` (no bare POST button); a GET action renders as a plain link/button.
  4. `DropdownMenu` no longer appears in `BUILTIN_TYPES`, `BUILTIN_SPECS`, the public `lib.rs` export, or the catalog; both drift guards (catalog.rs runtime length check + `builtin_types_count_drift_guard`) pass with updated counts; the json-ui schema export lists `ActionGroup` and omits `DropdownMenu`.
  5. Projection codegen `emit_actions_placeholder` emits an `ActionGroup` element; ferro-internal/example/test specs and json-ui docs no longer reference `DropdownMenu`.
  6. The ferro workspace is version-bumped `0.2.71 → 0.2.72` and `ferro-json-ui` (+ `ferro-rs` re-export) is published to crates.io (operator-gated publish step).

**Plans:** 3/4 plans executed
- [x] 237-01-PLAN.md — ActionItem + ActionGroupProps structs + render_action_group (partition / overflow kebab / destructive-last / form-wrap non-GET / visible_if) + tests (Wave 1)
- [x] 237-02-PLAN.md — Atomic registration swap: add ActionGroup + remove public DropdownMenu (BUILTIN_TYPES / dispatch / BUILTIN_SPECS / lib.rs / drift guards) + ferro-mcp 45→47 mirror fix (Wave 2)
- [x] 237-03-PLAN.md — Projection codegen emit_actions_placeholder → ActionGroup + docs migration + delete dead render_dropdown_menu (Wave 3)
- [ ] 237-04-PLAN.md — ferro-base.css regen + version bump 0.2.72→0.2.73 + operator-gated crates.io publish (Wave 4)

## ✅ v16.2 ferro-inertia First-Load HTML Shell (Phase 238) — Completed 2026-06-21

`ferro-inertia` owns the `X-Inertia` JSON contract (part 2 of Inertia) but had no
server-rendered first-load HTML document (part 1). This phase added the missing
shell so a Ferro+Inertia app can be opened cold in a browser and hydrated from the
backend. Promoted [backlog/2026-06-21-inertia-first-load-shell.md](backlog/2026-06-21-inertia-first-load-shell.md);
field-reported by downstream consumer app `u` (Phase 5 OQ-4 deferral).

### Phase 238: Inertia first-load HTML shell — server-rendered initial document ✅

**Goal:** `ferro-inertia` emits a complete first-load HTML document — embedded
`data-page` page object plus resolved Vite asset tags — when a request is not
`X-Inertia`, while continuing to emit the JSON contract when it is. Asset
resolution runs in two modes off the existing `vite_dev_server` config: **dev** →
Vite client + entry module tags against the dev-server URL; **prod** → hashed
`<script>`/`<link>` tags read from the Vite `manifest.json`. A configurable
root-template (title, `<head>` extras, `#app` mount node) ships with a sane
default. Docs cover the same-origin story and a Vite `server.proxy` recipe for the
split-port dev flow (so the session cookie flows).

**Reconcile finding:** the HTML-shell substrate already existed in
`ferro-inertia/src/response.rs` (content negotiation + dev/prod asset modes). The
real work was wiring + surfacing: the documented-but-missing `App::set_inertia_config`
+ `InertiaConfig::from_env`, structured root-template fields (title/head_extras/mount_id),
the same-origin/Vite-proxy docs, and end-to-end tests. `ferro-inertia` stayed a
zero-ferro-dep leaf crate.

**Success Criteria** (all met):
  1. ✅ A non-`X-Inertia` GET returns a full HTML document with `<div id="app" data-page="{…}">` matching the JSON-path page object.
  2. ✅ The same handler with `X-Inertia` headers still returns the JSON contract (content negotiation, single handler).
  3. ✅ Dev mode emits Vite client + entry module tags against `vite_dev_server`; prod mode emits hashed tags from the Vite `manifest.json`.
  4. ✅ The root template (title, `<head>` extras, mount node) is configurable with a working default.
  5. ✅ Docs include the same-origin convention and a Vite `server.proxy` recipe.

**Plans:** 4 plans, all complete.
- [x] 238-01-PLAN.md — `InertiaConfig::from_env()` + title/head_extras/mount_id fields & builders (Wave 1)
- [x] 238-02-PLAN.md — Extend HTML templates for new fields + content-negotiation/SC-1/SC-2/SC-3 tests (Wave 2)
- [x] 238-03-PLAN.md — Process-global `InertiaConfig` + `App::set_inertia_config` + render-path wiring (Wave 2)
- [x] 238-04-PLAN.md — Docs: fix drift + First-Load HTML Shell (same-origin + Vite proxy) section (Wave 3)

**Closeout:** verified (5/5 SC), code-reviewed (4 warnings fixed), threat-secure (7/7 closed), UAT 5/5 (live dev hydration + prod manifest). See `238-VERIFICATION.md`, `238-REVIEW-FIX.md`, `238-SECURITY.md`, `238-UAT.md`.

---

## 🚧 v16.3 MCP CRUD Data Surface (Track A) (Phases 239–243)

**Goal:** A projection that opts in derives a complete, safe, tenant-scoped CRUD
interface — create / read+query / update / soft-delete — as MCP tools with zero
hand-written tool code. Foundational track (highest compression) of the four-track
MCP capability program (A–D).

**Anchor spec:** `docs/superpowers/specs/2026-06-23-projection-crud-data-surface-design.md`
(see its "Within-Track sequencing" section — the phase skeleton below mirrors it).

**Builds on shipped work:**
- v16.0 (Phases 231/232) — `derive_transition_plan` + the channel-agnostic
  `framework::write` kernel (`dispatch_write`, `ExecutorFn`, `OverrideFn`,
  `WriteDispatcher`, idempotency, channel-parameterized audit, confirmation,
  guard re-eval, tenant isolation).
- Phase 212 — `TenantScoped` + `find_for_tenant(id, tenant_id)`.
- Phase 205 — the `tools/call` `CallToolResult::structured` `content[]` envelope.

**Already shipped (do NOT re-plan):** CRUD-07 and the CRUD-01 *declaration surface*
(`.creatable`/`.updatable`/`.deletable`/`.mcp_write_ability`/`.table`/`.soft_delete_column`
builders + the `ServiceDef::validate()` write-ability fail-fast rule) landed in
`5cb17d60`. The remaining CRUD-01 work is the `create_<svc>` tool + its schema derivation.

**Architectural constraint (encoded in every phase goal):** the CRUD dispatch
**extends** the `framework::write` kernel via a new `derive_crud_plan` (the CRUD analog of
`derive_transition_plan`). It MUST NOT rebuild the `WriteDispatcher`/override-hook/idempotency/
audit/confirmation machinery — those already exist. Update/delete targeting reuses
`TenantScoped` + `find_for_tenant`. Rebuilding the dispatcher would create the duplicate
write-control surface ferro's conventions forbid.

### Phases

- [x] **Phase 239: Soft-delete data model + `deleted_at` migration** — Add a nullable `deleted_at` column substrate so soft-delete + non-disclosure can be enforced uniformly. (completed 2026-06-23)
- [x] **Phase 240: CRUD input-schema derivation + `list_` query polish** — Auto-derive `create_`/`update_`/`delete_` input schemas from existing `field()` declarations and extend `list_` with range/sort/pagination. (completed 2026-06-23)
- [x] **Phase 241: `derive_crud_plan` + wire CRUD verbs into `framework::write`** — Mirror `derive_transition_plan` with a CRUD plan and run it through the existing kernel (override registry / idempotency / audit / confirmation reused). (completed 2026-06-23)
- [x] **Phase 242: Write authorization, tenant injection & non-disclosure** — Gate C/U/D on `read_write` scope + `.mcp_write_ability`; inject `tenant_id` server-side; make cross-tenant/soft-deleted targets indistinguishable from "not found". (completed 2026-06-24)
- [ ] **Phase 243: App integration, e2e, envelope guard & catalog/docs** — Flip the app's `order` projection to CRUD, drive create→list→update→delete over `:8090/mcp` and the visual surface, extend the structured-envelope regression guard, update `ferro-mcp` catalog/docs.

### Phase Details

#### Phase 239: Soft-delete data model + `deleted_at` migration
**Goal:** Establish the soft-delete data substrate every CRUD read/update/delete path
depends on — a nullable `deleted_at` column on soft-deletable tables plus the
`field->column` binding the kernel needs — so a deleted row becomes invisible by
construction rather than by ad-hoc filtering. `created_at`-on-create and the
tenant-column-as-server-injected contract are fixed here at the data layer.
**Depends on:** Phase 232 (the `framework::write` kernel this milestone extends) and the
shipped declaration surface (`5cb17d60`).
**Requirements:** (foundation phase — no v1 requirement uniquely owned; provides the
`deleted_at` substrate consumed by CRUD-03 in Phase 241 and the non-disclosure substrate
consumed by CRUD-05 in Phase 242).
**Success Criteria** (what must be TRUE):
  1. A backend-portable migration adds a nullable `deleted_at` to the soft-deletable
     table(s); a fresh `db:migrate` applies clean on both SQLite and Postgres.
  2. The `table()`/`soft_delete_column()` binding resolves a projection's field set to its
     concrete columns (default `deleted_at`, explicit override honored).
  3. A row with a non-null `deleted_at` is excluded from a baseline read query in a unit
     test (the `deleted_at IS NULL` predicate is enforced at the data layer, not per-tool).
  4. `created_at` is set on insert and the tenant column is identified as server-injected
     (never an agent input) at the schema-derivation boundary.
**Plans:** 3/3 plans complete
- [x] 239-01-PLAN.md — additive deleted_at migration + orders entity sync (SC#1)
- [x] 239-02-PLAN.md — ServiceDef resolver accessors + is_server_injected_field classifier (SC#2, SC#4)
- [x] 239-03-PLAN.md — dispatch resolved_table wiring + deleted_at IS NULL predicate + exclusion test (SC#3)

#### Phase 240: CRUD input-schema derivation + `list_` query polish
**Goal:** Derive correct, safe MCP input schemas for `create_`/`update_`/`delete_<svc>`
from the *existing* `field()` declarations (single source of truth) and extend the
already-derived `list_<svc>` equality filters with range/comparison ops, sort, and
pagination — so a projection authored for reads yields correct write schemas and a
richer query surface for free.
**Depends on:** Phase 239.
**Requirements:** CRUD-01 (create_<svc> tool + auto-derived input schema — excludes
Identifier, CreatedAt, tenant column, Sensitive; Status set to SM initial state when an SM
exists), CRUD-02 (update_<svc> patch schema, data fields only; Status never an update input
under an SM), CRUD-04 (`list_` range/comparison filters `__{gt,gte,lt,lte,ne,in}`, `sort`
`field`/`-field`, `limit`/`offset` atop existing equality filters).
**Success Criteria** (what must be TRUE):
  1. An opted-in projection lists a `create_<svc>` tool whose input schema contains exactly
     the creatable data fields — Identifier, CreatedAt, the tenant column, and `Sensitive`
     fields are absent; when an SM exists, `Status` is absent (set server-side to the
     initial state) and present as a writable field only when no SM exists.
  2. The `update_<svc>` tool requires the identifier and exposes the data fields as optional
     (patch semantics); under an SM, `Status` is never an update input.
  3. `list_<svc>` accepts `<field>__gt/gte/lt/lte/ne/in`, `sort=field` / `sort=-field`, and
     `limit`/`offset`, while the pre-existing equality params remain unchanged (back-compat).
  4. Field-set and query-param derivation are covered by table tests asserting Status
     inclusion/exclusion with vs without an SM and the full range/sort/pagination param set.
**Plans:** 4/4 plans complete
- [x] 240-01-PLAN.md — ServiceDef::is_write_excluded_field shared predicate (CRUD-01/02)
- [x] 240-02-PLAN.md — is_range_filter_field + create/update/delete schema builders + list_ range/sort schema (CRUD-01/02/04)
- [x] 240-03-PLAN.md — CRUD tool emission + NTI envelope + Phase 205 guard extension (CRUD-01/02)
- [x] 240-04-PLAN.md — list_ dispatch __op filters + sort read execution (CRUD-04)

#### Phase 241: `derive_crud_plan` + wire CRUD verbs into `framework::write`
**Goal:** Add the CRUD analog of `derive_transition_plan` —
`derive_crud_plan(svc, verb, inputs)` in `ferro-projections` producing a pure,
serializable INSERT/UPDATE/soft-delete plan — and teach the existing `framework::write`
kernel a CRUD verb alongside the transition path, so create/update/soft-delete execute
through the *same* dispatcher, override registry, idempotency, audit, and confirmation
that transitions already use. The kernel is extended, never forked.
**Depends on:** Phase 240.
**Requirements:** CRUD-06 (CRUD verbs dispatch through `framework::write` via
`derive_crud_plan`, reusing override-hook/idempotency/audit/confirmation — single-source
across MCP and visual surfaces; does NOT rebuild the dispatcher), CRUD-03 (`delete_<svc>`
soft-deletes by setting `deleted_at`, is confirmation-gated, and is filtered out of
`list_` and every read/update/delete path).
**Success Criteria** (what must be TRUE):
  1. A `create_<svc>` call inserts a row with the creatable columns plus server-set
     `created_at` and (under an SM) the initial `Status`, returning the created record.
  2. An `update_<svc>` call applies a patch via `UPDATE … WHERE id=? AND deleted_at IS NULL`,
     and a `delete_<svc>` call sets `deleted_at` (soft-delete) rather than removing the row;
     a soft-deleted row no longer appears in `list_<svc>`.
  3. Registering `with_override("create_order", …)` (or update/delete) replaces the generic
     derived plan for that verb with no new mechanism — the generic plan is the default when
     no override is registered.
  4. A grep/structural check confirms exactly one `dispatch_write` kernel with no second CRUD
     dispatcher and no transition `match` re-encoded on the CRUD path; the same derived plan
     drives both the MCP and the visual/form surface (channel the only divergence).
**Plans:** 3/3 plans complete
- [x] 241-01-PLAN.md — `CrudPlan`/`CrudVerb`/`TenantColumn` + pure `derive_crud_plan` + re-exports + 6 derivation/serde tests in `ferro-projections` (CRUD-06, Wave 1, no deps)
- [x] 241-02-PLAN.md — `execute_crud_plan` + `dispatch_write` CRUD param + confirmation-seam extension + 8 sqlite-in-memory dispatch tests in `framework::write` (CRUD-06, CRUD-03, Wave 2, depends_on 01)
- [x] 241-03-PLAN.md — replace NTI block with derive→dispatch + structured envelope, synthesize delete confirm tools, CRUD confirm handlers + framing tests in `ferro-mcp-server` (CRUD-06, CRUD-03, Wave 3, depends_on 01+02)

#### Phase 242: Write authorization, tenant injection & non-disclosure
**Goal:** Make every CRUD write require `read_write` key scope and pass the
`.mcp_write_ability` policy Gate, inject `tenant_id` from context (never an agent input),
and ensure cross-tenant or soft-deleted targets are indistinguishable from "not found" —
closing the safety envelope so an agent can only create/update/delete within its own tenant
and can never set or read across the tenant boundary. The shipped `validate()` write-ability
fail-fast rule (CRUD-07) is verified at this boundary.
**Depends on:** Phase 241.
**Requirements:** CRUD-05 (`create`/`update`/`delete` require `read_write` scope + the
`.mcp_write_ability` Gate; `tenant_id` server-injected and excluded from every write schema;
cross-tenant / soft-deleted targets non-disclosing), CRUD-07 (`ServiceDef::validate()`
fails fast at registration when a CRUD verb is enabled without `mcp_write_ability` —
*shipped in `5cb17d60`*; verified here at the authz/boot boundary).
**Success Criteria** (what must be TRUE):
  1. A `read`-scope key calling any `create_`/`update_`/`delete_` tool is rejected
     (scope-denied) before dispatch; a `read_write` key that fails the `.mcp_write_ability`
     Gate is denied.
  2. `tenant_id` is injected from context on create and predicated (`AND tenant_id = ctx`)
     on update/delete; the tenant column is absent from every write input schema, so an
     agent cannot set or override it.
  3. An update/delete targeting another tenant's row, or a soft-deleted row, returns the
     same non-disclosing "not found / denied" envelope — no row/column/filter leakage.
  4. A boot-time test confirms `ServiceDef::validate()` rejects a projection that enables
     any CRUD verb without `mcp_write_ability` (a config error at registration, never a
     silent deny at call time).
**Plans:** 4/4 plans complete

#### Phase 243: App integration, e2e, envelope guard & catalog/docs
**Goal:** Prove the whole Track A surface end-to-end against the sample app and bring the
introspection surface to the same quality bar as the Rust API — flip the app's `order`
projection to `.creatable/.updatable/.deletable`, drive a create→list→update→delete cycle
over both the MCP endpoint and the visual surface, extend the `tools/call`
structured-envelope regression guard to each new verb, and update `ferro-mcp`
`json_ui_catalog`/`code_templates` and the docs.
**Depends on:** Phase 242.
**Requirements:** (integration phase — exercises CRUD-01..07 end-to-end; no requirement
uniquely owned here, all are delivered by Phases 240–242 and validated in this phase).
**Success Criteria** (what must be TRUE):
  1. With the app's `order` projection flipped to CRUD, an agent drives
     create → list → update → delete through `:8090/mcp` with a seeded `read_write` bearer
     key, and the same CRUD plan succeeds on the visual/form surface (shared kernel).
  2. Each `create_`/`update_`/`delete_` result is returned through the Phase 205
     `CallToolResult::structured` envelope, and the regression guard asserts a well-formed
     `content[]` for every new verb.
  3. A `delete_<svc>` without a valid confirmation token returns `confirmation_required`
     echoing the `request_confirm_delete_<svc>` affordance; with a valid token it soft-deletes.
  4. `ferro-mcp` `json_ui_catalog`/`code_templates` and `docs/src/` reflect the new CRUD
     tools (create/update/delete/query polish) accurately.
**Plans:** TBD

### Coverage

All 7 v16.3 requirements map to exactly one phase (foundation phase 239 and integration
phase 243 own no requirement uniquely — they provide the substrate and the end-to-end
validation respectively):

| Requirement | Phase |
|-------------|-------|
| CRUD-01 (create tool + schema derivation) | Phase 240 |
| CRUD-02 (update schema, data fields only) | Phase 240 |
| CRUD-03 (delete soft-delete + confirmation + filtering) | Phase 241 |
| CRUD-04 (list query polish: range/sort/pagination) | Phase 240 |
| CRUD-05 (write authz + tenant injection + non-disclosure) | Phase 242 |
| CRUD-06 (derive_crud_plan + framework::write wiring) | Phase 241 |
| CRUD-07 (validate() write-ability fail-fast — shipped `5cb17d60`) | Phase 242 (verified) |

✓ 7/7 requirements mapped, no orphans, no duplicates.

### Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 239. Soft-delete data model + `deleted_at` migration | 3/3 | Complete    | 2026-06-23 |
| 240. CRUD input-schema derivation + `list_` query polish | 4/4 | Complete    | 2026-06-23 |
| 241. `derive_crud_plan` + wire CRUD verbs into `framework::write` | 3/3 | Complete    | 2026-06-23 |
| 242. Write authorization, tenant injection & non-disclosure | 4/4 | Complete   | 2026-06-24 |
| 243. App integration, e2e, envelope guard & catalog/docs | 0/0 | Not started | - |
