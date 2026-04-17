# Phase 135: ServiceDef Derivation Bridge - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-17
**Phase:** 135-servicedef-derivation-bridge
**Areas discussed:** Model metadata source, Field type mapping, MCP tool design, Round-trip scope
**Mode:** --auto (all decisions auto-selected)

---

## Model Metadata Source

| Option | Description | Selected |
|--------|-------------|----------|
| Direct SeaORM type | from_model() takes SeaORM entity types directly | |
| Intermediate metadata struct | from_model() takes a ModelMetadata struct, keeping ferro-projections SeaORM-free | ✓ |
| String-based JSON input | from_model() takes raw JSON matching list_models output | |

**User's choice:** [auto] Intermediate metadata struct (recommended default)
**Notes:** Keeps ferro-projections free of SeaORM dependencies. ferro-mcp bridges by converting its parsed ModelDetails → ModelMetadata → ServiceDef.

---

## Field Type Mapping

| Option | Description | Selected |
|--------|-------------|----------|
| Pattern matching on type string | DataType::from_column_type() matches common Rust/SeaORM type strings | ✓ |
| Enum-based column type | Introduce a ColumnType enum mirroring SeaORM's | |
| Mapping table config | External mapping config file | |

**User's choice:** [auto] Pattern matching on type string (recommended default)
**Notes:** Simple, lives in field.rs alongside existing infer_meaning(). Handles the common types without introducing new dependencies.

---

## MCP Tool Design

| Option | Description | Selected |
|--------|-------------|----------|
| ServiceDef JSON | Tool returns serialized ServiceDef for agent inspection | ✓ |
| Rust source code | Tool generates ServiceDef builder code as a string | |
| Both formats | JSON + code, selectable via parameter | |

**User's choice:** [auto] ServiceDef JSON (recommended default)
**Notes:** JSON output is more flexible — agent can inspect, refine, and decide how to use it. Code generation is brittle.

---

## Round-trip Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Fields only with relationship hints | Derive fields (name, type, meaning) + FK detection. Actions/state machines stay manual. | ✓ |
| Fields + relationships | Also infer RelationshipDef from FK analysis | |
| Full inference | Attempt to infer actions, state, and relationships | |

**User's choice:** [auto] Fields only with relationship hints (recommended default)
**Notes:** 80/20 path. Actions and state machines are too domain-specific to infer from schema alone.

## Claude's Discretion

- ModelMetadata module location
- from_model() as inherent method vs standalone function
- Display name derivation heuristic
- Test structure for round-trip demonstration

## Deferred Ideas

- Cross-model relationship inference → future phase
- Action inference from route handlers → future phase
- State machine inference from enum fields → future phase
