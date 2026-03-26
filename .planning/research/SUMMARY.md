# Project Research Summary

**Project:** v11.0 Framework Consolidation Audit
**Domain:** Documentation accuracy, completeness, and philosophy consistency audit — 14-crate Rust workspace
**Researched:** 2026-03-26
**Confidence:** HIGH

## Executive Summary

The v11.0 milestone is not a feature release — it is a pre-publication audit-then-fix milestone for a ~90,000-line Rust workspace that evolved through 10 rapid development milestones. The framework has outrun its documentation: features shipped in v8.1, v9.0, and v10.0 have no user-facing docs, existing docs contain stale import paths and TODO stubs presented as working code, and the public introduction positions Ferro as "the Laravel of Rust" while burying its actual value proposition — agent-first AI code generation — entirely. Before crates.io publication, all four artifact layers (mdBook prose, public API surface, MCP tool descriptions, and CLI scaffold templates) must be brought into accurate, consistent, and agent-ready alignment.

The recommended approach follows a strict layered sequence: audit and fix the public API surface first, then MCP tools, then CLI templates, and finally prose documentation. This order is not arbitrary — MCP tools are the primary interface agents encounter before docs, and CLI templates produce code that immediately reveals inaccuracies. Fixing prose before fixing the code it describes propagates errors with better grammar. The audit must separate read-only audit phases (produce a severity-ranked issue list) from fix phases (address specific items on that list) to prevent scope creep from making the milestone unshippable.

The key risks are: (1) phantom feature documentation — stub functions presented as working code, specifically `ferro-stripe`'s `stripe_is_processed()` which always returns `false`; (2) import path inconsistency — `ferro_rs::` vs `ferro::` scattered across multi-tenancy and JSON-UI docs with 28 confirmed occurrences; (3) a complete documentation gap for Service Projections (v9.0), which has its own crate, 315 tests, 5 MCP tools, and a protocol spec, but no user-facing how-to page. These three issues must be treated as P0 blockers alongside the README roadmap inaccuracy (JSON-UI listed as "Work in Progress" despite shipping in v10.0).

---

## Key Findings

### Recommended Stack

All tooling for this milestone is Rust-native and stable-toolchain-compatible (MSRV 1.88.0). The existing CI already enforces `RUSTDOCFLAGS: -Dwarnings` with `cargo doc --no-deps --all-features`, catching broken intra-doc links automatically. The audit requires three additional tools: `mdbook-linkcheck` (broken link detection in the 40-page mdBook), `cargo-rdme` (sync per-crate READMEs from crate-level `//!` docs), and `cargo-semver-checks` (catch accidental API breaks during cleanup). All three are stable-toolchain-compatible — do not use nightly-only alternatives like `--show-coverage`, `cargo-sync-rdme`, or `rust-semverver`.

**Core technologies:**
- `rustdoc` + `#![warn(missing_docs)]`: API doc coverage enforcement — stable, authoritative, integrates with existing CI `-D warnings` pipeline; add per-crate incrementally starting with `framework`
- `mdbook-linkcheck 0.7.7`: Detects broken internal/external links in `docs/src/` — add to `docs/book.toml`; catches dead cross-references introduced by page renames
- `cargo-rdme 1.4.8`: Syncs per-crate `README.md` from `//!` crate-level doc comments — use after fixing crate docs to propagate to crates.io README display; works on stable (unlike `cargo-sync-rdme` which requires nightly)
- `cargo-semver-checks 0.44.0+`: 245 lints; catches API consistency violations during doc-driven cleanup — prevents accidental breaking changes appearing as doc-only PRs

**What to avoid:**
- `RUSTDOCFLAGS='--show-coverage'` — nightly-only, unstable, known regression behavior (tracking issue #58154)
- `#![deny(missing_docs)]` workspace-wide in one pass — causes hundreds of failures; use `warn`, escalate to `deny` per crate only after completion
- Nightly toolchain for any audit tooling — workspace is pinned to stable 1.88.0

### Expected Features (Audit Check Categories)

This milestone's "features" are audit check categories, not product capabilities. They map to a priority-ordered execution plan.

**Must have — P0 (Actively wrong, fix before anything else):**
- Fix `ferro_rs::` import paths — 28 confirmed occurrences across multi-tenancy.md, json-ui/actions.md, json-ui/data-binding.md; canonical alias is `ferro::` as set by `ferro new`
- Remove `// TODO: Implement` stubs from CLI reference examples — 8+ occurrences in reference/cli.md presented as working code
- Fix README roadmap section — JSON-UI listed as "Work in Progress" is false; it shipped in v10.0
- Fix storage docs — S3 labeled "coming soon" is stale; S3 is shipped
- Correct MCP tool count claims — docs say "35+ tools", actual count is 57

**Should have — P1 (Missing critical coverage, must have for publication):**
- Document 13 undocumented CLI commands: `api:check`, `clean`, `generate-routes`, `make:api`, `make:api-key`, `make:lang`, `make:policy`, `make:projection`, `make:stripe`, `make:theme`, `make:whatsapp`, `projection:check`, `validate-contracts`
- Create Service Projections user documentation page — v9.0 feature with 315 tests and 5 MCP tools but zero how-to docs in `docs/src/`
- Rewrite introduction.md to lead with agent-first value proposition — currently describes "the Laravel of Rust" with zero mention of agents, MCP, or AI generation
- Add "Working with Agents" guide documenting the MCP workflow (`application_info` → `list_routes` → `get_handler`)
- Audit and refresh generation_hints across all 57 MCP tool responses — drift is likely after v9.0 added significant new tool surface
- Document FerroModel and ValidateRules derive macros in user docs

**Should have — P2 (Quality, should have before publication):**
- Standardize import style across all code examples — mixed `use ferro::*` vs explicit multi-import patterns cause agent generation failures
- Resolve COMPONENT_CATALOG duplication between `ferro-cli/src/ai.rs` and `ferro-mcp/src/tools/json_ui_generate.rs` — known P2 concern since v5.0, requires a design decision (shared data file, build script, or shared crate; direct dependency creates a cycle)
- Fix Cargo.toml metadata gaps: `ferro-broadcast` (missing readme), `ferro-theme` (missing categories), `ferro-projections` (missing homepage and readme)
- Add `#![warn(missing_docs)]` to `framework` crate; expand stub READMEs for ferro-json-ui (9 lines), ferro-lang (9 lines), ferro-whatsapp (3 lines)

**Defer:**
- Implementing features discovered incomplete during the audit — file as future milestone items; do not act in v11.0
- Auditing `app/` sample application generated output — audit scaffold templates in `ferro-cli/src/templates/`, not generated output

### Architecture Approach

The audit operates across four interconnected artifact layers. Inconsistencies cluster at the boundaries between them because only Rust source is compiler-checked; Markdown, MCP string literals, and CLI templates drift silently.

```
docs/src/ (40 .md files)          — highest drift risk; prose not compiled
framework/src/lib.rs               — single re-export facade; source of truth for all type names
ferro-mcp/src/tools/ (54 .rs)     — agent-first delivery; tool descriptions age independently of API
ferro-cli/src/templates/ (50+)    — generated code; tests check structural presence, not compilation
```

**Major components and audit responsibilities:**
1. `framework/src/lib.rs` — canonical export surface; every type named in any doc must be verified against this file; if a type is in docs but not here, that is either a doc error or a missing export
2. `docs/src/**/*.md` (40 files) — user-readable contract; highest volume, most drift; audit order: P0 accuracy fixes before prose quality
3. `ferro-mcp/src/tools/*.rs` (54 tools) — agent-readable truth; `code_templates.rs`, `generation_context.rs`, and `application_info.rs` are highest-impact, highest-drift; fix before prose
4. `ferro-cli/src/templates/*.rs` — generated code quality; template tests use `contains()` not compilation; must trace patterns against `framework/src/lib.rs` manually
5. `ferro-cli/src/commands/*.rs` — CLI help text; 13 commands have no reference documentation in `reference/cli.md`

### Critical Pitfalls

1. **Phantom feature documentation** — Stub functions (`todo!()`, `// TODO` in public methods) presented as working examples. Confirmed: `ferro-stripe/src/webhook/mod.rs` line 40 has `// TODO: implement by checking a processed-events DB table`; the stripe.md doc presents `stripe_is_processed()` as callable. Prevention: classify each feature as fully/partially/not implemented before fixing its doc; stubs get an "NOT YET IMPLEMENTED" callout at the top, not in a footnote.

2. **Wrong import path in examples** — `ferro_rs::` vs `ferro::`. The canonical alias is `ferro::` as set in `app/Cargo.toml`: `ferro = { path = "../framework", package = "ferro-rs" }`. Prevention: fix in a single atomic grep-replace pass as the first action; add a CI check that fails on `ferro_rs::` in `.md` files to prevent regression.

3. **Fix phase propagating inaccuracies** — Auditors "fix" docs by improving prose clarity rather than verifying against Rust source. Prevention: require that every changed API example in a PR names the Rust source file verified (`framework/src/lib.rs` line X confirmed); no doc example ships without source verification.

4. **Audit scope creep** — Each fix uncovers new gaps; the milestone expands until nothing ships. Prevention: separate read-only audit phases (produce a severity-ranked list) from fix phases (address only items on that list); newly discovered issues go to a backlog file, not the current PR; "while I was in there" is a scope creep warning sign.

5. **Service Projections documentation gap** — The v9.0 crate has 315 tests, 5 MCP tools, and a protocol spec but no entry in `docs/src/SUMMARY.md` and no how-to page. Prevention: create `docs/src/features/projections.md` as a new artifact, pattern it on `events.md` structure; the protocol spec in `docs/protocol/` is a technical standard, not a user guide.

---

## Implications for Roadmap

The audit follows a dependency-respecting sequence: fix foundational layers before surface layers. Philosophy coherence can only be evaluated holistically after each individual layer is accurate.

### Phase 1: P0 Accuracy Fixes

**Rationale:** Actively wrong information (stale imports, TODO stubs, README lies about shipped features) contaminates all downstream work. Auditing coherence on wrong examples is wasted effort.
**Delivers:** All five P0 issues resolved; docs no longer actively mislead users or agents
**Addresses:** Import path normalization (28 occurrences), README roadmap accuracy, storage S3 "coming soon" stale note, MCP tool count correction, CLI reference stub removal
**Avoids:** The pitfall of fixing clarity while leaving factual errors; the wrong import pitfall (single atomic pass eliminates mixed states)

### Phase 2: CLI and MCP Accuracy

**Rationale:** Agents encounter MCP tools before prose. CLI templates produce code that immediately reveals inaccuracies. Fix code-based artifacts before prose — a wrong template breaks a build; a wrong prose sentence just confuses.
**Delivers:** All 57 MCP tools verified and generation_hints refreshed; 13 undocumented CLI commands added to `reference/cli.md`; `code_templates.rs` patterns traced against current framework exports
**Addresses:** MCP tool drift, CLI reference completeness (diff `ferro-cli/src/commands/` vs `reference/cli.md`), `code_templates.rs` accuracy
**Avoids:** Fixing prose before code-layer artifacts; the anti-pattern of treating template `contains()` tests as proof of correctness

### Phase 3: Documentation Completeness

**Rationale:** With accurate foundations, fill coverage gaps for shipped features that lack user-facing docs.
**Delivers:** `docs/src/features/projections.md` (new page); FerroModel and ValidateRules derive macro docs; `make:model` command documentation
**Addresses:** v9.0 Service Projections documentation gap, macro discoverability, agent discovery of scaffolding commands
**Avoids:** Creating stub pages — only document features confirmed fully implemented; no placeholder content

### Phase 4: Agent-First Philosophy

**Rationale:** Philosophy coherence is an overlay on all layers; it can only be evaluated after each layer is accurate. Rewriting introduction.md before fixing the docs it introduces would be premature.
**Delivers:** Rewritten `introduction.md` leading with agent-first thesis; "Working with Agents" guide; MCP tool references added to each feature doc section (one line listing relevant tools e.g. `list_events`, `inspect_projection`)
**Addresses:** Introduction philosophy drift, agent discoverability of capabilities, generation_hint consistency across all 57 tools
**Avoids:** Treating philosophy work as a full rewrite — it is an overlay on individually-accurate docs, not a replacement

### Phase 5: Metadata and API Guidelines

**Rationale:** Pre-publication housekeeping that does not affect user-facing accuracy. Do last to avoid blocking critical fixes.
**Delivers:** Cargo.toml metadata gaps fixed (ferro-broadcast, ferro-theme, ferro-projections); `#![warn(missing_docs)]` added to `framework` crate; stub READMEs expanded; COMPONENT_CATALOG duplication resolved or design decision documented
**Addresses:** crates.io publication readiness, Rust API Guidelines compliance, long-term doc quality enforcement
**Avoids:** Adding `#![deny(missing_docs)]` workspace-wide in one pass; the Tokio pattern (add `#![allow(missing_docs)]` first, then `warn` module by module) is the correct incremental approach

### Phase Ordering Rationale

- P0 fixes first because contaminated examples make all subsequent quality judgments unreliable
- MCP and CLI before prose because agents read MCP output before docs; broken templates surface immediately at build time; prose errors require a developer to attempt the example before the error is visible
- Completeness before philosophy because agent-first coherence cannot be evaluated for a feature that has no docs yet
- Metadata last because it is independent of user-facing accuracy and has no blockers on the critical path

### Research Flags

Phases needing deeper research during planning:
- **Phase 2 (CLI/MCP Accuracy):** `code_templates.rs` pattern verification must trace each code snippet against current `framework/src/lib.rs` exports — manual, crate-by-crate work with no automated shortcut; estimate effort before committing to scope; the `UpdateBuilder` vs `ActiveModel` pattern drift specifically needs grounding
- **Phase 5 (COMPONENT_CATALOG):** Resolving duplication between `ferro-cli/src/ai.rs` and `ferro-mcp/src/tools/json_ui_generate.rs` requires a design decision with architectural implications; options need explicit evaluation before the phase begins

Phases with standard patterns (research-phase optional):
- **Phase 1 (P0 Accuracy):** All fixes are grep-replace or straightforward prose corrections; no design decisions required
- **Phase 3 (Completeness):** Creating `projections.md` is pattern-matched from `events.md`; standard structure applies
- **Phase 4 (Philosophy):** `introduction.md` rewrite is pure prose with no API dependencies; philosophy pillars are clear from PROJECT.md

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All tooling verified against stable Rust 1.88.0 MSRV; stability status confirmed for each tool; nightly-only alternatives explicitly identified and excluded with tracking issue references |
| Features | HIGH | All audit check categories derived from direct codebase inspection of all 14 crates, 40 doc files, 54 MCP tool files, and 50+ CLI command files; specific file locations and line numbers cited for confirmed issues |
| Architecture | HIGH | All findings from direct codebase inspection; single re-export facade pattern confirmed in `framework/src/lib.rs`; 54 tool files confirmed in `ferro-mcp/src/tools/mod.rs`; data flow traced through actual call chains |
| Pitfalls | HIGH | All pitfalls confirmed with specific instances: line numbers in multi-tenancy.md (8 occurrences), specific function in ferro-stripe/src/webhook/mod.rs (line 40), diff of command files (50) vs reference entries (37), SUMMARY.md confirmed to have no projections entry |

**Overall confidence:** HIGH

### Gaps to Address

- **ferro-planning crate status:** The workspace member `ferro-planning` appears to have no source files. Confirm during Phase 5 — either add content or remove from workspace; an empty crate should not publish to crates.io.
- **COMPONENT_CATALOG resolution strategy:** No recommended approach exists yet; the three options (shared data file, build script, new shared crate) each have trade-offs that need evaluation before Phase 5 can scope this item.
- **Theme token name in docs:** `docs/src/features/themes.md` may reference `--font-family-sans` while the v10.0 KEY DECISION established `--font-sans` (Tailwind v4 namespace). Needs verification against `ferro-theme/assets/default.css` before the themes.md audit is considered complete.
- **ferro-stripe stub scope:** `stripe_is_processed()` is confirmed as a stub. The full scope of incomplete stripe functionality needs a single audit pass to determine whether the feature is partially or fully stubbed beyond the webhook idempotency handler.

---

## Sources

### Primary (HIGH confidence)
- Direct codebase inspection: `docs/src/` (40 files), `ferro-mcp/src/tools/mod.rs`, `ferro-cli/src/commands/`, all 14 `Cargo.toml` files, `ferro-stripe/src/webhook/mod.rs` (line 40), `app/Cargo.toml` (canonical alias), `docs/src/SUMMARY.md` (confirmed no projections entry)
- [Rustdoc lints — official reference](https://doc.rust-lang.org/rustdoc/lints.html) — lint names and stability status
- [Rust API Guidelines — documentation section](https://rust-lang.github.io/api-guidelines/documentation.html) — C-CRATE-DOC through C-HIDDEN checklist
- [RFC 1574 — API documentation conventions](https://rust-lang.github.io/rfcs/1574-more-api-documentation-conventions.md) — Panics/Errors/Safety section conventions
- [cargo-semver-checks 0.44.0](https://crates.io/crates/cargo-semver-checks) — 245 lints, stable toolchain confirmed
- [cargo-rdme 1.4.8](https://docs.rs/crate/cargo-rdme/latest) — stable README sync tool
- `.github/workflows/ci.yml` — confirmed existing `RUSTDOCFLAGS: -Dwarnings` and `cargo doc` CI job
- `.planning/codebase/CONCERNS.md` — confirmed COMPONENT_CATALOG drift as P2 concern since v5.0
- `.planning/v9.0-MILESTONE-AUDIT.md` — confirmed v9.0 predates formal verification workflow; all 12 phases missing VALIDATION.md

### Secondary (MEDIUM confidence)
- [mdbook-linkcheck 0.7.7](https://github.com/Michael-F-Bryan/mdbook-linkcheck) — last release 2022, still functional; used by the Rust project itself
- [Axum lib.rs source](https://github.com/tokio-rs/axum/blob/main/axum/src/lib.rs) — documentation style and selective `missing_docs` enforcement patterns
- [LLM-Friendly API Design patterns](https://agentic-patterns.com/patterns/llm-friendly-api-design/) — agent-first design principles for self-descriptive naming and actionable errors

### Tertiary (LOW confidence)
- [rustdoc --show-coverage tracking issue #58154](https://github.com/rust-lang/rust/issues/58154) — cited to justify exclusion of `--show-coverage` from the toolchain, not for adoption

---
*Research completed: 2026-03-26*
*Ready for roadmap: yes*
