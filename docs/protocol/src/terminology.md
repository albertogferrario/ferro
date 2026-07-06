# Terminology

This section defines domain-specific terms used throughout the specification.
Terms are listed alphabetically for reference. Capitalized instances of these
terms in the specification carry the precise meanings defined here.

---

**Action**
: A named business operation that a service can perform, described by
  `ActionDef`. Actions declare their input parameters, preconditions
  (guards that MUST hold), side effects, and an optional transition trigger
  linking the action to a state machine event. Examples: "submit order",
  "approve review", "send invoice".

**Cardinality**
: The structural multiplicity of a relationship between two services. One of
  four values: `OneToOne`, `OneToMany`, `ManyToOne`, `ManyToMany`. Each
  cardinality maps to a default Navigation Hint.

**Confidence Score**
: A floating-point value in the range \[0.0, 1.0\] indicating how strongly a
  Service Definition's structural signals match a given Intent. Higher values
  indicate stronger structural alignment. Confidence scores are NOT
  probabilities and do not sum to 1.0.

**Data Type**
: An abstract category for field values, independent of database storage
  details. One of ten values: `String`, `Integer`, `Float`, `Boolean`,
  `DateTime`, `Date`, `Json`, `Binary`, `Uuid`, `Enum`.

**Derivation**
: The process of analyzing a Service Definition's structural properties to
  produce a ranked list of Intent Scores. Derivation is performed by a
  pipeline of analyzers, each examining different structural signals. The
  output is always at least one Intent Score (default fallback: Focus at
  0.5).

**Field Meaning**
: A semantic annotation on a field indicating its domain role, described by
  `FieldMeaning`. Drives rendering decisions: a field annotated `Money`
  renders as currency, `Status` renders as a badge, `Email` renders as a
  mailto link. Eighteen known variants exist (Identifier, ForeignKey,
  EntityName, Email, Phone, Url, ImageUrl, Money, Percentage, Quantity,
  Status, Category, Boolean, FreeText, CreatedAt, UpdatedAt, DateTime,
  Sensitive) plus a `Custom(String)` fallback for domain-specific meanings.

**Guard**
: A named boolean precondition for transitions or actions, described by
  `GuardDef`. Guards are declarative: they declare what condition MUST hold,
  but the evaluation logic lives outside the protocol schema. Referenced by
  name from `Transition.guard` and `ActionDef.preconditions`.

**Intent**
: A high-level user interaction pattern derived from service structure. Seven
  structurally-derivable intents are defined: `Browse` (collection
  navigation), `Focus` (single-entity detail), `Collect` (data capture),
  `Process` (workflow progression), `Summarize` (overview dashboard),
  `Analyze` (time-series exploration), `Track` (timeline / audit trail).
  A `Custom(String)` variant provides an escape hatch for intents not
  structurally derivable.

**Intent Hint**
: An optional manual override for intent derivation. `Primary(Intent)` forces
  a specific intent as the top-ranked result. `Exclude(Intent)` removes an
  intent from consideration entirely. Intent hints allow service authors to
  correct derivation when structural signals are ambiguous.

**Intent Score**
: The output of derivation for a single intent, described by `IntentScore`.
  Composed of the intent classification, a confidence score, and a list of
  matching signals (human-readable strings identifying which structural
  features contributed to the score).

**Navigation Hint**
: Presentational guidance for how a relationship SHOULD be rendered in a user
  interface. One of five values: `Inline` (embed in current view), `Link`
  (navigable reference), `Tab` (separate tab in detail view), `Nested`
  (nested list or table), `Hidden` (exists but not displayed by default).
  Defaults are derived from Cardinality and MAY be overridden per
  relationship.

**Render Context**
: Runtime parameters passed to a Renderer for a single render call, described
  by `RenderContext`. Includes the intent index (which ranked intent to
  render), the current workflow state (for stateful intents), and the render
  mode (Display or Input).

**Render Mode**
: Controls whether fields render as read-only display or editable inputs. Two
  values: `Display` (read-only views: detail pages, lists, summaries) and
  `Input` (editable form views: create, edit).

**Renderer**
: A component that transforms a Service Definition, its derived Intent
  Scores, and a Render Context into a UI component tree. Defined by the
  `Renderer` trait. The output format is implementation-specific:
  `JsonUiRenderer` produces a `Spec` conforming to the
  `ferro-json-ui/v2` schema; implementations MAY target A2UI, HTML,
  native components, or any other format.

**Service Definition**
: The protocol's core input type, described by `ServiceDef`. A complete,
  serializable schema describing a business service's structure: its fields
  (with data types and semantic meanings), actions (with inputs and
  preconditions), guards, inter-service relationships, state machine
  (lifecycle definition), and intent hints. Service Definitions contain no
  closures, no runtime logic, and are fully JSON-round-trip safe.

**Signal**
: A structural feature of a Service Definition that contributes to intent
  derivation. Signals are typed tuples of (Intent, weight, description).
  Examples: a field with meaning `Money` is a signal for the `Summarize`
  intent; a guarded transition is a signal for the `Process` intent. Signal
  types are normative; exact weights are informative.

**State Machine**
: A lifecycle definition describing the states a service entity can occupy and
  the transitions between them, described by `StateMachine`. Schema-only: no
  runtime execution logic. Composed of states (with entry/exit effects),
  transitions (with guards and actions), and an initial state. State machines
  are flat (no hierarchical or compound states).

**Transition**
: A named event that moves a state machine from one state to another,
  described by `Transition`. Each transition specifies a source state, an
  event name, a target state, an optional guard condition, and optional
  side-effect actions. The event name MAY be referenced by an ActionDef's
  `transition_trigger` to link business operations to state progression.
