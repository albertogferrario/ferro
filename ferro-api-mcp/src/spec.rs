use crate::error::Error;
use crate::types::{ApiOperation, ApiParam, ParamLocation};

/// Fetch an OpenAPI spec from a URL.
///
/// Returns the raw JSON string for parsing with `parse_spec`.
pub async fn fetch_spec(url: &str) -> Result<String, Error> {
    todo!()
}

/// Parse an OpenAPI 3.0.x JSON spec into a list of API operations.
///
/// Validates the spec version (only 3.0.x supported), extracts all
/// operations from paths, resolves `$ref` references, and builds
/// `ApiOperation` for each (method, path, operation) tuple.
pub fn parse_spec(json: &str) -> Result<Vec<ApiOperation>, Error> {
    todo!()
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
