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
    DesignRule {
        id: "form-default-values",
        title: "Edit-form fields pre-fill from data",
        rationale: "On an edit form every field must restore its stored value; a blank field silently discards data on save.",
        intents: &["collect"],
        check: check_form_default_values,
    },
    DesignRule {
        id: "destructive-confirmation",
        title: "Destructive actions require confirmation",
        rationale: "An irreversible action behind a single click is a data-loss hazard; a confirm dialog is the guard.",
        intents: &[], // all intents
        check: check_destructive_confirmation,
    },
    DesignRule {
        id: "prefer-components",
        title: "Prefer catalog components over RawHtml",
        rationale: "UI inside a RawHtml escape hatch is invisible to the design system: tokens, variants, and every other lint rule cannot see it. Each use should be a deliberate, `allow`-justified exception.",
        intents: &[], // all intents
        check: check_prefer_components,
    },
    DesignRule {
        id: "register-fill-viewport",
        title: "Register pages must fill the viewport",
        rationale: "A TileGrid, SelectionPanel, or Numpad outside a fill_viewport spec causes silent whole-page scroll, breaking the register feel.",
        intents: &[], // all intents — internal presence gate is inside check_pos_fill_viewport
        check: check_pos_fill_viewport,
    },
    DesignRule {
        id: "register-grid-fill",
        title: "The register-root Grid must set fill:true under fill_viewport",
        rationale: "A fill_viewport spec whose root Grid lacks fill:true loses per-pane internal scroll — the panes scroll the page instead.",
        intents: &[], // all intents — fill_viewport gate is inside check_pos_grid_fill
        check: check_pos_grid_fill,
    },
    DesignRule {
        id: "register-selection-present",
        title: "A TileGrid register needs a SelectionPanel",
        rationale: "A TileGrid with no SelectionPanel anywhere is an incomplete register — the operator has products but nowhere to accumulate the sale.",
        intents: &[], // all intents — internal presence gate is inside check_pos_cart_present
        check: check_pos_cart_present,
    },
    DesignRule {
        id: "fill-viewport-layout-unknown",
        title: "fill_viewport requires an app-shell layout",
        rationale: "The ferro-fill CSS chain only supports the app and dashboard layouts; on any other layout fill_viewport silently degrades to whole-page scroll.",
        intents: &[], // all intents — fill_viewport gate is inside check_fill_viewport_layout_unknown
        check: check_fill_viewport_layout_unknown,
    },
    DesignRule {
        id: "skin-raw-literals",
        title: "Skin rules must use var(--token) references, not raw literals",
        rationale: "Raw color/size literals in fjui-* rules bypass the token contract; the skin cannot be rethemed by overriding tokens alone.",
        intents: &[], // all intents — CSS file lint, not intent-specific
        // Actual check runs via --skin CLI flag; this stub returns empty from the spec-lint path.
        check: check_skin_raw_literals_stub,
    },
    DesignRule {
        id: "skin-interaction-states",
        title: "Interactive fjui-* rules must define all four interaction states",
        rationale: "Missing hover/focus-visible/active/disabled states silently drop keyboard and pointer affordances.",
        intents: &[], // all intents — CSS file lint, not intent-specific
        // Actual check runs via --skin CLI flag; this stub returns empty from the spec-lint path.
        check: check_skin_interaction_states_stub,
    },
    DesignRule {
        id: "contrast-lint",
        title: "Token contrast ratios must meet WCAG floors",
        rationale: "Text token pairs must achieve >=4.5:1 and UI/non-text pairs >=3:1 in both light and dark modes.",
        intents: &[], // all intents — tokens.css file lint, not intent-specific
        // Actual check runs via --tokens CLI flag; this stub returns empty from the spec-lint path.
        check: check_token_contrast_stub,
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
                .map(|v| !v.is_null())
                .unwrap_or(false);
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
                .map(|v| !v.is_null())
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

/// Form-field element types for `form-default-values` detection.
const FIELD_TYPES: &[&str] = &["Input", "Select", "RichTextEditor"];

fn check_form_default_values(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    // Detect edit form: at least one field binds default_value via a $data path.
    let is_edit_form = spec.elements.values().any(|e| {
        FIELD_TYPES.contains(&e.type_name.as_str())
            && e.props
                .get("default_value")
                .and_then(|v| v.get("$data"))
                .is_some()
    });
    if !is_edit_form {
        // Pure create form — no binding detected; rule does not apply.
        return vec![];
    }
    // Warn for each field missing a default_value (or with null).
    spec.elements
        .iter()
        .filter(|(_, el)| {
            FIELD_TYPES.contains(&el.type_name.as_str())
                && el
                    .props
                    .get("default_value")
                    .map(|v| v.is_null())
                    .unwrap_or(true) // absent key counts as missing
        })
        .map(|(id, _)| Finding {
            rule: "form-default-values",
            element_id: Some(id.clone()),
            severity: Severity::Warning,
            message: format!("Edit-form field `{id}` has no default_value."),
            suggestion: "Pre-fill the field: bind default_value via a $data path \
                 (e.g. req.old(..).or_else(|| record.field))."
                .into(),
        })
        .collect()
}

fn check_destructive_confirmation(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (id, el) in &spec.elements {
        let mut flagged = false;
        // Element-level: Button with variant=destructive and action without confirm.
        if el.type_name == "Button"
            && el.props.get("variant").and_then(|v| v.as_str()) == Some("destructive")
        {
            if let Some(action) = &el.action {
                if action.confirm.is_none() {
                    findings.push(Finding {
                        rule: "destructive-confirmation",
                        element_id: Some(id.clone()),
                        severity: Severity::Warning,
                        message: "Destructive action has no confirmation.".into(),
                        suggestion: "Add a `confirm` (ConfirmDialog) to the action before it runs."
                            .into(),
                    });
                    flagged = true;
                }
            }
        }
        // Props-embedded: row_actions or items arrays with destructive entries missing confirm.
        if !flagged {
            'outer: for key in &["row_actions", "items"] {
                if let Some(arr) = el.props.get(*key).and_then(|v| v.as_array()) {
                    for entry in arr {
                        // Conformance is `Action.confirm` — the confirm dialog lives on
                        // the nested action object, not on the item entry (which the
                        // renderer would silently ignore).
                        if entry.get("destructive").and_then(|v| v.as_bool()) == Some(true)
                            && entry.pointer("/action/confirm").is_none()
                        {
                            findings.push(Finding {
                                rule: "destructive-confirmation",
                                element_id: Some(id.clone()),
                                severity: Severity::Warning,
                                message: "Destructive action has no confirmation.".into(),
                                suggestion: "Add a `confirm` (ConfirmDialog) to the action \
                                     before it runs."
                                    .into(),
                            });
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
    findings
}

fn check_prefer_components(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    spec.elements
        .iter()
        .filter(|(_, el)| el.type_name == "RawHtml")
        .map(|(id, _)| Finding {
            rule: "prefer-components",
            element_id: Some(id.clone()),
            severity: Severity::Info,
            message: format!("Element `{id}` is a RawHtml escape hatch."),
            suggestion: "Compose from catalog components where possible; if the escape is \
                 deliberate, add `prefer-components` to `design.allow` with the reason in review."
                .into(),
        })
        .collect()
}

// ── Register rules (Phase 254, POS-11) ───────────────────────────────────────

/// Component type names that indicate a register composition.
/// Matched against raw spec type_name strings; lint never consults BUILTIN_TYPES (D-13).
const REGISTER_TRIGGER_TYPES: &[&str] = &["TileGrid", "SelectionPanel", "Numpad"];

fn check_pos_fill_viewport(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    let has_pos = spec
        .elements
        .values()
        .any(|el| REGISTER_TRIGGER_TYPES.contains(&el.type_name.as_str()));
    if !has_pos || spec.fill_viewport {
        return vec![];
    }
    vec![Finding {
        rule: "register-fill-viewport",
        element_id: None,
        severity: Severity::Warning,
        message: "Spec contains register components but fill_viewport is not set.".into(),
        suggestion: "Set fill_viewport: true at the spec level and fill: true on the root Grid."
            .into(),
    }]
}

fn check_pos_grid_fill(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    if !spec.fill_viewport {
        return vec![];
    }
    // Register-root Grid identification: the spec root element when it is a Grid.
    let root = match spec.elements.get(&spec.root) {
        Some(el) if el.type_name == "Grid" => el,
        _ => return vec![],
    };
    // Non-null acceptance: lint runs on the pre-resolve spec, where `fill` may
    // legitimately be `$data`-bound. Mirror the list-empty-state and
    // breadcrumb-on-subpages pattern — any non-null value other than a literal
    // `false` counts as set.
    let fill_set = root
        .props
        .get("fill")
        .map(|v| !v.is_null() && v.as_bool() != Some(false))
        .unwrap_or(false);
    if fill_set {
        return vec![];
    }
    vec![Finding {
        rule: "register-grid-fill",
        element_id: Some(spec.root.clone()),
        severity: Severity::Warning,
        message:
            "fill_viewport spec has a root Grid without fill:true; panes lose internal scroll."
                .into(),
        suggestion: "Add fill: true to the root Grid props.".into(),
    }]
}

fn check_pos_cart_present(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    let has_grid = spec.elements.values().any(|el| el.type_name == "TileGrid");
    let has_cart = spec
        .elements
        .values()
        .any(|el| el.type_name == "SelectionPanel");
    if !has_grid || has_cart {
        return vec![];
    }
    vec![Finding {
        rule: "register-selection-present",
        element_id: None,
        severity: Severity::Warning,
        message: "TileGrid present but no SelectionPanel anywhere in the spec.".into(),
        suggestion: "Add a SelectionPanel element so the register can accumulate the sale.".into(),
    }]
}

fn check_fill_viewport_layout_unknown(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    if !spec.fill_viewport || is_app_shell_layout(spec) {
        return vec![];
    }
    vec![Finding {
        rule: "fill-viewport-layout-unknown",
        element_id: None,
        severity: Severity::Warning,
        message: "fill_viewport is set but the layout is not in the supported set (\"app\", \"dashboard\")."
            .into(),
        suggestion:
            "Use layout: \"app\" or \"dashboard\"; fill_viewport degrades to whole-page scroll on other layouts."
                .into(),
    }]
}

// ── Stub check fns for CSS-file rules (skin-raw-literals, skin-interaction-states, contrast-lint)
//
// These rules operate on CSS files, not JSON specs.  The RULE_REGISTRY entries
// are required so D-09 (patterns.md drift guard) stays green.  The actual checks
// are invoked from the `--skin` / `--tokens` CLI flags in ferro-cli's design_lint
// command, NOT from the spec-lint path.  These stubs always return vec![] when
// called from `lint(&spec)`.

fn check_skin_raw_literals_stub(_spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    vec![]
}

fn check_skin_interaction_states_stub(_spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    vec![]
}

fn check_token_contrast_stub(_spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    vec![]
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
    fn list_empty_state_conforming_data_bound_empty_message() {
        // Browse intent, DataTable where empty_message is a $data binding → 0 findings.
        // Regression guard for WR-01: the rule must accept any non-null value, not only string literals.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {"r": {"type": "DataTable", "props": {"empty_message": {"$data": "/i18n/no_items"}}}},
                "design": {"intent": "browse"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "list-empty-state");
        assert!(
            findings.is_empty(),
            "$data-bound empty_message must be accepted, got: {findings:#?}"
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

    #[test]
    fn breadcrumb_on_subpages_conforming_data_bound_breadcrumb_prop() {
        // Dashboard layout, collect intent, PageHeader with $data-bound breadcrumb → 0 findings.
        // Regression guard for WR-02: any non-null breadcrumb value must be accepted.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "ph",
                "layout": "dashboard",
                "elements": {
                    "ph": {"type": "PageHeader", "props": {"title": "New Item", "breadcrumb": {"$data": "/breadcrumb_items"}}},
                    "f": {"type": "Form"}
                },
                "design": {"intent": "collect"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "breadcrumb-on-subpages");
        assert!(
            findings.is_empty(),
            "$data-bound breadcrumb must be accepted, got: {findings:#?}"
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

    // ── prefer-components ─────────────────────────────────────────────────────

    #[test]
    fn prefer_components_violating_raw_html_element() {
        // A RawHtml element → 1 Info finding naming the element.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {
                    "r": {"type": "RawHtml", "props": {"html": "<b>hi</b>"}}
                },
                "design": {"intent": "browse"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "prefer-components");
        assert_eq!(
            findings.len(),
            1,
            "expected 1 prefer-components finding, got: {findings:#?}"
        );
        assert_eq!(findings[0].severity, Severity::Info);
    }

    #[test]
    fn prefer_components_conforming_no_raw_html() {
        // No RawHtml anywhere → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {
                    "r": {"type": "Text", "props": {"content": "hi"}}
                },
                "design": {"intent": "browse"}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "prefer-components");
        assert!(
            findings.is_empty(),
            "component-only spec should be conforming, got: {findings:#?}"
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
        // row_action with destructive=true and action.confirm present → 0 findings.
        // The confirm dialog lives on the nested Action object (DropdownMenuAction
        // has no entry-level confirm field).
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {
                    "r": {"type": "KanbanBoard", "props": {
                        "row_actions": [
                            {"label": "Delete", "destructive": true,
                             "action": {"handler": "items.destroy", "method": "POST",
                                        "confirm": {"title": "Delete?", "tone": "destructive"}}}
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
            "row_action with action.confirm should be conforming, got: {findings:#?}"
        );
    }

    #[test]
    fn destructive_confirmation_violating_row_action_entry_level_confirm() {
        // Entry-level confirm is not a real DropdownMenuAction field — the renderer
        // ignores it and no dialog appears. Must still be flagged (1 Warning).
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {
                    "r": {"type": "KanbanBoard", "props": {
                        "row_actions": [
                            {"label": "Delete", "destructive": true,
                             "confirm": {"title": "Delete?", "tone": "destructive"},
                             "action": {"handler": "items.destroy", "method": "POST"}}
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
            "entry-level confirm does not reach the renderer and must be flagged, got: {findings:#?}"
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

    // ── register-fill-viewport ──────────────────────────────────────────────

    #[test]
    fn register_fill_viewport_violating_tile_grid_no_fill_viewport() {
        // TileGrid present, fill_viewport absent → 1 Warning.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {"r": {"type": "TileGrid"}},
                "design": {}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "register-fill-viewport");
        assert_eq!(
            findings.len(),
            1,
            "expected 1 register-fill-viewport finding, got: {findings:#?}"
        );
        assert_eq!(findings[0].severity, Severity::Warning);
    }

    #[test]
    fn register_fill_viewport_conforming_fill_viewport_set() {
        // TileGrid present and fill_viewport: true → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "fill_viewport": true,
                "elements": {"r": {"type": "TileGrid"}},
                "design": {}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "register-fill-viewport");
        assert!(
            findings.is_empty(),
            "fill_viewport set must be conforming, got: {findings:#?}"
        );
    }

    #[test]
    fn register_fill_viewport_data_bound_no_misfire() {
        // No register type names present, only a DataTable with a $data binding → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "fill_viewport": false,
                "elements": {"r": {"type": "DataTable", "props": {"data_path": {"$data": "/products"}}}},
                "design": {}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "register-fill-viewport");
        assert!(
            findings.is_empty(),
            "spec without register type names must not misfire: {findings:#?}"
        );
    }

    // ── register-grid-fill ──────────────────────────────────────────────────

    #[test]
    fn register_grid_fill_violating_fill_viewport_root_grid_no_fill() {
        // fill_viewport: true, root element is Grid without fill prop → 1 Warning.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "fill_viewport": true,
                "elements": {"r": {"type": "Grid", "props": {"columns": 2}}},
                "design": {}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "register-grid-fill");
        assert_eq!(
            findings.len(),
            1,
            "expected 1 register-grid-fill finding, got: {findings:#?}"
        );
        assert_eq!(findings[0].severity, Severity::Warning);
    }

    #[test]
    fn register_grid_fill_conforming_root_grid_with_fill_true() {
        // fill_viewport: true, root Grid with fill: true → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "fill_viewport": true,
                "elements": {"r": {"type": "Grid", "props": {"columns": 2, "fill": true}}},
                "design": {}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "register-grid-fill");
        assert!(
            findings.is_empty(),
            "root Grid with fill:true must be conforming, got: {findings:#?}"
        );
    }

    #[test]
    fn register_grid_fill_data_bound_no_misfire() {
        // fill_viewport: true, root Grid with fill: true carrying a $data-bound child prop → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "fill_viewport": true,
                "elements": {
                    "r": {"type": "Grid", "props": {"columns": 2, "fill": true}},
                    "tbl": {"type": "DataTable", "props": {"rows": {"$data": "/rows"}}}
                },
                "design": {}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "register-grid-fill");
        assert!(
            findings.is_empty(),
            "$data-bound child must not misfire register-grid-fill: {findings:#?}"
        );
    }

    #[test]
    fn register_grid_fill_data_bound_fill_no_misfire() {
        // WR-02 regression guard: fill_viewport: true, root Grid whose `fill`
        // prop itself is $data-bound → 0 findings (non-null acceptance,
        // mirroring the list-empty-state/breadcrumb-on-subpages guards).
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "fill_viewport": true,
                "elements": {"r": {"type": "Grid", "props": {"columns": 2, "fill": {"$data": "/ui/fill"}}}},
                "design": {}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "register-grid-fill");
        assert!(
            findings.is_empty(),
            "$data-bound fill prop must not misfire register-grid-fill: {findings:#?}"
        );
    }

    // ── register-selection-present ───────────────────────────────────────────

    #[test]
    fn register_selection_present_violating_tile_grid_no_selection_panel() {
        // TileGrid present, SelectionPanel absent → 1 Warning.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "elements": {"r": {"type": "TileGrid"}},
                "design": {}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "register-selection-present");
        assert_eq!(
            findings.len(),
            1,
            "expected 1 register-selection-present finding, got: {findings:#?}"
        );
        assert_eq!(findings[0].severity, Severity::Warning);
    }

    #[test]
    fn register_selection_present_conforming_both_tile_grid_and_selection_panel() {
        // Both TileGrid and SelectionPanel present → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "fill_viewport": true,
                "elements": {
                    "r": {"type": "Grid", "props": {"fill": true}},
                    "grid": {"type": "TileGrid"},
                    "cart": {"type": "SelectionPanel"}
                },
                "design": {}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "register-selection-present");
        assert!(
            findings.is_empty(),
            "TileGrid + SelectionPanel must be conforming, got: {findings:#?}"
        );
    }

    #[test]
    fn register_selection_present_data_bound_no_misfire() {
        // TileGrid + SelectionPanel both present with $data-bound props → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "fill_viewport": true,
                "elements": {
                    "r": {"type": "Grid", "props": {"fill": true}},
                    "grid": {"type": "TileGrid", "props": {"items": {"$data": "/products"}}},
                    "cart": {"type": "SelectionPanel", "props": {"lines": {"$data": "/cart/lines"}}}
                },
                "design": {}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "register-selection-present");
        assert!(
            findings.is_empty(),
            "$data-bound TileGrid+SelectionPanel must not misfire: {findings:#?}"
        );
    }

    // ── fill-viewport-layout-unknown ─────────────────────────────────────────

    #[test]
    fn fill_viewport_layout_unknown_violating_fill_viewport_auth_layout() {
        // fill_viewport: true with layout: "auth" → 1 Warning.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "fill_viewport": true,
                "layout": "auth",
                "elements": {"r": {"type": "Grid"}},
                "design": {}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "fill-viewport-layout-unknown");
        assert_eq!(
            findings.len(),
            1,
            "expected 1 fill-viewport-layout-unknown finding, got: {findings:#?}"
        );
        assert_eq!(findings[0].severity, Severity::Warning);
    }

    #[test]
    fn fill_viewport_layout_unknown_conforming_fill_viewport_app_layout() {
        // fill_viewport: true with layout: "app" → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "fill_viewport": true,
                "layout": "app",
                "elements": {"r": {"type": "Grid", "props": {"fill": true}}},
                "design": {}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "fill-viewport-layout-unknown");
        assert!(
            findings.is_empty(),
            "fill_viewport + layout:app must be conforming, got: {findings:#?}"
        );
    }

    #[test]
    fn fill_viewport_layout_unknown_data_bound_no_misfire() {
        // fill_viewport: true, layout: "app", element with $data-bound prop → 0 findings.
        let spec = Spec::from_json(
            r#"{
                "$schema": "ferro-json-ui/v2",
                "root": "r",
                "fill_viewport": true,
                "layout": "app",
                "elements": {
                    "r": {"type": "Grid", "props": {"fill": true}},
                    "tbl": {"type": "DataTable", "props": {"rows": {"$data": "/rows"}}}
                },
                "design": {}
            }"#,
        )
        .unwrap();
        let findings = findings_for(lint(&spec), "fill-viewport-layout-unknown");
        assert!(
            findings.is_empty(),
            "$data-bound element on conforming layout must not misfire: {findings:#?}"
        );
    }
}
