# Phase 253: MCP surface + docs + publish - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-04
**Phase:** 253-mcp-surface-docs-publish
**Mode:** `--auto` (recommended defaults selected without interactive prompts)
**Areas discussed:** design_lint tool contract, catalog/generation-context sourcing, docs chapter structure, publish mechanics & pre-publish gates

---

## design_lint tool contract

| Option | Description | Selected |
|--------|-------------|----------|
| Inline `spec_json` OR single file `path`, exactly one required | Matches spec §4 "inline JSON or path" and §7's both-modes test requirement; directory sweeps stay CLI-owned | ✓ |
| Inline only | Simpler, but fails spec §7 path-input test requirement | |
| Inline + path + recursive directory | Duplicates the CLI walker inside MCP; larger surface than the author→validate loop needs | |

**User's choice:** auto — inline OR single path (recommended)
**Notes:** Output reuses the 252 D-11 `Finding` serialization / CLI `--json` envelope. Parse failures return findings-envelope diagnostics (CLI WR-03 posture), never tool errors. In-process call — ferro-mcp already has ferro-json-ui with `projections`.

---

## Catalog & generation-context sourcing

| Option | Description | Selected |
|--------|-------------|----------|
| Derive from canonical enums + `design::rules()` registry, drift-guarded | Single source of truth; no hand-maintained parallel table (252 D-10 philosophy) | ✓ |
| Hand-authored guidance strings per component | Readable but creates a second control surface that drifts | |

**User's choice:** auto — derive + drift-guard (recommended)
**Notes:** generation_context summary kept compact (ids + one-liners, docs pointer): token v2 vocabulary, per-intent expectations grouped from the registry, canonical value lists. Component count mirror stays 47.

---

## Docs chapter structure

| Option | Description | Selected |
|--------|-------------|----------|
| Five pages (principles, tokens, variants, patterns, linting) + SUMMARY.md section | Spec §4 outline; cross-links themes.md and components.md migration table instead of duplicating | ✓ |
| Single long design-system.md page | Simpler nav but poor for agent retrieval and cross-linking | |

**User's choice:** auto — five pages (recommended)
**Notes:** Pattern catalog hand-written but drift-guarded (registry ids ↔ patterns.md, both directions); rationale text sourced from the registry `rationale` field. Neutral public-docs voice, no legacy framing.

---

## Publish mechanics & pre-publish gates

| Option | Description | Selected |
|--------|-------------|----------|
| One final bump after 253 code lands, push → CI publishes | Local master carries unpushed 0.2.81–0.2.83; crates.io at 0.2.80; single publish per milestone constraint | ✓ |
| Push current 0.2.83 first, then publish 253 separately | Two publishes — violates the friction-loop single-publish cadence | |

**User's choice:** auto — final bump + single push (recommended)
**Notes:** CI-exact gate incl. `--all-features` clippy/test, fmt --check, docs build, cargo-deny awareness. ferro-payments untouched (no bump). Operator-gated publish presents pre-publish UAT: 252 CLI output check + 251 pixel pass. Folded 252 deferred info-findings IN-01 (dead `Textarea` constant) and IN-02 (misleading clean message on zero files) as pre-publish cleanup.

---

## Claude's Discretion

- Field names / struct layout of new catalog & generation_context fields
- Per-finding metadata depth in the MCP response (consistency with CLI `--json` wins)
- Doc page ordering and titles within the five-section requirement
- Home of the docs↔registry drift test
- Final version number at publish-commit time

## Deferred Ideas

- gestiscilo Phase 232 adoption (consumer repo, gated on publish; handoff brief only)
- Milestone archival (`/gsd-complete-milestone` backlog for v16.0–v16.3, then v16.5)
- CSS-hygiene lint for dead generated utilities (carried from 252)
- OQ-3 `dot_colors` raw-Tailwind rule (FRICTION.md loop decides)
- Flaky `serve.rs` PGID test fix (standalone quick task)
