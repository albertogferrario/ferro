# Architecture Research

**Domain:** Documentation/philosophy audit for a 14-crate Rust workspace
**Researched:** 2026-03-26
**Confidence:** HIGH (all findings from direct codebase inspection)

## Standard Architecture

### System Overview

The audit operates across four interconnected artifact layers. Inconsistencies live at the boundaries between them.

```
┌──────────────────────────────────────────────────────────────────┐
│                     docs/src/  (mdBook pages)                     │
│  introduction.md  the-basics/  features/  json-ui/  reference/   │
│  40 Markdown files describing how the framework is supposed to    │
│  work — the authoritative user-facing contract                    │
├──────────────────────────────────────────────────────────────────┤
│               framework/src/lib.rs  (re-export surface)           │
│  Single pub API surface. Everything an app imports comes from     │
│  here. Docs reference types, macros, and traits by their public   │
│  names — a rename here invalidates every doc page that shows it.  │
├──────────────────────────────────────────────────────────────────┤
│  14 Crates  (individual crate lib.rs + src/ implementations)      │
│  framework | ferro-cli | ferro-macros | ferro-events              │
│  ferro-queue | ferro-notifications | ferro-broadcast              │
│  ferro-storage | ferro-cache | ferro-mcp | ferro-inertia          │
│  ferro-json-ui | ferro-lang | ferro-api-mcp | ferro-projections   │
│  ferro-stripe | ferro-theme | ferro-ai | ferro-whatsapp           │
│  ferro-planning (internal tooling, not user-facing)               │
├──────────────────────────────────────────────────────────────────┤
│     ferro-mcp/src/tools/  (54 introspection tools)                │
│  Tools return code templates, explain routes/models, suggest      │
│  patterns. An agent reads these BEFORE reading docs — they are    │
│  the primary delivery channel for the agent-first philosophy.     │
├──────────────────────────────────────────────────────────────────┤
│     ferro-cli/src/  (50+ commands, templates/, commands/)         │
│  Generated code is what agents and humans run. Template           │
│  correctness is a doc promise. Wrong template = broken quickstart.│
└──────────────────────────────────────────────────────────────────┘
```

### Four Artifact Layers

| Layer | Files | Owner | Inconsistency Risk |
|-------|-------|-------|-------------------|
| Docs | `docs/src/**/*.md` (40 files) | Human-readable contract | Highest — written prose drifts after code changes |
| Public API | `framework/src/lib.rs` + each crate's `lib.rs` | Machine-checked by compiler | Medium — type/trait renames surface here |
| MCP tools | `ferro-mcp/src/tools/*.rs` (54 tools) | Agent-first delivery | High — tool descriptions and code templates age independently |
| CLI templates | `ferro-cli/src/templates/*.rs` + `ferro-cli/src/commands/*.rs` | Generated code | High — templates reference patterns that have evolved |

## Where Inconsistencies Hide in Multi-Crate Workspaces

This is the core question for the audit. Based on codebase inspection, inconsistencies cluster in five structural locations.

### Location 1: Docs Reference Old Import Paths

**What breaks:** A crate gets renamed or a re-export moves. The compiler catches call sites in Rust code, but Markdown code blocks are not compiled. The doc page still shows the old import.

**Specific risk in ferro:** `use ferro_rs::*` vs `use ferro::*` — the introduction page and several feature docs may show the pre-rebrand crate name. The whatsapp.md page explicitly shows `ferro-rs = { version = "0.1", features = ["whatsapp"] }` in the Cargo.toml example, which is the old crate name.

**Detection pattern:** `grep -r "ferro_rs" docs/` and `grep -r "ferro-rs" docs/` catch old references.

### Location 2: Docs Describe Features That Do Not Exist Yet

**What breaks:** A doc page was written speculatively or as part of planning, but the implementation was deferred. The page appears in SUMMARY.md and is publicly accessible.

**Specific risk in ferro:**
- `docs/src/features/multi-tenancy.md` — references `ferro_rs::` prefix (old name) and covers `register_tenant_capture_hook`, `FrameworkTenantScopeProvider`, `.for_tenant()` — these need verification against the actual framework codebase
- `docs/src/features/stripe.md` — `ferro-stripe` crate exists in the workspace but `ferro_stripe::testing::*` test helpers need verification
- `docs/src/features/whatsapp.md` — `ferro-whatsapp` crate exists but WhatsApp integration scope needs verification against `ferro-mcp/src/tools/whatsapp.rs`
- `docs/src/features/themes.md` — references `ThemeMiddleware`, `TenantThemeResolver`, `HeaderThemeResolver`, `DefaultResolver` — need to verify these types are exported from `framework/src/lib.rs`
- `docs/src/features/ai.md` — `ferro-ai` crate exists; classification/confirmation primitives need verification

**Detection pattern:** Cross-reference each type and function named in docs against `framework/src/lib.rs` exports and the relevant crate's `lib.rs`.

### Location 3: MCP Tool Descriptions Diverge from Actual API

**What breaks:** A tool's description text says "returns X" but the underlying implementation returns Y, or the tool's code template uses a pattern that the framework no longer supports.

**Specific risk in ferro:** The `code_templates.rs` tool returns code snippets agents copy verbatim. If these templates use deprecated patterns (e.g., direct `ActiveModel` construction instead of the `UpdateBuilder` pattern introduced in v2.2, or missing `#[handler]` macro usage), agents generate broken code.

**Key tool files to audit:**
- `ferro-mcp/src/tools/code_templates.rs` — the highest-impact tool; everything agents generate starts here
- `ferro-mcp/src/tools/application_info.rs` — describes the project; must reflect current crate count and philosophy
- `ferro-mcp/src/tools/generation_context.rs` — the "what to know before generating code" context; must be current

### Location 4: CLI Templates Use Outdated Patterns

**What breaks:** `ferro make:controller` generates a controller that uses a pattern the framework has since improved. Users running the CLI get code that works but contradicts current best practices.

**Specific risks in ferro:**
- `ferro-cli/src/templates/make.rs` — controller templates should use `#[handler]` macro; migration from pre-v1.0 patterns may linger
- `ferro-cli/src/templates/scaffold.rs` — scaffold templates may reference pre-v2.2 `ActiveModel` update patterns instead of `UpdateBuilder`
- `ferro-cli/src/templates/entity.rs` — entity templates must include `FerroModel` derive (confirmed present in tests, but verify the template string matches current macro API)
- The test suite in `ferro-cli/src/templates/mod.rs` verifies structure but not semantic correctness — it asserts `contains("#[handler]")` but not whether the generated code compiles

### Location 5: Philosophy Drift — Agent-First Claims vs Actual API Design

**What breaks:** The framework claims "agent-first" but specific APIs require knowledge that agents cannot infer from introspection alone. The MCP tools don't expose enough context to understand what a poorly-named type does.

**Specific risks in ferro:**
- `SavedInertiaContext` — an unusual pattern where you must save context before consuming the request. Documented in CLAUDE.md and `ferro-inertia`, but does the `application_info` or `generation_context` MCP tool surface this footgun?
- `Option<Option<T>>` for nullable update fields — the decision log shows this pattern; do code templates demonstrate it?
- `#[derive(ValidateRules)]` vs `#[derive(Validate)]` naming — the name was chosen to avoid a conflict with the `validator` crate. Do docs explain why, so agents don't "fix" it to `Validate`?
- `json_ui_generate` tool — documented in PROJECT.md as "consuming agent IS the LLM, avoids double-LLM calls"; does the tool description in `json_ui_generate.rs` explain this to agents reading it?

## Recommended Audit Order

The order should respect dependency direction: fix foundational problems before surface problems. A wrong import path in docs is irrelevant if the feature doesn't exist.

### Phase Order Rationale

```
Phase 1: Foundation Audit (public API surface)
    ↓
Phase 2: MCP Tool Accuracy (primary agent interface)
    ↓
Phase 3: CLI Template Correctness (generated code quality)
    ↓
Phase 4: Documentation Accuracy (user-facing prose)
    ↓
Phase 5: Philosophy Coherence (agent-first consistency)
```

**Why this order:**
- Phase 1 first: If `framework/src/lib.rs` exports are wrong or missing, every other layer (docs, MCP, CLI) is describing something agents cannot access.
- Phase 2 before docs: Agents read MCP tools before reading docs. MCP correctness has higher urgency than prose quality.
- Phase 3 before prose: Template bugs produce broken code immediately. A prose error delays an agent; a template error breaks its build.
- Phase 4 last among fixes: Prose is the most numerous artifact (40 files). Fix the code-based issues first so doc fixes can cite correct APIs.
- Phase 5 final: Philosophy coherence is an overlay on all layers — you can only evaluate it holistically after each layer is accurate.

### Crate Audit Order (within each phase)

Within each phase, audit crates in this order based on integration surface and risk:

```
1. framework         — core API, highest surface area, all other layers depend on it
2. ferro-macros      — proc macros are invisible in docs but critical for compilation
3. ferro-mcp         — primary agent-first delivery channel
4. ferro-cli         — generated code quality
5. ferro-inertia     — complex SavedInertiaContext footgun
6. ferro-json-ui     — large (426+ tests, 30 components), recently changed (v10.0)
7. ferro-theme       — new system, low code count, documentation likely thin
8. ferro-lang        — recent addition (v6.0), localization patterns
9. ferro-events      — established, lower risk
10. ferro-queue      — established, lower risk
11. ferro-notifications — established, lower risk
12. ferro-broadcast  — established, lower risk
13. ferro-cache      — established, lower risk
14. ferro-storage    — established, lower risk
15. ferro-projections — v9.0 feature, documentation freshness uncertain
16. ferro-api-mcp    — v8.0 feature, consumer bridge for external agents
17. ferro-stripe     — optional feature, high doc complexity
18. ferro-whatsapp   — optional feature, likely thin
19. ferro-ai         — newest, documentation freshness uncertain
```

## Component Responsibilities

| Component | Audit Responsibility | Files to Inspect |
|-----------|---------------------|------------------|
| `framework/src/lib.rs` | Canonical export surface — every public type must be here and named correctly | `framework/src/lib.rs` |
| `docs/src/` | User-readable truth — must match actual exported API | All 40 `.md` files |
| `ferro-mcp/src/tools/` | Agent-readable truth — tool descriptions must match current capabilities | 54 `.rs` files, especially `code_templates.rs`, `generation_context.rs`, `application_info.rs` |
| `ferro-cli/src/templates/` | Generated code quality — templates must use current best-practice patterns | `make.rs`, `scaffold.rs`, `entity.rs`, `auth.rs` |
| `ferro-cli/src/commands/` | CLI command descriptions shown in `--help` must match what commands actually do | 50+ `.rs` files |

## Data Flow

### How Inconsistencies Propagate to Agents

```
Agent reads: application_info tool
    ↓
Agent reads: generation_context tool
    ↓ (if stale: wrong patterns embedded)
Agent reads: code_templates tool
    ↓ (if stale: wrong patterns generated)
Agent generates code
    ↓ (if template wrong: build fails)
Agent reads: last_error tool
    ↓
Agent reads: docs/src/features/*.md
    ↓ (if stale: contradicts working code; agent confused)
Agent tries fix from docs
    ↓
[Loop or hallucination]
```

**Key insight:** Stale MCP tools cause more damage than stale docs because agents encounter MCP output before docs. Fix MCP tools before prose.

### How Inconsistencies Propagate to Human Developers

```
Developer runs: ferro make:controller
    ↓ (if template wrong: generates deprecated code)
Developer reads: docs/src/the-basics/controllers.md
    ↓ (if stale: contradicts generated code)
Developer reads: docs/src/features/*.md
    ↓ (if missing: no guidance for feature)
Developer reads: framework source directly
    ↓ (always accurate — source of truth)
```

## Architectural Patterns

### Pattern 1: Single Public API Surface (framework/src/lib.rs)

**What:** Every user-facing type, trait, and macro is re-exported from `framework/src/lib.rs`. Individual crate `lib.rs` files hold implementations; `framework/src/lib.rs` is the import façade.

**Audit implication:** One file to verify for completeness. Cross-reference every type named in docs against this file. If a type is in docs but not in `lib.rs`, it is either a documentation error or a missing export.

**When correct pattern breaks down:** Optional/feature-gated crates (`ferro-stripe`, `ferro-whatsapp`, `ferro-ai`) expose types through their own crate — docs must show the crate name, not `ferro::`. This is correct by design but inconsistency-prone.

### Pattern 2: Agent-First Tool Architecture (ferro-mcp)

**What:** Each MCP tool is a self-contained `.rs` file in `ferro-mcp/src/tools/`. Tools return formatted strings (not JSON schemas) designed for LLM context consumption. Tools include "generation hints" — snippets of the pattern to use.

**Audit implication:** Tool hint strings are informal prose embedded in Rust string literals. They are not tested. They drift silently. The `code_templates` tool is the highest-risk file: it provides copy-paste Rust code that agents use verbatim.

**How to audit:** For each code snippet in `code_templates.rs`, verify it compiles against the current framework API. This is the only reliable check.

### Pattern 3: CLI Template as Test Baseline

**What:** `ferro-cli/src/templates/mod.rs` contains ~800 lines of template tests. Tests verify structural presence (`contains("#[handler]")`) but not semantic correctness or compilation.

**Audit implication:** The test suite provides a partial baseline but misses semantic drift. A template can pass all tests while generating code that uses a deprecated pattern. The audit must go beyond `contains()` checks.

### Pattern 4: Documentation Philosophy (agent-first framing)

**What:** The framework's stated philosophy is "agent-first" — every API optimized for AI agent comprehension and generation. Docs should explain not just what to do, but why, in terms agents can use to infer analogous patterns.

**Audit implication:** "Agent-first" philosophy auditing is qualitative. For each feature doc, ask: "Could an agent generate correct code for this feature after reading only the `application_info` and `generation_context` MCP tools plus this doc page?" If no, the doc or the tool is incomplete.

## Integration Points

### Critical Boundary: docs ↔ framework/src/lib.rs

**Communication:** Prose references type names; compiler enforces types. No automated bridge.

**Inconsistency vector:** A type is renamed or moved; docs still show the old name. Compiler does not catch Markdown.

**Detection:** For every type/macro/function name in docs, `grep -r "TypeName" framework/src/lib.rs`. Missing hits = inconsistency.

### Critical Boundary: MCP tools ↔ actual framework behavior

**Communication:** Tool implementations read from disk (migrations, routes, source files). Some tools have hardcoded templates.

**Inconsistency vector:** `code_templates.rs` returns Rust code strings that were correct at writing time. Framework API evolves; templates do not auto-update.

**Detection:** Attempt to compile each template snippet in isolation or trace the pattern to current framework exports.

### Critical Boundary: CLI templates ↔ framework patterns

**Communication:** Templates are string literals in `ferro-cli/src/templates/`. No link to the framework code they reference.

**Inconsistency vector:** Template uses `ActiveModel` update pattern; framework now uses `UpdateBuilder`. Both patterns compile (the old pattern still works via SeaORM); the inconsistency is in best-practice guidance, not compilation.

**Detection:** Compare template patterns against `app/src/` (sample application) — the sample app should always demonstrate current best practices.

### Moderate Boundary: SUMMARY.md ↔ actual feature set

**Communication:** `docs/src/SUMMARY.md` lists all doc pages. mdBook enforces that listed pages exist as files. It does not enforce that the features they describe are implemented.

**Inconsistency vector:** A doc page was written speculatively. The feature stub exists but is incomplete.

**Detection:** For each feature doc, verify that the primary type or function it documents can be found in `framework/src/lib.rs` or the relevant crate's `lib.rs`.

### Moderate Boundary: ferro-planning ↔ workspace

**Status:** `ferro-planning` is a workspace member but appears to have no source files (`ls ferro-planning/src/` returned nothing). This crate should either have content or be removed from the workspace.

## Anti-Patterns

### Anti-Pattern 1: Fixing Docs Without Fixing the Code

**What people do:** Find a doc page that describes a non-existent feature. Update the doc to describe what exists. Move on.

**Why it's wrong:** If the feature is missing entirely, the correct fix may be to either implement the feature or remove the doc page. A doc page that accurately describes a "stub" feature is still misleading if the stub is incomplete.

**Do this instead:** For each doc page, classify the feature as: (a) fully implemented, (b) partially implemented, (c) not implemented. Fix (a) docs to be accurate. For (b) and (c), either implement the missing piece or remove/mark the doc as a placeholder.

### Anti-Pattern 2: Auditing Only the Happy Path

**What people do:** Read the "Quick Start" section of each feature doc and verify it works. Declare the doc accurate.

**Why it's wrong:** Most inconsistencies live in the advanced sections — error handling, edge cases, configuration options. The `TenantScope` panic behavior, the `Option<Option<T>>` nullable field pattern, the `SavedInertiaContext` footgun — these are the things that bite agents and developers.

**Do this instead:** For each feature doc, audit every code example from top to bottom. Every type name, every method call, every configuration key.

### Anti-Pattern 3: Auditing Each Crate in Isolation

**What people do:** Audit `ferro-cache` independently. Find it accurate. Move on.

**Why it's wrong:** Cross-crate interactions are where philosophy inconsistencies hide. `ferro-cache` may be internally consistent, but if the `application_info` MCP tool doesn't mention caching as a capability, agents won't know to use it.

**Do this instead:** After per-crate audits, do a cross-cutting pass: "Does the MCP layer accurately reflect all capabilities found in the per-crate audits?"

### Anti-Pattern 4: Treating Template Tests as Proof of Correctness

**What people do:** The CLI templates have 60+ tests in `mod.rs`. Tests pass. Declare templates correct.

**Why it's wrong:** Tests check structural presence (`contains`), not correctness. A template that passes `contains("#[handler]")` may still use a deprecated authentication pattern, missing `.await?` propagation, or import a type that no longer exists.

**Do this instead:** Run generated code through `cargo check` in a test project. Or trace each template's imports and patterns against `framework/src/lib.rs` manually.

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Import paths in docs | `ferro_rs` vs `ferro` (old vs new crate name) | `grep -r "ferro_rs" docs/` — fix all hits |
| Optional feature docs (stripe, whatsapp, ai) | Types exposed via feature-gated crates, not `framework/src/lib.rs` | Verify that docs show correct crate name for optional features |
| MCP code templates | `ActiveModel` pattern vs `UpdateBuilder` pattern | Compare to `app/src/controllers/` sample — sample is ground truth |
| Agent-first philosophy | `SavedInertiaContext`, `Option<Option<T>>`, `ValidateRules` naming — footguns undocumented in MCP tools | Check `generation_context.rs` tool for these warnings |
| ferro-projections docs | v9.0 feature; `inspect_projection`, `validate_projection`, `render_projection` MCP tools exist but user-facing docs may be missing | Check `docs/src/SUMMARY.md` — no projections page exists |
| ferro-planning crate | Empty workspace member — no `src/` found | Remove from workspace or add content |
| `docs/src/features/themes.md` | References `--font-family-sans` token but v10.0 KEY DECISION was that Tailwind v4 uses `--font-sans` (not `--font-family-sans`) | Verify token name in `ferro-theme/assets/default.css` against docs |

## Artifact Classification: New vs Modified

For the v11.0 audit milestone:

**Modified artifacts (existing files that will be edited):**
- All `docs/src/**/*.md` files where inaccuracies are found
- `ferro-mcp/src/tools/code_templates.rs` — update code templates to current patterns
- `ferro-mcp/src/tools/generation_context.rs` — add missing footgun warnings
- `ferro-mcp/src/tools/application_info.rs` — update crate count and feature list
- `ferro-cli/src/templates/*.rs` — fix any pattern drift found in templates
- `framework/src/lib.rs` — add missing re-exports found during audit

**New artifacts (files that will be created):**
- `docs/src/features/projections.md` — service projections feature is undocumented
- Possibly `docs/src/features/planning.md` if `ferro-planning` has user-facing functionality

**Removed artifacts:**
- `ferro-planning` from workspace if confirmed empty
- Any doc pages for features confirmed non-existent

## Sources

- Direct codebase inspection: `docs/src/SUMMARY.md`, all feature docs
- `ferro-mcp/src/tools/mod.rs` — 54 tool files confirmed
- `ferro-cli/src/commands/mod.rs` — 50+ command files confirmed
- `ferro-cli/src/templates/mod.rs` — template test suite reviewed
- `.planning/PROJECT.md` — KEY DECISIONS table is authoritative for patterns and tradeoffs
- `.planning/codebase/ARCHITECTURE.md`, `STACK.md`, `CONVENTIONS.md`, `STRUCTURE.md`
- Workspace `Cargo.toml` — 20 workspace members confirmed

---
*Architecture research for: v11.0 Framework Consolidation Audit*
*Researched: 2026-03-26*
