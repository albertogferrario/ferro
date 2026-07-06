# Architecture

## Pipeline Overview

The Ferro Projections Protocol defines a three-layer pipeline that transforms
a structured service description into a rendered UI component tree:

```
┌──────────────┐     ┌──────────────────┐     ┌──────────────┐
│  ServiceDef  │────>│ derive_intents() │────>│   Renderer   │
│  (Layer 1)   │     │    (Layer 2)     │     │   (Layer 3)  │
└──────────────┘     └──────────────────┘     └──────────────┘
       │                      │                       │
  Data Model           Intent Scores            UI Output
  (fields,             (ranked list             (JSON-UI,
   actions,             of intents               A2UI,
   guards,              with confidence          HTML,
   relationships,       and signals)             native)
   state machine,
   intent hints)
```

Each layer has a well-defined input and output. Layers are independent:
a consumer MAY use Layer 1 alone (for schema exchange), Layers 1-2 (for
intent analysis without rendering), or the full pipeline.

## Layer 1: Service Definition (Data Model)

The Service Definition is the protocol's core input type. It describes a
business service's structure as a serializable, introspectable schema.

A ServiceDef is composed of:

- **Fields** — Named attributes with a DataType (structural category) and
  FieldMeaning (semantic annotation). Each field declares whether it is
  required, list-valued, readable, and writable.

- **Actions** — Named business operations with input parameters,
  preconditions (guard references), side effects, and optional transition
  triggers linking actions to state machine events.

- **Guards** — Named boolean conditions referenced by transitions and action
  preconditions. Guards are declarative: they declare WHAT must hold, not HOW
  to evaluate it.

- **Relationships** — Inter-service references with structural cardinality
  (OneToOne, OneToMany, ManyToOne, ManyToMany) and presentational navigation
  hints (Inline, Link, Tab, Nested, Hidden).

- **State Machine** — A lifecycle definition with states, transitions, an
  initial state, and optional entry/exit effects. State machines are flat (no
  hierarchical states) and schema-only (no closures, no runtime logic).

- **Intent Hints** — Optional manual overrides for intent derivation.
  `Primary(Intent)` forces a specific intent as top-ranked; `Exclude(Intent)`
  removes an intent from consideration.

### Schema-Only Constraint

Service Definitions MUST be fully serializable to JSON and back without
information loss. This means:

- No closures or function pointers.
- No runtime logic or executable code.
- Guards and effects are string references, not implementations.
- All types derive `Serialize` and `Deserialize`.

This constraint enables schema exchange across language boundaries, network
transport, and persistent storage. JSON Schema is generated from the Rust type
definitions via schemars and MUST NOT be hand-written.

## Layer 2: Intent Derivation (Analysis)

The derivation layer analyzes structural signals in a ServiceDef to produce a
ranked list of IntentScores. Each IntentScore contains an intent
classification, a confidence value, and the signals that contributed to the
score.

### Analyzers

Five analyzers examine different structural aspects of the ServiceDef:

1. **Field Meaning Analyzer** — Examines the FieldMeaning annotations on
   non-system fields. Proportional count-weighted signals map semantic
   annotations to intents (e.g., Money fields contribute to Summarize,
   EntityName fields contribute to Browse, DateTime + numeric fields
   contribute to Analyze).

2. **Writability Analyzer** — Examines the readable/writable ratios across
   fields. High writability signals Collect; high read-only ratios signal
   Summarize; mixed ratios signal Focus.

3. **State Machine Analyzer** — Examines guard density, branching patterns,
   transition triggers, and workflow state structure. Complex guarded
   workflows signal Process; linear progressions signal Track.

4. **Relationship Analyzer** — Examines cardinality patterns and navigation
   hints. OneToMany and ManyToMany relationships signal Browse; OneToOne with
   Inline navigation signals Focus; high relationship counts signal Browse.

5. **Action Analyzer** — Examines action signatures. Transition triggers
   signal Process; actions with many inputs signal Collect; actions with
   preconditions signal Process; simple CRUD actions contribute weakly to
   Browse.

### Normative vs. Informative

The signal types each analyzer MUST consider are **normative**: a conforming
implementation MUST examine field meanings, writability ratios, state machine
structure, relationship patterns, and action signatures.

The exact numeric weights assigned to each signal are **informative**:
implementations MAY tune weights for their specific use case. The reference
implementation's weights are documented in the [Intent
Derivation](derivation.md) section for guidance.

### Guarantees

Derivation MUST satisfy these invariants:

- The output MUST contain at least one IntentScore.
- If no analyzer produces a signal, the default fallback MUST be Focus with
  confidence 0.5.
- Browse and Focus MUST receive baseline scores (the reference implementation
  uses 0.1) to ensure they always appear in results.
- IntentScores MUST be sorted by confidence in descending order.
- Ties MUST be broken by a stable ordering defined by the implementation.
- IntentHint overrides MUST be applied after structural analysis:
  `Primary(X)` sets intent X to confidence 1.0 and moves it to rank 0;
  `Exclude(X)` removes intent X from the result set entirely.

## Layer 3: Rendering (Output)

The rendering layer transforms a ServiceDef, its derived IntentScores, and a
RenderContext into a framework-independent UI component tree.

### Renderer Trait

The Renderer trait defines a single method:

```
render(service: &ServiceDef, intents: &[IntentScore], ctx: &RenderContext)
    -> Result<Value, Error>
```

The output is `serde_json::Value` to avoid coupling to any specific component
vocabulary. Implementations produce output conforming to their target format.

### Render Context

A RenderContext controls the rendering call:

- **intent_index** — Which ranked intent to render (0 = primary, 1 = second,
  etc.). Index into the intents slice.
- **current_state** — The entity's current workflow state name, if applicable.
  Used by Process and Track intents to render state-aware UI.
- **mode** — Display (read-only views) or Input (editable forms).

### Render Mode

- **Display** — Read-only rendering. Fields render as text, badges, links,
  formatted values. Used for detail pages, list views, dashboards, and
  summaries.
- **Input** — Editable rendering. Fields render as form inputs appropriate to
  their DataType and FieldMeaning. Used for create and edit forms.

### Pluggability

The Renderer trait is the protocol's extension point for output formats. Any
target format MAY implement the trait:

- **JsonUiRenderer** — The reference implementation. Produces a `Spec`
  conforming to the `ferro-json-ui/v2` schema: a flat ID-keyed element map
  with components such as Table, Card, Form, Badge, and Progress.
- **A2UI** — A potential implementation targeting A2UI component catalogs.
- **HTML** — A potential implementation producing static or server-rendered
  HTML.
- **Native** — A potential implementation producing native mobile component
  trees.

The protocol does not mandate any particular output format. Conforming
implementations MUST implement the Renderer trait; the choice of output format
is implementation-specific.

## CAMELEON Correspondence

The three-layer pipeline corresponds to the W3C CAMELEON Reference Framework
(Calvary et al., 2003), which defines a four-level abstraction chain for
multi-target user interfaces:

| CAMELEON Level | Ferro Layer | Description |
|----------------|-------------|-------------|
| Task & Domain Model | Layer 1: ServiceDef | Describes the service's structure, operations, and lifecycle |
| Abstract UI (AUI) | Layer 2: IntentScores | Abstract interaction patterns derived from structural analysis |
| Concrete UI (CUI) | Layer 3: Renderer output | Framework-specific component trees (JSON-UI, A2UI, HTML) |
| Final UI (FUI) | (outside protocol) | Rendered pixels in browser, native app, or other runtime |

The protocol covers the first three CAMELEON levels. The fourth level (Final
UI) is outside protocol scope — it depends on the runtime environment
interpreting the Renderer output.

**Differentiation from CAMELEON:** The CAMELEON Abstract UI layer is
tree-based and statically defined. Ferro's IntentScores are dynamically
derived from structural signals with confidence scores, enabling ranked
multi-intent analysis and state-dependent rendering context. This combination
of schema-only constraints, confidence-scored intent derivation, and pluggable
rendering within a server-side framework has no direct precedent in the
CAMELEON literature.

## Roles

The protocol defines three participant roles:

### Service Author

Creates ServiceDef instances describing business services. Service authors
define the fields, actions, guards, relationships, state machines, and intent
hints that characterize their domain. Service authors do not need to
understand intent derivation or rendering — the protocol handles these
concerns automatically.

### Protocol Consumer

Reads ServiceDef instances, derives intents, and renders UI. Protocol
consumers operate the full pipeline: they accept a ServiceDef (via MCP, HTTP,
file, or any transport), call the derivation layer to obtain IntentScores, and
pass both to a Renderer to produce output. Consumers MAY also use individual
layers independently (e.g., deriving intents without rendering).

### Renderer Implementor

Implements the Renderer trait for a specific output format. Renderer
implementors define how each intent maps to UI components in their target
format. They receive a ServiceDef, IntentScores, and RenderContext, and
produce a component tree. Renderer implementors do not need to understand
derivation — they receive pre-computed IntentScores.
