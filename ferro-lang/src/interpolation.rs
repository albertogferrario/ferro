/// Replace `:key` placeholders in a template with parameter values.
///
/// Stub implementation — full version in Task 3.
pub fn interpolate(template: &str, params: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in params {
        result = result.replace(&format!(":{}", key), value);
    }
    result
}
