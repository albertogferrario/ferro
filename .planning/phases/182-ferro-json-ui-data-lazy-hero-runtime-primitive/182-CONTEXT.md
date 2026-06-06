---
phase: 182
name: ferro-json-ui data-lazy-hero runtime primitive
status: Ready for planning
gathered: 2026-06-06
discovered-by: jetskiadriatic startup-lifecycle audit (2026-06-06)
mode: auto
---

# Phase 182: `ferro-json-ui` `data-lazy-hero` runtime primitive — Context

<domain>
## Phase Boundary

Add a new runtime primitive to `ferro-json-ui` that lazily promotes below-the-fold hero videos from `preload="none"` to `preload="auto"` when the viewport approaches them.

Concretely: extend `ferro-json-ui/src/runtime/` with a new sibling primitive that, on `DOMContentLoaded`, selects every `<video preload="none" data-lazy-hero>` element, attaches one IntersectionObserver per distinct `rootMargin` value (default `200px 0px`, override per element via `data-lazy-hero-margin="…"`), and on each entry's first intersection: sets `preload="auto"`, calls `.load()` defensively, marks the element with `data-lazy-hero-promoted="1"`, and `unobserve()`s it. The marker attribute is the idempotency contract — re-running the setup is a no-op for already-promoted elements.

In scope:
- New JS source file in `ferro-json-ui/src/runtime/` (sibling to `sse.rs`, `sidebar.rs`, `scroll_preserve.rs`).
- Wire-up in `ferro-json-ui/src/runtime/mod.rs`: `mod` import, concatenation into `FERRO_RUNTIME_JS`, and dispatcher invocation inside `ferroRuntime()`.
- Public-contract attribute names: `data-lazy-hero` (the opt-in marker), `data-lazy-hero-margin` (per-element rootMargin override), `data-lazy-hero-promoted` (idempotency sentinel).
- String-presence tests in `ferro-json-ui/src/runtime/mod.rs` matching the sibling test pattern (assertions that the bundle contains `setupLazyHeroes`, `data-lazy-hero`, `IntersectionObserver`, `preload`).
- Docs page that catalogs ferro-json-ui's runtime data-* attributes (this is the first ferro-json-ui primitive that is part of the public ferro contract for tenant HTML to use, so the surface is documented now).
- Workspace version bump and crates.io publish via the existing GH Actions Wave1A flow.

Out of scope:
- Lazy-loading anything other than `<video>` elements (images already have native `loading="lazy"`; iframes have `loading="lazy"` too — no need to duplicate).
- Dynamic-insertion support: elements added to the DOM after `DOMContentLoaded` are not observed. Re-invoking the runtime on demand is deferred.
- A consumer-facing Rust API for emitting `<video data-lazy-hero>` from a JSON-UI component spec. This phase is a pure DOM-level primitive — tenant HTML (or future JSON-UI Video components) emits the attribute; the runtime acts on it.
- gestiscilo / jetskiadriatic consumer adoption — cross-tracked as gestiscilo Phase 186, lands after this phase publishes.

</domain>

<decisions>
## Implementation Decisions

### D-01: Observer cardinality — one observer per distinct `rootMargin` string
The roadmap's "single observer per page fans out … reading per-element `rootMargin`" framing is in tension with the IntersectionObserver API: `rootMargin` is per-observer, not per-element. Three readings are possible — only one survives Success Criterion 2 ("per-element override … honored at observer setup") AND Success Criterion 4 ("single-observer fan-out, no per-element observer cost"):

- ❌ One global observer with the default `rootMargin`, ignoring per-element overrides at runtime — violates SC-2.
- ❌ One observer per element — violates SC-4.
- ✅ **Group elements by their resolved `rootMargin` string; instantiate one IntersectionObserver per distinct value, with all elements sharing that value fanned out to it.** In the common case every element uses the default, so the page has exactly one observer. When a page mixes defaults with overrides (e.g., 4 default heroes + 1 hero needing `400px 0px`), the runtime creates exactly two observers — still "single-observer fan-out" relative to the elements per group, and "no per-element observer cost" because two observers serve five elements.

This reading is what the roadmap calls "string parsed at observer setup": at setup time, the per-element `data-lazy-hero-margin` attribute is read once, used to bucket elements, then each bucket is given to an observer constructed with that exact `rootMargin` string.

### D-02: Target only `<video preload="none">` — `data-lazy-hero` on other element shapes is silently ignored
The querySelector at setup is `video[preload="none"][data-lazy-hero]:not([data-lazy-hero-promoted])`. Three implications:

1. **Non-video elements with `data-lazy-hero` are ignored.** The attribute name encodes intent (hero video), and the promote action (flip `preload` attribute + call `.load()`) is a video-specific operation. Generalizing the primitive to other element types is a deferred idea — see deferred list.
2. **`<video>` elements without `preload="none"` are ignored.** If `preload="metadata"` or no `preload` is set, there is nothing to promote — the element is either already fetching metadata or browser-default. This avoids "promoting" elements that the page author specifically tuned.
3. **Already-promoted elements (`data-lazy-hero-promoted` present) are ignored.** Idempotency contract from Success Criterion 3 — re-running the setup is a no-op on already-promoted elements.

The promoted marker is set to the string `"1"` (per roadmap), checked with `hasAttribute` (any value present = promoted; matches the roadmap's documented attribute name).

### D-03: Single setup pass at `DOMContentLoaded` — no MutationObserver, no re-entry
Sibling runtime primitives (`setupSidebar`, `setupTabs`, `setupSSE`, `setupScrollPreserve`) are all one-shot at `DOMContentLoaded`. Phase 182 follows the same pattern.

Trade-off accepted: dynamically inserted `[data-lazy-hero]` elements (e.g., a kanban card revealed after a fetch) are not observed. The consumer surfacing motivating this phase (jetskiadriatic landing page, gestiscilo public-facing tenant pages) renders the hero set server-side as part of the initial HTML — the gap is theoretical, not field-reported.

If dynamic insertion ever becomes a real consumer need, the resolution is to expose `setupLazyHeroes` as a public function on `window.ferroRuntime` (or similar) that consumer code can call after DOM mutations, NOT to add a MutationObserver that increases the IIFE's permanent runtime cost. Logged as a deferred idea.

### D-04: Per-element cleanup — `unobserve(entry.target)` after promote; observer is left alive
On each successful promote, call `observer.unobserve(entry.target)`. This frees per-element observer bookkeeping inside the browser. The observer object itself is NOT disconnected even when its watched count drops to zero — the cost of leaving an empty observer alive is negligible, and avoiding the count-and-disconnect bookkeeping keeps the IIFE size budget closer to the 400-byte target (Success Criterion 4).

If a tenant page has zero `[data-lazy-hero]` elements, the setup function returns early before any observer is created (the `!els.length` guard) — there is no idle observer.

### D-05: Feature detection — `typeof IntersectionObserver === 'undefined'` early-return guard
Matches the pattern already established in `ferro-json-ui/src/plugins/map.rs` §306. Modern evergreen browsers have shipped IO since 2017, but the guard protects against:
- Legacy embedded WebViews on older Android/iOS devices.
- Testing harnesses with minimal DOM polyfills.
- SSR-during-test scenarios where a partial DOM is present.

When the guard fires, the function silently returns. The downside (no lazy promotion → all videos behave as the author wrote them) is strictly better than the alternative (JS exception aborting `ferroRuntime()`, breaking unrelated primitives downstream in the dispatch chain).

### D-06: Module placement and naming
New file: `ferro-json-ui/src/runtime/hero_lazy.rs`. Follows the snake_case sibling convention (`scroll_preserve.rs`, `form_guards.rs`). The setup function inside the SOURCE string is `setupLazyHeroes` (camelCase, plural — matching `setupDropdowns`, `setupTabs`, `setupToasts`).

Wire-up in `ferro-json-ui/src/runtime/mod.rs`:
1. `mod hero_lazy;` in the module list (kept alphabetical: between `form_guards` and `kanban`).
2. `s.push_str(hero_lazy::SOURCE);` in the IIFE assembly. Position: between `scroll_preserve` and `sse` (i.e., before any primitive that might trigger network fetches), or simply appended after `scroll_preserve` — the dispatch order does not change observable behavior, and the planner picks a position that minimizes the diff. The dispatcher comment block is updated to include `setupLazyHeroes();`.
3. The `bundle_contains_all_setup_functions` and `dispatcher_invokes_every_setup` tests in `runtime/mod.rs` add `"setupLazyHeroes"` and `"setupLazyHeroes();"` to their assertion lists.

### D-07: Test strategy — string-presence tests in `runtime/mod.rs`, no headless browser test
This is consistent with how every other sibling primitive is tested. The runtime bundle is a string; the unit tests assert that the assembled string contains the expected setup-function name, IntersectionObserver call, `preload`/`auto` literals, and the public attribute names. Behavior is verified manually in-browser against tenant pages.

New tests to add in `ferro-json-ui/src/runtime/mod.rs`:
- `runtime_contains_lazy_hero_setup` — asserts `setupLazyHeroes`, `data-lazy-hero`, `IntersectionObserver`, `preload`, `"auto"` are all present in `FERRO_RUNTIME_JS`.
- Update `bundle_contains_all_setup_functions` to include `"setupLazyHeroes"`.
- Update `dispatcher_invokes_every_setup` to include `"setupLazyHeroes();"`.

In-browser verification path (out of phase, on the consumer side): jetskiadriatic landing page after gestiscilo Phase 186 bumps `ferro-json-ui` — open the page with the Network panel filtered to `.mp4`/`.webm`, scroll, verify that video bytes only appear after the rootMargin threshold is crossed. This is the Success Criterion 1 verification described in the roadmap; it lives in the consumer-side UAT, not in ferro's test suite.

### D-08: Single publish at end of phase — gestiscilo Phase 186 consumes after merge
Per memory `feedback_friction_loop_release_cadence.md`: mid-loop publishes freeze the API before later batches can revise it. Phase 182 publishes once, at the end, via the existing Wave1A GH Actions workflow (`ferro-json-ui` is already in `WAVE1A_CRATES` at `.github/workflows/publish.yml:N`).

Workspace version is currently `0.2.41` (Cargo.toml workspace.package.version, post the Phase 181 cycle). Phase 182's merge bumps to `0.2.42` as a single workspace-wide bump — matches the existing release cadence; no per-crate semver staging.

The cross-repo consumer (gestiscilo Phase 186 [FERRO REPO]) bumps `ferro-json-ui` in its `Cargo.toml` after `0.2.42` is published. That bump is gestiscilo's responsibility, not phase 182's deliverable. The phase here is "publish a runtime primitive"; the phase there is "consume the primitive in tenant HTML."

### D-09: Docs — new page `docs/src/json-ui/runtime-primitives.md`
No current docs page catalogs ferro-json-ui's data-* runtime attributes. The existing pages (`json-ui/components.md`, `json-ui/forms.md`, `json-ui/data-binding.md`, etc.) cover the Rust authoring surface; the JS runtime side is an implementation detail today.

`data-lazy-hero` changes that: it is the first ferro-json-ui primitive where the public contract is a DOM attribute that tenant HTML (or future component output) sets directly — not a Rust API call. This makes the runtime side part of the public surface and forces it to be documented.

The new page covers:
1. **`data-lazy-hero`** — opt-in marker on `<video preload="none">`. Default `rootMargin: 200px 0px`. Idempotent via `data-lazy-hero-promoted`.
2. **`data-lazy-hero-margin`** — per-element rootMargin override (string passed verbatim to the IntersectionObserver constructor).
3. **Browser support** — IntersectionObserver feature-detection note; no-op on browsers without IO.
4. **Forward-compat note** — the runtime is one-shot at `DOMContentLoaded`; dynamically inserted elements are not observed.

The page is structured to grow: it has an explicit subsection for `data-lazy-hero` now, and the framing accommodates future runtime-attribute additions (e.g., a future `data-defer-load` for image embeds). Sibling runtime primitives that are already implementation-only (`data-sse-url`, `data-sidebar-toggle`, `data-popover-menu`, etc.) are NOT enumerated on this page in Phase 182 — those remain internal to the components that emit them. The page is specifically "public DOM attributes you can set on hand-authored or component-output HTML to opt into ferro-json-ui runtime behaviors."

Per CLAUDE.md global instruction: docs must reflect current framework features. Per CLAUDE.md project instruction: docs/src/ updates are required when ferro-* features change. Non-negotiable for this phase.

### D-10: IIFE size budget — ~400-byte target is a guideline, not a hard fail
Success Criterion 4 sets a ~400-byte growth target for `FERRO_RUNTIME_JS`. A faithful implementation of D-01 through D-09 — with feature detection, group-by-margin bucketing, observer instantiation per bucket, promote action including `.load()` and marker, and unobserve — comes in around 500–700 bytes raw including whitespace and the `pub(super) const SOURCE: &str = r#"…"#;` wrapper.

The 400-byte target is interpreted as: keep the implementation lean. Avoid bookkeeping that doesn't directly serve the contract. The planner picks the exact minified-or-near-minified composition; the success criterion is treated as "growth budget under ~500 bytes" in practice, with the constraint that any growth above ~700 bytes triggers a redesign conversation. Comments inside the SOURCE string are minimized — explanation of intent lives in the docs page (D-09) and in this CONTEXT.md.

### Claude's Discretion
- Exact line-by-line composition of the JS source (whitespace, var naming, exact iteration style). The contract is fixed by D-01 through D-05; the prose is the planner's call.
- Insertion point of `hero_lazy::SOURCE` in `runtime/mod.rs`'s `push_str` chain (anywhere in the chain produces equivalent behavior; pick the position that minimizes the diff).
- Exact title and section ordering of the new docs page (D-09).
- Whether the new mod.rs string-presence test is one combined test or several focused tests. (Sibling pattern shows both styles in the existing test module.)

### Folded Todos
None — `gsd-tools todo match-phase 182` was not run because phase 182 is not yet in the tool's phase index (no directory existed when init was invoked). The roadmap entry is the only source of scope; no separate backlog matches surface.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Source — runtime architecture (ferro-json-ui)
- `ferro-json-ui/src/runtime/mod.rs` §26-60 — `FERRO_RUNTIME_JS` `LazyLock<String>` assembly: IIFE wrapper, per-module `SOURCE` concatenation, `ferroRuntime()` dispatcher, `DOMContentLoaded` registration. Phase 182 adds one entry to each of: the `mod` list, the `s.push_str(...)` chain, the dispatcher comment block.
- `ferro-json-ui/src/runtime/mod.rs` §141-191 — existing test patterns (`bundle_contains_all_setup_functions`, `dispatcher_invokes_every_setup`, individual primitive presence tests). Phase 182 extends these.
- `ferro-json-ui/src/runtime/sse.rs` §1-42 — closest sibling for the data-attribute-driven setup pattern (`document.body.getAttribute('data-sse-url')`).
- `ferro-json-ui/src/runtime/sidebar.rs` §1-37 — sibling using `document.querySelector('[data-sidebar]')` selector pattern.
- `ferro-json-ui/src/runtime/scroll_preserve.rs` §1-69 — sibling that uses `querySelectorAll` with capture-phase event listeners and `sessionStorage` — most complex sibling, shows the upper bound on per-primitive complexity.
- `ferro-json-ui/src/runtime/dropdowns.rs` §1-80 — sibling that uses `querySelectorAll('[data-popover-menu]')` and iterates with init function per element. Phase 182's fan-out pattern is analogous (single observer, multiple targets) but with the additional grouping layer for `rootMargin` (D-01).

### Source — existing IntersectionObserver usage in ferro-json-ui
- `ferro-json-ui/src/plugins/map.rs` §242-321 — Leaflet map plugin uses `IntersectionObserver` to call `map.invalidateSize()` when a hidden map becomes visible. §306 shows the feature-detection guard (`if (typeof IntersectionObserver !== 'undefined')`) that Phase 182's D-05 mirrors verbatim.
- `ferro-json-ui/src/plugins/map.rs` §474-475 — existing test pattern asserting `script.contains("IntersectionObserver")`; same shape of assertion Phase 182 adds for `setupLazyHeroes`.

### Source — runtime embedding into HTML output
- `ferro-json-ui/src/layout.rs` §312-322 — `with_runtime(ctx_scripts: &str) -> String` wraps `FERRO_RUNTIME_JS` in a `<script>` tag and combines with plugin scripts. The new primitive ships through this exact channel — no layout-side changes required.
- `ferro-json-ui/src/layout.rs` §316, §608 — call sites that pull `FERRO_RUNTIME_JS.as_str()` into the page. Two layouts emit it: `DefaultLayout` and `DashboardLayout`. Both are exercised by tenant pages.

### Crate metadata and publishing
- `Cargo.toml` (workspace root) §`workspace.package.version` — current `0.2.41`. Phase 182 bumps to `0.2.42` as part of the merge cycle.
- `ferro-json-ui/Cargo.toml` — `version.workspace = true`, so the crate inherits the workspace bump automatically.
- `.github/workflows/publish.yml` — `ferro-json-ui` is listed in `WAVE1A_CRATES`. No workflow change required for Phase 182's publish — the existing wave handles it.

### Discovery and consumer context
- `.planning/ROADMAP.md` §`Phase 182: ferro-json-ui data-lazy-hero runtime primitive` — full phase definition, goal, success criteria, discovery note. Single source of truth for the public contract (attribute names, default `rootMargin`, idempotency marker).
- Discovery: 2026-06-06 jetskiadriatic startup-lifecycle audit — tenant `index.html` has 4 below-the-fold heroes at `preload="none"`. Pure generic web primitive; any ferro app with above-the-fold + below-the-fold hero videos benefits. Same elevation rule as Phase 165 F11/F13/F14 (runtime gaps belong in ferro, not in consumer-side scripts).
- Cross-tracked as gestiscilo Phase 186 [FERRO REPO] — consumer-side adoption phase that bumps `ferro-json-ui` in `Cargo.toml` after Phase 182 publishes. Out of phase 182's deliverable scope.

### Project-level conventions
- `CLAUDE.md` (project root) — "Crates under `ferro-*` are libraries shared across every ferro application; they must not hardcode any application identity." Phase 182's primitive is generic web behavior (preload promotion); the attribute names (`data-lazy-hero`, `data-lazy-hero-margin`, `data-lazy-hero-promoted`) carry no tenant identity. ✓
- `CLAUDE.md` (project root) §"Always update docs when framework changes" — D-09's new docs page is mandated by this rule.
- `CLAUDE.md` (project root) §"When adding a new crate to the workspace, always add it to `.github/workflows/publish.yml`" — N/A for Phase 182 (extends existing crate, not adding new).
- `.planning/PROJECT.md` — pre-1.0, breaking changes acceptable. Phase 182 introduces a brand-new contract surface (the attribute names); not a break of anything existing.

### Prior-phase context (already-decided constraints to honor)
- `.planning/phases/181-json-ui-input-error-prop-inline-render/181-CONTEXT.md` — sibling-phase template for the documentation/contract framing applied here.
- Memory: `feedback_friction_loop_release_cadence.md` — single publish at end of release loop; gestiscilo bumps after merge (D-08).
- Memory: `feedback_breaking_changes_v12_ai.md` — breaking changes acceptable; not relevant here because Phase 182 is purely additive.
- Memory: `feedback_no_duplicate_control_surface.md` — before adding a new annotation/config knob, check if an existing ferro layer already decides that thing. Verified: no existing layer in ferro promotes video preload — native `loading="lazy"` covers `<img>` and `<iframe>` but not `<video>`. The new attribute is filling a genuine gap, not duplicating existing control.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Feature-detection idiom** — `if (typeof IntersectionObserver !== 'undefined') { var observer = new IntersectionObserver(...); observer.observe(el); }` is locked from `ferro-json-ui/src/plugins/map.rs` §306-315. D-05 reuses this exact guard.
- **Per-primitive `pub(super) const SOURCE: &str = r#"…"#;` shape** — every sibling file in `runtime/` follows this pattern. Phase 182's `hero_lazy.rs` follows it verbatim.
- **ES5-style JS authoring conventions** — sibling files use `var`, `function() {…}` expressions, no arrow functions, no template literals, no `let`/`const`, no destructuring. This is a deliberate choice for broadest WebView compatibility (the same audience the IntersectionObserver feature-detection guard exists for). Phase 182 follows the same style.
- **`document.querySelectorAll` + array iteration with `for (var i = 0; i < n; i++)`** — used in `sidebar.rs`, `dropdowns.rs`, `scroll_preserve.rs`. The pattern matches Phase 182's element-bucket-by-margin loop.
- **`.hasAttribute(...)` guard for idempotency** — implicit in the `data-lazy-hero-promoted` selector exclusion (`:not([data-lazy-hero-promoted])`); also checked inside the observer callback as a belt-and-suspenders guard for the case where the same element is in two groups (impossible by construction, but cheap to assert).

### Established Patterns
- **One IIFE, one DOMContentLoaded listener** — Phase 182 must not add a separate listener; the existing `ferroRuntime()` dispatcher invokes every primitive's setup function in turn. The dispatcher is the entry point.
- **Setup function returns early on absence** — every sibling (`setupSidebar`, `setupSSE`, `setupTabs`) returns immediately if the relevant DOM attribute is absent. Phase 182's `setupLazyHeroes` returns early when the IntersectionObserver guard fires OR when `querySelectorAll` finds no matches. Both early-returns serve the same goal: zero runtime cost on pages that don't opt in.
- **No `try`/`catch` around the happy path** — siblings use `try`/`catch` only around explicitly unsafe operations (`sessionStorage` access in `scroll_preserve.rs`, `JSON.parse` in `sse.rs`). The promote action (`setAttribute`, `setAttribute`, `load()`) is wrapped in a minimal `try { … } catch (_) {}` because `<video>.load()` can throw under some browser quirks (e.g., cross-origin restrictions during early page load).
- **Tests assert string presence in the assembled bundle** — every sibling has corresponding string-presence assertions in `runtime/mod.rs`. Phase 182 follows the same shape; no separate test file.

### Integration Points
- **`FERRO_RUNTIME_JS` flows to every page via `with_runtime()` in `layout.rs`** — Phase 182's primitive ships to every tenant page that uses `DefaultLayout` or `DashboardLayout` from `ferro-json-ui`. No layout-side changes.
- **Tenant HTML opt-in** — the primitive is dormant on any page that doesn't have `[data-lazy-hero]` elements. Tenant pages (jetskiadriatic, gestiscilo public-facing) add the attribute to their `<video>` tags; the runtime sees them on the next load.
- **No JSON-UI component needed in Phase 182** — Phase 182 only adds the runtime side. If a future phase adds a `Video` JSON-UI component, that component will emit `data-lazy-hero` for opt-in but is out of scope here.
- **`ferro-mcp` introspection** — no change needed. The new attribute is a DOM-level contract, not a Rust API surface that `ferro-mcp` enumerates via `json_ui_catalog`.

</code_context>

<specifics>
## Specific Ideas

- Default `rootMargin` of `200px 0px` was chosen by the roadmap author as "~half a second of network warmup before viewport entry" at typical scroll speeds. The value is locked at the runtime level; the per-element override is the escape hatch for tenants that profile their own scroll behavior.
- The roadmap's reference to "the same elevation rule as Phase 165 F11/F13/F14" frames Phase 182 as filling a runtime gap that belongs in ferro rather than in tenant-side scripts. Decisions throughout treat this primitive as a public ferro contract, not as an internal implementation detail — hence the docs page (D-09) and the careful attribute-name freeze.
- The closest existing primitive in spirit is `ferro-json-ui/src/plugins/map.rs`'s IntersectionObserver for `map.invalidateSize()`: same browser API, same feature-detection guard, same "do work when element enters viewport" intent. Phase 182's planner can read map.rs §306-315 verbatim for the API shape.
- Roadmap discovery context: "Tenant `index.html` has 4 below-the-fold heroes at `preload='none'`; the only way to lazily promote them today is per-page IntersectionObserver boilerplate." Confirms the practical scale (a handful of heroes per page) — the single-observer fan-out architecture (D-01) is right-sized for this.
- The promoted-marker attribute name is `data-lazy-hero-promoted` (not `data-promoted`) — keeping the `lazy-hero-` namespace prefix on every related attribute prevents collisions with other libraries or hand-authored markup.

</specifics>

<deferred>
## Deferred Ideas

- **Generalize `data-lazy-hero` to other element types.** Audio (`<audio preload="none">`), iframes that wrap third-party players, or arbitrary "defer-load" sentinels could share the same intersection-driven promotion infrastructure. Not Phase 182's scope — the contract is explicitly `<video>`-shaped. If a real consumer surfaces the need, a sibling primitive (e.g., `data-defer-load` with a more general action set) is the cleaner answer than expanding `data-lazy-hero`.
- **Dynamic-insertion support via MutationObserver.** Today's primitive is one-shot at `DOMContentLoaded` (D-03). If a tenant page renders heroes after a `fetch` or via client-side route transitions, those heroes are not lazy-promoted. Resolution path when needed: expose `setupLazyHeroes` as a callable on a window namespace so consumer code can re-run it after DOM mutations. Adding a global MutationObserver is rejected on cost grounds.
- **Public Rust API for emitting `<video data-lazy-hero>` from a JSON-UI Video component.** No `Video` component exists in `ferro-json-ui` today. When one is added (separate phase), it should default to emitting `preload="none" data-lazy-hero` for video sources outside the above-the-fold region — that policy lives in the component, not in this runtime phase.
- **Network-aware `rootMargin` tuning.** A more sophisticated primitive could read `navigator.connection.effectiveType` and shrink the rootMargin on slow connections (to avoid pre-fetching a 50MB hero on 3G). Possible v2 of the primitive; out of Phase 182 because it muddies the public attribute contract.
- **Catalog all existing runtime data-* attributes in the new docs page.** Phase 182's docs page (D-09) covers only `data-lazy-hero` because that is the first runtime attribute that is part of the public contract for tenant HTML. The existing internal attributes (`data-sse-url`, `data-sidebar-toggle`, `data-popover-menu`, etc.) are component-implementation details that flow from Rust component output, not consumer-authored HTML — documenting them as if they were public would calcify implementation details into the public surface. If consumer demand emerges to set those attributes by hand, that is a separate elevation phase.

### Reviewed Todos (not folded)
None — no GSD todo matching was performed for this phase (phase-op init returned `phase_found: false` because the phase directory did not yet exist; `gsd-tools todo match-phase` requires the resolved phase number to look up matches). If pending todos surface relevant to Phase 182 during planning, the planner can fold them into PLAN.md task descriptions at that point.

</deferred>

---

## Discovery Transcript (preserved from roadmap)

The original discovery note from the 2026-06-06 jetskiadriatic startup-lifecycle audit, preserved verbatim from the Phase 182 ROADMAP.md entry:

> Discovery: surfaced during the 2026-06-06 jetskiadriatic startup-lifecycle audit. Tenant `index.html` has 4 below-the-fold heroes at `preload="none"`; the only way to lazily promote them today is per-page IntersectionObserver boilerplate. Pure generic web primitive — any ferro app with above-the-fold + below-the-fold hero videos benefits. Cross-tracked as gestiscilo Phase 186 [FERRO REPO]. Same elevation rule as Phase 165 F11/F13/F14 (runtime gaps belong in ferro, not in consumer-side scripts).

### Concrete repro context

Tenant `index.html` pattern (pre-Phase 182):

```html
<video preload="none" poster="…hero-1.jpg" muted playsinline>
  <source src="/assets/hero-1.mp4" type="video/mp4">
</video>
…
<video preload="none" poster="…hero-2.jpg" muted playsinline>
  <source src="/assets/hero-2.mp4" type="video/mp4">
</video>
```

Without Phase 182, the tenant has two equally bad options:
1. Leave `preload="none"` — first frame doesn't render until scroll reaches the element; users see a stale poster image until ~1.5s after viewport entry while the browser starts the fetch from cold.
2. Set `preload="auto"` — every page load fetches every hero video on the page, regardless of fold position; on a long landing page with 4 heroes, that's ~80MB pre-fetched even for users who never scroll past the first hero.

Phase 182 closes the gap: tenants set `data-lazy-hero` on the below-the-fold videos. The runtime promotes each as the viewport approaches, giving the browser ~half a second to fetch the first frame so it lands before the user reaches the element.

Post-Phase 182 tenant pattern:

```html
<!-- Above-the-fold: load eagerly -->
<video preload="auto" poster="…hero-0.jpg" muted playsinline>
  <source src="/assets/hero-0.mp4" type="video/mp4">
</video>

<!-- Below-the-fold: lazy-promote on approach -->
<video preload="none" data-lazy-hero poster="…hero-1.jpg" muted playsinline>
  <source src="/assets/hero-1.mp4" type="video/mp4">
</video>

<!-- Below-the-fold with bigger lead time (slower-loading hero) -->
<video preload="none" data-lazy-hero data-lazy-hero-margin="400px 0px" poster="…hero-2.jpg" muted playsinline>
  <source src="/assets/hero-2.mp4" type="video/mp4">
</video>
```

---

*Phase: 182-ferro-json-ui-data-lazy-hero-runtime-primitive*
*Context gathered: 2026-06-06 (--auto)*
