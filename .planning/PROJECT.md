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
- v0.2.55 shipped (2026-06-13).
- ~26 workspace crates.
- v13.0 Compressive Validation complete (Phases 207–211): COMP-02 (207), COMP-05 (208), COMP-01 Slice A (209), COMP-03 (210), COMP-04 (211). The benchmark/harness phases surfaced real weaknesses by design — notably COMP-04 found the published 0.2.55 scaffold does not compile (scaffold↔library API drift).
- v13.3 Scaffold↔Library Parity complete (Phase 214, SCAF-01–05): the scaffold↔library drift COMP-04 found is fixed — `error_response!` macro and `ActiveValue` exported from the `ferro` facade, queue routed through `ferro::queue::*`, and the `--api` + full-stack controller, auth, and job templates corrected to emit only published-facade symbols. A two-layer CI guard (per-PR workspace path-dep `scaffold_builds_against_workspace_ferro` test + release-time published-artifact Docker smoke job) makes a non-compiling scaffold a pipeline failure. The full scaffold sequence now `cargo build`s exit 0.
- v13.1 CRUD Handler Proc Macros complete (Phase 212, CRUD-01–06): `#[resource_get]` / `#[resource_post]` fold the tenant-scoped CRUD prelude (typed param + `current_tenant()` + tenant-scoped lookup + 404/303-on-miss) into one route attribute while keeping tenant + resource as real typed params; they inline the handler/action boilerplate (no nested attribute). Backed by a `TenantScoped` trait (cross-tenant reads impossible by construction) and `Validator::validate_or_redirect(url)`. trybuild suite (pass + compile-fail fixtures), facade exports, 0.2.56 bump. With this, the v13.x batch scoped so far (v13.0/v13.1/v13.2/v13.3) is complete; nothing in v13.x is published yet beyond v13.2's 0.2.55.
- v14.0 Channel Projection complete (Phases 215–216, CHAN-01–04): the first production non-visual `Renderer` ships. Phase 215 extended the renderer-free surface (`BaseContext.evaluated_guards` + `verbosity`, `Intent::label()`, `Error::NoIntents`); Phase 216 added `FieldDef.render_hint` (`AltText`/`Skip`, additive, serde-backward-compatible) and a new `ferro-text` output crate whose `TextRenderer` projects the *same* `ServiceDef` to deterministic conversational text — per-intent strategies for Browse/Collect/Process/Summarize/Track, guard-filtered (absent key renders, explicit `false` hides), verbosity-aware, with a defined Focus/Analyze fallback. Re-exported via the `ferro` facade behind the `projections` feature; registered in publish.yml Wave 1b; `insta` snapshots over the COMP-05 `approval_workflow` anchor pin both guard states. The projection/intent abstraction is now validated against a non-screen modality. ~27 workspace crates.

## Current Milestone: v16.5 JSON-UI Design System

**Goal:** Complete the design system above the token layer — density/motion/focus-ring tokens with opinionated defaults, a canonical variant vocabulary across all 47 builtin components, and composition patterns codified as machine-readable, intent-keyed lint rules enforced at the agent-authoring boundary (an agent reads the system through ferro-mcp, authors a spec, and `design_lint` validates conformance before human review).

**Target features:**
- Token vocabulary v2 (23 → 30 slots: `--spacing`, motion ×4, `--color-ring`, `--font-display`) with defaults — every v1 theme stays valid.
- Default theme refreshed to the documented design language.
- Canonical `variant`/`tone`/`size` enums + interactive-state quality bar across all components.
- `Spec.design` field + `design::lint` rule engine + `ferro design:lint` CLI.
- `design_lint` MCP tool, catalog/generation-context extensions, `docs/src/design-system/` chapter; single publish at the end.

**Active requirements:** DS-01..08 (`.planning/REQUIREMENTS.md`). Anchor spec: `docs/superpowers/specs/2026-07-03-json-ui-design-system-design.md`. Consumer-paired with the reference application's adoption phase, gated on the final publish.

**Progress:** Phase 251 complete (2026-07-03, verified 11/11 incl. visual UAT) — one canonical component vocabulary: shared `Variant` (primary/secondary/outline/ghost/destructive), `Tone` (neutral/success/warning/destructive), `Size` (sm/md/lg) enums replace all per-component copies (Card `variant`→`appearance`, `link`→ghost, Badge collapses to tone-only, action-level `variant`→`tone`); interactive-state pass via shared `render/classes.rs` constants (`focus-visible:ring-ring`, `duration-fast/base/slow` + `ease-base` tiers, uniform disabled, toast dismissal `transitionend`-driven); D-19 schema-walking drift guard ($ref-resolved, zero exclusions) + retired-prop lint in `Catalog::validate`; consumer migration table in `docs/src/json-ui/components.md`; `ferro-base.css` regenerated; full CI-exact gate green; code review 0 critical, 5/5 warnings fixed. Prior: Phase 250 (tokens v2, 30 slots, cool-tinted hue-250 default theme). Phases 252–253 (design lint, MCP surface + publish) remain.

## Shipped Milestone: v16.3 MCP CRUD Data Surface (Track A) (shipped 2026-06-24, 0.2.80)

**Goal:** A projection that opts in derives a complete, safe, tenant-scoped CRUD interface (create / read+query / update / soft-delete) as MCP tools with zero hand-written tool code — the foundational track of the broader MCP capability program (Tracks A–D).

**Target features:**
- Opt-in CRUD derivation (`.creatable`/`.updatable`/`.deletable` + `.mcp_write_ability`) — declaration surface + `validate()` write-ability rule shipped (`5cb17d60`).
- `create_`/`update_`/`delete_<svc>` tools with auto-derived field-set rules (exclude Identifier/CreatedAt/tenant; `Status` workflow-only under a StateMachine).
- Query polish on `list_`: range/comparison filters, sort, pagination (atop existing equality filters).
- Soft-delete (`deleted_at`) + confirmation gating.
- Write authz: `read_write` scope + `.mcp_write_ability` Gate + server-side tenant injection.
- `derive_crud_plan` extending the shipped `framework::write` kernel (231/232) — not a rebuild.

**Active requirements:** CRUD-01..07 (`.planning/REQUIREMENTS.md`). Anchor spec: `docs/superpowers/specs/2026-06-23-projection-crud-data-surface-design.md`. Builds on shipped v16.0 (231/232) + the Phase 205 `content[]` structured envelope.

**Progress:** All phases (239–243) complete — milestone delivered. Phase 239 added the soft-delete substrate (`deleted_at` migration, `resolved_table`/`resolved_soft_delete_column`, `is_server_injected_field`, read-path `deleted_at IS NULL`); Phase 240 added CRUD input-schema derivation + `list_` query polish (CRUD-01/02/04 — `is_write_excluded_field`, `build_create/update/delete_input_schema`, `is_range_filter_field`, `<field>__{gt,gte,lt,lte,ne,in}` + `sort`). **Phase 241 (CRUD-03/06) made the `create_/update_/delete_` tools executable**: `derive_crud_plan(svc, verb, inputs) -> CrudPlan` (pure serializable enum, mirrors `derive_transition_plan`/`TransitionPlan`) in `ferro-projections`; the single `framework::write::dispatch_write` kernel extended with one `crud_plan: Option<&CrudPlan>` parameter + a framework-provided generic `execute_crud_plan` (parameterized SQL, INSERT/UPDATE/soft-delete) — no second dispatcher, reusing guards/idempotency/audit/override-hook/confirmation unchanged. The `not_yet_implemented` stub is replaced with real derive→dispatch through the Phase 205 `structured` envelope; `delete_<svc>` soft-deletes (`deleted_at`), is confirmation-gated via synthesized `request_confirm_delete_<svc>`/`confirm_delete_<svc>` (reusing `ConfirmationStore` + token binding), and is filtered from `list_`. `CrudPlan` carries a `tenant_column: Option<TenantColumn> = None` slot (the Phase 242 extension point). Phase 242 wired write authz (`read_write` scope + `.mcp_write_ability` Gate) + server-side tenant injection + cross-tenant non-disclosure. Phase 243 flipped the sample app's `order` projection to CRUD, drove create→list→update→delete through an in-process MCP e2e harness (per-verb Phase 205 structured-envelope regression guard, MCP↔visual parity, confirmation flow), and brought `ferro-mcp` `code_templates`/`generation_context` + `docs/src` to the same quality bar (new `projection_crud` template + `MCP CRUD Opt-In` docs section matching the shipped projection). The full workspace gate is green; one human-UAT smoke (live `:8090/mcp` drive) remains intentionally manual.

## Next Milestone (queued): v16.4 Work Distribution — `#[offload]` Service Methods

**Status:** Queued behind v16.3 Phase 243. v16.3 stays current until 243 closes; v16.4 is
phases 244–249 (planned/executed after).

**Goal:** A `#[service]` trait method marked `#[offload]` becomes a distributable unit of work
with zero hand-written queue plumbing — the framework derives the `ferro-queue` Job, serializable
payload, and a typed result handle from the method signature, runs it on a horizontally scalable
worker, and streams the result back via the read-model + broadcast path. Work distribution as the
operational analog of projection/intent: one trait declaration is the in-process contract, the
wire payload, the result-path, and the MCP spec.

**Target features:**
- `#[offload]` macro deriving the `ferro-queue` Job + serializable payload from the method signature.
- Typed result handle + compile-time serializable-contract enforcement (which doubles as the module-isolation boundary).
- Fire-and-forward result path: worker → `ferro-projection` snapshot → `ferro-broadcast` delta (request never blocks).
- Deployable `ferro worker` runtime, runnable at N replicas — capacity scales by running more workers.
- `ferro-mcp` introspection of offloadable methods + scaling docs.

**Scope decision (CTO):** build the scalable primitive, defer the auto-deciding. Autonomous
machine lifecycle / scale-to-zero (KEDA, CRIU, Nomad, WASM isolates) is cost-optimization, not
capacity — parked as a 2.0 direction. The many-user requirement is met by stateless replicas +
data-tier scaling + cache + replicable workers, not by a framework autoscaler. Anchor spec:
`docs/superpowers/specs/2026-06-24-offload-work-distribution-design.md`.

## Shipped Milestone: v16.0 Write-Boundary AX — StateMachine-Derived Executor (shipped 2026-06-16)

**Goal:** Eliminate the "declare twice" duplication on the projection write path — derive a default write executor from the `ServiceDef` StateMachine the framework already knows, with an override hook for the app-specific 20% (side effects, related-record writes, custom guards).

**Target features:**
- **StateMachine-derived default executor** — a write whose `ActionDef.transition_trigger` names a StateMachine transition dispatches through a framework-generated executor (state read → guard check → transition → persist) with no hand-written `match` re-encoding the transition facts.
- **Override hook for the 20%** — app-specific side effects (related records, notifications, custom guards) attach to the derived executor without replacing it, so the common path stays declaration-only.
- **Sync-by-construction** — the executor and the StateMachine cannot drift, because the executor is generated from the StateMachine; this removes the class of "declared twice, fell out of sync" bugs.

**Why this milestone:** the projection/intent killer feature's READ path is complete (visual, text, and MCP renderers) and Phase 213 closed the render-content gaps; the WRITE path still re-imports the imperative surface projection was meant to eliminate (verified 2026-06-16: `ferro-projections` has no executor-derivation machinery at 0.2.65 — the hand-written `WriteDispatcher` re-encodes StateMachine transitions in a `match`). This is the last load-bearing gap in the write path.

**Coherence constraint:** the derived executor must read FROM the existing StateMachine / `ActionDef` declarations — it does not introduce a parallel imperative control surface. Projection/intent stays the single source of truth.

**Out of scope:** the operating-AX side (NL description quality ≡ classification accuracy), gated on a funded COMP-03 live run; the optional projection `body` slot (free-form rich content); consumer-app adoption (gestiscilo), a separate consumer-repo effort.

## Shipped Milestone: v15.0 Agent-Operable App (Consumer MCP)

**Goal:** A tenant can operate a live ferro application through a per-tenant MCP endpoint whose tools are derived from the app's projections — reading and acting on real data through an agent rather than the dashboard. Extends the projection/intent abstraction to a fourth `Renderer` target (`ServiceDef → MCP tools`) and adds the inbound message → action loop. Validated against gestiscilo.

**Target features:**
- **Projection → MCP tools** — each `ServiceDef`'s queries and guarded actions are projected into callable, tenant-scoped MCP tools from the same definition the visual and text renderers consume.
- **Write/act via MCP** — create, update, and state-transition records through those tools, guard-filtered by reusing v14.0's `evaluated_guards` so an agent is only offered actions its tenant may perform.
- **Inbound intent loop** — natural-language message → intent classification (`ferro-ai`) → action dispatch, completing the listen/act half deferred from v14.0.
- **Per-tenant API-key auth** — the endpoint and its tools are scoped to a tenant by API key, building on the `TenantScoped` contract so cross-tenant access is structurally impossible.

**Builds on:** the v12.6 consumer-MCP OAuth endpoint (this milestone adds the projection-derived tools and the action loop, not a new endpoint), the v14.0 projection/intent + guard-evaluation surface (`evaluated_guards`, `ServiceDef`), `ferro-ai` structured classification, and the v13.1 `TenantScoped` isolation contract.

**Out of scope:** the remaining channel renderers (voice, structured-API, mobile `device_class` / chart-card) — those remain a separate channel milestone. Consumer-app adoption (e.g. migrating gestiscilo's own views) is a consumer-repo follow-up; ferro phases deliver the framework capability plus synthetic validation only.

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

**v16.0 Write-Boundary AX / StateMachine-Derived Executor (shipped 2026-06-16):**
- ✓ Default write executor derived from the `ServiceDef` StateMachine — `TransitionPlan` + `derive_transition_plan`, no hand-written `match` (EXEC-01) — v16.0
- ✓ Server-side guard re-evaluation at execution, fail-closed (EXEC-02) — v16.0
- ✓ Post-persist override hook for app-specific side effects; common path declaration-only (EXEC-03) — v16.0
- ✓ Registration/boot-time drift gate — `validate()` rejects undeclared-transition references (EXEC-04) — v16.0
- ✓ Single-source write surfaces — one `framework::write` kernel backs MCP + the new visual `POST /{service}/{action}` write path (EXEC-05) — v16.0

**v15.0 Agent-Operable App / Consumer MCP (shipped 2026-06-14):**
- ✓ Per-tenant API-key auth + tenant/guard context on the MCP endpoint (AMCP-01/02) — v15.0
- ✓ `ActionDef`-derived MCP write tools, guard-filtered (AMCP-03) — v15.0
- ✓ Tenant-scoped write dispatch with server-side guard re-evaluation, idempotency, and audit (AMCP-04) — v15.0
- ✓ Confirmation gating for destructive actions via `ferro-ai::ConfirmationStore` (AMCP-05) — v15.0
- ✓ Inbound natural-language intent loop, CI-testable without live-LLM spend (AMCP-06) — v15.0

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

## Shipped Milestone: v12.6 Consumer App MCP (Browser Login)

**Goal:** A deployed ferro application serves its own OAuth-protected MCP endpoint so a consumer agent can authenticate through the browser and use the application's projections as per-tenant tools.

**Target features:**
- `ferro-mcp-server` (new output crate) — an `McpRenderer` mapping a projection/intent to an MCP tool, mirroring `JsonUiRenderer` in `ferro-json-ui`; `ferro-projections` stays renderer-free.
- Application-served MCP endpoint over Streamable HTTP (`initialize` / `tools/list` / `tools/call`).
- Browser-based OAuth 2.1: the application is its own authorization and resource server, reusing existing login plus a consent step; dynamic client registration, PKCE, audience-bound tokens.
- Opt-in projection exposure rendered as a read tool, scoped per tenant through the existing multi-tenant middleware and policy layer (no parallel permission system).
- Dogfood acceptance: a real MCP client completes a browser login against a live consumer application and lists one projection's data, tenant-scoped.

**Walking skeleton first:** read-only, one opt-in projection. Write intents, multi-projection auto-exposure, MCP-specific scopes, and development-time MCP experience are deferred to later milestones.

**Design spec:** `docs/superpowers/specs/2026-06-10-consumer-app-mcp-browser-login-design.md`.

## Shipped Milestone: v12.5 Projection Checkpoint

**Goal:** Close the agent write→verify loop. A projection-anchored checkpoint walks the intent-slice spine, dispatches to the existing validators at each seam, runs the one seam no validator covers today (projection field → model column), and returns a single structured verdict with ranked next steps — honest about coverage, and closing by default after generation.

**Target features:**
- `checkpoint_projection` MCP tool — anchored on a `ServiceDef` name; walks a 5-seam spine (well-formed, field→column, action→route, rendered view valid, props→contract); aggregates into one `pass`/`warn`/`fail` verdict with ranked `next_steps`.
- Field→column seam (the new check) — resolves the projection to its source model via the `src/projections/` ↔ `src/models/` mapping `projection_coverage` already uses, and flags projection fields with no backing entity/migration column.
- Coverage honesty — every seam reports `pass`/`fail`/`warn`/`not_checked` distinctly; `not_checked` is never collapsed into `pass`.
- Loop closes by default — `generate_projection` and `json_ui_generate` return the checkpoint verdict inline; `application_info` / `projection_coverage` surface per-projection checkpoint status.

**Killer feature:** an agent that adds a projection field referencing a model attribute the migration never created learns it statically, in one call, instead of at runtime — the silent F11-class seam becomes a ranked, actionable next step.

**Key context:**
- Read-only and introspective: no `cargo`/compile; reads source, route registry, DB schema.
- Composes existing validators (`validate_projection`, `json_ui_verify_action`, `render_projection` + `json_ui_validate_spec`, `validate_contracts`); owns only the field→column seam + aggregation — no duplicate control surface.
- Dogfood gate: must surface a real seam defect against the synthetic app catalog and one live consumer, or the design is revisited rather than shipped.
- Design spec: `docs/superpowers/specs/2026-06-09-projection-checkpoint-design.md`.

## Shipped Milestone: v12.4 Form Validation DX

**Goal:** Make uniqueness validation a first-class, ergonomic part of ferro forms — both proactively (async DB-backed rules) and defensively (DB constraint violations surfaced as field-level errors instead of raw SQL), so consumer forms stop leaking raw database errors to end users.

**Target features:**
- Async DB-backed validation rules — `unique` (with exclude-self for edit forms), checked against the DB before insert/update.
- DB constraint → field-level error mapping — opt-in mapping of constraint violations (e.g. a `UNIQUE` index hit) to a specific field's validation error, routed through the ferro validation error path so user input is preserved; replaces the raw-message `From<sea_orm::DbErr> for ActionError` passthrough.

**Killer feature:** a uniqueness violation that today shows the user a raw SQL error instead lands inline under the right field with their input intact — uniqueness "just works" both before the write (async rule) and as a safety net at the write (constraint mapping).

**Key context:**
- Source: gestiscilo-it field test — slug uniqueness violations surfaced as raw SQL errors.
- Original v12.1-era Phase 137–139 scope (Validator struct, sync rules, old-input flash, `req.old()`) already shipped organically via the validation module — not in scope here.
- Composes with Phase 180's `#[action]` / `ActionError` and the existing validation module; the project-agnostic-crates rule applies (no consumer strings in `framework`/`ferro-*`).

## Shipped Milestone: v12.0 JSON-UI v2 — Spec-Driven Rendering

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
- Phase 239 complete (2026-06-23) — Soft-delete data substrate, the foundation phase of milestone v16.3 (MCP CRUD Data Surface, Track A). A nullable `deleted_at` column is added to the `orders` table via an additive, backend-portable sea-orm migration (`m20260623_add_deleted_at_to_orders`, registered append-only; SQLite apply verified, Postgres apply tracked as a human-UAT item). `ServiceDef` (ferro-projections) gains three pure accessors — `resolved_table()` (table or `pluralize(name)` default, byte-identical to the prior inline derivation), `resolved_soft_delete_column()` (column or `"deleted_at"` default), and `is_server_injected_field()` (true for `FieldMeaning::Identifier`/`CreatedAt` and the projection's tenant column, read dynamically — no hardcoded identity), the schema-derivation boundary Phase 240 consumes. The read path (`ferro-mcp-server/src/dispatch.rs`) now calls `resolved_table()` (inline `format!` + TODO removed) and injects a `deleted_at IS NULL` predicate — gated on `soft_delete_column.is_some()`, adding no bound value and not incrementing the placeholder index (Postgres LIMIT/OFFSET unaffected), applied to the shared WHERE clause covering both COUNT and DATA queries — so a soft-deleted row is invisible to reads by construction, not per-tool. 4/4 success criteria verified by live scoped test runs; code review 0 critical / 2 warnings / 3 info (advisory, non-blocking). The substrate consumed by CRUD-03 (Phase 241) and CRUD-05 (Phase 242).
- Phase 221 complete (2026-06-14) — Inbound NL Intent Loop (milestone v15.0 Agent-Operable App / Consumer MCP, AMCP-06), the FINAL v15.0 phase. `ferro-mcp-server::intent::process_nl_turn` classifies a natural-language message into a `ToolSelection { tool_name, arguments, confidence }` via `ferro-ai::Classifier`, then routes it through the EXISTING machinery with zero new dispatch/guard/confirm/envelope logic: `list_*` → read path (`handle_tools_call`) gated by an `authorize_read` ability closure mirroring the direct `/mcp` path; all other tools → `handle_write_call` (scope gate + live guard re-eval + Phase 220 confirmation seam); `Error::LowConfidence` → a `needs_clarification` envelope with no dispatch. The classifier output is treated as untrusted (prompt-injection surface) and re-validated through the same pipeline as any direct call. CI-testability without LLM spend (the AMCP-06 spine) is delivered by a reqwest-free `ReplayClassificationProvider` + committed transcript fixtures behind a new `ai` Cargo feature; the live `AnthropicProvider` path is isolated behind `ai-live` + `#[ignore]` + `FERRO_AI_LIVE_EVAL=1` and announces estimated cost before the first call. A `classifier-trait` feature on `ferro-ai` exposes the provider trait without `reqwest` so the replay path compiles llm-free. The sample app wires a thin `POST /mcp/chat` endpoint (tenant from principal, never the body). Code review found a read-path authorization bypass (WR-01) — the NL read path skipped the app `mcp_ability` Gate the direct path enforces — fixed by the `authorize_read` fail-closed closure seam + a deny-path regression test. 5/5 success criteria verified after the fix; full `--all-features` gate green. Two non-blocking follow-ups remain (WR-02 `/mcp/chat` route registered without `ai-live` returns a feature-availability-leaking error envelope; WR-03 `replay_deterministic` does not assert single-execution idempotency). Milestone v15.0 is ready to archive via `/gsd-complete-milestone`.
- Phase 215 complete (2026-06-13) — Non-visual rendering context (milestone v14.0 Channel Projection, CHAN-01 + CHAN-02 validated). Extends the renderer-free `ferro-projections` surface so a non-visual renderer can guard-filter actions and label intents: `BaseContext` gains `evaluated_guards: HashMap<String, bool>` (absent key = render, explicit `false` = filter) + `verbosity: Verbosity` (`Brief`/`Full`, `#[default] Full`, no serde — `BaseContext::default()` preserves current visual behavior); a new `Intent::label() -> &str` returns the stable snake_case names (`Custom(s)` → inner string), replacing four `format!("{:?}", intent)` label sites in `ferro-mcp`; `Error::NoIntents` is a typed variant defined for the Phase 216 text renderer (not wired into the visual path — that already uses `ProjectionError::EmptyIntents`, per D-09). Downstream adoption: `VisualContext` now embeds `base: BaseContext` (collapsing the previously-duplicated `intent_index`/`current_state`), `builder.rs` reads `ctx.base.*`, and the one `render_projection` test expectation legitimately changed `Browse`→`browse`. Seven-intent vocabulary unchanged; crate stays renderer-free (no new dependency). 5/5 must-haves verified; per-crate tests green (ferro-projections 272, ferro-json-ui 608, ferro-mcp 307); code review clean (0 critical / 0 warning / 3 forward-looking info). The full `--all-features` gate was not re-run at phase close under disk pressure (~98%, a known ENOSPC condition); `clippy --all --all-targets -D warnings` is clean.
- Phase 196 complete (2026-06-10) — Dogfood acceptance + hardening, closing milestone v12.5 Projection Checkpoint. The `checkpoint_projection` tool was run against a deliberately-poisoned synthetic fixture (`poisoned_projection_dangling_field_acceptance`: one planted dangling field → `status: fail`, exact subject, no other field flagged) and against the in-repo `app/` live consumer (`dogfood_app_projections`: 20 findings across 8 projections; the genuine driver is seam 3 `action_to_route` on 4 unregistered actions) — acceptance verdict **GO**, recorded in `196-ACCEPTANCE.md`. `next_steps` cap reduced 10→5 via `const MAX_NEXT_STEPS`. The one wrapper seam with zero findings across all dogfood inputs, `props_to_contract`, was demoted to `not_checked`-by-default (reason `unproven_against_real_inputs`, source `validate_contracts`) and documented in `service.rs` + the agent doc; seams 1/2/3/4 remain active. 4/4 success criteria verified; full `--all-features` gate green; code review 0 critical + 2 warnings + 2 info, all addressed.
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
| ferro-assets pipeline = content-type router with byte-identical passthrough | Unknown file types pass untouched; one pipeline serves HTML sites, JSON-UI bundles, SSR manifests (artifact-agnostic) | ✓ Good |
| ferro-assets is a pure leaf crate, zero ferro-* deps, Wave 1a | Transforms operate only on bytes; no storage/HTTP/DB → lightest dependency posture, parallel-capable | ✓ Good |
| Pure-Rust asset codecs (lol_html/lightningcss/swc/image/ravif), libvips rejected | `cargo build` adds zero C system deps; libvips `VipsImage` is thread-unsafe inside spawn_blocking | ✓ Good |
| Pipeline::run() synchronous; consumer wraps in spawn_blocking | Keeps the leaf crate runtime-free; CPU work stays off the async executor | ✓ Good |
| html_minify treats `<script>`/`<style>` as opaque (no text handler) | Inline JS with template literals/JSON survives byte-correct; structural, not best-effort | ✓ Good |
| Pipeline failure is all-or-nothing (no partial output set) | Consumer two-phase upload builds its all-or-nothing guarantee on this atomic in-memory result | ✓ Good |
| cdn_url() is facade-level over any driver, with origin url() fallback | CDN is presentation, orthogonal to the backend; drivers stay unchanged; pure-string, zero-dep, always available | ✓ Good |
| PurgeApi trait + DoSpacesCdn default adapter; Bunny/Cloudflare feature-gated | DO adapter encapsulates batching/throttle/wildcard/no-op; consumers never reimplement provider quirks | ✓ Good |
| CDN purge token never in Debug/logs/errors (hand-written redacting Debug) | Credential-leak prevention is structural, not incidental; ASVS L1 | ✓ Good |
| Internal sliding-window throttle (loop-recheck under the lock) for CDN rate limits | ≤5 req/10s bound holds under concurrent callers; caller never manages rate limiting | ✓ Good |
| Missing CDN id → purge() is a logged no-op | Consumers without a CDN keep working; no accidental purge against a wrong endpoint | ✓ Good |
| Schema-driven projections replace field_map.rs | Projections and catalog stay consistent by construction | Planned |

---
*Last updated: 2026-07-03 — Phase 252 complete (milestone v16.5 JSON-UI Design System, DS-05 + DS-06): the composition patterns are now a machine-readable, testable rule set enforced at the agent-authoring boundary. `Spec` gained an optional `design` field (`DesignMeta { intent: Option<String>, allow: Vec<String> }` — string-typed so invalid intent values and unknown `allow` ids are lint findings, never parse errors), a new pure `ferro_json_ui::design` module (`lint(&Spec) -> Vec<Finding>`, `Severity::{Info,Warning}`, static `RULE_REGISTRY` with public machine-readable metadata for the Phase 253 docs/MCP derivation, `KNOWN_INTENTS` drift-tested against `ferro_projections::Intent::label()` behind the `projections` feature) implements all 10 anchor-spec intent-keyed rules (page-header, prefer-data-table, list-empty-state, row-actions-grouped, process-kanban, create-separate-page, breadcrumb-on-subpages, form-default-values, destructive-confirmation, card-actions-in-menu) each with violating+conforming test pairs, plus content-based intent inference reported as an info finding. `ferro design:lint [path] [--json] [--deny]` CLI ships (recursive spec walk, `$schema` marker gating, exit non-zero only under `--deny` with warning-level findings; file-read errors surface as warning findings so the CI gate can't false-negative). D-16 single-home held: stale-prop detection extended in catalog Stage 2b (element-level `action` walk via `ConfirmDialog` serde-flatten `unknown_fields`), NOT duplicated in design lint. Sample app views declare `design.intent` (pagamenti gained a PageHeader) and a zero-findings app-crate gate test enforces lint-clean. Lint is diagnostics-only — rendering and catalog validation untouched. Verifier 6/6 success criteria; code review 0 critical / 3 warnings — all FIXED with regression tests (`252-REVIEW-FIX.md`: WR-01/WR-02 rules no longer misfire on `$data`-bound `empty_message`/`breadcrumb` props; WR-03 CLI I/O errors now findings); 4 info recorded (IN-03 `allow: ["allow"]` self-suppression quirk, IN-04 `SpecBuilder` lacks a `.design()` setter — Phase 253 candidates). Full CI-exact gate green in plan 06 (fmt / clippy --all-features / test --all-features 541 passed / docs build); one cosmetic human-UAT persists (`252-HUMAN-UAT.md`: findings-present CLI formatting eyeball). Next: Phase 253 (design_lint MCP tool + catalog/generation-context extensions + docs/src/design-system/ + single crates.io publish that unblocks gestiscilo Phase 232). Previous: Phase 243 complete — milestone v16.3 MCP CRUD Data Surface (Track A) fully delivered (Phases 239–243). Phase 243 (integration/e2e/docs) flipped the sample app's `order` projection to CRUD (`.creatable/.updatable/.deletable` + `.mcp_write_ability` + `.soft_delete_column("deleted_at")`), proved the whole Track A surface end-to-end via an in-process MCP e2e harness (`app/src/tests/crud_e2e.rs`: create→list→update→delete, a per-verb Phase 205 `CallToolResult::structured` envelope regression guard, MCP↔visual single-source parity through the shared `execute_crud_plan` kernel, cross-tenant non-disclosure, and the feature-gated `request_confirm_delete_order`/`confirm_delete_order` confirmation flow), and brought the authoring surface to parity: a new `projection_crud` `code_templates` category (guarded by `test_all_categories_present`), a `generation_context` "Option B" crud_handler note, and a `## MCP CRUD Opt-In` `docs/src/features/projections.md` section matching the shipped projection. Boundaries held: D-09 (`crud_operations.rs` byte-unchanged), D-10 (json-ui builtin-component drift guards stay at 47). Verifier 4/4 automated success criteria; one human-UAT smoke (live `:8090/mcp` drive, intentionally manual per CONTEXT D-01/D-02) persists in `243-HUMAN-UAT.md`. Code review 0 critical / 1 warning (WR-01 stale `dispatch_write` doc example missing the trailing `crud_plan` arg — fixed) / 4 info. Full workspace gate green (fmt + clippy `--all --all-targets -D warnings` + test `--all-features`). Previous: Phase 241 complete (milestone v16.3 MCP CRUD Data Surface Track A, CRUD-03 + CRUD-06): the `create_/update_/delete_<svc>` tools are now **executable**. `derive_crud_plan(svc, verb, inputs) -> CrudPlan` (a pure serializable enum — `Create`/`Update`/`Delete`, mirroring `derive_transition_plan`/`TransitionPlan`, derives `PartialEq` but not `Eq` since `serde_json::Value` isn't `Eq`) lands in `ferro-projections/src/executor.rs` and is re-exported from `lib.rs`. The single `framework::write::dispatch_write` kernel is **extended, not forked**: one trailing `crud_plan: Option<&CrudPlan>` param + a framework-provided generic `execute_crud_plan` interpreting the plan into parameterized SQL (`Statement::from_sql_and_values`; SQLite INSERT+`last_insert_rowid()`+SELECT / Postgres `RETURNING *`; `created_at` injected as a server-side SQL literal per the Plan-01 contract, never bound). The seven pipeline steps (guard re-eval / idempotency / confirmation seam / execute / idempotency-store / channel-parameterized audit `{channel}.crud.{name}` / post-persist override-hook keyed on tool name) run identically for CRUD and transition verbs — `with_override("create_order", …)` reuses the existing registry with no new mechanism. `delete_<svc>` soft-deletes (`UPDATE … SET deleted_at=now WHERE id=? AND deleted_at IS NULL`, never physical `DELETE FROM`), is confirmation-gated via synthesized `request_confirm_delete_<svc>`/`confirm_delete_<svc>` tools (reusing `ConfirmationStore` + the `{tenant_id, action_name, record_id}` token binding; the confirm handler strips the `delete_` prefix to locate the `ServiceDef` since CRUD verbs aren't `ActionDef`s), and is filtered from `list_`. The `ferro-mcp-server` `not_yet_implemented` stub is replaced with real derive→`dispatch_write(.., Some(&plan))` routed through the Phase 205 `CallToolResult::structured` envelope. SC#4 structurally proven: exactly one `pub async fn dispatch_write` (`framework/src/write/mod.rs:596`), zero `DELETE FROM`, NTI count 0. `CrudPlan` carries `tenant_column: Option<TenantColumn> = None` — the Phase 242 extension point (tenant injection/authz/non-disclosure deliberately deferred to 242; app flip/e2e/regression-guard/catalog-docs to 243). Verifier 4/4 success criteria; full `--all-features` gate green (fmt + clippy `--all --all-targets -D warnings` + test). Code review 0 critical / 4 advisory warnings (WR-01 post-update SELECT TOCTOU on concurrent soft-delete; WR-02 missing-`id` falls to NULL semantics not an explicit error; WR-03 no string-length cap; WR-04 guard re-eval skipped at delete-token issuance — relevant when 242 wires write-ability preconditions) recorded in `241-REVIEW.md`, none goal-blocking, optional `/gsd-code-review-fix 241`. Previous: Phase 240 complete (CRUD input-schema derivation + `list_` query polish, milestone v16.3 MCP CRUD Data Surface Track A, CRUD-01/02/04). Added `ServiceDef::is_write_excluded_field` (single shared write-field predicate composing the Phase 239 `is_server_injected_field` boundary + UpdatedAt/Sensitive/list/SM-Status gates), `build_create/update/delete_input_schema`, `is_range_filter_field` (DataType-based, independent of the equality allowlist so Money/Quantity/Percentage get range ops), and extended `build_input_schema` + read dispatch with `<field>__{gt,gte,lt,lte,ne,in}` flat params + `sort=field`/`-field` (equality params and `limit`/`offset` byte-for-byte unchanged; tenant + `deleted_at IS NULL` predicates preserved). Scope boundary held: `create_/update_/delete_<svc>` tools are LISTED with correct schemas but NOT executable — calls return a flag-gated `not_yet_implemented` structured envelope (never `-32601` for an opted-in verb, never an INSERT/UPDATE); `derive_crud_plan` + `framework::write` wiring is Phase 241. Verifier 4/4 success criteria. Code review 0 critical / 4 warnings — all FIXED before close (`240-REVIEW-FIX.md`): WR-01 sort now accepts range-filterable fields, WR-04 NTI envelope gated on the matching opt-in flag (+ `crud_nti_not_returned_when_verb_flag_disabled` regression test), WR-03 update builder explicit Identifier-skip, WR-02 informative inputSchema descriptions; 3 info deferred with rationale. Full per-crate gate green (fmt + clippy `--all-targets -D warnings` + test; 277 ferro-projections + 56 ferro-mcp-server lib). Phase 239 (soft-delete substrate) shipped immediately prior. Previous: Phase 221 complete (Inbound NL Intent Loop, milestone v15.0 Agent-Operable App / Consumer MCP, AMCP-06), the FINAL v15.0 phase. `ferro-mcp-server::intent::process_nl_turn` is the conversational-turn core: classify NL → `ToolSelection { tool_name, arguments, confidence }` via `ferro-ai::Classifier`, then route through the EXISTING read/write/confirm machinery with zero new dispatch/guard/confirm/envelope logic — `list_*` → `handle_tools_call` (gated by an `authorize_read` ability closure mirroring the direct `/mcp` path), all other tools → `handle_write_call` (217 scope gate + live guard re-eval + 220 confirmation seam), `Error::LowConfidence` → `needs_clarification` envelope with no dispatch. The classifier output is untrusted (prompt-injection surface) and re-validated through the same pipeline as any direct call; `tenant_id` always from the authenticated principal, never the classified arguments. The AMCP-06 spine — CI-testable without live-LLM spend — is delivered by a reqwest-free `ReplayClassificationProvider` + committed transcript fixtures behind a new `ai` Cargo feature (a non-ignored deterministic replay test exercises classify/guard/confirm/dispatch/clarify with no network); the live `AnthropicProvider` path is isolated behind `ai-live` + `#[ignore]` + `FERRO_AI_LIVE_EVAL=1` and announces estimated cost before the first call (isolate-before-spend). Feature wiring (D-06): `ai = ["dep:ferro-ai"]` (replay, reqwest-free) and `ai-live = ["ai", "ferro-ai/llm"]` (live); a new `classifier-trait` feature on `ferro-ai` exposes `ClassificationProvider`/`Classifier`/`ClassifierConfig` without `reqwest` so the replay path compiles llm-free (`cargo tree -p ferro-ai --features classifier-trait` shows zero reqwest). The sample app wires a thin `POST /mcp/chat` endpoint. Reuses the Phase 210 transcript-replay pattern rather than inventing a new mechanism; no API keys in committed fixtures. **Code review found a read-path authorization bypass (WR-01) — the NL read path called `handle_tools_call` directly, skipping the `Gate::authorize_for`/`mcp_ability` fail-closed check the direct `/mcp` path applies, so a user denied a projection's ability could read it via natural language; confirmed by the goal verifier (4/5). FIXED before phase close (commit `13378c0a`): `process_nl_turn` gained an `authorize_read: &(dyn Fn(Option<&str>) -> bool + Sync)` fail-closed closure (the app builds it from the principal's `User` + `Gate`), gating the resolved read service's `mcp_ability` after classification and before any dispatch; a non-ignored `read_denied_by_ability_gate` regression test asserts a denied read returns `access_denied` and does not dispatch.** Re-verified 5/5 success criteria after the fix; full `--all-features` gate green (fmt/clippy `--all --all-targets -D warnings`/test). Two non-blocking follow-ups recorded in `221-REVIEW.md`/`221-VERIFICATION.md`: WR-02 (`/mcp/chat` route registered unconditionally; without `ai-live` it returns a 200 `isError:true` envelope leaking feature availability — gate the route or return 501) and WR-03 (`replay_deterministic` does not assert single-execution idempotency for the write path). **Milestone v15.0 (Phases 217–221) is complete and ready to archive via `/gsd-complete-milestone`.** Previous: Phase 220 complete (Confirmation Gating for Destructive Actions, milestone v15.0 Agent-Operable App / Consumer MCP, AMCP-05): a destructive action (`transition_trigger.is_some()`) can no longer execute in a single tool call — it requires a server-issued confirmation token validated at dispatch. Wraps the Phase-219 D-08 seam (`write_dispatch.rs:281`): when the `confirmation` feature is on and an action is destructive and `!is_confirmed`, the seam returns `Err(ConfirmationRequired)` — `is_confirmed` is hardcoded `false` at the `handle_write_call` site and only set true internally by `handle_confirm` after token validation (never settable by agent input). Two synthesized tools per destructive action (`request_confirm_<action>` → `confirm_<action>`, emitted in `render_exposed_tools` AFTER the disambiguation pass): `request_confirm_` validates inputs + re-evaluates guards, mints a `cfm_`-prefixed CSPRNG/BASE62 token (reusing `generate_mcp_api_key`'s pattern — server-generated, never agent-supplied), and stores the validated payload bound to `(tenant_id, action_name, record_id)` in `ferro_ai::InMemoryConfirmationStore` with a `McpServerConfig` TTL (default 300s, clamp 300–600); `confirm_` calls `store.confirm()` (single-use — `DashMap::remove`), rejects expired (`None`) / action-mismatch / record-mismatch / tenant-mismatch, **re-evaluates guards at confirm time** (preserves the 219 fail-closed guarantee across the gap), then runs `dispatch_write(.., is_confirmed=true)` exactly once. **Dependency-hygiene decision (D-06):** `ferro-ai`'s `reqwest`/`reqwest-eventsource`/`futures`/`async-stream`/`schemars` made optional under a new `default=["llm"]` feature with `#[cfg(feature="llm")]` gates on all client/classifier/embed/tools modules; a reqwest-free `confirmation` feature exposes only the store. `ferro-mcp-server` depends on `ferro-ai = { optional, default-features=false, features=["confirmation"] }` behind its own `confirmation` feature — proven: `cargo tree -p ferro-mcp-server` feature-OFF shows **zero ferro-ai** (the 3 reqwest in-graph are pre-existing from `ferro-mcp-oauth`, not new); every ferro-ai consumer (`ferro-mcp`, `ferro-cli`, `framework`) re-enabled `llm`. Result envelopes reuse 219's `CallToolResult::structured`/`write_tool_error_result`. 5/5 success criteria verified end-to-end; full `--all-features` gate green; 40/40 confirmation tests pass. Code review 0 critical / 4 warnings (all FIXED, `220-REVIEW-FIX.md`): WR-01/02/03 three confirmation error-string leaks redacted (the 219 CR-01 discipline — no store/internal errors to the agent); WR-04 the token-TOCTOU (a retry mints a second TTL-bounded token) documented as accepted — single-use + TTL + tuple-bound + dispatch idempotency make it non-exploitable; re-keying deferred with the DB-backed store. Phase 219 direct-write tests for destructive actions gated `#[cfg(not(feature="confirmation"))]` (under confirmation, destructive actions only run via the two-step flow). v15.0 has one phase left: Phase 221 (inbound NL intent loop — `Classifier<ToolSelection>` → tool+args, guard- + confirmation-gated, with a `FERRO_AI_LIVE_EVAL` replay/smoke path). Previous: Phase 219 complete (Write Dispatch, milestone v15.0 Agent-Operable App / Consumer MCP, AMCP-04): the security-critical core — the Phase-218 write tools become **callable**, executed tenant-scoped with the action's guard **re-evaluated server-side at call time against live DB**, idempotency enforced, an audit trail recorded, and a `CallToolResult::structured` result returned. New `ferro-mcp-server/src/write_dispatch.rs`: `dispatch_write` + `handle_write_call` replace the 218 `-32601` placeholder. Registration is a `WriteDispatcher` struct holding boxed-future `ExecutorFn`/`GuardEvaluatorFn` (no `async-trait` dep) passed as a param into `handle_tools_call` — `McpServerConfig` stays identity-only; the app registers a concrete executor + guard evaluator (`make_write_dispatcher` in `app/src/controllers/mcp.rs`). Call pipeline (D-07): 217 scope gate → resolve `ActionDef` → validate inputs → **live guard re-eval per precondition, fail-closed, NEVER consulting `ctx.evaluated_guards`** (the PITFALLS §2 structural fix; the stale ARCHITECTURE diagram showing `evaluated_guards` at call time was explicitly overridden) → idempotency → D-08 confirmation seam (pass-through; no `ferro-ai` dep) → executor → `ferro-audit` entry → structured result. Tenant scoping: `TenantScoped` impl added to the sample `Order` model; the executor loads via `find_by_id(id).filter(TenantId.eq(tenant_id))` and denies on `None` before mutation — `tenant_id` from the principal, never the payload. Idempotency: new `mcp_idempotency_keys` migration in `ferro-mcp-oauth` (composite UNIQUE `(tenant_id, idempotency_key)`, consumer-run); `INSERT OR IGNORE`/`ON CONFLICT DO NOTHING`; second identical call replays the stored result, executor fires exactly once. Audit: `ferro-audit` reused (`AuditEntry::record("mcp.action.{name}").tenant().actor().target().after().write(db)`) — recoverable via `history_for_target`; no new audit table. Results: `CallToolResult::structured` for success, a single `write_tool_error_result` (isError:true) for denials — no bare `content[]`. error.rs gains `GuardFailed`/`ActionNotFound`/`Validation`. A `DeriveMigrationName` version-collision between `ferro-mcp-oauth` and `ferro-audit` (both ship `migration.rs`) was resolved with local wrapper migrations. 5/5 success criteria verified end-to-end; full `--all-features` gate green (fmt/clippy `--all --all-targets -D warnings`/test). **Code review found 2 critical + 3 warnings — all FIXED before phase close** (`219-REVIEW-FIX.md`): CR-02 the guard evaluator defaulted unknown guard names to `Ok(true)` — a **fail-OPEN inversion of SC#1's core invariant** — now `Err(GuardFailed)` (fail-closed) in both production and test dispatchers; CR-01 raw SeaORM DB error strings (SQL/table/column names) were forwarded verbatim to the agent — now redacted to a generic "write operation failed" (only Validation/ActionNotFound/guard messages pass through); WR-01 `idempotency_key` capped at 128 chars; WR-02 `ExecutorFn` audit-PII contract documented; WR-03 the app Gate was unreachable for write tools — routing restructured so write authorization is explicitly the 217 scope gate + dispatch_write guard re-eval (Gate covers read tools). Verifier independently re-confirmed the fail-closed guards and error redaction in source. v15.0 continues with Phase 220 (confirmation gating for destructive actions — the D-08 seam + `ferro-ai::ConfirmationStore`). Previous: Phase 218 complete (Write-Tool Rendering from ActionDef, milestone v15.0 Agent-Operable App / Consumer MCP, AMCP-03): each `mcp_exposed` `ServiceDef`'s `ActionDef`s are projected into MCP **write tools visible in `tools/list`**, derived purely from `ActionDef` — no hand-authored per-tool surface. Rendering only; dispatch/execution is Phase 219 (a write-tool `tools/call` returns `-32601` by design — no executor wired yet). `ferro-mcp-server/src/schema.rs` adds `build_action_input_schema(action, service)` reusing the promoted `pub(crate) data_type_to_json_schema` (single DataType→JSON mapping, no duplication); it injects the `ServiceDef`'s first `FieldMeaning::Identifier` field as a required param (silent skip when none — create-style actions), excludes `FieldMeaning::Sensitive` inputs (the only sensitive variant — T-218-01), and allows `Json`/`Binary` types (unlike read filters). `render_exposed_tools` (`renderer.rs`) converted from `.map().collect()` to an explicit `for` loop: per service, the `list_<svc>` read tool then one write tool per `ActionDef` (name = `action.name` verbatim; cross-service collisions disambiguated `<name>_on_<service>`). Annotations `ToolAnnotations::new().read_only(false).destructive(action.transition_trigger.is_some())` (no `idempotentHint` — no `ActionDef` attribute; a 219/220 concern). Guard filter: a precondition `evaluated_guards.get(p) == Some(&false)` omits the tool; absent = show — documented in code as a **VISIBILITY filter, not an authorization gate** (enforcement is 217's scope gate + 219's server-side re-eval; `evaluated_guards` is empty at runtime in 217/218, so SC#3 is proven via explicit-map unit tests). SC#5: the Phase 205 strict-deser guard extended — `write_tools_definitions_parse_as_valid_mcp_tool` round-trips every write-tool definition through `rmcp::model::Tool`. `ferro-projections` untouched (only reads `ActionDef`/`ServiceDef`); no new crate. 5/5 success criteria verified; full `--all-features` gate green (fmt/clippy `--all --all-targets -D warnings`/test, 0 failed). Code review 0 critical / 1 warning (WR-01 `disambiguate_write_tool_collisions` counts total occurrences not distinct services — misfires only on the exotic intra-service duplicate-action-name case; verifier judged non-blocking for AMCP-03) / 3 info, recorded in `218-REVIEW.md`. v15.0 continues with Phase 219 (write dispatch: tenant-scoped, server-side guard re-evaluation, idempotency, audit). Previous: Phase 217 complete (Tenant Context + Per-Tenant API-Key Auth, milestone v15.0 Agent-Operable App / Consumer MCP, AMCP-01 + AMCP-02): the consumer MCP endpoint becomes tenant- and permission-aware, with a per-tenant API key as a second auth path beside OAuth JWT. `McpContext` (was an empty unit struct) now embeds `tenant_id: Option<i64>` + `evaluated_guards: HashMap<String,bool>` + `scope: Option<String>` and is threaded through `handle_tools_list`/`handle_tools_call` (`ferro-mcp-server/src/{renderer,jsonrpc}.rs`); `tenant_id` flows into the existing fail-closed `dispatch()` and is never sourced from the call payload. `ferro-mcp-oauth` gains `validate_api_key(header,&db,expected_tenant) -> BearerCheck` (async SHA-256-hashed lookup, parallel to `validate_bearer`, same `Authenticated(principal)` outcome — D-01 resolved a CONTEXT/ARCHITECTURE crate-placement conflict in favour of the auth crate), `generate_mcp_api_key` (`ferro_`-prefixed BASE62 CSPRNG key, plaintext returned once, only `key_hash` stored), and a new canonical `mcp_api_keys` migration (`tenant_id`/`key_hash` UNIQUE/`scope`/`revoked_at`; the pre-existing `framework` `api_keys` table was incompatible — no tenant/scope — so a new table was defined, not reused). `ferro-mcp-server/src/auth.rs` replaces the `BearerOutcome` stub with an async `resolve_tenant` unifier branching on the `ferro_` prefix; `error.rs` gains `Auth(String)` → same `-32603` envelope as the OAuth invalid-token path. Server-side scope gate at `tools/call` (`read` key + write tool → `-32603`, independent of the list filter) is real and unit-tested against a synthetic write-tool name so Phase 218 plugs in. New `ferro-mcp-server/tests/mcp_tenant_isolation.rs` (created, not "extended" — D-09 was wrong; raw-SQL in-memory SQLite fixture, not the app migration stack) proves SC#2/#3/#5; `cargo test -p ferro-mcp-oauth` 84 green, `-p ferro-mcp-server` 9 green. publish.yml Wave-2 reordered so `ferro-mcp-oauth` precedes `ferro-mcp-server` (the new path dep); auth docs added. 5/5 success criteria verified; full `--all-features` gate green (fmt/clippy `--all --all-targets -D warnings`/test, 31 GB free). Code review 2 critical / 1 warning / 2 info — both criticals (CR-01 sample-app `bearer_auth.rs` middleware still JWT-only so `resolve_tenant` is unreached in the live HTTP path; CR-02 `app/src/controllers/mcp.rs` builds `McpContext::default()` so `scope` is never populated and the gate can't fire end-to-end through the controller) confirmed accurate by the verifier but classified as **consumer-app wiring follow-ups, not phase gaps** — v15.0 is "framework capability + synthetic validation only", and the framework primitives are proven by passing tests; precise fix locations recorded in `217-REVIEW.md`/`217-VERIFICATION.md` for the eventual app-adoption pass. v15.0 continues with Phase 218 (write-tool rendering from `ActionDef`). Previous: Phase 216 complete (Conversational-text Renderer, milestone v14.0 Channel Projection, CHAN-03 + CHAN-04): the first production non-visual `Renderer`. `FieldDef.render_hint` (`AltText(String)`/`Skip`, `Option`, `#[serde(default, skip_serializing_if)]` so absent key → `None`) lands in `ferro-projections/src/field.rs` with all 11 `FieldDef {}` literal sites migrated; the new `ferro-text` crate (workspace member, publish.yml Wave 1b, `insta` dev-dep) hosts `TextRenderer: Renderer<Output=String, Context=BaseContext>` (reuses `BaseContext`, no wrapper). Seven per-intent strategies render deterministic conversational text via `Intent::label()`; guard filtering uses `evaluated_guards.get(g).copied().unwrap_or(true)` (absent renders, explicit `false` hides); `Verbosity::Brief`/`Full` differ; `render_hint` → `AltText`→alt / `Skip`→omit / `None`-on-`ImageUrl`/`Url`→`(image)`/`(link)` label not raw URL; Focus/Analyze degraded fallback with no fabricated stats; empty intents → `Error::NoIntents`. `ferro::TextRenderer` + `ferro::RenderHint`/`Verbosity` re-exported behind the `projections` feature. COMP-05 `approval_workflow` anchor copied into tests; both Process guard states snapshot-pinned (unfiltered = 4 actions, `is_approver:false` = approve/reject hidden). 4/4 must-haves verified; `cargo test -p ferro-text` 13/13, `cargo doc -Dwarnings` clean, full `--all-features` gate green; code review 0 critical / 1 warning (render_collect field-filtering — by-design per D-09: guards filter actions, not Collect fields) / 4 info. v14.0 ready to archive via `/gsd-complete-milestone`. Previous: Phase 215 complete (Non-visual rendering context — BaseContext + Intent extensions, milestone v14.0 Channel Projection, CHAN-01 + CHAN-02): `ferro-projections` `BaseContext` gains `evaluated_guards: HashMap<String,bool>` (absent = render, `false` = filter) + `verbosity: Verbosity` (Brief/Full, `#[default] Full`, no serde, default preserves visual behavior); new infallible `Intent::label() -> &str` (snake_case names, `Custom(s)`→inner) replaces four `format!("{:?}", intent)` label sites in `ferro-mcp`; typed `Error::NoIntents` defined for the Phase 216 text renderer, deliberately NOT wired into the visual path (D-09 — that path keeps `ProjectionError::EmptyIntents`). `VisualContext` refactored to embed `base: BaseContext` (collapses the duplicated `intent_index`/`current_state`), `builder.rs` migrated to `ctx.base.*`, one `render_projection` test expectation changed `Browse`→`browse`. Seven-intent vocabulary frozen; crate stays renderer-free (no new dep). 5/5 must-haves verified; per-crate tests green (ferro-projections 272 / ferro-json-ui 608 / ferro-mcp 307), `clippy --all --all-targets -D warnings` clean; code review 0 critical / 0 warning / 3 forward-looking info. Full `--all-features` not re-run at phase close under disk pressure (~98%, known ENOSPC). Phase 216 (CHAN-03/04) builds the conversational-text `Renderer` + `FieldDef.render_hint` on this surface. Previous: Phase 212 complete (CRUD Handler Proc Macros, milestone v13.1, CRUD-01–06): two route-attribute proc macros in `ferro-macros` — `#[resource_get]` / `#[resource_post]` — fold the tenant-scoped CRUD prelude (typed `param_as("id")` → sync `current_tenant()` → `<R as TenantScoped>::find_for_tenant(__resource_id, __tenant.id)` → 404/303-on-miss) into one attribute while tenant + resource stay real typed params (rust-analyzer-friendly), body in a named `__{name}_inner` fn. They INLINE the handler/action boilerplate (no nested `#[handler]`/`#[action]` — discuss-phase guess D-06 overridden by research after the double-extraction Pitfall 1) and emit absolute `::ferro::` paths. Backed by `framework` additions: a `TenantScoped` trait (`#[async_trait]`, assoc `Id: FromStr`) whose contract makes cross-tenant reads structurally impossible, and `Validator::validate_or_redirect(url)` composing the existing `with_old_input`+`into_action_error` chain (the redundant `&data` arg dropped in review — the validator already holds it). Facade re-exports; trybuild harness (pass fixtures incl. `tenant = expr` + `full_crud_reference`, 4 compile-fail fixtures with `.stderr`); rustdoc cargo-expand walkthroughs; 0.2.56 bump. Code review 0 critical / 4 warnings / 5 info — security property (tenant-scoped lookup on every path) verified intact; WR-01 (escape-hatch `&&TenantContext` bug) + WR-02/03 (compile_error robustness) + WR-04 (POST 302→303) + the redundant-arg/rustdoc info all fixed, helper-dedup deferred. 12/12 must-haves verified. Also fixed a Phase 214 regression discovered here: `test_api_controller_template_substitution` asserted the old `.update()/.set_title()/.save()` builder that 214 replaced with the ActiveModel `Entity::update_one` shape (214's verify ran build + scaffold-smoke, not `cargo test -p ferro-cli`). With Phase 212, the v13.x batch scoped so far (v13.0/v13.1/v13.2/v13.3) is complete. Nothing in v13.x beyond v13.2 (0.2.55) is published — 0.2.56 is a local bump; the eventual release bundles the committed-not-released Phase 214 + 212 work and needs a manual `workflow`-scope push for the 214 `ci.yml`/`publish.yml`. Previous: Phase 214 complete (Scaffold↔Library Parity & Published-Artifact Smoke Test, milestone v13.3): fixes the scaffold↔library drift COMP-04 surfaced. Framework exports `error_response!` (`#[macro_export]`, bare `HttpResponse` for `.map_err`/`.ok_or_else` arms) and `ActiveValue` (sea_orm facade re-export); the `make:job` template routes through `ferro::queue::*` (no generated `ferro-queue` dep); the `--api` and full-stack (Inertia) controller templates, `auth.rs` (`crate::models::user` singular, `ferro::DB::connection()?.inner()`), and FK templates emit only published-facade symbols. A new framework `Validator::with_error` (+ `passes()`/`fails()` honoring pre-errors) backs the auth templates. Guard: a non-ignored `scaffold_builds_against_workspace_ferro` test scaffolds the full sequence incl. one non-`--api` resource and `cargo build`s against the workspace `ferro` via `[patch.crates-io]` (passes); a per-PR `ci.yml scaffold-smoke` job runs it; a release-time `publish.yml post-publish-scaffold-smoke` job builds the `ARG FERRO_VERSION` Dockerfile against the just-published `ferro-rs`. ferro-mcp `code_templates` surfaces `ferro::error_response!`; docs updated. Code review found 1 critical (CR-01: full-stack templates still emitted nonexistent `HttpResponse::{internal_server_error,not_found,bad_request,redirect}` — the same drift class, uncaught because the guard only ran `--api`) + 2 warnings, all fixed and the guard extended to the non-`--api` path. 10/10 must-haves verified; SCAF-01–05 traceable. Operational handoff: the `ci.yml`/`publish.yml` changes need a manual `git push` (CI token lacks `workflow` scope). Previous: Phase 211 complete (COMP-04 Time-to-Working-App Benchmark) — the FINAL phase of milestone v13.0 Compressive Validation, all five COMP items done (207 COMP-02, 208 COMP-05, 209 COMP-01 Slice A, 210 COMP-03, 211 COMP-04). COMP-04 ships a gated criterion `iter_custom` benchmark (`ferro-cli/tests/benchmark_new_project.rs`, FERRO_BENCH=1 gate, no second target dir, build asserts exit 0) plus a committed cold-cache Dockerfile (`debian:bookworm-slim`, TLS-pinned rustup, version-pinned `--locked` install) and RESULTS.md. The first real cold-cache run (Apple M1 Pro, rustc 1.96.0, ferro-rs 0.2.55) delivered the honesty requirement decisively: CLI scaffolding steps are sub-second, but **the published 0.2.55 scaffold does not compile** — `cargo build` fails with 52 errors (missing `error_response!` macro export, `#[rule]` attribute, `ferro::Queue`/`QueueConfig`, an undeclared `ferro-queue` dep in the generated Cargo.toml, unimported `ActiveValue`, `crate::models::users`, `ferro::database::connection`-as-function). Two further cold findings: a clean Debian needs `libssl-dev`+`pkg-config` to install the CLI (openssl-sys), and `make:scaffold` swallows flags placed after the greedy `[FIELDS]...` positional. Full finding in `211-WEAKNESSES.md`; 5/5 SC verified; code review 0 critical / 2 warning (both fixed) / 4 info. Follow-up (out of scope here): align scaffold templates with the published `ferro` surface + add a published-artifact scaffold→`cargo build` smoke test to CI. Milestone v13.0 is ready to archive via `/gsd-complete-milestone`. Previous: Phase 208 complete (COMP-05 Cross-Modality Vocabulary Sketch, milestone v13.0 Compressive Validation): three `pub(crate)` research-sketch renderers (`CliSummaryRenderer` → String, `VoiceRenderer` → String/no-SSML, `MobileCardRenderer` → `serde_json::Value`) in `ferro-projections/src/render/sketch/`, all rendering a shared `approval_workflow` Process fixture and each marked `// Research sketch — not stable API`; the seven-intent vocabulary (`intent.rs`/`derive.rs`) byte-frozen and verified, no sketch re-exported from `lib.rs`. The COMP-05 deliverable is the analysis at `docs/research/comp-05-cross-modality-vocabulary-sketch.md` (7×3 intent×modality matrix, three named vocabulary tensions — Focus+non-screen-media, Analyze+time-series, Process guard-visibility — a v14.0 implications table of CHAN-* candidates including `device_class`/`evaluated_guards`, and a discovered-weaknesses section grounded in the actual sketch behavior). 5/5 success criteria verified; code review clean on correctness (3 info-only notes). v14.0 Channel Projection consumes this analysis. Previous: Phase 196 complete (Dogfood Acceptance + Hardening), the final phase of milestone v12.5 Projection Checkpoint: the `checkpoint_projection` tool earned its place by surfacing real seam defects — a poisoned synthetic fixture (exact-subject field→column fail) plus the in-repo `app/` live consumer (20 findings, seam 3 `action_to_route` the genuine driver), acceptance verdict GO in `196-ACCEPTANCE.md`. `next_steps` capped 10→5 (`MAX_NEXT_STEPS`); the only zero-finding wrapper seam `props_to_contract` demoted to `not_checked`-by-default (source `validate_contracts`, reason `unproven_against_real_inputs`), documented in `service.rs` + agent doc; seams 1/2/3/4 active. 4/4 SC verified, `--all-features` gate green, code review (0 critical / 2 warn / 2 info) addressed. Milestone v12.5 is ready to archive via `/gsd-complete-milestone`. Previous: Milestone v12.5 Projection Checkpoint started (agent write→verify loop: `checkpoint_projection` MCP tool walks the intent-slice spine, owns the field→column seam, dispatches to existing validators for the rest, closes the loop by default; design spec `docs/superpowers/specs/2026-06-09-projection-checkpoint-design.md`). Previous: Milestone v12.4 Form Validation DX (async DB-backed `unique` rules + DB-constraint→field-level error mapping; source: gestiscilo-it slug-uniqueness field test). 0.2.48 released to crates.io (all crates; new crates ferro-deployments + ferro-assets bootstrapped; ferro-assets nasm defect fixed pure-Rust). Previous: Phase 188 complete (ferro-storage CDN Extension) — closes the v12.3 Deployment Platform Primitives milestone (Phases 185-188): extends the existing `ferro-storage` crate with CDN awareness. Killer feature: the CDN cache-coherence primitive — `Storage::cdn_url(path)`/`Disk::cdn_url(path)` (facade-level, double-slash-safe, origin `url()` fallback, configured via `AWS_CDN_URL`) plus a `PurgeApi` async trait make promote→purge a clean two-call sequence. `DoSpacesCdn` is the default adapter (DELETE `/v2/cdn/endpoints/{id}/cache`, body `{"files":[...]}`, Bearer, 204; ≤50-file batching; internal sliding-window throttle ≤5 req/10s via loop-recheck-under-lock; wildcard = 1 slot; missing-`DO_SPACES_CDN_ID` → logged no-op). Bunny + Cloudflare adapters behind `cdn-bunny`/`cdn-cloudflare` features (Cloudflare chunks ≤30 URLs/call; Bunny throttled) — `cargo tree` proves zero default-graph impact (identical 343-line tree). reqwest lean rustls (`default-features=false, ["json","rustls-tls"]`) reuses existing `ring` → no new C deps; thiserror stayed 1.0; workspace bumped 0.2.45 → 0.2.46. API tokens are redacted in all Debug impls (never logged/in errors). 4/4 success criteria verified; full `--all-features` CI gate green; code review 0 critical + 4 warnings + 2 info all fixed (WR-01 throttle concurrency race, WR-02 Bunny throttle, WR-03 Cloudflare 30-URL chunking, WR-04 empty-config validation, IN-01 register_disk_with_cdn). ferro-storage is an existing published crate — CI publish-update handles the version bump, no manual bootstrap. Previous footer: Phase 187 complete (ferro-assets — asset pipeline composer): new pure-Rust leaf crate (zero ferro-* deps, publish Wave 1a) providing a composable, content-type-aware asset pipeline. Killer feature: a content-type router with a byte-identical passthrough guarantee — unknown file types (e.g. JSON-UI spec bundles) pass untouched through the full HTML/CSS/JS/image pipeline, making it artifact-agnostic. Seven built-in transforms: `html_minify` (lol_html, `<script>`/`<style>` bodies opaque — no text handler, so inline JS with template literals/JSON survives byte-correct), `css_minify` (lightningcss `=1.0.0-alpha.71` exact pin), `js_minify` (swc 66 `Compiler::minify`), `image_transcode` (pure-Rust `image`+`ravif` → AVIF+JPEG responsive variants, Lanczos3, no-upscale, deterministic `{stem}-{width}w.{ext}` naming, rayon-bounded concurrency default ≤2 via `into_par_iter` in a sized pool), `responsive_images` (lol_html `<img>`→`<picture><source srcset>` discovering variants from the asset set, HTML-encoded paths), `inject_before_tag` (lol_html), `replace_tokens` (raw-byte `%%TOKEN%%` substitution on text content types only — binary image variants passed through). `Pipeline::run()` is synchronous (no tokio); the consumer wraps it in `spawn_blocking`. All-or-nothing failure semantics: any transform error returns a structured per-file `Error` and produces no partial output set (consumer two-phase upload builds on this). Zero new C system deps (`cargo tree` clean; libvips rejected). Registered in workspace members + publish.yml WAVE1A; docs/src page + README shipped. 5/5 success criteria verified; phase-close gate green (fmt/clippy --all --all-targets/full --all-features suite); code review 0 critical + 3 warnings all fixed (WR-01 real bounded parallelism via `into_par_iter`, WR-02 srcset HTML-encoding, WR-03 ReplaceTokens binary-image guard). Manual first-publish of ferro-assets deferred to the milestone master-push (CI token is publish-update only; reminder in STATE.md alongside ferro-bundle + ferro-deployments). Previous footer: Phase 186 complete (ferro-deployments — immutable deployments + atomic promote): new leaf crate providing the deployment abstraction — immutable `Deployment` rows (`building`/`ready`/`failed`, terminal-immutable enforced at the API layer), crate-owned active pointer with a single-statement atomic flip (`INSERT … ON CONFLICT DO UPDATE SET previous_deployment_id = deployment_id, deployment_id = ?`) returning the prior id, dual-backend raw SQL (SQLite + Postgres) with `conn.begin()` pinning, `rollback` = promote-of-previous with non-`ready`/artifact-deleted guards, `DeploymentStorage` trait + S3-compatible default delegating to ferro-storage (`deployments/{id}/` prefix), `preview_url(config, identifier)` wildcard-subdomain helper reading `DEPLOYMENT_PREVIEW_DOMAIN`. Portable `CreateDeploymentsTable` + `CreateDeploymentPointersTable` migration helpers (SchemaManager DDL only). Artifact-shape agnosticism proven by a runnable doc-test storing a JSON spec bundle through the full lifecycle (zero HTML/app-identity strings). Registered in publish.yml Wave 1b; workspace bumped 0.2.44 → 0.2.45. Concurrent-promote race test green (SQLite always-on, Postgres cfg-gated). Phase-close gate green; code review 0 critical + 3 warnings all fixed (WR-01 path-traversal guard, WR-02 `ph()` unsupported-backend error, WR-03 promote RETURNING-None hardening). Manual first-publish of ferro-deployments pending (CI token is publish-update only). Previous footer: Phase 185 complete (ferro::queue DB-backed job queue): Redis backend replaced with a DB-backed queue in ferro-queue — dual-backend atomic claim (Postgres `FOR UPDATE SKIP LOCKED` / SQLite single-txn claim), stuck-job reaper with poison-job parking, full-jitter exponential backoff, `idempotency_key()` dedupe hook, `WorkerLoop` auto-start inside the app server with SIGTERM drain + re-queue, namespaced `ferro::queue` module replacing flat re-exports, debug endpoints + ferro-mcp `job_history` over the `jobs` table, race-test proof on shared temp-file SQLite (Postgres cfg-gated). Phase-close gate green; code review 1 critical + 6 warnings all fixed (CR-01 pooled-connection claim atomicity). 3 live-binary checks pending in 185-HUMAN-UAT.md. Previous footer: Phase 189 complete (ferro-stripe manual capture): `CheckoutBuilder::manual_capture()` with pre-flight mode guard, `payment_intent` capability module (capture/cancel/retrieve with positive-amount guard), `StripePaymentIntentAmountCapturableUpdated` + `StripePaymentIntentCanceled` typed events with golden fixtures, Connect destination-charge composition via single merged `payment_intent_data` construction, Manual Capture docs with ferro-reservation hold/commit/release correspondence table. Phase-close gate green (fmt/clippy/full test suite); code review WR-01/WR-02 fixed. Previous footer: Phase 181 complete (JSON-UI inline error rendering, 2026-05-31; one human-verification item remains in 181-HUMAN-UAT.md).*
