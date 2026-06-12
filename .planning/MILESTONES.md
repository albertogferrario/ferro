# Project Milestones: Ferro Framework

## v12.7 Passwordless MCP Auth (Shipped: 2026-06-12)

**Phases completed:** 2 phases (202–203), 10 plans

**Delivered:** Passwordless and cross-device authentication for the consumer-app MCP
surface. A magic-link (async) ferro app now completes the OAuth/MCP browser-login flow
by resuming the in-flight authorize request, and `ferro-mcp-oauth` gains the OAuth 2.0
Device Authorization Grant (RFC 8628) for passwordless, cross-device, and headless/CLI
MCP clients — both reusing the v12.6 consent + tenant-scoping surfaces and the single
existing token issuer (no second token path).

**Key accomplishments:**

- Phase 202 — Login-resume contract + magic-link sample app: a documented helper
  (`oauth_resume_redirect` / `take_oauth_return_to`) any login handler calls to obtain
  the post-login redirect target from the session `oauth_return_to`; the bundled sample
  app login converted from password to magic-link as the golden-path exemplar, with an
  async-flow acceptance test (unauthenticated `/authorize` → login → verify → resume →
  consent). Verified 5/5.
- Phase 203 — OAuth Device Authorization Grant (RFC 8628): `device_authorization`
  endpoint returning RFC-8628 §3.2 fields, a user-code verification page bound to the
  existing consent + `(user, tenant)` scoping (login-resume reused), and the §3.5
  device-code token polling state machine whose Approved arm mints through the same
  `build_claims` + `mint_token` path as the authorization-code arm. Discovery advertises
  `device_authorization_endpoint` + the device-code grant. Verified 5/5 (13-test SC-5 matrix).

**Verification:** both phases passed 5/5. Milestone audit `v12.7-MILESTONE-AUDIT.md`:
status tech_debt (no blockers) — cross-phase integration 8/8 wired, single-token-issuer
invariant held. Deferred edge cases: WR-02/WR-03 (resume not triggered for an
already-authenticated tab clicking a magic-link mid-flow, or for `POST /auth/register`).

## v12.5 Projection Checkpoint (Shipped: 2026-06-10)

**Phases completed:** 3 phases (194–196), 11 plans

**Delivered:** An agent-facing write→verify loop for projections. The
`checkpoint_projection` MCP tool walks a five-seam spine, owns the
projection-field→model-column seam (the one check no existing validator covered),
delegates the other seams to existing validators, and returns a single
`pass`/`warn`/`fail` verdict with ranked next steps — honest about coverage,
closing by default after generation.

**Key accomplishments:**

- Phase 194 — Core checkpoint tool: structured verdict (per-seam results + ranked,
  deduplicated `next_steps`); the field→column seam resolves projection→model and
  flags dangling fields; `not_checked` is a distinct status never coerced to `pass`
  (coverage honesty).
- Phase 195 — Close the loop by default: wrapper seams 1/3/4/5 dispatch to existing
  validators (no logic reimplemented; each finding names its source);
  `generate_projection`/`json_ui_generate` embed the verdict inline;
  `application_info`/`projection_coverage` surface per-projection checkpoint status
  from the cache.
- Phase 196 — Dogfood acceptance + hardening: a deliberately-poisoned synthetic
  fixture proves the field→column seam; the in-repo `app/` live consumer produced
  20 findings (seam 3 `action_to_route` the genuine driver — unregistered actions);
  `next_steps` capped 10→5; the one zero-finding wrapper seam (`props_to_contract`)
  demoted to `not_checked`-by-default and documented.

**Requirements:** CHK-01 … CHK-10 all complete.

**Acceptance:** GO — the checkpoint surfaced a real seam defect in a real project
(recorded in `196-ACCEPTANCE.md`).

---

## v12.4 Form Validation DX (Shipped: 2026-06-09)

**Phases 190-192** (4+2+2 plans). Async DB-backed uniqueness validation as a first-class ferro form primitive, two layers:

- **190 — proactive:** `AsyncRule` trait + `AsyncValidator`/`validate_async` + `unique(table, col)` with `.ignore(id)` exclude-self; fails before the write with a field-level error via the existing `ValidationError` → redirect-back flow. SQLite + Postgres (live-PG gate test).
- **191 — defensive:** `ConstraintMap` + `try_map` + `MapConstraintExt::map_constraint` — maps a DB UNIQUE violation at the write site to the same field error, closing the TOCTOU race; portable detection (`sql_err()` + Postgres `constraint()` / SQLite message parse); unmatched `DbErr` falls through unchanged. Live-PG gate test.
- **192 — surface:** ferro-mcp `action_handler` template + `validation.md` show both layers together (no surface shows one without the other).

All requirements VALID-01..06 Complete. Both live-Postgres manual gates closed via `#[ignore]`d tests.

## v12.1 AI — ferro-ai SDK & AI as Projection Consumer (Shipped: 2026-06-09)

**Phases 165-173** (9 phases). AI as a first-class consumer of the projection/intent core: `LlmClient` trait + providers (165), structured outputs + ServiceDef-aware schema normalizer + tool calling (166), embeddings + pgvector (167), framework SSE primitives (168), `StreamText` component (169), ferro-cli SDK migration (170), `ai:make`/`ai:explain` killer-feature CLI commands producing typed `ServiceDef` (171), MCP tool wrappers (172). **Capstone (173):** `make:json-view` consumes a `ServiceDef` via the existing `Spec::from_service_def` renderer + the offline projection-roundtrip proof test (NL → ServiceDef → rendered JSON-UI, pinned to the ServiceDef-aware path via the `Money → currency` assertion) — the structural proof that AI feeds the projection core, not a parallel scaffolder.

## v11.6.2 ferro-stripe Refund Event Completeness + 0.7.0 Release (Code complete: 2026-06-09)

**Phase 193** (1 plan). Adds `StripeChargeRefunded::refund_id: Option<String>` parsed from the charge's refunds list (`charge.refunds.data[].id` — corrected from the roadmap's mistaken `EventObject::Refund`); golden-JSON fixture + parser-contract test; ferro-stripe `0.5.0 → 0.7.0` + CHANGELOG bundling the Phase 189 manual-capture work. **Publish pending:** the 0.7.0 crates.io release fires on the operator's `git push` (GH Actions), which unblocks gestiscilo Phase 99. Requirements STRIPE-REFUND-01/02 (code) complete.

## v12.0 JSON-UI v2 — Spec-Driven Rendering (Shipped: 2026-05-19)

**Phases completed:** 115-121, 159-164 (incl. friction loop with gestiscilo)

**Key accomplishments:**

- Spec-driven rendering pipeline: `Spec` JSON is now the public wire contract; the renderer walks the spec, resolves expressions, applies visibility rules, and emits HTML deterministically
- Component catalog grew to 42 built-in components including DataTable, KanbanBoard, DetailPage, PageHeader, EmptyState, RichTextEditor, Calendar, NotificationDropdown, CheckboxList, RawHtml — each with structured props, JSON Schema, and catalog entries surfaced via `mcp__ferro__json_ui_catalog`
- JSON Schema contract for `Spec` with `json_ui_validate_spec` and `json_ui_verify_action` MCP tools; round-trip and reject test fixtures enforce the schema
- Expression engine: `{$data: "/path"}` bindings, `{$template: "..."}` interpolation, `$each` iteration, `IsTrue`/`IsFalse` visibility operators, `Action.handler` accepting `{$data}` bindings for per-row navigation
- Renderer ergonomics: `JsonUi::render_file`, back-aware redirects via `Redirect::back(&req, fallback)`, `Request::back_or(fallback)` with same-origin host enforcement, `scroll_preserve` runtime capturing `<main>.scrollTop`
- Visual polish: translucent backdrop-blur toasts with auto-dismiss, popover dropdowns, anchored Buttons, kanban with column caps and full-bleed cards, DataTable density (`px-4 py-2`), DetailPage shape
- v1 view/component materialization API fully removed (Phase 160)
- Production-validated via gestiscilo v7.0 integration loop — friction Phases 138-143 absorbed by ferro Phases 162-164

**Stats:** 491 commits, 13 phases (115-121, 159-164), single-publish cadence at merge

---

## v11.7 Tailwind Static CSS Pipeline (Shipped: 2026-04-20)

**Phases completed:** 1 phase (143), 4 plans

**Key accomplishments:**

- Pre-built `ferro-base.css` (36 KB) embedded at compile time via `include_str!`, eliminating the in-browser Tailwind JIT runtime that failed on Safari/WebKit
- Framework serves `/_ferro/ferro-base.css` automatically with `Cache-Control: public, max-age=86400`; CI drift check enforces the committed file stays in sync with Tailwind CLI output
- `JsonUiConfig::stylesheet_urls: Vec<String>` added (default `["/_ferro/ferro-base.css"]`); `tailwind_cdn` default flipped to `false`
- Theme injection migrated from `<style type="text/tailwindcss">` (Tailwind-CDN-specific magic MIME) to plain `<style>` with `:root { }` CSS variable overrides
- `ferro make:theme` scaffolder updated to emit plain CSS `:root { }` blocks instead of Tailwind `@theme { }` syntax

**Known deferred tech debt:** D-08 — no test for "app appends own token URL alongside ferro-base default" via `stylesheet_urls`; mechanism verified correct, coverage gap only.

---

## v11.1 Template Renderer (Shipped: 2026-04-07)

**Phases completed:** 1 phases, 1 plans, 2 tasks

**Key accomplishments:**

- TemplateRenderer struct implementing Renderer trait: produces intent-agnostic serde_json::Value context with fields (object), actions (array with inputs), and state_machine (object or null)

---

## v11.0 Framework Consolidation Audit (Shipped: 2026-04-07)

**Phases completed:** 7 phases, 13 plans, 14 tasks

**Key accomplishments:**

- 24 stale `ferro_rs::` import paths corrected to `ferro::` across multi-tenancy, actions, and data-binding docs
- CLI reference examples now use real logic (tracing + SeaORM patterns), S3 marked shipped, and README presents JSON-UI as a delivered feature with a corrected crate badge
- All 65 MCP tool descriptions audited — one doc bug fixed (CodeTemplatesParams missing 'api' category), three cross-references added for newer tools
- FerroModel and ValidateRules documented with complete worked examples on a dedicated derive-macros.md page linked between Database and Validation in SUMMARY.md
- introduction.md rewritten with agent-first identity and MCP callouts; new Working with Agents guide covers ferro-mcp setup for Claude Desktop, Claude Code, and generic stdio with discovery loop and agent-to-CLI workflow
- Standardized 22 documentation files to use explicit crate-root imports, #[handler] attributes, and ? / .expect() error propagation instead of glob imports and .unwrap()
- COMPONENT_CATALOG moved from two identical 100+ line local constants to a single pub const in ferro-json-ui, with ferro-cli and ferro-mcp importing it via direct dependency.

---

## --help --help (Shipped: 2026-04-07)

**Phases completed:** 7 phases, 13 plans, 14 tasks

**Key accomplishments:**

- 24 stale `ferro_rs::` import paths corrected to `ferro::` across multi-tenancy, actions, and data-binding docs
- CLI reference examples now use real logic (tracing + SeaORM patterns), S3 marked shipped, and README presents JSON-UI as a delivered feature with a corrected crate badge
- All 65 MCP tool descriptions audited — one doc bug fixed (CodeTemplatesParams missing 'api' category), three cross-references added for newer tools
- FerroModel and ValidateRules documented with complete worked examples on a dedicated derive-macros.md page linked between Database and Validation in SUMMARY.md
- introduction.md rewritten with agent-first identity and MCP callouts; new Working with Agents guide covers ferro-mcp setup for Claude Desktop, Claude Code, and generic stdio with discovery loop and agent-to-CLI workflow
- Standardized 22 documentation files to use explicit crate-root imports, #[handler] attributes, and ? / .expect() error propagation instead of glob imports and .unwrap()
- COMPONENT_CATALOG moved from two identical 100+ line local constants to a single pub const in ferro-json-ui, with ferro-cli and ferro-mcp importing it via direct dependency.

---

## v10.0 JSON-UI Visual Overhaul (Shipped: 2026-03-26)

**Delivered:** Professional visual quality uplift across all JSON-UI components — Inter font, surface elevation, typography scale, form polish, interactive states, and SVG icon refinements.

**Phases completed:** 102-107 (8 plans total)

**Key accomplishments:**

- Inter Variable font loaded via Bunny Fonts CDN with correct Tailwind v4 --font-sans token namespace (Phase 102)
- Three-tier surface elevation hierarchy (background → surface → card) with WCAG 4.5:1 dark mode contrast verification (Phase 103)
- Typography scale: H1/H2 tight tracking, H3 snug, body relaxed line-height across all text elements (Phase 104)
- Form polish: inline SVG select chevron, destructive error focus rings, 150ms transitions with reduced-motion, disabled states (Phase 105)
- Focus-visible rings and hover states on all interactive elements (buttons, tabs, pagination, breadcrumbs, sidebar, table rows) (Phase 106)
- SVG icons for alerts/bell/breadcrumb/collapsible, CSS shimmer animation for skeleton, font-semibold active tabs (Phase 107)

**Stats:**

- 39 files changed (+6,847, -204 lines)
- 6 phases, 8 plans, 46 commits
- 2 days (2026-03-24 → 2026-03-26)

**Git range:** `9d906347` → `67d74d51`

**What's next:** Planning next milestone.

---

## v8.1 API DX Polish (Shipped: 2026-02-28)

**Delivered:** Closed the DX gaps between `ferro make:api` scaffold and a working MCP integration with five targeted improvements.

**Phases completed:** 83 (5 plans total)

**Key accomplishments:**

- `ferro make:api-key` CLI command generates API keys with SHA-256 hashing, SQL/Rust code snippets (8 tests)
- Route-level x-MCP builder API: .mcp_tool_name(), .mcp_description(), .mcp_hint(), .mcp_hidden() with group-level defaults (5 tests)
- Sensitive field auto-exclusion in make:api with --exclude/--include-all flags and 8 known patterns (8 tests)
- `ferro api:check` validates server connectivity, OpenAPI spec, and API key auth with actionable error messages (7 tests)
- Enhanced post-scaffold guidance with setup steps, MCP config snippets for Claude Desktop and Claude Code
- Complete API-to-MCP documentation: Quick Start Workflow and Route Customization guides

**Stats:**

- 20 files changed (+1,995, -78 lines)
- 1 phase, 5 plans, 10 tasks
- 1 day (2026-02-28)

**Git range:** `495edd9` → `7aae50e`

**What's next:** Planning next milestone.

---

## v6.0 ferro-lang — Localization (Shipped: 2026-02-13)

**Delivered:** Added localization infrastructure via new ferro-lang crate with JSON translations, per-request locale detection, validation message localization, CLI scaffolding, MCP introspection, and comprehensive test coverage.

**Phases completed:** 58-66 (11 plans total)

**Key accomplishments:**

- Created ferro-lang crate with JSON translation loading, :param interpolation, and pipe-separated pluralization with range syntax
- Per-request locale detection via task_local! with LangMiddleware (Accept-Language + query param)
- OnceLock-based validation bridge decoupling all 22 rules from ferro-lang with English fallback
- Framework integration with t()/trans()/choice() helpers auto-booted in Application::run()
- CLI scaffolding: make:lang command + ferro new templates with localization defaults
- MCP introspection (list_lang_files) + comprehensive documentation page (253 lines)

**Stats:**

- 69 files changed (+6,811, -85 lines)
- 9 phases, 11 plans, 48 commits
- 1 day (2026-02-13)

**Git range:** `d99fbcd` → `5073fc2`

**What's next:** Publish to crates.io and public announcement.

---

## v5.1 Housekeeping (Shipped: 2026-02-13)

**Delivered:** Resolved technical debt and improved project hygiene: fixed deployment templates, split oversized template files, updated env defaults, and audited concerns.

**Phases completed:** 54-57 (5 plans total)

**Key accomplishments:**

- Updated env.example.tpl to match all 63 framework env vars (removed 8 phantom, added 23 missing)
- Split templates/mod.rs from 2,987 to 831 lines across 7 focused modules
- Audited CONCERNS.md: resolved 6/8 items, rebuilt priority matrix to 4 remaining
- Fixed deployment templates: health check path, Rust image version, deployment tip text

**Stats:**

- 26 files changed (+4,821, -3,669 lines)
- 4 phases, 5 plans, 17 commits
- 1 day (2026-02-13)

**Git range:** `3f5e0e1` → `fa1375f`

**What's next:** v6.0 ferro-lang — Localization.

---

## v5.0 Proximity — JSON-UI Field Test (Shipped: 2026-02-10)

**Delivered:** Built a complete map-based social network app (app-proximity) as the first real-world validation of JSON-UI and v4.0 features, including a plugin system, geospatial queries, real-time presence, and end-to-end UI polish.

**Phases completed:** 47-53 (20 plans total)

**Key accomplishments:**

- JSON-UI plugin system with trait-based extensibility, global registry, and Map plugin with Leaflet rendering
- app-proximity workspace crate — complete social network with auth, geo profiles, location posts, and nearby feeds
- Geospatial proximity queries with bounding-box + Haversine filtering, nearby users map, and nearby posts feed
- Real-time presence via WebSocket broadcasting with channel authorization, presence data, and live location/post events
- UI polish with ProximityLayout navigation, Avatar/Badge/DescriptionList components, and relative timestamps
- JSON-UI field validation: discovered and fixed issues (Div/Section variants, SQLite Haversine in Rust, input step attribute)

**Stats:**

- 104 files changed (+11,900, -77 lines)
- 3,042 lines of Rust (app-proximity)
- 7 phases, 20 plans, 82 commits
- 1 day (2026-02-10)

**Git range:** `dbdb0f0` → `24fecfe`

**What's next:** Publish to crates.io and public announcement.

---

## v4.0 Production Readiness (Shipped: 2026-02-10)

**Delivered:** Authentication, API resources, rate limiting, real-time WebSocket broadcasting, and DX polish to make Ferro production-ready.

**Phases completed:** 38-46 (24 plans total)

**Key accomplishments:**

- Complete session-based authentication system with bcrypt hashing, Auth facade, AuthUser/OptionalUser extractors, middleware guards, and `ferro make:auth` CLI scaffolding
- Production-ready API Resources with derive macro, ResourceMap builder, pagination envelope, collection mapping, and batch-loaded relationship support
- Cache-backed rate limiting with RateLimiter::define() and Throttle middleware supporting named limiters, multiple limits per route, and fail-open behavior
- Real-time WebSocket broadcasting with upgrade handler, heartbeat/timeout, channel authorization, and whisper support for client-to-client messaging
- Enhanced DX with actionable error hints, comprehensive MCP introspection (list_resources, list_policies, list_rate_limiters, list_broadcast_channels), and v4.0 feature documentation
- Stabilized foundation: fixed flaky tests, replaced S3 driver panics, removed CDN dependencies, added 100+ unit tests across all new features

**Stats:**

- 128 files changed (+16,105, -878 lines)
- ~80,900 lines of Rust (total codebase)
- 9 phases, 24 plans
- 2 days (2026-02-09 → 2026-02-10)

**Git range:** `94c73c1` → `a9dcd8a`

**What's next:** Publish to crates.io and public announcement.

---

## v3.0 JSON-UI (Shipped: 2026-02-09)

**Delivered:** JSON-based UI rendering system as an alternative to Inertia, enabling rapid UI without frontend builds.

**Phases completed:** 23-32 (24 plans total)

**Key accomplishments:**

- Created ferro-json-ui crate with 20-component catalog (Card, Table, Form, Modal, Tabs, etc.) using serde-tagged enums and shadcn/ui-aligned variants
- Built complete Rust HTML renderer with Tailwind CSS output, XSS prevention, and progressive enhancement (no-JS modals, SSR tabs)
- Integrated data binding with slash-separated JSON paths, 11 visibility operators with And/Or/Not composition, and automatic validation error propagation
- Implemented action system with builder API, callback-based URL resolution, and confirmation/outcome chaining
- Added layout system with trait-based registry, 3 default layouts (Default/App/Auth), and composable partial functions
- Built AI-powered `ferro make:json-view` CLI command with Anthropic API and 3 MCP tools (catalog, inspect, generate) for agent-driven development
- Created comprehensive documentation: getting-started guide, component reference (all 20), actions, data binding, layouts, and CLI reference updates

**Stats:**

- 336 files changed (+39,266, -1,297 lines)
- 7,203 lines of Rust (ferro-json-ui crate)
- 2,134 lines of documentation (6 pages)
- 10 phases, 24 plans, 241 commits
- 24 days (2026-01-16 → 2026-02-09)

**Git range:** `2cd48df` → `45e5487`

**What's next:** Publish to crates.io and public announcement.

---

## v2.2 CLI Improvements (Shipped: 2026-02-09)

**Delivered:** CLI commands for database workflows, gitignore for generated types, and typed UpdateBuilder pattern for model updates.

**Phases completed:** 35-37 (5 plans total)

**Key accomplishments:**

- Added `ferro db:seed` CLI command completing the seeder workflow
- Unified all database commands under `db:` namespace (db:migrate, db:rollback, db:status, db:fresh, db:seed)
- Excluded generated TypeScript types directory from version control in project template
- Implemented typed UpdateBuilder with selective field tracking via `model.update().set_field(v).save().await`
- Updated scaffold templates, MCP code templates, and documentation with builder pattern

**Stats:**

- 40 files modified (+2098, -310 lines)
- 3 phases, 5 plans, ~11 tasks
- 22 days (2026-01-18 to 2026-02-09)

**Git range:** `09e01d3` → `3c7dcfb`

**What's next:** v3.0 JSON-UI for JSON-based UI rendering without frontend builds.

---

## v2.1 Inertia DX & Fixes (Shipped: 2026-01-17)

**Delivered:** Improved Inertia developer experience with JSON API fallback, auto type generation, utility types, and fixed documentation URLs.

**Phases completed:** 33-34 (4 plans total)

**Key accomplishments:**

- Added JSON Accept header fallback for API testing via `render_with_json_fallback()` method
- Enhanced SavedInertiaContext documentation with Common Patterns and Troubleshooting sections
- Enabled auto type generation by default in `ferro serve` with file watching
- Added `JsonValue` and `ValidationErrors` utility types to generated TypeScript
- Fixed documentation URLs to use docs.ferro-rs.dev subdomain

**Stats:**

- 34 files modified (+1165, -219 lines)
- 2 phases, 4 plans, ~12 tasks
- Same day completion (2026-01-17)

**Git range:** `e69749d` → `556eee7`

**What's next:** v3.0 JSON-UI for JSON-based UI rendering without frontend builds.

---

## v2.0.3 DO Apps Deploy (Shipped: 2026-01-17)

**Delivered:** One-click deployment to DigitalOcean App Platform with `ferro do:init` CLI command.

**Phases completed:** 22.10 (1 plan total)

**Key accomplishments:**

- Created DO App Platform spec template with service, database, and redis configuration
- Implemented `ferro do:init --repo owner/repo` command following docker_init pattern
- Generated YAML includes GitHub integration with deploy-on-push
- Health check endpoint, environment variables, and database bindings pre-configured

**Stats:**

- 9 files modified (606 insertions)
- 1 phase, 1 plan, 4 tasks
- Same day completion (2026-01-17)

**Git range:** `87bd781` → `705750d`

**What's next:** v2.1 JSON-UI milestone for JSON-based UI rendering.

---

## v2.0.2 Type Generator Fixes (Shipped: 2026-01-17)

**Delivered:** TypeScript type generation fixes for production-ready frontend integration.

**Phases completed:** 22.4-22.9 (6 plans total)

**Key accomplishments:**

- Fixed serde case handling with enum-based approach
- Resolved prop naming collisions with namespaced names
- Added contract validation CLI command
- Implemented datetime type recognition for chrono types
- Added nested types generation with fixed-point iteration
- Mapped ValidationErrors to Record<string, string[]>

**Stats:**

- 6 phases, 6 plans
- Same day completion (2026-01-17)

**Git range:** Phase 22.4 → Phase 22.9

**What's next:** v2.0.3 DO Apps Deploy

---

## v2.0.1 Macro Fix (Shipped: 2026-01-17)

**Delivered:** Fixed hardcoded macro crate paths from `::ferro_rs::` to canonical `ferro::`.

**Phases completed:** 22.1-22.3 (6 plans total)

**Key accomplishments:**

- Fixed proc macro crate path generation
- Simplified macro path handling
- Completed remaining rebrand items

**Stats:**

- 3 phases, 6 plans
- Same day completion (2026-01-17)

**Git range:** Phase 22.1 → Phase 22.3

**What's next:** v2.0.2 Type Generator Fixes

---

## v2.0 Rebrand (Shipped: 2026-01-16)

**Delivered:** Complete framework rebrand from "cancer" to "ferro" for crates.io publication and public release.

**Phases completed:** 13-22 (13 plans total)

**Key accomplishments:**

- Renamed all 11 crates from cancer-* to ferro-* (framework, CLI, MCP, events, queue, etc.)
- Updated all documentation, READMEs, and code comments to use "ferro" branding
- Created comprehensive migration guide for existing users at docs/src/upgrading/migration-guide.md
- Prepared crates.io metadata and publishing checklist (PUBLISHING.md)
- Updated repository URLs to ferroframework/ferro
- Migrated sample app to use ferro imports

**Stats:**

- 321 files modified
- 60,000 lines of Rust (total codebase)
- 10 phases, 13 plans
- 1 day (intensive single-day rebrand)

**Git range:** `docs(13-01)` -> `docs(phase-22)`

**What's next:** Publish crates to crates.io using PUBLISHING.md checklist, then announce public release.

---

## v1.0 DX Overhaul (Shipped: 2026-01-16)

**Delivered:** Agent-first developer experience transformation with reduced boilerplate, expanded MCP introspection, and improved CLI scaffolding.

**Phases completed:** 1-12 (18 plans total)

**Key accomplishments:**

- Simplified handler definitions with #[handler] macro reducing ceremony
- Created FerroModel derive macro for automatic SeaORM trait implementations
- Added ValidateRules derive macro for concise validation rule definitions
- Expanded MCP to 30+ introspection tools including domain glossary, relationship graphs, and generation hints
- Added CLI feature scaffolding with smart defaults and FK detection
- Implemented actionable error messages with fix suggestions

**Stats:**

- 200+ files modified
- 60,000 lines of Rust
- 12 phases, 18 plans
- 2 days from start to ship

**Git range:** `feat(01-01)` -> `feat(12-05)`

**What's next:** v2.0 Rebrand (cancer -> ferro for crates.io publication)

---
