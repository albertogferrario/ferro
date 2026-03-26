# Feature Research: Framework Documentation & Philosophy Audit

**Domain:** Multi-crate Rust web framework — documentation accuracy, completeness, and philosophy consistency audit
**Researched:** 2026-03-26
**Milestone:** v11.0 Framework Consolidation Audit
**Confidence:** HIGH (based on direct codebase inspection of all 14 crates + docs tree)

---

## Context: What the Audit Is

v11.0 is not a feature milestone. It is an audit-then-fix milestone. "Features" here are **audit check categories** — what to inspect, what issues to look for, what to fix. The roadmap phases map to audit categories, not product capabilities.

The framework has ~90,000 lines of Rust across 14 crates, ~9,500 lines of mdBook documentation, 57 MCP tools, and ~50 CLI commands. The goal is to bring all of these into accurate, consistent, agent-ready shape before crates.io publication.

---

## Table Stakes

Audit checks every serious framework performs before a public release. Absence means the framework will lose trust with the first developer who encounters a discrepancy.

| Check Category | Why Required | Complexity | Current Evidence |
|----------------|--------------|------------|-----------------|
| Stale import paths in docs | Documentation showing wrong crate names causes immediate failures when users copy examples. One occurrence poisons trust. | Low | `ferro_rs::` found in `multi-tenancy.md`, `json-ui/actions.md`, `json-ui/data-binding.md` — 28 occurrences across 3 files |
| CLI reference completeness | Every published command needs a doc entry. Agents use the CLI reference to know what scaffolding is available. | Low-Med | 13 CLI commands exist with no reference entry: `api:check`, `clean`, `generate-routes`, `make:api`, `make:api-key`, `make:lang`, `make:policy`, `make:projection`, `make:stripe`, `make:theme`, `make:whatsapp`, `projection:check`, `validate-contracts` |
| "Coming soon" / TODO in user-facing docs | Published docs must not show internal scaffolding. Every TODO in an example is a trust failure for users and breaks agent comprehension. | Low | `reference/cli.md` has 8+ `// TODO: Implement` blocks; `storage.md` references S3 feature as "coming soon" though S3 is shipped |
| README.md accuracy | Root README is the first document indexed by crates.io, search engines, and LLMs. Stale information compounds over time. | Med | README has a "Roadmap" section listing JSON-UI as "Work in Progress" — JSON-UI is shipped and production-ready at v10.0 |
| Agent-first philosophy presence | Ferro's core value proposition is agent-first. The introduction doc does not mention agents, MCP, or the AI-first workflow. This is the primary differentiator and it is absent from the entry point. | Med | `docs/src/introduction.md` describes Ferro as "the Laravel of Rust" with zero mention of agent-first, MCP introspection, or AI generation capability |
| MCP tool count accuracy | Documentation claims "30+ tools" in several places; the actual count is 57. Inaccurate claims undermine credibility. | Low | Project.md says "35+ tools". Actual count in `ferro-mcp/src/tools/mod.rs` is 57 tool modules. |
| Feature docs for all shipped features | Every feature listed in PROJECT.md must have a corresponding doc page. | Low | All major features have pages. However: Service Projections (v9.0) has no user-facing doc page in `docs/src/`. The `docs/src/features/` directory does not contain a projections doc. |
| Rust API Guidelines compliance | Crates intended for crates.io publication should follow Rust API Guidelines: crate-level docs, C-EXAMPLE (examples for all public items), C-FAILURE (error/panic documentation), C-LINK (cross-references). | High | No `#![warn(missing_docs)]` found in any crate. Several crates have minimal `lib.rs` crate-level docs (e.g., `ferro-json-ui`, `ferro-lang`). |
| Cargo.toml metadata completeness | crates.io requires: description, license, repository. Preferred: keywords, categories, readme. All 14 publishable crates need a check. | Low | `ferro-broadcast` missing readme field. `ferro-theme` missing categories field. `ferro-projections` missing homepage and readme fields. Several crates need validation. |

## Differentiators

Audit checks specific to Ferro's agent-first identity. These go beyond standard framework documentation quality.

| Check Category | Value Proposition | Complexity | Notes |
|----------------|-------------------|------------|-------|
| Agent workflow documentation | Ferro's MCP server is a major differentiator. There is no dedicated "working with agents" guide explaining the `application_info` → `list_routes` → `get_handler` workflow that agents use. | Med | The MCP tools exist and work but are not documented as a workflow. The CLAUDE.md has this pattern; user docs do not. |
| MCP tool coverage gap | 57 MCP tools exist. Many tools added in v9.0 (projections) and specialized tools (session_inspect, dependency_graph, request_metrics, route_dependencies, browser_logs, relation_map, model_usages, render_projection, projection_coverage, validate_projection, search_docs) have no user-facing documentation. | Med | These tools are visible to agents via MCP but invisible to human developers reading docs. |
| Error message agent-friendliness audit | One of Ferro's core features is "actionable error messages with fix suggestions." The consistency of this across all crates needs verification — do all error types include hints? | Med | Framework errors have hints. Cross-crate errors (ferro-lang, ferro-storage, ferro-queue) need verification. |
| generation_hint coverage | Ferro embeds generation hints in MCP responses to guide agents. These hints need audit across all tools to ensure they are current, specific, and actionable. | Med | Hints exist in many tools but drift is likely after v9.0 (projections) added significant new tool surface. |
| Pattern coherence for agent code generation | Agents generate code from examples in docs and MCP tool responses. Inconsistent patterns across crates (e.g., import style, handler shape, error propagation) cause agent generation failures. Check: do all code examples use `use ferro::*` or explicit imports? Are handler signatures consistent? | High | Mixed patterns found: some docs use `use ferro::*`, others use explicit multi-import statements. |
| COMPONENT_CATALOG synchronization | The component catalog is duplicated between `ferro-cli/src/ai.rs` and `ferro-mcp/src/tools/json_ui_generate.rs`. The known drift (noted in CONCERNS.md) means AI-generated JSON-UI via CLI and via MCP may produce different schemas. | Med | Documented as P2 concern since v5.0. Still unresolved as of v10.0. |

## Anti-Features

Audit approaches that seem thorough but waste effort or create new problems.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Generating stub documentation for everything | Adding placeholder pages for unimplemented features makes the gap look smaller but creates "empty section" docs that confuse agents and users. | Document what is built; add a "not yet documented" tracking issue for deferred content |
| Rewriting docs from scratch | Complete rewrites break internal cross-references, reset link indexes, and risk introducing new errors while fixing old ones. | Audit existing docs, fix in place, add missing sections |
| Enforcing strict `#![deny(missing_docs)]` workspace-wide in one pass | This will cause hundreds of compiler warnings or errors across 90,000 lines and make the diff unmanageable. | Enable `#![warn(missing_docs)]` incrementally per crate, starting with the most user-facing (`framework`) |
| Auditing generated code in `app/` | The sample app's generated code follows templates. Template auditing belongs in `ferro-cli` templates, not the sample app output. | Audit scaffold templates in `ferro-cli/src/templates/`, not `app/src/` |
| Adding new features during audit | Scope creep. Any "we should also add X" finding during audit should be filed as a future milestone, not acted on in v11.0. | Track as PROJECT.md items under "Active" with [future] tag |

---

## Feature Dependencies

```
[Stale import path fix] → all other doc fixes build on clean examples
    └── fixes ferro_rs:: → ferro:: in multi-tenancy.md, json-ui/actions.md, json-ui/data-binding.md

[README accuracy fix]
    └── requires: knowing what is actually shipped (PROJECT.md as source of truth)

[CLI reference completeness]
    └── requires: knowing all shipped commands (ferro-cli/src/commands/ as source of truth)

[MCP tool documentation]
    └── requires: knowing all shipped tools (ferro-mcp/src/tools/mod.rs as source of truth)

[Agent workflow guide]
    └── enhances: MCP tool documentation (needs tool docs to reference)

[Cargo.toml metadata] ──independent──> can be fixed in any order

[generation_hint audit] ──depends on──> understanding each tool's purpose

[Pattern coherence audit] ──depends on──> stale import fix (can't assess coherence on wrong code)

[COMPONENT_CATALOG sync] ──requires──> resolving to a single source of truth (ferro-json-ui)
    └── conflicts with: workspace binary crate isolation (cannot directly share between ferro-cli and ferro-mcp)
```

### Dependency Notes

- **Stale imports must be fixed first:** All other documentation checks that reference code examples are contaminated by the `ferro_rs` naming issue. Fix this before assessing pattern coherence.
- **README roadmap section blocks credibility:** The "JSON-UI (Work in Progress)" roadmap entry is immediately visible to anyone landing on the repo. This is the highest-visibility single-file fix.
- **COMPONENT_CATALOG sync requires a design decision:** Options are (1) extract to a shared data file loaded by both crates, (2) generate from a build script, or (3) make `ferro-mcp` depend on `ferro-cli` (creates a cycle — not viable). This is not a simple fix; it needs a phase of its own.

---

## Audit Scope: All 14 Crates

| Crate | Doc Page | Cargo.toml | Crate lib.rs Docs | Key Concerns |
|-------|----------|------------|-------------------|--------------|
| `framework` (ferro-rs) | introduction.md, the-basics/ | Complete | Partial | Agent-first philosophy absent from intro |
| `ferro-cli` | reference/cli.md | Complete | — (binary) | 13 commands undocumented |
| `ferro-mcp` | reference/cli.md partial | Complete | — (binary) | 40+ tools undocumented to humans |
| `ferro-macros` | inertia.md (InertiaProps only) | Complete | Minimal | FerroModel, ValidateRules not in user docs |
| `ferro-inertia` | features/inertia.md (good) | Complete | Good README | Well documented |
| `ferro-json-ui` | json-ui/* (good) | Complete | Stub README (9 lines) | README is near-empty |
| `ferro-events` | features/events.md | Complete | Has README | Good |
| `ferro-queue` | features/queues.md | Complete | Has README | Good |
| `ferro-notifications` | features/notifications.md | Complete | Has README | Good |
| `ferro-broadcast` | features/broadcasting.md | Missing readme in Cargo.toml | Has README | Missing Cargo.toml readme field |
| `ferro-storage` | features/storage.md | Complete | Has README | "coming soon" S3 note is stale |
| `ferro-cache` | features/caching.md | Complete | Has README | Good |
| `ferro-lang` | features/localization.md | Complete | Stub README (9 lines) | README near-empty |
| `ferro-projections` | No user doc page | Missing homepage, readme in Cargo.toml | — | Major gap: v9.0 feature with zero user docs |
| `ferro-ai` | features/ai.md | Complete | — | Good |
| `ferro-api-mcp` | features/api-mcp.md | — | — | Check binary-only crate metadata |
| `ferro-theme` | features/themes.md | Missing categories | — | Minor metadata gap |

---

## MVP Audit Definition

What must be done for v11.0 to be considered complete.

### Phase 1: Accuracy Fixes (P0)

Critical correctness issues that actively mislead users and agents.

- [ ] Fix all `ferro_rs::` → `ferro::` in user docs (28 occurrences, 3 files)
- [ ] Remove `// TODO: Implement` blocks from CLI reference examples (8+ occurrences)
- [ ] Fix README roadmap section — JSON-UI is shipped, not "Work in Progress"
- [ ] Fix S3 "coming soon" note in storage docs — S3 is shipped
- [ ] Correct MCP tool count claims to reflect actual 57 tools

### Phase 2: Completeness Fixes (P1)

Missing coverage that blocks users from using shipped features.

- [ ] Document 13 undocumented CLI commands in reference/cli.md
- [ ] Add Service Projections user documentation page (v9.0 feature)
- [ ] Document FerroModel and ValidateRules derive macros in user docs
- [ ] Document `make:model` command (absent from CLI reference entirely)

### Phase 3: Agent-First Philosophy (P1)

Ferro's identity needs to be present where users first encounter it.

- [ ] Rewrite introduction.md to lead with agent-first value proposition
- [ ] Add "Working with Agents" guide documenting the MCP workflow
- [ ] Audit and refresh generation_hints in all 57 MCP tool responses
- [ ] Document the agent-to-CLI workflow (agent calls MCP → reads hints → uses CLI)

### Phase 4: Pattern Coherence (P2)

Consistency issues that cause agent generation failures.

- [ ] Standardize import style across all code examples (audit `use ferro::*` vs explicit imports)
- [ ] Audit handler macro patterns — all examples should use `#[handler]` not raw `async fn`
- [ ] Verify error propagation examples use `?` not `unwrap()` or `try!`
- [ ] Resolve COMPONENT_CATALOG duplication (design decision required)

### Phase 5: Metadata & Rust Guidelines (P2)

Pre-publication housekeeping.

- [ ] Fix Cargo.toml metadata gaps (ferro-broadcast, ferro-theme, ferro-projections)
- [ ] Add `#![warn(missing_docs)]` to framework crate
- [ ] Expand stub READMEs: ferro-json-ui (9 lines), ferro-lang (9 lines), ferro-whatsapp (3 lines)
- [ ] Add crate-level `//!` doc comments with examples to ferro-json-ui, ferro-lang lib.rs

---

## Feature Prioritization Matrix

| Audit Check | User Value | Fix Cost | Priority |
|-------------|------------|----------|----------|
| Fix ferro_rs imports in docs | HIGH — breaks copy-paste examples | LOW | P0 |
| Remove TODO from CLI reference | HIGH — confusing to users and agents | LOW | P0 |
| README roadmap accuracy | HIGH — first impression | LOW | P0 |
| Storage S3 "coming soon" | MEDIUM — misleads about shipped feature | LOW | P0 |
| CLI reference completeness (13 commands) | HIGH — agents rely on this | MEDIUM | P1 |
| Service Projections docs | MEDIUM — v9.0 feature undocumented | MEDIUM | P1 |
| FerroModel/ValidateRules docs | MEDIUM — derive macros are core DX | LOW | P1 |
| Introduction agent-first rewrite | HIGH — primary value prop missing | MEDIUM | P1 |
| Agent workflow guide | HIGH — core differentiator | MEDIUM | P1 |
| MCP tool hint audit | HIGH — directly affects agent quality | HIGH | P1 |
| Import pattern coherence | HIGH — agent generation correctness | MEDIUM | P2 |
| COMPONENT_CATALOG sync | MEDIUM — drift causes inconsistent AI output | MEDIUM | P2 |
| Cargo.toml metadata fixes | LOW — crates.io quality | LOW | P2 |
| missing_docs lint enablement | MEDIUM — long-term quality | HIGH | P2 |
| Stub README expansions | LOW — crates.io discoverability | MEDIUM | P2 |

**Priority key:**
- P0: Actively wrong — fix before anything else
- P1: Missing critical coverage — must have for publication
- P2: Quality improvements — should have before publication

---

## Audit Scope Boundaries

### In Scope

- `docs/src/` — all mdBook user documentation
- `docs/src/reference/cli.md` — CLI reference completeness
- `*/Cargo.toml` — metadata for publishable crates
- `ferro-mcp/src/tools/*.rs` — generation_hint strings in tool responses
- `ferro-cli/src/templates/` — scaffold template accuracy
- Root `README.md` — accuracy and philosophy alignment
- `*/src/lib.rs` — crate-level doc comments for 14 crates

### Out of Scope

- `app/` sample application code (test bed only)
- `docs/protocol/` service projection protocol spec (separate audience)
- Frontend TypeScript types and React components
- Test file documentation
- Changelog or release notes format

---

## Sources

- Direct codebase inspection: `docs/src/` (all mdBook pages), `ferro-mcp/src/tools/mod.rs`, `ferro-cli/src/commands/`, all 14 `Cargo.toml` files
- [Rust API Guidelines — Documentation](https://rust-lang.github.io/api-guidelines/documentation.html) — C-CRATE-DOC, C-EXAMPLE, C-FAILURE, C-LINK, C-METADATA requirements (HIGH confidence)
- [Rust API Guidelines — Checklist](https://rust-lang.github.io/api-guidelines/checklist.html) — 11 category audit framework (HIGH confidence)
- [LLM-Friendly API Design — Awesome Agentic Patterns](https://agentic-patterns.com/patterns/llm-friendly-api-design/) — Agent-first design principles: self-descriptive naming, shallow indirection, actionable errors, explicit schemas (MEDIUM confidence)
- [mdbook-linkcheck](https://docs.rs/mdbook-linkcheck) — Broken link checking for mdBook (HIGH confidence — tooling exists)
- PROJECT.md — authoritative list of shipped features vs. documented features (HIGH confidence — primary source)

---

*Feature research for: v11.0 Framework Consolidation Audit*
*Researched: 2026-03-26*
