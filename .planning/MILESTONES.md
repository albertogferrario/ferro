# Project Milestones: Ferro Framework

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
