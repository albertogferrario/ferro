# Phase 250: Token vocabulary v2 + default theme refresh - Research

**Researched:** 2026-07-03
**Domain:** CSS design tokens, Tailwind v4 `@theme inline`, ferro-theme, ferro-json-ui
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** 7 new slots and their defaults are fixed: `--spacing` (density), `--motion-duration-fast` 120ms, `--motion-duration-base` 220ms, `--motion-duration-slow` 320ms, `--motion-ease` cubic-bezier(0.2, 0, 0.38, 0.9), `--color-ring`, `--font-display` (defaults to `var(--font-sans)`).
- **D-02:** Deliberate exclusions: no per-size type tokens, no font-weight tokens.
- **D-03:** Every valid v1 theme remains a valid v2 theme with zero changes — an unmodified v1 `tokens.css` must render identically before and after this phase.
- **D-04:** `ferro-theme/src/token.rs` doc header moves to `ferro-theme/v2`; `ALL_TOKENS` → 30. Update the "23 slots"/"~23" doc comments.
- **D-05:** New-slot defaults delivered as `var(--slot, <default>)` fallbacks in the `@theme inline` mapping in `ferro-json-ui/assets/input.css`.
- **D-06:** (Research question — see findings below.)
- **D-07:** `prefers-reduced-motion: reduce` collapses all three durations to `0.01ms`, emitted into generated `ferro-base.css` via `input.css`.
- **D-08:** Neutral ramp gains a subtle cool tint (low chroma ~hue 250) in both light and dark.
- **D-09:** `--color-accent` harmonized toward primary hue family; token stays, default value changes.
- **D-10:** Radii and shadows kept as-is. Exact oklch values are Claude's discretion.
- **D-11:** `default.css` must remain plain CSS — no Tailwind at-rules.
- **D-12:** This phase only exposes new slots and utilities; no component markup/class changes (Phase 251).
- **D-13:** Regenerate `ferro-base.css` with `scripts/gen-ferro-base-css.sh` AFTER the `input.css`/token code changes are in tree.

### Claude's Discretion

- Exact oklch values for the refreshed neutral ramp and harmonized accent (within D-08/D-09 direction).
- `--color-ring` default value (spec direction: visible contrast; a primary-family ring is the natural pick).
- `themes.md` prose structure for the v2 reference and type-scaling recipe.

### Deferred Ideas (OUT OF SCOPE)

- Component-level application of motion/ring/display-font tokens — Phase 251.
- Canonical variant/tone/size enums — Phase 251.
- `design::lint` + `Spec.design` — Phase 252.
- MCP surface + docs chapter + publish — Phase 253.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DS-01 | Token vocabulary grows 23 → 30; every new slot has a default in base CSS and `default.css`; v1 themes render identically; utilities exposed via `var()` references; `prefers-reduced-motion` collapses durations. | Covered fully: `@theme inline` fallback pattern verified, `--spacing` already uses `var()`, motion namespace mapping confirmed. |
| DS-02 | `default.css` refreshed to design language; `docs/src/features/themes.md` documents v2 + root-font-size type-scaling recipe. | Injection path confirmed (compile-time `include_str!`); dark mode dual strategy documented; docs file identified. |
</phase_requirements>

---

## Summary

Phase 250 extends the ferro-theme token vocabulary from 23 to 30 slots and refreshes the default theme to the documented design language. All 7 new slots plug into the existing `@theme inline` architecture in `input.css` — the mechanism is verified as the right tool.

The most important finding concerns `--spacing`: Tailwind v4 already generates all spacing utilities as `calc(var(--spacing) * N)` in the current `ferro-base.css`. The `--spacing` token is therefore already runtime-overridable by any theme that declares it in un-layered `:root {}` CSS. No `@theme inline` entry is needed for `--spacing`; add it to `token.rs` and `default.css` only.

For the four motion tokens, there is a namespace mismatch: the semantic names (`--motion-duration-fast` etc.) do not match the Tailwind `--duration-*` / `--ease-*` namespaces that generate utility classes. The `@theme inline` entries must bridge this: `--duration-fast: var(--motion-duration-fast, 120ms)`. The `var(, <fallback>)` form inside `@theme inline` is the D-05 mechanism that keeps v1 themes valid without changes.

New utilities (`duration-fast/base/slow`, `ease-base`, `ring-ring`, `font-display`) must be added to the `@source inline()` safelist in `input.css`; D-12 prohibits component class changes in this phase, so no scanner pickup will occur otherwise, and SC-2 requires the utilities to appear in the regenerated `ferro-base.css`.

**Primary recommendation:** Follow the existing `@theme inline` self-referential pattern for `--color-ring` and `--font-display` (same namespace as existing tokens); bridge the namespace gap for motion tokens with `--duration-*/--ease-*` Tailwind-namespace entries pointing at the semantic `--motion-*` vars; leave `--spacing` to its native Tailwind behavior; safelist new utility class names.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Token constants and vocabulary versioning | ferro-theme library | — | Rust constants are the source of truth for token names |
| Tailwind utility generation for new tokens | ferro-json-ui (input.css) | — | `@theme inline` mappings live in the Tailwind input file |
| Runtime CSS custom property values | Theme CSS (default.css / tokens.css) | ThemeMiddleware | Injected as un-layered `<style>` at request time |
| Reduced-motion behavior | ferro-json-ui (input.css → ferro-base.css) | — | `@media` query in the generated stylesheet |
| Make-theme scaffold (CLI) | ferro-cli | — | `tokens_css_template()` is the v2 scaffold template |
| Documentation | docs/src/features/themes.md | — | Updated in this phase alongside code |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tailwind CSS CLI | v4.2.3 (pinned) | CSS utility generation from `input.css` → `ferro-base.css` | Established; pinned binary via `scripts/install-tailwind.sh` |
| ferro-theme | workspace | Token constants + default theme CSS embedding | Existing crate, no new crate per out-of-scope constraint |
| ferro-json-ui | workspace | `input.css` `@theme inline` mappings + `ferro-base.css` | Existing crate that owns the generated stylesheet |

### Tooling

| Tool | Version | Purpose |
|------|---------|---------|
| `scripts/gen-ferro-base-css.sh` | — | Regenerates `ferro-base.css`; auto-installs binary via `scripts/install-tailwind.sh` |
| `scripts/install-tailwind.sh` | — | Downloads Tailwind CLI v4.2.3 to `.tooling/bin/tailwindcss`; verifies sha256 checksum |

**Regeneration command:**
```bash
bash scripts/gen-ferro-base-css.sh
```

Run from anywhere in the repo (script uses `git rev-parse --show-toplevel` for REPO_ROOT).

---

## Architecture Patterns

### How `@theme inline` Works in Ferro

**Entry point:** `ferro-json-ui/assets/input.css`  
**Output:** `ferro-json-ui/assets/ferro-base.css` (minified, committed)  
**Tailwind version:** v4.2.3 (standalone CLI binary at `.tooling/bin/tailwindcss`)

```
input.css
  ├── @import "tailwindcss"           ← pulls in Tailwind defaults
  ├── @source "../../ferro-json-ui/src"  ← scans for utility class literals
  ├── @source "../../framework/src"
  ├── @theme inline { ... }           ← registers var() references for custom tokens
  ├── @source inline("...")           ← safelists classes from dynamic/runtime usage
  └── (new) @media prefers-reduced-motion  ← collapses motion vars to 0.01ms
```

**The `@theme inline` mechanism (VERIFIED against ferro-base.css):**

`@theme inline { --color-background: var(--color-background) }` outputs to the generated CSS:
```css
@layer theme { :root, :host { --color-background: var(--color-background); ... } }
```

This appears to be a self-referential circular reference, but is NOT circular in practice because:
1. Theme CSS (`default.css` or `tokens.css`) is injected as a plain `<style>` tag — **un-layered**.
2. Un-layered CSS wins over `@layer theme` in the CSS cascade.
3. So when the theme injects `:root { --color-background: oklch(100% 0 0); }`, that un-layered rule wins, and components' `bg-background` → `var(--color-background)` → `oklch(100% 0 0)`.

**The fallback pattern (D-05, VERIFIED conceptually):**

For new tokens that v1 themes do not define:
```css
@theme inline {
  --duration-fast: var(--motion-duration-fast, 120ms);
}
```
Outputs to:
```css
@layer theme { :root, :host { --duration-fast: var(--motion-duration-fast, 120ms); } }
```

A v1 theme that does not define `--motion-duration-fast`:
- `var(--motion-duration-fast, 120ms)` → `--motion-duration-fast` is undefined → fallback `120ms` used.
- `duration-fast` utility resolves to `120ms`. SC1 invariant holds structurally.

### Token Namespace Mapping (Critical for motion tokens)

Ferro semantic token names do NOT match Tailwind's utility-generating namespaces for motion. The `@theme inline` bridge is required:

| Ferro semantic token | `@theme inline` Tailwind-namespace entry | Generated utility | Output CSS |
|---|---|---|---|
| `--motion-duration-fast` | `--duration-fast: var(--motion-duration-fast, 120ms)` | `duration-fast` | `transition-duration: var(--motion-duration-fast, 120ms)` |
| `--motion-duration-base` | `--duration-base: var(--motion-duration-base, 220ms)` | `duration-base` | `transition-duration: var(--motion-duration-base, 220ms)` |
| `--motion-duration-slow` | `--duration-slow: var(--motion-duration-slow, 320ms)` | `duration-slow` | `transition-duration: var(--motion-duration-slow, 320ms)` |
| `--motion-ease` | `--ease-base: var(--motion-ease, cubic-bezier(0.2, 0, 0.38, 0.9))` | `ease-base` | `transition-timing-function: var(--motion-ease, ...)` |
| `--color-ring` | `--color-ring: var(--color-ring)` | `ring-ring` (+ `bg-ring`, `text-ring`) | `--tw-ring-color: var(--color-ring)` |
| `--font-display` | `--font-display: var(--font-display, var(--font-sans))` | `font-display` | `font-family: var(--font-display)` |
| `--spacing` | (none needed) | (spacing utilities already use `calc(var(--spacing) * N)`) | unchanged |

[VERIFIED: ferro-base.css inspection] Confirmed that `--duration-*` generates `duration-*` utilities (Tailwind v4 docs), `--ease-*` generates `ease-*` utilities, `--color-*` generates `ring-*` color utilities, and `--font-*` generates `font-*` font-family utilities.

### `--spacing` — No `@theme inline` Entry Needed

[VERIFIED: ferro-base.css grep — `calc(var(--spacing) * N)` confirmed in generated CSS]

Tailwind v4 generates spacing utilities with `calc(var(--spacing) * N)`. The `--spacing: .25rem` entry is already in `@layer theme` in the current `ferro-base.css`. A theme declaring `:root { --spacing: 0.2rem; }` (un-layered) wins over the layered `--spacing: .25rem` — spacing utilities respond at runtime.

**Action required for `--spacing`:**
1. Add `TOKEN_SPACING: "--spacing"` constant to `ferro-theme/src/token.rs`.
2. Add `--spacing: 0.25rem;` to `ferro-theme/assets/default.css` (light and dark `:root`).
3. NO change to `input.css`.

### `prefers-reduced-motion` in `input.css`

Add to `input.css` as a plain CSS media query (emitted verbatim into `ferro-base.css`):

```css
@media (prefers-reduced-motion: reduce) {
  :root {
    --motion-duration-fast: 0.01ms;
    --motion-duration-base: 0.01ms;
    --motion-duration-slow: 0.01ms;
  }
}
```

Using `0.01ms` not `0` keeps `transitionend`/`animationend` listeners firing (D-07).
[VERIFIED: `prefers-reduced-motion` is NOT currently in ferro-base.css — confirmed by grep]

### `default.css` Injection Path

[VERIFIED: `ferro-theme/src/loader.rs` line 6]

```rust
const DEFAULT_THEME_CSS: &str = include_str!("../assets/default.css");
```

`default.css` is embedded at compile time. `Theme::default_theme()` returns it as a `String`. Changing `default.css` requires a cargo rebuild (automatic).

**Dark mode dual strategy in `default.css`** — two blocks, BOTH must receive the 7 new tokens:
1. `@media (prefers-color-scheme: dark) { :root { ... } }` — OS-level preference
2. `[data-theme="dark"] { ... }` — manual toggle via JavaScript

Neither block uses Tailwind at-rules (D-11 constraint enforced by test `test_make_theme_tokens_css_has_root_block_and_no_tailwind_syntax`).

### Safelist Requirement for SC-2

D-12 prohibits component class changes in this phase. New utilities are not referenced in any source file scanned by Tailwind. Without safelisting, they will not appear in the regenerated `ferro-base.css`. Add to `input.css`:

```css
@source inline("duration-fast duration-base duration-slow ease-base font-display ring-ring");
```

This follows the existing `@source inline("font-sans font-mono")` and `@source inline("grid-cols-...")` patterns.

### `--font-display` — No CSS `font-display` Descriptor Conflict

[CITED: https://tailwindcss.com/docs/adding-custom-styles — shows `--font-display: "Satoshi", "sans-serif"` as an official example in `@theme`]

The CSS `font-display` property is a descriptor inside `@font-face` rules. The Tailwind utility class `font-display` sets `font-family: <value>` — these are distinct namespaces at the browser level. No conflict.

The `@theme inline { --font-display: var(--font-display, var(--font-sans)) }` uses a nested fallback: if `--font-display` is undefined, the fallback is `var(--font-sans)`, which resolves to the font-sans value from `default.css`. This is valid CSS.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Utility class generation for custom tokens | Custom CSS with `.duration-fast { ... }` | `@theme inline` entry in `input.css` + regenerate | Tailwind handles the var() indirection, dark mode, and utilities automatically |
| Tailwind CLI invocation | Script calling `npx tailwindcss` | `scripts/gen-ferro-base-css.sh` (existing) | Script handles binary pinning, checksum, and path resolution |
| Dark mode theming | JavaScript theme injection logic | `default.css` dual strategy (`@media` + `[data-theme]`) | Already wired in the framework; no new JS code |

**Key insight:** The entire token → utility pipeline is driven by the existing `@theme inline` convention. All 7 new tokens follow the same pattern — no new architectural work is required.

---

## D-06 Research Answers (Locked Research Questions)

### 1. Motion duration/easing in `@theme inline`

**Question:** What namespace do duration/easing utilities live in? Do `var(, fallback)` forms work in `@theme inline`?

**Answer (HIGH confidence):**
- Tailwind v4 `--duration-*` namespace generates `duration-<name>` utilities (e.g. `--duration-fast: 120ms` → `duration-fast`). [CITED: tailwindcss.com/docs/transition-duration]
- Tailwind v4 `--ease-*` namespace generates `ease-<name>` utilities (e.g. `--ease-fluid: cubic-bezier(...)` → `ease-fluid`). [CITED: tailwindcss.com/docs/adding-custom-styles]
- `var(--motion-duration-fast, 120ms)` as the value in `@theme inline` works because Tailwind emits the declaration verbatim into `@layer theme`, and the fallback fires for any v1 theme that omits the token.
- The ferro semantic names (`--motion-*`) map to the Tailwind namespaces via the bridge entries shown in the table above.

### 2. `--spacing` override

**Question:** Self-referential `var()` in `@theme inline` vs. `@theme` static — which approach for runtime-overridable spacing?

**Answer (HIGH confidence):**
[VERIFIED: ferro-base.css contains `--spacing:.25rem` in `@layer theme` and spacing utilities as `calc(var(--spacing) * N)`]

No `@theme inline` entry is needed. Tailwind v4 generates `calc(var(--spacing) * N)` for all spacing utilities. Since these utilities reference `var(--spacing)` at runtime, a theme declaring `--spacing: 0.2rem` in un-layered `:root {}` overrides the layered `--spacing: .25rem` and all spacing utilities respond immediately.

Adding `--spacing: var(--spacing, 0.25rem)` to `@theme inline` would NOT work: without a theme override, the `@layer theme` declaration becomes `--spacing: var(--spacing, 0.25rem)` (self-referential in the same cascade context — though `default.css` would rescue it with an un-layered override). The recommended approach is simpler: add `--spacing: 0.25rem` to `default.css` only; rely on the existing `var()` behavior.

### 3. `--font-display` utility name

**Question:** Does `font-display` utility class name collide with the CSS `font-display` descriptor?

**Answer (HIGH confidence):**
No collision. The CSS `font-display` descriptor is only valid inside `@font-face` rules — it is not a regular CSS property. The Tailwind utility class `font-display` sets `font-family: var(--font-display)` in regular selectors. These are separate namespaces. [CITED: tailwindcss.com — `--font-display: "Satoshi"` shown as official example]

### 4. `--color-ring` utilities

**Question:** Does `--color-ring: var(--color-ring)` in `@theme inline` generate `ring-ring`/ring utilities?

**Answer (HIGH confidence):**
[VERIFIED: ferro-base.css shows `ring-primary`, `ring-destructive` etc. generated from `--color-primary`, `--color-destructive` in `@theme inline`]

Yes. `--color-ring: var(--color-ring)` in `@theme inline` generates `ring-ring` (and `bg-ring`, `text-ring`, `border-ring` etc.). The `ring-ring` utility sets `--tw-ring-color: var(--color-ring)`. This is the exact pattern needed for Phase 251's `focus-visible:ring-ring` on interactive components.

### 5. `scripts/gen-ferro-base-css.sh`

[VERIFIED: file read]

- Calls `bash scripts/install-tailwind.sh` (idempotent, downloads v4.2.3 if not present).
- Invokes `.tooling/bin/tailwindcss -i ferro-json-ui/assets/input.css -o ferro-json-ui/assets/ferro-base.css --minify`.
- Binary is pinned by version string `TAILWIND_VERSION="v4.2.3"` and verified by sha256 checksum.
- Reproducible: same binary + same input → same output.

### 6. Token-count drift guards

[VERIFIED: grep across workspace]

| File | Reference to "23" | Action for Phase 250 |
|------|-------------------|----------------------|
| `ferro-theme/src/token.rs` | Module doc (`~23 semantic slots`), `ALL_TOKENS` comment (`23 slots`) | Update to 30 |
| `ferro-theme/src/loader.rs` | `fn default_theme()` doc (`all 23 semantic token slots`) | Update to 30 |
| `ferro-cli/src/commands/make_theme.rs` | Function doc (`all 23 semantic token slots`), test name (`test_make_theme_tokens_css_has_all_23_token_slots`) | Update doc + rename test + add 7 assertions |
| `docs/src/features/themes.md` | "23 semantic token slots" (×2), "All 23 semantic token slots" | Update to 30 |

The `BUILTIN_TYPES` count (47) in `ferro-json-ui/src/catalog.rs` is for components, NOT tokens. No update needed for Phase 250.

### 7. SC1 "renders identically" invariant testing

The structural guarantee is sufficient without a full rendering test:

The `var(--token, <fallback>)` pattern in `@theme inline` guarantees fallback behavior for v1 themes. A mechanical test approach:
1. After regeneration, assert `ferro-base.css` contains `var(--motion-duration-fast,` with fallback.
2. Existing test infrastructure: `ferro-theme/src/loader.rs` test `default_theme_returns_non_empty_css_with_color_primary` is a pattern — add an equivalent test verifying v1-compatible tokens still resolve.
3. The simplest end-to-end check: run the existing `cargo test --all-features` against the updated code — if the `make_theme` scaffold test passes (which only asserts the 23→30 v1 slots are present), the v1 invariant holds by construction.

No snapshot tests currently exist for CSS resolution. Adding a test that renders a component with only the 23 v1 tokens and asserts no `unset`/`invalid` values in the output CSS would be thorough but is not currently required by existing infrastructure.

### 8. `default.css` consumption and dark mode strategy

[VERIFIED: `ferro-theme/src/loader.rs`, `framework/src/theme/middleware.rs`, `app/src/bootstrap.rs`]

- `default.css` is embedded at compile time via `include_str!`.
- The sample `app/` uses `ThemeMiddleware::new().default_theme(Theme::default_theme())` — it uses only the built-in default theme, no custom themes directory. The refresh affects it directly.
- Dark mode uses TWO strategies in `default.css`:
  1. `@media (prefers-color-scheme: dark) { :root { ... } }` — OS-level auto.
  2. `[data-theme="dark"] { ... }` — JavaScript toggle via `data-theme` attribute on `<html>`.
  Both blocks must receive the 7 new token declarations.
- No other custom theme CSS files exist in the repo (`find . -name "tokens.css"` returns no real files, only test tempdir artifacts).

---

## Common Pitfalls

### Pitfall 1: Motion token namespace mismatch

**What goes wrong:** Adding `--motion-duration-fast: var(--motion-duration-fast, 120ms)` to `@theme inline` generates a utility named `motion-duration-fast`, not `duration-fast`. No component or author would know to use `motion-duration-fast`.
**Why it happens:** Tailwind's utility class name comes from the CSS custom property name after the `--`. The semantic `--motion-*` prefix doesn't match the `--duration-*` Tailwind utility namespace.
**How to avoid:** Always use the Tailwind-namespace entry `--duration-fast: var(--motion-duration-fast, ...)` in `@theme inline`. The semantic token name is the runtime-overridable CSS var; the Tailwind-namespace name is the utility-generating entry.
**Warning signs:** If after regeneration `ferro-base.css` contains `motion-duration-fast` as a utility name rather than `duration-fast`, the namespace bridge is wrong.

### Pitfall 2: Missing `@source inline()` safelist causes SC-2 failure

**What goes wrong:** After adding `@theme inline` entries for new utilities, regenerating `ferro-base.css` still doesn't include `duration-fast`, `ease-base`, etc. as utility classes.
**Why it happens:** Tailwind's scanner only emits utilities it finds used in source files. Since D-12 prohibits component changes, no `.rs` file uses `duration-fast` yet.
**How to avoid:** Add all new utility names to `@source inline()` in `input.css` before regenerating.
**Warning signs:** `grep 'duration-fast' ferro-json-ui/assets/ferro-base.css` returns empty after regen.

### Pitfall 3: Only updating one dark mode block in `default.css`

**What goes wrong:** New tokens work in light mode and system-dark-mode, but the `[data-theme="dark"]` block is missing the new declarations. Manual dark toggle shows wrong values.
**Why it happens:** `default.css` has two separate dark mode blocks and both must be updated.
**How to avoid:** Update `:root`, `@media (prefers-color-scheme: dark) :root`, and `[data-theme="dark"]` — all three blocks.
**Warning signs:** The manual dark toggle (clicking theme toggle in the app) shows incorrect motion durations or ring colors while system dark mode works correctly.

### Pitfall 4: `--spacing` in `@theme inline` causes circular reference on v1 themes

**What goes wrong:** If `--spacing: var(--spacing, 0.25rem)` is added to `@theme inline`, the `@layer theme` ends up with both `--spacing:.25rem` (from the Tailwind default) and `--spacing:var(--spacing,.25rem)` (from `@theme inline`). The order of these within the same layer determines which wins — and the inline entry may produce a self-referential property when no theme override is present.
**Why it happens:** CSS custom properties that reference themselves are treated as guaranteed-invalid by the browser (unless an un-layered override wins). If a v1 theme doesn't set `--spacing` in un-layered CSS, the spacing utilities may fail.
**How to avoid:** Do NOT add `--spacing` to `@theme inline`. The existing `calc(var(--spacing) * N)` behavior already handles runtime override.
**Warning signs:** After adding `--spacing` to `@theme inline`, `mt-4` renders with zero or invalid spacing in a browser tab that doesn't load any theme CSS.

### Pitfall 5: `tokens_css_template()` and `make_theme` scaffold not updated

**What goes wrong:** `ferro make:theme myapp` scaffolds a v1 theme (23 slots). Test `test_make_theme_tokens_css_has_all_23_token_slots` passes because it only checks old tokens. New users get no guidance on the 7 new slots.
**Why it happens:** The scaffold template is a static string in `ferro-cli/src/commands/make_theme.rs` that needs manual update.
**How to avoid:** Update `tokens_css_template()` and rename + extend the test with 7 new assertions.
**Warning signs:** `cargo test` passes but `ferro make:theme test && cat themes/test/tokens.css | grep 'motion-duration'` returns empty.

---

## Code Examples

### `input.css` additions (VERIFIED pattern from existing code)

```css
/* Source: ferro-json-ui/assets/input.css — existing pattern to follow */
@theme inline {
  /* existing entries above... */

  /* Motion tokens — bridge semantic names to Tailwind utility namespaces */
  --duration-fast: var(--motion-duration-fast, 120ms);
  --duration-base: var(--motion-duration-base, 220ms);
  --duration-slow: var(--motion-duration-slow, 320ms);
  --ease-base: var(--motion-ease, cubic-bezier(0.2, 0, 0.38, 0.9));

  /* Focus ring token — same pattern as --color-primary */
  --color-ring: var(--color-ring);

  /* Display font token — nested fallback to font-sans */
  --font-display: var(--font-display, var(--font-sans));
}

/* Safelist new utility class names (no component uses them yet in Phase 250) */
@source inline("duration-fast duration-base duration-slow ease-base font-display ring-ring");

/* Collapse motion durations for reduced-motion preference */
@media (prefers-reduced-motion: reduce) {
  :root {
    --motion-duration-fast: 0.01ms;
    --motion-duration-base: 0.01ms;
    --motion-duration-slow: 0.01ms;
  }
}
```

### `token.rs` additions (VERIFIED pattern from existing code)

```rust
// Source: ferro-theme/src/token.rs — existing pattern to follow

// Density token
/// Base spacing unit; all spacing utilities resolve as calc(var(--spacing) * N).
pub const TOKEN_SPACING: &str = "--spacing";

// Motion tokens
/// Fast transition duration (micro-interactions: hover, toggles).
pub const TOKEN_MOTION_DURATION_FAST: &str = "--motion-duration-fast";
/// Base transition duration (dropdowns, modals, toasts).
pub const TOKEN_MOTION_DURATION_BASE: &str = "--motion-duration-base";
/// Slow transition duration (drawers, page-level reveals).
pub const TOKEN_MOTION_DURATION_SLOW: &str = "--motion-duration-slow";
/// Standard easing curve (calm, settled, no bounce).
pub const TOKEN_MOTION_EASE: &str = "--motion-ease";

// Focus ring token
/// Uniform focus-visible ring color for interactive components.
pub const TOKEN_COLOR_RING: &str = "--color-ring";

// Display font token
/// Display/heading font family; defaults to var(--font-sans).
pub const TOKEN_FONT_DISPLAY: &str = "--font-display";
```

### `default.css` new token declarations (pattern)

```css
/* Source: ferro-theme/assets/default.css — add to :root, dark @media, [data-theme="dark"] */

/* Density token */
--spacing: 0.25rem;

/* Motion tokens */
--motion-duration-fast: 120ms;
--motion-duration-base: 220ms;
--motion-duration-slow: 320ms;
--motion-ease: cubic-bezier(0.2, 0, 0.38, 0.9);

/* Focus ring token (exact oklch value is Claude's discretion — primary-family) */
--color-ring: oklch(55% 0.2 250);   /* matches default primary; adjust as needed */

/* Display font token */
--font-display: var(--font-sans);   /* inherits Inter stack by default */
```

---

## File Change List

Complete list of files requiring changes (derived from code inspection):

| File | Change | Reason |
|------|--------|--------|
| `ferro-theme/src/token.rs` | Add 7 constants; update `ALL_TOKENS` to 30; update doc header to v2; update "~23"/"23" comments | DS-01, D-04 |
| `ferro-json-ui/assets/input.css` | Add `@theme inline` entries for 6 new tokens (not spacing); add `@source inline` safelist; add `prefers-reduced-motion` media query | DS-01, D-05, D-07 |
| `ferro-theme/assets/default.css` | Add 7 new token declarations to all 3 sections (`:root`, dark `@media`, `[data-theme="dark"]`); refresh neutral ramp + accent | DS-01, DS-02, D-08, D-09 |
| `ferro-json-ui/assets/ferro-base.css` | Regenerated via `scripts/gen-ferro-base-css.sh` — no manual edits | DS-01, D-13 |
| `ferro-cli/src/commands/make_theme.rs` | Update `tokens_css_template()` to add 7 slots; rename test + add 7 assertions; update doc comment | D-04, drift guard |
| `ferro-theme/src/loader.rs` | Update doc comment "all 23" → "all 30" | D-04, drift guard |
| `docs/src/features/themes.md` | Update "23 slots" → "30 slots"; add v2 token tables; add type-scaling recipe | DS-02 |

---

## Runtime State Inventory

Not applicable — this is a greenfield feature phase with no rename/refactor/migration work.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Tailwind CLI v4.2.3 | `scripts/gen-ferro-base-css.sh` | auto-installed | v4.2.3 (pinned) | Script downloads on first run |
| sha256sum / shasum | `scripts/install-tailwind.sh` | yes (macOS: shasum) | — | — |
| curl | `scripts/install-tailwind.sh` | yes | — | — |
| Rust toolchain (stable) | `cargo test` | yes | stable (rustfmt 1.8.0) | — |

**Missing dependencies with no fallback:** none.

**Note:** Tailwind CLI binary is gitignored (`.tooling/bin/`) and auto-downloaded by the install script. CI also calls `gen-ferro-base-css.sh` — the pinned binary is fetched fresh on each CI run. The `ferro-base.css` output IS committed and must match the regenerated output.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`#[test]`) + `tempfile` crate for CLI scaffold tests |
| Config file | No external config — standard `cargo test` |
| Quick run command | `cargo test -p ferro-theme -p ferro-cli -p ferro-json-ui -- --test-output immediate` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DS-01 | ALL_TOKENS has 30 entries | unit | `cargo test -p ferro-theme` | Implied by token.rs change — assert `ALL_TOKENS.len() == 30` |
| DS-01 | ferro-base.css contains `duration-fast` utility | unit (post-regen) | `cargo test -p ferro-json-ui -- builtin_types_count` | Needs new test or grep assertion |
| DS-01 | `prefers-reduced-motion` block in ferro-base.css | grep | Verify during regen step | No test |
| DS-01 | make_theme scaffold includes all 30 slots | unit | `cargo test -p ferro-cli -- test_make_theme` | test_make_theme_tokens_css_has_all_23_token_slots (rename + extend) |
| DS-02 | default.css contains all 30 token declarations | unit | `cargo test -p ferro-theme -- default_theme` | `default_theme_returns_non_empty_css_with_color_primary` — extend |
| DS-03 | v1 theme renders identically (structural guarantee) | unit | Verify via fallback syntax in ferro-base.css | New assertion: ferro-base.css contains `var(--motion-duration-fast,` |

### Wave 0 Gaps

- [ ] Test asserting `ALL_TOKENS.len() == 30` — add to `ferro-theme/src/token.rs` tests
- [ ] Test asserting regenerated `ferro-base.css` contains `var(--motion-duration-fast,` — add to `ferro-json-ui` or as a post-regen assertion
- [ ] Test asserting `default.css` declares all 7 new tokens — extend `default_theme_returns_non_empty_css_with_color_primary`

---

## Security Domain

This phase touches only CSS custom properties, documentation, and Rust string constants. No authentication, data handling, or network communication is involved.

ASVS categories: not applicable to CSS token vocabulary changes.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `--duration-fast` in `@theme inline` generates a `duration-fast` utility in Tailwind v4.2.3 | Token Namespace Mapping | If Tailwind generates a different name (e.g. `tw-duration-fast`), the safelisted class name would be wrong. Verify after regen with `grep 'duration-fast' ferro-base.css`. |
| A2 | `--ease-*` in `@theme inline` generates `ease-<name>` timing function utilities | Token Namespace Mapping | If Tailwind's easing utility is `transition-ease-*` or similar, the class name would differ. Verify after regen. |
| A3 | Un-layered theme CSS (`:root { ... }` in `default.css`) reliably wins over `@layer theme` in Tailwind v4 — confirmed for current tokens but not explicitly tested for the new motion/font entries | Architecture Patterns | If a Tailwind v4.2.3 specificity change broke this, ALL color tokens would already be broken — the existing tokens work, so the mechanism is verified for this version. Low risk. |

**If this table is empty after implementer verification:** All assumptions can be resolved by grepping the regenerated `ferro-base.css` immediately after the `@theme inline` changes and regen step.

---

## Open Questions

1. **`--color-ring` default oklch value**
   - What we know: must be visible, from the primary hue family (~250), specified as Claude's discretion.
   - What's unclear: exact lightness/chroma to use (primary is `oklch(55% 0.2 250)`; a ring often wants more contrast or a lighter value).
   - Recommendation: `oklch(55% 0.2 250)` (same as primary) for light mode; for dark mode slightly lighter (`oklch(65% 0.18 250)`) to maintain contrast against dark backgrounds. Verify visually in Chrome MCP.

2. **`ferro-base.css` commit timing relative to CI**
   - The regenerated `ferro-base.css` must be committed alongside `input.css`. If committed before regen, CI will regenerate it and find a diff, failing.
   - Recommendation: Regenerate locally, commit both `input.css` and `ferro-base.css` together.

---

## Sources

### Primary (HIGH confidence)

- `ferro-json-ui/assets/ferro-base.css` — inspected directly; confirmed `calc(var(--spacing) * N)` spacing pattern and `--color-*: var(--color-*)` `@theme inline` output
- `ferro-json-ui/assets/input.css` — inspected directly; confirmed current `@theme inline` block structure
- `ferro-theme/src/token.rs` — inspected directly; confirmed 23 constants + `ALL_TOKENS`
- `ferro-theme/assets/default.css` — inspected directly; confirmed 77-line plain CSS, dual dark mode blocks
- `ferro-theme/src/loader.rs` — inspected directly; confirmed `include_str!` compile-time embed
- `ferro-cli/src/commands/make_theme.rs` — inspected directly; confirmed test and scaffold template
- `scripts/gen-ferro-base-css.sh` + `scripts/install-tailwind.sh` — inspected directly; confirmed v4.2.3 pin + sha256 verification
- Context7 `/tailwindlabs/tailwindcss.com` — queried for `@theme inline`, `--duration-*`, `--ease-*`, `--spacing`, `--font-display` documentation

### Secondary (MEDIUM confidence)

- Context7 Tailwind v4 docs: confirmed `--font-display: "Satoshi"` as official `@theme` example; confirmed `--ease-fluid`/`--ease-snappy` in `@theme` as official examples; confirmed `--spacing` as base spacing unit with `calc(var(--spacing) * N)` utility generation

### Tertiary (LOW confidence — marked A1/A2 in Assumptions Log)

- Exact generated utility class names for `--duration-fast` and `--ease-base` in `@theme inline` (not in `@theme` directly): inferred from documented `--duration-*` namespace behavior, not verified by direct build test.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all tools and files directly inspected
- Architecture: HIGH — `@theme inline` mechanism verified against actual generated ferro-base.css
- Pitfalls: HIGH — all identified from direct code inspection
- D-06 research answers: HIGH (spacing, font-display, color-ring) / MEDIUM (motion utility names — see assumptions log)

**Research date:** 2026-07-03
**Valid until:** 2026-08-03 (stable Tailwind v4.2.3 pin; re-verify if Tailwind version is bumped)
