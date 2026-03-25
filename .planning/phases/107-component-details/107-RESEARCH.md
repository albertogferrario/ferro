# Phase 107: Component Details - Research

**Researched:** 2026-03-25
**Domain:** Inline SVG injection, CSS shimmer animation, tab active state, Rust HTML generation
**Confidence:** HIGH

## Summary

Phase 107 is the final visual polish phase for v10.0. It addresses six component-level details: replacing emoji/HTML-entity indicators with inline SVG, replacing `animate-pulse` with a shimmer keyframe animation on the skeleton loader, adding `font-semibold` to active tabs, and replacing the breadcrumb `/` text separator with an SVG chevron. All work is in `ferro-json-ui/src/render.rs` with one coordination point in `ferro-json-ui/src/runtime.rs` (tabs JS switcher must also add `font-semibold` for CMP-04).

Two patterns are already proven: (1) `concat!` macro for inline SVG strings (established in Phase 105 for the select dropdown arrow at line 873) and (2) the `css_head` field in `RenderResult` for injecting `<style>` blocks into the page `<head>` (used by the plugin asset system). The shimmer animation is the only case requiring CSS injection — the keyframe cannot be expressed as a Tailwind class and must be emitted as a `<style type="text/tailwindcss">` block appended to `css_head` during skeleton rendering.

Three emoji/entity locations must be updated: `render_notification_dropdown()` at line 1622 (`&#x1F514;` bell emoji), `render_header()` at lines 1760 and 1764 (`&#x1F514;` in the standalone Header component), and `render_collapsible()` at line 1423 (`&#9660;` down-arrow entity). The DashboardLayout `layout_header_html()` in `layout.rs` already uses a proper SVG bell (lines 244-257) and does NOT need changes. Existing full-string tests for `alert_info_variant` (line 2212) and `alert_with_title` (line 2278) will need updating once SVG icons are added to alerts.

**Primary recommendation:** One plan covering all six requirements in `render.rs`, with targeted test updates where existing exact-string assertions would break on the new SVG output.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CMP-01 | Alert renders inline SVG icon per variant (info, success, warning, error) | `render_alert()` at line 1166 — currently no icon element; `concat!` macro SVG pattern from Phase 105 (line 873) applies directly. Four icon designs needed: info (circle-i), success (checkmark circle), warning (triangle exclamation), error (circle x). |
| CMP-02 | Skeleton uses shimmer animation instead of `animate-pulse` | `render_skeleton()` at line 1247 — replaces `animate-pulse` class with a shimmer CSS keyframe. Keyframe must be injected via `css_head` in `RenderResult` since Tailwind CDN cannot generate it from a class name alone. |
| CMP-03 | Breadcrumb uses SVG chevron separator instead of `/` text | `render_breadcrumb()` at line 1280 — `<span>/</span>` on line 1281 replaced with inline SVG chevron-right using `concat!` macro pattern. |
| CMP-04 | Active tab has `font-semibold` weight | `render_tabs()` at lines 481-510 — active tab `text` variable gets `font-semibold` added; `runtime.rs` JS switcher (line 251) also needs `font-semibold` in classList add/remove. |
| CMP-05 | NotificationDropdown bell renders as SVG icon (not emoji) | `render_notification_dropdown()` at line 1622 uses `&#x1F514;`; `render_header()` at lines 1760/1764 also uses `&#x1F514;`. Both replaced with inline SVG. DashboardLayout `layout_header_html()` already has SVG — no change needed there. |
| CMP-06 | Collapsible renders rotating SVG chevron indicator | `render_collapsible()` at line 1423 — `&#9660;` down-arrow entity replaced with inline SVG chevron using existing `group-open:rotate-180 transition-transform` Tailwind classes already present on the wrapping `<span>`. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust `concat!` macro | stdlib | Compile-time string concatenation for SVG HTML | Established in Phase 105 (select chevron, line 873); avoids `format!` allocation for static strings |
| Tailwind v4 (CDN) | v4.x | Utility classes for icon sizing, color, transitions | Already integrated; `h-4 w-4`, `currentColor`, `group-open:rotate-180`, `transition-transform` all proven working |
| `<style type="text/tailwindcss">` | HTML | Custom keyframe injection for shimmer animation | Used by theme injection in `framework/src/json_ui/mod.rs` line 112; same mechanism for skeleton shimmer |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `RenderResult.css_head` | project-local | Appends `<style>` blocks to page `<head>` | For shimmer keyframe — only needed when Skeleton component is rendered |
| `has_class()` helper | project-local | Resilient class assertions in structural tests | All new tests; avoids full class-string fragility |

**Installation:** No new dependencies required.

## Architecture Patterns

### Recommended Project Structure

No new files needed. All changes in:
```
ferro-json-ui/src/
├── render.rs     # All six render functions modified
└── runtime.rs    # Tab JS switcher: add font-semibold to classList operations
```

### Pattern 1: Inline SVG via `concat!` Macro

**What:** Static SVG markup compiled into a `&'static str` using `concat!`. The SVG uses `currentColor` for stroke/fill so it inherits the parent's `text-*` color token automatically.

**When to use:** Any static SVG icon (breadcrumb chevron, notification bell, collapsible chevron, alert icons). Does not allocate at runtime unlike `format!`.

**Example:**
```rust
// Source: ferro-json-ui/src/render.rs lines 873-878 (Phase 105 select chevron)
html.push_str(concat!(
    "<span class=\"pointer-events-none absolute inset-y-0 right-3 flex items-center\" aria-hidden=\"true\">",
    "<svg class=\"h-4 w-4 text-text-muted\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 20 20\" fill=\"currentColor\">",
    "<path fill-rule=\"evenodd\" d=\"M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z\" clip-rule=\"evenodd\"/>",
    "</svg></span>"
));
```

For alert icons that vary by variant, use a `match` with `concat!`-produced literals per arm:
```rust
// Pattern for CMP-01 alert variant icons
const ICON_INFO: &str = concat!(
    "<span aria-hidden=\"true\" class=\"inline-flex shrink-0 h-4 w-4 mr-3 mt-0.5\">",
    "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 20 20\" fill=\"currentColor\">",
    "<path fill-rule=\"evenodd\" d=\"M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z\" clip-rule=\"evenodd\"/>",
    "</svg></span>"
);

let icon = match props.variant {
    AlertVariant::Info    => ICON_INFO,
    AlertVariant::Success => ICON_SUCCESS,
    AlertVariant::Warning => ICON_WARNING,
    AlertVariant::Error   => ICON_ERROR,
};
```

### Pattern 2: CSS Shimmer via `RenderResult.css_head`

**What:** The `RenderResult` struct has a `css_head: String` field that gets appended to the page `<head>` after the Tailwind CDN tag (see `framework/src/json_ui/mod.rs` lines 120-125). Rendering a Skeleton component appends a `<style>` block with the `@keyframes shimmer` definition and a `.shimmer` class.

**When to use:** Only for the Skeleton component. No other component needs keyframe injection.

**Example:**
```rust
// Source: concept based on ferro-json-ui/src/render.rs RenderResult.css_head field (line 54)
// and framework/src/json_ui/mod.rs lines 120-125 showing css_head appended to head

// In render_to_html_with_plugins() or a new shimmer-aware path:
const SHIMMER_CSS: &str = concat!(
    "<style>",
    "@keyframes shimmer {",
    "  0% { background-position: -200% 0; }",
    " 100% { background-position: 200% 0; }",
    "}",
    ".ferro-shimmer {",
    "  background: linear-gradient(90deg, var(--color-card) 25%, var(--color-border) 50%, var(--color-card) 75%);",
    "  background-size: 200% 100%;",
    "  animation: shimmer 1.5s infinite;",
    "}",
    "</style>"
);

// Skeleton div uses .ferro-shimmer class instead of animate-pulse
format!(
    "<div class=\"ferro-shimmer {rounded}\" style=\"width: {width}; height: {height}\"></div>"
)
```

The `css_head` field is accumulated in `RenderResult` (line 54 in render.rs) and merged into the page `<head>` by the framework integration layer. Multiple Skeleton components on the same page would inject the `<style>` block multiple times — deduplicate by checking `css_head.contains("@keyframes shimmer")` before appending.

### Pattern 3: Active Tab `font-semibold` + JS Sync

**What:** The active tab render adds `font-semibold` to the active text class. The JS tab switcher in `runtime.rs` must also add/remove `font-semibold` when switching tabs to maintain consistent state.

**When to use:** CMP-04 only.

**Current code (render.rs line 481-484):**
```rust
// Source: render.rs lines 481-484
let text = if is_active {
    "text-primary"           // MISSING font-semibold
} else {
    "text-text-muted hover:text-text"
};
```

**After Phase 107:**
```rust
let text = if is_active {
    "text-primary font-semibold"
} else {
    "text-text-muted hover:text-text"
};
```

**runtime.rs tab switcher (line 251-256) must also sync:**
```javascript
// Source: runtime.js lines 251-256 (current)
t.classList.remove('border-transparent', 'text-text-muted', 'hover:text-text');
t.classList.add('border-primary', 'text-primary');
// After Phase 107:
t.classList.remove('border-transparent', 'text-text-muted', 'hover:text-text', 'font-semibold');
t.classList.add('border-primary', 'text-primary', 'font-semibold');
// And for inactive tabs:
t.classList.remove('border-primary', 'text-primary', 'font-semibold');
t.classList.add('border-transparent', 'text-text-muted', 'hover:text-text');
```

### Pattern 4: SVG Bell Icon (replacing emoji)

**What:** The `&#x1F514;` bell emoji in `render_notification_dropdown()` (line 1622) and `render_header()` (lines 1760, 1764) is replaced with an inline SVG bell path. Use `currentColor` stroke so the icon inherits `text-text-muted` automatically.

**Reference SVG already in codebase** (layout.rs line 244 — DashboardLayout uses this exact SVG):
```
M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9
```

This path is proven working in the DashboardLayout (line 244-257). Reuse the identical SVG so bell icons are visually consistent across all render paths.

### Anti-Patterns to Avoid

- **`animate-pulse` for shimmer:** Tailwind's `animate-pulse` produces an opacity fade, not a shimmer sweep. CMP-02 explicitly requires shimmer (gradient sweep), not pulse. Do not add `shimmer` as a Tailwind class name — it has no built-in definition.
- **HTML entities for icons:** `&#9660;` (▼) and `&#x1F514;` (🔔) render differently across operating systems and fonts. This is the problem CMP-05/CMP-06 exist to fix. Never introduce new entity-based icons.
- **Formatting SVG with `format!`:** For static SVGs with no interpolated values, use `concat!` to avoid string allocation. Only use `format!` when the SVG needs runtime values (none of the six requirements do).
- **Updating DashboardLayout bell:** `layout_header_html()` in `layout.rs` already uses SVG. Changing it would break tests unnecessarily.
- **Breaking existing exact-string tests:** `alert_info_variant` (line 2212) asserts `html.contains("bg-primary/10 border-primary text-primary")` on the container class — this survives SVG addition. But `alert_with_title` (line 2278) asserts `html.contains("<p>Details here</p>")` — this also survives if the SVG is placed before the `<p>`. Check that no existing tests assert on absence of `<svg` in alert output.
- **Injecting shimmer CSS without deduplication:** Multiple Skeleton components on one page must not inject multiple `<style>` blocks with the same keyframe definition. Check before appending.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SVG icon library | Custom `Icon` component type | Inline SVG constants via `concat!` | Phase 107 needs ~6 icons; a full icon system is explicitly out of scope per REQUIREMENTS.md |
| Custom animation utilities | Tailwind plugin | CSS `@keyframes` in `<style>` block via `css_head` | Framework already has a `css_head` injection mechanism; CDN cannot generate keyframes from class names |
| Cross-platform emoji rendering | Font fallback CSS | Inline SVG | SVGs are binary-independent; emoji rendering is OS-controlled and inconsistent |

**Key insight:** The `concat!` macro pattern (already established for the select chevron in Phase 105) is the exact right tool for all static SVG injection in this phase. No new Rust types or trait implementations required.

## Common Pitfalls

### Pitfall 1: Shimmer keyframe injected on every render
**What goes wrong:** Each call to `render_skeleton()` appends `SHIMMER_CSS` to `css_head`, producing `<style>` blocks duplicated N times for N skeletons on one page.
**Why it happens:** `render_node()` calls `render_skeleton()` which mutates the accumulation string without checking for duplicates.
**How to avoid:** Before appending, check: `if !result.css_head.contains("@keyframes shimmer") { result.css_head.push_str(SHIMMER_CSS); }`. The check is O(n) on a small string — acceptable.
**Warning signs:** HTML source shows multiple identical `<style>` blocks for shimmer.

### Pitfall 2: JS tab switcher not updated for `font-semibold`
**What goes wrong:** On initial load, the active tab (rendered server-side) shows `font-semibold`. After clicking another tab, the JS switcher adds `text-primary` but not `font-semibold`, making the newly-active tab render in normal weight.
**Why it happens:** `runtime.rs` JS has hardcoded lists of classes to add/remove (lines 251-256). Adding `font-semibold` to server-side render without updating JS produces inconsistent state after first tab switch.
**How to avoid:** Update both the `render_tabs()` active class string in render.rs AND the classList operations in runtime.rs together in the same task/commit.
**Warning signs:** Active tab looks bold on first load but becomes normal weight after clicking.

### Pitfall 3: Alert SVG layout breaks with title
**What goes wrong:** When an alert has a title (`<h4>` block) and an SVG icon, the icon and text don't align correctly if the container uses block layout.
**Why it happens:** The current alert renders `<h4>` then `<p>` in a block container. Adding an SVG icon inline requires the outer container to be flex.
**How to avoid:** Change the alert container to `flex items-start gap-3` and wrap text content in a `<div>`. The icon goes in a `<span>` with `shrink-0` before the text div.
**Warning signs:** Icon and text appear stacked vertically instead of side-by-side; title misaligned with body text.

### Pitfall 4: Two bell emoji locations, only one found
**What goes wrong:** `render_notification_dropdown()` gets updated but `render_header()` still uses `&#x1F514;`, so the standalone `Header` component continues rendering emoji.
**Why it happens:** The bell icon appears in three functions: `render_notification_dropdown()` (line 1622), `render_header()` (lines 1760 and 1764), and `layout_header_html()` (already SVG — no change needed). Searching for `&#x1F514;` finds both problem locations.
**How to avoid:** Search the full render.rs for `1F514` before declaring complete. Both must use SVG.
**Warning signs:** `header_renders_notification_count_badge` test still passes (it doesn't assert on icon type) but visual output shows emoji in standalone Header component.

### Pitfall 5: Collapsible `transition-transform` class already present
**What goes wrong:** The collapsible summary already has `group-open:rotate-180 transition-transform` on the indicator `<span>`. These classes must be preserved when replacing the `&#9660;` entity with SVG.
**Why it happens:** The SVG replaces only the icon content inside the `<span>`, not the `<span>` wrapper itself. If the entire `<span>` is rebuilt carelessly, these transition classes may be lost.
**How to avoid:** Keep the `<span class="text-text-muted group-open:rotate-180 transition-transform">` wrapper unchanged; replace only its inner content with `concat!` SVG string. Or use `concat!` for the full span+svg unit to ensure nothing is dropped.

## Code Examples

Verified patterns from the existing codebase:

### SVG chevron-right for Breadcrumb (CMP-03)
```rust
// Pattern: reuse Phase 105 select-chevron shape but right-pointing
// Source: extends render.rs line 873 pattern
const BREADCRUMB_SEP: &str = concat!(
    "<span aria-hidden=\"true\" class=\"text-text-muted\">",
    "<svg class=\"h-4 w-4 inline\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 20 20\" fill=\"currentColor\">",
    "<path fill-rule=\"evenodd\" d=\"M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z\" clip-rule=\"evenodd\"/>",
    "</svg></span>"
);

// Replace: html.push_str("<span>/</span>");
// With:    html.push_str(BREADCRUMB_SEP);
```

### SVG Bell Icon (CMP-05) — exact path from layout.rs line 244
```rust
const BELL_SVG_PATH: &str =
    "M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9";

const BELL_SVG: &str = concat!(
    "<svg class=\"h-5 w-5\" fill=\"none\" stroke=\"currentColor\" viewBox=\"0 0 24 24\">",
    "<path stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" \
     d=\"M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9\"/>",
    "</svg>"
);
// Replace: html.push_str("<span class=\"text-xl\">&#x1F514;</span>");
// With:    html.push_str(BELL_SVG);
```

### Collapsible SVG Chevron (CMP-06)
```rust
// Source: Current line 1423 uses &#9660; inside this span:
// <span class=\"text-text-muted group-open:rotate-180 transition-transform\">&#9660;</span>
// After:
const CHEVRON_DOWN: &str = concat!(
    "<svg class=\"h-4 w-4\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 20 20\" fill=\"currentColor\">",
    "<path fill-rule=\"evenodd\" d=\"M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z\" clip-rule=\"evenodd\"/>",
    "</svg>"
);
// The wrapping <span> keeps its rotation classes unchanged.
```

### Shimmer CSS (CMP-02)
```rust
// Source: concept from framework/src/json_ui/mod.rs line 112 (theme CSS injection)
const SHIMMER_CSS: &str = concat!(
    "<style>",
    "@keyframes ferro-shimmer{0%{background-position:-200% 0}100%{background-position:200% 0}}",
    ".ferro-shimmer{",
    "background:linear-gradient(90deg,var(--color-card,#f1f5f9) 25%,var(--color-border,#e2e8f0) 50%,var(--color-card,#f1f5f9) 75%);",
    "background-size:200% 100%;",
    "animation:ferro-shimmer 1.5s ease-in-out infinite;",
    "}",
    "</style>"
);

// render_skeleton() output:
format!(
    "<div class=\"ferro-shimmer {rounded}\" style=\"width: {width}; height: {height}\"></div>"
)
```

Use a prefixed name `ferro-shimmer` to avoid colliding with any user-defined `.shimmer` class.

### Existing structural test pattern (from render.rs line 5237)
```rust
fn has_class(html: &str, class: &str) -> bool {
    html.contains(&format!("class=\"{class}\""))
        || html.contains(&format!("class=\"{class} "))
        || html.contains(&format!(" {class}\""))
        || html.contains(&format!(" {class} "))
}

// New CMP-04 test:
assert!(
    has_class(&html, "font-semibold"),
    "active tab should have font-semibold"
);
// New CMP-02 test:
assert!(html.contains("ferro-shimmer"), "skeleton should use shimmer class");
assert!(!html.contains("animate-pulse"), "skeleton should not use animate-pulse");
// New CMP-01 test:
assert!(html.contains("<svg"), "alert should contain SVG icon");
assert!(html.contains("role=\"alert\""), "alert should have role=alert");
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `&#x1F514;` emoji bell | Inline SVG bell | Phase 107 | Cross-platform consistency; font-independent rendering |
| `&#9660;` HTML entity arrow | Inline SVG chevron | Phase 107 | Consistent sizing with Tailwind `h-*`/`w-*`; inherits color via `currentColor` |
| `/` text separator in breadcrumb | SVG chevron-right | Phase 107 | Semantic, scalable, matches design systems |
| `animate-pulse` on skeleton | CSS `@keyframes` shimmer | Phase 107 | Shimmer is a sweep animation (gradient moves); pulse is a fade — fundamentally different visual |
| `font-medium` on all tabs | `font-semibold` on active tab only | Phase 107 | Visual distinction between active and inactive tabs without color being the only signal |

**Deprecated/outdated:**
- `&#x1F514;` in `render_notification_dropdown()` and `render_header()`: replaced by SVG
- `&#9660;` in `render_collapsible()`: replaced by SVG
- `<span>/</span>` in `render_breadcrumb()`: replaced by SVG

## Open Questions

1. **Shimmer gradient tokens**
   - What we know: The Tailwind v4 semantic tokens use `--color-card` and `--color-border`. The shimmer gradient uses these as the light/dark stops.
   - What's unclear: Whether CSS custom property references work correctly inside `@keyframes` when injected as a `<style>` block alongside the Tailwind CDN.
   - Recommendation: Use `var(--color-card, #f1f5f9)` with fallback values so the shimmer degrades gracefully if tokens are not resolved. Alternatively, use literal light-mode values since this is CDN mode.

2. **Alert layout change — flex container**
   - What we know: Adding an icon to the alert requires the container to become `flex items-start gap-3` to place icon and text side-by-side.
   - What's unclear: Whether existing tests assert on the container class string `rounded-md border p-4 {variant_classes}`.
   - Recommendation: Check `alert_info_variant` (line 2212) — it asserts `html.contains("bg-primary/10 border-primary text-primary")` which is a substring check. Adding flex classes to the container will not break this. The container can become `rounded-md border p-4 flex items-start gap-3 {variant_classes}` safely.

3. **Notification bell in `render_header()` — test coverage**
   - What we know: `header_renders_notification_count_badge()` (line 4443) asserts on `data-notification-count` attribute and badge styling, not on the bell icon type. Replacing emoji with SVG will not break this test.
   - What's unclear: Whether any test asserts on `&#x1F514;` presence directly.
   - Recommendation: Search for `1F514` in test assertions before implementing. If none found, the change is test-safe.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` via `cargo test` |
| Config file | none — cargo workspace |
| Quick run command | `cargo test -p ferro-json-ui 2>&1 \| tail -5` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CMP-01 | Alert renders `<svg>` before message content, per variant | unit | `cargo test -p ferro-json-ui alert_svg_icon -- --nocapture` | Wave 0 |
| CMP-02 | Skeleton renders `.ferro-shimmer` class, not `animate-pulse` | unit | `cargo test -p ferro-json-ui skeleton_shimmer_class` | Wave 0 |
| CMP-03 | Breadcrumb separator renders `<svg>` not `/` text | unit | `cargo test -p ferro-json-ui breadcrumb_svg_separator` | Wave 0 |
| CMP-04 | Active tab has `font-semibold`; inactive does not | unit | `cargo test -p ferro-json-ui tab_active_font_semibold` | Wave 0 |
| CMP-05 | Notification dropdown and standalone Header bell renders `<svg>`, no emoji | unit | `cargo test -p ferro-json-ui notification_bell_svg` | Wave 0 |
| CMP-06 | Collapsible chevron is `<svg>` with rotation classes preserved | unit | `cargo test -p ferro-json-ui collapsible_svg_chevron` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-json-ui 2>&1 | tail -5`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `ferro-json-ui/src/render.rs` — 6 new structural tests for CMP-01 through CMP-06, placed in `mod structural_tests`
- [ ] Update existing exact-string tests that will break: verify `alert_info_variant`, `alert_with_title`, `alert_without_title`, `skeleton_default`, `collapsible_renders_details_summary`, `notification_dropdown_renders_bell_icon` — adjust assertions as needed after implementation

*(No new test files — all tests live inline in render.rs following the project pattern)*

## Sources

### Primary (HIGH confidence)
- Direct code read of `ferro-json-ui/src/render.rs` — all render function locations verified by line number
- Direct code read of `ferro-json-ui/src/layout.rs` — DashboardLayout bell SVG at lines 244-257 confirmed
- Direct code read of `ferro-json-ui/src/runtime.rs` — tab switcher classList operations at lines 251-256 confirmed
- Direct code read of `framework/src/json_ui/mod.rs` — `css_head` assembly and `<style type="text/tailwindcss">` injection mechanism at lines 120-125
- `.planning/REQUIREMENTS.md` — CMP-01 through CMP-06 requirements verified verbatim
- `.planning/phases/106-interactive-states/106-RESEARCH.md` — architectural patterns confirmed

### Secondary (MEDIUM confidence)
- Phase 105 plan (105-01-PLAN.md) — `concat!` macro pattern for SVG injection confirmed as working approach

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all patterns (`concat!`, `css_head`, `has_class` helper) proven working in this codebase
- Architecture: HIGH — all six render function locations verified by direct line-number inspection
- Pitfalls: HIGH — identified from direct code inspection; JS/server sync issue (CMP-04) confirmed by reading both render.rs and runtime.rs

**Research date:** 2026-03-25
**Valid until:** 2026-04-25 (stable codebase; no external dependencies in flux)
