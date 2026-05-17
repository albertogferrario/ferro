// Fixture: v1 controller exercising all five HTTP verb actions.
// Regression fixture for V7-RUNTIME-FRICTION F2 / Phase 164 D-19.
// The codemod must emit uppercase HTTP method values (POST/GET/PUT/PATCH/DELETE).

pub async fn show(req: Request) -> Response {
    let view = JsonUiView::new(
        "Verb Test",
        vec![make_node(
            "root",
            Component::Form(FormProps {
                fields: vec![
                    make_node_with_action(
                        "btn-post",
                        Component::Button(ButtonProps {
                            label: "Create".to_string(),
                            ..Default::default()
                        }),
                        Action::post("items.store"),
                    ),
                    make_node_with_action(
                        "btn-get",
                        Component::Button(ButtonProps {
                            label: "Load".to_string(),
                            ..Default::default()
                        }),
                        Action::get("items.index"),
                    ),
                    make_node_with_action(
                        "btn-put",
                        Component::Button(ButtonProps {
                            label: "Replace".to_string(),
                            ..Default::default()
                        }),
                        Action::put("items.update"),
                    ),
                    make_node_with_action(
                        "btn-patch",
                        Component::Button(ButtonProps {
                            label: "Patch".to_string(),
                            ..Default::default()
                        }),
                        Action::patch("items.partial"),
                    ),
                    make_node_with_action(
                        "btn-delete",
                        Component::Button(ButtonProps {
                            label: "Drop".to_string(),
                            ..Default::default()
                        }),
                        Action::delete("items.destroy"),
                    ),
                ],
                ..Default::default()
            }),
        )],
    );
    JsonUi::render(&view, &json!({}))
}
