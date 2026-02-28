//! API documentation routes

use ferro::*;

pub fn docs_routes() -> GroupDef {
    group!("/api", {
        get!("/docs", api_docs).name("api.docs"),
        get!("/openapi.json", openapi_json).name("api.openapi"),
    })
}

#[handler]
pub async fn api_docs() -> Response {
    let config = OpenApiConfig {
        title: ferro::env("APP_NAME", "API".to_string()),
        version: "1.0.0".to_string(),
        description: Some("Auto-generated API documentation".to_string()),
        api_prefix: "/api/".to_string(),
    };
    let routes = get_registered_routes();
    let resp = openapi_docs_response(&config, &routes);
    Ok(resp)
}

#[handler]
pub async fn openapi_json() -> Response {
    let config = OpenApiConfig {
        title: ferro::env("APP_NAME", "API".to_string()),
        version: "1.0.0".to_string(),
        description: Some("Auto-generated API documentation".to_string()),
        api_prefix: "/api/".to_string(),
    };
    let routes = get_registered_routes();
    let resp = openapi_json_response(&config, &routes);
    Ok(resp)
}
