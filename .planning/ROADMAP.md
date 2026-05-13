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
- 📋 **v11.9 Notifications & Rich-Text Foundations** — Phases 149-150 (planned 2026-04-28). Source: gestiscilo-it v6.4 Documents & Notifications field test. Extends `ferro-notifications` with `Channel::WhatsApp` + `Channel::InApp` adapters and `MailMessage::attachment()` builder; ships `ferro-json-ui RichTextEditor` component (Quill 2.0.3 plugin pattern) so consumer apps can author rich-text bodies without bundling. Auto-publishes via GH Actions. Single load-bearing prerequisite for gestiscilo-it v6.4 Phase 120 (notification dispatcher) and Phase 125 (document template editor).
- 📋 **v11.10 ferro-wallet — Digital Wallet Passes** — Phase 151 (planned 2026-05-11). Source: gestiscilo-it digital wallet booking pass field test. New project-agnostic crate `ferro-wallet` providing the `WalletSubject` trait, `ApplePassBuilder` (PKCS#7-signed `.pkpass`), `GoogleWalletBuilder` (RS256 save-link JWT), and image / QR primitives. Follows architecture principle 6 (project-agnostic, reads `APP_NAME` / `APP_URL` via `WalletConfig::from_env`). Single load-bearing prerequisite for gestiscilo-it wallet booking passes integration. [Context](phases/151-ferro-wallet-crate/151-CONTEXT.md) · [Spec](../docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md)
- 📋 **v11.11 Resource Reservation & Live Read-Model Primitives** — Phases 152-155 (planned 2026-05-13). Source: gestiscilo-it inventory monitoring field test. Four reusable horizontal primitives: `ferro-orm::GuardedUpdate` (atomic conditional updates), `ferro-audit` (structured before/after log), `ferro-reservation` (generic hold/commit/release with TTL), `ferro-projection` (live read-model from domain events with broadcast deltas). Unblocks gestiscilo-it v6.3 online checkout reservation TTL and v6.7 inventory monitoring. [Design](research/INVENTORY-PRIMITIVES.md)
- 📋 **v12.0 JSON-UI v2 — Spec-Driven Rendering** — Phases 115-121 (planned, enriched with JSON Schema contract). Depends on v11.5.
- 📋 **v12.1 Form Validation DX** — Phases 137-139. Validator struct, old input preservation, DB constraint error mapping. Source: gestiscilo-it field test.
- 📋 **v13.0 Road to v1.0** — sustained investment program across compressive / operational / conceptual / aesthetic dimensions. 19+ requirements (COMP-01..05, OPER-01..07, CONC-01..04, AEST-01..04) in `.planning/REQUIREMENTS.md`. Includes crate consolidation audit and ServiceDef derivation bridge. Phase numbering continues after v12.0. No target date.
- 📋 **v14.0 Channel Projection — Non-Visual Rendering** — non-visual Renderer implementations (conversational text, voice, structured API). Reuses ferro-ai for inbound intent classification. 5 requirements (CHAN-01..05) in `.planning/REQUIREMENTS.md`. Depends on COMP-05 (intent vocabulary validation). v11.5 prerequisite (generalized Renderer trait) shipped 2026-04-17.

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

### 📋 v12.0 JSON-UI v2 — Spec-Driven Rendering (Planned)

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

- [ ] **Phase 115: Spec v2 Data Structures** — New `Spec` type with flat element map, props separation, clean break from v1
- [ ] **Phase 116: Flat Element Renderer** — Update render pipeline to walk flat element map via ID lookups
- [ ] **Phase 117: Catalog & JSON Schema** — Machine-readable `Catalog` with per-component JSON Schema, full spec schema, validation, and `ferro json-ui:schema` CLI export
- [ ] **Phase 117.1: Schema-Driven Projections** — `Spec::from_service_def()` generates v2 specs from ServiceDef using JSON Schema type mapping, replacing hardcoded `field_to_input()` mappings
- [ ] **Phase 118: Server-Side Expressions** — `$data` path resolution and `$template` string interpolation at render time
- [ ] **Phase 119: Page Loader** — Framework loads JSON spec files, merges handler data, integrates with layouts
- [ ] **Phase 120: CLI & MCP Updates** — Update `make:json-view` and MCP tools for v2 format with JSON Schema as structured output constraint
- [ ] **Phase 121: Documentation & Field Test** — Update all JSON-UI docs, convert one gestiscilo page as proof of concept

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
| 115. Spec v2 Data Structures | 0/? | Not started | - |
| 116. Flat Element Renderer | 0/? | Not started | - |
| 117. Catalog & JSON Schema | 0/? | Not started | - |
| 117.1. Schema-Driven Projections | 0/? | Not started | - |
| 118. Server-Side Expressions | 0/? | Not started | - |
| 119. Page Loader | 0/? | Not started | - |
| 120. CLI & MCP Updates | 0/5 | Planned | - |
| 121. Documentation & Field Test | 0/6 | Planned | - |

**Plans:**
- [ ] 121-01-PLAN.md — Add JsonUi::render_file to framework (Wave 1, FIELD-01 blocker)
- [ ] 121-02-PLAN.md — Rewrite getting-started.md, actions.md, features/json-ui.md (Wave 2, DOC-01)
- [ ] 121-03-PLAN.md — Rewrite components.md and data-binding.md (Wave 2, DOC-01)
- [ ] 121-04-PLAN.md — Rewrite layouts.md and plugins.md (Wave 2, DOC-01)
- [ ] 121-05-PLAN.md — Create expressions.md, json-schema.md, update SUMMARY.md (Wave 3, DOC-02)
- [ ] 121-06-PLAN.md — Field test: pagamenti.json + handler + route (Wave 4, FIELD-01)

**v12.0 scope is held firm.** No expansion beyond the 8 phases above. The projection / intent abstraction already exists in v9.0 ferro-projections; v12.0 refines the rendering target.

---

### 📋 v12.1 Form Validation DX (Planned)

**Milestone Goal:** Eliminate form validation boilerplate across Ferro apps. Currently every controller manually validates fields, builds redirect URLs with query params, maps error codes to user-facing strings, and handles DB constraint violations as raw 500 errors. This milestone adds a `Validator` struct, old input preservation via flash, and DB constraint error mapping — reducing ~50 lines of per-form boilerplate to ~5.

**Source:** gestiscilo-it field test (2026-04-18). Uniqueness constraint violations on page slug_path surfaced as raw SQL errors on a separate page instead of inline form errors.

**What changes:**
- `Validator` struct with declarative rules (`required`, `max_len`, `custom`, `unique`)
- Old input flash: on validation failure, all submitted values are flashed into the session
- `req.old("field")` and `req.validation_error("field")` convenience methods on `Request`
- `errors.redirect_back()` helper that flashes errors + old input and redirects to `Referer`
- DB constraint middleware that catches `UNIQUE constraint failed` / `duplicate key value` and converts to validation-style redirect-back

**What stays:**
- Session flash mechanism (`session.flash()` / `session.get_flash()`) — already exists, used as foundation
- Manual validation remains possible for cases where the declarative API doesn't fit
- Query-param error passing still works — `Validator` is additive, not a breaking change

#### Phases

- [ ] **Phase 137: Validator & Old Input** — `Validator` struct with sync rules (`required`, `max_len`, `min_len`, `regex`, `in_list`, `custom`), old input flash on failure, `req.old()` and `req.validation_error()` methods, `redirect_back()` with flashed state
- [ ] **Phase 138: Async Validation Rules** — `unique` and other DB-backed rules via `validate_async()`, SeaORM integration for uniqueness checks with exclude-self support (for updates)
- [ ] **Phase 139: DB Constraint Error Mapping** — Opt-in middleware that catches SQLite/Postgres constraint violation errors from SeaORM and converts them to validation-style redirect-back responses with field-level errors

#### Phase Details

### Phase 137: Validator & Old Input
**Goal**: Declarative form validation with automatic old input preservation and inline error display
**Depends on**: Nothing (uses existing session flash)
**Success Criteria** (what must be TRUE):
  1. `Validator::new().required("name", "Required").max_len("name", 200)` builds a rule set
  2. `validator.validate(&HashMap<String, String>)` returns `Result<(), ValidationErrors>`
  3. `ValidationErrors::redirect_back(&req)` flashes errors and old input into session, returns 302 to `Referer` (or explicit fallback URL)
  4. `req.old("field_name")` returns `Option<String>` from flash — previous submission's value
  5. `req.validation_error("field_name")` returns `Option<String>` — the error message for that field
  6. Flash data is consumed on read (one-request lifetime, per existing flash behavior)
  7. Custom rule: `.custom("field", |v| predicate, "message")` for app-specific validation
  8. All rules are sync — no DB access in this phase

### Phase 138: Async Validation Rules
**Goal**: DB-backed validation rules, primarily `unique` for constraint pre-checking
**Depends on**: Phase 137
**Success Criteria** (what must be TRUE):
  1. `validator.validate_async(&data).await` runs both sync and async rules
  2. `.unique::<Entity>(field, filter, "message")` checks uniqueness via SeaORM query
  3. `.unique_except::<Entity>(field, exclude_id, filter, "message")` excludes current record (for updates)
  4. Async rules run after sync rules pass — no DB queries if basic validation fails
  5. Custom async rule: `.custom_async("field", |v| async_predicate, "message")`

### Phase 139: DB Constraint Error Mapping
**Goal**: Catch DB constraint violations and convert to user-friendly redirect-back responses
**Depends on**: Phase 137
**Success Criteria** (what must be TRUE):
  1. `ConstraintErrorMiddleware` catches `UNIQUE constraint failed` (SQLite) and `duplicate key value violates unique constraint` (Postgres) from SeaORM errors
  2. Extracts the column name from the error message and maps it to a field-level error
  3. Redirects back with the error flashed, same as `ValidationErrors::redirect_back()`
  4. Middleware is opt-in — must be explicitly added to route groups
  5. Does not swallow non-constraint DB errors — only handles uniqueness violations
  6. Acts as a safety net for TOCTOU races after Phase 138's pre-check

#### Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 137. Validator & Old Input | 0/? | Not started | - |
| 138. Async Validation Rules | 0/? | Not started | - |
| 139. DB Constraint Error Mapping | 0/? | Not started | - |

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
**Plans:** 4/4 plans complete

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

### 🚧 v11.3 S3 Storage Driver

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

### 🚧 v11.5 Projection Architecture Prep

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

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** none (parallel with Phase 152)
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 153 to break down)

### Phase 154: ferro-reservation crate — generic hold/commit/release with TTL and event broadcast

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 152 (guarded updates), Phase 153 (audit log)
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 154 to break down)

### Phase 155: ferro-projection crate — live read-model from domain events with delta broadcast

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** none (uses existing ferro-events + ferro-broadcast)
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 155 to break down)

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
| v12.0 JSON-UI v2 — Spec-Driven Rendering | 115-121 | ? | 📋 Planned | - |

**Total: 28 milestones shipped, 270 plans complete.**
mplete.**
