/// Validates that a name is a safe Rust identifier for use as a file/module name.
///
/// Accepts only names that start with a letter or underscore, followed by alphanumeric
/// characters or underscores. Rejects path separators, whitespace, and any other chars.
pub(crate) fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();

    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }

    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Converts a PascalCase or mixed-case name to snake_case.
///
/// "OrderItem" -> "order_item", "userId" -> "user_id".
pub(crate) fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_valid_identifier_rejects_path_traversal() {
        assert!(!is_valid_identifier("../../etc/passwd"));
    }

    #[test]
    fn is_valid_identifier_accepts_simple_name() {
        assert!(is_valid_identifier("order_item"));
    }

    #[test]
    fn is_valid_identifier_rejects_rust_code() {
        assert!(!is_valid_identifier("mod foo; use std"));
    }

    #[test]
    fn is_valid_identifier_rejects_path_separator() {
        assert!(!is_valid_identifier("foo/bar"));
        assert!(!is_valid_identifier("foo\\bar"));
    }

    #[test]
    fn is_valid_identifier_rejects_empty() {
        assert!(!is_valid_identifier(""));
    }

    #[test]
    fn to_snake_case_converts_pascal_case() {
        assert_eq!(to_snake_case("OrderItem"), "order_item");
    }

    #[test]
    fn to_snake_case_passes_through_snake() {
        assert_eq!(to_snake_case("order_item"), "order_item");
    }

    #[test]
    fn to_snake_case_single_uppercase() {
        assert_eq!(to_snake_case("Order"), "order");
    }
}
