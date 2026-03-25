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
- 🔧 **v7.1 Static File Serving** — Phase 69
- 🔧 **v7.2 CI Stability** — Phase 70
- 🔧 **v7.3 Vite Manifest** — Phase 71
- ✅ **v7.4 Security Hardening** — Phases 72-74 (shipped 2026-02-26)
- ✅ **v7.5 Type Generator Fix** — Phase 75 (shipped 2026-02-27)
- ✅ **v7.6 Default API Scaffold** — Phase 76 (shipped 2026-02-27)
- ✅ **v7.7 Validate & Fix API Scaffold** — Phase 77 (shipped 2026-02-28)
- ✅ **v7.8 Memory Leak Fixes** — Phase 78 (shipped 2026-02-28)
- ✅ **v8.0 Consumer MCP — OpenAPI Bridge** — Phases 79-82 (shipped 2026-02-28)
- ✅ **v8.1 API DX Polish** — Phase 83 (shipped 2026-02-28)
- ✅ [**v9.0 Service Projections**](milestones/v9.0-ROADMAP.md) — Phases 84-94 (shipped 2026-03-01)
- 🚧 **v10.0 JSON-UI Visual Overhaul** — Phases 102-107 (in progress)

---

### ✅ v9.0 Service Projections (Complete)

**Milestone Goal:** Introduce a three-layer architecture where services describe business capabilities and UI is generated from those descriptions: Model → ServiceDef → IntentGraph → Renderer → Output. Define a standardized protocol for interoperability.

#### Phase 84: Service Identity & Field Semantics ✅

**Goal**: Create `ferro-projections` crate with `ServiceDef` and `FieldMeaning`.
**Depends on**: Previous milestone complete
**Completed**: 2026-02-28
**Plans**: 2/2

Plans:
- [x] 84-01: ferro-projections crate with ServiceDef builder, FieldMeaning, DataType, infer_meaning
- [x] 84-02: Comprehensive test suite (22 unit tests + 1 doctest)

#### Phase 85: State Machines ✅

**Goal**: Add state machine definitions — states, transitions, guards, side effects as schema.
**Depends on**: Phase 84
**Completed**: 2026-02-28
**Plans**: 2/2

Plans:
- [x] 85-01: StateMachine, StateDef, Transition, Warning types with builder APIs, BFS validation, ServiceDef integration
- [x] 85-02: Comprehensive test suite (49 unit tests + 2 doctests)

#### Phase 85.1: Architecture Refinement (INSERTED) ✅

**Goal**: Incorporate prior art insights. Add schemars JSON Schema derives. Refine Phase 86-93 descriptions shifting from "map SAP floorplans" to "derive intent from structure." Descope Phase 94 to protocol documentation only.
**Depends on**: Phase 85
**Completed**: 2026-02-28
**Plans**: 2/2

Plans:
- [x] 85.1-01: schemars JSON Schema derives on 7 public types + 4 schema tests
- [x] 85.1-02: Refined Phase 86-93 descriptions with structural intent derivation principles

#### Phase 86: Actions & Preconditions ✅

**Goal**: Add Siren-inspired ActionDef with inputs reusing DataType/FieldMeaning. Add readable/writable to FieldDef for intent derivation.
**Depends on**: Phase 85.1
**Completed**: 2026-02-28
**Plans**: 2/2

Plans:
- [x] 86-01: ActionDef, InputDef, GuardDef types with builder APIs + readable/writable on FieldDef (78 tests)
- [x] 86-02: ServiceDef integration, cross-phase validate(), comprehensive test suite (97 tests total)

#### Phase 87: Service Relationships ✅

**Goal**: Declare service relationships with cardinality as structural signal for intent derivation.
**Depends on**: Phase 84 (parallel with 85-86)
**Completed**: 2026-02-28
**Plans**: 1/1

Plans:
- [x] 87-01: RelationshipDef, Cardinality, NavigationHint types with ServiceDef integration and validation (118 tests)

#### Phase 88: Intent Layer — Core Types ✅

**Goal**: Define Intent enum from structural derivation analysis. 7 derivable intents + Custom escape hatch + IntentHint override.
**Depends on**: Phase 84, Phase 86
**Completed**: 2026-03-01
**Plans**: 2/2

Plans:
- [x] 88-01: Intent, IntentScore, IntentHint types + ServiceDef integration + validation (143 tests)
- [x] 88-02: Comprehensive test suite — serde, schema, builder, validation, integration (148 tests)

#### Phase 89: Intent Graph Generation ✅

**Goal**: Structural analysis engine — pattern matching on ServiceDef signals to ranked intents with confidence scores. Core innovation phase.
**Depends on**: Phase 88
**Completed**: 2026-03-01
**Plans**: 3/3

Plans:
- [x] 89-01: Derivation engine foundation — derive_intents() API, field meaning + writability analyzers, scoring pipeline
- [x] 89-02: State machine, relationship, and action signal analyzers — full 5-analyzer pipeline
- [x] 89-03: Validation test suite — 12 representative ServiceDefs, 100% primary intent accuracy, edge cases

#### Phase 90: Renderer Trait & JSON-UI Renderer ✅

**Goal**: Define Renderer trait, implement first renderer mapping intents to JSON-UI components.
**Depends on**: Phase 89
**Completed**: 2026-03-01
**Plans**: 3/3

Plans:
- [x] 90-01: Renderer trait, RenderContext/RenderMode types, field mapping (display+input+column), relationship mapping
- [x] 90-02: JsonUiRenderer — Browse, Focus, Collect, Summarize intent layouts
- [x] 90-03: JsonUiRenderer — Process, Analyze, Track intent layouts + full pipeline integration tests

#### Phase 91: Framework Integration ✅

**Goal**: Wire ferro-projections into the framework — re-exports, handler helpers, route generation.
**Depends on**: Phase 90
**Completed**: 2026-03-01
**Plans**: 3/3

Plans:
- [x] 91-01: Feature-gated re-exports of all 22 ferro-projections types + error conversion for `?` in handlers
- [x] 91-02: `ferro make:projection` CLI command — scaffolds ServiceDef module with builder template
- [x] 91-03: 3 MCP introspection tools — list_projections, inspect_projection, render_projection

#### Phase 92: MCP Introspection & CLI

**Goal**: MCP tools for service discovery, CLI scaffolding.
**Depends on**: Phase 91
**Completed**: 2026-03-01
**Plans**: 3/3

Plans:
- [x] 92-01: Model-aware projection scaffolding (--from-model flag, type/meaning mapping, FK relationships)
- [x] 92-02: Projection validation CLI (projection:check) + MCP tool (validate_projection)
- [x] 92-03: Service coverage MCP tool (projection_coverage with intent derivation)

#### Phase 93: Field Test & Polish ✅

**Goal**: Build representative services exercising full stack. Validate structural intent derivation accuracy (target: >70% correct without manual hints).
**Depends on**: Phase 92
**Completed**: 2026-03-01
**Plans**: 2/2

Plans:
- [x] 93-01: 8 representative projections in sample app covering all 7 intents (User, Todo, ApiKey, Order, Product, RevenueDashboard, SalesAnalytics, FeedbackForm)
- [x] 93-02: Fixed MCP parser for action/guard/transition details + integration tests validating 100% intent accuracy

#### Phase 94: Protocol Documentation ✅

**Goal**: Define a standardized protocol specification for data/intent-oriented web services with auto-generated UIs. Formalize the ServiceDef → IntentGraph → Renderer chain as a documented, versioned protocol.
**Depends on**: Phase 93
**Completed**: 2026-03-01
**Plans**: 5/5

Plans:
- [x] 94-01: Protocol infrastructure — mdBook setup, JSON Schema generation from Rust types (17 schemas + combined protocol.json)
- [x] 94-02: Specification foundation — introduction, terminology (18 terms), architecture (three-layer pipeline)
- [x] 94-03: Data model specification — all 22 public types across 7 pages (ServiceDef, FieldDef, DataType, FieldMeaning, StateMachine, Actions, Relationships, Intent)
- [x] 94-04: Derivation, rendering & validation — 5 analyzers, Renderer trait, 4 errors + 9 warnings
- [x] 94-05: Governance & appendix — extensions, conformance (3 levels), security (7 considerations), related work (9 cited), worked examples (7 intents)

### Phase 95: Multi-tenant middleware

**Goal:** Add TenantMiddleware with pluggable resolver chain (Subdomain, Header, Path, JWT), task-local TenantContext, TenantScope query helper for cross-tenant data isolation, and FromRequest handler extraction.
**Requirements**: [MT-01, MT-02, MT-03, MT-04, MT-05, MT-06, MT-07, MT-08, MT-09, MT-10]
**Depends on:** Phase 94
**Plans:** 3/3 plans complete

Plans:
- [ ] 95-01-PLAN.md — Core types: TenantContext, task-local context, TenantResolver trait, TenantLookup with moka cache
- [ ] 95-02-PLAN.md — TenantMiddleware with resolver chain + 4 concrete resolver implementations
- [ ] 95-03-PLAN.md — TenantScope query scoping, FromRequest extractor, lib.rs re-exports, documentation

### Phase 96: Stripe integration

**Goal:** Add `ferro-stripe` crate with two-tier billing (platform SaaS subscriptions + Stripe Connect), webhook handling via ferro-events, TenantContext enrichment with SubscriptionInfo, RequiresPlan middleware, CLI scaffolding, MCP tools, and documentation.
**Requirements**: [STRIPE-01, STRIPE-02, STRIPE-03, STRIPE-04, STRIPE-05, STRIPE-06, STRIPE-07, STRIPE-08, STRIPE-09, STRIPE-10, STRIPE-11, STRIPE-12, STRIPE-13]
**Depends on:** Phase 95
**Plans:** 7/7 plans complete

Plans:
- [x] 96-01-PLAN.md — ferro-stripe crate foundation: types (SubscriptionInfo, SubscriptionStatus, Error), Stripe client facade, checkout/billing portal/Connect functions
- [x] 96-02-PLAN.md — TenantContext enrichment with subscription data, cache invalidation, RequiresPlan middleware
- [x] 96-03-PLAN.md — Webhook verification, event types (ferro-events integration), handler functions, subscription sync
- [x] 96-04-PLAN.md — Test helpers: subscription factories, signed webhook payloads, event fixture generators
- [x] 96-05-PLAN.md — CLI scaffolding (ferro make:stripe) and MCP introspection tools
- [x] 96-06-PLAN.md — Publish workflow, documentation, full workspace validation
- [ ] 96-07-PLAN.md — Gap closure: fix make:stripe webhook templates to use correct API (queue_dispatch, struct literal construction)

### Phase 97: Tenant-aware background jobs

**Goal:** Propagate tenant context into ferro-queue jobs so current_tenant() and TenantScope work inside job handlers. Jobs dispatched from tenant-scoped request handlers automatically carry tenant_id through Redis and restore full TenantContext in the worker before executing.
**Requirements**: [TBJ-01, TBJ-02, TBJ-03, TBJ-04, TBJ-05, TBJ-06, TBJ-07, TBJ-08, TBJ-09, TBJ-10, TBJ-11]
**Depends on:** Phase 96
**Plans:** 3/3 plans complete

Plans:
- [ ] 97-01-PLAN.md — ferro-queue types: JobPayload tenant_id, TenantScopeProvider trait, Error::TenantNotFound, OnceLock capture hook, PendingDispatch::for_tenant()
- [ ] 97-02-PLAN.md — Worker tenant scope: with_tenant_scope() builder, process_job scope wrapping, Clone fix, tracing span
- [ ] 97-03-PLAN.md — Framework wiring: FrameworkTenantScopeProvider, hook registration, lib.rs re-exports, documentation

### Phase 98: ferro-json-ui stable release

**Goal:** Stabilize ferro-json-ui from experimental to production-ready: add 6 dashboard-driven components (StatCard, Checklist, Toast, NotificationDropdown, Sidebar, Header), DashboardLayout with persistent shell, built-in JS runtime for SSE/toast/live-value, schemars JSON Schema generation, API visibility audit, 60+ tests, and comprehensive documentation.
**Requirements**: [COMP-01, COMP-02, COMP-03, DASH-01, DASH-02, DASH-03, JS-01, JS-02, JS-03, API-01, API-02, API-03, API-04, TEST-01, TEST-02, TEST-03, TEST-04, TEST-05, DOCS-01, DOCS-02, DOCS-03]
**Depends on:** Phase 97
**Plans:** 5/5 plans complete

Plans:
- [ ] 98-01-PLAN.md — 6 new component types + render implementations + convenience constructors for all 26 variants
- [ ] 98-02-PLAN.md — DashboardLayout with persistent sidebar/header shell + built-in JS runtime (~5-10KB)
- [ ] 98-03-PLAN.md — API visibility audit (pub(crate) internals) + schemars JSON Schema + framework re-export update
- [ ] 98-04-PLAN.md — Comprehensive test suite: serde round-trips, render tests, schema tests, plugin pipeline (60+ total)
- [ ] 98-05-PLAN.md — Documentation: component catalog (26 types), plugin guide, DashboardLayout docs, clean rustdoc

### Phase 99: Semantic theme system with intent-driven templates

**Goal:** Make JSON-UI visually customizable through semantic CSS tokens and intent-to-layout mappings configurable through declarative templates. New `ferro-theme` crate defines token vocabulary (~23 slots) and intent template schema. ThemeMiddleware enables per-request theme selection for multi-tenant white-labeling. All ~224 hardcoded Tailwind classes in render.rs/layout.rs migrated to semantic token references. JsonUiRenderer updated to consume intent template overrides.
**Requirements**: [THEME-01, THEME-02, THEME-03, THEME-04, THEME-05, THEME-06, THEME-07, THEME-08, THEME-09, THEME-10, THEME-11, THEME-12]
**Depends on:** Phase 98
**Plans:** 5/5 plans complete

Plans:
- [x] 99-01-PLAN.md — ferro-theme crate: Theme struct, ThemeError, token vocabulary, IntentTemplate schema, default embedded CSS, filesystem loader
- [x] 99-02-PLAN.md — Framework ThemeMiddleware with resolver chain, task-local current_theme(), feature-gated re-exports
- [x] 99-03-PLAN.md — render.rs + layout.rs semantic token migration (~224 hardcoded Tailwind classes replaced)
- [ ] 99-04-PLAN.md — JsonUiRenderer intent template consumption from ThemeTemplates
- [ ] 99-05-PLAN.md — CLI make:theme command, publish workflow update, theme documentation

### Phase 100: AI Structured Classification & Confirmation Primitives

**Goal:** Add framework-level primitives for AI-powered structured intent classification (Claude structured JSON output) and a confirmation state machine for gating destructive actions behind explicit user confirmation with TTL expiry.
**Requirements**: [AI-01, AI-02, AI-03, CONF-01, CONF-02, CONF-03]
**Depends on:** Phase 99
**Plans:** 3/3 plans complete

Plans:
- [ ] 100-01-PLAN.md — ferro-ai crate foundation + AI classification: ClassificationProvider trait, AnthropicProvider, Classifier<T> facade, ClassifierConfig, retry logic
- [ ] 100-02-PLAN.md — Confirmation state machine: ConfirmationStore trait, InMemoryConfirmationStore (DashMap), TTL expiry via tokio::spawn, ConfirmationExpired ferro-events integration
- [ ] 100-03-PLAN.md — Framework integration: feature-gated re-exports, MCP tools (test_classifier, list_pending_confirmations), publish workflow, documentation

### Phase 101: ferro-whatsapp Plugin

**Goal:** Create `ferro-whatsapp` plugin crate providing WhatsApp Business Cloud API integration: outbound message sender, inbound webhook dispatcher with HMAC verification, wamid-level message deduplication, and sender-identity routing (owner vs customer message classification).
**Requirements**: [WA-01, WA-02, WA-03, WA-04, WA-05]
**Depends on:** Phase 100
**Plans:** 3/3 plans complete

Plans:
- [ ] 101-01-PLAN.md — ferro-whatsapp crate foundation: core types (Error, Config, Message, SenderIdentity, DeliveryStatus), OnceLock facade, outbound sender via Meta Cloud API v23.0
- [ ] 101-02-PLAN.md — Webhook HMAC verification (X-Hub-Signature-256), DeduplicationStore trait + InMemoryDeduplicationStore, event structs (WhatsAppTextReceived, WhatsAppStatusUpdate), ProcessWhatsAppWebhook job with sender-identity routing
- [ ] 101-03-PLAN.md — Framework re-exports, CLI scaffolding (ferro make:whatsapp), MCP introspection tools, publish workflow, documentation

---

### 🚧 v10.0 JSON-UI Visual Overhaul (In Progress)

**Milestone Goal:** Reach professional visual quality across all JSON-UI components. Inter Variable font loaded and wired through correct Tailwind v4 token names. Surface elevation hierarchy (background → surface → card) applied to all components. Typography scale, form polish, interactive states, and component details completed in dependency order.

## Phases

- [x] **Phase 102: Foundation** - Fix Tailwind font token namespace and load Inter Variable font (completed 2026-03-24)
- [x] **Phase 103: Surface Elevation** - Apply card/modal/dropdown surface hierarchy and verify dark mode tokens (completed 2026-03-25)
- [x] **Phase 104: Typography Scale** - Apply heading and body line-height and letter-spacing classes (completed 2026-03-25)
- [x] **Phase 105: Form Polish** - Custom select arrow, error focus rings, transitions, disabled states (completed 2026-03-25)
- [ ] **Phase 106: Interactive States** - Focus rings and hover states across all interactive elements
- [ ] **Phase 107: Component Details** - Alert icons, Skeleton shimmer, Breadcrumb chevron, tab weight, bell SVG

## Phase Details

### Phase 102: Foundation
**Goal**: Font token namespace bug is fixed and Inter Variable font renders in all JSON-UI pages
**Depends on**: Phase 101
**Requirements**: FND-01, FND-02, FND-03, FND-04
**Success Criteria** (what must be TRUE):
  1. JSON-UI pages render text in Inter Variable (or system sans-serif fallback), not the browser default serif
  2. The `--font-sans` CSS custom property resolves to the Inter font stack in `default.css`
  3. A test that asserts on a Tailwind class string does not break unrelated tests when only that class changes
  4. Bunny Fonts `<link>` tag is present in the `<head>` of every JSON-UI document
**Plans**: 2 plans

Plans:
- [ ] 102-01-PLAN.md — Fix font token namespace (--font-sans), add Bunny Fonts CDN link, wire font-sans body class
- [ ] 102-02-PLAN.md — Add has_class test helper and structural component tests for resilient assertions

### Phase 103: Surface Elevation
**Goal**: Cards, modals, stat cards, and notification dropdowns are visually elevated above the page background in both light and dark mode
**Depends on**: Phase 102
**Requirements**: SRF-01, SRF-02, SRF-03, SRF-04, SRF-05, SRF-06, SRF-07
**Success Criteria** (what must be TRUE):
  1. Card components render with a background visually distinct from the page body (bg-card, not bg-background)
  2. Modal panels and notification dropdown panels use the card surface color, creating visible layering
  3. StatCard uses bg-card so dashboard metric cards are visually elevated from the page background
  4. All 8 critical dark mode token pairs pass WCAG 4.5:1 contrast ratio when verified with an oklch-native tool
  5. Runtime JS toast and tab switcher use semantic token classes (bg-primary, not bg-blue-500)
**Plans**: 2 plans

Plans:
- [ ] 103-01-PLAN.md — Surface elevation hierarchy: bg-card on Card, Modal, StatCard, Checklist, NotificationDropdown + dark mode contrast verification
- [ ] 103-02-PLAN.md — Runtime JS semantic tokens: VARIANT_CLASSES, toast text, tab switcher class migration

### Phase 104: Typography Scale
**Goal**: All text elements render with the correct line-height and letter-spacing for their heading level
**Depends on**: Phase 102
**Requirements**: TYP-01, TYP-02, TYP-03, TYP-04, TYP-05
**Success Criteria** (what must be TRUE):
  1. H1 and H2 headings render tighter tracking and tight line-height compared to default browser rendering
  2. H3 headings render with snug line-height
  3. Body text (P, Div, Section) renders with relaxed line-height for comfortable reading
  4. Muted text uses the same `text-text-muted` class consistently across all components
**Plans**: 1 plan

Plans:
- [ ] 104-01-PLAN.md — Typography scale classes on render_text (H1/H2/H3/P/Div/Section) + inline headings + layout.rs muted text fix

### Phase 105: Form Polish
**Goal**: All form elements have a custom select arrow, correct error focus rings, transition animations, and disabled state styling
**Depends on**: Phase 103
**Requirements**: FRM-01, FRM-02, FRM-03, FRM-04, FRM-05, FRM-06, FRM-07
**Success Criteria** (what must be TRUE):
  1. Select elements display a visible SVG chevron arrow in all major browsers without JavaScript
  2. Input, Select, and Textarea in error state show a red (destructive) focus ring, not the primary ring
  3. All form elements animate color transitions over 150ms and suppress animation for users with reduced-motion preference
  4. Disabled form elements render at reduced opacity with a not-allowed cursor
  5. Form field layout always renders in label → input → description → error message order
**Plans**: 1 plan

Plans:
- [ ] 105-01-PLAN.md — Form polish: SVG chevron on select, error focus rings, transitions, disabled states, DOM reorder

### Phase 106: Interactive States
**Goal**: Every interactive element has a visible focus ring and hover state, applied consistently across the full component catalog
**Depends on**: Phase 103
**Requirements**: INT-01, INT-02, INT-03, INT-04, INT-05, INT-06, INT-07
**Success Criteria** (what must be TRUE):
  1. Tabbing through a JSON-UI page shows a visible 2px focus ring on every interactive element (buttons, tabs, pagination, breadcrumb links, sidebar nav items)
  2. Table rows highlight on hover with a surface background tint
  3. All interactive elements animate color transitions over 150ms with reduced-motion suppression
  4. Focus rings use `focus-visible:` (not `focus:`) so mouse clicks do not trigger the ring
**Plans**: TBD

### Phase 107: Component Details
**Goal**: Alert components, Skeleton loader, Breadcrumb separator, active tab, and notification bell use refined visual treatments that eliminate emoji and add motion
**Depends on**: Phase 104, Phase 106
**Requirements**: CMP-01, CMP-02, CMP-03, CMP-04, CMP-05, CMP-06
**Success Criteria** (what must be TRUE):
  1. Alert components display an inline SVG icon that matches the variant (info, success, warning, error)
  2. Skeleton loader uses a smooth shimmer animation instead of a pulsing opacity effect
  3. Breadcrumb separator renders as an SVG chevron, not a `/` text character
  4. Active tabs render with semibold font weight, visually distinct from inactive tabs
  5. Notification bell renders as an SVG icon, consistent across all operating systems
  6. Collapsible components display a rotating SVG chevron that indicates open/closed state
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 102 → 103 → 104 → 105 → 106 → 107

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 102. Foundation | 2/2 | Complete    | 2026-03-24 |
| 103. Surface Elevation | 2/2 | Complete    | 2026-03-25 |
| 104. Typography Scale | 1/1 | Complete    | 2026-03-25 |
| 105. Form Polish | 1/1 | Complete   | 2026-03-25 |
| 106. Interactive States | 0/TBD | Not started | - |
| 107. Component Details | 0/TBD | Not started | - |

---

## Completed Milestones

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

---

## Completed Milestones

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
| v10.0 JSON-UI Visual Overhaul | 102-107 | TBD | In progress | - |

**Total: 22 milestones shipped, 197 plans.**
