//! Static design-rule registry — batch A (Plans 02–03).
use super::types::{DesignRule, Finding, Severity};
use crate::spec::Spec;

/// The static rule registry. Iterated by [`super::lint`] and [`super::rules`].
pub(super) static RULE_REGISTRY: &[DesignRule] = &[
    DesignRule {
        id: "page-header",
        title: "Dashboard pages start with a PageHeader",
        rationale: "A PageHeader gives every app page a consistent title, breadcrumb, and action-button slot.",
        intents: &[], // all intents — layout gate is inside check_page_header
        check: check_page_header,
    },
    DesignRule {
        id: "prefer-data-table",
        title: "Prefer DataTable over raw Table",
        rationale: "DataTable adds responsive mobile cards and DropdownMenu row actions the raw Table lacks.",
        intents: &["browse"],
        check: check_prefer_data_table,
    },
    DesignRule {
        id: "list-empty-state",
        title: "List pages define an empty state",
        rationale: "An empty state with a create CTA turns a blank list into a first-run affordance.",
        intents: &["browse"],
        check: check_list_empty_state,
    },
    // row-actions-grouped and breadcrumb-on-subpages added in Task 2
];

// ── Rule check functions ──────────────────────────────────────────────────────

/// Returns `true` for layouts that own page chrome (PageHeader, Breadcrumb).
/// Auth layouts and custom / absent layouts are excluded.
fn is_app_shell_layout(spec: &Spec) -> bool {
    matches!(spec.layout.as_deref(), Some("dashboard") | Some("app"))
}

fn check_page_header(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    if !is_app_shell_layout(spec) {
        return vec![];
    }
    // Search the flat element map for any PageHeader.
    let header = spec
        .elements
        .iter()
        .find(|(_, el)| el.type_name == "PageHeader");
    match header {
        None => vec![Finding {
            rule: "page-header",
            element_id: None,
            severity: Severity::Warning,
            message: "Dashboard-family layout has no PageHeader element.".into(),
            suggestion:
                "Add a PageHeader element (with a `title` prop) as the first child of root.".into(),
        }],
        Some((id, el)) => {
            // PageHeader exists — check that it carries a non-null title.
            let title_missing = el.props.get("title").map(|v| v.is_null()).unwrap_or(true);
            if title_missing {
                vec![Finding {
                    rule: "page-header",
                    element_id: Some(id.clone()),
                    severity: Severity::Warning,
                    message: "PageHeader is missing a title.".into(),
                    suggestion: "Set the PageHeader `title` prop.".into(),
                }]
            } else {
                vec![]
            }
        }
    }
}

fn check_prefer_data_table(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    spec.elements
        .iter()
        .filter(|(_, el)| el.type_name == "Table")
        .map(|(id, _)| Finding {
            rule: "prefer-data-table",
            element_id: Some(id.clone()),
            severity: Severity::Warning,
            message: "Raw Table used for an entity list.".into(),
            suggestion:
                "Replace with a DataTable (responsive mobile cards, DropdownMenu row actions)."
                    .into(),
        })
        .collect()
}

fn check_list_empty_state(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    let has_empty_state = spec.elements.values().any(|e| e.type_name == "EmptyState");
    spec.elements
        .iter()
        .filter(|(_, el)| el.type_name == "DataTable" || el.type_name == "MediaCardGrid")
        .filter_map(|(id, el)| {
            let has_empty_message = el
                .props
                .get("empty_message")
                .and_then(|v| v.as_str())
                .is_some();
            if !has_empty_message && !has_empty_state {
                Some(Finding {
                    rule: "list-empty-state",
                    element_id: Some(id.clone()),
                    severity: Severity::Warning,
                    message: "List component has no empty-state config.".into(),
                    suggestion: "Add an `empty_message` to the DataTable or include an EmptyState \
                         element with a create CTA."
                        .into(),
                })
            } else {
                None
            }
        })
        .collect()
}

// ── Rule tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::design::{lint, Finding, Severity};
    use crate::spec::Spec;

    /// Filter findings down to those produced by a specific rule id.
    fn findings_for(all: Vec<Finding>, rule: &str) -> Vec<Finding> {
        all.into_iter().filter(|f| f.rule == rule).collect()
    }

    // ── page-header ──────────────────────────────────────────────────────────

    #[test]
    fn page_header_violating_dashboard_no_header() {
        // Dashboard layout with no PageHeader element → 1 Warning.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "layout": "dashboard",
                "elements": {"r": {"type": "DataTable", "props": {"empty_message": "No items"}}},
                "design": {"intent": "browse"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "page-header");
        assert_eq!(
            findings.len(),
            1,
            "expected 1 page-header finding, got: {findings:#?}"
        );
        assert_eq!(findings[0].severity, Severity::Warning);
    }

    #[test]
    fn page_header_conforming_dashboard_with_titled_header() {
        // Dashboard layout with PageHeader carrying a title → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "ph",
                "layout": "dashboard",
                "elements": {
                    "ph": {"type": "PageHeader", "props": {"title": "Items"}},
                    "dt": {"type": "DataTable", "props": {"empty_message": "No items"}}
                },
                "design": {"intent": "browse"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "page-header");
        assert!(
            findings.is_empty(),
            "conforming dashboard page should have no page-header findings, got: {findings:#?}"
        );
    }

    #[test]
    fn page_header_auth_layout_exempt() {
        // Auth layout → the rule never fires regardless of page structure.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "layout": "auth",
                "elements": {"r": {"type": "Form"}},
                "design": {"intent": "collect"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "page-header");
        assert!(
            findings.is_empty(),
            "auth layout must be exempt from page-header, got: {findings:#?}"
        );
    }

    // ── prefer-data-table ────────────────────────────────────────────────────

    #[test]
    fn prefer_data_table_violating_raw_table_on_browse() {
        // Browse intent with a raw Table element → 1 Warning.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {"r": {"type": "Table"}},
                "design": {"intent": "browse"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "prefer-data-table");
        assert_eq!(
            findings.len(),
            1,
            "expected 1 prefer-data-table finding, got: {findings:#?}"
        );
        assert_eq!(findings[0].severity, Severity::Warning);
    }

    #[test]
    fn prefer_data_table_conforming_data_table_on_browse() {
        // Browse intent with a DataTable → 0 findings (DataTable is the intended component).
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {"r": {"type": "DataTable", "props": {"empty_message": "No items"}}},
                "design": {"intent": "browse"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "prefer-data-table");
        assert!(
            findings.is_empty(),
            "DataTable should be conforming for prefer-data-table, got: {findings:#?}"
        );
    }

    // ── list-empty-state ─────────────────────────────────────────────────────

    #[test]
    fn list_empty_state_violating_data_table_no_empty_config() {
        // Browse intent, DataTable with no empty_message and no EmptyState → 1 Warning.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {"r": {"type": "DataTable"}},
                "design": {"intent": "browse"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "list-empty-state");
        assert_eq!(
            findings.len(),
            1,
            "expected 1 list-empty-state finding, got: {findings:#?}"
        );
        assert_eq!(findings[0].severity, Severity::Warning);
    }

    #[test]
    fn list_empty_state_conforming_with_empty_message() {
        // Browse intent, DataTable with empty_message prop → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {"r": {"type": "DataTable", "props": {"empty_message": "No items"}}},
                "design": {"intent": "browse"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "list-empty-state");
        assert!(
            findings.is_empty(),
            "DataTable with empty_message should be conforming, got: {findings:#?}"
        );
    }

    #[test]
    fn list_empty_state_conforming_with_empty_state_sibling() {
        // Browse intent, DataTable with no empty_message but an EmptyState element → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {
                    "r": {"type": "DataTable"},
                    "es": {"type": "EmptyState", "props": {"title": "No items", "action_label": "Create"}}
                },
                "design": {"intent": "browse"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "list-empty-state");
        assert!(
            findings.is_empty(),
            "DataTable with EmptyState sibling should be conforming, got: {findings:#?}"
        );
    }
}
