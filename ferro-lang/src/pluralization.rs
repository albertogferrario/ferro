/// Select the correct plural form from a pipe-separated value.
///
/// Supports several patterns:
/// - Single form: returned as-is
/// - Two forms: `"one|other"` — count == 1 selects first, else second
/// - Three+ forms with explicit syntax:
///   - `{N}` matches exact count
///   - `[N,M]` matches inclusive range N..=M
///   - `[N,*]` matches N and above
///   - Falls back to: count == 1 selects first, else last
pub fn select_plural_form(value: &str, count: i64) -> String {
    let forms: Vec<&str> = value.split('|').collect();

    match forms.len() {
        0 => value.to_string(),
        1 => forms[0].trim().to_string(),
        2 => {
            if count == 1 {
                forms[0].trim().to_string()
            } else {
                forms[1].trim().to_string()
            }
        }
        _ => {
            // Try explicit syntax first.
            for form in &forms {
                let trimmed = form.trim();
                if let Some(text) = match_explicit(trimmed, count) {
                    return text;
                }
            }

            // No explicit match — simple fallback.
            if count == 1 {
                strip_prefix_syntax(forms[0].trim()).to_string()
            } else {
                strip_prefix_syntax(forms.last().unwrap().trim()).to_string()
            }
        }
    }
}

/// Try to match a form with explicit `{N}` or `[N,M]` / `[N,*]` prefix.
///
/// Returns the text portion (after the prefix) if the count matches.
fn match_explicit(form: &str, count: i64) -> Option<String> {
    let trimmed = form.trim();

    // {N} exact match
    if trimmed.starts_with('{') {
        if let Some(close) = trimmed.find('}') {
            let num_str = &trimmed[1..close];
            if let Ok(n) = num_str.trim().parse::<i64>() {
                if n == count {
                    let text = trimmed[close + 1..].trim();
                    // If text starts with a space, trim it
                    return Some(text.to_string());
                }
            }
            return None;
        }
    }

    // [N,M] or [N,*] range match
    if trimmed.starts_with('[') {
        if let Some(close) = trimmed.find(']') {
            let range_str = &trimmed[1..close];
            let parts: Vec<&str> = range_str.split(',').collect();
            if parts.len() == 2 {
                let low = parts[0].trim().parse::<i64>().ok();
                let high_str = parts[1].trim();

                if let Some(low) = low {
                    let matches = if high_str == "*" {
                        count >= low
                    } else if let Ok(high) = high_str.parse::<i64>() {
                        count >= low && count <= high
                    } else {
                        false
                    };

                    if matches {
                        let text = trimmed[close + 1..].trim();
                        return Some(text.to_string());
                    }
                }
            }
            return None;
        }
    }

    None
}

/// Strip any `{N}` or `[N,M]` prefix from a form, returning just the text.
fn strip_prefix_syntax(form: &str) -> &str {
    let trimmed = form.trim();

    if trimmed.starts_with('{') {
        if let Some(close) = trimmed.find('}') {
            return trimmed[close + 1..].trim();
        }
    }

    if trimmed.starts_with('[') {
        if let Some(close) = trimmed.find(']') {
            return trimmed[close + 1..].trim();
        }
    }

    trimmed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_form_returns_as_is() {
        assert_eq!(select_plural_form("item", 1), "item");
        assert_eq!(select_plural_form("item", 5), "item");
    }

    #[test]
    fn two_forms_count_one() {
        assert_eq!(select_plural_form("One item|:count items", 1), "One item");
    }

    #[test]
    fn two_forms_count_many() {
        assert_eq!(
            select_plural_form("One item|:count items", 2),
            ":count items"
        );
    }

    #[test]
    fn two_forms_count_zero() {
        // Zero selects the second form (not one).
        assert_eq!(
            select_plural_form("One item|:count items", 0),
            ":count items"
        );
    }

    #[test]
    fn explicit_exact_match() {
        let value = "{0} No items|{1} One item|[2,*] :count items";
        assert_eq!(select_plural_form(value, 0), "No items");
        assert_eq!(select_plural_form(value, 1), "One item");
        assert_eq!(select_plural_form(value, 5), ":count items");
    }

    #[test]
    fn explicit_range_match() {
        let value = "{0} none|[1,5] a few|[6,*] many";
        assert_eq!(select_plural_form(value, 0), "none");
        assert_eq!(select_plural_form(value, 3), "a few");
        assert_eq!(select_plural_form(value, 6), "many");
        assert_eq!(select_plural_form(value, 100), "many");
    }

    #[test]
    fn fallback_when_no_range_matches() {
        // No explicit syntax, 3 forms → count==1 gets first, else last.
        let value = "zero|one|many";
        assert_eq!(select_plural_form(value, 1), "zero");
        assert_eq!(select_plural_form(value, 5), "many");
    }

    #[test]
    fn whitespace_trimmed() {
        assert_eq!(
            select_plural_form(" One item | :count items ", 1),
            "One item"
        );
        assert_eq!(
            select_plural_form(" One item | :count items ", 2),
            ":count items"
        );
    }
}
