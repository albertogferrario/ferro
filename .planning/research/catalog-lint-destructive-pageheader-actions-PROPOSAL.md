# Catalog Lint — Destructive Inline in PageHeader.actions

**Status:** proposal for future phase (no PLAN.md yet — promote via `/gsd-plan-phase` when picked up)
**Author:** Claude (session 2026-06-16)
**Triggered by:** gestiscilo dashboard UX sweep 2026-06-16 — actions normalization rule "destructive sempre dentro kebab; max 2-3 inline buttons"

## What is broken

ferro-json-ui's `PageHeader.actions` (and `DetailPage.actions`) accepts a `Vec<String>` of element IDs. Catalog enforces no rules on what those elements are. In practice, consumers (gestiscilo) have to police placement themselves: destructive actions (variant=`destructive` or confirm dialog with `variant=danger`) must go inside a `DropdownMenu` (kebab "Altro"), never as inline buttons in the header.

The convention is documented in gestiscilo's `CLAUDE.md` "Dashboard Page Patterns" and was applied across 10+ pages on 2026-06-16. There is no scaffolding-time signal when a consumer violates it — the warning depends on a human reviewer noticing the visual pattern, which scales poorly.

## What we want

A lint-style warning emitted by ferro's catalog validator (the same path that surfaces "failed to decode props") when:

1. A `Button` element with `variant: "destructive"` is referenced from `PageHeader.actions` or `DetailPage.actions`.
2. A `Form` wrapping a destructive Button is referenced from `PageHeader.actions` / `DetailPage.actions`.
3. An action with `confirm.variant: "danger"` lives directly in header actions instead of inside a `DropdownMenu.items`.

The warning surfaces in the rendered HTML as an HTML comment (matches existing decode-failure pattern) so it is visible in dev but does not break the page. Optionally: a strict-mode env var (`FERRO_LINT_STRICT=1`) elevates warnings to render-time errors so CI can gate.

## Why a lint, not a type-system constraint

Tightening `PageHeaderProps.actions` to a sum type (`InlineAction | DestructiveAction` etc.) would force every consumer to refactor every header at once, with no escape valve for legitimate edge cases (e.g. a destructive operator-only debug action on a developer-only page). A lint warns the 95% bad placements while preserving consumer override authority for the 5%.

This also matches `gestiscilo.md`'s structural-guarantees principle ("abstract so inconsistency is architecturally impossible") with a softer landing: the lint is the structural signal, the override path stays open.

## Design space — four options

### Option a — soft warning only

- Detect destructive-in-header at render time.
- Emit `<!-- ferro-json-ui: destructive 'btn_X' in PageHeader.actions; prefer DropdownMenu (kebab) per dashboard conventions -->` immediately before the offending button's HTML.
- No env-var, no strict mode, no error path.

**Pros:** smallest surface, zero breakage risk, ships in ~2h.
**Cons:** comments are noise; experienced devs ignore them. No CI gate.

### Option b — warning + strict mode env var

Same as (a), plus:
- `FERRO_LINT_STRICT=1` env (or `Spec::with_strict_lint()` builder) elevates the warning into an `Element` render-time error.
- In strict mode the page comments out the entire actions slot and renders an inline error message in its place.

**Pros:** opt-in CI gate; team can enable in `cargo test` / `cargo check --release` runs to fail builds on regressions.
**Cons:** two render paths to maintain, env-var sprawl risk.

### Option c — catalog-level static analysis

Add a `Catalog::lint(spec: &Spec) -> Vec<LintWarning>` API that consumers call at boot or in tests. Lint rules live as data, registered on the catalog (`Catalog::register_lint_rule(rule)`).

- Rule for "destructive in PageHeader.actions" is the first rule, ships with ferro.
- Consumers can register their own rules (e.g. gestiscilo could add "Files breadcrumb must not repeat 'File' twice").
- ferro CLI gets a `ferro doctor --lint` subcommand that runs all rules against every JSON view spec in the project.

**Pros:** clean extension point, decouples lint detection from render, testable in isolation, CI gate via CLI exit code.
**Cons:** larger surface (~1-2 days), introduces a new pluggable subsystem; risk of over-engineering for "one rule".

### Option d — ActionGroup component with built-in placement rules (the original "option d" from the gestiscilo backlog)

Introduce `ActionGroup` as a first-class component that wraps header actions with placement awareness:

```rust
pub struct ActionGroupProps {
    pub inline: Vec<String>,       // primary actions (max 2-3 enforced)
    pub overflow: Vec<String>,     // dropdown items
    pub destructive: Vec<String>,  // always rendered in overflow, never inline
}
```

`PageHeader.actions` accepts either `Vec<String>` (legacy) or a single `ActionGroup` element ID (new). Inside an `ActionGroup` the rules are enforced by construction: `destructive` always lands in the kebab, `inline` is capped at 3 (extras silently demoted to `overflow`), and the catalog can lint when consumers route a destructive button through `inline`.

**Pros:** structural guarantee (matches `gestiscilo.md` principle); cleanest API; the abstraction lives in ferro so every dashboard inherits the discipline.
**Cons:** biggest surface (~2-3 days); requires migration path for existing consumers; new component to document and version.

## Recommendation

**Land (a) first as Phase X, plan (c) as Phase X+1 if a second rule appears, defer (d) until 3+ rules exist or until a consumer asks for it.**

Rationale:
- Today there is exactly one rule (destructive-in-header). Building a plugin system (option c) or a new component (option d) for one rule is premature.
- The HTML-comment warning (option a) closes the immediate gap: gestiscilo devs see the warning during dev → fix the placement → done.
- If/when a second rule shows up (e.g. "Files breadcrumb redundancy" lint, "max inline actions = 3" lint), promote to option (c) and migrate the first rule into the new system.
- Option (d) is the "long game" answer — when ferro has a stable lint system and gestiscilo has lived with the rule for some months, an `ActionGroup` component becomes the natural next abstraction. Capture as a forward-looking item.

## Detection algorithm (option a — concrete)

At the top of `render_page_header` and `render_detail_page` in `ferro-json-ui/src/render/containers.rs`:

```rust
for action_id in &props.actions {
    if let Some(action_el) = spec.elements.get(action_id) {
        if is_destructive(action_el, spec) {
            html.push_str(&format!(
                "<!-- ferro-json-ui: lint: '{}' is destructive — prefer DropdownMenu (kebab) per dashboard conventions -->\n",
                html_escape(action_id)
            ));
        }
    }
}

fn is_destructive(el: &Element, spec: &Spec) -> bool {
    // (a) Button with variant: destructive
    if el.type_name == "Button" {
        if let Some(v) = el.props.get("variant").and_then(|v| v.as_str()) {
            if v == "destructive" { return true; }
        }
    }
    // (b) Form wrapping a destructive Button (common gestiscilo pattern)
    if el.type_name == "Form" {
        for child_id in &el.children {
            if let Some(child) = spec.elements.get(child_id) {
                if is_destructive(child, spec) { return true; }
            }
        }
    }
    // (c) Action with confirm.variant: danger
    if let Some(action) = el.props.get("action") {
        if let Some(confirm) = action.get("confirm") {
            if confirm.get("variant").and_then(|v| v.as_str()) == Some("danger") {
                return true;
            }
        }
    }
    false
}
```

## Phase scope (when promoted)

Single plan, single wave, ~2-3 hours of work:

1. **Plan A — Lint emit + tests** (`ferro-json-ui/src/render/containers.rs` + `tests/lint_destructive.rs`)
   - `is_destructive(&Element, &Spec) -> bool` helper
   - Call site in `render_page_header` and `render_detail_page`
   - 4-6 unit tests: destructive Button inline → warning emitted; destructive Button in DropdownMenu → no warning; Form-wrapped destructive → warning; non-destructive Button → no warning; destructive Action with danger confirm → warning
   - Update existing `render_page_header` golden tests to expect the new comment when input is intentionally lint-bait

2. **Plan B — Docs + release notes**
   - Add a short section to `ferro-json-ui/README.md` documenting the lint and the override (just don't trigger it)
   - Mention in next `CHANGELOG.md` entry

## Non-goals (this phase)

- Strict mode / env var elevation (option b) — pull when a consumer asks for CI gate.
- Pluggable rule system (option c) — pull when a second rule lands.
- `ActionGroup` component (option d) — capture in a follow-up proposal once option a has lived in production for ≥1 milestone.

## Risks

- **False positives:** if a consumer legitimately wants a destructive action inline (operator-only debug page, intentional emphasis), the warning is noise. Mitigation: warning is an HTML comment — invisible in production, ignored by linters. Document the override path in release notes.
- **Test fragility:** golden tests in `containers.rs` that assert exact HTML output for PageHeader will need updates if they include destructive actions. Audit and update at plan time — should be <5 sites.

## Forward link

When `ActionGroup` (option d) gets promoted later, this proposal stays as the design-history record. The `ActionGroup` proposal can reference back to this one as "the simpler precursor that bought us time to learn what the right component shape was".
