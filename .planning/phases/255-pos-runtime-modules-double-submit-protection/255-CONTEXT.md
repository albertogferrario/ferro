# Phase 255: POS Runtime Modules + Double-Submit Protection - Context

**Gathered:** 2026-07-05 (auto mode — recommended defaults selected, logged in 255-DISCUSSION-LOG.md)
**Status:** Ready for planning

<domain>
## Phase Boundary

The POS runtime modules are in the JS bundle with a stable data-attribute
contract BEFORE any Phase 256 render function targets it, and cart-mutation
forms are double-submit protected. Concretely:

- `setupNumpad()` — tap-surface keypad runtime writing to a declared target
  hidden field, dispatching `input` events (form-guard compatible)
- `setupPosFilter()` — category/search tile-visibility filtering, fully
  client-side, via `data-product-categories` token matching + text search
- `runtime/mod.rs` wiring: bundle concatenation, `ferroRuntime()` dispatcher
  entries, both drift-list tests extended
- `data-disable-on-submit` double-submit guard + documented idempotency-key
  pattern on the existing `framework::write` idempotency hook (POS-08)

NOT this phase: **NO cart-state JS** — quantity accumulation stays on the
existing `product_tiles.rs` hidden-input contract (`data-qty-input` /
`data-qty-display`), submitted as a single confirm POST. The live CartRuntime
(per-tap cart panel updates, client-computed totals) is DEFERRED
(REQUIREMENTS.md Future Requirements). No new render functions for the five
POS builtins (Phase 256); no BUILTIN_TYPES/dispatch/count changes (both drift
guards stay at 47); no projection-builder work (Phase 257); barcode
keyboard-wedge deferred. Requirement: POS-08.

**Milestone constraints carried into every decision:** all POS components are
builtins; seven-intent vocabulary frozen; no new crates; every emitted class
is a full string literal; no raw palette classes; single publish at Phase 258.

</domain>

<decisions>
## Implementation Decisions

### Numpad runtime contract (`runtime/numpad.rs`)
- **D-01:** Adopt the research data-attribute contract from
  `.planning/research/STACK.md` §Vanilla-JS Patterns as-is: container
  `data-numpad` + `data-numpad-target="{field}"`; display element
  `data-numpad-display` inside the container; keys
  `data-numpad-key="0".."9" | "backspace" | "clear"`; the written hidden input
  is located via `data-numpad-input="{field}"`. Event delegation: one `click`
  listener per `[data-numpad]` container using
  `event.target.closest('[data-numpad-key]')`. Phase 256's `render_numpad`
  emits exactly this contract (`NumpadProps.target_field` → both attributes).
- **D-02:** Quantity mode (default): integer digit entry with leading-zero
  collapse (`current === '0' ? key : current + key`), backspace removes last
  digit, clear empties; empty state displays and writes `"0"`.
- **D-03:** Price mode: **cents-shift entry** (real-POS convention — digits
  shift in from the right, no decimal-point key): tapping `1`,`2`,`5` shows
  `1.25`-style two-decimal formatting on the display while the **hidden field
  carries the raw digit string as integer cents** (`"125"`). Integer-cents
  arithmetic per `.planning/research/PITFALLS.md` (never float money). The
  display's decimal separator character is planner's discretion; the field
  value contract (integer cents) is locked and must be stated in the runtime
  module comment and in `NumpadProps::mode` rustdoc.
- **D-04:** Every key tap writes the field then dispatches
  `new Event('input', { bubbles: true })` on the hidden input (SC-3,
  form-guard compatible) — same pattern as `product_tiles.rs`.
- **D-05:** Form-guard visibility: `form_guards.rs`'s number guard currently
  collects only `input[type="number"]` and `input[data-qty-input]` — a numpad
  target field would be invisible to `number-gt-0`. Extend the guard's merged
  input list to include `input[data-numpad-input]`. Keep `data-numpad-input`
  as its own attribute (do NOT overload `data-qty-input`, which pairs with the
  ProductTile display contract).
- **D-06:** A max-length cap on entry prevents overflow (exact bound —
  e.g. 9 digits — planner's discretion). `setupNumpad()` is a no-op when no
  `[data-numpad]` exists (SC-2).

### PosFilter runtime contract (`runtime/pos_filter.rs`)
- **D-07:** Scope contract: the runtime iterates `[data-pos-filter]` scope
  containers. Within a scope it finds category tabs `[data-category-tab]`
  (attribute value = normalized category token; **empty value = "All"**),
  an optional search input `[data-pos-search]`, and product tiles identified
  by `[data-product-name]`. Multiple independent scopes per page are
  supported by construction. Phase 256's `render_product_grid` /
  `render_category_nav` emit exactly these attributes.
- **D-08:** `render_product_tile` gains an always-emitted
  `data-product-name="{name}"` attribute (raw prop value, HTML-escaped) in
  THIS phase — it is the search source AND the universal tile marker (tiles
  without categories carry no `data-product-categories`, so the runtime needs
  a marker present on every tile). This is a deliberate render touch within
  the phase goal ("stable data-attribute contract before any render function
  targets it"). The Phase 254 `product_tile_legacy_render_is_byte_identical`
  test is assertion-based, not a snapshot — extend it to assert the new
  attribute IS present on legacy tiles (name is a required prop), alongside a
  new escaping assertion mirroring `product_tile_escapes_categories`.
- **D-09:** Matching semantics — **intersection (AND)**: a tile is visible iff
  it matches the active category AND the search text.
  - Category match: active tab token ∈ the tile's space-separated
    `data-product-categories` token list; empty active token (All) matches
    every tile. Tokens are already space→hyphen normalized at render time
    (254 contract, `ProductTileProps::categories` rustdoc) — the runtime
    compares tokens verbatim, case-insensitively.
  - Search match: case-insensitive substring of the search input value
    against `data-product-name` (JS lowercases both sides); empty search
    matches everything. No debounce — catalogs are small and matching is
    attribute-only.
- **D-10:** Uncategorized tiles (no `data-product-categories` attribute) are
  visible under All and hidden under any specific category tab. The
  "Uncategorized" virtual sentinel tab (gestiscilo `data-tab=""` finding,
  FEATURES.md LOW) is a Phase 256 render decision — the runtime's
  empty-token-equals-All rule must not collide with it (if 256 adds a
  sentinel, it needs a distinct token, e.g. a reserved non-empty value).
- **D-11:** Hide/show mechanism: inline `el.style.display = 'none'` / `''`
  from JS. NOT the `hidden` attribute (UA `[hidden]{display:none}` loses to
  any author display utility on the tile) and NOT Tailwind's `hidden` class
  (display-utility order conflict with grid/flex classes is a stylesheet
  ordering gamble).
- **D-12:** Active-tab visual state toggles semantic-token classes only
  (mirror `tabs.rs` — `border-primary`/`text-primary` family; exact strings
  planner's call). The bundle-wide `variant_classes_use_semantic_tokens`
  scan must stay green; no raw palette classes in JS string literals.
  `setupPosFilter()` is a no-op when no `[data-pos-filter]` exists (SC-2).

### Double-submit guard (POS-08, `form_guards.rs` extension)
- **D-13:** The guard lives in `form_guards.rs`, initialized from the existing
  `setupFormGuards()` — NOT a new setup function. SC-1/SC-2 name exactly two
  new setups (`setupNumpad`, `setupPosFilter`); the double-submit guard is
  conceptually a form guard and adding a third dispatcher entry would drift
  from the phase's own success criteria.
- **D-14:** Behavior: for each `button[data-disable-on-submit]`, resolve its
  form (`closest('form')`, falling back to the HTML5 `form="<id>"` attribute
  — reuse the `findGuardedSubmit` inverse logic). On the form's **`submit`
  event** (never on button `click` — disabling in a click handler before
  submit fires can cancel the submission): if already submitted,
  `preventDefault()`; else mark submitted, set `disabled` and the house
  visual classes (`opacity-50`, `cursor-not-allowed`, matching existing
  guards).
- **D-15:** bfcache recovery: on `pageshow` with `event.persisted`, reset the
  submitted flag and re-enable the button — otherwise back-navigation to the
  register restores a permanently dead confirm button (real iPad Safari
  behavior).
- **D-16:** Emission point: additive `disable_on_submit: Option<bool>` on
  `ButtonProps` (`#[serde(default, skip_serializing_if = "Option::is_none")]`,
  rustdoc, backward-compat — mirrors 254 additive-prop conventions);
  `render_button` emits `data-disable-on-submit` when true. The `/cassa`
  sample's confirm button carries the attribute so SC-4 is demonstrable
  end-to-end this phase. Phase 256's CartPanel confirm slot consumes the same
  prop — named handoff.

### Idempotency-key pattern (documentation only)
- **D-17:** No new mechanism — PITFALLS.md is explicit: attach to the existing
  `framework::write` idempotency hook (`dispatch_write` steps 2/5, keyed on
  `(tenant_id, inputs["idempotency_key"])`). Ships as documentation.
- **D-18:** Home: `docs/src/features/write-kernel.md` gains a "Double-submit
  protection for forms" section documenting the layered pattern: (1) client
  guard — `data-disable-on-submit` on the confirm button; (2) server dedupe —
  a per-render UUID hidden input named `idempotency_key` in the cart-mutation
  form, consumed by `dispatch_write`'s existing check; (3) PRG so back/refresh
  doesn't re-POST. Whether the `/cassa` demo handler also demonstrates the
  hidden field is Claude's discretion (it is a plain-redirect demo, not a
  `dispatch_write` consumer).

### Module organization + wiring + tests
- **D-19:** Two new files: `runtime/numpad.rs` and `runtime/pos_filter.rs`,
  each `pub(super) const SOURCE: &str = r#"…"#` in the house ES5 style
  (`var`, `function`, no arrow functions, no template literals — match
  existing modules). Wire both into `FERRO_RUNTIME_JS` concatenation and the
  `ferroRuntime()` dispatcher in `runtime/mod.rs`.
- **D-20:** Extend BOTH existing drift lists in the same commit:
  `bundle_contains_all_setup_functions` (add `setupNumpad`, `setupPosFilter`)
  and `dispatcher_invokes_every_setup` (both invoked exactly once). SC-1/SC-2
  are satisfied by the existing test mechanism, not new test scaffolding.
- **D-21:** Inline-source inspection tests per SC-3/SC-4: bundle contains
  `data-numpad-key` handling, `data-product-categories` matching,
  `data-pos-search`, the bubbling `input` event dispatch, and
  `data-disable-on-submit` wiring. HTML attribute assertions:
  `data-product-name` on rendered tiles (with escaping test),
  `data-disable-on-submit` on a `render_button` with the prop set, absent
  without it.
- **D-22:** CI-exact gate before commit: `cargo fmt --all -- --check`,
  `cargo clippy --all --all-targets --all-features -- -D warnings`,
  `cargo test --all-features`, plus the docs build (`cargo doc` clean —
  write-kernel.md and new rustdoc ship this phase). Re-run fmt after any
  hand-edit. No `gen-ferro-base-css.sh` regen expected: the only new
  class-literal strings are JS-side visual-state toggles already present in
  the scanned crate source — verify with a diff check; if the generated CSS
  changes, commit the regen (the script scans crate source including runtime
  strings).

### Claude's Discretion
- Price-mode display separator character; exact max-length entry cap (D-03/D-06).
- Exact active-tab class strings (token-compliant, full literals) (D-12).
- Whether the `/cassa` demo handler demonstrates the idempotency hidden field
  (D-18).
- Internal JS naming, helper factoring inside the two new modules.
- Whether `data-pos-search` matching also strips diacritics (keep simple —
  lowercase-only is acceptable).

### Folded Todos
None — `todo match-phase 255` returned 0 matches.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone research (2026-07-04 — anchor for all v16.6 phases)
- `.planning/research/STACK.md` §Vanilla-JS Patterns — the Numpad module
  design this phase adopts verbatim (D-01): data model, event delegation,
  digit-entry sketch. Also touch-class rationale. NOTE: the Cart Runtime and
  Barcode Scanner sections are DEFERRED scope — do not implement them.
- `.planning/research/PITFALLS.md` — Pitfall 2 (double-submit: layered
  client-guard + idempotency-key prevention, `framework::write` hook, no new
  mechanism); Pitfall 4 (numpad = custom tap surface, never native input);
  integer-cents money rule; `active:` feedback rationale.
- `.planning/research/FEATURES.md` — filter semantics evidence (All tab,
  client-side substring search, uncategorized sentinel as LOW/deferred).
- `.planning/research/ARCHITECTURE.md` — module placement (`runtime/numpad.rs`
  Wave-2 mapping) and integration-point file anchors.

### Seed friction
- `.planning/phases/253-mcp-surface-docs-publish/253-FRICTION.md` — the
  ~1500-line RawHtml cassa picker audit; the filter/numpad runtimes are what
  make its elimination possible.

### Planning
- `.planning/ROADMAP.md` — v16.6 section: Phase 255 goal + SC 1–4 (the SC
  wording is the acceptance contract: exact test names
  `bundle_contains_all_setup_functions` / `dispatcher_invokes_every_setup`).
- `.planning/REQUIREMENTS.md` — POS-08 (lines ~270–272); Future Requirements
  (CartRuntime, barcode-wedge deferrals).

### Prior phase contracts this phase extends
- `.planning/phases/254-props-contracts-touch-foundation-design-rules/254-CONTEXT.md`
  — D-01 (categories plural, space→hyphen token normalization), D-17
  (behavioral contract anchors for NumpadProps/CategoryNavProps/
  ProductGridProps), D-18 (NO CartRuntime hooks — binding here too).
- `ferro-json-ui/src/component.rs` — `ProductTileProps::categories` rustdoc
  (token normalization contract the filter runtime must mirror);
  `NumpadProps` (`target_field`, `mode: NumpadMode`);
  `ProductGridProps::search`; `CategoryNavProps::items` rustdoc.
- `ferro-json-ui/src/runtime/mod.rs` — bundle assembly, dispatcher, and the
  two drift-list tests this phase extends (lines ~180, ~210).
- `ferro-json-ui/src/runtime/product_tiles.rs` — the hidden-input contract
  (`data-qty-input`/`data-qty-display`) and the house input-event dispatch
  pattern.
- `ferro-json-ui/src/runtime/form_guards.rs` — `findGuardedSubmit`, the
  number-guard input collection D-05 extends, the visual disabled-state
  classes D-14 reuses.

### Write kernel (idempotency documentation target)
- `docs/src/features/write-kernel.md` — existing idempotency documentation
  the new section extends (steps 2/5, `(tenant_id, idempotency_key)` dedupe).
- `framework/src/write/mod.rs` — `lookup_idempotency` / `store_idempotency`
  (the hook the docs reference; read-only this phase).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `runtime/mod.rs` LazyLock bundle assembly + `ferroRuntime()` dispatcher —
  the exact wiring pattern for the two new modules; both drift-list tests
  already exist and just gain entries.
- `runtime/product_tiles.rs` — input-event dispatch idiom
  (`dispatchEvent(new Event('input', { bubbles: true }))`) to copy in numpad.
- `runtime/form_guards.rs` — `findGuardedSubmit` (inside-form + `form="<id>"`
  fallback) reusable for the double-submit guard's button→form resolution;
  `opacity-50`/`cursor-not-allowed` disabled-state vocabulary.
- `runtime/tabs.rs` — semantic-token active-state class toggling pattern for
  category tabs.
- `render/atoms.rs` `render_product_tile` — already emits
  `data-product-categories` (254); gains `data-product-name` here; test
  module has the fixture helper (`make_product_tile`) and escaping-test
  pattern to extend.

### Established Patterns
- ES5-only runtime JS (var/function), one `setup*` per concern, no-op when
  target elements absent, single IIFE, no extra HTTP requests.
- Bundle-wide `variant_classes_use_semantic_tokens` scan — any class string
  in JS must be semantic-token, full-literal.
- Additive `Option<T>`/`Vec<T>` props with
  `#[serde(default, skip_serializing_if = …)]` + rustdoc + schema smoke test.
- CI runs clippy/test with `--all-features`; fmt after any hand-edit
  (publish-gate lesson).

### Integration Points
- `runtime/mod.rs` — concat list, dispatcher body, two test arrays.
- `component.rs` `ButtonProps` — additive `disable_on_submit` field (D-16).
- `render/atoms.rs` — `render_button` attribute emission;
  `render_product_tile` `data-product-name` emission.
- `app/src/controllers/cassa.rs` + its view — confirm button gains the
  attribute (sample demonstration; still RawHtml until Phase 257 flips it).
- `docs/src/features/write-kernel.md` — new double-submit section.

</code_context>

<specifics>
## Specific Ideas

- The phase's real deliverable is the **attribute contract**: Phase 256
  render functions must be able to target `data-numpad*`, `data-pos-filter`,
  `data-category-tab`, `data-pos-search`, `data-product-name`,
  `data-disable-on-submit` without guessing — every name above is final once
  this phase commits.
- Cents-shift price entry deliberately mirrors physical POS terminals
  (Square/Loyverse convention) — no decimal key exists on the pad.
- The double-submit guard binds on the form's `submit` event, not button
  `click` — the click-time disable race (button disabled before submit
  dispatch cancels the POST in some engines) is the classic bug this avoids.

</specifics>

<deferred>
## Deferred Ideas

- **CartRuntime** (live per-tap cart updates, client-computed totals,
  `runtime/cart_runtime.rs` sketch in STACK.md) — operator-deferred to
  Future Requirements; revisit on gestiscilo adoption friction.
- **Barcode keyboard-wedge module** (STACK.md sketch) — operator-deferred.
- **`pos-text-input-position` lint rule** (Warning for text inputs outside
  the top panel in fill-viewport specs — PITFALLS.md Pitfall 4 candidate) —
  not in the POS-11 rule set; backlog candidate for a future design-lint
  batch.
- **"Uncategorized" virtual sentinel tab** — Phase 256 render decision
  (D-10); runtime rule here is forward-compatible.

</deferred>

---

*Phase: 255-pos-runtime-modules-double-submit-protection*
*Context gathered: 2026-07-05*
