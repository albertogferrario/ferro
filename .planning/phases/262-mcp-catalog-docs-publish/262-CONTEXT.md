# Phase 262: MCP + catalog + docs + publish - Context

**Gathered:** 2026-07-26 (auto mode — recommended defaults selected, logged in 262-DISCUSSION-LOG.md)
**Status:** Ready for planning

<domain>
## Phase Boundary

Close the v17.0 Live Projection Surface milestone: complete the single-source
loop so an agent reading `ferro-mcp` alone can discover and compose the three
new capabilities shipped in Phases 259–261 — `LiveFragment` (live projection
binding), `#[memoize]` (request-scoped render-path fetch dedup), and `asset!()`
(one-line content-hashed embed) — then document them in `docs/src`, run the full
CI-exact gate, and perform the single operator-gated crates.io publish.
Requirement: LIVE-04.

**Killer-feature framing:** this phase completes the agent-authoring loop for the
v17.0 killer feature (`LiveFragment`). An agent that has never seen the ferro
source discovers `LiveFragment` through `json_ui_catalog`, learns WHEN and HOW to
bind a live fragment to a per-key projection snapshot through
`generation_context` (plus the `#[memoize]` render-dedup pattern and `asset!()`
embed), and composes a live, server-authoritative UI — all before human review.
The publish makes all three pinnable for consumers. `generation_context` quality
is where the polish budget goes; the catalog count and publish choreography are
commodity.

Out of scope: any renderer / element / macro / CLI behavior change (259/260/261
shipped them — only bug-fix-level touches); new components or lint rules; list
diffing / delta-granular patches / multiple-templates-per-projection (standing
v17.0 non-goals); the consumer adoption work itself (brief-only handoff, no
consumer-tree edits).

**World-state corrections found during scouting
(feedback_validate_scope_premises):**

1. **SC-1 is already satisfied in-tree.** Phase 260 Plan 04 bumped BOTH the
   canonical drift guard (`ferro-json-ui/src/catalog.rs:1303` —
   `BUILTIN_TYPES.len() == 53`) AND the ferro-mcp mirror
   (`ferro-mcp/src/tools/json_ui_catalog.rs:420` — `catalog.components.len() ==
   53`, message explicitly naming `LiveFragment`, plus the name assertion at
   `:478`). This went beyond 260's stated D-06 boundary ("Phase 262 owns only the
   ferro-mcp mirror count"), so SC-1 needs no re-implementation. This phase
   re-runs `builtin_types_count_drift_guard` + `test_all_components_present` and
   records the evidence — verification must NOT claim the count/mirror work as
   this phase's output.

2. **SC-2 and SC-3 are the substantive scope.** `generation_context.rs` has ZERO
   mentions of `LiveFragment`, `#[memoize]`, or `asset!()` today (grep count 0),
   and `docs/src` has zero coverage of any of the three. The agent-authoring
   guidance (SC-2) is the killer-feature deliverable of this phase; the docs
   (SC-3) are the human-facing companion.

3. **No `ferro-base.css` regen is needed.** The `LiveFragment` container renderer
   (`ferro-json-ui/src/render/containers.rs:1639` `render_live_fragment`) emits
   only `<div data-live-fragment data-channel="…">` — data attributes, ZERO
   Tailwind utility classes. The ROADMAP goal's conditional ("ferro-base.css
   regen IF the client runtime adds classes") evaluates to no. Verify with a grep
   during execution; skip the regen step unless a class actually surfaces.

4. **SC-4's version bar is stale relative to tree state.** The workspace is
   ALREADY at `0.2.91` (`Cargo.toml:47`, a pre-existing unpublished bump). The
   ROADMAP SC-4 bar is "published version exceeds 0.2.89" — 0.2.91 satisfies it
   IF crates.io is below 0.2.91 at gate time. crates.io state could not be read
   in this session (network sandboxed); the bump decision is world-state
   dependent and resolved at the gate (see D-11).

</domain>

<decisions>
## Implementation Decisions

### json_ui_catalog surface (SC-1) — verification-first
- **D-01:** `[auto]` SC-1 is pre-satisfied (world-state correction 1). Work =
  run the canonical drift guard (`catalog.rs:1303`
  `builtin_types_count_drift_guard`) + the ferro-mcp mirror
  (`json_ui_catalog.rs:405` `test_all_components_present`, asserting 53 AND all
  builtin names incl. `LiveFragment`) and record them as pre-existing evidence.
  No re-implementation, no count churn. If a hidden third cross-crate mirror
  surfaces, bring it into lockstep here (260 D-06 research flag) — but scouting
  found none beyond the ferro-mcp mirror.
- **D-02:** `[auto]` Audit the `json_ui_catalog` per-component output for
  `LiveFragment` (props schema for `projection`, `key`, child template). Fix only
  additive gaps found; existing output shape stays backward-compatible. Any
  static supplement carries a drift test tying it to the builtin
  registry/`BUILTIN_SPECS` (single-registry philosophy, 258 D-02 pattern).

### generation_context guidance (SC-2 — the killer feature)
- **D-03:** `[auto]` Content contract — add agent-authoring guidance for all
  three capabilities (SC-2 requires all three documented):
  - **(a) `LiveFragment`** — WHEN to use (bind a rendered fragment to ONE
    `ferro-projection` per-key snapshot for live, server-authoritative,
    no-WASM updates); the `projection` / `key` / child-template contract; that
    the server wraps the child in a `data-live-fragment` /
    `data-channel="projection.{name}.{key}"` container and pushes re-rendered
    HTML on delta; the ONE-binding-pattern limitation (per-key snapshot only —
    no list/collection reconciliation, an explicit non-goal); first-paint
    behavior with an absent snapshot (empty binding, container still rendered).
  - **(b) `#[memoize]`** — WHEN to mark an async fn / `#[service]` method
    memoized (request-scoped fetch dedup on the render path so N intents over
    one key issue one fetch); that it coalesces concurrent callers and is
    dropped with the request; that it COMPLEMENTS `eager_loading`/`BatchLoad`
    and is NOT cross-request caching (that stays `ferro-cache`).
  - **(c) `asset!()`** — the one-line embed (`asset!("path")` → content-hashed
    URL, content-type inferred from extension, lazy register-once, `&'static
    str`); that the app must still mount `ferro::bundle` serving for the URL to
    resolve; the opt-in `ferro assets fetch iconify|fontsource` author-time
    subcommand.
- **D-04:** `[auto]` Style mirrors 253 D-06 / 258 D-04 — compact: ids and
  one-liners with a pointer to `docs/src` for depth. `generation_context` is
  inline agent context, not a manual.
- **D-05:** `[auto]` Derive what is derivable (component name from the builtin
  registry; the data-attribute vocabulary from the runtime contract). All
  hand-written guidance is drift-guarded: a test asserts every component name,
  macro name, and data attribute mentioned in the live-fragment / memoize /
  asset guidance exists in its authoritative source (`BUILTIN_TYPES` /
  `BUILTIN_SPECS` for `LiveFragment`; the `#[memoize]` / `asset!` exports for
  the macros; the `data-live-fragment` / `data-channel` runtime contract in
  `ferro-json-ui/src/render/containers.rs` + `runtime/live_fragment.rs`) — the
  258 D-05 / 253 D-09 pattern applied to the three v17.0 capabilities.

### docs/src coverage (SC-3)
- **D-06:** `[auto]` Extend existing pages first over inventing new ones (258
  D-08 practice), placement is planner's call within this constraint:
  - **`LiveFragment`** → a component section in `docs/src/json-ui/components.md`
    (per-component format anchor; it is a builtin) with a props table and ≥1
    usage example, plus a live-update behavior note in
    `docs/src/json-ui/runtime-primitives.md` (the no-WASM client-runtime page).
  - **`asset!()` + `ferro assets fetch`** → `docs/src/features/ferro-assets.md`
    (the existing Asset Pipeline page).
  - **`#[memoize]`** → the render-path / projection docs
    (`docs/src/features/projections.md` or a short dedicated section) — framed
    as request-scoped render-dedup, complementing eager loading.
  - Cross-link, never duplicate. If a dedicated page is genuinely warranted,
    wire it into `docs/src/SUMMARY.md`. Every capability gets ≥1 usage example.
- **D-07:** `[auto]` mdBook docs build exits 0 (SC-3 gate). Neutral product
  documentation voice; no internal-strategy framing; no version-vs-version or
  legacy/v1-vs-v2 comparison framing (feedback_json_ui_naming). Domain-neutral —
  commerce/sample naming only in examples explicitly framed as samples.

### CSS regeneration (conditional)
- **D-08:** `[auto]` NO `ferro-base.css` regen (world-state correction 3): the
  `LiveFragment` container emits only `data-*` attributes, no Tailwind utility
  classes. During execution, grep `render_live_fragment` +
  `runtime/live_fragment.rs` for class strings to confirm; run the regen script
  ONLY if a new utility class is found. Do not run it speculatively (avoids
  gratuitous CSS churn in the publish commit).

### Publish + gate (SC-4)
- **D-09:** `[auto]` CI-exact gate before the publish push:
  `cargo fmt --all -- --check`,
  `cargo clippy --all --all-targets --all-features -- -D warnings`,
  `cargo test --all-features`, plus `cargo doc` with `-D warnings` and
  cargo-deny awareness — CI's matrix is wider than the three-command gate
  (feedback_ci_matrix_wider_than_local_gate). Re-run fmt after ANY hand-edit
  (recurrent publish-blocker). Serialize CPU-heavy runs one at a time
  (feedback_one_cpu_op_at_a_time); check disk and clean `target/` before the
  full `--all-features` test gate (recurrent ENOSPC). Schema-export churn is
  discarded unless a real diff appears.
- **D-10:** `[auto]` Operator-gated publish (236/253/258 practice): present a
  pre-publish checklist at the gate (gate results, resolved version, what ships).
  The publish command is not run without the operator's go. Post-publish: verify
  via crates.io / gh API, never local `origin/*` refs (recurrent local-refs lie);
  run `git update-ref refs/remotes/origin/master HEAD` after a verified push.
- **D-11:** `[auto]` Version resolution is world-state-dependent (world-state
  correction 4). Workspace is at `0.2.91` (already bumped, unpublished). At the
  gate, read crates.io ferro-rs current version (gh API / `cargo search` / curl):
  - If crates.io < 0.2.91 → publish 0.2.91 as-is (a no-op auto-bump may have
    already advanced remote; that is fine — SC-4 needs published > 0.2.89, met).
  - If crates.io ≥ 0.2.91 (a remote no-op auto-bump landed there) → bump the
    workspace to `crates.io_max + 1` patch so `cargo publish` has a new version.
  Single publish commit, manual bump so CI publishes directly with no double-bump
  (established 0.2.75/0.2.85/0.2.89 practice). No mid-phase publishes.
- **D-12:** `[auto]` `ferro-payments` is at `0.1.6` (`ferro-payments/Cargo.toml:3`,
  shipped in the 258 close). At the gate, check crates.io ferro-payments: if it
  already carries 0.1.6, ferro-payments does NOT re-publish this cycle (cargo
  skips already-published); if crates.io is behind, it rides the publish and
  verification confirms both. No ferro-payments code changes in v17.0 — no forced
  bump.
- **D-13:** `[auto]` No new crates in v17.0 (memoize in framework+ferro-macros,
  live fragment in ferro-json-ui+ferro-projection, asset macro over
  ferro-assets/ferro-bundle) — so NO `publish.yml` wave changes and no
  publish-new token bootstrap. EXCEPTION TO VERIFY: Phase 261 D-06 moved
  `ferro-bundle` to a Wave 1a leaf (decoupled from ferro-rs) and `framework`
  gained a `ferro-bundle` dependency. Confirm `publish.yml` already reflects that
  wave move (261 shipped it); if the wave ordering is still wrong, fix it here as
  a publish-correctness touch. `ferro-a2ui` stays `publish = false` and out of
  `publish.yml`.
- **D-14:** `[auto]` Branch topology: currently ON `master`, and `master ==
  HEAD` (0 ahead / 0 behind) — all 259/260/261 work is already on local master;
  no feat-branch merge needed (simpler than 258's topology). Assert `HEAD`=master
  from the main repo root before any ref move (feedback_worktree_merge_cwd_trap),
  then push master via the gh HTTPS credential helper (SSH is denied for this
  repo). Verify remote is an ancestor via gh API before pushing.

### Commit hygiene
- **D-15:** `[auto]` Stage specific files only. Exclude from phase commits: the
  stale `app/frontend/node_modules/.vite/deps_temp_*` deletions (Vite cache
  artifacts, 36 tracked-path deletions in the working tree), the
  `.planning/config.json` workflow-flag churn, and any phantom
  `planning/phases/158-…` deletion (258 D-18 practice + phantom-path memory).

### Claude's Discretion
- Exact docs placement within the D-06 constraint (which existing page each
  capability extends; whether `#[memoize]` gets its own short page) and section
  ordering inside components.md.
- Exact `generation_context` section naming/structure for the three-capability
  guidance and how much detail is inline vs. deferred to the docs pointer.
- Whether any `json_ui_catalog` guidance gap found under D-02 is fixed in
  ferro-json-ui or ferro-mcp (whichever owns the derivation point).
- Test organization for the D-05 drift guards (one combined test vs. per-capability).
- Pre-publish checklist composition details at the D-10 gate.

### Folded Todos
None — `todo match-phase 262` returned 0 matches (checked in cross_reference_todos).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase definition + requirement
- `.planning/ROADMAP.md` §"Phase 262: MCP + catalog + docs + publish"
  (~L4195–4210) — goal, Depends on, SC 1–4 (with world-state corrections above
  applied), and the v17.0 "Architectural constraints" block (~L4096–4108:
  no-new-crates, server-authoritative, single-publish-at-262).
- Requirement **LIVE-04** (`ferro-mcp` catalog + generation_context + docs +
  publish). v17.0 requirements are defined INLINE in `.planning/ROADMAP.md`
  (Requirement → Phase Mapping ~L4212–4219) — they are **not** in
  `.planning/REQUIREMENTS.md` (confirmed: grep count 0).

### Design spec (authoritative)
- `docs/superpowers/specs/2026-07-21-live-projection-surface-design.md` — the
  three-capability contract this phase documents; §Non-Goals (no list diffing,
  no client reactive state) and §"Honest limitations" are the guardrails the
  guidance/docs must state faithfully.

### Prior phases this phase surfaces + publishes
- `.planning/phases/259-request-scoped-memoization/259-CONTEXT.md` —
  `#[memoize]` semantics (request-scoped, coalescing, complements eager loading)
  the generation_context/docs must describe accurately.
- `.planning/phases/260-live-reactive-fragment/260-CONTEXT.md` — D-06 (catalog
  membership boundary; the mirror-count handoff this phase discharges — now
  pre-satisfied), D-02/D-03 (the `data-live-fragment`/`data-channel` container +
  `fragment` event contract the guidance describes), D-04 (absent-snapshot
  first paint), the one-binding-pattern non-goal.
- `.planning/phases/261-asset-ergonomics/261-CONTEXT.md` — D-01..D-05 (`asset!()`
  embedding, lazy register-once, `&'static str`, ext→MIME), D-06 (the
  `ferro::bundle` re-export + ferro-bundle Wave 1a move — the publish.yml
  verification in D-13), D-07..D-09 (`ferro assets fetch` command shape).

### Sibling publish-phase pattern (mirror deliberately)
- `.planning/phases/258-mcp-surface-docs-publish/258-CONTEXT.md` — the one-
  milestone-prior closeout this phase mirrors: D-01 (verification-first
  pre-satisfied catalog), D-03/D-04/D-05 (generation_context content + compact
  style + drift guard), D-07..D-10 (docs placement + mdBook gate + neutral
  voice), D-11..D-18 (publish mechanics, operator gate, consumer handoff, commit
  hygiene). Same phase shape, one milestone later.
- `.planning/phases/253-mcp-surface-docs-publish/253-CONTEXT.md` — the original
  of this closeout shape (design-system publish); the operator-gated publish
  choreography executed successfully twice already (253-05, 258).

### Source anchors (current on `master`)
- `ferro-mcp/src/tools/generation_context.rs` — the SC-2 target (zero v17.0
  content today; the section-style anchor is the existing design-system / POS
  guidance added at 253/258).
- `ferro-mcp/src/tools/json_ui_catalog.rs:405–478` — mirror `test_all_components_present`
  (already 53 with all names incl. `LiveFragment`; D-01 evidence).
- `ferro-json-ui/src/catalog.rs:1294–1303` — canonical
  `builtin_types_count_drift_guard` (already 53; D-01 evidence);
  `BUILTIN_SPECS.len() == BUILTIN_TYPES.len()` relational guard at `:1503`.
- `ferro-json-ui/src/render/mod.rs:90,229` — `LiveFragment` in `BUILTIN_TYPES` +
  dispatch to `render_live_fragment`.
- `ferro-json-ui/src/render/containers.rs:1639` — `render_live_fragment`
  (the `data-live-fragment` / `data-channel` container; D-08 CSS-regen evidence).
- `ferro-json-ui/src/runtime/live_fragment.rs` — the no-WASM `setupLiveFragments`
  client runtime (docs/runtime-primitives source).
- `docs/src/json-ui/components.md` (per-component format anchor),
  `docs/src/json-ui/runtime-primitives.md`, `docs/src/features/ferro-assets.md`
  (Asset Pipeline), `docs/src/features/projections.md` (Service Projections),
  `docs/src/SUMMARY.md:63–81` (json-ui + design-system TOC).
- `.github/workflows/publish.yml` — publish waves (verify the 261 ferro-bundle
  Wave 1a move landed; ferro-a2ui absence is intentional).
- `Cargo.toml:47` — workspace version `0.2.91` (D-11 base);
  `ferro-payments/Cargo.toml:3` — `0.1.6` (D-12).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- The 253/258 `generation_context` guidance sections + their drift-guard tests —
  the exact structural pattern (compact ids + one-liners + docs pointer +
  registry-tied tests) the three-capability guidance extends.
- The canonical `builtin_types_count_drift_guard` + the ferro-mcp
  `test_all_components_present` mirror — already green at 53 with `LiveFragment`
  (SC-1 evidence; re-run, do not re-implement).
- The `Tile` / POS component sections in `components.md` — the per-component docs
  format (description, props table, data-attribute notes, usage example).
- The 253-05 / 258 publish plan artifacts — the operator-gated publish
  choreography executed successfully twice already.

### Established Patterns
- Verification-first for pre-satisfied SCs: record evidence, never re-do or claim
  credit (257/258 world-state practice).
- Drift-guard every hand-written surface that mirrors a registry.
- CI-exact gate incl. `--all-features` + docs build; fmt after any hand-edit;
  serialize CPU-heavy runs; disk check before full test runs.
- Operator-gated publish with post-publish crates.io/gh-API verification; never
  trust local `origin/*` refs.

### Integration Points
- `ferro-mcp/src/tools/generation_context.rs` — three-capability guidance (SC-2).
- `ferro-mcp/src/tools/json_ui_catalog.rs` + `ferro-json-ui` catalog — D-02 audit
  surface (already at 53).
- `docs/src/json-ui/*` + `docs/src/features/*` (+ `SUMMARY.md` if a page is
  added) — SC-3.
- `Cargo.toml` workspace version — the single publish commit (D-11).
- `.github/workflows/publish.yml` — verify the 261 ferro-bundle wave move (D-13).
- Consumers: whoever pins the v17.0 primitives (`LiveFragment` / `#[memoize]` /
  `asset!()`) after publish — brief-only handoff, no consumer-tree edits.

</code_context>

<specifics>
## Specific Ideas

- The phase's one-sentence user story: "an agent that has never seen the ferro
  source binds a `LiveFragment` to a projection key, memoizes the render fetch,
  and embeds an asset — from MCP context alone." `generation_context` quality is
  where the polish budget goes; the catalog count is already done and the publish
  is commodity.
- The compressive claim to preserve in all prose: ferro already carried both
  halves of a live-rendering story (per-key snapshot + delta broadcast, and
  server-side rendering); v17.0 joined them into ONE declarative element with no
  WASM, no new crates, and one new builtin — plus two supporting ergonomics.
- Mirror 258's structure deliberately — same closeout shape, same operator
  publish gate, one milestone later. Where 258 documented POS component sections +
  the register projection surface, 262 documents the live fragment + the two
  authoring-ergonomic macros.

</specifics>

<deferred>
## Deferred Ideas

- **Keyed live lists / collection reconciliation** — v17.0 spec Future direction;
  a second binding pattern, explicit non-goal. Docs state it as a non-goal, do
  not build it.
- **Delta-granular fragment patches** (patch instead of full re-render) — spec
  Future direction; whole-fragment re-render is the accepted v17.0 cost.
- **Multiple distinct fragment templates over the same projection** — v17.0 is
  one canonical renderer per projection name (260 D-01).
- **Macro-emitted stable alias** (`asset!("path", alias = "/app.js")`) — 261
  deferral; add only on consumer need.
- **Auto-wiring fetched assets into `asset!()` calls / route generation** — 261
  deferral; fetch downloads + writes only.
- **v16.6 / earlier milestone archival** (`/gsd-complete-milestone`) — the
  archive backlog holds v16.0/16.1/16.2/16.3/16.5/16.6; separate operator action
  after this phase. v16.4 Work Distribution (244–249) remains queued.

### Reviewed Todos (not folded)
None — no pending todos matched this phase.

</deferred>

---

*Phase: 262-mcp-catalog-docs-publish*
*Context gathered: 2026-07-26*
