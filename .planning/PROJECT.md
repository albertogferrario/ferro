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
- v0.2.0 shipped.
- 20 workspace crates.

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

### Active

<!-- Current scope. Building toward these. -->

- [ ] v12.0 JSON-UI v2 spec-driven rendering
- [ ] Continuous conceptual coherence across all 20 crates
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

## Next Milestone: v13.0 Road to v1.0

**Goal:** Close the gap between 0.2.0 and v1.0 by addressing known work across the four design dimensions — aesthetic, conceptual, operational, compressive — applied in substance-first investment priority order. v13.0 is a sustained investment program spanning multiple minor releases, not a single-feature milestone.

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

**Relationship to v12.0:** v12.0 (JSON-UI v2) runs first and refines the rendering target. v13.0 begins after v12.0 and may overlap with late v12.0 phases. Target date: none. Multiple minor releases (0.3.x, 0.4.x, 0.5.x) expected across v13.0's span.



## Context

**Current State:**
- ~90,000 lines of Rust across 20 crates
- Phase 150 complete — RichTextEditor component in ferro-json-ui (Quill 2.0.3, SRI-pinned, dual-format Delta+HTML)
- v11.7 shipped: Tailwind Static CSS Pipeline — Safari/WebKit production fix; static CSS with compile-time embedding
- v11.6 shipped: ferro-stripe Capability Refactor — capability-axis module tree, SyncDispatcher, typed events
- v11.5 shipped: Projection Architecture Prep — Renderer trait generalization, renderer relocation, ServiceDef derivation bridge
- v0.2.0 published on crates.io as `ferro-rs`; workspace at 0.2.2+
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

---
*Last updated: 2026-05-01 after Phase 150 (RichTextEditor component)*
