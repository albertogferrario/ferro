//! Intent inference heuristic for specs that do not declare `design.intent`.

use crate::spec::Spec;

/// Infer the dominant intent from spec structure when `design.intent` is absent.
///
/// Signal priority (highest to lowest):
/// 1. Any `KanbanBoard` element → `"process"`
/// 2. Any `Form` element → `"collect"`
/// 3. Any `DataTable` or `Table` element → `"browse"`
/// 4. Two or more `StatCard` elements → `"summarize"`
/// 5. No clear signal → `None`
///
/// Returns the inferred intent label or `None` if no signal is found.
pub(super) fn infer_intent(spec: &Spec) -> Option<&'static str> {
    let types: Vec<&str> = spec
        .elements
        .values()
        .map(|el| el.type_name.as_str())
        .collect();

    if types.iter().any(|t| *t == "KanbanBoard") {
        return Some("process");
    }
    let form_count = types.iter().filter(|t| **t == "Form").count();
    if form_count >= 1 {
        return Some("collect");
    }
    if types.iter().any(|t| *t == "DataTable" || *t == "Table") {
        return Some("browse");
    }
    let stat_count = types.iter().filter(|t| **t == "StatCard").count();
    if stat_count >= 2 {
        return Some("summarize");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::Spec;

    fn spec_with(element_type: &str) -> Spec {
        Spec::from_json(&format!(
            r#"{{"$schema":"ferro-json-ui/v2","root":"r","elements":{{"r":{{"type":"{element_type}"}}}}}}"#
        ))
        .unwrap()
    }

    fn spec_with_two(type_a: &str, type_b: &str) -> Spec {
        Spec::from_json(&format!(
            r#"{{"$schema":"ferro-json-ui/v2","root":"r","elements":{{"r":{{"type":"{type_a}","children":["b"]}},"b":{{"type":"{type_b}"}}}}}}"#
        ))
        .unwrap()
    }

    #[test]
    fn infer_kanban_board_is_process() {
        let spec = spec_with("KanbanBoard");
        assert_eq!(infer_intent(&spec), Some("process"));
    }

    #[test]
    fn infer_form_is_collect() {
        let spec = spec_with("Form");
        assert_eq!(infer_intent(&spec), Some("collect"));
    }

    #[test]
    fn infer_data_table_is_browse() {
        let spec = spec_with("DataTable");
        assert_eq!(infer_intent(&spec), Some("browse"));
    }

    #[test]
    fn infer_table_is_browse() {
        let spec = spec_with("Table");
        assert_eq!(infer_intent(&spec), Some("browse"));
    }

    #[test]
    fn infer_two_stat_cards_is_summarize() {
        let spec = spec_with_two("StatCard", "StatCard");
        assert_eq!(infer_intent(&spec), Some("summarize"));
    }

    #[test]
    fn infer_text_only_is_none() {
        let spec = spec_with("Text");
        assert_eq!(infer_intent(&spec), None);
    }

    #[test]
    fn infer_one_stat_card_is_none() {
        // Single StatCard below threshold — no inference.
        let spec = spec_with("StatCard");
        assert_eq!(infer_intent(&spec), None);
    }
}
