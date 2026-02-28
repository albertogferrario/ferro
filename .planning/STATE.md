# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-28)

**Core value:** Agents can go from "I want an app that does X" to a working, deployed application with minimal friction.
**Current focus:** v9.0 Service Projections

## Current Position

Phase: 88 (Intent Core Types) — Plan 01 COMPLETE
Plan: 01/01 complete
Status: Phase 88 complete — Intent + IntentScore + IntentHint + ServiceDef integration + validation + 143 tests
Last activity: 2026-02-28 — Plan 01 complete (2 tasks, 2 commits)

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
| v9.0 Service Projections | 84-94 | 10/28 | In Progress | - |

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

Last session: 2026-02-28
Stopped at: Phase 88 Plan 01 COMPLETE — Intent + IntentScore + IntentHint + ServiceDef integration + validation + 143 tests
Resume file: None
