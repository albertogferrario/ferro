# Phase 252: Design module + lint + CLI - Context

**Gathered:** 2026-07-03 (auto mode — recommended defaults selected, logged in 252-DISCUSSION-LOG.md)
**Status:** Ready for planning

<domain>
## Phase Boundary

Codify the composition patterns as a machine-readable, testable rule set:
`Spec` gains an optional `design` field (`intent` + `allow`), a pure
`design::lint(&Spec)` engine in a new `ferro-json-ui/src/design/` module
implements the intent-keyed rules from the anchor spec, and
`ferro design:lint [path] [--json] [--deny]` surfaces findings from the command
line. Requirements: DS-05, DS-06.

The `design_lint` MCP tool, `json_ui_catalog`/`generation_context` extensions,
`docs/src/design-system/` chapter, and the crates.io publish are **Phase 253**.
No new crate (rules live in ferro-json-ui — spec's non-goal §8). Lint is
diagnostics-only: it never affects rendering or spec validation.

**Killer feature framing:** this is the phase where the design system becomes
*enforceable at the agent-authoring boundary* — the composition patterns stop
being prose and become a versioned rule set keyed by the seven projection
intents (the framework's core abstraction), checkable before any human review.
The intent-keying is the point: page archetypes ARE the projection intents, no
parallel page-type vocabulary.

</domain>

<decisions>
## Implementation Decisions

### Locked by the anchor spec (do not re-derive)
- **D-01:** Wire shape is fixed: `"design": {"intent": "browse", "allow":
  ["prefer-data-table"]}` — one optional serde-default field on `Spec`, absent
  from serialized output when unset (`skip_serializing_if`).
- **D-02:** `intent` values are the seven projection intents (`browse`, `focus`,
  `collect`, `process`, `summarize`, `analyze`, `track`). Invalid intent values
  and unknown `allow` ids are reported as **findings, never errors** — spec
  parse and rendering are unaffected by anything in `design`.
- **D-03:** The rule set is the anchor spec §3 table (10 rules): `page-header`,
  `prefer-data-table`, `list-empty-state`, `row-actions-grouped`,
  `process-kanban`, `create-separate-page`, `breadcrumb-on-subpages`,
  `form-default-values`, `destructive-confirmation`, `card-actions-in-menu` —
  each with the intents column and rule text as specified, each shipping a
  violating + conforming unit-test pair.
- **D-04:** `Severity` is `Info | Warning` only. `Finding` carries `rule`,
  `element_id: Option<String>`, `severity`, `message`, `suggestion`.
- **D-05:** CLI contract: `ferro design:lint [path] [--json] [--deny]`, default
  path `src/views`, recursive over `*.json`, human-readable findings grouped by
  file, `--json` for machine consumption, exit 0 always unless `--deny` (CI
  mode: non-zero when any warning-level finding exists). Info findings never
  fail `--deny`.
- **D-06:** Undeclared intent is inferred from spec content (DataTable → browse,
  KanbanBoard → process, root-dominant Form → collect, StatCard cluster →
  summarize, …) and the inference is reported as an info-level finding.

### Intent plumbing (feature-flag reality)
- **D-07:** `Spec.design` and the whole `design` module compile **without** the
  `projections` feature — `ferro-projections` is an optional dependency of
  ferro-json-ui and ferro-cli consumes default features. The wire type is
  string-typed: `DesignMeta { intent: Option<String>, allow: Vec<String> }`.
  A string intent can never fail spec parse, making D-02 structural.
- **D-08:** The seven archetype labels live in the `design` module as the
  canonical `&'static str` set (or tiny internal enum — planner's call). A
  **drift test** asserts this set equals `ferro_projections::Intent::label()`
  for the seven known variants, so the "archetypes ARE the projection intents"
  invariant is guarded, not aspirational. Gate the test behind the
  `projections` feature or a dev-dependency — CI runs `--all-features`, so
  either enforces it.
- **D-09:** A declared intent outside the seven (including would-be
  `Intent::Custom` strings) produces a **warning** finding (`unknown intent`)
  and lint falls back to the inference path for intent-keyed rules.
  All-intents rules always run regardless.

### Rule engine architecture
- **D-10:** Rules are a static registry — `DesignRule { id, title, rationale,
  intents, check }` with a public iterator (`design::rules()`). The metadata is
  machine-readable and public **because Phase 253 derives the pattern-catalog
  docs and MCP design guidance from this same registry** — one source of truth,
  no prose duplication.
- **D-11:** `Finding` (and `Severity`) derive `Serialize` (+ `JsonSchema`,
  matching crate conventions) so the CLI `--json` output and Phase 253's
  `design_lint` MCP tool share one serialization. The `--json` shape is the
  stable contract gestiscilo's CI (consumer Phase 232) will consume.
- **D-12:** `lint(&Spec) -> Vec<Finding>` is pure and static: no I/O, no data
  resolution — it runs on the raw spec **before** `$each`/`$if` expansion, and
  rules must tolerate `$data` bindings in props (the `form-default-values` rule
  is defined over `$data` paths explicitly).
- **D-13:** `allow` exempts rule ids page-wide, including the `declare-intent`
  inference finding. Unknown `allow` ids produce a **warning** finding — a
  typo'd escape hatch that silently disables nothing must fail CI under
  `--deny`, not pass it.

### Rule specifics (resolved against the codebase)
- **D-14:** `page-header` "dashboard-family layout" = builtin layouts
  `"dashboard"` and `"app"` (the registry ships `dashboard`, `app`, `auth`);
  `auth`, custom layouts, and layout-less specs do not trigger the rule.
- **D-15:** `destructive-confirmation`: an action is "styled destructive" when
  its element/item carries the canonical `variant: destructive` (Button,
  ActionGroup items) or equivalent destructive styling surfaced by the Phase
  251 vocabulary; conformance = the `Action.confirm` field (`ConfirmDialog`,
  `action.rs:148`) is present. Covers both element-level `action` and
  props-embedded actions (row_actions, buttons, ActionGroup items).
- **D-16 (folded Phase 251 handoff):** stale-prop / migration-hygiene
  diagnostics join the rule set as an additional all-intents rule (the "~10" is
  approximate by spec wording): retired prop names and values from the Phase
  251 migration table (D-17 there — e.g. Alert `variant`, `badge_variant_key`,
  `size: xs`) that serde silently ignores are flagged with the old → new
  suggestion. **Research directive:** locate the WR-01 retired-prop lint that
  Phase 251's REVIEW-FIX added (it exists; element-level typed `action` fields
  escape it — see 251-VERIFICATION.md finding 1) and decide single-home
  placement — extend it or absorb it into `design::lint`, never two parallel
  stale-prop control surfaces (`feedback_no_duplicate_control_surface`).

### Tests & sample-app gate
- **D-17:** The sample `app/` views lint clean, enforced by a test in the `app`
  crate that walks `app/src/views/*.json` (currently `login.json`,
  `login_confirm.json`, `pagamenti.json`). The app views **declare**
  `design.intent` (dogfooding the field per success criterion 1) so the gate
  asserts **zero findings** — no inference info noise.
- **D-18:** Per-rule violating + conforming test pairs live in the design
  module's tests. Inference heuristics get their own coverage (each inference
  branch: one spec that triggers it).
- **D-19:** No ferro-mcp changes this phase (Phase 253 owns the MCP surface).
  Component count stays 47 — the documented ferro-mcp mirror is untouched; grep
  for mirrored assertions before the gate anyway (established practice).
- **D-20:** CI-exact gate before commit: `cargo fmt --all -- --check`,
  `cargo clippy --all --all-targets --all-features -- -D warnings`,
  `cargo test --all-features`, plus the Docs build (`cargo doc` clean) since
  the new public module ships rustdoc.

### Claude's Discretion
- Rule engine internals: fn pointers vs trait objects, one file per rule vs
  grouped modules under `design/`, traversal helpers over the flat element map.
- Inference heuristic details beyond the spec's named signals (tie-breaking,
  "StatCard cluster" threshold, fallback when nothing matches → intent stays
  undeclared, only all-intents rules run, info finding suggests declaring).
- Human-readable CLI output formatting (grouping, colors, summary counts).
- CLI file-discovery edge semantics: recommended — JSON files without the
  `"$schema": "ferro-json-ui/v2"` marker are skipped silently; files with the
  marker that fail `Spec` parse are reported as warning-level file diagnostics.
- OQ-3 `dot_colors` raw-Tailwind lint (Phase 251 handoff): optional bonus rule
  if the engine makes it cheap; otherwise leave for the gestiscilo FRICTION.md
  loop to prioritize.
- Exact `--json` envelope (flat findings array with `file` field vs grouped by
  file) — pick one, document it in the command's `--help`, keep it stable.

### Folded Todos
None — no pending todos matched this phase.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design spec (anchor — source of truth for this milestone)
- `docs/superpowers/specs/2026-07-03-json-ui-design-system-design.md` §3
  (pattern layer: Spec extension, rule engine sketch, the 10-rule table), §4
  (ferro-cli surface), §7 (testing & error handling), §8 (non-goals: no new
  crate, no hard validation).

### Prior phase decisions (vocabulary this phase's rules reference)
- `.planning/phases/251-component-variant-discipline-interactive-state-pass/251-CONTEXT.md`
  — D-01 canonical enums, D-17 migration table location (public docs), D-19
  drift guard.
- `.planning/phases/251-component-variant-discipline-interactive-state-pass/251-03-SUMMARY.md`
  §Handoff Notes — stale-prop posture (serde-ignored retired prop names;
  "Phase 252's design lint is the natural home").
- `.planning/phases/251-component-variant-discipline-interactive-state-pass/251-VERIFICATION.md`
  finding 1 — element-level `action` escapes the WR-01 retired-prop lint (the
  D-16 gap to close).

### Planning
- `.planning/ROADMAP.md` — v16.5 section, Phase 252 details (goal, success
  criteria 1–6).
- `.planning/REQUIREMENTS.md` — DS-05, DS-06 (v16.5 section).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-json-ui/src/spec.rs:74` — `Spec` struct (schema/root/elements/title/
  layout/data); the `design` field slots in with the established
  `#[serde(default, skip_serializing_if = ...)]` pattern.
- `ferro-projections/src/intent.rs:18` — `Intent` enum, 7 known variants +
  `#[serde(untagged)] Custom(String)`; `Intent::label()` returns the
  snake_case strings the D-08 drift test asserts against.
- `ferro-json-ui/src/action.rs:148` — `Action.confirm: Option<ConfirmDialog>`
  (with `tone: Tone` since Phase 251) — the conformance signal for
  `destructive-confirmation`.
- `ferro-json-ui/src/catalog.rs:689` — `validate(&self, spec)` exists and is a
  SEPARATE surface: catalog validation stays hard errors, design lint stays
  diagnostics. Do not blur them.
- `ferro-cli/src/main.rs:345,358` — `#[command(name = "db:migrate")]` /
  `"json-ui:schema"` — the colon-namespaced clap pattern `design:lint` follows;
  command impls live in `ferro-cli/src/commands/` (one file per command,
  e.g. `design_lint.rs`).
- `app/src/views/*.json` — the three sample specs for the D-17 lint-clean gate.

### Established Patterns
- ferro-json-ui `projections` feature is **non-default** (`Cargo.toml:14`);
  ferro-cli depends on ferro-json-ui with default features — the D-07
  feature-independence constraint comes from here.
- Enums: `#[serde(rename_all = "snake_case")]`, `#[serde(untagged)]` escape
  hatches, schemars derives — follow for `Severity`/finding types.
- Builtin layout registry ships `dashboard`, `app`, `auth`
  (`layout.rs:654-666`) — grounds D-14.
- Workspace gate (CI-exact): fmt, clippy `--all --all-targets --all-features
  -D warnings`, test `--all-features`, docs build.

### Integration Points
- `Spec` deserialization sites (loader, handlers, tests) — the new field is
  additive/optional, so no call-site changes expected; serialization
  round-trip tests must cover present + absent `design`.
- `ferro-cli` command enum in `main.rs` + `commands/mod.rs` registration.
- Phase 253 consumes: `design::rules()` metadata (docs + MCP guidance),
  `Finding` serialization (MCP tool output), the CLI `--json` shape.
- gestiscilo Phase 232 (consumer repo) consumes: the CLI `--deny` CI gate and
  the `--json` output — treat both as public contracts from day one.

</code_context>

<specifics>
## Specific Ideas

- "Structural guarantees over one-off fixes": the D-08 drift test and D-10
  single-registry design make vocabulary drift and docs drift compile-visible,
  mirroring Phase 251's D-19 enum-set guard philosophy.
- The rules encode the user's own dashboard-page patterns (global CLAUDE.md:
  PageHeader on every page, DataTable never raw Table, EmptyState with CTA,
  kebab-menu actions, separate create pages, breadcrumbs, form default_value
  discipline, destructive confirmation) — the lint is those conventions made
  machine-checkable.
- Findings should read like a good reviewer: `message` states what's wrong,
  `suggestion` states the concrete fix ("wrap the 3 row buttons in an
  ActionGroup"), `rationale` on the rule explains why the pattern exists.

</specifics>

<deferred>
## Deferred Ideas

- `design_lint` MCP tool, `json_ui_catalog` variant vocabulary,
  `generation_context` design-system summary, `docs/src/design-system/`
  chapter, crates.io publish — **Phase 253** (by design; single publish at
  milestone end per friction-loop release cadence).
- CSS-hygiene lint (dead utility definitions leaking into `ferro-base.css`
  from negative test assertions — Phase 251 Plan 04 observation): different
  artifact class (generated CSS, not specs); revisit only if a CSS-hygiene
  rule category ever materializes. Not this phase.
- OQ-3 `dot_colors` raw-Tailwind rule — discretionary here (see decisions);
  if skipped, the gestiscilo FRICTION.md loop decides its priority.
- gestiscilo reference-case adoption (68-spec sweep, `--deny` CI gate,
  FRICTION.md) — gestiscilo Phase 232, separate repo, gated on Phase 253
  publish.

</deferred>

---

*Phase: 252-design-module-lint-cli*
*Context gathered: 2026-07-03*
