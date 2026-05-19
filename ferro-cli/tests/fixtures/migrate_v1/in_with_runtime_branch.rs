// Fixture: runtime-branch handler. The codemod must refuse to translate
// this shape and emit a TODO marker above the signature; the body stays
// intact.

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
