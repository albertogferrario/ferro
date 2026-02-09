# Ferro Framework

## What This Is

A developer experience and agent-friendliness optimized web framework for Rust. Ferro enables AI agents to build complete web applications from natural language descriptions — with reduced boilerplate, deep introspection via MCP, and intelligent CLI scaffolding.

## Core Value

Agents can go from "I want an app that does X" to a working, deployed application with minimal friction. Every framework decision optimizes for agent comprehension and generation capability.

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

### Active

<!-- Current scope. Building toward these. -->

- [ ] Publish to crates.io (manual step using PUBLISHING.md)
- [ ] Public announcement and marketing

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- New major features (payments, subscriptions, etc.) — focus is publishing, not feature expansion
- Frontend framework changes — React/Inertia stack stays as-is
- Database driver changes — SeaORM works, no need to replace

## Context

**Current State:**
- 60,000 lines of Rust across 11 crates
- Framework ready for crates.io publication
- Sample application demonstrating all capabilities
- Comprehensive MCP introspection (30+ tools)

**Tech Stack:**
- Rust 2021 edition
- Axum web framework
- SeaORM database layer
- React/Inertia.js frontend
- Redis for queue/cache/broadcast

**Primary use case:** Agent-built applications for non-technical users. This requires:
- Patterns simple enough for agents to reliably generate
- Introspection deep enough for agents to understand existing code
- Error messages clear enough for agents to self-correct

Reference codebase documentation in `.planning/codebase/`:
- ARCHITECTURE.md — Layer breakdown and request lifecycle
- STACK.md — Dependencies and tooling
- PATTERNS.md, HOTSPOTS.md, TESTING.md, CONVENTIONS.md, DOCUMENTATION.md

## Constraints

- **Compatibility**: Existing sample app works with framework
- **Rust Edition**: 2021 edition, no nightly-only features

## Key Decisions

<!-- Decisions that constrain future work. Add throughout project lifecycle. -->

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Agent-first over developer-first | Non-technical users via agents is the target market | ✓ Good |
| Breaking changes acceptable | No backwards compatibility constraint allows cleaner APIs | ✓ Good |
| FerroModel derive on entities | Apply derive to entity files (auto-generated) not model files | ✓ Good |
| ValidateRules not Validate | Avoid conflict with validator crate's `Validate` derive | ✓ Good |
| Tool vs Resource for MCP | Implemented features as tools rather than MCP resources for simpler agent consumption | ✓ Good |
| Rebrand to "ferro" | Name appropriate for crates.io publication and public release | ✓ Good |
| Alias pattern for migration | Keep code imports working during phased rename | ✓ Good |
| JSON fallback opt-in | `render_with_json_fallback()` per route for security | ✓ Good |
| accepts_json() on InertiaRequest | Framework-agnostic Accept header detection | ✓ Good |
| Docs URL: docs.ferro-rs.dev | Dedicated subdomain for documentation | ✓ Good |
| db: namespace for CLI | All database commands under unified db: prefix | ✓ Good |
| UpdateBuilder consumes model | Takes self for simpler ownership, matches create pattern | ✓ Good |
| Option<Option<T>> for nullable fields | None=unchanged, Some(None)=clear, Some(Some(v))=set | ✓ Good |
| Exclude frontend/src/types/ | Directory-level gitignore over individual files | ✓ Good |

---
*Last updated: 2026-02-09 after v2.2 CLI Improvements milestone*
