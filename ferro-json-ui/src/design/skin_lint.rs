//! Skin lint: detect raw color/size literals and missing interaction states
//! in `@layer components` CSS skin rules.
//!
//! Two checks:
//! 1. `check_skin_raw_literals` — any `fjui-` rule using a raw color literal
//!    (#hex, rgb(, rgba(, hsl(, oklch() not inside var(...)) fails.
//! 2. `check_skin_interaction_states` — any interactive `fjui-` rule missing
//!    :hover, :focus-visible, :active, or :disabled fails.

use super::types::{Finding, Severity};

// ── Interactive component prefixes ────────────────────────────────────────────

/// Interactive `fjui-` block prefixes that require all four interaction states.
///
/// Derived from UI-SPEC §Interactive State Requirements.
const INTERACTIVE_PREFIXES: &[&str] = &[
    "fjui-btn",
    "fjui-input",
    "fjui-select",
    "fjui-textarea",
    "fjui-sidebar__nav-item",
    "fjui-menu-item",
    "fjui-tab",
    "fjui-table__row",
    "fjui-kanban__card",
    "fjui-action-card",
    "fjui-tile",
    "fjui-notification-dropdown__trigger",
    "fjui-toast__close",
];

/// Non-interactive `fjui-` prefixes that are exempt from interaction-state checks.
///
/// Note: `fjui-table__row` is interactive (listed above) but
/// `fjui-table` (the table wrapper) and `fjui-table__cell*` are not.
/// `fjui-table__row` is exempt from `:disabled` (rows are not form controls —
/// documented in check_skin_interaction_states).
const NON_INTERACTIVE_PREFIXES: &[&str] = &[
    "fjui-card",
    "fjui-badge",
    "fjui-alert",
    "fjui-stat-card",
    "fjui-header",
    "fjui-table__header",
    "fjui-table__cell",
    "fjui-table__body",
    "fjui-stat-card__value",
    "fjui-sidebar__group-label",
];

// ── CSS properties that carry color values (checked for raw literals) ─────────

const COLOR_PROPERTIES: &[&str] = &[
    "color",
    "background",
    "background-color",
    "border-color",
    "box-shadow",
    "fill",
    "stroke",
    "outline-color",
    "caret-color",
    "text-decoration-color",
    "accent-color",
];

// ── Rule block extractor ──────────────────────────────────────────────────────

/// A parsed `fjui-` rule block from `@layer components`.
struct RuleBlock {
    /// The base selector name (e.g. "fjui-btn", "fjui-table__row").
    selector: String,
    /// Full CSS text of the rule body (between the outermost `{` and `}`).
    body: String,
}

/// Extract the `@layer components { ... }` content from `css`.
///
/// Returns the inner text of the first `@layer components` block, or
/// an empty string if not found. Tracks brace depth to handle nesting.
fn extract_components_layer(css: &str) -> &str {
    let marker = "@layer components";
    let start = match css.find(marker) {
        Some(s) => s,
        None => return "",
    };
    let after = &css[start..];
    let brace_start = match after.find('{') {
        Some(b) => b + 1,
        None => return "",
    };
    let content = &after[brace_start..];

    let mut depth = 1usize;
    let mut end = 0;
    for (i, ch) in content.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = i;
                    break;
                }
            }
            _ => {}
        }
    }
    &content[..end]
}

/// Extract all top-level `.fjui-*` rule blocks from `layer_content`.
///
/// Segments the text by `.fjui-` selector boundaries, tracking brace depth
/// to handle nested `&:hover { }` etc.
fn extract_fjui_rules(layer_content: &str) -> Vec<RuleBlock> {
    let mut rules = Vec::new();
    let mut pos = 0;
    let bytes = layer_content.as_bytes();
    let len = bytes.len();

    while pos < len {
        // Find next ".fjui-" occurrence
        let rest = &layer_content[pos..];
        let dot_fjui = match rest.find(".fjui-") {
            Some(p) => pos + p,
            None => break,
        };

        // Extract selector name: from ".fjui-" to the next whitespace or "{"
        let sel_start = dot_fjui + 1; // skip "."
        let sel_rest = &layer_content[sel_start..];
        let sel_end = sel_rest
            .find(|c: char| c.is_whitespace() || c == '{' || c == ':' || c == ',')
            .unwrap_or(sel_rest.len());
        let selector = sel_rest[..sel_end].to_string();

        // Find the opening brace of this rule
        let after_sel = &layer_content[dot_fjui..];
        let brace_rel = match after_sel.find('{') {
            Some(b) => b,
            None => break,
        };
        let body_start = dot_fjui + brace_rel + 1;

        // Collect the body up to the matching closing brace (depth tracking)
        let body_content = &layer_content[body_start..];
        let mut depth = 1usize;
        let mut body_end = 0;
        for (i, ch) in body_content.char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        body_end = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        let body = body_content[..body_end].to_string();
        rules.push(RuleBlock { selector, body });

        // Advance past the closing brace
        pos = body_start + body_end + 1;
    }

    rules
}

// ── Raw-literal detection ─────────────────────────────────────────────────────

/// CSS named color keywords that must not appear as raw values in skin rules.
///
/// Whole-word matching is used to avoid false positives on identifiers containing
/// these strings (e.g. `--color-background`, `blacklist`, var names, comments).
const NAMED_COLORS: &[&str] = &[
    "black", "white", "red", "green", "blue", "yellow", "orange", "purple",
    "gray", "grey", "transparent",
    // extend per CSS Color Level 4 named colors as needed
];

/// Returns true if `value` contains a raw color literal not wrapped in `var(...)`.
///
/// Allowed: `var(--token)`, `color-mix(in oklab, var(...) N%, ...)`, `transparent`,
/// `currentColor`, `inherit`, `initial`, structural geometry values.
///
/// Flagged: `#hex`, `rgb(`, `rgba(`, `hsl(`, `oklch(` used as direct values, and
/// CSS named color keywords (`black`, `white`, etc.) used as whole words in the value.
fn contains_raw_color_literal(value: &str) -> bool {
    let v = value.trim();

    // Allow pure token references and CSS globals
    if v.starts_with("var(--")
        || v == "transparent"
        || v == "currentColor"
        || v == "inherit"
        || v == "initial"
        || v == "none"
        || v == "unset"
        || v.is_empty()
    {
        return false;
    }

    // Allow color-mix() — inspect its arguments for raw literals (named colors included)
    if v.starts_with("color-mix(") {
        let inner = &v["color-mix(".len()..];
        // Flag raw hex or functional notations directly
        if inner.contains('#')
            || inner.contains("rgb(")
            || inner.contains("rgba(")
            || inner.contains("hsl(")
            || (inner.contains("oklch(") && !inner.contains("var(--"))
        {
            return true;
        }
        // Flag named color keywords inside color-mix arguments.
        // Avoid false positives on var(--...) names that contain keyword substrings
        // (e.g. var(--color-background) contains "background", not a named color;
        // but `black` inside color-mix is a raw named-color argument).
        let lower = inner.to_lowercase();
        for kw in NAMED_COLORS {
            if kw == &"transparent" {
                // `transparent` is an allowed CSS keyword — skip in color-mix too
                continue;
            }
            if contains_named_color_word(&lower, kw) {
                return true;
            }
        }
        return false;
    }

    // Check for raw functional color literals
    if v.contains('#') {
        return true;
    }
    if v.contains("rgb(") || v.contains("rgba(") {
        return true;
    }
    if v.contains("hsl(") {
        return true;
    }
    // oklch( used directly (not inside var() which we already approved above)
    if v.contains("oklch(") {
        return true;
    }

    // Check for standalone CSS named color keywords.
    // Uses whole-word matching to avoid false positives on var(--...) identifiers.
    let lower = v.to_lowercase();
    for kw in NAMED_COLORS {
        if kw == &"transparent" {
            // `transparent` is whitelisted above — skip here
            continue;
        }
        if contains_named_color_word(&lower, kw) {
            return true;
        }
    }

    false
}

/// Returns true if `haystack` contains `keyword` as a whole word.
///
/// A "word" boundary here is any character that is not alphanumeric and not `-`
/// (to avoid matching `black` inside `--color-background` or `blacklist`).
fn contains_named_color_word(haystack: &str, keyword: &str) -> bool {
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(keyword) {
        let abs = start + pos;
        let before_ok = abs == 0 || {
            let c = haystack.as_bytes()[abs - 1] as char;
            !c.is_alphanumeric() && c != '-'
        };
        let after_pos = abs + keyword.len();
        let after_ok = after_pos >= haystack.len() || {
            let c = haystack.as_bytes()[after_pos] as char;
            !c.is_alphanumeric() && c != '-'
        };
        if before_ok && after_ok {
            return true;
        }
        start = abs + 1;
        if start >= haystack.len() {
            break;
        }
    }
    false
}

/// Check a single rule body's declarations for raw color literals.
///
/// Scans top-level (non-nested) declarations only; nested `&:hover {}` bodies
/// are scanned via their own segments when split on `;`.
fn check_rule_for_raw_literals(selector: &str, body: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Split on semicolons to get individual declarations (handles multiline/single-line)
    for segment in body.split(';') {
        let segment = segment.trim();
        // Skip empty, nested block openers/closers, and pseudo-class declarations
        if segment.is_empty() || segment.starts_with('}') || segment.starts_with('{') {
            continue;
        }
        // Strip nested block contents: if segment contains '{', skip (it's a block open)
        if segment.contains('{') {
            continue;
        }

        // Check if this looks like a color property declaration
        if let Some(colon_pos) = segment.find(':') {
            let prop = segment[..colon_pos].trim().trim_start_matches('&').trim();
            // Strip pseudo-class from property (e.g. ":hover" before property)
            let prop = prop.trim_start_matches(':').trim();
            let value = segment[colon_pos + 1..].trim();

            let is_color_prop = COLOR_PROPERTIES.iter().any(|&cp| cp == prop);
            if is_color_prop && contains_raw_color_literal(value) {
                findings.push(Finding {
                    rule: "skin-raw-literals",
                    element_id: None,
                    severity: Severity::Warning,
                    message: format!(
                        "Rule `.{selector}` contains raw literal `{value}` in `{prop}` — use var(--token) instead."
                    ),
                    suggestion: "Replace the raw color/size literal with a var(--token-name) reference.".into(),
                });
            }
        }
    }

    findings
}

// ── Interaction-state checks ──────────────────────────────────────────────────

/// Required interaction states for interactive components.
const REQUIRED_STATES: &[&str] = &[":hover", ":focus-visible", ":active", ":disabled"];

/// Returns true if `selector` starts with one of the INTERACTIVE_PREFIXES.
fn is_interactive(selector: &str) -> bool {
    INTERACTIVE_PREFIXES.iter().any(|&p| selector.starts_with(p))
}

/// Returns true if `selector` is a modifier/variant of an interactive prefix
/// (e.g. "fjui-btn--primary") but NOT the base interactive rule itself.
/// Variant rules don't need to repeat all four states — the base rule carries them.
fn is_interactive_variant(selector: &str) -> bool {
    // A variant contains "--" after the base prefix
    INTERACTIVE_PREFIXES.iter().any(|&p| {
        selector.starts_with(p) && selector.len() > p.len() && selector[p.len()..].starts_with("--")
    })
}

// ── Public check functions ────────────────────────────────────────────────────

/// Check all `fjui-` rules in `@layer components` for raw color/size literals.
///
/// Returns one `Warning` finding per violation naming the selector and literal.
/// Structural geometry values (1px border widths, layout utilities) are not flagged.
pub fn check_skin_raw_literals(css: &str) -> Vec<Finding> {
    let layer = extract_components_layer(css);
    if layer.is_empty() {
        // If no @layer components block, scan the whole CSS
        // (to support unit tests that pass raw rule strings)
        return check_raw_literals_in_content(css);
    }
    check_raw_literals_in_content(layer)
}

fn check_raw_literals_in_content(content: &str) -> Vec<Finding> {
    let rules = extract_fjui_rules(content);
    let mut findings = Vec::new();
    for rule in &rules {
        findings.extend(check_rule_for_raw_literals(&rule.selector, &rule.body));
    }
    findings
}

/// Check all interactive `fjui-` rules for missing interaction states.
///
/// For each base interactive rule (not a `--variant` modifier), verifies that
/// the rule body (or the surrounding layer CSS) contains all four of:
/// `:hover`, `:focus-visible`, `:active`, `:disabled`.
///
/// `fjui-table__row` is exempt from `:disabled` because table rows are not
/// form controls and cannot receive the `disabled` attribute.
///
/// Non-interactive components (fjui-card, fjui-badge, fjui-alert, fjui-stat-card,
/// fjui-header) are silently skipped.
pub fn check_skin_interaction_states(css: &str) -> Vec<Finding> {
    let layer = extract_components_layer(css);
    let search_content = if layer.is_empty() { css } else { layer };

    let rules = extract_fjui_rules(search_content);
    let mut findings = Vec::new();

    for rule in &rules {
        let sel = &rule.selector;

        // Skip non-interactive components
        if NON_INTERACTIVE_PREFIXES.iter().any(|&p| sel.starts_with(p)) {
            continue;
        }

        // Only check base interactive rules, not --variant modifiers
        if !is_interactive(sel) || is_interactive_variant(sel) {
            continue;
        }

        // Determine required states for this selector.
        // fjui-table__row is exempt from :disabled (rows aren't form controls).
        let required: Vec<&str> = REQUIRED_STATES
            .iter()
            .filter(|&&s| !(sel == "fjui-table__row" && s == ":disabled"))
            .copied()
            .collect();

        for state in &required {
            // Check if the state appears anywhere in this rule's body
            // (as &:state nested or as a separate .fjui-...:state occurrence in the layer)
            let in_body = rule.body.contains(state);
            // Also check in the full layer for a separate rule like ".fjui-btn:hover"
            let separate_rule = format!("{sel}{state}");
            let in_layer = search_content.contains(&separate_rule);

            if !in_body && !in_layer {
                findings.push(Finding {
                    rule: "skin-interaction-states",
                    element_id: None,
                    severity: Severity::Warning,
                    message: format!(
                        "Rule `.{sel}` is missing interaction state `{state}`."
                    ),
                    suggestion: format!(
                        "Add `&{state} {{ ... }}` inside the `.{sel}` rule body."
                    ),
                });
            }
        }
    }

    findings
}

// ── Border-or-shadow elevation rule ──────────────────────────────────────────

/// Check that no `.fjui-*` rule block in `@layer components` declares BOTH a
/// visible border/border-color AND a non-none box-shadow on the same element.
///
/// The elevation discipline (LANG-04) requires:
///   - flat surfaces: border only, no shadow
///   - overlays:      shadow only, no border
///
/// Exceptions (not flagged):
///   - `border: none` or `box-shadow: none` declarations (opt-out of the property)
///   - `box-shadow` that appears only inside a `:focus-visible` nested block
///     (focus rings use box-shadow in some patterns; they are not elevation)
///
/// Returns one `Warning` finding per violating rule block.
pub fn check_skin_border_and_shadow(css: &str) -> Vec<Finding> {
    let layer = extract_components_layer(css);
    let content = if layer.is_empty() { css } else { layer };
    let rules = extract_fjui_rules(content);
    let mut findings = Vec::new();

    for rule in &rules {
        if has_border_and_shadow_violation(&rule.selector, &rule.body) {
            findings.push(Finding {
                rule: "skin-border-or-shadow",
                element_id: None,
                severity: Severity::Warning,
                message: format!(
                    "Rule `.{}` declares both a visible border and a non-none box-shadow — use border OR shadow, never both (LANG-04).",
                    rule.selector
                ),
                suggestion: "Flat surfaces use border only (no box-shadow). Overlays use box-shadow only (border: none). Move box-shadow inside :focus-visible if it is a focus ring.".into(),
            });
        }
    }

    findings
}

/// Returns true if `body` declares both a visible border/border-color AND a
/// non-none box-shadow at the top level of the rule (not inside :focus-visible).
fn has_border_and_shadow_violation(selector: &str, body: &str) -> bool {
    // Build a "top-level-only" view of the rule body by stripping :focus-visible
    // nested blocks. We keep other nested blocks (e.g. :hover) because a hover
    // shadow lift paired with a base border IS a violation.
    let top_level = strip_focus_visible_blocks(body);

    let has_visible_border = rule_has_visible_border(selector, &top_level);
    let has_non_none_shadow = rule_has_non_none_box_shadow(&top_level);

    has_visible_border && has_non_none_shadow
}

/// Strip any `&:focus-visible { ... }` nested blocks from `body`, returning the
/// remainder. This is a simple brace-depth scan — it handles one level of nesting.
fn strip_focus_visible_blocks(body: &str) -> String {
    let mut result = String::with_capacity(body.len());
    let mut pos = 0;
    let bytes = body.as_bytes();
    let len = body.len();

    while pos < len {
        // Look for a :focus-visible token followed (eventually) by `{`
        let rest = &body[pos..];
        if let Some(fv_rel) = rest.find(":focus-visible") {
            let fv_abs = pos + fv_rel;
            // Check if there's a `{` after it (possibly with whitespace)
            let after_fv = &body[fv_abs + ":focus-visible".len()..];
            if let Some(brace_rel) = after_fv.find('{') {
                // Verify nothing unexpected between :focus-visible and the brace
                let between = &after_fv[..brace_rel];
                if between.chars().all(|c| c.is_whitespace()) {
                    // Emit text up to the :focus-visible token
                    result.push_str(&body[pos..fv_abs]);
                    // Skip past the entire nested block
                    let block_start = fv_abs + ":focus-visible".len() + brace_rel + 1;
                    let block_body = &body[block_start..];
                    let mut depth = 1usize;
                    let mut end_rel = block_body.len();
                    for (i, ch) in block_body.char_indices() {
                        match ch {
                            '{' => depth += 1,
                            '}' => {
                                depth -= 1;
                                if depth == 0 {
                                    end_rel = i + 1; // include the closing brace
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    pos = block_start + end_rel;
                    continue;
                }
            }
            // Not a clean :focus-visible { block — emit one char and keep scanning
            result.push(bytes[pos] as char);
            pos += 1;
        } else {
            // No more :focus-visible tokens — emit the rest
            result.push_str(&body[pos..]);
            break;
        }
    }

    result
}

/// Returns true if the rule body contains a visible border declaration (not `none`).
///
/// Considers:
///   - `border: <value>` where value is not `none`
///   - `border-color: <value>` where value is not `none`/`transparent`
///   - `border-left:`, `border-right:`, `border-top:`, `border-bottom:` with non-none values
///   - `border-left-color:` etc. with non-none values
///
/// Note: `border-radius` and `border-width` alone are NOT visible borders.
/// Note: `border-left-width` / `border-top-width` alone are not flagged either.
fn rule_has_visible_border(_selector: &str, body: &str) -> bool {
    for segment in body.split(';') {
        let segment = segment.trim();
        if segment.is_empty() || segment.contains('{') || segment.starts_with('}') {
            continue;
        }
        let Some(colon) = segment.find(':') else { continue };
        let prop = segment[..colon].trim().trim_start_matches('&').trim();
        // Strip pseudo-class prefix (e.g. ":hover color" → "color")
        let prop = prop.trim_start_matches(':').trim();
        let value = segment[colon + 1..].trim();

        let is_border_prop = prop == "border"
            || prop == "border-color"
            || prop == "border-left"
            || prop == "border-right"
            || prop == "border-top"
            || prop == "border-bottom"
            || prop == "border-left-color"
            || prop == "border-right-color"
            || prop == "border-top-color"
            || prop == "border-bottom-color";

        if !is_border_prop {
            continue;
        }

        // Opt-out values — not a visible border
        let v = value.to_lowercase();
        if v == "none" || v == "transparent" || v.is_empty() {
            continue;
        }

        return true;
    }
    false
}

/// Returns true if the rule body contains a non-none `box-shadow` declaration
/// at the TOP LEVEL of the rule (not inside any nested pseudo-class block).
///
/// Only top-level declarations are checked. A `box-shadow` that appears
/// exclusively inside `&:hover {}`, `&:focus-visible {}`, `&:active {}`, etc.
/// is NOT flagged — those are interaction-state overrides, not resting elevation.
/// This means a card can have a base border and add a hover-lift shadow without
/// violating the elevation rule; the resting state is still border-only.
fn rule_has_non_none_box_shadow(body: &str) -> bool {
    for segment in body.split(';') {
        let segment = segment.trim();
        // Skip empty segments, nested block openers, and closing braces.
        // Segments containing '{' are block openers for nested pseudo-class blocks
        // — skip them entirely so only top-level declarations are checked.
        if segment.is_empty() || segment.contains('{') || segment.starts_with('}') {
            continue;
        }
        let Some(colon) = segment.find(':') else { continue };
        let prop = segment[..colon].trim().trim_start_matches('&').trim();
        let prop = prop.trim_start_matches(':').trim();
        let value = segment[colon + 1..].trim();

        if prop != "box-shadow" {
            continue;
        }

        let v = value.to_lowercase();
        if v == "none" || v.is_empty() {
            continue;
        }

        return true;
    }
    false
}

/// Run both raw-literal and interaction-state checks over `css`.
pub fn check_all(css: &str) -> Vec<Finding> {
    let mut findings = check_skin_raw_literals(css);
    findings.extend(check_skin_interaction_states(css));
    findings.extend(check_skin_border_and_shadow(css));
    findings
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Raw-literal tests ─────────────────────────────────────────────────────

    /// A raw hex literal in a color property must be flagged.
    #[test]
    fn raw_hex_in_fjui_rule_returns_warning() {
        let css = "@layer components { .fjui-btn { color: #1a1a1a; } }";
        let findings = check_skin_raw_literals(css);
        assert_eq!(
            findings.len(),
            1,
            "expected 1 warning for raw hex literal, got: {findings:#?}"
        );
        assert_eq!(findings[0].severity, Severity::Warning);
        assert!(
            findings[0].message.contains("fjui-btn"),
            "message should name the selector"
        );
        assert!(
            findings[0].message.contains("#1a1a1a"),
            "message should name the literal"
        );
    }

    /// A token var() reference must not be flagged.
    #[test]
    fn token_var_ref_no_warning() {
        let css = "@layer components { .fjui-btn { color: var(--color-text); } }";
        let findings = check_skin_raw_literals(css);
        assert!(
            findings.is_empty(),
            "var(--token) must not be flagged, got: {findings:#?}"
        );
    }

    /// rgb() as a color value must be flagged.
    #[test]
    fn raw_rgb_in_fjui_rule_returns_warning() {
        let css = "@layer components { .fjui-btn { background-color: rgb(255, 0, 0); } }";
        let findings = check_skin_raw_literals(css);
        assert!(
            !findings.is_empty(),
            "rgb() literal must be flagged"
        );
    }

    /// rgba() as a color value must be flagged.
    #[test]
    fn raw_rgba_in_fjui_rule_returns_warning() {
        let css = "@layer components { .fjui-btn { background: rgba(0,0,0,0.5); } }";
        let findings = check_skin_raw_literals(css);
        assert!(
            !findings.is_empty(),
            "rgba() literal must be flagged"
        );
    }

    /// oklch() used directly (not in var()) must be flagged.
    #[test]
    fn raw_oklch_direct_returns_warning() {
        let css = "@layer components { .fjui-btn { color: oklch(50% 0 0); } }";
        let findings = check_skin_raw_literals(css);
        assert!(
            !findings.is_empty(),
            "oklch() used directly must be flagged"
        );
    }

    /// color-mix() with only var() references must NOT be flagged.
    #[test]
    fn color_mix_with_var_refs_no_warning() {
        let css = "@layer components { .fjui-btn { background: color-mix(in oklab, var(--color-primary) 88%, transparent); } }";
        let findings = check_skin_raw_literals(css);
        assert!(
            findings.is_empty(),
            "color-mix with var() refs must not be flagged, got: {findings:#?}"
        );
    }

    // ── Interaction-state tests ───────────────────────────────────────────────

    /// A complete fjui-btn rule (all four states) must return no warnings.
    #[test]
    fn complete_fjui_btn_no_warnings() {
        let css = "@layer components {
            .fjui-btn {
                color: var(--color-text);
                &:hover { background: var(--color-surface); }
                &:focus-visible { outline: 2px solid var(--color-ring); }
                &:active { opacity: 0.85; }
                &:disabled { opacity: 0.5; }
            }
        }";
        let findings = check_skin_interaction_states(css);
        assert!(
            findings.is_empty(),
            "complete fjui-btn should produce no interaction-state warnings, got: {findings:#?}"
        );
    }

    /// A fjui-btn rule missing :active must be flagged.
    #[test]
    fn fjui_btn_missing_active_returns_warning() {
        let css = "@layer components {
            .fjui-btn {
                color: var(--color-text);
                &:hover { background: var(--color-surface); }
                &:focus-visible { outline: 2px solid var(--color-ring); }
                &:disabled { opacity: 0.5; }
            }
        }";
        let findings = check_skin_interaction_states(css);
        assert_eq!(
            findings.len(),
            1,
            "should find exactly 1 missing :active warning, got: {findings:#?}"
        );
        assert!(
            findings[0].message.contains(":active"),
            "message must name the missing state"
        );
        assert!(
            findings[0].message.contains("fjui-btn"),
            "message must name the selector"
        );
    }

    /// A non-interactive rule (fjui-card) must NOT require interaction states.
    #[test]
    fn non_interactive_fjui_card_no_warnings() {
        let css = "@layer components {
            .fjui-card {
                background: var(--color-card);
                border: 1px solid var(--color-border);
            }
        }";
        let findings = check_skin_interaction_states(css);
        assert!(
            findings.is_empty(),
            "fjui-card is non-interactive and must not require interaction states, got: {findings:#?}"
        );
    }

    /// fjui-badge is non-interactive — no state warnings.
    #[test]
    fn non_interactive_fjui_badge_no_warnings() {
        let css = "@layer components {
            .fjui-badge {
                background: var(--color-surface);
            }
        }";
        let findings = check_skin_interaction_states(css);
        assert!(
            findings.is_empty(),
            "fjui-badge must not require interaction states, got: {findings:#?}"
        );
    }

    /// fjui-table__row requires :hover, :focus-visible, :active but NOT :disabled.
    #[test]
    fn fjui_table_row_exempt_from_disabled() {
        let css = "@layer components {
            .fjui-table__row {
                border-bottom: 1px solid var(--color-border);
                &:hover { background: var(--color-surface); }
                &:focus-visible { outline: 2px solid var(--color-ring); }
                &:active { background: var(--color-surface); }
            }
        }";
        let findings = check_skin_interaction_states(css);
        assert!(
            findings.is_empty(),
            "fjui-table__row without :disabled must not warn (rows exempt from :disabled), got: {findings:#?}"
        );
    }

    // ── LANG-03: tabular-nums in ferro-skin.css ───────────────────────────────

    /// LANG-03: the committed ferro-skin.css must declare font-variant-numeric:tabular-nums
    /// in both the numeric table-cell rule and the stat-value rule.
    ///
    /// Reads the CSS from disk via CARGO_MANIFEST_DIR — the same path the
    /// render pipeline would serve — so the test breaks immediately if either
    /// rule is removed or misspelled.
    #[test]
    fn ferro_skin_css_contains_tabular_nums_in_numeric_cell() {
        let css = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ferro-skin.css"
        ))
        .expect("assets/ferro-skin.css must exist and be readable");
        assert!(
            css.contains("fjui-table__cell--numeric"),
            "ferro-skin.css must contain the .fjui-table__cell--numeric rule (LANG-03)"
        );
        // Locate the numeric-cell rule block and verify tabular-nums is inside it.
        // Strategy: find the rule selector, then scan forward until the closing brace.
        let marker = "fjui-table__cell--numeric";
        let start = css.find(marker).expect("fjui-table__cell--numeric selector not found");
        // Find the opening brace after the selector
        let brace = css[start..].find('{').expect("opening brace for fjui-table__cell--numeric not found");
        let body_start = start + brace;
        // Find the matching closing brace (depth-tracked)
        let mut depth = 0usize;
        let mut body_end = body_start;
        for (i, ch) in css[body_start..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        body_end = body_start + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        let numeric_cell_block = &css[body_start..=body_end];
        assert!(
            numeric_cell_block.contains("tabular-nums"),
            "fjui-table__cell--numeric rule must declare font-variant-numeric:tabular-nums (LANG-03); block: {numeric_cell_block}"
        );
    }

    /// LANG-03: ferro-skin.css must declare tabular-nums on the stat value element.
    #[test]
    fn ferro_skin_css_contains_tabular_nums_in_stat_value() {
        let css = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ferro-skin.css"
        ))
        .expect("assets/ferro-skin.css must exist and be readable");
        assert!(
            css.contains("fjui-stat-card__value"),
            "ferro-skin.css must contain the .fjui-stat-card__value rule (LANG-03)"
        );
        let marker = "fjui-stat-card__value";
        let start = css.find(marker).expect("fjui-stat-card__value selector not found");
        let brace = css[start..].find('{').expect("opening brace for fjui-stat-card__value not found");
        let body_start = start + brace;
        let mut depth = 0usize;
        let mut body_end = body_start;
        for (i, ch) in css[body_start..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        body_end = body_start + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        let stat_block = &css[body_start..=body_end];
        assert!(
            stat_block.contains("tabular-nums"),
            "fjui-stat-card__value rule must declare font-variant-numeric:tabular-nums (LANG-03); block: {stat_block}"
        );
    }

    // ── check_all ─────────────────────────────────────────────────────────────

    /// check_all combines both checks.
    #[test]
    fn check_all_combines_both_checks() {
        // Has a raw literal AND missing interaction state
        let css = "@layer components {
            .fjui-btn {
                color: #ff0000;
                &:hover { background: var(--color-surface); }
                &:focus-visible { outline: 2px solid var(--color-ring); }
                &:active { opacity: 0.85; }
                &:disabled { opacity: 0.5; }
            }
        }";
        let findings = check_all(css);
        // Should have at least the raw literal finding
        assert!(
            findings.iter().any(|f| f.rule == "skin-raw-literals"),
            "check_all must include raw-literal findings"
        );
    }

    // ── LANG-04: border-or-shadow elevation rule ──────────────────────────────

    /// A rule with BOTH a visible border AND a non-none box-shadow must be flagged.
    #[test]
    fn border_and_shadow_on_same_rule_returns_warning() {
        let css = "@layer components {
            .fjui-card {
                border: 1px solid var(--color-border);
                box-shadow: var(--shadow-md);
            }
        }";
        let findings = check_skin_border_and_shadow(css);
        assert_eq!(
            findings.len(),
            1,
            "expected 1 border-or-shadow violation, got: {findings:#?}"
        );
        assert_eq!(findings[0].severity, Severity::Warning);
        assert!(
            findings[0].message.contains("fjui-card"),
            "message must name the selector"
        );
        assert_eq!(findings[0].rule, "skin-border-or-shadow");
    }

    /// A rule with border only (no box-shadow) must NOT be flagged.
    #[test]
    fn border_only_no_shadow_no_warning() {
        let css = "@layer components {
            .fjui-card {
                border: 1px solid var(--color-border);
            }
        }";
        let findings = check_skin_border_and_shadow(css);
        assert!(
            findings.is_empty(),
            "border only (no shadow) must not be flagged, got: {findings:#?}"
        );
    }

    /// A rule with box-shadow only (no border) must NOT be flagged.
    #[test]
    fn shadow_only_no_border_no_warning() {
        let css = "@layer components {
            .fjui-menu {
                box-shadow: var(--shadow-md);
            }
        }";
        let findings = check_skin_border_and_shadow(css);
        assert!(
            findings.is_empty(),
            "shadow only (no border) must not be flagged, got: {findings:#?}"
        );
    }

    /// `border: none` is an opt-out — NOT a visible border, must NOT be flagged.
    #[test]
    fn border_none_with_shadow_no_warning() {
        let css = "@layer components {
            .fjui-btn--primary {
                border: none;
                box-shadow: var(--shadow-sm);
            }
        }";
        let findings = check_skin_border_and_shadow(css);
        assert!(
            findings.is_empty(),
            "border:none with shadow must not be flagged (border is opted out), got: {findings:#?}"
        );
    }

    /// `box-shadow` inside `:focus-visible` only — NOT a violation (focus ring exemption).
    #[test]
    fn shadow_only_in_focus_visible_no_warning() {
        let css = "@layer components {
            .fjui-input {
                border: 1px solid var(--color-border);
                &:focus-visible {
                    box-shadow: 0 0 0 2px var(--color-ring);
                }
            }
        }";
        let findings = check_skin_border_and_shadow(css);
        assert!(
            findings.is_empty(),
            "box-shadow inside :focus-visible with a base border must not be flagged (focus-ring exemption), got: {findings:#?}"
        );
    }

    /// A hover lift (box-shadow only in :hover) with a base border must NOT be flagged.
    ///
    /// The rule checks only TOP-LEVEL declarations — a hover-lift shadow that
    /// appears exclusively inside `&:hover {}` is an interaction-state override,
    /// not a resting elevation. The resting state is border-only (conforming).
    /// This matches the real ferro-skin.css kanban card pattern.
    #[test]
    fn hover_shadow_lift_with_base_border_no_warning() {
        let css = "@layer components {
            .fjui-card {
                border: 1px solid var(--color-border);
                &:hover {
                    box-shadow: var(--shadow-md);
                }
            }
        }";
        let findings = check_skin_border_and_shadow(css);
        assert!(
            findings.is_empty(),
            "hover-lift shadow (only in :hover) + base border must NOT be flagged — resting state is border-only, got: {findings:#?}"
        );
    }

    /// A top-level box-shadow with a visible border IS a violation (both at rest).
    #[test]
    fn top_level_shadow_with_border_returns_warning() {
        let css = "@layer components {
            .fjui-card {
                border: 1px solid var(--color-border);
                box-shadow: var(--shadow-md);
            }
        }";
        let findings = check_skin_border_and_shadow(css);
        assert_eq!(
            findings.len(),
            1,
            "top-level border AND top-level box-shadow must be flagged (LANG-04), got: {findings:#?}"
        );
        assert_eq!(findings[0].rule, "skin-border-or-shadow");
    }

    /// The real ferro-skin.css must pass the border-or-shadow rule with zero violations.
    #[test]
    fn ferro_skin_css_passes_border_or_shadow_rule() {
        let css = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ferro-skin.css"
        ))
        .expect("assets/ferro-skin.css must exist and be readable");
        let findings = check_skin_border_and_shadow(&css);
        assert!(
            findings.is_empty(),
            "ferro-skin.css must have zero border-or-shadow violations (LANG-04); found: {findings:#?}"
        );
    }

    // ── DX-02: token slot docs coverage ──────────────────────────────────────

    /// DX-02: every token slot in ferro-theme's ALL_TOKENS registry must appear
    /// in the docs/tokens.md file shipped with ferro-json-ui.
    ///
    /// This test breaks as soon as a new slot is added to ALL_TOKENS without a
    /// corresponding entry in the documentation — preventing silent drift.
    #[test]
    fn all_tokens_slots_covered_in_tokens_md() {
        use ferro_theme::token::ALL_TOKENS;

        let tokens_md = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/docs/tokens.md"
        ))
        .expect("docs/tokens.md must exist and be readable (DX-02)");

        let mut missing: Vec<&str> = Vec::new();
        for slot in ALL_TOKENS {
            if !tokens_md.contains(slot) {
                missing.push(slot);
            }
        }

        assert!(
            missing.is_empty(),
            "docs/tokens.md is missing {} token slot(s) — add them to prevent DX-02 drift: {missing:#?}",
            missing.len()
        );
    }
}
