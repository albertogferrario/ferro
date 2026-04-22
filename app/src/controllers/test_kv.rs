use ferro::serde_json::json;
use ferro::{
    handler, Component, ComponentNode, JsonUi, JsonUiView, KeyValueEditorProps, Request, Response,
};

#[handler]
pub async fn show(_req: Request) -> Response {
    let view = JsonUiView::new()
        .title("KeyValueEditor Test")
        .component(ComponentNode {
            key: "kv-empty".to_string(),
            component: Component::KeyValueEditor(KeyValueEditorProps {
                field: "meta".to_string(),
                label: Some("Metadata (empty)".to_string()),
                suggested_keys: vec![],
                allow_custom_keys: true,
                data_path: None,
                error: None,
            }),
            action: None,
            visibility: None,
        })
        .component(ComponentNode {
            key: "kv-prefilled".to_string(),
            component: Component::KeyValueEditor(KeyValueEditorProps {
                field: "tags".to_string(),
                label: Some("Tags (prefilled)".to_string()),
                suggested_keys: vec!["env".to_string(), "region".to_string()],
                allow_custom_keys: false,
                data_path: Some("/tags".to_string()),
                error: None,
            }),
            action: None,
            visibility: None,
        })
        .component(ComponentNode {
            key: "kv-error".to_string(),
            component: Component::KeyValueEditor(KeyValueEditorProps {
                field: "config".to_string(),
                label: Some("Config (error state)".to_string()),
                suggested_keys: vec![],
                allow_custom_keys: true,
                data_path: None,
                error: Some("At least one entry is required".to_string()),
            }),
            action: None,
            visibility: None,
        });

    let data = json!({
        "tags": {
            "env": "production",
            "region": "us-east-1"
        }
    });

    JsonUi::render(&view, &data)
}
