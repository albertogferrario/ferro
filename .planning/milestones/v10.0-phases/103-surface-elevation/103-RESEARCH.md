# Phase 103: Surface Elevation - Research

**Researched:** 2026-03-25
**Domain:** Tailwind CSS semantic surface tokens, WCAG contrast in oklch, Rust runtime JS class management
**Confidence:** HIGH

## Summary

Phase 103 establishes a three-tier surface hierarchy across all ferro-json-ui components. The theme (`ferro-theme/assets/default.css`) already defines the three surface tokens correctly:
- `bg-background`: page-level (light: `oklch(100% 0 0)`, dark: `oklch(12% 0 0)`)
- `bg-surface`: panels/sidebar (light: `oklch(97% 0 0)`, dark: `oklch(17% 0 0)`)
- `bg-card`: cards/modals/dropdowns (light: `oklch(95% 0 0)`, dark: `oklch(20% 0 0)`)

The problem is not missing tokens — it is that `render_card`, `render_stat_card`, `render_modal`, and `render_notification_dropdown` all use `bg-background` where they should use `bg-card`. Additionally, the runtime JS in `ferro-json-ui/src/runtime.rs` uses hardcoded Tailwind color utilities (`bg-blue-500`, `bg-green-500`, `border-blue-600`, `text-gray-500`) instead of semantic token classes.

Phase 102 delivered the `has_class` helper and the structural test separation, so Phase 103 can safely change class strings without triggering a test avalanche. However, the four existing full-string cosmetic tests (Card line 2873, StatCard line 3915, NotificationDropdown, Modal) will need to be updated in the same commit as the class changes.

**Primary recommendation:** Change `bg-background` to `bg-card` in `render_card`, `render_modal`, `render_stat_card`, and `render_notification_dropdown`. Fix JS runtime hardcoded colors. Update the three full-string test assertions that reference the old classes. Verify dark mode contrast via OddContrast before committing.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SRF-01 | Card component uses `bg-card` (visually distinct from page `bg-background`) | `render_card` at line 381 uses `bg-background shadow-sm` — change to `bg-card shadow-sm`. One cosmetic test at line 2873 asserts `bg-background`, must be updated in same commit. |
| SRF-02 | Modal panel uses `bg-card` for elevated surface appearance | `render_modal` at line 423 uses `bg-background rounded-lg shadow-lg` — change to `bg-card rounded-lg shadow-lg`. No separate full-string test for modal inner panel class exists; line 2943 checks structural elements only. |
| SRF-03 | StatCard uses `bg-card` for elevated surface appearance | `render_stat_card` at line 1467 uses `bg-background rounded-lg shadow-sm` — change to `bg-card rounded-lg shadow-sm`. Cosmetic test at line 3915 asserts `bg-background rounded-lg shadow-sm`, must be updated. |
| SRF-04 | NotificationDropdown panel uses `bg-card` for elevated surface appearance | `render_notification_dropdown` at line 1612 uses `bg-background rounded-lg shadow-lg` — change to `bg-card rounded-lg shadow-lg`. The layout.rs DashboardLayout notification dropdown panel at line 261 also uses `bg-background` — must also change. |
| SRF-05 | Three-tier surface hierarchy enforced: background → surface → card | Hierarchy exists in CSS tokens. Phase 103 makes the component layer reflect it. Sidebar (`bg-background border-r` at line 1665 and layout.rs line 187) is intentional at background level — sidebar is a persistent frame, not an elevated surface. Table thead (`bg-surface` at line 598) correctly uses surface tier. |
| SRF-06 | All 8 critical dark mode token pairs pass WCAG 4.5:1 contrast ratio | Dark mode tokens defined in `default.css` lines 39-54. Must verify 8 pairs with OddContrast (oklch-native tool). Key pairs: text/background, text/surface, text/card, text-muted/background, primary/background, primary-foreground/primary, destructive/background, success/background. |
| SRF-07 | Runtime JS uses semantic token classes (bg-primary, not bg-blue-500) | `ferro-json-ui/src/runtime.rs` VARIANT_CLASSES object (lines 85-89) uses `bg-blue-500`, `bg-green-500`, `bg-yellow-500`, `bg-red-500`. Tab switcher (lines 251-256) uses `border-blue-600`, `text-blue-600`, `text-gray-500`, `hover:text-gray-700`. All must be replaced with semantic tokens. |
</phase_requirements>

## Standard Stack

### Core (no new dependencies)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tailwind CSS v4 CDN | `@4` (jsdelivr) | CSS utility framework | Already wired — `bg-card`, `bg-background`, `bg-surface` resolve via `@theme { --color-card: ... }` |
| ferro-theme/assets/default.css | project-local | Surface token definitions | Three tiers already defined correctly; no CSS changes needed for SRF-01 to SRF-05 |

### No new Rust dependencies required
All changes are string substitutions in existing Rust source files.

**Installation:** None required.

## Architecture Patterns

### Recommended File Change Map
```
ferro-json-ui/
├── src/render.rs     # SRF-01,02,03,04: bg-background → bg-card in 4 render functions
│                     # SRF-07: no changes here (toast HTML already uses semantic tokens)
│                     # Tests: update 2 full-string assertions (Card line 2873, StatCard line 3915)
└── src/runtime.rs    # SRF-07: fix VARIANT_CLASSES and tab switcher class manipulation

ferro-json-ui/
└── src/layout.rs     # SRF-04: DashboardLayout notification dropdown panel (line 261)

ferro-theme/
└── assets/default.css  # SRF-06: verify values with OddContrast, adjust if needed
```

### Pattern 1: Three-Tier Surface Hierarchy
**What:** Page background is the lowest layer. UI panels (sidebar, sticky header) sit at the surface tier. Floating/interactive elements (cards, modals, dropdowns) float at the card tier.
**When to use:** Any component that is visually contained and "lifted" from the page.

```
background (oklch 100% / 12% dark)  — body, table body, form inputs
    surface (oklch 97% / 17% dark)  — sidebar, table header, collapsible summary, hover states
        card (oklch 95% / 20% dark) — cards, modals, dropdowns, stat cards
```

**Correct vs incorrect assignments:**

| Component | Before (wrong) | After (correct) | Tier rationale |
|-----------|----------------|-----------------|----------------|
| Card outer div | `bg-background` | `bg-card` | Elevated surface floating on page |
| Modal inner panel | `bg-background` | `bg-card` | Elevated surface above overlay |
| StatCard outer div | `bg-background` | `bg-card` | Dashboard metric card floats above bg |
| NotificationDropdown panel | `bg-background` | `bg-card` | Dropdown floats above header |
| Sidebar | `bg-background` | keep `bg-background` | Persistent frame, not elevated |
| Table thead | `bg-surface` | keep `bg-surface` | Mid-tier panel stripe |
| Table body | `bg-background` | keep `bg-background` | Flush with page, not elevated |
| Form inputs | `bg-background` | keep `bg-background` | Form fields sit on page/card |

### Pattern 2: Semantic Token Classes in Runtime JS
**What:** The runtime JS manipulates CSS classes dynamically. It must use the same semantic token classes as the static HTML, not Tailwind palette utilities.
**When to use:** Any classList.add/remove in `runtime.rs` that changes visual appearance.

```javascript
// BEFORE (hardcoded, breaks with theme changes):
var VARIANT_CLASSES = {
    info: 'bg-blue-500',
    success: 'bg-green-500',
    warning: 'bg-yellow-500',
    error: 'bg-red-500'
};

// AFTER (semantic tokens, works with any ferro theme):
var VARIANT_CLASSES = {
    info: 'bg-primary',
    success: 'bg-success',
    warning: 'bg-warning',
    error: 'bg-destructive'
};
```

```javascript
// BEFORE (tab switcher — hardcoded blue):
t.classList.remove('border-transparent', 'text-gray-500', 'hover:text-gray-700');
t.classList.add('border-blue-600', 'text-blue-600');
// ...
t.classList.remove('border-blue-600', 'text-blue-600');
t.classList.add('border-transparent', 'text-gray-500', 'hover:text-gray-700');

// AFTER (semantic tokens):
t.classList.remove('border-transparent', 'text-text-muted', 'hover:text-text');
t.classList.add('border-primary', 'text-primary');
// ...
t.classList.remove('border-primary', 'text-primary');
t.classList.add('border-transparent', 'text-text-muted', 'hover:text-text');
```

Note: The `text-white` class on the dynamically-created toast element div and button in `showToast` should also be reviewed. Toast text color should be variant-appropriate. The static `render_toast` function already uses semantic classes (`text-primary`, `text-success`, etc.) — the JS `showToast` for SSE-driven toasts should mirror the same pattern.

### Pattern 3: WCAG 4.5:1 Contrast Verification with oklch
**What:** Standard contrast checkers convert colors to sRGB before computing contrast. oklch values can be mis-converted. Use OddContrast (oddcontrast.com) which is oklch-native.
**When to use:** Verifying dark mode token pairs before finalizing default.css values.

The 8 critical dark mode pairs to verify:

| Pair | Text color | Background | Requirement |
|------|-----------|------------|-------------|
| 1 | `text` `oklch(95% 0 0)` | `background` `oklch(12% 0 0)` | >= 4.5:1 |
| 2 | `text` `oklch(95% 0 0)` | `surface` `oklch(17% 0 0)` | >= 4.5:1 |
| 3 | `text` `oklch(95% 0 0)` | `card` `oklch(20% 0 0)` | >= 4.5:1 |
| 4 | `text-muted` `oklch(60% 0 0)` | `background` `oklch(12% 0 0)` | >= 4.5:1 |
| 5 | `primary-foreground` `oklch(100% 0 0)` | `primary` `oklch(65% 0.2 250)` | >= 4.5:1 |
| 6 | `primary` `oklch(65% 0.2 250)` | `background` `oklch(12% 0 0)` | >= 4.5:1 (links/labels on page bg) |
| 7 | `primary-foreground` `oklch(100% 0 0)` | `destructive` `oklch(60% 0.22 25)` | >= 4.5:1 |
| 8 | `secondary-foreground` `oklch(95% 0 0)` | `secondary` `oklch(60% 0.05 250)` | >= 4.5:1 |

**Analysis of current dark mode values (approximate — verify with OddContrast):**

The neutral token pairs (text vs background tiers) are all achromatic oklch. For achromatic oklch, L% maps predictably to WCAG relative luminance. Pairs 1-3 with text at L=95% vs backgrounds at L=12-20% should achieve very high contrast (estimated 15:1 to 12:1 range). Pair 4 with text-muted at L=60% vs background at L=12% may be close to the 4.5:1 threshold — this is the highest-risk pair.

The chromatic pairs (5-8) involving primary color `oklch(65% 0.2 250)` need actual tool verification because chroma affects perceived brightness.

**If a pair fails:** Adjust the failing token's L value in the dark mode `@theme` block. Increase text L (make brighter) or decrease background L (make darker). Each adjustment affects multiple pairs, so adjust minimally.

### Anti-Patterns to Avoid
- **Changing sidebar to `bg-card`:** The sidebar is a persistent layout frame, not an elevated surface. It should remain `bg-background` to avoid visual noise. The SRF-05 hierarchy places persistent shells at background tier.
- **Changing form inputs to `bg-card`:** Input fields sit on whatever surface contains them. `bg-background` is correct — they appear on both page background and card surfaces. Using `bg-card` would make inputs invisible on card backgrounds.
- **Changing table body to `bg-card`:** Table body content should be at background tier; table header at surface tier. Elevating table body to card would collapse the background/surface distinction.
- **Using `text-white` in JS toast:** Static toast uses variant-color text (`text-primary`, `text-success`). JS-created toasts should use the same class pattern, not `text-white` which bypasses the token system.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| WCAG contrast calculation | Manual oklch→sRGB math | OddContrast tool (oddcontrast.com) | oklch chroma affects relative luminance; tool handles it correctly |
| Semantic color mapping in JS | String parsing or CSS variable reads | Pre-defined VARIANT_CLASSES object with known token classes | Token classes are static strings; no runtime resolution needed |
| New surface token tier | Additional CSS custom property | Existing `--color-card` token | Token already exists with correct light/dark values |

**Key insight:** All three surface tokens are already defined and Tailwind already generates `bg-card`, `bg-surface`, and `bg-background` utilities. Phase 103 is purely a string substitution exercise in existing Rust source files.

## Common Pitfalls

### Pitfall 1: Forgetting layout.rs DashboardLayout notification dropdown
**What goes wrong:** `render_notification_dropdown` in `render.rs` is fixed (SRF-04), but the layout header's notification dropdown panel in `layout.rs` at line 261 also uses `bg-background`. This is the panel attached to the header bell button — a separate code path.
**Why it happens:** There are two independent paths that render notification dropdowns: the `Component::NotificationDropdown` in render.rs (standalone component) and the inline dropdown in `layout_header_html` in layout.rs (always-present header element).
**How to avoid:** Fix both occurrences in the same commit. Grep for `data-notification-dropdown` to find both.
**Warning signs:** Notification dropdown looks correct in standalone usage but still uses bg-background in the DashboardLayout header.

### Pitfall 2: Test assertion failures from full-string cosmetic tests
**What goes wrong:** `card_renders_title_and_description` at line 2873 asserts `html.contains("rounded-lg border border-border bg-background shadow-sm")`. Changing to `bg-card` breaks this test. Similarly `stat_card_renders_label_and_value` at line 3915 asserts `bg-background rounded-lg shadow-sm`.
**Why it happens:** These are cosmetic full-string tests established before Phase 102's structural separation.
**How to avoid:** Update both full-string assertions in the same commit as the class changes. The structural tests (`card_structural_title_and_description`, `stat_card_structural_value_and_label`) do not assert the background class and will pass through unchanged.
**Warning signs:** `cargo test -p ferro-json-ui` fails on `card_renders_title_and_description` or `stat_card_renders_label_and_value`.

### Pitfall 3: Tab switcher JS class list de-sync
**What goes wrong:** The tab switcher adds `border-blue-600 text-blue-600` for active state and removes them for inactive. When replacing with `border-primary text-primary`, the remove call must also be updated to remove `border-primary text-primary` (not `border-blue-600 text-blue-600`).
**Why it happens:** The JS code has paired add/remove calls — both sides must be updated together.
**How to avoid:** Update both the `classList.add` and the corresponding `classList.remove` in the same edit. The inactive-state `classList.add` (`border-transparent`, `text-gray-500`) must also change to semantic tokens (`border-transparent`, `text-text-muted`).
**Warning signs:** Tab switching visually works but leaves stale classes from the old palette on inactive tabs.

### Pitfall 4: Toast JS creates elements with `text-white` override
**What goes wrong:** The `showToast` function in runtime.js sets `el.className` to include `text-white` as a static class alongside the variant color class. This hardcodes white text for all JS-created toasts, bypassing the token system. In a theme where toast background is light-colored, white text would be invisible.
**Why it happens:** The original implementation assumed colored backgrounds always need white text.
**How to avoid:** Use `text-current` (inherits from parent) or omit text color from the wrapper div, relying on the variant classes from VARIANT_CLASSES to set the appropriate text color via `text-primary` etc.

### Pitfall 5: Failing contrast check requires careful token adjustment
**What goes wrong:** If `text-muted` (dark: `oklch(60% 0 0)`) fails 4.5:1 against `background` (dark: `oklch(12% 0 0)`), naively increasing text-muted to L=65% may fix pair 4 but affect its contrast against surface/card. Any token value change must be re-verified across all pairs that use that token.
**Why it happens:** Tokens are shared across multiple component contexts.
**How to avoid:** Adjust one token at a time, re-verify all pairs after each change.

## Code Examples

### SRF-01: Card — bg-background → bg-card
```rust
// Source: ferro-json-ui/src/render.rs, render_card function (line 379)

// BEFORE:
"<div class=\"rounded-lg border border-border bg-background shadow-sm\"><div class=\"p-6\">"

// AFTER:
"<div class=\"rounded-lg border border-border bg-card shadow-sm\"><div class=\"p-6\">"

// Also update the cosmetic test at line 2873:
// BEFORE: assert!(html.contains("rounded-lg border border-border bg-background shadow-sm"));
// AFTER:  assert!(html.contains("rounded-lg border border-border bg-card shadow-sm"));
```

### SRF-02: Modal — bg-background → bg-card
```rust
// Source: ferro-json-ui/src/render.rs, render_modal function (line 414)

// BEFORE:
"<div class=\"relative bg-background rounded-lg shadow-lg max-w-lg w-full mx-4 p-6\">"

// AFTER:
"<div class=\"relative bg-card rounded-lg shadow-lg max-w-lg w-full mx-4 p-6\">"
// No full-string cosmetic test for this specific class string; modal structural test passes unchanged.
```

### SRF-03: StatCard — bg-background → bg-card
```rust
// Source: ferro-json-ui/src/render.rs, render_stat_card function (line 1465)

// BEFORE (both instances at lines 1467 and 1500):
String::from("<div class=\"bg-background rounded-lg shadow-sm p-4 border border-border\">")

// AFTER:
String::from("<div class=\"bg-card rounded-lg shadow-sm p-4 border border-border\">")

// Also update cosmetic test at line 3915:
// BEFORE: assert!(html.contains("bg-background rounded-lg shadow-sm"));
// AFTER:  assert!(html.contains("bg-card rounded-lg shadow-sm"));
```

### SRF-04: NotificationDropdown — bg-background → bg-card (two locations)
```rust
// Location 1: ferro-json-ui/src/render.rs, render_notification_dropdown (line 1612)
// BEFORE:
"<div class=\"hidden absolute right-0 mt-2 w-80 bg-background rounded-lg shadow-lg border border-border z-50\" data-notification-panel>"
// AFTER:
"<div class=\"hidden absolute right-0 mt-2 w-80 bg-card rounded-lg shadow-lg border border-border z-50\" data-notification-panel>"

// Location 2: ferro-json-ui/src/layout.rs, layout_header_html (line 259-261)
// BEFORE:
"<div data-notification-dropdown class=\"hidden absolute right-0 top-full mt-1 w-80 \
 bg-background rounded-lg shadow-lg border border-border z-50\"></div></div>"
// AFTER:
"<div data-notification-dropdown class=\"hidden absolute right-0 top-full mt-1 w-80 \
 bg-card rounded-lg shadow-lg border border-border z-50\"></div></div>"
```

### SRF-07: Runtime JS — semantic token replacement
```javascript
// Source: ferro-json-ui/src/runtime.rs, FERRO_RUNTIME_JS constant

// BEFORE (lines 85-89):
var VARIANT_CLASSES = {
    info: 'bg-blue-500',
    success: 'bg-green-500',
    warning: 'bg-yellow-500',
    error: 'bg-red-500'
};

// AFTER:
var VARIANT_CLASSES = {
    info: 'bg-primary',
    success: 'bg-success',
    warning: 'bg-warning',
    error: 'bg-destructive'
};

// BEFORE toast element class string (line 101):
el.className = 'flex items-start gap-3 px-4 py-3 rounded-lg shadow-lg text-white max-w-sm ' +
    colorClass + ' opacity-0 transition-opacity duration-300';

// AFTER (remove text-white, use text-current to inherit from variant class):
el.className = 'flex items-start gap-3 px-4 py-3 rounded-lg shadow-lg text-current max-w-sm ' +
    colorClass + ' opacity-0 transition-opacity duration-300';

// BEFORE tab switcher (lines 251-256):
t.classList.remove('border-transparent', 'text-gray-500', 'hover:text-gray-700');
t.classList.add('border-blue-600', 'text-blue-600');
// ...
t.classList.remove('border-blue-600', 'text-blue-600');
t.classList.add('border-transparent', 'text-gray-500', 'hover:text-gray-700');

// AFTER:
t.classList.remove('border-transparent', 'text-text-muted', 'hover:text-text');
t.classList.add('border-primary', 'text-primary');
// ...
t.classList.remove('border-primary', 'text-primary');
t.classList.add('border-transparent', 'text-text-muted', 'hover:text-text');
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| All elevated components use `bg-background` | Cards/modals/dropdowns use `bg-card` | Phase 103 | Visual depth: floating elements are distinguishable from page background |
| JS runtime uses Tailwind palette classes (`bg-blue-500`) | JS runtime uses semantic token classes (`bg-primary`) | Phase 103 | Theme portability: dynamic toasts and tab switching match the active theme |
| Tab active state uses `border-blue-600 text-blue-600` | Tab active state uses `border-primary text-primary` | Phase 103 | Custom themes (non-blue primary) display correctly |

**Correctly-used patterns to preserve:**
- `bg-surface` on table thead: correct — header stripe is mid-tier
- `bg-card` on avatar initials span: correct — already using card tier
- `bg-card` on sidebar active nav item: correct — elevated indicator within sidebar frame
- `bg-background` on form inputs: correct — inputs sit on containing surface
- `hover:bg-surface` on pagination/buttons: correct — surface as hover feedback for background-level items

## Open Questions

1. **toast `text-white` removal impact**
   - What we know: Static `render_toast` uses `text-primary`, `text-success` etc. on the toast div. The JS `showToast` uses `text-white` statically.
   - What's unclear: Whether `text-current` suffices or whether specific text colors should be added to VARIANT_CLASSES (e.g., `info: 'bg-primary text-primary-foreground'`).
   - Recommendation: Use the pattern `bg-primary text-primary-foreground` in VARIANT_CLASSES to mirror the static toast pattern exactly.

2. **Dark mode contrast for `text-muted` (oklch 60% 0 0) vs background tiers**
   - What we know: L=60% neutral vs L=12% background. Approximate WCAG relative luminance for L=60% neutral is ~0.318; L=12% neutral is ~0.012. Contrast ratio ≈ (0.318+0.05)/(0.012+0.05) ≈ 5.9:1. This appears to pass.
   - What's unclear: Exact values without a color tool — chroma is zero so the math is straightforward, but official tool verification is required by SRF-06.
   - Recommendation: Run OddContrast verification as a dedicated task before any token adjustments.

3. **Checklist component also uses `bg-background` (line 1500)**
   - What we know: `render_checklist` at line 1500 uses `bg-background rounded-lg shadow-sm`. This is a dashboard-adjacent component.
   - What's unclear: Whether the checklist should be elevated to `bg-card` (it behaves like a card) or remain at background.
   - Recommendation: Treat checklist as an elevated surface (card tier) since it is a contained widget with title and shadow. Change to `bg-card` in SRF-01 or as part of SRF-05 (three-tier enforcement). Include in the Card task.

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
| SRF-01 | Card renders with `bg-card` class | unit | `cargo test -p ferro-json-ui card_renders_title` | existing — update assertion |
| SRF-01 | Card structural test survives class change | unit | `cargo test -p ferro-json-ui card_structural` | existing in structural_tests mod |
| SRF-02 | Modal panel uses `bg-card` | unit | `cargo test -p ferro-json-ui modal_renders_details` | existing — structural test already passes (no bg class assertion) |
| SRF-03 | StatCard uses `bg-card` | unit | `cargo test -p ferro-json-ui stat_card_renders_label` | existing — update assertion at line 3915 |
| SRF-04 | NotificationDropdown panel uses `bg-card` | unit | `cargo test -p ferro-json-ui notification_dropdown` | existing — check if test asserts bg class |
| SRF-05 | Sidebar stays at `bg-background` | unit | `cargo test -p ferro-json-ui sidebar_renders` | existing — no change needed |
| SRF-06 | Dark mode 8 pairs >= 4.5:1 contrast | manual | OddContrast tool verification | manual-only — no automated WCAG test exists |
| SRF-07 | Runtime JS uses semantic classes | unit | `cargo test -p ferro-json-ui runtime_js` | add new test asserting VARIANT_CLASSES contains `bg-primary` |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-json-ui`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] New test for `runtime.rs` VARIANT_CLASSES — assert FERRO_RUNTIME_JS contains `bg-primary` (not `bg-blue-500`)
- [ ] New test for tab switcher — assert FERRO_RUNTIME_JS contains `border-primary` (not `border-blue-600`)

*(Existing full-string cosmetic tests at lines 2873 and 3915 must be updated in the same commit as class changes — not new tests, just assertion string updates.)*

## Sources

### Primary (HIGH confidence)
- Direct codebase inspection: `ferro-json-ui/src/render.rs` — all render functions, line numbers verified
- Direct codebase inspection: `ferro-json-ui/src/runtime.rs` — full FERRO_RUNTIME_JS constant, all hardcoded classes found
- Direct codebase inspection: `ferro-json-ui/src/layout.rs` — layout_header_html DashboardLayout notification dropdown panel
- Direct codebase inspection: `ferro-theme/assets/default.css` — three-tier surface tokens with oklch values confirmed correct
- `ferro-json-ui/src/render.rs` structural tests — `has_class` helper confirmed present at line 5187, structural_tests module confirmed present at line 5215

### Secondary (MEDIUM confidence)
- WCAG 2.1 contrast algorithm — relative luminance calculation for achromatic oklch values (zero chroma = straight luminance mapping)
- OddContrast tool (oddcontrast.com) — referenced as oklch-native WCAG verification tool; verified it exists and supports oklch input

### Tertiary (LOW confidence)
- Approximate contrast ratio calculations for dark mode token pairs — estimated from oklch L values without running through official tool; SRF-06 requires tool verification

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all tokens already defined in CSS
- Architecture (what to change): HIGH — all file locations and line numbers identified from direct code inspection
- Surface tier assignment: HIGH — three-tier hierarchy is unambiguous from the token definitions
- JS runtime changes: HIGH — exact class strings identified from source
- Dark mode contrast: MEDIUM — achromatic pairs likely pass by large margin; chromatic pairs (primary, destructive) need tool verification
- Pitfalls: HIGH — all identified from direct code reading

**Research date:** 2026-03-25
**Valid until:** 2026-06-25 (token definitions and render.rs structure are stable)
