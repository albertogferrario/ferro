---
phase: 182-ferro-json-ui-data-lazy-hero-runtime-primitive
verified: 2026-06-06T13:40:22Z
status: passed
score: 5/5 must-haves verified
overrides_applied: 2
overrides:
  - must_have: "SC-1: Loading any page with <video preload='none' data-lazy-hero> below the fold and scrolling causes preload to flip to 'auto' at the rootMargin boundary"
    reason: "Verified via consumer-side UAT (gestiscilo Phase 186) per CONTEXT.md D-07 and RESEARCH.md §Validation Architecture — verifier confirms the code path exists, not the runtime behavior. Pre-authorized by phase context."
    accepted_by: "phase-context (D-07)"
    accepted_at: "2026-06-06T00:00:00Z"
  - must_have: "SC-4: Runtime IIFE size grows by at most ~400 bytes"
    reason: "Reframed per CONTEXT.md D-10 and RESEARCH.md §585. Sibling-consistent indentation is preserved; raw delta 1455 bytes is pre-authorized in exchange for sibling-shape consistency. Redesign-trigger applies only above ~700 bytes of post-strip meaningful content; post-strip is 914 bytes — D-10 explicitly frames the budget as guideline, not hard fail, and the planner picked option (a) sibling-consistency over option (b) minification."
    accepted_by: "phase-context (D-10)"
    accepted_at: "2026-06-06T00:00:00Z"
re_verification: null
---

# Phase 182: ferro-json-ui `data-lazy-hero` runtime primitive — Verification Report

**Phase Goal (from ROADMAP / CONTEXT):** Extend `ferro-json-ui/src/runtime.rs` with an IntersectionObserver primitive that promotes `<video preload="none">` to `preload="auto"` (and calls `.load()` defensively) when the video crosses a configurable `rootMargin`. A single observer per page fans out to all `[data-lazy-hero]` elements, reading per-element `rootMargin` via `data-lazy-hero-margin="400px 0px"`. Default `200px 0px`. Idempotent via `data-lazy-hero-promoted="1"` marker. The `data-lazy-hero` family is part of the public ferro contract.

**Verified:** 2026-06-06T13:40:22Z
**Status:** passed
**Re-verification:** No — initial verification.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Page with `<video preload="none" data-lazy-hero>` below the fold scrolls → `preload` flips to `"auto"` at the rootMargin boundary | PASSED (override) | Override: Verified via consumer-side UAT (gestiscilo Phase 186) per CONTEXT.md D-07 / RESEARCH §Validation Architecture — accepted by phase-context (D-07). Code path verified: `hero_lazy.rs` §10-29 implements feature-detect + selector + IntersectionObserver per-margin-group setup + promote action (`setAttribute('preload', 'auto')` + `.load()` with try/catch + `unobserve`). |
| 2 | Per-element `data-lazy-hero-margin="400px 0px"` override is honored at observer setup | VERIFIED | `hero_lazy.rs:15` reads `getAttribute('data-lazy-hero-margin') \|\| '200px 0px'`, trims whitespace, buckets into `groups` indexed by margin string. `hero_lazy.rs:18-29` constructs `new IntersectionObserver(..., { rootMargin: key })` per distinct bucket key. String-presence test `runtime_contains_lazy_hero_setup` asserts `"data-lazy-hero-margin"` present in `FERRO_RUNTIME_JS`. |
| 3 | Promoted-marker (`data-lazy-hero-promoted="1"`) prevents double-promotion; re-running is a no-op | VERIFIED | Selector `video[preload="none"][data-lazy-hero]:not([data-lazy-hero-promoted])` (`hero_lazy.rs:11`) excludes already-promoted at setup. Callback double-checks via `!e.target.hasAttribute('data-lazy-hero-promoted')` (`hero_lazy.rs:22`). Marker set via `setAttribute('data-lazy-hero-promoted', '1')` (`hero_lazy.rs:24`). Followed by per-element `obs.unobserve(e.target)` (`hero_lazy.rs:26`). Both setup-time and runtime guards present. |
| 4 | Runtime IIFE size growth budget (~400 bytes target) | PASSED (override) | Override: D-10 frames target as guideline, not hard fail; raw delta 1455 bytes accepted per RESEARCH §585 sibling-consistency tradeoff (option a chosen over option b minification) — accepted by phase-context (D-10). Measured raw JS body 1427 bytes; post-strip meaningful content 914 bytes (above 700 informational threshold but explicitly pre-authorized in verification_context "Pre-authorized" framing). |
| 5 | `ferro-json-ui` publishes 0.2.42 via Wave1A on master push (pre-merge form: workspace version bumped, Cargo.lock synced) | VERIFIED | `Cargo.toml:33` = `version = "0.2.42"`. `Cargo.lock` contains 25 occurrences of `version = "0.2.42"` and 0 occurrences of `version = "0.2.41"`. ferro-stripe pins its own `version = "0.5.0"` (out of workspace inheritance) — 25 of 26 expected. `.github/workflows/publish.yml:211` lists `ferro-json-ui` in `WAVE1A_CRATES`. Publish itself is a post-merge CI artifact (per LAZYHERO-05 / Plan 03 design). |

**Score:** 5/5 truths verified (3 directly + 2 via pre-authorized phase-context override)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-json-ui/src/runtime/hero_lazy.rs` | New file containing `pub(super) const SOURCE` with `setupLazyHeroes` JS body | VERIFIED | File present (36 lines). Contains `setupLazyHeroes` (1×), `IntersectionObserver` (2×: feature-detect + constructor), `data-lazy-hero` family (4 distinct selector forms), `'auto'` literal, `.load()` with `try {…} catch (_) {}`, `unobserve` call, `rootMargin: key`, default `'200px 0px'`. ES5-only: no `forEach`, no `=>`, no `let`/`const`, no backticks in JS body. No `addEventListener('DOMContentLoaded'`, no `window.ferroRuntime`, no `MutationObserver`. |
| `ferro-json-ui/src/runtime/mod.rs` | `mod hero_lazy;` alphabetical entry; `s.push_str(hero_lazy::SOURCE);` in IIFE chain; `setupLazyHeroes();` in dispatcher; tests extended; new `runtime_contains_lazy_hero_setup` test | VERIFIED | `mod hero_lazy;` at line 11 (alphabetical between `form_guards` line 10 and `kanban` line 12). `s.push_str(hero_lazy::SOURCE);` at line 42. `setupLazyHeroes();` in dispatcher at line 57. `bundle_contains_all_setup_functions` array extended (line 159). `dispatcher_invokes_every_setup` array extended (line 192). New test `runtime_contains_lazy_hero_setup` present at lines 198-210, asserts 8 substrings (`setupLazyHeroes`, `data-lazy-hero`, `data-lazy-hero-margin`, `data-lazy-hero-promoted`, `IntersectionObserver`, `preload`, `'auto'`, `unobserve`). |
| `docs/src/json-ui/runtime-primitives.md` | Docs page documenting data-lazy-hero family; neutral voice | VERIFIED | File present (62 lines). 7 mentions of `data-lazy-hero` (selector form), 3 of `data-lazy-hero-margin`, 2 of `data-lazy-hero-promoted`, 3 of `IntersectionObserver`, 1 of `200px 0px`, 2 of `performance, not access control` framing. Negative greps: no `gestiscilo`, no `jetskiadriatic`, no `Phase 182`, no `killer feature`. Sections present: Contract table, Selector, Usage, Observer cardinality, Browser support, Lifecycle, Performance/not access control. |
| `docs/src/SUMMARY.md` | New `- [Runtime Primitives](json-ui/runtime-primitives.md)` entry registered between Plugins and Spec construction | VERIFIED | Line 61 exactly: `- [Runtime Primitives](json-ui/runtime-primitives.md)`. Ordering verified Plugins (60) → Runtime Primitives (61) → Spec construction (62). Single occurrence. |
| `Cargo.toml` | Workspace `version = "0.2.42"` at line 33 | VERIFIED | `Cargo.toml:33` = `version = "0.2.42"`. No remaining `0.2.41` references. |
| `Cargo.lock` | Synced lockfile with 25 ferro-* stanzas at 0.2.42, 0 at 0.2.41 | VERIFIED | 25 × `version = "0.2.42"` present; 0 × `version = "0.2.41"`. ferro-stripe (independent pin 0.5.0) confirmed at `ferro-stripe/Cargo.toml:version = "0.5.0"`. Plan 03 SUMMARY discrepancy (25 vs documented "26") explained — count is correct. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| `runtime/mod.rs` mod-list | `runtime/hero_lazy.rs` | `mod hero_lazy;` declaration | WIRED | `mod.rs:11` declares; module resolves at build (cargo test runs green). |
| `FERRO_RUNTIME_JS` LazyLock initializer | `hero_lazy::SOURCE` constant | `s.push_str(hero_lazy::SOURCE);` chain | WIRED | `mod.rs:42` after `scroll_preserve::SOURCE`. Confirmed by `bundle_contains_all_setup_functions` test pass (asserts `setupLazyHeroes` present in assembled bundle). |
| `ferroRuntime()` dispatcher | `setupLazyHeroes` JS function | dispatcher invocation line | WIRED | `mod.rs:57` line `\x20       setupLazyHeroes();\n\` appears inside the dispatcher push_str block, after `setupToasts();`. Confirmed by `dispatcher_invokes_every_setup` test pass (slices dispatcher region and asserts `setupLazyHeroes();` present). |
| `FERRO_RUNTIME_JS` bundle | All pages via `DefaultLayout` / `DashboardLayout` | `layout.rs::with_runtime()` | WIRED | `layout.rs:313` defines `with_runtime`; lines 316, 337, 369, 395 confirm `FERRO_RUNTIME_JS.as_str()` flows into the `<script>` emission for both layouts. No layout-side changes required (existing channel). |
| `docs/src/SUMMARY.md` | `docs/src/json-ui/runtime-primitives.md` | mdbook TOC link | WIRED | Single registration at line 61. Plan 02 SUMMARY records `mdbook build docs/` ran clean (no orphaned-page warning). |
| `Cargo.toml` workspace.package.version | every ferro-* crate | `version.workspace = true` inheritance | WIRED | 25 / 26 ferro-* stanzas in Cargo.lock now read 0.2.42; ferro-stripe is correctly excluded by its own pin. |
| Cargo.toml + Cargo.lock + `ferro-json-ui/src/runtime/hero_lazy.rs` (new file) | crates.io Wave1A publish | `.github/workflows/publish.yml` `WAVE1A_CRATES` path filter on master push | WIRED (pre-merge) | Workflow file already lists `ferro-json-ui` in WAVE1A_CRATES at line 211. Library-change gate covers `ferro-json-ui/**` paths. No workflow edits required. Post-merge CI artifact. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `FERRO_RUNTIME_JS` LazyLock<String> | Bundle string | 13 `push_str` calls including `hero_lazy::SOURCE` | Yes — string concatenation populates from real per-module SOURCE constants | FLOWING |
| `hero_lazy::SOURCE` | const &str | Compile-time raw string literal | Yes — contains 1427 bytes of real JS function body, not placeholder | FLOWING |
| Dispatcher dispatch | `setupLazyHeroes()` invocation | Inlined JS literal in mod.rs push_str | Yes — invocation present at runtime in browser via DOMContentLoaded callback (verified by `dispatcher_invokes_every_setup` test slicing the dispatcher region) | FLOWING |

No HOLLOW_PROP or STATIC concerns — the runtime bundle is a string-concatenated JS source, and the JS source contains real promotion logic, not a stub function body.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| New test `runtime_contains_lazy_hero_setup` passes | `cargo test -p ferro-json-ui --lib runtime::tests::runtime_contains_lazy_hero_setup` | 1 passed; 0 failed (0.00s) | PASS |
| Aggregate test `bundle_contains_all_setup_functions` includes new primitive | `cargo test -p ferro-json-ui --lib runtime::tests::bundle_contains_all_setup_functions` | included in `runtime::tests` pass | PASS |
| Aggregate test `dispatcher_invokes_every_setup` includes new primitive | `cargo test -p ferro-json-ui --lib runtime::tests::dispatcher_invokes_every_setup` | included in `runtime::tests` pass | PASS |
| Full runtime test suite (regression check) | `cargo test -p ferro-json-ui --lib runtime::tests` | 12 passed; 0 failed | PASS |
| Workspace builds at new version | `cargo build` already exercised by `cargo test` invocation against `ferro-json-ui v0.2.42` | Compiled `ferro-json-ui v0.2.42` cleanly | PASS |
| In-browser preload promotion / network-panel observation (SC-1) | manual UAT in gestiscilo Phase 186 | n/a (out-of-phase) | SKIP (per D-07 / Override 1) |
| Post-merge Wave1A publish (LAZYHERO-05) | GH Actions run on master push + `cargo search ferro-json-ui` | n/a (post-merge CI artifact) | SKIP (per D-08 / Override 5-tied) |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| LAZYHERO-01 | 182-01-PLAN | In-browser preload promotion on viewport approach | SATISFIED (code-ready) | Code path in `hero_lazy.rs:10-33` implements feature-detect → selector → group-by-margin → IntersectionObserver per bucket → promote (setAttribute + setAttribute + load + unobserve). In-browser verification is consumer-side UAT (gestiscilo Phase 186) per D-07. |
| LAZYHERO-02 | 182-01-PLAN, 182-02-PLAN | Per-element `data-lazy-hero-margin` override honored at observer setup | SATISFIED | Group-by-margin bucketing at `hero_lazy.rs:14-17`; `new IntersectionObserver(..., { rootMargin: key })` per bucket at line 29. Documented in `docs/src/json-ui/runtime-primitives.md` contract table and observer-cardinality section. String-presence test asserts attribute name present in bundle. |
| LAZYHERO-03 | 182-01-PLAN, 182-02-PLAN | Idempotency via `data-lazy-hero-promoted` marker | SATISFIED | Selector excludes via `:not([data-lazy-hero-promoted])` at `hero_lazy.rs:11`; callback re-guards via `hasAttribute` at line 22; marker set via `setAttribute(..., '1')` at line 24; `unobserve` at line 26 prevents callback re-fire. Documented in `runtime-primitives.md` Contract row. String-presence test asserts attribute name. |
| LAZYHERO-04 | 182-01-PLAN | IIFE size budget (~400 byte target) | SATISFIED (override) | Raw delta 1455 bytes; meaningful-content delta 914 bytes. Pre-authorized by D-10 (target framed as guideline, not hard fail) and RESEARCH §585 (sibling-consistency outweighs byte-budget guideline; planner option a preferred over option b). |
| LAZYHERO-05 | 182-03-PLAN | ferro-json-ui publishes 0.2.42 via Wave1A on master push | SATISFIED (pre-merge form) | Workspace version bumped to 0.2.42 (Cargo.toml:33). Cargo.lock synced (25 ferro-* stanzas at 0.2.42, 0 at 0.2.41). Wave1A workflow has ferro-json-ui in WAVE1A_CRATES (.github/workflows/publish.yml:211). Publish itself is post-merge CI artifact. |

No orphaned REQ-IDs detected. REQUIREMENTS.md verified — Phase 182 is not enumerated in v12.1 AI REQ-IDs (per RESEARCH §Sources line 743); the LAZYHERO-* IDs are phase-local labels declared in each plan frontmatter and listed against the success criteria.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|

None detected. Scans performed:
- `TODO|FIXME|XXX|HACK|PLACEHOLDER|placeholder|coming soon|not yet implemented` across `hero_lazy.rs`, `mod.rs`, `runtime-primitives.md`: NONE
- Deferred-idea leak scan (`MutationObserver`, `window.ferroRuntime`, `<audio|<iframe`, `navigator.connection`, `effectiveType`): NONE
- ES5-discipline scan (`forEach`, `=>`, `let `, `const `, backticks in JS body): NONE in JS body (matches present in Rust polarity-note comment only, which is correct).
- Tenant-identity scan (`gestiscilo`, `jetskiadriatic`): NONE in source or docs.
- Marketing-voice scan (`killer feature`): NONE in docs.
- Co-author / Claude attribution scan in last 11 phase commits (`Co-Authored-By`, `Generated with`, `🤖`): NONE.

### Human Verification Required

None for the ferro-side deliverable. The two manual-UAT items below are explicitly out-of-phase per CONTEXT.md D-07 and D-08, and are tracked separately:

1. **(Cross-phase UAT, gestiscilo Phase 186)** In-browser network-panel verification of preload promotion on viewport approach against a live tenant page after 0.2.42 publishes. Out of ferro Phase 182 deliverable scope.
2. **(Post-merge CI artifact)** Confirm GH Actions Wave1A run completes after master push and `cargo search ferro-json-ui` returns 0.2.42. Out of pre-merge verification scope.

These are NOT verification gaps for Phase 182 itself — both are explicitly scheduled-elsewhere per the phase contract.

### Gaps Summary

No gaps. All five success criteria are either directly verified (SC-2, SC-3, SC-5-pre-merge) or covered by pre-authorized phase-context overrides for verification routed elsewhere (SC-1 via consumer-side UAT per D-07; SC-4 via D-10 budget reframing). The deliverable is complete to the ferro-side contract:

- The runtime primitive `setupLazyHeroes` is implemented in `hero_lazy.rs` and wired into `FERRO_RUNTIME_JS` via `mod.rs`, the assembly chain, the dispatcher block, and three string-presence tests.
- The public DOM-attribute contract (`data-lazy-hero`, `data-lazy-hero-margin`, `data-lazy-hero-promoted`) is documented in a new `docs/src/json-ui/runtime-primitives.md` page registered in `docs/src/SUMMARY.md`.
- The workspace version is bumped to 0.2.42 with Cargo.lock synchronized, and the existing Wave1A workflow handles publication on master push.

All ten locked decisions (D-01 through D-10) are honored:
- **D-01:** One observer per distinct `rootMargin` — implemented via group-by-string bucketing at `hero_lazy.rs:14-17`.
- **D-02:** video-only, preload=none, not-promoted selector — `hero_lazy.rs:11`.
- **D-03:** one-shot at DOMContentLoaded, no MutationObserver — no `MutationObserver` reference; runtime relies on the existing outer dispatcher.
- **D-04:** per-element `unobserve`, no observer disconnect — `hero_lazy.rs:26`, no `disconnect()` call.
- **D-05:** feature-detect early-return — `hero_lazy.rs:10` with inverse polarity vs `plugins/map.rs` §306 (Rust comment block explains polarity choice).
- **D-06:** module at `runtime/hero_lazy.rs`, camelCase plural `setupLazyHeroes` — both present.
- **D-07:** string-presence tests, no headless-browser tests — only test additions are string-presence assertions; SC-1 routed to consumer UAT (override 1).
- **D-08:** workspace version bump only, no manual `cargo publish` — Plan 03 only edits Cargo.toml + Cargo.lock; no publish invocation locally.
- **D-09:** new docs page + SUMMARY.md registration — both present.
- **D-10:** size budget treated as guideline; overshoot pre-authorized — explicit override frontmatter entry.

No leaked deferred ideas. No tenant identity in shipped source or docs. No co-author or Claude attribution lines in phase commits.

---

*Verified: 2026-06-06T13:40:22Z*
*Verifier: Claude (gsd-verifier)*
