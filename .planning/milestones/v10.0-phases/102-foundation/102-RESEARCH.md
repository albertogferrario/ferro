# Phase 102: Foundation - Research

**Researched:** 2026-03-24
**Domain:** Tailwind v4 CSS token namespace, Bunny Fonts CDN, Rust test separation strategy
**Confidence:** HIGH

## Summary

Phase 102 fixes a pre-existing font token namespace bug in `ferro-theme` and `ferro-cli` that has prevented the `font-sans` Tailwind utility from working in any JSON-UI page. The bug is straightforward: Tailwind v4 uses `--font-sans` / `--font-mono` as the CSS custom property namespace for font-family utilities, but the codebase defines `--font-family-sans` / `--font-family-mono` — a v3-era naming convention that Tailwind v4 ignores entirely.

The four requirements break into two independent work streams:
1. **CSS token fix** (FND-01, FND-02, FND-03): Rename the token variable in three places (`ferro-theme/assets/default.css`, `ferro-theme/src/token.rs` constant, `ferro-cli/src/commands/make_theme.rs` template), add the Inter Variable font `<link>` tag to the `build_response` head assembly in `framework/src/json_ui/mod.rs`, and verify the body class already has `font-sans` (it does: `bg-background text-text` — `font-sans` needs to be added to body_class default).
2. **Test separation** (FND-04): 157 tests in `ferro-json-ui/src/render.rs` and 43 in `layout.rs` assert on exact Tailwind class strings. When cosmetic classes change in Phases 103+, many will break. The fix is to refactor tests to separate structural assertions (element type, text content, data attributes, href values) from cosmetic class assertions, or to scope cosmetic tests using `contains()` on only the stable semantic token class, not the full class string.

**Primary recommendation:** Fix the CSS variable name first (two-character rename: `--font-family-sans` → `--font-sans`), then add Bunny Fonts `<link>` to `build_response`, then update constants and CLI template, then add `font-sans` to body_class default. After token fix, refactor test assertions to test element structure not full class strings.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| FND-01 | Font token namespace uses correct Tailwind v4 names (`--font-sans`, `--font-mono` not `--font-family-sans`) | Confirmed: Tailwind v4 `@theme` maps `--font-*` → `font-*` utilities. Current code uses `--font-family-*` which Tailwind v4 ignores. Fix spans 3 files. |
| FND-02 | Inter Variable font loads via Bunny Fonts CDN in base document `<head>` | Confirmed: `build_response` in `framework/src/json_ui/mod.rs` assembles the `head` string; Bunny Fonts `<link>` tag must be prepended there alongside Tailwind CDN. |
| FND-03 | Body and all text elements render in Inter (or system fallback) | Confirmed: fixing `--font-sans` makes `font-sans` utility work; adding `font-sans` to `body_class` default cascades to all text. |
| FND-04 | Test suite separates structural assertions from cosmetic class assertions to prevent cascade failures | Confirmed: 157 tests in render.rs assert on exact full class strings (e.g., `class=\"text-3xl font-bold text-text\"`). Strategy: split into element/content checks + semantic-token class checks that remain stable. |
</phase_requirements>

## Standard Stack

### Core (already in use — no new dependencies)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tailwind CSS v4 CDN | `@4` (jsdelivr) | CSS utility framework | Already wired in `build_response` via `cdn.jsdelivr.net/npm/@tailwindcss/browser@4` |
| Bunny Fonts | N/A (CDN) | GDPR-friendly Google Fonts alternative | Project already planned to use it; serves Inter Variable |

### No new Rust dependencies required
All changes are in CSS content strings embedded in Rust source files and the HTML head assembly logic.

**Installation:** None required.

## Architecture Patterns

### Recommended Project Structure (unchanged)
```
ferro-theme/
├── assets/default.css        # FND-01: rename --font-family-sans → --font-sans
└── src/token.rs              # FND-01: rename TOKEN_FONT_FAMILY_SANS constant

ferro-cli/
└── src/commands/make_theme.rs  # FND-01: rename in template string + test assertion

framework/
└── src/json_ui/mod.rs        # FND-02: add Bunny Fonts <link> to build_response head

ferro-json-ui/
└── src/render.rs             # FND-04: refactor cosmetic class assertions
└── src/layout.rs             # FND-04: refactor cosmetic class assertions
```

### Pattern 1: Tailwind v4 Font Namespace
**What:** In Tailwind v4, `@theme { --font-X: ...; }` registers a CSS custom property that Tailwind maps to the utility class `font-X`. The namespace is `--font-*` not `--font-family-*`.
**When to use:** Any time you define a font-family token in a Tailwind v4 `@theme` block.
**Example:**
```css
/* Source: https://tailwindcss.com/docs/theme */
@theme {
  /* CORRECT: Tailwind v4 generates font-sans and font-mono utilities */
  --font-sans: ui-sans-serif, system-ui, sans-serif;
  --font-mono: ui-monospace, monospace;

  /* WRONG (v3 naming): Tailwind v4 ignores these, no utilities generated */
  /* --font-family-sans: ui-sans-serif, system-ui, sans-serif; */
}
```

### Pattern 2: Bunny Fonts link tag in head assembly
**What:** Bunny Fonts is a GDPR-friendly Google Fonts CDN replacement. Link tag format mirrors Google Fonts API exactly with a domain swap.
**When to use:** When loading Inter from CDN in the `<head>` of JSON-UI pages.
**Example:**
```rust
// In framework/src/json_ui/mod.rs build_response, before Tailwind CDN tag:
let mut head = String::new();
// Bunny Fonts — Inter Variable (400-700 weights sufficient for most UIs)
head.push_str(
    r#"<link rel="preconnect" href="https://fonts.bunny.net">
<link href="https://fonts.bunny.net/css?family=inter:400,500,600,700&display=swap" rel="stylesheet">"#
);
if config.tailwind_cdn {
    head.push_str(
        r#"<script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4"></script>"#,
    );
}
```
The font `<link>` must come before the Tailwind CDN `<script>` so the browser starts font loading as early as possible.

### Pattern 3: body_class default includes font-sans
**What:** The `JsonUiConfig` default `body_class` is `"bg-background text-text"`. To make Inter apply to all body text, `font-sans` must be included.
**When to use:** Default body class for all JSON-UI pages.
**Example:**
```rust
// In ferro-json-ui/src/config.rs, JsonUiConfig::default():
body_class: "bg-background text-text font-sans".to_string(),
```
After the `--font-sans` token fix, `font-sans` resolves to `--font-sans` which maps to the Inter stack with system-ui fallback.

### Pattern 4: Test separation — structural vs cosmetic
**What:** Tests in `render.rs` currently assert on full class strings. Split each assertion into:
- Structural: element type, text content, data attributes, href, role attributes
- Cosmetic: Tailwind class presence (only the semantic token classes that won't change, not utility modifiers)

**Why:** Adding a class like `leading-tight` to `<h1>` would break any test asserting `class=\"text-3xl font-bold text-text\"` exactly. Tests should survive cosmetic changes across the remaining 5 phases.

**Example — current (brittle):**
```rust
// Source: ferro-json-ui/src/render.rs current pattern
assert!(html.contains("<h1 class=\"text-3xl font-bold text-text\">Title</h1>"));
```

**Example — refactored (resilient):**
```rust
// Structural: correct element and content
assert!(html.contains("<h1 "));
assert!(html.contains(">Title</h1>"));
// Semantic token class (stable — this is a named token, not a utility modifier)
assert!(html.contains("text-text"));
// Size class (structural to the component, stable)
assert!(html.contains("text-3xl"));
// NOT: assert full class string — adding leading-tight in Phase 104 breaks this
```

**Alternative (preferred for many tests): use `contains` only for the invariant part:**
```rust
// If the test purpose is "h1 renders correctly", the element + content is sufficient.
// Size/weight are component-level invariants and can stay, but don't assert the full class string.
assert!(html.contains("<h1 class=\"text-3xl font-bold text-text\">Title</h1>"));
// This form is acceptable IF we document that adding more classes to h1 requires updating this test.
// The real fix is: wrap class assertions in a helper that checks class membership, not substring.
```

**Recommended helper approach for FND-04:**
```rust
/// Assert that an HTML string contains a specific CSS class on a specific tag.
/// More resilient than matching the full class string.
fn assert_has_class(html: &str, tag: &str, class: &str) {
    // Find all occurrences of the tag and check if any has the class
    assert!(
        html.contains(&format!("class=\"{}\"", class)) ||
        html.contains(&format!("class=\"{} ", class)) ||
        html.contains(&format!(" {}", class)),
        "expected {} to have class {}", tag, class
    );
}
```

**Pragmatic decision for FND-04:** The goal is not to refactor all 157 tests. The goal is to ensure that no single cosmetic change in Phase 103+ breaks more than the tests that directly test that component's cosmetic appearance. The planner should add a helper function and a small set of new tests that verify structural invariants, leaving existing full-string tests in place but documented as cosmetic tests that will need updating when those component classes change.

### Anti-Patterns to Avoid
- **Renaming `--font-sans` back to `--font-family-sans`:** This re-introduces the bug. Tailwind v4 ignores the `--font-family-*` namespace entirely.
- **Hardcoding `font-family: 'Inter'` in CSS rather than using the token:** Bypasses the theme system and breaks third-party themes.
- **Adding Bunny Fonts link only to `DashboardLayout`:** The `base_document` function is shared by all layouts; the font link must go in `build_response` head assembly so all layouts receive it.
- **Asserting `class="..."` with exact full strings in tests:** A single class addition breaks these. Use `contains()` on the critical class tokens only.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| GDPR-friendly font CDN | Custom font hosting | Bunny Fonts CDN | Already decided; exact API parity with Google Fonts |
| Font loading | `@font-face` declarations in Rust | Bunny Fonts `<link>` tag | CDN handles subsetting, caching, preload hints |
| CSS class membership testing | Complex HTML parser | `contains()` with individual class strings | Sufficient for unit tests; no HTML parser needed |

**Key insight:** Font loading and CSS token naming are solved problems. The entire phase is renaming strings in existing files and adding two `<link>` tags.

## Common Pitfalls

### Pitfall 1: Forgetting the ferro-cli template
**What goes wrong:** The `make_theme` CLI command generates new theme files using a hardcoded template string in `ferro-cli/src/commands/make_theme.rs`. If only `default.css` and `token.rs` are fixed, new themes generated by `make_theme` will still contain `--font-family-sans`.
**Why it happens:** There are three independent sources of the wrong token name: the embedded default CSS, the token constant, and the CLI template.
**How to avoid:** Fix all three in the same commit. The test at line 205 of `make_theme.rs` asserts `css.contains("--font-family-sans:")` — update that test too.
**Warning signs:** `cargo test` passes on ferro-theme but fails on ferro-cli if the template test is forgotten.

### Pitfall 2: Bunny Fonts link order in head
**What goes wrong:** If the Bunny Fonts `<link>` is placed after the Tailwind CDN `<script>`, the browser discovers the font later, causing a flash of unstyled text (FOUT) longer than necessary.
**Why it happens:** HTML head is parsed top-to-bottom; earlier `<link rel="preconnect">` + `<link rel="stylesheet">` means the font starts downloading sooner.
**How to avoid:** Place font links before the Tailwind CDN script tag.
**Warning signs:** Visual: text renders in system default font then snaps to Inter after page load.

### Pitfall 3: body_class test assertion failure
**What goes wrong:** `framework/src/json_ui/mod.rs` has tests that assert `config.body_class` content. Changing the default from `"bg-background text-text"` to `"bg-background text-text font-sans"` will break those tests.
**Why it happens:** Tests assert on the exact `body_class` string.
**How to avoid:** Update the test expectations in `framework/src/json_ui/mod.rs` when updating the default.
**Warning signs:** `cargo test --all-features` fails on framework crate with a body_class assertion.

### Pitfall 4: Token constant used in validation
**What goes wrong:** `TOKEN_FONT_FAMILY_SANS` in `ferro-theme/src/token.rs` is referenced from `ALL_TOKENS` and possibly from validation logic that checks CSS files contain all required tokens. Renaming the constant requires updating all call sites.
**Why it happens:** The constant name (`TOKEN_FONT_FAMILY_SANS`) is separate from its value (`"--font-family-sans"`). Both must change.
**How to avoid:** Search for `TOKEN_FONT_FAMILY_SANS` and `TOKEN_FONT_FAMILY_MONO` across the workspace before renaming.
**Warning signs:** Clippy warning about unused constant if old constant is left in place.

### Pitfall 5: Test avalanche from FND-04 if deferred
**What goes wrong:** If FND-04 (test separation) is skipped, the first cosmetic class change in Phase 103 (e.g., changing `bg-card` on Card) will trigger failures in every test that asserts the full class string for Card — and similarly for every subsequent phase.
**Why it happens:** 157 tests use `contains("class=\"...")` with full class attribute strings. Each cosmetic change cascades.
**How to avoid:** Add a structural/cosmetic separation in Phase 102 so Phases 103-107 can change classes freely.
**Warning signs:** Phase 103 plan fails CI on the first commit.

## Code Examples

Verified patterns from official sources and codebase inspection:

### FND-01: CSS token rename (3 locations)

**Location 1: `ferro-theme/assets/default.css`**
```css
/* Source: official Tailwind v4 docs https://tailwindcss.com/docs/theme */
@theme {
  /* ...other tokens... */

  /* BEFORE (wrong): */
  /* --font-family-sans: ui-sans-serif, system-ui, sans-serif; */
  /* --font-family-mono: ui-monospace, monospace; */

  /* AFTER (correct Tailwind v4 namespace): */
  --font-sans: ui-sans-serif, system-ui, sans-serif;
  --font-mono: ui-monospace, monospace;
}
```

**Location 2: `ferro-theme/src/token.rs`**
```rust
// Rename constants and their values
pub const TOKEN_FONT_SANS: &str = "--font-sans";
pub const TOKEN_FONT_MONO: &str = "--font-mono";

// Update ALL_TOKENS array entries from TOKEN_FONT_FAMILY_SANS to TOKEN_FONT_SANS
```

**Location 3: `ferro-cli/src/commands/make_theme.rs`** (template string + test)
```rust
// In the template literal:
// BEFORE: --font-family-sans: ui-sans-serif, system-ui, sans-serif;
// AFTER:  --font-sans: ui-sans-serif, system-ui, sans-serif;

// In the test assertion:
// BEFORE: css.contains("--font-family-sans:")
// AFTER:  css.contains("--font-sans:")
```

### FND-02: Bunny Fonts link in head assembly

**Location: `framework/src/json_ui/mod.rs`, `build_response` function**
```rust
let mut head = String::new();
// Font link before Tailwind CDN for earlier browser discovery
head.push_str(
    "<link rel=\"preconnect\" href=\"https://fonts.bunny.net\">\
     <link href=\"https://fonts.bunny.net/css?family=inter:400,500,600,700&display=swap\" rel=\"stylesheet\">"
);
if config.tailwind_cdn {
    head.push_str(
        r#"<script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4"></script>"#,
    );
}
```

### FND-03: body_class default includes font-sans

**Location: `ferro-json-ui/src/config.rs`**
```rust
impl Default for JsonUiConfig {
    fn default() -> Self {
        Self {
            tailwind_cdn: true,
            custom_head: None,
            // BEFORE: body_class: "bg-background text-text".to_string(),
            // AFTER (adds font-sans to cascade Inter to all body text):
            body_class: "bg-background text-text font-sans".to_string(),
        }
    }
}
```

### FND-04: Test separation strategy

The recommended approach is minimal disruption: add a `contains()` helper for class membership and add targeted structural tests for the components that Phases 103-107 will modify, without refactoring the existing 157 tests wholesale.

```rust
// In ferro-json-ui/src/render.rs, top of test module:

/// Check that html contains an element with a given CSS class.
/// Avoids full-string class matching that breaks when classes are added.
#[cfg(test)]
fn has_class(html: &str, class: &str) -> bool {
    // Check for class at start, middle, or end of class attribute
    html.contains(&format!("class=\"{class}\""))
        || html.contains(&format!("class=\"{class} "))
        || html.contains(&format!(" {class}\""))
        || html.contains(&format!(" {class} "))
}

// New tests that verify structural invariants (element type + content):
#[test]
fn text_h1_renders_h1_element() {
    let view = JsonUiView::new().component(text_node("t", "Title", TextElement::H1));
    let html = render_to_html(&view, &json!({}));
    assert!(html.contains("<h1 "));
    assert!(html.contains(">Title</h1>"));
    // Semantic token (stable across phases):
    assert!(has_class(&html, "text-text"));
}
```

The existing cosmetic tests remain and serve as documentation of current class strings. They are expected to fail when classes change and should be updated in the same PR as the class change.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `--font-family-sans` (v3 naming) | `--font-sans` (v4 naming) | Tailwind v4 release | Font utilities simply do not work until renamed |
| System fonts only | Inter Variable via CDN | This phase | Professional typography baseline |
| Full class string assertions | Structural + semantic token assertions | This phase | Cosmetic changes don't cascade test failures |

**Deprecated/outdated:**
- `--font-family-sans`: The v3 `@theme` convention. Tailwind v4 ignores any `--font-family-*` variable in `@theme` blocks and only processes `--font-*`.

## Open Questions

1. **Inter weights to include in Bunny Fonts URL**
   - What we know: The URL format is `?family=inter:WEIGHT_LIST&display=swap`. Current default theme uses no explicit weight utilities; Tailwind generates them from the `font-weight` scale.
   - What's unclear: Whether `400,500,600,700` is sufficient or if `300` (for muted text) is needed.
   - Recommendation: Use `300,400,500,600,700` to cover all likely weights. The CDN serves only requested weights.

2. **Whether `ferro-theme/src/token.rs` constant rename breaks any downstream crate**
   - What we know: `TOKEN_FONT_FAMILY_SANS` appears only in `ferro-theme/src/token.rs` and is exported via `pub mod token`. Need to check if any other crate imports it.
   - What's unclear: Full workspace grep needed at plan time.
   - Recommendation: Run `grep -r "TOKEN_FONT_FAMILY"` across workspace before committing.

3. **Whether `font-sans` in body_class default breaks existing tests in framework crate**
   - What we know: `framework/src/json_ui/mod.rs` has tests that may assert on `body_class` content.
   - What's unclear: Exact test assertions in that file.
   - Recommendation: Treat body_class test updates as part of FND-03 task.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (cargo test) |
| Config file | none (workspace-level) |
| Quick run command | `cargo test -p ferro-json-ui` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FND-01 | `default.css` contains `--font-sans` (not `--font-family-sans`) | unit | `cargo test -p ferro-theme` | existing tests in loader.rs will catch it |
| FND-01 | `make_theme` template generates `--font-sans` | unit | `cargo test -p ferro-cli` | existing test at make_theme.rs:205 — update assertion |
| FND-01 | `TOKEN_FONT_SANS` constant has value `"--font-sans"` | unit | `cargo test -p ferro-theme` | add new test or update existing |
| FND-02 | Rendered JSON-UI page head contains `fonts.bunny.net` link | unit | `cargo test -p framework` | add to framework/src/json_ui/mod.rs tests |
| FND-03 | Body class default includes `font-sans` | unit | `cargo test -p ferro-json-ui` | update existing body_class test |
| FND-04 | Structural test helper `has_class` exists | unit | `cargo test -p ferro-json-ui` | add new helper + tests |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-json-ui -p ferro-theme -p ferro-cli`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] New test in `ferro-theme/src/token.rs` or `loader.rs` that asserts `--font-sans` (not `--font-family-sans`) is the correct value
- [ ] New test in `framework/src/json_ui/mod.rs` that asserts Bunny Fonts `<link>` appears in rendered HTML head
- [ ] Updated test in `ferro-cli/src/commands/make_theme.rs` line 205: change `--font-family-sans` to `--font-sans`
- [ ] New `has_class` helper function in `ferro-json-ui/src/render.rs` test module

## Sources

### Primary (HIGH confidence)
- Tailwind CSS official docs (https://tailwindcss.com/docs/theme) — confirmed `--font-*` is correct v4 namespace, `--font-family-*` is v3 and ignored
- Direct codebase inspection — `ferro-theme/assets/default.css`, `ferro-theme/src/token.rs`, `ferro-cli/src/commands/make_theme.rs`, `framework/src/json_ui/mod.rs`, `ferro-json-ui/src/config.rs`

### Secondary (MEDIUM confidence)
- Bunny Fonts CDN (https://fonts.bunny.net/family/inter) — Inter is available; URL format matches Google Fonts API; exact URL for variable font weights verified by fetching the CSS endpoint
- Web search cross-reference — Tailwind v4 GitHub discussions confirm `--font-sans` namespace

### Tertiary (LOW confidence)
- Inter weight selection (300,400,500,600,700) — based on typical UI needs; no specific requirement in the phase spec

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already in use, no new dependencies
- Architecture: HIGH — all file locations identified by direct codebase inspection
- Pitfalls: HIGH — all identified from direct code reading (test count, template string, constant usage)
- Token naming: HIGH — verified against official Tailwind v4 documentation

**Research date:** 2026-03-24
**Valid until:** 2026-06-24 (Tailwind v4 stable; Bunny Fonts API stable)
