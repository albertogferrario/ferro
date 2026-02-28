use openapiv3::{OpenAPI, Operation, Parameter, ReferenceOr, RequestBody};
use serde_json::Value;

use crate::error::Error;
use crate::types::{ApiOperation, ApiParam, ParamLocation};

/// Fetch an OpenAPI spec from a URL.
///
/// Returns the raw JSON string for parsing with `parse_spec`.
pub async fn fetch_spec(url: &str) -> Result<String, Error> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| Error::SpecFetch(e.to_string()))?;

    response
        .text()
        .await
        .map_err(|e| Error::SpecFetch(e.to_string()))
}

/// Parse an OpenAPI 3.0.x JSON spec into a list of API operations.
///
/// Validates the spec version (only 3.0.x supported), extracts all
/// operations from paths, resolves `$ref` references, and builds
/// `ApiOperation` for each (method, path, operation) tuple.
pub fn parse_spec(json: &str) -> Result<Vec<ApiOperation>, Error> {
    let spec: OpenAPI = serde_json::from_str(json).map_err(|e| Error::SpecParse(e.to_string()))?;

    if !spec.openapi.starts_with("3.0") {
        return Err(Error::UnsupportedVersion(spec.openapi.clone()));
    }

    let mut operations = Vec::new();

    for (path, path_item_ref) in &spec.paths.paths {
        let path_item = match path_item_ref {
            ReferenceOr::Item(item) => item,
            ReferenceOr::Reference { .. } => continue,
        };

        let methods: &[(&str, &Option<Operation>)] = &[
            ("GET", &path_item.get),
            ("POST", &path_item.post),
            ("PUT", &path_item.put),
            ("PATCH", &path_item.patch),
            ("DELETE", &path_item.delete),
        ];

        for &(method, op_opt) in methods {
            if let Some(operation) = op_opt {
                let tool_name = generate_tool_name(operation.operation_id.as_deref(), method, path);
                let description = build_description(operation, &tool_name);
                let parameters =
                    extract_parameters(&spec, &operation.parameters, &path_item.parameters);
                let request_body_schema = extract_request_body(&spec, &operation.request_body);

                let mut op =
                    ApiOperation::new(tool_name, method.to_string(), path.clone(), description);
                op.parameters = parameters;
                op.request_body_schema = request_body_schema;

                operations.push(op);
            }
        }
    }

    Ok(operations)
}

/// Generate a tool name from an operation ID or method+path.
///
/// If `operation_id` is present, dots are replaced with underscores.
/// Otherwise, a name is generated from `{method}_{sanitized_path}`.
fn generate_tool_name(operation_id: Option<&str>, method: &str, path: &str) -> String {
    if let Some(id) = operation_id {
        return id.replace('.', "_");
    }

    let sanitized_path = path
        .split('/')
        .filter(|s| !s.is_empty() && !s.starts_with('{'))
        .collect::<Vec<_>>()
        .join("_");

    format!("{}_{}", method.to_lowercase(), sanitized_path)
}

/// Build a description from operation summary and description fields.
///
/// Priority: "summary - description" > "summary" > tool_name fallback.
fn build_description(operation: &Operation, tool_name: &str) -> String {
    match (&operation.summary, &operation.description) {
        (Some(summary), Some(desc)) => format!("{summary} - {desc}"),
        (Some(summary), None) => summary.clone(),
        (None, Some(desc)) => desc.clone(),
        (None, None) => tool_name.to_string(),
    }
}

/// Extract parameters from both operation-level and path-level parameter lists.
///
/// Resolves `$ref` references to `#/components/parameters/...`. Operation-level
/// parameters are processed first, then path-level parameters that don't
/// duplicate names already seen.
fn extract_parameters(
    spec: &OpenAPI,
    operation_params: &[ReferenceOr<Parameter>],
    path_params: &[ReferenceOr<Parameter>],
) -> Vec<ApiParam> {
    let mut result = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    // Operation-level parameters take precedence
    for param_ref in operation_params {
        if let Some(param) = resolve_parameter(spec, param_ref) {
            seen_names.insert(param.name.clone());
            result.push(param);
        }
    }

    // Path-level parameters (skip if already defined at operation level)
    for param_ref in path_params {
        if let Some(param) = resolve_parameter(spec, param_ref) {
            if seen_names.insert(param.name.clone()) {
                result.push(param);
            }
        }
    }

    result
}

/// Resolve a single parameter reference to an `ApiParam`.
fn resolve_parameter(spec: &OpenAPI, param_ref: &ReferenceOr<Parameter>) -> Option<ApiParam> {
    let param = match param_ref {
        ReferenceOr::Item(p) => p,
        ReferenceOr::Reference { reference } => resolve_parameter_ref(spec, reference)?,
    };

    let data = param.parameter_data_ref();
    let location = match param {
        Parameter::Path { .. } => ParamLocation::Path,
        Parameter::Query { .. } => ParamLocation::Query,
        Parameter::Header { .. } => ParamLocation::Header,
        Parameter::Cookie { .. } => return None, // Skip cookie params
    };

    let schema = extract_parameter_schema(data);

    Some(ApiParam {
        name: data.name.clone(),
        location,
        required: data.required,
        schema,
        description: data.description.clone(),
    })
}

/// Look up a parameter in `#/components/parameters/`.
fn resolve_parameter_ref<'a>(spec: &'a OpenAPI, reference: &str) -> Option<&'a Parameter> {
    let name = reference.strip_prefix("#/components/parameters/")?;
    let components = spec.components.as_ref()?;
    let param_ref = components.parameters.get(name)?;

    match param_ref {
        ReferenceOr::Item(p) => Some(p),
        ReferenceOr::Reference { .. } => {
            tracing::warn!("nested $ref in parameter {reference}, skipping");
            None
        }
    }
}

/// Extract the JSON Schema value from a parameter's schema-or-content.
fn extract_parameter_schema(data: &openapiv3::ParameterData) -> Value {
    match &data.format {
        openapiv3::ParameterSchemaOrContent::Schema(schema_ref) => match schema_ref {
            ReferenceOr::Item(schema) => {
                serde_json::to_value(schema).unwrap_or(Value::Object(Default::default()))
            }
            ReferenceOr::Reference { .. } => {
                // For parameter schemas that are $ref, serialize the schema type as-is
                Value::Object(Default::default())
            }
        },
        openapiv3::ParameterSchemaOrContent::Content(_) => Value::Object(Default::default()),
    }
}

/// Extract the request body JSON schema from an operation.
///
/// Looks for `application/json` content type and resolves `$ref` in the
/// schema. Returns `None` if no request body, no JSON content, or
/// unresolvable reference.
fn extract_request_body(spec: &OpenAPI, body: &Option<ReferenceOr<RequestBody>>) -> Option<Value> {
    let body_ref = body.as_ref()?;

    let request_body = match body_ref {
        ReferenceOr::Item(rb) => rb,
        ReferenceOr::Reference { reference } => {
            tracing::warn!("$ref for requestBody not yet supported: {reference}");
            return None;
        }
    };

    let media_type = request_body.content.get("application/json")?;
    let schema_ref = media_type.schema.as_ref()?;

    match schema_ref {
        ReferenceOr::Item(schema) => serde_json::to_value(schema).ok(),
        ReferenceOr::Reference { reference } => resolve_schema_ref(spec, reference),
    }
}

/// Resolve a `$ref` pointing to `#/components/schemas/...` into a JSON Value.
///
/// Looks up the schema name in `spec.components.schemas`, serializes it
/// via serde. Returns `None` and logs a warning for unresolvable refs.
fn resolve_schema_ref(spec: &OpenAPI, reference: &str) -> Option<Value> {
    let name = reference
        .strip_prefix("#/components/schemas/")
        .or_else(|| {
            tracing::warn!("unsupported $ref path: {reference}");
            None
        })?;

    let components = spec.components.as_ref().or_else(|| {
        tracing::warn!("$ref {reference} but no components section");
        None
    })?;

    let schema_ref = components.schemas.get(name).or_else(|| {
        tracing::warn!("unresolved $ref: {reference}");
        None
    })?;

    match schema_ref {
        ReferenceOr::Item(schema) => serde_json::to_value(schema).ok(),
        ReferenceOr::Reference { reference: nested } => {
            tracing::warn!("nested $ref in schema {reference} -> {nested}, skipping");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Helper: minimal valid OpenAPI 3.0.3 spec shell
    fn spec_shell(paths: serde_json::Value) -> String {
        json!({
            "openapi": "3.0.3",
            "info": { "title": "Test API", "version": "1.0.0" },
            "paths": paths
        })
        .to_string()
    }

    fn spec_shell_with_components(
        paths: serde_json::Value,
        components: serde_json::Value,
    ) -> String {
        json!({
            "openapi": "3.0.3",
            "info": { "title": "Test API", "version": "1.0.0" },
            "paths": paths,
            "components": components
        })
        .to_string()
    }

    // ── 1. Version validation ──────────────────────────────────────

    #[test]
    fn version_3_0_3_accepted() {
        let spec = spec_shell(json!({}));
        let result = parse_spec(&spec);
        assert!(result.is_ok(), "3.0.3 should be accepted");
    }

    #[test]
    fn version_3_0_0_accepted() {
        let spec = json!({
            "openapi": "3.0.0",
            "info": { "title": "Test", "version": "1.0.0" },
            "paths": {}
        })
        .to_string();
        let result = parse_spec(&spec);
        assert!(result.is_ok(), "3.0.0 should be accepted");
    }

    #[test]
    fn version_3_1_rejected() {
        let spec = json!({
            "openapi": "3.1.0",
            "info": { "title": "Test", "version": "1.0.0" },
            "paths": {}
        })
        .to_string();
        let result = parse_spec(&spec);
        assert!(result.is_err(), "3.1.0 should be rejected");
        let err = result.unwrap_err();
        assert!(
            matches!(err, Error::UnsupportedVersion(_)),
            "expected UnsupportedVersion, got: {err:?}"
        );
    }

    #[test]
    fn version_2_0_rejected() {
        // openapiv3 cannot parse Swagger 2.0 at all, so we expect a parse error
        let spec = json!({
            "swagger": "2.0",
            "info": { "title": "Test", "version": "1.0.0" },
            "paths": {}
        })
        .to_string();
        let result = parse_spec(&spec);
        assert!(result.is_err(), "2.0 should be rejected");
    }

    // ── 2. Operation extraction ────────────────────────────────────

    #[test]
    fn extracts_single_get_operation() {
        let spec = spec_shell(json!({
            "/api/users": {
                "get": {
                    "operationId": "api.users.index",
                    "summary": "List users",
                    "responses": { "200": { "description": "OK" } }
                }
            }
        }));
        let ops = parse_spec(&spec).unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].method, "GET");
        assert_eq!(ops[0].path, "/api/users");
    }

    #[test]
    fn extracts_multiple_operations() {
        let spec = spec_shell(json!({
            "/api/users": {
                "post": {
                    "operationId": "api.users.store",
                    "responses": { "201": { "description": "Created" } }
                }
            },
            "/api/users/{id}": {
                "delete": {
                    "operationId": "api.users.destroy",
                    "responses": { "204": { "description": "Deleted" } }
                }
            }
        }));
        let ops = parse_spec(&spec).unwrap();
        assert_eq!(ops.len(), 2);

        let methods: Vec<&str> = ops.iter().map(|o| o.method.as_str()).collect();
        assert!(methods.contains(&"POST"));
        assert!(methods.contains(&"DELETE"));
    }

    #[test]
    fn empty_paths_returns_empty_vec() {
        let spec = spec_shell(json!({}));
        let ops = parse_spec(&spec).unwrap();
        assert!(ops.is_empty());
    }

    // ── 3. Tool naming ─────────────────────────────────────────────

    #[test]
    fn tool_name_from_operation_id_dots_to_underscores() {
        let spec = spec_shell(json!({
            "/api/users": {
                "get": {
                    "operationId": "api.users.index",
                    "responses": { "200": { "description": "OK" } }
                }
            }
        }));
        let ops = parse_spec(&spec).unwrap();
        assert_eq!(ops[0].tool_name, "api_users_index");
    }

    #[test]
    fn tool_name_generated_when_no_operation_id() {
        let spec = spec_shell(json!({
            "/api/users": {
                "get": {
                    "responses": { "200": { "description": "OK" } }
                }
            }
        }));
        let ops = parse_spec(&spec).unwrap();
        assert_eq!(ops[0].tool_name, "get_api_users");
    }

    #[test]
    fn tool_name_mixed_with_and_without_operation_id() {
        let spec = spec_shell(json!({
            "/api/users": {
                "get": {
                    "operationId": "api.users.index",
                    "responses": { "200": { "description": "OK" } }
                }
            },
            "/api/posts": {
                "get": {
                    "responses": { "200": { "description": "OK" } }
                }
            }
        }));
        let ops = parse_spec(&spec).unwrap();
        assert_eq!(ops.len(), 2);

        let names: Vec<&str> = ops.iter().map(|o| o.tool_name.as_str()).collect();
        assert!(names.contains(&"api_users_index"));
        assert!(names.contains(&"get_api_posts"));
    }

    // ── 4. Parameter extraction ────────────────────────────────────

    #[test]
    fn extracts_path_parameter() {
        let spec = spec_shell(json!({
            "/api/users/{id}": {
                "get": {
                    "operationId": "api.users.show",
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "integer" }
                        }
                    ],
                    "responses": { "200": { "description": "OK" } }
                }
            }
        }));
        let ops = parse_spec(&spec).unwrap();
        assert_eq!(ops[0].parameters.len(), 1);
        assert_eq!(ops[0].parameters[0].name, "id");
        assert_eq!(ops[0].parameters[0].location, ParamLocation::Path);
        assert!(ops[0].parameters[0].required);
    }

    #[test]
    fn extracts_query_parameter() {
        let spec = spec_shell(json!({
            "/api/users": {
                "get": {
                    "operationId": "api.users.index",
                    "parameters": [
                        {
                            "name": "page",
                            "in": "query",
                            "required": false,
                            "schema": { "type": "integer" }
                        }
                    ],
                    "responses": { "200": { "description": "OK" } }
                }
            }
        }));
        let ops = parse_spec(&spec).unwrap();
        assert_eq!(ops[0].parameters.len(), 1);
        assert_eq!(ops[0].parameters[0].name, "page");
        assert_eq!(ops[0].parameters[0].location, ParamLocation::Query);
        assert!(!ops[0].parameters[0].required);
    }

    #[test]
    fn no_parameters_returns_empty_vec() {
        let spec = spec_shell(json!({
            "/api/health": {
                "get": {
                    "operationId": "health.check",
                    "responses": { "200": { "description": "OK" } }
                }
            }
        }));
        let ops = parse_spec(&spec).unwrap();
        assert!(ops[0].parameters.is_empty());
    }

    #[test]
    fn merges_path_level_and_operation_level_parameters() {
        let spec = spec_shell(json!({
            "/api/users/{id}": {
                "parameters": [
                    {
                        "name": "id",
                        "in": "path",
                        "required": true,
                        "schema": { "type": "integer" }
                    }
                ],
                "get": {
                    "operationId": "api.users.show",
                    "parameters": [
                        {
                            "name": "include",
                            "in": "query",
                            "required": false,
                            "schema": { "type": "string" }
                        }
                    ],
                    "responses": { "200": { "description": "OK" } }
                }
            }
        }));
        let ops = parse_spec(&spec).unwrap();
        assert_eq!(ops[0].parameters.len(), 2);

        let names: Vec<&str> = ops[0].parameters.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"id"));
        assert!(names.contains(&"include"));
    }

    // ── 5. Request body extraction ─────────────────────────────────

    #[test]
    fn extracts_json_request_body_schema() {
        let spec = spec_shell(json!({
            "/api/users": {
                "post": {
                    "operationId": "api.users.store",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "name": { "type": "string" },
                                        "email": { "type": "string" }
                                    },
                                    "required": ["name", "email"]
                                }
                            }
                        }
                    },
                    "responses": { "201": { "description": "Created" } }
                }
            }
        }));
        let ops = parse_spec(&spec).unwrap();
        let body = ops[0].request_body_schema.as_ref().unwrap();
        let props = body.get("properties").unwrap();
        assert!(props.get("name").is_some());
        assert!(props.get("email").is_some());
    }

    #[test]
    fn get_has_no_request_body() {
        let spec = spec_shell(json!({
            "/api/users": {
                "get": {
                    "operationId": "api.users.index",
                    "responses": { "200": { "description": "OK" } }
                }
            }
        }));
        let ops = parse_spec(&spec).unwrap();
        assert!(ops[0].request_body_schema.is_none());
    }

    // ── 6. $ref resolution ─────────────────────────────────────────

    #[test]
    fn resolves_request_body_schema_ref() {
        let spec = spec_shell_with_components(
            json!({
                "/api/users": {
                    "post": {
                        "operationId": "api.users.store",
                        "requestBody": {
                            "required": true,
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/CreateUserRequest"
                                    }
                                }
                            }
                        },
                        "responses": { "201": { "description": "Created" } }
                    }
                }
            }),
            json!({
                "schemas": {
                    "CreateUserRequest": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "email": { "type": "string" }
                        },
                        "required": ["name", "email"]
                    }
                }
            }),
        );
        let ops = parse_spec(&spec).unwrap();
        let body = ops[0].request_body_schema.as_ref().unwrap();
        let props = body.get("properties").unwrap();
        assert!(props.get("name").is_some());
        assert!(props.get("email").is_some());
    }

    #[test]
    fn unresolvable_ref_degrades_gracefully() {
        // An unresolvable $ref should NOT fail the entire parse.
        // The operation should still be extracted, with request_body_schema = None.
        let spec = spec_shell_with_components(
            json!({
                "/api/users": {
                    "post": {
                        "operationId": "api.users.store",
                        "requestBody": {
                            "required": true,
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/NonExistent"
                                    }
                                }
                            }
                        },
                        "responses": { "201": { "description": "Created" } }
                    }
                }
            }),
            json!({
                "schemas": {}
            }),
        );
        let ops = parse_spec(&spec).unwrap();
        assert_eq!(ops.len(), 1);
        // Body schema should be None since the ref couldn't be resolved
        assert!(ops[0].request_body_schema.is_none());
    }

    #[test]
    fn resolves_parameter_schema_ref() {
        let spec = spec_shell_with_components(
            json!({
                "/api/users": {
                    "get": {
                        "operationId": "api.users.index",
                        "parameters": [
                            {
                                "$ref": "#/components/parameters/PageParam"
                            }
                        ],
                        "responses": { "200": { "description": "OK" } }
                    }
                }
            }),
            json!({
                "parameters": {
                    "PageParam": {
                        "name": "page",
                        "in": "query",
                        "required": false,
                        "schema": { "type": "integer" }
                    }
                }
            }),
        );
        let ops = parse_spec(&spec).unwrap();
        assert_eq!(ops[0].parameters.len(), 1);
        assert_eq!(ops[0].parameters[0].name, "page");
        assert_eq!(ops[0].parameters[0].location, ParamLocation::Query);
    }

    // ── 7. Description extraction ──────────────────────────────────

    #[test]
    fn description_from_summary_and_description() {
        let spec = spec_shell(json!({
            "/api/users": {
                "get": {
                    "operationId": "api.users.index",
                    "summary": "List users",
                    "description": "Returns all users",
                    "responses": { "200": { "description": "OK" } }
                }
            }
        }));
        let ops = parse_spec(&spec).unwrap();
        assert_eq!(ops[0].description, "List users - Returns all users");
    }

    #[test]
    fn description_from_summary_only() {
        let spec = spec_shell(json!({
            "/api/users": {
                "get": {
                    "operationId": "api.users.index",
                    "summary": "List users",
                    "responses": { "200": { "description": "OK" } }
                }
            }
        }));
        let ops = parse_spec(&spec).unwrap();
        assert_eq!(ops[0].description, "List users");
    }

    #[test]
    fn description_fallback_to_tool_name() {
        let spec = spec_shell(json!({
            "/api/users": {
                "get": {
                    "operationId": "api.users.index",
                    "responses": { "200": { "description": "OK" } }
                }
            }
        }));
        let ops = parse_spec(&spec).unwrap();
        assert_eq!(ops[0].description, "api_users_index");
    }
}
