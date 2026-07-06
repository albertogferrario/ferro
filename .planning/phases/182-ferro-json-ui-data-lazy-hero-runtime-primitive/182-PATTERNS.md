# Phase 182: ferro-json-ui `data-lazy-hero` runtime primitive — Pattern Map

**Mapped:** 2026-06-06
**Files analyzed:** 6 (2 created, 4 modified)
**Analogs found:** 6 / 6

## File Classification

| New/Modified File | Status | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|--------|------|-----------|----------------|---------------|
| `ferro-json-ui/src/runtime/hero_lazy.rs` | CREATE | runtime-primitive (SOURCE const) | event-driven (DOMContentLoaded → IntersectionObserver) | `ferro-json-ui/src/runtime/sidebar.rs` (shape) + `ferro-json-ui/src/runtime/dropdowns.rs` (querySelectorAll fan-out) + `ferro-json-ui/src/plugins/map.rs` §306-315 (IntersectionObserver API) | exact (shape) + exact (browser API) |
| `ferro-json-ui/src/runtime/mod.rs` | MODIFY | bundle-assembler + test module | transform (LazyLock string concat) + unit-test | self (the file is its own analog — sibling primitives already wired) | exact |
| `docs/src/json-ui/runtime-primitives.md` | CREATE | docs page (public DOM contract) | static markdown | `docs/src/json-ui/forms.md` (closest in topic: public attribute contract on form-control components) + `docs/src/json-ui/plugins.md` (voice/structure) | role-match |
| `docs/src/SUMMARY.md` | MODIFY | mdbook TOC | static markdown | self (single-line addition to existing JSON-UI section) | exact |
| `Cargo.toml` (workspace root) | MODIFY | workspace metadata | config | self (same `workspace.package.version` field bumped each release cycle) | exact |
| `Cargo.lock` | MODIFY | dependency lockfile | config (auto-synced via `cargo build`) | commit `474f4490` (precedent for the 0.2.40→0.2.41 sync) | exact |

---

## Pattern Assignments

### `ferro-json-ui/src/runtime/hero_lazy.rs` (CREATE — runtime-primitive)

**Primary analog:** `ferro-json-ui/src/runtime/sidebar.rs` (file shape: single `pub(super) const SOURCE: &str = r#"…"#;`, ASCII-box header comment, single `function setupXxx()` body).

**Secondary analog:** `ferro-json-ui/src/runtime/dropdowns.rs` (querySelectorAll + indexed `for` fan-out to per-element init).

**Browser-API analog:** `ferro-json-ui/src/plugins/map.rs` §306-315 (IntersectionObserver feature-detect, construction, observe).

#### File-shape pattern (verbatim from `sidebar.rs` lines 1-9, 36-37)

```rust
pub(super) const SOURCE: &str = r#"
    // ── Sidebar mobile toggle ─────────────────────────────────────────────

    function setupSidebar() {
        var toggleBtn = document.querySelector('[data-sidebar-toggle]');
        var sidebarEl = document.querySelector('[data-sidebar]');
        var backdropEl = document.querySelector('[data-sidebar-backdrop]');
        if (!toggleBtn || !sidebarEl) return;
        // … body …
    }
"#;
```

**What to copy literally:**
- `pub(super) const SOURCE: &str = r#"…"#;` declaration (visibility, raw-string delimiter).
- 4-space inner indentation for JS body (matches every sibling).
- ASCII-box header comment `// ── Lazy hero video promotion ──────────────────────────────────────────` (matches `sidebar.rs:2`, `scroll_preserve.rs:2`, `dropdowns.rs:2`).
- `function setupXxx() { … }` declaration form (no IIFE per primitive — the outer dispatcher wraps everything).
- Early-return guard immediately after `var` declarations.
- No trailing newline inside the raw string before `"#;` (matches all siblings).

#### querySelectorAll + indexed-for fan-out (verbatim from `dropdowns.rs` lines 20-25)

```rust
function setupDropdowns() {
    var menus = document.querySelectorAll('[data-popover-menu]');
    for (var i = 0; i < menus.length; i++) {
        initPopoverMenu(menus[i]);
    }
}
```

**What to copy:** the `var els = document.querySelectorAll('…'); for (var i = 0; i < els.length; i++) { … }` shape. Never `.forEach()` on a NodeList (no sibling uses it).

**Adapt for hero_lazy:** after querySelectorAll, add an early-return on `!els.length` (sibling pattern: every primitive returns early when its selector matches zero elements).

#### IntersectionObserver setup (verbatim from `plugins/map.rs` lines 306-315)

```rust
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

**Polarity adjustment (D-05, RESEARCH Pitfall 3):** invert the guard to early-return at the top of `setupLazyHeroes()`:
```js
if (typeof IntersectionObserver === 'undefined') return;
```
The polarity differs from `map.rs` because `map.rs` is inside a per-element callback (positive polarity skips that element); `setupLazyHeroes` is at the top of a setup function (early-return abandons the whole primitive). Document this with a one-line Rust source comment above the `pub(super) const SOURCE: …` declaration so reviewers do not flag the inconsistency.

**Adapt for hero_lazy:**
- Substitute `map.invalidateSize()` with the promote action: `setAttribute('preload', 'auto')`, `setAttribute('data-lazy-hero-promoted', '1')`, `try { e.target.load(); } catch (_) {}`, `obs.unobserve(e.target)`.
- Replace `entries.forEach(...)` with `for (var j = 0; j < entries.length; j++)` per sibling convention (no `.forEach` on NodeLists; sibling stylistic consistency outweighs the fact that `entries` is technically a true Array).
- Construct one observer per distinct `rootMargin` bucket key (D-01 grouping layer — no precedent in any sibling, planner-original logic).

#### Defensive try/catch around browser-throw-prone calls (verbatim from `scroll_preserve.rs` line 52)

```rust
} catch (e) { /* sessionStorage may be unavailable */ }
```

**What to copy:** the `try { unsafe_op(); } catch (_) {}` shape with a short inline comment naming the failure mode. Phase 182 applies this to `e.target.load()` (Safari can throw `InvalidStateError` mid-resource-selection — RESEARCH Pitfall 2):

```js
try { e.target.load(); } catch (_) {}
```

Underscore parameter name (`_`) is shorter than `e` and signals "ignored"; siblings use both styles (`scroll_preserve.rs` uses `e`, planner picks `_` here for byte budget).

#### Whitespace-trim guard on the rootMargin string (RESEARCH Pitfall 1)

No direct sibling precedent (none of the siblings parse user-provided CSS strings). Planner adds:

```js
var m = (els[i].getAttribute('data-lazy-hero-margin') || '200px 0px').replace(/^\s+|\s+$/g, '');
```

`.trim()` is equivalent and shorter; either is acceptable per RESEARCH Example 5 / Assumption A3 (`String.prototype.trim` is ES5.1, available everywhere IntersectionObserver works).

#### Canonical composed sketch (from RESEARCH §Example 5, lines 545-575)

This is the planner-blessed body the executor should land:

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

**ES5 style locks (siblings never use; planner must not use):**
- Arrow functions
- `let` / `const`
- Template literals
- Destructuring
- `.forEach()` on NodeLists
- `console.log` for happy-path logging (only `console.error` for caught errors, per `map.rs:317`)
- A second `addEventListener('DOMContentLoaded', …)` — the outer dispatcher already registers it

---

### `ferro-json-ui/src/runtime/mod.rs` (MODIFY — bundle-assembler + test module)

**Analog:** self. The file is its own analog because every existing sibling primitive is already wired through the same four touchpoints. Phase 182 adds one entry to each.

#### Touchpoint 1 — `mod` declaration (existing lines 8-19)

Current state:
```rust
mod dismissibles;
mod dropdowns;
mod form_guards;
mod kanban;
mod modals;
mod notifications;
mod product_tiles;
mod scroll_preserve;
mod sidebar;
mod sse;
mod tabs;
mod toasts;
```

**Change:** Insert `mod hero_lazy;` alphabetically — between `form_guards` (line 10) and `kanban` (line 11). D-06-locked.

#### Touchpoint 2 — `push_str` chain (existing lines 28-40)

Current state:
```rust
s.push_str("(function() {\n    'use strict';\n");
s.push_str(sse::SOURCE);
s.push_str(tabs::SOURCE);
s.push_str(toasts::SOURCE);
s.push_str(dismissibles::SOURCE);
s.push_str(notifications::SOURCE);
s.push_str(dropdowns::SOURCE);
s.push_str(modals::SOURCE);
s.push_str(sidebar::SOURCE);
s.push_str(form_guards::SOURCE);
s.push_str(product_tiles::SOURCE);
s.push_str(kanban::SOURCE);
s.push_str(scroll_preserve::SOURCE);
```

**Change:** Append `s.push_str(hero_lazy::SOURCE);` as the LAST line of the chain (after `scroll_preserve::SOURCE` on line 40). RESEARCH §File Map and CONTEXT.md §Claude's Discretion both recommend this position — minimizes the diff (single-line addition at one location; no reordering of unrelated siblings). The push order does not affect observable behavior because every primitive's setup function is invoked from the dispatcher, not from declaration order.

#### Touchpoint 3 — dispatcher comment block (existing lines 41-58)

Current state (the dispatcher is written as a single multi-line `push_str` of a raw-byte-escaped string literal):
```rust
s.push_str(
    "\n    function ferroRuntime() {\n\
     \x20       setupScrollPreserve();\n\
     \x20       setupSSE();\n\
     \x20       setupTabs();\n\
     \x20       setupDismissibles();\n\
     \x20       setupNotifications();\n\
     \x20       setupDropdowns();\n\
     \x20       setupKanban();\n\
     \x20       setupSidebar();\n\
     \x20       setupFormGuards();\n\
     \x20       setupProductTiles();\n\
     \x20       setupModals();\n\
     \x20       setupToasts();\n\
     \x20   }\n\
     \x20   document.addEventListener('DOMContentLoaded', ferroRuntime);\n\
     })();\n",
);
```

**Change:** Insert `\x20       setupLazyHeroes();\n\` as the LAST line before `\x20   }\n\` (after `setupToasts();`). RESEARCH §Open Question 4 recommendation: "last line of the dispatcher block" — smallest diff. The relative dispatch order does not change observable behavior; the primitives are independent (lazy heroes do not depend on or block any sibling).

**Critical formatting detail:** the line-continuation backslash at the end of each line is what makes this a single string literal across lines. Match the exact `\x20       setupLazyHeroes();\n\` shape (3 chars `\x20` literal for a leading space, 7 spaces of indentation, the call, `\n`, then a trailing `\`).

#### Touchpoint 4 — Test array additions (existing lines 142-162, 170-191)

**Sub-change 4a:** Extend `bundle_contains_all_setup_functions` (existing lines 141-162). Pattern excerpt (verbatim from existing lines 142-162):

```rust
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
    ] {
        assert!(
            FERRO_RUNTIME_JS.contains(fn_name),
            "bundle missing {fn_name}"
        );
    }
}
```

Add `"setupLazyHeroes",` to the array (position discretion: at the bottom of the existing list mirrors the `push_str` chain order).

**Sub-change 4b:** Extend `dispatcher_invokes_every_setup` (existing lines 170-191). Pattern excerpt (verbatim from existing lines 171-191):

```rust
#[test]
fn dispatcher_invokes_every_setup() {
    let js: &str = FERRO_RUNTIME_JS.as_str();
    let dispatcher_start = js.find("function ferroRuntime()").unwrap();
    let dispatcher = &js[dispatcher_start..];
    for call in [
        "setupSSE();",
        "setupTabs();",
        "setupToasts();",
        "setupSidebar();",
        "setupDropdowns();",
        "setupModals();",
        "setupDismissibles();",
        "setupNotifications();",
        "setupFormGuards();",
        "setupProductTiles();",
        "setupKanban();",
        "setupScrollPreserve();",
    ] {
        assert!(dispatcher.contains(call), "dispatcher missing {call}");
    }
}
```

Add `"setupLazyHeroes();",` to the array.

**Sub-change 4c:** Add NEW test `runtime_contains_lazy_hero_setup`. Closest-in-shape analog is the existing `test_runtime_contains_popover_dropdown_wiring` (lines 98-105) — a single test bundling all primitive-related string-presence assertions. Pattern excerpt:

```rust
#[test]
fn test_runtime_contains_popover_dropdown_wiring() {
    assert!(FERRO_RUNTIME_JS.contains("data-popover-menu"));
    assert!(FERRO_RUNTIME_JS.contains(":popover-open"));
    assert!(FERRO_RUNTIME_JS.contains("hidePopover"));
    assert!(FERRO_RUNTIME_JS.contains("positionUnderTrigger"));
    assert!(FERRO_RUNTIME_JS.contains("getBoundingClientRect"));
}
```

New test (RESEARCH Example 4, verbatim):

```rust
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

Position discretion: append after `runtime_contains_init_tab_from_url` (line 132) and before `bundle_contains_dispatcher` (line 135), or at the end of the test module. Either is acceptable; the existing module has no strict ordering.

**Test-naming convention note:** the existing module mixes `test_runtime_contains_*` and `runtime_contains_*` prefixes. The newer style is the bare `runtime_contains_*` form (`runtime_contains_init_tab_from_url:123`). Phase 182 follows the newer style per RESEARCH Open Question 1 recommendation.

---

### `docs/src/json-ui/runtime-primitives.md` (CREATE — docs page)

**Primary analog:** `docs/src/json-ui/forms.md` lines 1-11 (closest in topic — both pages document a public DOM-attribute contract that tenant HTML or component output sets, with the framework's runtime consuming it).

**Voice analog:** `docs/src/json-ui/data-binding.md` lines 1-6 (intro paragraph that names the primitive, its purpose, and the contract surfaces).

**Structure analog:** `docs/src/json-ui/plugins.md` lines 1-19 (one-sentence opening, "when to use" framing, scope statement).

#### Voice and structure pattern (verbatim from `forms.md` lines 1-11)

```markdown
# Form Validation

Server-rendered form validation in ferro pairs three primitives: a `ValidationError` value built during request handling, the `_flash.old._validation_errors` session key, and the JSON-UI form-control prop `error`. This page covers the four authoring patterns: the blessed `JsonUi::render_validation_error` path, the manual `$data` binding escape hatch, the flash round-trip on POST→GET, and the cross-field validation summary.

All form-control components (`Input`, `Select`, `Input { input_type: "textarea" }`, `Checkbox`, `CheckboxList`, `Switch`) accept an `error` prop of type `Option<String>`. When set, the renderer emits a destructive-tone class chain on the control and an inline error paragraph below the field:

```html
<p id="err-{field}" class="text-sm text-destructive">{error}</p>
```

The paragraph carries `id="err-{field}"` so the control's `aria-describedby="err-{field}"` pairing announces the error to assistive technology.
```

**What to copy:**
- `# Page Title` H1 only (no frontmatter — mdbook does not require it; no existing `docs/src/json-ui/*.md` page uses frontmatter; verified by `head` of `forms.md`, `data-binding.md`, `plugins.md`).
- Single intro paragraph naming the primitive and the page's scope.
- H2 section per public attribute / contract surface.
- Fenced code blocks for HTML examples (` ```html `).
- Neutral, third-person voice. No "we", no project-internal references.

#### Required content (from CONTEXT.md D-09 and RESEARCH §User Constraints)

H2 sections (planner discretion on exact titles per CONTEXT.md §Claude's Discretion):

1. **`data-lazy-hero`** — opt-in marker on `<video preload="none">`. Default `rootMargin: 200px 0px`. Selector and behavior.
2. **`data-lazy-hero-margin`** — per-element rootMargin override (string passed verbatim to the IntersectionObserver constructor after `.trim()`).
3. **`data-lazy-hero-promoted`** — idempotency sentinel set to `"1"` after promotion. Re-running the runtime is a no-op for already-promoted elements.
4. **Browser support** — IntersectionObserver feature-detection note; no-op on browsers without IO.
5. **Forward-compat note** — the runtime is one-shot at `DOMContentLoaded`; dynamically inserted elements are not observed.

**Voice constraints (RESEARCH §Project Constraints + Open Question 5):**
- No "Phase 182" references.
- No "jetskiadriatic" / "gestiscilo" tenant names.
- No "killer feature" framing, no internal-strategy voice.
- Scientific, minimalistic — no marketing.
- Generic web-primitive framing only.
- Page is forward-looking: framing accommodates future runtime-attribute additions (e.g., a hypothetical `data-defer-load`) without naming them.

**Out-of-scope content (RESEARCH §User Constraints / Deferred):** sibling internal attributes (`data-sse-url`, `data-sidebar-toggle`, `data-popover-menu`) are NOT enumerated on this page — those remain implementation details of the components that emit them.

---

### `docs/src/SUMMARY.md` (MODIFY — mdbook TOC)

**Analog:** self. The existing JSON-UI section (lines 52-63) is the analog; Phase 182 adds one bulleted entry.

#### Current state (verbatim, lines 52-63)

```markdown
# JSON-UI

- [Getting Started](json-ui/getting-started.md)
- [Components](json-ui/components.md)
- [Actions](json-ui/actions.md)
- [Data Binding & Visibility](json-ui/data-binding.md)
- [Form Validation](json-ui/forms.md)
- [Layouts](json-ui/layouts.md)
- [Plugins](json-ui/plugins.md)
- [Spec construction](./json-ui/spec-construction.md)
- [Expressions](json-ui/expressions.md)
- [JSON Schema](json-ui/json-schema.md)
```

#### Change

Insert one line between `Plugins` (line 60) and `Spec construction` (line 61):

```markdown
- [Runtime Primitives](json-ui/runtime-primitives.md)
```

This groups "browser-side concerns" (Plugins + Runtime Primitives) before the spec-authoring pages. RESEARCH §File Map line 118 confirms this placement. Title `Runtime Primitives` is planner discretion — alternatives `Runtime Attributes` or `Runtime DOM Contract` are equally acceptable.

**Format note:** the existing `Spec construction` entry uses a `./json-ui/…` relative path with leading `./` while every other entry uses `json-ui/…` without it. Phase 182 follows the majority style: `json-ui/runtime-primitives.md` (no leading `./`).

---

### `Cargo.toml` (workspace root) (MODIFY — workspace metadata)

**Analog:** self. The `[workspace.package].version` field has been bumped each release cycle.

#### Current state (verbatim, line 33)

```toml
[workspace.package]
version = "0.2.41"
```

#### Change

```toml
[workspace.package]
version = "0.2.42"
```

Single-character `1` → `2` change at line 33. No other workspace metadata changes.

Verified: `ferro-json-ui/Cargo.toml` declares `version.workspace = true`, so the bump propagates automatically (RESEARCH Assumption A5 verified at file read). No per-crate version edits needed.

---

### `Cargo.lock` (MODIFY — dependency lockfile)

**Analog:** commit `474f4490` — `chore: sync Cargo.lock to workspace version 0.2.41` (the immediate-prior precedent for this exact sync pattern). RESEARCH Pitfall 6 names this commit explicitly.

#### Precedent diff shape (verbatim from `git show 474f4490 -- Cargo.lock`, partial)

```diff
@@ -150,7 +150,7 @@ checksum = "…"
 [[package]]
 name = "app"
-version = "0.2.40"
+version = "0.2.41"
 dependencies = [
 …

@@ -1899,7 +1899,7 @@ dependencies = [
 [[package]]
 name = "ferro-ai"
-version = "0.2.40"
+version = "0.2.41"
 dependencies = [
 …

@@ -1947,7 +1947,7 @@ dependencies = [
 [[package]]
 name = "ferro-broadcast"
-version = "0.2.40"
+version = "0.2.41"
 dependencies = [
 …
```

That commit touched 25 workspace-crate entries (50 lines changed, all of the form `-version = "0.2.40"` → `+version = "0.2.41"`).

#### Change for Phase 182

Identical shape: every workspace-crate entry in `Cargo.lock` bumps from `"0.2.41"` to `"0.2.42"`. The mechanism is `cargo build --workspace` after editing `Cargo.toml` — Cargo regenerates `Cargo.lock` and the bump propagates automatically. The developer commits both files together. Do NOT hand-edit `Cargo.lock`; let `cargo` produce the diff. Verification: `grep -c 'version = "0.2.42"' Cargo.lock` should match the count from the prior cycle (25 entries).

**Pitfall guard (RESEARCH Pitfall 6):** if `Cargo.lock` is not synced, CI fails the lockfile-drift check on master push and the publish workflow aborts. The plan's verification step must run `cargo build --workspace` (or equivalently `cargo check --workspace`) immediately after the `Cargo.toml` edit, then `git status` to confirm `Cargo.lock` is dirty before committing.

---

## Shared Patterns

### S-1 — ES5 baseline JS in every runtime SOURCE string

**Source:** every file under `ferro-json-ui/src/runtime/*.rs`.
**Apply to:** `ferro-json-ui/src/runtime/hero_lazy.rs`.

Locks (CONTEXT.md §Existing Code Insights, RESEARCH §Anti-Patterns):
- `var`, never `let` / `const`.
- `function() {…}` expressions, never arrow functions.
- String concatenation with `+`, never template literals.
- Indexed `for (var i = 0; i < n; i++)` NodeList iteration, never `.forEach()`.
- No destructuring.
- No `console.log` for happy-path logging (only `console.error` for caught errors, precedent: `map.rs:317`).

### S-2 — Per-primitive `pub(super) const SOURCE: &str = r#"…"#;` declaration

**Source:** every file under `ferro-json-ui/src/runtime/*.rs` (verified across `sidebar.rs:1`, `dropdowns.rs:1`, `scroll_preserve.rs:1`, etc.).
**Apply to:** `ferro-json-ui/src/runtime/hero_lazy.rs`.

Locks:
- Visibility: `pub(super)` — module-private to `runtime/`.
- Type: `&str` (not `String`).
- Delimiter: `r#"…"#` raw string (allows embedded double quotes in attribute selectors).
- One trailing newline before `"#;` is NOT used by siblings — closing delimiter sits flush.

### S-3 — Early-return guard pattern

**Source:** every sibling primitive returns immediately when its selector finds nothing OR the required browser API is absent.
**Apply to:** `ferro-json-ui/src/runtime/hero_lazy.rs`.

Examples in tree:
- `sidebar.rs:8` — `if (!toggleBtn || !sidebarEl) return;`
- `dropdowns.rs:29` (in `initPopoverMenu`) — `if (!trigger) return;`

Phase 182 has TWO early-return guards (both at top of `setupLazyHeroes`):
1. `if (typeof IntersectionObserver === 'undefined') return;` — feature-detect (D-05).
2. `if (!els.length) return;` — selector-empty guard.

### S-4 — String-presence unit tests in `runtime/mod.rs`

**Source:** `ferro-json-ui/src/runtime/mod.rs` lines 95-191 (the entire `mod tests` block).
**Apply to:** the new `runtime_contains_lazy_hero_setup` test AND the two extended aggregate-array tests.

The convention: every primitive's behavior is "verified" by asserting that the assembled `FERRO_RUNTIME_JS` string contains the function name, the DOM API calls, and the public attribute literals. Behavioral correctness is verified manually in-browser per project precedent (D-07).

### S-5 — Defensive try/catch around browser-throw-prone calls

**Source:** `ferro-json-ui/src/runtime/scroll_preserve.rs:52`, `ferro-json-ui/src/runtime/sse.rs` (JSON.parse wrap).
**Apply to:** `ferro-json-ui/src/runtime/hero_lazy.rs` around `e.target.load()`.

Shape:
```js
try { unsafe_op(); } catch (_) {}
```
or with a short comment:
```js
try { unsafe_op(); } catch (e) { /* reason */ }
```

Phase 182 uses the bare `catch (_)` form for byte budget — the reason (Safari `<video>.load()` throw) lives in the docs page and CONTEXT.md, not as an inline comment.

### S-6 — Docs voice — scientific, neutral, no internal-strategy framing

**Source:** `docs/src/json-ui/forms.md`, `docs/src/json-ui/data-binding.md`, `docs/src/json-ui/plugins.md` (all three open with neutral third-person intro paragraphs naming the primitive and scope).
**Apply to:** `docs/src/json-ui/runtime-primitives.md`.

Locks (CLAUDE.md user instruction + project CLAUDE.md + RESEARCH §Project Constraints):
- No "we" / first-person plural.
- No tenant names (jetskiadriatic, gestiscilo).
- No phase references (Phase 182, ROADMAP).
- No marketing adjectives ("powerful", "best-in-class", "killer feature").
- No "the bet" / "load-bearing" / strategy-voice trigger phrases.
- Plain technical description of the contract: what attribute, on what element, what the runtime does, what the browser-support story is.

### S-7 — `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` before commit

**Source:** project `CLAUDE.md` §"Testing & Linting (MUST run before every commit)".
**Apply to:** every commit in the implementation plan.

CI enforces `-D warnings` with the exact `--all --all-targets` flag combination. Match this exactly in pre-commit verification — the local convenience version (`cargo clippy` without `--all-targets`) misses test-code warnings that CI catches (RESEARCH §Project Constraints, memory `feedback_ci_clippy_command_match.md`).

### S-8 — One CPU-intensive operation at a time

**Source:** memory `feedback_one_cpu_op_at_a_time.md` (referenced in user's global instructions).
**Apply to:** any plan-step verification that runs `cargo build`, `cargo test`, or `cargo clippy`.

Never chain or parallelize multiple `cargo` invocations within a single Bash call. Run them sequentially, reuse the prior step's evidence rather than re-running. The `cargo build --workspace` step after the `Cargo.toml` bump is one CPU op; the `cargo test --all-features` final verification is another — they must not be combined.

---

## No Analog Found

None. Every file in Phase 182's scope has a concrete in-tree analog. The single piece of planner-original logic (the `groups[m] = groups[m] || []` bucketing layer in `hero_lazy.rs`) has no sibling precedent but is documented in RESEARCH §Example 5 and Pitfall 4 — the planner should reference RESEARCH directly for this piece, not the sibling pattern.

---

## Metadata

**Analog search scope:**
- `ferro-json-ui/src/runtime/*.rs` (all 12 sibling primitives read)
- `ferro-json-ui/src/runtime/mod.rs` (full read — bundle assembly, dispatcher, test module)
- `ferro-json-ui/src/plugins/map.rs` §290-321 (IntersectionObserver precedent)
- `docs/src/SUMMARY.md` (full read — TOC structure)
- `docs/src/json-ui/forms.md` lines 1-60 (docs voice analog)
- `docs/src/json-ui/data-binding.md` lines 1-50 (docs voice analog)
- `docs/src/json-ui/plugins.md` lines 1-25 (docs structure analog)
- `Cargo.toml` lines 1-37 (workspace metadata)
- `Cargo.lock` (version grep + commit `474f4490` diff inspection)

**Files scanned:** 9 source/config files + 1 git commit for precedent diff.

**Pattern extraction date:** 2026-06-06

## PATTERN MAPPING COMPLETE
