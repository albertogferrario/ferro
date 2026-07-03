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
