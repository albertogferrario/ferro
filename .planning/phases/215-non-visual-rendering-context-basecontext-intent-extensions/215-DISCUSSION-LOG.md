# Phase 215: Non-visual rendering context — BaseContext + Intent extensions - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-13
**Phase:** 215-non-visual-rendering-context-basecontext-intent-extensions
**Mode:** `--auto` (Claude selected recommended option for each gray area)
**Areas discussed:** BaseContext/VisualContext composition, evaluated_guards representation, verbosity, Intent labeling, empty-intent handling

---

## BaseContext / VisualContext composition

| Option | Description | Selected |
|--------|-------------|----------|
| Embed `base: BaseContext` in VisualContext | Add new fields to BaseContext; refactor VisualContext to embed it, collapsing the duplicated `intent_index`/`current_state` | ✓ |
| Duplicate fields again | Add new fields to both BaseContext and VisualContext, keep them flat | |
| Add to BaseContext only, ignore in visual | Visual renderer never reads the new fields | |

**Choice:** Embed (recommended). **Notes:** VisualContext currently re-declares
`intent_index` + `current_state` rather than embedding BaseContext — parallel sources of
truth that have already drifted. Phase collapses them. Fallback documented in CONTEXT D-02
if builder.rs churn is excessive.

---

## evaluated_guards representation

| Option | Description | Selected |
|--------|-------------|----------|
| `HashMap<String, bool>`, absent = render | Keyed by guard/precondition name; only explicit `false` hides an action | ✓ |
| `HashMap<String, bool>`, absent = hide | Stricter; would change current "render all" behavior | |
| Typed wrapper / `Vec<(String,bool)>` | More ceremony, no clear benefit | |

**Choice:** HashMap, absent = render (recommended, per COMP-05 v14.0 table).
**Notes:** Default empty map = render everything = backward-compatible with today's visual
output. Keys reuse `ActionDef::preconditions` / `GuardDef::name` strings.

---

## verbosity

| Option | Description | Selected |
|--------|-------------|----------|
| `enum Verbosity { Brief, Full }`, default Full | Full = current full-render behavior | ✓ |
| Numeric level (0..n) | Over-general for a two-state need | |
| Defer verbosity to Phase 216 | CHAN-01 bundles it with the context extension | |

**Choice:** Brief/Full enum, default Full (recommended). **Notes:** First consumer is the
Phase 216 text renderer; visual renderer ignores it. Default preserves current behavior.

---

## Intent labeling

| Option | Description | Selected |
|--------|-------------|----------|
| `Intent::label() -> &str` in ferro-projections; migrate mcp call sites | Infallible; Custom returns inner string | ✓ |
| Keep `{:?}` but centralize in a helper fn | Still couples to Debug derive | |
| Add a serde-name lookup | Heavier; label is already the snake_case serde name | |

**Choice:** `label()` method (recommended, per COMP-05 weakness #2). **Notes:** Migrate
3 `ferro-mcp` call sites (render_projection.rs:94/102, generate_projection.rs:89) +
review projection_coverage.rs:173. `intent_layout.rs` `{intent:?}` uses are error messages,
not labels.

---

## empty-intent handling

| Option | Description | Selected |
|--------|-------------|----------|
| Typed `Error::NoIntents` in ferro-projections | Modality-agnostic; reusable by Phase 216 text renderer; tested | ✓ |
| Reuse json-ui `IntentIndexOutOfBounds` only | Lives in the wrong crate for non-visual reuse | |
| Emit a `tracing::warn!` and continue | Success criterion calls for a typed error/warning under test | |

**Choice:** Add `Error::NoIntents` to ferro-projections (recommended, per COMP-05
weakness #3). **Notes:** Visual renderer's existing `IntentIndexOutOfBounds` path stays
unchanged (D-09); new variant exists for the non-visual surface.

---

## Claude's Discretion

- Exact error variant name, `Verbosity` derive set, and whether the new context fields get
  serde — constrained by "default preserves visual behavior" + "no new serde unless a
  consumer needs it."

## Deferred Ideas

- `device_class` / `MobileContext` (text-renderer-first milestone drops mobile)
- `FieldDef::render_hint` (CHAN-03, Phase 216)
- conversational-text `Renderer` (CHAN-04, Phase 216)
- intent vocabulary reshaping (CHAN-05, research outcome)
