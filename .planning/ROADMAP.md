# Roadmap: Ferro Framework

## Milestones

- ✅ [**v1.0 DX Overhaul**](milestones/v1.0-ROADMAP.md) — Phases 1-12 (shipped 2026-01-16)
- ✅ [**v2.0 Rebrand**](milestones/v2.0-ROADMAP.md) — Phases 13-22 (shipped 2026-01-16)
- ✅ **v2.0.1 Macro Fix** — Phase 22.1-22.3 (shipped 2026-01-17)
- ✅ [**v2.0.2 Type Generator Fixes**](milestones/v2.0.2-ROADMAP.md) — Phase 22.4-22.9 (shipped 2026-01-17)
- ✅ [**v2.0.3 DO Apps Deploy**](milestones/v2.0.3-ROADMAP.md) — Phase 22.10 (shipped 2026-01-17)
- ✅ [**v2.1 Inertia DX & Fixes**](milestones/v2.1-ROADMAP.md) — Phases 33-34 (shipped 2026-01-17)
- ✅ [**v2.2 CLI Improvements**](milestones/v2.2-ROADMAP.md) — Phases 35-37 (shipped 2026-02-09)
- 🚧 **v3.0 JSON-UI** — Phases 23-32 (in progress)

---

### 🚧 v3.0 JSON-UI (In Progress)

**Milestone Goal:** Add JSON-based UI rendering as an alternative to Inertia for rapid, beautiful UI without frontend builds.

#### Phase 23: JSON-UI Schema

**Goal**: Define core JSON schema for UI elements (components, props, visibility rules, actions)
**Depends on**: v2.1 complete
**Research**: Likely (study json-render patterns, JSON schema design)
**Research topics**: json-render by Vercel, component catalog patterns, action declaration

Plans:
- [x] 23-01: Core JSON-UI schema types (completed 2026-02-09)
- [x] 23-02: Framework integration with render API (completed 2026-02-09)

#### Phase 24: Component Catalog

**Goal**: Implement default components: Table, Form, Card, Input, Button, Alert, Badge, Modal, etc.
**Depends on**: Phase 23
**Research**: Unlikely (internal patterns)

Plans:
- [x] 24-01: Enriched component props and shared variant enums (completed 2026-02-09)
- [x] 24-02: Form field and utility components (completed 2026-02-09)
- [x] 24-03: Navigation and layout components (completed 2026-02-09)

#### Phase 25: Data Binding

**Goal**: JSONPath-based data binding to handler props and responses
**Depends on**: Phase 24
**Research**: Unlikely (internal patterns)

Plans:
- [x] 25-01: Data path resolver and form field data_path (completed 2026-02-09)
- [x] 25-02: View data integration and framework re-exports (completed 2026-02-09)

#### Phase 26: Action System

**Goal**: Map declared actions to Ferro handlers with form submissions and confirmations
**Depends on**: Phase 25
**Research**: Unlikely (internal patterns)

Plans:
- [x] 26-01: Action URL field and resolve_actions tree walker (completed 2026-02-09)
- [x] 26-02: Framework bridge integration (completed 2026-02-09)

#### Phase 27: Validation Integration

**Goal**: Connect to existing Ferro validation system, display errors in components
**Depends on**: Phase 26
**Research**: Unlikely (internal patterns)

Plans:
- [x] 27-01: Error resolver and view errors field (completed 2026-02-09)
- [x] 27-02: Framework render integration and re-exports (completed 2026-02-09)

#### Phase 28: HTML Renderer

**Goal**: Rust-based JSON→HTML renderer outputting Tailwind classes
**Depends on**: Phase 27
**Research**: Likely (templating approaches, HTML generation in Rust)
**Research topics**: maud, askama, or custom builder patterns

Plans:
- [x] 28-01: HTML render engine with tree walker and leaf component renderers (completed 2026-02-09)
- [x] 28-02: Container/form component renderers and framework integration (completed 2026-02-09)

#### Phase 29: Layout System

**Goal**: Layouts, partials, and slots for page structure
**Depends on**: Phase 28
**Research**: Unlikely (internal patterns)

Plans:
- [x] 29-01: Layout trait, registry, default layouts, and partials in ferro-json-ui (completed 2026-02-09)
- [x] 29-02: Framework integration and re-exports (completed 2026-02-09)

#### Phase 30: CLI Scaffolding

**Goal**: `ferro make:json-view` command to generate JSON view files
**Depends on**: Phase 29
**Research**: Retroactive (completed 2026-02-09)

Plans:
- [x] 30-01: AI-powered `make:json-view` command with Anthropic API and static fallback (completed 2026-02-09)
- [x] 30-02: Anthropic API best practices — system prompt, prefill, caching, few-shot, Sonnet default (completed 2026-02-09)

#### Phase 31: MCP UI Tools

**Goal**: MCP tools to generate and inspect JSON-UI specs from models/routes
**Depends on**: Phase 30
**Research**: Unlikely (internal patterns)

Plans:
- [ ] 31-01: TBD

#### Phase 32: Documentation

**Goal**: Guides, component reference, and examples for JSON-UI
**Depends on**: Phase 31
**Research**: Unlikely (internal patterns)

Plans:
- [ ] 32-01: TBD

---

## Completed Milestones

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
| v3.0 JSON-UI | 23-32 | 13/? | 🚧 In Progress | - |

## Progress (v3.0 JSON-UI)

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
| 31. MCP UI Tools | 0/? | Not started | - |
| 32. Documentation | 0/? | Not started | - |
