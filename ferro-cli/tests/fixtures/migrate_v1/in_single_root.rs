// Fixture: v1 controller with a single top-level node — successfully migratable.
// Phase 163.1 test fixture: exercises the single-root branch of try_migrate_handler.

pub async fn dashboard(req: Request) -> Response {
    let view = JsonUiView::new(
        "Dashboard",
        vec![make_node(
            "header",
            Component::PageHeader(PageHeaderProps {
                title: "Dashboard".to_string(),
                ..Default::default()
            }),
        )],
    );
    JsonUi::render(&view, &json!({}))
}
