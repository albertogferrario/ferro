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

### Phase 95: Multi-tenant middleware

**Goal:** Add TenantMiddleware with pluggable resolver chain (Subdomain, Header, Path, JWT), task-local TenantContext, TenantScope query helper for cross-tenant data isolation, and FromRequest handler extraction.
**Requirements**: [MT-01, MT-02, MT-03, MT-04, MT-05, MT-06, MT-07, MT-08, MT-09, MT-10]
**Depends on:** Phase 94
**Plans:** 2/3 plans executed

Plans:
- [ ] 95-01-PLAN.md — Core types: TenantContext, task-local context, TenantResolver trait, TenantLookup with moka cache
- [ ] 95-02-PLAN.md — TenantMiddleware with resolver chain + 4 concrete resolver implementations
- [ ] 95-03-PLAN.md — TenantScope query scoping, FromRequest extractor, lib.rs re-exports, documentation

### Phase 96: Stripe integration

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 95
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd:plan-phase 96 to break down)

### Phase 97: QR code generation

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 96
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd:plan-phase 97 to break down)

### Phase 98: Tenant-aware background jobs

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 97
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd:plan-phase 98 to break down)

### Phase 99: ferro-json-ui stable release

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 98
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd:plan-phase 99 to break down)

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

**Total: 22 milestones shipped, 197 plans.**
