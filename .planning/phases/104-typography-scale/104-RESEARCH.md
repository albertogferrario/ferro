# Phase 104: Typography Scale - Research

**Researched:** 2026-03-25
**Domain:** Tailwind CSS typography utilities (leading, tracking), Rust string substitution in ferro-json-ui render pipeline
**Confidence:** HIGH

## Summary

Phase 104 adds line-height and letter-spacing classes to heading and body text elements in `ferro-json-ui`. The token infrastructure and Tailwind utilities are already fully present — `leading-tight`, `leading-snug`, `leading-relaxed`, `tracking-tight` are all standard Tailwind v4 utility classes generated automatically. No CSS changes to `ferro-theme/assets/default.css` are needed.

The primary targets are:
1. `render_text` in `ferro-json-ui/src/render.rs` (line 1054) — the `TextElement` renderer for all five text variants
2. Two inline h2/h3 heading strings in `render_page_header` (line 350) and `render_card` / `render_modal` / `render_checklist` (lines 384, 424, 1501) that emit headings without typography scale classes
3. One inconsistency in `layout.rs` (line 172): sidebar group label uses `text-text` where `render.rs` line 1682 uses `text-text-muted` — TYP-05 requires the consistent form

Phase 102 established the `has_class` helper and `structural_tests` module. Phase 103 confirmed the test-update protocol: cosmetic full-string assertions must be updated in the same commit as the class changes. The structural tests in `structural_tests::h1_structural_element_and_semantic_class` et al. are already annotated with "Phase 104 adds leading-tight tracking-tight" comments and will pass without modification.

**Primary recommendation:** Add `leading-tight tracking-tight` to H1 and H2 renders, `leading-snug` to H3 renders, and `leading-relaxed` to P/Div/Section renders — all in `render_text`. Update the six cosmetic full-string tests that assert the old class strings. Fix the `layout.rs` sidebar group label inconsistency as part of TYP-05.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| TYP-01 | H1 renders with `leading-tight tracking-tight` | `render_text` at line 1058 currently: `"<h1 class=\"text-3xl font-bold text-text\">"`. Add `leading-tight tracking-tight` to class string. Cosmetic test `text_h1_variant` at line 1889 asserts old string — must update. |
| TYP-02 | H2 renders with `leading-tight tracking-tight` | `render_text` at line 1060 and `render_page_header` at line 350 both emit h2. Both need `leading-tight tracking-tight`. Cosmetic test `text_h2_variant` at line 1896 and `test_render_page_header_title_only` at line 4925 both assert old class strings — must update. |
| TYP-03 | H3 renders with `leading-snug` | `render_text` at line 1063, plus inline h3 in `render_card` (line 384), `render_modal` (line 424), `render_checklist` (line 1501) — all need `leading-snug`. Cosmetic test `text_h3_variant` at line 1903 asserts old string — must update. Tests at lines 2872, 2944 also assert h3 class strings — must update. |
| TYP-04 | Body text (P, Div, Section) renders with `leading-relaxed` | `render_text` for `TextElement::P` (line 1057), `TextElement::Div` (line 1066), `TextElement::Section` (line 1068) — add `leading-relaxed`. Cosmetic tests at lines 1873, 1882 assert P class string — must update. Div and Section have no full-string cosmetic tests (structural tests only). |
| TYP-05 | Muted text uses consistent `text-text-muted` across all components | `layout.rs` line 172: sidebar group label uses `text-xs font-semibold text-text` — should be `text-xs font-semibold text-text-muted` to match `render.rs` line 1682. All other muted-text usages already use `text-text-muted` consistently. Update layout.rs test at line 930 if it asserts the old class string. |
</phase_requirements>

## Standard Stack

### Core (no new dependencies)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tailwind CSS v4 CDN | `@4` (jsdelivr) | Typography utilities | `leading-tight`, `leading-snug`, `leading-relaxed`, `tracking-tight` are standard Tailwind scale utilities — auto-generated, no configuration needed |
| ferro-theme/assets/default.css | project-local | Token definitions | No changes required for Phase 104 — typography tokens are Tailwind scale, not custom CSS properties |

### No new Rust dependencies required
All changes are class string additions in existing Rust source files.

**Installation:** None required.

## Architecture Patterns

### Recommended File Change Map
```
ferro-json-ui/
├── src/render.rs   # TYP-01/02/03/04: add leading-/tracking- in render_text
│                   # TYP-02: add leading-tight tracking-tight to render_page_header h2 (line 350)
│                   # TYP-03: add leading-snug to render_card h3 (line 384)
│                   # TYP-03: add leading-snug to render_modal h3 (line 424)
│                   # TYP-03: add leading-snug to render_checklist h3 (line 1501)
│                   # Tests: update cosmetic full-string assertions (lines 1873, 1882, 1889, 1896, 1903, 2872, 2944, 4925)
└── src/layout.rs   # TYP-05: fix sidebar group label text-text → text-text-muted (line 172)
```

### Pattern 1: Typography Class Additions to render_text
**What:** The `render_text` function at line 1054 is the single source for text element rendering. All five heading/body variants live here.
**When to use:** TYP-01 through TYP-04 all target this function first.

```rust
// Source: ferro-json-ui/src/render.rs, fn render_text (line 1054)

// BEFORE:
TextElement::P       => format!("<p class=\"text-base text-text\">{content}</p>"),
TextElement::H1      => format!("<h1 class=\"text-3xl font-bold text-text\">{content}</h1>"),
TextElement::H2      => format!("<h2 class=\"text-2xl font-semibold text-text\">{content}</h2>"),
TextElement::H3      => format!("<h3 class=\"text-xl font-semibold text-text\">{content}</h3>"),
TextElement::Span    => format!("<span class=\"text-base text-text\">{content}</span>"),
TextElement::Div     => format!("<div class=\"text-base text-text\">{content}</div>"),
TextElement::Section => format!("<section class=\"text-base text-text\">{content}</section>"),

// AFTER:
TextElement::P       => format!("<p class=\"text-base leading-relaxed text-text\">{content}</p>"),
TextElement::H1      => format!("<h1 class=\"text-3xl font-bold leading-tight tracking-tight text-text\">{content}</h1>"),
TextElement::H2      => format!("<h2 class=\"text-2xl font-semibold leading-tight tracking-tight text-text\">{content}</h2>"),
TextElement::H3      => format!("<h3 class=\"text-xl font-semibold leading-snug text-text\">{content}</h3>"),
TextElement::Span    => format!("<span class=\"text-base text-text\">{content}</span>"),
TextElement::Div     => format!("<div class=\"text-base leading-relaxed text-text\">{content}</div>"),
TextElement::Section => format!("<section class=\"text-base leading-relaxed text-text\">{content}</section>"),
```

Note: `TextElement::Span` does not receive `leading-relaxed` — spans are inline and do not establish their own line-height context. Only block-level body elements (P, Div, Section) get `leading-relaxed`.

### Pattern 2: Inline Headings in Container Components
**What:** Several container components hardcode heading HTML strings inline rather than calling `render_text`. These also need typography classes.
**When to use:** After `render_text` is updated, scan for all remaining `<h2`, `<h3` strings in render.rs.

```rust
// render_page_header (line 350) — h2 title:
// BEFORE:
"<h2 class=\"text-2xl font-semibold text-text\">{}</h2>"
// AFTER:
"<h2 class=\"text-2xl font-semibold leading-tight tracking-tight text-text\">{}</h2>"

// render_card (line 384) — h3 title:
// BEFORE:
"<h3 class=\"text-lg font-semibold text-text\">{}</h3>"
// AFTER:
"<h3 class=\"text-lg font-semibold leading-snug text-text\">{}</h3>"

// render_modal (line 424) — h3 title:
// BEFORE:
"<h3 class=\"text-lg font-semibold text-text\">{}</h3>"
// AFTER:
"<h3 class=\"text-lg font-semibold leading-snug text-text\">{}</h3>"

// render_checklist (line 1501) — h3 title:
// BEFORE:
"<h3 class=\"text-sm font-semibold text-text\">{}</h3>"
// AFTER:
"<h3 class=\"text-sm font-semibold leading-snug text-text\">{}</h3>"
```

The `<h4>` in `render_alert` (line 1156) uses `"<h4 class=\"font-semibold mb-1\">"` — no `text-text` class, no leading class required by this phase's requirements (no TYP requirement covers h4).

### Pattern 3: TYP-05 Muted Text Consistency
**What:** Two code paths render sidebar group labels. `render.rs` (line 1682) correctly uses `text-text-muted`. `layout.rs` (line 172) uses `text-text` instead — an inconsistency.
**When to use:** TYP-05 fix: align `layout.rs` to match `render.rs`.

```rust
// layout.rs, layout_sidebar_group (line 172):
// BEFORE:
"<p class=\"px-2 py-1 text-xs font-semibold text-text\">{}</p>"
// AFTER:
"<p class=\"px-2 py-1 text-xs font-semibold text-text-muted\">{}</p>"
```

### Tailwind v4 Typography Scale Reference
These are standard Tailwind utilities — no custom configuration needed:

| Class | CSS property | Value |
|-------|-------------|-------|
| `leading-tight` | `line-height` | `1.25` |
| `leading-snug` | `line-height` | `1.375` |
| `leading-relaxed` | `line-height` | `1.625` |
| `tracking-tight` | `letter-spacing` | `-0.025em` |

### Anti-Patterns to Avoid
- **Adding `leading-relaxed` to Span:** Inline elements inherit line-height from their block parent. Adding `leading-relaxed` to spans is redundant at best, confusing at worst.
- **Adding leading classes to non-text components:** Table cells (`<td>`), labels (`<label>`), buttons, and badges have their own spacing rhythms. Phase 104 scope is strictly: H1, H2, H3 (via TextElement), P, Div, Section (via TextElement), plus the inline headings in card/modal/checklist/page-header.
- **Changing StatCard value paragraph:** The `<p class="text-2xl font-bold text-text">` at line 1476/1482 is a numeric display element — its value is a statistic, not body text. No `leading-relaxed` needed.
- **Changing the h2 in layout.rs sidebar app-name span (line 234):** That span renders as `text-lg font-semibold text-text` and is a UI label, not a document heading.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Line-height values | Custom CSS or inline styles | Tailwind `leading-*` utilities | Already in CDN, consistent scale, responsive |
| Letter-spacing values | Custom CSS or inline styles | Tailwind `tracking-tight` | Already in CDN, pairs correctly with Inter Variable |
| Muted text color token | New CSS custom property | `text-text-muted` (existing token) | `--color-text-muted` already defined in default.css |

**Key insight:** This phase is purely additive — class string insertions only. No new tokens, no new utilities, no CSS file edits.

## Common Pitfalls

### Pitfall 1: Missing inline headings in container components
**What goes wrong:** Developer updates `render_text` but forgets the four inline heading strings in `render_page_header`, `render_card`, `render_modal`, and `render_checklist`.
**Why it happens:** These heading strings are not routed through `render_text` — they are hardcoded in each container render function.
**How to avoid:** After updating `render_text`, grep for all `<h2` and `<h3` strings in render.rs and update each occurrence.
**Warning signs:** H1/H2/H3 via `Component::Text` get leading classes, but card/modal/page-header headings do not — visual inconsistency in the rendered demo app.

### Pitfall 2: Cosmetic full-string test failures
**What goes wrong:** Five cosmetic tests assert the exact class string without the new leading/tracking classes:
- `text_p_variant` (line 1882) — asserts `"<p class=\"text-base text-text\">Paragraph</p>"`
- `render_view_with_component_wraps_in_div` (line 1873) — asserts `"<p class=\"text-base text-text\">Hello</p>"`
- `text_h1_variant` (line 1889) — asserts `"<h1 class=\"text-3xl font-bold text-text\">Title</h1>"`
- `text_h2_variant` (line 1896) — asserts `"<h2 class=\"text-2xl font-semibold text-text\">Subtitle</h2>"`
- `text_h3_variant` (line 1903) — asserts `"<h3 class=\"text-xl font-semibold text-text\">Section</h3>"`

Plus two more from container component tests:
- `card_renders_title_and_description` (line 2872) — asserts `"<h3 class=\"text-lg font-semibold text-text\">My Card</h3>"`
- `modal_renders_details_summary` (line 2944) — asserts `"<h3 class=\"text-lg font-semibold text-text\">Confirm</h3>"`
- `test_render_page_header_title_only` (line 4925) — asserts `"<h2 class=\"text-2xl font-semibold text-text\">My Page</h2>"`

**Why it happens:** Cosmetic tests assert full class strings, which breaks when any class is added.
**How to avoid:** Update all eight cosmetic test assertions in the same commit as the class changes. The structural tests in `structural_tests` module already use `assert_element` + `has_class` and will pass without modification.
**Warning signs:** `cargo test -p ferro-json-ui` fails with messages like "assertion failed: html.contains(...)".

### Pitfall 3: Span element incorrectly gets leading-relaxed
**What goes wrong:** Developer adds `leading-relaxed` to all TextElement variants including Span, reasoning "body text gets leading-relaxed".
**Why it happens:** Span is listed with P/Div/Section in the same match arm visually, tempting bulk edits.
**How to avoid:** Span is inline — body text line-height applies to block containers, not inline spans. Leave `TextElement::Span` unchanged.
**Warning signs:** Span tests fail if class string changes; visually no difference since span line-height is inherited.

### Pitfall 4: layout.rs structural test may assert the old text-text class
**What goes wrong:** The `sidebar_renders_sections` or equivalent test in `layout.rs` asserts the group label uses `text-text` — updating line 172 breaks it.
**Why it happens:** The layout.rs tests at lines 930 and 1220 check for `text-text-muted hover:text-text` and `text-text` respectively — check whether either covers the group label `<p>`.
**How to avoid:** After changing line 172, run `cargo test -p ferro-json-ui` and update any failing layout test that asserts the old `text-text` class on the group label paragraph.
**Warning signs:** Layout test failures after line 172 change.

## Code Examples

Verified patterns from codebase inspection:

### TYP-01 + TYP-02: H1 and H2 in render_text
```rust
// Source: ferro-json-ui/src/render.rs, fn render_text, line 1054
TextElement::H1 => format!(
    "<h1 class=\"text-3xl font-bold leading-tight tracking-tight text-text\">{content}</h1>"
),
TextElement::H2 => format!(
    "<h2 class=\"text-2xl font-semibold leading-tight tracking-tight text-text\">{content}</h2>"
),
```

### TYP-03: H3 in render_text
```rust
// Source: ferro-json-ui/src/render.rs, fn render_text, line 1062
TextElement::H3 => format!(
    "<h3 class=\"text-xl font-semibold leading-snug text-text\">{content}</h3>"
),
```

### TYP-04: P, Div, Section in render_text
```rust
// Source: ferro-json-ui/src/render.rs, fn render_text, line 1057
TextElement::P       => format!("<p class=\"text-base leading-relaxed text-text\">{content}</p>"),
TextElement::Div     => format!("<div class=\"text-base leading-relaxed text-text\">{content}</div>"),
TextElement::Section => format!("<section class=\"text-base leading-relaxed text-text\">{content}</section>"),
```

### TYP-02: Inline h2 in render_page_header
```rust
// Source: ferro-json-ui/src/render.rs, fn render_page_header, line 350
html.push_str(&format!(
    "<h2 class=\"text-2xl font-semibold leading-tight tracking-tight text-text\">{}</h2>",
    html_escape(&props.title)
));
```

### TYP-03: Inline h3 in render_card, render_modal, render_checklist
```rust
// render_card (line 384):
html.push_str(&format!(
    "<h3 class=\"text-lg font-semibold leading-snug text-text\">{}</h3>",
    html_escape(&props.title)
));

// render_modal (line 424):
html.push_str(&format!(
    "<h3 class=\"text-lg font-semibold leading-snug text-text\">{}</h3>",
    html_escape(&props.title)
));

// render_checklist (line 1501):
html.push_str(&format!(
    "<h3 class=\"text-sm font-semibold leading-snug text-text\">{}</h3>",
    html_escape(&props.title)
));
```

### TYP-05: Sidebar group label in layout.rs
```rust
// Source: ferro-json-ui/src/layout.rs, fn layout_sidebar_group, line 172
html.push_str(&format!(
    "<p class=\"px-2 py-1 text-xs font-semibold text-text-muted\">{}</p>",
    html_escape(&group.label)
));
```

### Updated Cosmetic Test Assertions
```rust
// text_p_variant (line 1882):
assert!(html.contains("<p class=\"text-base leading-relaxed text-text\">Paragraph</p>"));

// render_view_with_component_wraps_in_div (line 1873):
assert!(html.contains("<p class=\"text-base leading-relaxed text-text\">Hello</p>"));

// text_h1_variant (line 1889):
assert!(html.contains("<h1 class=\"text-3xl font-bold leading-tight tracking-tight text-text\">Title</h1>"));

// text_h2_variant (line 1896):
assert!(html.contains("<h2 class=\"text-2xl font-semibold leading-tight tracking-tight text-text\">Subtitle</h2>"));

// text_h3_variant (line 1903):
assert!(html.contains("<h3 class=\"text-xl font-semibold leading-snug text-text\">Section</h3>"));

// card_renders_title_and_description (line 2872):
assert!(html.contains("<h3 class=\"text-lg font-semibold leading-snug text-text\">My Card</h3>"));

// modal_renders_details_summary (line 2944):
assert!(html.contains("<h3 class=\"text-lg font-semibold leading-snug text-text\">Confirm</h3>"));

// test_render_page_header_title_only (line 4925):
assert!(html.contains("<h2 class=\"text-2xl font-semibold leading-tight tracking-tight text-text\">My Page</h2>"));
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| H1/H2 render with browser default line-height (~1.5) | H1/H2 render with `leading-tight` (1.25) and `tracking-tight` (-0.025em) | Phase 104 | Headings have tighter optical rhythm matching Inter Variable font design intent |
| H3 render with browser default line-height | H3 renders with `leading-snug` (1.375) | Phase 104 | Subheadings transition naturally between tight headings and relaxed body |
| Body text (P/Div/Section) renders with browser default line-height (~1.5) | Body text renders with `leading-relaxed` (1.625) | Phase 104 | Long-form text is more comfortable to read — Inter Variable at 16px benefits from 1.6 line-height |
| Sidebar group label uses `text-text` in layout.rs | Sidebar group label uses `text-text-muted` consistently | Phase 104 | Both code paths (standalone Sidebar component and DashboardLayout sidebar) visually match |

**Retained patterns (no change):**
- `TextElement::Span` remains `text-base text-text` with no leading class
- StatCard value paragraph (`text-2xl font-bold text-text`) remains unchanged
- Form labels, table cells, badge text do not receive leading classes

## Open Questions

1. **text_span_variant cosmetic test (line 1910)**
   - What we know: `TextElement::Span` is not changing — no leading class added.
   - What's unclear: The test at line 1910 asserts `"<span class=\"text-base text-text\">Inline</span>"`. Since Span is not changing, this test passes without update.
   - Recommendation: No action needed. Document in plan to skip Span test updates.

2. **render_checklist h3 size is text-sm (not text-xl)**
   - What we know: Checklist title uses `text-sm` to fit the compact card widget. Requirements say H3 should have `leading-snug` — this applies to any h3 element regardless of font size.
   - What's unclear: Whether a checklist title at `text-sm` visually benefits from `leading-snug` (1.375 vs browser default ~1.5).
   - Recommendation: Apply `leading-snug` consistently — the requirement is about semantic heading level, not visual size. It does no harm at small sizes.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (cargo test) |
| Config file | none (workspace-level) |
| Quick run command | `cargo test -p ferro-json-ui` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TYP-01 | H1 renders with `leading-tight tracking-tight` | unit | `cargo test -p ferro-json-ui text_h1_variant` | existing — update assertion string |
| TYP-01 | H1 structural test passes with new classes | unit | `cargo test -p ferro-json-ui h1_structural_element_and_semantic_class` | existing — passes without change |
| TYP-02 | H2 (TextElement) renders with `leading-tight tracking-tight` | unit | `cargo test -p ferro-json-ui text_h2_variant` | existing — update assertion string |
| TYP-02 | H2 (PageHeader) renders with `leading-tight tracking-tight` | unit | `cargo test -p ferro-json-ui test_render_page_header_title_only` | existing — update assertion string |
| TYP-02 | H2 structural test passes | unit | `cargo test -p ferro-json-ui h2_structural_element_and_semantic_class` | existing — passes without change |
| TYP-03 | H3 (TextElement) renders with `leading-snug` | unit | `cargo test -p ferro-json-ui text_h3_variant` | existing — update assertion string |
| TYP-03 | H3 (Card) renders with `leading-snug` | unit | `cargo test -p ferro-json-ui card_renders_title_and_description` | existing — update assertion string |
| TYP-03 | H3 (Modal) renders with `leading-snug` | unit | `cargo test -p ferro-json-ui modal_renders_details_summary` | existing — update assertion string |
| TYP-03 | H3 structural test passes | unit | `cargo test -p ferro-json-ui h3_structural_element_and_semantic_class` | existing — passes without change |
| TYP-04 | P renders with `leading-relaxed` | unit | `cargo test -p ferro-json-ui text_p_variant` | existing — update assertion string |
| TYP-04 | P structural test passes | unit | `cargo test -p ferro-json-ui p_structural_element_and_semantic_class` | existing — passes without change |
| TYP-05 | Sidebar group label uses `text-text-muted` in layout.rs | unit | `cargo test -p ferro-json-ui` (covers layout tests) | existing — check/update layout test at line 930 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-json-ui`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
None — existing test infrastructure covers all phase requirements. No new test files needed. Cosmetic test assertions need updating in the same commit as class changes, but the test functions themselves already exist.

## Sources

### Primary (HIGH confidence)
- Direct codebase inspection: `ferro-json-ui/src/render.rs` — all render functions, line numbers verified
  - `render_text` at line 1054: all TextElement variants and their current class strings
  - `render_page_header` at line 324: h2 inline heading at line 350
  - `render_card` at line 379: h3 inline heading at line 384
  - `render_modal` at line 414: h3 inline heading at line 424
  - `render_checklist` at line 1496: h3 inline heading at line 1501
  - Cosmetic test assertions at lines 1873, 1882, 1889, 1896, 1903, 2872, 2944, 4925 — verified current strings
  - `structural_tests` module at line 5213 — Phase 104 annotations confirmed present
- Direct codebase inspection: `ferro-json-ui/src/layout.rs` — `layout_sidebar_group` at line 165, sidebar group label at line 172 uses `text-text` (inconsistency with render.rs line 1682)
- Direct codebase inspection: `ferro-theme/assets/default.css` — no typography scale tokens; confirms `leading-*` and `tracking-*` classes come from Tailwind's default scale

### Secondary (MEDIUM confidence)
- Tailwind CSS v4 documentation: `leading-tight` = 1.25, `leading-snug` = 1.375, `leading-relaxed` = 1.625, `tracking-tight` = -0.025em — standard scale values unchanged between v3 and v4

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; Tailwind v4 generates these utilities automatically
- Architecture (what to change): HIGH — all file locations and line numbers identified from direct code inspection
- Class string targets: HIGH — exact current strings verified, exact new strings specified
- Cosmetic test updates: HIGH — all eight affected test lines identified with line numbers
- TYP-05 inconsistency: HIGH — both occurrences found and verified

**Research date:** 2026-03-25
**Valid until:** 2026-06-25 (render.rs structure is stable; class strings will not change between phases)
