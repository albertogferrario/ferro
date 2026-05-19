// Fixture: v1 controller using make_node + JsonUiView::new pattern.
// Migration target for ferro json-ui:migrate-v1 (Plan 163-07).

pub async fn login_form(req: Request) -> Response {
    let view = JsonUiView::new(
        "Login",
        vec![
            make_node(
                "page-title",
                Component::PageHeader(PageHeaderProps {
                    title: "Login".to_string(),
                    ..Default::default()
                }),
            ),
            make_node_with_action(
                "login-form",
                Component::Form(FormProps {
                    fields: vec![
                        make_node(
                            "email",
                            Component::Input(InputProps {
                                field: "email".to_string(),
                                label: Some("Email".to_string()),
                                input_type: InputType::Email,
                                ..Default::default()
                            }),
                        ),
                        make_node(
                            "password",
                            Component::Input(InputProps {
                                field: "password".to_string(),
                                label: Some("Password".to_string()),
                                input_type: InputType::Password,
                                ..Default::default()
                            }),
                        ),
                        make_node(
                            "submit",
                            Component::Button(ButtonProps {
                                label: "Sign in".to_string(),
                                button_type: ButtonType::Submit,
                                ..Default::default()
                            }),
                        ),
                    ],
                    ..Default::default()
                }),
                Action::post("auth.login"),
            ),
        ],
    )
    .layout("auth");
    JsonUi::render(&view, &json!({}))
}
