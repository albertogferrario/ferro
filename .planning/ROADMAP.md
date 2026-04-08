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
- ✅ **v7.1 Static File Serving** — Phase 69 (shipped)
- ✅ **v7.2 CI Stability** — Phase 70 (shipped)
- ✅ **v7.3 Vite Manifest** — Phase 71 (shipped)
- ✅ **v7.4 Security Hardening** — Phases 72-74 (shipped 2026-02-26)
- ✅ **v7.5 Type Generator Fix** — Phase 75 (shipped 2026-02-27)
- ✅ **v7.6 Default API Scaffold** — Phase 76 (shipped 2026-02-27)
- ✅ **v7.7 Validate & Fix API Scaffold** — Phase 77 (shipped 2026-02-28)
- ✅ **v7.8 Memory Leak Fixes** — Phase 78 (shipped 2026-02-28)
- ✅ **v8.0 Consumer MCP — OpenAPI Bridge** — Phases 79-82 (shipped 2026-02-28)
- ✅ **v8.1 API DX Polish** — Phase 83 (shipped 2026-02-28)
- ✅ [**v9.0 Service Projections**](milestones/v9.0-ROADMAP.md) — Phases 84-94 (shipped 2026-03-01)
- ✅ [**v10.0 JSON-UI Visual Overhaul**](milestones/v10.0-ROADMAP.md) — Phases 102-107 (shipped 2026-03-26)
- ✅ [**v11.0 Framework Consolidation Audit**](milestones/v11.0-ROADMAP.md) — Phases 108-114 (shipped 2026-04-05)
- ✅ [**v11.1 Template Renderer**](milestones/v11.1-ROADMAP.md) — Phase 114.1 (shipped 2026-04-05)
- 📋 **v12.0 JSON-UI v2 — Spec-Driven Rendering** — Phases 115-121 (planned, enriched with JSON Schema contract)

---

### ✅ v11.0 Framework Consolidation Audit (Shipped 2026-04-05)

Phases 108–114 — full details archived in [milestones/v11.0-ROADMAP.md](milestones/v11.0-ROADMAP.md).

---

### ✅ v11.1 Template Renderer (Shipped 2026-04-05)

Phase 114.1 — full details archived in [milestones/v11.1-ROADMAP.md](milestones/v11.1-ROADMAP.md).

---

### 📋 v12.0 JSON-UI v2 — Spec-Driven Rendering (Planned)

**Milestone Goal:** Pivot ferro-json-ui from nested component trees built in Rust to flat, JSON-first specs with JSON Schema as the validation contract. AI generates specs constrained by schema; developers write static JSON files validated by the same schema. Handlers become data-only providers.

**Context:** Three proven approaches inform this design:
- **Vercel json-render** (13k+ GitHub stars, Jan 2026): flat element maps, Zod-defined catalogs, AI-constrained generation. Validates the AI → JSON → UI thesis. Early issues: infinite re-render bugs, tight Zod coupling, expensive model dependency.
- **JSON Forms** (jsonforms.io): two-schema separation — JSON Schema for data, UI Schema for layout hints. Framework-agnostic core with pluggable renderers. Pain points: slow array rendering, limited layout types (4), low maintenance velocity.
- **react-jsonschema-form** (rjsf): JSON Schema → auto-generated forms with uiSchema overrides. Pain points: catastrophic performance with large oneOf (86 variants freezes UI), full re-render on every keystroke, schema version lag.
- **Production SDUI** (Airbnb, DoorDash, Lyft): GraphQL unions or protobuf for component typing, 3-tier hierarchy (Screen > Section > Component), version fragmentation is the hardest operational problem.

Ferro adopts the structural patterns (flat element map, props separation, formalized catalog with JSON Schema export) while keeping its server-authoritative model. The key enrichment over the original plan: **JSON Schema becomes the single source of truth** for validation, AI generation constraints, and tooling interop.

**Key risks identified by domain research:**
1. **Inner platform effect** (HIGH): Expression system must stay minimal (`$data` + `$template` only). Every SDUI system warns about schemas becoming programming languages.
2. **AI schema complexity** (HIGH): 36-component oneOf is too complex for LLM structured output. Two-tier strategy: concise prompt + per-component schemas.
3. **Manual JsonSchema impls** (HIGH): Component enum + recursive Props structs need ~200 lines of manual `JsonSchema` implementations.
4. **Schema size** (LOW): Estimated 40-80 KB for full catalog — acceptable for validation, too large for AI prompts.

**Rust ecosystem:** schemars 1.2.0 (generation, already in Cargo.lock) + jsonschema 0.45.0 (validation, to add). Both target JSON Schema 2020-12, no known incompatibilities. Compiled validators for zero per-request overhead.

**What changes:**
- Spec format: flat `elements` map + `root` key (replaces nested `Vec<ComponentNode>`)
- Props: separate `props` object per element (replaces flattened custom serialization)
- Catalog: machine-readable struct with `prompt()`, `validate()`, `json_schema()` (replaces `COMPONENT_CATALOG` const string)
- JSON Schema contract: per-component schemas via `schemars::JsonSchema` derives, full spec schema, standalone export
- Expressions: `$data` and `$template` resolved server-side at render time (enriches current `data_path`)
- Schema-driven projection: `Spec::from_service_def()` generates v2 specs from ServiceDef using JSON Schema type mapping
- Page loader: framework loads JSON spec files, merges handler data, renders HTML
- AI constraints: `catalog.prompt()` embeds JSON Schema for structured output; `catalog.validate()` uses `jsonschema` crate
- **Clean break**: v1 types (`JsonUiView`, nested `ComponentNode`) are removed entirely — no backward compatibility layer

**What stays:**
- Server-side HTML + Tailwind rendering (zero client JS runtime)
- Server-authoritative state (no client-side state management)
- Action → handler POST model (server round-trips)
- SSE for live updates
- Compile-time Rust type safety (bonus layer over runtime validation)
- Layout system (dashboard chrome is first-class)

## Phases

- [ ] **Phase 115: Spec v2 Data Structures** — New `Spec` type with flat element map, props separation, clean break from v1
- [ ] **Phase 116: Flat Element Renderer** — Update render pipeline to walk flat element map via ID lookups
- [ ] **Phase 117: Catalog & JSON Schema** — Machine-readable `Catalog` with per-component JSON Schema, full spec schema, validation, and `ferro json-ui:schema` CLI export
- [ ] **Phase 117.1: Schema-Driven Projections** — `Spec::from_service_def()` generates v2 specs from ServiceDef using JSON Schema type mapping, replacing hardcoded `field_to_input()` mappings
- [ ] **Phase 118: Server-Side Expressions** — `$data` path resolution and `$template` string interpolation at render time
- [ ] **Phase 119: Page Loader** — Framework loads JSON spec files, merges handler data, integrates with layouts
- [ ] **Phase 120: CLI & MCP Updates** — Update `make:json-view` and MCP tools for v2 format with JSON Schema as structured output constraint
- [ ] **Phase 121: Documentation & Field Test** — Update all JSON-UI docs, convert one gestiscilo page as proof of concept

## Phase Details

### Phase 115: Spec v2 Data Structures
**Goal**: Replace v1 types with the v2 spec format — flat element map, props separation, manual `JsonSchema` impl for Component enum, clean break
**Depends on**: Nothing (first phase of milestone)
**Requirements**: SPEC-01, SPEC-02, SPEC-03, SPEC-04
**Caveats** (from domain research):
  - Component enum has custom ser/de (not `#[serde(tag = "type")]`), so `#[derive(JsonSchema)]` won't work. Need manual impl building `oneOf` with `"type"` discriminator const. ~200 lines.
  - ~10 Props structs containing recursive `Vec<ComponentNode>` currently skip `JsonSchema` derive. Must add manual impls using `$ref: "#"` for self-references (schemars 1.x handles this).
  - Max nesting depth: enforce 3 levels (Screen > Section > Component) — matches Airbnb/DoorDash/Lyft production patterns.
**Success Criteria** (what must be TRUE):
  1. `Spec` struct exists with `root: String`, `elements: HashMap<String, Element>`, `title`, `layout`, `data`
  2. `Element` struct has `type_name`, `props: serde_json::Value`, `children: Vec<String>`, `action`, `visible`
  3. `Spec::from_json()` parses flat JSON specs and round-trips cleanly (serialize → deserialize = identity)
  4. `JsonUiView`, nested `ComponentNode`, and `Vec<ComponentNode>` patterns are deleted — clean break, no v1 types remain
  5. Schema version is `ferro-json-ui/v2`
  6. All Component variants and Props structs implement `JsonSchema` (manual impls where derive is blocked by custom ser/de or recursion)
  7. Spec nesting depth is validated: reject specs deeper than 3 levels

### Phase 116: Flat Element Renderer
**Goal**: New render pipeline that walks the flat element map by ID lookups, replacing the recursive tree walker
**Depends on**: Phase 115
**Requirements**: RENDER-01, RENDER-02, RENDER-03
**Success Criteria** (what must be TRUE):
  1. `render_spec_to_html(spec, data)` renders all component types from flat element map
  2. Element ID lookup handles missing children gracefully (skip + warn, don't panic)
  3. Action resolution works on flat elements (handler → URL via callback)
  4. Visibility evaluation works on flat elements (conditional rendering)
  5. Plugin components render correctly in v2 specs
  6. Old `render_to_html(view, data)` function is deleted

### Phase 117: Catalog & JSON Schema
**Goal**: Replace `COMPONENT_CATALOG` const string with a machine-readable `Catalog` backed by JSON Schema. Each component's props schema is derived from `schemars::JsonSchema` impls (Phase 115). The catalog validates specs, generates LLM prompts, and exports standalone schema files.
**Depends on**: Phase 116
**Requirements**: CAT-01, CAT-02, CAT-03, CAT-04, SCHEMA-01, SCHEMA-02, SCHEMA-03
**Caveats** (from domain research):
  - Full catalog schema (36-component oneOf) estimated at 40-80 KB — too large for AI system prompts. `catalog.prompt()` must emit a concise text summary, NOT the raw JSON Schema.
  - `jsonschema` crate doesn't optimize oneOf with discriminators (checks sub-schemas sequentially). Add pre-dispatch by `"type"` string for O(1) per-element validation.
  - Compile the schema validator once at startup via `jsonschema::validator_for()`, reuse for all requests. No per-request compilation.
  - AI models work reliably with per-component schemas but may produce malformed output when given 30+ component oneOf. Two-tier strategy: concise prompt + per-component structured output.
**Success Criteria** (what must be TRUE):
  1. `Catalog::build()` auto-discovers all Component variants with descriptions and JSON Schema per props struct
  2. `catalog.prompt()` generates a concise text system prompt summarizing components, props, and constraints — NOT the raw JSON Schema (too large for AI context)
  3. `catalog.validate(&spec)` validates specs using the `jsonschema` crate with compiled validator — returns typed errors for unknown component types, invalid props, missing required fields. Pre-dispatches by `"type"` string before full schema validation.
  4. `catalog.json_schema()` exports the complete JSON Schema document for the full v2 spec format (root + elements + all component types via `oneOf`)
  5. `catalog.component_schema("Card")` returns the JSON Schema for a single component's props — for targeted AI structured output generation
  6. `ferro json-ui:schema` CLI command exports the spec schema to stdout or file — consumable by external tools and IDEs
  7. `COMPONENT_CATALOG` const string is replaced by `catalog.prompt()` output
  8. Schema validator is compiled once (e.g., in `Catalog::build()`) and reused — no per-validation compilation

### Phase 117.1: Schema-Driven Projections
**Goal**: Bridge ferro-projections and ferro-json-ui v2 — generate v2 specs directly from ServiceDef definitions using JSON Schema type mappings instead of hardcoded `field_to_input()` / `field_to_column()` functions
**Depends on**: Phase 117
**Requirements**: PROJ-01, PROJ-02, PROJ-03
**Success Criteria** (what must be TRUE):
  1. `Spec::from_service_def(service, intents, ctx)` produces a valid v2 spec from a ServiceDef
  2. `DataType` + `FieldMeaning` → component selection uses the catalog's JSON Schema (not hardcoded match arms)
  3. Intent-to-layout mapping produces flat element specs (Browse → table layout, Collect → form layout, etc.)
  4. Output validates against `catalog.json_schema()` — projections and catalog are consistent by construction (two-pass: generate then validate)
  5. `render/json_ui.rs` (v1 JsonUiRenderer) and `render/field_map.rs` are replaced by the new schema-driven pipeline

### Phase 118: Server-Side Expressions
**Goal**: Add `$data` and `$template` expression types that resolve against handler data at render time. Hard cap: ONLY these two expression types. No `$if`, `$for`, `$state`, `$bind`.
**Depends on**: Phase 116
**Requirements**: EXPR-01, EXPR-02, EXPR-03
**Caveats** (from domain research):
  - Inner platform effect is the #1 strategic risk in SDUI. Every production SDUI system (Airbnb, DoorDash, Lyft) warns about schemas evolving into programming languages. `$data` and `$template` are the correct scope — resist pressure to add conditionals or loops.
  - Binding expressions (`{{query.data}}`) used by Appsmith/ToolJet/Retool are more flexible but harder to validate at compile time. Ferro's `$data`/`$template` approach is deliberately simpler.
**Success Criteria** (what must be TRUE):
  1. `{"$data": "path/to/value"}` in any props field resolves against `spec.data` before rendering
  2. `{"$template": "Hello, {user.name}!"}` interpolates data paths within strings
  3. Expressions work in all props positions (string, number, boolean values)
  4. Missing data paths resolve to `null`/empty — never panic
  5. Expressions are evaluated before component rendering, so renderers receive resolved concrete values
  6. No other expression types exist — only `$data` and `$template`. This is a hard architectural constraint, not a backlog item.

### Phase 119: Page Loader
**Goal**: Framework-level support for loading JSON spec files and merging with handler-provided data
**Depends on**: Phase 118
**Requirements**: LOAD-01, LOAD-02, LOAD-03
**Success Criteria** (what must be TRUE):
  1. `Spec::from_file("path/to/page.json")` or `include_str!()` loads and parses specs
  2. Loaded specs are validated against `catalog.json_schema()` at load time using the compiled validator — invalid specs fail fast with clear errors
  3. Handler data merges into `spec.data` (handler data takes precedence over spec defaults)
  4. Layout data (sidebar, header, sse_url) injects automatically for dashboard-layout specs
  5. Loaded specs are cached (compiled once, reused across requests)
  6. Dev mode: file watcher reloads specs on change (hot reload without recompilation)

### Phase 120: CLI & MCP Updates
**Goal**: Update all AI-facing tools to generate v2 specs using two-tier AI strategy (concise prompt + per-component structured output)
**Depends on**: Phase 117, Phase 119
**Requirements**: TOOL-01, TOOL-02, TOOL-03, TOOL-04
**Caveats** (from domain research):
  - Two-pass AI generation reduces hallucination: generate description first, then structured spec. v0.dev and Lovable both use this pattern.
  - LLMs hallucinate to fill arrays — may generate unnecessary components. Validate AI output against schema and flag suspiciously large specs.
  - Token overhead: JSON output costs ~2-3x tokens vs free text. Per-component schema keeps overhead manageable.
**Success Criteria** (what must be TRUE):
  1. `ferro make:json-view` generates v2 flat specs using two-pass generation (describe → structure)
  2. MCP `json_ui_generate` tool uses `catalog.prompt()` for concise context and `catalog.component_schema()` for per-component structured output
  3. MCP `json_ui_catalog` tool exposes JSON Schema per component (replaces text-only catalog inspection)
  4. MCP `json_ui_inspect` tool works with v2 format and reports validation errors against schema
  5. All code templates in ferro-mcp use v2 spec format
  6. No references to v1 types remain in CLI or MCP code
  7. Generated specs are validated against `catalog.json_schema()` before being returned to the user

### Phase 121: Documentation & Field Test
**Goal**: Complete docs rewrite for v2 and validate with a real gestiscilo page conversion
**Depends on**: Phase 120
**Requirements**: DOC-01, DOC-02, FIELD-01
**Success Criteria** (what must be TRUE):
  1. All JSON-UI documentation pages rewritten for v2 spec format with flat element examples — no v1 references remain
  2. JSON Schema export documented with usage examples (IDE validation, external tool integration, AI structured output)
  3. Expression system documented with explicit "hard cap" rationale — only `$data` and `$template`, with explanation of why no `$if`/`$for`
  4. One gestiscilo dashboard page (e.g., pagamenti) converted from Rust component tree to JSON spec file — handler reduced to data-only
  5. Converted page renders identically to the Rust-built version

## Progress

**Execution Order:**
Phases execute in order: 115 → 116 → 117 → 117.1 → 118 (parallel with 117) → 119 → 120 → 121

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 115. Spec v2 Data Structures | 0/? | Not started | - |
| 116. Flat Element Renderer | 0/? | Not started | - |
| 117. Catalog & JSON Schema | 0/? | Not started | - |
| 117.1. Schema-Driven Projections | 0/? | Not started | - |
| 118. Server-Side Expressions | 0/? | Not started | - |
| 119. Page Loader | 0/? | Not started | - |
| 120. CLI & MCP Updates | 0/? | Not started | - |
| 121. Documentation & Field Test | 0/? | Not started | - |

**v12.0 scope is held firm.** No expansion beyond the 8 phases above. The projection / intent abstraction already exists in v9.0 ferro-projections; v12.0 refines the rendering target.

---

## v1.0 Criteria

Ferro v1.0 is the first release where the framework is considered feature-complete for its target domain. No target date.

**Modality:**
- Visual modality complete (HTML + Tailwind, server-rendered).
- Additional rendering modalities (audio, physical) are out of scope for v1.0.

**Projection / intent validation:**
- Validated through real-world applications and a synthetic catalog of canonical app classes covering the seven intents.

**Quality bars:**
- Conceptual coherence pass complete across all 20 crates.
- Beauty across four dimensions: aesthetic, conceptual, operational, compressive.

---

## Future Milestones (v2.0+)

Items intentionally out of v1.0 scope. No phase numbers, no dates.

| Item | Target | Notes |
|------|--------|-------|
| **Multimodal generation exploration** | exploratory | Evaluate whether the seven intents (Browse, Focus, Collect, Process, Summarize, Analyze, Track) generalize cleanly to non-visual rendering targets. Inform any required revision of the intent vocabulary. |
| **Audio modality renderer** | v2.0+ | Render projections as voice / conversational interfaces. May require intent vocabulary revision. |
| **Physical modality** | v3.0+ | Haptic, gesture, and tangible rendering targets. |

---

## Design Principles

Operating principles applied across every phase. See [`.planning/VISION.md`](VISION.md) for the full design philosophy.

- **Substance-first investment ordering:** compressive → operational → conceptual → aesthetic.
- **Continuous conceptual coherence:** every phase pays a coherence cost against the existing 20 crates at write-time; no deferred cleanup milestones.
- **Validation through real-world applications and synthetic catalogs:** the projection / intent system is iterated against both.

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

### Phase 122: Deploy scaffold core rewrite

**Goal:** Rewrite ferro-cli `docker_init`/`do_init` and templates so generated `Dockerfile` + `.do/app.yaml` work for real Ferro apps with zero hand-patching. Conditional frontend stage, multi-binary support, runtime extras hook, themes/lang/public/migrations detection, GITHUB_TOKEN ARG, rust-toolchain.toml pickup, workspace-aware cargo-chef recipe. Path→git ferro dep rewrite via generated `scripts/rewrite-ferro-deps.sh` invoked from Dockerfile + CLI pre-flight verifying ferro git ref is pushed/reachable. `app.yaml` gains `--region`, envs block from `.env.example` with auto SECRET classification, optional `databases:` block, `workers:` for non-server bins. CLI commands gain `--force`, walk-up Cargo.toml lookup, owner/repo validation, shared `project::package_name()` helper. `.dockerignore` adds `database.db`, `*.sqlite*`, `.planning/`, `storage/`, `data/`. Validation: regenerating in gestiscilo and mkmenu produces working builds with zero hand edits. See `phases/122-deploy-scaffold-core-rewrite/SCOPE.md`.
**Requirements**: TBD
**Depends on:** Phase 121
**Plans:** 8/8 plans complete

Plans:
- [x] TBD (run /gsd:plan-phase 122 to break down) (completed 2026-04-07)

### Phase 123: Deploy MCP tools

**Goal:** Expose deploy lifecycle helpers via ferro-mcp: `deploy_check` (pre-flight against missing env, path deps, sqlite in DATABASE_URL, dirty git tree, missing Dockerfile/app.yaml), `deploy_diff_env` (local .env vs .do/app.yaml drift), `runtime_requirements` (scan source for chromium/ffmpeg/etc and report needed runtime apt packages). Read-only.
**Requirements**: TBD
**Depends on:** Phase 122
**Plans:** 5/5 plans complete

Plans:
- [x] 123-01-PLAN.md — runtime_deps registry + Cargo.toml scanner in ferro-cli
- [x] 123-02-PLAN.md — ferro-mcp depends on ferro-cli; deploy_common re-exports
- [x] 123-03-PLAN.md — deploy_check MCP tool (severity-tagged pre-flight report)
- [x] 123-04-PLAN.md — deploy_diff_env MCP tool (.env vs app.yaml drift)
- [x] 123-05-PLAN.md — runtime_requirements MCP tool + Dockerfile cross-check

### Phase 124: Doctor, introspection, CI scaffold

**Goal:** `ferro doctor` (toolchain + DB + migrations + env completeness in one command), `ferro routes --json` (machine-readable for MCP/agents), CI workflow scaffold dropped by `do:init` (`.github/workflows/ci.yml` running `cargo test`, `ferro api:check`, `ferro validate:contracts`), keep `.dockerignore` and `.gitignore` in sync via shared template.
**Requirements**: D-01..D-22 (decisions in 124-CONTEXT.md)
**Depends on:** Phase 122, Phase 123
**Plans:** 5/5 plans complete

Plans:
- [x] 124-01-PLAN.md — ignore_patterns.toml single source of truth + ignore:sync command
- [x] 124-02-PLAN.md — ferro generate-routes --json (stable schema for agents/MCP)
- [x] 124-03-PLAN.md — GitHub Actions CI workflow template + ferro ci:init command
- [x] 124-04-PLAN.md — ferro doctor (7 health checks, human + JSON output)
- [x] 124-05-PLAN.md — wire ci.yml generation into do:init

### Phase 125: Module scaffolder and json-ui runtime split

**Goal:** `ferro make:module <name>` creating `controllers/`, `models/`, `views/`, `routes.rs` skeleton enforcing feature-module convention. Split ferro-json-ui monolithic IIFE in `runtime.rs` into named functions (tabs, SSE, toasts, sidebar) with a small dispatcher, still emitted as one file but testable in isolation.
**Requirements**: TBD
**Depends on:** Phase 122
**Plans:** 1/2 plans executed

Plans:
- [x] 125-01-PLAN.md — ferro make:module command + stub templates + clap wiring (D-01..D-05)
- [ ] 125-02-PLAN.md — ferro-json-ui runtime split into per-concern submodules + ferroRuntime dispatcher (D-06..D-11)

### Phase 122.2: Deploy simplification

**Goal:** Replace the Phase 122/122.1/123/124 deploy machinery with a simpler, heuristic-light, provider-honest shape. Cut custom logic from ~1500 LOC to ~375 LOC. Reduce surviving heuristics from 6 to 1. Delete the 3 deploy MCP tools, revert the ferro-cli↔ferro-mcp circular-dep workaround to in-process launch, delete the golden fixture suite, and fold surviving deploy checks into `ferro doctor`. New Cargo.toml `[package.metadata.ferro.deploy]` schema (runtime_apt, copy_dirs, ferro_version) drives the new Dockerfile renderer. New `.env.production` key-only parser replaces the `.env.example` SECRET classifier. `.do/app.yaml` becomes a one-shot starter owned by the user after scaffold. See `phases/122.2-deploy-simplification/SCOPE.md`.
**Requirements**: SCOPE §1..§13 + Verification
**Depends on:** Phase 122, 122.1, 123, 124
**Plans:** 8/9 plans executed

Plans:
- [x] 122.2-01-PLAN.md — Delete ferro-mcp deploy tools (SCOPE §9)
- [x] 122.2-02-PLAN.md — Revert ferro-cli↔ferro-mcp circular dep, in-process mcp launch (SCOPE §10)
- [x] 122.2-03-PLAN.md — Delete obsolete ferro-cli deploy modules, commands, golden tests (SCOPE §6, §8, §11, §13)
- [x] 122.2-04-PLAN.md — Stub docker_init and do_init before Wave 2 rewrite (SCOPE §2-§5 prep)
- [x] 122.2-05-PLAN.md — Metadata reader + env_production parser + rewrite_ferro_version rewriter (SCOPE §1, §2, §6)
- [x] 122.2-06-PLAN.md — New Dockerfile renderer + static ignores + docker:init rewrite (SCOPE §2, §3, §8)
- [x] 122.2-07-PLAN.md — New app.yaml renderer + do:init rewrite + decouple ci.yml (SCOPE §4, §5, §7)
- [x] 122.2-08-PLAN.md — ferro doctor 9-check revision (SCOPE §12)
- [ ] 122.2-09-PLAN.md — Live UAT against gestiscilo-it/app + phase-end gate (SCOPE §Verification)

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
| v11.0 Framework Consolidation Audit | 108-114 | 13 | ✅ Shipped | 2026-04-05 |
| v11.1 Template Renderer | 114.1 | 1 | ✅ Shipped | 2026-04-05 |
| v12.0 JSON-UI v2 — Spec-Driven Rendering | 115-121 | ? | 📋 Planned | - |

**Total: 23 milestones shipped, 205 plans complete. 13 plans in progress.**
