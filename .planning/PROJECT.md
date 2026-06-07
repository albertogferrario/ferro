# Ferro Framework

## What this is

A Rust web framework optimized for AI-assisted authoring. Applications are expressed as data plus intent and rendered through a projection layer. At v1.0 the supported projection target is visual (HTML/CSS via JSON-UI). Additional rendering modalities are a v2.0+ direction.

## Audience

Developers building applications with AI coding agents (Cursor, Claude Code, and similar) as their primary build interface. The framework's surface is shaped for an agent reading it through `ferro-mcp` rather than for hand-typing.

## Core abstraction

**Projection / intent.** Shipped as `ferro-projections` (v9.0): seven structural intents (Browse, Focus, Collect, Process, Summarize, Analyze, Track), signal analyzers, and a `JsonUiRenderer` that turns a `ServiceDef` into a rendered view. The framework is built around this abstraction; v12.0 (JSON-UI v2 + spec-driven rendering) refines its rendering target.

## v1.0 criteria

v1.0 ships when all of the following hold:

1. **Visual stack is feature-complete** for the target use cases.
2. **Projection / intent is validated** through real applications and a synthetic catalog of canonical app classes.
3. **Conceptual coherence pass** across all 20 crates — the surface holds together as one mental model.
4. **Beauty across four dimensions** — aesthetic, conceptual, operational, compressive.

v1.0 does not have a target date.

## Beauty as a design criterion

Four dimensions, all required at v1.0, ordered by investment priority when time is scarce:

1. **Compressive** — small inputs produce disproportionate outputs.
2. **Operational** — setup, errors, edges, and defaults work without surprise.
3. **Conceptual** — the surface holds together as a small mental model.
4. **Aesthetic** — visual quality of rendered output, documentation, and surface.

A weakness in any dimension is a v1.0 blocker.

## Continuous conceptual coherence

Conceptual coherence is enforced at write-time on every phase. Every new feature asks whether it fits the existing surface or whether the surface needs to evolve to absorb it. No phase ships without an answer.

## Day-one v1.0 experience

Install `ferro-cli`, wire an existing AI agent to `ferro-mcp` via standard MCP configuration, and let the agent introspect the project and generate code. `ferro-mcp` is the introspection layer agents use to read routes, models, handlers, and generation context.

## Status

- **Pre-1.0.** Breaking changes acceptable across all 0.x.
- Published on crates.io as `ferro-rs`. Repo public.
- v0.2.33 shipped.
- 25 workspace crates.

---

## Requirements

### Validated

<!-- Shipped and confirmed valuable -->

**Existing Framework Capabilities:**
- ✓ Laravel-inspired architecture with handlers, middleware, routing — existing
- ✓ SeaORM database layer with migrations and model abstraction — existing
- ✓ Session-based authentication with pluggable providers — existing
- ✓ Policy-based authorization with Gate abstraction — existing
- ✓ Validation builder with field-level error messages — existing
- ✓ React/Inertia.js full-stack integration with compile-time validation — existing
- ✓ CLI with make:controller, make:model, make:migration scaffolding — existing
- ✓ MCP server with 30+ introspection tools (routes, models, schema, events, etc.) — existing
- ✓ Auto-generated TypeScript types from Rust models — existing
- ✓ Event dispatcher with async listeners — existing
- ✓ Redis-backed job queue with workers — existing
- ✓ Multi-channel notifications (email, database) — existing
- ✓ WebSocket broadcasting support — existing
- ✓ File storage abstraction (local, S3) — existing
- ✓ Tag-based caching — existing

**v1.0 DX Overhaul (shipped 2026-01-16):**
- ✓ Simplified handler definitions with #[handler] macro — v1.0
- ✓ FerroModel derive macro for automatic SeaORM trait implementations — v1.0
- ✓ ValidateRules derive macro for concise validation rule definitions — v1.0
- ✓ Convention-over-configuration for common scenarios — v1.0
- ✓ MCP intent understanding (domain glossary, app overview) — v1.0
- ✓ Better error context for agent diagnosis — v1.0
- ✓ Relationship and data flow visibility through MCP — v1.0
- ✓ Generation hints embedded in introspection responses — v1.0
- ✓ CLI feature scaffolding with smart defaults and FK detection — v1.0
- ✓ Actionable error messages with fix suggestions — v1.0

**v2.0 Rebrand (shipped 2026-01-16):**
- ✓ Framework renamed from "cancer" to "ferro" for crates.io publication — v2.0
- ✓ All 11 crates rebranded (ferro, ferro-*, ferro-cli, ferro-mcp) — v2.0
- ✓ Documentation and READMEs updated with ferro branding — v2.0
- ✓ Migration guide for existing users — v2.0
- ✓ Publishing checklist for crates.io — v2.0

**v2.0.1 Macro Fix (shipped 2026-01-17):**
- ✓ Fixed hardcoded ::ferro_rs:: paths in proc macros — v2.0.1
- ✓ Simplified macro crate path handling — v2.0.1

**v2.0.2 Type Generator Fixes (shipped 2026-01-17):**
- ✓ Serde case handling with exhaustive enum matching — v2.0.2
- ✓ Prop naming collisions resolved with namespaced names — v2.0.2
- ✓ Contract validation CLI command — v2.0.2
- ✓ DateTime type recognition for chrono types — v2.0.2
- ✓ Nested types generation with fixed-point iteration — v2.0.2
- ✓ ValidationErrors mapped to Record<string, string[]> — v2.0.2

**v2.0.3 DO Apps Deploy (shipped 2026-01-17):**
- ✓ `ferro do:init` command for DigitalOcean App Platform — v2.0.3
- ✓ .do/app.yaml template with service, database, redis config — v2.0.3

**v2.1 Inertia DX & Fixes (shipped 2026-01-17):**
- ✓ JSON Accept header fallback via `render_with_json_fallback()` — v2.1
- ✓ Enhanced SavedInertiaContext documentation with patterns and troubleshooting — v2.1
- ✓ Auto type generation enabled by default in `ferro serve` — v2.1
- ✓ `JsonValue` and `ValidationErrors` utility types in generated TypeScript — v2.1
- ✓ Documentation URLs corrected to docs.ferro-rs.dev — v2.1

**v2.2 CLI Improvements (shipped 2026-02-09):**
- ✓ `ferro db:seed` CLI command for running database seeders — v2.2
- ✓ Unified database commands under `db:` namespace (db:migrate, db:rollback, db:status, db:fresh) — v2.2
- ✓ Generated TypeScript types excluded from version control in project template — v2.2
- ✓ Typed UpdateBuilder pattern for model updates via `model.update().set_field(v).save()` — v2.2
- ✓ Scaffold templates and MCP code templates updated with builder pattern — v2.2

**v3.0 JSON-UI (shipped 2026-02-09):**
- ✓ ferro-json-ui crate with 20-component catalog (serde-tagged enums, shadcn/ui-aligned variants) — v3.0
- ✓ Rust HTML renderer with Tailwind CSS output and XSS prevention — v3.0
- ✓ Data binding with slash-separated JSON paths and 11 visibility operators — v3.0
- ✓ Action system with builder API and callback-based URL resolution — v3.0
- ✓ Layout system with trait-based registry and 3 default layouts — v3.0
- ✓ AI-powered `ferro make:json-view` CLI command with Anthropic API — v3.0
- ✓ 3 MCP tools for JSON-UI (catalog, inspect, generate) — v3.0
- ✓ Comprehensive JSON-UI documentation (6 pages, 2,134 lines) — v3.0

**v4.0 Production Readiness (shipped 2026-02-10):**
- ✓ Session-based authentication with bcrypt hashing, Auth facade, login/register/logout — v4.0
- ✓ AuthUser<T> and OptionalUser<T> handler extractors with middleware guards — v4.0
- ✓ `ferro make:auth` CLI command for scaffolding complete auth system — v4.0
- ✓ API Resources with derive macro, ResourceMap builder, conditional fields — v4.0
- ✓ Pagination envelope (PaginationMeta/Links) and ResourceCollection — v4.0
- ✓ Batch-loaded relationship support via when_loaded/when_loaded_many — v4.0
- ✓ Cache-backed rate limiting with RateLimiter::define() and Throttle middleware — v4.0
- ✓ WebSocket upgrade handler with heartbeat, channel authorization, and whisper — v4.0
- ✓ Actionable error hints with fix guidance in JSON error responses — v4.0
- ✓ 4 new MCP tools (list_resources, list_policies, list_rate_limiters, list_broadcast_channels) — v4.0
- ✓ Comprehensive documentation for auth, API resources, rate limiting, and broadcasting — v4.0

**v5.0 Proximity — JSON-UI Field Test (shipped 2026-02-10):**
- ✓ JSON-UI plugin system with trait-based extensibility, global registry, and asset injection — v5.0
- ✓ Map plugin with Leaflet rendering, fitBounds auto-zoom, and data-attribute config — v5.0
- ✓ Proximity reference app: map-based social network with auth, geo profiles, location posts (separate repo) — v5.0
- ✓ Geospatial proximity queries with bounding-box + Haversine filtering — v5.0
- ✓ Real-time presence via WebSocket broadcasting with channel authorization and presence data — v5.0
- ✓ JSON-UI improvements: Div/Section text elements, input step attribute, SQLite-compatible geo — v5.0
- ✓ End-to-end JSON-UI validation proving zero-JS app development workflow — v5.0

**v5.1 Housekeeping (shipped 2026-02-13):**
- ✓ Env template updated to match all 63 framework env vars — v5.1
- ✓ Template module split from 2,987 to 831 lines across 7 focused modules — v5.1
- ✓ Concerns audit: 6/8 items resolved, priority matrix rebuilt — v5.1
- ✓ Deployment template fixes: health check path, Rust image version — v5.1

**v6.0 ferro-lang — Localization (shipped 2026-02-13):**
- ✓ ferro-lang crate with JSON translation loading, :param interpolation, pluralization — v6.0
- ✓ Per-request locale detection via task_local! with LangMiddleware — v6.0
- ✓ OnceLock validation bridge decoupling 22 rules from ferro-lang — v6.0
- ✓ t()/trans()/choice() helpers auto-booted in Application::run() — v6.0
- ✓ make:lang CLI command + ferro new templates with localization defaults — v6.0
- ✓ list_lang_files MCP tool for locale/key/coverage introspection — v6.0
- ✓ Comprehensive localization documentation (253 lines) — v6.0

**v8.1 API DX Polish (shipped 2026-02-28):**
- ✓ `ferro make:api-key` CLI command for API key generation without code — v8.1
- ✓ Route-level x-MCP customization API (.mcp_tool_name, .mcp_description, .mcp_hint, .mcp_hidden) — v8.1
- ✓ Sensitive field auto-exclusion in make:api with --exclude/--include-all flags — v8.1
- ✓ `ferro api:check` local API verification command — v8.1
- ✓ Post-scaffold guidance with MCP config snippets for Claude Desktop/Code — v8.1
- ✓ Complete API-to-MCP documentation (Quick Start Workflow, Route Customization) — v8.1

**v9.0 Service Projections (shipped 2026-03-23):**
- ✓ ferro-projections crate: ServiceDef → IntentGraph → JsonUiRenderer pipeline — v9.0
- ✓ 7 structural intents (Browse, Focus, Collect, Process, Summarize, Analyze, Track) — v9.0
- ✓ 5 signal analyzers ranking IntentScore — v9.0
- ✓ Killer-feature substrate: projection/intent shipped — v9.0

**v11.7 Tailwind Static CSS Pipeline (shipped 2026-04-21):**
- ✓ `ferro-base.css` pre-built via Tailwind v4 standalone CLI and embedded at compile time (`include_str!`) — v11.7
- ✓ `JsonUiConfig::tailwind_cdn` default flipped to `false`; CDN remains as explicit opt-in — v11.7
- ✓ `JsonUiConfig::stylesheet_urls: Vec<String>` added; default `["/_ferro/ferro-base.css"]` — v11.7
- ✓ `/_ferro/ferro-base.css` static route (exact-string-match, zero-copy `Bytes::from_static`, 24h cache) — v11.7
- ✓ Theme injection changed from `<style type="text/tailwindcss">` to plain `<style>` with CSS variable overrides — v11.7
- ✓ `ferro-theme/assets/default.css` converted from `@theme` to `:root { }` + `@media prefers-color-scheme: dark` — v11.7
- ✓ `ferro make:theme` scaffolder emits plain CSS (`:root { }`) not Tailwind `@theme` syntax — v11.7
- ✓ CI drift job (`ferro-base-css-drift`) ensures committed CSS stays in sync with source — v11.7

**v10.0 JSON-UI Visual Overhaul (shipped 2026-03-26):**
- ✓ Inter Variable font loaded via Bunny Fonts CDN with correct Tailwind v4 --font-sans token — v10.0
- ✓ Three-tier surface elevation (background → surface → card) with WCAG 4.5:1 dark mode contrast — v10.0
- ✓ Typography scale: H1/H2 tight tracking, H3 snug, body relaxed line-height — v10.0
- ✓ Form polish: SVG select chevron, destructive error focus rings, transitions, disabled states — v10.0
- ✓ Focus-visible rings and hover states on all interactive elements — v10.0
- ✓ SVG icons replacing emoji (alerts, bell, breadcrumb, collapsible), shimmer animation, semibold active tabs — v10.0

**v12.2 Frontend Performance Hardening (shipped 2026-06-06):**
- ✓ `ferro-json-ui` `data-lazy-hero` runtime primitive — IntersectionObserver-based promotion of below-the-fold `<video preload="none">` to `preload="auto"`, single-observer fan-out grouped by `data-lazy-hero-margin`, idempotent via `data-lazy-hero-promoted` marker (Phase 182) — v12.2
- ✓ `ferro-bundle` capability (new crate) — in-memory immutable byte blobs registered via `Bundle::new(name, bytes).content_type(ct).with_alias(path).hashed_url()`, served with `Cache-Control: max-age=31536000, immutable` + SHA-256 ETag + `If-None-Match` 304 + 301 alias redirects (Phase 183) — v12.2
- ✓ `ferro::InlineBudget` request extension — `req.inline_budget(key, bytes, fallback_url) -> Decision::{Inline, Preload(url)}` for per-request inline-vs-preload decisioning, configurable threshold via `AppConfig::inline_budget_threshold_bytes` (default 100 KB / `INLINE_BUDGET_BYTES` env var), fire-once `tracing::warn!` per (key, request) with structured fields key/cumulative_bytes/threshold_bytes/fallback_url/route_pattern (Phase 184) — v12.2
- ✓ `ferro::RequestTelemetry` per-key in-process ring buffer — `req.telemetry_record(key, sample)` and `req.telemetry_record_scoped(key, scope, sample)` writers, `RequestTelemetry::snapshot(key, scope) -> Vec<Sample>` reader, `Sample { recorded_at: SystemTime, value: serde_json::Value }`, thread-safe via `OnceLock<DashMap>`, 128 samples per (key, scope), lost-on-restart documented (Phase 184) — v12.2

### Active

<!-- Current scope. Building toward these. -->

- [ ] v12.0 JSON-UI v2 spec-driven rendering
- [ ] v12.1 AI — ferro-ai SDK expansion + AI-assisted scaffolding CLI
- [ ] v13.0 Road to v1.0 — close the gap across all four beauty dimensions
- [ ] Continuous conceptual coherence across all 25 crates
- [ ] Case-study diversification across application domains
- [ ] Synthetic canonical-app-class catalog

## Current Milestone: v12.0 JSON-UI v2 — Spec-Driven Rendering

**Goal:** Pivot ferro-json-ui from Rust-built component trees to flat, JSON-first specs with JSON Schema as the validation contract. AI generates specs constrained by schema; developers write static JSON files validated by the same schema. Handlers become data-only providers.

**Target features:**
- v2 spec format: flat `elements` map + `root` key (replaces nested `Vec<ComponentNode>`)
- Props separation: `props` object per element (cleaner schema validation)
- Formalized catalog: `Catalog` struct with `prompt()`, `validate()`, `json_schema()` — backed by `schemars` derives
- JSON Schema contract: per-component schemas, full spec schema, `ferro json-ui:schema` CLI export
- Schema-driven projections: `Spec::from_service_def()` replaces hardcoded field mapping with schema-based component selection
- Server-side expressions: `$data` and `$template` resolved at render time
- Page loader: framework loads JSON files, validates against schema, merges handler data, renders HTML
- AI constraints: `catalog.prompt()` embeds JSON Schema for structured output; `catalog.validate()` uses `jsonschema` crate
- CLI/MCP updates for v2 format with schema-aware generation

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- New major features (payments, subscriptions, etc.) — focus is consolidation, not feature expansion
- Frontend framework changes — React/Inertia stack stays as-is, JSON-UI is the alternative
- Database driver changes — SeaORM works, no need to replace
- New JSON-UI components — v10.0 was polish, not features
- JavaScript-powered interactivity — JSON-UI is CSS-only; JS features are a separate concern
- Custom icon library — inline SVG strings in Rust sufficient for needed icons
- Client-side state management ($state, $bindState) — server-authoritative model is correct
- Multi-platform renderers (React, Native, PDF) — HTML first, projections later
- RFC 6902 streaming — server builds full spec before responding
- UI Schema hints layer (JSON Forms-style `ui` object per element) — adds developer ergonomics but not needed for AI generation; revisit post-v12.0
- Expression language beyond `$data` and `$template` — no `$if`, `$for`, `$state`. Inner platform effect is the #1 strategic risk in SDUI (Airbnb, DoorDash, Lyft all learned this). Keep expressions minimal.
- Full catalog schema in AI prompts — 36-component oneOf produces 40-80 KB schema, too large for system prompts. Use per-component schemas for AI; full schema only for validation.
- JSON Typedef (JTD) as alternative schema format — simpler than JSON Schema but adds a second format. Revisit only if third-party plugin authors struggle with JSON Schema.
- Multimodal projection (audio/voice/physical) — v2.0+ direction
- Bundled agent UX — `ferro-mcp` plus the user's existing agent is the supported workflow

## Upcoming Milestone: v12.1 AI — ferro-ai SDK & AI as Projection Consumer

**Goal:** Expand `ferro-ai` into a production-grade, provider-agnostic AI SDK and make AI a first-class consumer of the projection / intent core. The killer feature: `ferro ai:make <description>` produces a typed `ferro_projections::ServiceDef` — the universal projection contract — and the existing rendering pipeline (`ferro-json-ui` renderer, `ferro-mcp` introspection renderer, future modality renderers) covers everything downstream. AI does not recreate pre-projections multi-file scaffolding; it generates the input the projection layer already knows how to render.

**Target features:**

*SDK (ferro-ai expansion — foundation):*
- Multi-provider LLM client: Anthropic, OpenAI, Groq (OpenAI-compatible), Ollama via a provider-agnostic trait; config from env vars
- Structured outputs: `ferro_ai::complete::<T>()` returns typed Rust structs via JSON Schema
- **`ServiceDef`-aware schema normalizer**: when the LLM completes into a `ServiceDef`, the schema locks output to valid projection shapes (`FieldMeaning`, `Intent`, `Cardinality`, `ActionDef` / `GuardDef`, `StateMachine`). Structural guarantee that AI cannot drift from the intent system.
- Tool calling: register Rust functions as AI tools; SDK dispatches tool-use calls automatically with a hard `max_iterations` guard
- Embeddings + cosine similarity helpers; optional pgvector integration for semantic search

*Streaming:*
- SSE streaming support so handlers can push LLM tokens to the browser as they arrive
- `ferro-json-ui` streaming text component for token-by-token display

*AI CLI commands (built on SDK, framed as projection produce / consume / render):*
- `ferro ai:make <description>` — natural language → typed `ferro_projections::ServiceDef`, using ferro-mcp introspection as context. No multi-file scaffold output; no `ScaffoldPlan` intermediary.
- `ferro ai:explain <route|model|service>` — projection-framed explanation (`Intent`, `FieldMeaning`, `ActionDef` / `GuardDef`, `StateMachine`); plain code prose is the fallback only when no `ServiceDef` is found.
- `ferro make:json-view` v2 — first concrete `Renderer` over a ServiceDef produced by `ai:make`. Now unblocked since v12.0 shipped 2026-05-19. Closes the produce-then-render loop end-to-end via a projection-roundtrip test (NL description → `ServiceDef` → rendered JSON-UI spec).

**Phases:** 165-173 (per ROADMAP.md v12.1 AI section)

**Relationship to v12.0:** v12.0 (JSON-UI v2, phases 115-121) shipped 2026-05-19. The `Renderer` trait + `Catalog` + spec schema from v12.0 are exactly the surfaces `make:json-view` v2 consumes — v12.1 builds AI on top of the projection contract v12.0 made addressable.

## Upcoming Milestone: v13.0 Road to v1.0

**Goal:** Close the gap between 0.2.x and v1.0 by addressing known work across the four design dimensions — aesthetic, conceptual, operational, compressive — applied in substance-first investment priority order. v13.0 is a sustained investment program spanning multiple minor releases, not a single-feature milestone.

**Scope by priority:**

*Compressive — validation of projection / intent (priority 1):*
- Gestiscilo migration to projection-driven rendering as the first real-world validation
- Synthetic canonical-class catalog with regression tests across the seven intents
- Agent-success-rate measurement via `ferro-mcp` introspection
- Time-to-working-app benchmark
- Intent vocabulary cross-modality sketch experiment

*Operational — polish and documentation (priority 2):*
- MCP integration documentation for Claude Code, Cursor, and common agent runtimes
- Projection MCP tool description audit (`list_projections`, `inspect_projection`, `render_projection`, `validate_projection`, `projection_coverage`)
- Projection authoring guide via MCP introspection
- Agent-assisted deploy workflow end-to-end walkthrough
- Projection-driven starter template option for `ferro new`
- Iteration loop ergonomics investigation
- `ferro doctor` multi-bin `--bin <pkg>` auto-resolution

*Conceptual — coherence pass (priority 3):*
- Systematic coherence audit across all 20 crates — first pass since Phase 113
- Cross-cutting consistency: naming, error patterns, middleware shapes, CLI verbs, file layouts, module organization
- Refactor outlier crates into prevailing patterns

*Aesthetic — incremental polish (priority 4):*
- mdBook custom theme (colors, typography, code block styling)
- Crates.io README polish (shields, visual hierarchy, clear CTA)
- GitHub repo social preview image
- Simple logo / wordmark

**Relationship to v12.1:** v12.1 (AI features) runs between v12.0 and v13.0. v13.0 begins after v12.1 and may overlap with late v12.1 phases. Target date: none. Multiple minor releases (0.3.x, 0.4.x, 0.5.x) expected across v13.0's span.



## Context

**Current State:**
- ~90,000 lines of Rust across 24 crates
- Phase 181 complete (2026-05-31) — JSON-UI inline error rendering: `JsonUi::render` / `render_with_errors_config` now clone the spec and `merge_data(data.clone())` before `resolve` so `$data` bindings against handler-supplied data resolve correctly (Fix A — D-02 root cause 1, two call sites in `framework/src/json_ui/mod.rs`); `attach_errors` per-field branch writes singular `error: String` (first message wins) matching `InputProps.error: Option<String>` shape exactly (Fix B — D-02 root cause 2, `ferro-json-ui/src/resolve.rs:178-201`); the `else if all` full-bag branch is intentionally unchanged. Checkbox / CheckboxList / Switch / Input(file) brought to error-state class+ARIA parity with Input/Select per D-06: `border-destructive` + `focus-visible:ring-destructive` (or `peer-focus:ring-destructive/30` for Switch) when `has_error`, `aria-invalid="true"` + `aria-describedby="err-{field}"` on the correct ARIA target (input for Checkbox/Switch/Input-file, fieldset for CheckboxList), and `id="err-{field}"` added to all four error `<p>` blocks. Cross-repo grep audit (D-08) confirmed bucket B empty — no gestiscilo consumer reads the pre-fix plural `errors` shape (clean break, no shim). Docs page `docs/src/json-ui/forms.md` (146 lines) covers four authoring patterns per D-09: blessed `render_validation_error`, manual `$data` escape hatch, flash round-trip via `req.old(...)`, cross-field summary via `toast_validation`. Test corpus: 4 new pipeline integration tests + 4 new render-form unit tests (1 per D-06 variant), all RED→GREEN. Two existing render tests upgraded from spec-only to html_body + p-tag assertions (Wave 0). Code review found 1 warning (deferred — `render_json` / `render_json_with_errors` JSON paths skip `merge_data`; no current consumer hits this with `$data` error bindings; tracked as RESEARCH open question 1) and 2 info items. Full workspace gate: 2812 tests passing. One human-verification item remains in 181-HUMAN-UAT.md: gestiscilo browser UAT on 5 representative forms (release-time gate per `feedback_friction_loop_release_cadence.md`).
- Phase 180 complete (2026-05-30) — declarative `#[action]` handler primitive. `framework::http::action` defines `ActionError` / `ActionKind` / `FlashVariant` / `ActionResult = Result<(), ActionError>` / `IntoActionError` / `ActionResultExt` and the `handle_action_result` runtime helper (303 redirect, session flash, T-180-02 same-origin guard on success+error paths, T-180-03 control-char sanitizer). `Request::flash` / `Request::redirect_to` setters carry success-side overrides (D-02 revised). `ferro-macros::action` proc-macro wraps user bodies in `async move { ... }.await` so `?` propagates to `ActionResult`. Test corpus: 14 inline unit tests + 9 trybuild fixtures (6 pass + 3 fail with locked .stderr) + 13 integration tests covering happy/error/override paths, T-180-02 open-redirect mitigation on both sides, T-180-03 percent-encoding, query-string separator and flash-key encoding regressions (WR-01/WR-02 from REVIEW.md). Docs page at `docs/src/the-basics/action-handlers.md`; MCP `action_handler` template registered in `ferro-mcp/src/tools/code_templates.rs`. All 8 locked decisions D-01..D-08 honored. Full workspace gate: 2769 tests passing.
- Phase 176 complete (2026-05-21) — v12.0.2 JSON-UI runtime patches from gestiscilo β booking↔staff binding field test: Card gained optional `badge` and `subtitle` render slots with schema-includes / serde / render coverage; Grid visibility regression tests pinned both branches (true / false / consumer chip-strip mirror) and confirmed the element-level evaluator is correct (F9 closed as could-not-reproduce, no production code change). Docs gained Card slot table rows and a `#### Visibility` subsection clarifying universality.
- Phase 160 complete — v1 JSON-UI API deletion: all `JsonUiView` / `Component` / `ComponentNode` / `PluginProps` surface removed from ferro-json-ui, framework, and ferro-mcp; `migration_v1_to_v2_templates` MCP category deleted; `application_info::scan_json_ui_specs` rewritten to count v2 JSON spec files; protocol docs reframed; ferro workspace + gestiscilo cross-repo green. Phase 161 (v12.0 merge + single end-of-loop publish) cleared to start.
- Phase 155 complete — ferro-projection v0.2.33: live read-model runtime (subscribe to domain events, persist per-key snapshots, broadcast deltas)
- v11.11 shipped: Resource Reservation & Live Read-Model Primitives — ferro-orm GuardedUpdate (Ph 152), ferro-audit (Ph 153), ferro-reservation (Ph 154), ferro-projection (Ph 155)
- v11.7 shipped: Tailwind Static CSS Pipeline — Safari/WebKit production fix; static CSS with compile-time embedding
- v11.6 shipped: ferro-stripe Capability Refactor — capability-axis module tree, SyncDispatcher, typed events
- v11.5 shipped: Projection Architecture Prep — Renderer trait generalization, renderer relocation, ServiceDef derivation bridge
- v10.0 shipped: JSON-UI Visual Overhaul
- v9.0 shipped: Service Projections — projection / intent substrate
- Phase 117 shipped: Catalog & JSON Schema — machine-readable `Catalog` with 39 built-in components, compiled jsonschema validator, per-component schema accessor, concise `prompt()` output (≤ 8 KB), full spec schema export via `ferro json-ui:schema` CLI, `COMPONENT_CATALOG` const retired
- Phase 117.1 shipped: Schema-Driven Projections — `Spec::from_service_def()` bridges ferro-projections and ferro-json-ui v2 via catalog-verified meaning→component dispatch, intent→layout template resolution, and two-pass generate-then-validate; legacy `field_map.rs` and `relationship_map.rs` deleted
- v0.2.35 published on crates.io as `ferro-rs`
- Pre-1.0; breaking changes acceptable
- Sample application (app/) demonstrating Inertia integration
- Comprehensive MCP introspection (35+ tools) — this is the v1.0 product surface
- `ServiceDef::from_model()` derivation bridge — agents can generate projections from model metadata without hand-writing builders

**Tech Stack:**
- Rust 2021 edition
- Axum web framework
- SeaORM database layer
- React/Inertia.js frontend (full-stack SPA option)
- JSON-UI server-side rendering (zero-JS option, projection target)
- Redis for queue/cache/broadcast

**Primary use case:** Building applications with AI coding agents. Ferro's surface is shaped for an agent reading it through MCP, with a human directing.

Reference codebase documentation in `.planning/codebase/`:
- ARCHITECTURE.md — Layer breakdown and request lifecycle
- STACK.md — Dependencies and tooling
- PATTERNS.md, HOTSPOTS.md, TESTING.md, CONVENTIONS.md, DOCUMENTATION.md

See also `.planning/VISION.md` for design philosophy.

## Constraints

- **Compatibility**: Existing sample app works with framework
- **Rust Edition**: 2021 edition, no nightly-only features
- **Coherence**: Every phase enforces conceptual coherence at write-time

## Key Decisions

<!-- Decisions that constrain future work. Add throughout project lifecycle. -->

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Designed for AI-assisted authoring | Agent reads framework surface through MCP; human directs | ✓ Good |
| Projection / intent as core abstraction | Framework is built around data + intent projected onto a medium | ✓ Good |
| Visual-only at v1.0, additional modalities at v2.0+ | Ship the supported modality first | ✓ Good |
| ferro-mcp as the introspection layer | Agents read routes, models, handlers, and context through MCP tools | ✓ Good |
| No bundled agent UX | Users bring their own agent (Cursor, Claude Code, etc.) | ✓ Good |
| Continuous coherence discipline | Coherence enforced at write-time per phase, not retro-patched | ✓ Good |
| Validation through real-world apps and synthetic catalog | Surfaces gaps in projection / intent | ✓ Good |
| Substance-first beauty ordering | Compressive → operational → conceptual → aesthetic when time is scarce | ✓ Good |
| Breaking changes acceptable | Pre-1.0; no backwards compatibility constraint allows cleaner APIs | ✓ Good |
| FerroModel derive on entities | Apply derive to entity files (auto-generated) not model files | ✓ Good |
| ValidateRules not Validate | Avoid conflict with validator crate's `Validate` derive | ✓ Good |
| Tool vs Resource for MCP | Implemented features as tools rather than MCP resources for simpler agent consumption | ✓ Good |
| Rebrand to "ferro" | Concise, industrial name suited to a framework crate | ✓ Good |
| JSON fallback opt-in | `render_with_json_fallback()` per route for security | ✓ Good |
| accepts_json() on InertiaRequest | Framework-agnostic Accept header detection | ✓ Good |
| Docs URL: docs.ferro-rs.dev | Dedicated subdomain for documentation | ✓ Good |
| db: namespace for CLI | All database commands under unified db: prefix | ✓ Good |
| UpdateBuilder consumes model | Takes self for simpler ownership, matches create pattern | ✓ Good |
| Option<Option<T>> for nullable fields | None=unchanged, Some(None)=clear, Some(Some(v))=set | ✓ Good |
| Exclude frontend/src/types/ | Directory-level gitignore over individual files | ✓ Good |
| Serde tagged enum for Component | Clean JSON with `{"type": "Card", ...}` | ✓ Good |
| Callback-based URL resolver | Keeps ferro-json-ui decoupled from framework route registry | ✓ Good |
| Sonnet default for AI generation | ~5x cost reduction vs Opus for CLI code generation | ✓ Good |
| json_ui_generate returns context | Consuming agent IS the LLM, avoids double-LLM calls | ✓ Good |
| COMPONENT_CATALOG in ferro-json-ui | Single pub const shared by ferro-cli and ferro-mcp via direct dependency | ✓ Good |
| Separate ferro-lang crate | Follows ferro-cache/ferro-events pattern, keeps i18n decoupled | ✓ Good |
| Pre-merge fallback at load time | O(1) runtime lookup, no fallback chain per request | ✓ Good |
| OnceLock validation bridge | Zero coupling: validation has no ferro-lang dependency | ✓ Good |
| task_local! for locale | Async-safe per-request context, matches session middleware pattern | ✓ Good |
| fn pointer for TranslatorFn | No state capture needed, simpler than Box<dyn Fn> | ✓ Good |
| lang::init() after config_fn() | User can override LangConfig before translator loads | ✓ Good |
| serde_json preserve_order | ResourceMap needs insertion-order field output | ✓ Good |
| Fail-open rate limiting | Availability over strictness; never block on infra failure | ✓ Good |
| WS upgrade before middleware | Upgrade needs raw hyper Request, not framework Request | ✓ Good |
| Always include error hints | Errors are developer-facing APIs, not user-facing | ✓ Good |
| 401 via FrameworkError::domain | 401 is authentication failure; Unauthorized is 403 | ✓ Good |
| Tailwind v4 --font-sans namespace | v3 used --font-family-sans which v4 ignores; token fix enables font rendering | ✓ Good |
| Three-tier surface hierarchy | background < surface < card; persistent frames stay background | ✓ Good |
| focus-visible: over focus: | Keyboard-only rings; mouse clicks don't trigger visual noise | ✓ Good |
| Inline SVG via concat! macro | Avoids data URI which fails in CDN mode; self-contained per component | ✓ Good |
| Dark mode pair 6 trade-off | 4.45:1 accepted (0.05 below AA) — lowering primary L breaks pair 5 | ⚠️ Revisit |
| Shimmer CSS injected inline | Keeps skeleton self-contained; no external stylesheet dependency | ✓ Good |
| Flat element map for v2 specs | Better for AI generation, streaming, human readability | Planned |
| Props object separation in v2 | Clean boundary enables schema validation | Planned |
| Server-side expressions only | `$data` and `$template`; server-authoritative model | Planned |
| Clean break: delete v1 entirely | Pre-1.0; no backward compat layer | Planned |
| JSON Schema as validation contract | `schemars` already on props structs; enables AI structured output + standalone schema | Planned |
| Per-component schema export | Targeted AI generation; full catalog schema is too large for prompts | Planned |
| Schema-driven projections replace field_map.rs | Projections and catalog stay consistent by construction | Planned |
| Hard cap on expression language | Inner platform effect is the #1 strategic risk in SDUI | Planned |
| Max nesting depth: 3 levels | All production SDUI systems converge here; keeps generation reliable | Planned |

| Static CSS pipeline replaces Tailwind CDN runtime | `@tailwindcss/browser@4` is dev-only per Tailwind docs; fails silently on Safari/WebKit; field-confirmed on gestiscilo.it | ✓ Good |
| `stylesheet_urls` replaces `tailwind_cdn` as primary CSS injection | Vec<String> is composable; apps append their own token files; CDN flag remains for explicit opt-in | ✓ Good |
| `html_escape()` on `stylesheet_urls` values before href emission | Defense-in-depth for app-provided URLs; ASVS L1 V5.3 | ✓ Good |
| Exact-string-match for `/_ferro/ferro-base.css` route | No path parsing → path traversal structurally impossible | ✓ Good |
| ServiceDef::from_model() derivation bridge | Agents generate projections from model metadata; no hand-written builders | ✓ Good |
| StripeEvent::from_raw pattern-matches EventObject | No JSON re-serialization; type guard is `event.type_` check before object match | ✓ Good |
| BoxedHandler returns (bool, Result) tuple | `bool` flag distinguishes "no handler matched" from handler returning Ok(()); enables unknown-event logging | ✓ Good |
| Missing SyncDispatcher → Err(JobFailed) not panic | Queue workers survive misconfiguration; recoverable error lets queue mark job failed and continue | ✓ Good |
| amount_total_cents: i64 with zero-means-absent doc | Zero maps to absent Stripe field on free/setup sessions; callers must not use field alone to assert payment | ✓ Good |
| Schema-driven projections replace field_map.rs | Projections and catalog stay consistent by construction | Planned |

---
*Last updated: 2026-06-07 — Phase 189 complete (ferro-stripe manual capture): `CheckoutBuilder::manual_capture()` with pre-flight mode guard, `payment_intent` capability module (capture/cancel/retrieve with positive-amount guard), `StripePaymentIntentAmountCapturableUpdated` + `StripePaymentIntentCanceled` typed events with golden fixtures, Connect destination-charge composition via single merged `payment_intent_data` construction, Manual Capture docs with ferro-reservation hold/commit/release correspondence table. Phase-close gate green (fmt/clippy/full test suite); code review WR-01/WR-02 fixed. Previous footer: Phase 181 complete (JSON-UI inline error rendering, 2026-05-31; one human-verification item remains in 181-HUMAN-UAT.md).*
