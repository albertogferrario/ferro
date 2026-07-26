# Phase 262: MCP + catalog + docs + publish - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-26
**Phase:** 262-mcp-catalog-docs-publish
**Mode:** `--auto` (all gray areas auto-selected; recommended defaults chosen and logged)
**Areas discussed:** Catalog/mirror verification, generation_context guidance, generation_context drift-guarding, docs coverage & placement, CSS regeneration, publish mechanics & gate, commit hygiene

---

## Catalog / mirror verification (SC-1)

| Option | Description | Selected |
|--------|-------------|----------|
| Verification-first (record pre-existing evidence) | SC-1 is already satisfied in-tree (canonical + ferro-mcp mirror both at 53 incl. LiveFragment). Re-run the two guards, record evidence, no re-implementation. | ✓ |
| Re-implement the mirror count | Treat mirror count as this phase's output per 260 D-06 boundary. | |

**Auto-selected:** Verification-first. **Rationale:** Scouting found Phase 260 Plan 04 already bumped both the canonical drift guard (`catalog.rs:1303` = 53) and the ferro-mcp mirror (`json_ui_catalog.rs:420` = 53, names incl. LiveFragment). Claiming credit or re-implementing would be false work (feedback_validate_scope_premises).

---

## generation_context guidance (SC-2 — killer feature)

| Option | Description | Selected |
|--------|-------------|----------|
| All three capabilities, compact style + drift-guard | Guidance for LiveFragment (when/how to bind a per-key snapshot), `#[memoize]` (render-path dedup), `asset!()` (embed + fetch); compact ids/one-liners + docs pointer; drift-guarded. | ✓ |
| LiveFragment only | Document only the killer element, defer the two macros. | |
| Full prose manual inline | Long-form guidance embedded in generation_context. | |

**Auto-selected:** All three, compact + drift-guarded. **Rationale:** SC-2 explicitly requires all three documented. Compact style mirrors the proven 253 D-06 / 258 D-04 pattern (generation_context is inline agent context, not a manual). generation_context quality is this phase's polish budget.

---

## generation_context drift-guarding

| Option | Description | Selected |
|--------|-------------|----------|
| Test ties every mention to its authoritative source | Assert component name / macro / data-attribute in the guidance exists in BUILTIN registry / macro exports / runtime contract. | ✓ |
| No guard (hand-written, unverified) | Accept drift risk. | |

**Auto-selected:** Registry-tied drift guard (258 D-05 / 253 D-09 pattern). **Rationale:** Hand-written surfaces that mirror a registry must be drift-guarded — an established v16.x convention.

---

## docs/src coverage & placement (SC-3)

| Option | Description | Selected |
|--------|-------------|----------|
| Extend existing pages first | LiveFragment → components.md + runtime-primitives.md; asset!() → features/ferro-assets.md; #[memoize] → projections.md (or short section). Cross-link, wire new page into SUMMARY.md only if warranted. | ✓ |
| New dedicated v17.0 docs chapter | One new page/section cluster for all three. | |

**Auto-selected:** Extend existing pages first (258 D-08 practice). **Rationale:** LiveFragment is a builtin component (components.md is the format anchor); asset!() belongs on the existing Asset Pipeline page; #[memoize] is a render-path concern. Neutral voice, ≥1 example each, mdBook exits 0. Exact placement = Claude's discretion within the constraint.

---

## CSS regeneration (ferro-base.css)

| Option | Description | Selected |
|--------|-------------|----------|
| No regen (verify then skip) | LiveFragment container emits only data-* attributes, zero Tailwind classes; grep to confirm, skip regen unless a class surfaces. | ✓ |
| Regen unconditionally | Run the regen script as a matter of course. | |

**Auto-selected:** No regen. **Rationale:** `render_live_fragment` (`containers.rs:1639`) emits `<div data-live-fragment data-channel="…">` — no utility classes. The ROADMAP goal's conditional evaluates to no; speculative regen would add gratuitous CSS churn to the publish commit.

---

## Publish mechanics & gate (SC-4)

| Option | Description | Selected |
|--------|-------------|----------|
| World-state-dependent bump + operator gate | Workspace already 0.2.91; read crates.io at gate; publish 0.2.91 if remote is behind, else bump to max+1. Operator-gated with pre-publish checklist; CI-exact gate; push master via gh HTTPS. | ✓ |
| Fixed bump to a hardcoded version | Pre-decide the version now. | |

**Auto-selected:** World-state-dependent, operator-gated (258 D-11/D-16 pattern). **Rationale:** crates.io could not be read this session (network sandboxed); no-op auto-bumps from ferro-a2ui may have advanced remote independently. Resolve at the gate. On clean master (HEAD == master, 0/0), no feat-branch merge. Verify remote via gh API, not local origin refs.

---

## Commit hygiene

| Option | Description | Selected |
|--------|-------------|----------|
| Stage specific files only | Exclude stale `.vite/deps_temp_*` deletions, config.json workflow churn, phantom 158 path. | ✓ |
| `git add -A` | Stage everything in the working tree. | |

**Auto-selected:** Stage specific files only (258 D-18 + phantom-path memory). **Rationale:** 36 stale Vite-cache deletions + workflow-flag churn are unrelated to this phase and must not enter its commits.

---

## Claude's Discretion

- Exact docs placement within the extend-existing constraint; section ordering in components.md.
- generation_context section naming/structure; inline-vs-pointer detail balance.
- Whether a catalog-guidance gap is fixed in ferro-json-ui or ferro-mcp.
- Drift-guard test organization (combined vs. per-capability).
- Pre-publish checklist composition at the operator gate.

## Deferred Ideas

- Keyed live lists / collection reconciliation; delta-granular patches; multiple templates per projection (v17.0 non-goals).
- asset!() stable alias; auto-wiring fetched assets (261 deferrals).
- Milestone archival (v16.0–16.6 backlog) — separate operator action; v16.4 (244–249) queued.
