# Phase 194: Core Checkpoint Tool - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-10
**Phase:** 194-core-checkpoint-tool
**Mode:** --auto (recommended defaults auto-selected)
**Areas discussed:** Field→column matching rule, Computed-field exemption, Reconstruction-completeness detection, Output type design, Status cache shape, next_steps ranking/dedup

---

## Field→column matching rule

| Option | Description | Selected |
|--------|-------------|----------|
| Exact snake_case name match vs `list_models` column set | Reuse projection_coverage's lowercase model-name match for source resolution; compare FieldDef.name to FieldInfo.name exactly | ✓ |
| Fuzzy / normalized match | Normalize both sides (strip suffixes, plural/singular) before compare | |

**Choice:** Exact match (recommended). Both sides are already snake_case; no DB needed via `list_models::execute` (CHK-02). Unresolved source model → `not_checked` (CHK-03).

---

## Computed/virtual field exemption (CHK-04)

| Option | Description | Selected |
|--------|-------------|----------|
| Exempt by construction (relationships) + builder vocabulary | Relationships live in `.relationships` not `.fields`; only column-backed builders are checked | ✓ |
| Add explicit `computed` flag to FieldDef | Grow the field model with a marker | |

**Choice:** By-construction exemption (recommended), with a **research flag**: `FieldDef`/`FieldMeaning` currently have no computed/virtual marker. Researcher confirms how computed fields surface; explicit-marker option deferred unless research shows it's required.

---

## Reconstruction-completeness detection (CHK-05)

| Option | Description | Selected |
|--------|-------------|----------|
| Builder-invocation count vs reconstructed `fields.len()` | Regex-count `.field`/`.optional_field`/`.read_only_field` in source, compare to ServiceDef.fields.len(); mismatch → warn | ✓ |
| Assume reconstruction complete | Trust reconstruct_service_def output | |

**Choice:** Count-based completeness check (recommended) — roadmap success criterion 4 requires a real assertion, not an assumption. Mismatch → `warn` with `reason: "reconstruction_incomplete"`.

---

## Output type design

| Option | Description | Selected |
|--------|-------------|----------|
| Public types in `checkpoint_projection.rs`, reused by Phase 195 | `Finding`/`SeamStatus`/`SeamResult`/verdict defined once at the module boundary | ✓ |
| Shared types crate | Extract to a separate types module | |

**Choice:** Public types in `checkpoint_projection.rs` (recommended) — matches STATE.md design decision; Phase 195 wrapper seams reuse verbatim. `SeamStatus` is a distinct enum so `not_checked` can never coerce to `pass` (CHK-03).

---

## Status cache shape

| Option | Description | Selected |
|--------|-------------|----------|
| Full verdict + derived ambient status + timestamp | Write complete verdict to `.ferro/checkpoints/{name}.json` so Phase 195 stale-ok reads without recompute | ✓ |
| Status-only summary | Write only `clean`/`failing`/`unverified` | |

**Choice:** Full verdict + ambient status (recommended) — roadmap freshness decision: ambient status is a stale-ok read of this cache. Pass timestamp in (testability).

---

## next_steps ranking & dedup (CHK-06)

| Option | Description | Selected |
|--------|-------------|----------|
| Failures before warnings, seam-order within rank, dedup by (subject, fix) | Locked ranking from roadmap; actionable string per entry | ✓ |

**Choice:** Recommended ranking (locked by roadmap). Each entry `"<fix> (seam: <seam_name>)"`.

## Claude's Discretion

- Field-builder counting regex specifics; internal module layout; test fixture layout.

## Deferred Ideas

- Phase 195 wrapper seams + inline hook + ambient read-surfacing.
- Model-anchored fan-out (design-spec non-goal).
- Explicit computed/virtual FieldDef marker (only if research requires it).
