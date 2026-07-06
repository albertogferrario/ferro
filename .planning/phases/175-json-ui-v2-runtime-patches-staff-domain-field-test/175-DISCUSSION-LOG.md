# Phase 175: JSON-UI v2 runtime patches — staff-domain field test findings (F1–F6) — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `175-CONTEXT.md` — this log preserves the alternatives considered
> and the rationale for the selected option.

**Date:** 2026-05-20
**Phase:** 175 — json-ui-v2-runtime-patches-staff-domain-field-test
**Mode:** `--auto` (recommended defaults selected from the existing CONTEXT.md "Decisions Required at Planning Time" section; no interactive prompts)
**Areas discussed:** F1 depth ceiling, F1 diagnostic semantics, F2 CheckboxGroup posture, F3 tabs render strategy, F4 Switch posture, F5 file-upload scope, F6 DataTable interpolation scope

---

## F1.a — `MAX_NESTING_DEPTH` value

| Option | Description | Selected |
|--------|-------------|----------|
| 8 | Exact ceiling consumer evidence requires today (`dashboard → root → DetailPage → tab → card → form → row → switch`) | |
| 12 | Moderate headroom — adds one nested tab + nested card | |
| 16 | Generous headroom — nested cards inside nested tabs inside layout shell with margin | ✓ |
| Unbounded | Remove the depth limit entirely; rely only on cycle detection | |

**Selected:** 16
**Rationale:** Consumer evidence requires at least 8. Phase 164's "5 is enough" rationale underestimated current usage; the next field-test should not have to re-trip the same gate. 16 is the high end of the "12 or 16" range the original CONTEXT recommended, chosen so plans have explicit headroom past today's worst-case spec. Unbounded is rejected because the limit serves as a runaway guard for malformed specs, and cycle detection alone cannot catch deeply-nested but acyclic generated specs.

---

## F1.b — Diagnostic split (depth-limit vs cycle)

| Option | Description | Selected |
|--------|-------------|----------|
| Keep current "cycle guard tripped at depth N" | Single diagnostic conflates depth-limit and cycle | |
| Split into two distinct diagnostics | `depth limit exceeded at depth N (max=M)` for depth trip; `cycle detected: <path>` for real cycles | ✓ |

**Selected:** Split
**Rationale:** The current diagnostic is misleading — it says "cycle" for a non-cycle condition. Future failures need to be legible. Splitting also lets the cycle detector emit only when it observes a real revisit, recovering its diagnostic value.

---

## F2 — `CheckboxGroup` component posture

| Option | Description | Selected |
|--------|-------------|----------|
| (a) Register only | Reintroduce `CheckboxGroup` as a first-class v2 component, same semantics as v1; no documented substitution | |
| (b) Document only | No reintroduction; document the `Form` + repeated `Checkbox[]` substitution path | |
| (c) Both | Register the component AND document the substitution path | ✓ |

**Selected:** (c) Both
**Rationale:** Default from CONTEXT.md is "both" unless substitution is obviously the cleaner v2 idiom. The `Form` + `Checkbox[]` substitution is more verbose and creates a sibling-component contract (shared `name="copy_to[]"`) that's easy to get wrong. Reintroduction unblocks the immediate consumer; the documented substitution preserves compositional simplicity for ad-hoc cases where authors want flat checkbox lists.

---

## F3 — Tab panel render strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Server-side conditional render | Server uses `?tab=` to render only the selected panel; others are empty stubs or omitted | |
| Client-side IIFE | Small runtime IIFE sets `hidden=true` on inactive panels at boot; tab-strip click handler toggles | ✓ |

**Selected:** Client-side IIFE
**Rationale:** Default from CONTEXT.md is client-side unless a consumer needs instant tab switching to be avoided. Consumer ergonomics — no flash, instant switching without round-tripping — outweigh the "cleaner cut" of server-side. The IIFE composes with the existing tab-strip click handler that already toggles the URL query.

---

## F4 — `Switch` component posture

| Option | Description | Selected |
|--------|-------------|----------|
| (a) Register only | Reintroduce `Switch` as a first-class v2 component with toggle semantics distinct from `Checkbox` | |
| (b) Document only | No reintroduction; document `Checkbox` with `variant: "switch"` styling as substitution | |
| (c) Both | Register native `Switch` AND document the `Checkbox variant=switch` alternative | ✓ |

**Selected:** (c) Both
**Rationale:** Same shape as F2. `Switch` is semantically distinct from `Checkbox` (state-flip vs binary-choice), so a dedicated native component is justified — this is the "ferro-direction" framing in CONTEXT.md. The documented `Checkbox variant=switch` substitution covers consumers who don't need the distinct semantic and prefer composing existing primitives.

---

## F5 — File-upload scope (Input file + Form enctype)

| Option | Description | Selected |
|--------|-------------|----------|
| Ship file input alone | Land `Input[input_type=file]` rendering; defer `Form.enctype` propagation | |
| Ship enctype alone | Land `Form.enctype` emission; defer file-input rendering | |
| Ship both in one plan | Single plan, both emitter changes, one self-check submitting a real multipart body | ✓ |

**Selected:** Ship both
**Rationale:** CONTEXT.md already locked this: shipping one without the other unblocks nothing. A `<form>` without `enctype=multipart/form-data` cannot carry a file; a file input without the surrounding `enctype` is encoded as `application/x-www-form-urlencoded`. One plan, two emitter changes, end-to-end multipart self-check.

---

## F6 — DataTable interpolation scope

| Option | Description | Selected |
|--------|-------------|----------|
| Extend column-cell pass to action URLs | Reuse the existing `{row.X}` interpolation path; add it to per-row action templates | ✓ |
| Rewrite interpolation as a separate pipeline | New pass dedicated to row-action templates | |
| Push interpolation responsibility to spec authors (Approach B only) | Deprecate Approach A; require explicit per-row inline buttons everywhere | |

**Selected:** Extend column-cell pass
**Rationale:** CONTEXT.md identifies the likely cause as the interpolation pass not extending to action-URL templates. The additive extension keeps the existing column-cell behavior identical and avoids the verbose Approach B fallback that consumers already report as inconsistent across the catalog.

---

## Claude's Discretion

The auto-resolved decisions above are the planner's locked inputs. The following remain Claude's discretion at plan/execute time, per CONTEXT.md scope:

- Exact wording of the new depth-limit and cycle diagnostics (must mention the limit value and offending depth)
- Internal implementation choice for the tabs IIFE (single-script vs co-located with the tab-strip handler)
- Self-check fixture choice for F5 (in-process multipart submit vs spec round-trip)
- Naming of the `Switch` component's underlying ARIA semantics (`role="switch"` is the obvious pick but the planner can confirm against accessibility refs at plan time)

## Deferred Ideas

None surfaced during auto-discuss. The consumer-side urlencoded fallback (mentioned in CONTEXT.md "Out of Scope") remains a separate consumer-repo cleanup. v1 catalog re-imports beyond F2/F4 stay out of scope.
