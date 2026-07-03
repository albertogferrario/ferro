//! Static design-rule registry — batch A (Plans 02–03).
use crate::spec::Spec;
use super::types::{DesignRule, Finding, Severity};

/// The static rule registry. Iterated by [`super::lint`] and [`super::rules`].
pub(super) static RULE_REGISTRY: &[DesignRule] = &[];

// ── Rule check functions (Plans 02–03) ───────────────────────────────────────

/// Returns `true` for layouts that own page chrome (PageHeader, Breadcrumb).
/// Auth layouts and custom / absent layouts are excluded.
fn is_app_shell_layout(spec: &Spec) -> bool {
    matches!(spec.layout.as_deref(), Some("dashboard") | Some("app"))
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
        assert_eq!(findings.len(), 1, "expected 1 page-header finding, got: {findings:#?}");
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
