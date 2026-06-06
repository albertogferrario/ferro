# Phase 182: `ferro-json-ui` `data-lazy-hero` runtime primitive — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in 182-CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-06
**Phase:** 182-ferro-json-ui-data-lazy-hero-runtime-primitive
**Mode:** `--auto` (all gray areas auto-resolved with recommended option; choices logged below)
**Areas discussed:** Observer cardinality / fan-out architecture, Element-shape targeting, Setup lifecycle (one-shot vs MutationObserver), Cleanup policy, Feature detection, Module placement and naming, Test strategy, Publish cadence, Docs strategy, IIFE size budget

---

## Observer cardinality / fan-out architecture

The roadmap's framing — "single observer per page fans out … reading per-element `rootMargin`" — is in tension with the IntersectionObserver API contract (`rootMargin` is per-observer, not per-element). Three readings were considered:

| Option | Description | Honors SC-2 (per-element override) | Honors SC-4 (no per-element cost) | Selected |
|--------|-------------|------------------------------------|-----------------------------------|----------|
| One global observer with default `rootMargin`; per-element override ignored at runtime | Strict single-observer reading | ❌ | ✅ | |
| One IntersectionObserver instance per element | Per-element rootMargin trivially honored | ✅ | ❌ | |
| Group elements by their resolved `rootMargin` string; one observer per distinct value | Common case = single observer; mixed-margin pages get a small handful | ✅ | ✅ | ✓ |

**User's choice (auto):** Group by `rootMargin` string — the only reading that satisfies both success criteria. The "single-observer fan-out" framing is interpreted as "no per-element observer cost," not literal observer-cardinality-equals-one.

**Notes:** In the common case (all elements use the default `200px 0px`), the page has exactly one observer. The bucketing layer adds ~50 bytes of JS over the strict-single-observer reading; survives the IIFE budget conversation in D-10.

---

## Element-shape targeting

| Option | Description | Selected |
|--------|-------------|----------|
| Promote any element with `data-lazy-hero` regardless of shape | Generic; future-flexible | |
| Target only `<video preload="none">` with the attribute; ignore other shapes silently | Specific to the documented intent | ✓ |
| Target `<video>` + `<audio>` + `<iframe>` with `preload="none"` | Broaden to all preload-bearing media | |

**User's choice (auto):** Target only `<video preload="none">`. The selector at setup is `video[preload="none"][data-lazy-hero]:not([data-lazy-hero-promoted])`. The attribute name encodes intent (hero video), and the promote action (flip `preload` + call `.load()`) is video-specific. Generalizing is a deferred idea — see 182-CONTEXT.md `<deferred>`.

**Notes:** `<img>` and `<iframe>` have native `loading="lazy"` — no reason to duplicate that surface. `<audio>` is rare in tenant pages; revisit on real demand.

---

## Setup lifecycle (one-shot vs MutationObserver)

| Option | Description | Selected |
|--------|-------------|----------|
| One-shot at `DOMContentLoaded` (matches every sibling primitive) | Zero ongoing cost; matches established convention | ✓ |
| One-shot + global MutationObserver to catch dynamically inserted elements | Handles client-side route transitions / async fetch insertions | |
| Expose `window.setupLazyHeroes` for consumer code to re-call on demand | Hybrid: one-shot by default, opt-in re-run | |

**User's choice (auto):** One-shot at `DOMContentLoaded`. Every other sibling runtime primitive (`setupSidebar`, `setupTabs`, `setupSSE`, `setupScrollPreserve`) follows this pattern. The motivating consumers (jetskiadriatic landing page, gestiscilo public-facing tenant pages) render the hero set server-side as part of the initial HTML; there is no field-reported gap.

**Notes:** Future MutationObserver support is rejected on permanent-cost grounds. If dynamic insertion ever becomes a real consumer need, the resolution is to expose the setup function as a window namespace callable, not to add a global observer. Logged as a deferred idea.

---

## Cleanup policy

| Option | Description | Selected |
|--------|-------------|----------|
| Per-element `unobserve(entry.target)` after promote; leave observer alive | Frees per-element bookkeeping; minimal IIFE bytes | ✓ |
| `observer.disconnect()` after the last element is promoted | Marginal memory savings; requires count-and-disconnect bookkeeping | |
| No cleanup; let the observer fire repeatedly until page navigation | Smallest possible IIFE; wastes observer fires | |

**User's choice (auto):** `unobserve(entry.target)` after promote. Frees per-element observer state; avoids the bookkeeping cost of a count-and-disconnect. The observer object lingering with zero watched elements has negligible cost.

**Notes:** The setup function's early-return guard (`if (!els.length) return;`) means an idle observer is never created on pages with zero `[data-lazy-hero]` elements.

---

## Feature detection

| Option | Description | Selected |
|--------|-------------|----------|
| `if (typeof IntersectionObserver === 'undefined') return;` early-return guard | Matches `ferro-json-ui/src/plugins/map.rs` §306 pattern | ✓ |
| No feature detection — IO has been stable since 2017 | Smaller bytecount; risks exception in legacy WebViews | |
| `try`/`catch` wrap entire setup | Catches anything; opaque failure mode | |

**User's choice (auto):** Explicit `typeof IntersectionObserver === 'undefined'` early-return guard, mirroring the established pattern in `ferro-json-ui/src/plugins/map.rs` §306. Protects against legacy embedded WebViews, testing harnesses with minimal DOM polyfills, and partial-DOM SSR test scenarios. When the guard fires, the function silently returns — strictly better than a JS exception aborting `ferroRuntime()` and breaking unrelated primitives downstream.

---

## Module placement and naming

| Option | Description | Selected |
|--------|-------------|----------|
| New file `ferro-json-ui/src/runtime/hero_lazy.rs`; setup function `setupLazyHeroes` | Matches snake_case file / camelCase function sibling convention | ✓ |
| Merge into an existing primitive file (e.g., `scroll_preserve.rs`) | Smaller diff; concedes conceptual coherence | |
| Sub-module under a future `media/` directory | Anticipates broader media-handling primitives | |

**User's choice (auto):** New file `ferro-json-ui/src/runtime/hero_lazy.rs`. Sibling convention is strict: one primitive per file, snake_case filename, camelCase JS setup function with plural noun (`setupDropdowns`, `setupTabs`, `setupToasts`). Sub-module directories are deferred until there are two or more sibling primitives in the same conceptual area.

**Notes:** Insertion point in `runtime/mod.rs`'s `push_str` chain is Claude's discretion (alphabetical between `form_guards` and `kanban` is the natural slot). Dispatcher comment block updates to include `setupLazyHeroes();`. Tests in `runtime/mod.rs` extend `bundle_contains_all_setup_functions` and `dispatcher_invokes_every_setup`.

---

## Test strategy

| Option | Description | Selected |
|--------|-------------|----------|
| String-presence tests in `runtime/mod.rs` matching sibling pattern | Consistent with every other primitive in `runtime/` | ✓ |
| Headless-browser integration test (Playwright/Puppeteer) | Verifies actual IO behavior; new dependency surface | |
| Property-style test asserting `FERRO_RUNTIME_JS` doesn't grow past the byte budget | Enforces SC-4 mechanically | |

**User's choice (auto):** String-presence tests. Every sibling primitive is tested by asserting that the assembled `FERRO_RUNTIME_JS` string contains the expected setup-function name and key API references. Phase 182 follows the same pattern (`setupLazyHeroes`, `data-lazy-hero`, `IntersectionObserver`, `preload`, `"auto"`).

**Notes:** Browser-level behavior verification (Success Criterion 1, "verified via Chrome DevTools Network panel") lives in the consumer-side UAT — jetskiadriatic and gestiscilo Phase 186 after `ferro-json-ui` 0.2.42 publishes. Not in ferro's test suite.

---

## Publish cadence

| Option | Description | Selected |
|--------|-------------|----------|
| Single publish at end of phase via existing Wave1A GH Actions; workspace bump to 0.2.42 | Matches `feedback_friction_loop_release_cadence.md` | ✓ |
| Mid-phase publish so consumer can start integrating | Speeds parallel work; risks freezing the API before later batches revise | |
| Per-crate semver staging | Diverges from current workspace-bump convention | |

**User's choice (auto):** Single publish at end of phase. Memory `feedback_friction_loop_release_cadence.md` is explicit: mid-loop publishes freeze the API before later batches can revise it. Consumer (gestiscilo Phase 186) waits for `0.2.42` to land before bumping `ferro-json-ui` in its `Cargo.toml`.

**Notes:** `ferro-json-ui` is already in `WAVE1A_CRATES` at `.github/workflows/publish.yml`; no workflow change required. Workspace-level Cargo.toml version inheritance means `ferro-json-ui` picks up the 0.2.42 bump automatically.

---

## Docs strategy

| Option | Description | Selected |
|--------|-------------|----------|
| New page `docs/src/json-ui/runtime-primitives.md` covering only `data-lazy-hero` (extensible framing) | First public-contract DOM attribute; documents the new surface scoped tightly | ✓ |
| Append to existing `docs/src/json-ui/components.md` | Misclassifies a DOM-attribute contract as a component | |
| Enumerate every existing `data-*` runtime attribute on the new page | Surfaces implementation details as public API | |

**User's choice (auto):** New page documenting only `data-lazy-hero` (with framing that admits future runtime attribute additions). Existing internal attributes (`data-sse-url`, `data-popover-menu`, `data-sidebar-toggle`, etc.) are component-implementation details emitted by Rust component output — documenting them as if they were public would calcify implementation details into the public surface. CLAUDE.md project instruction "always update docs when framework changes" is non-negotiable; this page is the docs deliverable for Phase 182.

**Notes:** Page structure leaves room for future public DOM attributes without restructuring. If consumer demand ever surfaces to set the currently-internal attributes by hand, that is a separate elevation phase, not a Phase 182 retrofit.

---

## IIFE size budget

| Option | Description | Selected |
|--------|-------------|----------|
| Treat ~400-byte target as a guideline; aim for ≤500 bytes; >700 bytes triggers redesign | Honors the spirit of SC-4 without false precision | ✓ |
| Hard-fail at >400 bytes; minify aggressively or cut feature scope | Strict literal reading | |
| Drop the budget; prioritize readability over byte count | Concedes the success criterion | |

**User's choice (auto):** Soft target. The faithful implementation of D-01 through D-09 — with feature detection, group-by-margin bucketing, observer instantiation per bucket, promote action including `.load()` and marker, and unobserve — comes in around 500–700 bytes raw. The 400-byte target is interpreted as "keep it lean"; the constraint that anything above ~700 bytes triggers a redesign conversation. Comments inside the SOURCE string are minimized — intent explanation lives in the docs page and the CONTEXT.md.

**Notes:** The planner picks the exact line-by-line composition; the success criterion is treated as a target, not a build-failing check. If the implementation lands above ~700 bytes, the planner surfaces it for a real conversation rather than silently shipping over budget.

---

## Claude's Discretion

The following decisions are explicitly delegated to the planner / executor:

- Exact line-by-line composition of the JS source (whitespace, var naming, exact iteration style). The contract is fixed by the decisions above; the prose is the planner's call.
- Insertion point of `hero_lazy::SOURCE` in `runtime/mod.rs`'s `push_str` chain (anywhere in the chain produces equivalent behavior; pick the position that minimizes the diff).
- Exact title and section ordering of the new docs page.
- Whether the new `mod.rs` string-presence test is one combined test (`runtime_contains_lazy_hero_setup`) or several focused tests. Sibling pattern shows both styles in the existing test module.

---

## Deferred Ideas

Captured in 182-CONTEXT.md `<deferred>` section. Summary:

- Generalize `data-lazy-hero` to other element types (`<audio>`, `<iframe>`, generic defer-load sentinels).
- Dynamic-insertion support via MutationObserver (or `window.setupLazyHeroes` namespace callable).
- Public Rust API for a JSON-UI `Video` component that defaults to emitting `data-lazy-hero` for below-the-fold sources.
- Network-aware `rootMargin` tuning via `navigator.connection.effectiveType`.
- Catalog all existing runtime data-* attributes in the new docs page (rejected: those are implementation details, not public contract).

### Reviewed Todos (not folded)

None — `gsd-tools todo match-phase 182` was not run because the phase directory did not exist when `init phase-op` was invoked. If pending todos surface relevant to Phase 182 during the planning step, the planner can fold them into PLAN.md task descriptions at that point.
