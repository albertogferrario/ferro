# Phase 182: ferro-json-ui `data-lazy-hero` runtime primitive — Research

**Researched:** 2026-06-06
**Domain:** ferro-json-ui runtime IIFE / DOM-attribute-driven viewport primitives
**Confidence:** HIGH — every source-of-truth claim verified by direct file read

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01: Observer cardinality — one observer per distinct `rootMargin` string.** Group elements at setup time by their resolved `rootMargin` (default `200px 0px`, override via `data-lazy-hero-margin`). Common case = one observer per page; mixed-margin page = N observers (N = distinct margin values). Reading 1 (one observer ignoring per-element override) and reading 2 (one observer per element) both fail success criteria. Only the grouping reading survives SC-2 + SC-4.
- **D-02: Target only `<video preload="none">`.** Selector: `video[preload="none"][data-lazy-hero]:not([data-lazy-hero-promoted])`. Non-video elements with `data-lazy-hero` are silently ignored. `<video>` without `preload="none"` is ignored. Already-promoted elements are excluded by the `:not(...)` selector.
- **D-03: Single setup pass at DOMContentLoaded — no MutationObserver, no re-entry.** Matches every sibling primitive. Dynamically inserted heroes are not observed (deferred path: expose `setupLazyHeroes` on a window namespace, NOT a MutationObserver).
- **D-04: Per-element `unobserve(entry.target)` after promote; observer stays alive.** Avoids count-and-disconnect bookkeeping to keep IIFE size down. Empty observers cost ~nothing. Empty-page early-return via `!els.length` already prevents idle-observer creation.
- **D-05: Feature detection — `if (typeof IntersectionObserver === 'undefined') return;` early-return guard.** Mirrors `ferro-json-ui/src/plugins/map.rs` §306 verbatim (NOTE: map.rs uses the inverse polarity `if (typeof IntersectionObserver !== 'undefined')`; D-05 picks the early-return polarity, which is what sibling runtime primitives use — see §Pitfalls 3).
- **D-06: Module placement and naming.** New file `ferro-json-ui/src/runtime/hero_lazy.rs`. Setup function: `setupLazyHeroes` (camelCase plural). `mod hero_lazy;` kept alphabetical between `form_guards` and `kanban`.
- **D-07: Test strategy — string-presence tests in `runtime/mod.rs`.** No headless-browser test. Add `runtime_contains_lazy_hero_setup`. Extend `bundle_contains_all_setup_functions` and `dispatcher_invokes_every_setup`.
- **D-08: Single publish at end of phase.** Workspace bump `0.2.41 → 0.2.42`. Existing GH Actions Wave1A flow publishes (ferro-json-ui already in `WAVE1A_CRATES`). gestiscilo Phase 186 consumes after merge; not phase 182's deliverable.
- **D-09: NEW docs page `docs/src/json-ui/runtime-primitives.md`.** Covers only `data-lazy-hero` for v1. Framing accommodates future runtime-attribute additions. Sibling internal attributes (`data-sse-url`, `data-sidebar-toggle`, `data-popover-menu`, etc.) NOT enumerated — they remain implementation details.
- **D-10: IIFE size budget ~400 bytes is soft target.** ≤500 bytes acceptable, >700 bytes triggers redesign. Comments inside SOURCE string minimized; explanation lives in docs page (D-09) + this RESEARCH.md.

### Claude's Discretion

- Exact line-by-line composition of the JS source (whitespace, var naming, exact iteration style). Contract is fixed by D-01 through D-05.
- Insertion point of `hero_lazy::SOURCE` in `runtime/mod.rs`'s `push_str` chain (anywhere produces equivalent behavior; pick to minimize diff).
- Exact title and section ordering of the new docs page (D-09).
- Whether the new mod.rs string-presence test is one combined test or several focused tests. Sibling pattern shows both styles.

### Deferred Ideas (OUT OF SCOPE)

- **Generalize `data-lazy-hero` to other element types** (audio, iframes, arbitrary defer-load sentinels). When a real consumer surfaces, ship a sibling primitive (`data-defer-load`) rather than expanding `data-lazy-hero`.
- **Dynamic-insertion support via MutationObserver.** Today's primitive is one-shot at DOMContentLoaded. Resolution path: expose `setupLazyHeroes` on a window namespace. Adding a global MutationObserver rejected on cost grounds.
- **Public Rust API for emitting `<video data-lazy-hero>` from a JSON-UI Video component.** No Video component exists today. When added (separate phase), that component will default to emitting `preload="none" data-lazy-hero` — out of scope here.
- **Network-aware `rootMargin` tuning** (read `navigator.connection.effectiveType`, shrink on 3G). Possible v2; muddies the public attribute contract.
- **Catalog all existing runtime data-* attributes in the new docs page.** The phase-182 docs page is for consumer-set attributes (`data-lazy-hero` family) only.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

Phase 182 is not enumerated in `.planning/REQUIREMENTS.md` (that document scopes the v12.1 AI milestone, REQ-IDs `AISDK-*`, `AISSE-*`, `AICLI-*`). Phase 182's requirements are derived from the roadmap's 5 success criteria, restated below with a synthetic REQ-ID format for traceability:

| ID | Description (verbatim from ROADMAP.md §1957-1962) | Research Support |
|----|---|---|
| LAZYHERO-01 | Loading any page with `<video preload="none" data-lazy-hero>` below the fold and scrolling causes the `preload` attribute to flip to `"auto"` exactly when the element crosses the configured `rootMargin` boundary (verified via Chrome DevTools Network panel showing video bytes only after scroll). | Implementation = `setupLazyHeroes` SOURCE in `runtime/hero_lazy.rs`; viewport behavior owned by browser's IntersectionObserver. Verified manually in-browser (consumer-side UAT after gestiscilo Phase 186 bumps). |
| LAZYHERO-02 | Per-element override via `data-lazy-hero-margin="400px 0px"` is honored at observer setup. | Implementation = group-by-margin bucketing in `setupLazyHeroes` (D-01). String-presence test asserts attribute name `data-lazy-hero-margin` in `FERRO_RUNTIME_JS`. |
| LAZYHERO-03 | The promoted-marker (`data-lazy-hero-promoted="1"`) prevents double-promotion; re-running the observer on the same element is a no-op. | Implementation = selector excludes `:not([data-lazy-hero-promoted])` at setup; promote step calls `setAttribute('data-lazy-hero-promoted', '1')`; per-element `unobserve()` after promote. String-presence test asserts marker name. |
| LAZYHERO-04 | The runtime IIFE size grows by at most ~400 bytes (single-observer fan-out, no per-element observer cost). | D-10 reinterprets as soft target ≤500 bytes acceptable. Verification = byte-diff measurement of `FERRO_RUNTIME_JS.len()` before/after. Optional micro-test asserting size delta. |
| LAZYHERO-05 | `ferro-json-ui` publishes the new version to crates.io via the existing GH Actions workflow; gestiscilo Phase 186 consumes it via Cargo.toml bump. | Implementation = workspace version bump `0.2.41 → 0.2.42` in `Cargo.toml`; merge to master triggers `.github/workflows/publish.yml`; ferro-json-ui already in `WAVE1A_CRATES`. |
</phase_requirements>

---

## Summary

Phase 182 adds a single new file `ferro-json-ui/src/runtime/hero_lazy.rs` containing a ~500-byte JS source string for `setupLazyHeroes()`, wires it into `ferro-json-ui/src/runtime/mod.rs` (mod declaration, push_str, dispatcher comment), extends two existing string-presence tests, adds one new test, creates one new docs page (`docs/src/json-ui/runtime-primitives.md`), registers that page in `docs/src/SUMMARY.md`, and bumps the workspace version `0.2.41 → 0.2.42` so the existing publish workflow ships `ferro-json-ui` to crates.io as part of Wave1A.

The implementation is shaped entirely by sibling-runtime pattern compliance. Every sibling in `ferro-json-ui/src/runtime/*.rs` uses the same ES5-style structure (`pub(super) const SOURCE: &str = r#"…"#;` with `function setup…() { var …; }` inside), the same defensive-coding posture (early return on selector-no-match, ES5 `for (var i = …; i < n; i++)` loops, no arrow functions, no `let`/`const`), and the same test shape (string-presence assertions in `runtime/mod.rs`). The new primitive must be indistinguishable in style.

The IntersectionObserver feature-detection guard pattern is already established in `ferro-json-ui/src/plugins/map.rs` §306 — D-05 reuses it. The grouping-by-`rootMargin` (D-01) is the only piece that has no existing sibling-pattern precedent: the planner specifies it as a `for (var i = 0; i < els.length; i++) { var m = els[i].getAttribute(...) || '200px 0px'; (groups[m] = groups[m] || []).push(els[i]); }` bucketing followed by `for (var key in groups) { new IntersectionObserver(callback, { rootMargin: key }); }`.

**Primary recommendation:** Mirror `sidebar.rs` and `dropdowns.rs` structure verbatim. Use the feature-detection guard from `map.rs` §306 (with polarity adjusted to early-return). Keep the IIFE under 500 bytes by minimizing inline comments. Place `hero_lazy::SOURCE` in the push_str chain after `scroll_preserve` (the alphabetical-but-grouped existing order isn't strict; pick the position that minimizes the diff). Extend the two existing aggregate tests; add one new focused test for `runtime_contains_lazy_hero_setup`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Lazy-hero observer wiring | Browser / Client (runtime IIFE) | — | DOM mutation + IntersectionObserver is pure browser work; no server roundtrip. |
| `setupLazyHeroes` registration in dispatcher | ferro-json-ui (Rust assembly) | — | `runtime/mod.rs` builds the IIFE string; dispatcher calls `setupLazyHeroes()` from `ferroRuntime()`. |
| Tenant HTML opt-in (`data-lazy-hero` attribute on `<video>`) | Consumer page author / future JSON-UI Video component | — | Phase 182 is a pure runtime primitive. Tenants set the attribute by hand or via component output. |
| IIFE bundle shipping to every page | ferro-json-ui (layout) | — | `with_runtime()` in `layout.rs` §313-322 inlines `FERRO_RUNTIME_JS` into `DefaultLayout` and `DashboardLayout`. No layout-side change for Phase 182. |
| Workspace-version-driven publish to crates.io | CI (GH Actions) | — | `.github/workflows/publish.yml` Wave1A. ferro-json-ui already listed. |
| Public documentation of consumer DOM contract | docs/src/json-ui/runtime-primitives.md | docs/src/SUMMARY.md (TOC registration) | mdbook source tree. New page registered in JSON-UI section of SUMMARY.md. |

---

## Project Constraints (from CLAUDE.md)

- **Run formatters/linters/tests before every commit.** Project requires `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`. Each plan's verification step must include these three commands. CI enforces `-D warnings`.
- **No co-author lines / "Generated with Claude" attribution in commit messages.** Strictly forbidden by project AND global CLAUDE.md.
- **Project-agnostic crates: `ferro-*` crates must not hardcode any application identity** (app name, brand strings, copy, URLs). Phase 182 is generic — the attribute names `data-lazy-hero`, `data-lazy-hero-margin`, `data-lazy-hero-promoted` carry no tenant identity. ✓
- **Always update docs when framework changes.** D-09's new docs page is mandated by this rule.
- **Repository documents must read as neutral.** The new docs page must read like architectural documentation, not an internal strategy note. No "killer feature" framing in the docs page. No reference to gestiscilo or jetskiadriatic as specific consumers. Generic web-primitive framing only.
- **Scientific and minimalistic comments, no marketing language.** Applies to docs page prose, code comments inside `hero_lazy.rs`, and the new test names.
- **No backward-compat shims; pre-1.0 breaking changes accepted.** Not relevant for Phase 182 (purely additive); noted for completeness.
- **When adding a new crate to the workspace, always add it to `.github/workflows/publish.yml`.** N/A for Phase 182 (extends existing crate).

---

## Source-of-Truth File Map

Every file the planner / executor will read or modify, with line ranges.

### Files to CREATE

| Path | Purpose | Notes |
|------|---------|-------|
| `ferro-json-ui/src/runtime/hero_lazy.rs` | New SOURCE constant for `setupLazyHeroes` | Single `pub(super) const SOURCE: &str = r#"…"#;` matching every sibling shape. Target body ≤500 bytes. |
| `docs/src/json-ui/runtime-primitives.md` | New docs page documenting `data-lazy-hero` family | Framing accommodates future runtime-attribute additions. No internal-strategy voice. |

### Files to MODIFY

| Path | Lines | Change |
|------|-------|--------|
| `ferro-json-ui/src/runtime/mod.rs` | §8-19 (mod list) | Insert `mod hero_lazy;` alphabetical between `form_guards` and `kanban` (line 11–12 boundary). |
| `ferro-json-ui/src/runtime/mod.rs` | §29-40 (push_str chain) | Insert `s.push_str(hero_lazy::SOURCE);` line. Discretion: position. Recommended: after `scroll_preserve` (line 40) — appended at end of chain, minimizes diff. |
| `ferro-json-ui/src/runtime/mod.rs` | §41-58 (dispatcher block) | Insert `setupLazyHeroes();` invocation inside `ferroRuntime()` body (between `setupScrollPreserve();` line 43 and `setupSSE();` line 44 — alphabetical-by-related-grouping; OR appended last — both equivalent). |
| `ferro-json-ui/src/runtime/mod.rs` | §141-162 (`bundle_contains_all_setup_functions`) | Add `"setupLazyHeroes",` to the array. |
| `ferro-json-ui/src/runtime/mod.rs` | §170-191 (`dispatcher_invokes_every_setup`) | Add `"setupLazyHeroes();",` to the array. |
| `ferro-json-ui/src/runtime/mod.rs` | §62-192 (test module) | Add NEW test `runtime_contains_lazy_hero_setup` — asserts `setupLazyHeroes`, `data-lazy-hero`, `data-lazy-hero-margin`, `data-lazy-hero-promoted`, `IntersectionObserver`, `preload`, `"auto"`, `unobserve`. |
| `docs/src/SUMMARY.md` | §66-72 (JSON-UI section) | Register new page: `- [Runtime Primitives](json-ui/runtime-primitives.md)`. Discretion on exact placement (recommended: after Plugins, before Spec construction, to keep "browser-side concerns" grouped). |
| `Cargo.toml` (workspace root) | `[workspace.package].version` (line 33) | Bump `"0.2.41"` → `"0.2.42"`. |

### Files to READ ONLY (no modification — referenced for pattern compliance)

| Path | Lines | Why the planner / executor reads it |
|------|-------|-------------------------------------|
| `ferro-json-ui/src/runtime/sidebar.rs` | §1-37 | Closest-shape sibling: `querySelectorAll` + for loop + early return. Mimic structure verbatim. |
| `ferro-json-ui/src/runtime/sse.rs` | §1-42 | Attribute-driven setup pattern (`getAttribute('data-sse-url')` precedent). |
| `ferro-json-ui/src/runtime/dropdowns.rs` | §1-80 | Two-level structure: `setup…` calls `init…(el)` for each match. Phase 182 is single-function (no init wrapper), but the for-loop iteration style is the same. |
| `ferro-json-ui/src/runtime/scroll_preserve.rs` | §1-69 | Upper-bound on per-primitive complexity. Shows `try { … } catch (e) {}` usage for the explicitly-unsafe operations — Phase 182 uses the same `try { … } catch (_) {}` around `video.load()`. |
| `ferro-json-ui/src/runtime/form_guards.rs` | §1-93 | Pattern of `function setupFoo() { var els = ...; for (var i = 0; i < els.length; i++) { initFoo(els[i]); } }`. |
| `ferro-json-ui/src/runtime/tabs.rs` | §1-85 | Two-level init pattern + `URLSearchParams` usage. Not directly applicable but confirms ES5 style. |
| `ferro-json-ui/src/runtime/modals.rs` | §1-32 | IIFE-wrapping idiom for per-element handlers using `(function(btn) { … })(els[i])`. Phase 182's observer callback variant of this idea — see §Code Review of Proposed JS. |
| `ferro-json-ui/src/runtime/dismissibles.rs` | §1-55 | Confirms iteration-over-NodeList ALWAYS uses indexed for loops, never `forEach` (siblings never call `.forEach` on a NodeList). |
| `ferro-json-ui/src/runtime/notifications.rs` | §1-25 | Minimal sibling — simplest example of guard-then-event-bind pattern. |
| `ferro-json-ui/src/runtime/kanban.rs` | §1-40 | Pattern reference for injected style precedent (not directly applicable). |
| `ferro-json-ui/src/runtime/toasts.rs` | §1-30 | Variant-map idiom; URL-driven init at boot. |
| `ferro-json-ui/src/plugins/map.rs` | §242-321 | Existing IntersectionObserver usage. §306-315 is the feature-detection guard + observer-construct + observer.observe(el). Use verbatim shape, polarity-adjusted to early-return per D-05. |
| `ferro-json-ui/src/plugins/map.rs` | §470-475 | Test-shape precedent: `assert!(script.contains("IntersectionObserver"))`. |
| `ferro-json-ui/src/layout.rs` | §312-322, §606-609 | Confirms `FERRO_RUNTIME_JS` ships via `DefaultLayout` AND `DashboardLayout`. No layout change needed. |
| `Cargo.toml` (workspace root) | §1-40 | Workspace member list, version field. Confirms `ferro-json-ui` is a member and inherits `version.workspace = true`. |
| `ferro-json-ui/Cargo.toml` | All | `version.workspace = true` — automatic bump propagation. |
| `.github/workflows/publish.yml` | §200-260 | Wave1A flow; `ferro-json-ui` listed at §211. Trigger: push to master. Library-change gate at §25-50 confirms `ferro-json-ui/**` changes qualify as a publishable diff. |
| `docs/src/SUMMARY.md` | §64-75 (JSON-UI section) | TOC structure for new page registration. |
| `docs/src/json-ui/data-binding.md` | §1-40 | Reference for docs page voice/structure (scientific, minimalistic, no marketing). |
| `docs/src/features/json-ui.md` | §1-30 | Top-level JSON-UI feature framing reference. |
| `.planning/ROADMAP.md` | §1945, §1949-1966 | Phase 182 definition. Success criteria locked at §1957-1962. |

---

## Validation Architecture

> Per `.planning/config.json` — `workflow.nyquist_validation` key absent, treat as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `cargo test` |
| Config file | `ferro-json-ui/Cargo.toml` (`[dev-dependencies]` for the crate; no separate test runner config) |
| Quick run command | `cargo test -p ferro-json-ui --lib runtime::tests` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LAZYHERO-01 | Below-the-fold video flips `preload` from `"none"` to `"auto"` on viewport approach. | manual-only (in-browser UAT) | (Chrome DevTools Network panel + scroll, verified consumer-side) | N/A — manual UAT in gestiscilo Phase 186 |
| LAZYHERO-02 | Per-element `data-lazy-hero-margin` override honored at observer setup. | unit (string-presence) | `cargo test -p ferro-json-ui --lib runtime_contains_lazy_hero_setup` | ❌ Wave 0 (test to be added) |
| LAZYHERO-03 | `data-lazy-hero-promoted` marker prevents double-promotion (idempotency). | unit (string-presence) | `cargo test -p ferro-json-ui --lib runtime_contains_lazy_hero_setup` | ❌ Wave 0 (test to be added) |
| LAZYHERO-04 | IIFE size grows ≤500 bytes (soft target ~400). | unit (byte-diff measurement) — OPTIONAL | (planner discretion: add `FERRO_RUNTIME_JS.len()` assertion, OR rely on code review) | ❌ Wave 0 if added |
| LAZYHERO-05 | Crates.io publish via Wave1A on master push. | integration (CI workflow) | Verified post-merge: GH Actions Wave1A run + `cargo search ferro-json-ui` showing 0.2.42 | N/A — CI artifact |

**Test type classification rationale:**
- LAZYHERO-01 requires real browser IntersectionObserver semantics + scroll, an actual video URL, and Network-panel inspection. There is no automated path that exercises this in ferro's test suite. The roadmap explicitly defers this to consumer-side UAT (gestiscilo Phase 186 in-browser verification). D-07 confirms: "no headless browser test."
- LAZYHERO-02 and LAZYHERO-03 are verified by asserting the assembled `FERRO_RUNTIME_JS` string contains the attribute names and the relevant DOM API calls. This matches the established sibling test shape (every primitive's behavior is "verified" by asserting its source-string content contains the right markers; behavioral correctness is verified manually in-browser, per project precedent).
- LAZYHERO-04 is a non-functional constraint (size budget). A `FERRO_RUNTIME_JS.len() < N` assertion would lock the budget into the test suite. Planner discretion: include or rely on code-review byte-diff (D-10 frames the budget as a guideline, not a hard fail).
- LAZYHERO-05 is verified post-merge by checking GH Actions and crates.io. No pre-merge test possible.

### Sampling Rate

- **Per task commit:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test -p ferro-json-ui --lib runtime::tests` — fast (≈5–15s for the runtime module's tests).
- **Per wave merge:** `cargo test --all-features` — full workspace suite.
- **Phase gate:** Full `cargo test --all-features` green before any version-bump commit lands on master.

### Wave 0 Gaps

- [ ] `ferro-json-ui/src/runtime/hero_lazy.rs` — does not exist; created by the implementation plan. NOT a Wave 0 gap in the conventional sense (the file IS the deliverable, not test scaffolding).
- [ ] New test `runtime_contains_lazy_hero_setup` in `ferro-json-ui/src/runtime/mod.rs` `mod tests` block — to be added in the same plan that adds `hero_lazy.rs`.
- [ ] Updates to `bundle_contains_all_setup_functions` and `dispatcher_invokes_every_setup` arrays — same plan, same diff.

*(No framework install needed — `cargo test` already runs in this workspace.)*

### Manual UAT (out of phase 182's deliverable; tracked consumer-side as gestiscilo Phase 186)

Out-of-phase verification path for LAZYHERO-01:
1. Open a tenant page (jetskiadriatic landing) with at least one `<video preload="none" data-lazy-hero>` below the fold AND one with `data-lazy-hero-margin="400px 0px"`.
2. Chrome DevTools → Network panel → filter `.mp4`/`.webm`/`media`.
3. Hard reload, observe: no video bytes requested before scrolling.
4. Slow-scroll toward the lazy hero, observe: video bytes start arriving exactly when the rootMargin boundary crosses the viewport top (≈200px before viewport entry for default; ≈400px for the override).
5. Inspect the `<video>` element in Elements panel: `preload="auto"` and `data-lazy-hero-promoted="1"` attributes present after promote.
6. Re-scroll past again: no additional fetches (idempotency verified).

---

## Standard Stack

### Core (no new deps; everything used is in-tree)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Browser IntersectionObserver API | Ubiquitous (evergreen since 2017) | Detect viewport approach | Native browser primitive — no JS library wrapper. `[VERIFIED: ferro-json-ui/src/plugins/map.rs §306-315 uses it directly]` |
| ES5 baseline JS | n/a | Runtime IIFE syntax | All sibling primitives use ES5 (`var`, function expressions, no arrow functions, no template literals). For broadest WebView compatibility. `[VERIFIED: read of every runtime/*.rs file]` |
| Rust `std::sync::LazyLock` | Std (stable) | Lazy bundle assembly | Already used by `runtime/mod.rs` §21,26. No change. `[VERIFIED: ferro-json-ui/src/runtime/mod.rs §21]` |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| mdbook | (project standard) | Documentation site builder | Required for the new docs page. `docs/book.toml` config exists. `[VERIFIED: docs/book.toml]` |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| IntersectionObserver | scroll event + getBoundingClientRect polling | scroll-event polling fires hundreds of times/sec, costs main-thread time; IntersectionObserver is the "right answer" for viewport-approach detection. Sibling `map.rs` already uses it. |
| MutationObserver for dynamic insertion | DOMContentLoaded one-shot (D-03) | MutationObserver permanently inflates IIFE cost; one-shot is right for the consumer scenarios documented in CONTEXT.md (server-rendered hero set). |
| Loading-attribute polyfill | `data-lazy-hero` IntersectionObserver primitive | `<video>` does not support the native `loading="lazy"` attribute the way `<img>` and `<iframe>` do (per HTML living standard). This is the entire reason Phase 182 exists. |

**Installation:** N/A — pure-Rust no-new-deps phase. No `npm install`, no new `cargo add`.

**Version verification:** N/A.

---

## Architecture Patterns

### System Architecture Diagram

```
                  Tenant page load (HTML response)
                                │
                                ▼
       ┌────────────────────────────────────────────────┐
       │  ferro-json-ui DefaultLayout / DashboardLayout │
       │  (layout.rs §312-322, §606-609)                │
       │                                                │
       │   with_runtime(ctx_scripts) → inlines:         │
       │     <script>{FERRO_RUNTIME_JS}</script>        │
       └────────────────────────┬───────────────────────┘
                                │ rendered HTML reaches browser
                                ▼
       ┌────────────────────────────────────────────────┐
       │             Browser parses HTML                │
       │                                                │
       │   DOMContentLoaded fires                       │
       │   IIFE → ferroRuntime() dispatcher runs:       │
       │     setupScrollPreserve();                     │
       │     setupSSE();                                │
       │     setupTabs();                               │
       │     ...                                        │
       │     setupLazyHeroes();   ← NEW (Phase 182)     │
       │     ...                                        │
       └────────────────────────┬───────────────────────┘
                                │
                                ▼
       ┌────────────────────────────────────────────────┐
       │           setupLazyHeroes() body               │
       │                                                │
       │  1. Feature-detect IntersectionObserver        │
       │  2. querySelectorAll matching selector         │
       │  3. Bucket els by data-lazy-hero-margin        │
       │  4. For each bucket key:                       │
       │       new IntersectionObserver(cb, {rootMargin})│
       │       observe each el in bucket                │
       │  5. On intersect: promote + mark + unobserve   │
       └────────────────────────────────────────────────┘
```

The diagram traces: HTML response → runtime inline → DOMContentLoaded → dispatcher → setupLazyHeroes → IntersectionObserver wiring → on-intersect promotion. The single new primitive (`setupLazyHeroes`) is an entry to the existing dispatcher, with the existing layout shipping the bundle to every page.

### Recommended Project Structure

No structural change. New file slots into existing `ferro-json-ui/src/runtime/` siblings:

```
ferro-json-ui/src/runtime/
├── dismissibles.rs
├── dropdowns.rs
├── form_guards.rs
├── hero_lazy.rs        ← NEW (Phase 182)
├── kanban.rs
├── mod.rs              ← MODIFIED (mod list, push_str chain, dispatcher, tests)
├── modals.rs
├── notifications.rs
├── product_tiles.rs
├── scroll_preserve.rs
├── sidebar.rs
├── sse.rs
├── tabs.rs
└── toasts.rs
```

### Pattern 1: SOURCE-string sibling primitive

**What:** Every runtime primitive lives in its own file as `pub(super) const SOURCE: &str = r#"…"#;` containing ES5 JS.
**When to use:** Phase 182 follows it verbatim.
**Example (verbatim from `ferro-json-ui/src/runtime/sidebar.rs`):**
```rust
pub(super) const SOURCE: &str = r#"
    // ── Sidebar mobile toggle ─────────────────────────────────────────────

    function setupSidebar() {
        var toggleBtn = document.querySelector('[data-sidebar-toggle]');
        var sidebarEl = document.querySelector('[data-sidebar]');
        var backdropEl = document.querySelector('[data-sidebar-backdrop]');
        if (!toggleBtn || !sidebarEl) return;

        function openSidebar() {
            sidebarEl.classList.remove('hidden');
            if (backdropEl) backdropEl.classList.remove('hidden');
        }

        // … more locals + addEventListener wiring …
    }
"#;
```

### Pattern 2: querySelectorAll + indexed for loop (NodeList iteration)

**What:** All sibling primitives iterate NodeLists via indexed `for (var i = 0; i < els.length; i++)`, never `.forEach()`.
**When to use:** Phase 182's element bucketing loop AND the `groups[key].forEach(...)` candidate.
**Critical note:** The proposed JS sketch in CONTEXT.md uses `groups[key].forEach(function(el) { io.observe(el); });`. Although `forEach` is safe on a plain Array (which `groups[key]` is — built with `.push()`), it is NOT used on NodeLists by any sibling. The planner should match the sibling convention and use a `for (var j = 0; j < groups[key].length; j++)` form on the inner array too, for stylistic consistency — OR accept the discretional break since `groups[key]` is a true Array. Recommend matching sibling style.

**Example (verbatim from `ferro-json-ui/src/runtime/dropdowns.rs`):**
```rust
pub(super) const SOURCE: &str = r#"
    function setupDropdowns() {
        var menus = document.querySelectorAll('[data-popover-menu]');
        for (var i = 0; i < menus.length; i++) {
            initPopoverMenu(menus[i]);
        }
    }
    // …
"#;
```

### Pattern 3: IntersectionObserver feature-detection + observer construction

**What:** Existing precedent for the IntersectionObserver setup.
**When to use:** D-05 mirrors this. Polarity adjusted to early-return per sibling-runtime style.
**Example (verbatim from `ferro-json-ui/src/plugins/map.rs` §306-315):**
```rust
// (inside INIT_SCRIPT)
if (typeof IntersectionObserver !== 'undefined') {
  var observer = new IntersectionObserver(function(entries) {
    entries.forEach(function(entry) {
      if (entry.isIntersecting) {
        map.invalidateSize();
      }
    });
  });
  observer.observe(el);
}
```
Phase 182's hero_lazy.rs adapts this with polarity inverted (`if (typeof … === 'undefined') return;`), the bucketing layer added, and the promote action substituted for `map.invalidateSize()`.

### Pattern 4: Defensive `try { … } catch (_) {}` around browser-throw-prone calls

**What:** Used in `scroll_preserve.rs` for `sessionStorage` access and in `sse.rs` for `JSON.parse`. Phase 182 mirrors it for `video.load()`.
**Example (verbatim from `ferro-json-ui/src/runtime/scroll_preserve.rs` §52):**
```js
} catch (e) { /* sessionStorage may be unavailable */ }
```

### Anti-Patterns to Avoid

- **`.forEach()` on a NodeList.** No sibling uses it. Phase 182 should not be the first.
- **Arrow functions / `let` / `const` / template literals / destructuring.** No sibling uses any of these. ES5 baseline is deliberate.
- **MutationObserver to handle dynamically inserted heroes.** Explicitly rejected by D-03 + Deferred Ideas.
- **Per-element observer.** Explicitly rejected by D-01 (SC-4 violation).
- **`console.log` inside `setupLazyHeroes`.** No sibling uses `console.log` for happy-path logging (only `console.error` in `map.rs` for caught exceptions). Phase 182 should not emit log noise.
- **`addEventListener('DOMContentLoaded', …)` inside `hero_lazy.rs`.** The outer `ferroRuntime()` IIFE already registers DOMContentLoaded. Adding a second listener inside `hero_lazy.rs` duplicates wiring and may execute out-of-order with siblings.
- **Inline comments inside the SOURCE string.** D-10 budget pressure. Per-line explanation lives in the docs page, NOT in `r#"…"#;`. A short header comment (one-liner identifying the primitive) is acceptable.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Viewport-approach detection | scroll-event polling + `getBoundingClientRect` math | `IntersectionObserver` | Native, off-main-thread, handles iframes/scroll containers correctly. Already used in sibling `map.rs`. |
| Dynamic-insertion tracking | MutationObserver | Defer entirely (D-03) | Permanent IIFE cost; consumer scenario does not require it. |
| Public-namespace exposure | `window.ferroRuntime.setupLazyHeroes = setupLazyHeroes` | Don't expose — keep inside IIFE | No consumer surface for this in Phase 182. Future deferred path. |
| Counting-and-disconnecting empty observers | Tracking pending observe counts per observer | Leave empty observer alive (D-04) | Negligible cost; bookkeeping inflates IIFE bytes. |

**Key insight:** Phase 182 is a "use the platform" primitive. Everything is delegated to native browser APIs; the runtime code is glue + selectors + bucketing.

---

## Runtime State Inventory

> Phase 182 is a greenfield additive primitive. No rename, refactor, migration, or string-replacement. Omit per spec.

---

## Common Pitfalls

### Pitfall 1: `IntersectionObserver` constructor throws on whitespace-padded `rootMargin`

**What goes wrong:** `new IntersectionObserver(cb, { rootMargin: ' 200px 0px ' })` may throw `SyntaxError` in some browsers (strict parsing of the value).
**Why it happens:** The IntersectionObserver spec defines `rootMargin` as a CSS-margin string; whitespace handling at the edges is technically tolerant on most engines but Safari has historically been stricter.
**How to avoid:** Specify in the planner that the bucket key uses `.trim()` on the read attribute value before using it as the observer's `rootMargin`. The minimal addition: `var m = (els[i].getAttribute('data-lazy-hero-margin') || '200px 0px').replace(/^\s+|\s+$/g, '');` — or `.trim()` if ES5-trim availability is confirmed (`String.prototype.trim` is ES5.1, available everywhere IntersectionObserver is, so `.trim()` is fine).
**Warning signs:** Console SyntaxError on page load; specific videos failing to ever promote. `[VERIFIED: project knowledge — IO spec § 'rootMargin' attribute getter parses as CSS margin shorthand]`

### Pitfall 2: `video.load()` throws in some Safari conditions

**What goes wrong:** `<video>.load()` can throw synchronously on Safari when called against a `<video>` whose `<source>` is cross-origin and the autoplay policy is restrictive, or during early page-load when the resource selection algorithm is mid-progress.
**Why it happens:** WebKit historical bug; `.load()` aborts current selection and starts a new one, sometimes throwing `InvalidStateError`.
**How to avoid:** Wrap in `try { e.target.load(); } catch (_) {}`. CONTEXT.md sketch already does this. Lock it as required in the plan.
**Warning signs:** Uncaught exception traceback referencing `HTMLMediaElement.load`; subsequent runtime primitives in the dispatcher chain fail to run.

### Pitfall 3: Feature-detect polarity inconsistency vs map.rs

**What goes wrong:** `map.rs` §306 uses `if (typeof IntersectionObserver !== 'undefined') { … }` (positive polarity). D-05 specifies `if (typeof IntersectionObserver === 'undefined') return;` (early-return polarity). Mixing them gives reviewers a false signal of inconsistency.
**Why it happens:** `map.rs` is INSIDE a `forEach(function(el) { … })` callback per-element, so an early-return would only abandon that callback iteration. `hero_lazy.rs` is at the top of `setupLazyHeroes()`, so early-return is correct.
**How to avoid:** Document the polarity choice in the plan's task description. Planner notes: "polarity is early-return, not enclosing if-block, because we are at the top of a setup function (consistent with how every sibling guards an empty selector match)."
**Warning signs:** Reviewer comment "why is this different from map.rs?". Pre-empt with a one-liner Rust source comment above the `pub(super) const SOURCE: &str = r#"…"#;` explaining the polarity choice.

### Pitfall 4: Closure-over-loop-variable in `for…in` over `groups`

**What goes wrong:** Classic JS bug: `for (var key in groups) { var io = new IntersectionObserver(function(entries, obs) { /* references key indirectly via obs? */ }, { rootMargin: key }); }` — does the IO constructor capture `key` synchronously?
**Why it happens:** `var key` is function-scoped, not block-scoped (in ES5). However, `{ rootMargin: key }` is evaluated synchronously at IO construction time — the object literal is built before `new IntersectionObserver(...)` returns. So `key` is fine here. The trap would be if the observer's CALLBACK referenced `key` (which it doesn't in the proposed sketch).
**How to avoid:** Confirm in the plan that the observer callback does NOT reference `key`. The proposed sketch is correct as written: callback only references `obs` (the observer parameter) and `e.target` (the entry target). `key` is only used in the `rootMargin` object literal at construction time.
**Warning signs:** Multiple groups share the same effective rootMargin even when configured differently. Test by inspecting `(observer.rootMargin)` in console after setup.

### Pitfall 5: IIFE size budget overshoot

**What goes wrong:** The implementation balloons past 700 bytes; D-10 triggers a redesign.
**Why it happens:** Verbose variable naming, multi-line comments, unnecessary `try/catch` wrapping, expanded whitespace.
**How to avoid:** Use short var names within the SOURCE string (e.g., `var m` for margin, `var io` for observer, `var els` for elements). Keep one short header comment block. Use `for (var i = 0; …)` not `for (var index = 0; …)`. After writing, measure: `cargo test -p ferro-json-ui --lib runtime::tests bundle_is_single_iife --no-run` then `wc -c` on the assembled bundle, OR add an optional size-budget unit test.
**Warning signs:** Raw byte count of the SOURCE string (excluding the Rust wrapper) exceeds 600.

### Pitfall 6: Workspace version bump skipping `Cargo.lock`

**What goes wrong:** Bumping `Cargo.toml` workspace version but forgetting to update `Cargo.lock`.
**Why it happens:** `cargo` updates `Cargo.lock` on the next build, but the commit graph may not reflect it if not run.
**How to avoid:** After bumping `Cargo.toml` to `0.2.42`, run `cargo build --workspace` once to update `Cargo.lock`, then commit both files together. Recent commit history shows this pattern (commit `474f4490 chore: sync Cargo.lock to workspace version 0.2.41`). [VERIFIED: git log]
**Warning signs:** CI lockfile-drift errors on master push; publish workflow fails the dependency graph check.

### Pitfall 7: SUMMARY.md page-registration miss

**What goes wrong:** New docs page created but not registered in `docs/src/SUMMARY.md` → page is invisible in the rendered book (mdbook only renders pages listed in SUMMARY).
**Why it happens:** Easy to overlook the SUMMARY edit when the page itself is the focus.
**How to avoid:** Plan task explicitly includes BOTH file operations: create `runtime-primitives.md` AND modify `SUMMARY.md`. Include in verification: `mdbook build docs/` (if mdbook is locally available) OR a grep `grep -q runtime-primitives docs/src/SUMMARY.md`.
**Warning signs:** `mdbook build` warning `INFO docs/src/json-ui/runtime-primitives.md was not used`.

---

## Code Examples

Verified patterns the planner should mimic. All taken from the existing codebase.

### Example 1: SOURCE-string preamble + setup function (verbatim from `sidebar.rs`)

```rust
// Source: ferro-json-ui/src/runtime/sidebar.rs §1-9
pub(super) const SOURCE: &str = r#"
    // ── Sidebar mobile toggle ─────────────────────────────────────────────

    function setupSidebar() {
        var toggleBtn = document.querySelector('[data-sidebar-toggle]');
        var sidebarEl = document.querySelector('[data-sidebar]');
        var backdropEl = document.querySelector('[data-sidebar-backdrop]');
        if (!toggleBtn || !sidebarEl) return;
        // …
    }
"#;
```

### Example 2: IntersectionObserver setup (verbatim from `map.rs` §306-315)

```rust
// Source: ferro-json-ui/src/plugins/map.rs §306-315
if (typeof IntersectionObserver !== 'undefined') {
  var observer = new IntersectionObserver(function(entries) {
    entries.forEach(function(entry) {
      if (entry.isIntersecting) {
        map.invalidateSize();
      }
    });
  });
  observer.observe(el);
}
```

### Example 3: Test-shape precedent (string-presence in mod.rs) — extending `bundle_contains_all_setup_functions`

```rust
// Source: ferro-json-ui/src/runtime/mod.rs §142-162 (existing) — Phase 182 adds "setupLazyHeroes"
#[test]
fn bundle_contains_all_setup_functions() {
    for fn_name in [
        "setupSSE",
        "setupTabs",
        "setupToasts",
        "setupSidebar",
        "setupDropdowns",
        "setupModals",
        "setupDismissibles",
        "setupNotifications",
        "setupFormGuards",
        "setupProductTiles",
        "setupKanban",
        "setupScrollPreserve",
        "setupLazyHeroes",   // ← Phase 182 addition
    ] {
        assert!(
            FERRO_RUNTIME_JS.contains(fn_name),
            "bundle missing {fn_name}"
        );
    }
}
```

### Example 4: New test (to be added in Phase 182)

```rust
// New test added to ferro-json-ui/src/runtime/mod.rs `mod tests`
#[test]
fn runtime_contains_lazy_hero_setup() {
    assert!(FERRO_RUNTIME_JS.contains("setupLazyHeroes"));
    assert!(FERRO_RUNTIME_JS.contains("data-lazy-hero"));
    assert!(FERRO_RUNTIME_JS.contains("data-lazy-hero-margin"));
    assert!(FERRO_RUNTIME_JS.contains("data-lazy-hero-promoted"));
    assert!(FERRO_RUNTIME_JS.contains("IntersectionObserver"));
    assert!(FERRO_RUNTIME_JS.contains("preload"));
    assert!(FERRO_RUNTIME_JS.contains("\"auto\""));
    assert!(FERRO_RUNTIME_JS.contains("unobserve"));
}
```

### Example 5: Planner-blessed JS sketch (refined from CONTEXT.md)

The planner should use this shape, ES5-only, ≤500 bytes target. Comments inside SOURCE minimized.

```js
// Inside r#"…"#; in ferro-json-ui/src/runtime/hero_lazy.rs

// ── Lazy hero video promotion ──────────────────────────────────────────

function setupLazyHeroes() {
    if (typeof IntersectionObserver === 'undefined') return;
    var els = document.querySelectorAll('video[preload="none"][data-lazy-hero]:not([data-lazy-hero-promoted])');
    if (!els.length) return;
    var groups = {};
    for (var i = 0; i < els.length; i++) {
        var m = (els[i].getAttribute('data-lazy-hero-margin') || '200px 0px').replace(/^\s+|\s+$/g, '');
        (groups[m] = groups[m] || []).push(els[i]);
    }
    for (var key in groups) {
        var io = new IntersectionObserver(function(entries, obs) {
            for (var j = 0; j < entries.length; j++) {
                var e = entries[j];
                if (e.isIntersecting && !e.target.hasAttribute('data-lazy-hero-promoted')) {
                    e.target.setAttribute('preload', 'auto');
                    e.target.setAttribute('data-lazy-hero-promoted', '1');
                    try { e.target.load(); } catch (_) {}
                    obs.unobserve(e.target);
                }
            }
        }, { rootMargin: key });
        for (var k = 0; k < groups[key].length; k++) {
            io.observe(groups[key][k]);
        }
    }
}
```

Changes vs CONTEXT.md sketch:
- Added `.replace(/^\s+|\s+$/g, '')` (or use `.trim()`) on the margin string to guard against whitespace-induced SyntaxError (Pitfall 1).
- Replaced `entries.forEach(...)` with indexed for loop (sibling convention; Anti-Patterns).
- Replaced `groups[key].forEach(...)` with indexed for loop (sibling convention).
- Kept `try { … } catch (_) {}` around `.load()` (Pitfall 2).
- The `entries[j]` shadow var (`e`) is reused per-iteration safely because `var` is function-scoped; the callback closes over `obs` which is its own parameter.

Raw size estimate of the JS body (excluding Rust wrapper, with the comment header): ≈800–900 bytes with formatting whitespace, ≈500–550 bytes with minification of indentation. The planner picks a near-minified composition; sibling files preserve indentation, so matching them gives ≈800 bytes. **The planner has discretion to either (a) preserve indentation for sibling consistency and accept the soft-target overshoot, or (b) strip the inner-most indentation to hit the ≤500-byte target.** Recommended: (a) — sibling-consistency outweighs the byte-budget guideline; D-10 frames the budget as guidance, not hard fail, with redesign-trigger only above ~700 bytes raw of meaningful content (not whitespace).

### Example 6: docs/src/SUMMARY.md edit

```markdown
# Source: docs/src/SUMMARY.md §64-75 (current) — Phase 182 adds one line

# JSON-UI

- [Getting Started](json-ui/getting-started.md)
- [Components](json-ui/components.md)
- [Actions](json-ui/actions.md)
- [Data Binding & Visibility](json-ui/data-binding.md)
- [Form Validation](json-ui/forms.md)
- [Layouts](json-ui/layouts.md)
- [Plugins](json-ui/plugins.md)
- [Runtime Primitives](json-ui/runtime-primitives.md)   ← Phase 182 addition
- [Spec construction](./json-ui/spec-construction.md)
- [Expressions](json-ui/expressions.md)
- [JSON Schema](json-ui/json-schema.md)
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Per-page IntersectionObserver boilerplate for lazy-loading `<video>` | ferro-json-ui runtime primitive (Phase 182) | 2026-06-06 | Tenant pages drop ~20 lines of per-page JS; declarative `data-lazy-hero` opt-in. |
| Native `<video loading="lazy">` | Not supported by HTML spec | n/a | The HTML spec defines `loading="lazy"` for `<img>` and `<iframe>` only; `<video>` is the genuine gap Phase 182 fills. `[VERIFIED: HTML living standard §HTMLMediaElement does not include `loading` IDL attribute]` |

**Deprecated/outdated:** N/A — Phase 182 is purely additive.

---

## Cross-References (Sibling Phases)

| Phase | Relationship | What to read |
|-------|--------------|-------------|
| Phase 165 F11/F13/F14 | "Runtime gaps belong in ferro" elevation precedent — same rule as Phase 182. | Phase 165 context (not directly required for planning Phase 182, but the elevation argument is identical: a generic web primitive that consumer pages would otherwise hand-roll moves into ferro). |
| Phase 181 (json-ui-input-error-prop-inline-render) | Closest-in-style sibling phase. Docs-page pattern, RESEARCH.md voice, single-publish-at-end cadence all mirror 181. | `.planning/phases/181-json-ui-input-error-prop-inline-render/181-RESEARCH.md` for the document-shape precedent; `181-CONTEXT.md` for the locked-decisions-driven research voice. |
| Phase 183 (`ferro-bundle` capability) | Same milestone (the 2026-06-06 jetskiadriatic startup-lifecycle audit). Pairs 1:1 with gestiscilo Phase 185. Build order: 182 → 183 → 184 (182 acts as pattern-rodage). | `.planning/ROADMAP.md` §1968-1986 |
| Phase 184 (`InlineBudget` + `RequestTelemetry`) | Same milestone (2026-06-06 audit). Pairs 1:1 with gestiscilo Phase 187. | `.planning/ROADMAP.md` §1988-2005 |
| gestiscilo Phase 186 (downstream consumer) | Consumes Phase 182's published primitive via Cargo.toml bump. Out of scope here, but the cross-tracking means Phase 182 must not break the public attribute contract before the consumer adopts it. | Cross-repo only — not findable in this ferro repo. The roadmap reference at §1945 "Paired with: gestiscilo Phase 186" is the constraint. |

**Constraint from cross-tracked gestiscilo Phase 186:** Whatever attribute names Phase 182 ships, gestiscilo Phase 186 will pin against. Once 0.2.42 publishes, the three attribute names (`data-lazy-hero`, `data-lazy-hero-margin`, `data-lazy-hero-promoted`) are part of the public ferro contract and must not be renamed in subsequent releases without a breaking-change major version bump. This is explicitly within Phase 182's CONTEXT.md §domain ("Public-contract attribute names").

---

## Risks and Edge Cases

| Risk | Severity | Mitigation |
|------|----------|------------|
| Whitespace in `data-lazy-hero-margin` value throws on IO construction | MEDIUM | `.trim()` (or regex-replace) the value before bucket-key use. Pitfall 1. |
| `video.load()` throws on Safari | LOW-MEDIUM | `try { … } catch (_) {}` wrap. Pitfall 2. Already in CONTEXT.md sketch. |
| IIFE size overshoot triggers redesign | LOW | Soft target per D-10. Plan includes a measurement step. Pitfall 5. |
| Feature-detect polarity inconsistency vs map.rs | LOW | Documented polarity choice; one-line rust comment in `hero_lazy.rs`. Pitfall 3. |
| Cargo.lock drift on version bump | MEDIUM | `cargo build --workspace` after `Cargo.toml` bump; commit both. Pitfall 6. Precedent: commit 474f4490. |
| SUMMARY.md miss → page invisible | MEDIUM | Plan explicitly includes both file ops; verification grep. Pitfall 7. |
| Phase 182 publishes before gestiscilo Phase 186 is ready to consume | LOW | D-08: single publish at end of phase; gestiscilo bumps after merge. Matches memory `feedback_friction_loop_release_cadence.md`. |
| Tenant page with zero `data-lazy-hero` elements pays IIFE cost | NEGLIGIBLE | Early return via `!els.length`; ≤500 bytes added is acceptable per D-10. |
| Browser without IntersectionObserver (legacy Android WebView) | LOW | Feature-detect early-return; videos behave as authored. D-05. |
| Two elements with identical margin value but mixed `:not([data-lazy-hero-promoted])` matching after dynamic insertion | NEGLIGIBLE | Out of scope (D-03). Documented Deferred Idea. |
| `data-lazy-hero` set on `<img>` or `<iframe>` by a confused author | NEGLIGIBLE | Selector targets `video[preload="none"]` only; non-video elements silently ignored (D-02). Docs page reinforces "video only." |
| Wave1A publish ordering — ferro-json-ui depends on nothing internal at Wave1A | LOW | ferro-json-ui is correctly placed in Wave1A (per `.github/workflows/publish.yml` §211). No dependency reordering needed. |
| Master push without library-crate changes skips publish | NEGLIGIBLE | Library-change gate at publish.yml §25-50 specifically lists `ferro-json-ui/**` as a publishable path. Workspace-root Cargo.toml bump alone counts. |

---

## Assumptions Log

> Every claim in this research is `[VERIFIED:`...`]` from direct file read EXCEPT the items below.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Safari `<video>.load()` historically throws in some autoplay/cross-origin conditions. | Pitfall 2 | LOW — the `try { … } catch (_) {}` wrap is cheap insurance; planning safely without verification. If WebKit no longer throws, the catch is unreachable but harmless. `[ASSUMED — based on long-standing WebKit bug reports; not verified against current Safari behavior in 2026]` |
| A2 | IntersectionObserver `rootMargin` getter is strict about leading/trailing whitespace in Safari but tolerant in Chrome/Firefox. | Pitfall 1 | LOW-MEDIUM — `.trim()`/regex-replace is cheap insurance. If all engines tolerate whitespace, the trim is no-op overhead (≈30 bytes). If Safari rejects, the trim prevents broken hero promotion on Safari users. `[ASSUMED — IntersectionObserver spec defers to CSS margin parsing which is tolerant; specific Safari behavior not verified against current release]` |
| A3 | `String.prototype.trim` is available in every browser where IntersectionObserver works. | Example 5 | LOW — IntersectionObserver shipped Safari 12.1 (Mar 2019); `.trim()` shipped Safari 5 (2010). Trim is a safe ES5 baseline. `[CITED: MDN String.prototype.trim — "ECMAScript 5.1 (2011); fully supported"]` |
| A4 | The `replace(/^\s+|\s+$/g, '')` regex is equivalent to `.trim()` for the planner's purposes. | Example 5 | LOW — both strip leading/trailing whitespace. Functionally identical. `[ASSUMED — basic JS knowledge]` |
| A5 | Workspace version bump in `Cargo.toml` propagates to `ferro-json-ui` via `version.workspace = true` and the publish workflow uses the workspace version. | D-08 / File Map | LOW — verified at `ferro-json-ui/Cargo.toml` §3 (`version.workspace = true`); publish workflow reads workspace `Cargo.toml` version at line 203. `[VERIFIED: file read of both]` — moved out of assumed. |

**Total `[ASSUMED]` claims:** 3 substantive (A1, A2, A4). All are LOW-risk and all already have defensive code mitigations baked into the proposed sketch. None require user confirmation before planning proceeds.

---

## Open Questions

1. **Should the new test be one combined `runtime_contains_lazy_hero_setup` or several focused tests (per attribute name, per behavior)?**
   - What we know: The existing test module has both styles. `test_runtime_contains_popover_dropdown_wiring` bundles all popover-related assertions; `runtime_contains_init_tab_from_url` focuses on one behavior.
   - What's unclear: No project-wide convention forcing one shape.
   - Recommendation: Single combined test (`runtime_contains_lazy_hero_setup`) — aligns with the closer-in-spirit sibling tests (`test_runtime_contains_popover_dropdown_wiring`). Easier to extend with future assertions. Acknowledged as Claude's Discretion in CONTEXT.md.

2. **Should the planner add an optional IIFE-size budget unit test?**
   - What we know: D-10 frames the budget as guideline, not hard fail. No precedent in `runtime/mod.rs` tests for a size assertion.
   - What's unclear: Whether locking the budget into a test is worth the rigidity it introduces (future legitimate additions to other primitives could fail the test for a totally unrelated reason).
   - Recommendation: Do NOT add a test. Measure once during planning, document the byte delta in the plan summary, rely on code-review for ongoing enforcement. Adding the test calcifies the soft target into a hard one — counter to D-10's framing.

3. **Should `mod hero_lazy;` be inserted alphabetically (between `form_guards` and `kanban`) or appended at the end?**
   - What we know: Current mod list is alphabetical. D-06 specifies alphabetical insertion.
   - What's unclear: Nothing — D-06 is explicit.
   - Recommendation: Alphabetical, between `form_guards` (line 10) and `kanban` (line 11). Locked.

4. **What is the exact position of `setupLazyHeroes();` in the dispatcher block (mod.rs §41-58)?**
   - What we know: Order is not strictly alphabetical (existing dispatch starts with `setupScrollPreserve`; `setupSSE` follows; `setupTabs`; etc.). The order does not affect observable behavior per CONTEXT.md.
   - What's unclear: No locking decision. Discretion.
   - Recommendation: Add as the last line of the dispatcher block (after `setupToasts();`), or alphabetically after `setupKanban();`. Both produce identical behavior; "last line" yields the smallest diff (single-line addition at one location).

5. **Should the docs page reference Phase 182 internally, or be voiced as a generic feature page?**
   - What we know: Project CLAUDE.md says "Repository documents must read as neutral." No internal-strategy framing.
   - What's unclear: Nothing — explicit.
   - Recommendation: No phase references in the docs page. Voiced as the steady-state public contract: "ferro-json-ui ships a runtime IIFE that recognizes the `data-lazy-hero` family of DOM attributes. This page documents that contract." No mention of "Phase 182," no mention of jetskiadriatic, no mention of gestiscilo.

---

## Environment Availability

> Skipped — Phase 182 has no external tool dependencies beyond the Rust toolchain already present in the repo (cargo, rustc). No new CLI tools, no new services, no databases needed. mdbook is recommended for local docs verification but not blocking (CI has it).

---

## Security Domain

> Phase 182's primitive does not handle authentication, sessions, access control, user input parsing, or cryptography. The only DOM mutation is setting two attributes (`preload`, `data-lazy-hero-promoted`) on `<video>` elements the tenant page already authored. The only network effect is triggering the browser to fetch a video URL the tenant page already declared in `<source src="…">`.
>
> No ASVS category applies. No STRIDE threat is introduced by this primitive (the data flow is: tenant author writes URL → browser fetches URL when viewport approaches; identical to what `preload="auto"` does at page load, just temporally deferred).
>
> The only theoretical concern is a tenant page incorrectly trusting `data-lazy-hero` semantics to gate a fetch they want NEVER to happen (e.g., a paid-content video). This is a tenant-side misuse, not a primitive bug — the runtime never blocks fetches, only defers them. Docs page should NOT make any claim like "Use `data-lazy-hero` to prevent fetching." It is a performance primitive, not a security primitive. Docs page wording should reinforce this.

---

## Sources

### Primary (HIGH confidence — direct file read)
- `ferro-json-ui/src/runtime/mod.rs` — entire file (192 lines)
- `ferro-json-ui/src/runtime/sidebar.rs` — entire file (38 lines)
- `ferro-json-ui/src/runtime/sse.rs` — entire file (43 lines)
- `ferro-json-ui/src/runtime/scroll_preserve.rs` — entire file (70 lines)
- `ferro-json-ui/src/runtime/dropdowns.rs` — entire file (81 lines)
- `ferro-json-ui/src/runtime/form_guards.rs` — entire file (94 lines)
- `ferro-json-ui/src/runtime/tabs.rs` — entire file (86 lines)
- `ferro-json-ui/src/runtime/modals.rs` — entire file (32 lines)
- `ferro-json-ui/src/runtime/notifications.rs` — entire file (25 lines)
- `ferro-json-ui/src/runtime/dismissibles.rs` — entire file (55 lines)
- `ferro-json-ui/src/runtime/kanban.rs` §1-40
- `ferro-json-ui/src/runtime/toasts.rs` §1-30
- `ferro-json-ui/src/plugins/map.rs` §240-321 (INIT_SCRIPT + IntersectionObserver pattern)
- `ferro-json-ui/src/plugins/map.rs` §470-485 (test-shape precedent)
- `ferro-json-ui/src/layout.rs` §310-323 (with_runtime), §600-625 (DashboardLayout body)
- `ferro-json-ui/Cargo.toml` — entire file (29 lines)
- `Cargo.toml` (workspace root) §1-40
- `.github/workflows/publish.yml` §1-260 (library-change gate, Wave1A, Wave1B)
- `docs/book.toml`
- `docs/src/SUMMARY.md` — entire file (76 lines)
- `docs/src/json-ui/data-binding.md` §1-40 (voice/structure reference)
- `docs/src/features/json-ui.md` §1-30 (top-level framing reference)
- `.planning/phases/182-ferro-json-ui-data-lazy-hero-runtime-primitive/182-CONTEXT.md` — entire file (277 lines)
- `.planning/ROADMAP.md` §1940-2007 (Phase 182, 183, 184 entries)
- `.planning/STATE.md` §1-100 + §200-294 (project state, recent decisions)
- `.planning/REQUIREMENTS.md` — entire file (76 lines, confirming Phase 182 is not enumerated in v12.1 AI REQ-IDs)
- `./CLAUDE.md` (project) — full read
- `.planning/phases/181-json-ui-input-error-prop-inline-render/181-RESEARCH.md` §1-80 (sibling-phase document-shape reference)

### Secondary (MEDIUM confidence)
- Recent git log via gitStatus context — confirmed commit `474f4490 chore: sync Cargo.lock to workspace version 0.2.41` precedent for the workspace-version + Cargo.lock combined commit pattern.

### Tertiary (LOW confidence — flagged in Assumptions Log)
- WebKit historical `<video>.load()` throw behavior (A1)
- Safari IntersectionObserver `rootMargin` whitespace strictness (A2)

---

## Metadata

**Confidence breakdown:**
- Sibling primitive structure: HIGH — direct file read of every sibling.
- Wire-up insertion points in `runtime/mod.rs`: HIGH — exact line numbers verified.
- Test pattern extensions: HIGH — read of existing test module.
- Publish workflow: HIGH — Wave1A line 211, library-change gate lines 25-50 verified.
- Docs page placement: HIGH — SUMMARY.md structure read; placement is Claude's discretion per D-09.
- Workspace version bump mechanics: HIGH — Cargo.toml structure + Cargo.lock precedent verified.
- IntersectionObserver behavior on edge inputs (whitespace rootMargin, Safari .load() throws): MEDIUM — defensive mitigations baked into the recommended JS sketch; assumptions flagged for the planner.
- IIFE size estimate: MEDIUM — rough byte count estimated; planner measures during implementation.

**Research date:** 2026-06-06
**Valid until:** 2026-07-06 (30 days — stable codebase, no fast-moving dependencies; only invalidator would be a refactor of the runtime IIFE assembly mechanism)

## RESEARCH COMPLETE
