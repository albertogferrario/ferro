---
status: testing
phase: 99-semantic-theme-system-with-intent-driven-templates
source: [99-01-SUMMARY.md, 99-02-SUMMARY.md, 99-03-SUMMARY.md, 99-04-SUMMARY.md, 99-05-SUMMARY.md]
started: 2026-03-12T12:00:00Z
updated: 2026-03-12T12:15:00Z
---

## Current Test

number: 10
name: Visual Rendering in Browser
expected: |
  Start the app server, navigate to any projection page in Chrome. Inspect rendered HTML — components should use semantic CSS classes (bg-primary, text-text, etc.). If ThemeMiddleware is configured, the <head> should contain a <style> tag with the theme CSS custom properties.
awaiting: user response

## Tests

### 1. Theme CLI Scaffolding
expected: Running `cargo run -p ferro-cli -- make:theme test-brand` creates `themes/test-brand/tokens.css` and `themes/test-brand/theme.json`. tokens.css contains all 23 semantic token slots in Tailwind v4 `@theme` format plus a `@media (prefers-color-scheme: dark)` block. theme.json contains `{}`.
result: pass

### 2. Duplicate Theme Rejection
expected: Running `cargo run -p ferro-cli -- make:theme test-brand` a second time returns a clear error message indicating the theme already exists.
result: pass

### 3. Semantic Classes in Component Output
expected: Running `cargo test -p ferro-json-ui` passes all 364 tests. Inspecting render.rs test output shows semantic classes like `bg-primary`, `text-text`, `border-border`, `rounded-radius-md` — no hardcoded Tailwind colors like `bg-blue-600`, `bg-gray-50`, `text-gray-900`.
result: pass

### 4. Theme CSS Injection in JSON-UI Head
expected: Running `cargo test -p ferro-rs --features theme` passes. The json_ui theme tests confirm that when a theme is active in task-local context, a `<style>` tag with the theme CSS appears in the HTML head after the Tailwind CDN script.
result: pass

### 5. Theme Middleware Resolver Chain
expected: Running `cargo test -p ferro-rs --features theme theme::` passes all 24 theme module tests. TenantThemeResolver reads tenant plan, HeaderThemeResolver reads X-Theme header, DefaultResolver always returns fallback. First match wins.
result: pass

### 6. Intent Template Override Rendering
expected: Running `cargo test -p ferro-projections` passes all 315 tests. When RenderContext.templates contains a ThemeTemplates override for Browse intent, the renderer uses the template's slot arrangement instead of the built-in layout. When no template is provided, output is identical to built-in.
result: pass

### 7. Default Theme CSS Token Completeness
expected: `ferro-theme/assets/default.css` contains a `@theme` block with all 23 semantic tokens: 6 surface (background, surface, card, border, text, text-muted), 8 role (primary, primary-foreground, secondary, secondary-foreground, accent, destructive, success, warning), 4 radius (sm, md, lg, full), 3 shadow (sm, md, lg), 2 typography (font-family-sans, font-family-mono). Plus a dark mode media query block.
result: pass

### 8. Publish Workflow Updated
expected: `.github/workflows/publish.yml` includes `ferro-theme` in WAVE1_CRATES alongside other leaf crates (ferro-lang, ferro-stripe, etc.).
result: pass

### 9. Documentation Coverage
expected: `docs/src/features/themes.md` exists and covers: token reference table (all 23 slots), dark mode configuration, intent templates with slot vocabulary, ThemeMiddleware setup, multi-tenant theme resolution. `docs/src/SUMMARY.md` lists the Themes page in the Features section.
result: pass

### 10. Visual Rendering in Browser
expected: Start the app server, navigate to any projection page in Chrome. Inspect rendered HTML — components should use semantic CSS classes (bg-primary, text-text, etc.). If ThemeMiddleware is configured, the `<head>` should contain a `<style>` tag with the theme CSS custom properties.
result: skipped
reason: Framework repo has no runnable app with frontend. Will test visually in gestiscilo app.

## Summary

total: 10
passed: 9
issues: 0
pending: 0
skipped: 1

## Gaps

[none yet]
