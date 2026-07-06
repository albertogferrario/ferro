# Phase 133: Generalize Renderer Trait - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-14
**Phase:** 133-generalize-renderer-trait
**Areas discussed:** Trait object compatibility, Context type design, TemplateRenderer output, Migration path
**Mode:** --auto (all decisions auto-selected as recommended defaults)

---

## Trait Object Compatibility

| Option | Description | Selected |
|--------|-------------|----------|
| Associated types (no trait objects) | Renderers used as concrete types, never dyn Renderer | ✓ |
| Generic parameter on trait | `Renderer<O, C>` — verbose but allows trait objects with turbofish | |
| Keep serde_json::Value with conversion | Leave trait signature, add From impls | |

**User's choice:** [auto] Associated types (recommended default)
**Notes:** Codebase audit confirms: no dyn Renderer usage anywhere. JsonUiRenderer and TemplateRenderer always concrete.

---

## Context Type Design

| Option | Description | Selected |
|--------|-------------|----------|
| Base struct with modality-agnostic fields only | intent_index + current_state in base; mode + templates in visual context | ✓ |
| Fully generic (no base) | Each renderer defines everything from scratch | |
| Keep current RenderContext, add visual extension | Backward compatible but keeps visual cruft in base | |

**User's choice:** [auto] Base struct with modality-agnostic fields (recommended default)
**Notes:** TemplateRenderer already only uses intent_index and current_state — validates the split.

---

## TemplateRenderer Output Type

| Option | Description | Selected |
|--------|-------------|----------|
| serde_json::Value | Generic structured data, consumed by MiniJinja | ✓ |
| Custom TemplateContext struct | More type-safe but less flexible | |

**User's choice:** [auto] serde_json::Value (recommended default)

---

## Migration Path

| Option | Description | Selected |
|--------|-------------|----------|
| Change trait + minimal downstream fix | Update trait, both renderers, minimum ferro-mcp/cli compilation fix | ✓ |
| Full downstream rewrite | Update all consumers in one phase | |
| Trait alias for backward compatibility | Type alias old signature to new | |

**User's choice:** [auto] Change trait + minimal downstream fix (recommended default)
**Notes:** Full downstream rewrite deferred to Phase 134 when renderers relocate.

---

## Claude's Discretion

- Naming choices (BaseContext vs ProjectionContext)
- Composition vs flattening for VisualContext
- RenderMode location
- Test structure

## Deferred Ideas

- Renderer relocation → Phase 134
- ServiceDef derivation → Phase 135
