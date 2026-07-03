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
    DesignRule {
        id: "row-actions-grouped",
        title: "Group row/card actions in an ActionGroup",
        rationale: "Loose inline buttons per row are inconsistent and crowd small screens; an ActionGroup/DropdownMenu keeps them tidy.",
        intents: &["browse", "process"],
        check: check_row_actions_grouped,
    },
    DesignRule {
        id: "breadcrumb-on-subpages",
        title: "Create/edit/detail pages carry a Breadcrumb",
        rationale: "A breadcrumb back to the list page keeps navigation reversible on nested pages.",
        intents: &["collect", "focus"],
        check: check_breadcrumb_on_subpages,
    },
    DesignRule {
        id: "process-kanban",
        title: "Status-workflow pages use a KanbanBoard",
        rationale: "A KanbanBoard with per-column count badges is the canonical view for status workflows.",
        intents: &["process"],
        check: check_process_kanban,
    },
    DesignRule {
        id: "card-actions-in-menu",
        title: "Kanban card actions belong in the menu, destructive last",
        rationale: "Consistent action order (detail first, destructive last) inside the ActionGroup prevents mis-clicks on cards.",
        intents: &["process"],
        check: check_card_actions_in_menu,
    },
    DesignRule {
        id: "create-separate-page",
        title: "Entity creation is a dedicated page, not a Modal",
        rationale: "A separate create/edit page is linkable, refresh-safe, and leaves room for validation feedback.",
        intents: &["collect"],
        check: check_create_separate_page,
    },
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

fn check_row_actions_grouped(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    spec.elements
        .iter()
        .filter_map(|(id, el)| {
            let btn_count = el
                .children
                .iter()
                .filter(|c| {
                    spec.elements
                        .get(c.as_str())
                        .map(|child| child.type_name == "Button")
                        .unwrap_or(false)
                })
                .count();
            if btn_count >= 2 {
                Some(Finding {
                    rule: "row-actions-grouped",
                    element_id: Some(id.clone()),
                    severity: Severity::Warning,
                    message: format!("Element `{id}` has {btn_count} loose Button children."),
                    suggestion: "Group these row/card actions in an ActionGroup (DropdownMenu) \
                         instead of loose inline Buttons."
                        .into(),
                })
            } else {
                None
            }
        })
        .collect()
}

fn check_breadcrumb_on_subpages(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    if !is_app_shell_layout(spec) {
        // Auth pages and layout-less specs are exempt.
        return vec![];
    }
    let has_breadcrumb_element = spec.elements.values().any(|e| e.type_name == "Breadcrumb");
    let has_breadcrumb_in_header = spec.elements.values().any(|e| {
        e.type_name == "PageHeader"
            && e.props
                .get("breadcrumb")
                .and_then(|v| v.as_array())
                .map(|a| !a.is_empty())
                .unwrap_or(false)
    });
    if !has_breadcrumb_element && !has_breadcrumb_in_header {
        vec![Finding {
            rule: "breadcrumb-on-subpages",
            element_id: None,
            severity: Severity::Warning,
            message: "App-shell subpage has no Breadcrumb.".into(),
            suggestion: "Add a Breadcrumb (or a PageHeader with a non-empty `breadcrumb`) \
                 linking back to the list page."
                .into(),
        }]
    } else {
        vec![]
    }
}

fn check_process_kanban(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    if spec.elements.values().any(|e| e.type_name == "KanbanBoard") {
        vec![]
    } else {
        vec![Finding {
            rule: "process-kanban",
            element_id: None,
            severity: Severity::Warning,
            message: "Process page has no KanbanBoard.".into(),
            suggestion: "Use a KanbanBoard with per-column count badges for status workflows."
                .into(),
        }]
    }
}

fn check_card_actions_in_menu(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (id, el) in &spec.elements {
        if el.type_name != "KanbanBoard" {
            continue;
        }
        let Some(acts) = el.props.get("row_actions").and_then(|v| v.as_array()) else {
            continue;
        };
        for (i, act) in acts.iter().enumerate() {
            if act.get("destructive").and_then(|v| v.as_bool()) == Some(true) && i != acts.len() - 1
            {
                findings.push(Finding {
                    rule: "card-actions-in-menu",
                    element_id: Some(id.clone()),
                    severity: Severity::Warning,
                    message: "A destructive card action is not last in the menu.".into(),
                    suggestion: "Order card actions: detail/view first, destructive last, all \
                         inside the ActionGroup."
                        .into(),
                });
                break; // one warning per offending KanbanBoard
            }
        }
    }
    findings
}

fn check_create_separate_page(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (id, el) in &spec.elements {
        if el.type_name != "Modal" {
            continue;
        }
        let has_form_child = el.children.iter().any(|c| {
            spec.elements
                .get(c.as_str())
                .map(|e| e.type_name == "Form")
                .unwrap_or(false)
        });
        if has_form_child {
            findings.push(Finding {
                rule: "create-separate-page",
                element_id: Some(id.clone()),
                severity: Severity::Warning,
                message: "Modal contains a Form — entity creation should be a dedicated page."
                    .into(),
                suggestion: "Move the form to a separate /nuovo or /modifica page instead of a \
                     Modal."
                    .into(),
            });
        }
    }
    findings
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

    // ── row-actions-grouped ──────────────────────────────────────────────────

    #[test]
    fn row_actions_grouped_violating_two_inline_buttons_on_browse() {
        // Browse intent, a Card with 2 Button children → 1 Warning.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "card",
                "elements": {
                    "card": {"type": "Card", "children": ["btn1", "btn2"]},
                    "btn1": {"type": "Button", "props": {"label": "Edit"}},
                    "btn2": {"type": "Button", "props": {"label": "Delete"}}
                },
                "design": {"intent": "browse"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "row-actions-grouped");
        assert_eq!(
            findings.len(),
            1,
            "expected 1 row-actions-grouped finding, got: {findings:#?}"
        );
        assert_eq!(findings[0].severity, Severity::Warning);
    }

    #[test]
    fn row_actions_grouped_conforming_single_button_child() {
        // Browse intent, a Card with 1 Button child → 0 findings (threshold is ≥2).
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "card",
                "elements": {
                    "card": {"type": "Card", "children": ["btn1"]},
                    "btn1": {"type": "Button", "props": {"label": "Edit"}}
                },
                "design": {"intent": "browse"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "row-actions-grouped");
        assert!(
            findings.is_empty(),
            "single Button child should be conforming, got: {findings:#?}"
        );
    }

    // ── breadcrumb-on-subpages ───────────────────────────────────────────────

    #[test]
    fn breadcrumb_on_subpages_violating_dashboard_collect_no_breadcrumb() {
        // Dashboard layout, collect intent, no Breadcrumb → 1 Warning.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "ph",
                "layout": "dashboard",
                "elements": {
                    "ph": {"type": "PageHeader", "props": {"title": "New Item"}},
                    "f": {"type": "Form"}
                },
                "design": {"intent": "collect"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "breadcrumb-on-subpages");
        assert_eq!(
            findings.len(),
            1,
            "expected 1 breadcrumb-on-subpages finding, got: {findings:#?}"
        );
        assert_eq!(findings[0].severity, Severity::Warning);
    }

    #[test]
    fn breadcrumb_on_subpages_conforming_breadcrumb_element_present() {
        // Dashboard layout, collect intent, Breadcrumb element present → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "ph",
                "layout": "dashboard",
                "elements": {
                    "ph": {"type": "PageHeader", "props": {"title": "New Item"}},
                    "bc": {"type": "Breadcrumb"},
                    "f": {"type": "Form"}
                },
                "design": {"intent": "collect"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "breadcrumb-on-subpages");
        assert!(
            findings.is_empty(),
            "Breadcrumb element present should be conforming, got: {findings:#?}"
        );
    }

    #[test]
    fn breadcrumb_on_subpages_auth_layout_exempt() {
        // Auth layout → rule never fires (regression guard for login pages).
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
        let findings = findings_for(lint(&spec), "breadcrumb-on-subpages");
        assert!(
            findings.is_empty(),
            "auth layout must be exempt from breadcrumb-on-subpages, got: {findings:#?}"
        );
    }

    #[test]
    fn breadcrumb_on_subpages_conforming_page_header_breadcrumb_prop() {
        // Dashboard layout, collect intent, PageHeader with non-empty breadcrumb array → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "ph",
                "layout": "dashboard",
                "elements": {
                    "ph": {"type": "PageHeader", "props": {"title": "New Item", "breadcrumb": [{"label": "Items", "href": "/items"}]}},
                    "f": {"type": "Form"}
                },
                "design": {"intent": "collect"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "breadcrumb-on-subpages");
        assert!(
            findings.is_empty(),
            "PageHeader with breadcrumb prop should be conforming, got: {findings:#?}"
        );
    }

    // ── process-kanban ───────────────────────────────────────────────────────

    #[test]
    fn process_kanban_violating_no_kanban_board() {
        // Process intent, no KanbanBoard element → 1 Warning.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {"r": {"type": "DataTable", "props": {"empty_message": "No items"}}},
                "design": {"intent": "process"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "process-kanban");
        assert_eq!(
            findings.len(),
            1,
            "expected 1 process-kanban finding, got: {findings:#?}"
        );
        assert_eq!(findings[0].severity, Severity::Warning);
    }

    #[test]
    fn process_kanban_conforming_with_kanban_board() {
        // Process intent, KanbanBoard present → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {"r": {"type": "KanbanBoard", "props": {}}},
                "design": {"intent": "process"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "process-kanban");
        assert!(
            findings.is_empty(),
            "KanbanBoard present should be conforming for process-kanban, got: {findings:#?}"
        );
    }

    // ── card-actions-in-menu ─────────────────────────────────────────────────

    #[test]
    fn card_actions_in_menu_violating_destructive_not_last() {
        // Process intent, KanbanBoard with destructive row_action NOT in last position → 1 Warning.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {
                    "r": {"type": "KanbanBoard", "props": {
                        "row_actions": [
                            {"label": "Delete", "destructive": true},
                            {"label": "View details"}
                        ]
                    }}
                },
                "design": {"intent": "process"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "card-actions-in-menu");
        assert_eq!(
            findings.len(),
            1,
            "expected 1 card-actions-in-menu finding, got: {findings:#?}"
        );
        assert_eq!(findings[0].severity, Severity::Warning);
    }

    #[test]
    fn card_actions_in_menu_conforming_destructive_last() {
        // Process intent, KanbanBoard with destructive action last → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {
                    "r": {"type": "KanbanBoard", "props": {
                        "row_actions": [
                            {"label": "View details"},
                            {"label": "Delete", "destructive": true}
                        ]
                    }}
                },
                "design": {"intent": "process"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "card-actions-in-menu");
        assert!(
            findings.is_empty(),
            "destructive last should be conforming for card-actions-in-menu, got: {findings:#?}"
        );
    }

    // ── create-separate-page ─────────────────────────────────────────────────

    #[test]
    fn create_separate_page_violating_modal_contains_form() {
        // Collect intent, Modal element whose child is a Form → 1 Warning.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "m",
                "elements": {
                    "m": {"type": "Modal", "children": ["f"]},
                    "f": {"type": "Form"}
                },
                "design": {"intent": "collect"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "create-separate-page");
        assert_eq!(
            findings.len(),
            1,
            "expected 1 create-separate-page finding, got: {findings:#?}"
        );
        assert_eq!(findings[0].severity, Severity::Warning);
    }

    #[test]
    fn create_separate_page_conforming_no_modal() {
        // Collect intent, Form without a Modal wrapper → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {"r": {"type": "Form"}},
                "design": {"intent": "collect"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "create-separate-page");
        assert!(
            findings.is_empty(),
            "no Modal should be conforming for create-separate-page, got: {findings:#?}"
        );
    }

    // ── form-default-values ──────────────────────────────────────────────────

    #[test]
    fn form_default_values_violating_edit_form_missing_default() {
        // Collect intent, one Input has $data default_value (edit form detected),
        // another Input has no default_value → 1 Warning on the latter.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {
                    "r": {"type": "Form"},
                    "name": {"type": "Input", "props": {
                        "field": "name",
                        "default_value": {"$data": "/record/name"}
                    }},
                    "email": {"type": "Input", "props": {
                        "field": "email"
                    }}
                },
                "design": {"intent": "collect"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "form-default-values");
        assert_eq!(
            findings.len(),
            1,
            "expected 1 form-default-values finding on the field missing default_value, got: {findings:#?}"
        );
        assert_eq!(findings[0].severity, Severity::Warning);
        assert_eq!(findings[0].element_id.as_deref(), Some("email"));
    }

    #[test]
    fn form_default_values_conforming_all_fields_prefilled() {
        // Collect intent, all fields have $data default_value → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {
                    "r": {"type": "Form"},
                    "name": {"type": "Input", "props": {
                        "field": "name",
                        "default_value": {"$data": "/record/name"}
                    }},
                    "email": {"type": "Input", "props": {
                        "field": "email",
                        "default_value": {"$data": "/record/email"}
                    }}
                },
                "design": {"intent": "collect"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "form-default-values");
        assert!(
            findings.is_empty(),
            "all fields pre-filled should be conforming, got: {findings:#?}"
        );
    }

    #[test]
    fn form_default_values_conforming_pure_create_form_login_shape() {
        // Collect intent, pure create form (no field binds $data for default_value)
        // — this is the login.json shape: Input has data_path and error but no default_value.
        // The rule must produce 0 findings (pure create form → skip entirely).
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "layout": "auth",
                "elements": {
                    "r": {"type": "Form"},
                    "email": {"type": "Input", "props": {
                        "field": "email",
                        "data_path": "/email",
                        "error": {"$data": "/error"}
                    }}
                },
                "design": {"intent": "collect"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "form-default-values");
        assert!(
            findings.is_empty(),
            "pure create form (no $data default_value on any field) must produce 0 findings, got: {findings:#?}"
        );
    }

    // ── destructive-confirmation ──────────────────────────────────────────────

    #[test]
    fn destructive_confirmation_violating_button_no_confirm() {
        // Button with variant=destructive and action without confirm → 1 Warning.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {
                    "r": {"type": "Button", "props": {"label": "Delete", "variant": "destructive"},
                          "action": {"handler": "items.destroy", "method": "DELETE"}}
                },
                "design": {"intent": "collect"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "destructive-confirmation");
        assert_eq!(
            findings.len(),
            1,
            "expected 1 destructive-confirmation finding, got: {findings:#?}"
        );
        assert_eq!(findings[0].severity, Severity::Warning);
    }

    #[test]
    fn destructive_confirmation_conforming_button_with_confirm() {
        // Button with variant=destructive and action.confirm present → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {
                    "r": {"type": "Button", "props": {"label": "Delete", "variant": "destructive"},
                          "action": {"handler": "items.destroy", "method": "DELETE",
                                     "confirm": {"title": "Delete item?", "tone": "destructive"}}}
                },
                "design": {"intent": "collect"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "destructive-confirmation");
        assert!(
            findings.is_empty(),
            "destructive Button with confirm should be conforming, got: {findings:#?}"
        );
    }

    #[test]
    fn destructive_confirmation_violating_row_action_no_confirm() {
        // Element with row_actions containing a destructive entry without confirm → 1 Warning.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {
                    "r": {"type": "KanbanBoard", "props": {
                        "row_actions": [
                            {"label": "Delete", "destructive": true}
                        ]
                    }}
                },
                "design": {"intent": "process"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "destructive-confirmation");
        assert_eq!(
            findings.len(),
            1,
            "expected 1 destructive-confirmation finding for row_action without confirm, got: {findings:#?}"
        );
        assert_eq!(findings[0].severity, Severity::Warning);
    }

    #[test]
    fn destructive_confirmation_conforming_row_action_with_confirm() {
        // row_action with destructive=true and confirm key present → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {
                    "r": {"type": "KanbanBoard", "props": {
                        "row_actions": [
                            {"label": "Delete", "destructive": true,
                             "confirm": {"title": "Delete?", "tone": "destructive"}}
                        ]
                    }}
                },
                "design": {"intent": "process"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "destructive-confirmation");
        assert!(
            findings.is_empty(),
            "row_action with confirm should be conforming, got: {findings:#?}"
        );
    }

    #[test]
    fn destructive_confirmation_conforming_non_destructive_button() {
        // Button with variant=primary (not destructive) with no confirm → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {
                    "r": {"type": "Button", "props": {"label": "Save", "variant": "primary"},
                          "action": {"handler": "items.store", "method": "POST"}}
                },
                "design": {"intent": "collect"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "destructive-confirmation");
        assert!(
            findings.is_empty(),
            "non-destructive Button should be conforming, got: {findings:#?}"
        );
    }
}
