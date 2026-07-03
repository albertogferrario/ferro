# Phase 250: Token vocabulary v2 + default theme refresh - Pattern Map

**Mapped:** 2026-07-03
**Files analyzed:** 8 (7 source + 1 generated)
**Analogs found:** 8 / 8 (all files are modifications of existing files; no new files)

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-theme/src/token.rs` | utility (constants + vocab version) | transform | self (extend in-place) | exact |
| `ferro-json-ui/assets/input.css` | config (Tailwind pipeline input) | transform | self (extend in-place) | exact |
| `ferro-theme/assets/default.css` | config (runtime CSS) | transform | self (refresh in-place) | exact |
| `ferro-json-ui/assets/ferro-base.css` | generated (do not hand-edit) | transform | self (regenerated via script) | exact |
| `ferro-cli/src/commands/make_theme.rs` | utility (CLI scaffold + tests) | transform | self (extend in-place) | exact |
| `ferro-theme/src/loader.rs` | utility (doc comment only) | — | self (doc update only) | exact |
| `docs/src/features/themes.md` | documentation | — | self (update + extend) | exact |
| new test assertions (in-file) | test | — | existing `#[test]` blocks in same files | exact |

---

## Pattern Assignments

### `ferro-theme/src/token.rs` (utility, token constants)

**Analog:** self — extend in-place

**Existing module doc header** (lines 1–5) — update `v1` → `v2` and `~23` → `~30`:
```rust
//! Fixed semantic token vocabulary for ferro-theme/v1.
//!
//! Defines ~23 semantic slots that every theme must provide. Tokens are
//! CSS custom properties resolved by the Tailwind v4 `@theme` directive.
//! Components reference them as utility classes (`bg-primary`, `text-surface`, etc.).
```

**Existing constant pattern** (lines 8–61) — each new constant follows this exact form:
```rust
/// {one-line description}.
pub const TOKEN_{NAME}: &str = "--{css-var-name}";
```

For example, the typography group (lines 57–61) shows the grouping comment + constant pattern:
```rust
// Typography tokens — font family scale only; Tailwind size scale stays as-is
/// Sans-serif font stack.
pub const TOKEN_FONT_SANS: &str = "--font-sans";
/// Monospace font stack.
pub const TOKEN_FONT_MONO: &str = "--font-mono";
```

**New constants to add** — copy the grouping comment + constant pattern above, inserting after the typography block and before `ALL_TOKENS`:
```rust
// Density token — base spacing unit
/// Base spacing unit; all spacing utilities resolve as calc(var(--spacing) * N).
pub const TOKEN_SPACING: &str = "--spacing";

// Motion tokens — frequency-tiered transition discipline
/// Fast transition duration (micro-interactions: hover, toggles). Default: 120ms.
pub const TOKEN_MOTION_DURATION_FAST: &str = "--motion-duration-fast";
/// Base transition duration (dropdowns, modals, toasts). Default: 220ms.
pub const TOKEN_MOTION_DURATION_BASE: &str = "--motion-duration-base";
/// Slow transition duration (drawers, page-level reveals). Default: 320ms.
pub const TOKEN_MOTION_DURATION_SLOW: &str = "--motion-duration-slow";
/// Standard easing curve (calm, settled, no bounce).
pub const TOKEN_MOTION_EASE: &str = "--motion-ease";

// Focus token — uniform keyboard-navigation ring
/// Focus-visible ring color for interactive components.
pub const TOKEN_COLOR_RING: &str = "--color-ring";

// Display font token
/// Display/heading font family; defaults to var(--font-sans).
pub const TOKEN_FONT_DISPLAY: &str = "--font-display";
```

**Existing `ALL_TOKENS` pattern** (lines 63–88) — update comment and append 7 new constants:
```rust
/// All token names in the ferro-theme/v1 vocabulary (23 slots).
pub const ALL_TOKENS: &[&str] = &[
    TOKEN_BACKGROUND,
    TOKEN_SURFACE,
    // ... existing 23 entries ...
    TOKEN_FONT_MONO,
];
```
Change comment to `ferro-theme/v2 vocabulary (30 slots)` and add:
```rust
    TOKEN_SPACING,
    TOKEN_MOTION_DURATION_FAST,
    TOKEN_MOTION_DURATION_BASE,
    TOKEN_MOTION_DURATION_SLOW,
    TOKEN_MOTION_EASE,
    TOKEN_COLOR_RING,
    TOKEN_FONT_DISPLAY,
```

**New test to add** inside the existing `#[cfg(test)]` block (none currently exists in token.rs, so add one):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_tokens_len_is_30() {
        assert_eq!(ALL_TOKENS.len(), 30, "ALL_TOKENS must have exactly 30 slots");
    }
}
```

---

### `ferro-json-ui/assets/input.css` (config, Tailwind pipeline input)

**Analog:** self — extend in-place

**Existing `@theme inline` block structure** (lines 18–47) — all current entries are self-referential:
```css
@theme inline {
  /* Surface tokens */
  --color-background: var(--color-background);
  --color-surface: var(--color-surface);
  /* ... */

  /* Role tokens */
  --color-primary: var(--color-primary);
  /* ... 14 entries total ... */

  /* Shape tokens */
  --radius-sm: var(--radius-sm);
  /* ... */

  /* Shadow tokens */
  --shadow-sm: var(--shadow-sm);
  /* ... */
}
```

**New entries to append inside `@theme inline`** — motion entries use the namespace-bridge pattern (semantic `--motion-*` → Tailwind `--duration-*`/`--ease-*`) with `var(, fallback)` for v1 compatibility (D-05). Color and font entries use the self-referential and nested-fallback patterns respectively:
```css
  /* Motion tokens — bridge semantic names to Tailwind utility namespaces */
  --duration-fast: var(--motion-duration-fast, 120ms);
  --duration-base: var(--motion-duration-base, 220ms);
  --duration-slow: var(--motion-duration-slow, 320ms);
  --ease-base: var(--motion-ease, cubic-bezier(0.2, 0, 0.38, 0.9));

  /* Focus ring token */
  --color-ring: var(--color-ring);

  /* Display font token — nested fallback to font-sans */
  --font-display: var(--font-display, var(--font-sans));
```

Note: `--spacing` does NOT get a `@theme inline` entry. Tailwind v4 already generates spacing utilities as `calc(var(--spacing) * N)`; the token is runtime-overridable without an entry here.

**Existing `@source inline` pattern** (lines 53, 59) — each line safelists a space-separated list of class names as a quoted string:
```css
@source inline("font-sans font-mono");
@source inline("grid-cols-1 grid-cols-2 ... grid-cols-12 ...");
```

**New `@source inline` line to add** after the existing safelist lines:
```css
@source inline("duration-fast duration-base duration-slow ease-base font-display ring-ring");
```

**New `@media` block to add** at the bottom of the file — emitted verbatim into `ferro-base.css`:
```css
/* Collapse motion durations for users who prefer reduced motion.
 * Uses 0.01ms (not 0ms) so transitionend/animationend listeners still fire. */
@media (prefers-reduced-motion: reduce) {
  :root {
    --motion-duration-fast: 0.01ms;
    --motion-duration-base: 0.01ms;
    --motion-duration-slow: 0.01ms;
  }
}
```

---

### `ferro-theme/assets/default.css` (config, runtime CSS)

**Analog:** self — refresh in-place

**Existing file structure** (lines 1–77) — three blocks, all plain CSS (no Tailwind at-rules):

1. `:root { ... }` — light mode (lines 8–41)
2. `@media (prefers-color-scheme: dark) { :root { ... } }` — OS dark (lines 43–60)
3. `[data-theme="dark"] { ... }` — manual toggle (lines 62–77)

**Constraint:** The file header (lines 1–6) explicitly prohibits Tailwind at-rules:
```css
/* Ferro default theme — plain CSS variables.
 *
 * Injected verbatim into <style> by framework/src/json_ui/mod.rs.
 * MUST NOT contain Tailwind-specific at-rules — those are not standard
 * CSS and browsers ignore the entire block as an unknown at-rule.
 */
```

**Existing neutral ramp (light mode)** (lines 10–15) — current zero-chroma greys to be replaced with cool-tinted values (D-08):
```css
  --color-background: oklch(100% 0 0);
  --color-surface: oklch(97% 0 0);
  --color-card: oklch(95% 0 0);
  --color-border: oklch(90% 0 0);
  --color-text: oklch(15% 0 0);
  --color-text-muted: oklch(50% 0 0);
```

**Existing accent (both modes)** (line 21, 55, 74) — current separate-hue cyan (hue 200) to be harmonized toward primary hue ~250 (D-09):
```css
  --color-accent: oklch(65% 0.15 200);   /* light */
  --color-accent: oklch(60% 0.15 200);   /* dark @media */
  --color-accent: oklch(60% 0.15 200);   /* [data-theme="dark"] */
```

**Pattern for new token declarations** — append at the end of the token group for each section, following the existing comment style:
```css
  /* Density token */
  --spacing: 0.25rem;

  /* Motion tokens */
  --motion-duration-fast: 120ms;
  --motion-duration-base: 220ms;
  --motion-duration-slow: 320ms;
  --motion-ease: cubic-bezier(0.2, 0, 0.38, 0.9);

  /* Focus ring token */
  --color-ring: oklch(55% 0.2 250);   /* primary-family; exact value is Claude's discretion */

  /* Display font token */
  --font-display: var(--font-sans);
```

This block must appear in ALL THREE sections (`:root`, `@media` dark, `[data-theme="dark"]`). The dark-mode ring color is slightly lighter for contrast (e.g. `oklch(65% 0.18 250)`). Motion defaults are identical across all three (durations don't need to differ between light/dark).

---

### `ferro-json-ui/assets/ferro-base.css` (generated — do not hand-edit)

**Analog:** self — regenerated via script after `input.css` is updated

**Regeneration command:**
```bash
bash scripts/gen-ferro-base-css.sh
```

Run from the repo root. The script auto-downloads Tailwind v4.2.3 if not present. Commit the regenerated file together with `input.css`.

**Post-regen assertions** — verify with grep before committing:
```bash
grep 'duration-fast' ferro-json-ui/assets/ferro-base.css
grep 'var(--motion-duration-fast,' ferro-json-ui/assets/ferro-base.css
grep 'prefers-reduced-motion' ferro-json-ui/assets/ferro-base.css
grep 'ring-ring\|font-display' ferro-json-ui/assets/ferro-base.css
```

If any grep returns empty, the corresponding `@theme inline` entry or `@source inline` safelist is wrong.

---

### `ferro-cli/src/commands/make_theme.rs` (utility, CLI scaffold + tests)

**Analog:** self — extend in-place

**Existing `tokens_css_template()` function** (lines 74–136) — returns a `&'static str` using a raw string literal. The template mirrors `default.css` structure (`:root` + dark `@media`) but omits the `[data-theme="dark"]` block (scaffolded themes start with OS dark mode only):

```rust
fn tokens_css_template() -> &'static str {
    r#"/* Theme tokens — plain CSS variables.
 * ...
 */

:root {
  /* Surface tokens */
  --color-background: oklch(100% 0 0);
  /* ... 23 slots ... */
}

@media (prefers-color-scheme: dark) {
  :root {
    /* ... dark overrides ... */
  }
}
"#
}
```

**New slots to add to the template** — append in the `:root` block and dark `@media` block, following the grouping comment pattern:
```css
  /* Density token */
  --spacing: 0.25rem;

  /* Motion tokens */
  --motion-duration-fast: 120ms;
  --motion-duration-base: 220ms;
  --motion-duration-slow: 320ms;
  --motion-ease: cubic-bezier(0.2, 0, 0.38, 0.9);

  /* Focus ring token */
  --color-ring: oklch(55% 0.2 250);

  /* Display font token */
  --font-display: var(--font-sans);
```

**Function doc comment to update** (line 13): `"all 23 semantic token slots"` → `"all 30 semantic token slots"`.

**Existing test pattern** (lines 158–213) — test `test_make_theme_tokens_css_has_all_23_token_slots` uses grouped `assert!(css.contains("--token-name:"), "missing --token-name")` assertions:
```rust
#[test]
fn test_make_theme_tokens_css_has_all_23_token_slots() {
    let tmp = TempDir::new().unwrap();
    make_theme_in_dir("test", tmp.path()).unwrap();

    let css = read_file(&tmp.path().join("themes/test/tokens.css"));

    // Surface tokens (6)
    assert!(css.contains("--color-background:"), "missing --color-background");
    // ... one assert per token ...

    // Typography tokens (2)
    assert!(css.contains("--font-sans:"), "missing --font-sans");
    assert!(css.contains("--font-mono:"), "missing --font-mono");
}
```

**Test changes required:**
1. Rename to `test_make_theme_tokens_css_has_all_30_token_slots`
2. Add 7 new assertions at the end of the function body:
```rust
    // Density token (1)
    assert!(css.contains("--spacing:"), "missing --spacing");

    // Motion tokens (4)
    assert!(css.contains("--motion-duration-fast:"), "missing --motion-duration-fast");
    assert!(css.contains("--motion-duration-base:"), "missing --motion-duration-base");
    assert!(css.contains("--motion-duration-slow:"), "missing --motion-duration-slow");
    assert!(css.contains("--motion-ease:"), "missing --motion-ease");

    // Focus ring token (1)
    assert!(css.contains("--color-ring:"), "missing --color-ring");

    // Display font token (1)
    assert!(css.contains("--font-display:"), "missing --font-display");
```

---

### `ferro-theme/src/loader.rs` (utility, doc comment update)

**Analog:** self — doc-only update

**Existing doc comment to update** (lines 25–28):
```rust
    /// The CSS contains plain `:root { ... }` CSS variable declarations for all 23
    /// semantic token slots (light and dark modes). Safe to inject into a `<style>` tag
    /// without Tailwind processing.
```
Change `23` → `30`.

**Existing test to extend** (lines 72–76) — `default_theme_returns_non_empty_css_with_color_primary` currently only asserts `--color-primary` is present:
```rust
#[test]
fn default_theme_returns_non_empty_css_with_color_primary() {
    let theme = Theme::default_theme();
    assert!(!theme.css.is_empty());
    assert!(theme.css.contains("--color-primary"));
}
```

Extend to cover at least one new slot from each of the 3 new groups:
```rust
#[test]
fn default_theme_returns_all_30_token_slots() {
    let theme = Theme::default_theme();
    assert!(!theme.css.is_empty());
    // v1 tokens still present
    assert!(theme.css.contains("--color-primary"), "missing --color-primary");
    assert!(theme.css.contains("--font-sans"), "missing --font-sans");
    // v2 new tokens
    assert!(theme.css.contains("--spacing"), "missing --spacing");
    assert!(theme.css.contains("--motion-duration-fast"), "missing --motion-duration-fast");
    assert!(theme.css.contains("--motion-ease"), "missing --motion-ease");
    assert!(theme.css.contains("--color-ring"), "missing --color-ring");
    assert!(theme.css.contains("--font-display"), "missing --font-display");
}
```

Rename the existing test or add alongside it — prefer renaming to reflect the new count.

---

### `ferro-json-ui/src/assets/mod.rs` (test — new assertion)

**Analog:** existing `ferro_base_css_non_empty` test (lines 20–31)

**Existing test pattern:**
```rust
#[test]
#[allow(clippy::const_is_empty)]
fn ferro_base_css_non_empty() {
    assert!(!FERRO_BASE_CSS.is_empty(), "embedded CSS must not be empty");
    assert!(
        FERRO_BASE_CSS.contains("flex"),
        "expected `flex` utility in generated CSS"
    );
}
```

**New test to add** — asserts the regenerated CSS contains the motion fallback (structural v1-invariant proof, DS-01/SC-3):
```rust
#[test]
fn ferro_base_css_contains_motion_duration_fallback() {
    // Verifies the @theme inline bridge entry was regenerated correctly.
    // The `var(--motion-duration-fast,` pattern confirms the fallback fires
    // for v1 themes that do not define --motion-duration-fast.
    assert!(
        FERRO_BASE_CSS.contains("var(--motion-duration-fast,"),
        "expected motion-duration-fast fallback in generated CSS; run scripts/gen-ferro-base-css.sh"
    );
    assert!(
        FERRO_BASE_CSS.contains("prefers-reduced-motion"),
        "expected prefers-reduced-motion block in generated CSS"
    );
}
```

---

### `docs/src/features/themes.md` (documentation)

**Analog:** self — update + extend

**Existing count references to update** (lines 8, 29, 57) — three occurrences of `23`:
- Line 8: `"defining the 23 semantic token slots"` → `"defining the 30 semantic token slots"`
- Line 29: `"all 23 token slots pre-filled"` → `"all 30 token slots pre-filled"`
- Line 57: `"All 23 semantic token slots."` → `"All 30 semantic token slots."`

**Existing token table structure** (lines 60–102) — markdown table with Token | Default (light) | Purpose columns. New sections to add after the Typography table:

```markdown
### Density Token (1)

| Token | Default | Purpose |
|-------|---------|---------|
| `--spacing` | `0.25rem` | Base spacing unit; all spacing utilities scale as `calc(var(--spacing) * N)` |

### Motion Tokens (4)

| Token | Default | Purpose |
|-------|---------|---------|
| `--motion-duration-fast` | `120ms` | Micro-interactions (hover, toggles) |
| `--motion-duration-base` | `220ms` | Panel transitions (dropdowns, modals) |
| `--motion-duration-slow` | `320ms` | Page-level reveals (drawers, sheets) |
| `--motion-ease` | `cubic-bezier(0.2, 0, 0.38, 0.9)` | Standard easing curve — calm, no bounce |

### Focus Ring Token (1)

| Token | Default (light) | Purpose |
|-------|----------------|---------|
| `--color-ring` | primary-family oklch | Uniform focus-visible ring for interactive components |

### Display Font Token (1)

| Token | Default | Purpose |
|-------|---------|---------|
| `--font-display` | `var(--font-sans)` | Display/heading font family |
```

**Type-scaling recipe to add** — new section (per D-06 / docs requirement):

```markdown
## Type Scaling

The token vocabulary does not include per-size type tokens. Use the `font-size`
property on `:root` in a theme's `tokens.css` to shift the entire Tailwind
size scale relative to the browser default:

```css
:root {
  font-size: 14px;   /* compact — all rem-based sizes scale down */
}
```

Tailwind spacing utilities already respond to `--spacing` (the density token).
Font size is a separate axis — root `font-size` controls the `rem` anchor.
```

---

## Shared Patterns

### Plain CSS constraint (applies to `default.css` and `tokens_css_template()`)

**Source:** `ferro-theme/assets/default.css` lines 1–6 and `ferro-cli/src/commands/make_theme.rs` line 78–80

Both files include this invariant: no Tailwind at-rules (`@import "tailwindcss"`, `@theme {`) allowed. The guard is enforced by `test_make_theme_tokens_css_has_root_block_and_no_tailwind_syntax`. Every new token declaration must use plain `:root { --name: value; }` CSS, not `@theme`.

### Token group comment style (applies to `token.rs`, `default.css`, `tokens_css_template()`)

**Source:** `ferro-theme/src/token.rs` lines 7–61

Groups are prefixed with a `//` or `/* */` comment naming the group (e.g. `// Surface tokens — structural background hierarchy`). New groups follow the same convention. Groups in `default.css` use CSS block comments (`/* Surface tokens */`).

### `var(, fallback)` in `@theme inline` (applies to `input.css`)

**Source:** RESEARCH.md Architecture Patterns section

New token entries that v1 themes may omit must use `var(--token-name, <fallback>)` rather than `var(--token-name)`. This is the D-05 structural guarantee. The self-referential form `var(--name)` is correct only for tokens that existed in v1 (they are always defined by a valid v1 theme). The 5 motion/font entries added in this phase use the fallback form.

### Token-count drift guard update sequence

**Source:** RESEARCH.md "Token-count drift guards" table

When `ALL_TOKENS` changes, update ALL of these in the same commit:
1. `ferro-theme/src/token.rs` — `ALL_TOKENS` slice + doc header (`v1` → `v2`, `23` → `30`)
2. `ferro-theme/src/loader.rs` — `default_theme()` doc comment (`23` → `30`)
3. `ferro-cli/src/commands/make_theme.rs` — function doc + test name + 7 new assertions
4. `docs/src/features/themes.md` — three occurrences of `23` + new token tables

The `BUILTIN_TYPES` count (47) in `ferro-json-ui/src/catalog.rs` is for components — it does NOT change in this phase.

---

## No Analog Found

None. All files in scope are modifications to existing files; no new files are created in this phase.

---

## Metadata

**Analog search scope:** `ferro-theme/`, `ferro-json-ui/`, `ferro-cli/`, `docs/src/features/`
**Files read:** 7 source files + catalog test excerpt
**Pattern extraction date:** 2026-07-03
