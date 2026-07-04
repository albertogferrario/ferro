# Design System Principles

JSON-UI specs are validated against a set of design rules at authoring time and in CI. The design system is structured around three pillars: a semantic token vocabulary, intent-keyed structural patterns, and a diagnostic lint engine.

## Semantic Tokens

Components emit semantic class names (`bg-primary`, `text-text-muted`, `border-border`) rather than literal Tailwind utility values. Themes supply the values for those names through CSS custom properties. This separation means that changing a color, radius, or motion curve requires editing one theme file — no component code changes.

The token vocabulary has 30 fixed slots, grouped into surface, role, shape, shadow, typography, density, motion, focus, and display categories. See [Token Reference](tokens.md) for the complete table.

## Intent-Keyed Patterns

The seven projection intents (Browse, Focus, Collect, Process, Summarize, Analyze, Track) also serve as page archetypes. Each intent has a canonical composition: a Browse page uses a DataTable or KanbanBoard, a Collect page uses a dedicated Form page with breadcrumb navigation, a Process page uses a KanbanBoard with grouped card actions.

Specifying `design.intent` in a JSON-UI spec tells the lint engine which structural rules apply to that page. Rules that apply to all intents (such as PageHeader and destructive-confirmation) run regardless of the declared intent.

See [Pattern Catalog](patterns.md) for the full per-intent rule set with conforming and violating examples.

## Lint as Diagnostics

The design lint engine is diagnostic-only. It never rejects a spec at parse time or affects rendering. Findings are warnings or informational notices that an agent or developer can inspect, act on, or explicitly allow.

Rules can be silenced per-spec by listing the rule id in the `design.allow` array:

```json
{
  "design": {
    "intent": "collect",
    "allow": ["breadcrumb-on-subpages"]
  }
}
```

An empty findings array means the spec conforms to all applicable rules. The `--deny` flag on `ferro design:lint` turns warnings into a non-zero exit code for CI gates.

See [Linting Guide](linting.md) for CLI usage, the `design_lint` MCP tool, and the full output shape.

---

See also: [Token Reference](tokens.md) — [Variant Vocabulary](variants.md) — [Pattern Catalog](patterns.md) — [Linting Guide](linting.md)
