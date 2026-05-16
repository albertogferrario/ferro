// Fixture: v1 controller using make_node + JsonUiView::new pattern.
// Migration target for ferro json-ui:migrate-v1 (Plan 163-07).

pub async fn login_form(req: Request) -> Response {
    JsonUi::render_file("src/views/in_auth/login_form.json", serde_json::json!({}))
}
