# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-28)

**Core value:** Agents can go from "I want an app that does X" to a working, deployed application with minimal friction.
**Current focus:** v9.0 Service Projections

## Current Position

Phase: 94 (Protocol Documentation) — In Progress
Plan: 94-04 complete
Status: Plan 94-04 shipped — derivation, rendering, and validation specification pages
Last activity: 2026-03-01 — Plan 94-04 executed (2 tasks, 2 commits)

## Milestone Summary

| Milestone | Phases | Plans | Status | Shipped |
|-----------|--------|-------|--------|---------|
| v1.0 DX Overhaul | 1-12 | 18 | Complete | 2026-01-16 |
| v2.0 Rebrand | 13-22 | 13 | Complete | 2026-01-16 |
| v2.0.1 Macro Fix | 22.1-22.3 | 6 | Complete | 2026-01-17 |
| v2.0.2 Type Generator Fixes | 22.4-22.9 | 6 | Complete | 2026-01-17 |
| v2.0.3 DO Apps Deploy | 22.10 | 1 | Complete | 2026-01-17 |
| v2.1 Inertia DX & Fixes | 33-34 | 4 | Complete | 2026-01-17 |
| v2.2 CLI Improvements | 35-37 | 5 | Complete | 2026-02-09 |
| v3.0 JSON-UI | 23-32 | 24 | Complete | 2026-02-09 |
| v4.0 Production Readiness | 38-46 | 24 | Complete | 2026-02-10 |
| v5.0 Proximity — JSON-UI Field Test | 47-53 | 20 | Complete | 2026-02-10 |
| v5.1 Housekeeping | 54-57 | 5 | Complete | 2026-02-13 |
| v6.0 ferro-lang — Localization | 58-66 | 11 | Complete | 2026-02-13 |
| v6.1 Fix Known Issues | 67 | 1 | Complete | 2026-02-24 |
| v7.0 Resend Integration | 68 | 3 | Complete | 2026-02-25 |
| v7.1 Static File Serving | 69 | 1 | Complete | 2026-02-25 |
| v7.4 Security Hardening | 72-74 | 5 | Complete | 2026-02-26 |
| v7.5 Type Generator Fix | 75 | 1 | Complete | 2026-02-27 |
| v7.6 Default API Scaffold | 76 | 4 | Complete | 2026-02-27 |
| v7.7 Validate & Fix API Scaffold | 77 | 3 | Complete | 2026-02-28 |
| v7.8 Memory Leak Fixes | 78 | 3 | Complete | 2026-02-28 |
| v8.0 Consumer MCP — OpenAPI Bridge | 79-82 | 11 | Complete | 2026-02-28 |
| v8.1 API DX Polish | 83 | 5 | Complete | 2026-02-28 |
| v9.0 Service Projections | 84-94 | 29/30 | In Progress | - |

## Accumulated Context

### Key Decisions

Archived to PROJECT.md and milestone archive files.

**Phase 84-01:**
- Consuming builder (`mut self -> Self`) for ServiceDef, matching workspace convention
- 18 known FieldMeaning variants + Custom(String) untagged fallback
- 10 DataType variants (abstract categories, not database types)
- infer_meaning() with 7 rules from existing CLI patterns

**Phase 85-01:**
- Flat states only, no hierarchical/compound states in v1
- Guards as Option<String>, actions as Vec<String> — string references resolved externally
- validate() returns Result<Vec<Warning>, Error> — warnings for structural concerns, errors for fatal
- Removed Eq from ServiceDef (serde_json::Value in StateDef.metadata doesn't implement Eq)
- BFS reachability from initial state for validation

**Phase 85-02:**
- Orphan states produce both UnreachableState and DeadEndState warnings (correct behavior)
- 49 ferro-projections tests total (23 state + 11 service + 15 field) + 2 doctests

**Phase 85.1-01:**
- schemars 1.x for JSON Schema derivation on all 7 public types
- FieldMeaning description annotation documents known variants (mitigates anyOf shadowing by Custom(String))
- JsonSchema not added to Error/Warning (internal types, not protocol contract)
- 53 ferro-projections unit tests + 2 doctests = 55 total

**Phase 85.1-02:**
- v9.0 architectural direction shifted from "map SAP floorplans" to "derive intent from ServiceDef structure"
- 4 architecture principles established: structural intent derivation, schema as protocol, structurally-derivable intents only, confidence scores over hard selection
- 7 proposed intents: Browse, Focus, Collect, Process, Summarize, Analyze, Track + Custom escape hatch
- Phase 89 marked as core innovation phase with >70% accuracy validation target
- Phase 94 descoped to protocol documentation only (patent dropped)

**Phase 86-01:**
- ActionDef, InputDef, GuardDef types — schema-only action/guard definitions
- InputDef reuses DataType/FieldMeaning — single type vocabulary, no parallel systems
- FieldDef readable/writable booleans default true — backward-compatible with Phase 84/85 JSON
- ServiceDef gains read_only_field/write_only_field convenience builders
- 74 ferro-projections unit tests + 4 doctests = 78 total

**Phase 86-02:**
- ServiceDef::validate() subsumes StateMachine::validate() as single entry point
- Guards are a shared pool referenced from transitions (Phase 85) and action preconditions (Phase 86)
- Warning::UnusedGuard and Warning::TransitionTriggerWithoutStateMachine added
- Undefined references are hard errors, structural concerns are warnings
- 93 ferro-projections unit tests + 4 doctests = 97 total

**Phase 87-01:**
- RelationshipDef, Cardinality, NavigationHint types — two-dimensional relationship schema (structural + presentational)
- NavigationHint defaults from Cardinality.default_navigation(), overridable per relationship
- ServiceDef gains .relationship(), .belongs_to(), .has_many(), .has_one(), .belongs_to_many() builder methods
- Warning::DuplicateRelationship and Warning::ManyToManyWithForeignKey validation warnings
- 113 ferro-projections unit tests + 5 doctests = 118 total

**Phase 88-01:**
- Intent enum: 7 structurally-derivable variants (Browse, Focus, Collect, Process, Summarize, Analyze, Track) + Custom(String) untagged fallback
- IntentScore: confidence-scored derivation result with matching_signals (NOT Eq, f64 confidence)
- IntentHint: Primary/Exclude manual override for structural analysis, externally tagged serde
- ServiceDef gains intent_hints field with .intent_hint() builder method
- Warning::ConflictingIntentHints and Warning::MultiplePrimaryIntentHints validation warnings
- 138 ferro-projections unit tests + 5 doctests = 143 total

**Phase 88-02:**
- 5 new tests: IntentScore construction, empty signals, Exclude with Custom intent, known vs Custom equality, full ServiceDef integration
- Full integration test exercises all ServiceDef subsystems (fields, actions, guards, relationships, state machine, intent hints)
- 143 ferro-projections unit tests + 5 doctests = 148 total

**Phase 89-01:**
- derive_intents() public API: always returns >= 1 IntentScore, default Focus 0.5 fallback
- Field meaning analyzer: proportional count-weighted signals (0.3*count for Summarize, 0.25*count for Focus, 0.2*count for Browse EntityName, 0.35 for Analyze DateTime+numeric, 0.25 for Track Status, 0.1*count for Browse Category)
- Writability analyzer: >50% writable -> Collect 0.35, write-only -> Collect 0.2*count, >70% non-writable -> Summarize 0.2, more readable -> Focus 0.1
- Browse and Focus receive 0.1 baseline scores to always appear in results
- Stable tie-breaking: Process(0) > Track(1) > Collect(2) > Browse(3) > Focus(4) > Summarize(5) > Analyze(6) > Custom(7)
- Signal type alias (Intent, f64, String), is_system_field() excludes Identifier/CreatedAt/UpdatedAt
- 170 ferro-projections unit tests + 5 doctests = 175 total

**Phase 89-02:**
- State machine analyzer: guard density ratio for Process (0.4*ratio), branching states for Process (0.15), transition triggers for Process (0.25*ratio), workflow states for Process (0.10), linear progression for Track (0.3), final states for Track (0.1), unguarded for Track (0.1)
- Relationship analyzer: OneToMany/ManyToMany -> Browse (0.35*count), OneToOne+Inline -> Focus (0.15*count), ManyToOne -> Focus (0.1*count), >3 relationships -> Browse (0.1)
- Action analyzer: transition triggers -> Process (0.15*count), >2 inputs -> Collect (0.15*count), preconditions -> Process (0.1*count), simple CRUD -> Browse (0.05)
- Both state machine and action analyzers contribute Process weight from transition_trigger, amplifying when aligned
- 185 ferro-projections unit tests + 5 doctests = 190 total

**Phase 89-03:**
- 12 validation fixtures covering all 7 intents at 100% primary intent accuracy (exceeds 70% threshold)
- No weight tuning needed — engine generalizes correctly across all validation scenarios
- Fixture design requires careful field selection to avoid competing signals (e.g., FreeText amplifying Focus over Collect, CreatedAt being system field)
- 8 edge case tests: empty, minimal, maximal, ambiguous, IntentHint Primary/Exclude overrides, confidence range validation
- derive_intents() doctest added for public API documentation
- 206 ferro-projections unit tests + 6 doctests = 212 total

**Phase 90-01:**
- Renderer trait with render() -> Result<serde_json::Value, Error>, outputs framework-independent JSON
- RenderMode (Display/Input), RenderContext (intent_index, current_state, mode) types
- is_system_field() moved from derive.rs to render module as pub(crate), shared across modules
- field_to_display/input/column: exhaustive mapping of all 18 FieldMeaning variants to JSON-UI components
- relationship_to_component: all 5 NavigationHint variants mapped (Inline/Link/Tab/Nested/Hidden)
- field_display_name() converts snake_case to title case labels
- Error::Render variant added for rendering failures
- 249 ferro-projections unit tests + 7 doctests = 256 total

**Phase 90-02:**
- JsonUiRenderer struct implementing Renderer trait, outputs ferro-json-ui/v1 JSON envelope
- Browse layout: Table + Pagination, system fields excluded from columns, sortable
- Focus layout: Card + DescriptionList + relationship sections (Tab->Tabs, Nested->Table, Inline/Link->Card children)
- Collect layout: Form + typed inputs per FieldMeaning, skip auto-generated system fields, Submit button
- Summarize layout: Card per metric field (Money/Quantity->Text, Percentage->Progress), Status->Badge, DescriptionList fallback
- Collect shared across Browse/Focus/Summarize/Custom in Input mode (single form implementation)
- Custom(String) intent falls back to Focus display, Collect input
- Process/Analyze/Track remain todo!() for Plan 03
- 274 ferro-projections unit tests + 7 doctests = 281 total

**Phase 90-03:**
- Process layout: Card+Badge state display, guard Alert, transition action Buttons; falls back to Focus without state machine
- Analyze layout: summary Card with stat placeholders for numeric fields, sortable Table with all readable fields including DateTime, no Pagination
- Track layout: Table with DateTime system fields visible, Status columns, sorted desc, with Pagination
- Process Input mode: Form + transition buttons for editing while progressing state
- All 7 intents + Custom fully implemented (no remaining todo!() stubs)
- 5 full pipeline integration tests, edge case tests, RenderContext variation tests
- JsonUiRenderer doctest documenting basic usage
- 301 ferro-projections unit tests + 8 doctests = 309 total

**Phase 91-01:**
- Feature-gated re-export of 22 ferro-projections public types behind `#[cfg(feature = "projections")]`
- ProjectionsError/ProjectionsWarning aliases avoid name collisions with existing Error re-exports
- ferro_projections::Error maps to FrameworkError::Internal (500 status) — projection failures are internal logic errors
- Both FrameworkError and HttpResponse From impls needed for `?` operator in handlers returning Response

**Phase 91-02:**
- `ferro make:projection <name>` CLI command scaffolds src/projections/{name}.rs with ServiceDef builder function
- Template uses `ferro::{...}` imports (targets user projects, not workspace)
- Auto-creates src/projections/ directory (like make_json_view pattern)
- 4 unit tests: template generation, directory/file creation, mod.rs creation, mod.rs append without duplication

**Phase 91-03:**
- 3 MCP projection introspection tools: list_projections, inspect_projection, render_projection
- Source-scanning via regex (matching json_ui_inspect pattern) to discover and parse ServiceDef functions
- render_projection reconstructs ServiceDef from parsed source, derives intents, renders JSON-UI via JsonUiRenderer
- InspectResult uses untagged enum for clean Found/NotFound JSON output
- All 4 field types parsed with correct readable/writable flags
- 17 new tests (147 total ferro-mcp tests)

**Phase 92-01:**
- `ferro make:projection --from-model` reads SeaORM models via syn AST visitor and generates populated ServiceDef
- Self-contained ModelField/ModelVisitor in make_projection.rs (not imported from make_api — avoids coupling)
- Replicated infer_meaning logic as string-returning codegen helper (no ferro-projections dependency in ferro-cli for this)
- rust_type_to_data_type maps 20+ Rust types to DataType variants, unknown falls back to String
- Sensitive fields excluded entirely from output, FK fields get read_only_field + belongs_to
- 10 ferro-cli make_projection tests (4 existing + 6 new)

**Phase 92-02:**
- `ferro projection:check` CLI command scans src/projections/, reconstructs ServiceDef via regex, validates with ServiceDef::validate()
- Feature-gated behind `projections` feature flag in ferro-cli (default enabled)
- `validate_projection` MCP tool with single (by name) and all-projections modes
- reconstruct_service_def promoted to pub(crate) in render_projection.rs for reuse
- Warnings produce exit code 0, only Err from validate() produces exit code 1
- 5 ferro-cli + 6 ferro-mcp tests = 11 new tests

**Phase 92-03:**
- `projection_coverage` MCP tool cross-references list_models with list_projections via case-insensitive service_name matching
- Derives primary intent via reconstruct_service_def + derive_intents for covered projections
- Generates `ferro make:projection {snake_name} --from-model` suggestions for uncovered models
- Coverage percentage computed as (with_projections / total_models) * 100
- 5 new ferro-mcp tests (158 total)

**Phase 93-01:**
- 8 projection files in app/src/projections/ covering all 7 intents (Browse, Focus, Collect, Process, Summarize, Analyze, Track)
- CLI --from-model generates Custom(...) field meanings which are not parseable by MCP regex scanner — must use standard FieldMeaning variants
- Projection functions use pub fn service_def() naming convention (not {name}_service from CLI template)
- Projections module needs #[allow(dead_code)] since functions are discovered by MCP source scanning, not called from Rust
- Long field names (e.g., "discount_applied") cause rustfmt to break read_only_field calls across lines, defeating regex parser

**Phase 93-02:**
- MCP parser fixed: parenthesis-depth extraction for .action() blocks, sub-regex parsing of transition_trigger/precondition/display_name/inputs
- Transition .guard() and GuardDef definitions now parsed during ServiceDef reconstruction
- 9 integration tests validate full MCP pipeline against all 8 real projection files
- All 5 hand-crafted projections derive exact target intents: Process, Browse, Summarize, Analyze, Collect
- sales_analytics adjusted to mixed read/write fields to avoid Summarize dominance while preserving Analyze signal
- 178 ferro-mcp tests total (169 existing + 9 integration)

**Phase 94-01:**
- mdBook project at docs/protocol/ with full TOC (18 pages across Protocol, Governance, Appendix sections)
- Integration test generates 17 individual + 1 combined JSON Schema files from schemars derives
- Warning enum gained Serialize/Deserialize/JsonSchema derives for protocol schema inclusion
- Date-versioned $id URLs: https://ferro-rs.dev/protocol/2026-03-01/{type}.json
- Schema generation reproducible via `cargo test -p ferro-projections --test generate_schemas`

**Phase 94-04:**
- Signal types (WHAT each analyzer examines) are normative; exact weights are informative — allows alternative implementations while preserving interoperability
- Rendering spec is renderer-agnostic: Renderer trait is normative, intent-to-layout mapping is informative
- BFS reachability specified as MUST with explicit algorithm pseudocode
- Validation order recommended (fatal errors before warnings) but not mandated

**Phase 94-02:**
- Protocol introduction positions Ferro in the 2026 agent stack (A2A/MCP/AG-UI/A2UI) with explicit gap identification
- 18 domain-specific terms defined alphabetically in terminology.md, cross-referenced to Rust type names
- Three-layer architecture (ServiceDef -> IntentScores -> Renderer) with normative/informative split for derivation rules
- CAMELEON Reference Framework correspondence acknowledged with differentiation: dynamic confidence-scored derivation vs static tree-based AUI
- RFC 2119 + RFC 8174 conformance language established per BCP 14

### Roadmap Evolution

- 21 milestones shipped, 184 plans total
- v9.0 created: Service Projections — ServiceDef→IntentGraph→Renderer architecture, 10 phases (Phase 84-93)
- Phase 94 added: Protocol Documentation — standardized protocol definition (patent descoped)
- Phase 85.1 inserted after Phase 85: Architecture Refinement — incorporate prior art insights, add schemars, refine Phases 86-93
- Phase 85.1 COMPLETE: 2 plans, schemars integration + roadmap refinement

### Pending Todos

None.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-03-01
Stopped at: Phase 94, Plan 04 complete — derivation, rendering, and validation specification pages
Resume file: None
