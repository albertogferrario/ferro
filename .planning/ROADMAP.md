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
- ✅ [**v10.0 JSON-UI Visual Overhaul**](milestones/v10.0-ROADMAP.md) — Phases 102-107 (shipped 2026-03-26)
- 🚧 **v11.0 Framework Consolidation Audit** — Phases 108-114 (in progress)

---

### 🚧 v11.0 Framework Consolidation Audit (In Progress)

**Milestone Goal:** Comprehensive audit and fix of documentation accuracy, completeness, and agent-first philosophy consistency across all 14 crates — preparing Ferro for crates.io publication.

## Phases

- [ ] **Phase 108: P0 Accuracy Fixes** - Eliminate actively wrong information that contaminates all downstream work
- [ ] **Phase 109: CLI Reference Completeness** - Document all 13 missing CLI commands in reference/cli.md
- [ ] **Phase 110: MCP Tool Accuracy** - Verify generation_hints and code_templates against current framework exports
- [ ] **Phase 111: Documentation Coverage** - Create missing user docs for Service Projections and derive macros
- [ ] **Phase 112: Agent-First Philosophy** - Rewrite introduction and add agent workflow guides
- [ ] **Phase 113: Pattern Coherence** - Standardize code examples and resolve COMPONENT_CATALOG duplication
- [ ] **Phase 114: Metadata & Publication Readiness** - Fix Cargo.toml gaps, missing_docs, and stub READMEs

## Phase Details

### Phase 108: P0 Accuracy Fixes
**Goal**: Remove all actively wrong information from user-facing docs — stale import paths, TODO stubs presented as working code, and false claims about feature status
**Depends on**: Nothing (first phase of milestone)
**Requirements**: ACC-01, ACC-02, ACC-03, ACC-04, ACC-05
**Success Criteria** (what must be TRUE):
  1. All `ferro_rs::` occurrences in docs replaced with `ferro::` — grep finds zero matches in docs/src/
  2. CLI reference examples contain no `// TODO: Implement` stubs — every example shown is runnable
  3. README roadmap section accurately reflects shipped features — JSON-UI marked shipped, not "Work in Progress"
  4. Storage documentation accurately describes S3 as shipped — "coming soon" note removed
  5. All MCP tool count claims in docs reflect the actual 57 tools
**Plans**: TBD

Plans:
- [ ] 108-01: Import path normalization (ferro_rs:: → ferro::, atomic grep-replace across docs/src/)
- [ ] 108-02: README roadmap + storage docs + MCP tool count corrections + CLI stub removal

### Phase 109: CLI Reference Completeness
**Goal**: Every CLI command that exists in ferro-cli has a reference entry in docs — no undiscoverable commands
**Depends on**: Phase 108
**Requirements**: CLIMCP-01
**Success Criteria** (what must be TRUE):
  1. `reference/cli.md` contains entries for all 13 previously undocumented commands (api:check, clean, generate-routes, make:api, make:api-key, make:lang, make:policy, make:projection, make:stripe, make:theme, make:whatsapp, projection:check, validate-contracts)
  2. Each new entry follows the same format as existing entries (synopsis, flags, description, example)
  3. The count of documented commands in reference/cli.md matches the count of command files in ferro-cli/src/commands/
**Plans**: TBD

Plans:
- [ ] 109-01: Document all 13 missing CLI commands in reference/cli.md

### Phase 110: MCP Tool Accuracy
**Goal**: All 57 MCP tool responses carry accurate generation_hints that reflect current framework APIs, and code_templates.rs patterns compile against current framework exports
**Depends on**: Phase 109
**Requirements**: CLIMCP-02, CLIMCP-03
**Success Criteria** (what must be TRUE):
  1. generation_hints across all 57 MCP tools reference types and patterns that exist in framework/src/lib.rs
  2. code_templates.rs code snippets compile if pasted into a ferro project (no references to removed or renamed APIs)
  3. UpdateBuilder pattern in MCP templates matches current implementation (not legacy ActiveModel pattern)
**Plans**: TBD

Plans:
- [ ] 110-01: Audit and refresh generation_hints across all 57 MCP tool responses
- [ ] 110-02: Verify code_templates.rs patterns against current framework/src/lib.rs exports

### Phase 111: Documentation Coverage
**Goal**: Every shipped framework feature that agents and users need to understand has a user-facing documentation page
**Depends on**: Phase 110
**Requirements**: DOC-01, DOC-02, DOC-03
**Success Criteria** (what must be TRUE):
  1. docs/src/features/projections.md exists, is linked in SUMMARY.md, and covers the ServiceDef → IntentGraph → Renderer pipeline with a worked example
  2. FerroModel derive macro is documented in user docs with at least one complete usage example
  3. ValidateRules derive macro is documented in user docs with at least one complete usage example
**Plans**: TBD

Plans:
- [ ] 111-01: Create docs/src/features/projections.md (Service Projections user guide)
- [ ] 111-02: Document FerroModel and ValidateRules derive macros in user docs

### Phase 112: Agent-First Philosophy
**Goal**: Ferro's documentation leads with and consistently reinforces its agent-first identity — every feature page makes MCP tools discoverable
**Depends on**: Phase 111
**Requirements**: PHIL-01, PHIL-02, PHIL-03, PHIL-04
**Success Criteria** (what must be TRUE):
  1. introduction.md leads with agent-first value proposition — the phrase "agent-first" appears in the first paragraph and MCP is mentioned before any framework comparison
  2. A "Working with Agents" guide exists in docs that documents the MCP workflow (application_info → list_routes → get_handler → use CLI)
  3. Each feature documentation page lists the relevant MCP tools for that feature
  4. The agent-to-CLI workflow is documented end-to-end (agent reads MCP hints → selects CLI command → scaffolds code)
**Plans**: TBD

Plans:
- [ ] 112-01: Rewrite introduction.md with agent-first thesis
- [ ] 112-02: Create "Working with Agents" guide + add MCP tool references to all feature pages

### Phase 113: Pattern Coherence
**Goal**: All code examples in docs use consistent import style and idiomatic patterns, and the COMPONENT_CATALOG duplication has a documented resolution
**Depends on**: Phase 112
**Requirements**: COH-01, COH-02, COH-03, COH-04
**Success Criteria** (what must be TRUE):
  1. All code examples in docs use a single consistent import style — no mixed `use ferro::*` vs explicit multi-import
  2. All handler examples in docs use `#[handler]` macro — no legacy handler signatures
  3. All error propagation examples use `?` operator — no `.unwrap()` in doc examples
  4. COMPONENT_CATALOG duplication between ferro-cli and ferro-mcp is either resolved (shared source) or documented with a clear design decision in PROJECT.md
**Plans**: TBD

Plans:
- [ ] 113-01: Standardize import style and handler patterns across all doc examples
- [ ] 113-02: Fix error propagation examples + resolve or document COMPONENT_CATALOG duplication

### Phase 114: Metadata & Publication Readiness
**Goal**: All crates are publication-ready with complete Cargo.toml metadata, crate-level doc comments, and expanded READMEs
**Depends on**: Phase 113
**Requirements**: META-01, META-02, META-03, META-04
**Success Criteria** (what must be TRUE):
  1. ferro-broadcast, ferro-theme, and ferro-projections Cargo.toml files have no missing metadata fields (readme, categories, homepage as applicable)
  2. framework crate compiles with `#![warn(missing_docs)]` without new warnings introduced by this phase
  3. ferro-json-ui, ferro-lang, and ferro-whatsapp READMEs contain meaningful content beyond 9 lines
  4. ferro-json-ui and ferro-lang lib.rs files have crate-level `//!` doc comment blocks
**Plans**: TBD

Plans:
- [ ] 114-01: Fix Cargo.toml metadata gaps + add #![warn(missing_docs)] to framework crate
- [ ] 114-02: Expand stub READMEs + add //! crate-level doc comments to ferro-json-ui and ferro-lang

## Progress

**Execution Order:**
Phases execute in numeric order: 108 → 109 → 110 → 111 → 112 → 113 → 114

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 108. P0 Accuracy Fixes | 0/2 | Not started | - |
| 109. CLI Reference Completeness | 0/1 | Not started | - |
| 110. MCP Tool Accuracy | 0/2 | Not started | - |
| 111. Documentation Coverage | 0/2 | Not started | - |
| 112. Agent-First Philosophy | 0/2 | Not started | - |
| 113. Pattern Coherence | 0/2 | Not started | - |
| 114. Metadata & Publication Readiness | 0/2 | Not started | - |

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
| v11.0 Framework Consolidation Audit | 108-114 | 13 | 🚧 In progress | - |

**Total: 23 milestones shipped, 205 plans complete. 13 plans in progress.**
