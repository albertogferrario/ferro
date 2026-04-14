# Phase 134: Relocate Renderers to Output Crates - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-14
**Phase:** 134-relocate-renderers-to-output-crates
**Areas discussed:** Module organization, Dependency direction, Re-export strategy, Feature flag design
**Mode:** --auto (all decisions auto-selected)

---

## Module Organization

| Option | Description | Selected |
|--------|-------------|----------|
| New `projection/` module | Separate directory in ferro-json-ui for relocated projection renderer code | ✓ |
| Merge into existing `render.rs` | Add projection rendering alongside HTML rendering | |
| Flat files in `src/` | Place relocated files at root of ferro-json-ui/src | |

**User's choice:** [auto] New `projection/` module (recommended default)
**Notes:** Existing `render.rs` handles HTML rendering of component trees. Projection rendering (ServiceDef → JSON-UI spec) is a separate concern — mixing them would conflate two different rendering pipelines.

---

## Dependency Direction

| Option | Description | Selected |
|--------|-------------|----------|
| Behind feature flag | ferro-json-ui optionally depends on ferro-projections via `projections` feature | ✓ |
| Always-on dependency | ferro-json-ui unconditionally depends on ferro-projections | |
| No dependency (copy types) | Duplicate needed types to avoid cross-crate dependency | |

**User's choice:** [auto] Behind feature flag (recommended default)
**Notes:** Keeps ferro-json-ui usable standalone for schema types. Matches existing pattern in ferro-cli.

---

## Re-export Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Clean break | Remove all visual re-exports from ferro-projections | ✓ |
| Deprecated re-exports | Keep re-exports with `#[deprecated]` attribute pointing to ferro-json-ui | |
| Dual availability | Keep working re-exports in both crates | |

**User's choice:** [auto] Clean break (recommended default)
**Notes:** Pre-1.0, no backward compatibility needed. Clean break prevents stale imports.

---

## Feature Flag Design

| Option | Description | Selected |
|--------|-------------|----------|
| `projections` | Feature name matching ferro-cli's existing pattern | ✓ |
| `renderer` | Feature name describing what it enables | |
| `visual` | Feature name matching ferro-projections' old pattern | |

**User's choice:** [auto] `projections` (recommended default)
**Notes:** Consistent with ferro-cli's existing `projections = ["dep:ferro-projections"]` pattern.

---

## Claude's Discretion

- Internal module visibility for helper functions after relocation
- Test migration strategy
- Whether `field_map` and `relationship_map` modules stay in ferro-projections `render/mod.rs`

## Deferred Ideas

- ServiceDef derivation from models → Phase 135
- Crate consolidation audit → CONC-04
- WhatsApp renderer → v14.0+
