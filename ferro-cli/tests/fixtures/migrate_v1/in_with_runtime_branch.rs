// Fixture: runtime-branch handler. The codemod MUST refuse to translate this
// and emit a `// TODO: ferro json-ui:migrate-v1 could not auto-translate this
// handler` marker above the signature.

pub async fn dynamic_view(req: Request) -> Response {
    let view = if some_condition() {
        JsonUiView::new(
            "A",
            vec![make_node(
                "title",
                Component::PageHeader(PageHeaderProps {
                    title: "Variant A".to_string(),
                    ..Default::default()
                }),
            )],
        )
    } else {
        JsonUiView::new(
            "B",
            vec![make_node(
                "title",
                Component::PageHeader(PageHeaderProps {
                    title: "Variant B".to_string(),
                    ..Default::default()
                }),
            )],
        )
    };
    JsonUi::render(&view, &json!({}))
}
