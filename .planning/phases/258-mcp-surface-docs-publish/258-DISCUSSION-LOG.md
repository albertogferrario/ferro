# Phase 258: MCP Surface + Docs + Publish - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-06
**Phase:** 258-mcp-surface-docs-publish
**Mode:** `--auto` — all gray areas selected; recommended option chosen per question
**Areas discussed:** json_ui_catalog surface, generation_context register guidance, docs/src scope & placement, publish mechanics & branch topology

---

## json_ui_catalog surface (POS-12)

| Option | Description | Selected |
|--------|-------------|----------|
| Verification-first | SC-1 is already satisfied in-tree (256 bumped both counts to 52; mirror asserts all five names). Record evidence, audit derived per-component guidance, fix only additive gaps. | ✓ |
| Re-implement per SC text | Treat SC-1 as open work and re-touch both count sites. | |

**Auto-selected:** Verification-first — re-doing shipped work would churn drift guards for nothing; established 257 world-state practice.

---

## generation_context register guidance (POS-12)

| Option | Description | Selected |
|--------|-------------|----------|
| Compact derived section (253 D-06 pattern) | Ids + one-liners covering all six SC-2 items; derive rule ids/rationale from `design::rules()`; drift-guard hand-written mentions against their registries; point to docs/src for depth. | ✓ |
| Full inline manual | Embed complete register composition tutorial in generation_context. | |
| Docs-pointer only | Minimal mention, rely on docs/src. | |

**Auto-selected:** Compact derived section — generation_context is inline agent context; the 253 pattern already proved the shape and the drift-guard style.
**Follow-up (auto):** Numpad documented as author-composable (not in v1 template) — discharges 257 D-07 handoff. → CONTEXT D-06.

---

## docs/src scope & placement (POS-12)

| Option | Description | Selected |
|--------|-------------|----------|
| Extend existing pages | Five component sections in components.md (Tile section is the format anchor); register projection surface in layouts.md/spec-construction.md; cross-link, never duplicate; new page only if warranted (then wire SUMMARY.md). | ✓ |
| New register chapter | Dedicated docs/src/json-ui/register.md page holding everything. | |

**Auto-selected:** Extend existing pages first — matches 253 D-08 practice and the existing docs information architecture; planner may still add one page if content genuinely doesn't fit.
**Notes:** Must also cover the 257-VERIFICATION handoff (`register_template()`, `each()`, `fill_viewport()`), interaction model, `disable_on_submit` + idempotency pointer. mdBook build exits 0. Neutral voice.

---

## Publish mechanics & branch topology (POS-13)

| Option | Description | Selected |
|--------|-------------|----------|
| Single manual bump 0.2.88→0.2.89, ff master, operator-gated push | World-state: crates.io already at 0.2.88 (a2ui no-op auto-bumps); all 256/257 work on feat/billable-return-url-seam (remote master is ancestor of HEAD). Land 258 on branch, ff local master at repo root with HEAD=master asserted, CI-exact gate + docs build, one bump commit, operator gate, push via gh HTTPS helper, verify via crates.io/gh API. | ✓ |
| Publish per roadmap SC text | Bump relative to 0.2.86 and push without topology reconciliation. | |

**Auto-selected:** The first — the roadmap SC predates the 0.2.87/0.2.88 no-op releases and the branch topology; 253 D-11..D-16 choreography reused.
**Notes:** ferro-payments 0.1.6 rider ships with this push (crates.io at 0.1.5; bump already committed) — publish verification covers both crates. ferro-a2ui stays publish=false. Stray untracked planning files from other phases stay out of 258 commits.

---

## Claude's Discretion

- Exact docs placement within the extend-existing-pages constraint; components.md section ordering.
- generation_context section naming/structure; rationale verbatim vs. trimmed.
- Where a D-02 catalog-guidance gap (if any) gets fixed.
- Pre-publish checklist composition; drift-guard test organization.

## Deferred Ideas

- Numpad in the register template; register template knobs; FilterTabs↔TileGrid `data-filter-for` pairing; category strip derivation hint; barcode wedge / payment flow / receipts / shift close; milestone archival backlog (`/gsd-complete-milestone`).
