# Changelog

All notable changes to the Ferro Projections Protocol specification are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Protocol versions use date-based identifiers with a stability suffix.

## 0.1.0-draft (2026-03-01)

Initial protocol specification.

### Added

- **Core data model:** 22 public types formalized as JSON Schema 2020-12 documents
  - `ServiceDef`, `FieldDef`, `DataType`, `FieldMeaning` (18 variants + Custom)
  - `StateMachine`, `StateDef`, `Transition`
  - `ActionDef`, `InputDef`, `GuardDef`
  - `RelationshipDef`, `Cardinality`, `NavigationHint`
  - `Intent` (7 variants + Custom), `IntentScore`, `IntentHint`
  - `Renderer` trait, `RenderContext`, `RenderMode`
  - `Warning` (8 variants), `Error` (4 variants)

- **Intent derivation engine:** 5-analyzer pipeline documented
  - Field meaning analyzer
  - Writability analyzer
  - State machine analyzer
  - Relationship analyzer
  - Action analyzer

- **Rendering:** `Renderer` trait contract with `JsonUiRenderer` reference implementation

- **Validation:** `ServiceDef::validate()` rules for structural correctness

- **Extension mechanism:** Two-tier system (vendor `x-*` prefix + URI-namespaced critical extensions)

- **Conformance levels:** Three levels defined (Schema, Derivation, Rendering)

- **Security considerations:** 7 security properties documented

- **Prior art:** 9 related works acknowledged (CAMELEON, SAP Fiori, MECANO, Siren, A2UI, AG-UI, MCP, Open-JSON-UI, json-render)

### Notes

- Based on the `ferro-projections` crate, Phases 84-93 of Ferro Framework v9.0
- Reference implementation: 309+ tests validating schema, derivation, and rendering
- 100% primary intent accuracy across 12 representative ServiceDefs covering all 7 intents
- Derivation signal categories are normative; exact numeric weights are implementation-specific
