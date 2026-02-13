/// Select the correct plural form from a pipe-separated value.
///
/// Stub implementation — full version in Task 3.
pub fn select_plural_form(value: &str, count: i64) -> String {
    let forms: Vec<&str> = value.split('|').collect();
    match forms.len() {
        0 | 1 => value.to_string(),
        _ => {
            if count == 1 {
                forms[0].trim().to_string()
            } else {
                forms.last().unwrap().trim().to_string()
            }
        }
    }
}
