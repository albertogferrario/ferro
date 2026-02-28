use serde_json::{json, Value};

use crate::types::ApiParam;

/// Adds a "description" key to a JSON Schema object.
pub fn inject_description(schema: &mut Value, description: &str) {
    if let Value::Object(map) = schema {
        map.insert(
            "description".to_string(),
            Value::String(description.to_string()),
        );
    }
}

/// Converts a list of `ApiParam`s and an optional request body schema into a
/// JSON Schema suitable for MCP tool `input_schema`.
///
/// Path/query/header parameters become top-level properties. Required parameters
/// (typically path params) are listed in the "required" array. Request body
/// content is nested under a "body" key to avoid name collisions with
/// path/query parameters.
pub fn build_input_schema(params: &[ApiParam], request_body_schema: Option<&Value>) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required: Vec<Value> = Vec::new();

    for param in params {
        let mut schema = param.schema.clone();
        if let Some(desc) = &param.description {
            inject_description(&mut schema, desc);
        }
        properties.insert(param.name.clone(), schema);
        if param.required {
            required.push(Value::String(param.name.clone()));
        }
    }

    if let Some(body_schema) = request_body_schema {
        let mut body_prop = body_schema.clone();
        inject_description(&mut body_prop, "Request body (JSON)");
        properties.insert("body".to_string(), body_prop);
    }

    json!({
        "type": "object",
        "properties": properties,
        "required": required,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ParamLocation;

    #[test]
    fn build_input_schema_path_params_only() {
        let params = vec![ApiParam {
            name: "id".to_string(),
            location: ParamLocation::Path,
            required: true,
            schema: json!({"type": "string"}),
            description: None,
        }];

        let schema = build_input_schema(&params, None);

        assert_eq!(schema["type"], "object");
        assert_eq!(schema["properties"]["id"]["type"], "string");
        assert_eq!(schema["required"], json!(["id"]));
    }

    #[test]
    fn build_input_schema_query_params() {
        let params = vec![ApiParam {
            name: "page".to_string(),
            location: ParamLocation::Query,
            required: false,
            schema: json!({"type": "integer"}),
            description: Some("Page number".to_string()),
        }];

        let schema = build_input_schema(&params, None);

        assert_eq!(schema["properties"]["page"]["type"], "integer");
        assert_eq!(schema["properties"]["page"]["description"], "Page number");
        assert_eq!(schema["required"], json!([]));
    }

    #[test]
    fn build_input_schema_with_body() {
        let body_schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "email": {"type": "string"}
            },
            "required": ["name", "email"]
        });

        let schema = build_input_schema(&[], Some(&body_schema));

        assert!(schema["properties"]["body"].is_object());
        assert_eq!(
            schema["properties"]["body"]["description"],
            "Request body (JSON)"
        );
        assert_eq!(
            schema["properties"]["body"]["properties"]["name"]["type"],
            "string"
        );
    }

    #[test]
    fn build_input_schema_no_params() {
        let schema = build_input_schema(&[], None);

        assert_eq!(schema["type"], "object");
        assert_eq!(schema["properties"], json!({}));
        assert_eq!(schema["required"], json!([]));
    }

    #[test]
    fn build_input_schema_mixed_params_and_body() {
        let params = vec![
            ApiParam {
                name: "user_id".to_string(),
                location: ParamLocation::Path,
                required: true,
                schema: json!({"type": "integer"}),
                description: Some("User identifier".to_string()),
            },
            ApiParam {
                name: "page".to_string(),
                location: ParamLocation::Query,
                required: false,
                schema: json!({"type": "integer"}),
                description: None,
            },
        ];

        let body_schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            }
        });

        let schema = build_input_schema(&params, Some(&body_schema));

        // All three property types present
        assert!(schema["properties"]["user_id"].is_object());
        assert!(schema["properties"]["page"].is_object());
        assert!(schema["properties"]["body"].is_object());

        // Only path param is required
        assert_eq!(schema["required"], json!(["user_id"]));

        // Descriptions applied
        assert_eq!(
            schema["properties"]["user_id"]["description"],
            "User identifier"
        );
        assert_eq!(
            schema["properties"]["body"]["description"],
            "Request body (JSON)"
        );
    }

    #[test]
    fn inject_description_adds_to_object() {
        let mut schema = json!({"type": "string"});
        inject_description(&mut schema, "A test field");
        assert_eq!(schema["description"], "A test field");
    }

    #[test]
    fn inject_description_ignores_non_object() {
        let mut schema = json!("not an object");
        inject_description(&mut schema, "ignored");
        assert_eq!(schema, json!("not an object"));
    }
}
