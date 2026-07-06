# Phase 227: Documentation Audit and Update for v0.2.61 - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-15
**Phase:** 227-documentation-audit-and-update-for-v0-2-61
**Mode:** `--auto` (recommended defaults auto-selected)
**Areas discussed:** Audit breadth & fix threshold, Verification method, Install-flow consistency, CHANGELOG decision

---

## Audit breadth & fix threshold

| Option | Description | Selected |
|--------|-------------|----------|
| Audit every page, factual fixes only | Sweep all `docs/src/`; correct facts, no prose rewrites | ✓ |
| Audit only flagged-stale candidates | Faster, but risks missing drift on un-flagged pages | |
| Full rewrite/restructure | Out of scope per ROADMAP "not a rewrite" | |

**Choice:** Audit every page; fix factual inaccuracies only (D-01, D-02).
**Notes:** Install page already known-good — confirm, don't re-touch.

## Verification method

| Option | Description | Selected |
|--------|-------------|----------|
| Verify against live CLI + scaffold source | Check commands vs `ferro-cli/src/commands/`, config vs `Cargo.toml.tpl` | ✓ |
| Read-only review | Faster but cannot catch command/flag drift | |

**Choice:** Verify against ground-truth source (D-03, D-04).
**Notes:** Scout confirmed scaffold template already uses `runtime-tokio-rustls`; TLS sweep is a checkpoint, not a large fix.

## Install-flow consistency

| Option | Description | Selected |
|--------|-------------|----------|
| Brew-first everywhere + neutralize version pins | Match installation.md; reorder reference/cli.md; replace 0.2.33 pin | ✓ |
| Leave per-page install ordering as-is | Inconsistent with the new recommended path | |

**Choice:** Brew lead method consistently; replace stale hard pins with neutral/placeholder phrasing (D-05, D-06).
**Notes:** Concrete found discrepancies — `reference/cli.md` leads with cargo; `frontend-types.md:97` pins `0.2.33`.

## CHANGELOG decision

| Option | Description | Selected |
|--------|-------------|----------|
| Defer — no changelog infra this phase | Keeps phase to factual-accuracy boundary | ✓ |
| Add 0.2.60/0.2.61 CHANGELOG now | Introduces new doc infrastructure (rewrite-adjacent) | |

**Choice:** Defer CHANGELOG (recorded in Deferred Ideas).
**Notes:** No changelog exists today; creating one is infrastructure, not an audit fix. Folds into a follow-up / Phase 228 if wanted.

## Claude's Discretion

- Exact per-page edits determined during execution from verification results.
- Sweep ordering (getting-started + reference/cli first recommended).

## Deferred Ideas

- CHANGELOG for 0.2.60/0.2.61 — deferred (see above).
- README / scaffold README / tap README / install-script messaging — Phase 228.
