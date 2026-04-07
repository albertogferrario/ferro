# Ferro 0.2.0

Released: 2026-04-07

## Highlights

Ferro 0.2.0 is a large release spanning 590 commits and roughly eight months of work across milestones v9.0 through v12.0-in-progress. The headline additions are **Service Projections** (`ferro-projections`) — a typed, intent-driven model for turning backend services into UIs — and a completely stabilized **JSON-UI** stack with 30+ components, semantic Tailwind v4 theming, a persistent dashboard shell, and a spec-driven rendering pipeline.

On the platform side, this release introduces first-class subsystems for multi-tenancy (`TenantContext`, `TenantMiddleware`, tenant-aware background jobs), Stripe billing (`ferro-stripe`), AI classification and human-in-the-loop confirmations (`ferro-ai`), WhatsApp messaging (`ferro-whatsapp`), and semantic theming (`ferro-theme`). The JSON-UI visual overhaul (v10.0) and framework consolidation audit (v11.0) closed dozens of accuracy, accessibility, and documentation gaps and migrated the entire renderer to semantic design tokens.

Finally, the deploy story has been rewritten twice. Phase 122 introduced a full Docker + DigitalOcean scaffold with an MCP-exposed doctor. Phase 122.2 then aggressively simplified it: the deploy MCP tools and several CLI flags were removed, `ferro-mcp` became a library launched in-process from `ferro-cli`, and the Dockerfile/app.yaml renderers were rebuilt from scratch. Users upgrading from 0.1.x should read the Breaking Changes and Upgrade Guide sections carefully.

## Breaking Changes

- **`ferro-mcp` is now a library crate launched in-process by `ferro-cli`.** The standalone `ferro mcp` binary pathway still works but the crate no longer has a `main`. If you built tooling against `ferro-mcp` as a binary, switch to `ferro mcp` via the CLI or call `ferro_mcp::run()` directly.
- **Deploy MCP tools removed.** `deploy_check`, `deploy_diff_env`, and `runtime_requirements` (added in Phase 123) were deleted in Phase 122.2. Use `ferro doctor` for equivalent diagnostics.
- **CLI `ferro deploy:check` removed.** Replaced by `ferro doctor`, which runs the full SCOPE §12 check list in human or JSON mode.
- **CLI `ferro ignore:sync` behavior:** the `ignore_patterns.toml` SoT now drives `.gitignore` / `.dockerignore` generation; manual edits to generated files will be overwritten on sync.
- **`docker:init` and `do:init` flags changed.**
  - Removed: `--ferro-ref`, `--region`, `--repo`, `--runtime-deps`.
  - The commands were rewritten twice (122 then 122.2); the current surface is `--force` only. Templates, env parsing, and worker discovery are now handled internally.
- **Dockerfile and `.do/app.yaml` renderers rewritten.** Any project that committed Phase 122 output needs to regenerate via `ferro docker:init --force` and `ferro do:init --force`. Worker filtering now excludes test/dev/debug bins, DO app names are sanitized, dev-default env values are replaced with placeholders, and inline comments are stripped from `.env.example`.
- **Public API import paths (Phase 110).** All documentation and code templates were corrected from `ferro_rs::{...}` to `ferro::{...}`. External code still importing from `ferro_rs` must switch to `ferro`.
- **Validation rule rename (Phase 110).** The nonexistent `sometimes` rule referenced in older templates is replaced by `nullable`. Update any scaffolded validators accordingly.
- **`ferro-json-ui` API surface tightened (Phase 98-03).** Several types had their visibility demoted; re-exports now flow through `framework` (`ferro::json_ui::*`). Direct `ferro_json_ui::internal::*` imports may break — use the public re-exports.
- **`JsonUiConfig` default `body_class` now includes `dark`** (Phase 52-01). If you relied on the previous default being light-only, override `body_class` explicitly.
- **`ferro-projections` reconstruct API** now parses full action details, guards, and transition guards (Phase 93-02). Callers inspecting the reconstructed `ServiceDef` will see richer data than before.
- **Tailwind v4 migration.** The JSON-UI renderer, runtime JS, and layout emit Tailwind v4 class names (e.g. `--font-*` token namespace rather than `--font-family-*`). Custom themes targeting the Tailwind v3 vocabulary must migrate.
- **`ferro-json-ui` runtime module split** (Phase 125-02). `runtime.rs` was split into per-concern submodules; any crate reaching into `ferro_json_ui::runtime::*` internals must update paths.
- **`COMPONENT_CATALOG` consolidated to `ferro-json-ui`** (Phase 113-02). The duplicate catalog in `ferro-mcp` was removed.

## New Features

### Service Projections (`ferro-projections`, v9.0)
- New crate `ferro-projections` with typed `ServiceDef`, `FieldDef`, `StateMachine`, `ActionDef`, `GuardDef`, `RelationshipDef`, and `IntentHint` types plus builder APIs.
- Intent layer: 7 structural intents (Browse, Focus, Collect, Process, Summarize, Analyze, Track) with 5-analyzer derivation pipeline (field meaning, writability, state machine, relationships, actions).
- `Renderer` trait and `JsonUiRenderer` with layout strategies for every intent.
- JSON Schema generation for all public types via `schemars`.
- Framework integration: `projections` feature gate, `make:projection` CLI (with `--from-model`), `projection:check` CLI.
- MCP tools: `list_projections`, `inspect_projection`, `render_projection`, `validate_projection`, `projection_coverage`.
- Full protocol specification published as an mdBook.

### Multi-Tenancy (Phase 95)
- `TenantContext`, `TenantFailureMode`, task-local context, `TenantResolver` trait, and four concrete resolvers.
- `TenantMiddleware` with resolver chain and configurable failure modes.
- `TenantScope` and `TenantContext` request extractor.
- `DbTenantLookup` with moka cache.

### Stripe Integration (`ferro-stripe`, Phase 96)
- New crate with core types, client facade, subscription checkout, sync, and Connect checkout.
- `TenantContext` subscription enrichment and `RequiresPlan` middleware for plan-gated routes.
- Webhook HMAC verification, event types, and subscription sync.
- `make:stripe` CLI scaffolding and MCP Stripe introspection tools.

### Tenant-Aware Background Jobs (Phase 97)
- `JobPayload.tenant_id`, `TenantNotFound` error, `TenantScopeProvider` trait.
- `Worker` runs jobs inside the originating tenant scope.
- `FrameworkTenantScopeProvider` wiring.

### JSON-UI Stable Release (Phase 98 + v10.0 visual overhaul)
- 30+ built-in components: Grid, Collapsible, EmptyState, FormSection, PageHeader, ButtonGroup, DropdownMenu, KanbanBoard, CalendarCell, ActionCard, ProductTile, DataTable, Image, and more.
- `DashboardLayout` with persistent sidebar/header shell and mobile backdrop.
- Built-in JS runtime for SSE, toasts, live values, modals, form guards, kanban reload, and text-equals guards.
- Native `<dialog>` modal replaces the custom implementation.
- `JsonSchema` derives on public types; API surface audited.
- Typography scale, surface elevation (`bg-card`), interactive states (focus rings, hover, transitions), form polish, and accessibility fixes (ARIA, focus management).
- Responsive `md_columns` on Grid; `max_width` on Card and FormSection; auto-submit Switch; datalist on Input.
- URL templating in DataTable row actions.
- `TemplateRenderer` (Phase 114.1) for declarative intent templates.

### Semantic Theming (`ferro-theme`, Phase 99)
- New crate with a fixed semantic token vocabulary (`ferro-theme/v1`) and intent templates.
- `ThemeMiddleware`, `ThemeResolver` trait, three concrete resolvers.
- Inline CSS injection into JSON-UI `<head>`.
- Render, layout, and config migrated to semantic Tailwind v4 tokens (`bg-primary`, `text-text-muted`, …).
- `make:theme` CLI scaffolding.
- Dark-mode WCAG 4.5:1 contrast verified.

### AI Classification + Confirmations (`ferro-ai`, Phase 100)
- New crate with classification types, provider trait, and `AnthropicProvider` with retry.
- `ConfirmationStore` trait and `InMemoryConfirmationStore` for human-in-the-loop flows.
- MCP tools for AI primitives.

### WhatsApp (`ferro-whatsapp`, Phase 101)
- New crate with outbound message sender, webhook HMAC verification, deduplication store.
- Events: `WhatsAppTextReceived`, `WhatsAppStatusUpdate`; job `ProcessWhatsAppWebhook`.
- CLI scaffold and MCP tools.

### Deploy Scaffolding (Phases 122, 122.1, 122.2)
- `ferro docker:init` — parameterized Dockerfile + dockerignore renderer.
- `ferro do:init` — DigitalOcean `app.yaml` renderer with envs/databases/workers blocks, DO app name sanitization, dev-default placeholder substitution, worker filtering (excludes test/dev/debug bins).
- `FerroDeployMetadata` reader, `.env` parser with SECRET classifier, inline-comment stripping, `rewrite_ferro_version` helper.
- Shared project introspection module (`ferro-cli/src/project`).

### Doctor, Routes JSON, CI Scaffold (Phase 124)
- `ferro doctor` — trait-based check framework with 7 checks (later revised to SCOPE §12), human and JSON output, exit codes.
- `ferro generate-routes --json` — stable JSON schema for routes.
- `ignore_patterns.toml` single source of truth with `ferro ignore:sync`.
- `ferro ci:init` — CI workflow template renderer, wired into `do:init`.

### Module Scaffolder (Phase 125)
- `ferro make:module` — module stub templates for handler/service/model layouts.
- `ferro-json-ui` runtime split into per-concern submodules.

### Framework Consolidation (v11.0, Phases 108-114)
- P0 documentation accuracy fixes: `ferro_rs::` → `ferro::`, CLI stubs replaced with real examples, S3 status and README corrected.
- 12 missing CLI command bodies added to the reference.
- MCP tool description audit; code template import patterns corrected.
- Service Projections and derive macros documentation pages added.
- Introduction rewritten with agent-first thesis; new "Working with Agents" guide.
- MCP Tools sections added to 17 feature documentation pages.
- Cargo.toml metadata gaps fixed across publication target crates.
- All 50 remaining `missing_docs` warnings in `framework` resolved.

## Bug Fixes

### JSON-UI
- Double-stacked `space-y-4` in `render_form_section` removed.
- Italian localization: Actions → Azioni.
- Table component wrapper spacing corrected (`flex-wrap gap-4`).
- Select and Tabs tree-walker fixes.
- StatCard SVG icon escaping.
- Sidebar nav icons render as raw SVG.
- Card footer `justify-between` regression test update.
- KanbanBoard: muted badge for zero-count columns, tap-to-open card menus, horizontal scroll restored, fixed positioning for card menus.
- DataTable row-action URL resolution from handler field, explicit `action.url` preserved during route resolution.
- ProductTile dispatches `input` events for form-guard compatibility; form guard skips ProductTile +/- buttons.
- PageHeader wraps actions on mobile.
- Grid uses `w-full`.
- Calendar cells use collapsed borders.
- Mobile padding reduced (`px-3 py-4`).
- Breadcrumb and title fused into one inline flow.
- EmptyState and CalendarCell Apple-Calendar-style polish.
- Image `object-top` anchor.
- Auth layout uses `bg-card`.

### Framework / Tailwind
- Tailwind v4 CDN upgrade and class naming alignment.
- Dark-mode L values adjusted for WCAG 4.5:1.
- Theme docs rewritten to match real API.

### Deploy (122.1)
- Inline comments stripped from `.env.example` values.
- `sanitize_do_app_name` helper.
- Dev-default env values substituted with placeholders.
- Test/dev/debug bins filtered from DO workers block.

### Other
- `/prenotazioni` path guard on `reload_kanban` SSE handler.
- `ferro-projections` `reconstruct_service_def` now parses action details, guards, and transition guards.
- Webhook scaffold templates corrected to use `queue_dispatch` and struct literals (`96-07`).
- Nonexistent `sometimes` validation rule replaced with `nullable` in MCP code templates.

## Internal

- v9.0 Service Projections milestone (Phases 84-94) — 10 phases, research, plans, execution, protocol spec.
- v10.0 JSON-UI Visual Overhaul (Phases 102-107) — font namespace, surface elevation, typography, form polish, interactive states, component details.
- v11.0 Framework Consolidation Audit (Phases 108-114) — accuracy, CLI reference, MCP accuracy, documentation coverage, agent-first philosophy, pattern coherence, metadata.
- Phase 113 pattern coherence: standardized imports, handler macros, error propagation; consolidated `COMPONENT_CATALOG` to `ferro-json-ui`.
- Phase 122.2 simplification: deleted deploy MCP tools, deleted obsolete deploy modules, stubbed legacy docker/do commands, made `ferro-mcp` library-only and launched in-process.
- Comprehensive test suites added across `ferro-projections`, `ferro-json-ui`, `ferro-theme`, `ferro-stripe`, `ferro-whatsapp`, `ferro-ai`, deploy golden fixtures.
- Publish workflow updated for new crates (`ferro-projections`, `ferro-api-mcp`, `ferro-stripe`, `ferro-whatsapp`, `ferro-ai`, `ferro-theme`).
- Milestone artifacts for v10.0 and v11.0 archived.

## Upgrade Guide

1. **Update your dependency:** bump `ferro` (and `ferro-macros`, etc.) to `0.2.0` in `Cargo.toml`.
2. **Fix import paths:** replace any `use ferro_rs::...` with `use ferro::...`. Public API re-exports are only available through `ferro`.
3. **Validation rules:** replace any use of `sometimes` with `nullable` in your `Validator` rule chains.
4. **Regenerate deploy scaffolding:**
   ```bash
   ferro docker:init --force
   ferro do:init --force
   ```
   Review the new `Dockerfile`, `.dockerignore`, and `.do/app.yaml` — worker entries, env placeholders, and app name formatting will differ.
5. **Remove deprecated CLI usage:** drop `--ferro-ref`, `--region`, `--repo`, and `--runtime-deps` flags from any scripts. Remove calls to `ferro deploy:check` and switch to `ferro doctor`.
6. **Remove deploy MCP tool calls:** if you were invoking `deploy_check`, `deploy_diff_env`, or `runtime_requirements` via MCP, migrate to running `ferro doctor` (supports `--json`).
7. **Run `ferro ignore:sync`** to regenerate `.gitignore` and `.dockerignore` from the new `ignore_patterns.toml` SoT. Move any custom patterns into the SoT file.
8. **JSON-UI:**
   - Audit any direct imports from `ferro_json_ui` internals; prefer `ferro::json_ui::*` re-exports.
   - If you have a custom theme, migrate any `--font-family-*` tokens to the `--font-*` namespace and verify Tailwind v4 compatibility.
   - If you override `JsonUiConfig::body_class`, note that the default now includes `dark`.
   - Replace custom Modal implementations with the native `<dialog>` pattern emitted by the renderer.
9. **Theming:** adopt `ferro-theme` if you want semantic token control. Custom render output targeting hardcoded color classes (`bg-blue-600`, …) should migrate to semantic classes (`bg-primary`, …) to benefit from theme overrides.
10. **Projections (optional):** enable the `projections` feature and run `ferro make:projection --from-model <Model>` to generate initial projections. Use `ferro projection:check` before shipping.
11. **Doctor:** run `ferro doctor` after upgrading. It replaces `deploy:check` and validates the SCOPE §12 check list.
12. **Run the test and lint suite:**
    ```bash
    cargo fmt --all -- --check
    cargo clippy --all --all-targets -- -D warnings
    cargo test --all-features
    ```
