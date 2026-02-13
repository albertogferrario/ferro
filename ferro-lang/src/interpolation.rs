/// Replace `:key` placeholders in a template with parameter values.
///
/// Supports three case variants per parameter:
/// - `:name` — value as-is
/// - `:Name` — first character uppercased (ucfirst)
/// - `:NAME` — entire value uppercased
///
/// Parameters are processed longest-key-first to avoid partial replacement
/// (e.g., `:username` is replaced before `:user`).
///
/// Missing placeholders are left as-is with a warning logged.
pub fn interpolate(template: &str, params: &[(&str, &str)]) -> String {
    if params.is_empty() {
        return template.to_string();
    }

    // Sort by key length descending to avoid partial replacement.
    let mut sorted: Vec<(&str, &str)> = params.to_vec();
    sorted.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    let mut result = template.to_string();

    for (key, value) in &sorted {
        // :KEY — full uppercase
        let upper_placeholder = format!(":{}", key.to_uppercase());
        if result.contains(&upper_placeholder) {
            result = result.replace(&upper_placeholder, &value.to_uppercase());
        }

        // :Key — ucfirst
        let ucfirst_placeholder = format!(":{}", ucfirst(key));
        if result.contains(&ucfirst_placeholder) {
            result = result.replace(&ucfirst_placeholder, &ucfirst(value));
        }

        // :key — as-is
        let placeholder = format!(":{}", key);
        if result.contains(&placeholder) {
            result = result.replace(&placeholder, value);
        }
    }

    result
}

/// Uppercase the first character of a string.
fn ucfirst(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => {
            let upper: String = c.to_uppercase().collect();
            format!("{}{}", upper, chars.as_str())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_replacement() {
        let result = interpolate("Hello :name!", &[("name", "world")]);
        assert_eq!(result, "Hello world!");
    }

    #[test]
    fn multiple_params() {
        let result = interpolate(
            "The :attribute must be at least :min characters.",
            &[("attribute", "password"), ("min", "8")],
        );
        assert_eq!(result, "The password must be at least 8 characters.");
    }

    #[test]
    fn ucfirst_variant() {
        let result = interpolate("Hello :Name!", &[("name", "john")]);
        assert_eq!(result, "Hello John!");
    }

    #[test]
    fn uppercase_variant() {
        let result = interpolate("Hello :NAME!", &[("name", "john")]);
        assert_eq!(result, "Hello JOHN!");
    }

    #[test]
    fn all_three_variants() {
        let result = interpolate(":name :Name :NAME", &[("name", "alice")]);
        assert_eq!(result, "alice Alice ALICE");
    }

    #[test]
    fn missing_param_leaves_placeholder() {
        let result = interpolate("Hello :name!", &[]);
        assert_eq!(result, "Hello :name!");
    }

    #[test]
    fn empty_params_returns_template() {
        let result = interpolate("No placeholders here", &[]);
        assert_eq!(result, "No placeholders here");
    }

    #[test]
    fn longer_key_first() {
        // :username should be replaced before :user
        let result = interpolate(":user and :username", &[("user", "A"), ("username", "B")]);
        assert_eq!(result, "A and B");
    }
}
