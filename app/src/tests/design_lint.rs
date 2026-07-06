//! D-17 lint-clean gate: every view under `app/src/views/*.json` must lint clean
//! (zero findings) with `design.intent` declared.

#[cfg(test)]
mod tests {
    use ferro_json_ui::design::lint;
    use ferro_json_ui::spec::Spec;

    #[test]
    fn app_views_lint_clean() {
        let views_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/views");
        let entries = std::fs::read_dir(views_dir).expect("app/src/views must exist");
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
            let spec = Spec::from_json(&content)
                .unwrap_or_else(|e| panic!("parse {}: {e:?}", path.display()));
            let findings = lint(&spec);
            assert!(
                findings.is_empty(),
                "{}: {} finding(s)\n{:#?}",
                path.display(),
                findings.len(),
                findings
            );
        }
    }
}
