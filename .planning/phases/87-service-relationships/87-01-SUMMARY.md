---
phase: 87-service-relationships
plan: 01
subsystem: projections
tags: [relationships, cardinality, navigation-hint, service-def, serde, schemars]

# Dependency graph
requires:
  - phase: 86-actions-preconditions
    provides: ServiceDef with actions/guards/validate(), Warning enum
provides:
  - RelationshipDef, Cardinality, NavigationHint types
  - ServiceDef relationship builder methods (.relationship(), .belongs_to(), .has_many(), .has_one(), .belongs_to_many())
  - Validation warnings for duplicate relationships and ManyToMany with FK
affects: [88-intent-vocabulary, 89-intent-graph]

# Tech tracking
tech-stack:
  added: []
  patterns: [two-dimensional relationships (cardinality + navigation), cardinality-derived defaults]

key-files:
  created: [ferro-projections/src/relationship.rs]
  modified: [ferro-projections/src/service.rs, ferro-projections/src/state.rs, ferro-projections/src/lib.rs]

key-decisions:
  - "NavigationHint defaults from Cardinality.default_navigation(), overridable per relationship"
  - "Convenience shorthands (belongs_to, has_many, has_one, belongs_to_many) delegate to .relationship() with appropriate Cardinality"
  - "DuplicateRelationship and ManyToManyWithForeignKey are warnings, not errors"

patterns-established:
  - "Two-dimensional relationship schema: structural (Cardinality) + presentational (NavigationHint)"
  - "Convenience builder shorthands wrapping full-form builder method"

# Metrics
duration: 8min
completed: 2026-02-28
---

# Phase 87-01: Service Relationships Summary

**RelationshipDef, Cardinality, NavigationHint types with ServiceDef builder integration and validation**

## Performance

- **Duration:** 8 min
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- RelationshipDef struct with builder API (foreign_key, inverse, navigation, description)
- Cardinality enum (OneToOne, OneToMany, ManyToOne, ManyToMany) with default_navigation() mapping
- NavigationHint enum (Inline, Link, Tab, Nested, Hidden) for UI presentation signals
- ServiceDef builder methods: .relationship(), .belongs_to(), .has_many(), .has_one(), .belongs_to_many()
- Validation: DuplicateRelationship and ManyToManyWithForeignKey warnings
- 20 new tests (8 relationship + 12 service), total 113 unit + 5 doc = 118

## Task Commits

1. **Task 1: Create relationship types and ServiceDef integration** - `bfe77fb` (feat)
2. **Task 2: Add validation rules and comprehensive test suite** - `dc977ab` (test)

## Files Created/Modified
- `ferro-projections/src/relationship.rs` - RelationshipDef, Cardinality, NavigationHint types with builder API
- `ferro-projections/src/service.rs` - ServiceDef integration: relationships field, builder methods, validation steps 8-9
- `ferro-projections/src/state.rs` - DuplicateRelationship and ManyToManyWithForeignKey warning variants
- `ferro-projections/src/lib.rs` - Module registration and re-exports

## Decisions Made
None - followed plan as specified.

## Deviations from Plan
None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- RelationshipDef types ready for Phase 88 (Intent Vocabulary) to consume as IntentGraph edge signals
- Cardinality + NavigationHint provide the two dimensions needed for intent derivation
- All types serializable and schema-capable (schemars) for protocol use

---
*Phase: 87-service-relationships*
*Completed: 2026-02-28*
