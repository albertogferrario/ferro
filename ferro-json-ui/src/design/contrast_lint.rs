//! Token contrast lint: parse tokens.css and check WCAG contrast ratios.
//!
//! Parses `:root` and `.dark` blocks from a `tokens.css` file, converts
//! oklch color values to WCAG relative luminance, and checks that token
//! pairs meet the required contrast floors in both light and dark modes.
//!
//! Text token pairs must achieve ≥4.5:1 (WCAG AA for normal text).
//! UI/non-text pairs (focus ring) must achieve ≥3:1 (WCAG AA for UI components).

use super::types::{Finding, Severity};

// ── Pure color-math functions ─────────────────────────────────────────────────

/// Convert an OKLch color to WCAG relative luminance.
///
/// Implements the full chain: OKLch → OKLab → LMS-cubed → linear sRGB → Y.
/// Matrix constants from the OKLch spec (Björn Ottosson, 2020).
///
/// `l_percent`: L channel in percent (0–100).
/// `c`: chroma (0–0.4 typical range).
/// `h_deg`: hue angle in degrees (0–360).
pub fn oklch_to_relative_luminance(l_percent: f64, c: f64, h_deg: f64) -> f64 {
    let l = l_percent / 100.0;
    let a = c * (h_deg * std::f64::consts::PI / 180.0).cos();
    let b = c * (h_deg * std::f64::consts::PI / 180.0).sin();

    // OKLab → LMS³ (cube roots of cone responses)
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;

    // Cube to recover actual LMS
    let lms = [l_.powi(3), m_.powi(3), s_.powi(3)];

    // LMS → linear sRGB
    let r = 4.0767416621 * lms[0] - 3.3077115913 * lms[1] + 0.2309699292 * lms[2];
    let g = -1.2684380046 * lms[0] + 2.6097574011 * lms[1] - 0.3413193965 * lms[2];
    let b_lin = -0.0041960863 * lms[0] - 0.7034186147 * lms[1] + 1.7076147010 * lms[2];

    // Linearize (gamma-expand) sRGB channel and convert to luminance Y
    fn to_y(c: f64) -> f64 {
        let c = c.clamp(0.0, 1.0);
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    0.2126 * to_y(r) + 0.7152 * to_y(g) + 0.0722 * to_y(b_lin)
}

/// Compute WCAG contrast ratio from two relative luminance values.
///
/// Returns a value in [1.0, 21.0]. The formula is (L_lighter + 0.05) / (L_darker + 0.05).
pub fn contrast_ratio(y1: f64, y2: f64) -> f64 {
    let (lighter, darker) = if y1 > y2 { (y1, y2) } else { (y2, y1) };
    (lighter + 0.05) / (darker + 0.05)
}

// ── CSS parser ────────────────────────────────────────────────────────────────

/// Extract CSS custom-property declarations from a block bounded by `{` and `}`.
///
/// Returns a map of `--property-name` → value string (trimmed, semicolon stripped).
fn extract_declarations(block: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for line in block.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("--") {
            if let Some(colon_pos) = rest.find(':') {
                let name = format!("--{}", &rest[..colon_pos].trim());
                let value = rest[colon_pos + 1..].trim().trim_end_matches(';').trim();
                map.insert(name, value.to_string());
            }
        }
    }
    map
}

/// Extract the content of the first block matching `selector {` in `css`.
///
/// Returns the content between the first `{` after the selector and the
/// matching `}`, tracking brace depth. Returns `None` if not found.
fn extract_block<'a>(css: &'a str, selector: &str) -> Option<&'a str> {
    let start = css.find(selector)?;
    let after = &css[start..];
    let brace_start = after.find('{')? + 1;
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
    Some(&content[..end])
}

/// Parse oklch value string: `oklch(L% C H)` or `oklch(L C H)`.
///
/// Returns `(l_percent, c, h_deg)` or `None` if not parseable.
fn parse_oklch(value: &str) -> Option<(f64, f64, f64)> {
    let value = value.trim();
    let inner = value.strip_prefix("oklch(")?.strip_suffix(')')?;
    let parts: Vec<&str> = inner.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    // L may be a percentage ("60%") or a bare number ("0.6")
    let l_str = parts[0];
    let l: f64 = if let Some(p) = l_str.strip_suffix('%') {
        p.parse().ok()?
    } else {
        // bare fraction (0.0–1.0) → convert to percent
        let v: f64 = l_str.parse().ok()?;
        v * 100.0
    };
    let c: f64 = parts[1].parse().ok()?;
    let h_str = parts[2].trim_end_matches("deg");
    let h: f64 = h_str.parse().ok()?;
    Some((l, c, h))
}

// ── Contrast-check pairs (from UI-SPEC Design:lint Rules) ────────────────────

/// A pair of tokens to check, with the contrast floor and a human label.
struct ContrastPair {
    fg: &'static str,
    bg: &'static str,
    floor: f64,
    label: &'static str,
}

const CONTRAST_PAIRS: &[ContrastPair] = &[
    ContrastPair {
        fg: "--color-text",
        bg: "--color-background",
        floor: 4.5,
        label: "--color-text / --color-background",
    },
    ContrastPair {
        fg: "--color-text",
        bg: "--color-card",
        floor: 4.5,
        label: "--color-text / --color-card",
    },
    ContrastPair {
        fg: "--color-primary-foreground",
        bg: "--color-primary",
        floor: 4.5,
        label: "--color-primary-foreground / --color-primary",
    },
    ContrastPair {
        fg: "--color-ring",
        bg: "--color-background",
        floor: 3.0,
        label: "--color-ring / --color-background",
    },
    ContrastPair {
        fg: "--color-ring",
        bg: "--color-card",
        floor: 3.0,
        label: "--color-ring / --color-card",
    },
];

// ── Public check function ─────────────────────────────────────────────────────

/// Check token contrast ratios in both `:root` (light) and `.dark` modes.
///
/// Parses the `:root` and `.dark` blocks from `css`. For dark mode, overlays
/// `.dark` declarations on top of `:root` (inheriting undeclared tokens).
///
/// Checks the five required pairs (from CONTRAST_PAIRS). Text pairs must
/// achieve ≥4.5:1; ring/UI pairs must achieve ≥3:1. Returns one `Warning`
/// finding per violation, naming the mode.
///
/// `--color-text-muted` is advisory only — not included in the hard gate.
pub fn check_token_contrast(css: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Parse :root block
    let root_block = extract_block(css, ":root").unwrap_or("");
    let root_tokens = extract_declarations(root_block);

    // Parse .dark block (inherits from :root)
    let dark_block = extract_block(css, ".dark").unwrap_or("");
    let dark_override = extract_declarations(dark_block);
    let mut dark_tokens = root_tokens.clone();
    for (k, v) in &dark_override {
        dark_tokens.insert(k.clone(), v.clone());
    }

    let modes = [("light", &root_tokens), ("dark", &dark_tokens)];

    for (mode_name, tokens) in &modes {
        for pair in CONTRAST_PAIRS {
            let fg_val = match tokens.get(pair.fg) {
                Some(v) => v,
                None => continue, // token not declared — skip silently
            };
            let bg_val = match tokens.get(pair.bg) {
                Some(v) => v,
                None => continue,
            };

            let fg_lum = match parse_oklch(fg_val) {
                Some((l, c, h)) => oklch_to_relative_luminance(l, c, h),
                None => continue, // non-oklch value — skip
            };
            let bg_lum = match parse_oklch(bg_val) {
                Some((l, c, h)) => oklch_to_relative_luminance(l, c, h),
                None => continue,
            };

            let ratio = contrast_ratio(fg_lum, bg_lum);
            if ratio < pair.floor {
                findings.push(Finding {
                    rule: "contrast-lint",
                    element_id: None,
                    severity: Severity::Warning,
                    message: format!(
                        "{} in {} mode is {:.2}:1, below the {}:1 floor",
                        pair.label, mode_name, ratio, pair.floor
                    ),
                    suggestion: format!(
                        "Adjust {} or {} so the pair meets {}:1 in {} mode.",
                        pair.fg, pair.bg, pair.floor, mode_name
                    ),
                });
            }
        }
    }

    findings
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // oklch(100% 0 0) = white → luminance ≈ 1.0
    #[test]
    fn white_oklch_luminance_is_one() {
        let y = oklch_to_relative_luminance(100.0, 0.0, 0.0);
        assert!(
            (y - 1.0).abs() < 1e-3,
            "white oklch(100% 0 0) should have luminance ≈ 1.0, got {y}"
        );
    }

    // oklch(0% 0 0) = black → luminance ≈ 0.0
    #[test]
    fn black_oklch_luminance_is_zero() {
        let y = oklch_to_relative_luminance(0.0, 0.0, 0.0);
        assert!(
            y.abs() < 1e-3,
            "black oklch(0% 0 0) should have luminance ≈ 0.0, got {y}"
        );
    }

    // contrast_ratio(1.0, 0.0) = (1.0 + 0.05) / (0.0 + 0.05) = 21.0
    #[test]
    fn max_contrast_ratio_is_21() {
        let r = contrast_ratio(1.0, 0.0);
        assert!(
            (r - 21.0).abs() < 1e-2,
            "contrast_ratio(1.0, 0.0) should equal 21.0, got {r}"
        );
    }

    // Low-contrast fixture: both colors close in lightness → below 4.5:1
    #[test]
    fn low_contrast_text_pair_returns_warning() {
        let css = ":root { --color-text: oklch(60% 0 0); --color-background: oklch(55% 0 0); }";
        let findings = check_token_contrast(css);
        assert!(
            !findings.is_empty(),
            "low-contrast text pair should return at least one Warning"
        );
        assert!(
            findings.iter().any(|f| f.severity == Severity::Warning),
            "findings must include at least one Warning"
        );
    }

    // Dark-mode-only failure: .dark block overrides a passing :root pair to a failing one
    #[test]
    fn dark_only_failure_names_dark_mode() {
        let css = ":root { \
            --color-text: oklch(10% 0 0); \
            --color-background: oklch(100% 0 0); \
        } \
        .dark { \
            --color-text: oklch(55% 0 0); \
            --color-background: oklch(50% 0 0); \
        }";
        let findings = check_token_contrast(css);
        assert!(
            findings.iter().any(|f| f.message.contains("dark")),
            "a dark-mode violation should name 'dark' in the message; got: {:#?}",
            findings.iter().map(|f| &f.message).collect::<Vec<_>>()
        );
    }

    // Real gestiscilo v2 tokens.css must pass all checks (zero findings).
    // The fixture is embedded here rather than via include_str! because the
    // gestiscilo repo is a sibling (not a Cargo workspace dependency).
    // Values verified against the actual tokens.css at:
    //   gestiscilo-it/app/themes/gestiscilo/tokens.css (Plan 246-01 output)
    #[test]
    fn gestiscilo_v2_tokens_pass_all_checks() {
        // Exact v2 token values from the gestiscilo tokens.css (Plan 246-01).
        // Only the pairs checked by this lint are included:
        //   --color-text / --color-background  → oklch(18%/100%) => very high contrast
        //   --color-text / --color-card        → oklch(18%/100%) => very high contrast
        //   --color-primary-foreground / --color-primary → oklch(100%/18%) => very high
        //   --color-ring / --color-background  → oklch(60%/100%) => ~4.5:1 in light
        //   --color-ring / --color-card        → oklch(60%/100%) => ~4.5:1 in light
        // Dark mode:
        //   --color-text oklch(95%) vs --color-background oklch(16%) => very high
        //   --color-ring oklch(72%) vs --color-background oklch(16%) => ≥3:1
        let css = r#":root {
  --color-background: oklch(100% 0 0);
  --color-surface: oklch(97% 0.005 270);
  --color-card: oklch(100% 0 0);
  --color-border: oklch(88% 0.008 270);
  --color-text: oklch(18% 0.01 270);
  --color-text-muted: oklch(52% 0.01 270);
  --color-primary: oklch(18% 0 0);
  --color-primary-foreground: oklch(100% 0 0);
  --color-secondary: oklch(94% 0 0);
  --color-secondary-foreground: oklch(18% 0 0);
  --color-accent: oklch(18% 0 0);
  --color-destructive: oklch(52% 0.22 25);
  --color-success: oklch(52% 0.18 145);
  --color-warning: oklch(68% 0.18 75);
  --radius-sm: 0.375rem;
  --radius-md: 0.625rem;
  --radius-lg: 0.75rem;
  --radius-full: 9999px;
  --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.05);
  --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.08);
  --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.08);
  --font-sans: 'Geist', ui-sans-serif, system-ui, -apple-system, sans-serif;
  --font-mono: 'Geist Mono', ui-monospace, monospace;
  --spacing: 0.25rem;
  --motion-duration-fast: 120ms;
  --motion-duration-base: 220ms;
  --motion-duration-slow: 320ms;
  --motion-ease: cubic-bezier(0.2, 0, 0.38, 0.9);
  --color-ring: oklch(60% 0.01 270);
  --font-display: var(--font-sans);
  --text-display-size: 1.75rem;
  --text-display-weight: 600;
  --text-section-size: 0.9375rem;
  --text-section-weight: 600;
  --text-body-size: 0.875rem;
  --text-body-weight: 400;
  --text-meta-size: 0.8125rem;
  --text-meta-weight: 400;
  --text-micro-size: 0.75rem;
  --text-micro-weight: 500;
}
.dark {
  --color-background: oklch(16% 0.01 270);
  --color-surface: oklch(19% 0.01 270);
  --color-card: oklch(22% 0.01 270);
  --color-border: oklch(30% 0 0);
  --color-text: oklch(95% 0 0);
  --color-text-muted: oklch(60% 0 0);
  --color-primary: oklch(95% 0 0);
  --color-primary-foreground: oklch(18% 0 0);
  --color-secondary: oklch(25% 0 0);
  --color-secondary-foreground: oklch(95% 0 0);
  --color-accent: oklch(95% 0 0);
  --color-destructive: oklch(60% 0.22 25);
  --color-success: oklch(60% 0.18 145);
  --color-warning: oklch(65% 0.18 75);
  --color-ring: oklch(72% 0.01 270);
}"#;
        let findings = check_token_contrast(css);
        assert!(
            findings.is_empty(),
            "gestiscilo v2 tokens.css should pass all contrast checks, got: {:#?}",
            findings.iter().map(|f| &f.message).collect::<Vec<_>>()
        );
    }
}
